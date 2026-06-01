use std::collections::BTreeMap;
use std::error::Error;
use std::fs;

use boundline::FileConfigStore;
use boundline::FnAgentAdapter;
use boundline::cli::CommandExitStatus;
use boundline::cli::config::execute_show;
use boundline::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, ConfigFile, ConfigShowScope, ModelRoute,
    PersistedAdapterConfiguration, RoutingConfig, RoutingOverrides, RuntimeKind, ValueSource,
    resolve_effective_routing,
};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use boundline::domain::step::StepExecutionResult;
use boundline::registry::agent_registry::{
    AgentRegistry, FrameworkAdapterProfileRegistry, FrameworkAdapterRegistryError, RegistryError,
    speckit_known_profile,
};
use serde_json::json;
use uuid::Uuid;

const SAMPLE_ADAPTER_ID: &str = "speckit";
const SAMPLE_ADAPTER_COMMAND: &str = "boundline-adapter-speckit";
const SAMPLE_FIELD_KEY: &str = "template_repo";
const SAMPLE_FIELD_PATH: &str = "../boundline-framework-template";

fn temp_workspace(prefix: &str) -> std::path::PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

#[test]
fn effective_routing_prefers_workspace_over_global() {
    let workspace = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "gpt-4o".to_string(),
        }),
        ..RoutingConfig::default()
    };
    let global = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Claude,
            model: "sonnet-4".to_string(),
        }),
        ..RoutingConfig::default()
    };

    let resolved = resolve_effective_routing(
        &RoutingOverrides::default(),
        Some(&workspace),
        None,
        Some(&global),
    );
    assert_eq!(resolved.verification.source, ValueSource::Workspace);
    assert_eq!(resolved.verification.route.runtime, RuntimeKind::Copilot);
}

#[test]
fn reviewer_role_routes_can_be_resolved_from_cli_overrides() {
    let mut cli = RoutingOverrides::default();
    cli.reviewer_roles.insert(
        "security".to_string(),
        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
    );

    let mut workspace_roles = BTreeMap::new();
    workspace_roles.insert(
        "security".to_string(),
        ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() },
    );
    let workspace = RoutingConfig { reviewer_roles: workspace_roles, ..RoutingConfig::default() };

    let resolved = resolve_effective_routing(&cli, Some(&workspace), None, None);
    let security = resolved.reviewer_roles.get("security").expect("security role should exist");
    assert_eq!(security.source, ValueSource::Cli);
    assert_eq!(security.route.runtime, RuntimeKind::Claude);
}

#[test]
fn known_profile_registry_resolves_speckit_by_alias_and_discovery_name()
-> Result<(), Box<dyn Error>> {
    let registry = FrameworkAdapterProfileRegistry::boundline_known_profiles()?;

    let by_alias =
        registry.resolve_profile("speckit").ok_or("expected speckit alias to resolve")?;
    let by_discovery = registry
        .resolve_discovery_name(SAMPLE_ADAPTER_COMMAND)
        .ok_or("expected discovery name to resolve")?;
    let shipped_profile = speckit_known_profile();

    assert_eq!(by_alias.adapter_id, SAMPLE_ADAPTER_ID);
    assert_eq!(by_alias.compatibility_line, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1);
    assert_eq!(by_discovery.adapter_id, SAMPLE_ADAPTER_ID);
    assert_eq!(shipped_profile.discovery_names, vec![SAMPLE_ADAPTER_COMMAND.to_string()]);
    assert_eq!(shipped_profile.prefilled_fields.len(), 2);

    Ok(())
}

#[test]
fn registry_helpers_cover_blank_names_duplicates_and_alias_resolution() -> Result<(), Box<dyn Error>>
{
    let mut agent_registry = AgentRegistry::new();
    let blank_name_error = agent_registry
        .register("   ", FnAgentAdapter::new(|_| StepExecutionResult::success(json!({}))))
        .err()
        .ok_or("blank agent registry name should fail")?;
    assert_eq!(blank_name_error, RegistryError::EmptyName);

    agent_registry.register(
        "planner",
        FnAgentAdapter::new(|_| StepExecutionResult::success(json!({"ok": true}))),
    )?;
    assert!(agent_registry.get("planner").is_some());
    assert!(agent_registry.get("missing").is_none());

    let shipped_profile = speckit_known_profile();
    let mut registry = FrameworkAdapterProfileRegistry::new();
    registry.register_known_profile(shipped_profile.clone())?;
    assert!(registry.get_profile(SAMPLE_ADAPTER_ID).is_some());

    let duplicate_id_error = registry
        .register_known_profile(shipped_profile.clone())
        .err()
        .ok_or("duplicate adapter id should fail")?;
    assert_eq!(
        duplicate_id_error,
        FrameworkAdapterRegistryError::DuplicateAdapterId(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut alias_conflict = shipped_profile.clone();
    alias_conflict.adapter_id = "speckit-alt".to_string();
    alias_conflict.default_command = "boundline-adapter-speckit-alt".to_string();
    alias_conflict.discovery_names = vec!["boundline-adapter-speckit-alt".to_string()];
    let alias_error = registry
        .register_known_profile(alias_conflict)
        .err()
        .ok_or("duplicate alias should fail")?;
    assert_eq!(
        alias_error,
        FrameworkAdapterRegistryError::DuplicateRegistrationAlias(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut discovery_conflict = shipped_profile.clone();
    discovery_conflict.adapter_id = "speckit-another".to_string();
    discovery_conflict.registration_alias = "speckit-another".to_string();
    let discovery_error = registry
        .register_known_profile(discovery_conflict)
        .err()
        .ok_or("duplicate discovery name should fail")?;
    assert_eq!(
        discovery_error,
        FrameworkAdapterRegistryError::DuplicateDiscoveryName(SAMPLE_ADAPTER_COMMAND.to_string())
    );

    let mut alias_profile = shipped_profile;
    alias_profile.adapter_id = "speckit-profile".to_string();
    alias_profile.registration_alias = "speckit-profile-alias".to_string();
    alias_profile.default_command = "boundline-adapter-speckit-profile".to_string();
    alias_profile.discovery_names = vec!["boundline-adapter-speckit-profile".to_string()];
    let mut alias_registry = FrameworkAdapterProfileRegistry::new();
    alias_registry.register_known_profile(alias_profile.clone())?;
    let resolved = alias_registry
        .resolve_profile("speckit-profile-alias")
        .ok_or("registration alias should resolve")?;
    assert_eq!(resolved.adapter_id, alias_profile.adapter_id);
    assert_eq!(alias_registry.profiles().count(), 1);

    Ok(())
}

#[test]
fn known_profile_registry_validation_rejects_blank_fields() -> Result<(), Box<dyn Error>> {
    let mut profile = speckit_known_profile();
    profile.adapter_id = "   ".to_string();
    let empty_adapter_id = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank adapter id should fail")?;
    assert_eq!(empty_adapter_id, FrameworkAdapterRegistryError::EmptyAdapterId);

    let mut profile = speckit_known_profile();
    profile.display_name = "   ".to_string();
    let empty_display_name = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank display name should fail")?;
    assert_eq!(
        empty_display_name,
        FrameworkAdapterRegistryError::EmptyDisplayName(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut profile = speckit_known_profile();
    profile.registration_alias = "   ".to_string();
    let empty_alias = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank alias should fail")?;
    assert_eq!(
        empty_alias,
        FrameworkAdapterRegistryError::EmptyRegistrationAlias(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut profile = speckit_known_profile();
    profile.default_command = "   ".to_string();
    let empty_command = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank default command should fail")?;
    assert_eq!(
        empty_command,
        FrameworkAdapterRegistryError::EmptyDefaultCommand(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut profile = speckit_known_profile();
    profile.compatibility_line = "   ".to_string();
    let empty_compatibility_line = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank compatibility line should fail")?;
    assert_eq!(
        empty_compatibility_line,
        FrameworkAdapterRegistryError::EmptyCompatibilityLine(SAMPLE_ADAPTER_ID.to_string())
    );

    let mut profile = speckit_known_profile();
    profile.discovery_names = vec!["   ".to_string()];
    let empty_discovery_name = FrameworkAdapterProfileRegistry::new()
        .register_known_profile(profile)
        .err()
        .ok_or("blank discovery name should fail")?;
    assert_eq!(
        empty_discovery_name,
        FrameworkAdapterRegistryError::EmptyDiscoveryName(SAMPLE_ADAPTER_ID.to_string())
    );

    Ok(())
}

#[test]
fn persisted_adapter_configuration_projects_resolved_config() {
    let persisted = PersistedAdapterConfiguration {
        selection: AdapterSelectionRecord {
            selection_mode: AdapterSelectionMode::KnownProfile,
            adapter_id: SAMPLE_ADAPTER_ID.to_string(),
            display_name: "Speckit".to_string(),
            command: SAMPLE_ADAPTER_COMMAND.to_string(),
            args: Vec::new(),
            registration_source: AdapterRegistrationSource::AdapterAdd,
            discovery_state: AdapterDiscoveryState::ExplicitCommand,
            compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
            updated_at: 42,
        },
        schema_fingerprint: "schema-v1".to_string(),
        completeness_state: AdapterConfigCompletenessState::Complete,
        interactive_resolution: true,
        last_validated_at: Some(42),
        value_count: 1,
        values: vec![AdapterConfigValueRecord {
            field_key: SAMPLE_FIELD_KEY.to_string(),
            value_kind: AdapterValueKind::Path,
            secret: false,
            string_value: None,
            path_value: Some(SAMPLE_FIELD_PATH.to_string()),
            bool_value: None,
            int_value: None,
            value_source: AdapterValueSource::KnownProfileDefault,
            resolution_state: StoredAdapterConfigValueState::Present,
        }],
    };

    let resolved = persisted.resolved_config();
    assert_eq!(resolved.adapter_id, SAMPLE_ADAPTER_ID);
    assert_eq!(resolved.schema_fingerprint, "schema-v1");
    assert_eq!(resolved.value_count, 1);
    assert_eq!(resolved.values[0].path_value.as_deref(), Some(SAMPLE_FIELD_PATH));
}

#[test]
fn config_show_surfaces_framework_adapter_for_workspace_and_effective_scopes()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_workspace("boundline-config-show-adapter");
    let config = ConfigFile {
        routing: RoutingConfig::default(),
        canon: None,
        adapter: Some(PersistedAdapterConfiguration {
            selection: AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::KnownProfile,
                adapter_id: SAMPLE_ADAPTER_ID.to_string(),
                display_name: "Speckit".to_string(),
                command: SAMPLE_ADAPTER_COMMAND.to_string(),
                args: Vec::new(),
                registration_source: AdapterRegistrationSource::AdapterAdd,
                discovery_state: AdapterDiscoveryState::DiscoveredOnPath,
                compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                updated_at: 42,
            },
            schema_fingerprint: "schema-v1".to_string(),
            completeness_state: AdapterConfigCompletenessState::Complete,
            interactive_resolution: false,
            last_validated_at: Some(42),
            value_count: 2,
            values: vec![
                AdapterConfigValueRecord {
                    field_key: "template_repo".to_string(),
                    value_kind: AdapterValueKind::Path,
                    secret: false,
                    string_value: None,
                    path_value: Some("../boundline-framework-template".to_string()),
                    bool_value: None,
                    int_value: None,
                    value_source: AdapterValueSource::KnownProfileDefault,
                    resolution_state: StoredAdapterConfigValueState::Present,
                },
                AdapterConfigValueRecord {
                    field_key: "api_token".to_string(),
                    value_kind: AdapterValueKind::String,
                    secret: true,
                    string_value: Some("super-secret".to_string()),
                    path_value: None,
                    bool_value: None,
                    int_value: None,
                    value_source: AdapterValueSource::OperatorPrompt,
                    resolution_state: StoredAdapterConfigValueState::Present,
                },
            ],
        }),
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config)?;

    let workspace_view =
        execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Workspace))?;
    assert_eq!(workspace_view.exit_status, CommandExitStatus::Succeeded);
    assert!(workspace_view.terminal_output.contains("framework_adapter:"));
    assert!(workspace_view.terminal_output.contains("  adapter_id: speckit"));
    assert!(workspace_view.terminal_output.contains("  discovery_state: discovered_on_path"));
    assert!(workspace_view.terminal_output.contains("  interactive_resolution: false"));
    assert!(workspace_view.terminal_output.contains("  value_count: 2"));
    assert!(
        workspace_view
            .terminal_output
            .contains("  - template_repo: ../boundline-framework-template [known_profile_default]")
    );
    assert!(workspace_view.terminal_output.contains("  - api_token: <redacted> [operator_prompt]"));

    let effective_view =
        execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Effective))?;
    assert_eq!(effective_view.exit_status, CommandExitStatus::Succeeded);
    assert!(
        effective_view.terminal_output.contains("framework_adapter_status: configured [workspace]")
    );
    assert!(effective_view.terminal_output.contains("framework_adapter_id: speckit"));
    assert!(effective_view.terminal_output.contains("framework_adapter_value_count: 2"));
    assert!(effective_view.terminal_output.contains("- api_token: <redacted> [operator_prompt]"));

    Ok(())
}
