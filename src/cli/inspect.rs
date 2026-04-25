use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

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
enum InspectionTarget {
    ExplicitTrace,
    LatestWorkspaceTrace,
}

impl InspectionTarget {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitTrace => "explicit-trace",
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
    let inspection_target = inspection_target_for(trace, workspace);
    let trace_ref = trace.map(|path| path.to_string_lossy().into_owned());
    let workspace_ref = workspace.map(|path| path.to_string_lossy().into_owned());
    let terminal_reason = match error {
        InspectCommandError::MissingTraceReference => "inspect requires --trace or --workspace",
        InspectCommandError::MissingLatestTrace | InspectCommandError::TraceStore(_) => {
            "failed to read the requested trace"
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
                        format!("succeeded after {} attempt(s)", executed_steps[index].attempts)
                    }
                    StepStatus::Failed => {
                        format!("failed after {} attempt(s)", executed_steps[index].attempts)
                    }
                    _ => "completed".to_string(),
                };
                executed_steps[index].final_status = final_status;
                executed_steps[index].headline = headline;
            }
            TraceEventType::RetryScheduled | TraceEventType::Replanned => {
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
) -> Result<(InspectionTarget, PathBuf, ExecutionTrace), InspectCommandError> {
    if let Some(trace_path) = trace {
        let store = FileTraceStore::new(trace_path.parent().unwrap_or_else(|| Path::new(".")));
        let trace = store.load(trace_path)?;
        return Ok((InspectionTarget::ExplicitTrace, trace_path.to_path_buf(), trace));
    }

    let Some(workspace_path) = workspace else {
        return Err(InspectCommandError::MissingTraceReference);
    };

    let store = FileTraceStore::for_workspace(workspace_path);
    let Some(trace_path) = store.latest()? else {
        return Err(InspectCommandError::MissingLatestTrace);
    };
    let trace = store.load(&trace_path)?;
    Ok((InspectionTarget::LatestWorkspaceTrace, trace_path, trace))
}

fn inspection_target_for(trace: Option<&Path>, workspace: Option<&Path>) -> InspectionTarget {
    if trace.is_some() {
        InspectionTarget::ExplicitTrace
    } else if workspace.is_some() {
        InspectionTarget::LatestWorkspaceTrace
    } else {
        InspectionTarget::ExplicitTrace
    }
}

fn corrected_command(inspection_target: InspectionTarget) -> &'static str {
    match inspection_target {
        InspectionTarget::ExplicitTrace => "cargo run --bin synod -- inspect --trace <trace>",
        InspectionTarget::LatestWorkspaceTrace => {
            "cargo run --bin synod -- inspect --workspace <workspace>"
        }
    }
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
