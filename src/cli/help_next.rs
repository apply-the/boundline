//! `boundline help-next` CLI command and output rendering.
//!
//! Inspects workspace state and produces a human-readable or JSON
//! recommendation for the next action.

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::domain::help_next::{
    HelpNextDiagnostic, HelpNextRecommendation, HelpNextState,
};

/// Inspect the current workspace and recommend the next action.
#[derive(Debug, Args)]
pub struct HelpNextArgs {
    /// Output as structured JSON instead of human-readable text.
    #[arg(long)]
    pub json: bool,
    /// List all detected issues instead of just the top blocking issue.
    #[arg(long)]
    pub all: bool,
}

/// Run the help-next diagnostic and return the recommendation.
pub fn run_help_next(
    workspace_root: &PathBuf,
    args: &HelpNextArgs,
) -> Result<HelpNextRecommendation, HelpNextError> {
    let _ = workspace_root;
    let _ = args;

    // Detect state from the filesystem
    let boundline_dir = workspace_root.join(".boundline");
    if !boundline_dir.exists() {
        let rec = HelpNextRecommendation::from_diagnostics(
            HelpNextState::Uninitialized,
            vec![],
            resolve_docs_link("workspace_not_initialized"),
        );
        return Ok(rec);
    }

    // Check for active session
    let session_file = boundline_dir.join("session.json");
    if !session_file.exists() {
        let rec = HelpNextRecommendation::from_diagnostics(
            HelpNextState::Initialized,
            vec![],
            resolve_docs_link("workspace_initialized_no_session"),
        );
        return Ok(rec);
    }

    // Session exists — try to load and inspect it.
    // For now, produce a simplified ready recommendation.
    // Full session-model integration (blocked/failed detection) is deferred
    // to a follow-on slice that integrates with the existing session loader.
    let rec = HelpNextRecommendation::ready(resolve_docs_link("session_healthy"));
    Ok(rec)
}

/// Error type for help-next diagnostics.
#[derive(Debug, thiserror::Error)]
pub enum HelpNextError {
    #[error("failed to read workspace state: {0}")]
    Io(#[from] std::io::Error),
    #[error("link map is unreadable: {0}")]
    LinkMap(String),
}

/// Resolve a documentation link from the diagnostic key.
fn resolve_docs_link(_key: &str) -> Option<String> {
    // Link map resolution is deferred — always returns None for now.
    None
}

/// Render the recommendation as human-readable text.
pub fn render_human(rec: &HelpNextRecommendation) -> String {
    let mut out = String::new();
    out.push_str(&format!("State: {}\n", rec.state.label()));

    if let Some(ref primary) = rec.primary_issue {
        out.push_str(&format!("Blockers found: yes\n"));
        out.push_str("---\n");
        out.push_str(&format!("{}\n", primary.message));
    } else {
        out.push_str("No blockers found.\n");
    }

    out.push_str(&format!("Next action: {}\n", rec.recommended_action));
    if let Some(ref cmd) = rec.recommended_command {
        out.push_str(&format!("Command: {cmd}\n"));
    }
    out.push_str(&format!("Why: {}\n", rec.reason));
    if let Some(ref link) = rec.docs_link {
        out.push_str(&format!("Docs: {link}\n"));
    }

    if rec.additional_count > 0 {
        out.push_str(&format!(
            "{} additional issue{} detected. Run `boundline help-next --all` to list them.\n",
            rec.additional_count,
            if rec.additional_count == 1 { "" } else { "s" }
        ));
    }
    out
}

/// Render the recommendation as JSON.
pub fn render_json(rec: &HelpNextRecommendation) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct JsonOutput<'a> {
        state: &'static str,
        blockers_found: bool,
        primary_issue: Option<&'a HelpNextDiagnostic>,
        additional_issues: &'a [HelpNextDiagnostic],
        additional_count: u64,
        recommended_action: &'a str,
        recommended_command: Option<&'a str>,
        reason: &'a str,
        docs_link: Option<&'a str>,
        output_format: &'static str,
    }

    let output = JsonOutput {
        state: rec.state.label(),
        blockers_found: rec.blockers_found,
        primary_issue: rec.primary_issue.as_ref(),
        additional_issues: &rec.additional_issues,
        additional_count: rec.additional_count,
        recommended_action: &rec.recommended_action,
        recommended_command: rec.recommended_command.as_deref(),
        reason: &rec.reason,
        docs_link: rec.docs_link.as_deref(),
        output_format: "json",
    };
    serde_json::to_string_pretty(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninitialized_workspace_produces_init_recommendation() {
        let tmp = std::env::temp_dir().join("boundline-help-next-test-nonexistent");
        let args = HelpNextArgs { json: false, all: false };
        let rec = run_help_next(&tmp, &args).unwrap();
        assert_eq!(rec.state, HelpNextState::Uninitialized);
        assert_eq!(rec.recommended_command.as_deref(), Some("boundline init"));
    }

    #[test]
    fn human_output_contains_state_and_command() {
        let rec = HelpNextRecommendation::ready(Some("wiki/Daily".into()));
        let out = render_human(&rec);
        assert!(out.contains("State: ready"));
        assert!(out.contains("boundline run"));
    }

    #[test]
    fn json_output_is_valid_json() {
        let rec = HelpNextRecommendation::ready(Some("wiki/Daily".into()));
        let json = render_json(&rec).unwrap();
        assert!(json.contains("\"state\""));
        assert!(json.contains("\"ready\""));
    }

    #[test]
    fn blocked_state_shows_additional_count() {
        let diag = HelpNextDiagnostic {
            key: "block".into(),
            severity: crate::domain::help_next::DiagnosticSeverity::Blocking,
            message: "blocked".into(),
            source: None,
            command: Some("boundline plan".into()),
            docs_key: "fallback".into(),
        };
        let diag2 = HelpNextDiagnostic {
            key: "warn".into(),
            severity: crate::domain::help_next::DiagnosticSeverity::Warning,
            message: "warning".into(),
            source: None,
            command: None,
            docs_key: "fallback".into(),
        };
        let rec = HelpNextRecommendation::from_diagnostics(
            HelpNextState::Blocked,
            vec![diag, diag2],
            None,
        );
        let out = render_human(&rec);
        assert!(out.contains("1 additional issue"));
    }
}
