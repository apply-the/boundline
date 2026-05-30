use std::path::Path;

use serde_json::{Value, json};

use crate::adapters::checkpoint_store::{CheckpointCaptureRequest, FileCheckpointStore};
use crate::domain::checkpoint::CheckpointAuthorityScope;
use crate::domain::session::{ActiveSessionRecord, SessionCommand};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::current_timestamp_millis;
use crate::fixture::{FixtureRuntimeError, load_workspace_execution_profile};

use super::{SessionRuntime, SessionRuntimeError};

const LATEST_CHECKPOINT_ID_KEY: &str = "latest_checkpoint_id";
const LATEST_CHECKPOINT_SCOPE_KEY: &str = "latest_checkpoint_scope";
const LATEST_CHECKPOINT_RESTORE_COMMAND_KEY: &str = "latest_checkpoint_restore_command";
const LATEST_CHECKPOINT_WORKSPACES_KEY: &str = "latest_checkpoint_workspace_refs";
const LATEST_CHANGED_FILES_KEY: &str = "latest_changed_files";
const CHECKPOINT_SCOPE_CLUSTER: &str = "cluster";
const CHECKPOINT_SCOPE_WORKSPACE: &str = "workspace";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckpointCaptureScope {
    workspace_ref: String,
    authority_scope: CheckpointAuthorityScope,
    candidate_paths: Vec<String>,
    already_modified_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointProjectionState {
    pub(crate) checkpoint_id: String,
    pub(crate) scope: String,
    pub(crate) restore_command: String,
    pub(crate) workspace_refs: Vec<String>,
}

pub(super) fn checkpoint_event_payload(projection: &CheckpointProjectionState) -> Value {
    json!({
        "checkpoint_id": projection.checkpoint_id,
        "checkpoint_scope": projection.scope,
        "checkpoint_restore_command": projection.restore_command,
        "checkpoint_workspace_refs": projection.workspace_refs,
    })
}

pub(super) fn apply_checkpoint_projection_to_context(
    context: &mut TaskContext,
    projection: &CheckpointProjectionState,
) {
    context.state.insert(LATEST_CHECKPOINT_ID_KEY.to_string(), json!(projection.checkpoint_id));
    context.state.insert(LATEST_CHECKPOINT_SCOPE_KEY.to_string(), json!(projection.scope));
    context.state.insert(
        LATEST_CHECKPOINT_RESTORE_COMMAND_KEY.to_string(),
        json!(projection.restore_command),
    );
    context
        .state
        .insert(LATEST_CHECKPOINT_WORKSPACES_KEY.to_string(), json!(projection.workspace_refs));
}

pub(super) fn checkpoint_projection_from_context(
    context: &TaskContext,
) -> Option<CheckpointProjectionState> {
    let checkpoint_id = context.state.get(LATEST_CHECKPOINT_ID_KEY)?.as_str()?.to_string();
    let scope = context.state.get(LATEST_CHECKPOINT_SCOPE_KEY)?.as_str()?.to_string();
    let restore_command =
        context.state.get(LATEST_CHECKPOINT_RESTORE_COMMAND_KEY)?.as_str()?.to_string();
    let workspace_refs = context
        .state
        .get(LATEST_CHECKPOINT_WORKSPACES_KEY)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(CheckpointProjectionState { checkpoint_id, scope, restore_command, workspace_refs })
}

impl SessionRuntime {
    pub(crate) fn prepare_checkpoint_for_mutation(
        &self,
        session: &mut ActiveSessionRecord,
        trigger_command: SessionCommand,
    ) -> Result<Option<CheckpointProjectionState>, SessionRuntimeError> {
        let scopes = self.checkpoint_capture_scopes(session)?;
        if scopes.is_empty() {
            return Ok(None);
        }

        let group_id =
            (scopes.len() > 1).then(|| format!("checkpoint-group-{}", current_timestamp_millis()));
        let restore_id = group_id
            .clone()
            .unwrap_or_else(|| format!("checkpoint-{}", current_timestamp_millis()));
        let restore_command = if scopes.len() > 1 {
            format!(
                "boundline checkpoint restore {restore_id} --cluster {}",
                self.workspace_ref.display()
            )
        } else {
            format!(
                "boundline checkpoint restore {restore_id} --workspace {}",
                self.workspace_ref.display()
            )
        };

        let task_id = session
            .active_task
            .as_ref()
            .map(|task| task.id.clone())
            .or_else(|| session.goal_plan.as_ref().map(|goal_plan| goal_plan.plan_id.clone()));
        let step_id = session
            .active_task
            .as_ref()
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone()));

        for (index, scope) in scopes.iter().enumerate() {
            let checkpoint_id = group_id
                .clone()
                .map(|group_id| format!("{group_id}-{index}"))
                .unwrap_or_else(|| restore_id.clone());
            FileCheckpointStore::for_session(Path::new(&scope.workspace_ref), &session.session_id)
                .capture(CheckpointCaptureRequest {
                    checkpoint_id,
                    group_id: group_id.clone(),
                    workspace_ref: scope.workspace_ref.clone(),
                    authority_scope: scope.authority_scope,
                    trigger_command,
                    session_id: Some(session.session_id.clone()),
                    task_id: task_id.clone(),
                    step_id: step_id.clone(),
                    candidate_paths: scope.candidate_paths.clone(),
                    already_modified_paths: scope.already_modified_paths.clone(),
                })
                .map_err(SessionRuntimeError::CheckpointStore)?;
        }

        let projection = CheckpointProjectionState {
            checkpoint_id: restore_id,
            scope: if scopes.len() > 1 {
                CHECKPOINT_SCOPE_CLUSTER.to_string()
            } else {
                scopes
                    .first()
                    .map(|scope| scope.authority_scope.as_str().to_string())
                    .unwrap_or_else(|| CHECKPOINT_SCOPE_WORKSPACE.to_string())
            },
            restore_command,
            workspace_refs: scopes.iter().map(|scope| scope.workspace_ref.clone()).collect(),
        };

        if let Some(task) = session.active_task.as_mut() {
            apply_checkpoint_projection_to_context(&mut task.context, &projection);
        }

        Ok(Some(projection))
    }

    pub(crate) fn refresh_checkpoint_projection(
        &self,
        session: &ActiveSessionRecord,
        projection: &CheckpointProjectionState,
    ) -> Result<(), SessionRuntimeError> {
        if projection.workspace_refs.len() > 1 {
            for workspace_ref in &projection.workspace_refs {
                let store =
                    FileCheckpointStore::for_session(Path::new(workspace_ref), &session.session_id);
                for manifest in store
                    .load_group(&projection.checkpoint_id)
                    .map_err(SessionRuntimeError::CheckpointStore)?
                {
                    store
                        .refresh_observed_state(&manifest.checkpoint_id)
                        .map_err(SessionRuntimeError::CheckpointStore)?;
                }
            }
        } else if let Some(workspace_ref) = projection.workspace_refs.first() {
            FileCheckpointStore::for_session(Path::new(workspace_ref), &session.session_id)
                .refresh_observed_state(&projection.checkpoint_id)
                .map_err(SessionRuntimeError::CheckpointStore)?;
        }

        Ok(())
    }

    fn checkpoint_capture_scopes(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<Vec<CheckpointCaptureScope>, SessionRuntimeError> {
        let cluster_projection = session
            .active_task
            .as_ref()
            .and_then(|task| task.context.cluster_session_projection().ok().flatten())
            .or_else(|| {
                session
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.cluster_session_projection.clone())
            });

        if let Some(cluster_projection) = cluster_projection {
            let mut scopes = Vec::new();
            scopes.push(self.build_checkpoint_scope(
                &cluster_projection.primary_workspace_ref,
                CheckpointAuthorityScope::ClusterPrimary,
                session,
            )?);
            for member_workspace in &cluster_projection.member_workspace_refs {
                if member_workspace == &cluster_projection.primary_workspace_ref {
                    continue;
                }
                let scope = self.build_checkpoint_scope(
                    member_workspace,
                    CheckpointAuthorityScope::ClusterMember,
                    session,
                )?;
                if !scope.candidate_paths.is_empty() {
                    scopes.push(scope);
                }
            }
            return Ok(scopes
                .into_iter()
                .filter(|scope| !scope.candidate_paths.is_empty())
                .collect());
        }

        let scope = self.build_checkpoint_scope(
            &self.workspace_ref.to_string_lossy(),
            CheckpointAuthorityScope::Workspace,
            session,
        )?;
        Ok((!scope.candidate_paths.is_empty()).then_some(scope).into_iter().collect())
    }

    fn build_checkpoint_scope(
        &self,
        workspace_ref: &str,
        authority_scope: CheckpointAuthorityScope,
        session: &ActiveSessionRecord,
    ) -> Result<CheckpointCaptureScope, SessionRuntimeError> {
        let workspace = Path::new(workspace_ref);
        let mut candidate_paths = load_workspace_execution_profile(workspace)
            .map(|profile| {
                profile
                    .attempts
                    .into_iter()
                    .flat_map(|attempt| attempt.changes.into_iter().map(|change| change.path))
                    .collect::<Vec<_>>()
            })
            .or_else(|error| match error {
                FixtureRuntimeError::MissingExecutionProfile(_) => Ok(Vec::new()),
                other => Err(SessionRuntimeError::FixtureRuntime(other)),
            })?;

        let already_modified_paths = session
            .active_task
            .as_ref()
            .and_then(|task| {
                (task.context.workspace_ref == workspace_ref)
                    .then(|| task.context.state.get(LATEST_CHANGED_FILES_KEY))
                    .flatten()
            })
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if candidate_paths.is_empty() {
            candidate_paths = already_modified_paths.clone();
        }

        candidate_paths.sort();
        candidate_paths.dedup();

        Ok(CheckpointCaptureScope {
            workspace_ref: workspace_ref.to_string(),
            authority_scope,
            candidate_paths,
            already_modified_paths,
        })
    }
}
