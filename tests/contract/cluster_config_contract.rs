use std::path::PathBuf;

use boundline::cli::{Cli, ConfigSubcommand, DeveloperCommand};
use boundline::domain::configuration::{ConfigShowScope, ConfigWriteScope, RouteSlot, RuntimeKind};
use clap::Parser;

#[test]
fn cluster_scope_show_accepts_primary_workspace() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "show",
        "--cluster",
        "/tmp/primary",
        "--scope",
        "cluster",
    ])
    .unwrap();

    match cli.command {
        Some(DeveloperCommand::Config { command }) => match command {
            ConfigSubcommand::Show { workspace, cluster, scope } => {
                assert_eq!(workspace, None);
                assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
                assert_eq!(scope, Some(ConfigShowScope::Cluster));
            }
            other => panic!("expected Show, got {other:?}"),
        },
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn cluster_scope_set_accepts_cluster_slot_runtime_and_model() {
    let cli = Cli::try_parse_from([
        "boundline",
        "config",
        "set",
        "--cluster",
        "/tmp/primary",
        "--scope",
        "cluster",
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
                assert_eq!(workspace, None);
                assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
                assert_eq!(scope, ConfigWriteScope::Cluster);
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
