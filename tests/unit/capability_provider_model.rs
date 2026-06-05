use boundline::domain::capability_provider::{
    CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
    CapabilityProviderRegistration, CapabilityProviderRegistrationSource, CommandProviderTransport,
    ProviderSetupRequiredState, ProviderSetupRequirement, ProviderSetupRequirementKind,
    ProviderSetupResolutionState, ProviderTransportDescriptor,
};

fn sample_registration(
    setup_requirements: Vec<ProviderSetupRequirement>,
) -> CapabilityProviderRegistration {
    CapabilityProviderRegistration {
        provider_id: "demo-provider".to_string(),
        display_name: "Demo Provider".to_string(),
        transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
            command_ref: "python3".to_string(),
            args: Vec::new(),
            working_directory_ref: None,
            environment_ref_names: Vec::new(),
        }),
        registration_source: CapabilityProviderRegistrationSource::OperatorCli,
        discovery_state: CapabilityProviderDiscoveryState::Explicit,
        activation_state: CapabilityProviderActivationState::Inactive,
        config_refs: Vec::new(),
        secret_handle_refs: Vec::new(),
        setup_requirements,
        capability_ids: Vec::new(),
        active_profile_id: None,
    }
}

#[test]
fn setup_requirement_blocks_activation_only_when_required_and_missing() {
    let missing_required = ProviderSetupRequirement {
        requirement_id: "secret-api".to_string(),
        kind: ProviderSetupRequirementKind::SecretHandle,
        required_state: ProviderSetupRequiredState::Required,
        resolution_state: ProviderSetupResolutionState::Missing,
        display_label: "api_token".to_string(),
        source_ref: None,
    };
    let optional_missing = ProviderSetupRequirement {
        requirement_id: "optional-config".to_string(),
        kind: ProviderSetupRequirementKind::ConfigValue,
        required_state: ProviderSetupRequiredState::Optional,
        resolution_state: ProviderSetupResolutionState::Missing,
        display_label: "notes".to_string(),
        source_ref: None,
    };

    assert!(missing_required.blocks_activation());
    assert!(!optional_missing.blocks_activation());
}

#[test]
fn registration_detects_blocking_setup_requirements() {
    let registration = sample_registration(vec![ProviderSetupRequirement {
        requirement_id: "config-project".to_string(),
        kind: ProviderSetupRequirementKind::ConfigValue,
        required_state: ProviderSetupRequiredState::Required,
        resolution_state: ProviderSetupResolutionState::Invalid,
        display_label: "project_id".to_string(),
        source_ref: Some("project_id=demo".to_string()),
    }]);

    assert!(registration.has_blocking_setup_requirements());
}

#[test]
fn registration_ignores_fully_satisfied_setup_requirements() {
    let registration = sample_registration(vec![ProviderSetupRequirement {
        requirement_id: "config-project".to_string(),
        kind: ProviderSetupRequirementKind::ConfigValue,
        required_state: ProviderSetupRequiredState::Required,
        resolution_state: ProviderSetupResolutionState::Present,
        display_label: "project_id".to_string(),
        source_ref: Some("project_id=demo".to_string()),
    }]);

    assert!(!registration.has_blocking_setup_requirements());
}
