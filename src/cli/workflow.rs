use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::cli::inspect;
use crate::cli::session::build_status_view;
use crate::cli::{CommandExitStatus, output};
use crate::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingOutcome, RoutingSource, SessionStatus,
    task_state_governance_blocked_reason, task_state_governance_state_text,
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

pub fn execute_list(
    workspace: Option<&Path>,
) -> Result<WorkflowCommandReport, WorkflowCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let workflow_path = workspace.join(".synod/workflows.toml");

    match WorkflowRegistry::load(&workflow_path) {
        Ok(registry) => Ok(render_workflow_discovery_report(
            &workspace,
            &registry.discovery_entries(&workspace),
        )),
        Err(error) => Ok(render_workflow_registry_error_report(&workspace, &error)),
    }
}

pub fn execute_status(
    workspace: Option<&Path>,
) -> Result<WorkflowCommandReport, WorkflowCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_workflow_session(&runtime)?;
    let workflow_name =
        record.active_workflow_name().expect("active workflow was checked before rendering status");
    let workflow = match load_workflow_definition(&workspace, &workflow_name) {
        Ok(workflow) => workflow,
        Err(error) => {
            return Ok(blocked_definition_report(&workflow_name, &workspace, error.to_string()));
        }
    };
    let refreshed = runtime
        .refresh_governance_state(&mut record)
        .map_err(WorkflowCommandError::SessionRuntime)?;
    if refreshed {
        runtime.persist_session(&record).map_err(WorkflowCommandError::SessionRuntime)?;
    }
    refresh_routing_summary(&mut record);

    Ok(render_workflow_report(
        &workspace,
        &record,
        if refreshed {
            format!(
                "refreshed workflow `{}` state from the persisted governed session",
                workflow.workflow_name
            )
        } else {
            format!("current persisted state for workflow `{}`", workflow.workflow_name)
        },
    ))
}

pub fn execute_resume(
    workspace: Option<&Path>,
) -> Result<WorkflowCommandReport, WorkflowCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_workflow_session(&runtime)?;
    let workflow_name =
        record.active_workflow_name().expect("active workflow was checked before resume");
    let workflow = match load_workflow_definition(&workspace, &workflow_name) {
        Ok(workflow) => workflow,
        Err(error) => {
            return Ok(blocked_definition_report(&workflow_name, &workspace, error.to_string()));
        }
    };

    let _ = runtime
        .refresh_governance_state(&mut record)
        .map_err(WorkflowCommandError::SessionRuntime)?;

    let explanation = advance_workflow(&runtime, &mut record, &workflow, None)?;
    runtime.persist_session(&record).map_err(WorkflowCommandError::SessionRuntime)?;

    Ok(render_workflow_report(&workspace, &record, explanation))
}

pub fn execute_inspect(
    workspace: Option<&Path>,
) -> Result<WorkflowCommandReport, WorkflowCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_workflow_session(&runtime)?;
    let workflow_name =
        record.active_workflow_name().expect("active workflow was checked before inspect");
    let workflow = match load_workflow_definition(&workspace, &workflow_name) {
        Ok(workflow) => workflow,
        Err(error) => {
            return Ok(blocked_definition_report(&workflow_name, &workspace, error.to_string()));
        }
    };
    refresh_routing_summary(&mut record);
    let workflow_report = render_workflow_report(
        &workspace,
        &record,
        format!("inspection summary for workflow `{}`", workflow.workflow_name),
    );

    if record.latest_trace_ref.is_none() {
        return Ok(workflow_report);
    }

    let inspect_report =
        inspect::execute_inspect(None, Some(&workspace)).map_err(WorkflowCommandError::Inspect)?;

    Ok(WorkflowCommandReport {
        exit_status: inspect_report.exit_status,
        terminal_output: format!(
            "{}\n{}",
            workflow_report.terminal_output, inspect_report.terminal_output
        ),
    })
}

#[derive(Debug, Error)]
pub enum WorkflowCommandError {
    #[error("workflow workspace could not be resolved: {0}")]
    WorkspaceResolution(String),
    #[error("workflow runtime error: {0}")]
    SessionRuntime(SessionRuntimeError),
    #[error("no active workflow session found for the current workspace")]
    MissingActiveWorkflowSession,
    #[error("the active session does not currently own a named workflow")]
    MissingActiveWorkflow,
    #[error("workflow definitions are invalid: {0}")]
    WorkflowDefinition(WorkflowDefinitionError),
    #[error("workflow inspect failed: {0}")]
    Inspect(inspect::InspectCommandError),
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
                        .plan_task(record, None, false)
                        .map_err(WorkflowCommandError::SessionRuntime)?;

                    if let Some(flow_name) = record
                        .goal_plan
                        .as_ref()
                        .and_then(|goal_plan| goal_plan.flow.as_ref())
                        .filter(|flow| !flow.confirmed)
                        .map(|flow| flow.flow_name.clone())
                    {
                        runtime
                            .plan_task(record, Some(&flow_name), false)
                            .map_err(WorkflowCommandError::SessionRuntime)?;
                    }
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

                let next_phase = next_pending_phase(workflow, &completed_phases);
                if matches!(next_phase, Some(WorkflowPhase::Review | WorkflowPhase::Govern)) {
                    continue;
                }

                if matches!(next_phase, Some(WorkflowPhase::Inspect))
                    && record.latest_status.is_terminal()
                {
                    continue;
                }

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
            WorkflowPhase::Review => {
                if latest_review_outcome(record).is_some() {
                    push_completed_phase(&mut completed_phases, WorkflowPhase::Review);
                    continue;
                }

                let terminal_failure = workflow_terminal_failure(record);
                let lifecycle_state = if terminal_failure.is_some() {
                    WorkflowLifecycleState::Failed
                } else if latest_review_trigger(record).is_some() {
                    WorkflowLifecycleState::Paused
                } else {
                    WorkflowLifecycleState::Blocked
                };
                let blocked_reason = terminal_failure.unwrap_or_else(|| {
                    if latest_review_trigger(record).is_some() {
                        "review outcome is still pending before workflow can continue".to_string()
                    } else {
                        "workflow review phase requires review evidence from the active session"
                            .to_string()
                    }
                });
                let next_action = if matches!(lifecycle_state, WorkflowLifecycleState::Paused) {
                    Some(workflow_resume_command(runtime.workspace_ref()))
                } else {
                    Some(workflow_inspect_command(runtime.workspace_ref()))
                };
                update_workflow_progress(
                    record,
                    workflow,
                    lifecycle_state,
                    Some(WorkflowPhase::Review),
                    completed_phases.clone(),
                    Some(blocked_reason.clone()),
                    next_action,
                );
                return Ok(format!(
                    "workflow `{}` stopped at review because {blocked_reason}",
                    workflow.workflow_name,
                ));
            }
            WorkflowPhase::Govern => {
                if should_resume_governed_execution(record) {
                    update_workflow_progress(
                        record,
                        workflow,
                        WorkflowLifecycleState::Active,
                        Some(WorkflowPhase::Govern),
                        completed_phases.clone(),
                        None,
                        Some(workflow_resume_command(runtime.workspace_ref())),
                    );
                    runtime
                        .run_to_terminal(record)
                        .map_err(WorkflowCommandError::SessionRuntime)?;
                }

                if govern_phase_completed(record) {
                    push_completed_phase(&mut completed_phases, WorkflowPhase::Govern);
                    continue;
                }

                let terminal_failure = workflow_terminal_failure(record);
                let lifecycle_state = if terminal_failure.is_some() {
                    WorkflowLifecycleState::Failed
                } else if matches!(
                    latest_governance_state(record).as_deref(),
                    Some("awaiting_approval")
                ) {
                    WorkflowLifecycleState::Paused
                } else {
                    WorkflowLifecycleState::Blocked
                };
                let blocked_reason = terminal_failure.unwrap_or_else(|| {
                    match latest_governance_state(record).as_deref() {
                        Some("awaiting_approval") => {
                            "governance approval is still required before workflow progression can continue".to_string()
                        }
                        Some("blocked") => latest_governance_blocked_reason(record).unwrap_or_else(|| {
                            "governance cannot continue until the required approval state is resolved".to_string()
                        }),
                        Some("failed") => latest_governance_blocked_reason(record).unwrap_or_else(|| {
                            "governance failed before workflow progression could continue".to_string()
                        }),
                        _ => "workflow govern phase requires governance evidence from the active session"
                            .to_string(),
                    }
                });
                let next_action = if matches!(lifecycle_state, WorkflowLifecycleState::Paused) {
                    Some(workflow_resume_command(runtime.workspace_ref()))
                } else {
                    Some(workflow_inspect_command(runtime.workspace_ref()))
                };
                update_workflow_progress(
                    record,
                    workflow,
                    lifecycle_state,
                    Some(WorkflowPhase::Govern),
                    completed_phases.clone(),
                    Some(blocked_reason.clone()),
                    next_action,
                );
                return Ok(format!(
                    "workflow `{}` stopped at govern because {blocked_reason}",
                    workflow.workflow_name,
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
    let next_command = workflow_next_command(workspace, record);
    let view = build_status_view(record, next_command, explanation);

    WorkflowCommandReport {
        exit_status: workflow_exit_status(record),
        terminal_output: output::render_session_status(&view),
    }
}

fn render_workflow_discovery_report(
    workspace: &Path,
    entries: &[crate::domain::workflow::WorkflowDiscoveryEntry],
) -> WorkflowCommandReport {
    let mut lines = vec![
        "workflow registry status: ready".to_string(),
        format!("workflow_count: {}", entries.len()),
    ];

    for entry in entries {
        lines.push(format!("workflow: {}", entry.workflow_name));
        lines.push(format!("summary: {}", entry.summary));
        lines.push(format!(
            "phases: {}",
            entry.phases.iter().map(|phase| phase.as_str()).collect::<Vec<_>>().join(" -> ")
        ));
        if let Some(recommended_when) = entry.recommended_when.as_deref() {
            lines.push(format!("recommended_when: {recommended_when}"));
        }
        lines.push(format!("invoke_with: {}", entry.invocation_command));
    }

    lines.push(format!(
        "explanation: discovered {} workflow definition(s) in workspace {}",
        entries.len(),
        workspace.display()
    ));

    WorkflowCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: lines.join("\n"),
    }
}

fn render_workflow_registry_error_report(
    workspace: &Path,
    error: &WorkflowDefinitionError,
) -> WorkflowCommandReport {
    let status = match error {
        WorkflowDefinitionError::ReadWorkflowDefinitions(io_error)
            if io_error.kind() == std::io::ErrorKind::NotFound =>
        {
            "missing"
        }
        _ => "invalid",
    };

    WorkflowCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: [
            format!("workflow registry status: {status}"),
            format!("reason: {error}"),
            format!("next_command: {}", workflow_inspect_command(workspace)),
            format!(
                "explanation: named workflow discovery is unavailable in workspace {} until .synod/workflows.toml is valid",
                workspace.display()
            ),
        ]
        .join("\n"),
    }
}

fn workflow_next_command(workspace: &Path, record: &ActiveSessionRecord) -> Option<String> {
    if let Some(progress) = record.active_workflow_progress()
        && matches!(progress.current_phase, Some(WorkflowPhase::Capture))
    {
        if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            return Some(capture_command(workspace));
        }

        return Some(workflow_resume_command(workspace));
    }

    if let Some(progress) = record.active_workflow_progress()
        && matches!(progress.current_phase, Some(WorkflowPhase::Clarify))
        && record.authored_brief.as_ref().and_then(|bundle| bundle.clarification.as_ref()).is_some()
    {
        return Some(capture_command(workspace));
    }

    record.active_workflow_next_action().or_else(|| default_next_command(workspace, record))
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
    if let Some(progress) = record.active_workflow_progress() {
        match progress.lifecycle_state {
            WorkflowLifecycleState::Blocked | WorkflowLifecycleState::Failed => {
                return CommandExitStatus::NonSuccess;
            }
            WorkflowLifecycleState::Idle
            | WorkflowLifecycleState::Active
            | WorkflowLifecycleState::Paused
            | WorkflowLifecycleState::Completed => {}
        }
    }

    match record.latest_status {
        SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted => {
            CommandExitStatus::NonSuccess
        }
        _ => CommandExitStatus::Succeeded,
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
        negotiation_packet: None,
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

fn load_active_workflow_session(
    runtime: &SessionRuntime,
) -> Result<ActiveSessionRecord, WorkflowCommandError> {
    let Some(record) = runtime.load_session().map_err(WorkflowCommandError::SessionRuntime)? else {
        return Err(WorkflowCommandError::MissingActiveWorkflowSession);
    };

    if record.active_workflow_name().is_none() {
        return Err(WorkflowCommandError::MissingActiveWorkflow);
    }

    Ok(record)
}

fn load_workflow_definition(
    workspace: &Path,
    workflow_name: &str,
) -> Result<WorkflowDefinition, WorkflowDefinitionError> {
    let registry = WorkflowRegistry::load(&workspace.join(".synod/workflows.toml"))?;
    registry.workflow(workflow_name).cloned().ok_or_else(|| {
        WorkflowDefinitionError::MissingNamedWorkflow { workflow_name: workflow_name.to_string() }
    })
}

fn refresh_routing_summary(record: &mut ActiveSessionRecord) {
    let routing_summary =
        Some(output::render_route_outcome(&crate::domain::session::routing_outcome(record)));
    if let Some(progress) = record.workflow_progress.as_mut() {
        progress.routing_summary = routing_summary.clone();
    }
    if let Some(goal_plan) = record.goal_plan.as_mut()
        && let Some(workflow_progress) = goal_plan.workflow_progress.as_mut()
    {
        workflow_progress.routing_summary = routing_summary;
    }
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

fn next_pending_phase(
    workflow: &WorkflowDefinition,
    completed_phases: &[WorkflowPhase],
) -> Option<WorkflowPhase> {
    workflow.phases.iter().find(|candidate| !completed_phases.contains(candidate)).copied()
}

fn latest_review_trigger(record: &ActiveSessionRecord) -> Option<&str> {
    record
        .active_task
        .as_ref()
        .and_then(|task| task.context.state.get("latest_review_trigger"))
        .and_then(serde_json::Value::as_str)
}

fn latest_review_outcome(record: &ActiveSessionRecord) -> Option<&str> {
    record
        .active_task
        .as_ref()
        .and_then(|task| task.context.state.get("latest_review_outcome"))
        .and_then(serde_json::Value::as_str)
}

fn latest_governance_state(record: &ActiveSessionRecord) -> Option<String> {
    record.active_task.as_ref().and_then(task_state_governance_state_text)
}

fn latest_governance_blocked_reason(record: &ActiveSessionRecord) -> Option<String> {
    record.active_task.as_ref().and_then(task_state_governance_blocked_reason)
}

fn workflow_terminal_failure(record: &ActiveSessionRecord) -> Option<String> {
    matches!(
        record.latest_status,
        SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
            | SessionStatus::Invalid
    )
    .then(|| {
        terminal_reason(record).unwrap_or_else(|| {
            "the underlying session ended with a non-success outcome".to_string()
        })
    })
}

fn should_resume_governed_execution(record: &ActiveSessionRecord) -> bool {
    record.active_task.is_some()
        && !record.latest_status.is_terminal()
        && governance_state_in(
            record,
            &["pending_selection", "running", "governed_ready", "completed"],
        )
}

fn govern_phase_completed(record: &ActiveSessionRecord) -> bool {
    record.latest_status.is_terminal()
        && governance_state_in(record, &["governed_ready", "completed"])
}

fn governance_state_in(record: &ActiveSessionRecord, expected_states: &[&str]) -> bool {
    latest_governance_state(record).as_deref().is_some_and(|state| expected_states.contains(&state))
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
