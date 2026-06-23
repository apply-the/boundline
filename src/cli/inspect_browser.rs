//! Browser inspection CLI support for Boundline.
//!
//! Provides the programmatic surface for `boundline inspect browser`.
//! The clap subcommand wire-up is deferred to a follow-up PR.

use boundline_core::domain::browser_provider::BrowserEvidencePacket;
use std::path::Path;

/// Inspect a browser validation run by reading its evidence packet from
/// the session-scoped artifact directory.
///
/// # Errors
///
/// Returns an error message string if the evidence packet cannot be
/// read or parsed.
pub fn inspect_browser_run(
    session_id: &str,
    run_id: &str,
    workspace_root: &Path,
    show_artifacts: bool,
    show_findings: bool,
) -> Result<String, String> {
    let evidence_path = workspace_root
        .join(".boundline")
        .join("sessions")
        .join(session_id)
        .join("browser")
        .join(run_id)
        .join("evidence.json");

    let json = std::fs::read_to_string(&evidence_path).map_err(|e| {
        format!("failed to read evidence packet at {}: {e}", evidence_path.display())
    })?;

    let packet: BrowserEvidencePacket =
        serde_json::from_str(&json).map_err(|e| format!("invalid evidence packet: {e}"))?;

    let mut output = format!("Browser Validation Run: {}\n", packet.validation_run_id);
    output.push_str(&format!("  provider:  {}\n", packet.provider_id));
    output.push_str(&format!("  status:    {:?}\n", packet.status));
    output.push_str(&format!(
        "  page:      {} (HTTP {})\n",
        packet.page_title.as_deref().unwrap_or("(none)"),
        packet.http_status.map_or("(none)".into(), |s| s.to_string())
    ));
    output.push_str(&format!("  started:   {}\n", packet.started_at));
    output.push_str(&format!("  completed: {}\n", packet.completed_at));
    output.push_str(&format!("  duration:  {}ms\n", packet.timing.total_ms));
    output.push_str(&format!("  artifacts: {} files\n", packet.artifacts.len()));
    output.push_str(&format!("  findings:  {}\n", packet.findings.len()));
    output.push_str(&format!("  capabilities: {}\n", packet.capabilities_active.join(", ")));

    if show_artifacts && !packet.artifacts.is_empty() {
        output.push_str("\nArtifacts:\n");
        for a in &packet.artifacts {
            output.push_str(&format!(
                "  [{:?}] {} — {} bytes, hash={}, retention={:?}\n",
                a.kind, a.relative_path, a.byte_size, a.content_hash, a.retention_class
            ));
        }
    }

    if show_findings && !packet.findings.is_empty() {
        output.push_str("\nFindings:\n");
        for f in &packet.findings {
            output.push_str(&format!("  [{:?}] {:?}: {}\n", f.severity, f.kind, f.message));
            if let Some(ref hint) = f.retryability {
                output.push_str(&format!(
                    "    retryability: {:?}/{:?} — {}\n",
                    hint.level, hint.category, hint.reason
                ));
            }
        }
    }

    Ok(output)
}
