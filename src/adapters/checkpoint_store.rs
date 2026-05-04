use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::checkpoint::{
    CheckpointAuthorityScope, CheckpointFileRecord, CheckpointFileState, CheckpointManifest,
    CheckpointRestoreMode, CheckpointRestoreOutcome, CheckpointRestoreRecord,
};
use crate::domain::session::SessionCommand;
use crate::domain::trace::current_timestamp_millis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointCaptureRequest {
    pub checkpoint_id: String,
    pub group_id: Option<String>,
    pub workspace_ref: String,
    pub authority_scope: CheckpointAuthorityScope,
    pub trigger_command: SessionCommand,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub step_id: Option<String>,
    pub candidate_paths: Vec<String>,
    pub already_modified_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointRestoreResult {
    pub manifest: CheckpointManifest,
    pub record: CheckpointRestoreRecord,
}

#[derive(Debug, Clone)]
pub struct FileCheckpointStore {
    root: PathBuf,
}

impl FileCheckpointStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        Self::new(workspace_ref.as_ref().join(".boundline").join("checkpoints"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn persist(&self, manifest: &CheckpointManifest) -> Result<PathBuf, CheckpointStoreError> {
        manifest
            .validate()
            .map_err(|error| CheckpointStoreError::InvalidManifest(error.to_string()))?;
        fs::create_dir_all(&self.root).map_err(CheckpointStoreError::CreateDirectory)?;
        let path = self.manifest_path(&manifest.checkpoint_id);
        let contents =
            serde_json::to_vec_pretty(manifest).map_err(CheckpointStoreError::Serialize)?;
        fs::write(&path, contents).map_err(CheckpointStoreError::Write)?;
        Ok(path)
    }

    pub fn load(
        &self,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointManifest>, CheckpointStoreError> {
        let path = self.manifest_path(checkpoint_id);
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read(&path).map_err(CheckpointStoreError::Read)?;
        let manifest = serde_json::from_slice::<CheckpointManifest>(&contents)
            .map_err(CheckpointStoreError::Deserialize)?;
        manifest
            .validate()
            .map_err(|error| CheckpointStoreError::InvalidManifest(error.to_string()))?;
        Ok(Some(manifest))
    }

    pub fn list(&self) -> Result<Vec<CheckpointManifest>, CheckpointStoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut manifests = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(CheckpointStoreError::ReadDirectory)? {
            let entry = entry.map_err(CheckpointStoreError::ReadDirectory)?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("json")
            {
                continue;
            }

            let contents = fs::read(&path).map_err(CheckpointStoreError::Read)?;
            let manifest = serde_json::from_slice::<CheckpointManifest>(&contents)
                .map_err(CheckpointStoreError::Deserialize)?;
            manifest
                .validate()
                .map_err(|error| CheckpointStoreError::InvalidManifest(error.to_string()))?;
            manifests.push(manifest);
        }

        manifests.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| left.checkpoint_id.cmp(&right.checkpoint_id))
        });
        Ok(manifests)
    }

    pub fn load_group(
        &self,
        group_id: &str,
    ) -> Result<Vec<CheckpointManifest>, CheckpointStoreError> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|manifest| manifest.group_id.as_deref() == Some(group_id))
            .collect())
    }

    pub fn restore_conflicts(
        &self,
        checkpoint_id: &str,
    ) -> Result<Option<Vec<String>>, CheckpointStoreError> {
        let Some(manifest) = self.load(checkpoint_id)? else {
            return Ok(None);
        };
        Ok(Some(self.collect_restore_conflicts(&manifest)?))
    }

    pub fn refuse_restore(
        &self,
        checkpoint_id: &str,
        mode: CheckpointRestoreMode,
        conflicting_paths: Vec<String>,
    ) -> Result<CheckpointRestoreResult, CheckpointStoreError> {
        let Some(mut manifest) = self.load(checkpoint_id)? else {
            return Err(CheckpointStoreError::MissingCheckpoint(checkpoint_id.to_string()));
        };

        self.persist_restore_result(
            &mut manifest,
            mode,
            CheckpointRestoreOutcome::Refused,
            conflicting_paths,
            Vec::new(),
        )
    }

    pub fn capture(
        &self,
        request: CheckpointCaptureRequest,
    ) -> Result<CheckpointManifest, CheckpointStoreError> {
        let workspace = PathBuf::from(&request.workspace_ref);
        let already_modified = request.already_modified_paths.into_iter().collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::new();
        let mut captured_files = Vec::new();

        for path in request.candidate_paths {
            if path.trim().is_empty() || !seen.insert(path.clone()) {
                continue;
            }

            let absolute_path = workspace.join(&path);
            let captured_contents = read_optional_contents(&absolute_path)?;
            let capture_state = match captured_contents.as_ref() {
                Some(_) if already_modified.contains(&path) => CheckpointFileState::AlreadyModified,
                Some(_) => CheckpointFileState::PreExisting,
                None => CheckpointFileState::NewlyCreated,
            };
            captured_files.push(CheckpointFileRecord::from_capture(
                &request.workspace_ref,
                path,
                capture_state,
                captured_contents,
            ));
        }

        let manifest = CheckpointManifest {
            checkpoint_id: request.checkpoint_id,
            group_id: request.group_id,
            workspace_ref: request.workspace_ref,
            authority_scope: request.authority_scope,
            trigger_command: request.trigger_command,
            session_id: request.session_id,
            task_id: request.task_id,
            step_id: request.step_id,
            created_at: current_timestamp_millis(),
            captured_files,
            restore_history: Vec::new(),
        };
        self.persist(&manifest)?;
        Ok(manifest)
    }

    pub fn refresh_observed_state(
        &self,
        checkpoint_id: &str,
    ) -> Result<Option<CheckpointManifest>, CheckpointStoreError> {
        let Some(mut manifest) = self.load(checkpoint_id)? else {
            return Ok(None);
        };

        for file in &mut manifest.captured_files {
            let absolute_path = Path::new(&manifest.workspace_ref).join(&file.path);
            let current_contents = read_optional_contents(&absolute_path)?;
            file.update_observed_state(current_contents.as_deref());
        }

        self.persist(&manifest)?;
        Ok(Some(manifest))
    }

    pub fn restore(
        &self,
        checkpoint_id: &str,
        mode: CheckpointRestoreMode,
    ) -> Result<CheckpointRestoreResult, CheckpointStoreError> {
        let Some(mut manifest) = self.load(checkpoint_id)? else {
            return Err(CheckpointStoreError::MissingCheckpoint(checkpoint_id.to_string()));
        };

        let workspace = PathBuf::from(&manifest.workspace_ref);
        let conflicting_paths = if mode == CheckpointRestoreMode::Safe {
            self.collect_restore_conflicts(&manifest)?
        } else {
            Vec::new()
        };
        let mut restored_paths = Vec::new();

        if !conflicting_paths.is_empty() {
            return self.persist_restore_result(
                &mut manifest,
                mode,
                CheckpointRestoreOutcome::Refused,
                conflicting_paths,
                restored_paths,
            );
        }

        for file in &manifest.captured_files {
            let absolute_path = workspace.join(&file.path);
            if file.restore_requires_delete() {
                if absolute_path.exists() {
                    fs::remove_file(&absolute_path).map_err(CheckpointStoreError::Delete)?;
                }
            } else if let Some(contents) = file.captured_contents.as_ref() {
                if let Some(parent) = absolute_path.parent() {
                    fs::create_dir_all(parent).map_err(CheckpointStoreError::CreateDirectory)?;
                }
                fs::write(&absolute_path, contents).map_err(CheckpointStoreError::Write)?;
            }
            restored_paths.push(file.path.clone());
        }

        self.persist_restore_result(
            &mut manifest,
            mode,
            CheckpointRestoreOutcome::Succeeded,
            Vec::new(),
            restored_paths,
        )
    }

    fn manifest_path(&self, checkpoint_id: &str) -> PathBuf {
        self.root.join(format!("{checkpoint_id}.json"))
    }

    fn collect_restore_conflicts(
        &self,
        manifest: &CheckpointManifest,
    ) -> Result<Vec<String>, CheckpointStoreError> {
        let workspace = PathBuf::from(&manifest.workspace_ref);
        let mut conflicting_paths = Vec::new();

        for file in &manifest.captured_files {
            let absolute_path = workspace.join(&file.path);
            let current_contents = read_optional_contents(&absolute_path)?;
            let matches_observed = file.current_matches_observed_state(current_contents.as_deref());
            let matches_captured = file.current_matches_captured_state(current_contents.as_deref());
            if !matches_observed && !matches_captured {
                conflicting_paths.push(file.path.clone());
            }
        }

        Ok(conflicting_paths)
    }

    fn persist_restore_result(
        &self,
        manifest: &mut CheckpointManifest,
        mode: CheckpointRestoreMode,
        outcome: CheckpointRestoreOutcome,
        conflicting_paths: Vec<String>,
        restored_paths: Vec<String>,
    ) -> Result<CheckpointRestoreResult, CheckpointStoreError> {
        let record = CheckpointRestoreRecord {
            restore_id: format!("restore-{}", current_timestamp_millis()),
            requested_at: current_timestamp_millis(),
            mode,
            outcome,
            conflicting_paths,
            restored_paths,
        };
        manifest.add_restore_record(record.clone());
        self.persist(manifest)?;

        Ok(CheckpointRestoreResult { manifest: manifest.clone(), record })
    }
}

fn read_optional_contents(path: &Path) -> Result<Option<String>, CheckpointStoreError> {
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path).map(Some).map_err(CheckpointStoreError::Read)
}

#[derive(Debug, Error)]
pub enum CheckpointStoreError {
    #[error("failed to create checkpoint directory: {0}")]
    CreateDirectory(std::io::Error),
    #[error("failed to list checkpoints: {0}")]
    ReadDirectory(std::io::Error),
    #[error("failed to read checkpoint file: {0}")]
    Read(std::io::Error),
    #[error("failed to serialize checkpoint manifest: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize checkpoint manifest: {0}")]
    Deserialize(serde_json::Error),
    #[error("failed to write checkpoint file: {0}")]
    Write(std::io::Error),
    #[error("failed to delete checkpoint-managed file: {0}")]
    Delete(std::io::Error),
    #[error("invalid checkpoint manifest: {0}")]
    InvalidManifest(String),
    #[error("checkpoint '{0}' was not found")]
    MissingCheckpoint(String),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::{CheckpointCaptureRequest, FileCheckpointStore};
    use crate::domain::checkpoint::{
        CheckpointAuthorityScope, CheckpointFileState, CheckpointRestoreMode,
        CheckpointRestoreOutcome,
    };
    use crate::domain::session::SessionCommand;

    fn temp_workspace(prefix: &str) -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join("src")).unwrap();
        workspace
    }

    #[test]
    fn capture_refresh_and_restore_round_trip_workspace_files() {
        let workspace = temp_workspace("boundline-checkpoint-store-round-trip");
        fs::write(workspace.join("src/lib.rs"), "before").unwrap();

        let store = FileCheckpointStore::for_workspace(&workspace);
        let manifest = store
            .capture(CheckpointCaptureRequest {
                checkpoint_id: "checkpoint-1".to_string(),
                group_id: None,
                workspace_ref: workspace.to_string_lossy().into_owned(),
                authority_scope: CheckpointAuthorityScope::Workspace,
                trigger_command: SessionCommand::Run,
                session_id: Some("session-1".to_string()),
                task_id: Some("task-1".to_string()),
                step_id: Some("step-1".to_string()),
                candidate_paths: vec!["src/lib.rs".to_string(), "src/new.rs".to_string()],
                already_modified_paths: Vec::new(),
            })
            .unwrap();

        assert_eq!(manifest.captured_files.len(), 2);
        assert_eq!(manifest.captured_files[0].capture_state, CheckpointFileState::PreExisting);
        assert_eq!(manifest.captured_files[1].capture_state, CheckpointFileState::NewlyCreated);

        fs::write(workspace.join("src/lib.rs"), "after-run").unwrap();
        fs::write(workspace.join("src/new.rs"), "created-by-run").unwrap();
        store.refresh_observed_state("checkpoint-1").unwrap();

        let result = store.restore("checkpoint-1", CheckpointRestoreMode::Safe).unwrap();
        assert_eq!(result.record.outcome, CheckpointRestoreOutcome::Succeeded);
        assert_eq!(fs::read_to_string(workspace.join("src/lib.rs")).unwrap(), "before");
        assert!(!workspace.join("src/new.rs").exists());
    }

    #[test]
    fn safe_restore_refuses_after_newer_edits_and_force_restores_original_contents() {
        let workspace = temp_workspace("boundline-checkpoint-store-refusal");
        fs::write(workspace.join("src/lib.rs"), "before").unwrap();

        let store = FileCheckpointStore::for_workspace(&workspace);
        store
            .capture(CheckpointCaptureRequest {
                checkpoint_id: "checkpoint-2".to_string(),
                group_id: Some("group-1".to_string()),
                workspace_ref: workspace.to_string_lossy().into_owned(),
                authority_scope: CheckpointAuthorityScope::Workspace,
                trigger_command: SessionCommand::Step,
                session_id: Some("session-2".to_string()),
                task_id: Some("task-2".to_string()),
                step_id: Some("step-2".to_string()),
                candidate_paths: vec!["src/lib.rs".to_string()],
                already_modified_paths: Vec::new(),
            })
            .unwrap();

        fs::write(workspace.join("src/lib.rs"), "after-run").unwrap();
        store.refresh_observed_state("checkpoint-2").unwrap();
        fs::write(workspace.join("src/lib.rs"), "edited-after-run").unwrap();

        let conflicts = store.restore_conflicts("checkpoint-2").unwrap().unwrap();
        assert_eq!(conflicts, vec!["src/lib.rs".to_string()]);

        let refused = store.restore("checkpoint-2", CheckpointRestoreMode::Safe).unwrap();
        assert_eq!(refused.record.outcome, CheckpointRestoreOutcome::Refused);
        assert_eq!(refused.record.conflicting_paths, vec!["src/lib.rs".to_string()]);
        assert_eq!(fs::read_to_string(workspace.join("src/lib.rs")).unwrap(), "edited-after-run");

        let forced = store.restore("checkpoint-2", CheckpointRestoreMode::Forced).unwrap();
        assert_eq!(forced.record.outcome, CheckpointRestoreOutcome::Succeeded);
        assert_eq!(fs::read_to_string(workspace.join("src/lib.rs")).unwrap(), "before");

        let grouped = store.load_group("group-1").unwrap();
        assert_eq!(grouped.len(), 1);
    }

    #[test]
    fn store_helpers_cover_missing_state_and_custom_root_access() {
        let custom_root = std::env::temp_dir()
            .join(format!("boundline-checkpoint-store-root-{}", Uuid::new_v4()));
        let store = FileCheckpointStore::new(custom_root.clone());

        assert_eq!(store.root(), custom_root.as_path());
        assert!(store.list().unwrap().is_empty());
        assert!(store.load("missing").unwrap().is_none());
        assert!(store.restore_conflicts("missing").unwrap().is_none());
        assert!(store.refresh_observed_state("missing").unwrap().is_none());
        assert!(matches!(
            store.refuse_restore(
                "missing",
                CheckpointRestoreMode::Safe,
                vec!["src/lib.rs".to_string()],
            ),
            Err(super::CheckpointStoreError::MissingCheckpoint(id)) if id == "missing"
        ));
        assert!(matches!(
            store.restore("missing", CheckpointRestoreMode::Safe),
            Err(super::CheckpointStoreError::MissingCheckpoint(id)) if id == "missing"
        ));
    }

    #[test]
    fn capture_filters_duplicate_paths_and_restore_recreates_missing_parents() {
        let workspace = temp_workspace("boundline-checkpoint-store-filtering");
        fs::create_dir_all(workspace.join("nested/dir")).unwrap();
        fs::write(workspace.join("nested/dir/lib.rs"), "before").unwrap();

        let store = FileCheckpointStore::for_workspace(&workspace);
        let manifest = store
            .capture(CheckpointCaptureRequest {
                checkpoint_id: "checkpoint-3".to_string(),
                group_id: Some("group-2".to_string()),
                workspace_ref: workspace.to_string_lossy().into_owned(),
                authority_scope: CheckpointAuthorityScope::Workspace,
                trigger_command: SessionCommand::Run,
                session_id: Some("session-3".to_string()),
                task_id: Some("task-3".to_string()),
                step_id: None,
                candidate_paths: vec![
                    String::new(),
                    "nested/dir/lib.rs".to_string(),
                    "nested/dir/lib.rs".to_string(),
                    "scratch/generated.rs".to_string(),
                ],
                already_modified_paths: vec!["nested/dir/lib.rs".to_string()],
            })
            .unwrap();
        store
            .capture(CheckpointCaptureRequest {
                checkpoint_id: "checkpoint-4".to_string(),
                group_id: None,
                workspace_ref: workspace.to_string_lossy().into_owned(),
                authority_scope: CheckpointAuthorityScope::Workspace,
                trigger_command: SessionCommand::Step,
                session_id: Some("session-4".to_string()),
                task_id: Some("task-4".to_string()),
                step_id: None,
                candidate_paths: vec!["nested/dir/lib.rs".to_string()],
                already_modified_paths: Vec::new(),
            })
            .unwrap();

        assert_eq!(manifest.captured_files.len(), 2);
        assert_eq!(manifest.captured_files[0].capture_state, CheckpointFileState::AlreadyModified);
        assert_eq!(manifest.captured_files[1].capture_state, CheckpointFileState::NewlyCreated);

        fs::write(store.root().join("ignore.txt"), "ignored").unwrap();
        let listed = store.list().unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(store.load_group("group-2").unwrap().len(), 1);

        fs::remove_dir_all(workspace.join("nested")).unwrap();
        fs::create_dir_all(workspace.join("scratch")).unwrap();
        fs::write(workspace.join("scratch/generated.rs"), "generated").unwrap();

        let result = store.restore("checkpoint-3", CheckpointRestoreMode::Forced).unwrap();
        assert_eq!(result.record.outcome, CheckpointRestoreOutcome::Succeeded);
        assert_eq!(fs::read_to_string(workspace.join("nested/dir/lib.rs")).unwrap(), "before");
        assert!(!workspace.join("scratch/generated.rs").exists());
    }
}
