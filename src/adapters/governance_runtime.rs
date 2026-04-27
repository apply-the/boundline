use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::governance::{
    ApprovalState, CanonMode, GovernanceLifecycleState, GovernanceRuntimeKind, GovernedStagePacket,
    PacketReadiness, SystemContextBinding, classify_packet_readiness,
    derived_packet_missing_sections,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceRequestKind {
    Start,
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceInputDocument {
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReusedPacketInput {
    pub stage_key: String,
    pub packet_ref: String,
    pub headline: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceBoundedContext {
    #[serde(default)]
    pub read_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_brief_ref: Option<String>,
    #[serde(default)]
    pub reused_packets: Vec<ReusedPacketInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceRuntimeRequest {
    pub request_kind: GovernanceRequestKind,
    pub governance_attempt_id: String,
    pub stage_key: String,
    pub goal: String,
    pub workspace_ref: String,
    pub autopilot: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<CanonMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_context: Option<SystemContextBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ref: Option<String>,
    pub bounded_context: GovernanceBoundedContext,
    #[serde(default)]
    pub input_documents: Vec<GovernanceInputDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceRuntimeResponse {
    pub status: GovernanceLifecycleState,
    pub approval_state: ApprovalState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet: Option<GovernedStagePacket>,
    pub message: String,
}

pub trait GovernanceRuntime: Send + Sync {
    fn kind(&self) -> GovernanceRuntimeKind;
    fn execute(
        &self,
        request: &GovernanceRuntimeRequest,
    ) -> Result<GovernanceRuntimeResponse, GovernanceRuntimeError>;
}

#[derive(Debug, Clone, Default)]
pub struct LocalGovernanceRuntime;

impl GovernanceRuntime for LocalGovernanceRuntime {
    fn kind(&self) -> GovernanceRuntimeKind {
        GovernanceRuntimeKind::Local
    }

    fn execute(
        &self,
        request: &GovernanceRuntimeRequest,
    ) -> Result<GovernanceRuntimeResponse, GovernanceRuntimeError> {
        if request.stage_key.trim().is_empty() {
            return Err(GovernanceRuntimeError::InvalidRequest(
                "stage_key must not be empty".to_string(),
            ));
        }

        if request.governance_attempt_id.trim().is_empty() {
            return Err(GovernanceRuntimeError::InvalidRequest(
                "governance_attempt_id must not be empty".to_string(),
            ));
        }

        let stage_brief_ref = request.bounded_context.stage_brief_ref.clone().or_else(|| {
            request
                .input_documents
                .iter()
                .find(|document| document.kind == "stage-brief")
                .map(|document| document.path.clone())
        });

        if request.bounded_context.read_targets.is_empty() && stage_brief_ref.is_none() {
            return Ok(GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::Blocked,
                approval_state: ApprovalState::NotNeeded,
                run_ref: None,
                packet: None,
                message: format!(
                    "local governance blocked {} because no bounded stage context was provided",
                    request.stage_key
                ),
            });
        }

        let packet_ref = request.packet_ref.clone().unwrap_or_else(|| {
            format!(
                ".synod/governance/{}/{}",
                request.stage_key.replace(':', "-"),
                request.governance_attempt_id
            )
        });

        let expected_document_refs = stage_brief_ref.clone().map_or_else(
            || vec![format!("{packet_ref}/brief.md")],
            |stage_brief_ref| vec![stage_brief_ref],
        );
        let document_refs = stage_brief_ref.map_or_else(
            || {
                if request.bounded_context.read_targets.is_empty() {
                    Vec::new()
                } else {
                    expected_document_refs.clone()
                }
            },
            |stage_brief_ref| vec![stage_brief_ref],
        );
        let readiness = if document_refs.is_empty() {
            PacketReadiness::Incomplete
        } else {
            PacketReadiness::Reusable
        };

        let packet = GovernedStagePacket {
            packet_ref,
            runtime: GovernanceRuntimeKind::Local,
            canon_mode: None,
            expected_document_refs,
            document_refs,
            readiness,
            missing_sections: Vec::new(),
            headline: format!("local governance packet ready for {}", request.stage_key),
        };

        let status = if packet.readiness == PacketReadiness::Reusable {
            GovernanceLifecycleState::GovernedReady
        } else {
            GovernanceLifecycleState::Blocked
        };

        Ok(GovernanceRuntimeResponse {
            status,
            approval_state: ApprovalState::NotNeeded,
            run_ref: None,
            packet: Some(packet),
            message: format!("local governance evaluated {}", request.stage_key),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CanonCliRuntime {
    command: String,
    working_directory: Option<PathBuf>,
}

impl CanonCliRuntime {
    pub fn new(command: impl Into<String>) -> Self {
        Self { command: command.into(), working_directory: None }
    }

    pub fn with_working_directory(mut self, working_directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(working_directory.into());
        self
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn working_directory(&self) -> Option<&std::path::Path> {
        self.working_directory.as_deref()
    }
}

impl GovernanceRuntime for CanonCliRuntime {
    fn kind(&self) -> GovernanceRuntimeKind {
        GovernanceRuntimeKind::Canon
    }

    fn execute(
        &self,
        request: &GovernanceRuntimeRequest,
    ) -> Result<GovernanceRuntimeResponse, GovernanceRuntimeError> {
        if request.stage_key.trim().is_empty() {
            return Err(GovernanceRuntimeError::InvalidRequest(
                "stage_key must not be empty".to_string(),
            ));
        }

        if request.governance_attempt_id.trim().is_empty() {
            return Err(GovernanceRuntimeError::InvalidRequest(
                "governance_attempt_id must not be empty".to_string(),
            ));
        }

        if let Some(response) = validate_canon_request(request) {
            return Ok(response);
        }

        let request_payload = serde_json::to_vec(request)
            .map_err(|error| GovernanceRuntimeError::MalformedOutput(error.to_string()))?;

        let mut command = Command::new(&self.command);
        command
            .arg("governance")
            .arg(request_kind_text(request.request_kind))
            .arg("--json")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(working_directory) = self.working_directory.as_deref() {
            command.current_dir(working_directory);
        } else if Path::new(&request.workspace_ref).is_dir() {
            command.current_dir(&request.workspace_ref);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                return Ok(failed_canon_response(format!(
                    "failed to start Canon for {}: {error}",
                    request.stage_key
                )));
            }
        };

        if let Some(mut stdin) = child.stdin.take()
            && let Err(error) = stdin.write_all(&request_payload)
        {
            let _ = child.kill();
            return Ok(failed_canon_response(format!(
                "failed to send Canon request for {}: {error}",
                request.stage_key
            )));
        }

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(error) => {
                return Ok(failed_canon_response(format!(
                    "failed to wait for Canon response for {}: {error}",
                    request.stage_key
                )));
            }
        };

        if let Some(response) = parse_canon_response(request, &output.stdout) {
            return Ok(response);
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };

        Ok(failed_canon_response(if output.status.success() {
            format!(
                "Canon returned malformed output for {}{}",
                request.stage_key,
                if detail.is_empty() { String::new() } else { format!(": {detail}") }
            )
        } else {
            format!(
                "Canon command failed for {}{}",
                request.stage_key,
                if detail.is_empty() { String::new() } else { format!(": {detail}") }
            )
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CanonCliWireResponse {
    pub status: GovernanceLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ref: Option<String>,
    #[serde(default)]
    pub expected_document_refs: Vec<String>,
    #[serde(default)]
    pub document_refs: Vec<String>,
    #[serde(default = "default_approval_state")]
    pub approval_state: ApprovalState,
    #[serde(default = "default_packet_readiness")]
    pub packet_readiness: PacketReadiness,
    #[serde(default)]
    pub missing_sections: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    pub message: String,
}

fn request_kind_text(kind: GovernanceRequestKind) -> &'static str {
    match kind {
        GovernanceRequestKind::Start => "start",
        GovernanceRequestKind::Refresh => "refresh",
    }
}

fn default_approval_state() -> ApprovalState {
    ApprovalState::NotNeeded
}

fn default_packet_readiness() -> PacketReadiness {
    PacketReadiness::Pending
}

fn validate_canon_request(request: &GovernanceRuntimeRequest) -> Option<GovernanceRuntimeResponse> {
    let missing_field = if request.mode.is_none() {
        Some("mode")
    } else if request.system_context.is_none() {
        Some("system_context")
    } else if request.risk.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        Some("risk")
    } else if request.zone.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        Some("zone")
    } else if request.owner.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        Some("owner")
    } else if matches!(request.request_kind, GovernanceRequestKind::Refresh)
        && request.run_ref.as_deref().map(str::trim).unwrap_or_default().is_empty()
    {
        Some("run_ref")
    } else {
        None
    };

    missing_field.map(|field| {
        blocked_canon_response(format!(
            "Canon blocked {} because required field '{field}' was not provided",
            request.stage_key
        ))
    })
}

fn parse_canon_response(
    request: &GovernanceRuntimeRequest,
    stdout: &[u8],
) -> Option<GovernanceRuntimeResponse> {
    if stdout.is_empty() {
        return None;
    }

    let wire = serde_json::from_slice::<CanonCliWireResponse>(stdout).ok()?;
    Some(normalize_canon_response(request, wire))
}

fn normalize_canon_response(
    request: &GovernanceRuntimeRequest,
    wire: CanonCliWireResponse,
) -> GovernanceRuntimeResponse {
    let packet = wire.packet_ref.as_ref().map(|packet_ref| {
        let expected_document_refs = if wire.expected_document_refs.is_empty() {
            request.mode.map(|mode| mode.expected_document_refs(packet_ref)).unwrap_or_default()
        } else {
            wire.expected_document_refs.clone()
        };
        let missing_sections = derived_packet_missing_sections(
            Path::new(&request.workspace_ref),
            &expected_document_refs,
            &wire.document_refs,
            &wire.missing_sections,
        );
        let readiness = classify_packet_readiness(
            Path::new(&request.workspace_ref),
            &expected_document_refs,
            &wire.document_refs,
            &wire.missing_sections,
            wire.packet_readiness,
        );

        GovernedStagePacket {
            packet_ref: packet_ref.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: request.mode,
            expected_document_refs,
            document_refs: wire.document_refs.clone(),
            readiness,
            missing_sections,
            headline: wire.headline.clone().unwrap_or_else(|| wire.message.clone()),
        }
    });

    let mut status = wire.status;
    let mut message = wire.message;
    if matches!(status, GovernanceLifecycleState::GovernedReady)
        && packet.as_ref().is_none_or(|packet| packet.readiness != PacketReadiness::Reusable)
    {
        status = GovernanceLifecycleState::Blocked;
        message = format!("Canon produced a non-reusable packet for {}", request.stage_key);
    }

    GovernanceRuntimeResponse {
        status,
        approval_state: wire.approval_state,
        run_ref: wire.run_ref,
        packet,
        message,
    }
}

fn blocked_canon_response(message: String) -> GovernanceRuntimeResponse {
    GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Blocked,
        approval_state: ApprovalState::NotNeeded,
        run_ref: None,
        packet: None,
        message,
    }
}

fn failed_canon_response(message: String) -> GovernanceRuntimeResponse {
    GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Failed,
        approval_state: ApprovalState::NotNeeded,
        run_ref: None,
        packet: None,
        message,
    }
}

#[derive(Debug, Error)]
pub enum GovernanceRuntimeError {
    #[error("governance runtime request is invalid: {0}")]
    InvalidRequest(String),
    #[error("governance runtime is not supported: {0}")]
    Unsupported(String),
    #[error("governance runtime I/O failed: {0}")]
    Io(std::io::Error),
    #[error("governance runtime returned malformed output: {0}")]
    MalformedOutput(String),
}
