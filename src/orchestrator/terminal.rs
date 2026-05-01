use serde_json::Value;
use serde_json::json;

use crate::domain::limits::TerminalCondition;
use crate::domain::task::{TaskStatus, TerminalReason};
use crate::orchestrator::planner::PlanningError;

pub fn select_terminal_condition(
    precedence: &[TerminalCondition],
    candidates: &[TerminalCondition],
) -> Option<TerminalCondition> {
    precedence.iter().find(|condition| candidates.contains(condition)).copied()
}

pub fn task_status_for_condition(condition: TerminalCondition) -> TaskStatus {
    match condition {
        TerminalCondition::GoalSatisfied => TaskStatus::Succeeded,
        TerminalCondition::StepLimitExceeded
        | TerminalCondition::RetryBudgetExhausted
        | TerminalCondition::ReplanBudgetExhausted => TaskStatus::Exhausted,
        TerminalCondition::TaskNotCredible
        | TerminalCondition::UnrecoverableError
        | TerminalCondition::NoCredibleNextStep => TaskStatus::Failed,
    }
}

pub fn build_terminal_reason(
    condition: TerminalCondition,
    message: impl Into<String>,
    details: Option<Value>,
) -> TerminalReason {
    TerminalReason::new(condition, message, details)
}

pub fn build_planning_failure_reason(step_id: &str, error: &PlanningError) -> TerminalReason {
    match error {
        PlanningError::ReplanUnavailable(message) => build_terminal_reason(
            TerminalCondition::NoCredibleNextStep,
            message.clone(),
            Some(json!({
                "step_id": step_id,
                "error": error.to_string(),
            })),
        ),
        PlanningError::InvalidPlan(message) => build_terminal_reason(
            TerminalCondition::TaskNotCredible,
            "replacement plan did not provide a credible next step",
            Some(json!({
                "step_id": step_id,
                "error": message,
            })),
        ),
        PlanningError::Internal(message) => build_terminal_reason(
            TerminalCondition::TaskNotCredible,
            "planner could not produce a credible replacement plan",
            Some(json!({
                "step_id": step_id,
                "error": message,
            })),
        ),
    }
}
