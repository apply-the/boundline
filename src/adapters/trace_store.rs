use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::trace::ExecutionTrace;

pub trait TraceStore: Send + Sync {
    fn persist(&self, trace: &ExecutionTrace) -> Result<PathBuf, TraceStoreError>;
    fn load(&self, path: &Path) -> Result<ExecutionTrace, TraceStoreError>;
    fn latest(&self) -> Result<Option<PathBuf>, TraceStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileTraceStore {
    root: PathBuf,
}

impl FileTraceStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        Self::new(workspace_ref.as_ref().join(".boundline").join("traces"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl TraceStore for FileTraceStore {
    fn persist(&self, trace: &ExecutionTrace) -> Result<PathBuf, TraceStoreError> {
        fs::create_dir_all(&self.root).map_err(TraceStoreError::CreateDirectory)?;
        let path = self.root.join(format!("{}.json", trace.task_id));
        let contents = serde_json::to_vec_pretty(trace).map_err(TraceStoreError::Serialize)?;
        fs::write(&path, contents).map_err(TraceStoreError::Write)?;
        Ok(path)
    }

    fn load(&self, path: &Path) -> Result<ExecutionTrace, TraceStoreError> {
        let contents = fs::read(path).map_err(TraceStoreError::Read)?;
        serde_json::from_slice(&contents).map_err(TraceStoreError::Deserialize)
    }

    fn latest(&self) -> Result<Option<PathBuf>, TraceStoreError> {
        if !self.root.exists() {
            return Ok(None);
        }

        let mut latest: Option<(u64, String, PathBuf)> = None;
        for entry in fs::read_dir(&self.root).map_err(TraceStoreError::ReadDirectory)? {
            let entry = entry.map_err(TraceStoreError::ReadDirectory)?;
            let path = entry.path();

            if !path.is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("json")
            {
                continue;
            }

            let trace = self.load(&path)?;
            let sort_key = trace.ended_at.unwrap_or(trace.started_at);
            let path_key = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();

            let should_replace = match latest.as_ref() {
                Some((current_sort_key, current_path_key, _)) => {
                    (sort_key, path_key) > (*current_sort_key, current_path_key.as_str())
                }
                None => true,
            };

            if should_replace {
                latest = Some((sort_key, path_key.to_string(), path));
            }
        }

        Ok(latest.map(|(_, _, path)| path))
    }
}

#[derive(Debug, Error)]
pub enum TraceStoreError {
    #[error("failed to create trace directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("failed to list traces: {0}")]
    ReadDirectory(std::io::Error),
    #[error("failed to read trace file: {0}")]
    Read(std::io::Error),
    #[error("failed to serialize trace: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize trace: {0}")]
    Deserialize(serde_json::Error),
    #[error("failed to write trace file: {0}")]
    Write(std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{FileTraceStore, TraceStore};
    use crate::domain::limits::TerminalCondition;
    use crate::domain::task::{TaskStatus, TerminalReason};
    use crate::domain::trace::{ExecutionTrace, TraceEventType};

    fn temp_workspace() -> PathBuf {
        let workspace =
            std::env::temp_dir().join(format!("boundline-trace-store-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn build_trace(task_id: &str, started_at: u64, ended_at: u64) -> ExecutionTrace {
        let mut trace = ExecutionTrace::new(task_id, "session-trace", format!("goal-{task_id}"));
        trace.started_at = started_at;
        trace.record_event(TraceEventType::TaskStarted, None, 0, json!({"goal": trace.goal}));
        trace.finalize(
            TaskStatus::Succeeded,
            TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        );
        trace.ended_at = Some(ended_at);
        trace
    }

    #[test]
    fn load_round_trips_a_persisted_trace() {
        let workspace = temp_workspace();
        let store = FileTraceStore::for_workspace(&workspace);
        let trace = build_trace("task-load", 10, 20);

        let path = store.persist(&trace).unwrap();
        let loaded = store.load(&path).unwrap();

        assert_eq!(loaded.task_id, "task-load");
        assert_eq!(loaded.goal, "goal-task-load");
        assert_eq!(loaded.terminal_status, Some(TaskStatus::Succeeded));
        assert_eq!(loaded.ended_at, Some(20));
    }

    #[test]
    fn latest_returns_the_most_recent_trace_path() {
        let workspace = temp_workspace();
        let store = FileTraceStore::for_workspace(&workspace);
        let first = build_trace("task-first", 10, 20);
        let second = build_trace("task-second", 30, 40);

        let first_path = store.persist(&first).unwrap();
        let second_path = store.persist(&second).unwrap();

        assert_eq!(store.latest().unwrap(), Some(second_path));
        assert_ne!(store.latest().unwrap(), Some(first_path));
    }

    #[test]
    fn latest_returns_none_when_the_trace_directory_is_missing() {
        let workspace = temp_workspace();
        let trace_root = workspace.join(".boundline").join("traces");
        fs::remove_dir_all(&workspace).unwrap();

        let store = FileTraceStore::new(trace_root);

        assert_eq!(store.latest().unwrap(), None);
    }
}
