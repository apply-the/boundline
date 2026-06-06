use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::domain::evals::EvalSummary;

/// Evaluate outputs and runs against fixtures.
#[derive(Debug, Subcommand)]
pub enum EvalsSubcommand {
    Run(EvalsRunArgs),
}

/// Run evaluations based on configured fixtures.
#[derive(Debug, Args)]
pub struct EvalsRunArgs {
    /// Workspace directory containing evals configuration.
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    /// Filter for specific evaluation suite.
    #[arg(long)]
    pub suite: Option<String>,
    /// Produce output as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Execute the evals run command.
pub fn run_evals(
    workspace_root: &PathBuf,
    args: &EvalsRunArgs,
) -> Result<EvalSummary, EvalsCliError> {
    let _ = workspace_root;
    let _ = args;
    // MVP stub for evals runner
    let summary = EvalSummary::from_results(vec![]);
    Ok(summary)
}

#[derive(Debug, thiserror::Error)]
pub enum EvalsCliError {
    #[error("failed to read evaluations: {0}")]
    Io(#[from] std::io::Error),
}

/// Render eval summary to JSON
pub fn render_json(summary: &EvalSummary) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(summary)
}

/// Render eval summary to text
pub fn render_human(summary: &EvalSummary) -> String {
    format!(
        "Evals run completed.\nTotal: {}\nPassed: {}\nFailed: {}\nStatus: {:?}\nDuration: {}ms",
        summary.total_count,
        summary.pass_count,
        summary.fail_count,
        summary.suite_status,
        summary.duration_ms
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_evals_returns_empty_summary() {
        let args = EvalsRunArgs { workspace: None, suite: None, json: false };
        let root = PathBuf::from(".");
        let summary = run_evals(&root, &args).unwrap();
        assert_eq!(summary.total_count, 0);
    }

    #[test]
    fn render_json_formats_correctly() {
        let summary = EvalSummary::from_results(vec![]);
        let json = render_json(&summary).unwrap();
        assert!(json.contains("\"total_count\": 0"));
    }

    #[test]
    fn render_human_formats_correctly() {
        let summary = EvalSummary::from_results(vec![]);
        let text = render_human(&summary);
        assert!(text.contains("Total: 0"));
    }

    #[test]
    fn evals_cli_error_display() {
        let err =
            EvalsCliError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test error"));
        assert_eq!(err.to_string(), "failed to read evaluations: test error");
    }
}
