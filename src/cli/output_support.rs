use serde_json::Value;

use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::task::TaskStatus;

pub(crate) fn checkpoint_projection_from_state(
    state: &serde_json::Map<String, Value>,
) -> (Option<String>, Option<String>, Option<String>) {
    (
        state.get("latest_checkpoint_id").and_then(Value::as_str).map(str::to_string),
        state.get("latest_checkpoint_scope").and_then(Value::as_str).map(str::to_string),
        state.get("latest_checkpoint_restore_command").and_then(Value::as_str).map(str::to_string),
    )
}

/// Renders an inspect failure while preserving the trace-resolution context.
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

    lines.push("next_command: /boundline-inspect".to_string());
    lines.push(format!("corrected_command: {corrected_command}"));
    lines.join("\n")
}

/// Renders a session-command failure with an optional suggested next command.
pub fn render_session_error(action: &str, message: &str, next_command: Option<&str>) -> String {
    let mut lines = vec![format!("{action}: session error"), format!("reason: {message}")];

    if let Some(next_command) = next_command {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.join("\n")
}

/// Converts the flattened guidance and guardian projection into compact summary
/// lines for status and inspect output.
pub fn render_guidance_projection_lines(
    guidance_guardian: &GuidanceGuardianProjection,
) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(summary) = &guidance_guardian.capability_resolution_summary {
        lines.push(format!("guidance_resolution_summary: {summary}"));
    }
    if !guidance_guardian.loaded_packs.is_empty() {
        lines.push(format!("loaded_packs: {}", guidance_guardian.loaded_packs.join(", ")));
    }
    if !guidance_guardian.skipped_packs.is_empty() {
        lines.push(format!("skipped_packs: {}", guidance_guardian.skipped_packs.join(" | ")));
    }
    if !guidance_guardian.catalog_validation_findings.is_empty() {
        lines.push(format!(
            "catalog_validation_findings: {}",
            guidance_guardian.catalog_validation_findings.join(" | ")
        ));
    }
    if !guidance_guardian.loaded_guidance_sources.is_empty() {
        lines.push(format!(
            "loaded_guidance_sources: {}",
            guidance_guardian.loaded_guidance_sources.join(", ")
        ));
    }
    if !guidance_guardian.skipped_guidance_sources.is_empty() {
        lines.push(format!(
            "skipped_guidance_sources: {}",
            guidance_guardian.skipped_guidance_sources.join(", ")
        ));
    }
    if !guidance_guardian.loaded_guardian_sources.is_empty() {
        lines.push(format!(
            "loaded_guardian_sources: {}",
            guidance_guardian.loaded_guardian_sources.join(", ")
        ));
    }
    if !guidance_guardian.skipped_guardian_sources.is_empty() {
        lines.push(format!(
            "skipped_guardian_sources: {}",
            guidance_guardian.skipped_guardian_sources.join(", ")
        ));
    }
    if !guidance_guardian.guardian_timeline.is_empty() {
        lines.push(format!(
            "guardian_timeline: {}",
            guidance_guardian.guardian_timeline.join(" | ")
        ));
    }
    if let Some(summary) = &guidance_guardian.guardian_findings_summary {
        lines.push(format!("guardian_findings_summary: {summary}"));
    }
    if !guidance_guardian.guardian_findings.is_empty() {
        lines.push(format!(
            "guardian_findings: {}",
            guidance_guardian
                .guardian_findings
                .iter()
                .map(|finding| format!(
                    "{}:{}:{}",
                    finding.guardian_id,
                    finding.disposition.as_str(),
                    finding.summary
                ))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !guidance_guardian.guardian_degradations.is_empty() {
        lines.push(format!(
            "guardian_degradations: {}",
            guidance_guardian.guardian_degradations.join(" | ")
        ));
    }
    if let Some(outcome) = &guidance_guardian.guardian_blocking_outcome {
        lines.push(format!("guardian_blocking_outcome: {outcome}"));
    }

    lines
}

/// Returns the next recommended command after a `run` response.
pub const fn next_command_after_run(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Succeeded => "/boundline-status",
        TaskStatus::Planned
        | TaskStatus::Running
        | TaskStatus::Failed
        | TaskStatus::Exhausted
        | TaskStatus::Aborted => "/boundline-next",
    }
}

/// Returns the next recommended command after an `inspect` response.
pub const fn next_command_after_inspect(_: TaskStatus) -> &'static str {
    "/boundline-next"
}

/// Appends the four governance display lines using `: ` separator format.
pub(crate) fn push_governance_display_lines(
    lines: &mut Vec<String>,
    runtime: Option<&str>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) {
    if let Some(runtime) = runtime {
        lines.push(format!("requested_governance_runtime: {runtime}"));
    }
    if let Some(risk) = risk {
        lines.push(format!("requested_governance_risk: {risk}"));
    }
    if let Some(zone) = zone {
        lines.push(format!("requested_governance_zone: {zone}"));
    }
    if let Some(owner) = owner {
        lines.push(format!("requested_governance_owner: {owner}"));
    }
}
