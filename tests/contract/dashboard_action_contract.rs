use std::fs;

use serde_json::json;

use crate::dashboard_fixture::{
    DashboardTestResult, dashboard_workspace, require_eq, write_session,
};
use boundline::adapters::dashboard_action::DashboardActionDispatcher;
use boundline::domain::dashboard::{
    DashboardActionKind, DashboardActionOutcome, DashboardActionRequest, DashboardRefusalReason,
};
use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::trace::{ExecutionTrace, TraceEvent, TraceEventType};

fn request(action_kind: DashboardActionKind, revision: Option<u64>) -> DashboardActionRequest {
    DashboardActionRequest {
        request_id: "request-1".to_string(),
        workspace_ref: "/workspace".to_string(),
        action_kind,
        target_session_id: Some("session-1".to_string()),
        target_session_revision: revision,
        operator_reason: None,
        requested_at: "2026-05-20T00:00:00Z".to_string(),
    }
}

#[test]
fn inspect_only_action_applies_without_session_revision() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-inspect")?;
    write_session(&workspace, "goal_captured", 7)?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::InspectOnly, None))?;

    require_eq(result.outcome, DashboardActionOutcome::Applied, "outcome")?;
    require_eq(result.state_transition.as_deref(), Some("focus_changed"), "transition")
}

#[test]
fn mutating_action_refuses_stale_session_revision() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-stale")?;
    write_session(&workspace, "goal_captured", 7)?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::Continue, Some(6)))?;

    require_eq(result.outcome, DashboardActionOutcome::Refused, "outcome")?;
    require_eq(result.refusal_reason, Some(DashboardRefusalReason::StaleSessionRevision), "refusal")
}

#[test]
fn rejection_requires_operator_reason() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-reject")?;
    write_session(&workspace, "goal_captured", 7)?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::Reject, Some(7)))?;

    require_eq(
        result.refusal_reason,
        Some(DashboardRefusalReason::MissingRequiredContext),
        "refusal",
    )
}

#[test]
fn replan_requires_operator_reason() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-replan")?;
    write_session(&workspace, "goal_captured", 7)?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::Replan, Some(7)))?;

    require_eq(
        result.refusal_reason,
        Some(DashboardRefusalReason::MissingRequiredContext),
        "refusal",
    )
}

#[test]
fn mutating_action_refuses_when_dashboard_snapshot_is_degraded() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-degraded")?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::Continue, Some(1)))?;

    require_eq(result.outcome, DashboardActionOutcome::Refused, "outcome")?;
    require_eq(result.refusal_reason, Some(DashboardRefusalReason::DashboardDegraded), "refusal")?;
    require_eq(
        result.next_command.as_deref(),
        Some(&format!("boundline status --workspace {}", workspace.display())),
        "fallback next command",
    )
}

#[test]
fn mutating_action_carries_trace_refs_from_current_snapshot() -> DashboardTestResult {
    let workspace = dashboard_workspace("dashboard-action-trace-refs")?;
    let session_path = write_session(&workspace, "goal_captured", 7)?;
    let trace_path = workspace.join(".boundline").join("traces").join("dashboard-action.json");

    let trace = ExecutionTrace {
        task_id: "task-dashboard".to_string(),
        session_id: "session-7".to_string(),
        goal: "Fix the failing checkout flow".to_string(),
        started_at: 1,
        ended_at: None,
        terminal_status: None,
        terminal_reason: None,
        events: vec![TraceEvent {
            event_id: "event-1".to_string(),
            event_type: TraceEventType::DecisionVerified,
            step_id: Some("verify-dashboard".to_string()),
            plan_revision: 1,
            payload: json!({ "status": "ok" }),
            recorded_at: 1,
        }],
        trace_location: Some(trace_path.to_string_lossy().into_owned()),
    };
    fs::write(&trace_path, serde_json::to_vec_pretty(&trace)?)?;

    let mut session: ActiveSessionRecord = serde_json::from_slice(&fs::read(&session_path)?)?;
    session.latest_trace_ref = Some(trace_path.to_string_lossy().into_owned());
    fs::write(&session_path, serde_json::to_vec_pretty(&session)?)?;

    let result = DashboardActionDispatcher::for_workspace(&workspace)
        .apply(&request(DashboardActionKind::Continue, Some(7)))?;

    require_eq(result.outcome, DashboardActionOutcome::Applied, "outcome")?;
    require_eq(result.trace_refs.len(), 1, "trace ref count")?;
    require_eq(
        result.trace_refs.first().map(String::as_str),
        Some(trace_path.to_string_lossy().as_ref()),
        "trace ref",
    )
}
