use std::path::PathBuf;

use clap::Parser;
use synod::cli::{Cli, ConfigSubcommand, DeveloperCommand};
use synod::domain::configuration::{
    CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
    RouteSlot, RuntimeKind,
};

#[test]
fn config_show_accepts_effective_scope() {
    let cli = Cli::try_parse_from([
        "synod",
        "config",
        "show",
        "--workspace",
        "/tmp/ws",
        "--scope",
        "effective",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Config { command } => match command {
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
        "synod",
        "config",
        "set",
        "--scope",
        "workspace",
        "--slot",
        "planning",
        "--runtime",
        "codex",
        "--model",
        "gpt-5-codex",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Config { command } => match command {
            ConfigSubcommand::Set {
                workspace,
                cluster,
                scope,
                slot,
                reviewer,
                adjudicator,
                runtime,
                model,
            } => {
                assert!(workspace.is_none());
                assert!(cluster.is_none());
                assert_eq!(scope, ConfigWriteScope::Workspace);
                assert_eq!(slot, Some(RouteSlot::Planning));
                assert_eq!(reviewer, None);
                assert!(!adjudicator);
                assert_eq!(runtime, RuntimeKind::Codex);
                assert_eq!(model, "gpt-5-codex");
            }
            other => panic!("expected Set, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_set_capability_accepts_runtime_profile_fields() {
    let cli = Cli::try_parse_from([
        "synod",
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
        DeveloperCommand::Config { command } => match command {
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
        "synod",
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
        DeveloperCommand::Config { command } => match command {
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
