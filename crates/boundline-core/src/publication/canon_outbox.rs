//! Durable, explicitly driven terminal-outcome delivery to Canon.
//!
//! Every call performs at most one transport attempt. Scheduling is persisted,
//! but callers retain control of when another attempt is made.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use boundline_protocol::CanonOutcomeSyncStatus;
use canon_contracts::{
    DecisionMemoryDigest, RecordOutcomeDisposition, RecordOutcomeRejectionReason,
    RecordOutcomeRequest, RecordOutcomeResponse, Revision,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const OUTBOX_SCHEMA_VERSION: &str = "boundline-canon-outbox-v1";
const OUTBOX_DIRECTORY: &str = "canon-outbox";
const RECORDS_DIRECTORY: &str = "records";
const OWNERSHIP_LOCK_FILE: &str = "delivery.lock";
const RECORD_EXTENSION: &str = "json";
const INITIAL_RETRY_DELAY_MS: u64 = 1_000;
const MAX_RETRY_DELAY_MS: u64 = 60_000;
const MAX_DELIVERY_ATTEMPTS: u32 = 5;

/// Persisted lifecycle of one terminal-outcome delivery.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutboxState {
    /// Durable request has not yet been attempted.
    Pending,
    /// One fenced owner durably announced an attempt.
    InFlight,
    /// A transient failure has a durable next eligibility time.
    RetryScheduled,
    /// Canon durably recorded or replayed the event and Boundline acknowledged it.
    Synchronized,
    /// Canon returned a deterministic non-retry rejection or retries were exhausted.
    PermanentRejected,
    /// Canon or local persistence found the identity bound to changed content.
    Conflict,
    /// Named authority archived a completed or permanently rejected record.
    Archived,
}

/// One immutable delivery-attempt audit entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    /// One-based attempt number.
    pub attempt_number: u32,
    /// Monotonic ownership token for the attempt.
    pub fencing_token: u64,
    /// Injected authoritative attempt time.
    pub attempted_at_ms: u64,
    /// Stable transport or Canon reason, when present.
    pub reason_code: Option<String>,
    /// Validated Canon response, when one was received.
    pub response: Option<RecordOutcomeResponse>,
}

/// Named authority that retained a terminal record outside active delivery.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveAuthorization {
    /// Stable actor identity.
    pub actor: String,
    /// Bounded reason for archival.
    pub reason: String,
    /// Authoritative archival time.
    pub occurred_at_ms: u64,
}

impl ArchiveAuthorization {
    /// Creates a named archival authorization.
    pub fn new(actor: impl Into<String>, reason: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self { actor: actor.into(), reason: reason.into(), occurred_at_ms }
    }
}

/// Complete portable outbox state for one event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonOutcomeOutboxRecord {
    /// Persistence schema identity.
    pub schema_version: String,
    /// Complete frozen public request for deterministic reconstruction.
    pub request: RecordOutcomeRequest,
    /// Current delivery lifecycle.
    pub state: OutboxState,
    /// Number of transport attempts durably admitted.
    pub attempt_count: u32,
    /// Monotonic delivery fencing token.
    pub fencing_token: u64,
    /// Ordered immutable attempt history.
    pub attempt_history: Vec<DeliveryAttempt>,
    /// Earliest injected time at which an explicit retry is eligible.
    pub next_retry_at_ms: Option<u64>,
    /// Last stable diagnostic classification.
    pub last_reason_code: Option<String>,
    /// Last validated Canon response.
    pub last_response: Option<RecordOutcomeResponse>,
    /// Canon revision after durable synchronization.
    pub canon_revision: Option<Revision>,
    /// Canon decision-memory digest after durable synchronization.
    pub canon_digest: Option<DecisionMemoryDigest>,
    /// Named non-destructive archival authority.
    pub archive: Option<ArchiveAuthorization>,
}

/// Result of durable enqueue admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnqueueOutcome {
    /// A new durable record was created.
    Enqueued,
    /// Exact event identity and digest were already durable.
    Replayed,
}

/// Result of one bounded delivery call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryAttemptResult {
    /// Canon recorded the event and Boundline durably acknowledged it.
    Synchronized,
    /// A transient failure produced a durable retry schedule.
    RetryScheduled,
    /// The caller arrived before the durable retry eligibility time.
    NotEligible,
    /// Another process owns the delivery lock.
    AlreadyOwned,
    /// The record was already durably synchronized.
    AlreadySynchronized,
    /// Canon returned a permanent typed rejection.
    PermanentRejected(RecordOutcomeRejectionReason),
    /// The finite retry budget was exhausted.
    RetryExhausted,
    /// Identity and canonical content conflict.
    Conflict,
    /// The validated response was received but local acknowledgement was interrupted.
    AcknowledgementPending,
    /// Named authority archived the record.
    Archived,
}

/// External time source used to keep retry decisions deterministic in tests.
pub trait Clock {
    /// Returns an authoritative millisecond timestamp.
    fn now_millis(&self) -> u64;
}

mod state;
mod transport;

use state::*;
pub use transport::{
    CanonOutcomeTransport, CanonSubprocessTransport, TransportFailure, TransportFailureKind,
};

/// Qualification-only acknowledgement fault control.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeliveryControl {
    fail_before_ack: bool,
}

impl DeliveryControl {
    /// Simulates response loss after Canon commit and before local acknowledgement.
    pub const fn fail_before_ack() -> Self {
        Self { fail_before_ack: true }
    }
}

/// Fail-closed outbox errors.
#[derive(Debug, Error)]
pub enum OutboxError {
    /// Local persistence failed.
    #[error("Canon outcome outbox persistence failed: {0}")]
    Io(#[from] std::io::Error),
    /// Typed persistence encoding failed.
    #[error("Canon outcome outbox serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    /// The public terminal contract rejected the request.
    #[error("Canon outcome rejected before enqueue: {0:?}")]
    Rejected(RecordOutcomeRejectionReason),
    /// Existing identity is bound to different canonical content.
    #[error("Canon outcome identity has different canonical content")]
    IdentityDigestConflict,
    /// No record exists for the supplied logical identity.
    #[error("Canon outcome outbox record was not found")]
    NotFound,
    /// Archive policy does not admit the current lifecycle or authority.
    #[error("Canon outcome outbox record is not eligible for archival")]
    ArchiveNotPermitted,
    /// A Canon response did not match the exact request identity and digest.
    #[error("Canon outcome response did not match the request")]
    ResponseMismatch,
}

/// Durable repository for terminal-outcome delivery records.
#[derive(Clone, Debug)]
pub struct CanonOutcomeOutbox {
    root: PathBuf,
}

impl CanonOutcomeOutbox {
    /// Opens or creates one protected state-root outbox directory.
    pub fn open(state_root: impl AsRef<Path>) -> Result<Self, OutboxError> {
        let root = state_root.as_ref().join(OUTBOX_DIRECTORY);
        fs::create_dir_all(root.join(RECORDS_DIRECTORY))?;
        sync_directory(&root)?;
        Ok(Self { root })
    }

    /// Persists a valid terminal request before any delivery side effect.
    pub fn enqueue_terminal_outcome(
        &self,
        request: RecordOutcomeRequest,
    ) -> Result<EnqueueOutcome, OutboxError> {
        request.validate().map_err(|error| OutboxError::Rejected(error.reason_code()))?;
        let path = self.record_path(request.event_id.as_str());
        if path.is_file() {
            let recorded = read_record(&path)?;
            return if recorded.request.event_digest == request.event_digest {
                Ok(EnqueueOutcome::Replayed)
            } else {
                Err(OutboxError::IdentityDigestConflict)
            };
        }
        let record = CanonOutcomeOutboxRecord {
            schema_version: OUTBOX_SCHEMA_VERSION.to_string(),
            request,
            state: OutboxState::Pending,
            attempt_count: 0,
            fencing_token: 0,
            attempt_history: Vec::new(),
            next_retry_at_ms: None,
            last_reason_code: None,
            last_response: None,
            canon_revision: None,
            canon_digest: None,
            archive: None,
        };
        write_record(&path, &record)?;
        Ok(EnqueueOutcome::Enqueued)
    }

    /// Loads one complete portable record by its logical event identity.
    pub fn load(&self, event_id: &str) -> Result<CanonOutcomeOutboxRecord, OutboxError> {
        let record = read_record(&self.record_path(event_id))?;
        if record.request.event_id.as_str() == event_id {
            Ok(record)
        } else {
            Err(OutboxError::IdentityDigestConflict)
        }
    }

    /// Executes at most one delivery attempt with production acknowledgement behavior.
    pub fn deliver_once(
        &self,
        event_id: &str,
        transport: &mut impl CanonOutcomeTransport,
        clock: &impl Clock,
    ) -> Result<DeliveryAttemptResult, OutboxError> {
        self.deliver_once_with_control(event_id, transport, clock, DeliveryControl::default())
    }

    /// Executes one attempt with a declared qualification fault boundary.
    pub fn deliver_once_with_control(
        &self,
        event_id: &str,
        transport: &mut impl CanonOutcomeTransport,
        clock: &impl Clock,
        control: DeliveryControl,
    ) -> Result<DeliveryAttemptResult, OutboxError> {
        let Some(_ownership) = self.try_acquire_ownership()? else {
            return Ok(DeliveryAttemptResult::AlreadyOwned);
        };
        let path = self.record_path(event_id);
        let mut record = read_record(&path)?;
        if let Some(result) = terminal_delivery_result(&record) {
            return Ok(result);
        }
        let now = clock.now_millis();
        if record.next_retry_at_ms.is_some_and(|eligible| now < eligible) {
            return Ok(DeliveryAttemptResult::NotEligible);
        }
        admit_attempt(&path, &mut record)?;
        let response = match transport.deliver(&record.request) {
            Ok(response) => response,
            Err(failure) => return record_transport_failure(&path, record, failure, now),
        };
        if validate_response(&record, &response).is_err() {
            return record_transport_failure(
                &path,
                record,
                TransportFailure::new(
                    TransportFailureKind::ResponseRead,
                    "response failed identity or contract validation",
                ),
                now,
            );
        }
        if control.fail_before_ack {
            return Ok(DeliveryAttemptResult::AcknowledgementPending);
        }
        record_response(&path, record, response, now)
    }

    /// Archives only synchronized or permanently rejected records with named authority.
    pub fn archive_outbox_record(
        &self,
        event_id: &str,
        authorization: ArchiveAuthorization,
    ) -> Result<(), OutboxError> {
        if authorization.actor.trim().is_empty()
            || authorization.reason.trim().is_empty()
            || authorization.occurred_at_ms == 0
        {
            return Err(OutboxError::ArchiveNotPermitted);
        }
        let Some(_ownership) = self.try_acquire_ownership()? else {
            return Err(OutboxError::ArchiveNotPermitted);
        };
        let path = self.record_path(event_id);
        let mut record = read_record(&path)?;
        if !matches!(record.state, OutboxState::Synchronized | OutboxState::PermanentRejected) {
            return Err(OutboxError::ArchiveNotPermitted);
        }
        record.state = OutboxState::Archived;
        record.archive = Some(authorization);
        write_record(&path, &record)
    }

    /// Maps detailed internal lifecycle to the frozen public coarse projection.
    pub fn sync_status(
        &self,
        event_id: Option<&str>,
    ) -> Result<CanonOutcomeSyncStatus, OutboxError> {
        let Some(event_id) = event_id else {
            return Ok(CanonOutcomeSyncStatus::NotRequired);
        };
        Ok(match self.load(event_id)?.state {
            OutboxState::Pending | OutboxState::InFlight | OutboxState::RetryScheduled => {
                CanonOutcomeSyncStatus::Pending
            }
            OutboxState::Synchronized => CanonOutcomeSyncStatus::Synchronized,
            OutboxState::PermanentRejected | OutboxState::Conflict => {
                CanonOutcomeSyncStatus::Failed
            }
            OutboxState::Archived => CanonOutcomeSyncStatus::NotRequired,
        })
    }

    fn record_path(&self, event_id: &str) -> PathBuf {
        let digest = Sha256::digest(event_id.as_bytes());
        self.root.join(RECORDS_DIRECTORY).join(format!("{digest:x}.{RECORD_EXTENSION}"))
    }

    fn try_acquire_ownership(&self) -> Result<Option<Ownership>, OutboxError> {
        let path = self.root.join(OWNERSHIP_LOCK_FILE);
        let file =
            OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path)?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Ownership { file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(error)) => Err(OutboxError::Io(error)),
        }
    }
}
