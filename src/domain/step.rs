//! Step execution models, endpoint requests, and persisted step summaries.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::task_context::TaskContext;

/// Kind of executable step in a task plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Agent,
    Tool,
    Decision,
}

/// Persisted lifecycle status of one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

/// Recovery policy attached to a failed step result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recoverability {
    Retryable,
    ReplanRequired,
    Terminal,
}

/// Success or failure status returned by a step endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
}

/// Structured error details recorded for failed steps or invalid endpoint output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorInfo {
    /// Creates a new error payload.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into(), details: None }
    }

    /// Attaches structured details to the error payload.
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Persisted plan step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub kind: StepKind,
    pub target_name: Option<String>,
    pub input: Value,
    pub status: StepStatus,
    pub attempt_count: usize,
    pub output: Option<Value>,
    pub error: Option<ErrorInfo>,
    pub recoverability: Option<Recoverability>,
}

impl Step {
    /// Creates a new step and validates any required target name.
    pub fn new(
        id: impl Into<String>,
        kind: StepKind,
        target_name: Option<String>,
        input: Value,
    ) -> Result<Self, StepError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(StepError::MissingId);
        }

        if matches!(kind, StepKind::Agent | StepKind::Tool)
            && target_name.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(StepError::MissingTargetName(kind));
        }

        Ok(Self {
            id,
            kind,
            target_name,
            input,
            status: StepStatus::Pending,
            attempt_count: 0,
            output: None,
            error: None,
            recoverability: None,
        })
    }

    /// Convenience constructor for an agent step.
    pub fn agent(
        id: impl Into<String>,
        target_name: impl Into<String>,
        input: Value,
    ) -> Result<Self, StepError> {
        Self::new(id, StepKind::Agent, Some(target_name.into()), input)
    }

    /// Convenience constructor for a tool step.
    pub fn tool(
        id: impl Into<String>,
        target_name: impl Into<String>,
        input: Value,
    ) -> Result<Self, StepError> {
        Self::new(id, StepKind::Tool, Some(target_name.into()), input)
    }

    /// Convenience constructor for a decision step.
    pub fn decision(id: impl Into<String>, input: Value) -> Result<Self, StepError> {
        Self::new(id, StepKind::Decision, None, input)
    }

    /// Marks the step as running and increments its attempt count.
    pub fn mark_running(&mut self) {
        self.status = StepStatus::Running;
        self.attempt_count += 1;
    }

    /// Marks the step as succeeded with the given output.
    pub fn mark_succeeded(&mut self, output: Value) {
        self.status = StepStatus::Succeeded;
        self.output = Some(output);
        self.error = None;
        self.recoverability = None;
    }

    /// Marks the step as failed with structured error and recoverability.
    pub fn mark_failed(&mut self, error: ErrorInfo, recoverability: Recoverability) {
        self.status = StepStatus::Failed;
        self.output = None;
        self.error = Some(error);
        self.recoverability = Some(recoverability);
    }
}

/// Validation failures for step construction.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StepError {
    #[error("step id must not be empty")]
    MissingId,
    #[error("step kind {0:?} requires a target name")]
    MissingTargetName(StepKind),
}

/// Request passed to an agent or tool endpoint when executing a step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepExecutionRequest {
    pub step_id: String,
    pub step_kind: StepKind,
    pub target_name: String,
    pub input: Value,
    pub task_snapshot: TaskContext,
    pub attempt_number: usize,
}

/// Result returned by a step endpoint before it is normalized into persisted state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub status: ExecutionStatus,
    pub output: Option<Value>,
    pub error: Option<ErrorInfo>,
    pub recoverability: Recoverability,
    pub evidence: Option<Value>,
    pub state_patch: Option<Map<String, Value>>,
}

impl StepExecutionResult {
    /// Creates a successful step result.
    pub fn success(output: Value) -> Self {
        Self {
            status: ExecutionStatus::Succeeded,
            output: Some(output),
            error: None,
            recoverability: Recoverability::Terminal,
            evidence: None,
            state_patch: None,
        }
    }

    /// Creates a successful step result with an accompanying task-state patch.
    pub fn success_with_patch(output: Value, state_patch: Map<String, Value>) -> Self {
        Self { state_patch: Some(state_patch), ..Self::success(output) }
    }

    /// Creates a failed step result.
    pub fn failure(error: ErrorInfo, recoverability: Recoverability) -> Self {
        Self {
            status: ExecutionStatus::Failed,
            output: None,
            error: Some(error),
            recoverability,
            evidence: None,
            state_patch: None,
        }
    }

    /// Attaches structured evidence to the step result.
    pub fn with_evidence(mut self, evidence: Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// Attaches a task-state patch to the step result.
    pub fn with_state_patch(mut self, state_patch: Map<String, Value>) -> Self {
        self.state_patch = Some(state_patch);
        self
    }

    /// Validates that the result shape matches its execution status.
    pub fn validate(&self) -> Result<(), StepExecutionResultError> {
        match self.status {
            ExecutionStatus::Succeeded => {
                if self.output.is_none() {
                    return Err(StepExecutionResultError::MissingOutput);
                }
                if self.error.is_some() {
                    return Err(StepExecutionResultError::ConflictingOutputAndError);
                }
            }
            ExecutionStatus::Failed => {
                if self.error.is_none() {
                    return Err(StepExecutionResultError::MissingError);
                }
                if self.output.is_some() {
                    return Err(StepExecutionResultError::ConflictingOutputAndError);
                }
            }
        }

        Ok(())
    }
}

/// Validation failures for step endpoint results.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StepExecutionResultError {
    #[error("successful step results must include output")]
    MissingOutput,
    #[error("failed step results must include error details")]
    MissingError,
    #[error("step results cannot contain both output and error")]
    ConflictingOutputAndError,
}

/// One concrete attempt to execute a step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepAttempt {
    pub attempt_id: String,
    pub step_id: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub input_snapshot: Value,
    pub result_snapshot: Option<Value>,
    pub failure_kind: Option<Recoverability>,
}

impl StepAttempt {
    /// Creates a new step-attempt record.
    pub fn new(step_id: impl Into<String>, input_snapshot: Value, started_at: u64) -> Self {
        Self {
            attempt_id: Uuid::new_v4().to_string(),
            step_id: step_id.into(),
            started_at,
            ended_at: None,
            input_snapshot,
            result_snapshot: None,
            failure_kind: None,
        }
    }

    /// Completes the attempt from the endpoint result snapshot.
    pub fn complete(&mut self, result: &StepExecutionResult, ended_at: u64) {
        self.ended_at = Some(ended_at);
        self.result_snapshot = result.output.clone().or_else(|| {
            result.error.as_ref().map(|error| serde_json::to_value(error).unwrap_or(Value::Null))
        });
        if matches!(result.status, ExecutionStatus::Failed) {
            self.failure_kind = Some(result.recoverability);
        }
    }
}

/// Flattened summary of a persisted step result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResultSummary {
    pub step_id: String,
    pub status: StepStatus,
    pub output: Option<Value>,
    pub error: Option<ErrorInfo>,
    pub recoverability: Option<Recoverability>,
}

impl StepResultSummary {
    /// Builds a flattened summary from the current persisted step state.
    pub fn from_step(step: &Step) -> Self {
        Self {
            step_id: step.id.clone(),
            status: step.status,
            output: step.output.clone(),
            error: step.error.clone(),
            recoverability: step.recoverability,
        }
    }
}
