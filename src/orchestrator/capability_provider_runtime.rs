//! Boundline-owned orchestration helpers for capability-provider registration,
//! activation, and bounded execution.

use std::path::Path;

use thiserror::Error;

use crate::adapters::capability_provider_runtime::{
    CapabilityProviderRuntimeError, collect_evidence, execute_request, fetch_capabilities,
    fetch_health, prepare_execution,
};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::domain::capability_provider::{
    CAPABILITY_PROVIDER_PROTOCOL_LINE_V1, CapabilityProviderActivationState,
    CapabilityProviderProjection, CapabilityProviderRegistration, ProviderCapabilityDeclaration,
    ProviderEvidenceCollectionRecord, ProviderExecutionRequest, ProviderExecutionResult,
    ProviderFailureClass, ProviderHealthSnapshot, ProviderPermissionEnvelope,
    ProviderPreparationReport, ProviderReadinessState, ProviderSetupRequirement,
    ProviderValidationDisposition, ProviderValidationOutcome,
};
use crate::domain::configuration::PersistedCapabilityProviderConfiguration;
use crate::domain::session::CapabilityProviderExecutionRecord;
use crate::domain::trace::{CapabilityProviderTraceRecord, current_timestamp_millis};

/// Outcome returned after one provider-backed request is admitted and validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityProviderExecutionOutcome {
    /// Session-visible execution record.
    pub session_record: CapabilityProviderExecutionRecord,
    /// Trace-visible execution record.
    pub trace_record: CapabilityProviderTraceRecord,
}

/// Errors surfaced by provider registration and execution orchestration.
#[derive(Debug, Error)]
pub enum CapabilityProviderOrchestratorError {
    #[error("provider commands require --workspace")]
    WorkspaceRequired,
    #[error("failed to persist provider configuration: {0}")]
    ConfigStore(#[from] ConfigStoreError),
    #[error("provider runtime failed: {0}")]
    ProviderRuntime(#[from] CapabilityProviderRuntimeError),
    #[error("provider `{provider_id}` is not registered")]
    ProviderNotRegistered { provider_id: String },
    #[error("no active capability provider is configured")]
    NoActiveProvider,
}

/// Upserts a provider registration and attempts activation when setup is
/// complete.
pub fn register_provider(
    workspace: &Path,
    registration: CapabilityProviderRegistration,
) -> Result<CapabilityProviderProjection, CapabilityProviderOrchestratorError> {
    let store = FileConfigStore::for_workspace(workspace);
    let mut config = store.load_local()?.unwrap_or_default();
    let mut provider_config =
        config.capability_provider.take().unwrap_or(PersistedCapabilityProviderConfiguration {
            registrations: Vec::new(),
            active_provider_id: None,
            last_validated_at: None,
        });
    upsert_registration(&mut provider_config, registration.clone());
    let projection = if registration.has_blocking_setup_requirements() {
        let blocked = block_registration(registration, "setup requirements are incomplete");
        apply_registration(&mut provider_config, blocked.clone(), false);
        projection_from_registration(
            &blocked,
            None,
            None,
            blocked.setup_requirements.clone(),
            blocked_reason_summary(&blocked.setup_requirements),
        )
    } else {
        activate_registration(&registration, &mut provider_config)?
    };
    provider_config.last_validated_at = Some(current_timestamp_millis());
    config.capability_provider = Some(provider_config);
    store.save_local(&config)?;
    Ok(projection)
}

/// Removes a provider registration from workspace config.
pub fn remove_provider(
    workspace: &Path,
    provider_id: &str,
) -> Result<bool, CapabilityProviderOrchestratorError> {
    let store = FileConfigStore::for_workspace(workspace);
    let mut config = match store.load_local()? {
        Some(config) => config,
        None => return Ok(false),
    };
    let Some(mut provider_config) = config.capability_provider.take() else {
        return Ok(false);
    };
    let previous_len = provider_config.registrations.len();
    provider_config.registrations.retain(|item| item.provider_id != provider_id);
    if provider_config.active_provider_id.as_deref() == Some(provider_id) {
        provider_config.active_provider_id = None;
    }
    let removed = provider_config.registrations.len() != previous_len;
    config.capability_provider =
        if provider_config.registrations.is_empty() { None } else { Some(provider_config) };
    store.save_local(&config)?;
    Ok(removed)
}

/// Loads the persisted provider configuration for a workspace.
pub fn load_provider_configuration(
    workspace: &Path,
) -> Result<Option<PersistedCapabilityProviderConfiguration>, CapabilityProviderOrchestratorError> {
    Ok(FileConfigStore::for_workspace(workspace).local_capability_provider()?)
}

/// Fetches the latest health snapshot for one provider registration.
pub fn provider_health(
    workspace: &Path,
    provider_id: Option<&str>,
) -> Result<ProviderHealthSnapshot, CapabilityProviderOrchestratorError> {
    let provider_config = load_provider_configuration(workspace)?
        .ok_or(CapabilityProviderOrchestratorError::NoActiveProvider)?;
    let registration = provider_registration_for_request(&provider_config, provider_id)?;
    fetch_health(registration).map_err(Into::into)
}

/// Executes one bounded provider-backed request against the active provider.
pub fn execute_provider(
    workspace: &Path,
    request: &ProviderExecutionRequest,
) -> Result<CapabilityProviderExecutionOutcome, CapabilityProviderOrchestratorError> {
    let provider_config = load_provider_configuration(workspace)?
        .ok_or(CapabilityProviderOrchestratorError::NoActiveProvider)?;
    let registration = active_provider_registration(&provider_config)?;
    let capabilities = fetch_capabilities(registration)?;
    let capability = capability_for_request(&capabilities, &request.capability_id).ok_or(
        CapabilityProviderOrchestratorError::ProviderNotRegistered {
            provider_id: request.capability_id.clone(),
        },
    )?;
    let health = fetch_health(registration)?;
    if matches!(health.readiness_state, ProviderReadinessState::Unavailable) {
        return Ok(blocked_execution_outcome(
            &registration.provider_id,
            registration.activation_state,
            Some(request.capability_id.clone()),
            Some(health.readiness_state),
            ProviderFailureClass::Readiness,
            "provider is unavailable",
            request.request_id.clone(),
        ));
    }
    if !lifecycle_phase_supported(capability, &request.lifecycle_phase)
        || !permissions_satisfied(capability, &request.permission_envelope)
    {
        return Ok(blocked_execution_outcome(
            &registration.provider_id,
            registration.activation_state,
            Some(request.capability_id.clone()),
            Some(health.readiness_state),
            ProviderFailureClass::PermissionAdmission,
            "provider lifecycle support or permission envelope is incompatible",
            request.request_id.clone(),
        ));
    }
    let preparation = prepare_execution(registration, request)?;
    if preparation_blocks_request(&preparation, request) {
        return Ok(blocked_execution_outcome(
            &registration.provider_id,
            registration.activation_state,
            Some(request.capability_id.clone()),
            Some(health.readiness_state),
            ProviderFailureClass::Readiness,
            "prepare reported missing required context or evidence",
            request.request_id.clone(),
        ));
    }
    let execution_result = execute_request(registration, request)?;
    let evidence = collect_evidence(registration, &request.request_id, &execution_result)?;
    Ok(finalize_execution_outcome(
        registration,
        &health,
        request.request_id.clone(),
        request.capability_id.clone(),
        &execution_result,
        &evidence,
    ))
}

fn upsert_registration(
    config: &mut PersistedCapabilityProviderConfiguration,
    registration: CapabilityProviderRegistration,
) {
    if let Some(existing) =
        config.registrations.iter_mut().find(|item| item.provider_id == registration.provider_id)
    {
        *existing = registration;
    } else {
        config.registrations.push(registration);
    }
}

fn apply_registration(
    config: &mut PersistedCapabilityProviderConfiguration,
    registration: CapabilityProviderRegistration,
    active: bool,
) {
    upsert_registration(config, registration.clone());
    if active {
        for item in &mut config.registrations {
            if item.provider_id != registration.provider_id
                && matches!(item.activation_state, CapabilityProviderActivationState::Active)
            {
                item.activation_state = CapabilityProviderActivationState::Inactive;
            }
        }
        config.active_provider_id = Some(registration.provider_id);
    }
}

fn activate_registration(
    registration: &CapabilityProviderRegistration,
    config: &mut PersistedCapabilityProviderConfiguration,
) -> Result<CapabilityProviderProjection, CapabilityProviderOrchestratorError> {
    let declarations = fetch_capabilities(registration)?;
    let health = fetch_health(registration)?;
    let active_registration = if declarations.is_empty()
        || matches!(health.readiness_state, ProviderReadinessState::Unavailable)
    {
        block_registration(
            registration.clone(),
            "provider capabilities are missing or the provider is unavailable",
        )
    } else {
        let mut updated = registration.clone();
        updated.capability_ids =
            declarations.iter().map(|item| item.capability_id.clone()).collect::<Vec<_>>();
        updated.activation_state = CapabilityProviderActivationState::Active;
        updated
    };
    let is_active =
        matches!(active_registration.activation_state, CapabilityProviderActivationState::Active);
    apply_registration(config, active_registration.clone(), is_active);
    let summary = if is_active {
        "provider is active".to_string()
    } else {
        "provider activation is blocked".to_string()
    };
    Ok(projection_from_registration(
        &active_registration,
        None,
        Some(health.readiness_state),
        active_registration.setup_requirements.clone(),
        summary,
    ))
}

fn block_registration(
    mut registration: CapabilityProviderRegistration,
    summary: &str,
) -> CapabilityProviderRegistration {
    let _ = summary;
    registration.activation_state = CapabilityProviderActivationState::Blocked;
    registration
}

fn blocked_reason_summary(setup_requirements: &[ProviderSetupRequirement]) -> String {
    if setup_requirements.is_empty() {
        "provider activation is blocked".to_string()
    } else {
        format!(
            "missing setup requirements: {}",
            setup_requirements
                .iter()
                .filter(|item| item.blocks_activation())
                .map(|item| item.display_label.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn active_provider_registration(
    provider_config: &PersistedCapabilityProviderConfiguration,
) -> Result<&CapabilityProviderRegistration, CapabilityProviderOrchestratorError> {
    let Some(active_provider_id) = provider_config.active_provider_id.as_deref() else {
        return Err(CapabilityProviderOrchestratorError::NoActiveProvider);
    };
    provider_registration_for_request(provider_config, Some(active_provider_id))
}

fn provider_registration_for_request<'a>(
    provider_config: &'a PersistedCapabilityProviderConfiguration,
    provider_id: Option<&str>,
) -> Result<&'a CapabilityProviderRegistration, CapabilityProviderOrchestratorError> {
    let requested = provider_id
        .or(provider_config.active_provider_id.as_deref())
        .ok_or(CapabilityProviderOrchestratorError::NoActiveProvider)?;
    provider_config.registrations.iter().find(|item| item.provider_id == requested).ok_or_else(
        || CapabilityProviderOrchestratorError::ProviderNotRegistered {
            provider_id: requested.to_string(),
        },
    )
}

fn capability_for_request<'a>(
    declarations: &'a [ProviderCapabilityDeclaration],
    capability_id: &str,
) -> Option<&'a ProviderCapabilityDeclaration> {
    declarations.iter().find(|item| {
        item.capability_id == capability_id
            && item.protocol_line == CAPABILITY_PROVIDER_PROTOCOL_LINE_V1
    })
}

fn lifecycle_phase_supported(
    capability: &ProviderCapabilityDeclaration,
    lifecycle_phase: &str,
) -> bool {
    capability.supported_lifecycle_phases.iter().any(|phase| phase == lifecycle_phase)
}

fn permissions_satisfied(
    capability: &ProviderCapabilityDeclaration,
    envelope: &ProviderPermissionEnvelope,
) -> bool {
    capability
        .required_permissions
        .iter()
        .all(|permission| permission_allowed(permission, envelope))
}

fn permission_allowed(permission: &str, envelope: &ProviderPermissionEnvelope) -> bool {
    match permission {
        "read_files" => envelope.read_files,
        "write_files" => envelope.write_files,
        "run_commands" => envelope.run_commands,
        "network" => envelope.network,
        "read_secrets" => envelope.read_secrets,
        "write_artifacts" => envelope.write_artifacts,
        _ => false,
    }
}

fn preparation_blocks_request(
    preparation: &ProviderPreparationReport,
    request: &ProviderExecutionRequest,
) -> bool {
    preparation
        .required_context_refs
        .iter()
        .any(|required| !request.context_pack_refs.iter().any(|item| item == required))
        || !preparation.missing_evidence_refs.is_empty()
}

fn finalize_execution_outcome(
    registration: &CapabilityProviderRegistration,
    health: &ProviderHealthSnapshot,
    request_id: String,
    capability_id: String,
    execution_result: &ProviderExecutionResult,
    evidence: &ProviderEvidenceCollectionRecord,
) -> CapabilityProviderExecutionOutcome {
    let validation = validation_disposition(request_id.clone(), execution_result, evidence);
    let projection = CapabilityProviderProjection {
        provider_id: registration.provider_id.clone(),
        activation_state: registration.activation_state,
        capability_id: Some(capability_id),
        readiness_state: Some(health.readiness_state),
        validation_disposition: Some(validation.disposition),
        failure_class: validation.failure_class,
        setup_requirements: registration.setup_requirements.clone(),
        accepted_evidence_refs: validation.accepted_evidence_refs.clone(),
        rejected_evidence_refs: validation.rejected_evidence_refs.clone(),
        limitations: evidence.limitations.clone(),
        summary: validation.reason.clone(),
    };
    let session_record = CapabilityProviderExecutionRecord {
        request_id: request_id.clone(),
        projection: projection.clone(),
    };
    let trace_record = CapabilityProviderTraceRecord { request_id, projection, validation };
    CapabilityProviderExecutionOutcome { session_record, trace_record }
}

fn validation_disposition(
    request_id: String,
    execution_result: &ProviderExecutionResult,
    evidence: &ProviderEvidenceCollectionRecord,
) -> ProviderValidationDisposition {
    if matches!(
        execution_result.status,
        crate::domain::capability_provider::ProviderExecutionStatus::Failed
    ) {
        return ProviderValidationDisposition {
            request_id,
            disposition: ProviderValidationOutcome::Rejected,
            failure_class: Some(ProviderFailureClass::Execution),
            accepted_evidence_refs: Vec::new(),
            rejected_evidence_refs: evidence.evidence_refs.clone(),
            reason: "provider execution failed".to_string(),
        };
    }
    if evidence.evidence_refs.is_empty() {
        return ProviderValidationDisposition {
            request_id,
            disposition: ProviderValidationOutcome::Rejected,
            failure_class: Some(ProviderFailureClass::PostExecutionValidation),
            accepted_evidence_refs: Vec::new(),
            rejected_evidence_refs: execution_result.evidence_refs.clone(),
            reason: "provider returned no reproducible evidence".to_string(),
        };
    }
    if !execution_result.state_patch_proposals.is_empty() {
        return ProviderValidationDisposition {
            request_id,
            disposition: ProviderValidationOutcome::Rejected,
            failure_class: Some(ProviderFailureClass::PostExecutionValidation),
            accepted_evidence_refs: Vec::new(),
            rejected_evidence_refs: evidence.evidence_refs.clone(),
            reason: "provider patch proposals require host validation before acceptance"
                .to_string(),
        };
    }
    let disposition = if matches!(
        execution_result.status,
        crate::domain::capability_provider::ProviderExecutionStatus::Partial
    ) {
        ProviderValidationOutcome::Degraded
    } else {
        ProviderValidationOutcome::Accepted
    };
    ProviderValidationDisposition {
        request_id,
        disposition,
        failure_class: None,
        accepted_evidence_refs: evidence.evidence_refs.clone(),
        rejected_evidence_refs: Vec::new(),
        reason: if matches!(disposition, ProviderValidationOutcome::Accepted) {
            "provider evidence accepted".to_string()
        } else {
            "provider evidence accepted with degradation".to_string()
        },
    }
}

fn blocked_execution_outcome(
    provider_id: &str,
    activation_state: CapabilityProviderActivationState,
    capability_id: Option<String>,
    readiness_state: Option<ProviderReadinessState>,
    failure_class: ProviderFailureClass,
    summary: &str,
    request_id: String,
) -> CapabilityProviderExecutionOutcome {
    let validation = ProviderValidationDisposition {
        request_id: request_id.clone(),
        disposition: ProviderValidationOutcome::Blocked,
        failure_class: Some(failure_class),
        accepted_evidence_refs: Vec::new(),
        rejected_evidence_refs: Vec::new(),
        reason: summary.to_string(),
    };
    let projection = CapabilityProviderProjection {
        provider_id: provider_id.to_string(),
        activation_state,
        capability_id,
        readiness_state,
        validation_disposition: Some(ProviderValidationOutcome::Blocked),
        failure_class: Some(failure_class),
        setup_requirements: Vec::new(),
        accepted_evidence_refs: Vec::new(),
        rejected_evidence_refs: Vec::new(),
        limitations: Vec::new(),
        summary: summary.to_string(),
    };
    let session_record = CapabilityProviderExecutionRecord {
        request_id: request_id.clone(),
        projection: projection.clone(),
    };
    let trace_record = CapabilityProviderTraceRecord { request_id, projection, validation };
    CapabilityProviderExecutionOutcome { session_record, trace_record }
}

fn projection_from_registration(
    registration: &CapabilityProviderRegistration,
    capability_id: Option<String>,
    readiness_state: Option<ProviderReadinessState>,
    setup_requirements: Vec<ProviderSetupRequirement>,
    summary: String,
) -> CapabilityProviderProjection {
    CapabilityProviderProjection {
        provider_id: registration.provider_id.clone(),
        activation_state: registration.activation_state,
        capability_id,
        readiness_state,
        validation_disposition: None,
        failure_class: None,
        setup_requirements,
        accepted_evidence_refs: Vec::new(),
        rejected_evidence_refs: Vec::new(),
        limitations: Vec::new(),
        summary,
    }
}
