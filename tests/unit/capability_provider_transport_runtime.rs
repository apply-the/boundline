use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use boundline::adapters::capability_provider_runtime::{
    collect_evidence, execute_request, fetch_capabilities, fetch_health, prepare_execution,
};
use boundline::domain::capability_provider::{
    CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
    CapabilityProviderRegistration, CapabilityProviderRegistrationSource, CommandProviderTransport,
    HttpProviderTransport, ProviderExecutionRequest, ProviderPermissionEnvelope,
    ProviderTransportDescriptor,
};
use uuid::Uuid;

#[test]
fn command_transport_round_trips_provider_lifecycle() -> Result<(), Box<dyn Error>> {
    let registration = command_registration(write_script("transport-success", success_script())?);
    let request = sample_execution_request("capability.demo");

    let capabilities = fetch_capabilities(&registration)?;
    assert_eq!(capabilities.len(), 1);
    assert_eq!(capabilities[0].capability_id, "capability.demo");

    let health = fetch_health(&registration)?;
    assert_eq!(health.provider_id, "demo-provider");

    let preparation = prepare_execution(&registration, &request)?;
    assert_eq!(preparation.required_context_refs, vec!["ctx-1".to_string()]);

    let execution = execute_request(&registration, &request)?;
    assert_eq!(execution.request_id, "req-1");

    let evidence = collect_evidence(&registration, "req-1", &execution)?;
    assert_eq!(evidence.evidence_refs, vec!["collected-evidence".to_string()]);
    Ok(())
}

#[test]
fn command_transport_uses_working_directory_when_configured() -> Result<(), Box<dyn Error>> {
    let working_directory = temp_directory("transport-working-directory");
    let output_path = working_directory.join("cwd.txt");
    let script = write_script(
        "transport-working-directory",
        &format!(
            "#!/bin/sh\ncat >/dev/null\npwd > \"{}\"\nprintf '%s' '{{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":7}}'\n",
            output_path.display()
        ),
    )?;
    let registration = command_registration_with_working_directory(script, &working_directory);
    let health = fetch_health(&registration)?;
    assert_eq!(health.checked_at, 7);

    let recorded = fs::read_to_string(output_path)?;
    let recorded_canonical = fs::canonicalize(Path::new(recorded.trim()))?;
    let expected_canonical = fs::canonicalize(&working_directory)?;
    assert_eq!(recorded_canonical, expected_canonical);
    Ok(())
}

#[test]
fn command_transport_surfaces_non_zero_status_without_stderr() -> Result<(), Box<dyn Error>> {
    let registration = command_registration(write_script(
        "transport-empty-stderr",
        "#!/bin/sh\ncat >/dev/null\nexit 9\n",
    )?);
    let error = fetch_health(&registration).expect_err("health should fail");
    let text = error.to_string();
    assert!(text.contains("provider command exited with status"));
    assert!(!text.contains("boom"));
    Ok(())
}

#[test]
fn http_transport_surfaces_request_failures() {
    let registration = http_registration("http://127.0.0.1:9/provider");
    let error = fetch_health(&registration).expect_err("health should fail");
    assert!(error.to_string().contains("provider HTTP request failed"));
}

fn sample_execution_request(capability_id: &str) -> ProviderExecutionRequest {
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
            read_files: true,
            write_files: true,
            run_commands: true,
            network: true,
            read_secrets: true,
            write_artifacts: true,
            allowed_paths: Vec::new(),
            max_runtime_ms: 1_000,
            max_output_bytes: 4_096,
        },
        expected_outputs: vec!["evidence".to_string()],
    }
}

fn command_registration(script_path: PathBuf) -> CapabilityProviderRegistration {
    CapabilityProviderRegistration {
        provider_id: "demo-provider".to_string(),
        display_name: "Demo Provider".to_string(),
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

fn command_registration_with_working_directory(
    script_path: PathBuf,
    working_directory: &Path,
) -> CapabilityProviderRegistration {
    CapabilityProviderRegistration {
        provider_id: "demo-provider".to_string(),
        display_name: "Demo Provider".to_string(),
        transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
            command_ref: script_path.to_string_lossy().into_owned(),
            args: Vec::new(),
            working_directory_ref: Some(working_directory.to_string_lossy().into_owned()),
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

fn http_registration(endpoint_ref: &str) -> CapabilityProviderRegistration {
    CapabilityProviderRegistration {
        provider_id: "demo-provider".to_string(),
        display_name: "Demo Provider".to_string(),
        transport: ProviderTransportDescriptor::Http(HttpProviderTransport {
            endpoint_ref: endpoint_ref.to_string(),
            auth_scheme: None,
            headers_ref: Vec::new(),
            tls_policy: None,
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

fn success_script() -> &'static str {
    "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{\"declarations\":[{\"provider_id\":\"demo-provider\",\"protocol_line\":\"capability-provider-v1\",\"protocol_version\":\"1\",\"capability_id\":\"capability.demo\",\"supported_lifecycle_phases\":[\"implement\"],\"supported_inputs\":[\"context_pack\"],\"supported_outputs\":[\"evidence\"],\"mutation_support\":\"proposal_only\",\"required_permissions\":[\"read_files\"],\"evidence_formats\":[\"markdown\"]}]}'\n    ;;\n  health)\n    printf '%s' '{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":1}'\n    ;;\n  prepare)\n    printf '%s' '{\"request_id\":\"req-1\",\"required_context_refs\":[\"ctx-1\"]}'\n    ;;\n  execute)\n    printf '%s' '{\"request_id\":\"req-1\",\"status\":\"succeeded\"}'\n    ;;\n  collect_evidence)\n    printf '%s' '{\"request_id\":\"req-1\",\"evidence_refs\":[\"collected-evidence\"]}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
}

fn temp_directory(prefix: &str) -> PathBuf {
    let directory = std::env::temp_dir()
        .join(format!("boundline-provider-transport-{prefix}-{}", Uuid::new_v4()));
    let _ = fs::create_dir_all(&directory);
    directory
}

fn write_script(prefix: &str, body: &str) -> Result<PathBuf, Box<dyn Error>> {
    let directory = temp_directory(prefix);
    let script_path = directory.join(format!("{prefix}-{}.sh", Uuid::new_v4()));
    fs::write(&script_path, body)?;
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(script_path)
}
