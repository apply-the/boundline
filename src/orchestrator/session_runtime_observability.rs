//! Session runtime observability exporter.
//!
//! Handles persistence of `StructuredRuntimeEvent` to the workspace-local
//! trace directory in JSONL format.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use uuid::Uuid;

use crate::domain::observability::StructuredRuntimeEvent;

/// Persists a structured runtime event to the session's event log.
///
/// The log is written to `.boundline/traces/<session_id>/events.jsonl` within
/// the provided workspace root. Directories are created if they do not exist.
pub fn export_runtime_event(
    workspace_root: &Path,
    session_id: Uuid,
    event: &StructuredRuntimeEvent,
) -> io::Result<()> {
    let trace_dir = workspace_root
        .join(".boundline")
        .join("traces")
        .join(session_id.to_string());
    
    if !trace_dir.exists() {
        fs::create_dir_all(&trace_dir)?;
    }
    
    let events_file = trace_dir.join("events.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_file)?;
    
    let json = serde_json::to_string(event).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    writeln!(file, "{}", json)
}
