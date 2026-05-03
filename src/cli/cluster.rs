use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::cluster::{
    ClusterConfigFile, ClusterInspectReport, ClusterMemberRegistration, ClusterMemberRole,
    ClusterMemberState, ClusterMemberStatusView, WorkspaceCluster,
};
use crate::domain::session::SessionStatus;
use crate::domain::trace::current_timestamp_millis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_init(
    workspace: &Path,
    cluster_id: &str,
    members: &[PathBuf],
) -> Result<ClusterCommandReport, ClusterCommandError> {
    validate_init_inputs(cluster_id)?;
    let primary_workspace = canonical_workspace(workspace)?;
    let cluster_store = FileClusterStore::for_workspace(&primary_workspace);

    if cluster_store.cluster_config_path().is_file() {
        return Err(ClusterCommandError::ClusterAlreadyExists(cluster_store.cluster_config_path()));
    }

    let normalized_members = normalize_members(&primary_workspace, members)?;

    let now = current_timestamp_millis();
    let config = ClusterConfigFile {
        version: 1,
        cluster: WorkspaceCluster {
            cluster_id: cluster_id.trim().to_string(),
            primary_workspace_ref: primary_workspace.to_string_lossy().into_owned(),
            members: normalized_members,
            created_at: now,
            updated_at: now,
        },
        ..ClusterConfigFile::default()
    };

    let cluster_path = cluster_store.save(&config)?;
    let members = config
        .cluster
        .members
        .iter()
        .map(|member| member.workspace_ref.clone())
        .collect::<Vec<_>>();

    Ok(ClusterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_cluster_init(
            &config.cluster.cluster_id,
            &cluster_path.display().to_string(),
            &members,
        ),
    })
}

pub fn execute_status(workspace: &Path) -> Result<ClusterCommandReport, ClusterCommandError> {
    let report = build_cluster_report(workspace, false)?;
    Ok(ClusterCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: output::render_cluster_status(&report),
    })
}

pub fn execute_inspect(workspace: &Path) -> Result<ClusterCommandReport, ClusterCommandError> {
    let report = build_cluster_report(workspace, true)?;
    let exit_status =
        if report.members.iter().any(|member| member.state != ClusterMemberState::Healthy) {
            CommandExitStatus::NonSuccess
        } else {
            CommandExitStatus::Succeeded
        };

    Ok(ClusterCommandReport {
        exit_status,
        terminal_output: output::render_cluster_inspect(&report),
    })
}

fn build_cluster_report(
    workspace: &Path,
    include_trace: bool,
) -> Result<ClusterInspectReport, ClusterCommandError> {
    let primary_workspace = canonical_workspace(workspace)?;
    let cluster_store = FileClusterStore::for_workspace(&primary_workspace);
    let config = cluster_store.load()?.ok_or_else(|| {
        ClusterCommandError::MissingClusterConfig(cluster_store.cluster_config_path())
    })?;

    let members = config
        .cluster
        .members
        .iter()
        .map(|member| summarize_member(member.workspace_ref.as_ref(), include_trace))
        .collect::<Vec<_>>();

    Ok(ClusterInspectReport {
        cluster_id: config.cluster.cluster_id,
        primary_workspace_ref: config.cluster.primary_workspace_ref,
        members,
    })
}

fn summarize_member(workspace_ref: &str, include_trace: bool) -> ClusterMemberStatusView {
    let workspace = PathBuf::from(workspace_ref);
    let session_store = FileSessionStore::for_workspace(&workspace);
    match session_store.load() {
        Ok(Some(record)) => {
            let latest_trace_ref = record.latest_trace_ref.clone().or_else(|| {
                FileTraceStore::for_workspace(&workspace)
                    .latest()
                    .ok()
                    .flatten()
                    .map(|path| path.to_string_lossy().into_owned())
            });

            if include_trace && latest_trace_ref.is_none() {
                return ClusterMemberStatusView {
                    workspace_ref: workspace_ref.to_string(),
                    state: ClusterMemberState::MissingTrace,
                    latest_status: Some(record.latest_status),
                    latest_trace_ref: None,
                    headline: "no trace recorded for this member".to_string(),
                };
            }

            let state = if matches!(
                record.latest_status,
                SessionStatus::Failed
                    | SessionStatus::Exhausted
                    | SessionStatus::Aborted
                    | SessionStatus::Invalid
            ) {
                ClusterMemberState::Blocked
            } else {
                ClusterMemberState::Healthy
            };

            ClusterMemberStatusView {
                workspace_ref: workspace_ref.to_string(),
                state,
                latest_status: Some(record.latest_status),
                latest_trace_ref,
                headline: format!("session is {}", session_status_text(record.latest_status)),
            }
        }
        Ok(None) => ClusterMemberStatusView {
            workspace_ref: workspace_ref.to_string(),
            state: ClusterMemberState::MissingSession,
            latest_status: None,
            latest_trace_ref: None,
            headline: "no active session found".to_string(),
        },
        Err(SessionStoreError::InvalidRecord(message)) => ClusterMemberStatusView {
            workspace_ref: workspace_ref.to_string(),
            state: ClusterMemberState::Invalid,
            latest_status: None,
            latest_trace_ref: None,
            headline: format!("invalid session record: {message}"),
        },
        Err(_) => ClusterMemberStatusView {
            workspace_ref: workspace_ref.to_string(),
            state: ClusterMemberState::Invalid,
            latest_status: None,
            latest_trace_ref: None,
            headline: "failed to read member session state".to_string(),
        },
    }
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn canonical_workspace(path: &Path) -> Result<PathBuf, ClusterCommandError> {
    let canonical = fs::canonicalize(path).map_err(|source| {
        ClusterCommandError::WorkspaceRead { path: path.to_path_buf(), source }
    })?;

    if !is_boundline_workspace(&canonical) {
        return Err(ClusterCommandError::NotBoundlineWorkspace(canonical));
    }

    Ok(canonical)
}

fn normalize_members(
    primary_workspace: &Path,
    members: &[PathBuf],
) -> Result<Vec<ClusterMemberRegistration>, ClusterCommandError> {
    if members.len() < 2 {
        return Err(ClusterCommandError::MemberCount { count: members.len() });
    }

    let mut normalized = Vec::with_capacity(members.len());
    let mut saw_primary = false;

    for member in members {
        let canonical = canonical_workspace(member)?;
        let role = if canonical == primary_workspace {
            saw_primary = true;
            ClusterMemberRole::Primary
        } else {
            ClusterMemberRole::Member
        };

        normalized.push(ClusterMemberRegistration {
            workspace_ref: canonical.to_string_lossy().into_owned(),
            display_name: None,
            role,
        });
    }

    if !saw_primary {
        return Err(ClusterCommandError::PrimaryWorkspaceMissing(primary_workspace.to_path_buf()));
    }

    Ok(normalized)
}

fn is_boundline_workspace(path: &Path) -> bool {
    path.join(".boundline/execution.json").is_file()
        || path.join(".boundline/config.toml").is_file()
        || path.join(".boundline/session.json").is_file()
}

#[derive(Debug, Error)]
pub enum ClusterCommandError {
    #[error("cluster id cannot be empty")]
    MissingClusterId,
    #[error("cluster init requires at least two --member paths, found {count}")]
    MemberCount { count: usize },
    #[error("primary workspace {0} must also appear in the --member list")]
    PrimaryWorkspaceMissing(PathBuf),
    #[error("workspace is not a valid Boundline workspace: {0}")]
    NotBoundlineWorkspace(PathBuf),
    #[error("failed to read workspace {path}: {source}")]
    WorkspaceRead { path: PathBuf, source: std::io::Error },
    #[error("cluster config already exists at {0}")]
    ClusterAlreadyExists(PathBuf),
    #[error("cluster config is missing at {0}")]
    MissingClusterConfig(PathBuf),
    #[error("cluster store error: {0}")]
    Store(#[from] ClusterStoreError),
    #[error("trace store error: {0}")]
    TraceStore(#[from] TraceStoreError),
}

impl ClusterCommandError {
    fn validate_cluster_id(cluster_id: &str) -> Result<(), Self> {
        if cluster_id.trim().is_empty() {
            return Err(Self::MissingClusterId);
        }
        Ok(())
    }
}

pub fn validate_init_inputs(cluster_id: &str) -> Result<(), ClusterCommandError> {
    ClusterCommandError::validate_cluster_id(cluster_id)
}
