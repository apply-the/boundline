use synod::domain::cluster::{
    ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, ClusterSessionProjection,
    WorkspaceCluster,
};
use synod::domain::configuration::{ModelRoute, RoutingConfig, RuntimeKind};

fn valid_cluster() -> WorkspaceCluster {
    WorkspaceCluster {
        cluster_id: "delivery-a".to_string(),
        primary_workspace_ref: "/tmp/a".to_string(),
        members: vec![
            ClusterMemberRegistration {
                workspace_ref: "/tmp/a".to_string(),
                display_name: Some("primary".to_string()),
                role: ClusterMemberRole::Primary,
            },
            ClusterMemberRegistration {
                workspace_ref: "/tmp/b".to_string(),
                display_name: Some("secondary".to_string()),
                role: ClusterMemberRole::Member,
            },
        ],
        created_at: 10,
        updated_at: 10,
    }
}

#[test]
fn cluster_validation_accepts_one_primary_and_two_members() {
    assert!(valid_cluster().validate().is_ok());
}

#[test]
fn cluster_validation_rejects_duplicate_member_paths() {
    let mut cluster = valid_cluster();
    cluster.members[1].workspace_ref = "/tmp/a".to_string();

    let error = cluster.validate().unwrap_err();
    assert!(error.to_string().contains("duplicate cluster member workspace"));
}

#[test]
fn cluster_validation_rejects_primary_missing_from_members() {
    let mut cluster = valid_cluster();
    cluster.primary_workspace_ref = "/tmp/other".to_string();

    let error = cluster.validate().unwrap_err();
    assert!(error.to_string().contains("primary workspace is not registered"));
}

#[test]
fn projection_validation_requires_member_list_and_command() {
    let projection = ClusterSessionProjection {
        cluster_id: "delivery-a".to_string(),
        primary_workspace_ref: "/tmp/a".to_string(),
        member_workspace_refs: Vec::new(),
        started_from_command: String::new(),
        updated_at: 10,
    };

    let error = projection.validate().unwrap_err();
    assert!(error.to_string().contains("must include at least one member"));
}

#[test]
fn cluster_config_validation_reuses_routing_validation() {
    let config = ClusterConfigFile {
        version: 1,
        cluster: valid_cluster(),
        routing: RoutingConfig {
            planning: Some(ModelRoute { runtime: RuntimeKind::Codex, model: " ".to_string() }),
            ..RoutingConfig::default()
        },
    };

    let error = config.validate().unwrap_err();
    assert!(error.to_string().contains("cluster routing is invalid"));
}
