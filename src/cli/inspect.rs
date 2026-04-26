use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::TaskStatus;
use crate::domain::trace::{
    ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceResolutionTarget {
    ExplicitTrace,
    SessionTraceRef,
    LatestWorkspaceTrace,
}

impl TraceResolutionTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitTrace => "explicit-trace",
            Self::SessionTraceRef => "session-trace-ref",
            Self::LatestWorkspaceTrace => "latest-workspace-trace",
        }
    }
}

pub fn execute_inspect(
    trace: Option<&Path>,
    workspace: Option<&Path>,
) -> Result<InspectCommandReport, InspectCommandError> {
    let (inspection_target, trace_ref, trace) = load_trace(trace, workspace)?;
    let summary = summarize_trace(&trace_ref, &trace)?;
    let exit_status = if summary.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };

    Ok(InspectCommandReport {
        exit_status,
        terminal_output: output::render_trace_summary(
            &summary,
            inspection_target.as_str(),
            output::next_command_after_inspect(summary.terminal_status),
        ),
    })
}

pub fn render_error(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    error: &InspectCommandError,
) -> String {
    if let InspectCommandError::InvalidSession(message) = error {
        return output::render_session_error("inspect", message, Some("synod start"));
    }

    let inspection_target = inspection_target_for(trace, workspace);
    let trace_ref = trace.map(|path| path.to_string_lossy().into_owned());
    let workspace_ref = workspace.map(|path| path.to_string_lossy().into_owned());
    let terminal_reason = match error {
        InspectCommandError::MissingTraceReference => "inspect requires --trace or --workspace",
        InspectCommandError::MissingLatestTrace | InspectCommandError::TraceStore(_) => {
            "failed to read the requested trace"
        }
        InspectCommandError::SessionStore(_) => "failed to read the active session",
        InspectCommandError::InvalidSession(_) => {
            unreachable!("invalid sessions are rendered separately")
        }
        InspectCommandError::Summary(_) => "failed to summarize the requested trace",
    };

    output::render_inspect_failure(
        inspection_target.as_str(),
        trace_ref.as_deref(),
        workspace_ref.as_deref(),
        terminal_reason,
        corrected_command(inspection_target),
    )
}

pub fn summarize_trace(
    trace_ref: impl AsRef<Path>,
    trace: &ExecutionTrace,
) -> Result<TraceSummaryView, TraceSummaryError> {
    let terminal_status = trace.terminal_status.ok_or(TraceSummaryError::MissingTerminalStatus)?;
    let terminal_reason =
        trace.terminal_reason.clone().ok_or(TraceSummaryError::MissingTerminalReason)?;
    let mut step_indexes: HashMap<String, usize> = HashMap::new();
    let mut executed_steps: Vec<TraceStepSummary> = Vec::new();
    let mut recovery_events: Vec<TraceRecoveryEvent> = Vec::new();

    for event in &trace.events {
        match event.event_type {
            TraceEventType::TaskStarted | TraceEventType::TerminalRecorded => {}
            TraceEventType::FlowSelected => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: format!(
                        "{} @ {}",
                        event
                            .payload
                            .get("flow_name")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-flow"),
                        event
                            .payload
                            .get("current_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage")
                    ),
                    related_step_id: None,
                });
            }
            TraceEventType::StageTransitioned => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: format!(
                        "{} -> {}",
                        event
                            .payload
                            .get("from_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage"),
                        event
                            .payload
                            .get("to_stage_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown-stage")
                    ),
                    related_step_id: event.step_id.clone(),
                });
            }
            TraceEventType::StepStarted => {
                let step_id = event
                    .step_id
                    .clone()
                    .ok_or(TraceSummaryError::MissingStepId(event.event_type))?;
                let step_kind = parse_step_kind(
                    event
                        .payload
                        .get("step_kind")
                        .and_then(|value| value.as_str())
                        .ok_or_else(|| TraceSummaryError::MissingStepKind(step_id.clone()))?,
                )?;

                if let Some(index) = step_indexes.get(&step_id) {
                    executed_steps[*index].attempts += 1;
                } else {
                    step_indexes.insert(step_id.clone(), executed_steps.len());
                    executed_steps.push(TraceStepSummary {
                        step_id,
                        step_kind,
                        attempts: 1,
                        final_status: StepStatus::Running,
                        headline: "started".to_string(),
                    });
                }
            }
            TraceEventType::StepCompleted => {
                let step_id = event
                    .step_id
                    .clone()
                    .ok_or(TraceSummaryError::MissingStepId(event.event_type))?;
                let final_status = match event
                    .payload
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                {
                    "succeeded" => StepStatus::Succeeded,
                    "failed" => StepStatus::Failed,
                    _ => StepStatus::Running,
                };

                let index = *step_indexes
                    .get(&step_id)
                    .ok_or_else(|| TraceSummaryError::MissingStartedStep(step_id.clone()))?;
                let headline = match final_status {
                    StepStatus::Succeeded => {
                        success_headline(&event.payload, executed_steps[index].attempts)
                    }
                    StepStatus::Failed => {
                        failure_headline(&event.payload, executed_steps[index].attempts)
                    }
                    _ => "completed".to_string(),
                };
                executed_steps[index].final_status = final_status;
                executed_steps[index].headline = headline;
            }
            TraceEventType::RetryScheduled
            | TraceEventType::StageRetryScheduled
            | TraceEventType::Replanned
            | TraceEventType::StageReplanned
            | TraceEventType::StageFailed => {
                recovery_events.push(TraceRecoveryEvent {
                    event_type: event.event_type,
                    trigger: event
                        .payload
                        .get("reason")
                        .and_then(|value| value.as_str())
                        .unwrap_or("recovery event")
                        .to_string(),
                    related_step_id: event.step_id.clone(),
                });
            }
        }
    }

    Ok(TraceSummaryView {
        trace_ref: trace_ref.as_ref().to_string_lossy().into_owned(),
        goal: trace.goal.clone(),
        executed_steps,
        recovery_events,
        terminal_status,
        terminal_reason,
        duration: trace.duration_millis(),
    })
}

fn load_trace(
    trace: Option<&Path>,
    workspace: Option<&Path>,
) -> Result<(TraceResolutionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    let session_trace_ref = workspace.map(resolve_session_trace_ref).transpose()?.flatten();
    let (target, trace_path) = resolve_trace_path(trace, workspace, session_trace_ref.as_deref())?;

    let trace = match target {
        TraceResolutionTarget::LatestWorkspaceTrace => {
            let workspace_path =
                workspace.expect("workspace is required for latest workspace trace resolution");
            let store = FileTraceStore::for_workspace(workspace_path);
            store.load(&trace_path)?
        }
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::SessionTraceRef => {
            let store = FileTraceStore::new(trace_path.parent().unwrap_or_else(|| Path::new(".")));
            store.load(&trace_path)?
        }
    };

    Ok((target, trace_path, trace))
}

fn resolve_session_trace_ref(workspace: &Path) -> Result<Option<String>, InspectCommandError> {
    match FileSessionStore::for_workspace(workspace).load() {
        Ok(Some(record)) => Ok(record.latest_trace_ref),
        Ok(None) => Ok(None),
        Err(SessionStoreError::InvalidRecord(message)) => Err(InspectCommandError::InvalidSession(
            format!("active session is invalid: {message}"),
        )),
        Err(error) => Err(InspectCommandError::SessionStore(error)),
    }
}

pub fn resolve_trace_path(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    session_trace_ref: Option<&str>,
) -> Result<(TraceResolutionTarget, PathBuf), InspectCommandError> {
    if let Some(trace_path) = trace {
        return Ok((TraceResolutionTarget::ExplicitTrace, trace_path.to_path_buf()));
    }

    if let Some(session_trace_ref) = session_trace_ref {
        return Ok((TraceResolutionTarget::SessionTraceRef, PathBuf::from(session_trace_ref)));
    }

    let Some(workspace_path) = workspace else {
        return Err(InspectCommandError::MissingTraceReference);
    };

    let store = FileTraceStore::for_workspace(workspace_path);
    let Some(trace_path) = store.latest()? else {
        return Err(InspectCommandError::MissingLatestTrace);
    };
    Ok((TraceResolutionTarget::LatestWorkspaceTrace, trace_path))
}

fn inspection_target_for(trace: Option<&Path>, workspace: Option<&Path>) -> TraceResolutionTarget {
    if trace.is_some() {
        TraceResolutionTarget::ExplicitTrace
    } else if workspace.is_some() {
        TraceResolutionTarget::LatestWorkspaceTrace
    } else {
        TraceResolutionTarget::ExplicitTrace
    }
}

fn corrected_command(inspection_target: TraceResolutionTarget) -> &'static str {
    match inspection_target {
        TraceResolutionTarget::ExplicitTrace | TraceResolutionTarget::SessionTraceRef => {
            "cargo run --bin synod -- inspect --trace <trace>"
        }
        TraceResolutionTarget::LatestWorkspaceTrace => {
            "cargo run --bin synod -- inspect --workspace <workspace>"
        }
    }
}

fn success_headline(payload: &serde_json::Value, attempts: usize) -> String {
    if let Some(changed_files) = payload
        .get("output")
        .and_then(|output| output.get("changed_files"))
        .and_then(|value| value.as_array())
    {
        let changed_files =
            changed_files.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
        if !changed_files.is_empty() {
            return format!("updated {} after {attempts} attempt(s)", changed_files.join(", "));
        }
    }

    if let Some(validation) = payload.get("output").and_then(|output| output.get("validation")) {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let succeeded =
            validation.get("succeeded").and_then(|value| value.as_bool()).unwrap_or(false);
        return format!(
            "validation {} after {attempts} attempt(s) via {command}",
            if succeeded { "passed" } else { "failed" }
        );
    }

    format!("succeeded after {attempts} attempt(s)")
}

fn failure_headline(payload: &serde_json::Value, attempts: usize) -> String {
    if let Some(validation) =
        payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let exit_code = validation.get("exit_code").and_then(|value| value.as_i64()).unwrap_or(-1);
        return format!(
            "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code})"
        );
    }

    format!("failed after {attempts} attempt(s)")
}

fn parse_step_kind(raw: &str) -> Result<StepKind, TraceSummaryError> {
    match raw {
        "agent" => Ok(StepKind::Agent),
        "tool" => Ok(StepKind::Tool),
        "decision" => Ok(StepKind::Decision),
        other => Err(TraceSummaryError::UnknownStepKind(other.to_string())),
    }
}

#[derive(Debug, Error)]
pub enum InspectCommandError {
    #[error("inspect requires --trace or --workspace")]
    MissingTraceReference,
    #[error("no persisted trace could be found for the selected workspace")]
    MissingLatestTrace,
    #[error("failed to read the active session: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("{0}")]
    InvalidSession(String),
    #[error("failed to read the requested trace: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("failed to summarize the requested trace: {0}")]
    Summary(#[from] TraceSummaryError),
}

#[derive(Debug, Error)]
pub enum TraceSummaryError {
    #[error("trace is missing a terminal status")]
    MissingTerminalStatus,
    #[error("trace is missing a terminal reason")]
    MissingTerminalReason,
    #[error("trace event {0:?} is missing a step id")]
    MissingStepId(TraceEventType),
    #[error("trace step '{0}' is missing its step kind payload")]
    MissingStepKind(String),
    #[error("trace step '{0}' completed without a matching start event")]
    MissingStartedStep(String),
    #[error("trace step kind '{0}' is unknown")]
    UnknownStepKind(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        InspectCommandError, TraceResolutionTarget, TraceSummaryError, corrected_command,
        failure_headline, inspection_target_for, render_error, resolve_session_trace_ref,
        success_headline, summarize_trace,
    };
    use crate::adapters::session_store::SessionStoreError;
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task::{TaskStatus, TerminalReason};
    use crate::domain::trace::{ExecutionTrace, TraceEventType};

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        workspace
    }

    fn terminal_trace() -> ExecutionTrace {
        let mut trace = ExecutionTrace::new("task-inspect", "session-inspect", "Inspect trace");
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason =
            Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "failed", None));
        trace.ended_at = Some(trace.started_at + 1);
        trace
    }

    #[test]
    fn render_error_maps_session_store_and_summary_failures() {
        let workspace = PathBuf::from("/tmp/workspace");
        let session_error = InspectCommandError::SessionStore(SessionStoreError::Read(
            std::io::Error::other("read failed"),
        ));
        let session_text = render_error(None, Some(workspace.as_path()), &session_error);
        assert!(session_text.contains("failed to read the active session"), "{session_text}");

        let summary_error = InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus);
        let summary_text = render_error(None, Some(workspace.as_path()), &summary_error);
        assert!(summary_text.contains("failed to summarize the requested trace"), "{summary_text}");
    }

    #[test]
    fn summarize_trace_reports_missing_step_id_and_step_kind() {
        let mut missing_step_id = terminal_trace();
        missing_step_id.record_event(TraceEventType::StepStarted, None, 0, json!({}));
        assert!(matches!(
            summarize_trace("/tmp/trace.json", &missing_step_id).unwrap_err(),
            TraceSummaryError::MissingStepId(TraceEventType::StepStarted)
        ));

        let mut missing_step_kind = terminal_trace();
        missing_step_kind.record_event(
            TraceEventType::StepStarted,
            Some("verify".to_string()),
            0,
            json!({}),
        );
        assert!(matches!(
            summarize_trace("/tmp/trace.json", &missing_step_kind).unwrap_err(),
            TraceSummaryError::MissingStepKind(step_id) if step_id == "verify"
        ));
    }

    #[test]
    fn resolve_session_trace_ref_maps_invalid_records_to_invalid_session_errors() {
        let workspace = temp_workspace("synod-inspect-invalid-session");
        let invalid_record = ActiveSessionRecord {
            session_id: "session-inspect".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            active_flow: None,
            active_task: None,
            latest_status: SessionStatus::GoalCaptured,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 20,
        };
        fs::write(
            workspace.join(".synod/session.json"),
            serde_json::to_vec_pretty(&invalid_record).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            resolve_session_trace_ref(&workspace).unwrap_err(),
            InspectCommandError::InvalidSession(message) if message.contains("active session is invalid")
        ));
    }

    #[test]
    fn inspection_helpers_cover_default_headlines_and_command_targets() {
        assert_eq!(
            inspection_target_for(Some(PathBuf::from("trace.json").as_path()), None),
            TraceResolutionTarget::ExplicitTrace
        );
        assert_eq!(
            inspection_target_for(None, Some(PathBuf::from("workspace").as_path())),
            TraceResolutionTarget::LatestWorkspaceTrace
        );
        assert_eq!(
            corrected_command(TraceResolutionTarget::SessionTraceRef),
            "cargo run --bin synod -- inspect --trace <trace>"
        );
        assert_eq!(success_headline(&json!({}), 2), "succeeded after 2 attempt(s)");
        assert_eq!(failure_headline(&json!({}), 1), "failed after 1 attempt(s)");
    }

    #[test]
    fn summarize_trace_collects_recovery_events_and_headlines_from_validation_payloads() {
        let mut trace = terminal_trace();
        trace.record_event(
            TraceEventType::StepStarted,
            Some("verify".to_string()),
            0,
            json!({"step_kind": "tool"}),
        );
        trace.record_event(
            TraceEventType::StepCompleted,
            Some("verify".to_string()),
            0,
            json!({
                "status": "failed",
                "evidence": {
                    "validation_record": {
                        "command": "cargo test --quiet",
                        "exit_code": 101,
                        "stdout": "",
                        "stderr": "",
                        "succeeded": false
                    }
                }
            }),
        );
        trace.record_event(
            TraceEventType::StageRetryScheduled,
            Some("verify".to_string()),
            0,
            json!({}),
        );

        let summary = summarize_trace("/tmp/trace.json", &trace).unwrap();

        assert_eq!(
            summary.executed_steps[0].headline,
            "validation failed after 1 attempt(s) via cargo test --quiet (exit_code=101)"
        );
        assert_eq!(summary.recovery_events[0].related_step_id.as_deref(), Some("verify"));
        assert_eq!(summary.recovery_events[0].trigger, "recovery event");

        let success = success_headline(
            &json!({
                "output": {
                    "validation": {
                        "command": "cargo test --quiet",
                        "succeeded": true
                    }
                }
            }),
            2,
        );
        assert_eq!(success, "validation passed after 2 attempt(s) via cargo test --quiet");
    }
}
