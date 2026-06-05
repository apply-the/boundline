//! Transport-neutral execution helpers for external capability providers.

#[path = "capability_provider_runtime/command.rs"]
mod command;
#[path = "capability_provider_runtime/http.rs"]
mod http;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::capability_provider::{
    CapabilityProviderRegistration, ProviderCapabilityDeclaration,
    ProviderEvidenceCollectionRecord, ProviderExecutionRequest, ProviderExecutionResult,
    ProviderHealthSnapshot, ProviderPreparationReport, ProviderTransportDescriptor,
};

const CAPABILITIES_OPERATION: &str = "capabilities";
const HEALTH_OPERATION: &str = "health";
const PREPARE_OPERATION: &str = "prepare";
const EXECUTE_OPERATION: &str = "execute";
const COLLECT_EVIDENCE_OPERATION: &str = "collect_evidence";

/// Typed request envelope shared by command and HTTP transports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
enum ProviderRequestEnvelope {
    Capabilities,
    Health,
    Prepare { request: ProviderExecutionRequest },
    Execute { request: ProviderExecutionRequest },
    CollectEvidence { request_id: String, execution_result: ProviderExecutionResult },
}

impl ProviderRequestEnvelope {
    const fn operation_name(&self) -> &'static str {
        match self {
            Self::Capabilities => CAPABILITIES_OPERATION,
            Self::Health => HEALTH_OPERATION,
            Self::Prepare { .. } => PREPARE_OPERATION,
            Self::Execute { .. } => EXECUTE_OPERATION,
            Self::CollectEvidence { .. } => COLLECT_EVIDENCE_OPERATION,
        }
    }
}

/// Response body for `capabilities`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderCapabilitiesEnvelope {
    declarations: Vec<ProviderCapabilityDeclaration>,
}

/// Errors surfaced by the generic provider transport layer.
#[derive(Debug, Error)]
pub enum CapabilityProviderRuntimeError {
    #[error("provider `{provider_id}` is missing transport metadata")]
    MissingTransportMetadata { provider_id: String },
    #[error("provider `{provider_id}` runtime failed: {message}")]
    Runtime { provider_id: String, message: String },
}

/// Fetches provider capability declarations.
pub fn fetch_capabilities(
    registration: &CapabilityProviderRegistration,
) -> Result<Vec<ProviderCapabilityDeclaration>, CapabilityProviderRuntimeError> {
    let response = execute_transport_call::<ProviderCapabilitiesEnvelope>(
        registration,
        &ProviderRequestEnvelope::Capabilities,
    )?;
    Ok(response.declarations)
}

/// Fetches the latest provider health snapshot.
pub fn fetch_health(
    registration: &CapabilityProviderRegistration,
) -> Result<ProviderHealthSnapshot, CapabilityProviderRuntimeError> {
    execute_transport_call(registration, &ProviderRequestEnvelope::Health)
}

/// Runs the provider `prepare` lifecycle step.
pub fn prepare_execution(
    registration: &CapabilityProviderRegistration,
    request: &ProviderExecutionRequest,
) -> Result<ProviderPreparationReport, CapabilityProviderRuntimeError> {
    execute_transport_call(
        registration,
        &ProviderRequestEnvelope::Prepare { request: request.clone() },
    )
}

/// Runs the provider `execute` lifecycle step.
pub fn execute_request(
    registration: &CapabilityProviderRegistration,
    request: &ProviderExecutionRequest,
) -> Result<ProviderExecutionResult, CapabilityProviderRuntimeError> {
    execute_transport_call(
        registration,
        &ProviderRequestEnvelope::Execute { request: request.clone() },
    )
}

/// Runs the provider `collect_evidence` lifecycle step.
pub fn collect_evidence(
    registration: &CapabilityProviderRegistration,
    request_id: &str,
    execution_result: &ProviderExecutionResult,
) -> Result<ProviderEvidenceCollectionRecord, CapabilityProviderRuntimeError> {
    execute_transport_call(
        registration,
        &ProviderRequestEnvelope::CollectEvidence {
            request_id: request_id.to_string(),
            execution_result: execution_result.clone(),
        },
    )
}

fn execute_transport_call<T: for<'de> Deserialize<'de>>(
    registration: &CapabilityProviderRegistration,
    envelope: &ProviderRequestEnvelope,
) -> Result<T, CapabilityProviderRuntimeError> {
    let provider_id = registration.provider_id.clone();
    match &registration.transport {
        ProviderTransportDescriptor::Command(transport) => {
            command::execute_command_call(transport, envelope)
                .map_err(|message| CapabilityProviderRuntimeError::Runtime { provider_id, message })
        }
        ProviderTransportDescriptor::Http(transport) => {
            http::execute_http_call(transport, envelope)
                .map_err(|message| CapabilityProviderRuntimeError::Runtime { provider_id, message })
        }
    }
}
