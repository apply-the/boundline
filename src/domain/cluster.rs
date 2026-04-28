use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::RoutingConfig;
use crate::domain::session::SessionStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterMemberRole {
    Primary,
    Member,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberRegistration {
    pub workspace_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub role: ClusterMemberRole,
}

impl ClusterMemberRegistration {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceCluster {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub members: Vec<ClusterMemberRegistration>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl WorkspaceCluster {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterSessionProjection {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub member_workspace_refs: Vec<String>,
    pub started_from_command: String,
    pub updated_at: u64,
}

impl ClusterSessionProjection {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterConfigFile {
    #[serde(default = "default_version")]
    pub version: u32,
    pub cluster: WorkspaceCluster,
    #[serde(default)]
    pub routing: RoutingConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterMemberState {
    Healthy,
    MissingSession,
    MissingTrace,
    Blocked,
    Invalid,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterInspectReport {
    pub cluster_id: String,
    pub primary_workspace_ref: String,
    pub members: Vec<ClusterMemberStatusView>,
}

impl ClusterConfigFile {
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
    #[error("cluster routing is invalid: {0}")]
    InvalidRouting(String),
}

#[cfg(test)]
mod tests {
    use super::{
        ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, WorkspaceCluster,
    };

    #[test]
    fn cluster_file_validation_rejects_default_state() {
        let cluster = ClusterConfigFile::default();
        assert!(cluster.validate().is_err());
    }

    #[test]
    fn workspace_cluster_validation_accepts_two_members_with_one_primary() {
        let cluster = WorkspaceCluster {
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
        };

        assert!(cluster.validate().is_ok());
    }
}
