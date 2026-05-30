use std::path::PathBuf;

use boundline::cli::{Cli, ConfigSubcommand, DeveloperCommand};
use boundline::domain::configuration::{
    CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
    RouteSlot, RuntimeKind,
};
use boundline::domain::domain_templates::{DomainFamily, ExternalContextKind};
use clap::Parser;

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
