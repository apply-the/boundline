use std::fs;

use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};

#[test]
fn cluster_init_persists_cluster_file_for_two_valid_members() {
    let primary = temp_fixture_workspace("synod-cluster-primary");
    let secondary = temp_fixture_workspace("synod-cluster-secondary");

    let output = run_synod_in(
        &primary,
        &[
            "cluster",
            "init",
            "--workspace",
            primary.to_string_lossy().as_ref(),
            "--cluster-id",
            "delivery-a",
            "--member",
            primary.to_string_lossy().as_ref(),
            "--member",
            secondary.to_string_lossy().as_ref(),
        ],
    );
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("cluster: initialized"), "{text}");

    let cluster_path = primary.join(".synod/cluster.toml");
    assert!(cluster_path.is_file());

    let cluster_contents = fs::read_to_string(cluster_path).unwrap();
    assert!(cluster_contents.contains("cluster_id = \"delivery-a\""));
    assert!(cluster_contents.contains(primary.to_string_lossy().as_ref()));
    assert!(cluster_contents.contains(secondary.to_string_lossy().as_ref()));
}

#[test]
fn cluster_init_rejects_non_synod_member_without_partial_state() {
    let primary = temp_fixture_workspace("synod-cluster-primary-invalid");
    let invalid_member = std::env::temp_dir().join("synod-non-member-invalid");
    let _ = fs::remove_dir_all(&invalid_member);
    fs::create_dir_all(&invalid_member).unwrap();

    let output = run_synod_in(
        &primary,
        &[
            "cluster",
            "init",
            "--workspace",
            primary.to_string_lossy().as_ref(),
            "--cluster-id",
            "delivery-a",
            "--member",
            primary.to_string_lossy().as_ref(),
            "--member",
            invalid_member.to_string_lossy().as_ref(),
        ],
    );
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("not a valid Synod workspace"), "{text}");
    assert!(!primary.join(".synod/cluster.toml").exists());
}
