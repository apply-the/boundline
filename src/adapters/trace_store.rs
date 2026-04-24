use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::trace::ExecutionTrace;

pub trait TraceStore: Send + Sync {
    fn persist(&self, trace: &ExecutionTrace) -> Result<PathBuf, TraceStoreError>;
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
        Self::new(workspace_ref.as_ref().join(".synod").join("traces"))
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
}

#[derive(Debug, Error)]
pub enum TraceStoreError {
    #[error("failed to create trace directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("failed to serialize trace: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to write trace file: {0}")]
    Write(std::io::Error),
}
