use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::domain::flow::SessionFlowState;
use crate::domain::task::{Task, TaskPersistenceError, TaskStatus, TerminalReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Initialized,
    GoalCaptured,
    Planned,
    Running,
    Succeeded,
    Failed,
    Exhausted,
    Aborted,
    Invalid,
}

impl SessionStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Exhausted | Self::Aborted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionCommand {
    Start,
    Capture,
    Flow,
    Plan,
    Step,
    Run,
    Status,
    Next,
    Inspect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveSessionRecord {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_flow: Option<SessionFlowState>,
    pub active_task: Option<Task>,
    pub latest_status: SessionStatus,
    pub latest_terminal_reason: Option<TerminalReason>,
    pub latest_trace_ref: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ActiveSessionRecord {
    pub fn validate(&self) -> Result<(), SessionValidationError> {
        if self.session_id.trim().is_empty() {
            return Err(SessionValidationError::MissingSessionId);
        }

        if self.workspace_ref.trim().is_empty() {
            return Err(SessionValidationError::MissingWorkspaceRef);
        }

        if self.updated_at < self.created_at {
            return Err(SessionValidationError::UpdatedBeforeCreated {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        if let Some(trace_ref) = &self.latest_trace_ref
            && !trace_within_workspace(&self.workspace_ref, trace_ref)
        {
            return Err(SessionValidationError::TraceOutsideWorkspace {
                workspace_ref: self.workspace_ref.clone(),
                trace_ref: trace_ref.clone(),
            });
        }

        if status_requires_goal(self.latest_status)
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(SessionValidationError::MissingGoal(self.latest_status));
        }

        if status_requires_task(self.latest_status) && self.active_task.is_none() {
            return Err(SessionValidationError::MissingActiveTask(self.latest_status));
        }

        if let Some(active_flow) = &self.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionValidationError::InvalidFlowState(error.to_string()))?;
        }

        if self.latest_status.is_terminal() && self.latest_terminal_reason.is_none() {
            return Err(SessionValidationError::MissingTerminalReason(self.latest_status));
        }

        if let Some(task) = &self.active_task {
            task.validate_persisted_state()
                .map_err(|error| SessionValidationError::InvalidTask(error.to_string()))?;

            if !task.context.belongs_to_workspace(&self.workspace_ref) {
                return Err(SessionValidationError::TaskWorkspaceMismatch {
                    expected: self.workspace_ref.clone(),
                    actual: task.context.workspace_ref.clone(),
                });
            }

            if let Some(goal) = &self.goal
                && task.goal.trim() != goal.trim()
            {
                return Err(SessionValidationError::TaskGoalMismatch {
                    expected: goal.clone(),
                    actual: task.goal.clone(),
                });
            }

            if let Some(expected_status) = expected_task_status(self.latest_status)
                && task.status != expected_status
            {
                return Err(SessionValidationError::TaskStatusMismatch {
                    expected: expected_status,
                    actual: task.status,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionTransition {
    pub trigger_command: SessionCommand,
    pub from_status: Option<SessionStatus>,
    pub to_status: SessionStatus,
    pub trace_ref: Option<String>,
    pub reason: String,
}

impl SessionTransition {
    pub fn validate(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        if self.reason.trim().is_empty() {
            return Err(SessionValidationError::MissingTransitionReason);
        }

        if self.to_status != record.latest_status {
            return Err(SessionValidationError::TransitionStatusMismatch {
                expected: record.latest_status,
                actual: self.to_status,
            });
        }

        if self.trace_ref != record.latest_trace_ref {
            return Err(SessionValidationError::TransitionTraceMismatch {
                expected: record.latest_trace_ref.clone(),
                actual: self.trace_ref.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStatusView {
    pub session_id: String,
    pub workspace_ref: String,
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_flow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_stage_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_stages: Option<usize>,
    pub plan_revision: Option<usize>,
    pub current_step_id: Option<String>,
    pub current_step_index: Option<usize>,
    pub latest_status: SessionStatus,
    pub latest_trace_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_changed_files: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_validation_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_vote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_review_headline: Option<String>,
    pub next_command: Option<String>,
    pub explanation: String,
}

impl SessionStatusView {
    pub fn validate(&self, record: &ActiveSessionRecord) -> Result<(), SessionValidationError> {
        if self.session_id != record.session_id {
            return Err(SessionValidationError::StatusViewSessionMismatch {
                expected: record.session_id.clone(),
                actual: self.session_id.clone(),
            });
        }

        if self.workspace_ref != record.workspace_ref {
            return Err(SessionValidationError::StatusViewWorkspaceMismatch {
                expected: record.workspace_ref.clone(),
                actual: self.workspace_ref.clone(),
            });
        }

        if self.latest_status != record.latest_status {
            return Err(SessionValidationError::StatusViewStatusMismatch {
                expected: record.latest_status,
                actual: self.latest_status,
            });
        }

        if self.goal != record.goal {
            return Err(SessionValidationError::StatusViewGoalMismatch {
                expected: record.goal.clone(),
                actual: self.goal.clone(),
            });
        }

        let expected_flow = record.active_flow.as_ref().map(|flow| flow.flow_name.clone());
        if self.active_flow != expected_flow {
            return Err(SessionValidationError::StatusViewFlowMismatch {
                expected: expected_flow,
                actual: self.active_flow.clone(),
            });
        }

        let expected_stage_id =
            record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone());
        if self.current_stage_id != expected_stage_id {
            return Err(SessionValidationError::StatusViewStageMismatch {
                expected: expected_stage_id,
                actual: self.current_stage_id.clone(),
            });
        }

        let expected_stage_index = record.active_flow.as_ref().map(|flow| flow.current_stage_index);
        if self.current_stage_index != expected_stage_index {
            return Err(SessionValidationError::StatusViewStageIndexMismatch {
                expected: expected_stage_index,
                actual: self.current_stage_index,
            });
        }

        let expected_total_stages = record.active_flow.as_ref().map(|flow| flow.total_stages);
        if self.total_stages != expected_total_stages {
            return Err(SessionValidationError::StatusViewStageCountMismatch {
                expected: expected_total_stages,
                actual: self.total_stages,
            });
        }

        if self.latest_trace_ref != record.latest_trace_ref {
            return Err(SessionValidationError::StatusViewTraceMismatch {
                expected: record.latest_trace_ref.clone(),
                actual: self.latest_trace_ref.clone(),
            });
        }

        let expected_changed_files = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_strings(task, "latest_changed_files"));
        if self.latest_changed_files != expected_changed_files {
            return Err(SessionValidationError::StatusViewChangedFilesMismatch {
                expected: expected_changed_files,
                actual: self.latest_changed_files.clone(),
            });
        }

        let expected_validation_status = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_validation_status"));
        if self.latest_validation_status != expected_validation_status {
            return Err(SessionValidationError::StatusViewValidationStatusMismatch {
                expected: expected_validation_status,
                actual: self.latest_validation_status.clone(),
            });
        }

        let expected_review_trigger = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_trigger"));
        if self.latest_review_trigger != expected_review_trigger {
            return Err(SessionValidationError::StatusViewReviewTriggerMismatch {
                expected: expected_review_trigger,
                actual: self.latest_review_trigger.clone(),
            });
        }

        let expected_review_vote = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_vote"));
        if self.latest_review_vote != expected_review_vote {
            return Err(SessionValidationError::StatusViewReviewVoteMismatch {
                expected: expected_review_vote,
                actual: self.latest_review_vote.clone(),
            });
        }

        let expected_review_outcome = record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_review_outcome"));
        if self.latest_review_outcome != expected_review_outcome {
            return Err(SessionValidationError::StatusViewReviewOutcomeMismatch {
                expected: expected_review_outcome,
                actual: self.latest_review_outcome.clone(),
            });
        }

        let expected_review_headline =
            record.active_task.as_ref().and_then(task_state_review_headline);
        if self.latest_review_headline != expected_review_headline {
            return Err(SessionValidationError::StatusViewReviewHeadlineMismatch {
                expected: expected_review_headline,
                actual: self.latest_review_headline.clone(),
            });
        }

        if self.explanation.trim().is_empty() {
            return Err(SessionValidationError::MissingStatusExplanation);
        }

        if let Some(next_command) = &self.next_command
            && next_command.trim().is_empty()
        {
            return Err(SessionValidationError::MissingNextCommand);
        }

        if let Some(task) = &record.active_task {
            let expected_index = task.plan.current_step_index;
            if self.current_step_index != Some(expected_index) {
                return Err(SessionValidationError::StatusViewStepIndexMismatch {
                    expected: Some(expected_index),
                    actual: self.current_step_index,
                });
            }

            let expected_step_id = task.plan.current_step().map(|step| step.id.clone());
            if self.current_step_id != expected_step_id {
                return Err(SessionValidationError::StatusViewStepIdMismatch {
                    expected: expected_step_id,
                    actual: self.current_step_id.clone(),
                });
            }

            if self.plan_revision != Some(task.plan.revision) {
                return Err(SessionValidationError::StatusViewPlanRevisionMismatch {
                    expected: Some(task.plan.revision),
                    actual: self.plan_revision,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SessionValidationError {
    #[error("session_id must not be empty")]
    MissingSessionId,
    #[error("workspace_ref must not be empty")]
    MissingWorkspaceRef,
    #[error("updated_at {updated_at} must be greater than or equal to created_at {created_at}")]
    UpdatedBeforeCreated { created_at: u64, updated_at: u64 },
    #[error("status {0:?} requires a goal")]
    MissingGoal(SessionStatus),
    #[error("status {0:?} requires an active task")]
    MissingActiveTask(SessionStatus),
    #[error("session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("status {0:?} requires a terminal reason")]
    MissingTerminalReason(SessionStatus),
    #[error("session task workspace_ref mismatch: expected {expected}, got {actual}")]
    TaskWorkspaceMismatch { expected: String, actual: String },
    #[error("session task goal mismatch: expected {expected}, got {actual}")]
    TaskGoalMismatch { expected: String, actual: String },
    #[error("session task status mismatch: expected {expected:?}, got {actual:?}")]
    TaskStatusMismatch { expected: TaskStatus, actual: TaskStatus },
    #[error("latest_trace_ref {trace_ref} must point inside workspace {workspace_ref}")]
    TraceOutsideWorkspace { workspace_ref: String, trace_ref: String },
    #[error("active task is invalid: {0}")]
    InvalidTask(String),
    #[error("session transition reason must not be empty")]
    MissingTransitionReason,
    #[error("session transition status mismatch: expected {expected:?}, got {actual:?}")]
    TransitionStatusMismatch { expected: SessionStatus, actual: SessionStatus },
    #[error("session transition trace mismatch: expected {expected:?}, got {actual:?}")]
    TransitionTraceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view session mismatch: expected {expected}, got {actual}")]
    StatusViewSessionMismatch { expected: String, actual: String },
    #[error("status view workspace mismatch: expected {expected}, got {actual}")]
    StatusViewWorkspaceMismatch { expected: String, actual: String },
    #[error("status view status mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStatusMismatch { expected: SessionStatus, actual: SessionStatus },
    #[error("status view goal mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewGoalMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view flow mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewFlowMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view stage mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view stage index mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageIndexMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view total stages mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStageCountMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view trace mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewTraceMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view changed files mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewChangedFilesMismatch { expected: Option<Vec<String>>, actual: Option<Vec<String>> },
    #[error("status view validation status mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewValidationStatusMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review trigger mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewTriggerMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review vote mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewVoteMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review outcome mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewOutcomeMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view review headline mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewReviewHeadlineMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view explanation must not be empty")]
    MissingStatusExplanation,
    #[error("status view next_command must not be empty when present")]
    MissingNextCommand,
    #[error("status view step index mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStepIndexMismatch { expected: Option<usize>, actual: Option<usize> },
    #[error("status view step id mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewStepIdMismatch { expected: Option<String>, actual: Option<String> },
    #[error("status view plan revision mismatch: expected {expected:?}, got {actual:?}")]
    StatusViewPlanRevisionMismatch { expected: Option<usize>, actual: Option<usize> },
}

fn status_requires_goal(status: SessionStatus) -> bool {
    !matches!(status, SessionStatus::Initialized | SessionStatus::Invalid)
}

fn status_requires_task(status: SessionStatus) -> bool {
    matches!(
        status,
        SessionStatus::Planned
            | SessionStatus::Running
            | SessionStatus::Succeeded
            | SessionStatus::Failed
            | SessionStatus::Exhausted
            | SessionStatus::Aborted
    )
}

fn expected_task_status(status: SessionStatus) -> Option<TaskStatus> {
    match status {
        SessionStatus::Planned => Some(TaskStatus::Planned),
        SessionStatus::Running => Some(TaskStatus::Running),
        SessionStatus::Succeeded => Some(TaskStatus::Succeeded),
        SessionStatus::Failed => Some(TaskStatus::Failed),
        SessionStatus::Exhausted => Some(TaskStatus::Exhausted),
        SessionStatus::Aborted => Some(TaskStatus::Aborted),
        SessionStatus::Initialized | SessionStatus::GoalCaptured | SessionStatus::Invalid => None,
    }
}

fn trace_within_workspace(workspace_ref: &str, trace_ref: &str) -> bool {
    let trace_path = Path::new(trace_ref);
    if trace_path.is_absolute() {
        trace_path.starts_with(Path::new(workspace_ref))
    } else {
        !trace_path.starts_with("..")
    }
}

fn task_state_string(task: &Task, key: &str) -> Option<String> {
    task.context.state.get(key).and_then(|value| value.as_str().map(str::to_string))
}

fn task_state_strings(task: &Task, key: &str) -> Option<Vec<String>> {
    task.context.state.get(key).and_then(|value| {
        value.as_array().map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
    })
}

fn task_state_review_headline(task: &Task) -> Option<String> {
    let latest_finding = task
        .context
        .state
        .get("latest_review_findings")
        .and_then(Value::as_array)
        .and_then(|findings| findings.last());
    if let Some(finding) = latest_finding {
        let reviewer_id = finding.get("reviewer_id").and_then(Value::as_str).unwrap_or("reviewer");
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        return Some(format!("{reviewer_id} {disposition}: {summary}"));
    }

    let participants = task
        .context
        .state
        .get("latest_review_participants")
        .and_then(Value::as_array)
        .map(|participants| {
            participants
                .iter()
                .filter_map(|participant| {
                    let reviewer_id = participant.get("reviewer_id").and_then(Value::as_str)?;
                    let status =
                        participant.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    Some(format!("{reviewer_id} {status}"))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if participants.is_empty() {
        None
    } else {
        Some(format!("participants: {}", participants.join(", ")))
    }
}

impl From<TaskPersistenceError> for SessionValidationError {
    fn from(value: TaskPersistenceError) -> Self {
        Self::InvalidTask(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ActiveSessionRecord, SessionStatus, SessionStatusView, SessionValidationError,
        task_state_review_headline, task_state_string, task_state_strings, trace_within_workspace,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::step::Step;
    use crate::domain::task::{Task, TaskPersistenceError, TaskRunRequest};

    fn build_task(workspace_ref: &str) -> Task {
        let request = TaskRunRequest {
            goal: "Deliver a session-backed CLI".to_string(),
            input: json!({"ticket": "SESSION-TEST"}),
            session_id: "session-1".to_string(),
            workspace_ref: workspace_ref.to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        };
        let plan = Plan::new(vec![Step::decision("analyze", json!({})).unwrap()]).unwrap();
        Task::new("task-1", &request, plan).unwrap()
    }

    fn build_record(workspace_ref: &str) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: "session-1".to_string(),
            workspace_ref: workspace_ref.to_string(),
            goal: Some("Deliver a session-backed CLI".to_string()),
            active_flow: Some(
                crate::domain::flow::built_in_flow("bug-fix").unwrap().initial_state(),
            ),
            active_task: Some(build_task(workspace_ref)),
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: Some(format!("{workspace_ref}/.synod/traces/task-1.json")),
            created_at: 10,
            updated_at: 20,
        }
    }

    fn build_view(record: &ActiveSessionRecord) -> SessionStatusView {
        SessionStatusView {
            session_id: record.session_id.clone(),
            workspace_ref: record.workspace_ref.clone(),
            goal: record.goal.clone(),
            active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
            current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
            current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
            total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
            plan_revision: record.active_task.as_ref().map(|task| task.plan.revision),
            current_step_id: record
                .active_task
                .as_ref()
                .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
            current_step_index: record
                .active_task
                .as_ref()
                .map(|task| task.plan.current_step_index),
            latest_status: record.latest_status,
            latest_trace_ref: record.latest_trace_ref.clone(),
            latest_changed_files: None,
            latest_validation_status: None,
            latest_review_trigger: None,
            latest_review_vote: None,
            latest_review_outcome: None,
            latest_review_headline: None,
            next_command: Some("synod step".to_string()),
            explanation: "view is consistent".to_string(),
        }
    }

    #[test]
    fn status_view_rejects_stage_count_trace_and_step_index_mismatches() {
        let workspace = "/tmp/synod-session-domain";
        let record = build_record(workspace);

        let mut wrong_stage_index = build_view(&record);
        wrong_stage_index.current_stage_index = Some(1);
        assert!(matches!(
            wrong_stage_index.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStageIndexMismatch { .. }
        ));

        let mut wrong_stage_count = build_view(&record);
        wrong_stage_count.total_stages = Some(99);
        assert!(matches!(
            wrong_stage_count.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStageCountMismatch { .. }
        ));

        let mut wrong_trace = build_view(&record);
        wrong_trace.latest_trace_ref = Some("/tmp/other/trace.json".to_string());
        assert!(matches!(
            wrong_trace.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewTraceMismatch { .. }
        ));

        let mut wrong_step_index = build_view(&record);
        wrong_step_index.current_step_index = Some(99);
        assert!(matches!(
            wrong_step_index.validate(&record).unwrap_err(),
            SessionValidationError::StatusViewStepIndexMismatch { .. }
        ));
    }

    #[test]
    fn helper_functions_cover_relative_trace_paths_and_state_extractors() {
        assert!(trace_within_workspace("/tmp/workspace", "trace.json"));
        assert!(!trace_within_workspace("/tmp/workspace", "../outside.json"));

        let mut task = build_task("/tmp/workspace");
        task.context.state.insert("latest_validation_status".to_string(), json!("passed"));
        task.context.state.insert("latest_changed_files".to_string(), json!(["src/lib.rs"]));
        task.context.state.insert("latest_review_trigger".to_string(), json!("pr_ready"));
        task.context.state.insert(
            "latest_review_findings".to_string(),
            json!([{
                "reviewer_id": "safety",
                "disposition": "approve",
                "summary": "No blockers"
            }]),
        );

        assert_eq!(
            task_state_string(&task, "latest_validation_status"),
            Some("passed".to_string())
        );
        assert_eq!(
            task_state_strings(&task, "latest_changed_files"),
            Some(vec!["src/lib.rs".to_string()])
        );
        assert_eq!(task_state_string(&task, "latest_review_trigger"), Some("pr_ready".to_string()));
        assert_eq!(
            task_state_review_headline(&task),
            Some("safety approve: No blockers".to_string())
        );
    }

    #[test]
    fn task_persistence_errors_convert_to_session_validation_errors() {
        let error = SessionValidationError::from(TaskPersistenceError::MissingGoal);
        assert!(
            matches!(error, SessionValidationError::InvalidTask(message) if message.contains("task goal must not be empty"))
        );
    }
}
