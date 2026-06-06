use std::io::{self, IsTerminal};

use serde::Serialize;

use crate::cli::orchestrate::OrchestrateEventEnvelope;
use crate::cli::{CommandExitStatus, DeveloperCommand};
use crate::domain::session::SessionStatusView;
use crate::domain::trace::TraceSummaryView;

use super::runtime::{capability_provider_output_projection, framework_adapter_output_projection};

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
    pub framework_adapter_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_execution_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_config_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_interactive_resolution: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_value_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_discovery_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_discovery_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_activation_required: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_supported_transports: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_compatibility_gate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework_adapter_blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_activation_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_capability_ids: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_setup_requirements: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_provider_summary: Option<String>,
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
    let adapter_projection =
        session_status.map(|view| framework_adapter_output_projection(&view.workspace_ref));
    let provider_projection =
        session_status.map(|view| capability_provider_output_projection(&view.workspace_ref));
    match serde_json::to_string_pretty(&HostCommandEnvelope {
        command_name: command_name.to_string(),
        exit_status: command_exit_status_label(exit_status).to_string(),
        rendered_output: rendered_output.to_string(),
        trace_location: trace_location.map(str::to_string),
        session_status: session_status.cloned(),
        framework_adapter_status: adapter_projection
            .as_ref()
            .map(|projection| projection.status.clone()),
        framework_adapter_execution_source: adapter_projection
            .as_ref()
            .map(|projection| projection.execution_source.clone()),
        framework_adapter_id: adapter_projection
            .as_ref()
            .and_then(|projection| projection.adapter_id.clone()),
        framework_adapter_config_state: adapter_projection
            .as_ref()
            .and_then(|projection| projection.config_state.clone()),
        framework_adapter_interactive_resolution: adapter_projection
            .as_ref()
            .and_then(|projection| projection.interactive_resolution),
        framework_adapter_value_count: adapter_projection
            .as_ref()
            .and_then(|projection| projection.value_count),
        framework_adapter_discovery_state: adapter_projection
            .as_ref()
            .and_then(|projection| projection.discovery_state.clone()),
        framework_adapter_discovery_hint: adapter_projection
            .as_ref()
            .and_then(|projection| projection.discovery_hint.clone()),
        framework_adapter_activation_required: adapter_projection
            .as_ref()
            .and_then(|projection| projection.activation_required.clone()),
        framework_adapter_supported_transports: adapter_projection
            .as_ref()
            .and_then(|projection| projection.supported_transports.clone()),
        framework_adapter_compatibility_gate: adapter_projection
            .as_ref()
            .and_then(|projection| projection.compatibility_gate.clone()),
        framework_adapter_blocked_reason: adapter_projection
            .as_ref()
            .and_then(|projection| projection.blocked_reason.clone()),
        capability_provider_status: provider_projection
            .as_ref()
            .map(|projection| projection.status.clone()),
        capability_provider_id: provider_projection
            .as_ref()
            .and_then(|projection| projection.provider_id.clone()),
        capability_provider_activation_state: provider_projection
            .as_ref()
            .and_then(|projection| projection.activation_state.clone()),
        capability_provider_capability_ids: provider_projection
            .as_ref()
            .and_then(|projection| projection.capability_ids.clone()),
        capability_provider_setup_requirements: provider_projection
            .as_ref()
            .and_then(|projection| projection.setup_requirements.clone()),
        capability_provider_summary: provider_projection
            .as_ref()
            .and_then(|projection| projection.summary.clone()),
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
        DeveloperCommand::Probe { .. } => "probe",
        DeveloperCommand::Step { .. } => "step",
        DeveloperCommand::Run { .. } => "run",
        DeveloperCommand::Workflow { .. } => "workflow",
        DeveloperCommand::Index { .. } => "index",
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
        DeveloperCommand::Adapter { .. } => "adapter",
        DeveloperCommand::Provider { .. } => "provider",
        DeveloperCommand::Cluster { .. } => "cluster",
        DeveloperCommand::Models { .. } => "models",
        DeveloperCommand::Council { .. } => "council",
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

#[cfg(test)]
mod tests {
    use super::{
        CommandExitStatus, DeveloperCommand, OrchestrateEventEnvelope, OutputPresentation,
        command_name, push_output_section, render_host_command_json, render_orchestrate_event_json,
        render_orchestrate_stream_json, stdout_presentation,
    };
    use crate::cli::{
        AdapterSubcommand, ClusterSubcommand, ConfigSubcommand, IndexSubcommand,
        ModelsAuthSubcommand, ModelsSubcommand, ProviderSubcommand,
    };
    use crate::domain::configuration::InitConfigScope;

    fn minimal_event(kind: &str, msg: &str) -> OrchestrateEventEnvelope {
        OrchestrateEventEnvelope {
            event_id: "evt-001".to_string(),
            timestamp_ms: 0,
            event_kind: kind.to_string(),
            audit: None,
            actor_kind: None,
            actor_name: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            decision_family: None,
            review_step: None,
            vote_summary: None,
            adjudication_summary: None,
            governance_mode: None,
            session_ref: None,
            phase_kind: None,
            stage_key: None,
            message: msg.to_string(),
            artifact: None,
            phase_request: None,
            instruction: None,
            resume_command: None,
            assistant_resume_command: None,
            next_command: None,
            assistant_next_command: None,
            session_status: None,
            trace_summary: None,
        }
    }

    #[test]
    fn render_host_command_json_produces_valid_json_with_all_fields() {
        let json = render_host_command_json(
            "orchestrate",
            CommandExitStatus::Succeeded,
            "done",
            Some("/tmp/trace.json"),
            None,
            None,
        );
        assert!(json.contains("\"command_name\""));
        assert!(json.contains("orchestrate"));
        assert!(json.contains("succeeded"));
    }

    #[test]
    fn render_orchestrate_event_json_produces_valid_json_for_event() {
        let event = minimal_event("session_start", "Session started");
        let json = render_orchestrate_event_json(&event);
        assert!(json.contains("session_start"));
        assert!(json.contains("Session started"));
    }

    #[test]
    fn render_orchestrate_stream_json_joins_events_with_newlines() {
        let events =
            vec![minimal_event("step_start", "step a"), minimal_event("step_done", "step b")];
        let output = render_orchestrate_stream_json(&events);
        assert!(output.contains('\n'));
        assert!(output.contains("step_start"));
        assert!(output.contains("step_done"));
    }

    #[test]
    fn command_name_returns_continue_for_continue_variant() {
        let cmd = DeveloperCommand::Continue { workspace: None, cluster: None, session: None };
        assert_eq!(command_name(&cmd), "continue");
    }

    #[test]
    fn command_name_returns_init_for_init_variant() {
        let cmd = DeveloperCommand::Init {
            scope: InitConfigScope::Workspace,
            workspace: "/tmp/ws".into(),
            non_interactive: false,
            template: None,
            ollama_profile: None,
            assistant: Vec::new(),
            adapter: None,
            ide: Vec::new(),
            auto_approve: None,
            semantic_index_hook_action: None,
            domain: Vec::new(),
            domain_standard: Vec::new(),
            context_binding: Vec::new(),
            required_context_binding: Vec::new(),
            canon_mode_selection: None,
            risk: None,
            zone: None,
            owner: None,
            export_docs: false,
            refresh: false,
            diff: false,
            to: None,
            route: Vec::new(),
            force: false,
        };
        assert_eq!(command_name(&cmd), "init");
    }

    #[test]
    fn command_name_returns_index_for_index_variant() {
        let cmd = DeveloperCommand::Index { command: IndexSubcommand::Status { workspace: None } };
        assert_eq!(command_name(&cmd), "index");
    }

    #[test]
    fn command_name_covers_recent_command_variants() {
        let cases = [
            (
                DeveloperCommand::Update {
                    workspace: ".".into(),
                    target: Vec::new(),
                    ide: Vec::new(),
                    auto_approve: None,
                    template: None,
                    diff: false,
                    apply: false,
                    adopt: false,
                    prune: false,
                    status: false,
                    force: false,
                },
                "update",
            ),
            (
                DeveloperCommand::Config {
                    command: ConfigSubcommand::Show {
                        workspace: Some(".".into()),
                        cluster: None,
                        scope: None,
                    },
                },
                "config",
            ),
            (
                DeveloperCommand::Adapter {
                    command: AdapterSubcommand::Show { workspace: Some(".".into()) },
                },
                "adapter",
            ),
            (
                DeveloperCommand::Provider {
                    command: ProviderSubcommand::Show { workspace: Some(".".into()) },
                },
                "provider",
            ),
            (
                DeveloperCommand::Cluster {
                    command: ClusterSubcommand::Status { workspace: ".".into() },
                },
                "cluster",
            ),
            (
                DeveloperCommand::Models {
                    command: ModelsSubcommand::Auth { command: ModelsAuthSubcommand::Status },
                },
                "models",
            ),
        ];
        for (command, expected) in cases {
            assert_eq!(command_name(&command), expected);
        }
    }

    #[test]
    fn stdout_presentation_returns_a_valid_variant() {
        let p = stdout_presentation();
        assert!(p == OutputPresentation::Plain || p == OutputPresentation::Rich);
    }

    #[test]
    fn push_output_section_inserts_blank_line_separator_in_rich_mode() {
        let mut lines = vec!["existing line".to_string()];
        push_output_section(
            &mut lines,
            OutputPresentation::Rich,
            "Section",
            vec!["content".to_string()],
        );
        assert!(lines.contains(&String::new()), "expected blank line separator in rich output");
        assert!(lines.iter().any(|l| l.contains("Section:")));
    }

    #[test]
    fn push_output_section_skips_blank_separator_when_lines_is_empty() {
        let mut lines: Vec<String> = Vec::new();
        push_output_section(
            &mut lines,
            OutputPresentation::Rich,
            "Section",
            vec!["content".to_string()],
        );
        assert!(!lines.contains(&String::new()), "no blank separator when starting fresh");
        assert_eq!(lines[0], "Section:");
    }
}
