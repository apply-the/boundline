use boundline::domain::limits::{RunLimits, TerminalCondition};
use boundline::domain::task::TaskStatus;
use boundline::orchestrator::planner::PlanningError;
use boundline::orchestrator::terminal::{
    build_planning_failure_reason, build_terminal_reason, select_terminal_condition,
    task_status_for_condition,
};
use serde_json::json;

#[test]
fn terminal_precedence_prefers_the_earliest_configured_condition() {
    let limits = RunLimits::default();
    let selected = select_terminal_condition(
        &limits.terminal_precedence,
        &[TerminalCondition::StepLimitExceeded, TerminalCondition::GoalSatisfied],
    );

    assert_eq!(selected, Some(TerminalCondition::GoalSatisfied));
}

#[test]
fn terminal_precedence_returns_none_when_no_candidate_matches() {
    let limits = RunLimits::default();

    assert_eq!(select_terminal_condition(&limits.terminal_precedence, &[]), None);
}

#[test]
fn terminal_condition_mapping_covers_success_failed_and_exhausted_states() {
    assert_eq!(task_status_for_condition(TerminalCondition::GoalSatisfied), TaskStatus::Succeeded);
    assert_eq!(
        task_status_for_condition(TerminalCondition::StepLimitExceeded),
        TaskStatus::Exhausted
    );
    assert_eq!(
        task_status_for_condition(TerminalCondition::RetryBudgetExhausted),
        TaskStatus::Exhausted
    );
    assert_eq!(
        task_status_for_condition(TerminalCondition::ReplanBudgetExhausted),
        TaskStatus::Exhausted
    );
    assert_eq!(task_status_for_condition(TerminalCondition::TaskNotCredible), TaskStatus::Failed);
    assert_eq!(
        task_status_for_condition(TerminalCondition::UnrecoverableError),
        TaskStatus::Failed
    );
    assert_eq!(
        task_status_for_condition(TerminalCondition::NoCredibleNextStep),
        TaskStatus::Failed
    );
}

#[test]
fn terminal_reason_builders_preserve_details_and_map_planner_failures() {
    let reason = build_terminal_reason(
        TerminalCondition::TaskNotCredible,
        "planner failed",
        Some(json!({"step_id": "verify"})),
    );
    assert_eq!(reason.condition, TerminalCondition::TaskNotCredible);
    assert_eq!(reason.message, "planner failed");
    assert_eq!(reason.details, Some(json!({"step_id": "verify"})));

    let replan = build_planning_failure_reason(
        "verify",
        &PlanningError::ReplanUnavailable("no bounded candidate remains".to_string()),
    );
    assert_eq!(replan.condition, TerminalCondition::NoCredibleNextStep);
    assert_eq!(replan.message, "no bounded candidate remains");
    assert_eq!(replan.details.as_ref().unwrap()["step_id"], "verify");
    assert!(
        replan.details.as_ref().unwrap()["error"]
            .as_str()
            .is_some_and(|error| error.contains("planner cannot provide a replan"))
    );

    let invalid = build_planning_failure_reason(
        "verify",
        &PlanningError::InvalidPlan("step list was empty".to_string()),
    );
    assert_eq!(invalid.condition, TerminalCondition::TaskNotCredible);
    assert_eq!(invalid.message, "replacement plan did not provide a credible next step");
    assert_eq!(invalid.details.as_ref().unwrap()["error"], "step list was empty");

    let internal = build_planning_failure_reason(
        "verify",
        &PlanningError::Internal("planner queue poisoned".to_string()),
    );
    assert_eq!(internal.condition, TerminalCondition::TaskNotCredible);
    assert_eq!(internal.message, "planner could not produce a credible replacement plan");
    assert_eq!(internal.details.as_ref().unwrap()["error"], "planner queue poisoned");
}
