use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::limits::RunLimits;
use crate::domain::review::{ReviewProfile, ReviewProfileError};
use crate::domain::step::Recoverability;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl ExecutionCommand {
    pub fn validate(&self) -> Result<(), ExecutionProfileError> {
        if self.program.trim().is_empty() {
            return Err(ExecutionProfileError::MissingValidationProgram);
        }

        Ok(())
    }

    pub fn rendered(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionFailureMode {
    Retry,
    Replan,
    Terminal,
}

impl ExecutionFailureMode {
    pub const fn recoverability(self) -> Recoverability {
        match self {
            Self::Retry => Recoverability::Retryable,
            Self::Replan => Recoverability::ReplanRequired,
            Self::Terminal => Recoverability::Terminal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceChange {
    pub path: String,
    pub find: String,
    pub replace: String,
}

impl WorkspaceChange {
    pub fn validate(&self) -> Result<(), ExecutionProfileError> {
        if self.path.trim().is_empty() {
            return Err(ExecutionProfileError::MissingChangePath);
        }

        if !is_workspace_relative(&self.path) {
            return Err(ExecutionProfileError::InvalidWorkspacePath(self.path.clone()));
        }

        if self.find.is_empty() {
            return Err(ExecutionProfileError::MissingFindPattern(self.path.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionAttemptDefinition {
    pub attempt_id: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default = "default_failure_mode")]
    pub failure_mode: ExecutionFailureMode,
    #[serde(default)]
    pub changes: Vec<WorkspaceChange>,
}

impl ExecutionAttemptDefinition {
    pub fn validate(&self) -> Result<(), ExecutionProfileError> {
        if self.attempt_id.trim().is_empty() {
            return Err(ExecutionProfileError::MissingAttemptId);
        }

        if self.changes.is_empty() {
            return Err(ExecutionProfileError::MissingAttemptChanges(self.attempt_id.clone()));
        }

        for change in &self.changes {
            change.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdaptiveChangeKind {
    ArithmeticSwap,
    ComparisonFlip,
    BooleanFlip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdaptiveExecutionProfile {
    #[serde(default = "default_max_selected_targets")]
    pub max_selected_targets: usize,
    #[serde(default = "default_max_generated_attempts")]
    pub max_generated_attempts: usize,
    #[serde(default)]
    pub path_preferences: Vec<String>,
    #[serde(default)]
    pub allowed_change_kinds: Vec<AdaptiveChangeKind>,
}

impl AdaptiveExecutionProfile {
    pub fn validate(&self, read_targets: &[String]) -> Result<(), ExecutionProfileError> {
        if self.max_selected_targets == 0 {
            return Err(ExecutionProfileError::InvalidAdaptiveSelectedTargetLimit);
        }

        if self.max_generated_attempts == 0 {
            return Err(ExecutionProfileError::InvalidAdaptiveAttemptLimit);
        }

        if read_targets.is_empty() {
            return Err(ExecutionProfileError::MissingAdaptiveReadTargets);
        }

        for preference in &self.path_preferences {
            if preference.trim().is_empty() {
                return Err(ExecutionProfileError::InvalidAdaptivePathPreference(
                    preference.clone(),
                ));
            }

            if !is_workspace_relative(preference) {
                return Err(ExecutionProfileError::InvalidAdaptivePathPreference(
                    preference.clone(),
                ));
            }
        }

        Ok(())
    }

    pub fn effective_change_kinds(&self) -> Vec<AdaptiveChangeKind> {
        if self.allowed_change_kinds.is_empty() {
            vec![
                AdaptiveChangeKind::ArithmeticSwap,
                AdaptiveChangeKind::ComparisonFlip,
                AdaptiveChangeKind::BooleanFlip,
            ]
        } else {
            self.allowed_change_kinds.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathScore {
    pub path: String,
    pub score: i64,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSliceSelection {
    pub selection_id: String,
    #[serde(default)]
    pub selected_targets: Vec<String>,
    #[serde(default)]
    pub scored_candidates: Vec<PathScore>,
    pub headline: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionEvidence {
    #[serde(default)]
    pub goal_terms: Vec<String>,
    #[serde(default)]
    pub validation_terms: Vec<String>,
    #[serde(default)]
    pub path_scores: Vec<PathScore>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptTransitionKind {
    Initial,
    Narrowed,
    Broadened,
    Replaced,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttemptLineage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_attempt_id: Option<String>,
    pub current_attempt_id: String,
    pub transition_kind: AttemptTransitionKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceExecutionProfile {
    pub name: String,
    #[serde(default)]
    pub read_targets: Vec<String>,
    pub validation_command: ExecutionCommand,
    #[serde(default)]
    pub attempts: Vec<ExecutionAttemptDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive: Option<AdaptiveExecutionProfile>,
    #[serde(default = "RunLimits::default")]
    pub limits: RunLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_source: Option<String>,
}

impl WorkspaceExecutionProfile {
    pub fn validate(&self) -> Result<(), ExecutionProfileError> {
        if self.name.trim().is_empty() {
            return Err(ExecutionProfileError::MissingProfileName);
        }

        for path in &self.read_targets {
            if !is_workspace_relative(path) {
                return Err(ExecutionProfileError::InvalidWorkspacePath(path.clone()));
            }
        }

        self.validation_command.validate()?;
        self.limits
            .validate()
            .map_err(|error| ExecutionProfileError::InvalidRunLimits(error.to_string()))?;

        if self.attempts.is_empty() && self.adaptive.is_none() {
            return Err(ExecutionProfileError::MissingAttempts);
        }

        let mut seen_attempts = std::collections::BTreeSet::new();
        for attempt in &self.attempts {
            attempt.validate()?;
            if !seen_attempts.insert(attempt.attempt_id.clone()) {
                return Err(ExecutionProfileError::DuplicateAttemptId(attempt.attempt_id.clone()));
            }
        }

        if let Some(adaptive) = &self.adaptive {
            adaptive.validate(&self.read_targets)?;
        }

        if let Some(review) = &self.review {
            review.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Updated,
    AlreadyApplied,
    MissingTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeEvidence {
    pub path: String,
    pub change_status: ChangeStatus,
    pub before_excerpt: String,
    pub after_excerpt: String,
    pub diff_preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationRecord {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub succeeded: bool,
}

fn default_failure_mode() -> ExecutionFailureMode {
    ExecutionFailureMode::Replan
}

const fn default_max_selected_targets() -> usize {
    1
}

const fn default_max_generated_attempts() -> usize {
    4
}

fn is_workspace_relative(path: &str) -> bool {
    let path = Path::new(path);
    if path.is_absolute() {
        return false;
    }

    !path.components().any(|component| matches!(component, std::path::Component::ParentDir))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExecutionProfileError {
    #[error("execution profile requires a stable name")]
    MissingProfileName,
    #[error("execution profile requires a validation command program")]
    MissingValidationProgram,
    #[error("execution profile must define at least one attempt or adaptive execution profile")]
    MissingAttempts,
    #[error("execution attempt requires a stable id")]
    MissingAttemptId,
    #[error("execution attempt '{0}' must define at least one workspace change")]
    MissingAttemptChanges(String),
    #[error("execution attempt '{0}' is duplicated")]
    DuplicateAttemptId(String),
    #[error("workspace change requires a relative path")]
    MissingChangePath,
    #[error("workspace path '{0}' must remain inside the workspace root")]
    InvalidWorkspacePath(String),
    #[error("workspace change for '{0}' requires a non-empty find pattern")]
    MissingFindPattern(String),
    #[error("execution run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("adaptive execution requires at least one read target")]
    MissingAdaptiveReadTargets,
    #[error("adaptive execution max_selected_targets must be greater than zero")]
    InvalidAdaptiveSelectedTargetLimit,
    #[error("adaptive execution max_generated_attempts must be greater than zero")]
    InvalidAdaptiveAttemptLimit,
    #[error("adaptive execution path preference '{0}' must remain inside the workspace root")]
    InvalidAdaptivePathPreference(String),
    #[error("review profile is invalid: {0}")]
    InvalidReviewProfile(#[from] ReviewProfileError),
}
