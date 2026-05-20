use crate::dashboard_fixture::{DashboardTestResult, require_eq};
use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace};
use boundline::adapters::dashboard_action::DashboardActionDispatcher;
use boundline::adapters::dashboard_state::DashboardStateAssembler;
use boundline::domain::dashboard::{
    DashboardActionKind, DashboardActionOutcome, DashboardActionRequest, DashboardRefusalReason,
};

#[test]
fn dashboard_refuses_stale_action_after_external_state_change() -> DashboardTestResult {
    let workspace = temp_fixture_workspace("dashboard-action-refusal");
    require_eq(run_boundline_in(&workspace, &["start"]).status.code(), Some(0), "start")?;
    require_eq(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix stale action"]).status.code(),
        Some(0),
        "capture",
    )?;

    let stale_snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    require_eq(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0), "plan")?;

    let result =
        DashboardActionDispatcher::for_workspace(&workspace).apply(&DashboardActionRequest {
            request_id: "request-stale".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            action_kind: DashboardActionKind::Continue,
            target_session_id: stale_snapshot
                .session
                .as_ref()
                .map(|session| session.session_id.clone()),
            target_session_revision: stale_snapshot.session_revision,
            operator_reason: None,
            requested_at: "2026-05-20T00:00:00Z".to_string(),
        })?;

    require_eq(result.outcome, DashboardActionOutcome::Refused, "outcome")?;
    require_eq(result.refusal_reason, Some(DashboardRefusalReason::StaleSessionRevision), "refusal")
}
