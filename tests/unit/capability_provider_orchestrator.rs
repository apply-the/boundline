use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use boundline::ConfigFile;
use boundline::FileConfigStore;
use boundline::domain::capability_provider::{
    CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
    CapabilityProviderRegistration, CapabilityProviderRegistrationSource, CommandProviderTransport,
    ProviderExecutionRequest, ProviderFailureClass, ProviderPermissionEnvelope,
    ProviderTransportDescriptor, ProviderValidationOutcome,
};
use boundline::orchestrator::capability_provider_runtime::{
    CapabilityProviderOrchestratorError, execute_provider, load_provider_configuration,
    provider_health, register_provider, remove_provider,
};
use uuid::Uuid;

#[test]
fn orchestrator_registration_and_removal_cover_activation_edges() -> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("provider-orchestrator-registration");
    FileConfigStore::for_workspace(&workspace).save_local(&ConfigFile::default())?;
    assert!(!remove_provider(&workspace, "missing-provider")?);

    let first = register_provider(
        &workspace,
        scripted_registration(
            "demo-provider",
            write_provider_script(
                "provider-one",
                active_provider_script("demo-provider", "capability.one"),
            )?,
        ),
    )?;
    assert_eq!(first.activation_state, CapabilityProviderActivationState::Active);

    let second = register_provider(
        &workspace,
        scripted_registration(
            "backup-provider",
            write_provider_script(
                "provider-two",
                active_provider_script("backup-provider", "capability.two"),
            )?,
        ),
    )?;
    assert_eq!(second.activation_state, CapabilityProviderActivationState::Active);

    let configuration =
        load_provider_configuration(&workspace)?.ok_or("missing provider configuration")?;
    assert!(configuration.registrations.iter().any(|item| {
        item.provider_id == "demo-provider"
            && item.activation_state == CapabilityProviderActivationState::Inactive
    }));

    let unknown = provider_health(&workspace, Some("unknown-provider"))
        .expect_err("unknown provider should fail");
    assert!(matches!(unknown, CapabilityProviderOrchestratorError::ProviderNotRegistered { .. }));

    assert!(remove_provider(&workspace, "backup-provider")?);
    let no_active = execute_provider(&workspace, &sample_execution_request("capability.one", true))
        .expect_err("inactive-only configuration should not execute");
    assert!(matches!(no_active, CapabilityProviderOrchestratorError::NoActiveProvider));
    Ok(())
}

#[test]
fn orchestrator_execute_provider_covers_permission_and_capability_paths()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("provider-orchestrator-permissions");
    register_provider(
        &workspace,
        scripted_registration(
            "demo-provider",
            write_provider_script("provider-permissions", all_permissions_provider_script())?,
        ),
    )?;

    let success = execute_provider(&workspace, &sample_execution_request("capability.demo", true))?;
    assert_eq!(
        success.session_record.projection.validation_disposition,
        Some(ProviderValidationOutcome::Accepted)
    );

    let blocked =
        execute_provider(&workspace, &sample_execution_request("capability.demo", false))?;
    assert_eq!(
        blocked.session_record.projection.validation_disposition,
        Some(ProviderValidationOutcome::Blocked)
    );
    assert_eq!(
        blocked.session_record.projection.failure_class,
        Some(ProviderFailureClass::PermissionAdmission)
    );

    let missing =
        execute_provider(&workspace, &sample_execution_request("capability.missing", true))
            .expect_err("missing capability should fail");
    assert!(matches!(missing, CapabilityProviderOrchestratorError::ProviderNotRegistered { .. }));
    Ok(())
}

#[test]
fn orchestrator_blocks_unavailable_and_missing_capabilities() -> Result<(), Box<dyn Error>> {
    let blocked_workspace = temp_workspace("provider-orchestrator-blocked");
    let blocked = register_provider(
        &blocked_workspace,
        scripted_registration(
            "demo-provider",
            write_provider_script(
                "provider-missing-capabilities",
                missing_capabilities_provider_script(),
            )?,
        ),
    )?;
    assert_eq!(blocked.activation_state, CapabilityProviderActivationState::Blocked);

    let unavailable_workspace = temp_workspace("provider-orchestrator-unavailable");
    register_provider(
        &unavailable_workspace,
        scripted_registration(
            "demo-provider",
            write_provider_script(
                "provider-unavailable",
                unavailable_after_activation_provider_script("provider-unavailable-state"),
            )?,
        ),
    )?;
    let outcome = execute_provider(
        &unavailable_workspace,
        &sample_execution_request("capability.demo", true),
    )?;
    assert_eq!(
        outcome.session_record.projection.validation_disposition,
        Some(ProviderValidationOutcome::Blocked)
    );
    assert_eq!(
        outcome.session_record.projection.failure_class,
        Some(ProviderFailureClass::Readiness)
    );
    Ok(())
}

fn sample_execution_request(
    capability_id: &str,
    all_permissions: bool,
) -> ProviderExecutionRequest {
    ProviderExecutionRequest {
        request_id: "req-1".to_string(),
        session_ref: "session-1".to_string(),
        step_or_stage_ref: "step-1".to_string(),
        capability_id: capability_id.to_string(),
        goal_summary: "demo goal".to_string(),
        lifecycle_phase: "implement".to_string(),
        authority_zone: "workspace".to_string(),
        context_pack_refs: vec!["ctx-1".to_string()],
        permission_envelope: ProviderPermissionEnvelope {
            read_files: all_permissions,
            write_files: all_permissions,
            run_commands: all_permissions,
            network: all_permissions,
            read_secrets: all_permissions,
            write_artifacts: all_permissions,
            allowed_paths: Vec::new(),
            max_runtime_ms: 1_000,
            max_output_bytes: 4_096,
        },
        expected_outputs: vec!["evidence".to_string()],
    }
}

fn scripted_registration(
    provider_id: &str,
    script_path: PathBuf,
) -> CapabilityProviderRegistration {
    CapabilityProviderRegistration {
        provider_id: provider_id.to_string(),
        display_name: format!("{provider_id} Display"),
        transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
            command_ref: script_path.to_string_lossy().into_owned(),
            args: Vec::new(),
            working_directory_ref: None,
            environment_ref_names: Vec::new(),
        }),
        registration_source: CapabilityProviderRegistrationSource::OperatorCli,
        discovery_state: CapabilityProviderDiscoveryState::Explicit,
        activation_state: CapabilityProviderActivationState::Inactive,
        config_refs: Vec::new(),
        secret_handle_refs: Vec::new(),
        setup_requirements: Vec::new(),
        capability_ids: Vec::new(),
        active_profile_id: None,
    }
}

fn active_provider_script(provider_id: &str, capability_id: &str) -> String {
    format!(
        "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{{\"declarations\":[{{\"provider_id\":\"{provider_id}\",\"protocol_line\":\"capability-provider-v1\",\"protocol_version\":\"1\",\"capability_id\":\"{capability_id}\",\"supported_lifecycle_phases\":[\"implement\"],\"supported_inputs\":[\"context_pack\"],\"supported_outputs\":[\"evidence\"],\"mutation_support\":\"proposal_only\",\"required_permissions\":[\"read_files\"],\"evidence_formats\":[\"markdown\"]}}]}}'\n    ;;\n  health)\n    printf '%s' '{{\"provider_id\":\"{provider_id}\",\"readiness_state\":\"ready\",\"checked_at\":1}}'\n    ;;\n  prepare)\n    printf '%s' '{{\"request_id\":\"req-1\",\"required_context_refs\":[\"ctx-1\"]}}'\n    ;;\n  execute)\n    printf '%s' '{{\"request_id\":\"req-1\",\"status\":\"succeeded\"}}'\n    ;;\n  collect_evidence)\n    printf '%s' '{{\"request_id\":\"req-1\",\"evidence_refs\":[\"collected-evidence\"]}}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
    )
}

fn all_permissions_provider_script() -> &'static str {
    "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{\"declarations\":[{\"provider_id\":\"demo-provider\",\"protocol_line\":\"capability-provider-v1\",\"protocol_version\":\"1\",\"capability_id\":\"capability.demo\",\"supported_lifecycle_phases\":[\"implement\"],\"supported_inputs\":[\"context_pack\"],\"supported_outputs\":[\"evidence\"],\"mutation_support\":\"proposal_only\",\"required_permissions\":[\"read_files\",\"write_files\",\"run_commands\",\"network\",\"read_secrets\",\"write_artifacts\"],\"evidence_formats\":[\"markdown\"]}]}'\n    ;;\n  health)\n    printf '%s' '{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":1}'\n    ;;\n  prepare)\n    printf '%s' '{\"request_id\":\"req-1\",\"required_context_refs\":[\"ctx-1\"]}'\n    ;;\n  execute)\n    printf '%s' '{\"request_id\":\"req-1\",\"status\":\"succeeded\"}'\n    ;;\n  collect_evidence)\n    printf '%s' '{\"request_id\":\"req-1\",\"evidence_refs\":[\"collected-evidence\"]}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
}

fn missing_capabilities_provider_script() -> &'static str {
    "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{\"declarations\":[]}'\n    ;;\n  health)\n    printf '%s' '{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":1}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
}

fn unavailable_after_activation_provider_script(state_file_name: &str) -> String {
    format!(
        "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\nstate_file=\"$(dirname \"$0\")/{state_file_name}\"\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{{\"declarations\":[{{\"provider_id\":\"demo-provider\",\"protocol_line\":\"capability-provider-v1\",\"protocol_version\":\"1\",\"capability_id\":\"capability.demo\",\"supported_lifecycle_phases\":[\"implement\"],\"supported_inputs\":[\"context_pack\"],\"supported_outputs\":[\"evidence\"],\"mutation_support\":\"proposal_only\",\"required_permissions\":[\"read_files\"],\"evidence_formats\":[\"markdown\"]}}]}}'\n    ;;\n  health)\n    if [ -f \"$state_file\" ]; then\n      printf '%s' '{{\"provider_id\":\"demo-provider\",\"readiness_state\":\"unavailable\",\"checked_at\":2}}'\n    else\n      : > \"$state_file\"\n      printf '%s' '{{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":1}}'\n    fi\n    ;;\n  prepare)\n    printf '%s' '{{\"request_id\":\"req-1\",\"required_context_refs\":[\"ctx-1\"]}}'\n    ;;\n  execute)\n    printf '%s' '{{\"request_id\":\"req-1\",\"status\":\"succeeded\"}}'\n    ;;\n  collect_evidence)\n    printf '%s' '{{\"request_id\":\"req-1\",\"evidence_refs\":[\"collected-evidence\"]}}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
    )
}

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-provider-orchestrator-{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).expect("workspace should be created");
    workspace
}

fn write_provider_script(prefix: &str, body: impl AsRef<str>) -> Result<PathBuf, Box<dyn Error>> {
    let directory = temp_workspace(prefix);
    let script_path = directory.join(format!("{prefix}-{}.sh", Uuid::new_v4()));
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&script_path)?;
        f.write_all(body.as_ref().as_bytes())?;
        f.flush()?;
    }
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(script_path)
}
