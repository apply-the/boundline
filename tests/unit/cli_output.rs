use std::path::PathBuf;

use synod::cli::diagnostics::{DiagnosticsCheck, DiagnosticsReport, DiagnosticsStatus};
use synod::cli::output::{
    CommandExitCode, command_name, render_diagnostics, render_trace_summary,
    validation_error_message,
};
use synod::cli::{
    CliValidationError, CommandExitStatus, CommandName, DeveloperCommand, DeveloperCommandSession,
};
use synod::domain::limits::TerminalCondition;
use synod::domain::step::{StepKind, StepStatus};
use synod::domain::task::{TaskStatus, TerminalReason};
use synod::domain::trace::{
    TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
};

#[test]
fn exit_codes_match_the_command_contract() {
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::Succeeded).code(), 0);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::NonSuccess).code(), 1);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::InvalidInvocation).code(), 2);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::TraceReadFailure).code(), 3);
}

#[test]
fn command_names_render_from_subcommands() {
    let command = DeveloperCommand::Demo { workspace: PathBuf::from("/tmp/workspace") };
    assert_eq!(command_name(&command), "demo");
    assert_eq!(command.name(), CommandName::Demo);
}

#[test]
fn run_session_requires_a_non_empty_goal() {
    let command = DeveloperCommand::Run {
        workspace: PathBuf::from("/tmp/workspace"),
        goal: "   ".to_string(),
    };
    let session = DeveloperCommandSession::from_command(&command);

    assert_eq!(session.validate(), Err(CliValidationError::MissingGoal));
}

#[test]
fn inspect_session_requires_trace_or_workspace() {
    let session = DeveloperCommandSession {
        command_name: CommandName::Inspect,
        workspace_ref: None,
        goal: None,
        trace_ref: None,
        started_at: 0,
        completed_at: None,
        exit_status: None,
        trace_location: None,
    };

    assert_eq!(session.validate(), Err(CliValidationError::MissingTraceSelection));
    assert_eq!(
        validation_error_message(&CliValidationError::MissingTraceSelection),
        "inspect requires --trace or --workspace"
    );
}

#[test]
fn diagnostics_renderer_lists_check_names_and_actions() {
    let report = DiagnosticsReport {
        workspace_ref: "/tmp/workspace".to_string(),
        checks: vec![
            DiagnosticsCheck {
                name: "workspace_exists".to_string(),
                status: DiagnosticsStatus::Passed,
                message: "workspace exists".to_string(),
            },
            DiagnosticsCheck {
                name: "trace_store".to_string(),
                status: DiagnosticsStatus::Failed,
                message: "fix the trace directory".to_string(),
            },
        ],
        ready: false,
        missing_prerequisites: vec!["trace_store".to_string()],
        suggested_actions: vec!["fix the trace directory".to_string()],
    };

    let rendered = render_diagnostics(&report);

    assert!(rendered.contains("doctor: not ready"));
    assert!(rendered.contains("workspace_exists"));
    assert!(rendered.contains("trace_store"));
    assert!(rendered.contains("actions:"));
    assert!(rendered.contains("fix the trace directory"));
}

#[test]
fn trace_summary_renderer_mentions_steps_recovery_and_terminal_reason() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/workspace/.synod/traces/task.json".to_string(),
        goal: "Inspect a recorded run".to_string(),
        executed_steps: vec![
            TraceStepSummary {
                step_id: "analyze".to_string(),
                step_kind: StepKind::Agent,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 1 attempt(s)".to_string(),
            },
            TraceStepSummary {
                step_id: "code".to_string(),
                step_kind: StepKind::Agent,
                attempts: 2,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 2 attempt(s)".to_string(),
            },
        ],
        recovery_events: vec![TraceRecoveryEvent {
            event_type: TraceEventType::RetryScheduled,
            trigger: "retrying step code within remaining retry budget".to_string(),
            related_step_id: Some("code".to_string()),
        }],
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied after step verify",
            None,
        ),
        duration: Some(42),
    };

    let rendered = render_trace_summary(&summary);

    assert!(rendered.contains("trace: /tmp/workspace/.synod/traces/task.json"));
    assert!(rendered.contains("step analyze (agent) succeeded [1 attempt(s)]"));
    assert!(rendered.contains("step code (agent) succeeded [2 attempt(s)]"));
    assert!(rendered.contains("retry: retrying step code within remaining retry budget"));
    assert!(rendered.contains("terminal_reason: goal satisfied after step verify"));
    assert!(rendered.contains("duration_ms: 42"));
}
