use std::path::{Path, PathBuf};

use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::domain::brief::{
    AuthoredBriefBundle, BriefIngestionError, normalize_governance_intent,
    normalize_inputs_with_governance,
};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::inspect::summarize_trace;
use crate::cli::output;
use crate::domain::cluster::ClusterSessionProjection;
use crate::domain::decision::ActionSelector;
use crate::domain::governance::GovernanceRuntimeKind;
use crate::domain::negotiation::NegotiatedDeliveryPacket;
use crate::domain::session::{
    ActiveSessionRecord, CompatibilityFollowUpMode, CompatibilityFollowUpView, ContinuityAuthority,
    SessionStatus, SessionStatusView, decision_status_text, delegation_next_command,
    delegation_status_view, execution_path_text, routing_outcome,
    task_state_attempt_lineage_summary, task_state_canon_memory_context_credibility,
    task_state_canon_memory_context_summary, task_state_canon_memory_primary_inputs,
    task_state_canon_memory_provenance, task_state_canon_memory_staleness_reason,
    task_state_governance_approval_text, task_state_governance_blocked_reason,
    task_state_governance_candidate_actions, task_state_governance_canon_run_ref,
    task_state_governance_decision_headline, task_state_governance_mode_text,
    task_state_governance_next_action, task_state_governance_packet_binding_reason,
    task_state_governance_packet_ref, task_state_governance_packet_source_stage,
    task_state_governance_runtime_text, task_state_governance_stage_key,
    task_state_governance_state_text, task_state_string, task_state_strings,
    task_state_workspace_slice_summary,
};
use crate::domain::task::TaskStatus;
use crate::domain::trace::current_timestamp_millis;
use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedSessionTarget {
    owner_workspace: PathBuf,
    cluster_projection: Option<ClusterSessionProjection>,
}

pub fn execute_start(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_start_with_target(workspace, None)
}

pub fn execute_start_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "start")?;
    let workspace = target.owner_workspace;
    let now = current_timestamp_millis();
    let record = ActiveSessionRecord {
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
    };

    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some("boundline capture --goal <goal>".to_string()),
            if target.cluster_projection.is_some() {
                "active clustered session initialized for the current primary workspace"
            } else {
                "active session initialized for the current workspace"
            },
        )),
    })
}

pub fn execute_capture(
    workspace: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_capture_with_target(workspace, None, goal, briefs, governance, risk, zone, owner)
}

#[allow(clippy::too_many_arguments)]
pub fn execute_capture_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "capture")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    let governance_intent = normalize_governance_intent(governance, risk, zone, owner)
        .map_err(SessionCommandError::BriefIngestion)?;
    let bundle = normalize_inputs_with_governance(&workspace, goal, briefs, governance_intent)
        .map_err(SessionCommandError::BriefIngestion)?;
    let effective_goal = bundle.render_goal_text();

    runtime.capture_goal(&mut record, &effective_goal).map_err(map_runtime_error)?;
    record.authored_brief = Some(bundle.clone());
    record.negotiation_packet = Some(NegotiatedDeliveryPacket::from_authored_brief(
        &record.session_id,
        &record.workspace_ref,
        &effective_goal,
        &bundle,
    ));
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let summary = if bundle.clarification.is_some() {
        "captured the active goal, but clarification is required before planning can continue"
            .to_string()
    } else if bundle.markdown_source_count() == 0 {
        "captured the active goal for the current workspace session".to_string()
    } else {
        format!(
            "captured the active goal with {} Markdown brief source(s) for the current workspace session",
            bundle.markdown_source_count()
        )
    };

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some("boundline plan".to_string()),
            if target.cluster_projection.is_some() {
                format!("{summary} for the current clustered delivery session")
            } else {
                summary
            },
        )),
    })
}

pub fn execute_flow(
    workspace: Option<&Path>,
    name: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_flow_with_target(workspace, None, name)
}

pub fn execute_flow_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    name: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "flow")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    runtime.select_flow(&mut record, name).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            if target.cluster_projection.is_some() {
                format!("selected the `{}` delivery flow for the active clustered session", name)
            } else {
                format!("selected the `{}` delivery flow for the active workspace session", name)
            },
        )),
    })
}

pub fn execute_plan(
    workspace: Option<&Path>,
    requested_flow: Option<&str>,
    no_flow: bool,
    confirm: bool,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_plan_with_target(workspace, None, requested_flow, no_flow, confirm)
}

pub fn execute_plan_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    requested_flow: Option<&str>,
    no_flow: bool,
    confirm: bool,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "plan")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    let plan_result = if confirm {
        runtime.confirm_goal_plan(&mut record)
    } else {
        runtime.plan_task(&mut record, requested_flow, no_flow)
    };

    if let Err(error) = plan_result {
        if matches!(&error, SessionRuntimeError::ClarificationRequired { .. }) {
            runtime.persist_session(&record).map_err(map_runtime_error)?;
        }
        return Err(map_runtime_error(error));
    }
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            if target.cluster_projection.is_some() {
                format!("{} for the clustered delivery story", planning_summary(&record))
            } else {
                planning_summary(&record)
            },
        )),
    })
}

pub fn execute_step(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_step_with_target(workspace, None)
}

pub fn execute_step_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "step")?.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    if runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)? {
        runtime.persist_session(&record).map_err(map_runtime_error)?;
        return Ok(SessionCommandReport {
            exit_status: exit_status_for_session(record.latest_status),
            terminal_output: output::render_session_status(&build_status_view(
                &record,
                suggested_next_command(&record),
                "refreshed governance approval state and returned without executing another step",
            )),
        });
    }

    runtime.execute_next_step(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: exit_status_for_session(record.latest_status),
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            "executed the next planned step and persisted the updated session state",
        )),
    })
}

pub fn execute_run(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_run_with_target(workspace, None)
}

pub fn execute_run_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "run")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if let Some(projection) = target.cluster_projection.as_ref() {
        runtime.prepare_cluster_run(&mut record, projection).map_err(map_runtime_error)?;
    }

    let uses_native_goal_plan =
        runtime.uses_native_goal_plan(&record).map_err(map_runtime_error)?;

    if !uses_native_goal_plan && record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    if !uses_native_goal_plan
        && runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?
    {
        runtime.persist_session(&record).map_err(map_runtime_error)?;
        return Ok(SessionCommandReport {
            exit_status: exit_status_for_session(record.latest_status),
            terminal_output: output::render_session_status(&build_status_view(
                &record,
                suggested_next_command(&record),
                "refreshed governance approval state and returned without resuming the governed stage",
            )),
        });
    }

    let response = runtime.run_to_terminal(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    if response.terminal_status == TaskStatus::Failed && delegation_status_view(&record).is_some() {
        return Ok(SessionCommandReport {
            exit_status: exit_status_for_task(response.terminal_status),
            terminal_output: output::render_session_status(&build_status_view(
                &record,
                suggested_next_command(&record),
                "run stopped at an explicit delegated continuity boundary and persisted the packet in session-owned state",
            )),
        });
    }

    let trace = runtime.trace_store().load(Path::new(&response.trace_location)).ok();
    let next_command =
        suggested_next_command(&record).unwrap_or_else(|| "boundline inspect".to_string());
    let routing_prefix = output::render_route_outcome(&routing_outcome(&record));

    Ok(SessionCommandReport {
        exit_status: exit_status_for_task(response.terminal_status),
        terminal_output: format!(
            "{routing_prefix}\n{}",
            output::render_run_trace("run", trace.as_ref(), &response, &next_command),
        ),
    })
}

pub fn execute_status(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_status_with_target(workspace, None)
}

pub fn execute_status_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "status")?.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    match load_active_session(&workspace) {
        Ok(mut record) => {
            let refreshed =
                runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?;
            if refreshed {
                runtime.persist_session(&record).map_err(map_runtime_error)?;
            }
            let compatibility_follow_up = latest_workspace_compatibility_follow_up(
                &workspace,
                record.latest_trace_ref.as_deref(),
            )?;

            Ok(SessionCommandReport {
                exit_status: CommandExitStatus::Succeeded,
                terminal_output: output::render_session_status(&build_status_view_with_follow_up(
                    &record,
                    suggested_next_command(&record),
                    if compatibility_follow_up.is_some() {
                        "current active session state for the workspace; latest compatibility follow-up remains inspect-only"
                    } else if refreshed {
                        "refreshed governance approval state for the active workspace session"
                    } else {
                        "current active session state for the workspace"
                    },
                    compatibility_follow_up,
                )),
            })
        }
        Err(SessionCommandError::MissingActiveSession) => {
            let Some(compatibility_follow_up) =
                latest_workspace_compatibility_follow_up(&workspace, None)?
            else {
                return Err(SessionCommandError::MissingActiveSession);
            };

            Ok(SessionCommandReport {
                exit_status: CommandExitStatus::Succeeded,
                terminal_output: output::render_compatibility_follow_up_status(
                    &workspace.to_string_lossy(),
                    ContinuityAuthority::CompatibilityTrace,
                    &compatibility_follow_up,
                    "no active session exists; latest compatibility trace is the authoritative follow-up state for the workspace",
                ),
            })
        }
        Err(error) => Err(error),
    }
}

pub fn execute_next(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_next_with_target(workspace, None)
}

pub fn execute_next_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "next")?.owner_workspace;
    match load_active_session(&workspace) {
        Ok(record) => {
            let next_command =
                suggested_next_command(&record).ok_or(SessionCommandError::NotImplemented {
                    command_name: "next",
                    next_command: None,
                })?;
            let compatibility_follow_up = latest_workspace_compatibility_follow_up(
                &workspace,
                record.latest_trace_ref.as_deref(),
            )?;

            Ok(SessionCommandReport {
                exit_status: CommandExitStatus::Succeeded,
                terminal_output: output::render_session_status(&build_status_view_with_follow_up(
                    &record,
                    Some(next_command.clone()),
                    if let Some(follow_up) = &compatibility_follow_up {
                        format!(
                            "next recommended command for the active session is `{next_command}`; latest compatibility follow-up remains {} via `{}`",
                            follow_up.follow_up_mode.as_str(),
                            follow_up.next_command
                        )
                    } else {
                        format!(
                            "next recommended command for the active session is `{next_command}`"
                        )
                    },
                    compatibility_follow_up,
                )),
            })
        }
        Err(SessionCommandError::MissingActiveSession) => {
            let Some(compatibility_follow_up) =
                latest_workspace_compatibility_follow_up(&workspace, None)?
            else {
                return Err(SessionCommandError::MissingActiveSession);
            };

            Ok(SessionCommandReport {
                exit_status: CommandExitStatus::Succeeded,
                terminal_output: output::render_compatibility_follow_up_status(
                    &workspace.to_string_lossy(),
                    ContinuityAuthority::CompatibilityTrace,
                    &compatibility_follow_up,
                    format!(
                        "next recommended command for the latest compatibility follow-up is `{}`",
                        compatibility_follow_up.next_command
                    ),
                ),
            })
        }
        Err(error) => Err(error),
    }
}

pub fn render_error(command_name: &str, error: &SessionCommandError) -> String {
    let next_command = error.next_command();
    output::render_session_error(command_name, &error.message(), next_command.as_deref())
}

fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, SessionCommandError> {
    let candidate = match workspace {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => std::env::current_dir()?.join(path),
        None => std::env::current_dir()?,
    };

    Ok(candidate.canonicalize().unwrap_or(candidate))
}

fn resolve_session_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    command_name: &'static str,
) -> Result<ResolvedSessionTarget, SessionCommandError> {
    if let Some(cluster_workspace) = cluster {
        let owner_workspace = resolve_workspace(Some(cluster_workspace))?;
        let cluster_store = FileClusterStore::for_workspace(&owner_workspace);
        let Some(config) = cluster_store.load().map_err(SessionCommandError::ClusterStore)? else {
            return Err(SessionCommandError::MissingClusterConfig {
                workspace: owner_workspace,
                command_name,
            });
        };
        let projection = ClusterSessionProjection {
            cluster_id: config.cluster.cluster_id,
            primary_workspace_ref: config.cluster.primary_workspace_ref,
            member_workspace_refs: config
                .cluster
                .members
                .into_iter()
                .map(|member| member.workspace_ref)
                .collect(),
            started_from_command: command_name.to_string(),
            updated_at: current_timestamp_millis(),
        };

        return Ok(ResolvedSessionTarget { owner_workspace, cluster_projection: Some(projection) });
    }

    Ok(ResolvedSessionTarget {
        owner_workspace: resolve_workspace(workspace)?,
        cluster_projection: None,
    })
}

fn load_active_session(workspace: &Path) -> Result<ActiveSessionRecord, SessionCommandError> {
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let store = FileSessionStore::for_workspace(workspace);
    let Some(record) = store.load().map_err(map_store_error)? else {
        return Err(SessionCommandError::MissingActiveSession);
    };

    if record.workspace_ref != workspace_ref {
        return Err(SessionCommandError::WorkspaceMismatch {
            expected: workspace_ref,
            actual: record.workspace_ref,
        });
    }

    Ok(record)
}

fn map_store_error(error: SessionStoreError) -> SessionCommandError {
    match error {
        SessionStoreError::InvalidRecord(message) => {
            SessionCommandError::InvalidActiveSession(message)
        }
        other => SessionCommandError::SessionStore(other),
    }
}

fn map_runtime_error(error: SessionRuntimeError) -> SessionCommandError {
    match error {
        SessionRuntimeError::MissingGoal => SessionCommandError::MissingCapturedGoal,
        SessionRuntimeError::ClarificationRequired { headline, prompt } => {
            SessionCommandError::ClarificationRequired { headline, prompt }
        }
        SessionRuntimeError::MissingActiveTask => SessionCommandError::MissingPlannedTask,
        SessionRuntimeError::PlanConfirmationRequired { flow_name } => {
            SessionCommandError::PlanConfirmationRequired { flow_name }
        }
        SessionRuntimeError::MissingGoalPlan => SessionCommandError::MissingPlanProposal,
        SessionRuntimeError::UnknownFlow { requested, supported } => {
            SessionCommandError::UnknownFlow { requested, supported }
        }
        SessionRuntimeError::FlowReplacementRequiresReset { current, requested } => {
            SessionCommandError::FlowReplacementRequiresReset { current, requested }
        }
        SessionRuntimeError::InvalidFlowState(message) => {
            SessionCommandError::InvalidFlowState(message)
        }
        other => SessionCommandError::SessionRuntime(other),
    }
}

fn exit_status_for_session(status: SessionStatus) -> CommandExitStatus {
    match status {
        SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted
        | SessionStatus::Invalid => CommandExitStatus::NonSuccess,
        SessionStatus::Initialized
        | SessionStatus::GoalCaptured
        | SessionStatus::Planned
        | SessionStatus::Running
        | SessionStatus::Succeeded => CommandExitStatus::Succeeded,
    }
}

fn exit_status_for_task(status: TaskStatus) -> CommandExitStatus {
    match status {
        TaskStatus::Failed | TaskStatus::Exhausted | TaskStatus::Aborted => {
            CommandExitStatus::NonSuccess
        }
        TaskStatus::Planned | TaskStatus::Running | TaskStatus::Succeeded => {
            CommandExitStatus::Succeeded
        }
    }
}

pub(crate) fn build_status_view(
    record: &ActiveSessionRecord,
    next_command: Option<String>,
    explanation: impl Into<String>,
) -> SessionStatusView {
    build_status_view_with_follow_up(record, next_command, explanation, None)
}

pub(crate) fn build_status_view_with_follow_up(
    record: &ActiveSessionRecord,
    next_command: Option<String>,
    explanation: impl Into<String>,
    compatibility_follow_up: Option<CompatibilityFollowUpView>,
) -> SessionStatusView {
    let governance_intent =
        record.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref());
    let latest_decision = record.decisions.last();
    let latest_decision_selector = latest_decision.map(|decision| decision.selector_kind());
    let delegation = delegation_status_view(record);
    let task_context_summary =
        record.active_task.as_ref().and_then(task_state_canon_memory_context_summary);
    let task_context_credibility =
        record.active_task.as_ref().and_then(task_state_canon_memory_context_credibility);
    let task_context_primary_inputs =
        record.active_task.as_ref().and_then(task_state_canon_memory_primary_inputs);
    let task_context_provenance =
        record.active_task.as_ref().and_then(task_state_canon_memory_provenance);
    let task_context_staleness_reason =
        record.active_task.as_ref().and_then(task_state_canon_memory_staleness_reason);

    SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        negotiation_goal_summary: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.goal_summary.clone()),
        negotiation_resolution: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.resolution_state.as_str().to_string()),
        negotiation_acceptance_boundary: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.acceptance_boundary.success_headline.clone()),
        cluster_delivery_story: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.cluster_delivery_story.clone())
            .or_else(|| {
                record
                    .active_task
                    .as_ref()
                    .and_then(|task| task.context.cluster_delivery_story().ok().flatten())
            }),
        authored_input_summary: record.authored_brief.as_ref().map(|bundle| bundle.summary_text()),
        authored_input_sources: record
            .authored_brief
            .as_ref()
            .map(|bundle| bundle.ordered_source_labels()),
        authored_input_deduplicated_sources: record.authored_brief.as_ref().and_then(|bundle| {
            let labels = bundle.deduplicated_source_labels();
            (!labels.is_empty()).then_some(labels)
        }),
        context_summary: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_summary())
            .or(task_context_summary),
        context_credibility: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_credibility())
            .or(task_context_credibility),
        context_primary_inputs: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let inputs = goal_plan.context_primary_inputs();
                (!inputs.is_empty()).then_some(inputs)
            })
            .or(task_context_primary_inputs),
        context_provenance: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let lines = goal_plan.context_provenance_lines();
                (!lines.is_empty()).then_some(lines)
            })
            .or(task_context_provenance),
        context_staleness_reason: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_pack.as_ref())
            .and_then(|pack| pack.staleness_reason.clone())
            .or_else(|| {
                record
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.canon_memory_staleness_reason())
            })
            .or(task_context_staleness_reason),
        clarification_headline: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_headline),
        clarification_prompt: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_prompt),
        clarification_missing_fields: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_missing_fields),
        requested_governance_runtime: governance_intent
            .and_then(|intent| intent.runtime_preference)
            .map(|runtime| requested_governance_runtime_text(runtime).to_string()),
        requested_governance_risk: governance_intent.and_then(|intent| intent.risk.clone()),
        requested_governance_zone: governance_intent.and_then(|intent| intent.zone.clone()),
        requested_governance_owner: governance_intent.and_then(|intent| intent.owner.clone()),
        active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
        flow_state: record
            .goal_plan
            .as_ref()
            .map(|goal_plan| goal_plan.flow_state().summary_text()),
        goal_plan_state: record
            .goal_plan
            .as_ref()
            .map(|goal_plan| goal_plan.proposal_state_text().to_string()),
        goal_plan_revision: record.goal_plan.as_ref().map(|goal_plan| goal_plan.proposal_revision),
        planning_rationale: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.planning_rationale.clone()),
        verification_strategy: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.verification_strategy.clone()),
        active_workflow: record.active_workflow_name(),
        workflow_phase: record.active_workflow_phase_text(),
        workflow_next_action: record.active_workflow_next_action(),
        continuity_authority: delegation
            .as_ref()
            .map(|_| ContinuityAuthority::NativeSession)
            .or_else(|| {
                compatibility_follow_up.as_ref().map(|_| ContinuityAuthority::CompatibilityTrace)
            }),
        delegation,
        compatibility_follow_up,
        current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
        current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
        total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
        plan_revision: record.active_task.as_ref().map(|task| task.plan.revision),
        current_step_id: record
            .active_task
            .as_ref()
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
        current_step_index: record.active_task.as_ref().map(|task| task.plan.current_step_index),
        latest_status: record.latest_status,
        execution_path: execution_path_text(record),
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_decision_status: latest_decision
            .map(|decision| decision_status_text(decision.status).to_string()),
        latest_decision_target: latest_decision.map(|decision| decision.target.clone()),
        latest_changed_files: record.active_task.as_ref().and_then(|task| {
            task.context.state.get("latest_changed_files").and_then(|value| {
                value.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
            })
        }),
        latest_workspace_slice: record
            .active_task
            .as_ref()
            .and_then(task_state_workspace_slice_summary),
        latest_selection_headline: record
            .active_task
            .as_ref()
            .and_then(|task| {
                task.context
                    .state
                    .get("latest_selection_headline")
                    .and_then(|value| value.as_str().map(str::to_string))
            })
            .or_else(|| {
                latest_decision.map(|decision| {
                    let evidence_suffix = decision_evidence_basis(decision)
                        .map(|basis| format!(" based on {basis}"))
                        .unwrap_or_default();
                    format!(
                        "selector {} -> {} (verify: {}){}",
                        decision.selector_kind().as_str(),
                        decision.target,
                        decision.expected_outcome,
                        evidence_suffix,
                    )
                })
            }),
        latest_candidate_family: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_candidate_family"))
            .or_else(|| latest_decision_selector.map(|selector| selector.as_str().to_string())),
        latest_selection_reason: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_selection_reason"))
            .or_else(|| latest_decision.map(|decision| decision.rationale.clone())),
        latest_rejected_candidates: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_strings(task, "latest_rejected_candidates")),
        latest_attempt_lineage: record
            .active_task
            .as_ref()
            .and_then(task_state_attempt_lineage_summary),
        latest_validation_status: record
            .active_task
            .as_ref()
            .and_then(|task| {
                task.context
                    .state
                    .get("latest_validation_status")
                    .and_then(|value| value.as_str().map(str::to_string))
            })
            .or_else(|| {
                latest_decision.and_then(|decision| {
                    match (decision.selector_kind(), decision.tool_result.as_ref()) {
                        (ActionSelector::Test, Some(tool_result)) => Some(if tool_result.success {
                            "passed".to_string()
                        } else {
                            "failed".to_string()
                        }),
                        _ => None,
                    }
                })
            }),
        latest_exhaustion_reason: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_exhaustion_reason")),
        latest_review_trigger: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_trigger")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_vote: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_vote")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_outcome: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_outcome")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_headline: record.active_task.as_ref().and_then(review_headline_from_task),
        latest_governance_stage: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_stage_key),
        latest_governance_runtime: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_runtime_text),
        latest_governance_mode: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_mode_text),
        latest_governance_run_ref: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_canon_run_ref),
        latest_governance_state: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_state_text),
        latest_governance_blocked_reason: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_blocked_reason),
        latest_governance_packet_ref: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_ref),
        latest_governance_packet_source_stage: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_source_stage),
        latest_governance_packet_binding_reason: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_binding_reason),
        latest_governance_approval: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_approval_text),
        latest_governance_decision: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_decision_headline),
        latest_governance_candidates: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_candidate_actions),
        governance_next_action: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_next_action),
        next_command,
        explanation: explanation.into(),
    }
}

fn latest_workspace_compatibility_follow_up(
    workspace: &Path,
    session_trace_ref: Option<&str>,
) -> Result<Option<CompatibilityFollowUpView>, SessionCommandError> {
    let store = FileTraceStore::for_workspace(workspace);
    let Some(trace_path) = store.latest().map_err(|error| {
        SessionCommandError::SessionRuntime(SessionRuntimeError::TraceStore(error))
    })?
    else {
        return Ok(None);
    };

    if session_trace_ref.is_some_and(|trace_ref| Path::new(trace_ref) == trace_path.as_path()) {
        return Ok(None);
    }

    let trace = store.load(&trace_path).map_err(|error| {
        SessionCommandError::SessionRuntime(SessionRuntimeError::TraceStore(error))
    })?;
    let summary = summarize_trace(&trace_path, &trace)
        .map_err(|error| SessionCommandError::TraceSummary(error.to_string()))?;
    let Some(routing_summary) = summary.routing_summary.clone() else {
        return Ok(None);
    };

    if !routing_summary.starts_with("routing: compatibility") {
        return Ok(None);
    }

    Ok(Some(CompatibilityFollowUpView {
        follow_up_mode: CompatibilityFollowUpMode::InspectOnly,
        trace_ref: trace_path.to_string_lossy().into_owned(),
        routing_summary,
        execution_condition: output::trace_execution_condition_text(&summary),
        terminal_status: summary.terminal_status,
        terminal_reason: summary.terminal_reason.message.clone(),
        next_command: format!("boundline inspect --workspace {}", workspace.display()),
    }))
}

fn requested_governance_runtime_text(runtime: GovernanceRuntimeKind) -> &'static str {
    match runtime {
        GovernanceRuntimeKind::Local => "local",
        GovernanceRuntimeKind::Canon => "canon",
    }
}

fn review_headline_from_task(task: &crate::domain::task::Task) -> Option<String> {
    let latest_finding = task
        .context
        .state
        .get("latest_review_findings")
        .and_then(Value::as_array)
        .and_then(|findings| findings.last());
    if let Some(finding) = latest_finding {
        let reviewer_id = finding.get("reviewer_id").and_then(Value::as_str).unwrap_or("reviewer");
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        return Some(format!("{reviewer_id} {disposition}: {summary}"));
    }

    let participants = task
        .context
        .state
        .get("latest_review_participants")
        .and_then(Value::as_array)
        .map(|participants| {
            participants
                .iter()
                .filter_map(|participant| {
                    let reviewer_id = participant.get("reviewer_id").and_then(Value::as_str)?;
                    let status =
                        participant.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    Some(format!("{reviewer_id} {status}"))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if participants.is_empty() {
        None
    } else {
        Some(format!("participants: {}", participants.join(", ")))
    }
}

fn suggested_next_command(record: &ActiveSessionRecord) -> Option<String> {
    if record.authored_brief.as_ref().and_then(|bundle| bundle.clarification.as_ref()).is_some() {
        return Some("boundline capture --goal <narrower goal>".to_string());
    }

    if let Some(next_command) = delegation_next_command(record) {
        return Some(next_command);
    }

    if record.goal_plan.as_ref().and_then(|goal_plan| goal_plan.context_pack.as_ref()).is_some_and(
        |pack| pack.credibility != crate::domain::goal_plan::ContextPackCredibility::Credible,
    ) {
        return Some("boundline capture --goal <narrower goal>".to_string());
    }

    if let Some(task) = record.active_task.as_ref()
        && let Some(governance_state) = task_state_governance_state_text(task)
    {
        match governance_state.as_str() {
            "awaiting_approval" => return Some("boundline status".to_string()),
            "blocked" | "failed" => return Some("boundline inspect".to_string()),
            _ => {}
        }
    }

    match record.latest_status {
        SessionStatus::Initialized => Some("boundline capture --goal <goal>".to_string()),
        SessionStatus::GoalCaptured => Some("boundline plan".to_string()),
        SessionStatus::Planned => {
            if let Some(goal_plan) = record.goal_plan.as_ref()
                && goal_plan.requires_confirmation()
            {
                return Some("boundline plan --confirm".to_string());
            }

            if record.goal_plan.is_some() && record.active_task.is_none() {
                return Some("boundline run".to_string());
            }

            Some("boundline step".to_string())
        }
        SessionStatus::Running => Some("boundline step".to_string()),
        SessionStatus::Succeeded
        | SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted => Some("boundline inspect".to_string()),
        SessionStatus::Invalid => Some("boundline start".to_string()),
    }
}

fn planning_summary(record: &ActiveSessionRecord) -> String {
    let Some(goal_plan) = record.goal_plan.as_ref() else {
        return "planned the active goal into a resumable task snapshot".to_string();
    };

    let task_count = goal_plan.tasks.len();
    if goal_plan.requires_confirmation() {
        if let Some(flow) = goal_plan.flow.as_ref() {
            return format!(
                "planned the active goal into {task_count} bounded goal-plan task(s); proposed `{}` flow is persisted and awaiting plan confirmation",
                flow.flow_name
            );
        }

        if goal_plan.flow_skipped {
            return format!(
                "planned the active goal into {task_count} bounded goal-plan task(s) with operator-skipped flow constraints; the proposed plan is awaiting confirmation"
            );
        }

        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s); the proposed plan is awaiting confirmation"
        );
    }

    if let Some(flow) = goal_plan.flow.as_ref()
        && flow.confirmed
    {
        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s) with confirmed `{}` flow",
            flow.flow_name
        );
    }

    if goal_plan.flow_skipped {
        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s) with operator-skipped flow constraints"
        );
    }

    format!(
        "planned the active goal into {task_count} bounded goal-plan task(s) without flow constraints"
    )
}

fn decision_evidence_basis(decision: &crate::domain::decision::Decision) -> Option<String> {
    let inputs = decision
        .evidence_inputs
        .iter()
        .map(|evidence| {
            let kind = match evidence.kind {
                crate::domain::decision::EvidenceKind::Trace => "trace",
                crate::domain::decision::EvidenceKind::File => "file",
                crate::domain::decision::EvidenceKind::Canon => "canon",
                crate::domain::decision::EvidenceKind::ToolOutput => "tool_output",
            };
            format!("{kind}:{}", evidence.reference)
        })
        .collect::<Vec<_>>();
    (!inputs.is_empty()).then_some(inputs.join(", "))
}

#[derive(Debug, Error)]
pub enum SessionCommandError {
    #[error("failed to resolve the current workspace: {0}")]
    WorkspaceResolution(#[from] std::io::Error),
    #[error("no active session found for the current workspace")]
    MissingActiveSession,
    #[error("active session is invalid: {0}")]
    InvalidActiveSession(String),
    #[error("active session belongs to a different workspace: expected {expected}, got {actual}")]
    WorkspaceMismatch { expected: String, actual: String },
    #[error("active session has no captured goal")]
    MissingCapturedGoal,
    #[error("active session has no planned task")]
    MissingPlannedTask,
    #[error("active session has no proposed goal plan")]
    MissingPlanProposal,
    #[error("active session has a proposed plan that must be confirmed before execution")]
    PlanConfirmationRequired { flow_name: Option<String> },
    #[error("unknown flow `{requested}`; supported flows: {supported}")]
    UnknownFlow { requested: String, supported: String },
    #[error(
        "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
    )]
    FlowReplacementRequiresReset { current: String, requested: String },
    #[error("active session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session runtime operation failed: {0}")]
    SessionRuntime(#[from] SessionRuntimeError),
    #[error("failed to ingest authored brief: {0}")]
    BriefIngestion(#[from] BriefIngestionError),
    #[error("cluster store operation failed: {0}")]
    ClusterStore(#[from] ClusterStoreError),
    #[error("`{command_name}` requires a valid cluster config in {workspace}")]
    MissingClusterConfig { workspace: PathBuf, command_name: &'static str },
    #[error("failed to summarize the latest compatibility trace: {0}")]
    TraceSummary(String),
    #[error("{headline}: {prompt}")]
    ClarificationRequired { headline: String, prompt: String },
    #[error("`{command_name}` session workflow is not implemented yet")]
    NotImplemented { command_name: &'static str, next_command: Option<&'static str> },
}

impl SessionCommandError {
    fn message(&self) -> String {
        match self {
            Self::MissingActiveSession => {
                "no active session found for the current workspace".to_string()
            }
            Self::InvalidActiveSession(message) => format!("active session is invalid: {message}"),
            Self::WorkspaceMismatch { expected, actual } => {
                format!(
                    "active session belongs to a different workspace: expected {expected}, got {actual}"
                )
            }
            Self::MissingCapturedGoal => "active session has no captured goal".to_string(),
            Self::MissingPlannedTask => "active session has no planned task".to_string(),
            Self::MissingPlanProposal => {
                "active session has no proposed goal plan; run `boundline plan` first".to_string()
            }
            Self::PlanConfirmationRequired { flow_name } => match flow_name.as_deref() {
                Some(flow_name) => format!(
                    "active session has a proposed `{flow_name}` plan that must be confirmed before execution; run `boundline plan --confirm` to confirm the proposal"
                ),
                None => {
                    "active session has a proposed plan that must be confirmed before execution; run `boundline plan --confirm` to confirm the proposal".to_string()
                }
            }
            Self::UnknownFlow { requested, supported } => {
                format!("unknown flow `{requested}`; supported flows: {supported}")
            }
            Self::FlowReplacementRequiresReset { current, requested } => {
                format!(
                    "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
                )
            }
            Self::InvalidFlowState(message) => {
                format!("active session flow state is invalid: {message}")
            }
            Self::NotImplemented { command_name, .. } => {
                format!("`{command_name}` session workflow is not implemented yet")
            }
            Self::WorkspaceResolution(error) => error.to_string(),
            Self::SessionStore(error) => error.to_string(),
            Self::SessionRuntime(error) => error.to_string(),
            Self::BriefIngestion(error) => format!("failed to ingest authored brief: {error}"),
            Self::ClusterStore(error) => error.to_string(),
            Self::MissingClusterConfig { workspace, command_name } => {
                format!(
                    "`{command_name}` requires a valid cluster config in {}",
                    workspace.display()
                )
            }
            Self::TraceSummary(message) => {
                format!("failed to summarize the latest compatibility trace: {message}")
            }
            Self::ClarificationRequired { headline, prompt } => format!("{headline}: {prompt}"),
        }
    }

    fn next_command(&self) -> Option<String> {
        match self {
            Self::MissingActiveSession
            | Self::WorkspaceMismatch { .. }
            | Self::InvalidActiveSession(_) => Some("boundline start".to_string()),
            Self::MissingCapturedGoal => Some("boundline capture --goal <goal>".to_string()),
            Self::MissingPlannedTask => Some("boundline plan".to_string()),
            Self::MissingPlanProposal => Some("boundline plan".to_string()),
            Self::PlanConfirmationRequired { .. } => Some("boundline plan --confirm".to_string()),
            Self::UnknownFlow { .. } => Some("boundline flow bug-fix".to_string()),
            Self::FlowReplacementRequiresReset { .. } => Some("boundline start".to_string()),
            Self::InvalidFlowState(_) => Some("boundline start".to_string()),
            Self::NotImplemented { next_command, .. } => next_command.map(str::to_string),
            Self::ClarificationRequired { .. } => {
                Some("boundline capture --goal <narrower goal>".to_string())
            }
            Self::WorkspaceResolution(_)
            | Self::SessionStore(_)
            | Self::SessionRuntime(_)
            | Self::ClusterStore(_) => None,
            Self::TraceSummary(_) => None,
            Self::BriefIngestion(_) => Some("boundline capture --goal <goal>".to_string()),
            Self::MissingClusterConfig { .. } => Some("boundline cluster init --workspace <primary> --cluster-id <id> --member <workspace> --member <workspace>".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        CommandExitStatus, SessionCommandError, build_status_view_with_follow_up, execute_capture,
        execute_flow, execute_next, execute_plan, execute_run, execute_start,
        execute_start_with_target, execute_status, exit_status_for_session, exit_status_for_task,
        latest_workspace_compatibility_follow_up, load_active_session, map_runtime_error,
        map_store_error, render_error, requested_governance_runtime_text, resolve_workspace,
        review_headline_from_task, suggested_next_command,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::adapters::session_store::SessionStoreError;
    use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
    use crate::domain::configuration::{
        CapabilityState, ConfigFile, EffortFallbackPolicy, EffortLevel, ModelRoute, RouteSlot,
        RoutingConfig, RuntimeCapabilityProfile, RuntimeKind, SlotEffortPolicy,
    };
    use crate::domain::decision::{Decision, DecisionType, EvidenceRef};
    use crate::domain::goal_plan::{
        ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan,
        InferredFlow, PlannedTask,
    };
    use crate::domain::governance::{
        CompactedCanonMemory, GovernanceRuntimeKind, MemoryCredibilityState,
    };
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::SessionStatus;
    use crate::domain::task::{Task, TaskStatus, TerminalReason};
    use crate::domain::trace::ExecutionTrace;
    use crate::fixture::{build_fixture_plan_for_goal, build_task_request};
    use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};

    const FIXTURE_CARGO_TOML: &str = r#"[package]
name = "session_cli_fixture"
version = "0.1.0"
edition = "2024"
"#;

    const RED_LIB_RS: &str = "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n";

    const FIXTURE_TEST_RS: &str = r#"#[test]
fn red_to_green_addition() {
    assert_eq!(session_cli_fixture::add(2, 2), 4);
}
"#;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn write_execution_workspace(prefix: &str) -> PathBuf {
        let workspace = temp_workspace(prefix);
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
        fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
        fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
        fs::write(
            workspace.join(".boundline/execution.json"),
            serde_json::to_string_pretty(&json!({
                "name": "session-execution",
                "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
                "validation_command": {
                    "program": "cargo",
                    "args": ["test", "--quiet"]
                },
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Replace subtraction with addition",
                        "failure_mode": "terminal",
                        "changes": [
                            {
                                "path": "src/lib.rs",
                                "find": "left - right",
                                "replace": "left + right"
                            }
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn write_review_execution_workspace(prefix: &str) -> PathBuf {
        let workspace = temp_workspace(prefix);
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
        fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
        fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
        fs::write(
            workspace.join(".boundline/execution.json"),
            serde_json::to_string_pretty(&json!({
                "name": "session-review-execution",
                "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
                "validation_command": {
                    "program": "cargo",
                    "args": ["test", "--quiet"]
                },
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Replace subtraction with addition",
                        "failure_mode": "terminal",
                        "changes": [
                            {
                                "path": "src/lib.rs",
                                "find": "left - right",
                                "replace": "left + right"
                            }
                        ]
                    }
                ],
                "review": {
                    "triggers": ["pr_ready"],
                    "reviewers": [
                        {
                            "reviewer_id": "safety",
                            "role": "Safety",
                            "source": "gpt",
                            "weight": 1
                        },
                        {
                            "reviewer_id": "maintainability",
                            "role": "Maintainability",
                            "source": "claude",
                            "weight": 1
                        }
                    ],
                    "vote_rule": {
                        "strategy": "majority"
                    },
                    "scenarios": [
                        {
                            "trigger": "pr_ready",
                            "findings": [
                                {
                                    "reviewer_id": "safety",
                                    "disposition": "approve",
                                    "summary": "No blockers"
                                },
                                {
                                    "reviewer_id": "maintainability",
                                    "disposition": "approve",
                                    "summary": "Ready to ship"
                                }
                            ]
                        }
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn seed_fixture_planned_session(workspace: &Path, flow_name: &str) {
        let canonical_workspace = workspace.canonicalize().unwrap();
        let runtime = SessionRuntime::for_workspace(&canonical_workspace);
        let mut record = load_active_session(&canonical_workspace).unwrap();
        runtime.select_flow(&mut record, flow_name).unwrap();

        let request = build_task_request(
            &canonical_workspace,
            record.goal.clone().unwrap_or_default(),
            record.session_id.clone(),
            record.authored_brief.as_ref(),
            record.negotiation_packet.as_ref(),
        )
        .unwrap();
        let plan = build_fixture_plan_for_goal(
            &canonical_workspace,
            record.active_flow.as_ref(),
            record.goal.as_deref().unwrap_or_default(),
        )
        .unwrap();

        record.active_task = Some(Task::new("task-session-cli", &request, plan).unwrap());
        record.goal_plan = None;
        record.active_flow_policy = None;
        record.latest_status = SessionStatus::Planned;
        runtime.persist_session(&record).unwrap();
    }

    #[test]
    fn resolve_workspace_and_status_helpers_cover_remaining_branches() {
        let workspace = temp_workspace("boundline-cli-session-resolve");
        let child = workspace.join("child");
        fs::create_dir_all(&child).unwrap();

        let previous_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&workspace).unwrap();
        let resolved = resolve_workspace(Some(Path::new("child"))).unwrap();
        std::env::set_current_dir(previous_dir).unwrap();

        assert_eq!(resolved, child.canonicalize().unwrap());
        assert_eq!(exit_status_for_session(SessionStatus::Invalid), CommandExitStatus::NonSuccess);
        assert_eq!(exit_status_for_task(TaskStatus::Failed), CommandExitStatus::NonSuccess);
        assert_eq!(
            suggested_next_command(&crate::domain::session::ActiveSessionRecord {
                session_id: "session".to_string(),
                workspace_ref: "/tmp/workspace".to_string(),
                goal: None,
                authored_brief: None,
                negotiation_packet: None,
                active_flow: None,
                active_task: None,
                goal_plan: None,
                workflow_progress: None,
                decisions: Vec::new(),
                active_flow_policy: None,
                latest_status: SessionStatus::Invalid,
                latest_terminal_reason: None,
                latest_trace_ref: None,
                created_at: 1,
                updated_at: 1,
            }),
            Some("boundline start".to_string())
        );
    }

    #[test]
    fn store_and_runtime_error_mapping_cover_translated_variants() {
        assert!(matches!(
            map_store_error(SessionStoreError::InvalidRecord("bad session".to_string())),
            SessionCommandError::InvalidActiveSession(message) if message == "bad session"
        ));
        assert!(matches!(
            map_store_error(SessionStoreError::Read(std::io::Error::other("read failed"))),
            SessionCommandError::SessionStore(_)
        ));

        assert!(matches!(
            map_runtime_error(SessionRuntimeError::MissingGoal),
            SessionCommandError::MissingCapturedGoal
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::MissingActiveTask),
            SessionCommandError::MissingPlannedTask
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::UnknownFlow {
                requested: "missing".to_string(),
                supported: "bug-fix".to_string(),
            }),
            SessionCommandError::UnknownFlow { .. }
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::FlowReplacementRequiresReset {
                current: "bug-fix".to_string(),
                requested: "delivery".to_string(),
            }),
            SessionCommandError::FlowReplacementRequiresReset { .. }
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::InvalidFlowState("bad flow".to_string())),
            SessionCommandError::InvalidFlowState(message) if message == "bad flow"
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::TraceStore(TraceStoreError::Read(
                std::io::Error::other("trace read failed")
            ))),
            SessionCommandError::SessionRuntime(_)
        ));
    }

    #[test]
    fn session_command_error_helpers_cover_messages_and_next_commands() {
        let unknown_flow = SessionCommandError::UnknownFlow {
            requested: "missing".to_string(),
            supported: "bug-fix, change, delivery".to_string(),
        };
        let text = render_error("flow", &unknown_flow);
        assert!(text.contains("boundline flow bug-fix"), "{text}");

        let reset_required = SessionCommandError::FlowReplacementRequiresReset {
            current: "bug-fix".to_string(),
            requested: "delivery".to_string(),
        };
        let text = render_error("flow", &reset_required);
        assert!(text.contains("boundline start"), "{text}");

        let not_implemented = SessionCommandError::NotImplemented {
            command_name: "next",
            next_command: Some("boundline inspect"),
        };
        let text = render_error("next", &not_implemented);
        assert!(text.contains("boundline inspect"), "{text}");

        let runtime_error =
            SessionCommandError::SessionRuntime(SessionRuntimeError::MissingTraceReference);
        let text = render_error("run", &runtime_error);
        assert!(!text.contains("next_command:"), "{text}");
    }

    #[test]
    fn clustered_session_commands_resolve_the_primary_workspace_explicitly() {
        let primary = write_execution_workspace("boundline-cli-session-cluster-primary");
        let secondary = write_execution_workspace("boundline-cli-session-cluster-secondary");
        crate::cli::cluster::execute_init(
            &primary,
            "cluster-1",
            &[primary.clone(), secondary.clone()],
        )
        .unwrap();

        let report = execute_start_with_target(None, Some(&primary)).unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            report
                .terminal_output
                .contains("active clustered session initialized for the current primary workspace"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn execute_run_status_and_next_cover_success_paths() {
        let workspace = write_execution_workspace("boundline-cli-session-success");

        assert_eq!(
            execute_start(Some(&workspace)).unwrap().exit_status,
            CommandExitStatus::Succeeded
        );
        assert_eq!(
            execute_capture(
                Some(&workspace),
                Some("Fix the failing add test"),
                &[],
                None,
                None,
                None,
                None,
            )
            .unwrap()
            .exit_status,
            CommandExitStatus::Succeeded
        );
        assert_eq!(
            execute_plan(Some(&workspace), None, false, false).unwrap().exit_status,
            CommandExitStatus::Succeeded
        );

        let planned = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();
        assert!(
            planned.terminal_output.contains("confirmed `bug-fix` flow"),
            "{}",
            planned.terminal_output
        );

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(
            run.terminal_output.contains("terminal_status: succeeded"),
            "{}",
            run.terminal_output
        );
        assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);

        let status = execute_status(Some(&workspace)).unwrap();
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(
            status.terminal_output.contains("latest_status: succeeded"),
            "{}",
            status.terminal_output
        );

        let next = execute_next(Some(&workspace)).unwrap();
        assert_eq!(next.exit_status, CommandExitStatus::Succeeded);
        assert!(
            next.terminal_output.contains("next_command: boundline inspect"),
            "{}",
            next.terminal_output
        );
    }

    #[test]
    fn execute_run_surfaces_delegation_packet_when_native_route_is_blocked() {
        let workspace = write_execution_workspace("boundline-cli-session-delegation");
        let mut config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                implementation: Some(ModelRoute {
                    runtime: RuntimeKind::Claude,
                    model: "sonnet-4".to_string(),
                }),
                assistant_runtimes: vec![RuntimeKind::Codex],
                ..RoutingConfig::default()
            },
        };
        config.routing.slot_effort_policies.insert(
            RouteSlot::Implementation,
            SlotEffortPolicy {
                level: EffortLevel::High,
                fallback: EffortFallbackPolicy::Preserve,
                rationale: Some(
                    "keep implementation continuation on the highest-effort path".to_string(),
                ),
            },
        );
        config.routing.runtime_capabilities.insert(
            RuntimeKind::Claude,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Unsupported,
                resume: CapabilityState::Unsupported,
                validation: CapabilityState::Unsupported,
                handoff_target: CapabilityState::Unsupported,
                escalation_context: CapabilityState::Supported,
                notes: Some("requires a handoff for bounded continuation".to_string()),
            },
        );
        config.routing.runtime_capabilities.insert(
            RuntimeKind::Codex,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Supported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: None,
            },
        );
        FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

        execute_start(Some(&workspace)).unwrap();
        execute_capture(
            Some(&workspace),
            Some("fix the failing add test"),
            &[],
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let plan = execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

        assert!(plan.terminal_output.contains("runtime_capabilities:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("slot_effort_policies:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("planning_rationale:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("routing policy:"), "{}", plan.terminal_output);

        let run = execute_run(Some(&workspace)).unwrap();

        assert_eq!(run.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            run.terminal_output.contains("delegation_mode: handoff_required"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("delegation_packet_kind: handoff"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("delegation_target_owner: codex"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("next_command: boundline status"),
            "{}",
            run.terminal_output
        );
    }

    #[test]
    fn execute_run_status_and_inspect_surface_review_evidence() {
        let workspace = write_review_execution_workspace("boundline-cli-session-review-success");

        assert_eq!(
            execute_start(Some(&workspace)).unwrap().exit_status,
            CommandExitStatus::Succeeded
        );
        assert_eq!(
            execute_capture(
                Some(&workspace),
                Some("Fix the failing add test and review it"),
                &[],
                None,
                None,
                None,
                None,
            )
            .unwrap()
            .exit_status,
            CommandExitStatus::Succeeded
        );
        assert_eq!(
            execute_flow(Some(&workspace), "bug-fix").unwrap().exit_status,
            CommandExitStatus::Succeeded
        );

        seed_fixture_planned_session(&workspace, "bug-fix");

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(
            run.terminal_output.contains("review_trigger: pr_ready"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("reviewer safety (Safety) approve: No blockers"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains(
                "review_vote: strategy=Majority approvals=2 concerns=0 blocks=0 decision=Accepted"
            ),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("review_outcome: accepted"),
            "{}",
            run.terminal_output
        );

        let status = execute_status(Some(&workspace)).unwrap();
        assert!(
            status.terminal_output.contains("latest_review_trigger: pr_ready"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("latest_review_outcome: accepted"),
            "{}",
            status.terminal_output
        );
        assert!(
            status
                .terminal_output
                .contains("latest_review_headline: maintainability approve: Ready to ship"),
            "{}",
            status.terminal_output
        );

        let inspect = crate::cli::inspect::execute_inspect(None, Some(&workspace)).unwrap();
        assert!(
            inspect.terminal_output.contains("review_trigger: pr_ready"),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains(
                "review_vote: strategy=Majority approvals=2 concerns=0 blocks=0 decision=Accepted"
            ),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains("review_outcome: accepted"),
            "{}",
            inspect.terminal_output
        );
    }

    #[test]
    fn execute_run_blocks_until_native_plan_is_confirmed() {
        let workspace = write_execution_workspace("boundline-cli-session-flow-confirmation");

        execute_start(Some(&workspace)).unwrap();
        execute_capture(
            Some(&workspace),
            Some("Fix the failing add test"),
            &[],
            None,
            None,
            None,
            None,
        )
        .unwrap();
        execute_plan(Some(&workspace), None, false, false).unwrap();

        let error = execute_run(Some(&workspace)).unwrap_err();
        assert!(matches!(error, SessionCommandError::PlanConfirmationRequired { .. }));

        let rendered = render_error("run", &error);
        assert!(rendered.contains("boundline plan --confirm"), "{rendered}");

        let confirmed = execute_plan(Some(&workspace), None, false, true).unwrap();
        assert!(confirmed.terminal_output.contains("execution_path: native_goal_plan"));

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
    }

    #[test]
    fn compatibility_follow_up_and_review_headline_helpers_cover_remaining_session_cli_branches() {
        let workspace = temp_workspace("boundline-cli-session-compat-follow-up");
        fs::create_dir_all(workspace.join(".boundline")).unwrap();

        let mut trace = ExecutionTrace::new("task-compat", "session-compat", "Compat trace");
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason = Some(TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "compatibility run failed",
            None,
        ));
        trace.ended_at = Some(trace.started_at + 1);
        let trace_path = FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();

        let follow_up =
            latest_workspace_compatibility_follow_up(&workspace, None).unwrap().unwrap();
        assert_eq!(follow_up.trace_ref, trace_path.to_string_lossy());
        assert!(follow_up.routing_summary.starts_with("routing: compatibility"));
        assert_eq!(
            follow_up.next_command,
            format!("boundline inspect --workspace {}", workspace.display())
        );
        assert!(
            latest_workspace_compatibility_follow_up(&workspace, Some(&follow_up.trace_ref))
                .unwrap()
                .is_none()
        );

        let execution_workspace =
            write_execution_workspace("boundline-cli-session-review-headline");
        let request = build_task_request(
            &execution_workspace,
            "Fix the failing add test".to_string(),
            "session-review".to_string(),
            None,
            None,
        )
        .unwrap();
        let plan =
            build_fixture_plan_for_goal(&execution_workspace, None, "Fix the failing add test")
                .unwrap();
        let mut task = Task::new("task-review", &request, plan).unwrap();
        task.context.state.insert(
            "latest_review_participants".to_string(),
            json!([
                {"reviewer_id": "safety", "status": "pending"},
                {"reviewer_id": "maintainability"}
            ]),
        );
        assert_eq!(
            review_headline_from_task(&task),
            Some("participants: safety pending, maintainability unknown".to_string())
        );

        assert_eq!(requested_governance_runtime_text(GovernanceRuntimeKind::Local), "local");
        assert_eq!(requested_governance_runtime_text(GovernanceRuntimeKind::Canon), "canon");
    }

    #[test]
    fn status_view_falls_back_to_test_decision_validation_and_evidence_basis() {
        let workspace = temp_workspace("boundline-cli-session-selector-fallback");
        let mut decision = Decision::new(
            DecisionType::Test,
            "test suite",
            "run bounded validation",
            "collect validation evidence",
            vec![
                EvidenceRef::trace("trace-1"),
                EvidenceRef::file("src/lib.rs"),
                EvidenceRef::canon(".canon/policy.json"),
                EvidenceRef::tool_output("decision-0"),
            ],
        );
        decision.mark_dispatched().unwrap();
        decision
            .mark_failed(crate::domain::tool_result::ToolResult::new(
                "tester",
                "tester test suite",
                false,
                1,
            ))
            .unwrap();

        let record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-selector-fallback".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: vec![decision],
            active_flow_policy: None,
            latest_status: SessionStatus::Failed,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
        };

        let view = build_status_view_with_follow_up(
            &record,
            Some("boundline inspect".to_string()),
            "inspect the latest decision",
            None,
        );

        assert_eq!(view.latest_validation_status.as_deref(), Some("failed"));
        let headline = view.latest_selection_headline.as_deref().unwrap();
        assert!(headline.contains("trace:trace-1"), "{headline}");
        assert!(headline.contains("file:src/lib.rs"), "{headline}");
        assert!(headline.contains("canon:.canon/policy.json"), "{headline}");
        assert!(headline.contains("tool_output:decision-0"), "{headline}");
        assert_eq!(view.latest_candidate_family.as_deref(), Some("test"));
    }

    #[test]
    fn status_view_projects_task_level_canon_memory_when_goal_plan_is_absent() {
        let workspace = write_execution_workspace("boundline-cli-session-canon-memory");
        let request = build_task_request(
            &workspace,
            "Fix the failing add test",
            "session-canon-memory",
            None,
            None,
        )
        .unwrap();
        let plan =
            build_fixture_plan_for_goal(&workspace, None, "Fix the failing add test").unwrap();
        let mut task = Task::new("task-canon-memory", &request, plan).unwrap();
        task.context
            .set_latest_compacted_canon_memory(&CompactedCanonMemory {
                headline: "Canon verification packet".to_string(),
                credibility: MemoryCredibilityState::Stale,
                stage_key: Some("change:verify".to_string()),
                run_ref: Some("run-9".to_string()),
                packet_ref: Some(".canon/runs/run-9".to_string()),
                reason_code: Some("refresh_required".to_string()),
                artifact_refs: vec![".canon/runs/run-9/verification.md".to_string()],
                mode_summary: None,
                possible_actions: Vec::new(),
                recommended_next_action: Some(
                    crate::domain::governance::CanonRecommendedActionSummary {
                        action: "refresh".to_string(),
                        rationale: "refresh the governed packet and reassess its credibility"
                            .to_string(),
                        target: Some("run-9".to_string()),
                    },
                ),
                evidence_summary: None,
            })
            .unwrap();
        let record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-canon-memory".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: Some(task),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Running,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
        };

        let view = build_status_view_with_follow_up(
            &record,
            Some("boundline inspect".to_string()),
            "inspect the Canon packet",
            None,
        );

        assert_eq!(
            view.context_summary.as_deref(),
            Some("canon memory: Canon verification packet [stale]")
        );
        assert_eq!(view.context_credibility.as_deref(), Some("stale"));
        assert_eq!(
            view.context_primary_inputs.as_deref(),
            Some([".canon/runs/run-9/verification.md".to_string()].as_slice())
        );
        assert_eq!(view.context_staleness_reason.as_deref(), Some("refresh_required"));
        assert_eq!(
            view.governance_next_action.as_deref(),
            Some("refresh: refresh the governed packet and reassess its credibility")
        );
    }

    #[test]
    fn suggested_next_command_and_error_helpers_cover_context_and_flow_follow_up() {
        let goal_plan_with_context = GoalPlan::new(
            "Fix the failing add test",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Fix arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-1".to_string(),
            summary: "bounded context from src/lib.rs".to_string(),
            credibility: ContextPackCredibility::Insufficient,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "closest source file".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            staleness_reason: None,
        });

        let base_record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-next".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(goal_plan_with_context.clone()),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::GoalCaptured,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
        };
        assert_eq!(
            suggested_next_command(&base_record),
            Some("boundline capture --goal <narrower goal>".to_string())
        );

        let mut pending_flow_plan = GoalPlan::new(
            "Fix the failing add test",
            vec![PlannedTask {
                task_id: "planned-task-2".to_string(),
                description: "Fix arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap();
        pending_flow_plan.flow = Some(InferredFlow {
            flow_name: "bug-fix".to_string(),
            confidence_reason: "goal contains fix".to_string(),
            confirmed: false,
        });
        let mut pending_flow_record = base_record.clone();
        pending_flow_record.goal_plan = Some(pending_flow_plan);
        pending_flow_record.latest_status = SessionStatus::Planned;
        assert_eq!(
            suggested_next_command(&pending_flow_record),
            Some("boundline plan --confirm".to_string())
        );

        let mut ready_run_record = pending_flow_record.clone();
        ready_run_record.goal_plan.as_mut().unwrap().confirm().unwrap();
        assert_eq!(suggested_next_command(&ready_run_record), Some("boundline run".to_string()));

        let clarification_error = SessionCommandError::ClarificationRequired {
            headline: "bounded context required before planning".to_string(),
            prompt: "pick one bounded outcome".to_string(),
        };
        let clarification_text = render_error("plan", &clarification_error);
        assert!(
            clarification_text.contains("boundline capture --goal <narrower goal>"),
            "{clarification_text}"
        );

        let cluster_error = SessionCommandError::MissingClusterConfig {
            workspace: PathBuf::from("/tmp/workspace"),
            command_name: "status",
        };
        let cluster_text = render_error("status", &cluster_error);
        assert!(
            cluster_text.contains("boundline cluster init --workspace <primary>"),
            "{cluster_text}"
        );
    }
}
