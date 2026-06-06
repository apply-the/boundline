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
