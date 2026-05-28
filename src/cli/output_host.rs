use std::io::{self, IsTerminal};

use serde::Serialize;

use crate::cli::orchestrate::OrchestrateEventEnvelope;
use crate::cli::{CommandExitStatus, DeveloperCommand};
use crate::domain::session::SessionStatusView;
use crate::domain::trace::TraceSummaryView;

/// Exit-code families used by host-facing command wrappers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExitCode {
    Success,
    NonSuccess,
    InvalidInvocation,
    TraceReadFailure,
}

impl CommandExitCode {
    pub const fn code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::NonSuccess => 1,
            Self::InvalidInvocation => 2,
            Self::TraceReadFailure => 3,
        }
    }

    pub const fn for_status(status: CommandExitStatus) -> Self {
        match status {
            CommandExitStatus::Succeeded => Self::Success,
            CommandExitStatus::NonSuccess => Self::NonSuccess,
            CommandExitStatus::InvalidInvocation => Self::InvalidInvocation,
            CommandExitStatus::TraceReadFailure => Self::TraceReadFailure,
        }
    }
}

/// Structured host payload used when commands need machine-readable output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HostCommandEnvelope {
    pub command_name: String,
    pub exit_status: String,
    pub rendered_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<SessionStatusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_summary: Option<TraceSummaryView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputPresentation {
    Plain,
    Rich,
}

fn command_exit_status_label(status: CommandExitStatus) -> &'static str {
    match status {
        CommandExitStatus::Succeeded => "succeeded",
        CommandExitStatus::NonSuccess => "non_success",
        CommandExitStatus::InvalidInvocation => "invalid_invocation",
        CommandExitStatus::TraceReadFailure => "trace_read_failure",
    }
}

/// Renders a command result as structured JSON for host integrations.
pub fn render_host_command_json(
    command_name: &str,
    exit_status: CommandExitStatus,
    rendered_output: &str,
    trace_location: Option<&str>,
    session_status: Option<&SessionStatusView>,
    trace_summary: Option<&TraceSummaryView>,
) -> String {
    match serde_json::to_string_pretty(&HostCommandEnvelope {
        command_name: command_name.to_string(),
        exit_status: command_exit_status_label(exit_status).to_string(),
        rendered_output: rendered_output.to_string(),
        trace_location: trace_location.map(str::to_string),
        session_status: session_status.cloned(),
        trace_summary: trace_summary.cloned(),
    }) {
        Ok(rendered) => rendered,
        Err(error) => serde_json::json!({
            "command_name": command_name,
            "exit_status": command_exit_status_label(exit_status),
            "rendered_output": rendered_output,
            "trace_location": trace_location,
            "session_status": session_status,
            "trace_summary": trace_summary,
            "serialization_error": error.to_string(),
        })
        .to_string(),
    }
}

/// Renders one orchestrator event as a compact NDJSON frame.
pub fn render_orchestrate_event_json(event: &OrchestrateEventEnvelope) -> String {
    match serde_json::to_string(event) {
        Ok(rendered) => rendered,
        Err(error) => serde_json::json!({
            "event_kind": "error",
            "message": "failed to serialize orchestrate event",
            "serialization_error": error.to_string(),
        })
        .to_string(),
    }
}

/// Renders the full orchestrator event stream as newline-delimited JSON.
pub fn render_orchestrate_stream_json(events: &[OrchestrateEventEnvelope]) -> String {
    events.iter().map(render_orchestrate_event_json).collect::<Vec<_>>().join("\n")
}

/// Returns the stable CLI command name used in output and host envelopes.
pub fn command_name(command: &DeveloperCommand) -> &'static str {
    match command {
        DeveloperCommand::Doctor { .. } => "doctor",
        DeveloperCommand::Checkpoint { .. } => "checkpoint",
        DeveloperCommand::Orchestrate { .. } => "orchestrate",
        DeveloperCommand::Goal { .. } => "goal",
        DeveloperCommand::Flow { .. } => "flow",
        DeveloperCommand::Plan { .. } => "plan",
        DeveloperCommand::Step { .. } => "step",
        DeveloperCommand::Run { .. } => "run",
        DeveloperCommand::Workflow { .. } => "workflow",
        DeveloperCommand::Inspect { .. } => "inspect",
        DeveloperCommand::Status { .. } => "status",
        DeveloperCommand::Next { .. } => "next",
        DeveloperCommand::Continue { .. } => "continue",
        DeveloperCommand::Session { .. } => "session",
        DeveloperCommand::Govern { .. } => "govern",
        DeveloperCommand::Assistant { .. } => "assistant",
        DeveloperCommand::Init { .. } => "init",
        DeveloperCommand::Update { .. } => "update",
        DeveloperCommand::Config { .. } => "config",
        DeveloperCommand::Cluster { .. } => "cluster",
        DeveloperCommand::Models { .. } => "models",
    }
}

pub(crate) fn stdout_presentation() -> OutputPresentation {
    if io::stdout().is_terminal() { OutputPresentation::Rich } else { OutputPresentation::Plain }
}

pub(crate) fn push_output_section(
    lines: &mut Vec<String>,
    presentation: OutputPresentation,
    title: &str,
    section_lines: Vec<String>,
) {
    if section_lines.is_empty() {
        return;
    }

    if matches!(presentation, OutputPresentation::Rich) && !lines.is_empty() {
        lines.push(String::new());
    }
    lines.push(format!("{title}:"));
    lines.extend(section_lines);
}
