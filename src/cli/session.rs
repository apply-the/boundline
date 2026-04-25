use std::path::{Path, PathBuf};

use crate::adapters::trace_store::TraceStore;
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::session::{ActiveSessionRecord, SessionStatus, SessionStatusView};
use crate::domain::task::TaskStatus;
use crate::domain::trace::current_timestamp_millis;
use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_start(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let now = current_timestamp_millis();
    let record = ActiveSessionRecord {
        session_id: Uuid::new_v4().to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        active_flow: None,
        active_task: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: now,
        updated_at: now,
    };

    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some("synod capture --goal <goal>".to_string()),
            "active session initialized for the current workspace",
        )),
    })
}

pub fn execute_capture(
    workspace: Option<&Path>,
    goal: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;
    runtime.capture_goal(&mut record, goal).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some("synod plan".to_string()),
            "captured the active goal for the current workspace session",
        )),
    })
}

pub fn execute_flow(
    workspace: Option<&Path>,
    name: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    runtime.select_flow(&mut record, name).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            format!("selected the `{}` delivery flow for the active workspace session", name),
        )),
    })
}

pub fn execute_plan(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    runtime.plan_task(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some("synod step".to_string()),
            "planned the active goal into a resumable task snapshot",
        )),
    })
}

pub fn execute_step(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    runtime.execute_next_step(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    Ok(SessionCommandReport {
        exit_status: exit_status_for_session(record.latest_status),
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            "executed the next planned step and persisted the updated session state",
        )),
    })
}

pub fn execute_run(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    let response = runtime.run_to_terminal(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let trace = runtime.trace_store().load(Path::new(&response.trace_location)).ok();
    let next_command =
        suggested_next_command(&record).unwrap_or_else(|| "synod inspect".to_string());

    Ok(SessionCommandReport {
        exit_status: exit_status_for_task(response.terminal_status),
        terminal_output: output::render_run_trace("run", trace.as_ref(), &response, &next_command),
    })
}

pub fn execute_status(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let record = load_active_session(&workspace)?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            suggested_next_command(&record),
            "current active session state for the workspace",
        )),
    })
}

pub fn execute_next(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_workspace(workspace)?;
    let record = load_active_session(&workspace)?;
    let next_command = suggested_next_command(&record)
        .ok_or(SessionCommandError::NotImplemented { command_name: "next", next_command: None })?;

    Ok(SessionCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_session_status(&build_status_view(
            &record,
            Some(next_command.clone()),
            format!("next recommended command for the active session is `{next_command}`"),
        )),
    })
}

pub fn render_error(command_name: &str, error: &SessionCommandError) -> String {
    output::render_session_error(command_name, &error.message(), error.next_command())
}

fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, SessionCommandError> {
    let candidate = match workspace {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => std::env::current_dir()?.join(path),
        None => std::env::current_dir()?,
    };

    Ok(candidate.canonicalize().unwrap_or(candidate))
}

fn load_active_session(workspace: &Path) -> Result<ActiveSessionRecord, SessionCommandError> {
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let store = FileSessionStore::for_workspace(workspace);
    let Some(record) = store.load().map_err(map_store_error)? else {
        return Err(SessionCommandError::MissingActiveSession);
    };

    if record.workspace_ref != workspace_ref {
        return Err(SessionCommandError::WorkspaceMismatch {
            expected: workspace_ref,
            actual: record.workspace_ref,
        });
    }

    Ok(record)
}

fn map_store_error(error: SessionStoreError) -> SessionCommandError {
    match error {
        SessionStoreError::InvalidRecord(message) => {
            SessionCommandError::InvalidActiveSession(message)
        }
        other => SessionCommandError::SessionStore(other),
    }
}

fn map_runtime_error(error: SessionRuntimeError) -> SessionCommandError {
    match error {
        SessionRuntimeError::MissingGoal => SessionCommandError::MissingCapturedGoal,
        SessionRuntimeError::MissingActiveTask => SessionCommandError::MissingPlannedTask,
        SessionRuntimeError::UnknownFlow { requested, supported } => {
            SessionCommandError::UnknownFlow { requested, supported }
        }
        SessionRuntimeError::FlowReplacementRequiresReset { current, requested } => {
            SessionCommandError::FlowReplacementRequiresReset { current, requested }
        }
        SessionRuntimeError::InvalidFlowState(message) => {
            SessionCommandError::InvalidFlowState(message)
        }
        other => SessionCommandError::SessionRuntime(other),
    }
}

fn exit_status_for_session(status: SessionStatus) -> CommandExitStatus {
    match status {
        SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted
        | SessionStatus::Invalid => CommandExitStatus::NonSuccess,
        SessionStatus::Initialized
        | SessionStatus::GoalCaptured
        | SessionStatus::Planned
        | SessionStatus::Running
        | SessionStatus::Succeeded => CommandExitStatus::Succeeded,
    }
}

fn exit_status_for_task(status: TaskStatus) -> CommandExitStatus {
    match status {
        TaskStatus::Failed | TaskStatus::Exhausted | TaskStatus::Aborted => {
            CommandExitStatus::NonSuccess
        }
        TaskStatus::Planned | TaskStatus::Running | TaskStatus::Succeeded => {
            CommandExitStatus::Succeeded
        }
    }
}

fn build_status_view(
    record: &ActiveSessionRecord,
    next_command: Option<String>,
    explanation: impl Into<String>,
) -> SessionStatusView {
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
        current_step_index: record.active_task.as_ref().map(|task| task.plan.current_step_index),
        latest_status: record.latest_status,
        latest_trace_ref: record.latest_trace_ref.clone(),
        next_command,
        explanation: explanation.into(),
    }
}

fn suggested_next_command(record: &ActiveSessionRecord) -> Option<String> {
    match record.latest_status {
        SessionStatus::Initialized => Some("synod capture --goal <goal>".to_string()),
        SessionStatus::GoalCaptured => Some("synod plan".to_string()),
        SessionStatus::Planned | SessionStatus::Running => Some("synod step".to_string()),
        SessionStatus::Succeeded
        | SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted => Some("synod inspect".to_string()),
        SessionStatus::Invalid => Some("synod start".to_string()),
    }
}

#[derive(Debug, Error)]
pub enum SessionCommandError {
    #[error("failed to resolve the current workspace: {0}")]
    WorkspaceResolution(#[from] std::io::Error),
    #[error("no active session found for the current workspace")]
    MissingActiveSession,
    #[error("active session is invalid: {0}")]
    InvalidActiveSession(String),
    #[error("active session belongs to a different workspace: expected {expected}, got {actual}")]
    WorkspaceMismatch { expected: String, actual: String },
    #[error("active session has no captured goal")]
    MissingCapturedGoal,
    #[error("active session has no planned task")]
    MissingPlannedTask,
    #[error("unknown flow `{requested}`; supported flows: {supported}")]
    UnknownFlow { requested: String, supported: String },
    #[error(
        "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
    )]
    FlowReplacementRequiresReset { current: String, requested: String },
    #[error("active session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session runtime operation failed: {0}")]
    SessionRuntime(#[from] SessionRuntimeError),
    #[error("`{command_name}` session workflow is not implemented yet")]
    NotImplemented { command_name: &'static str, next_command: Option<&'static str> },
}

impl SessionCommandError {
    fn message(&self) -> String {
        match self {
            Self::MissingActiveSession => {
                "no active session found for the current workspace".to_string()
            }
            Self::InvalidActiveSession(message) => format!("active session is invalid: {message}"),
            Self::WorkspaceMismatch { expected, actual } => {
                format!(
                    "active session belongs to a different workspace: expected {expected}, got {actual}"
                )
            }
            Self::MissingCapturedGoal => "active session has no captured goal".to_string(),
            Self::MissingPlannedTask => "active session has no planned task".to_string(),
            Self::UnknownFlow { requested, supported } => {
                format!("unknown flow `{requested}`; supported flows: {supported}")
            }
            Self::FlowReplacementRequiresReset { current, requested } => {
                format!(
                    "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
                )
            }
            Self::InvalidFlowState(message) => {
                format!("active session flow state is invalid: {message}")
            }
            Self::NotImplemented { command_name, .. } => {
                format!("`{command_name}` session workflow is not implemented yet")
            }
            Self::WorkspaceResolution(error) => error.to_string(),
            Self::SessionStore(error) => error.to_string(),
            Self::SessionRuntime(error) => error.to_string(),
        }
    }

    fn next_command(&self) -> Option<&str> {
        match self {
            Self::MissingActiveSession
            | Self::WorkspaceMismatch { .. }
            | Self::InvalidActiveSession(_) => Some("synod start"),
            Self::MissingCapturedGoal => Some("synod capture --goal <goal>"),
            Self::MissingPlannedTask => Some("synod plan"),
            Self::UnknownFlow { .. } => Some("synod flow bug-fix"),
            Self::FlowReplacementRequiresReset { .. } => Some("synod start"),
            Self::InvalidFlowState(_) => Some("synod start"),
            Self::NotImplemented { next_command, .. } => *next_command,
            Self::WorkspaceResolution(_) | Self::SessionStore(_) | Self::SessionRuntime(_) => None,
        }
    }
}
