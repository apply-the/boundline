use synod::cli::output;
use synod::domain::cluster::{ClusterInspectReport, ClusterMemberState, ClusterMemberStatusView};
use synod::domain::session::SessionStatus;

#[test]
fn render_cluster_status_includes_member_classifications() {
    let report = ClusterInspectReport {
        cluster_id: "delivery-a".to_string(),
        primary_workspace_ref: "/tmp/a".to_string(),
        members: vec![
            ClusterMemberStatusView {
                workspace_ref: "/tmp/a".to_string(),
                state: ClusterMemberState::Healthy,
                latest_status: Some(SessionStatus::Initialized),
                latest_trace_ref: None,
                headline: "session is initialized".to_string(),
            },
            ClusterMemberStatusView {
                workspace_ref: "/tmp/b".to_string(),
                state: ClusterMemberState::MissingSession,
                latest_status: None,
                latest_trace_ref: None,
                headline: "no active session found".to_string(),
            },
        ],
    };

    let text = output::render_cluster_status(&report);
    assert!(text.contains("cluster: status"), "{text}");
    assert!(text.contains("/tmp/a [healthy]"), "{text}");
    assert!(text.contains("/tmp/b [missing-session]"), "{text}");
}
