use boundline::domain::goal_plan::{GoalPlan, GoalPlanFlowMode, InferredFlow, PlannedTask};

fn build_plan() -> GoalPlan {
    GoalPlan::new(
        "fix the failing add test",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Fix the broken arithmetic path".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )
    .unwrap()
}

#[test]
fn goal_plan_can_mark_flow_as_skipped() {
    let mut plan = build_plan().with_flow(InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "goal contains keyword 'fix'".to_string(),
        confirmed: false,
    });

    plan.mark_flow_skipped();

    let flow_state = plan.flow_state();
    assert_eq!(flow_state.mode, GoalPlanFlowMode::Skipped);
    assert!(flow_state.flow_name.is_none());
    assert!(flow_state.confidence_reason.is_none());
}

#[test]
fn goal_plan_reports_absent_when_no_flow_is_inferred_or_selected() {
    let plan = build_plan();
    assert_eq!(plan.flow_state().mode, GoalPlanFlowMode::Absent);
}
