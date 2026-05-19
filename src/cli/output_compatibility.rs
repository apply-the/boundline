use super::task_status_text;
use crate::domain::follow_through::FollowThroughProjection;
use crate::domain::session::{CompatibilityFollowUpView, ContinuityAuthority};

/// Renders the latest compatibility follow-up when no native session state is authoritative.
pub fn render_compatibility_follow_up_status(
    workspace_ref: &str,
    continuity_authority: ContinuityAuthority,
    follow_up: &CompatibilityFollowUpView,
    explanation: impl Into<String>,
) -> String {
    let mut lines = vec![format!("workspace_ref: {workspace_ref}")];
    lines.push("route_owner: compatibility".to_string());
    lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    lines.extend(render_compatibility_follow_up_lines(
        follow_up,
        "routing",
        "execution_condition",
        "next_command",
    ));
    let follow_through = FollowThroughProjection::from_compatibility_follow_up(follow_up);
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }
    lines.push(format!("explanation: {}", explanation.into()));
    lines.join("\n")
}

pub(crate) fn render_compatibility_follow_up_lines(
    follow_up: &CompatibilityFollowUpView,
    routing_label: &str,
    execution_condition_label: &str,
    next_command_label: &str,
) -> Vec<String> {
    let routing_summary =
        follow_up.routing_summary.strip_prefix("routing: ").unwrap_or(&follow_up.routing_summary);

    vec![
        format!("compatibility_follow_up: {}", follow_up.follow_up_mode.as_str()),
        format!("compatibility_trace_ref: {}", follow_up.trace_ref),
        format!("{routing_label}: {routing_summary}"),
        format!("{execution_condition_label}: {}", follow_up.execution_condition),
        format!("compatibility_terminal_status: {}", task_status_text(follow_up.terminal_status)),
        format!("compatibility_terminal_reason: {}", follow_up.terminal_reason),
        format!("{next_command_label}: {}", follow_up.next_command),
    ]
}
