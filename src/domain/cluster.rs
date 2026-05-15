//! Cluster membership, follow-through authority, and cluster inspection models.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::RoutingConfig;
use crate::domain::session::SessionStatus;

/// Role of one workspace inside a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterMemberRole {
    Primary,
    Member,
}

/// Registration record for one cluster member workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberRegistration {
    pub workspace_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub role: ClusterMemberRole,
}

impl ClusterMemberRegistration {
    /// Validates the member registration.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.workspace_ref.trim().is_empty() {
            return Err(ClusterError::EmptyMemberWorkspace);
        }

        if let Some(display_name) = &self.display_name
            && display_name.trim().is_empty()
        {
            return Err(ClusterError::EmptyMemberDisplayName(self.workspace_ref.clone()));
        }

        Ok(())
    }
}

/// Persisted cluster membership definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCluster {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub members: Vec<ClusterMemberRegistration>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl WorkspaceCluster {
    /// Validates the cluster membership model.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.cluster_id.trim().is_empty() {
            return Err(ClusterError::MissingClusterId);
        }

        if self.primary_workspace_ref.trim().is_empty() {
            return Err(ClusterError::MissingPrimaryWorkspace);
        }

        if self.members.len() < 2 {
            return Err(ClusterError::NotEnoughMembers { count: self.members.len() });
        }

        if self.updated_at < self.created_at {
            return Err(ClusterError::UpdatedBeforeCreated {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }

        let mut seen = BTreeSet::new();
        let mut primary_count = 0usize;
        let mut contains_primary = false;

        for member in &self.members {
            member.validate()?;

            let canonical = member.workspace_ref.trim().to_string();
            if !seen.insert(canonical.clone()) {
                return Err(ClusterError::DuplicateMemberWorkspace(canonical));
            }

            if member.role == ClusterMemberRole::Primary {
                primary_count += 1;
                if member.workspace_ref.trim() == self.primary_workspace_ref.trim() {
                    contains_primary = true;
                }
            }
        }

        if primary_count != 1 {
            return Err(ClusterError::InvalidPrimaryMemberCount { count: primary_count });
        }

        if !contains_primary {
            return Err(ClusterError::PrimaryWorkspaceNotMember(
                self.primary_workspace_ref.clone(),
            ));
        }

        Ok(())
    }
}

/// Cluster projection embedded in session state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterSessionProjection {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub member_workspace_refs: Vec<String>,
    pub started_from_command: String,
    pub updated_at: u64,
}

impl ClusterSessionProjection {
    /// Validates the cluster session projection.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.cluster_id.trim().is_empty() {
            return Err(ClusterError::MissingClusterId);
        }

        if self.primary_workspace_ref.trim().is_empty() {
            return Err(ClusterError::MissingPrimaryWorkspace);
        }

        if self.member_workspace_refs.is_empty() {
            return Err(ClusterError::ProjectionMissingMembers);
        }

        if self.started_from_command.trim().is_empty() {
            return Err(ClusterError::ProjectionMissingCommand);
        }

        let mut seen = BTreeSet::new();
        let mut contains_primary = false;
        for workspace_ref in &self.member_workspace_refs {
            let canonical = workspace_ref.trim();
            if canonical.is_empty() {
                return Err(ClusterError::EmptyMemberWorkspace);
            }
            if !seen.insert(canonical.to_string()) {
                return Err(ClusterError::DuplicateMemberWorkspace(canonical.to_string()));
            }
            if canonical == self.primary_workspace_ref.trim() {
                contains_primary = true;
            }
        }

        if !contains_primary {
            return Err(ClusterError::PrimaryWorkspaceNotMember(
                self.primary_workspace_ref.clone(),
            ));
        }

        Ok(())
    }
}

/// Participation mode recorded for one workspace in a cluster story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceParticipationKind {
    Entry,
    ReadOnly,
    Mutated,
    Blocked,
    Skipped,
}

/// Participation record for one workspace inside a cluster delivery story.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceParticipationRecord {
    pub workspace_ref: String,
    pub participation_kind: WorkspaceParticipationKind,
    pub order: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_trace_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_status: Option<String>,
    pub headline: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
}

impl WorkspaceParticipationRecord {
    /// Validates the participation record.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.workspace_ref.trim().is_empty() {
            return Err(ClusterError::EmptyMemberWorkspace);
        }

        if self.headline.trim().is_empty() {
            return Err(ClusterError::MissingParticipationHeadline(self.workspace_ref.clone()));
        }

        Ok(())
    }
}

/// Authority surface that currently owns cluster follow-through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterAuthorityKind {
    ActiveSession,
    CompatibilityTrace,
    InspectOnly,
}

/// Route owner responsible for the authoritative cluster path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterRouteOwner {
    Native,
    Workflow,
    Review,
    Governance,
    Compatibility,
}

/// Follow-up authority for a clustered run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterFollowUpAuthority {
    pub authority_kind: ClusterAuthorityKind,
    pub route_owner: ClusterRouteOwner,
    pub authoritative_workspace_ref: String,
    pub continuity_reason: String,
    pub next_command: String,
}

impl ClusterFollowUpAuthority {
    /// Validates the follow-up authority projection.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.authoritative_workspace_ref.trim().is_empty() {
            return Err(ClusterError::MissingAuthoritativeWorkspace);
        }

        if self.continuity_reason.trim().is_empty() {
            return Err(ClusterError::MissingContinuityReason);
        }

        if self.next_command.trim().is_empty() {
            return Err(ClusterError::MissingNextCommand);
        }

        Ok(())
    }
}

/// Clustered execution condition shown in session and inspect output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusteredExecutionKind {
    Success,
    Paused,
    Blocked,
    Failed,
    Exhausted,
    InspectOnly,
}

/// Summary of the current execution condition across clustered workspaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusteredExecutionCondition {
    pub kind: ClusteredExecutionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_workspace_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_workspace_ref: Option<String>,
    pub summary: String,
    pub recovery_allowed: bool,
}

impl ClusteredExecutionCondition {
    /// Validates the clustered execution condition.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.summary.trim().is_empty() {
            return Err(ClusterError::MissingExecutionSummary);
        }

        if self.active_workspace_ref.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(ClusterError::MissingActiveWorkspace);
        }

        if self.blocking_workspace_ref.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(ClusterError::MissingBlockingWorkspace);
        }

        if matches!(self.kind, ClusteredExecutionKind::Blocked)
            && self.blocking_workspace_ref.is_none()
        {
            return Err(ClusterError::MissingBlockingWorkspace);
        }

        Ok(())
    }
}

/// Flattened cluster delivery story reused by status and inspect views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterDeliveryStory {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub authoritative_workspace_ref: String,
    pub route_owner: ClusterRouteOwner,
    pub member_workspace_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participating_workspaces: Vec<WorkspaceParticipationRecord>,
    pub started_from_command: String,
    pub execution_condition: ClusteredExecutionCondition,
    pub updated_at: u64,
}

impl ClusterDeliveryStory {
    /// Validates the cluster delivery story and its nested records.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.cluster_id.trim().is_empty() {
            return Err(ClusterError::MissingClusterId);
        }

        if self.primary_workspace_ref.trim().is_empty() {
            return Err(ClusterError::MissingPrimaryWorkspace);
        }

        if self.authoritative_workspace_ref.trim().is_empty() {
            return Err(ClusterError::MissingAuthoritativeWorkspace);
        }

        if self.member_workspace_refs.is_empty() {
            return Err(ClusterError::ProjectionMissingMembers);
        }

        if self.started_from_command.trim().is_empty() {
            return Err(ClusterError::ProjectionMissingCommand);
        }

        self.execution_condition.validate()?;

        let mut members = BTreeSet::new();
        let mut contains_primary = false;
        let mut contains_authoritative = false;
        for workspace_ref in &self.member_workspace_refs {
            let canonical = workspace_ref.trim();
            if canonical.is_empty() {
                return Err(ClusterError::EmptyMemberWorkspace);
            }
            if !members.insert(canonical.to_string()) {
                return Err(ClusterError::DuplicateMemberWorkspace(canonical.to_string()));
            }
            if canonical == self.primary_workspace_ref.trim() {
                contains_primary = true;
            }
            if canonical == self.authoritative_workspace_ref.trim() {
                contains_authoritative = true;
            }
        }

        if !contains_primary {
            return Err(ClusterError::PrimaryWorkspaceNotMember(
                self.primary_workspace_ref.clone(),
            ));
        }

        if !contains_authoritative {
            return Err(ClusterError::AuthoritativeWorkspaceNotMember(
                self.authoritative_workspace_ref.clone(),
            ));
        }

        let mut participating = BTreeSet::new();
        for record in &self.participating_workspaces {
            record.validate()?;
            if !members.contains(record.workspace_ref.trim()) {
                return Err(ClusterError::ParticipatingWorkspaceNotMember(
                    record.workspace_ref.clone(),
                ));
            }
            if !participating.insert(record.workspace_ref.trim().to_string()) {
                return Err(ClusterError::DuplicateParticipatingWorkspace(
                    record.workspace_ref.clone(),
                ));
            }
        }

        Ok(())
    }
}

/// Persisted cluster config file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    pub cluster: WorkspaceCluster,
    #[serde(default)]
    pub routing: RoutingConfig,
}

/// Health state used when inspecting each cluster member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterMemberState {
    Healthy,
    MissingSession,
    MissingTrace,
    Blocked,
    Invalid,
}

/// Status projection for one cluster member during cluster inspect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberStatusView {
    pub workspace_ref: String,
    pub state: ClusterMemberState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_status: Option<SessionStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_trace_ref: Option<String>,
    pub headline: String,
}

/// Cluster inspect report returned to the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterInspectReport {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub members: Vec<ClusterMemberStatusView>,
}

impl ClusterConfigFile {
    /// Validates the persisted cluster config file.
    pub fn validate(&self) -> Result<(), ClusterError> {
        self.cluster.validate()?;
        self.routing.validate().map_err(|error| ClusterError::InvalidRouting(error.to_string()))?;
        Ok(())
    }
}

impl Default for ClusterConfigFile {
    fn default() -> Self {
        Self {
            version: default_version(),
            cluster: WorkspaceCluster {
                cluster_id: String::new(),
                primary_workspace_ref: String::new(),
                members: Vec::new(),
                created_at: 0,
                updated_at: 0,
            },
            routing: RoutingConfig::default(),
        }
    }
}

const fn default_version() -> u32 {
    1
}

/// Validation errors for cluster membership, stories, and projections.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ClusterError {
    #[error("cluster id cannot be empty")]
    MissingClusterId,
    #[error("primary workspace cannot be empty")]
    MissingPrimaryWorkspace,
    #[error("cluster must contain at least two members, found {count}")]
    NotEnoughMembers { count: usize },
    #[error("cluster member workspace cannot be empty")]
    EmptyMemberWorkspace,
    #[error("cluster member display name cannot be empty for {0}")]
    EmptyMemberDisplayName(String),
    #[error("duplicate cluster member workspace: {0}")]
    DuplicateMemberWorkspace(String),
    #[error("cluster must contain exactly one primary member, found {count}")]
    InvalidPrimaryMemberCount { count: usize },
    #[error("primary workspace is not registered as the primary cluster member: {0}")]
    PrimaryWorkspaceNotMember(String),
    #[error("cluster updated_at {updated_at} cannot be earlier than created_at {created_at}")]
    UpdatedBeforeCreated { created_at: u64, updated_at: u64 },
    #[error("cluster session projection must include at least one member")]
    ProjectionMissingMembers,
    #[error("cluster session projection must record its triggering command")]
    ProjectionMissingCommand,
    #[error("cluster follow-up authority must name an authoritative workspace")]
    MissingAuthoritativeWorkspace,
    #[error("cluster follow-up authority must explain its continuity reason")]
    MissingContinuityReason,
    #[error("cluster follow-up authority must include a next command")]
    MissingNextCommand,
    #[error("clustered execution condition must include a summary")]
    MissingExecutionSummary,
    #[error("clustered execution condition must name an active workspace when present")]
    MissingActiveWorkspace,
    #[error("clustered execution condition must name a blocking workspace for blocked states")]
    MissingBlockingWorkspace,
    #[error("workspace participation for {0} must include a headline")]
    MissingParticipationHeadline(String),
    #[error("authoritative workspace is not registered as a cluster member: {0}")]
    AuthoritativeWorkspaceNotMember(String),
    #[error("participating workspace is not registered as a cluster member: {0}")]
    ParticipatingWorkspaceNotMember(String),
    #[error("duplicate participating workspace: {0}")]
    DuplicateParticipatingWorkspace(String),
    #[error("cluster routing is invalid: {0}")]
    InvalidRouting(String),
}

#[cfg(test)]
mod tests {
    use crate::domain::configuration::{ModelRoute, RoutingConfig, RuntimeKind};

    use super::{
        ClusterAuthorityKind, ClusterConfigFile, ClusterDeliveryStory, ClusterFollowUpAuthority,
        ClusterMemberRegistration, ClusterMemberRole, ClusterRouteOwner, ClusterSessionProjection,
        ClusteredExecutionCondition, ClusteredExecutionKind, WorkspaceCluster,
        WorkspaceParticipationKind, WorkspaceParticipationRecord,
    };

    fn valid_cluster() -> WorkspaceCluster {
        WorkspaceCluster {
            cluster_id: "delivery-a".to_string(),
            primary_workspace_ref: "/tmp/a".to_string(),
            members: vec![
                ClusterMemberRegistration {
                    workspace_ref: "/tmp/a".to_string(),
                    display_name: None,
                    role: ClusterMemberRole::Primary,
                },
                ClusterMemberRegistration {
                    workspace_ref: "/tmp/b".to_string(),
                    display_name: Some("backend".to_string()),
                    role: ClusterMemberRole::Member,
                },
            ],
            created_at: 1,
            updated_at: 1,
        }
    }

    fn valid_projection() -> ClusterSessionProjection {
        ClusterSessionProjection {
            cluster_id: "delivery-a".to_string(),
            primary_workspace_ref: "/tmp/a".to_string(),
            member_workspace_refs: vec!["/tmp/a".to_string(), "/tmp/b".to_string()],
            started_from_command: "boundline start --cluster /tmp/a".to_string(),
            updated_at: 1,
        }
    }

    fn valid_condition() -> ClusteredExecutionCondition {
        ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Success,
            active_workspace_ref: Some("/tmp/a".to_string()),
            blocking_workspace_ref: None,
            summary: "cluster completed".to_string(),
            recovery_allowed: false,
        }
    }

    fn valid_story() -> ClusterDeliveryStory {
        ClusterDeliveryStory {
            cluster_id: "delivery-a".to_string(),
            primary_workspace_ref: "/tmp/a".to_string(),
            authoritative_workspace_ref: "/tmp/a".to_string(),
            route_owner: ClusterRouteOwner::Native,
            member_workspace_refs: vec!["/tmp/a".to_string(), "/tmp/b".to_string()],
            participating_workspaces: vec![WorkspaceParticipationRecord {
                workspace_ref: "/tmp/a".to_string(),
                participation_kind: WorkspaceParticipationKind::Entry,
                order: 0,
                latest_trace_ref: Some("trace-1".to_string()),
                latest_status: Some("running".to_string()),
                headline: "primary workspace running".to_string(),
                terminal_reason: None,
            }],
            started_from_command: "boundline start --cluster /tmp/a".to_string(),
            execution_condition: valid_condition(),
            updated_at: 1,
        }
    }

    #[test]
    fn cluster_file_validation_rejects_default_state() {
        let cluster = ClusterConfigFile::default();
        assert!(cluster.validate().is_err());
    }

    #[test]
    fn workspace_cluster_validation_accepts_two_members_with_one_primary() {
        let cluster = valid_cluster();

        assert!(cluster.validate().is_ok());
    }

    #[test]
    fn cluster_membership_validation_covers_common_failure_paths() {
        let invalid_member = ClusterMemberRegistration {
            workspace_ref: "   ".to_string(),
            display_name: None,
            role: ClusterMemberRole::Member,
        };
        assert!(invalid_member.validate().is_err());

        let invalid_display = ClusterMemberRegistration {
            workspace_ref: "/tmp/c".to_string(),
            display_name: Some("  ".to_string()),
            role: ClusterMemberRole::Member,
        };
        assert!(invalid_display.validate().is_err());

        let mut duplicate_members = valid_cluster();
        duplicate_members.members[1].workspace_ref = "/tmp/a".to_string();
        assert!(duplicate_members.validate().is_err());

        let mut no_primary = valid_cluster();
        no_primary.members[0].role = ClusterMemberRole::Member;
        assert!(no_primary.validate().is_err());

        let mut primary_not_member = valid_cluster();
        primary_not_member.primary_workspace_ref = "/tmp/missing".to_string();
        assert!(primary_not_member.validate().is_err());

        let mut invalid_timestamps = valid_cluster();
        invalid_timestamps.updated_at = 0;
        assert!(invalid_timestamps.validate().is_err());
    }

    #[test]
    fn projection_and_follow_up_validation_cover_failure_paths() {
        let projection = valid_projection();
        assert!(projection.validate().is_ok());

        let mut no_members = valid_projection();
        no_members.member_workspace_refs.clear();
        assert!(no_members.validate().is_err());

        let mut missing_command = valid_projection();
        missing_command.started_from_command = "   ".to_string();
        assert!(missing_command.validate().is_err());

        let mut duplicate_member = valid_projection();
        duplicate_member.member_workspace_refs[1] = "/tmp/a".to_string();
        assert!(duplicate_member.validate().is_err());

        let follow_up = ClusterFollowUpAuthority {
            authority_kind: ClusterAuthorityKind::ActiveSession,
            route_owner: ClusterRouteOwner::Workflow,
            authoritative_workspace_ref: "/tmp/a".to_string(),
            continuity_reason: "continue from active workspace".to_string(),
            next_command: "boundline next --cluster /tmp/a".to_string(),
        };
        assert!(follow_up.validate().is_ok());

        let mut missing_workspace = follow_up.clone();
        missing_workspace.authoritative_workspace_ref = " ".to_string();
        assert!(missing_workspace.validate().is_err());

        let mut missing_reason = follow_up.clone();
        missing_reason.continuity_reason = " ".to_string();
        assert!(missing_reason.validate().is_err());

        let mut missing_command_follow_up = follow_up;
        missing_command_follow_up.next_command = " ".to_string();
        assert!(missing_command_follow_up.validate().is_err());
    }

    #[test]
    fn execution_condition_and_story_validation_cover_failure_paths() {
        let condition = valid_condition();
        assert!(condition.validate().is_ok());

        let mut missing_summary = valid_condition();
        missing_summary.summary = " ".to_string();
        assert!(missing_summary.validate().is_err());

        let mut missing_active = valid_condition();
        missing_active.active_workspace_ref = Some(" ".to_string());
        assert!(missing_active.validate().is_err());

        let blocked_without_workspace = ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Blocked,
            active_workspace_ref: Some("/tmp/a".to_string()),
            blocking_workspace_ref: None,
            summary: "blocked".to_string(),
            recovery_allowed: true,
        };
        assert!(blocked_without_workspace.validate().is_err());

        let story = valid_story();
        assert!(story.validate().is_ok());

        let mut missing_authoritative = valid_story();
        missing_authoritative.authoritative_workspace_ref = "/tmp/missing".to_string();
        assert!(missing_authoritative.validate().is_err());

        let mut non_member_participant = valid_story();
        non_member_participant.participating_workspaces.push(WorkspaceParticipationRecord {
            workspace_ref: "/tmp/c".to_string(),
            participation_kind: WorkspaceParticipationKind::Mutated,
            order: 1,
            latest_trace_ref: None,
            latest_status: None,
            headline: "mutated non-member workspace".to_string(),
            terminal_reason: None,
        });
        assert!(non_member_participant.validate().is_err());

        let mut duplicate_participant = valid_story();
        duplicate_participant.participating_workspaces.push(WorkspaceParticipationRecord {
            workspace_ref: "/tmp/a".to_string(),
            participation_kind: WorkspaceParticipationKind::Blocked,
            order: 1,
            latest_trace_ref: None,
            latest_status: None,
            headline: "duplicate participant".to_string(),
            terminal_reason: Some("waiting".to_string()),
        });
        assert!(duplicate_participant.validate().is_err());

        let invalid_participation = WorkspaceParticipationRecord {
            workspace_ref: "/tmp/a".to_string(),
            participation_kind: WorkspaceParticipationKind::Skipped,
            order: 0,
            latest_trace_ref: None,
            latest_status: None,
            headline: " ".to_string(),
            terminal_reason: None,
        };
        assert!(invalid_participation.validate().is_err());
    }

    #[test]
    fn cluster_config_validation_covers_success_and_invalid_routing() {
        let config = ClusterConfigFile {
            version: 1,
            cluster: valid_cluster(),
            routing: RoutingConfig::default(),
        };
        assert!(config.validate().is_ok());

        let invalid = ClusterConfigFile {
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "   ".to_string(),
                }),
                ..RoutingConfig::default()
            },
            ..config
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn projection_and_condition_validation_cover_blank_fields() {
        let mut missing_primary = valid_projection();
        missing_primary.primary_workspace_ref = " ".to_string();
        assert!(missing_primary.validate().is_err());

        let blank_blocking = ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Blocked,
            active_workspace_ref: Some("/tmp/a".to_string()),
            blocking_workspace_ref: Some(" ".to_string()),
            summary: "blocked".to_string(),
            recovery_allowed: true,
        };
        assert!(blank_blocking.validate().is_err());
    }
}
