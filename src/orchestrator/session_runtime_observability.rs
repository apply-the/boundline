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
    let trace_dir = workspace_root.join(".boundline").join("traces").join(session_id.to_string());

    if !trace_dir.exists() {
        fs::create_dir_all(&trace_dir)?;
    }

    let events_file = trace_dir.join("events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(events_file)?;

    let json =
        serde_json::to_string(event).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    writeln!(file, "{}", json)
}

/// Emits a structured event when a control level is graduated (promoted or demoted) due to trust metrics.
pub fn emit_control_level_graduated(
    workspace_root: &Path,
    session_id: Uuid,
    rule_id: &str,
    before_level: &str,
    after_level: &str,
    trigger: &str,
    confidence: f64,
) -> io::Result<()> {
    let payload = serde_json::json!({
        "rule_id": rule_id,
        "before_level": before_level,
        "after_level": after_level,
        "trigger": trigger,
        "confidence": confidence,
    });
    let event = StructuredRuntimeEvent::new(
        crate::domain::observability::EventType::ControlLevelGraduated,
        session_id,
        None,
        payload,
    );
    export_runtime_event(workspace_root, session_id, &event)
}

/// Emits a structured event when a control is degraded due to missing evidence or provider unavailability.
#[allow(clippy::too_many_arguments)]
pub fn emit_control_degraded(
    workspace_root: &Path,
    session_id: Uuid,
    rule_id: &str,
    original_level: &str,
    degraded_level: &str,
    trigger: &str,
    safety_flag: bool,
    human_gate_flag: bool,
) -> io::Result<()> {
    let payload = serde_json::json!({
        "rule_id": rule_id,
        "original_level": original_level,
        "degraded_level": degraded_level,
        "trigger": trigger,
        "safety_flag": safety_flag,
        "human_gate_flag": human_gate_flag,
    });
    let event = StructuredRuntimeEvent::new(
        crate::domain::observability::EventType::ControlDegraded,
        session_id,
        None,
        payload,
    );
    export_runtime_event(workspace_root, session_id, &event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_runtime_event_creates_dir_and_file() {
        let tmp = tempdir().unwrap();
        let session_id = Uuid::new_v4();
        let event = StructuredRuntimeEvent::new(
            crate::domain::observability::EventType::ControlDegraded,
            session_id,
            None,
            serde_json::json!({}),
        );
        export_runtime_event(tmp.path(), session_id, &event).unwrap();
        let events_file = tmp
            .path()
            .join(".boundline")
            .join("traces")
            .join(session_id.to_string())
            .join("events.jsonl");
        assert!(events_file.exists());
    }

    #[test]
    fn test_emit_control_level_graduated() {
        let tmp = tempdir().unwrap();
        let session_id = Uuid::new_v4();
        emit_control_level_graduated(
            tmp.path(),
            session_id,
            "test_rule",
            "advisory",
            "catch",
            "trust_promotion",
            0.95,
        )
        .unwrap();
        let events_file = tmp
            .path()
            .join(".boundline")
            .join("traces")
            .join(session_id.to_string())
            .join("events.jsonl");
        let content = std::fs::read_to_string(events_file).unwrap();
        assert!(content.contains("test_rule"));
        assert!(content.contains("trust_promotion"));
    }

    #[test]
    fn test_emit_control_degraded() {
        let tmp = tempdir().unwrap();
        let session_id = Uuid::new_v4();
        emit_control_degraded(
            tmp.path(),
            session_id,
            "test_rule",
            "rule",
            "advisory",
            "provider_unavailable",
            true,
            false,
        )
        .unwrap();
        let events_file = tmp
            .path()
            .join(".boundline")
            .join("traces")
            .join(session_id.to_string())
            .join("events.jsonl");
        let content = std::fs::read_to_string(events_file).unwrap();
        assert!(content.contains("test_rule"));
        assert!(content.contains("provider_unavailable"));
    }
}
