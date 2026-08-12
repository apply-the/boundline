//! Persistent external Git worktrees preserve resumable session mutations.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::identity::repository::{RepositoryIdentityError, RepositoryIdentityStore, WorktreeRole};

const WORKTREE_SCHEMA_VERSION: &str = "boundline-session-worktree-v1";
const WORKTREES_DIRECTORY: &str = "worktrees";
const LEASE_DIRECTORY: &str = "session-leases";

/// Durable states relevant to worktree retention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedSessionState {
    /// Session may execute admitted work.
    Active,
    /// Operator paused the session.
    Paused,
    /// A visible condition blocks progress.
    Blocked,
    /// Human authority is pending.
    ApprovalPending,
    /// Fresh proof is pending.
    ProofPending,
    /// One executor owns the session lease.
    ExecutorInFlight,
    /// Crash-produced changes await validation.
    UncommittedCandidate,
    /// Verified changes await publication.
    PublicationPending,
    /// Journaled state requires recovery.
    RecoveryRequired,
    /// Session finalized durably.
    Terminal,
    /// Operator explicitly aborted the session.
    Aborted,
}

impl ManagedSessionState {
    /// Returns every state whose worktree must remain available.
    pub fn retained_states() -> impl Iterator<Item = Self> {
        [
            Self::Active,
            Self::Paused,
            Self::Blocked,
            Self::ApprovalPending,
            Self::ProofPending,
            Self::ExecutorInFlight,
            Self::UncommittedCandidate,
            Self::PublicationPending,
            Self::RecoveryRequired,
        ]
        .into_iter()
    }
    const fn cleanup_allowed(self) -> bool {
        matches!(self, Self::Terminal | Self::Aborted)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SessionWorktreeRecord {
    schema_version: String,
    repository_id: String,
    session_id: String,
    state: ManagedSessionState,
}

/// Validated managed worktree projection.
#[derive(Clone, Debug)]
pub struct ManagedSessionWorktree {
    path: PathBuf,
    state: ManagedSessionState,
}

impl ManagedSessionWorktree {
    /// Returns the external managed checkout path.
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Returns the durable session lifecycle state.
    pub const fn state(&self) -> ManagedSessionState {
        self.state
    }
}

/// Creates and reopens persistent worktrees under a clone-scoped state root.
#[derive(Clone, Debug)]
pub struct SessionWorktreeManager {
    authoritative: PathBuf,
    state_root: PathBuf,
    repository_id: String,
}

impl SessionWorktreeManager {
    /// Opens a manager after validating authoritative role and ignored metadata.
    pub fn open(
        authoritative: impl AsRef<Path>,
        state_root: impl AsRef<Path>,
    ) -> Result<Self, SessionWorktreeError> {
        let authoritative = fs::canonicalize(authoritative)?;
        verify_metadata_ignored(&authoritative)?;
        let identity = RepositoryIdentityStore::open(&authoritative, WorktreeRole::Authoritative)?;
        fs::create_dir_all(state_root.as_ref())?;
        let state_root = fs::canonicalize(state_root)?;
        if state_root.starts_with(&authoritative) {
            return Err(SessionWorktreeError::NestedStateRoot);
        }
        Ok(Self {
            authoritative,
            state_root,
            repository_id: identity.repository_id().as_str().to_owned(),
        })
    }

    /// Creates one persistent detached session worktree at the admitted HEAD.
    pub fn create(&self, session_id: &str) -> Result<ManagedSessionWorktree, SessionWorktreeError> {
        validate_session_id(session_id)?;
        let path = self.session_path(session_id);
        if path.exists() {
            return Err(SessionWorktreeError::AlreadyExists);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        git(
            &self.authoritative,
            &[
                "worktree",
                "add",
                "--detach",
                path.to_str().ok_or(SessionWorktreeError::NonUtf8Path)?,
                "HEAD",
            ],
        )?;
        RepositoryIdentityStore::open(&path, WorktreeRole::SessionManaged)?;
        let record = SessionWorktreeRecord {
            schema_version: WORKTREE_SCHEMA_VERSION.to_owned(),
            repository_id: self.repository_id.clone(),
            session_id: session_id.to_owned(),
            state: ManagedSessionState::Active,
        };
        self.write_record(&record)?;
        Ok(ManagedSessionWorktree { path, state: record.state })
    }

    /// Reopens a durable worktree after process or machine restart.
    pub fn resume(&self, session_id: &str) -> Result<ManagedSessionWorktree, SessionWorktreeError> {
        let record = self.read_record(session_id)?;
        let path = self.session_path(session_id);
        if !path.is_dir() {
            return Err(SessionWorktreeError::MissingWorktree);
        }
        RepositoryIdentityStore::open(&path, WorktreeRole::SessionManaged)?;
        Ok(ManagedSessionWorktree { path, state: record.state })
    }

    /// Persists a lifecycle transition before any cleanup decision.
    pub fn set_state(
        &self,
        session_id: &str,
        state: ManagedSessionState,
    ) -> Result<(), SessionWorktreeError> {
        let mut record = self.read_record(session_id)?;
        record.state = state;
        self.write_record(&record)
    }

    /// Marks a session explicitly aborted while preserving its checkout.
    pub fn abort(&self, session_id: &str) -> Result<(), SessionWorktreeError> {
        self.set_state(session_id, ManagedSessionState::Aborted)
    }

    /// Deletes only durably terminal or explicitly aborted worktrees.
    pub fn cleanup(&self, session_id: &str) -> Result<(), SessionWorktreeError> {
        let record = self.read_record(session_id)?;
        if !record.state.cleanup_allowed() {
            return Err(SessionWorktreeError::RetentionRequired(record.state));
        }
        let path = self.session_path(session_id);
        git(
            &self.authoritative,
            &[
                "worktree",
                "remove",
                "--force",
                path.to_str().ok_or(SessionWorktreeError::NonUtf8Path)?,
            ],
        )?;
        fs::remove_file(self.record_path(session_id))?;
        Ok(())
    }

    /// Resolves the mandated external clone/session hierarchy.
    pub fn session_path(&self, session_id: &str) -> PathBuf {
        self.state_root.join(WORKTREES_DIRECTORY).join(&self.repository_id).join(session_id)
    }

    fn record_path(&self, session_id: &str) -> PathBuf {
        self.authoritative
            .join(".boundline")
            .join(LEASE_DIRECTORY)
            .join(format!("{session_id}.json"))
    }
    fn read_record(&self, session_id: &str) -> Result<SessionWorktreeRecord, SessionWorktreeError> {
        validate_session_id(session_id)?;
        let record: SessionWorktreeRecord =
            serde_json::from_slice(&fs::read(self.record_path(session_id))?)?;
        if record.schema_version != WORKTREE_SCHEMA_VERSION
            || record.repository_id != self.repository_id
            || record.session_id != session_id
        {
            return Err(SessionWorktreeError::RecordMismatch);
        }
        Ok(record)
    }
    fn write_record(&self, record: &SessionWorktreeRecord) -> Result<(), SessionWorktreeError> {
        let path = self.record_path(&record.session_id);
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("session lease parent is missing"))?;
        fs::create_dir_all(parent)?;
        let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
        let mut file = OpenOptions::new().create_new(true).write(true).open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(record)?)?;
        file.sync_all()?;
        fs::rename(temporary, &path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    }
}

/// Managed worktree lifecycle errors.
#[derive(Debug, Error)]
pub enum SessionWorktreeError {
    #[error("session worktree I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("session worktree record is invalid: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("repository identity validation failed: {0}")]
    Identity(#[from] RepositoryIdentityError),
    #[error("Git worktree operation failed: {0}")]
    Git(String),
    #[error("state root must be external to the authoritative worktree")]
    NestedStateRoot,
    #[error(".boundline metadata is not ignored by Git")]
    MetadataTracked,
    #[error("session worktree already exists")]
    AlreadyExists,
    #[error("session worktree is missing")]
    MissingWorktree,
    #[error("session identifier is invalid")]
    InvalidSessionId,
    #[error("session worktree path is not UTF-8")]
    NonUtf8Path,
    #[error("session record does not match manager identity")]
    RecordMismatch,
    #[error("session state {0:?} requires retention")]
    RetentionRequired(ManagedSessionState),
}

fn validate_session_id(value: &str) -> Result<(), SessionWorktreeError> {
    if !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Ok(())
    } else {
        Err(SessionWorktreeError::InvalidSessionId)
    }
}
fn verify_metadata_ignored(root: &Path) -> Result<(), SessionWorktreeError> {
    let output = Command::new("git")
        .args(["check-ignore", "-q", ".boundline/probe"])
        .current_dir(root)
        .output()?;
    if output.status.success() { Ok(()) } else { Err(SessionWorktreeError::MetadataTracked) }
}
fn git(root: &Path, arguments: &[&str]) -> Result<(), SessionWorktreeError> {
    let output = Command::new("git").args(arguments).current_dir(root).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(SessionWorktreeError::Git(String::from_utf8_lossy(&output.stderr).trim().to_owned()))
    }
}
