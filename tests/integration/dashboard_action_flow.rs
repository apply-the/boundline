use crate::dashboard_fixture::{DashboardTestResult, require_eq};
use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace};
use boundline::adapters::dashboard_action::DashboardActionDispatcher;
use boundline::adapters::dashboard_state::DashboardStateAssembler;
use boundline::domain::dashboard::{
    DashboardActionKind, DashboardActionOutcome, DashboardActionRequest,
};

#[test]
fn dashboard_continue_action_uses_current_revision_and_normal_next_command() -> DashboardTestResult
{
    let workspace = temp_fixture_workspace("dashboard-action-flow");
    require_eq(run_boundline_in(&workspace, &["start"]).status.code(), Some(0), "start")?;
    require_eq(
        run_boundline_in(&workspace, &["capture", "--goal", "Fix action flow"]).status.code(),
        Some(0),
        "capture",
    )?;

    let snapshot = DashboardStateAssembler::for_workspace(&workspace).snapshot(true)?;
    let result =
        DashboardActionDispatcher::for_workspace(&workspace).apply(&DashboardActionRequest {
            request_id: "request-action-flow".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            action_kind: DashboardActionKind::Continue,
            target_session_id: snapshot.session.as_ref().map(|session| session.session_id.clone()),
            target_session_revision: snapshot.session_revision,
            operator_reason: None,
            requested_at: "2026-05-20T00:00:00Z".to_string(),
        })?;

    require_eq(result.outcome, DashboardActionOutcome::Applied, "outcome")?;
    require_eq(
        result.next_command.as_deref().map(|command| command.starts_with("boundline")),
        Some(true),
        "next command",
    )
}

#[test]
fn dashboard_supported_actions_return_bounded_runtime_transition_results() -> DashboardTestResult {
    let workspace = crate::dashboard_fixture::dashboard_workspace("dashboard-action-all")?;
    crate::dashboard_fixture::write_session(&workspace, "goal_captured", 9)?;
    let actions = [
        (DashboardActionKind::Confirm, None),
        (DashboardActionKind::Reject, Some("wrong direction".to_string())),
        (DashboardActionKind::Replan, Some("narrow target".to_string())),
        (DashboardActionKind::Recover, None),
        (DashboardActionKind::Launch, None),
        (DashboardActionKind::Continue, None),
    ];

    for (action_kind, reason) in actions {
        let result = DashboardActionDispatcher::for_workspace(&workspace).apply(
            &DashboardActionRequest {
                request_id: format!("request-{action_kind:?}"),
                workspace_ref: workspace.to_string_lossy().into_owned(),
                action_kind,
                target_session_id: Some("session-9".to_string()),
                target_session_revision: Some(9),
                operator_reason: reason,
                requested_at: "2026-05-20T00:00:00Z".to_string(),
            },
        )?;
        require_eq(result.outcome, DashboardActionOutcome::Applied, "outcome")?;
    }
    Ok(())
}
