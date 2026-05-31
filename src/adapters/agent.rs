//! Step-agent adapters and framework-adapter subprocess hosts.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::adapters::framework_protocol::{
    FrameworkAdapterCommand, FrameworkAdapterDescribeResponse, FrameworkAdapterExecuteStageRequest,
    FrameworkAdapterExecuteStageResponse, FrameworkAdapterPreflightRequest,
    FrameworkAdapterPreflightResponse, FrameworkAdapterResponseEnvelope,
    FrameworkAdapterResponseEnvelopeError, HookEmissionRequest, HookEmissionResponse,
};
use crate::domain::step::{StepExecutionRequest, StepExecutionResult};

const NO_DIAGNOSTICS_DETAIL: &str =
    "subprocess returned a non-zero exit status without diagnostics";

pub trait AgentAdapter: Send + Sync {
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult;
}

/// Subprocess host surface for the framework-adapter stdio protocol.
pub trait FrameworkAdapterHost: Send + Sync {
    /// Executes the adapter `describe` command.
    fn describe(&self) -> Result<FrameworkAdapterDescribeResponse, FrameworkAdapterHostError>;

    /// Executes the adapter `preflight` command with the provided config payload.
    fn preflight(
        &self,
        request: &FrameworkAdapterPreflightRequest,
    ) -> Result<FrameworkAdapterPreflightResponse, FrameworkAdapterHostError>;

    /// Executes the adapter `execute-stage` command for a claimed stage.
    fn execute_stage(
        &self,
        request: &FrameworkAdapterExecuteStageRequest,
    ) -> Result<FrameworkAdapterExecuteStageResponse, FrameworkAdapterHostError>;

    /// Executes the adapter `emit-hook` command for one observable hook.
    fn emit_hook(
        &self,
        request: &HookEmissionRequest,
    ) -> Result<HookEmissionResponse, FrameworkAdapterHostError>;
}

pub type SharedAgentAdapter = Arc<dyn AgentAdapter>;
pub type SharedFrameworkAdapterHost = Arc<dyn FrameworkAdapterHost>;

/// Concrete one-shot subprocess runner for the framework-adapter protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubprocessFrameworkAdapterHost {
    command: String,
    args: Vec<String>,
    working_directory: Option<PathBuf>,
}

impl SubprocessFrameworkAdapterHost {
    /// Creates a new subprocess host for the persisted adapter command.
    pub fn new(command: impl Into<String>) -> Result<Self, FrameworkAdapterHostError> {
        let command = command.into();
        if command.trim().is_empty() {
            return Err(FrameworkAdapterHostError::EmptyCommand);
        }

        Ok(Self { command, args: Vec::new(), working_directory: None })
    }

    /// Replaces the fixed arguments passed to the adapter binary.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Sets the working directory used for subprocess invocations.
    pub fn with_working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    /// Returns the persisted adapter command.
    pub fn command(&self) -> &str {
        &self.command
    }

    fn process_failed_error(
        &self,
        request_kind: String,
        output: Output,
    ) -> FrameworkAdapterHostError {
        FrameworkAdapterHostError::ProcessFailed {
            command: self.command.clone(),
            request_kind,
            detail: transport_failure_detail(output),
        }
    }

    fn invoke_without_payload<Response>(
        &self,
        protocol_command: FrameworkAdapterCommand,
    ) -> Result<Response, FrameworkAdapterHostError>
    where
        Response: DeserializeOwned,
    {
        self.invoke(protocol_command, None::<&()>)
    }

    fn invoke<Request, Response>(
        &self,
        protocol_command: FrameworkAdapterCommand,
        request: Option<&Request>,
    ) -> Result<Response, FrameworkAdapterHostError>
    where
        Request: Serialize,
        Response: DeserializeOwned,
    {
        let request_kind = protocol_command.as_str().to_string();
        let request_payload = match request {
            Some(payload) => Some(serde_json::to_vec(payload).map_err(|source| {
                FrameworkAdapterHostError::SerializeRequest {
                    command: self.command.clone(),
                    request_kind: request_kind.clone(),
                    source,
                }
            })?),
            None => None,
        };

        let mut command = Command::new(&self.command);
        command
            .args(&self.args)
            .arg(protocol_command.as_str())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if request_payload.is_some() {
            command.stdin(Stdio::piped());
        } else {
            command.stdin(Stdio::null());
        }

        if let Some(working_directory) = self.working_directory.as_deref() {
            command.current_dir(working_directory);
        }

        let mut child = command.spawn().map_err(|source| FrameworkAdapterHostError::Spawn {
            command: self.command.clone(),
            request_kind: request_kind.clone(),
            source,
        })?;

        if let Some(payload) = request_payload
            && let Some(mut stdin) = child.stdin.take()
            && let Err(source) = stdin.write_all(&payload)
        {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child.wait_with_output().map_err(|wait_source| {
                        FrameworkAdapterHostError::Wait {
                            command: self.command.clone(),
                            request_kind: request_kind.clone(),
                            source: wait_source,
                        }
                    })?;
                    if !output.status.success() {
                        return Err(self.process_failed_error(request_kind, output));
                    }
                }
                Ok(None) | Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
            return Err(FrameworkAdapterHostError::WriteRequest {
                command: self.command.clone(),
                request_kind,
                source,
            });
        }

        let output =
            child.wait_with_output().map_err(|source| FrameworkAdapterHostError::Wait {
                command: self.command.clone(),
                request_kind: request_kind.clone(),
                source,
            })?;

        if !output.status.success() {
            return Err(self.process_failed_error(request_kind, output));
        }

        let envelope =
            serde_json::from_slice::<FrameworkAdapterResponseEnvelope<Response>>(&output.stdout)
                .map_err(|source| FrameworkAdapterHostError::DeserializeResponse {
                    command: self.command.clone(),
                    request_kind: request_kind.clone(),
                    source,
                })?;

        envelope.into_result().map_err(|error| match error {
            FrameworkAdapterResponseEnvelopeError::InvalidEnvelope { detail } => {
                FrameworkAdapterHostError::InvalidEnvelope {
                    command: self.command.clone(),
                    request_kind,
                    detail,
                }
            }
            FrameworkAdapterResponseEnvelopeError::Protocol { code, message, details } => {
                FrameworkAdapterHostError::ProtocolError {
                    command: self.command.clone(),
                    request_kind,
                    code,
                    message,
                    details: details.map(Box::new),
                }
            }
        })
    }
}

fn transport_failure_detail(output: Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    if detail.is_empty() { NO_DIAGNOSTICS_DETAIL.to_string() } else { detail }
}

pub struct FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    handler: F,
}

impl<F> FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> AgentAdapter for FnAgentAdapter<F>
where
    F: Fn(StepExecutionRequest) -> StepExecutionResult + Send + Sync,
{
    fn execute(&self, request: StepExecutionRequest) -> StepExecutionResult {
        (self.handler)(request)
    }
}

impl FrameworkAdapterHost for SubprocessFrameworkAdapterHost {
    fn describe(&self) -> Result<FrameworkAdapterDescribeResponse, FrameworkAdapterHostError> {
        self.invoke_without_payload(FrameworkAdapterCommand::Describe)
    }

    fn preflight(
        &self,
        request: &FrameworkAdapterPreflightRequest,
    ) -> Result<FrameworkAdapterPreflightResponse, FrameworkAdapterHostError> {
        self.invoke(FrameworkAdapterCommand::Preflight, Some(request))
    }

    fn execute_stage(
        &self,
        request: &FrameworkAdapterExecuteStageRequest,
    ) -> Result<FrameworkAdapterExecuteStageResponse, FrameworkAdapterHostError> {
        self.invoke(FrameworkAdapterCommand::ExecuteStage, Some(request))
    }

    fn emit_hook(
        &self,
        request: &HookEmissionRequest,
    ) -> Result<HookEmissionResponse, FrameworkAdapterHostError> {
        self.invoke(FrameworkAdapterCommand::EmitHook, Some(request))
    }
}

/// Transport and serialization failures surfaced by the framework-adapter host.
#[derive(Debug, Error)]
pub enum FrameworkAdapterHostError {
    #[error("framework-adapter command must not be empty")]
    EmptyCommand,
    #[error(
        "failed to serialize framework-adapter {request_kind} request for `{command}`: {source}"
    )]
    SerializeRequest { command: String, request_kind: String, source: serde_json::Error },
    #[error("failed to start framework-adapter `{command}` for {request_kind}: {source}")]
    Spawn { command: String, request_kind: String, source: std::io::Error },
    #[error("failed to send framework-adapter {request_kind} request to `{command}`: {source}")]
    WriteRequest { command: String, request_kind: String, source: std::io::Error },
    #[error("failed to wait for framework-adapter `{command}` {request_kind} response: {source}")]
    Wait { command: String, request_kind: String, source: std::io::Error },
    #[error("framework-adapter `{command}` {request_kind} command failed: {detail}")]
    ProcessFailed { command: String, request_kind: String, detail: String },
    #[error("framework-adapter `{command}` returned malformed {request_kind} JSON: {source}")]
    DeserializeResponse { command: String, request_kind: String, source: serde_json::Error },
    #[error("framework-adapter `{command}` returned an invalid {request_kind} envelope: {detail}")]
    InvalidEnvelope { command: String, request_kind: String, detail: String },
    #[error(
        "framework-adapter `{command}` returned protocol error for {request_kind}: {code}: {message}"
    )]
    ProtocolError {
        command: String,
        request_kind: String,
        code: String,
        message: String,
        details: Option<Box<Value>>,
    },
}
