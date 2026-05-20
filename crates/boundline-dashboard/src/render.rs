use boundline_core::domain::dashboard::{DashboardSnapshot, DegradedReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Interactive,
    Compact,
    Monochrome,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    pub mode: RenderMode,
    pub width: u16,
    pub height: u16,
    pub color: bool,
}

pub fn render_snapshot(snapshot: &DashboardSnapshot, options: RenderOptions) -> String {
    let mut lines = Vec::new();
    let mode = render_mode_label(options.mode);
    lines.push(snapshot.branding.fallback_label.clone());
    lines.push(format!("mode: {mode}"));
    lines.push(format!("workspace: {}", snapshot.workspace_ref));

    if let Some(session) = &snapshot.session {
        lines.push(format!("goal: {}", session.goal));
        if let Some(stage) = &session.current_stage {
            lines.push(format!("stage: {stage}"));
        }
        if let Some(step_id) = &session.current_step_id {
            lines.push(format!("step: {step_id}"));
        }
        lines.push(format!("condition: {:?}", session.execution_condition));
        lines.push(format!("next: {}", session.next_command));
        if let Some(reason) = &session.blocking_reason {
            lines.push(format!("blocking: {reason}"));
        }
    }

    if let Some(degraded) = &snapshot.degraded_state {
        lines.push(format!("degraded: {}", degraded_reason_label(degraded.reason)));
        if let Some(hint) = &degraded.recovery_hint {
            lines.push(format!("hint: {hint}"));
        }
        for command in &degraded.available_commands {
            lines.push(format!("fallback: {command}"));
        }
    }

    if !snapshot.timeline.is_empty() {
        lines.push("timeline:".to_string());
        for event in snapshot.timeline.iter().take(3) {
            lines.push(format!("- {}: {}", event.event_kind, event.headline));
        }
    }
    if let Some(goal_plan) = &snapshot.panels.goal_plan {
        lines.push(format!("goal_plan: {} rev {}", goal_plan.state, goal_plan.revision));
        if let Some(strategy) = &goal_plan.verification_strategy {
            lines.push(format!("verification: {strategy}"));
        }
    }
    if !snapshot.panels.context_pack.is_empty() {
        lines.push("context_pack:".to_string());
        for item in snapshot.panels.context_pack.iter().take(3) {
            lines
                .push(format!("- {} from {} ({})", item.evidence_ref, item.source, item.authority));
        }
    }
    if !snapshot.panels.governed_references.is_empty() {
        lines.push("governed_references:".to_string());
        for item in snapshot.panels.governed_references.iter().take(3) {
            lines.push(format!("- {} [{}]", item.reference, item.readiness));
        }
    }
    if !snapshot.panels.diagnostics.is_empty() {
        lines.push("diagnostics:".to_string());
        for item in snapshot.panels.diagnostics.iter().take(3) {
            lines.push(format!("- {}: {}", item.category, item.status));
        }
    }

    lines.join("\n") + "\n"
}

fn render_mode_label(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Interactive => "interactive",
        RenderMode::Compact => "compact",
        RenderMode::Monochrome => "monochrome",
        RenderMode::Degraded => "degraded",
    }
}

fn degraded_reason_label(reason: DegradedReason) -> &'static str {
    match reason {
        DegradedReason::InvalidWorkspace => "invalid_workspace",
        DegradedReason::MissingActiveSession => "missing_active_session",
        DegradedReason::InvalidSessionJson => "invalid_session_json",
        DegradedReason::StaleTraceReference => "stale_trace_reference",
        DegradedReason::TerminalUnsupported => "terminal_unsupported",
        DegradedReason::DashboardUnavailable => "dashboard_unavailable",
        DegradedReason::RuntimeCommandUnavailable => "runtime_command_unavailable",
        DegradedReason::StateReadFailed => "state_read_failed",
    }
}
