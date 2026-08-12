//! Durable governed projections admit human approval only from fresh exact evidence.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use boundline_protocol::{OperatingStage, PublicSessionProjection, SessionLifecycle};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::identity::repository::{RepositoryIdentityError, RepositoryIdentityStore, WorktreeRole};
use crate::transaction::challenge::ChallengeDecision;
use crate::transaction::evidence::{EvidenceKind, EvidenceLedger};

const GOVERNED_SESSION_SCHEMA_VERSION: &str = "boundline-governed-session-v1";
const SESSION_DIRECTORY: &str = "governed-sessions";

/// Durable authority state behind CLI projections.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedSessionRecord {
    schema_version: String,
    /// Transport-neutral authoritative session projection.
    pub projection: PublicSessionProjection,
    /// Exact evidence ledger for the current transaction state.
    pub evidence: EvidenceLedger,
    /// Deterministically evaluated challenge decision.
    pub challenge: ChallengeDecision,
    /// Human approval state bound to this record revision.
    pub approved: bool,
}

impl GovernedSessionRecord {
    /// Creates a record whose authority remains unapproved.
    pub fn new(
        projection: PublicSessionProjection,
        evidence: EvidenceLedger,
        challenge: ChallengeDecision,
    ) -> Self {
        Self {
            schema_version: GOVERNED_SESSION_SCHEMA_VERSION.to_owned(),
            projection,
            evidence,
            challenge,
            approved: false,
        }
    }
}

/// Clone-scoped governed session persistence.
#[derive(Clone, Debug)]
pub struct GovernedSessionStore {
    root: PathBuf,
}

impl GovernedSessionStore {
    /// Opens the external state hierarchy for one authoritative local clone.
    pub fn open(
        authoritative: impl AsRef<Path>,
        state_root: impl AsRef<Path>,
    ) -> Result<Self, GovernedSessionError> {
        let identity = RepositoryIdentityStore::open(authoritative, WorktreeRole::Authoritative)?;
        let root =
            state_root.as_ref().join(SESSION_DIRECTORY).join(identity.repository_id().as_str());
        fs::create_dir_all(&root)?;
        Ok(Self { root: fs::canonicalize(root)? })
    }

    /// Persists an authoritative projection produced by the runtime.
    pub fn save(&self, record: &GovernedSessionRecord) -> Result<(), GovernedSessionError> {
        write_record(&self.path(&session_key(&record.projection)?), record)
    }

    /// Loads one governed session projection.
    pub fn load(&self, session_id: &str) -> Result<GovernedSessionRecord, GovernedSessionError> {
        validate_key(session_id)?;
        let record: GovernedSessionRecord =
            serde_json::from_slice(&fs::read(self.path(session_id))?)?;
        if record.schema_version != GOVERNED_SESSION_SCHEMA_VERSION
            || session_key(&record.projection)? != session_id
        {
            return Err(GovernedSessionError::RecordMismatch);
        }
        Ok(record)
    }

    /// Records human approval only for fresh proof, verification, and challenge evidence.
    pub fn approve(&self, session_id: &str) -> Result<GovernedSessionRecord, GovernedSessionError> {
        let mut record = self.load(session_id)?;
        if record.projection.lifecycle != SessionLifecycle::ApprovalPending {
            return Err(GovernedSessionError::ApprovalNotPending);
        }
        if !record.evidence.all_fresh()
            || !record.evidence.has_fresh(EvidenceKind::Proof)
            || !record.evidence.has_fresh(EvidenceKind::Verification)
        {
            return Err(GovernedSessionError::FreshEvidenceRequired);
        }
        if !matches!(
            record.challenge,
            ChallengeDecision::Satisfied | ChallengeDecision::SatisfiedWithOverride { .. }
        ) {
            return Err(GovernedSessionError::ChallengeRequired);
        }
        record.approved = true;
        record.projection.lifecycle = SessionLifecycle::PublicationPending;
        record.projection.stage = OperatingStage::Publish;
        self.save(&record)?;
        Ok(record)
    }

    fn path(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }
}

/// Governed session persistence and approval failures.
#[derive(Debug, Error)]
pub enum GovernedSessionError {
    #[error("governed session I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("governed session record is invalid: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("repository identity validation failed: {0}")]
    Identity(#[from] RepositoryIdentityError),
    #[error("governed session identifier is invalid")]
    InvalidSessionId,
    #[error("governed session record identity does not match its storage key")]
    RecordMismatch,
    #[error("session is not awaiting approval")]
    ApprovalNotPending,
    #[error("fresh claim-matched proof and verification are required")]
    FreshEvidenceRequired,
    #[error("required independent challenge is missing")]
    ChallengeRequired,
}

fn session_key(projection: &PublicSessionProjection) -> Result<String, GovernedSessionError> {
    let value = serde_json::to_value(&projection.session_id)?;
    value.as_str().map(str::to_owned).ok_or(GovernedSessionError::InvalidSessionId)
}
fn validate_key(value: &str) -> Result<(), GovernedSessionError> {
    if !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        Ok(())
    } else {
        Err(GovernedSessionError::InvalidSessionId)
    }
}
fn write_record(path: &Path, record: &GovernedSessionRecord) -> Result<(), GovernedSessionError> {
    let temporary = path.with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(record)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
