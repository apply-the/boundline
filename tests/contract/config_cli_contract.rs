use std::path::PathBuf;

use clap::Parser;
use synod::cli::{Cli, ConfigSubcommand, DeveloperCommand};
use synod::domain::configuration::{ConfigShowScope, ConfigWriteScope, RouteSlot, RuntimeKind};

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
            ConfigSubcommand::Show { workspace, scope } => {
                assert_eq!(workspace, Some(PathBuf::from("/tmp/ws")));
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
                scope,
                slot,
                reviewer,
                adjudicator,
                runtime,
                model,
            } => {
                assert!(workspace.is_none());
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
