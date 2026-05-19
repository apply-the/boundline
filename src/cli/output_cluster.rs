use super::session_status_text;
use crate::domain::cluster::{
    ClusterDeliveryStory, ClusterInspectReport, ClusterMemberState, ClusterRouteOwner,
    ClusteredExecutionKind, WorkspaceParticipationKind,
};

/// Renders the result of initializing a workspace cluster.
pub fn render_cluster_init(cluster_id: &str, cluster_path: &str, members: &[String]) -> String {
    let mut lines = vec![
        "cluster: initialized".to_string(),
        format!("cluster_id: {cluster_id}"),
        format!("cluster_file: {cluster_path}"),
        "members:".to_string(),
    ];
    for member in members {
        lines.push(format!("- {member}"));
    }
    lines.join("\n")
}

/// Renders the current status projection for a workspace cluster.
pub fn render_cluster_status(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: status".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let mut line =
            format!("- {} [{}]", member.workspace_ref, cluster_member_state_text(member.state));
        if let Some(status) = member.latest_status {
            line.push_str(&format!(" status={}", session_status_text(status)));
        }
        line.push_str(&format!(" {}", member.headline));
        lines.push(line);
    }

    lines.join("\n")
}

/// Renders the current inspect projection for a workspace cluster.
pub fn render_cluster_inspect(report: &ClusterInspectReport) -> String {
    let mut lines = vec![
        "cluster: inspect".to_string(),
        format!("cluster_id: {}", report.cluster_id),
        format!("primary_workspace: {}", report.primary_workspace_ref),
        "members:".to_string(),
    ];

    for member in &report.members {
        let trace_text = member.latest_trace_ref.as_deref().unwrap_or("<missing>");
        lines.push(format!(
            "- {} [{}] trace={} {}",
            member.workspace_ref,
            cluster_member_state_text(member.state),
            trace_text,
            member.headline
        ));
    }

    lines.join("\n")
}

pub(crate) fn render_cluster_story_lines(story: &ClusterDeliveryStory) -> Vec<String> {
    let mut lines = vec![
        format!("cluster_id: {}", story.cluster_id),
        format!("cluster_route_owner: {}", cluster_route_owner_text(story)),
        format!("cluster_authoritative_workspace: {}", story.authoritative_workspace_ref),
        format!(
            "cluster_execution_condition: {} - {}",
            cluster_execution_kind_text(story.execution_condition.kind),
            story.execution_condition.summary
        ),
    ];

    if let Some(blocking_workspace_ref) = &story.execution_condition.blocking_workspace_ref {
        lines.push(format!("cluster_blocking_workspace: {blocking_workspace_ref}"));
    }

    if !story.participating_workspaces.is_empty() {
        lines.push(format!(
            "cluster_participating_workspaces: {}",
            story
                .participating_workspaces
                .iter()
                .map(|record| format!(
                    "{} [{}]",
                    record.workspace_ref,
                    participation_kind_text(record.participation_kind)
                ))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    lines
}

fn cluster_execution_kind_text(kind: ClusteredExecutionKind) -> &'static str {
    match kind {
        ClusteredExecutionKind::Success => "success",
        ClusteredExecutionKind::Paused => "paused",
        ClusteredExecutionKind::Blocked => "blocked",
        ClusteredExecutionKind::Failed => "failed",
        ClusteredExecutionKind::Exhausted => "exhausted",
        ClusteredExecutionKind::InspectOnly => "inspect_only",
    }
}

fn participation_kind_text(kind: WorkspaceParticipationKind) -> &'static str {
    match kind {
        WorkspaceParticipationKind::Entry => "entry",
        WorkspaceParticipationKind::ReadOnly => "read_only",
        WorkspaceParticipationKind::Mutated => "mutated",
        WorkspaceParticipationKind::Blocked => "blocked",
        WorkspaceParticipationKind::Skipped => "skipped",
    }
}

fn cluster_route_owner_text(story: &ClusterDeliveryStory) -> &'static str {
    match story.route_owner {
        ClusterRouteOwner::Native => "native",
        ClusterRouteOwner::Workflow => "workflow",
        ClusterRouteOwner::Review => "review",
        ClusterRouteOwner::Governance => "governance",
        ClusterRouteOwner::Compatibility => "compatibility",
    }
}

fn cluster_member_state_text(state: ClusterMemberState) -> &'static str {
    match state {
        ClusterMemberState::Healthy => "healthy",
        ClusterMemberState::MissingSession => "missing-session",
        ClusterMemberState::MissingTrace => "missing-trace",
        ClusterMemberState::Blocked => "blocked",
        ClusterMemberState::Invalid => "invalid",
    }
}
