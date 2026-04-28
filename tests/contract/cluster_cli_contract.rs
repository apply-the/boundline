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
