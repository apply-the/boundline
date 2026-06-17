use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::session::{
    ActiveSessionRecord, active_session_pointer_ref, legacy_session_record_ref, session_record_ref,
    session_storage_root_ref,
};

pub trait SessionStore: Send + Sync {
    fn persist(&self, session: &ActiveSessionRecord) -> Result<PathBuf, SessionStoreError>;
    fn load(&self) -> Result<Option<ActiveSessionRecord>, SessionStoreError>;
    fn clear(&self) -> Result<(), SessionStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileSessionStore {
    path: PathBuf,
    workspace_root: Option<PathBuf>,
}

impl FileSessionStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), workspace_root: None }
    }

    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        let workspace_root = workspace_ref.as_ref().to_path_buf();
        Self {
            path: workspace_root.join(legacy_session_record_ref()),
            workspace_root: Some(workspace_root),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_session(
        &self,
        session_id: &str,
    ) -> Result<Option<ActiveSessionRecord>, SessionStoreError> {
        if let Some(session_path) = self.session_path_for_id(session_id)
            && session_path.is_file()
        {
            return Self::load_record_from_path(&session_path).map(Some);
        }

        if self.path.is_file() {
            let record = Self::load_record_from_path(&self.path)?;
            if record.session_id == session_id {
                return Ok(Some(record));
            }
        }

        Ok(None)
    }

    pub fn list_sessions(&self) -> Result<Vec<ActiveSessionRecord>, SessionStoreError> {
        let mut records = Vec::new();

        if let Some(sessions_root) = self.sessions_root_path()
            && sessions_root.is_dir()
        {
            for entry in fs::read_dir(sessions_root).map_err(SessionStoreError::Read)? {
                let entry = entry.map_err(SessionStoreError::Read)?;
                if !entry.file_type().map_err(SessionStoreError::Read)?.is_dir() {
                    continue;
                }

                let session_id = entry.file_name().to_string_lossy().into_owned();
                if session_id.trim().is_empty() {
                    continue;
                }

                if let Some(record) = self.load_session(&session_id)? {
                    records.push(record);
                }
            }
        }

        if records.is_empty() && self.path.is_file() {
            records.push(Self::load_record_from_path(&self.path)?);
        }

        records.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.session_id.cmp(&left.session_id))
        });

        Ok(records)
    }

    pub fn select_active_session(
        &self,
        session_id: &str,
    ) -> Result<Option<ActiveSessionRecord>, SessionStoreError> {
        let Some(record) = self.load_session(session_id)? else {
            return Ok(None);
        };

        self.persist_active_projection(&record)?;
        Ok(Some(record))
    }

    pub fn persist_without_select(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<PathBuf, SessionStoreError> {
        session.validate().map_err(|error| SessionStoreError::InvalidRecord(error.to_string()))?;

        let contents = serde_json::to_vec_pretty(session).map_err(SessionStoreError::Serialize)?;

        if let Some(session_path) = self.session_path_for_id(&session.session_id) {
            if let Some(parent) = session_path.parent() {
                fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
            }
            fs::write(&session_path, &contents).map_err(SessionStoreError::Write)?;
            return Ok(session_path);
        }

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
        }
        fs::write(&self.path, &contents).map_err(SessionStoreError::Write)?;
        Ok(self.path.clone())
    }

    fn active_session_pointer_path(&self) -> Option<PathBuf> {
        self.workspace_root
            .as_ref()
            .map(|workspace_root| workspace_root.join(active_session_pointer_ref()))
    }

    fn sessions_root_path(&self) -> Option<PathBuf> {
        self.workspace_root
            .as_ref()
            .map(|workspace_root| workspace_root.join(session_storage_root_ref()))
    }

    fn session_path_for_id(&self, session_id: &str) -> Option<PathBuf> {
        self.workspace_root
            .as_ref()
            .map(|workspace_root| workspace_root.join(session_record_ref(session_id)))
    }

    fn load_record_from_path(path: &Path) -> Result<ActiveSessionRecord, SessionStoreError> {
        let contents = fs::read(path).map_err(SessionStoreError::Read)?;
        let session = serde_json::from_slice::<ActiveSessionRecord>(&contents)
            .map_err(SessionStoreError::Deserialize)?;
        session.validate().map_err(|error| SessionStoreError::InvalidRecord(error.to_string()))?;
        Ok(session)
    }

    fn persist_active_projection(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<(), SessionStoreError> {
        let contents = serde_json::to_vec_pretty(session).map_err(SessionStoreError::Serialize)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
        }
        fs::write(&self.path, &contents).map_err(SessionStoreError::Write)?;

        if let Some(pointer_path) = self.active_session_pointer_path() {
            if let Some(parent) = pointer_path.parent() {
                fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
            }
            fs::write(pointer_path, &session.session_id).map_err(SessionStoreError::Write)?;
        }

        Ok(())
    }

    fn resolved_active_session_path(&self) -> Result<Option<PathBuf>, SessionStoreError> {
        let Some(pointer_path) = self.active_session_pointer_path() else {
            return Ok(self.path.exists().then(|| self.path.clone()));
        };

        if pointer_path.is_file() {
            let session_id = fs::read_to_string(&pointer_path).map_err(SessionStoreError::Read)?;
            let session_id = session_id.trim();
            if session_id.is_empty() {
                return Err(SessionStoreError::InvalidActivePointer(format!(
                    "{} is empty",
                    pointer_path.display()
                )));
            }

            if let Some(session_path) = self.session_path_for_id(session_id)
                && session_path.is_file()
            {
                return Ok(Some(session_path));
            }
        }

        Ok(self.path.exists().then(|| self.path.clone()))
    }
}

impl SessionStore for FileSessionStore {
    fn persist(&self, session: &ActiveSessionRecord) -> Result<PathBuf, SessionStoreError> {
        session.validate().map_err(|error| SessionStoreError::InvalidRecord(error.to_string()))?;

        let contents = serde_json::to_vec_pretty(session).map_err(SessionStoreError::Serialize)?;

        if let Some(session_path) = self.session_path_for_id(&session.session_id) {
            if let Some(parent) = session_path.parent() {
                fs::create_dir_all(parent).map_err(SessionStoreError::CreateDirectory)?;
            }
            fs::write(&session_path, &contents).map_err(SessionStoreError::Write)?;

            self.persist_active_projection(session)?;

            return Ok(session_path);
        }

        self.persist_active_projection(session)?;
        Ok(self.path.clone())
    }

    fn load(&self) -> Result<Option<ActiveSessionRecord>, SessionStoreError> {
        let Some(path) = self.resolved_active_session_path()? else {
            return Ok(None);
        };

        Self::load_record_from_path(&path).map(Some)
    }

    fn clear(&self) -> Result<(), SessionStoreError> {
        if self.path.exists() {
            fs::remove_file(&self.path).map_err(SessionStoreError::Delete)?;
        }

        if let Some(pointer_path) = self.active_session_pointer_path()
            && pointer_path.exists()
        {
            fs::remove_file(pointer_path).map_err(SessionStoreError::Delete)?;
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
    #[error("invalid active session pointer: {0}")]
    InvalidActivePointer(String),
    #[error("invalid session record: {0}")]
    InvalidRecord(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{FileSessionStore, SessionStore};
    use crate::domain::session::{
        ActiveSessionRecord, SessionStatus, active_session_pointer_ref, legacy_session_record_ref,
        session_record_ref,
    };

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn build_record(workspace: &std::path::Path, session_id: &str) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: session_id.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    #[test]
    fn persist_for_workspace_writes_session_root_and_compatibility_projection() {
        let workspace = temp_workspace("boundline-session-store-root");
        let store = FileSessionStore::for_workspace(&workspace);
        let record = build_record(&workspace, "1748062123456-abcd1234-fix-add-test");

        let persisted = store.persist(&record).unwrap();

        assert_eq!(persisted, workspace.join(session_record_ref(&record.session_id)));
        assert!(workspace.join(legacy_session_record_ref()).is_file());
        assert_eq!(
            fs::read_to_string(workspace.join(active_session_pointer_ref())).unwrap(),
            record.session_id
        );
        assert_eq!(store.load().unwrap(), Some(record));
    }

    #[test]
    fn load_for_workspace_falls_back_to_legacy_session_projection() {
        let workspace = temp_workspace("boundline-session-store-legacy");
        let store = FileSessionStore::for_workspace(&workspace);
        let record = build_record(&workspace, "legacy-session-id");
        let legacy_path = workspace.join(legacy_session_record_ref());

        fs::create_dir_all(legacy_path.parent().unwrap()).unwrap();
        fs::write(&legacy_path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();

        assert_eq!(store.load().unwrap(), Some(record));
    }

    #[test]
    fn clear_for_workspace_removes_active_pointer_but_preserves_session_history() {
        let workspace = temp_workspace("boundline-session-store-clear");
        let store = FileSessionStore::for_workspace(&workspace);
        let record = build_record(&workspace, "1748062123456-abcd1234-history");
        let session_path = workspace.join(session_record_ref(&record.session_id));

        store.persist(&record).unwrap();
        store.clear().unwrap();

        assert!(session_path.is_file());
        assert!(!workspace.join(legacy_session_record_ref()).exists());
        assert!(!workspace.join(active_session_pointer_ref()).exists());
        assert_eq!(store.load().unwrap(), None);
    }

    #[test]
    fn persist_without_select_preserves_active_pointer_and_projection() -> Result<(), String> {
        let workspace = temp_workspace("boundline-session-store-without-select");
        let store = FileSessionStore::for_workspace(&workspace);
        let active = build_record(&workspace, "1748062123456-abcd1234-active");
        let historical = build_record(&workspace, "1748062123457-abcd1234-history");

        store.persist(&active).map_err(|error| error.to_string())?;
        store.persist_without_select(&historical).map_err(|error| error.to_string())?;

        let active_pointer = fs::read_to_string(workspace.join(active_session_pointer_ref()))
            .map_err(|error| error.to_string())?;
        if active_pointer != active.session_id {
            return Err(format!(
                "expected active pointer {}, got {}",
                active.session_id, active_pointer
            ));
        }

        let projection = fs::read(workspace.join(legacy_session_record_ref()))
            .map_err(|error| error.to_string())?;
        let projection: ActiveSessionRecord =
            serde_json::from_slice(&projection).map_err(|error| error.to_string())?;
        if projection.session_id != active.session_id {
            return Err(format!(
                "expected compatibility projection {}, got {}",
                active.session_id, projection.session_id
            ));
        }

        let loaded_active = store
            .load()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected active session record".to_string())?;
        if loaded_active != active {
            return Err(format!("expected active record {active:?}, got {loaded_active:?}"));
        }

        let loaded_historical = store
            .load_session(&historical.session_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected historical session record".to_string())?;
        if loaded_historical != historical {
            return Err(format!(
                "expected historical record {historical:?}, got {loaded_historical:?}"
            ));
        }

        if !workspace.join(session_record_ref(&historical.session_id)).is_file() {
            return Err("expected persisted historical session file".to_string());
        }

        Ok(())
    }
}
