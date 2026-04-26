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
        DeveloperCommand::Flow { .. } => "flow",
        DeveloperCommand::Plan { .. } => "plan",
        DeveloperCommand::Step { .. } => "step",
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
                TraceEventType::FlowSelected => {
                    let flow_name = event
                        .payload
                        .get("flow_name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-flow");
                    let stage_id = event
                        .payload
                        .get("current_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("flow {flow_name} selected at {stage_id}"));
                }
                TraceEventType::StageTransitioned => {
                    let from_stage = event
                        .payload
                        .get("from_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let to_stage = event
                        .payload
                        .get("to_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("stage {from_stage} -> {to_stage}"));
                }
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

                    if let Some(changed_files) = event
                        .payload
                        .get("output")
                        .and_then(|output| output.get("changed_files"))
                        .and_then(value_as_string_list)
                        && !changed_files.is_empty()
                    {
                        lines.push(format!("changed_files: {}", changed_files.join(", ")));
                    }

                    if let Some(validation_line) = validation_line_from_event(&event.payload) {
                        lines.push(validation_line);
                    }
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
                TraceEventType::StageRetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("stage retry for {step_id}: {reason}"));
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
                TraceEventType::StageReplanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("stage replan after {step_id}: {reason}"));
                }
                TraceEventType::StageFailed => {
                    let stage_id = event
                        .payload
                        .get("stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("stage failed");
                    lines.push(format!("stage {stage_id} failed: {reason}"));
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
            "step {} ({}) {} [{} attempt(s)] - {}",
            step.step_id,
            step_kind_text(step.step_kind),
            step_status_text(step.final_status),
            step.attempts,
            step.headline,
        ));
    }

    for recovery in &summary.recovery_events {
        let label = match recovery.event_type {
            TraceEventType::RetryScheduled => "retry",
            TraceEventType::StageRetryScheduled => "stage_retry",
            TraceEventType::Replanned => "replan",
            TraceEventType::StageReplanned => "stage_replan",
            TraceEventType::FlowSelected => "flow",
            TraceEventType::StageTransitioned => "stage",
            TraceEventType::StageFailed => "stage_failure",
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

    if let Some(active_flow) = &view.active_flow {
        lines.push(format!("active_flow: {active_flow}"));
    }

    if let Some(current_stage_id) = &view.current_stage_id {
        lines.push(format!("current_stage: {current_stage_id}"));
    }

    if let (Some(current_stage_index), Some(total_stages)) =
        (view.current_stage_index, view.total_stages)
    {
        lines.push(format!("stage_progress: {}/{}", current_stage_index + 1, total_stages));
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

    if let Some(latest_changed_files) = &view.latest_changed_files
        && !latest_changed_files.is_empty()
    {
        lines.push(format!("latest_changed_files: {}", latest_changed_files.join(", ")));
    }

    if let Some(latest_validation_status) = &view.latest_validation_status {
        lines.push(format!("latest_validation_status: {latest_validation_status}"));
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

fn value_as_string_list(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
    })
}

fn validation_line_from_event(payload: &Value) -> Option<String> {
    let validation =
        payload.get("output").and_then(|output| output.get("validation")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
        })?;
    let command = validation.get("command").and_then(Value::as_str).unwrap_or("validation");
    let succeeded = validation.get("succeeded").and_then(Value::as_bool).unwrap_or(false);
    let exit_code = validation.get("exit_code").and_then(Value::as_i64).unwrap_or(-1);
    Some(format!(
        "validation: {} ({command}, exit_code={exit_code})",
        if succeeded { "passed" } else { "failed" }
    ))
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{command_name, render_run_trace, render_session_status, render_trace_summary};
    use crate::cli::DeveloperCommand;
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::session::{SessionStatus, SessionStatusView};
    use crate::domain::step::{StepKind, StepStatus};
    use crate::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{
        ExecutionTrace, TraceEventType, TraceRecoveryEvent, TraceStepSummary, TraceSummaryView,
    };

    #[test]
    fn command_name_covers_every_developer_subcommand() {
        let commands = [
            (DeveloperCommand::Doctor { workspace: "/tmp/workspace".into() }, "doctor"),
            (DeveloperCommand::Start { workspace: None }, "start"),
            (DeveloperCommand::Capture { workspace: None, goal: "goal".to_string() }, "capture"),
            (DeveloperCommand::Flow { name: "bug-fix".to_string(), workspace: None }, "flow"),
            (DeveloperCommand::Plan { workspace: None }, "plan"),
            (DeveloperCommand::Step { workspace: None }, "step"),
            (DeveloperCommand::Run { workspace: None, goal: None }, "run"),
            (DeveloperCommand::Inspect { trace: None, workspace: None }, "inspect"),
            (DeveloperCommand::Status { workspace: None }, "status"),
            (DeveloperCommand::Next { workspace: None }, "next"),
        ];

        for (command, expected) in commands {
            assert_eq!(command_name(&command), expected);
        }
    }

    #[test]
    fn render_run_trace_covers_stage_replan_and_stage_failure_fallbacks() {
        let mut trace = ExecutionTrace::new("task-output", "session-output", "Render output");
        trace.record_event(TraceEventType::StageReplanned, None, 0, json!({}));
        trace.record_event(TraceEventType::StageFailed, None, 0, json!({}));

        let response = TaskRunResponse {
            task_id: "task-output".to_string(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "stage failed",
                None,
            ),
            final_context: TaskContext::new(
                "session-output",
                "/tmp/workspace",
                RunLimits::default(),
                serde_json::Map::new(),
            ),
            plan_revision: 0,
            trace_location: "/tmp/workspace/.synod/traces/task-output.json".to_string(),
        };

        let text = render_run_trace("run", Some(&trace), &response, "/synod-next");

        assert!(text.contains("stage replan after unknown-step: replan scheduled"), "{text}");
        assert!(text.contains("stage unknown-stage failed: stage failed"), "{text}");
    }

    #[test]
    fn render_trace_summary_labels_flow_stage_and_stage_failure_events() {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/workspace/.synod/traces/task-output.json".to_string(),
            goal: "Render trace summary".to_string(),
            executed_steps: vec![TraceStepSummary {
                step_id: "verify".to_string(),
                step_kind: StepKind::Tool,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "validation passed".to_string(),
            }],
            recovery_events: vec![
                TraceRecoveryEvent {
                    event_type: TraceEventType::FlowSelected,
                    trigger: "bug-fix @ investigate".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageTransitioned,
                    trigger: "investigate -> implement".to_string(),
                    related_step_id: None,
                },
                TraceRecoveryEvent {
                    event_type: TraceEventType::StageFailed,
                    trigger: "verify failed".to_string(),
                    related_step_id: Some("verify".to_string()),
                },
            ],
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "trace failed",
                None,
            ),
            duration: None,
        };

        let text = render_trace_summary(&summary, "latest-workspace-trace", "/synod-next");

        assert!(text.contains("flow: bug-fix @ investigate"), "{text}");
        assert!(text.contains("stage: investigate -> implement"), "{text}");
        assert!(text.contains("stage_failure: verify failed"), "{text}");
    }

    #[test]
    fn render_session_status_covers_invalid_status_without_changed_files() {
        let view = SessionStatusView {
            session_id: "session-output".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: None,
            active_flow: None,
            current_stage_id: None,
            current_stage_index: None,
            total_stages: None,
            plan_revision: None,
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Invalid,
            latest_trace_ref: None,
            latest_changed_files: Some(Vec::new()),
            latest_validation_status: None,
            next_command: None,
            explanation: "session is invalid".to_string(),
        };

        let text = render_session_status(&view);

        assert!(text.contains("latest_status: invalid"), "{text}");
        assert!(!text.contains("latest_changed_files:"), "{text}");
    }
}
