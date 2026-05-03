use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::session::ActiveSessionRecord;

pub trait SessionStore: Send + Sync {
    fn persist(&self, session: &ActiveSessionRecord) -> Result<PathBuf, SessionStoreError>;
    fn load(&self) -> Result<Option<ActiveSessionRecord>, SessionStoreError>;
    fn clear(&self) -> Result<(), SessionStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileSessionStore {
    path: PathBuf,
}

impl FileSessionStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        Self::new(workspace_ref.as_ref().join(".boundline").join("session.json"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SessionStore for FileSessionStore {
    fn persist(&self, session: &ActiveSessionRecord) -> Result<PathBuf, SessionStoreError> {
        session.validate().map_err(|error| SessionStoreError::InvalidRecord(error.to_string()))?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
        }

        let contents = serde_json::to_vec_pretty(session).map_err(SessionStoreError::Serialize)?;
        fs::write(&self.path, contents).map_err(SessionStoreError::Write)?;
        Ok(self.path.clone())
    }

    fn load(&self) -> Result<Option<ActiveSessionRecord>, SessionStoreError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read(&self.path).map_err(SessionStoreError::Read)?;
        let session = serde_json::from_slice::<ActiveSessionRecord>(&contents)
            .map_err(SessionStoreError::Deserialize)?;
        session.validate().map_err(|error| SessionStoreError::InvalidRecord(error.to_string()))?;
        Ok(Some(session))
    }

    fn clear(&self) -> Result<(), SessionStoreError> {
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(SessionStoreError::Delete)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error("failed to create session directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("failed to read session file: {0}")]
    Read(std::io::Error),
    #[error("failed to serialize session record: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize session record: {0}")]
    Deserialize(serde_json::Error),
    #[error("failed to write session file: {0}")]
    Write(std::io::Error),
    #[error("failed to delete session file: {0}")]
    Delete(std::io::Error),
    #[error("invalid session record: {0}")]
    InvalidRecord(String),
}
