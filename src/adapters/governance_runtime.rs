use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::governance::{
    ApprovalState, CanonAdaptiveGovernanceV1Envelope, CanonAuthorityGovernanceV1Envelope,
    CanonCapabilitySnapshot, CanonMode, CanonSemanticArtifactDescriptorV1Envelope,
    GovernanceLifecycleState, GovernanceRuntimeKind, GovernedStagePacket, PacketReadiness,
    SystemContextBinding, classify_packet_readiness, derived_packet_missing_sections,
    deserialize_known_canon_modes,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
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
                reason_code: None,
                message: format!(
                    "local governance blocked {} because no bounded stage context was provided",
                    request.stage_key
                ),
            });
        }

        let packet_ref = request.packet_ref.clone().unwrap_or_else(|| {
            format!(
                ".boundline/governance/{}/{}",
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
            reason_code: None,
            authority_governance: None,
            adaptive_governance: None,
            semantic_descriptor: None,
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
            reason_code: None,
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
struct CanonCapabilitiesWireResponse {
    pub canon_version: String,
    #[serde(default)]
    pub supported_schema_versions: Vec<String>,
    #[serde(default)]
    pub operations: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_known_canon_modes")]
    pub supported_modes: Vec<CanonMode>,
    #[serde(default)]
    pub status_values: Vec<String>,
    #[serde(default)]
    pub approval_state_values: Vec<String>,
    #[serde(default)]
    pub packet_readiness_values: Vec<String>,
    #[serde(default)]
    pub compatibility_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonCliWireResponse {
    pub status: GovernanceLifecycleState,
    pub run_ref: Option<String>,
    pub packet_ref: Option<String>,
    pub expected_document_refs: Vec<String>,
    pub document_refs: Vec<String>,
    pub approval_state: ApprovalState,
    pub packet_readiness: PacketReadiness,
    pub missing_sections: Vec<String>,
    pub headline: Option<String>,
    pub reason_code: Option<String>,
    pub authority_governance: Option<CanonAuthorityGovernanceV1Envelope>,
    pub adaptive_governance: Option<CanonAdaptiveGovernanceV1Envelope>,
    pub semantic_descriptor: Option<CanonSemanticArtifactDescriptorV1Envelope>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CanonCliWireResponseCore {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonPacketMetadata {
    pub authority_governance: Option<CanonAuthorityGovernanceV1Envelope>,
    pub adaptive_governance: Option<CanonAdaptiveGovernanceV1Envelope>,
    pub semantic_descriptor: Option<CanonSemanticArtifactDescriptorV1Envelope>,
}

const CANON_PACKET_METADATA_FILE_NAME: &str = "packet-metadata.json";

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

pub fn query_canon_capabilities(
    command: &str,
    workspace_ref: &Path,
) -> Result<Option<CanonCapabilitySnapshot>, GovernanceRuntimeError> {
    if command.trim().is_empty() {
        return Ok(None);
    }

    let output = match query_canon_capabilities_output(command, workspace_ref) {
        Some(output) => output,
        None => return Ok(None),
    };

    if !output.status.success() {
        return Ok(None);
    }

    Ok(parse_canon_capabilities(&output.stdout))
}

fn build_canon_capabilities_process(command: &str, workspace_ref: &Path) -> Command {
    let mut process = Command::new(command);
    process
        .arg("governance")
        .arg("capabilities")
        .arg("--json")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if workspace_ref.is_dir() {
        process.current_dir(workspace_ref);
    }

    process
}

#[cfg(unix)]
fn query_canon_capabilities_output(
    command: &str,
    workspace_ref: &Path,
) -> Option<std::process::Output> {
    let mut process = build_canon_capabilities_process(command, workspace_ref);
    match process.output() {
        Ok(output) => Some(output),
        Err(_) => {
            let command_path = Path::new(command);
            if !command_path.is_file() {
                return None;
            }

            let mut shell_process = Command::new("/bin/sh");
            shell_process
                .arg(command_path)
                .arg("governance")
                .arg("capabilities")
                .arg("--json")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            if workspace_ref.is_dir() {
                shell_process.current_dir(workspace_ref);
            }

            shell_process.output().ok()
        }
    }
}

#[cfg(not(unix))]
fn query_canon_capabilities_output(
    command: &str,
    workspace_ref: &Path,
) -> Option<std::process::Output> {
    let mut process = build_canon_capabilities_process(command, workspace_ref);
    process.output().ok()
}

fn parse_canon_capabilities(stdout: &[u8]) -> Option<CanonCapabilitySnapshot> {
    if stdout.is_empty() {
        return None;
    }

    let wire = serde_json::from_slice::<CanonCapabilitiesWireResponse>(stdout).ok()?;
    Some(CanonCapabilitySnapshot {
        canon_version: wire.canon_version,
        supported_schema_versions: wire.supported_schema_versions,
        operations: wire.operations,
        supported_modes: wire.supported_modes,
        status_values: wire.status_values,
        approval_state_values: wire.approval_state_values,
        packet_readiness_values: wire.packet_readiness_values,
        compatibility_notes: wire.compatibility_notes,
    })
}

fn parse_canon_response(
    request: &GovernanceRuntimeRequest,
    stdout: &[u8],
) -> Option<GovernanceRuntimeResponse> {
    if stdout.is_empty() {
        return None;
    }

    let raw = serde_json::from_slice::<serde_json::Value>(stdout).ok()?;
    let core = serde_json::from_value::<CanonCliWireResponseCore>(raw.clone()).ok()?;
    let wire = CanonCliWireResponse {
        status: core.status,
        run_ref: core.run_ref,
        packet_ref: core.packet_ref,
        expected_document_refs: core.expected_document_refs,
        document_refs: core.document_refs,
        approval_state: core.approval_state,
        packet_readiness: core.packet_readiness,
        missing_sections: core.missing_sections,
        headline: core.headline,
        reason_code: core.reason_code,
        authority_governance: deserialize_optional_json_field(&raw, "authority_governance"),
        adaptive_governance: deserialize_optional_json_field(&raw, "adaptive_governance")
            .filter(CanonAdaptiveGovernanceV1Envelope::is_supported_contract_line),
        semantic_descriptor: deserialize_optional_json_field(&raw, "semantic_descriptor")
            .filter(CanonSemanticArtifactDescriptorV1Envelope::is_supported_contract_line),
        message: core.message,
    };
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
        let authority_governance = wire.authority_governance.clone().or_else(|| {
            read_canon_packet_metadata(Path::new(&request.workspace_ref), packet_ref)
                .and_then(|metadata| metadata.authority_governance)
        });
        let adaptive_governance = wire.adaptive_governance.clone().or_else(|| {
            read_canon_packet_metadata(Path::new(&request.workspace_ref), packet_ref)
                .and_then(|metadata| metadata.adaptive_governance)
        });
        let semantic_descriptor = wire.semantic_descriptor.clone().or_else(|| {
            read_canon_packet_metadata(Path::new(&request.workspace_ref), packet_ref)
                .and_then(|metadata| metadata.semantic_descriptor)
        });

        GovernedStagePacket {
            packet_ref: packet_ref.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: request.mode,
            expected_document_refs,
            document_refs: wire.document_refs.clone(),
            readiness,
            missing_sections,
            headline: wire.headline.clone().unwrap_or_else(|| wire.message.clone()),
            reason_code: wire.reason_code.clone(),
            authority_governance,
            adaptive_governance,
            semantic_descriptor,
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
        reason_code: wire.reason_code,
        message,
    }
}

fn read_canon_packet_metadata(
    workspace_ref: &Path,
    packet_ref: &str,
) -> Option<CanonPacketMetadata> {
    let path = packet_metadata_path(workspace_ref, packet_ref);
    let contents = fs::read(path).ok()?;
    let raw = serde_json::from_slice::<serde_json::Value>(&contents).ok()?;
    let metadata = raw.get("metadata")?;

    Some(CanonPacketMetadata {
        authority_governance: deserialize_optional_json_field(metadata, "authority_governance"),
        adaptive_governance: deserialize_optional_json_field(metadata, "adaptive_governance")
            .filter(CanonAdaptiveGovernanceV1Envelope::is_supported_contract_line),
        semantic_descriptor: deserialize_optional_json_field(metadata, "semantic_descriptor")
            .filter(CanonSemanticArtifactDescriptorV1Envelope::is_supported_contract_line),
    })
}

fn deserialize_optional_json_field<T: DeserializeOwned>(
    raw: &serde_json::Value,
    field_name: &str,
) -> Option<T> {
    raw.get(field_name).cloned().and_then(|value| serde_json::from_value(value).ok())
}

fn packet_metadata_path(workspace_ref: &Path, packet_ref: &str) -> PathBuf {
    let packet_root = if Path::new(packet_ref).is_absolute() {
        PathBuf::from(packet_ref)
    } else {
        workspace_ref.join(packet_ref)
    };
    packet_root.join(CANON_PACKET_METADATA_FILE_NAME)
}

fn blocked_canon_response(message: String) -> GovernanceRuntimeResponse {
    GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Blocked,
        approval_state: ApprovalState::NotNeeded,
        run_ref: None,
        packet: None,
        reason_code: None,
        message,
    }
}

fn failed_canon_response(message: String) -> GovernanceRuntimeResponse {
    GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Failed,
        approval_state: ApprovalState::NotNeeded,
        run_ref: None,
        packet: None,
        reason_code: None,
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use uuid::Uuid;

    use super::{
        CanonCliWireResponse, GovernanceBoundedContext, GovernanceRequestKind,
        GovernanceRuntimeRequest, normalize_canon_response, parse_canon_capabilities,
        parse_canon_response, query_canon_capabilities,
    };
    use crate::domain::governance::{
        ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE, AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE, ApprovalState,
        CanonAdaptiveGovernanceState, CanonAdaptiveRolloutProfile, CanonAuthorityZone,
        CanonChangeClass, CanonIntendedPersona, CanonMode, CanonRiskClass,
        CanonSemanticEligibilityState, CanonSemanticProvenanceBoundary, CanonStageRoleHintKind,
        GovernanceLifecycleState, PacketReadiness, SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE,
        SystemContextBinding,
    };

    fn temp_workspace(prefix: &str) -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn write_shell_script(prefix: &str, body: &str) -> std::path::PathBuf {
        let workspace = temp_workspace(prefix);
        let script_path = workspace.join("canon-stub.sh");
        fs::write(&script_path, body).unwrap();
        let mut permissions = fs::metadata(&script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).unwrap();
        script_path
    }

    fn request(workspace_ref: &str) -> GovernanceRuntimeRequest {
        GovernanceRuntimeRequest {
            request_kind: GovernanceRequestKind::Start,
            governance_attempt_id: "attempt-1".to_string(),
            stage_key: "change:verify".to_string(),
            goal: "Verify a governed change".to_string(),
            workspace_ref: workspace_ref.to_string(),
            autopilot: false,
            mode: Some(CanonMode::Verification),
            system_context: Some(SystemContextBinding::Existing),
            risk: Some("medium".to_string()),
            zone: Some("internal".to_string()),
            owner: Some("boundline".to_string()),
            run_ref: None,
            packet_ref: None,
            bounded_context: GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            input_documents: Vec::new(),
        }
    }

    #[test]
    fn parse_canon_response_preserves_reason_code_and_packet_metadata() {
        let workspace = temp_workspace("canon-governance-runtime");
        let packet_ref = "canon/run-123/verification";
        let document_ref = format!("{packet_ref}/verification.md");
        let document_path = workspace.join(&document_ref);
        fs::create_dir_all(document_path.parent().unwrap()).unwrap();
        fs::write(&document_path, "# Verification\n\nCredible validation evidence.").unwrap();

        let request = request(workspace.to_string_lossy().as_ref());
        let stdout = format!(
            "{{\"status\":\"governed_ready\",\"approval_state\":\"not_needed\",\"message\":\"Canon verified the stage\",\"run_ref\":\"run-123\",\"packet_ref\":\"{packet_ref}\",\"expected_document_refs\":[\"{document_ref}\"],\"document_refs\":[\"{document_ref}\"],\"packet_readiness\":\"reusable\",\"headline\":\"Verification packet ready\",\"reason_code\":\"packet_ready\"}}"
        );

        let response = parse_canon_response(&request, stdout.as_bytes()).unwrap();

        assert_eq!(response.status, GovernanceLifecycleState::GovernedReady);
        assert_eq!(response.reason_code.as_deref(), Some("packet_ready"));
        assert_eq!(response.run_ref.as_deref(), Some("run-123"));

        let packet = response.packet.expect("packet should be present");
        assert_eq!(packet.readiness, PacketReadiness::Reusable);
        assert_eq!(packet.reason_code.as_deref(), Some("packet_ready"));
        assert_eq!(packet.headline, "Verification packet ready");

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn parse_canon_response_reads_inline_semantic_descriptor() {
        let workspace = temp_workspace("canon-governance-runtime-inline-semantic");
        let packet_ref = "canon/run-124/verification";
        let document_ref = format!("{packet_ref}/verification.md");
        let document_path = workspace.join(&document_ref);
        fs::create_dir_all(document_path.parent().unwrap()).unwrap();
        fs::write(&document_path, "# Verification\n\nCredible validation evidence.").unwrap();

        let request = request(workspace.to_string_lossy().as_ref());
        let stdout = format!(
            "{{\"status\":\"governed_ready\",\"approval_state\":\"granted\",\"message\":\"Canon verified the stage\",\"run_ref\":\"run-124\",\"packet_ref\":\"{packet_ref}\",\"expected_document_refs\":[\"{document_ref}\"],\"document_refs\":[\"{document_ref}\"],\"packet_readiness\":\"reusable\",\"headline\":\"Verification packet ready\",\"reason_code\":\"packet_ready\",\"semantic_descriptor\":{{\"semantic_contract_line\":\"{SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE}\",\"semantic_eligibility\":\"eligible\",\"semantic_provenance_boundary\":\"surface\",\"semantic_provenance_ref\":\"{document_ref}\",\"semantic_labels\":[\"verification\"]}}}}"
        );

        let response = parse_canon_response(&request, stdout.as_bytes()).unwrap();
        let packet = response.packet.expect("packet should be present");
        let semantic = packet.semantic_descriptor.expect("semantic descriptor should be present");

        assert_eq!(semantic.semantic_contract_line, SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE);
        assert_eq!(semantic.semantic_eligibility, CanonSemanticEligibilityState::Eligible);
        assert_eq!(
            semantic.semantic_provenance_boundary,
            Some(CanonSemanticProvenanceBoundary::Surface)
        );
        assert_eq!(semantic.semantic_provenance_ref.as_deref(), Some(document_ref.as_str()));
        assert_eq!(semantic.semantic_labels, vec!["verification".to_string()]);

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn parse_canon_response_reads_authority_governance_from_packet_metadata() {
        let workspace = temp_workspace("canon-governance-runtime-authority");
        let packet_ref = "canon/run-789/verification";
        let document_ref = format!("{packet_ref}/verification.md");
        let document_path = workspace.join(&document_ref);
        let metadata_path = workspace.join(packet_ref).join("packet-metadata.json");
        fs::create_dir_all(document_path.parent().unwrap()).unwrap();
        fs::write(&document_path, "# Verification\n\nCredible validation evidence.").unwrap();
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-789",
                "mode": "verification",
                "metadata": {
                    "authority_governance": {
                        "contract_line": AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE,
                        "authority_zone": "yellow",
                        "change_class": "systemic-impact",
                        "intended_persona": "system-architect",
                        "approval_state": "granted",
                        "packet_readiness": "reusable",
                        "risk": "systemic-impact",
                        "persona_anti_behaviors": ["unbounded implementation detail"],
                        "primary_artifact": "verification.md",
                        "artifact_order": ["verification.md"],
                        "promotion_refs": ["canon://promotions/run-789"],
                        "stage_role_hints": [
                            {
                                "hint_kind": "reviewer-capability",
                                "value": "independent-review",
                                "rationale": "Require independent validation"
                            }
                        ]
                    },
                    "adaptive_governance": {
                        "contract_line": ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE,
                        "governance_state": "rule",
                        "rollout_profile": "governed"
                    },
                    "semantic_descriptor": {
                        "semantic_contract_line": SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE,
                        "semantic_eligibility": "eligible",
                        "semantic_provenance_boundary": "managed_block",
                        "semantic_provenance_ref": "docs/project/operational-context.md#managed-block-1",
                        "semantic_labels": ["project-memory", "operational-context"]
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let request = request(workspace.to_string_lossy().as_ref());
        let stdout = format!(
            "{{\"status\":\"governed_ready\",\"approval_state\":\"granted\",\"message\":\"Canon verified the stage\",\"run_ref\":\"run-789\",\"packet_ref\":\"{packet_ref}\",\"expected_document_refs\":[\"{document_ref}\"],\"document_refs\":[\"{document_ref}\"],\"packet_readiness\":\"reusable\",\"headline\":\"Verification packet ready\",\"reason_code\":\"packet_ready\"}}"
        );

        let response = parse_canon_response(&request, stdout.as_bytes()).unwrap();
        let packet = response.packet.expect("packet should be present");
        let authority = packet.authority_governance.expect("authority should be present");

        assert_eq!(authority.contract_line, AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE);
        assert_eq!(authority.authority_zone, CanonAuthorityZone::Yellow);
        assert_eq!(authority.change_class, CanonChangeClass::SystemicImpact);
        assert_eq!(authority.intended_persona, CanonIntendedPersona::SystemArchitect);
        assert_eq!(authority.risk, CanonRiskClass::SystemicImpact);
        assert_eq!(authority.primary_artifact.as_deref(), Some("verification.md"));
        assert_eq!(authority.stage_role_hints.len(), 1);
        assert_eq!(
            authority.stage_role_hints[0].hint_kind,
            CanonStageRoleHintKind::ReviewerCapability
        );
        assert_eq!(authority.stage_role_hints[0].value, "independent-review");
        let adaptive = packet.adaptive_governance.expect("adaptive companion should be present");
        assert_eq!(adaptive.contract_line, ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE);
        assert_eq!(adaptive.governance_state, CanonAdaptiveGovernanceState::Rule);
        assert_eq!(adaptive.rollout_profile, CanonAdaptiveRolloutProfile::Governed);
        let semantic = packet.semantic_descriptor.expect("semantic descriptor should be present");
        assert_eq!(semantic.semantic_contract_line, SEMANTIC_ARTIFACT_DESCRIPTOR_V1_CONTRACT_LINE);
        assert_eq!(semantic.semantic_eligibility, CanonSemanticEligibilityState::Eligible);
        assert_eq!(
            semantic.semantic_provenance_boundary,
            Some(CanonSemanticProvenanceBoundary::ManagedBlock)
        );
        assert_eq!(
            semantic.semantic_provenance_ref.as_deref(),
            Some("docs/project/operational-context.md#managed-block-1")
        );
        assert_eq!(
            semantic.semantic_labels,
            vec!["project-memory".to_string(), "operational-context".to_string()]
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn parse_canon_response_treats_malformed_adaptive_metadata_as_absent() {
        let workspace = temp_workspace("canon-governance-runtime-malformed-adaptive");
        let packet_ref = "canon/run-790/verification";
        let document_ref = format!("{packet_ref}/verification.md");
        let document_path = workspace.join(&document_ref);
        let metadata_path = workspace.join(packet_ref).join("packet-metadata.json");
        fs::create_dir_all(document_path.parent().unwrap()).unwrap();
        fs::write(&document_path, "# Verification\n\nCredible validation evidence.").unwrap();
        fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-790",
                "mode": "verification",
                "metadata": {
                    "authority_governance": {
                        "contract_line": AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE,
                        "authority_zone": "yellow",
                        "change_class": "systemic-impact",
                        "intended_persona": "system-architect",
                        "approval_state": "granted",
                        "packet_readiness": "reusable",
                        "risk": "systemic-impact"
                    },
                    "adaptive_governance": {
                        "contract_line": ADAPTIVE_GOVERNANCE_V1_CONTRACT_LINE,
                        "governance_state": "rule"
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let request = request(workspace.to_string_lossy().as_ref());
        let stdout = format!(
            "{{\"status\":\"governed_ready\",\"approval_state\":\"granted\",\"message\":\"Canon verified the stage\",\"run_ref\":\"run-790\",\"packet_ref\":\"{packet_ref}\",\"expected_document_refs\":[\"{document_ref}\"],\"document_refs\":[\"{document_ref}\"],\"packet_readiness\":\"reusable\",\"headline\":\"Verification packet ready\",\"reason_code\":\"packet_ready\"}}"
        );

        let response = parse_canon_response(&request, stdout.as_bytes()).unwrap();
        let packet = response.packet.expect("packet should be present");

        assert!(packet.authority_governance.is_some());
        assert!(packet.adaptive_governance.is_none());

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn normalize_canon_response_blocks_non_reusable_ready_packet() {
        let workspace = temp_workspace("canon-governance-runtime-blocked");
        let request = request(workspace.to_string_lossy().as_ref());
        let wire = CanonCliWireResponse {
            status: GovernanceLifecycleState::GovernedReady,
            run_ref: Some("run-456".to_string()),
            packet_ref: Some("canon/run-456/verification".to_string()),
            expected_document_refs: vec!["canon/run-456/verification/verification.md".to_string()],
            document_refs: Vec::new(),
            approval_state: ApprovalState::NotNeeded,
            packet_readiness: PacketReadiness::Incomplete,
            missing_sections: vec!["summary".to_string()],
            headline: Some("Verification packet incomplete".to_string()),
            reason_code: Some("missing_sections".to_string()),
            authority_governance: None,
            adaptive_governance: None,
            semantic_descriptor: None,
            message: "Canon produced an incomplete packet".to_string(),
        };

        let response = normalize_canon_response(&request, wire);

        assert_eq!(response.status, GovernanceLifecycleState::Blocked);
        assert_eq!(response.reason_code.as_deref(), Some("missing_sections"));
        assert_eq!(response.message, "Canon produced a non-reusable packet for change:verify");

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn parse_canon_capabilities_reads_supported_surface() {
        let stdout = format!(
            r#"{{
            "canon_version": "{}",
            "supported_schema_versions": ["2026-02-01"],
            "operations": ["start", "refresh", "capabilities"],
            "supported_modes": ["discovery", "verification", "pr-review"],
            "status_values": ["pending_selection", "running", "governed_ready", "awaiting_approval", "blocked", "completed", "failed"],
            "approval_state_values": ["not_needed", "requested", "granted", "rejected", "expired"],
            "packet_readiness_values": ["pending", "incomplete", "reusable", "rejected"],
            "compatibility_notes": ["stable-json", "mode-summary-separate"]
        }}"#,
            crate::domain::distribution::SUPPORTED_CANON_VERSION
        );

        let snapshot = parse_canon_capabilities(stdout.as_bytes()).unwrap();

        assert_eq!(snapshot.canon_version, crate::domain::distribution::SUPPORTED_CANON_VERSION);
        assert_eq!(snapshot.supported_modes.len(), 3);
        assert!(snapshot.compatibility_notes.contains(&"stable-json".to_string()));
    }

    #[test]
    fn parse_canon_capabilities_ignores_additive_unknown_modes() {
        let stdout = format!(
            r#"{{
            "canon_version": "{}",
            "supported_schema_versions": ["v1"],
            "operations": ["start", "refresh", "capabilities"],
            "supported_modes": ["discovery", "verification", "domain-language", "domain-model"],
            "status_values": ["pending_selection", "running", "governed_ready", "awaiting_approval", "blocked", "completed", "failed"],
            "approval_state_values": ["not_needed", "requested", "granted", "rejected", "expired"],
            "packet_readiness_values": ["pending", "incomplete", "reusable", "rejected"],
            "compatibility_notes": ["stable-json"]
        }}"#,
            crate::domain::distribution::SUPPORTED_CANON_VERSION
        );

        let snapshot = parse_canon_capabilities(stdout.as_bytes()).unwrap();

        assert_eq!(snapshot.canon_version, crate::domain::distribution::SUPPORTED_CANON_VERSION);
        assert_eq!(snapshot.supported_modes, vec![CanonMode::Discovery, CanonMode::Verification]);
        assert_eq!(snapshot.supported_schema_versions, vec!["v1".to_string()]);
    }

    #[test]
    fn query_canon_capabilities_returns_none_for_blank_or_missing_command() {
        let workspace = temp_workspace("canon-capabilities-empty-command");

        assert_eq!(query_canon_capabilities("", &workspace).unwrap(), None);
        assert_eq!(
            query_canon_capabilities("/definitely/missing/canon", &workspace).unwrap(),
            None
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn query_canon_capabilities_reads_cli_output() {
        let workspace = temp_workspace("canon-capabilities-runtime");
        let script = write_shell_script(
            "canon-capabilities-command",
            &format!(
                "#!/bin/sh\nprintf '%s' '{{\"canon_version\":\"{}\",\"supported_schema_versions\":[\"2026-02-01\"],\"operations\":[\"start\",\"refresh\",\"capabilities\"],\"supported_modes\":[\"verification\"],\"status_values\":[\"governed_ready\"],\"approval_state_values\":[\"not_needed\"],\"packet_readiness_values\":[\"reusable\"],\"compatibility_notes\":[\"stable-json\"]}}'\n",
                crate::domain::distribution::SUPPORTED_CANON_VERSION
            ),
        );

        let snapshot = query_canon_capabilities(script.to_string_lossy().as_ref(), &workspace)
            .unwrap()
            .expect("snapshot should be parsed");

        assert_eq!(snapshot.canon_version, crate::domain::distribution::SUPPORTED_CANON_VERSION);
        assert_eq!(snapshot.operations, vec!["start", "refresh", "capabilities"]);

        fs::remove_dir_all(workspace).unwrap();
        fs::remove_dir_all(script.parent().unwrap()).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn query_canon_capabilities_falls_back_to_shell_for_non_executable_script() {
        let workspace = temp_workspace("canon-capabilities-runtime-shell-fallback");
        let script = write_shell_script(
            "canon-capabilities-command-shell-fallback",
            &format!(
                "#!/bin/sh\nprintf '%s' '{{\"canon_version\":\"{}\",\"supported_schema_versions\":[\"2026-02-01\"],\"operations\":[\"start\",\"refresh\",\"capabilities\"],\"supported_modes\":[\"verification\"],\"status_values\":[\"governed_ready\"],\"approval_state_values\":[\"not_needed\"],\"packet_readiness_values\":[\"reusable\"],\"compatibility_notes\":[\"stable-json\"]}}'\n",
                crate::domain::distribution::SUPPORTED_CANON_VERSION
            ),
        );
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&script, permissions).unwrap();

        let snapshot = query_canon_capabilities(script.to_string_lossy().as_ref(), &workspace)
            .unwrap()
            .expect("snapshot should be parsed via shell fallback");

        assert_eq!(snapshot.canon_version, crate::domain::distribution::SUPPORTED_CANON_VERSION);
        assert_eq!(snapshot.operations, vec!["start", "refresh", "capabilities"]);

        fs::remove_dir_all(workspace).unwrap();
        fs::remove_dir_all(script.parent().unwrap()).unwrap();
    }
}
