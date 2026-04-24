use serde_json::Value;

use crate::domain::limits::TerminalCondition;
use crate::domain::task::{TaskStatus, TerminalReason};

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
