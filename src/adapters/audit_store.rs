use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::audit::SessionAuditEntry;
use crate::domain::session::{session_audit_cursor_ref, session_audit_events_ref};
use crate::domain::trace::HookEventDispatchRecord;

const FRAMEWORK_ADAPTER_HOOK_EVENTS_FILE_NAME: &str = "framework-adapter-hooks.jsonl";

pub trait SessionAuditStore: Send + Sync {
    fn append(&self, entry: &SessionAuditEntry) -> Result<PathBuf, SessionAuditStoreError>;
    fn load_all(&self) -> Result<Vec<SessionAuditEntry>, SessionAuditStoreError>;
    fn load_cursor(&self) -> Result<SessionAuditCursor, SessionAuditStoreError>;
    fn persist_cursor(
        &self,
        cursor: &SessionAuditCursor,
    ) -> Result<PathBuf, SessionAuditStoreError>;
}

/// Persistence surface for framework-adapter hook dispatch audit records.
pub trait FrameworkAdapterHookAuditStore: Send + Sync {
    /// Appends one hook dispatch record to the session-scoped hook audit log.
    fn append_hook_dispatch(
        &self,
        record: &HookEventDispatchRecord,
    ) -> Result<PathBuf, SessionAuditStoreError>;

    /// Loads all hook dispatch records recorded for the session.
    fn load_hook_dispatches(&self) -> Result<Vec<HookEventDispatchRecord>, SessionAuditStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileSessionAuditStore {
    events_path: PathBuf,
    cursor_path: PathBuf,
}

impl FileSessionAuditStore {
    pub fn new(events_path: impl Into<PathBuf>, cursor_path: impl Into<PathBuf>) -> Self {
        Self { events_path: events_path.into(), cursor_path: cursor_path.into() }
    }

    pub fn for_session(workspace_ref: impl AsRef<Path>, session_id: &str) -> Self {
        let workspace_ref = workspace_ref.as_ref();
        Self {
            events_path: workspace_ref.join(session_audit_events_ref(session_id)),
            cursor_path: workspace_ref.join(session_audit_cursor_ref(session_id)),
        }
    }

    pub fn events_path(&self) -> &Path {
        &self.events_path
    }

    pub fn cursor_path(&self) -> &Path {
        &self.cursor_path
    }

    pub fn framework_adapter_hook_events_path(&self) -> PathBuf {
        self.events_path
            .parent()
            .map(|parent| parent.join(FRAMEWORK_ADAPTER_HOOK_EVENTS_FILE_NAME))
            .unwrap_or_else(|| PathBuf::from(FRAMEWORK_ADAPTER_HOOK_EVENTS_FILE_NAME))
    }
}

impl SessionAuditStore for FileSessionAuditStore {
    fn append(&self, entry: &SessionAuditEntry) -> Result<PathBuf, SessionAuditStoreError> {
        append_jsonl_record(&self.events_path, entry)
    }

    fn load_all(&self) -> Result<Vec<SessionAuditEntry>, SessionAuditStoreError> {
        load_jsonl_records(&self.events_path)
    }

    fn load_cursor(&self) -> Result<SessionAuditCursor, SessionAuditStoreError> {
        if !self.cursor_path.is_file() {
            return Ok(SessionAuditCursor::default());
        }

        let contents = fs::read(&self.cursor_path).map_err(SessionAuditStoreError::Read)?;
        serde_json::from_slice(&contents).map_err(SessionAuditStoreError::Deserialize)
    }

    fn persist_cursor(
        &self,
        cursor: &SessionAuditCursor,
    ) -> Result<PathBuf, SessionAuditStoreError> {
        if let Some(parent) = self.cursor_path.parent() {
            fs::create_dir_all(parent).map_err(SessionAuditStoreError::CreateDirectory)?;
        }

        let contents =
            serde_json::to_vec_pretty(cursor).map_err(SessionAuditStoreError::Serialize)?;
        fs::write(&self.cursor_path, contents).map_err(SessionAuditStoreError::Write)?;
        Ok(self.cursor_path.clone())
    }
}

impl FrameworkAdapterHookAuditStore for FileSessionAuditStore {
    fn append_hook_dispatch(
        &self,
        record: &HookEventDispatchRecord,
    ) -> Result<PathBuf, SessionAuditStoreError> {
        append_jsonl_record(&self.framework_adapter_hook_events_path(), record)
    }

    fn load_hook_dispatches(&self) -> Result<Vec<HookEventDispatchRecord>, SessionAuditStoreError> {
        load_jsonl_records(&self.framework_adapter_hook_events_path())
    }
}

fn append_jsonl_record<T: Serialize>(
    path: &Path,
    record: &T,
) -> Result<PathBuf, SessionAuditStoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(SessionAuditStoreError::CreateDirectory)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(SessionAuditStoreError::Open)?;
    let contents = serde_json::to_vec(record).map_err(SessionAuditStoreError::Serialize)?;
    file.write_all(&contents).map_err(SessionAuditStoreError::Write)?;
    file.write_all(b"\n").map_err(SessionAuditStoreError::Write)?;
    Ok(path.to_path_buf())
}

fn load_jsonl_records<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, SessionAuditStoreError> {
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path).map_err(SessionAuditStoreError::Read)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(SessionAuditStoreError::Read)?;
        if line.trim().is_empty() {
            continue;
        }
        let record =
            serde_json::from_str::<T>(&line).map_err(SessionAuditStoreError::Deserialize)?;
        records.push(record);
    }

    Ok(records)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionAuditCursor {
    #[serde(default)]
    pub last_sequence: u64,
    #[serde(default)]
    pub session_start_recorded: bool,
    #[serde(default)]
    pub session_end_recorded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_session_status: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub projected_trace_events: BTreeMap<String, BTreeSet<String>>,
}

impl SessionAuditCursor {
    pub fn next_sequence(&mut self) -> u64 {
        self.last_sequence = self.last_sequence.saturating_add(1);
        self.last_sequence
    }

    pub fn already_projected(&self, trace_id: &str, event_id: &str) -> bool {
        self.projected_trace_events
            .get(trace_id)
            .is_some_and(|event_ids| event_ids.contains(event_id))
    }

    pub fn mark_projected(&mut self, trace_id: impl Into<String>, event_id: impl Into<String>) {
        let trace_id = trace_id.into();
        let event_id = event_id.into();
        let event_ids = self.projected_trace_events.entry(trace_id).or_default();
        event_ids.insert(event_id);
    }
}

#[derive(Debug, Error)]
pub enum SessionAuditStoreError {
    #[error("failed to create audit directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("failed to open audit log: {0}")]
    Open(std::io::Error),
    #[error("failed to read audit data: {0}")]
    Read(std::io::Error),
    #[error("failed to serialize audit data: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize audit data: {0}")]
    Deserialize(serde_json::Error),
    #[error("failed to write audit data: {0}")]
    Write(std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{FileSessionAuditStore, SessionAuditCursor, SessionAuditStore};
    use crate::domain::audit::{
        SessionAuditActor, SessionAuditAlgorithm, SessionAuditEntry, SessionAuditEntryKind,
        SessionAuditIdentity, SessionAuditOutcome, SessionAuditOutcomeStatus, SessionAuditPhase,
        SessionAuditSource,
    };

    fn temp_workspace() -> Result<PathBuf, String> {
        let workspace =
            std::env::temp_dir().join(format!("boundline-audit-store-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).map_err(|error| error.to_string())?;
        Ok(workspace)
    }

    fn sample_entry(sequence: u64) -> SessionAuditEntry {
        SessionAuditEntry::new(
            "session-1",
            sequence,
            SessionAuditEntryKind::SessionStart,
            "started session",
            SessionAuditIdentity::default(),
            SessionAuditActor::system("boundline"),
            SessionAuditAlgorithm::new(
                SessionAuditPhase::Session,
                "session_runtime",
                "persist_session",
            ),
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session started"),
            SessionAuditSource::session_lifecycle(),
            json!({"sequence": sequence}),
        )
    }

    #[test]
    fn append_and_load_round_trip_jsonl_entries() -> Result<(), String> {
        let workspace = temp_workspace()?;
        let store = FileSessionAuditStore::for_session(&workspace, "session-1");

        store.append(&sample_entry(1)).map_err(|error| error.to_string())?;
        store.append(&sample_entry(2)).map_err(|error| error.to_string())?;

        let entries = store.load_all().map_err(|error| error.to_string())?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 1);
        assert_eq!(entries[1].sequence, 2);
        Ok(())
    }

    #[test]
    fn cursor_tracks_projected_trace_events() -> Result<(), String> {
        let workspace = temp_workspace()?;
        let store = FileSessionAuditStore::for_session(&workspace, "session-1");
        let mut cursor = SessionAuditCursor::default();

        let sequence = cursor.next_sequence();
        cursor.mark_projected("trace-1", "event-1");
        store.persist_cursor(&cursor).map_err(|error| error.to_string())?;

        let loaded = store.load_cursor().map_err(|error| error.to_string())?;
        assert_eq!(sequence, 1);
        assert_eq!(loaded.last_sequence, 1);
        assert!(loaded.already_projected("trace-1", "event-1"));
        Ok(())
    }

    #[test]
    fn new_constructor_exposes_configured_paths() -> Result<(), String> {
        let workspace = temp_workspace()?;
        let events_path = workspace.join("session/audit_events.ndjson");
        let cursor_path = workspace.join("session/audit_cursor.json");

        let store = FileSessionAuditStore::new(events_path.clone(), cursor_path.clone());
        assert_eq!(store.events_path(), events_path.as_path());
        assert_eq!(store.cursor_path(), cursor_path.as_path());
        Ok(())
    }

    #[test]
    fn load_all_skips_blank_lines_in_ndjson() -> Result<(), String> {
        use std::io::Write;

        let workspace = temp_workspace()?;
        let store = FileSessionAuditStore::for_session(&workspace, "session-blank-lines");

        store.append(&sample_entry(1)).map_err(|error| error.to_string())?;

        // Inject a blank line to exercise the `continue` branch in load_all.
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(store.events_path())
            .map_err(|error| error.to_string())?;
        writeln!(file).map_err(|error| error.to_string())?;
        drop(file);

        store.append(&sample_entry(2)).map_err(|error| error.to_string())?;

        let entries = store.load_all().map_err(|error| error.to_string())?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 1);
        assert_eq!(entries[1].sequence, 2);
        Ok(())
    }
}
