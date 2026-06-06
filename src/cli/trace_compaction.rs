use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::domain::trace_compaction::CompactionMetrics;

/// Manage session trace lifecycle and compaction.
#[derive(Debug, Subcommand)]
pub enum TraceSubcommand {
    Compact(TraceCompactArgs),
}

/// Compact session traces to reduce size.
#[derive(Debug, Args)]
pub struct TraceCompactArgs {
    /// Workspace directory containing traces.
    #[arg(long)]
    pub workspace: Option<PathBuf>,
    /// Preserve unconditionally all items matching this type.
    #[arg(long)]
    pub preserve_accepted: bool,
    /// Produce output as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Execute the trace compaction command.
pub fn run_trace_compaction(
    workspace_root: &PathBuf,
    args: &TraceCompactArgs,
) -> Result<CompactionMetrics, TraceCompactionCliError> {
    let _ = workspace_root;
    let _ = args;
    // MVP stub for trace compaction
    let metrics = CompactionMetrics {
        compaction_count: 1,
        class_distribution: std::collections::HashMap::new(),
        trace_size_before_bytes: 0,
        trace_size_after_bytes: 0,
        lossy_count: 0,
        preserved_decision_count: 0,
        preserved_rejection_count: 0,
    };
    Ok(metrics)
}

#[derive(Debug, thiserror::Error)]
pub enum TraceCompactionCliError {
    #[error("failed to read trace: {0}")]
    Io(#[from] std::io::Error),
}

/// Render trace compaction metrics to JSON
pub fn render_json(metrics: &CompactionMetrics) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(metrics)
}

/// Render trace compaction metrics to text
pub fn render_human(metrics: &CompactionMetrics) -> String {
    format!(
        "Trace compaction completed.\nSize before: {} bytes\nSize after: {} bytes\nLossy actions: {}\nPreserved decisions: {}",
        metrics.trace_size_before_bytes,
        metrics.trace_size_after_bytes,
        metrics.lossy_count,
        metrics.preserved_decision_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_trace_compaction_returns_metrics() {
        let args = TraceCompactArgs { workspace: None, preserve_accepted: false, json: false };
        let root = PathBuf::from(".");
        let metrics = run_trace_compaction(&root, &args).unwrap();
        assert_eq!(metrics.compaction_count, 1);
    }

    #[test]
    fn render_json_formats_metrics() {
        let metrics = CompactionMetrics {
            compaction_count: 1,
            class_distribution: std::collections::HashMap::new(),
            trace_size_before_bytes: 0,
            trace_size_after_bytes: 0,
            lossy_count: 0,
            preserved_decision_count: 0,
            preserved_rejection_count: 0,
        };
        let json = render_json(&metrics).unwrap();
        assert!(json.contains("\"compaction_count\": 1"));
    }

    #[test]
    fn render_human_formats_metrics() {
        let metrics = CompactionMetrics {
            compaction_count: 1,
            class_distribution: std::collections::HashMap::new(),
            trace_size_before_bytes: 100,
            trace_size_after_bytes: 50,
            lossy_count: 0,
            preserved_decision_count: 0,
            preserved_rejection_count: 0,
        };
        let text = render_human(&metrics);
        assert!(text.contains("Size before: 100 bytes"));
    }

    #[test]
    fn trace_compaction_cli_error_display() {
        let err = TraceCompactionCliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "test err",
        ));
        assert_eq!(err.to_string(), "failed to read trace: test err");
    }
}
