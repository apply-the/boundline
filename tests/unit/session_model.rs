use boundline::domain::goal_plan::{GoalPlan, InferredFlow, PlannedTask};
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingSource, SessionStatus, execution_path_text,
    routing_outcome,
};
use boundline::domain::step::Step;
use boundline::domain::task::{Task, TaskRunRequest};
use serde_json::json;

fn build_task(workspace_ref: &str) -> Task {
    let request = TaskRunRequest {
        goal: "Deliver a session-backed CLI".to_string(),
        input: json!({"ticket": "SESSION-1"}),
        session_id: "session-1".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    };
    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "bootstrap"})).unwrap()]).unwrap();

    Task::new("task-1", &request, plan).unwrap()
}

fn build_goal_plan(confirmed: bool) -> GoalPlan {
    let mut goal_plan = GoalPlan::new(
        "Fix the failing add test",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Fix the broken arithmetic path".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )
    .unwrap();
    goal_plan.flow = Some(InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "goal contains keyword 'fix'".to_string(),
        confirmed,
    });
    if confirmed {
        goal_plan.confirm().unwrap();
    }
    goal_plan
}

#[test]
fn execution_path_uses_native_goal_plan_for_proposed_plan() {
    let record = ActiveSessionRecord {
        session_id: "session-native".to_string(),
        workspace_ref: "/tmp/boundline-session-model".to_string(),
        goal: Some("Fix the failing add test".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(build_goal_plan(false)),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    assert_eq!(execution_path_text(&record).as_deref(), Some("native_goal_plan"));
}

#[test]
fn execution_path_uses_fixture_compatibility_when_only_task_state_exists() {
    let workspace_ref = "/tmp/boundline-session-model";
    let record = ActiveSessionRecord {
        session_id: "session-fixture".to_string(),
        workspace_ref: workspace_ref.to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Running,
        latest_terminal_reason: None,
        latest_trace_ref: Some(format!("{workspace_ref}/.boundline/traces/task-1.json")),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    assert_eq!(execution_path_text(&record).as_deref(), Some("fixture_compatibility"));
}

#[test]
fn execution_path_marks_goal_captured_sessions_as_pending_plan() {
    let record = ActiveSessionRecord {
        session_id: "session-captured".to_string(),
        workspace_ref: "/tmp/boundline-session-model".to_string(),
        goal: Some("Implement the workspace summary".to_string()),
        authored_brief: None,
        negotiation_packet: None,
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
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    assert_eq!(execution_path_text(&record).as_deref(), Some("native_session_pending_plan"));
}

#[test]
fn routing_outcome_routes_native_when_plan_confirmation_is_pending() {
    let record = ActiveSessionRecord {
        session_id: "session-native".to_string(),
        workspace_ref: "/tmp/boundline-session-model".to_string(),
        goal: Some("Fix the failing add test".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(build_goal_plan(false)),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let outcome = routing_outcome(&record);
    assert_eq!(outcome.mode, RoutingMode::Native);
    assert_eq!(outcome.source, RoutingSource::GoalPlan);
    assert!(outcome.reason.contains("native execution"));
}

#[test]
fn routing_outcome_prefers_native_goal_plan_when_plan_is_confirmed() {
    let record = ActiveSessionRecord {
        session_id: "session-native".to_string(),
        workspace_ref: "/tmp/boundline-session-model".to_string(),
        goal: Some("Fix the failing add test".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(build_goal_plan(true)),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    };

    let outcome = routing_outcome(&record);
    assert_eq!(outcome.mode, RoutingMode::Native);
    assert_eq!(outcome.source, RoutingSource::GoalPlan);
    assert!(outcome.reason.contains("goal plan"));
}
