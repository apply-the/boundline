use serde_json::Value;

use crate::cli::diagnostics::{DiagnosticsReport, DiagnosticsStatus};
use crate::cli::{CliValidationError, CommandExitStatus, DeveloperCommand};
use crate::domain::session::{SessionStatus, SessionStatusView};
use crate::domain::step::{StepKind, StepStatus};
use crate::domain::task::{TaskRunResponse, TaskStatus};
use crate::domain::trace::{ExecutionTrace, TraceEventType, TraceSummaryView};

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

pub fn unimplemented_message(command: &DeveloperCommand) -> String {
    format!("`{}` is not implemented yet", command_name(command))
}

pub fn command_name(command: &DeveloperCommand) -> &'static str {
    match command {
        DeveloperCommand::Doctor { .. } => "doctor",
        DeveloperCommand::Start { .. } => "start",
        DeveloperCommand::Capture { .. } => "capture",
        DeveloperCommand::Plan { .. } => "plan",
        DeveloperCommand::Step { .. } => "step",
        DeveloperCommand::Demo { .. } => "demo",
        DeveloperCommand::RunDemo { .. } => "run-demo",
        DeveloperCommand::Run { .. } => "run",
        DeveloperCommand::Inspect { .. } => "inspect",
        DeveloperCommand::Status { .. } => "status",
        DeveloperCommand::Next { .. } => "next",
    }
}

pub fn validation_error_message(error: &CliValidationError) -> String {
    error.to_string()
}

pub fn render_diagnostics(report: &DiagnosticsReport) -> String {
    let readiness = if report.ready { "ready" } else { "not ready" };
    let mut lines = vec![
        format!("doctor: {readiness} for workspace {}", report.workspace_ref),
        format!("assistant_hint: Diagnostic output format is optimized for chat parsing."),
    ];

    for check in &report.checks {
        let status = match check.status {
            DiagnosticsStatus::Passed => "passed",
            DiagnosticsStatus::Failed => "failed",
        };
        lines.push(format!("- {}: {} - {}", check.name, status, check.message));
    }

    if !report.suggested_actions.is_empty() {
        lines.push("actions:".to_string());
        for action in &report.suggested_actions {
            lines.push(format!("- {action}"));
        }
    }

    lines.join("\n")
}

pub fn render_run_trace(
    command_name: &str,
    trace: Option<&ExecutionTrace>,
    response: &TaskRunResponse,
    next_command: &str,
) -> String {
    let mut lines = vec![format!("{command_name}: {}", response.terminal_reason.message)];

    if let Some(trace) = trace {
        lines.insert(0, format!("goal: {}", trace.goal));

        for event in &trace.events {
            match event.event_type {
                TraceEventType::TaskStarted | TraceEventType::TerminalRecorded => {}
                TraceEventType::StepStarted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let step_kind =
                        event.payload.get("step_kind").and_then(Value::as_str).unwrap_or("step");
                    lines.push(format!("step {step_id} ({step_kind}) started"));
                }
                TraceEventType::StepCompleted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let status =
                        event.payload.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("step {step_id} {status}"));
                }
                TraceEventType::RetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("retry for {step_id}: {reason}"));
                }
                TraceEventType::Replanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("replan after {step_id}: {reason}"));
                }
            }
        }
    }

    lines.push(format!("terminal_status: {}", task_status_text(response.terminal_status)));
    lines.push(format!("terminal_reason: {}", response.terminal_reason.message));
    lines.push(format!("trace: {}", response.trace_location));
    lines.push(format!("next_command: {next_command}"));
    lines.join("\n")
}

pub fn render_trace_summary(
    summary: &TraceSummaryView,
    inspection_target: &str,
    next_command: &str,
) -> String {
    let mut lines = vec![
        format!("inspection_target: {inspection_target}"),
        format!("trace: {}", summary.trace_ref),
        format!("goal: {}", summary.goal),
    ];

    for step in &summary.executed_steps {
        lines.push(format!(
            "step {} ({}) {} [{} attempt(s)]",
            step.step_id,
            step_kind_text(step.step_kind),
            step_status_text(step.final_status),
            step.attempts,
        ));
    }

    for recovery in &summary.recovery_events {
        let label = match recovery.event_type {
            TraceEventType::RetryScheduled => "retry",
            TraceEventType::Replanned => "replan",
            _ => "recovery",
        };
        lines.push(format!("{label}: {}", recovery.trigger));
    }

    lines.push(format!("terminal_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    lines.push(format!("next_command: {next_command}"));

    if let Some(duration) = summary.duration {
        lines.push(format!("duration_ms: {duration}"));
    }

    lines.join("\n")
}

pub fn render_inspect_failure(
    inspection_target: &str,
    trace_ref: Option<&str>,
    workspace_ref: Option<&str>,
    terminal_reason: &str,
    corrected_command: &str,
) -> String {
    let mut lines = vec![
        "inspect: trace read failure".to_string(),
        format!("inspection_target: {inspection_target}"),
        format!("terminal_reason: {terminal_reason}"),
    ];

    if let Some(trace_ref) = trace_ref {
        lines.push(format!("trace: {trace_ref}"));
    }

    if let Some(workspace_ref) = workspace_ref {
        lines.push(format!("workspace_ref: {workspace_ref}"));
    }

    lines.push("next_command: /synod-inspect".to_string());
    lines.push(format!("corrected_command: {corrected_command}"));
    lines.join("\n")
}

pub fn render_session_status(view: &SessionStatusView) -> String {
    let mut lines = vec![
        format!("session_id: {}", view.session_id),
        format!("workspace_ref: {}", view.workspace_ref),
    ];

    if let Some(goal) = &view.goal {
        lines.push(format!("goal: {goal}"));
    }

    if let Some(plan_revision) = view.plan_revision {
        lines.push(format!("plan_revision: {plan_revision}"));
    }

    if let Some(current_step_index) = view.current_step_index {
        lines.push(format!("current_step_index: {current_step_index}"));
    }

    if let Some(current_step_id) = &view.current_step_id {
        lines.push(format!("current_step_id: {current_step_id}"));
    }

    lines.push(format!("latest_status: {}", session_status_text(view.latest_status)));

    if let Some(latest_trace_ref) = &view.latest_trace_ref {
        lines.push(format!("latest_trace_ref: {latest_trace_ref}"));
    }

    if let Some(next_command) = &view.next_command {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.push(format!("explanation: {}", view.explanation));
    lines.join("\n")
}

pub fn render_session_error(action: &str, message: &str, next_command: Option<&str>) -> String {
    let mut lines = vec![format!("{action}: session error"), format!("reason: {message}")];

    if let Some(next_command) = next_command {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.join("\n")
}

pub const fn next_command_after_run(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Succeeded => "/synod-status",
        TaskStatus::Planned
        | TaskStatus::Running
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "/synod-next",
    }
}

pub const fn next_command_after_inspect(_: TaskStatus) -> &'static str {
    "/synod-next"
}

fn task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

fn step_kind_text(kind: StepKind) -> &'static str {
    match kind {
        StepKind::Agent => "agent",
        StepKind::Tool => "tool",
        StepKind::Decision => "decision",
    }
}

fn step_status_text(status: StepStatus) -> &'static str {
    match status {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Succeeded => "succeeded",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
    }
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}
