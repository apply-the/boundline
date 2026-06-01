use std::fs;
use std::path::PathBuf;

use boundline::adapters::config_store::FileConfigStore;
use boundline::cli::CommandExitStatus;
use boundline::cli::config::execute_show;
use boundline::cli::{Cli, ConfigSubcommand, DeveloperCommand};
use boundline::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, CapabilityState, ConfigFile, ConfigShowScope,
    ConfigWriteScope, EffortFallbackPolicy, EffortLevel, PersistedAdapterConfiguration, RouteSlot,
    RuntimeKind,
};
use boundline::domain::domain_templates::{DomainFamily, ExternalContextKind};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use clap::Parser;
use uuid::Uuid;

fn temp_workspace(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn config_show_accepts_effective_scope() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "show",
        "--workspace",
        "/tmp/ws",
        "--scope",
        "effective",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::Show { workspace, cluster, scope } => {
                assert_eq!(workspace, Some(PathBuf::from("/tmp/ws")));
                assert_eq!(cluster, None);
                assert_eq!(scope, Some(ConfigShowScope::Effective));
            }
            other => panic!("expected Show, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_set_accepts_workspace_slot_runtime_and_model() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "set",
        "--scope",
        "workspace",
        "--slot",
        "planning",
        "--runtime",
        "codex",
        "--model",
        "o4-mini",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::Set {
                workspace,
                cluster,
                scope,
                slot,
                chat,
                reviewer,
                adjudicator,
                runtime,
                model,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(slot, Some(RouteSlot::Planning));
                assert!(!chat);
                assert_eq!(reviewer, None);
                assert!(!adjudicator);
                assert_eq!(runtime, RuntimeKind::Codex);
                assert_eq!(model, "o4-mini");
            }
            other => panic!("expected Set, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_set_capability_accepts_runtime_profile_fields() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "set-capability",
        "--scope",
        "workspace",
        "--runtime",
        "codex",
        "--continuation",
        "supported",
        "--resume",
        "supported",
        "--validation",
        "unsupported",
        "--handoff-target",
        "supported",
        "--escalation-context",
        "supported",
        "--notes",
        "needs explicit validation handoff",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::SetCapability {
                workspace,
                cluster,
                scope,
                runtime,
                continuation,
                resume,
                validation,
                handoff_target,
                escalation_context,
                notes,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(runtime, RuntimeKind::Codex);
                assert_eq!(continuation, CapabilityState::Supported);
                assert_eq!(resume, CapabilityState::Supported);
                assert_eq!(validation, CapabilityState::Unsupported);
                assert_eq!(handoff_target, CapabilityState::Supported);
                assert_eq!(escalation_context, CapabilityState::Supported);
                assert_eq!(notes.as_deref(), Some("needs explicit validation handoff"));
            }
            other => panic!("expected SetCapability, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_set_effort_accepts_slot_level_policy() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "set-effort",
        "--scope",
        "workspace",
        "--slot",
        "planning",
        "--level",
        "high",
        "--fallback",
        "allow-lower",
        "--rationale",
        "planning should stay thorough",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::SetEffort {
                workspace,
                cluster,
                scope,
                slot,
                level,
                fallback,
                rationale,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(slot, RouteSlot::Planning);
                assert_eq!(level, EffortLevel::High);
                assert_eq!(fallback, EffortFallbackPolicy::AllowLower);
                assert_eq!(rationale.as_deref(), Some("planning should stay thorough"));
            }
            other => panic!("expected SetEffort, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_domain_commands_accept_family_and_binding_fields() {
    let set_domain = Cli::try_parse_from([
        "boundline",
        "config",
        "set-domain",
        "--scope",
        "workspace",
        "--family",
        "react",
        "--enable",
        "--standards",
        "follow the shared UI system",
    ])
    .unwrap();
    match set_domain.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::SetDomain {
                workspace,
                cluster,
                scope,
                family,
                enable,
                disable,
                standards,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(family, DomainFamily::React);
                assert!(enable);
                assert!(!disable);
                assert_eq!(standards.as_deref(), Some("follow the shared UI system"));
            }
            other => panic!("expected SetDomain, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }

    let bind_context = Cli::try_parse_from([
        "boundline",
        "config",
        "bind-context",
        "--scope",
        "workspace",
        "--family",
        "react",
        "--kind",
        "design-system",
        "--reference",
        "mcp:design-system",
        "--required",
        "--notes",
        "shared system",
    ])
    .unwrap();
    match bind_context.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::BindContext {
                workspace,
                cluster,
                scope,
                family,
                kind,
                reference,
                required,
                notes,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(family, DomainFamily::React);
                assert_eq!(kind, ExternalContextKind::DesignSystem);
                assert_eq!(reference, "mcp:design-system");
                assert!(required);
                assert_eq!(notes.as_deref(), Some("shared system"));
            }
            other => panic!("expected BindContext, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_show_redacts_secret_adapter_values() {
    let workspace = temp_workspace("boundline-config-show-redacted-adapter-values");
    let config = ConfigFile {
        adapter: Some(PersistedAdapterConfiguration {
            selection: AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::Custom,
                adapter_id: "custom-guided".to_string(),
                display_name: "Custom Guided".to_string(),
                command: "/bin/sh".to_string(),
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
            value_count: 2,
            values: vec![
                AdapterConfigValueRecord {
                    field_key: "workspace_slug".to_string(),
                    value_kind: AdapterValueKind::String,
                    secret: false,
                    string_value: Some("workspace-demo".to_string()),
                    path_value: None,
                    bool_value: None,
                    int_value: None,
                    value_source: AdapterValueSource::OperatorPrompt,
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
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let report =
        execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Workspace)).unwrap();

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains("  - workspace_slug: workspace-demo [operator_prompt]")
    );
    assert!(report.terminal_output.contains("  - api_token: <redacted> [operator_prompt]"));
    assert!(!report.terminal_output.contains("super-secret"));
}
