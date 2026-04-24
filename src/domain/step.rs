use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::task_context::TaskContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Agent,
    Tool,
    Decision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recoverability {
    Retryable,
    ReplanRequired,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorInfo {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), message: message.into(), details: None }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

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

    pub fn agent(
        id: impl Into<String>,
        target_name: impl Into<String>,
        input: Value,
    ) -> Result<Self, StepError> {
        Self::new(id, StepKind::Agent, Some(target_name.into()), input)
    }

    pub fn tool(
        id: impl Into<String>,
        target_name: impl Into<String>,
        input: Value,
    ) -> Result<Self, StepError> {
        Self::new(id, StepKind::Tool, Some(target_name.into()), input)
    }

    pub fn decision(id: impl Into<String>, input: Value) -> Result<Self, StepError> {
        Self::new(id, StepKind::Decision, None, input)
    }

    pub fn mark_running(&mut self) {
        self.status = StepStatus::Running;
        self.attempt_count += 1;
    }

    pub fn mark_succeeded(&mut self, output: Value) {
        self.status = StepStatus::Succeeded;
        self.output = Some(output);
        self.error = None;
        self.recoverability = None;
    }

    pub fn mark_failed(&mut self, error: ErrorInfo, recoverability: Recoverability) {
        self.status = StepStatus::Failed;
        self.output = None;
        self.error = Some(error);
        self.recoverability = Some(recoverability);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StepError {
    #[error("step id must not be empty")]
    MissingId,
    #[error("step kind {0:?} requires a target name")]
    MissingTargetName(StepKind),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepExecutionRequest {
    pub step_id: String,
    pub step_kind: StepKind,
    pub target_name: String,
    pub input: Value,
    pub task_snapshot: TaskContext,
    pub attempt_number: usize,
}

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

    pub fn success_with_patch(output: Value, state_patch: Map<String, Value>) -> Self {
        Self { state_patch: Some(state_patch), ..Self::success(output) }
    }

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

    pub fn with_evidence(mut self, evidence: Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StepExecutionResultError {
    #[error("successful step results must include output")]
    MissingOutput,
    #[error("failed step results must include error details")]
    MissingError,
    #[error("step results cannot contain both output and error")]
    ConflictingOutputAndError,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResultSummary {
    pub step_id: String,
    pub status: StepStatus,
    pub output: Option<Value>,
    pub error: Option<ErrorInfo>,
    pub recoverability: Option<Recoverability>,
}

impl StepResultSummary {
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
