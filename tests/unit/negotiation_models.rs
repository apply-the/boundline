use crate::workspace_fixture::temp_fixture_workspace;
use synod::domain::negotiation::{
    NegotiatedDeliveryPacket, NegotiationConstraint, NegotiationConstraintKind,
    NegotiationConstraintSource, NegotiationConstraintState, NegotiationResolutionState,
};
use synod::domain::session::{ActiveSessionRecord, SessionStatus};
use synod::fixture::build_task_request;
use synod::orchestrator::session_runtime::SessionRuntime;
use uuid::Uuid;

#[test]
fn capture_goal_derives_a_credible_negotiation_packet_from_direct_goal() {
    let workspace = temp_fixture_workspace("synod-negotiation-goal-only");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-negotiation".to_string(),
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
        created_at: 10,
        updated_at: 10,
    };

    runtime.capture_goal(&mut session, "Fix the failing add test").unwrap();

    let packet =
        session.negotiation_packet.as_ref().expect("capture should persist a negotiated packet");
    assert_eq!(packet.goal_summary, "Fix the failing add test");
    assert_eq!(packet.resolution_state, NegotiationResolutionState::Credible);
    assert!(!packet.acceptance_boundary.required_outcomes.is_empty());
    assert!(!packet.constraints.is_empty());
    session.validate().unwrap();
}

#[test]
fn plan_task_respects_pending_negotiation_even_without_authored_brief() {
    let workspace = temp_fixture_workspace("synod-negotiation-plan-gate");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut packet = NegotiatedDeliveryPacket::from_goal(
        "session-negotiation",
        workspace.to_string_lossy().as_ref(),
        "Improve the platform docs and fix whatever tests are broken",
    );
    packet.resolution_state = NegotiationResolutionState::PendingClarification;
    packet.clarification_headline =
        Some("clarification required: narrow the request to one bounded outcome".to_string());
    packet.constraints.push(NegotiationConstraint {
        constraint_id: Uuid::new_v4().to_string(),
        kind: NegotiationConstraintKind::Scope,
        summary: "narrow the request to one bounded outcome before planning".to_string(),
        source: NegotiationConstraintSource::Brief,
        state: NegotiationConstraintState::Conflicting,
        blocks_planning: true,
    });

    let mut session = ActiveSessionRecord {
        session_id: "session-negotiation".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Improve the platform docs and fix whatever tests are broken".to_string()),
        authored_brief: None,
        negotiation_packet: Some(packet),
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
    };

    let error = runtime.plan_task(&mut session, None, false).unwrap_err();

    assert!(error.to_string().contains("clarification required"), "{error}");
}

#[test]
fn plan_task_projects_negotiation_summary_into_goal_plan() {
    let workspace = temp_fixture_workspace("synod-negotiation-plan-projection");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-negotiation".to_string(),
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
        created_at: 10,
        updated_at: 10,
    };

    runtime.capture_goal(&mut session, "Fix the failing add test").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let goal_plan = session.goal_plan.as_ref().expect("goal plan should exist");
    assert_eq!(goal_plan.negotiation_goal_summary.as_deref(), Some("Fix the failing add test"));
    assert_eq!(goal_plan.negotiation_resolution.as_deref(), Some("credible"));
    assert_eq!(
        goal_plan.negotiation_acceptance_boundary.as_deref(),
        Some("deliver the bounded outcome: Fix the failing add test")
    );
}

#[test]
fn build_task_request_serializes_negotiation_projection_into_input() {
    let workspace = temp_fixture_workspace("synod-negotiation-task-request");
    let packet = NegotiatedDeliveryPacket::from_goal(
        "session-negotiation",
        workspace.to_string_lossy().as_ref(),
        "Fix the failing add test",
    );

    let request = build_task_request(
        &workspace,
        "Fix the failing add test",
        "session-negotiation",
        None,
        Some(&packet),
    )
    .unwrap();

    let input = request.input.as_object().expect("task input should be an object");
    assert_eq!(
        input.get("negotiation_goal_summary").and_then(|value| value.as_str()),
        Some("Fix the failing add test")
    );
    assert_eq!(
        input.get("negotiation_resolution").and_then(|value| value.as_str()),
        Some("credible")
    );
    assert_eq!(
        input.get("negotiation_acceptance_boundary").and_then(|value| value.as_str()),
        Some("deliver the bounded outcome: Fix the failing add test")
    );
}
