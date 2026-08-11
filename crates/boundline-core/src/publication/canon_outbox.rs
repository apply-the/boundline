//! Durable, explicitly driven terminal-outcome delivery to Canon.
//!
//! Every call performs at most one transport attempt. Scheduling is persisted,
//! but callers retain control of when another attempt is made.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use boundline_protocol::CanonOutcomeSyncStatus;
use canon_contracts::{
    CanonContractVersion, DecisionMemoryDigest, OneShotOperation, OneShotResponse,
    RecordOutcomeDisposition, RecordOutcomeRejectionReason, RecordOutcomeRequest,
    RecordOutcomeResponse, Revision,
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
const MAX_CANON_REQUEST_BYTES: usize = 1_048_576;
const DEFAULT_MAX_RESPONSE_BYTES: usize = 1_048_576;
const PROCESS_POLL_INTERVAL_MS: u64 = 10;

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

/// One-shot transport boundary; implementations must perform no internal retry.
pub trait CanonOutcomeTransport {
    /// Delivers one exact public request and returns one typed response.
    fn deliver(
        &mut self,
        request: &RecordOutcomeRequest,
    ) -> Result<RecordOutcomeResponse, TransportFailure>;
}

/// Bounded local one-shot Canon subprocess transport.
#[derive(Clone, Debug)]
pub struct CanonSubprocessTransport {
    executable: PathBuf,
    canon_root: PathBuf,
    repository_root: PathBuf,
    timeout: Duration,
    max_response_bytes: usize,
}

impl CanonSubprocessTransport {
    /// Creates a transport with explicit runtime roots and deadline.
    pub fn new(
        executable: impl Into<PathBuf>,
        canon_root: impl Into<PathBuf>,
        repository_root: impl Into<PathBuf>,
        timeout: Duration,
    ) -> Self {
        Self {
            executable: executable.into(),
            canon_root: canon_root.into(),
            repository_root: repository_root.into(),
            timeout,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        }
    }

    /// Overrides the strict stdout and stderr framing limit.
    pub const fn with_max_response_bytes(mut self, max_response_bytes: usize) -> Self {
        self.max_response_bytes = max_response_bytes;
        self
    }

    fn invoke(
        &self,
        event_id: &str,
        encoded_request: &[u8],
    ) -> Result<RecordOutcomeResponse, TransportFailure> {
        let mut child = Command::new(&self.executable)
            .args(["--canon-root"])
            .arg(&self.canon_root)
            .args(["--repo-root"])
            .arg(&self.repository_root)
            .args(["rpc", "--stdio"])
            .current_dir(&self.repository_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                TransportFailure::new(TransportFailureKind::BeforeProcessStart, error.to_string())
            })?;
        if let Err(error) = write_child_request(&mut child, encoded_request) {
            terminate_child(&mut child);
            return Err(error);
        }
        let stdout = child.stdout.take().ok_or_else(|| {
            TransportFailure::new(TransportFailureKind::ResponseRead, "stdout unavailable")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            TransportFailure::new(TransportFailureKind::ResponseRead, "stderr unavailable")
        })?;
        let output_limit = self.max_response_bytes;
        let stdout_reader = thread::spawn(move || read_bounded(stdout, output_limit));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, output_limit));
        let status = wait_bounded(&mut child, self.timeout);
        let output = join_reader(stdout_reader)?;
        let _diagnostics = join_reader(stderr_reader)?;
        let status = status?;
        if output.len() > self.max_response_bytes {
            return Err(TransportFailure::new(
                TransportFailureKind::ResponseRead,
                "response exceeded the configured framing limit",
            ));
        }
        let response = serde_json::from_slice::<OneShotResponse<RecordOutcomeResponse>>(&output)
            .map_err(|error| {
                let kind = if status.success() {
                    TransportFailureKind::ResponseRead
                } else {
                    TransportFailureKind::ProcessExit
                };
                TransportFailure::new(kind, error.to_string())
            })?;
        if response.contract_version != CanonContractVersion::V1 || response.request_id != event_id
        {
            return Err(TransportFailure::new(
                TransportFailureKind::ResponseRead,
                "response envelope identity mismatch",
            ));
        }
        Ok(response.result)
    }
}

#[derive(Serialize)]
struct OutcomeOperationPayload<'a> {
    outcome: &'a RecordOutcomeRequest,
}

impl CanonOutcomeTransport for CanonSubprocessTransport {
    fn deliver(
        &mut self,
        request: &RecordOutcomeRequest,
    ) -> Result<RecordOutcomeResponse, TransportFailure> {
        let envelope = canon_contracts::OneShotRequest {
            contract_version: CanonContractVersion::V1,
            request_id: request.event_id.as_str().to_string(),
            operation: OneShotOperation::RecordOutcome,
            payload: OutcomeOperationPayload { outcome: request },
        };
        let bytes = serde_json::to_vec(&envelope).map_err(|error| {
            TransportFailure::new(TransportFailureKind::RequestWrite, error.to_string())
        })?;
        if bytes.len() > MAX_CANON_REQUEST_BYTES {
            return Err(TransportFailure::new(
                TransportFailureKind::RequestWrite,
                "request exceeded the Canon one-shot frame limit",
            ));
        }
        self.invoke(request.event_id.as_str(), &bytes)
    }
}

fn write_child_request(
    child: &mut std::process::Child,
    request: &[u8],
) -> Result<(), TransportFailure> {
    let mut input = child.stdin.take().ok_or_else(|| {
        TransportFailure::new(TransportFailureKind::RequestWrite, "stdin unavailable")
    })?;
    input.write_all(request).map_err(|error| {
        TransportFailure::new(TransportFailureKind::RequestWrite, error.to_string())
    })?;
    input.flush().map_err(|error| {
        TransportFailure::new(TransportFailureKind::RequestWrite, error.to_string())
    })
}

fn wait_bounded(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::ExitStatus, TransportFailure> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|error| {
            TransportFailure::new(TransportFailureKind::ProcessExit, error.to_string())
        })? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            terminate_child(child);
            return Err(TransportFailure::new(
                TransportFailureKind::Timeout,
                "Canon one-shot deadline elapsed",
            ));
        }
        thread::sleep(Duration::from_millis(PROCESS_POLL_INTERVAL_MS));
    }
}

fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_bounded(reader: impl Read, limit: usize) -> std::io::Result<Vec<u8>> {
    let limit = u64::try_from(limit)
        .map_err(|_| std::io::Error::other("response limit exceeds the platform range"))?;
    let mut bytes = Vec::new();
    reader.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(
    reader: thread::JoinHandle<std::io::Result<Vec<u8>>>,
) -> Result<Vec<u8>, TransportFailure> {
    reader
        .join()
        .map_err(|_| {
            TransportFailure::new(TransportFailureKind::ResponseRead, "reader thread failed")
        })?
        .map_err(|error| {
            TransportFailure::new(TransportFailureKind::ResponseRead, error.to_string())
        })
}

/// Stable transport failure stage used by deterministic retry classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportFailureKind {
    /// Canon could not be started.
    BeforeProcessStart,
    /// The request could not be written completely.
    RequestWrite,
    /// The request completed but no response boundary was observed.
    AfterRequestWrite,
    /// The response could not be read or decoded.
    ResponseRead,
    /// Canon exited before a complete response.
    ProcessExit,
    /// The bounded invocation exceeded its deadline.
    Timeout,
}

impl TransportFailureKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeProcessStart => "before_process_start",
            Self::RequestWrite => "request_write",
            Self::AfterRequestWrite => "after_request_write",
            Self::ResponseRead => "response_read",
            Self::ProcessExit => "process_exit",
            Self::Timeout => "timeout",
        }
    }
}

/// Bounded diagnostic from one transport attempt.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error("Canon transport failed at {kind:?}: {message}")]
pub struct TransportFailure {
    /// Stage that failed.
    pub kind: TransportFailureKind,
    /// Redacted bounded detail.
    pub message: String,
}

impl TransportFailure {
    /// Creates a redacted transport failure.
    pub fn new(kind: TransportFailureKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into() }
    }
}

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
        validate_response(&record, &response)?;
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

struct Ownership {
    file: File,
}

impl Drop for Ownership {
    fn drop(&mut self) {
        let _ = File::unlock(&self.file);
    }
}

fn terminal_delivery_result(record: &CanonOutcomeOutboxRecord) -> Option<DeliveryAttemptResult> {
    match record.state {
        OutboxState::Synchronized => Some(DeliveryAttemptResult::AlreadySynchronized),
        OutboxState::PermanentRejected => record
            .last_response
            .as_ref()
            .and_then(|response| response.reason_code)
            .map(DeliveryAttemptResult::PermanentRejected)
            .or(Some(DeliveryAttemptResult::RetryExhausted)),
        OutboxState::Conflict => Some(DeliveryAttemptResult::Conflict),
        OutboxState::Archived => Some(DeliveryAttemptResult::Archived),
        OutboxState::Pending | OutboxState::InFlight | OutboxState::RetryScheduled => None,
    }
}

fn admit_attempt(path: &Path, record: &mut CanonOutcomeOutboxRecord) -> Result<(), OutboxError> {
    record.attempt_count = record.attempt_count.saturating_add(1);
    record.fencing_token = record.fencing_token.saturating_add(1);
    record.state = OutboxState::InFlight;
    record.next_retry_at_ms = None;
    write_record(path, record)
}

fn record_transport_failure(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    failure: TransportFailure,
    now: u64,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = failure.kind.as_str().to_string();
    record.attempt_history.push(DeliveryAttempt {
        attempt_number: record.attempt_count,
        fencing_token: record.fencing_token,
        attempted_at_ms: now,
        reason_code: Some(reason.clone()),
        response: None,
    });
    record.last_reason_code = Some(reason);
    if record.attempt_count >= MAX_DELIVERY_ATTEMPTS {
        record.state = OutboxState::PermanentRejected;
        record.next_retry_at_ms = None;
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::RetryExhausted);
    }
    record.state = OutboxState::RetryScheduled;
    record.next_retry_at_ms = Some(now.saturating_add(retry_delay(record.attempt_count)));
    write_record(path, &record)?;
    Ok(DeliveryAttemptResult::RetryScheduled)
}

fn record_response(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    response: RecordOutcomeResponse,
    now: u64,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = response.reason_code.map(RecordOutcomeRejectionReason::as_str).map(str::to_string);
    record.attempt_history.push(DeliveryAttempt {
        attempt_number: record.attempt_count,
        fencing_token: record.fencing_token,
        attempted_at_ms: now,
        reason_code: reason.clone(),
        response: Some(response.clone()),
    });
    record.last_reason_code = reason;
    record.last_response = Some(response.clone());
    match response.disposition {
        RecordOutcomeDisposition::Recorded | RecordOutcomeDisposition::Replayed => {
            record.state = OutboxState::Synchronized;
            record.canon_revision = response.decision_memory_revision;
            record.canon_digest = response.decision_memory_digest;
            write_record(path, &record)?;
            Ok(DeliveryAttemptResult::Synchronized)
        }
        RecordOutcomeDisposition::Rejected => classify_rejection(path, record, response),
    }
}

fn classify_rejection(
    path: &Path,
    mut record: CanonOutcomeOutboxRecord,
    response: RecordOutcomeResponse,
) -> Result<DeliveryAttemptResult, OutboxError> {
    let reason = response.reason_code.ok_or(OutboxError::ResponseMismatch)?;
    if reason == RecordOutcomeRejectionReason::PersistenceFailed {
        if record.attempt_count >= MAX_DELIVERY_ATTEMPTS {
            record.state = OutboxState::PermanentRejected;
            record.next_retry_at_ms = None;
            write_record(path, &record)?;
            return Ok(DeliveryAttemptResult::RetryExhausted);
        }
        record.state = OutboxState::RetryScheduled;
        let attempted_at =
            record.attempt_history.last().map_or(0, |attempt| attempt.attempted_at_ms);
        record.next_retry_at_ms =
            Some(attempted_at.saturating_add(retry_delay(record.attempt_count)));
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::RetryScheduled);
    }
    if reason == RecordOutcomeRejectionReason::IdentityDigestConflict {
        record.state = OutboxState::Conflict;
        record.next_retry_at_ms = None;
        write_record(path, &record)?;
        return Ok(DeliveryAttemptResult::Conflict);
    }
    record.state = OutboxState::PermanentRejected;
    record.next_retry_at_ms = None;
    write_record(path, &record)?;
    Ok(DeliveryAttemptResult::PermanentRejected(reason))
}

fn validate_response(
    record: &CanonOutcomeOutboxRecord,
    response: &RecordOutcomeResponse,
) -> Result<(), OutboxError> {
    response.validate().map_err(|_| OutboxError::ResponseMismatch)?;
    if response.event_id != record.request.event_id
        || response.event_digest != record.request.event_digest
    {
        Err(OutboxError::ResponseMismatch)
    } else {
        Ok(())
    }
}

fn retry_delay(attempt: u32) -> u64 {
    let exponent = attempt.saturating_sub(1).min(16);
    INITIAL_RETRY_DELAY_MS.saturating_mul(1_u64 << exponent).min(MAX_RETRY_DELAY_MS)
}

fn read_record(path: &Path) -> Result<CanonOutcomeOutboxRecord, OutboxError> {
    if !path.is_file() {
        return Err(OutboxError::NotFound);
    }
    let record: CanonOutcomeOutboxRecord = serde_json::from_slice(&fs::read(path)?)?;
    if record.schema_version == OUTBOX_SCHEMA_VERSION {
        Ok(record)
    } else {
        Err(OutboxError::IdentityDigestConflict)
    }
}

fn write_record(path: &Path, record: &CanonOutcomeOutboxRecord) -> Result<(), OutboxError> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("missing outbox directory"))?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(record)?;
    let mut file = File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    sync_directory(parent)?;
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), OutboxError> {
    File::open(path)?.sync_all()?;
    Ok(())
}
