use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalCondition {
    GoalSatisfied,
    TaskNotCredible,
    StepLimitExceeded,
    RetryBudgetExhausted,
    ReplanBudgetExhausted,
    UnrecoverableError,
    NoCredibleNextStep,
}

impl TerminalCondition {
    pub const fn all() -> [Self; 7] {
        [
            Self::GoalSatisfied,
            Self::TaskNotCredible,
            Self::StepLimitExceeded,
            Self::RetryBudgetExhausted,
            Self::ReplanBudgetExhausted,
            Self::UnrecoverableError,
            Self::NoCredibleNextStep,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunLimits {
    #[serde(default = "default_max_steps")]
    pub max_steps: usize,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default = "default_max_replans")]
    pub max_replans: usize,
    #[serde(default = "default_terminal_precedence")]
    pub terminal_precedence: Vec<TerminalCondition>,
}

impl RunLimits {
    pub fn validate(&self) -> Result<(), RunLimitsError> {
        if self.max_steps == 0 {
            return Err(RunLimitsError::InvalidMaxSteps);
        }

        let mut seen = HashSet::new();
        for condition in &self.terminal_precedence {
            if !seen.insert(*condition) {
                return Err(RunLimitsError::DuplicateTerminalCondition(*condition));
            }
        }

        for condition in TerminalCondition::all() {
            if !seen.contains(&condition) {
                return Err(RunLimitsError::MissingTerminalCondition(condition));
            }
        }

        Ok(())
    }
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_steps: default_max_steps(),
            max_retries: default_max_retries(),
            max_replans: default_max_replans(),
            terminal_precedence: default_terminal_precedence(),
        }
    }
}

fn default_max_steps() -> usize {
    100
}

fn default_max_retries() -> usize {
    3
}

fn default_max_replans() -> usize {
    2
}

fn default_terminal_precedence() -> Vec<TerminalCondition> {
    vec![
        TerminalCondition::GoalSatisfied,
        TerminalCondition::UnrecoverableError,
        TerminalCondition::TaskNotCredible,
        TerminalCondition::RetryBudgetExhausted,
        TerminalCondition::ReplanBudgetExhausted,
        TerminalCondition::StepLimitExceeded,
        TerminalCondition::NoCredibleNextStep,
    ]
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RunLimitsError {
    #[error("max_steps must be greater than zero")]
    InvalidMaxSteps,
    #[error("terminal precedence duplicates condition {0:?}")]
    DuplicateTerminalCondition(TerminalCondition),
    #[error("terminal precedence is missing condition {0:?}")]
    MissingTerminalCondition(TerminalCondition),
}
