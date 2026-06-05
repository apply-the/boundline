//! Shared vocabulary for external capability-provider registrations,
//! execution, evidence, and validation.

use serde::{Deserialize, Serialize};

/// Stable protocol line identifier for the initial capability-provider
/// contract.
pub const CAPABILITY_PROVIDER_PROTOCOL_LINE_V1: &str = "capability-provider-v1";

/// Transport kinds supported by the generic provider protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderTransportKind {
    /// Local executable reached through a bounded subprocess call.
    Command,
    /// Remote endpoint reached through the existing HTTP client path.
    Http,
}

impl CapabilityProviderTransportKind {
    /// Returns the stable serialized identifier for the transport kind.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Http => "http",
        }
    }
}

/// Source that created or migrated one provider registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderRegistrationSource {
    /// Created by an explicit CLI registration flow.
    OperatorCli,
    /// Created by a guided setup helper.
    GuidedSetup,
    /// Migrated from an older config shape.
    MigratedConfig,
}

/// Discovery state recorded for a provider registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderDiscoveryState {
    /// The operator explicitly supplied the transport details.
    Explicit,
    /// The runtime found a candidate transport but did not trust it yet.
    Discovered,
    /// The registration exists but its transport is currently unresolved.
    Unresolved,
}

/// Activation state recorded for a provider registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityProviderActivationState {
    /// Registered but not selected for execution.
    Inactive,
    /// Activation is in progress and has not been committed yet.
    Activating,
    /// Fully validated and eligible for admission.
    Active,
    /// Activation or admission is blocked until the operator repairs input.
    Blocked,
    /// The registration metadata is malformed or incompatible.
    Invalid,
}

impl CapabilityProviderActivationState {
    /// Returns the stable serialized identifier for the activation state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inactive => "inactive",
            Self::Activating => "activating",
            Self::Active => "active",
            Self::Blocked => "blocked",
            Self::Invalid => "invalid",
        }
    }
}

/// Setup requirement families surfaced before activation can complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSetupRequirementKind {
    /// Non-secret config value persisted in workspace config.
    ConfigValue,
    /// Secret handle ref resolved through auth-profile storage.
    SecretHandle,
    /// Relative filesystem reference required by the provider.
    FilesystemRef,
    /// Health or connectivity check the runtime must verify.
    ConnectivityCheck,
}

/// Requiredness state for one setup requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSetupRequiredState {
    /// Missing state blocks activation.
    Required,
    /// Missing state is informational only.
    Optional,
}

/// Resolution state for one setup requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSetupResolutionState {
    /// The requirement is satisfied.
    Present,
    /// The requirement is missing.
    Missing,
    /// The requirement exists but is invalid.
    Invalid,
    /// The requirement was not evaluated yet.
    Unchecked,
}

/// One operator-visible setup requirement projected before activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSetupRequirement {
    /// Stable identifier unique within one provider registration.
    pub requirement_id: String,
    /// Logical requirement family.
    pub kind: ProviderSetupRequirementKind,
    /// Whether the requirement blocks activation.
    pub required_state: ProviderSetupRequiredState,
    /// Current resolution state for the requirement.
    pub resolution_state: ProviderSetupResolutionState,
    /// Human-readable label that never exposes secret values.
    pub display_label: String,
    /// Optional persisted config or handle reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

impl ProviderSetupRequirement {
    /// Returns true when this requirement blocks activation in its current state.
    pub const fn blocks_activation(&self) -> bool {
        matches!(self.required_state, ProviderSetupRequiredState::Required)
            && !matches!(self.resolution_state, ProviderSetupResolutionState::Present)
    }
}

/// Command transport descriptor for a provider registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandProviderTransport {
    /// Relative or operator-approved executable reference.
    pub command_ref: String,
    /// Additional launch arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Optional working-directory ref.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_directory_ref: Option<String>,
    /// Names of env-backed handle refs the runtime may resolve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environment_ref_names: Vec<String>,
}

/// HTTP transport descriptor for a provider registration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpProviderTransport {
    /// Endpoint reference without embedded secret material.
    pub endpoint_ref: String,
    /// Optional bounded auth scheme label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_scheme: Option<String>,
    /// Non-secret header references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers_ref: Vec<String>,
    /// Optional TLS policy label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_policy: Option<String>,
}

/// Transport descriptor used to contact a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport_kind", rename_all = "snake_case")]
pub enum ProviderTransportDescriptor {
    /// Local command/stdio transport.
    Command(CommandProviderTransport),
    /// HTTP endpoint transport.
    Http(HttpProviderTransport),
}

impl ProviderTransportDescriptor {
    /// Returns the stable transport kind for this descriptor.
    pub const fn transport_kind(&self) -> CapabilityProviderTransportKind {
        match self {
            Self::Command(_) => CapabilityProviderTransportKind::Command,
            Self::Http(_) => CapabilityProviderTransportKind::Http,
        }
    }
}

/// Operator-approved registration that Boundline may later activate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProviderRegistration {
    /// Stable workspace-scoped provider identifier.
    pub provider_id: String,
    /// Human-facing display name.
    pub display_name: String,
    /// Transport used to contact the provider.
    pub transport: ProviderTransportDescriptor,
    /// Registration provenance.
    pub registration_source: CapabilityProviderRegistrationSource,
    /// Discovery status recorded at registration time.
    pub discovery_state: CapabilityProviderDiscoveryState,
    /// Activation status recorded by the runtime.
    pub activation_state: CapabilityProviderActivationState,
    /// Non-secret config refs stored for the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_refs: Vec<String>,
    /// Opaque secret handle refs stored for the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secret_handle_refs: Vec<String>,
    /// Setup requirements projected before activation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub setup_requirements: Vec<ProviderSetupRequirement>,
    /// Stable capability identifiers known for this provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_ids: Vec<String>,
    /// Optional specialized execution profile overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_profile_id: Option<String>,
}

impl CapabilityProviderRegistration {
    /// Returns true when the registration still has unresolved blocking setup.
    pub fn has_blocking_setup_requirements(&self) -> bool {
        self.setup_requirements.iter().any(ProviderSetupRequirement::blocks_activation)
    }
}

/// Provider-declared lifecycle support.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilityDeclaration {
    /// Stable provider identifier expected by the runtime.
    pub provider_id: String,
    /// Stable protocol line identifier.
    pub protocol_line: String,
    /// Provider protocol version string.
    pub protocol_version: String,
    /// Capability identifier unique within the provider.
    pub capability_id: String,
    /// Lifecycle phases this capability supports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_lifecycle_phases: Vec<String>,
    /// Typed input families the capability accepts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_inputs: Vec<String>,
    /// Typed output families the capability can return.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported_outputs: Vec<String>,
    /// Declared mutation support level.
    pub mutation_support: ProviderMutationSupport,
    /// Declared permission requirements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_permissions: Vec<String>,
    /// Evidence formats the provider can emit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_formats: Vec<String>,
}

/// Declared mutation support level for a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMutationSupport {
    /// Capability never proposes mutations.
    ReadOnly,
    /// Capability may propose patches, but never apply them directly.
    ProposalOnly,
    /// Capability claims mutating ability, but host validation still applies.
    Mutating,
}

/// Provider readiness state returned by `health`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderReadinessState {
    /// Provider is ready for admission.
    Ready,
    /// Provider may still run with warnings or explicit degradation.
    Degraded,
    /// Provider is unavailable and must be blocked.
    Unavailable,
}

/// Latest readiness snapshot recorded for a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderHealthSnapshot {
    /// Stable provider identifier.
    pub provider_id: String,
    /// Readiness outcome.
    pub readiness_state: ProviderReadinessState,
    /// Missing dependencies or missing external services.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_dependencies: Vec<String>,
    /// Operator-visible warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    /// Compact runtime-environment summary.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_environment: Vec<String>,
    /// Timestamp of the latest health check.
    pub checked_at: u64,
}

/// Pre-execution report returned by `prepare`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPreparationReport {
    /// Stable request identifier shared across the lifecycle.
    pub request_id: String,
    /// Required context references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_context_refs: Vec<String>,
    /// Optional context references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub optional_context_refs: Vec<String>,
    /// Missing evidence references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_evidence_refs: Vec<String>,
    /// Expected artifacts the provider plans to emit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_artifacts: Vec<String>,
    /// Non-authoritative provider-supplied risk observations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_observations: Vec<String>,
    /// Optional cost or runtime estimate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_or_runtime: Option<String>,
}

/// Explicit least-privilege execution envelope attached to one request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPermissionEnvelope {
    /// Whether file reads are permitted.
    pub read_files: bool,
    /// Whether file writes are permitted.
    pub write_files: bool,
    /// Whether command execution is permitted.
    pub run_commands: bool,
    /// Whether network access is permitted.
    pub network: bool,
    /// Whether secret-handle resolution is permitted.
    pub read_secrets: bool,
    /// Whether provider-generated artifacts may be persisted.
    pub write_artifacts: bool,
    /// Allowed path refs for this request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_paths: Vec<String>,
    /// Maximum runtime budget in milliseconds.
    pub max_runtime_ms: u64,
    /// Maximum output budget in bytes.
    pub max_output_bytes: u64,
}

/// One bounded provider execution request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionRequest {
    /// Stable request identifier shared across the lifecycle.
    pub request_id: String,
    /// Session reference used for traceability.
    pub session_ref: String,
    /// Stable step or stage identifier.
    pub step_or_stage_ref: String,
    /// Capability being invoked.
    pub capability_id: String,
    /// Goal summary for the bounded request.
    pub goal_summary: String,
    /// Lifecycle phase associated with the request.
    pub lifecycle_phase: String,
    /// Authority zone under which the request runs.
    pub authority_zone: String,
    /// Context pack refs used by the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_pack_refs: Vec<String>,
    /// Least-privilege execution envelope.
    pub permission_envelope: ProviderPermissionEnvelope,
    /// Expected output families.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_outputs: Vec<String>,
}

/// Result returned by `execute`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExecutionResult {
    /// Stable request identifier.
    pub request_id: String,
    /// Execution outcome status.
    pub status: ProviderExecutionStatus,
    /// Non-authoritative runtime observations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<String>,
    /// Provider findings treated as claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<String>,
    /// Artifact refs returned by the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
    /// Evidence refs returned by the provider.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Proposed state patches that still require validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub state_patch_proposals: Vec<String>,
    /// Provider-declared limitations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Suggested next actions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub next_actions: Vec<String>,
}

/// Structured execution outcome returned by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExecutionStatus {
    /// The provider completed successfully.
    Succeeded,
    /// The provider blocked before finishing.
    Blocked,
    /// The provider failed irrecoverably.
    Failed,
    /// The provider returned a degraded partial result.
    Partial,
}

/// Normalized evidence record returned by `collect_evidence`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderEvidenceCollectionRecord {
    /// Stable request identifier.
    pub request_id: String,
    /// Normalized provider claims.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claims: Vec<String>,
    /// Evidence refs preserved for validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    /// Supporting artifacts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
    /// Additional findings carried forward from execution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<String>,
    /// Provider limitations preserved for inspection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Compact reproducibility metadata that supports replay or audit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reproducibility_metadata: Vec<String>,
}

/// Failure classes surfaced when provider admission or validation fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFailureClass {
    /// Failure happened before execution because readiness was insufficient.
    Readiness,
    /// Failure happened while validating the permission envelope.
    PermissionAdmission,
    /// Failure happened inside execute or transport execution.
    Execution,
    /// Failure happened during post-execution validation.
    PostExecutionValidation,
}

impl ProviderFailureClass {
    /// Returns the stable serialized identifier for the failure class.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Readiness => "readiness",
            Self::PermissionAdmission => "permission_admission",
            Self::Execution => "execution",
            Self::PostExecutionValidation => "post_execution_validation",
        }
    }
}

/// Final validation disposition recorded by Boundline for provider output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderValidationDisposition {
    /// Stable request identifier.
    pub request_id: String,
    /// Final host-owned validation disposition.
    pub disposition: ProviderValidationOutcome,
    /// Failure class when the result was not fully accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<ProviderFailureClass>,
    /// Evidence refs that were accepted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_evidence_refs: Vec<String>,
    /// Evidence refs that were explicitly rejected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_evidence_refs: Vec<String>,
    /// Operator-visible disposition summary.
    pub reason: String,
}

/// Final host-owned validation outcome for provider output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderValidationOutcome {
    /// Output was accepted.
    Accepted,
    /// Output was rejected.
    Rejected,
    /// Output was blocked before final acceptance.
    Blocked,
    /// Output was partially accepted with explicit degradation.
    Degraded,
}

impl ProviderValidationOutcome {
    /// Returns the stable serialized identifier for the validation outcome.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Blocked => "blocked",
            Self::Degraded => "degraded",
        }
    }
}

/// Optional profile overlay mapping generic provider capabilities to stage
/// semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecializedExecutionProfile {
    /// Stable profile identifier.
    pub profile_id: String,
    /// Provider this profile targets.
    pub provider_id: String,
    /// Stage or hook mappings derived from the provider capability list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_mappings: Vec<String>,
    /// Version of the overlay profile.
    pub profile_version: String,
    /// Conflict policy label preserved for inspection.
    pub conflict_policy: String,
}

/// Compact additive provider projection surfaced in session, status, inspect,
/// and host output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProviderProjection {
    /// Provider identifier used or currently selected.
    pub provider_id: String,
    /// Activation state visible to the operator.
    pub activation_state: CapabilityProviderActivationState,
    /// Selected capability when a request reached admission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
    /// Latest readiness state, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness_state: Option<ProviderReadinessState>,
    /// Final validation disposition, when a request ran far enough to produce one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_disposition: Option<ProviderValidationOutcome>,
    /// Failure class, when relevant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<ProviderFailureClass>,
    /// Remaining setup requirements that still need operator work.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub setup_requirements: Vec<ProviderSetupRequirement>,
    /// Accepted evidence refs visible to operators and assistant assets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_evidence_refs: Vec<String>,
    /// Rejected evidence refs visible to operators and assistant assets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_evidence_refs: Vec<String>,
    /// Provider-declared limitations preserved for inspection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Operator-visible explanation of the latest provider state.
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::{
        CAPABILITY_PROVIDER_PROTOCOL_LINE_V1, CapabilityProviderActivationState,
        CapabilityProviderDiscoveryState, CapabilityProviderRegistration,
        CapabilityProviderRegistrationSource, CapabilityProviderTransportKind,
        CommandProviderTransport, HttpProviderTransport, ProviderFailureClass,
        ProviderSetupRequiredState, ProviderSetupRequirement, ProviderSetupRequirementKind,
        ProviderSetupResolutionState, ProviderTransportDescriptor, ProviderValidationOutcome,
    };

    #[test]
    fn provider_vocabulary_helpers_cover_transport_and_outcome_strings() {
        assert_eq!(CapabilityProviderTransportKind::Command.as_str(), "command");
        assert_eq!(CapabilityProviderTransportKind::Http.as_str(), "http");
        assert_eq!(CapabilityProviderActivationState::Inactive.as_str(), "inactive");
        assert_eq!(CapabilityProviderActivationState::Activating.as_str(), "activating");
        assert_eq!(CapabilityProviderActivationState::Active.as_str(), "active");
        assert_eq!(CapabilityProviderActivationState::Blocked.as_str(), "blocked");
        assert_eq!(CapabilityProviderActivationState::Invalid.as_str(), "invalid");
        assert_eq!(ProviderFailureClass::Readiness.as_str(), "readiness");
        assert_eq!(ProviderFailureClass::PermissionAdmission.as_str(), "permission_admission");
        assert_eq!(ProviderFailureClass::Execution.as_str(), "execution");
        assert_eq!(
            ProviderFailureClass::PostExecutionValidation.as_str(),
            "post_execution_validation"
        );
        assert_eq!(ProviderValidationOutcome::Accepted.as_str(), "accepted");
        assert_eq!(ProviderValidationOutcome::Rejected.as_str(), "rejected");
        assert_eq!(ProviderValidationOutcome::Blocked.as_str(), "blocked");
        assert_eq!(ProviderValidationOutcome::Degraded.as_str(), "degraded");
        assert_eq!(CAPABILITY_PROVIDER_PROTOCOL_LINE_V1, "capability-provider-v1");
    }

    #[test]
    fn registration_helpers_cover_blocking_setup_and_transport_kind() {
        let blocking_requirement = ProviderSetupRequirement {
            requirement_id: "config-token".to_string(),
            kind: ProviderSetupRequirementKind::ConfigValue,
            required_state: ProviderSetupRequiredState::Required,
            resolution_state: ProviderSetupResolutionState::Missing,
            display_label: "token".to_string(),
            source_ref: None,
        };
        let optional_requirement = ProviderSetupRequirement {
            requirement_id: "secret-handle".to_string(),
            kind: ProviderSetupRequirementKind::SecretHandle,
            required_state: ProviderSetupRequiredState::Optional,
            resolution_state: ProviderSetupResolutionState::Invalid,
            display_label: "provider-secret".to_string(),
            source_ref: Some("profile/provider-secret".to_string()),
        };
        assert!(blocking_requirement.blocks_activation());
        assert!(!optional_requirement.blocks_activation());

        let command_registration = CapabilityProviderRegistration {
            provider_id: "demo-command".to_string(),
            display_name: "Demo Command".to_string(),
            transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
                command_ref: "/bin/echo".to_string(),
                args: Vec::new(),
                working_directory_ref: None,
                environment_ref_names: Vec::new(),
            }),
            registration_source: CapabilityProviderRegistrationSource::OperatorCli,
            discovery_state: CapabilityProviderDiscoveryState::Explicit,
            activation_state: CapabilityProviderActivationState::Inactive,
            config_refs: Vec::new(),
            secret_handle_refs: Vec::new(),
            setup_requirements: vec![blocking_requirement.clone(), optional_requirement.clone()],
            capability_ids: Vec::new(),
            active_profile_id: None,
        };
        let http_registration = CapabilityProviderRegistration {
            provider_id: "demo-http".to_string(),
            display_name: "Demo Http".to_string(),
            transport: ProviderTransportDescriptor::Http(HttpProviderTransport {
                endpoint_ref: "http://127.0.0.1:9/provider".to_string(),
                auth_scheme: None,
                headers_ref: Vec::new(),
                tls_policy: None,
            }),
            registration_source: CapabilityProviderRegistrationSource::GuidedSetup,
            discovery_state: CapabilityProviderDiscoveryState::Discovered,
            activation_state: CapabilityProviderActivationState::Blocked,
            config_refs: vec!["endpoint=http://127.0.0.1:9/provider".to_string()],
            secret_handle_refs: Vec::new(),
            setup_requirements: vec![optional_requirement],
            capability_ids: vec!["capability.demo".to_string()],
            active_profile_id: None,
        };

        assert!(command_registration.has_blocking_setup_requirements());
        assert!(!http_registration.has_blocking_setup_requirements());
        assert_eq!(
            command_registration.transport.transport_kind(),
            CapabilityProviderTransportKind::Command
        );
        assert_eq!(
            http_registration.transport.transport_kind(),
            CapabilityProviderTransportKind::Http
        );
    }
}
