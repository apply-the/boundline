use std::path::PathBuf;

use clap::Parser;
use synod::cli::{Cli, ClusterSubcommand, DeveloperCommand};

#[test]
fn cluster_init_accepts_primary_workspace_cluster_id_and_members() {
    let cli = Cli::try_parse_from([
        "synod",
        "cluster",
        "init",
        "--workspace",
        "/tmp/a",
        "--cluster-id",
        "delivery-a",
        "--member",
        "/tmp/a",
        "--member",
        "/tmp/b",
    ])
    .unwrap();

    match cli.command {
        DeveloperCommand::Cluster { command } => match command {
            ClusterSubcommand::Init { workspace, cluster_id, member } => {
                assert_eq!(workspace, PathBuf::from("/tmp/a"));
                assert_eq!(cluster_id, "delivery-a");
                assert_eq!(member, vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);
            }
            other => panic!("expected Init, got {other:?}"),
        },
        other => panic!("expected Cluster, got {other:?}"),
    }
}

#[test]
fn cluster_status_and_inspect_accept_primary_workspace() {
    let status =
        Cli::try_parse_from(["synod", "cluster", "status", "--workspace", "/tmp/a"]).unwrap();
    let inspect =
        Cli::try_parse_from(["synod", "cluster", "inspect", "--workspace", "/tmp/a"]).unwrap();

    match status.command {
        DeveloperCommand::Cluster { command } => {
            assert!(
                matches!(command, ClusterSubcommand::Status { workspace } if workspace == std::path::Path::new("/tmp/a"))
            );
        }
        other => panic!("expected Cluster status, got {other:?}"),
    }

    match inspect.command {
        DeveloperCommand::Cluster { command } => {
            assert!(
                matches!(command, ClusterSubcommand::Inspect { workspace } if workspace == std::path::Path::new("/tmp/a"))
            );
        }
        other => panic!("expected Cluster inspect, got {other:?}"),
    }
}

#[test]
fn session_native_commands_accept_cluster_entrypoint() {
    let start = Cli::try_parse_from(["synod", "start", "--cluster", "/tmp/primary"]).unwrap();
    let capture = Cli::try_parse_from([
        "synod",
        "capture",
        "--cluster",
        "/tmp/primary",
        "--goal",
        "clustered delivery",
    ])
    .unwrap();
    let plan = Cli::try_parse_from(["synod", "plan", "--cluster", "/tmp/primary"]).unwrap();
    let status = Cli::try_parse_from(["synod", "status", "--cluster", "/tmp/primary"]).unwrap();

    match start.command {
        DeveloperCommand::Start { cluster, .. } => {
            assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
        }
        other => panic!("expected Start, got {other:?}"),
    }

    match capture.command {
        DeveloperCommand::Capture { cluster, goal, .. } => {
            assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
            assert_eq!(goal.as_deref(), Some("clustered delivery"));
        }
        other => panic!("expected Capture, got {other:?}"),
    }

    match plan.command {
        DeveloperCommand::Plan { cluster, .. } => {
            assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
        }
        other => panic!("expected Plan, got {other:?}"),
    }

    match status.command {
        DeveloperCommand::Status { cluster, .. } => {
            assert_eq!(cluster, Some(PathBuf::from("/tmp/primary")));
        }
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn session_run_preserves_workspace_requirement_for_custom_compatibility_mode() {
    let cli = Cli::try_parse_from([
        "synod",
        "run",
        "--cluster",
        "/tmp/primary",
        "--goal",
        "fix the failing add test",
    ])
    .unwrap();
    let session = synod::cli::DeveloperCommandSession::from_command(&cli.command);

    assert!(session.validate().is_err());
}
