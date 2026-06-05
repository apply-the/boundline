//! Capability-provider registration, inspection, and health commands.

use std::path::Path;

use thiserror::Error;

use crate::adapters::config_store::ConfigStoreError;
use crate::cli::CommandExitStatus;
use crate::domain::capability_provider::{
    CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
    CapabilityProviderRegistration, CapabilityProviderRegistrationSource, CommandProviderTransport,
    HttpProviderTransport, ProviderSetupRequiredState, ProviderSetupRequirement,
    ProviderSetupRequirementKind, ProviderSetupResolutionState, ProviderTransportDescriptor,
};
use crate::domain::configuration::PersistedCapabilityProviderConfiguration;
use crate::orchestrator::capability_provider_runtime::{
    CapabilityProviderOrchestratorError, load_provider_configuration, provider_health,
    register_provider, remove_provider,
};

/// Rendered result for provider CLI commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

/// Input for `boundline provider add`.
#[derive(Debug)]
pub struct AddProviderRequest<'a> {
    pub provider_id: &'a str,
    pub display_name: Option<&'a str>,
    pub workspace: Option<&'a Path>,
    pub command: Option<&'a str>,
    pub endpoint: Option<&'a str>,
    pub arg: &'a [String],
    pub config_ref: &'a [String],
    pub secret_handle: &'a [String],
    pub require_config: &'a [String],
    pub require_secret: &'a [String],
}

/// Input for `boundline provider show`.
#[derive(Debug, Clone, Copy)]
pub struct ShowProviderRequest<'a> {
    pub workspace: Option<&'a Path>,
}

/// Input for `boundline provider remove`.
#[derive(Debug, Clone, Copy)]
pub struct RemoveProviderRequest<'a> {
    pub provider_id: &'a str,
    pub workspace: Option<&'a Path>,
}

/// Input for `boundline provider health`.
#[derive(Debug, Clone, Copy)]
pub struct HealthProviderRequest<'a> {
    pub provider_id: Option<&'a str>,
    pub workspace: Option<&'a Path>,
}

#[derive(Debug, Error)]
enum ProviderCommandError {
    #[error("provider commands require --workspace")]
    WorkspaceRequired,
    #[error("provider registration requires exactly one of --command or --endpoint")]
    MissingTransport,
    #[error("invalid --config-ref value `{entry}`; expected key=value")]
    InvalidConfigRef { entry: String },
    #[error(transparent)]
    ConfigStore(#[from] ConfigStoreError),
    #[error(transparent)]
    Orchestrator(#[from] CapabilityProviderOrchestratorError),
}

/// Executes `boundline provider add`.
pub fn execute_add(request: AddProviderRequest<'_>) -> ProviderCommandReport {
    match execute_add_inner(request) {
        Ok(report) => report,
        Err(error) => error_report("add", &error.to_string()),
    }
}

/// Executes `boundline provider show`.
pub fn execute_show(request: ShowProviderRequest<'_>) -> ProviderCommandReport {
    match execute_show_inner(request) {
        Ok(report) => report,
        Err(error) => error_report("show", &error.to_string()),
    }
}

/// Executes `boundline provider remove`.
pub fn execute_remove(request: RemoveProviderRequest<'_>) -> ProviderCommandReport {
    match execute_remove_inner(request) {
        Ok(report) => report,
        Err(error) => error_report("remove", &error.to_string()),
    }
}

/// Executes `boundline provider health`.
pub fn execute_health(request: HealthProviderRequest<'_>) -> ProviderCommandReport {
    match execute_health_inner(request) {
        Ok(report) => report,
        Err(error) => error_report("health", &error.to_string()),
    }
}

fn execute_add_inner(
    request: AddProviderRequest<'_>,
) -> Result<ProviderCommandReport, ProviderCommandError> {
    let workspace = request.workspace.ok_or(ProviderCommandError::WorkspaceRequired)?;
    let registration = build_registration(request)?;
    let projection = register_provider(workspace, registration)?;
    Ok(ProviderCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: [
            format!("provider_status: {}", projection.activation_state.as_str()),
            format!("provider_id: {}", projection.provider_id),
            format!("provider_summary: {}", projection.summary),
        ]
        .join("\n"),
    })
}

fn execute_show_inner(
    request: ShowProviderRequest<'_>,
) -> Result<ProviderCommandReport, ProviderCommandError> {
    let workspace = request.workspace.ok_or(ProviderCommandError::WorkspaceRequired)?;
    let configuration = load_provider_configuration(workspace)?;
    let output = configuration
        .map(show_configuration_lines)
        .unwrap_or_else(|| vec!["provider_status: unconfigured".to_string()])
        .join("\n");
    Ok(ProviderCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output: output })
}

fn execute_remove_inner(
    request: RemoveProviderRequest<'_>,
) -> Result<ProviderCommandReport, ProviderCommandError> {
    let workspace = request.workspace.ok_or(ProviderCommandError::WorkspaceRequired)?;
    let removed = remove_provider(workspace, request.provider_id)?;
    let terminal_output = if removed {
        format!("provider_status: removed\nprovider_id: {}", request.provider_id)
    } else {
        format!("provider_status: not_found\nprovider_id: {}", request.provider_id)
    };
    Ok(ProviderCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output })
}

fn execute_health_inner(
    request: HealthProviderRequest<'_>,
) -> Result<ProviderCommandReport, ProviderCommandError> {
    let workspace = request.workspace.ok_or(ProviderCommandError::WorkspaceRequired)?;
    let health = provider_health(workspace, request.provider_id)?;
    Ok(ProviderCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: [
            format!("provider_id: {}", health.provider_id),
            format!("provider_readiness: {}", provider_readiness_text(health.readiness_state)),
            format!("provider_warnings: {}", join_or_none(&health.warnings)),
            format!(
                "provider_missing_dependencies: {}",
                join_or_none(&health.missing_dependencies)
            ),
        ]
        .join("\n"),
    })
}

fn build_registration(
    request: AddProviderRequest<'_>,
) -> Result<CapabilityProviderRegistration, ProviderCommandError> {
    let transport = build_transport(&request)?;
    let config_refs = parse_config_refs(request.config_ref)?;
    let secret_handle_refs = request.secret_handle.to_vec();
    let setup_requirements = build_setup_requirements(
        request.require_config,
        request.require_secret,
        &config_refs,
        request.secret_handle,
    );
    Ok(CapabilityProviderRegistration {
        provider_id: request.provider_id.to_string(),
        display_name: request.display_name.unwrap_or(request.provider_id).to_string(),
        transport,
        registration_source: CapabilityProviderRegistrationSource::OperatorCli,
        discovery_state: CapabilityProviderDiscoveryState::Explicit,
        activation_state: CapabilityProviderActivationState::Inactive,
        config_refs,
        secret_handle_refs,
        setup_requirements,
        capability_ids: Vec::new(),
        active_profile_id: None,
    })
}

fn build_transport(
    request: &AddProviderRequest<'_>,
) -> Result<ProviderTransportDescriptor, ProviderCommandError> {
    match (request.command, request.endpoint) {
        (Some(command), None) => {
            Ok(ProviderTransportDescriptor::Command(CommandProviderTransport {
                command_ref: command.to_string(),
                args: request.arg.to_vec(),
                working_directory_ref: None,
                environment_ref_names: Vec::new(),
            }))
        }
        (None, Some(endpoint)) => Ok(ProviderTransportDescriptor::Http(HttpProviderTransport {
            endpoint_ref: endpoint.to_string(),
            auth_scheme: None,
            headers_ref: Vec::new(),
            tls_policy: None,
        })),
        _ => Err(ProviderCommandError::MissingTransport),
    }
}

fn parse_config_refs(entries: &[String]) -> Result<Vec<String>, ProviderCommandError> {
    let mut refs = Vec::new();
    for entry in entries {
        if !entry.contains('=') {
            return Err(ProviderCommandError::InvalidConfigRef { entry: entry.clone() });
        }
        refs.push(entry.clone());
    }
    Ok(refs)
}

fn build_setup_requirements(
    required_config: &[String],
    required_secret: &[String],
    config_refs: &[String],
    secret_handles: &[String],
) -> Vec<ProviderSetupRequirement> {
    let mut requirements = Vec::new();
    for field in required_config {
        requirements.push(ProviderSetupRequirement {
            requirement_id: format!("config-{field}"),
            kind: ProviderSetupRequirementKind::ConfigValue,
            required_state: ProviderSetupRequiredState::Required,
            resolution_state: requirement_state(
                config_refs.iter().any(|entry| entry.starts_with(&format!("{field}="))),
            ),
            display_label: field.clone(),
            source_ref: config_refs
                .iter()
                .find(|entry| entry.starts_with(&format!("{field}=")))
                .cloned(),
        });
    }
    for handle in required_secret {
        requirements.push(ProviderSetupRequirement {
            requirement_id: format!("secret-{handle}"),
            kind: ProviderSetupRequirementKind::SecretHandle,
            required_state: ProviderSetupRequiredState::Required,
            resolution_state: requirement_state(secret_handles.iter().any(|entry| entry == handle)),
            display_label: handle.clone(),
            source_ref: secret_handles.iter().find(|entry| *entry == handle).cloned(),
        });
    }
    requirements
}

fn requirement_state(is_present: bool) -> ProviderSetupResolutionState {
    if is_present {
        ProviderSetupResolutionState::Present
    } else {
        ProviderSetupResolutionState::Missing
    }
}

fn show_configuration_lines(
    configuration: PersistedCapabilityProviderConfiguration,
) -> Vec<String> {
    let mut lines = vec![format!(
        "provider_status: {}",
        configuration
            .active_provider_id
            .as_deref()
            .map_or("configured_inactive".to_string(), |provider_id| format!(
                "active:{provider_id}"
            ))
    )];
    for registration in configuration.registrations {
        lines.push(format!("provider_id: {}", registration.provider_id));
        lines
            .push(format!("provider_activation_state: {}", registration.activation_state.as_str()));
        lines.push(format!(
            "provider_capability_ids: {}",
            join_or_none(&registration.capability_ids)
        ));
        lines.push(format!(
            "provider_setup_requirements: {}",
            render_setup_requirements(&registration.setup_requirements)
        ));
    }
    lines
}

fn render_setup_requirements(requirements: &[ProviderSetupRequirement]) -> String {
    if requirements.is_empty() {
        return "none".to_string();
    }
    requirements
        .iter()
        .map(|requirement| {
            format!(
                "{}={}",
                requirement.display_label,
                match requirement.resolution_state {
                    ProviderSetupResolutionState::Present => "present",
                    ProviderSetupResolutionState::Missing => "missing",
                    ProviderSetupResolutionState::Invalid => "invalid",
                    ProviderSetupResolutionState::Unchecked => "unchecked",
                }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn provider_readiness_text(
    readiness: crate::domain::capability_provider::ProviderReadinessState,
) -> &'static str {
    match readiness {
        crate::domain::capability_provider::ProviderReadinessState::Ready => "ready",
        crate::domain::capability_provider::ProviderReadinessState::Degraded => "degraded",
        crate::domain::capability_provider::ProviderReadinessState::Unavailable => "unavailable",
    }
}

fn join_or_none(items: &[String]) -> String {
    if items.is_empty() { "none".to_string() } else { items.join(", ") }
}

fn error_report(action: &str, message: &str) -> ProviderCommandReport {
    let _ = action;
    ProviderCommandReport {
        exit_status: CommandExitStatus::NonSuccess,
        terminal_output: format!("provider_status: blocked\nprovider_reason: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use super::{
        AddProviderRequest, HealthProviderRequest, RemoveProviderRequest, ShowProviderRequest,
        build_registration, execute_add, execute_health, execute_remove, execute_show,
        provider_readiness_text, show_configuration_lines,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::cli::CommandExitStatus;
    use crate::domain::capability_provider::{
        CapabilityProviderActivationState, CapabilityProviderDiscoveryState,
        CapabilityProviderRegistration, CapabilityProviderRegistrationSource,
        CommandProviderTransport, ProviderReadinessState, ProviderSetupRequiredState,
        ProviderSetupRequirement, ProviderSetupRequirementKind, ProviderSetupResolutionState,
        ProviderTransportDescriptor,
    };
    use crate::domain::configuration::{ConfigFile, PersistedCapabilityProviderConfiguration};
    use crate::orchestrator::capability_provider_runtime::register_provider;
    use uuid::Uuid;

    #[test]
    fn provider_command_wrappers_surface_workspace_required_errors() {
        let add = execute_add(AddProviderRequest {
            provider_id: "demo",
            display_name: None,
            workspace: None,
            command: Some("/bin/echo"),
            endpoint: None,
            arg: &[],
            config_ref: &[],
            secret_handle: &[],
            require_config: &[],
            require_secret: &[],
        });
        let show = execute_show(ShowProviderRequest { workspace: None });
        let remove = execute_remove(RemoveProviderRequest { provider_id: "demo", workspace: None });
        let health = execute_health(HealthProviderRequest { provider_id: None, workspace: None });

        for report in [add, show, remove, health] {
            assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
            assert!(report.terminal_output.contains("provider_status: blocked"));
        }
    }

    #[test]
    fn build_registration_and_setup_rendering_cover_command_and_http_paths() {
        let config_refs = vec!["token=config/token".to_string()];
        let secret_handles = vec!["provider-secret".to_string()];
        let require_config = vec!["token".to_string()];
        let require_secret = vec!["provider-secret".to_string()];
        let command_request = AddProviderRequest {
            provider_id: "demo-command",
            display_name: Some("Demo Command"),
            workspace: None,
            command: Some("/bin/echo"),
            endpoint: None,
            arg: &["hello".to_string()],
            config_ref: &config_refs,
            secret_handle: &secret_handles,
            require_config: &require_config,
            require_secret: &require_secret,
        };
        let registration = build_registration(command_request).expect("command registration");
        assert_eq!(registration.provider_id, "demo-command");
        assert_eq!(registration.setup_requirements.len(), 2);
        assert!(!registration.has_blocking_setup_requirements());

        let invalid = build_registration(AddProviderRequest {
            provider_id: "invalid",
            display_name: None,
            workspace: None,
            command: None,
            endpoint: None,
            arg: &[],
            config_ref: &[],
            secret_handle: &[],
            require_config: &[],
            require_secret: &[],
        });
        assert!(invalid.is_err());

        let endpoint_registration = build_registration(AddProviderRequest {
            provider_id: "demo-http",
            display_name: None,
            workspace: None,
            command: None,
            endpoint: Some("http://127.0.0.1:9/provider"),
            arg: &[],
            config_ref: &[],
            secret_handle: &[],
            require_config: &[],
            require_secret: &[],
        })
        .expect("http registration");
        assert_eq!(endpoint_registration.transport.transport_kind().as_str(), "http");
    }

    #[test]
    fn show_configuration_and_readiness_helpers_render_provider_state() {
        let registration = CapabilityProviderRegistration {
            provider_id: "demo-provider".to_string(),
            display_name: "Demo Provider".to_string(),
            transport: ProviderTransportDescriptor::Command(CommandProviderTransport {
                command_ref: "/bin/echo".to_string(),
                args: Vec::new(),
                working_directory_ref: None,
                environment_ref_names: Vec::new(),
            }),
            registration_source: CapabilityProviderRegistrationSource::OperatorCli,
            discovery_state: CapabilityProviderDiscoveryState::Explicit,
            activation_state: CapabilityProviderActivationState::Active,
            config_refs: Vec::new(),
            secret_handle_refs: Vec::new(),
            setup_requirements: vec![ProviderSetupRequirement {
                requirement_id: "config-token".to_string(),
                kind: ProviderSetupRequirementKind::ConfigValue,
                required_state: ProviderSetupRequiredState::Required,
                resolution_state: ProviderSetupResolutionState::Present,
                display_label: "token".to_string(),
                source_ref: Some("config/token".to_string()),
            }],
            capability_ids: vec!["capability.demo".to_string()],
            active_profile_id: None,
        };
        let lines = show_configuration_lines(PersistedCapabilityProviderConfiguration {
            registrations: vec![registration],
            active_provider_id: Some("demo-provider".to_string()),
            last_validated_at: Some(42),
        });
        assert!(lines.iter().any(|line| line == "provider_status: active:demo-provider"));
        assert!(lines.iter().any(|line| line == "provider_capability_ids: capability.demo"));
        assert!(lines.iter().any(|line| line == "provider_setup_requirements: token=present"));

        assert_eq!(provider_readiness_text(ProviderReadinessState::Ready), "ready");
        assert_eq!(provider_readiness_text(ProviderReadinessState::Degraded), "degraded");
        assert_eq!(provider_readiness_text(ProviderReadinessState::Unavailable), "unavailable");
    }

    #[test]
    fn provider_helpers_cover_invalid_refs_not_found_and_unchecked_rendering() {
        let invalid = build_registration(AddProviderRequest {
            provider_id: "invalid-config-ref",
            display_name: None,
            workspace: None,
            command: Some("/bin/echo"),
            endpoint: None,
            arg: &[],
            config_ref: &["missing-delimiter".to_string()],
            secret_handle: &[],
            require_config: &[],
            require_secret: &[],
        });
        assert!(invalid.is_err());

        let workspace = std::env::temp_dir()
            .join(format!("boundline-provider-cli-not-found-{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&workspace);
        let remove = execute_remove(RemoveProviderRequest {
            provider_id: "unknown-provider",
            workspace: Some(&workspace),
        });
        assert_eq!(remove.exit_status, CommandExitStatus::Succeeded);
        assert!(remove.terminal_output.contains("provider_status: not_found"));

        let lines = show_configuration_lines(PersistedCapabilityProviderConfiguration {
            registrations: vec![CapabilityProviderRegistration {
                provider_id: "demo-provider".to_string(),
                display_name: "Demo Provider".to_string(),
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
                setup_requirements: vec![
                    ProviderSetupRequirement {
                        requirement_id: "config-demo".to_string(),
                        kind: ProviderSetupRequirementKind::ConfigValue,
                        required_state: ProviderSetupRequiredState::Required,
                        resolution_state: ProviderSetupResolutionState::Invalid,
                        display_label: "demo-config".to_string(),
                        source_ref: None,
                    },
                    ProviderSetupRequirement {
                        requirement_id: "secret-demo".to_string(),
                        kind: ProviderSetupRequirementKind::SecretHandle,
                        required_state: ProviderSetupRequiredState::Required,
                        resolution_state: ProviderSetupResolutionState::Unchecked,
                        display_label: "demo-secret".to_string(),
                        source_ref: None,
                    },
                ],
                capability_ids: Vec::new(),
                active_profile_id: None,
            }],
            active_provider_id: None,
            last_validated_at: None,
        });
        assert!(lines.iter().any(|line| line == "provider_status: configured_inactive"));
        assert!(lines.iter().any(|line| {
            line == "provider_setup_requirements: demo-config=invalid, demo-secret=unchecked"
        }));
    }

    #[test]
    fn provider_health_and_remove_cover_success_paths() {
        let workspace =
            std::env::temp_dir().join(format!("boundline-provider-cli-health-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&workspace);
        let _ = FileConfigStore::for_workspace(&workspace).save_local(&ConfigFile::default());
        let registration = scripted_registration(
            "demo-provider",
            write_script("provider-health", provider_health_script()).expect("provider script"),
        );
        register_provider(&workspace, registration).expect("provider should register");

        let health = execute_health(HealthProviderRequest {
            provider_id: None,
            workspace: Some(&workspace),
        });
        assert_eq!(health.exit_status, CommandExitStatus::Succeeded);
        assert!(health.terminal_output.contains("provider_readiness: ready"));

        let removed = execute_remove(RemoveProviderRequest {
            provider_id: "demo-provider",
            workspace: Some(&workspace),
        });
        assert_eq!(removed.exit_status, CommandExitStatus::Succeeded);
        assert!(removed.terminal_output.contains("provider_status: removed"));
    }

    fn scripted_registration(
        provider_id: &str,
        script_path: PathBuf,
    ) -> CapabilityProviderRegistration {
        CapabilityProviderRegistration {
            provider_id: provider_id.to_string(),
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

    fn provider_health_script() -> &'static str {
        "#!/bin/sh\noperation=\"$1\"\ncat >/dev/null\ncase \"$operation\" in\n  capabilities)\n    printf '%s' '{\"declarations\":[{\"provider_id\":\"demo-provider\",\"protocol_line\":\"capability-provider-v1\",\"protocol_version\":\"1\",\"capability_id\":\"capability.demo\",\"supported_lifecycle_phases\":[\"implement\"],\"supported_inputs\":[\"context_pack\"],\"supported_outputs\":[\"evidence\"],\"mutation_support\":\"proposal_only\",\"required_permissions\":[\"read_files\"],\"evidence_formats\":[\"markdown\"]}]}'\n    ;;\n  health)\n    printf '%s' '{\"provider_id\":\"demo-provider\",\"readiness_state\":\"ready\",\"checked_at\":1}'\n    ;;\n  *)\n    exit 1\n    ;;\nesac\n"
    }

    fn write_script(prefix: &str, body: &str) -> Result<PathBuf, std::io::Error> {
        let directory =
            std::env::temp_dir().join(format!("boundline-provider-cli-script-{prefix}"));
        fs::create_dir_all(&directory)?;
        let script_path = directory.join(format!("{prefix}-{}.sh", Uuid::new_v4()));
        fs::write(&script_path, body)?;
        let mut permissions = fs::metadata(&script_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)?;
        Ok(script_path)
    }
}
