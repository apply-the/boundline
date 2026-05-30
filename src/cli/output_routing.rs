use std::path::Path;

use serde_json::Value;

use super::{
    ExecutionTrace, FileClusterStore, FileConfigStore, GoalPlanFlowState, ModelRoute,
    ProfileActivationRecord, RoutingConfig, RoutingDecisionProjection, RoutingMode, RoutingOutcome,
    RoutingOverrides, RoutingSource, SessionStatus, SessionStatusView, TaskRunResponse, TaskStatus,
    TraceEventType, TraceSummaryView, resolve_effective_routing,
    resolve_effective_runtime_capabilities, resolve_effective_slot_effort_policies,
};
use crate::domain::session::ContinuityAuthority;

pub fn render_route_outcome(outcome: &RoutingOutcome) -> String {
    format!("routing: {} ({}) - {}", outcome.mode.as_str(), outcome.source.as_str(), outcome.reason)
}

pub fn render_goal_plan_flow_state(flow_state: &GoalPlanFlowState) -> String {
    format!("flow_state: {}", flow_state.summary_text())
}

pub(crate) fn render_session_projection_prefix(view: &SessionStatusView) -> String {
    [
        render_route_outcome(&routing_outcome_for_status_view(view)),
        render_session_execution_condition(view),
    ]
    .join("\n")
}

pub fn trace_execution_condition_text(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("{kind} - {reason}")
}

fn routing_outcome_for_status_view(view: &SessionStatusView) -> RoutingOutcome {
    match view.execution_path.as_deref() {
        Some("native_goal_plan") => RoutingOutcome {
            mode: RoutingMode::Native,
            source: RoutingSource::GoalPlan,
            reason: "goal plan is ready for native execution".to_string(),
        },
        Some("fixture_compatibility") => RoutingOutcome {
            mode: RoutingMode::Compatibility,
            source: RoutingSource::ExecutionProfile,
            reason: "compatibility execution remains active from the persisted task".to_string(),
        },
        Some("native_goal_plan_pending_plan_confirmation") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalPlan,
            reason: "plan confirmation is still pending before native execution".to_string(),
        },
        Some("native_session_pending_plan") => RoutingOutcome {
            mode: RoutingMode::Blocked,
            source: RoutingSource::GoalCapture,
            reason: "goal captured but a goal plan is not ready yet".to_string(),
        },
        _ => match view.latest_status {
            SessionStatus::Initialized => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "capture a goal before planning or execution can begin".to_string(),
            },
            SessionStatus::GoalCaptured => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::GoalCapture,
                reason: "goal captured but a goal plan is not ready yet".to_string(),
            },
            SessionStatus::Blocked => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "planning or governance is blocked and needs repaired input".to_string(),
            },
            SessionStatus::Invalid => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "active session state is invalid and must be recreated".to_string(),
            },
            _ => RoutingOutcome {
                mode: RoutingMode::Blocked,
                source: RoutingSource::SessionState,
                reason: "session has no goal plan or compatibility task to route".to_string(),
            },
        },
    }
}

pub(crate) fn render_route_config_projection(projection: Vec<String>) -> Option<String> {
    (!projection.is_empty()).then(|| format!("route_config_projection: {}", projection.join(" | ")))
}

pub(crate) fn session_route_owner(view: &SessionStatusView) -> &'static str {
    if view.latest_governance_state.is_some() || view.latest_governance_stage.is_some() {
        return "governance";
    }

    if view.latest_review_trigger.is_some()
        || view.latest_review_vote.is_some()
        || view.latest_review_outcome.is_some()
        || view.latest_review_council_profile.is_some()
        || view.latest_review_independence_state.is_some()
        || view.latest_review_stop_semantics.is_some()
        || view.latest_review_selection_summary.is_some()
        || view.latest_review_headline.is_some()
    {
        return "review";
    }

    if view.active_workflow.is_some() {
        return "workflow";
    }

    if matches!(view.continuity_authority, Some(ContinuityAuthority::CompatibilityTrace))
        || matches!(view.execution_path.as_deref(), Some("fixture_compatibility"))
    {
        return "compatibility";
    }

    "native"
}

pub(crate) fn trace_route_owner(summary: &TraceSummaryView) -> &'static str {
    if !summary.governance_timeline.is_empty() {
        return "governance";
    }

    if !summary.review_timeline.is_empty() {
        return "review";
    }

    if summary
        .routing_summary
        .as_deref()
        .is_some_and(|routing| routing.starts_with("routing: compatibility"))
    {
        return "compatibility";
    }

    "native"
}

pub(crate) fn route_config_projection_for_status_view(view: &SessionStatusView) -> Vec<String> {
    let mut projection = current_routing_projection(Path::new(&view.workspace_ref));

    if let Some(active_workflow) = &view.active_workflow {
        projection.push(format!("workflow={active_workflow}"));
    }

    if let Some(workflow_phase) = &view.workflow_phase {
        projection.push(format!("workflow_phase={workflow_phase}"));
    }

    if let Some(flow_state) = &view.flow_state {
        projection.push(format!("flow_state={flow_state}"));
    }

    push_governance_projection_fields(
        &mut projection,
        view.requested_governance_runtime.as_deref(),
        view.requested_governance_risk.as_deref(),
        view.requested_governance_zone.as_deref(),
        view.requested_governance_owner.as_deref(),
    );

    projection
}

pub(crate) fn route_config_projection_for_trace_summary(summary: &TraceSummaryView) -> Vec<String> {
    let mut projection = summary.routing_projection.projection_lines();

    if projection.is_empty()
        && let Some(workspace) = workspace_from_trace_ref(Path::new(&summary.trace_ref))
    {
        projection.extend(current_routing_projection(&workspace));
    }

    push_governance_projection_fields(
        &mut projection,
        summary.requested_governance_runtime.as_deref(),
        summary.requested_governance_risk.as_deref(),
        summary.requested_governance_zone.as_deref(),
        summary.requested_governance_owner.as_deref(),
    );

    projection
}

pub(crate) fn route_config_projection_for_run_trace(
    trace: &ExecutionTrace,
    trace_ref: &Path,
) -> Vec<String> {
    let mut projection = trace_routing_projection(trace);

    if projection.is_empty()
        && let Some(workspace) = workspace_from_trace_ref(trace_ref)
    {
        projection.extend(current_routing_projection(&workspace));
    }

    if let Some(input) = trace.events.iter().find_map(|event| {
        (event.event_type == TraceEventType::TaskStarted)
            .then(|| event.payload.get("input"))
            .flatten()
    }) {
        push_governance_projection_fields(
            &mut projection,
            input.get("requested_governance_runtime").and_then(Value::as_str),
            input.get("requested_governance_risk").and_then(Value::as_str),
            input.get("requested_governance_zone").and_then(Value::as_str),
            input.get("requested_governance_owner").and_then(Value::as_str),
        );
    }

    projection
}

fn trace_routing_projection(trace: &ExecutionTrace) -> Vec<String> {
    trace
        .events
        .iter()
        .find_map(|event| RoutingDecisionProjection::from_event_payload(&event.payload))
        .map(|projection| projection.projection_lines())
        .unwrap_or_default()
}

pub(crate) fn run_trace_route_owner(trace: &ExecutionTrace) -> &'static str {
    let mut saw_native_routing_signal = false;
    let mut saw_review_signal = false;
    let mut saw_governance_signal = false;

    for event in &trace.events {
        if event.event_type.is_decision_loop_event() {
            saw_native_routing_signal = true;
        }

        match event.event_type {
            TraceEventType::GovernanceSelected
            | TraceEventType::GovernanceStarted
            | TraceEventType::GovernanceDecisionRecorded
            | TraceEventType::GovernanceAwaitingApproval
            | TraceEventType::GovernanceCompleted
            | TraceEventType::GovernanceBlocked
            | TraceEventType::GovernancePacketRejected => saw_governance_signal = true,
            TraceEventType::ReviewStarted
            | TraceEventType::ReviewTriggerIgnored
            | TraceEventType::ReviewerCompleted
            | TraceEventType::ReviewCouncilAssembled
            | TraceEventType::ReviewStopSemanticsRecorded
            | TraceEventType::ReviewVoteResolved
            | TraceEventType::ReviewAdjudicated
            | TraceEventType::ReviewTerminalRecorded => saw_review_signal = true,
            _ => {}
        }
    }

    if saw_governance_signal {
        "governance"
    } else if saw_review_signal {
        "review"
    } else if saw_native_routing_signal {
        "native"
    } else {
        "compatibility"
    }
}

fn workspace_from_trace_ref(trace_ref: &Path) -> Option<std::path::PathBuf> {
    let traces_dir = trace_ref.parent()?;
    let boundline_dir = traces_dir.parent()?;
    if traces_dir.file_name()? != "traces" || boundline_dir.file_name()? != ".boundline" {
        return None;
    }

    boundline_dir.parent().map(Path::to_path_buf)
}

fn workspace_routing_projection(workspace: &Path) -> Option<String> {
    let routing = FileConfigStore::for_workspace(workspace).local_routing().ok().flatten()?;
    summarize_routing_config("workspace_routing", &routing)
}

fn current_routing_projection(workspace: &Path) -> Vec<String> {
    let workspace_routing =
        FileConfigStore::for_workspace(workspace).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();

    let mut projection = workspace_routing_projection(workspace).into_iter().collect::<Vec<_>>();

    let effective = resolve_effective_routing(
        &RoutingOverrides::default(),
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    let effective_capabilities = resolve_effective_runtime_capabilities(
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    let effective_effort = resolve_effective_slot_effort_policies(
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    );
    projection.extend(
        RoutingDecisionProjection::from_effective_state(
            &effective,
            &effective_capabilities,
            &effective_effort,
        )
        .projection_lines(),
    );

    projection
}

fn summarize_routing_config(label: &str, routing: &RoutingConfig) -> Option<String> {
    let mut configured_routes = Vec::new();

    if let Some(route) = routing.planning.as_ref() {
        configured_routes.push(format!("planning={}", format_model_route(route)));
    }
    if let Some(route) = routing.implementation.as_ref() {
        configured_routes.push(format!("implementation={}", format_model_route(route)));
    }
    if let Some(route) = routing.verification.as_ref() {
        configured_routes.push(format!("verification={}", format_model_route(route)));
    }
    if let Some(route) = routing.review.as_ref() {
        configured_routes.push(format!("review={}", format_model_route(route)));
    }
    if let Some(route) = routing.adjudication.as_ref() {
        configured_routes.push(format!("adjudication={}", format_model_route(route)));
    }

    if configured_routes.is_empty() {
        None
    } else {
        Some(format!("{label}: {}", configured_routes.join(", ")))
    }
}

fn format_model_route(route: &ModelRoute) -> String {
    format!("{}/{}", route.runtime.as_str(), route.model)
}

pub(crate) fn render_session_execution_condition(view: &SessionStatusView) -> String {
    let (kind, reason) = session_execution_condition_parts(view);
    format!("execution_condition: {kind} - {reason}")
}

pub(crate) fn session_execution_condition_parts(
    view: &SessionStatusView,
) -> (&'static str, String) {
    if let Some(governance_state) = view.latest_governance_state.as_deref() {
        match governance_state {
            "awaiting_approval" => {
                return (
                    "waiting",
                    "governance approval is still pending before execution can continue"
                        .to_string(),
                );
            }
            "blocked" => {
                return (
                    "blocked",
                    view.latest_governance_blocked_reason.clone().unwrap_or_else(|| {
                        "governance blocked further execution until the blocker is resolved"
                            .to_string()
                    }),
                );
            }
            _ => {}
        }
    }

    if let Some(reason) =
        view.latest_reasoning_profile.as_ref().and_then(reasoning_execution_block_reason)
    {
        return ("blocked", reason);
    }

    if let Some(delegation) = &view.delegation {
        return match delegation.mode {
            crate::domain::session::DelegationContinuityMode::HandoffRequired
            | crate::domain::session::DelegationContinuityMode::EscalationRequired
            | crate::domain::session::DelegationContinuityMode::Stuck => {
                ("blocked", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Resolved => {
                ("waiting", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Exhausted
            | crate::domain::session::DelegationContinuityMode::InspectOnly => {
                ("inspect_only", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::None => {
                ("waiting", delegation.headline.clone())
            }
        };
    }

    if let Some(workflow_phase) = view.workflow_phase.as_deref() {
        match workflow_phase {
            "capture" if view.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() => {
                return (
                    "waiting",
                    "workflow is waiting for a captured goal before it can continue".to_string(),
                );
            }
            "clarify"
                if view.clarification_headline.is_some()
                    || view.clarification_prompt.is_some()
                    || view
                        .clarification_missing_fields
                        .as_ref()
                        .is_some_and(|fields| !fields.is_empty()) =>
            {
                return (
                    "waiting",
                    "clarification is still required before workflow planning can continue"
                        .to_string(),
                );
            }
            "review" => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                if view.latest_review_trigger.is_some() && view.latest_review_outcome.is_none() {
                    return (
                        "waiting",
                        "review outcome is still pending before workflow can continue".to_string(),
                    );
                }

                return (
                    "blocked",
                    "workflow review phase requires review evidence from the active session"
                        .to_string(),
                );
            }
            "govern" if view.latest_governance_state.is_none() => {
                if matches!(
                    view.latest_status,
                    SessionStatus::Failed | SessionStatus::Exhausted | SessionStatus::Aborted
                ) {
                    return ("terminal", "work stopped after a non-success result".to_string());
                }

                return (
                    "blocked",
                    "workflow govern phase requires governance evidence from the active session"
                        .to_string(),
                );
            }
            "govern"
                if matches!(
                    view.latest_governance_state.as_deref(),
                    Some("governed_ready" | "completed")
                ) && !view.latest_status.is_terminal() =>
            {
                return ("waiting", "governance is ready and workflow can resume".to_string());
            }
            _ => {}
        }
    }

    if let Some("native_session_pending_plan") = view.execution_path.as_deref() {
        return ("blocked", "goal captured but a goal plan is not ready yet".to_string());
    }

    match view.latest_status {
        SessionStatus::Initialized => {
            ("blocked", "capture a goal before planning or execution can begin".to_string())
        }
        SessionStatus::GoalCaptured => {
            ("blocked", "goal captured but a goal plan is not ready yet".to_string())
        }
        SessionStatus::Planned => (
            "waiting",
            if view.current_step_id.is_some() {
                "a bounded task is ready for the next execution step".to_string()
            } else {
                "planning is complete and execution can begin".to_string()
            },
        ),
        SessionStatus::Blocked => {
            ("blocked", "planning or governance is blocked and needs repaired input".to_string())
        }
        SessionStatus::Running => ("running", running_condition_reason(view).to_string()),
        SessionStatus::Succeeded => ("terminal", "work completed successfully".to_string()),
        SessionStatus::Failed => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "work stopped after a non-success result".to_string())
        }
        SessionStatus::Exhausted => {
            if let Some(reason) = view.latest_exhaustion_reason.clone() {
                return ("terminal", reason);
            }
            ("terminal", "retry or recovery limits were exhausted".to_string())
        }
        SessionStatus::Aborted => ("terminal", "work was aborted before completion".to_string()),
        SessionStatus::Invalid => {
            ("blocked", "active session state is invalid and must be recreated".to_string())
        }
    }
}

fn running_condition_reason(view: &SessionStatusView) -> &'static str {
    match view.latest_decision_status.as_deref() {
        Some("pending") => "a bounded decision is pending dispatch",
        Some("dispatched") => "the latest bounded decision is in flight",
        Some("verified") => "the latest bounded decision was verified and more work may remain",
        Some("failed") => "the latest bounded decision failed and recovery is in progress",
        Some("recovered") => "the latest bounded decision recovered and execution can continue",
        _ if view.latest_review_trigger.is_some() => {
            "review is in progress as part of the active session"
        }
        _ => "bounded execution is in progress",
    }
}

pub(crate) fn render_trace_execution_condition(summary: &TraceSummaryView) -> String {
    let (kind, reason) = trace_execution_condition_parts(summary);
    format!("execution_condition: {kind} - {reason}")
}

pub(crate) fn trace_execution_condition_parts(
    summary: &TraceSummaryView,
) -> (&'static str, String) {
    if let Some(reason) =
        summary.reasoning_profile.as_ref().and_then(reasoning_execution_block_reason)
    {
        return ("blocked", reason);
    }

    if let Some(delegation) = &summary.delegation {
        return match delegation.mode {
            crate::domain::session::DelegationContinuityMode::HandoffRequired
            | crate::domain::session::DelegationContinuityMode::EscalationRequired
            | crate::domain::session::DelegationContinuityMode::Stuck => {
                ("blocked", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Resolved => {
                ("waiting", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::Exhausted
            | crate::domain::session::DelegationContinuityMode::InspectOnly => {
                ("inspect_only", delegation.headline.clone())
            }
            crate::domain::session::DelegationContinuityMode::None => {
                ("waiting", delegation.headline.clone())
            }
        };
    }

    let governance_waiting =
        summary.governance_timeline.iter().any(|line| line.contains("awaiting_approval"));
    let governance_blocked = summary.governance_timeline.iter().any(|line| {
        line.contains("governance_blocked") || line.contains("governance_packet_rejected")
    });

    if governance_waiting {
        return (
            "waiting",
            "governance approval is still pending before execution can continue".to_string(),
        );
    }

    if governance_blocked {
        return (
            "blocked",
            summary.governance_next_action.clone().unwrap_or_else(|| {
                "governance blocked further execution until the blocker is resolved".to_string()
            }),
        );
    }

    match summary.terminal_status {
        TaskStatus::Failed | TaskStatus::Exhausted => {
            if let Some(reason) = trace_adaptive_exhaustion_reason(summary) {
                ("terminal", reason)
            } else {
                ("terminal", summary.terminal_reason.message.clone())
            }
        }
        TaskStatus::Planned => ("waiting", summary.terminal_reason.message.clone()),
        TaskStatus::Running => ("running", summary.terminal_reason.message.clone()),
        TaskStatus::Succeeded | TaskStatus::Aborted => {
            ("terminal", summary.terminal_reason.message.clone())
        }
    }
}

fn reasoning_execution_block_reason(reasoning_profile: &ProfileActivationRecord) -> Option<String> {
    if !reasoning_profile.status.halts_outer_workflow() {
        return None;
    }

    let detail = reasoning_profile
        .outcome
        .as_ref()
        .and_then(|outcome| outcome.next_action.clone())
        .or_else(|| {
            reasoning_profile
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.disagreement_summary.clone())
        })
        .unwrap_or_else(|| reasoning_profile.activation_reason.clone());

    Some(format!(
        "reasoning profile {} blocked stage {}: {}",
        reasoning_profile.profile_id, reasoning_profile.stage_key, detail
    ))
}

fn trace_adaptive_exhaustion_reason(summary: &TraceSummaryView) -> Option<String> {
    summary
        .adaptive_evidence
        .iter()
        .find_map(|line| line.strip_prefix("adaptive_exhaustion: ").map(str::to_string))
}

pub(crate) fn render_run_execution_condition(response: &TaskRunResponse) -> String {
    let kind = match response.terminal_status {
        TaskStatus::Planned => "waiting",
        TaskStatus::Running => {
            let message = response.terminal_reason.message.to_ascii_lowercase();
            if message.contains("approval")
                || message.contains("wait")
                || message.contains("blocked")
            {
                "waiting"
            } else {
                "running"
            }
        }
        TaskStatus::Succeeded
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "terminal",
    };

    format!("execution_condition: {kind} - {}", response.terminal_reason.message)
}

fn push_governance_projection_fields(
    projection: &mut Vec<String>,
    runtime: Option<&str>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) {
    if let Some(runtime) = runtime {
        projection.push(format!("requested_governance_runtime={runtime}"));
    }
    if let Some(risk) = risk {
        projection.push(format!("requested_governance_risk={risk}"));
    }
    if let Some(zone) = zone {
        projection.push(format!("requested_governance_zone={zone}"));
    }
    if let Some(owner) = owner {
        projection.push(format!("requested_governance_owner={owner}"));
    }
}
