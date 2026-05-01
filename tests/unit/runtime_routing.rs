use serde_json::json;

use synod::domain::flow_policy::FlowPolicy;
use synod::domain::goal_plan::{GoalPlan, GoalPlanFlowMode, InferredFlow, PlannedTask};
use synod::domain::limits::RunLimits;
use synod::domain::plan::Plan;
use synod::domain::session::{ActiveSessionRecord, RoutingMode, RoutingSource, SessionStatus};
use synod::domain::step::Step;
use synod::domain::task::{Task, TaskRunRequest};
use synod::orchestrator::session_runtime::SessionRuntime;

fn build_task(workspace_ref: &str) -> Task {
    let request = TaskRunRequest {
        goal: "Deliver runtime refoundation".to_string(),
        input: json!({"ticket": "RUNTIME-15"}),
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
        }],
    )
    .unwrap();
    goal_plan.flow = Some(InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "goal contains keyword 'fix'".to_string(),
        confirmed,
    });
    goal_plan.confirm().unwrap();
    goal_plan
}

#[test]
fn goal_plan_flow_state_reports_proposed_and_confirmed_modes() {
    let proposed = build_goal_plan(false);
    assert_eq!(proposed.flow_state().mode, GoalPlanFlowMode::Proposed);
    assert_eq!(proposed.flow_state().flow_name.as_deref(), Some("bug-fix"));

    let confirmed = build_goal_plan(true);
    assert_eq!(confirmed.flow_state().mode, GoalPlanFlowMode::Confirmed);
    assert_eq!(confirmed.flow_state().flow_name.as_deref(), Some("bug-fix"));

    let unconstrained = GoalPlan::new(
        "Implement a workspace summary",
        vec![PlannedTask {
            task_id: "planned-task-2".to_string(),
            description: "Implement the summary".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("summary added".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    assert_eq!(unconstrained.flow_state().mode, GoalPlanFlowMode::Absent);
}

#[test]
fn flow_policy_helpers_report_stage_id_and_progress() {
    let mut policy = FlowPolicy::from_builtin("bug-fix").unwrap();
    assert_eq!(policy.current_stage_id(), Some("investigate"));
    assert_eq!(policy.stage_progress(), Some((1, 3)));

    policy.advance_stage().unwrap();
    assert_eq!(policy.current_stage_id(), Some("implement"));
    assert_eq!(policy.stage_progress(), Some((2, 3)));
}

#[test]
fn session_runtime_resolve_routing_outcome_blocks_pending_flow_confirmation() {
    let workspace = std::env::temp_dir().join("synod-runtime-routing-pending");
    std::fs::create_dir_all(&workspace).unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let record = ActiveSessionRecord {
        session_id: "session-native".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
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
    };

    let outcome = runtime.resolve_routing_outcome(&record).unwrap();
    assert_eq!(outcome.mode, RoutingMode::Blocked);
    assert_eq!(outcome.source, RoutingSource::GoalPlan);
    assert!(outcome.reason.contains("flow confirmation"));
}

#[test]
fn session_runtime_resolve_routing_outcome_uses_compatibility_when_only_task_exists() {
    let workspace = std::env::temp_dir().join("synod-runtime-routing-compat");
    std::fs::create_dir_all(&workspace).unwrap();
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let record = ActiveSessionRecord {
        session_id: "session-fixture".to_string(),
        workspace_ref: workspace_ref.clone(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(&workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Running,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 20,
    };

    let outcome = runtime.resolve_routing_outcome(&record).unwrap();
    assert_eq!(outcome.mode, RoutingMode::Compatibility);
    assert_eq!(outcome.source, RoutingSource::ExecutionProfile);
    assert!(outcome.reason.contains("compatibility"));
}
