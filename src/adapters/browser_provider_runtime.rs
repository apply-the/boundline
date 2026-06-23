//! Browser provider runtime adapter for Boundline.
//!
//! Spawns a registered browser capability provider as a subprocess,
//! writes JSON validation requests to stdin, reads JSON evidence
//! responses from stdout, and normalizes findings into Boundline
//! structured output.
//!
//! The browser provider is an external binary — Boundline does not
//! embed Playwright or any browser automation library.

use boundline_core::domain::browser_provider::{BrowserEvidencePacket, BrowserValidationStep};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Errors that can occur during provider runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum BrowserProviderError {
    /// The provider binary could not be started.
    #[error("failed to start browser provider `{command}`: {source}")]
    StartFailure { command: String, source: std::io::Error },

    /// The provider did not emit a startup handshake within the timeout.
    #[error("browser provider startup handshake timed out after {timeout_seconds}s")]
    HandshakeTimeout { timeout_seconds: u32 },

    /// The provider emitted a malformed handshake line.
    #[error("browser provider handshake malformed: {detail}")]
    HandshakeMalformed { detail: String },

    /// The provider exited unexpectedly before producing a response.
    #[error("browser provider exited with code {code:?}: {stderr_summary}")]
    ProviderExited { code: Option<i32>, stderr_summary: String },

    /// The provider response was not valid JSON.
    #[error("browser provider response was not valid JSON: {detail}")]
    InvalidResponse { detail: String },

    /// Writing the request to provider stdin failed.
    #[error("failed to write request to browser provider stdin: {source}")]
    RequestWriteFailure { source: std::io::Error },

    /// Reading the response from provider stdout failed.
    #[error("failed to read response from browser provider stdout: {source}")]
    ResponseReadFailure { source: std::io::Error },
}

/// Manages the lifecycle of a browser capability provider subprocess.
pub struct BrowserProviderRuntime {
    /// The provider child process, if currently running.
    child: Option<Child>,
    /// The configured command path.
    command: String,
    /// CLI arguments.
    args: Vec<String>,
    /// Maximum seconds to wait for the startup handshake.
    startup_timeout: Duration,
}

impl BrowserProviderRuntime {
    /// Create a new runtime handle for a browser provider.
    pub fn new(command: &str, args: &[String], startup_timeout_seconds: u32) -> Self {
        Self {
            child: None,
            command: command.to_string(),
            args: args.to_vec(),
            startup_timeout: Duration::from_secs(u64::from(startup_timeout_seconds)),
        }
    }

    /// Start the provider subprocess and read the startup handshake.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserProviderError::StartFailure`] if the subprocess
    /// cannot be spawned, [`BrowserProviderError::HandshakeTimeout`] if
    /// no handshake line arrives within the configured timeout, or
    /// [`BrowserProviderError::HandshakeMalformed`] if the handshake
    /// JSON is invalid.
    pub fn start(&mut self) -> Result<(), BrowserProviderError> {
        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| BrowserProviderError::StartFailure {
                command: self.command.clone(),
                source,
            })?;

        let stdout = child.stdout.take().ok_or_else(|| BrowserProviderError::StartFailure {
            command: self.command.clone(),
            source: std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "provider stdout unavailable",
            ),
        })?;
        let mut reader = BufReader::new(stdout);

        let deadline = Instant::now() + self.startup_timeout;
        let mut handshake_line = String::new();

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                let _ = child.kill();
                return Err(BrowserProviderError::HandshakeTimeout {
                    timeout_seconds: self.startup_timeout.as_secs() as u32,
                });
            }

            handshake_line.clear();
            let bytes = reader
                .read_line(&mut handshake_line)
                .map_err(|source| BrowserProviderError::ResponseReadFailure { source })?;
            if bytes > 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        let handshake: serde_json::Value =
            serde_json::from_str(handshake_line.trim()).map_err(|_| {
                BrowserProviderError::HandshakeMalformed {
                    detail: "handshake line is not valid JSON".into(),
                }
            })?;

        let protocol = handshake.get("protocol").and_then(|v| v.as_str()).unwrap_or("");
        if protocol != "browser-provider-v1" {
            return Err(BrowserProviderError::HandshakeMalformed {
                detail: format!("expected protocol 'browser-provider-v1', got '{protocol}'"),
            });
        }

        self.child = Some(child);
        Ok(())
    }

    /// Dispatch a validation step to the provider and collect the response.
    ///
    /// # Errors
    ///
    /// Returns a provider error if the request cannot be written, the
    /// response cannot be read, the JSON is malformed, or the process
    /// exits unexpectedly.
    pub fn dispatch(
        &mut self,
        step: &BrowserValidationStep,
    ) -> Result<BrowserEvidencePacket, BrowserProviderError> {
        let child = self.child.as_mut().ok_or_else(|| BrowserProviderError::StartFailure {
            command: self.command.clone(),
            source: std::io::Error::new(std::io::ErrorKind::NotConnected, "provider not started"),
        })?;

        let request_json =
            serde_json::to_string(step).map_err(|e| BrowserProviderError::InvalidResponse {
                detail: format!("request serialization failed: {e}"),
            })?;

        {
            let stdin =
                child.stdin.as_mut().ok_or_else(|| BrowserProviderError::RequestWriteFailure {
                    source: std::io::Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        "provider stdin unavailable",
                    ),
                })?;
            writeln!(stdin, "{request_json}")
                .map_err(|source| BrowserProviderError::RequestWriteFailure { source })?;
            stdin.flush().map_err(|source| BrowserProviderError::RequestWriteFailure { source })?;
        }

        let stdout =
            child.stdout.as_mut().ok_or_else(|| BrowserProviderError::ResponseReadFailure {
                source: std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "provider stdout unavailable",
                ),
            })?;
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .map_err(|source| BrowserProviderError::ResponseReadFailure { source })?;

        let packet: BrowserEvidencePacket =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                BrowserProviderError::InvalidResponse {
                    detail: format!("response deserialization failed: {e}"),
                }
            })?;

        Ok(packet)
    }

    /// Capture stderr output from the provider for diagnostics.
    #[must_use]
    pub fn capture_stderr(&mut self) -> Option<String> {
        let child = self.child.as_mut()?;
        let stderr = child.stderr.take()?;
        let mut reader = BufReader::new(stderr);
        let mut stderr_bytes = Vec::new();
        let _ = reader.read_to_end(&mut stderr_bytes);
        let output = String::from_utf8_lossy(&stderr_bytes).into_owned();
        if output.is_empty() { None } else { Some(output) }
    }
}

impl Drop for BrowserProviderRuntime {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
