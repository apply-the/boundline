//! Bounded one-shot Canon transport and redacted failure classification.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use canon_contracts::{
    CanonContractVersion, OneShotOperation, OneShotResponse, RecordOutcomeRequest,
    RecordOutcomeResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_CANON_REQUEST_BYTES: usize = 1_048_576;
const DEFAULT_MAX_RESPONSE_BYTES: usize = 1_048_576;
const PROCESS_POLL_INTERVAL_MS: u64 = 10;

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
    pub(super) const fn as_str(self) -> &'static str {
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
