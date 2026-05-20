use serde_json::Value;

use crate::dashboard_fixture::{DashboardTestResult, require, require_contains, require_eq};
use boundline::domain::dashboard::{
    DashboardActionKind, DashboardActionOption, DashboardAuthority, DashboardBrandMark,
    DashboardColorProfile, DashboardExpectedResult, DashboardPanels, DashboardSessionView,
    DashboardSnapshot, ExecutionCondition,
};

fn active_snapshot() -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: "snapshot-1".to_string(),
        workspace_ref: "/workspace".to_string(),
        captured_at: "2026-05-20T00:00:00Z".to_string(),
        authority: DashboardAuthority::SessionNative,
        session_revision: Some(3),
        session: Some(DashboardSessionView {
            session_id: "session-1".to_string(),
            goal: "Fix the failing checkout flow".to_string(),
            route_kind: "native_goal_plan".to_string(),
            route_owner: "runtime".to_string(),
            active_flow: Some("bug-fix".to_string()),
            flow_state: Some("confirmed".to_string()),
            goal_plan_state: Some("confirmed".to_string()),
            goal_plan_revision: Some(1),
            current_stage: Some("verify".to_string()),
            current_step_id: Some("run-tests".to_string()),
            current_step_index: Some(2),
            execution_condition: ExecutionCondition::Ready,
            latest_status: "planned".to_string(),
            next_action_label: "Continue bounded execution".to_string(),
            next_command: "boundline run".to_string(),
            blocking_reason: None,
            compatibility_context: None,
        }),
        timeline: Vec::new(),
        panels: DashboardPanels::empty(),
        actions: vec![DashboardActionOption {
            action_kind: DashboardActionKind::Continue,
            label: "Run".to_string(),
            description: "Continue bounded execution".to_string(),
            requires_reason: false,
            requires_confirmation: true,
            target_session_revision: Some(3),
            expected_result: DashboardExpectedResult::RunningOrTerminal,
            disabled_reason: None,
        }],
        degraded_state: None,
        branding: DashboardBrandMark {
            wordmark_lines: vec!["boundline".to_string()],
            color_profile: DashboardColorProfile::Color,
            min_width: 20,
            fallback_label: "boundline".to_string(),
        },
    }
}

#[test]
fn active_snapshot_serializes_the_stable_contract_shape() -> DashboardTestResult {
    let snapshot = active_snapshot();
    snapshot.validate()?;

    let json = serde_json::to_value(&snapshot)?;
    require_eq(json["authority"].as_str(), Some("session_native"), "authority")?;
    require_eq(json["session"]["route_kind"].as_str(), Some("native_goal_plan"), "route_kind")?;
    require_eq(
        json["actions"][0]["target_session_revision"].as_u64(),
        Some(3),
        "target_session_revision",
    )?;
    require_eq(json["branding"]["wordmark_lines"][0].as_str(), Some("boundline"), "wordmark")?;
    Ok(())
}

#[test]
fn invalid_snapshot_without_session_or_degraded_state_fails_closed() -> DashboardTestResult {
    let mut snapshot = active_snapshot();
    snapshot.session = None;
    let error = snapshot.validate().map(|_| Value::Null).err();
    require(error.is_some(), "snapshot without session or degraded state must be invalid")
}

#[test]
fn mutating_actions_require_target_session_revision() -> DashboardTestResult {
    let mut snapshot = active_snapshot();
    if let Some(action) = snapshot.actions.first_mut() {
        action.target_session_revision = None;
    }
    let error = snapshot.validate().map(|_| Value::Null).err();
    require(error.is_some(), "mutating dashboard action must carry target revision")
}

#[test]
fn snapshot_validate_requires_snapshot_and_workspace_refs() -> DashboardTestResult {
    let mut missing_snapshot_id = active_snapshot();
    missing_snapshot_id.snapshot_id.clear();
    let error =
        missing_snapshot_id.validate().err().ok_or("missing snapshot id must fail validation")?;
    require_contains(&error.to_string(), "snapshot_id", "snapshot id validation")?;

    let mut missing_workspace_ref = active_snapshot();
    missing_workspace_ref.workspace_ref.clear();
    let error = missing_workspace_ref
        .validate()
        .err()
        .ok_or("missing workspace ref must fail validation")?;
    require_contains(&error.to_string(), "workspace_ref", "workspace ref validation")
}

#[test]
fn session_validate_requires_identity_route_and_blocking_fields() -> DashboardTestResult {
    let mut missing_session_id = active_snapshot();
    if let Some(session) = missing_session_id.session.as_mut() {
        session.session_id.clear();
    }
    let error =
        missing_session_id.validate().err().ok_or("missing session id must fail validation")?;
    require_contains(&error.to_string(), "session_id", "session id validation")?;

    let mut missing_route_kind = active_snapshot();
    if let Some(session) = missing_route_kind.session.as_mut() {
        session.route_kind.clear();
    }
    let error =
        missing_route_kind.validate().err().ok_or("missing route kind must fail validation")?;
    require_contains(&error.to_string(), "route_kind", "route kind validation")?;

    let mut missing_next_command = active_snapshot();
    if let Some(session) = missing_next_command.session.as_mut() {
        session.next_command.clear();
    }
    let error =
        missing_next_command.validate().err().ok_or("missing next command must fail validation")?;
    require_contains(&error.to_string(), "next_command", "next command validation")?;

    let mut missing_blocking_reason = active_snapshot();
    if let Some(session) = missing_blocking_reason.session.as_mut() {
        session.execution_condition = ExecutionCondition::Waiting;
        session.blocking_reason = None;
    }
    let error = missing_blocking_reason
        .validate()
        .err()
        .ok_or("waiting session without blocking reason must fail validation")?;
    require_contains(&error.to_string(), "blocking_reason", "blocking reason validation")
}

#[test]
fn branding_validate_requires_wordmark_lines_and_fallback_label() -> DashboardTestResult {
    let mut missing_wordmark_lines = active_snapshot();
    missing_wordmark_lines.branding.wordmark_lines.clear();
    let error = missing_wordmark_lines
        .validate()
        .err()
        .ok_or("missing wordmark lines must fail validation")?;
    require_contains(&error.to_string(), "wordmark", "wordmark lines validation")?;

    let mut missing_fallback_label = active_snapshot();
    missing_fallback_label.branding.wordmark_lines = vec!["boundline".to_string()];
    missing_fallback_label.branding.fallback_label.clear();
    let error = missing_fallback_label
        .validate()
        .err()
        .ok_or("missing fallback label must fail validation")?;
    require_contains(&error.to_string(), "wordmark", "fallback label validation")
}

#[test]
fn dashboard_helper_enums_preserve_mutation_and_blocking_rules() -> DashboardTestResult {
    require_eq(DashboardActionKind::InspectOnly.mutates_state(), false, "inspect-only mutation")?;
    require_eq(DashboardActionKind::Confirm.mutates_state(), true, "confirm mutation")?;

    for condition in [
        ExecutionCondition::Waiting,
        ExecutionCondition::Blocked,
        ExecutionCondition::Failed,
        ExecutionCondition::Exhausted,
        ExecutionCondition::Invalid,
        ExecutionCondition::Degraded,
    ] {
        require(
            condition.requires_blocking_reason(),
            "non-ready execution conditions must require a blocking reason",
        )?;
    }

    for condition in [ExecutionCondition::Ready, ExecutionCondition::Complete] {
        require(
            !condition.requires_blocking_reason(),
            "ready and complete conditions must not require a blocking reason",
        )?;
    }

    Ok(())
}
