//! Task lifecycle models, run requests, and persisted task state.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

use crate::domain::limits::RunLimits;
use crate::domain::plan::{Plan, PlanError};
use crate::domain::task_context::{TaskContext, TaskContextError};

/// Lifecycle status of a persisted task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planned,
    Running,
    Succeeded,
    Failed,
    Exhausted,
    Aborted,
}

impl TaskStatus {
    /// Returns true when the task is in a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Exhausted | Self::Aborted)
    }
}

/// Terminal reason persisted when a task stops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalReason {
    pub condition: crate::domain::limits::TerminalCondition,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl TerminalReason {
    /// Creates a new terminal reason.
    pub fn new(
        condition: crate::domain::limits::TerminalCondition,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self { condition, message: message.into(), details }
    }
}

/// Why a clarification record was raised while preparing bounded work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarificationReasonKind {
    MissingContext,
    SourceConflict,
    MissingSource,
    UnsupportedSource,
    UnboundedRequest,
}

/// Status of a clarification record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClarificationStatus {
    Open,
    Answered,
    Exhausted,
}

/// Persisted clarification record attached to derived task preparation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClarificationRecord {
    pub clarification_id: String,
    pub reason_kind: ClarificationReasonKind,
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub questions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_sources: Vec<String>,
    pub turn_index: usize,
    pub status: ClarificationStatus,
}

impl ClarificationRecord {
    /// Returns a compact operator-facing headline for the clarification.
    pub fn headline(&self) -> String {
        match self.reason_kind {
            ClarificationReasonKind::MissingContext => {
                "clarification required: provide the missing business context".to_string()
            }
            ClarificationReasonKind::SourceConflict => {
                "clarification required: resolve the conflicting source material".to_string()
            }
            ClarificationReasonKind::MissingSource => {
                "clarification required: provide the missing authored source".to_string()
            }
            ClarificationReasonKind::UnsupportedSource => {
                "clarification required: replace the unsupported authored source".to_string()
            }
            ClarificationReasonKind::UnboundedRequest => {
                "clarification required: narrow the request to one bounded outcome".to_string()
            }
        }
    }
}

/// Intermediate bounded draft produced before a full task run request is built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedTaskDraft {
    pub draft_id: String,
    pub bundle_id: String,
    pub bounded_goal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_hint: Option<String>,
    pub planning_ready: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_clarification_ref: Option<String>,
}

/// Request used to create a runnable task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRunRequest {
    pub goal: String,
    pub input: Value,
    pub session_id: String,
    pub workspace_ref: String,
    pub limits: RunLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_context: Option<Map<String, Value>>,
}

impl TaskRunRequest {
    /// Validates the task run request and nested run limits.
    pub fn validate(&self) -> Result<(), TaskRequestError> {
        if self.goal.trim().is_empty() {
            return Err(TaskRequestError::EmptyGoal);
        }
        if self.session_id.trim().is_empty() {
            return Err(TaskRequestError::MissingSessionId);
        }
        if self.workspace_ref.trim().is_empty() {
            return Err(TaskRequestError::MissingWorkspaceRef);
        }

        self.limits
            .validate()
            .map_err(|error| TaskRequestError::InvalidRunLimits(error.to_string()))
    }
}

/// Terminal response returned after a task reaches a final state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRunResponse {
    pub task_id: String,
    pub terminal_status: TaskStatus,
    pub terminal_reason: TerminalReason,
    pub final_context: TaskContext,
    pub plan_revision: usize,
    pub trace_location: String,
}

/// Persisted task state used by compatibility execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub goal: String,
    pub input: Value,
    pub context: TaskContext,
    pub plan: Plan,
    pub status: TaskStatus,
    pub limits: RunLimits,
    pub terminal_reason: Option<TerminalReason>,
    pub retry_count: usize,
    pub replan_count: usize,
    pub total_step_attempts: usize,
}

impl Task {
    /// Creates a new persisted task from a validated run request and plan.
    pub fn new(
        id: impl Into<String>,
        request: &TaskRunRequest,
        plan: Plan,
    ) -> Result<Self, TaskRequestError> {
        request.validate()?;
        plan.validate().map_err(|error| TaskRequestError::InvalidPlan(error.to_string()))?;

        let context = TaskContext::new(
            request.session_id.clone(),
            request.workspace_ref.clone(),
            request.limits.clone(),
            request.initial_context.clone().unwrap_or_default(),
        );
        context.validate().map_err(|error| TaskRequestError::InvalidContext(error.to_string()))?;

        Ok(Self {
            id: id.into(),
            goal: request.goal.clone(),
            input: request.input.clone(),
            context,
            plan,
            status: TaskStatus::Planned,
            limits: request.limits.clone(),
            terminal_reason: None,
            retry_count: 0,
            replan_count: 0,
            total_step_attempts: 0,
        })
    }

    /// Marks the task as running.
    pub fn mark_running(&mut self) {
        self.status = TaskStatus::Running;
    }

    /// Applies terminal state and reason to the task.
    pub fn apply_terminal(&mut self, status: TaskStatus, reason: TerminalReason) {
        self.status = status;
        self.terminal_reason = Some(reason);
    }

    /// Validates the persisted task snapshot and nested state.
    pub fn validate_persisted_state(&self) -> Result<(), TaskPersistenceError> {
        if self.id.trim().is_empty() {
            return Err(TaskPersistenceError::MissingTaskId);
        }

        if self.goal.trim().is_empty() {
            return Err(TaskPersistenceError::MissingGoal);
        }

        self.context
            .validate()
            .map_err(|error| TaskPersistenceError::InvalidContext(error.to_string()))?;
        self.plan
            .validate()
            .map_err(|error| TaskPersistenceError::InvalidPlan(error.to_string()))?;
        self.limits
            .validate()
            .map_err(|error| TaskPersistenceError::InvalidRunLimits(error.to_string()))?;

        if self.status.is_terminal() && self.terminal_reason.is_none() {
            return Err(TaskPersistenceError::MissingTerminalReason(self.status));
        }

        if !self.status.is_terminal() && self.terminal_reason.is_some() {
            return Err(TaskPersistenceError::UnexpectedTerminalReason(self.status));
        }

        if self.total_step_attempts < self.retry_count
            || self.total_step_attempts < self.replan_count
        {
            return Err(TaskPersistenceError::InvalidAttemptCounters {
                total_step_attempts: self.total_step_attempts,
                retry_count: self.retry_count,
                replan_count: self.replan_count,
            });
        }

        Ok(())
    }
}

/// Errors raised while building a task run request or task.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TaskRequestError {
    #[error("task goal must not be empty")]
    EmptyGoal,
    #[error("task session_id must not be empty")]
    MissingSessionId,
    #[error("task workspace_ref must not be empty")]
    MissingWorkspaceRef,
    #[error("task run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("task plan is invalid: {0}")]
    InvalidPlan(String),
    #[error("task context is invalid: {0}")]
    InvalidContext(String),
}

impl From<PlanError> for TaskRequestError {
    fn from(value: PlanError) -> Self {
        Self::InvalidPlan(value.to_string())
    }
}

impl From<TaskContextError> for TaskRequestError {
    fn from(value: TaskContextError) -> Self {
        Self::InvalidContext(value.to_string())
    }
}

/// Errors raised while validating persisted task state.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TaskPersistenceError {
    #[error("task id must not be empty")]
    MissingTaskId,
    #[error("task goal must not be empty")]
    MissingGoal,
    #[error("persisted task run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("persisted task plan is invalid: {0}")]
    InvalidPlan(String),
    #[error("persisted task context is invalid: {0}")]
    InvalidContext(String),
    #[error("terminal task status {0:?} requires a terminal_reason")]
    MissingTerminalReason(TaskStatus),
    #[error("non-terminal task status {0:?} must not carry a terminal_reason")]
    UnexpectedTerminalReason(TaskStatus),
    #[error(
        "total_step_attempts {total_step_attempts} must be greater than or equal to retry_count {retry_count} and replan_count {replan_count}"
    )]
    InvalidAttemptCounters { total_step_attempts: usize, retry_count: usize, replan_count: usize },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ClarificationReasonKind, ClarificationRecord, ClarificationStatus, Task, TaskRunRequest,
        TaskStatus, TerminalReason,
    };
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::plan::Plan;
    use crate::domain::step::Step;

    fn valid_request() -> TaskRunRequest {
        TaskRunRequest {
            goal: "implement bounded change".to_string(),
            input: json!({"goal": "implement bounded change"}),
            session_id: "session-1".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        }
    }

    fn valid_plan() -> Plan {
        Plan::new(vec![Step::decision("decision-1", json!({"decision": "go"})).unwrap()]).unwrap()
    }

    #[test]
    fn task_status_and_clarification_helpers_cover_all_variants() {
        for status in [TaskStatus::Planned, TaskStatus::Running] {
            assert!(!status.is_terminal());
        }

        for status in
            [TaskStatus::Succeeded, TaskStatus::Failed, TaskStatus::Exhausted, TaskStatus::Aborted]
        {
            assert!(status.is_terminal());
        }

        let expected = [
            (ClarificationReasonKind::MissingContext, "provide the missing business context"),
            (ClarificationReasonKind::SourceConflict, "resolve the conflicting source material"),
            (ClarificationReasonKind::MissingSource, "provide the missing authored source"),
            (ClarificationReasonKind::UnsupportedSource, "replace the unsupported authored source"),
            (
                ClarificationReasonKind::UnboundedRequest,
                "narrow the request to one bounded outcome",
            ),
        ];

        for (reason_kind, fragment) in expected {
            let record = ClarificationRecord {
                clarification_id: format!("clarification-{fragment}"),
                reason_kind,
                prompt: "question".to_string(),
                missing_fields: Vec::new(),
                questions: Vec::new(),
                blocking_sources: Vec::new(),
                turn_index: 0,
                status: ClarificationStatus::Open,
            };
            assert!(record.headline().contains(fragment));
        }
    }

    #[test]
    fn task_validation_helpers_cover_request_and_persisted_state_paths() {
        let request = valid_request();
        assert!(request.validate().is_ok());

        let mut invalid_request = request.clone();
        invalid_request.goal = "   ".to_string();
        assert!(invalid_request.validate().is_err());

        let mut task = Task::new("task-1", &request, valid_plan()).unwrap();
        assert!(task.validate_persisted_state().is_ok());

        task.mark_running();
        assert_eq!(task.status, TaskStatus::Running);

        let success_reason = TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "completed",
            Some(json!({"evidence": "tests"})),
        );
        task.apply_terminal(TaskStatus::Succeeded, success_reason.clone());
        assert!(task.validate_persisted_state().is_ok());

        task.terminal_reason = None;
        assert!(task.validate_persisted_state().is_err());

        task.status = TaskStatus::Running;
        task.terminal_reason = Some(success_reason.clone());
        assert!(task.validate_persisted_state().is_err());

        task.status = TaskStatus::Failed;
        task.terminal_reason = Some(success_reason);
        task.total_step_attempts = 0;
        task.retry_count = 1;
        task.replan_count = 0;
        assert!(task.validate_persisted_state().is_err());
    }
}
