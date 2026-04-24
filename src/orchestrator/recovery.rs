use serde_json::json;

use crate::domain::limits::TerminalCondition;
use crate::domain::step::{ExecutionStatus, Step, StepExecutionResult};
use crate::domain::task::{Task, TerminalReason};
use crate::orchestrator::terminal::build_terminal_reason;

#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryDecision {
    Continue,
    Retry { reason: String },
    Replan { reason: String },
    Terminate(TerminalReason),
}

pub fn decide_recovery(task: &Task, step: &Step, result: &StepExecutionResult) -> RecoveryDecision {
    if matches!(result.status, ExecutionStatus::Succeeded) {
        return RecoveryDecision::Continue;
    }

    match result.recoverability {
        crate::domain::step::Recoverability::Retryable => {
            if task.retry_count < task.limits.max_retries {
                RecoveryDecision::Retry {
                    reason: format!("retrying step {} within remaining retry budget", step.id),
                }
            } else {
                RecoveryDecision::Terminate(build_terminal_reason(
                    TerminalCondition::RetryBudgetExhausted,
                    format!("retry budget exhausted while executing step {}", step.id),
                    Some(json!({
                        "step_id": step.id,
                        "retry_count": task.retry_count,
                        "max_retries": task.limits.max_retries,
                    })),
                ))
            }
        }
        crate::domain::step::Recoverability::ReplanRequired => {
            if task.replan_count < task.limits.max_replans {
                RecoveryDecision::Replan {
                    reason: format!(
                        "replanning after step {} invalidated the current path",
                        step.id
                    ),
                }
            } else {
                RecoveryDecision::Terminate(build_terminal_reason(
                    TerminalCondition::ReplanBudgetExhausted,
                    format!("replan budget exhausted after step {} failed", step.id),
                    Some(json!({
                        "step_id": step.id,
                        "replan_count": task.replan_count,
                        "max_replans": task.limits.max_replans,
                    })),
                ))
            }
        }
        crate::domain::step::Recoverability::Terminal => {
            RecoveryDecision::Terminate(build_terminal_reason(
                TerminalCondition::UnrecoverableError,
                format!("step {} failed with an unrecoverable error", step.id),
                result.error.as_ref().map(|error| serde_json::to_value(error).unwrap_or_default()),
            ))
        }
    }
}
