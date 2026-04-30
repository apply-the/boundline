use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::cli::session::build_status_view;
use crate::cli::{CommandExitStatus, output};
use crate::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingOutcome, RoutingSource, SessionStatus,
};
use crate::domain::trace::current_timestamp_millis;
use crate::domain::workflow::{
    WorkflowConditionKind, WorkflowDefinition, WorkflowDefinitionError, WorkflowLifecycleState,
    WorkflowPhase, WorkflowProgressState, WorkflowRegistry,
};
use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_run(
    workspace: Option<&Path>,
    name: &str,
    goal: Option<&str>,
) -> Result<WorkflowCommandReport, WorkflowCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let workflow_path = workspace.join(".synod/workflows.toml");
    let registry = match WorkflowRegistry::load(&workflow_path) {
        Ok(registry) => registry,
        Err(error) => {
            return Ok(blocked_definition_report(name, &workspace, error.to_string()));
        }
    };

    let Some(workflow) = registry.workflow(name).cloned() else {
        return Ok(blocked_definition_report(
            name,
            &workspace,
            format!("workflow `{name}` is not defined in .synod/workflows.toml"),
        ));
    };

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = runtime
        .load_session()
        .map_err(WorkflowCommandError::SessionRuntime)?
        .unwrap_or_else(|| initialize_session(&workspace));

    if let Some(active_workflow_name) = record.active_workflow_name()
        && active_workflow_name != workflow.workflow_name
    {
        return Ok(blocked_runtime_report(
            &workflow.workflow_name,
            &workspace,
            format!(
                "active workflow `{active_workflow_name}` must be completed or cleared before `{}` can start",
                workflow.workflow_name
            ),
            workflow_status_command(&workspace),
        ));
    }

    let explanation = advance_workflow(&runtime, &mut record, &workflow, goal)?;
    runtime.persist_session(&record).map_err(WorkflowCommandError::SessionRuntime)?;

    Ok(render_workflow_report(&workspace, &record, explanation))
}

#[derive(Debug, Error)]
pub enum WorkflowCommandError {
    #[error("workflow workspace could not be resolved: {0}")]
    WorkspaceResolution(String),
    #[error("workflow runtime error: {0}")]
    SessionRuntime(SessionRuntimeError),
    #[error("workflow definitions are invalid: {0}")]
    WorkflowDefinition(WorkflowDefinitionError),
}

fn advance_workflow(
    runtime: &SessionRuntime,
    record: &mut ActiveSessionRecord,
    workflow: &WorkflowDefinition,
    goal: Option<&str>,
) -> Result<String, WorkflowCommandError> {
    let mut completed_phases = record
        .active_workflow_progress()
        .map(|progress| progress.completed_phases.clone())
        .unwrap_or_default();

    update_workflow_progress(
        record,
        workflow,
        WorkflowLifecycleState::Active,
        Some(workflow.entry_phase),
        completed_phases.clone(),
        None,
        None,
    );

    for phase in &workflow.phases {
        if completed_phases.contains(phase) {
            continue;
        }

        if should_skip_phase(workflow, *phase, record) {
            push_completed_phase(&mut completed_phases, *phase);
            continue;
        }

        match phase {
            WorkflowPhase::Capture => {
                let captured_goal = record.goal.as_deref().map(str::trim).unwrap_or_default();
                if captured_goal.is_empty()
                    && let Some(goal_text) = goal.map(str::trim).filter(|value| !value.is_empty())
                {
                    runtime
                        .capture_goal(record, goal_text)
                        .map_err(WorkflowCommandError::SessionRuntime)?;
                }

                if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
                    update_workflow_progress(
                        record,
                        workflow,
                        WorkflowLifecycleState::Paused,
                        Some(WorkflowPhase::Capture),
                        completed_phases.clone(),
                        Some(
                            "workflow is waiting for a captured goal before it can continue"
                                .to_string(),
                        ),
                        Some(capture_command(runtime.workspace_ref())),
                    );
                    return Ok(format!(
                        "started workflow `{}` and paused at the capture phase because no goal was available",
                        workflow.workflow_name
                    ));
                }

                push_completed_phase(&mut completed_phases, WorkflowPhase::Capture);
            }
            WorkflowPhase::Clarify => {
                let clarification_required = record
                    .authored_brief
                    .as_ref()
                    .and_then(|bundle| bundle.clarification.as_ref())
                    .is_some();
                if clarification_required {
                    update_workflow_progress(
                        record,
                        workflow,
                        WorkflowLifecycleState::Paused,
                        Some(WorkflowPhase::Clarify),
                        completed_phases.clone(),
                        Some(
                            "clarification is still required before workflow planning can continue"
                                .to_string(),
                        ),
                        Some(capture_command(runtime.workspace_ref())),
                    );
                    return Ok(format!(
                        "workflow `{}` paused at the clarification phase because the current input is still underspecified",
                        workflow.workflow_name
                    ));
                }

                push_completed_phase(&mut completed_phases, WorkflowPhase::Clarify);
            }
            WorkflowPhase::Plan => {
                if record.goal_plan.is_none() {
                    runtime
                        .plan_task(record, None, true)
                        .map_err(WorkflowCommandError::SessionRuntime)?;
                }

                push_completed_phase(&mut completed_phases, WorkflowPhase::Plan);
            }
            WorkflowPhase::Run => {
                if !(record.latest_status.is_terminal() && record.active_task.is_none()) {
                    update_workflow_progress(
                        record,
                        workflow,
                        WorkflowLifecycleState::Active,
                        Some(WorkflowPhase::Run),
                        completed_phases.clone(),
                        None,
                        Some(workflow_resume_command(runtime.workspace_ref())),
                    );
                    runtime
                        .run_to_terminal(record)
                        .map_err(WorkflowCommandError::SessionRuntime)?;
                }

                push_completed_phase(&mut completed_phases, WorkflowPhase::Run);

                if record.latest_status.is_terminal() {
                    let next_phase = workflow
                        .phases
                        .iter()
                        .find(|candidate| !completed_phases.contains(candidate))
                        .copied()
                        .or(Some(WorkflowPhase::Run));
                    let lifecycle_state = terminal_lifecycle(record.latest_status);
                    let next_action = match next_phase {
                        Some(WorkflowPhase::Inspect) => {
                            Some(workflow_inspect_command(runtime.workspace_ref()))
                        }
                        _ => Some(workflow_resume_command(runtime.workspace_ref())),
                    };

                    update_workflow_progress(
                        record,
                        workflow,
                        lifecycle_state,
                        next_phase,
                        completed_phases.clone(),
                        terminal_reason(record),
                        next_action,
                    );

                    return Ok(match lifecycle_state {
                        WorkflowLifecycleState::Completed => format!(
                            "ran workflow `{}` through the session-native route",
                            workflow.workflow_name
                        ),
                        WorkflowLifecycleState::Failed => format!(
                            "workflow `{}` reached a terminal non-success session outcome",
                            workflow.workflow_name
                        ),
                        _ => format!(
                            "workflow `{}` updated the active session state",
                            workflow.workflow_name
                        ),
                    });
                }

                update_workflow_progress(
                    record,
                    workflow,
                    WorkflowLifecycleState::Active,
                    Some(WorkflowPhase::Run),
                    completed_phases.clone(),
                    None,
                    Some(workflow_resume_command(runtime.workspace_ref())),
                );

                return Ok(format!(
                    "workflow `{}` is executing through the session-native route",
                    workflow.workflow_name
                ));
            }
            WorkflowPhase::Review | WorkflowPhase::Govern => {
                update_workflow_progress(
                    record,
                    workflow,
                    WorkflowLifecycleState::Blocked,
                    Some(*phase),
                    completed_phases.clone(),
                    Some(format!(
                        "workflow phase `{}` is not yet executable from the workflow command surface",
                        phase.as_str()
                    )),
                    Some(workflow_inspect_command(runtime.workspace_ref())),
                );
                return Ok(format!(
                    "workflow `{}` is blocked because phase `{}` cannot execute yet",
                    workflow.workflow_name,
                    phase.as_str()
                ));
            }
            WorkflowPhase::Inspect => {
                update_workflow_progress(
                    record,
                    workflow,
                    terminal_lifecycle(record.latest_status),
                    Some(WorkflowPhase::Inspect),
                    completed_phases.clone(),
                    terminal_reason(record),
                    Some(workflow_inspect_command(runtime.workspace_ref())),
                );
                return Ok(format!(
                    "workflow `{}` reached its inspect phase with the current session evidence",
                    workflow.workflow_name
                ));
            }
        }
    }

    update_workflow_progress(
        record,
        workflow,
        terminal_lifecycle(record.latest_status),
        workflow.phases.last().copied(),
        completed_phases,
        terminal_reason(record),
        Some(workflow_inspect_command(runtime.workspace_ref())),
    );

    Ok(format!("workflow `{}` updated the active session state", workflow.workflow_name))
}

fn render_workflow_report(
    workspace: &Path,
    record: &ActiveSessionRecord,
    explanation: String,
) -> WorkflowCommandReport {
    let next_command =
        record.active_workflow_next_action().or_else(|| default_next_command(workspace, record));
    let view = build_status_view(record, next_command, explanation);

    WorkflowCommandReport {
        exit_status: workflow_exit_status(record),
        terminal_output: output::render_session_status(&view),
    }
}

fn default_next_command(workspace: &Path, record: &ActiveSessionRecord) -> Option<String> {
    if record.latest_status.is_terminal() {
        return Some(workflow_inspect_command(workspace));
    }

    if record.active_workflow_name().is_some() {
        return Some(workflow_resume_command(workspace));
    }

    None
}

fn workflow_exit_status(record: &ActiveSessionRecord) -> CommandExitStatus {
    match record.active_workflow_progress().map(|progress| progress.lifecycle_state) {
        Some(WorkflowLifecycleState::Blocked | WorkflowLifecycleState::Failed) => {
            CommandExitStatus::NonSuccess
        }
        _ => match record.latest_status {
            SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted => {
                CommandExitStatus::NonSuccess
            }
            _ => CommandExitStatus::Succeeded,
        },
    }
}

fn blocked_definition_report(
    workflow_name: &str,
    workspace: &Path,
    reason: String,
) -> WorkflowCommandReport {
    blocked_runtime_report(workflow_name, workspace, reason, workflow_inspect_command(workspace))
}

fn blocked_runtime_report(
    workflow_name: &str,
    workspace: &Path,
    reason: String,
    next_command: String,
) -> WorkflowCommandReport {
    let routing = RoutingOutcome {
        mode: RoutingMode::Blocked,
        source: RoutingSource::SessionState,
        reason: "workflow definition is not valid for session-native execution".to_string(),
    };

    WorkflowCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: [
            format!("workflow: {workflow_name}"),
            "workflow_phase: blocked".to_string(),
            output::render_route_outcome(&routing),
            format!("execution_condition: blocked - {reason}"),
            format!("next_command: {next_command}"),
            format!(
                "explanation: workflow `{workflow_name}` did not start in workspace {}",
                workspace.display()
            ),
        ]
        .join("\n"),
    }
}

fn initialize_session(workspace: &Path) -> ActiveSessionRecord {
    let now = current_timestamp_millis();
    ActiveSessionRecord {
        session_id: Uuid::new_v4().to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: now,
        updated_at: now,
    }
}

fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, WorkflowCommandError> {
    let workspace = match workspace {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir()
            .map_err(|error| WorkflowCommandError::WorkspaceResolution(error.to_string()))?,
    };

    workspace
        .canonicalize()
        .map_err(|error| WorkflowCommandError::WorkspaceResolution(error.to_string()))
}

fn should_skip_phase(
    workflow: &WorkflowDefinition,
    phase: WorkflowPhase,
    record: &ActiveSessionRecord,
) -> bool {
    let Some(condition_kind) = workflow
        .conditional_phases
        .iter()
        .find(|conditional_phase| conditional_phase.enabled && conditional_phase.phase == phase)
        .map(|conditional_phase| conditional_phase.condition_kind)
    else {
        return false;
    };

    !condition_is_met(record, condition_kind)
}

fn condition_is_met(record: &ActiveSessionRecord, condition_kind: WorkflowConditionKind) -> bool {
    match condition_kind {
        WorkflowConditionKind::MissingAuthoredInput => record
            .authored_brief
            .as_ref()
            .and_then(|bundle| bundle.clarification.as_ref())
            .is_some(),
        WorkflowConditionKind::ReviewTriggered => record
            .active_task
            .as_ref()
            .and_then(|task| task.context.state.get("latest_review_trigger"))
            .is_some(),
        WorkflowConditionKind::GovernanceRequired => {
            record
                .active_task
                .as_ref()
                .and_then(|task| task.context.state.get("latest_governance_stage"))
                .is_some()
                || record
                    .authored_brief
                    .as_ref()
                    .and_then(|bundle| bundle.governance_intent.as_ref())
                    .is_some()
        }
    }
}

fn update_workflow_progress(
    record: &mut ActiveSessionRecord,
    workflow: &WorkflowDefinition,
    lifecycle_state: WorkflowLifecycleState,
    current_phase: Option<WorkflowPhase>,
    completed_phases: Vec<WorkflowPhase>,
    blocked_reason: Option<String>,
    next_action: Option<String>,
) {
    let progress = WorkflowProgressState {
        workflow_name: workflow.workflow_name.clone(),
        lifecycle_state,
        current_phase,
        completed_phases,
        blocked_reason,
        next_action,
        routing_summary: Some(output::render_route_outcome(
            &crate::domain::session::routing_outcome(record),
        )),
    };

    record.workflow_progress = Some(progress.clone());
    if let Some(goal_plan) = record.goal_plan.as_mut() {
        goal_plan.workflow_progress = Some(progress);
    }
}

fn push_completed_phase(completed_phases: &mut Vec<WorkflowPhase>, phase: WorkflowPhase) {
    if !completed_phases.contains(&phase) {
        completed_phases.push(phase);
    }
}

fn terminal_lifecycle(status: SessionStatus) -> WorkflowLifecycleState {
    match status {
        SessionStatus::Succeeded => WorkflowLifecycleState::Completed,
        SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted
        | SessionStatus::Invalid => WorkflowLifecycleState::Failed,
        SessionStatus::Initialized
        | SessionStatus::GoalCaptured
        | SessionStatus::Planned
        | SessionStatus::Running => WorkflowLifecycleState::Active,
    }
}

fn terminal_reason(record: &ActiveSessionRecord) -> Option<String> {
    record.latest_terminal_reason.as_ref().map(|terminal_reason| terminal_reason.message.clone())
}

fn capture_command(workspace: &Path) -> String {
    format!("synod capture --workspace {} --goal <goal>", workspace.display())
}

fn workflow_resume_command(workspace: &Path) -> String {
    format!("synod workflow resume --workspace {}", workspace.display())
}

fn workflow_status_command(workspace: &Path) -> String {
    format!("synod workflow status --workspace {}", workspace.display())
}

fn workflow_inspect_command(workspace: &Path) -> String {
    format!("synod workflow inspect --workspace {}", workspace.display())
}
