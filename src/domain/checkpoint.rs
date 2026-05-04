use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::session::SessionCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointAuthorityScope {
    Workspace,
    ClusterPrimary,
    ClusterMember,
}

impl CheckpointAuthorityScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::ClusterPrimary => "cluster_primary",
            Self::ClusterMember => "cluster_member",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointFileState {
    PreExisting,
    NewlyCreated,
    Deleted,
    AlreadyModified,
}

impl CheckpointFileState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreExisting => "pre_existing",
            Self::NewlyCreated => "newly_created",
            Self::Deleted => "deleted",
            Self::AlreadyModified => "already_modified",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointRestoreMode {
    Safe,
    Forced,
}

impl CheckpointRestoreMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Forced => "forced",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointRestoreOutcome {
    Succeeded,
    Refused,
    Failed,
}

impl CheckpointRestoreOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Refused => "refused",
            Self::Failed => "failed",
        }
    }
}

pub fn checkpoint_fingerprint(contents: &str) -> String {
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointFileRecord {
    pub path: String,
    pub workspace_ref: String,
    pub capture_state: CheckpointFileState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_contents: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_after_capture_exists: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_after_capture_fingerprint: Option<String>,
}

impl CheckpointFileRecord {
    pub fn from_capture(
        workspace_ref: impl Into<String>,
        path: impl Into<String>,
        capture_state: CheckpointFileState,
        captured_contents: Option<String>,
    ) -> Self {
        let captured_fingerprint = captured_contents.as_deref().map(checkpoint_fingerprint);
        let observed_after_capture_exists = Some(captured_contents.is_some());
        let observed_after_capture_fingerprint = captured_fingerprint.clone();
        Self {
            path: path.into(),
            workspace_ref: workspace_ref.into(),
            capture_state,
            captured_contents,
            captured_fingerprint,
            observed_after_capture_exists,
            observed_after_capture_fingerprint,
        }
    }

    pub fn validate(&self) -> Result<(), CheckpointValidationError> {
        if self.path.trim().is_empty() {
            return Err(CheckpointValidationError::MissingPath);
        }

        if self.workspace_ref.trim().is_empty() {
            return Err(CheckpointValidationError::MissingWorkspaceRef);
        }

        if self.path.starts_with('/') || self.path.contains("..") {
            return Err(CheckpointValidationError::InvalidWorkspacePath(self.path.clone()));
        }

        Ok(())
    }

    pub fn update_observed_state(&mut self, current_contents: Option<&str>) {
        self.observed_after_capture_exists = Some(current_contents.is_some());
        self.observed_after_capture_fingerprint = current_contents.map(checkpoint_fingerprint);
    }

    pub fn current_matches_observed_state(&self, current_contents: Option<&str>) -> bool {
        self.observed_after_capture_exists == Some(current_contents.is_some())
            && self.observed_after_capture_fingerprint
                == current_contents.map(checkpoint_fingerprint)
    }

    pub fn current_matches_captured_state(&self, current_contents: Option<&str>) -> bool {
        self.captured_contents.as_deref() == current_contents
    }

    pub fn restore_requires_delete(&self) -> bool {
        self.captured_contents.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointRestoreRecord {
    pub restore_id: String,
    pub requested_at: u64,
    pub mode: CheckpointRestoreMode,
    pub outcome: CheckpointRestoreOutcome,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicting_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restored_paths: Vec<String>,
}

impl CheckpointRestoreRecord {
    pub fn validate(&self) -> Result<(), CheckpointValidationError> {
        if self.restore_id.trim().is_empty() {
            return Err(CheckpointValidationError::MissingRestoreId);
        }

        if self.outcome == CheckpointRestoreOutcome::Refused && self.conflicting_paths.is_empty() {
            return Err(CheckpointValidationError::MissingConflictingPaths);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointManifest {
    pub checkpoint_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub workspace_ref: String,
    pub authority_scope: CheckpointAuthorityScope,
    pub trigger_command: SessionCommand,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub captured_files: Vec<CheckpointFileRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restore_history: Vec<CheckpointRestoreRecord>,
}

impl CheckpointManifest {
    pub fn validate(&self) -> Result<(), CheckpointValidationError> {
        if self.checkpoint_id.trim().is_empty() {
            return Err(CheckpointValidationError::MissingCheckpointId);
        }

        if self.workspace_ref.trim().is_empty() {
            return Err(CheckpointValidationError::MissingWorkspaceRef);
        }

        if self.captured_files.is_empty() {
            return Err(CheckpointValidationError::MissingCapturedFiles);
        }

        for file in &self.captured_files {
            file.validate()?;
        }

        for record in &self.restore_history {
            record.validate()?;
        }

        Ok(())
    }

    pub fn add_restore_record(&mut self, record: CheckpointRestoreRecord) {
        self.restore_history.push(record);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CheckpointValidationError {
    #[error("checkpoint id must not be empty")]
    MissingCheckpointId,
    #[error("checkpoint restore id must not be empty")]
    MissingRestoreId,
    #[error("checkpoint workspace reference must not be empty")]
    MissingWorkspaceRef,
    #[error("checkpoint path must not be empty")]
    MissingPath,
    #[error("checkpoint path must remain workspace-relative: {0}")]
    InvalidWorkspacePath(String),
    #[error("checkpoint must include at least one captured file")]
    MissingCapturedFiles,
    #[error("refused restore must include conflicting paths")]
    MissingConflictingPaths,
}

#[cfg(test)]
mod tests {
    use super::{
        CheckpointAuthorityScope, CheckpointFileRecord, CheckpointFileState, CheckpointManifest,
        CheckpointRestoreMode, CheckpointRestoreOutcome, CheckpointRestoreRecord,
        CheckpointValidationError, checkpoint_fingerprint,
    };
    use crate::domain::session::SessionCommand;

    #[test]
    fn file_record_tracks_captured_and_observed_state() {
        let mut record = CheckpointFileRecord::from_capture(
            "/tmp/workspace",
            "src/lib.rs",
            CheckpointFileState::PreExisting,
            Some("before".to_string()),
        );

        assert!(record.validate().is_ok());
        assert!(record.current_matches_captured_state(Some("before")));
        assert!(record.current_matches_observed_state(Some("before")));

        record.update_observed_state(Some("after"));
        assert!(!record.current_matches_observed_state(Some("before")));
        assert!(record.current_matches_observed_state(Some("after")));
        assert_eq!(
            record.captured_fingerprint.as_deref(),
            Some(checkpoint_fingerprint("before").as_str())
        );
        assert_eq!(
            record.observed_after_capture_fingerprint.as_deref(),
            Some(checkpoint_fingerprint("after").as_str())
        );
    }

    #[test]
    fn manifest_and_restore_record_validation_reject_invalid_shapes() {
        let record = CheckpointRestoreRecord {
            restore_id: "restore-1".to_string(),
            requested_at: 1,
            mode: CheckpointRestoreMode::Safe,
            outcome: CheckpointRestoreOutcome::Refused,
            conflicting_paths: Vec::new(),
            restored_paths: Vec::new(),
        };
        assert_eq!(
            record.validate().unwrap_err(),
            CheckpointValidationError::MissingConflictingPaths
        );

        let manifest = CheckpointManifest {
            checkpoint_id: "checkpoint-1".to_string(),
            group_id: None,
            workspace_ref: "/tmp/workspace".to_string(),
            authority_scope: CheckpointAuthorityScope::Workspace,
            trigger_command: SessionCommand::Run,
            session_id: Some("session-1".to_string()),
            task_id: Some("task-1".to_string()),
            step_id: None,
            created_at: 1,
            captured_files: Vec::new(),
            restore_history: Vec::new(),
        };
        assert_eq!(
            manifest.validate().unwrap_err(),
            CheckpointValidationError::MissingCapturedFiles
        );
    }

    #[test]
    fn enums_expose_stable_wire_text() {
        assert_eq!(CheckpointAuthorityScope::ClusterMember.as_str(), "cluster_member");
        assert_eq!(CheckpointFileState::AlreadyModified.as_str(), "already_modified");
        assert_eq!(CheckpointRestoreMode::Forced.as_str(), "forced");
        assert_eq!(CheckpointRestoreOutcome::Succeeded.as_str(), "succeeded");
    }

    #[test]
    fn validation_helpers_cover_remaining_error_paths() {
        assert_eq!(CheckpointAuthorityScope::ClusterPrimary.as_str(), "cluster_primary");
        assert_eq!(CheckpointFileState::PreExisting.as_str(), "pre_existing");
        assert_eq!(CheckpointFileState::NewlyCreated.as_str(), "newly_created");
        assert_eq!(CheckpointFileState::Deleted.as_str(), "deleted");
        assert_eq!(CheckpointRestoreOutcome::Failed.as_str(), "failed");

        let missing_path = CheckpointFileRecord::from_capture(
            "/tmp/workspace",
            "",
            CheckpointFileState::Deleted,
            None,
        );
        assert_eq!(missing_path.validate().unwrap_err(), CheckpointValidationError::MissingPath);

        let missing_workspace = CheckpointFileRecord::from_capture(
            "",
            "src/lib.rs",
            CheckpointFileState::PreExisting,
            Some("before".to_string()),
        );
        assert_eq!(
            missing_workspace.validate().unwrap_err(),
            CheckpointValidationError::MissingWorkspaceRef
        );

        let invalid_path = CheckpointFileRecord::from_capture(
            "/tmp/workspace",
            "../src/lib.rs",
            CheckpointFileState::PreExisting,
            Some("before".to_string()),
        );
        assert_eq!(
            invalid_path.validate().unwrap_err(),
            CheckpointValidationError::InvalidWorkspacePath("../src/lib.rs".to_string())
        );

        let mut deleted = CheckpointFileRecord::from_capture(
            "/tmp/workspace",
            "src/deleted.rs",
            CheckpointFileState::Deleted,
            None,
        );
        assert!(deleted.restore_requires_delete());
        deleted.update_observed_state(None);
        assert!(deleted.current_matches_observed_state(None));

        let missing_restore_id = CheckpointRestoreRecord {
            restore_id: String::new(),
            requested_at: 1,
            mode: CheckpointRestoreMode::Safe,
            outcome: CheckpointRestoreOutcome::Succeeded,
            conflicting_paths: Vec::new(),
            restored_paths: vec!["src/lib.rs".to_string()],
        };
        assert_eq!(
            missing_restore_id.validate().unwrap_err(),
            CheckpointValidationError::MissingRestoreId
        );

        let captured_file = CheckpointFileRecord::from_capture(
            "/tmp/workspace",
            "src/lib.rs",
            CheckpointFileState::PreExisting,
            Some("before".to_string()),
        );
        let manifest_missing_id = CheckpointManifest {
            checkpoint_id: String::new(),
            group_id: None,
            workspace_ref: "/tmp/workspace".to_string(),
            authority_scope: CheckpointAuthorityScope::Workspace,
            trigger_command: SessionCommand::Run,
            session_id: None,
            task_id: None,
            step_id: None,
            created_at: 1,
            captured_files: vec![captured_file.clone()],
            restore_history: Vec::new(),
        };
        assert_eq!(
            manifest_missing_id.validate().unwrap_err(),
            CheckpointValidationError::MissingCheckpointId
        );

        let manifest_missing_workspace = CheckpointManifest {
            checkpoint_id: "checkpoint-1".to_string(),
            group_id: None,
            workspace_ref: String::new(),
            authority_scope: CheckpointAuthorityScope::Workspace,
            trigger_command: SessionCommand::Run,
            session_id: None,
            task_id: None,
            step_id: None,
            created_at: 1,
            captured_files: vec![captured_file],
            restore_history: Vec::new(),
        };
        assert_eq!(
            manifest_missing_workspace.validate().unwrap_err(),
            CheckpointValidationError::MissingWorkspaceRef
        );
    }
}
