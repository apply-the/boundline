//! Persistent per-session execution leases provide monotonic fencing.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

const LEASE_SCHEMA_VERSION: &str = "boundline-execution-lease-v1";
const INITIAL_REVISION: u64 = 0;

/// Exclusive executor ownership returned after durable admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLease {
    schema_version: String,
    session_id: String,
    owner_id: String,
    expected_revision: u64,
    fencing_token: u64,
}

impl ExecutionLease {
    /// Returns the monotonic token required by every subsequent write.
    pub const fn fencing_token(&self) -> u64 {
        self.fencing_token
    }
    /// Returns the transaction revision admitted with this executor.
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
}

/// Filesystem-backed execution lease service.
#[derive(Clone, Debug)]
pub struct ExecutionLeaseStore {
    root: PathBuf,
}

impl ExecutionLeaseStore {
    /// Opens an isolated durable lease root.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ExecutionLeaseError> {
        let root = root.as_ref().join("execution-leases");
        fs::create_dir_all(&root)?;
        Ok(Self { root: fs::canonicalize(root)? })
    }

    /// Sets the authoritative transaction revision when no executor is active.
    pub fn set_revision(&self, session_id: &str, revision: u64) -> Result<(), ExecutionLeaseError> {
        self.validate_session_id(session_id)?;
        if self.active_path(session_id).exists() {
            return Err(ExecutionLeaseError::AlreadyHeld);
        }
        write_atomic(&self.revision_path(session_id), &revision.to_string())
    }

    /// Acquires the sole session executor slot and allocates a new fencing token.
    pub fn acquire(
        &self,
        session_id: &str,
        owner_id: &str,
        expected_revision: u64,
    ) -> Result<ExecutionLease, ExecutionLeaseError> {
        self.validate_session_id(session_id)?;
        let current_revision =
            read_number(&self.revision_path(session_id))?.unwrap_or(INITIAL_REVISION);
        if current_revision != expected_revision {
            return Err(ExecutionLeaseError::StaleRevision {
                expected: expected_revision,
                actual: current_revision,
            });
        }
        let active_path = self.active_path(session_id);
        let mut reservation =
            OpenOptions::new().write(true).create_new(true).open(&active_path).map_err(
                |error| {
                    if error.kind() == std::io::ErrorKind::AlreadyExists {
                        ExecutionLeaseError::AlreadyHeld
                    } else {
                        error.into()
                    }
                },
            )?;
        let result = self.finish_acquire(session_id, owner_id, expected_revision, &mut reservation);
        if result.is_err() {
            let _ignored = fs::remove_file(active_path);
        }
        result
    }

    /// Validates that a write presents the current owner and fencing token.
    pub fn validate_write(&self, lease: &ExecutionLease) -> Result<(), ExecutionLeaseError> {
        let current: ExecutionLease =
            serde_json::from_slice(&fs::read(self.active_path(&lease.session_id))?)?;
        if current == *lease { Ok(()) } else { Err(ExecutionLeaseError::StaleFencingToken) }
    }

    /// Releases the lease only when the current fencing owner requests it.
    pub fn release(&self, lease: &ExecutionLease) -> Result<(), ExecutionLeaseError> {
        self.validate_write(lease)?;
        fs::remove_file(self.active_path(&lease.session_id))?;
        Ok(())
    }

    fn finish_acquire(
        &self,
        session_id: &str,
        owner_id: &str,
        expected_revision: u64,
        reservation: &mut std::fs::File,
    ) -> Result<ExecutionLease, ExecutionLeaseError> {
        let token_path = self.token_path(session_id);
        let fencing_token = read_number(&token_path)?
            .unwrap_or(0)
            .checked_add(1)
            .ok_or(ExecutionLeaseError::TokenExhausted)?;
        write_atomic(&token_path, &fencing_token.to_string())?;
        let lease = ExecutionLease {
            schema_version: LEASE_SCHEMA_VERSION.to_owned(),
            session_id: session_id.to_owned(),
            owner_id: owner_id.to_owned(),
            expected_revision,
            fencing_token,
        };
        reservation.write_all(&serde_json::to_vec(&lease)?)?;
        reservation.sync_all()?;
        Ok(lease)
    }

    fn validate_session_id(&self, value: &str) -> Result<(), ExecutionLeaseError> {
        if !value.is_empty()
            && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            Ok(())
        } else {
            Err(ExecutionLeaseError::InvalidSessionId)
        }
    }
    fn active_path(&self, session: &str) -> PathBuf {
        self.root.join(format!("{session}.active.json"))
    }
    fn token_path(&self, session: &str) -> PathBuf {
        self.root.join(format!("{session}.token"))
    }
    fn revision_path(&self, session: &str) -> PathBuf {
        self.root.join(format!("{session}.revision"))
    }
}

/// Fail-closed execution admission errors.
#[derive(Debug, Error)]
pub enum ExecutionLeaseError {
    /// Lease persistence failed.
    #[error("execution lease I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Lease record decoding failed.
    #[error("execution lease record is invalid: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Another executor owns the session.
    #[error("one executor is already in flight")]
    AlreadyHeld,
    /// Optimistic revision did not match before admission.
    #[error("stale transaction revision: expected {expected}, actual {actual}")]
    StaleRevision { expected: u64, actual: u64 },
    /// A prior owner attempted a fenced write.
    #[error("executor fencing token is stale")]
    StaleFencingToken,
    /// Session identifiers cannot affect storage paths.
    #[error("session identifier is invalid")]
    InvalidSessionId,
    /// No additional monotonic token can be allocated.
    #[error("executor fencing token exhausted")]
    TokenExhausted,
}

fn read_number(path: &Path) -> Result<Option<u64>, ExecutionLeaseError> {
    match fs::read_to_string(path) {
        Ok(value) => value
            .parse()
            .map(Some)
            .map_err(|error| ExecutionLeaseError::Io(std::io::Error::other(error))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn write_atomic(path: &Path, value: &str) -> Result<(), ExecutionLeaseError> {
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}
