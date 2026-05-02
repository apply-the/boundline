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
        "explanation: discovered {} workflow definition(s) in workspace {} for the primary Synod workflow surface",
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
            "route_owner: workflow".to_string(),
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        WorkflowCommandError, advance_workflow, blocked_runtime_report, capture_command,
        condition_is_met, default_next_command, execute_inspect, execute_list, execute_resume,
        execute_run, execute_status, govern_phase_completed, governance_state_in,
        initialize_session, latest_governance_blocked_reason, latest_governance_state,
        latest_review_outcome, latest_review_trigger, load_active_workflow_session,
        load_workflow_definition, next_pending_phase, push_completed_phase,
        refresh_routing_summary, render_workflow_registry_error_report, render_workflow_report,
        resolve_workspace, should_resume_governed_execution, should_skip_phase, terminal_lifecycle,
        terminal_reason, update_workflow_progress, workflow_exit_status, workflow_inspect_command,
        workflow_next_command, workflow_resume_command, workflow_status_command,
        workflow_terminal_failure,
    };
    use crate::cli::CommandExitStatus;
    use crate::domain::brief::{
        AuthoredBriefBundle, AuthoredBriefResolutionState, GovernanceIntent,
    };
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        ApprovalState, GovernanceLifecycleState, GovernanceRuntimeKind, GovernedStageRecord,
    };
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::Step;
    use crate::domain::task::{
        ClarificationReasonKind, ClarificationRecord, ClarificationStatus, Task, TaskRunRequest,
        TerminalReason,
    };
    use crate::domain::task_context::LATEST_GOVERNANCE_STAGE_KEY;
    use crate::domain::workflow::{
        ConditionalWorkflowPhase, WorkflowConditionKind, WorkflowDefinition,
        WorkflowDefinitionError, WorkflowGoalSource, WorkflowLifecycleState, WorkflowPhase,
        WorkflowProgressState,
    };
    use crate::orchestrator::session_runtime::SessionRuntime;

    struct TestWorkspace {
        path: PathBuf,
    }

    impl TestWorkspace {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("synod-workflow-tests-{}", Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write_registry(&self, contents: &str) {
            fs::create_dir_all(self.path.join(".synod")).unwrap();
            fs::write(self.path.join(".synod/workflows.toml"), contents).unwrap();
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn build_task(workspace_ref: &str) -> Task {
        let request = TaskRunRequest {
            goal: "Deliver workflow coverage".to_string(),
            input: json!({"ticket": "WF-1"}),
            session_id: "session-1".to_string(),
            workspace_ref: workspace_ref.to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::decision("analyze", json!({})).unwrap()]).unwrap();
        Task::new("task-1", &request, plan).unwrap()
    }

    fn build_goal_plan() -> GoalPlan {
        GoalPlan::new(
            "Deliver workflow coverage",
            vec![PlannedTask {
                task_id: "planned-1".to_string(),
                description: "Validate workflow reporting".to_string(),
                target: "src/cli/workflow.rs".to_string(),
                expected_outcome: None,
                decision_type_hint: None,
            }],
        )
        .unwrap()
    }

    fn build_record(workspace: &Path) -> ActiveSessionRecord {
        let mut record = initialize_session(workspace);
        record.goal = Some("Deliver workflow coverage".to_string());
        record.active_task = Some(build_task(&workspace.display().to_string()));
        record.latest_status = SessionStatus::Planned;
        record.goal_plan = Some(build_goal_plan());
        record
    }

    fn sample_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            workflow_name: "default".to_string(),
            goal_source: WorkflowGoalSource::Session,
            entry_phase: WorkflowPhase::Capture,
            phases: vec![
                WorkflowPhase::Capture,
                WorkflowPhase::Clarify,
                WorkflowPhase::Plan,
                WorkflowPhase::Run,
                WorkflowPhase::Review,
                WorkflowPhase::Govern,
                WorkflowPhase::Inspect,
            ],
            allow_review: true,
            allow_governance: true,
            conditional_phases: vec![
                ConditionalWorkflowPhase {
                    phase: WorkflowPhase::Clarify,
                    condition_kind: WorkflowConditionKind::MissingAuthoredInput,
                    enabled: true,
                },
                ConditionalWorkflowPhase {
                    phase: WorkflowPhase::Review,
                    condition_kind: WorkflowConditionKind::ReviewTriggered,
                    enabled: true,
                },
                ConditionalWorkflowPhase {
                    phase: WorkflowPhase::Govern,
                    condition_kind: WorkflowConditionKind::GovernanceRequired,
                    enabled: true,
                },
            ],
            output_preferences: Default::default(),
            summary: Some("Default workflow".to_string()),
            recommended_when: Some("bounded delivery needs follow-through".to_string()),
        }
    }

    fn workflow_with_phases(phases: Vec<WorkflowPhase>) -> WorkflowDefinition {
        let mut workflow = sample_workflow();
        workflow.entry_phase = *phases.first().unwrap();
        workflow.allow_review = phases.contains(&WorkflowPhase::Review);
        workflow.allow_governance = phases.contains(&WorkflowPhase::Govern);
        workflow.conditional_phases.retain(|phase| phases.contains(&phase.phase));
        workflow.phases = phases;
        workflow
    }

    fn clarification_bundle() -> AuthoredBriefBundle {
        AuthoredBriefBundle {
            bundle_id: "bundle-1".to_string(),
            primary_goal_text: Some("Deliver workflow coverage".to_string()),
            sources: Vec::new(),
            deduplicated_sources: Vec::new(),
            governance_intent: None,
            resolution_state: AuthoredBriefResolutionState::ClarificationRequired,
            clarification: Some(ClarificationRecord {
                clarification_id: "clarification-1".to_string(),
                reason_kind: ClarificationReasonKind::MissingContext,
                prompt: "Need more context".to_string(),
                missing_fields: vec!["scope".to_string()],
                blocking_sources: Vec::new(),
                turn_index: 1,
                status: ClarificationStatus::Open,
            }),
            derived_task_draft: None,
            captured_at: 1,
        }
    }

    fn governance_bundle() -> AuthoredBriefBundle {
        AuthoredBriefBundle {
            governance_intent: Some(GovernanceIntent {
                requested: true,
                runtime_preference: Some(GovernanceRuntimeKind::Local),
                risk: Some("medium".to_string()),
                zone: Some("delivery".to_string()),
                owner: Some("synod".to_string()),
            }),
            resolution_state: AuthoredBriefResolutionState::Ready,
            clarification: None,
            ..clarification_bundle()
        }
    }

    fn governed_stage_record(
        lifecycle_state: GovernanceLifecycleState,
        blocked_reason: Option<&str>,
    ) -> GovernedStageRecord {
        GovernedStageRecord {
            stage_key: "bug-fix:verify".to_string(),
            runtime: GovernanceRuntimeKind::Local,
            lifecycle_state,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::Requested,
            canon_run_ref: None,
            governance_attempt_id: "govern-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: None,
            blocked_reason: blocked_reason.map(str::to_string),
        }
    }

    #[test]
    fn execute_list_reports_registry_states_and_workflow_surface() {
        let workspace = TestWorkspace::new();

        let missing_report = execute_list(Some(workspace.path())).unwrap();
        assert_eq!(missing_report.exit_status, CommandExitStatus::NonSuccess);
        assert!(missing_report.terminal_output.contains("workflow registry status: missing"));
        assert!(missing_report.terminal_output.contains("named workflow discovery is unavailable"));

        workspace.write_registry(
            r#"[workflow.default]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "inspect"]
summary = "Default workflow"
recommended_when = "bounded delivery needs follow-through"
"#,
        );

        let ready_report = execute_list(Some(workspace.path())).unwrap();
        assert_eq!(ready_report.exit_status, CommandExitStatus::Succeeded);
        assert!(ready_report.terminal_output.contains("workflow registry status: ready"));
        assert!(ready_report.terminal_output.contains("workflow: default"));
        assert!(ready_report.terminal_output.contains("primary Synod workflow surface"));
    }

    #[test]
    fn load_active_workflow_session_requires_named_workflow_state() {
        let workspace = TestWorkspace::new();
        let runtime = SessionRuntime::for_workspace(workspace.path());

        assert!(matches!(
            load_active_workflow_session(&runtime),
            Err(WorkflowCommandError::MissingActiveWorkflowSession)
        ));

        let mut record = initialize_session(workspace.path());
        runtime.persist_session(&record).unwrap();
        assert!(matches!(
            load_active_workflow_session(&runtime),
            Err(WorkflowCommandError::MissingActiveWorkflow)
        ));

        record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: Some(workflow_resume_command(workspace.path())),
            routing_summary: None,
        });
        runtime.persist_session(&record).unwrap();

        let loaded = load_active_workflow_session(&runtime).unwrap();
        assert_eq!(loaded.active_workflow_name().as_deref(), Some("default"));
    }

    #[test]
    fn workflow_next_command_covers_paused_terminal_and_default_paths() {
        let workspace = TestWorkspace::new();
        let mut record = initialize_session(workspace.path());
        record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Paused,
            current_phase: Some(WorkflowPhase::Capture),
            completed_phases: Vec::new(),
            blocked_reason: Some("need goal".to_string()),
            next_action: None,
            routing_summary: None,
        });

        assert_eq!(
            workflow_next_command(workspace.path(), &record).as_deref(),
            Some(capture_command(workspace.path()).as_str())
        );

        record.goal = Some("Deliver workflow coverage".to_string());
        assert_eq!(
            workflow_next_command(workspace.path(), &record).as_deref(),
            Some(workflow_resume_command(workspace.path()).as_str())
        );

        record.authored_brief = Some(clarification_bundle());
        record.workflow_progress.as_mut().unwrap().current_phase = Some(WorkflowPhase::Clarify);
        assert_eq!(
            workflow_next_command(workspace.path(), &record).as_deref(),
            Some(capture_command(workspace.path()).as_str())
        );

        record.workflow_progress.as_mut().unwrap().current_phase = Some(WorkflowPhase::Run);
        record.workflow_progress.as_mut().unwrap().next_action = Some("custom next".to_string());
        assert_eq!(
            workflow_next_command(workspace.path(), &record).as_deref(),
            Some("custom next")
        );

        record.workflow_progress = None;
        record.latest_status = SessionStatus::Succeeded;
        assert_eq!(
            default_next_command(workspace.path(), &record).as_deref(),
            Some(workflow_inspect_command(workspace.path()).as_str())
        );

        record.latest_status = SessionStatus::Initialized;
        record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: None,
            routing_summary: None,
        });
        assert_eq!(
            default_next_command(workspace.path(), &record).as_deref(),
            Some(workflow_resume_command(workspace.path()).as_str())
        );

        record.workflow_progress = None;
        assert_eq!(default_next_command(workspace.path(), &record), None);
    }

    #[test]
    fn workflow_reports_cover_runtime_registry_and_status_rendering() {
        let workspace = TestWorkspace::new();
        let workflow = sample_workflow();
        let mut record = build_record(workspace.path());
        update_workflow_progress(
            &mut record,
            &workflow,
            WorkflowLifecycleState::Active,
            Some(WorkflowPhase::Run),
            vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            None,
            Some(workflow_resume_command(workspace.path())),
        );

        let report = render_workflow_report(
            workspace.path(),
            &record,
            "workflow `default` is executing through the session-native route".to_string(),
        );
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("route_owner: workflow"));
        assert!(report.terminal_output.contains("workflow_phase: run"));

        let blocked = blocked_runtime_report(
            "default",
            workspace.path(),
            "workflow definition is invalid".to_string(),
            workflow_status_command(workspace.path()),
        );
        assert_eq!(blocked.exit_status, CommandExitStatus::NonSuccess);
        assert!(blocked.terminal_output.contains("route_owner: workflow"));
        assert!(
            blocked
                .terminal_output
                .contains("execution_condition: blocked - workflow definition is invalid")
        );

        let missing_error = WorkflowDefinitionError::ReadWorkflowDefinitions(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing",
        ));
        let missing_report =
            render_workflow_registry_error_report(workspace.path(), &missing_error);
        assert!(missing_report.terminal_output.contains("workflow registry status: missing"));

        let invalid_report = render_workflow_registry_error_report(
            workspace.path(),
            &WorkflowDefinitionError::MissingWorkflowDefinitions,
        );
        assert!(invalid_report.terminal_output.contains("workflow registry status: invalid"));
    }

    #[test]
    fn workflow_execute_commands_cover_missing_definition_and_active_conflict_paths() {
        let workspace = TestWorkspace::new();
        workspace.write_registry(
            r#"[workflow.default]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "inspect"]
summary = "Default workflow"
"#,
        );

        let missing_run = execute_run(Some(workspace.path()), "missing", Some("goal")).unwrap();
        assert_eq!(missing_run.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            missing_run
                .terminal_output
                .contains("workflow `missing` is not defined in .synod/workflows.toml")
        );

        let runtime = SessionRuntime::for_workspace(workspace.path());
        let mut conflicting_record = initialize_session(workspace.path());
        conflicting_record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "other".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: Some(workflow_resume_command(workspace.path())),
            routing_summary: None,
        });
        runtime.persist_session(&conflicting_record).unwrap();

        let conflicting_run = execute_run(Some(workspace.path()), "default", Some("goal")).unwrap();
        assert_eq!(conflicting_run.exit_status, CommandExitStatus::NonSuccess);
        assert!(conflicting_run.terminal_output.contains(
            "active workflow `other` must be completed or cleared before `default` can start"
        ));

        let missing_registry_workspace = TestWorkspace::new();
        let missing_runtime = SessionRuntime::for_workspace(missing_registry_workspace.path());
        let mut named_record = initialize_session(missing_registry_workspace.path());
        named_record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: Some(workflow_resume_command(missing_registry_workspace.path())),
            routing_summary: None,
        });
        missing_runtime.persist_session(&named_record).unwrap();

        let status_report = execute_status(Some(missing_registry_workspace.path())).unwrap();
        assert!(
            status_report.terminal_output.contains("workflow `default` did not start in workspace")
        );

        let resume_report = execute_resume(Some(missing_registry_workspace.path())).unwrap();
        assert!(
            resume_report.terminal_output.contains("workflow `default` did not start in workspace")
        );

        let inspect_report = execute_inspect(Some(missing_registry_workspace.path())).unwrap();
        assert!(
            inspect_report
                .terminal_output
                .contains("workflow `default` did not start in workspace")
        );

        let inspect_workspace = TestWorkspace::new();
        inspect_workspace.write_registry(
            r#"[workflow.default]
goal_source = "session"
entry = "inspect"
phases = ["inspect"]
summary = "Inspect workflow"
"#,
        );
        let inspect_runtime = SessionRuntime::for_workspace(inspect_workspace.path());
        let mut inspect_record = initialize_session(inspect_workspace.path());
        inspect_record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Completed,
            current_phase: Some(WorkflowPhase::Inspect),
            completed_phases: vec![WorkflowPhase::Inspect],
            blocked_reason: None,
            next_action: Some(workflow_inspect_command(inspect_workspace.path())),
            routing_summary: None,
        });
        inspect_runtime.persist_session(&inspect_record).unwrap();

        let inspect_without_trace = execute_inspect(Some(inspect_workspace.path())).unwrap();
        assert_eq!(inspect_without_trace.exit_status, CommandExitStatus::Succeeded);
        assert!(
            inspect_without_trace
                .terminal_output
                .contains("inspection summary for workflow `default`")
        );
    }

    #[test]
    fn workflow_state_helpers_cover_conditions_reviews_governance_and_terminals() {
        let workspace = TestWorkspace::new();
        let workflow = sample_workflow();
        let mut record = build_record(workspace.path());

        record.authored_brief = Some(clarification_bundle());
        assert!(condition_is_met(&record, WorkflowConditionKind::MissingAuthoredInput));
        assert!(!should_skip_phase(&workflow, WorkflowPhase::Plan, &record));
        assert!(!should_skip_phase(&workflow, WorkflowPhase::Clarify, &record));

        let task = record.active_task.as_mut().unwrap();
        task.context.state.insert("latest_review_trigger".to_string(), json!("review_requested"));
        task.context.state.insert("latest_review_outcome".to_string(), json!("approved"));
        assert_eq!(latest_review_trigger(&record), Some("review_requested"));
        assert_eq!(latest_review_outcome(&record), Some("approved"));
        assert!(condition_is_met(&record, WorkflowConditionKind::ReviewTriggered));

        record.authored_brief = Some(governance_bundle());
        assert!(condition_is_met(&record, WorkflowConditionKind::GovernanceRequired));
        assert_eq!(
            next_pending_phase(&workflow, &[WorkflowPhase::Capture, WorkflowPhase::Clarify]),
            Some(WorkflowPhase::Plan)
        );

        let stage = governed_stage_record(
            GovernanceLifecycleState::Blocked,
            Some("approval is still pending"),
        );
        record
            .active_task
            .as_mut()
            .unwrap()
            .context
            .state
            .insert(LATEST_GOVERNANCE_STAGE_KEY.to_string(), serde_json::to_value(stage).unwrap());
        assert_eq!(latest_governance_state(&record).as_deref(), Some("blocked"));
        assert_eq!(
            latest_governance_blocked_reason(&record).as_deref(),
            Some("approval is still pending")
        );
        assert!(governance_state_in(&record, &["blocked"]));
        assert!(!should_resume_governed_execution(&record));

        let running_stage = governed_stage_record(GovernanceLifecycleState::Running, None);
        record.latest_status = SessionStatus::Running;
        record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(running_stage).unwrap(),
        );
        assert!(should_resume_governed_execution(&record));

        let ready_stage = governed_stage_record(GovernanceLifecycleState::GovernedReady, None);
        record.latest_status = SessionStatus::Succeeded;
        record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(ready_stage).unwrap(),
        );
        assert!(govern_phase_completed(&record));

        record.latest_status = SessionStatus::Failed;
        record.latest_terminal_reason = Some(TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "validation failed",
            None,
        ));
        assert_eq!(workflow_terminal_failure(&record).as_deref(), Some("validation failed"));

        record.latest_status = SessionStatus::Invalid;
        record.latest_terminal_reason = None;
        assert_eq!(
            workflow_terminal_failure(&record).as_deref(),
            Some("the underlying session ended with a non-success outcome")
        );
    }

    #[test]
    fn workflow_advance_covers_clarify_review_and_govern_blocked_paths() {
        let workspace = TestWorkspace::new();
        let runtime = SessionRuntime::for_workspace(workspace.path());

        let mut clarify_record = build_record(workspace.path());
        clarify_record.authored_brief = Some(clarification_bundle());
        clarify_record.goal_plan = Some(build_goal_plan());
        let clarify_workflow = workflow_with_phases(vec![WorkflowPhase::Clarify]);
        let clarify_message =
            advance_workflow(&runtime, &mut clarify_record, &clarify_workflow, None).unwrap();
        assert!(clarify_message.contains("paused at the clarification phase"));
        assert_eq!(
            clarify_record.workflow_progress.as_ref().and_then(|progress| progress.current_phase),
            Some(WorkflowPhase::Clarify)
        );

        let review_workflow = workflow_with_phases(vec![WorkflowPhase::Review]);
        let mut review_paused_record = build_record(workspace.path());
        review_paused_record
            .active_task
            .as_mut()
            .unwrap()
            .context
            .state
            .insert("latest_review_trigger".to_string(), json!("review_requested"));
        let review_paused_message =
            advance_workflow(&runtime, &mut review_paused_record, &review_workflow, None).unwrap();
        assert!(review_paused_message.contains("review outcome is still pending"));
        assert_eq!(
            review_paused_record
                .workflow_progress
                .as_ref()
                .map(|progress| progress.lifecycle_state),
            Some(WorkflowLifecycleState::Paused)
        );

        let mut review_failed_record = build_record(workspace.path());
        review_failed_record
            .active_task
            .as_mut()
            .unwrap()
            .context
            .state
            .insert("latest_review_trigger".to_string(), json!("review_requested"));
        review_failed_record.latest_status = SessionStatus::Failed;
        review_failed_record.latest_terminal_reason =
            Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "review failed", None));
        let review_failed_message =
            advance_workflow(&runtime, &mut review_failed_record, &review_workflow, None).unwrap();
        assert!(review_failed_message.contains("review failed"));
        assert_eq!(
            review_failed_record
                .workflow_progress
                .as_ref()
                .map(|progress| progress.lifecycle_state),
            Some(WorkflowLifecycleState::Failed)
        );

        let govern_workflow = workflow_with_phases(vec![WorkflowPhase::Govern]);
        let mut govern_blocked_record = build_record(workspace.path());
        govern_blocked_record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(governed_stage_record(
                GovernanceLifecycleState::Blocked,
                Some("approval is still pending"),
            ))
            .unwrap(),
        );
        let govern_blocked_message =
            advance_workflow(&runtime, &mut govern_blocked_record, &govern_workflow, None).unwrap();
        assert!(govern_blocked_message.contains("approval is still pending"));
        assert_eq!(
            govern_blocked_record
                .workflow_progress
                .as_ref()
                .map(|progress| progress.lifecycle_state),
            Some(WorkflowLifecycleState::Blocked)
        );

        let mut govern_failed_record = build_record(workspace.path());
        govern_failed_record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(governed_stage_record(
                GovernanceLifecycleState::Failed,
                Some("governance failed before workflow progression could continue"),
            ))
            .unwrap(),
        );
        let govern_failed_message =
            advance_workflow(&runtime, &mut govern_failed_record, &govern_workflow, None).unwrap();
        assert!(
            govern_failed_message
                .contains("governance failed before workflow progression could continue")
        );
    }

    #[test]
    fn workflow_advance_covers_terminal_run_and_final_summary_paths() {
        let workspace = TestWorkspace::new();
        let runtime = SessionRuntime::for_workspace(workspace.path());

        let run_workflow = workflow_with_phases(vec![WorkflowPhase::Run]);
        let mut completed_record = initialize_session(workspace.path());
        completed_record.latest_status = SessionStatus::Succeeded;
        completed_record.latest_terminal_reason =
            Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
        let completed_message =
            advance_workflow(&runtime, &mut completed_record, &run_workflow, None).unwrap();
        assert!(
            completed_message.contains("ran workflow `default` through the session-native route")
        );
        assert_eq!(
            completed_record.workflow_progress.as_ref().map(|progress| progress.lifecycle_state),
            Some(WorkflowLifecycleState::Completed)
        );

        let mut failed_record = initialize_session(workspace.path());
        failed_record.latest_status = SessionStatus::Failed;
        failed_record.latest_terminal_reason =
            Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "failed", None));
        let failed_message =
            advance_workflow(&runtime, &mut failed_record, &run_workflow, None).unwrap();
        assert!(failed_message.contains("terminal non-success session outcome"));
        assert_eq!(
            failed_record.workflow_progress.as_ref().map(|progress| progress.lifecycle_state),
            Some(WorkflowLifecycleState::Failed)
        );

        let final_workflow =
            workflow_with_phases(vec![WorkflowPhase::Capture, WorkflowPhase::Plan]);
        let mut final_record = initialize_session(workspace.path());
        final_record.goal = Some("Deliver workflow coverage".to_string());
        final_record.goal_plan = Some(build_goal_plan());
        let final_message =
            advance_workflow(&runtime, &mut final_record, &final_workflow, None).unwrap();
        assert!(final_message.contains("updated the active session state"));
        assert_eq!(
            final_record.workflow_progress.as_ref().and_then(|progress| progress.current_phase),
            Some(WorkflowPhase::Plan)
        );
    }

    #[test]
    fn workflow_helpers_cover_remaining_fallback_paths() {
        let workspace = TestWorkspace::new();
        workspace.write_registry(
            r#"[workflow.default]
goal_source = "session"
entry = "capture"
phases = ["capture", "plan", "run", "inspect"]
summary = "Default workflow"
"#,
        );

        let resolved = resolve_workspace(None).unwrap();
        assert!(resolved.is_absolute());

        let missing_path = workspace.path().join("missing-workspace");
        assert!(matches!(
            resolve_workspace(Some(&missing_path)),
            Err(WorkflowCommandError::WorkspaceResolution(_))
        ));

        let missing_definition = load_workflow_definition(workspace.path(), "missing").unwrap_err();
        assert!(matches!(missing_definition, WorkflowDefinitionError::MissingNamedWorkflow { .. }));

        let mut record = initialize_session(workspace.path());
        record.workflow_progress = Some(WorkflowProgressState {
            workflow_name: "default".to_string(),
            lifecycle_state: WorkflowLifecycleState::Active,
            current_phase: Some(WorkflowPhase::Run),
            completed_phases: vec![WorkflowPhase::Capture, WorkflowPhase::Plan],
            blocked_reason: None,
            next_action: None,
            routing_summary: None,
        });
        assert_eq!(
            workflow_next_command(workspace.path(), &record).as_deref(),
            Some(workflow_resume_command(workspace.path()).as_str())
        );

        let mut plain_record = initialize_session(workspace.path());
        refresh_routing_summary(&mut plain_record);
        assert_eq!(workflow_exit_status(&plain_record), CommandExitStatus::Succeeded);

        let runtime = SessionRuntime::for_workspace(workspace.path());

        let mut clarify_ready_record = build_record(workspace.path());
        clarify_ready_record.authored_brief = None;
        let mut clarify_ready_workflow = workflow_with_phases(vec![WorkflowPhase::Clarify]);
        clarify_ready_workflow.conditional_phases.clear();
        let clarify_ready_message =
            advance_workflow(&runtime, &mut clarify_ready_record, &clarify_ready_workflow, None)
                .unwrap();
        assert!(clarify_ready_message.contains("updated the active session state"));

        let mut review_blocked_record = build_record(workspace.path());
        let mut review_blocked_workflow = workflow_with_phases(vec![WorkflowPhase::Review]);
        review_blocked_workflow.conditional_phases.clear();
        let review_blocked_message =
            advance_workflow(&runtime, &mut review_blocked_record, &review_blocked_workflow, None)
                .unwrap();
        assert!(review_blocked_message.contains("requires review evidence"));

        let govern_workflow = workflow_with_phases(vec![WorkflowPhase::Govern]);

        let mut govern_blocked_default_record = build_record(workspace.path());
        govern_blocked_default_record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(governed_stage_record(GovernanceLifecycleState::Blocked, None))
                .unwrap(),
        );
        let govern_blocked_default_message =
            advance_workflow(&runtime, &mut govern_blocked_default_record, &govern_workflow, None)
                .unwrap();
        assert!(
            govern_blocked_default_message.contains(
                "governance cannot continue until the required approval state is resolved"
            )
        );

        let mut govern_failed_default_record = build_record(workspace.path());
        govern_failed_default_record.active_task.as_mut().unwrap().context.state.insert(
            LATEST_GOVERNANCE_STAGE_KEY.to_string(),
            serde_json::to_value(governed_stage_record(GovernanceLifecycleState::Failed, None))
                .unwrap(),
        );
        let govern_failed_default_message =
            advance_workflow(&runtime, &mut govern_failed_default_record, &govern_workflow, None)
                .unwrap();
        assert!(
            govern_failed_default_message
                .contains("governance failed before workflow progression could continue")
        );
    }

    #[test]
    fn workflow_progress_and_status_helpers_cover_updates_and_commands() {
        let workspace = TestWorkspace::new();
        let workflow = sample_workflow();
        let mut record = build_record(workspace.path());

        update_workflow_progress(
            &mut record,
            &workflow,
            WorkflowLifecycleState::Paused,
            Some(WorkflowPhase::Clarify),
            vec![WorkflowPhase::Capture],
            Some("need clarification".to_string()),
            Some(capture_command(workspace.path())),
        );
        refresh_routing_summary(&mut record);

        let progress = record.workflow_progress.as_ref().unwrap();
        assert_eq!(progress.current_phase, Some(WorkflowPhase::Clarify));
        assert_eq!(progress.blocked_reason.as_deref(), Some("need clarification"));
        assert!(progress.routing_summary.is_some());
        assert_eq!(
            record
                .goal_plan
                .as_ref()
                .and_then(|goal_plan| goal_plan.workflow_progress.as_ref())
                .and_then(|goal_plan_progress| goal_plan_progress.blocked_reason.as_deref()),
            Some("need clarification")
        );

        let mut completed = vec![WorkflowPhase::Capture];
        push_completed_phase(&mut completed, WorkflowPhase::Capture);
        push_completed_phase(&mut completed, WorkflowPhase::Plan);
        assert_eq!(completed, vec![WorkflowPhase::Capture, WorkflowPhase::Plan]);

        assert_eq!(workflow_exit_status(&record), CommandExitStatus::Succeeded);
        record.workflow_progress.as_mut().unwrap().lifecycle_state =
            WorkflowLifecycleState::Blocked;
        assert_eq!(workflow_exit_status(&record), CommandExitStatus::NonSuccess);

        record.workflow_progress.as_mut().unwrap().lifecycle_state = WorkflowLifecycleState::Active;
        record.latest_status = SessionStatus::Aborted;
        assert_eq!(workflow_exit_status(&record), CommandExitStatus::NonSuccess);

        assert_eq!(terminal_lifecycle(SessionStatus::Succeeded), WorkflowLifecycleState::Completed);
        assert_eq!(terminal_lifecycle(SessionStatus::Failed), WorkflowLifecycleState::Failed);
        assert_eq!(terminal_lifecycle(SessionStatus::Invalid), WorkflowLifecycleState::Failed);
        assert_eq!(terminal_lifecycle(SessionStatus::Running), WorkflowLifecycleState::Active);

        record.latest_terminal_reason =
            Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
        assert_eq!(terminal_reason(&record).as_deref(), Some("done"));

        assert!(capture_command(workspace.path()).contains("synod capture --workspace"));
        assert!(workflow_resume_command(workspace.path()).contains("synod workflow resume"));
        assert!(workflow_status_command(workspace.path()).contains("synod workflow status"));
        assert!(workflow_inspect_command(workspace.path()).contains("synod workflow inspect"));

        let initialized = initialize_session(workspace.path());
        assert_eq!(initialized.latest_status, SessionStatus::Initialized);
        assert_eq!(initialized.goal, None);
    }
}
