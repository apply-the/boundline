use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::checkpoint_store::{
    CheckpointRestoreResult, CheckpointStoreError, FileCheckpointStore,
};
use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::cli::CommandExitStatus;
use crate::domain::checkpoint::{
    CheckpointManifest, CheckpointRestoreMode, CheckpointRestoreOutcome,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedCheckpointTarget {
    owner_workspace: PathBuf,
    member_workspaces: Vec<String>,
}

pub fn execute_list(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<CheckpointCommandReport, CheckpointCommandError> {
    let target = resolve_checkpoint_target(workspace, cluster, "checkpoint list")?;

    let terminal_output = if target.member_workspaces.is_empty() {
        let manifests = FileCheckpointStore::for_workspace(&target.owner_workspace)
            .list()
            .map_err(CheckpointCommandError::CheckpointStore)?;
        render_workspace_checkpoint_list(&target.owner_workspace, manifests)
    } else {
        let manifests = load_cluster_group_manifests(&target)?;
        render_cluster_checkpoint_list(&target, manifests)
    };

    Ok(CheckpointCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output })
}

pub fn execute_restore(
    checkpoint_id: &str,
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    force: bool,
) -> Result<CheckpointCommandReport, CheckpointCommandError> {
    let target = resolve_checkpoint_target(workspace, cluster, "checkpoint restore")?;
    let mode = if force { CheckpointRestoreMode::Forced } else { CheckpointRestoreMode::Safe };

    if target.member_workspaces.is_empty() {
        let result = FileCheckpointStore::for_workspace(&target.owner_workspace)
            .restore(checkpoint_id, mode)
            .map_err(CheckpointCommandError::CheckpointStore)?;
        let exit_status = match result.record.outcome {
            CheckpointRestoreOutcome::Succeeded => CommandExitStatus::Succeeded,
            CheckpointRestoreOutcome::Refused | CheckpointRestoreOutcome::Failed => {
                CommandExitStatus::NonSuccess
            }
        };
        return Ok(CheckpointCommandReport {
            exit_status,
            terminal_output: render_workspace_restore_result(
                &target.owner_workspace,
                checkpoint_id,
                &result,
            ),
        });
    }

    let manifests = load_cluster_group_manifests_for_restore(&target, checkpoint_id)?;
    if manifests.is_empty() {
        return Err(CheckpointCommandError::MissingCheckpoint(checkpoint_id.to_string()));
    }

    if mode == CheckpointRestoreMode::Safe {
        let mut conflicts = Vec::new();
        for manifest in &manifests {
            let store = FileCheckpointStore::for_workspace(Path::new(&manifest.workspace_ref));
            let manifest_conflicts = store
                .restore_conflicts(&manifest.checkpoint_id)
                .map_err(CheckpointCommandError::CheckpointStore)?
                .unwrap_or_default();
            if !manifest_conflicts.is_empty() {
                conflicts.push((manifest.clone(), manifest_conflicts));
            }
        }

        if !conflicts.is_empty() {
            let mut results = Vec::new();
            for (manifest, conflicting_paths) in conflicts {
                let store = FileCheckpointStore::for_workspace(Path::new(&manifest.workspace_ref));
                results.push(
                    store
                        .refuse_restore(&manifest.checkpoint_id, mode, conflicting_paths)
                        .map_err(CheckpointCommandError::CheckpointStore)?,
                );
            }

            return Ok(CheckpointCommandReport {
                exit_status: CommandExitStatus::NonSuccess,
                terminal_output: render_cluster_restore_results(
                    &target,
                    checkpoint_id,
                    mode,
                    &results,
                ),
            });
        }
    }

    let mut results = Vec::new();
    for manifest in manifests {
        let store = FileCheckpointStore::for_workspace(Path::new(&manifest.workspace_ref));
        results.push(
            store
                .restore(&manifest.checkpoint_id, mode)
                .map_err(CheckpointCommandError::CheckpointStore)?,
        );
    }

    Ok(CheckpointCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: render_cluster_restore_results(&target, checkpoint_id, mode, &results),
    })
}

fn resolve_checkpoint_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    command_name: &'static str,
) -> Result<ResolvedCheckpointTarget, CheckpointCommandError> {
    if let Some(cluster_workspace) = cluster {
        let requested_workspace = resolve_workspace(Some(cluster_workspace))?;
        let cluster_store = FileClusterStore::for_workspace(&requested_workspace);
        let Some(config) = cluster_store.load().map_err(CheckpointCommandError::ClusterStore)?
        else {
            return Err(CheckpointCommandError::MissingClusterConfig {
                workspace: requested_workspace,
                command_name,
            });
        };

        let owner_workspace =
            resolve_workspace(Some(Path::new(&config.cluster.primary_workspace_ref)))?;
        let mut member_workspaces = BTreeSet::new();
        member_workspaces.insert(owner_workspace.to_string_lossy().into_owned());
        for member in config.cluster.members {
            member_workspaces.insert(member.workspace_ref);
        }

        return Ok(ResolvedCheckpointTarget {
            owner_workspace,
            member_workspaces: member_workspaces.into_iter().collect(),
        });
    }

    Ok(ResolvedCheckpointTarget {
        owner_workspace: resolve_workspace(workspace)?,
        member_workspaces: Vec::new(),
    })
}

fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, CheckpointCommandError> {
    let candidate = match workspace {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => std::env::current_dir()?.join(path),
        None => std::env::current_dir()?,
    };

    Ok(candidate.canonicalize().unwrap_or(candidate))
}

fn load_cluster_group_manifests(
    target: &ResolvedCheckpointTarget,
) -> Result<Vec<CheckpointManifest>, CheckpointCommandError> {
    let mut manifests = Vec::new();
    for workspace_ref in &target.member_workspaces {
        manifests.extend(
            FileCheckpointStore::for_workspace(Path::new(workspace_ref))
                .list()
                .map_err(CheckpointCommandError::CheckpointStore)?
                .into_iter()
                .filter(|manifest| manifest.group_id.is_some()),
        );
    }
    manifests.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| left.checkpoint_id.cmp(&right.checkpoint_id))
    });
    Ok(manifests)
}

fn load_cluster_group_manifests_for_restore(
    target: &ResolvedCheckpointTarget,
    checkpoint_id: &str,
) -> Result<Vec<CheckpointManifest>, CheckpointCommandError> {
    let mut manifests = Vec::new();
    for workspace_ref in &target.member_workspaces {
        manifests.extend(
            FileCheckpointStore::for_workspace(Path::new(workspace_ref))
                .load_group(checkpoint_id)
                .map_err(CheckpointCommandError::CheckpointStore)?,
        );
    }
    manifests.sort_by(|left, right| left.workspace_ref.cmp(&right.workspace_ref));
    Ok(manifests)
}

fn render_workspace_checkpoint_list(
    workspace: &Path,
    manifests: Vec<CheckpointManifest>,
) -> String {
    let mut lines = vec![
        "checkpoint_scope: workspace".to_string(),
        format!("workspace_ref: {}", workspace.display()),
        format!("checkpoint_count: {}", manifests.len()),
    ];

    if manifests.is_empty() {
        lines.push("status: no checkpoints recorded".to_string());
        return lines.join("\n");
    }

    for manifest in manifests {
        lines.push(format!("checkpoint_id: {}", manifest.checkpoint_id));
        lines.push(format!("created_at: {}", manifest.created_at));
        lines.push(format!("checkpoint_scope: {}", manifest.authority_scope.as_str()));
        let files =
            manifest.captured_files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
        if !files.is_empty() {
            lines.push(format!("captured_files: {}", files.join(", ")));
        }
        lines.push(format!(
            "restore_command: boundline checkpoint restore {} --workspace {}",
            manifest.checkpoint_id,
            workspace.display()
        ));
    }

    lines.join("\n")
}

fn render_cluster_checkpoint_list(
    target: &ResolvedCheckpointTarget,
    manifests: Vec<CheckpointManifest>,
) -> String {
    let mut lines = vec![
        "checkpoint_scope: cluster".to_string(),
        format!("primary_workspace: {}", target.owner_workspace.display()),
    ];

    if manifests.is_empty() {
        lines.push("checkpoint_group_count: 0".to_string());
        lines.push("status: no clustered checkpoints recorded".to_string());
        return lines.join("\n");
    }

    let mut grouped = BTreeMap::<String, Vec<CheckpointManifest>>::new();
    for manifest in manifests {
        let key = manifest.group_id.clone().unwrap_or_else(|| manifest.checkpoint_id.clone());
        grouped.entry(key).or_default().push(manifest);
    }

    let mut groups = grouped.into_iter().collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        max_created_at(&right.1).cmp(&max_created_at(&left.1)).then_with(|| left.0.cmp(&right.0))
    });

    lines.push(format!("checkpoint_group_count: {}", groups.len()));
    for (group_id, mut group_manifests) in groups {
        group_manifests.sort_by(|left, right| left.workspace_ref.cmp(&right.workspace_ref));
        lines.push(format!("checkpoint_group: {group_id}"));
        lines.push(format!("created_at: {}", max_created_at(&group_manifests)));
        lines.push(format!(
            "restore_command: boundline checkpoint restore {group_id} --cluster {}",
            target.owner_workspace.display()
        ));
        for manifest in group_manifests {
            lines.push(format!(
                "workspace: {} [{}]",
                manifest.workspace_ref,
                manifest.authority_scope.as_str()
            ));
            let files =
                manifest.captured_files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
            if !files.is_empty() {
                lines.push(format!("captured_files: {}", files.join(", ")));
            }
        }
    }

    lines.join("\n")
}

fn render_workspace_restore_result(
    workspace: &Path,
    checkpoint_id: &str,
    result: &CheckpointRestoreResult,
) -> String {
    let mut lines = vec![
        "checkpoint_scope: workspace".to_string(),
        format!("workspace_ref: {}", workspace.display()),
        format!("checkpoint_id: {checkpoint_id}"),
        format!("restore_mode: {}", result.record.mode.as_str()),
        format!("restore_outcome: {}", result.record.outcome.as_str()),
    ];

    if !result.record.conflicting_paths.is_empty() {
        lines.push(format!("conflicting_paths: {}", result.record.conflicting_paths.join(", ")));
    }

    if !result.record.restored_paths.is_empty() {
        lines.push(format!("restored_paths: {}", result.record.restored_paths.join(", ")));
    }

    lines.join("\n")
}

fn render_cluster_restore_results(
    target: &ResolvedCheckpointTarget,
    checkpoint_id: &str,
    mode: CheckpointRestoreMode,
    results: &[CheckpointRestoreResult],
) -> String {
    let overall_outcome = if results
        .iter()
        .any(|result| result.record.outcome != CheckpointRestoreOutcome::Succeeded)
    {
        CheckpointRestoreOutcome::Refused
    } else {
        CheckpointRestoreOutcome::Succeeded
    };

    let mut lines = vec![
        "checkpoint_scope: cluster".to_string(),
        format!("primary_workspace: {}", target.owner_workspace.display()),
        format!("checkpoint_group: {checkpoint_id}"),
        format!("restore_mode: {}", mode.as_str()),
        format!("restore_outcome: {}", overall_outcome.as_str()),
    ];

    for result in results {
        lines.push(format!(
            "workspace: {} [{}] {}",
            result.manifest.workspace_ref,
            result.manifest.authority_scope.as_str(),
            result.record.outcome.as_str()
        ));
        if !result.record.conflicting_paths.is_empty() {
            lines
                .push(format!("conflicting_paths: {}", result.record.conflicting_paths.join(", ")));
        }
        if !result.record.restored_paths.is_empty() {
            lines.push(format!("restored_paths: {}", result.record.restored_paths.join(", ")));
        }
    }

    lines.join("\n")
}

fn max_created_at(manifests: &[CheckpointManifest]) -> u64 {
    manifests.iter().map(|manifest| manifest.created_at).max().unwrap_or_default()
}

#[derive(Debug, Error)]
pub enum CheckpointCommandError {
    #[error("failed to resolve the current workspace: {0}")]
    WorkspaceResolution(#[from] std::io::Error),
    #[error("cluster store operation failed: {0}")]
    ClusterStore(#[from] ClusterStoreError),
    #[error("checkpoint store operation failed: {0}")]
    CheckpointStore(#[from] CheckpointStoreError),
    #[error("`{command_name}` requires a valid cluster config in {}", workspace.display())]
    MissingClusterConfig { workspace: PathBuf, command_name: &'static str },
    #[error("checkpoint '{0}' was not found")]
    MissingCheckpoint(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use uuid::Uuid;

    use super::{execute_list, execute_restore};
    use crate::adapters::checkpoint_store::{CheckpointCaptureRequest, FileCheckpointStore};
    use crate::adapters::cluster_store::FileClusterStore;
    use crate::domain::checkpoint::{CheckpointAuthorityScope, CheckpointRestoreMode};
    use crate::domain::cluster::{
        ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, WorkspaceCluster,
    };
    use crate::domain::session::SessionCommand;

    fn temp_workspace(prefix: &str) -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join("src")).unwrap();
        workspace
    }

    fn capture_workspace_checkpoint(
        workspace: &std::path::Path,
        checkpoint_id: &str,
        group_id: Option<&str>,
        file_path: &str,
    ) {
        FileCheckpointStore::for_workspace(workspace)
            .capture(CheckpointCaptureRequest {
                checkpoint_id: checkpoint_id.to_string(),
                group_id: group_id.map(str::to_string),
                workspace_ref: workspace.to_string_lossy().into_owned(),
                authority_scope: CheckpointAuthorityScope::Workspace,
                trigger_command: SessionCommand::Run,
                session_id: Some("session-checkpoint".to_string()),
                task_id: Some("task-checkpoint".to_string()),
                step_id: None,
                candidate_paths: vec![file_path.to_string()],
                already_modified_paths: Vec::new(),
            })
            .unwrap();
    }

    fn save_cluster(primary: &Path, member: &Path) {
        FileClusterStore::for_workspace(primary)
            .save(&ClusterConfigFile {
                version: 1,
                cluster: WorkspaceCluster {
                    cluster_id: "cluster-a".to_string(),
                    primary_workspace_ref: primary.to_string_lossy().into_owned(),
                    members: vec![
                        ClusterMemberRegistration {
                            workspace_ref: primary.to_string_lossy().into_owned(),
                            display_name: None,
                            role: ClusterMemberRole::Primary,
                        },
                        ClusterMemberRegistration {
                            workspace_ref: member.to_string_lossy().into_owned(),
                            display_name: None,
                            role: ClusterMemberRole::Member,
                        },
                    ],
                    created_at: 1,
                    updated_at: 1,
                },
                ..ClusterConfigFile::default()
            })
            .unwrap();
    }

    #[test]
    fn execute_list_renders_workspace_checkpoints() {
        let workspace = temp_workspace("boundline-cli-checkpoint-list");
        fs::write(workspace.join("src/lib.rs"), "before").unwrap();
        capture_workspace_checkpoint(&workspace, "checkpoint-1", None, "src/lib.rs");

        let report = execute_list(Some(&workspace), None).unwrap();

        assert_eq!(report.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("checkpoint_scope: workspace"));
        assert!(report.terminal_output.contains("checkpoint_id: checkpoint-1"));
        assert!(report.terminal_output.contains("captured_files: src/lib.rs"));
    }

    #[test]
    fn execute_restore_renders_workspace_refused_and_forced_outcomes() {
        let workspace = temp_workspace("boundline-cli-checkpoint-workspace-restore");
        fs::write(workspace.join("src/lib.rs"), "before").unwrap();
        capture_workspace_checkpoint(&workspace, "checkpoint-restore", None, "src/lib.rs");

        FileCheckpointStore::for_workspace(&workspace)
            .refresh_observed_state("checkpoint-restore")
            .unwrap();
        fs::write(workspace.join("src/lib.rs"), "after-run").unwrap();
        FileCheckpointStore::for_workspace(&workspace)
            .refresh_observed_state("checkpoint-restore")
            .unwrap();
        fs::write(workspace.join("src/lib.rs"), "edited-after-run").unwrap();

        let refused = execute_restore("checkpoint-restore", Some(&workspace), None, false).unwrap();
        assert_eq!(refused.exit_status, crate::cli::CommandExitStatus::NonSuccess);
        assert!(refused.terminal_output.contains("checkpoint_scope: workspace"));
        assert!(refused.terminal_output.contains("restore_outcome: refused"));
        assert!(refused.terminal_output.contains("conflicting_paths: src/lib.rs"));

        let forced = execute_restore("checkpoint-restore", Some(&workspace), None, true).unwrap();
        assert_eq!(forced.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(forced.terminal_output.contains("restore_mode: forced"));
        assert!(forced.terminal_output.contains("restored_paths: src/lib.rs"));
        assert_eq!(fs::read_to_string(workspace.join("src/lib.rs")).unwrap(), "before");
    }

    #[test]
    fn cluster_listing_and_helper_paths_cover_empty_and_grouped_outputs() {
        let primary = temp_workspace("boundline-cli-checkpoint-list-primary");
        let member = temp_workspace("boundline-cli-checkpoint-list-member");
        save_cluster(&primary, &member);

        let target =
            super::resolve_checkpoint_target(None, Some(&primary), "checkpoint list").unwrap();
        let empty = super::render_cluster_checkpoint_list(&target, Vec::new());
        assert!(empty.contains("checkpoint_group_count: 0"));
        assert!(empty.contains("status: no clustered checkpoints recorded"));
        assert!(matches!(
            execute_restore("missing-group", None, Some(&primary), false),
            Err(super::CheckpointCommandError::MissingCheckpoint(id)) if id == "missing-group"
        ));

        fs::write(primary.join("src/lib.rs"), "before").unwrap();
        fs::write(member.join("src/member.rs"), "before").unwrap();
        capture_workspace_checkpoint(&primary, "group-1-primary", Some("group-1"), "src/lib.rs");
        capture_workspace_checkpoint(&member, "group-1-member", Some("group-1"), "src/member.rs");

        let manifests = super::load_cluster_group_manifests(&target).unwrap();
        assert!(
            manifests.iter().any(|manifest| manifest.workspace_ref == primary.to_string_lossy())
        );
        assert!(
            manifests.iter().any(|manifest| manifest.workspace_ref == member.to_string_lossy())
        );
        assert!(manifests.iter().all(|manifest| manifest.group_id.is_some()));
        assert!(super::max_created_at(&manifests) > 0);

        let restore_manifests =
            super::load_cluster_group_manifests_for_restore(&target, "group-1").unwrap();
        assert!(
            restore_manifests
                .iter()
                .all(|manifest| manifest.group_id.as_deref() == Some("group-1"))
        );
        assert!(
            restore_manifests
                .iter()
                .any(|manifest| manifest.workspace_ref == primary.to_string_lossy())
        );
        assert!(
            restore_manifests
                .iter()
                .any(|manifest| manifest.workspace_ref == member.to_string_lossy())
        );

        let report = execute_list(None, Some(&primary)).unwrap();
        assert_eq!(report.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("checkpoint_scope: cluster"));
        assert!(report.terminal_output.contains("checkpoint_group_count: 1"));
        assert!(report.terminal_output.contains("checkpoint_group: group-1"));
        assert!(
            report
                .terminal_output
                .contains("restore_command: boundline checkpoint restore group-1 --cluster")
        );
        assert!(report.terminal_output.contains("workspace: "));
    }

    #[test]
    fn helper_rendering_and_missing_cluster_config_are_explicit() {
        let workspace = temp_workspace("boundline-cli-checkpoint-missing-cluster");
        let error = super::resolve_checkpoint_target(None, Some(&workspace), "checkpoint list")
            .unwrap_err();
        assert!(matches!(
            error,
            super::CheckpointCommandError::MissingClusterConfig {
                command_name: "checkpoint list",
                ..
            }
        ));

        fs::write(workspace.join("src/lib.rs"), "before").unwrap();
        capture_workspace_checkpoint(&workspace, "checkpoint-helper", None, "src/lib.rs");
        let refused = FileCheckpointStore::for_workspace(&workspace)
            .refuse_restore(
                "checkpoint-helper",
                CheckpointRestoreMode::Safe,
                vec!["src/lib.rs".to_string()],
            )
            .unwrap();
        let rendered = super::render_cluster_restore_results(
            &super::ResolvedCheckpointTarget {
                owner_workspace: workspace.clone(),
                member_workspaces: vec![workspace.to_string_lossy().into_owned()],
            },
            "group-helper",
            CheckpointRestoreMode::Safe,
            &[refused],
        );
        assert!(rendered.contains("restore_outcome: refused"));
        assert!(rendered.contains("conflicting_paths: src/lib.rs"));
    }

    #[test]
    fn execute_restore_preflights_cluster_conflicts_without_partial_restore() {
        let primary = temp_workspace("boundline-cli-checkpoint-primary");
        let member = temp_workspace("boundline-cli-checkpoint-member");
        fs::write(primary.join("src/lib.rs"), "before").unwrap();
        fs::write(member.join("src/member.rs"), "before").unwrap();

        save_cluster(&primary, &member);

        capture_workspace_checkpoint(&primary, "group-1-primary", Some("group-1"), "src/lib.rs");
        capture_workspace_checkpoint(&member, "group-1-member", Some("group-1"), "src/member.rs");

        FileCheckpointStore::for_workspace(&primary)
            .refresh_observed_state("group-1-primary")
            .unwrap();
        FileCheckpointStore::for_workspace(&member)
            .refresh_observed_state("group-1-member")
            .unwrap();

        fs::write(primary.join("src/lib.rs"), "after-run-primary").unwrap();
        FileCheckpointStore::for_workspace(&primary)
            .refresh_observed_state("group-1-primary")
            .unwrap();
        fs::write(member.join("src/member.rs"), "after-run-member").unwrap();
        FileCheckpointStore::for_workspace(&member)
            .refresh_observed_state("group-1-member")
            .unwrap();
        fs::write(member.join("src/member.rs"), "edited-after-run").unwrap();

        let refused = execute_restore("group-1", None, Some(&primary), false).unwrap();
        assert_eq!(refused.exit_status, crate::cli::CommandExitStatus::NonSuccess);
        assert!(refused.terminal_output.contains("restore_outcome: refused"));
        assert!(refused.terminal_output.contains("conflicting_paths: src/member.rs"));
        assert_eq!(fs::read_to_string(primary.join("src/lib.rs")).unwrap(), "after-run-primary");

        let forced = execute_restore("group-1", None, Some(&primary), true).unwrap();
        assert_eq!(forced.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(forced.terminal_output.contains("restore_outcome: succeeded"));
        assert_eq!(fs::read_to_string(primary.join("src/lib.rs")).unwrap(), "before");
        assert_eq!(fs::read_to_string(member.join("src/member.rs")).unwrap(), "before");

        let forced_record = FileCheckpointStore::for_workspace(&member)
            .restore("group-1-member", CheckpointRestoreMode::Forced)
            .unwrap();
        assert_eq!(forced_record.record.mode.as_str(), "forced");
    }
}
