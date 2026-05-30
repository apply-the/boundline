//! Workspace preview and planned-change assembly for the init flow.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{
    AssistantAsset, AssistantInitAction, BOUNDLINE_DIR_RELATIVE, CanonInitAssistantHost,
    ConfigFile, DocsExportFileStatus, DocsExportPlanEntry, EXECUTION_PROFILE_FILE_NAME,
    FileConfigStore, InitCommandError, InitConfigScope, InitPreferenceOverrides, InitRequest,
    PlannedHygieneEntry, PlannedIdeEntry, ProviderEnvTemplateScope, ResolvedInitInputs,
    ScaffoldFileStatus, apply_init_preferences, apply_requested_domain_templates,
    assets_for_assistants, build_workspace_scaffold_manifest, canon_workspace_planned_changes,
    collect_workspace_scaffold_artifacts, docs_assets_for_assistants,
    docs_assets_for_assistants_under, load_scaffold_manifest, plan_assistant_setup,
    plan_docs_export, plan_docs_setup, plan_ide_setup, plan_workspace_hygiene_defaults,
    provider_workspace_env_template_path, render_execution_profile_contents,
    render_provider_env_template, resolve_ide_setup, scaffold_file_status, scaffold_manifest_path,
    scope_includes_workspace, serialize_scaffold_manifest, summarize_assistant_assets,
};

#[derive(Debug)]
pub(super) struct WorkspaceInitPreview {
    pub(super) local_config: ConfigFile,
    pub(super) local_config_path: PathBuf,
    pub(super) local_env_template_path: PathBuf,
    pub(super) local_env_template_contents: String,
    pub(super) local_env_template_status: ScaffoldFileStatus,
    pub(super) execution_path: PathBuf,
    pub(super) execution_contents: String,
    pub(super) execution_status: ScaffoldFileStatus,
    pub(super) config_status: ScaffoldFileStatus,
    pub(super) assistant_assets: Vec<AssistantAsset>,
    pub(super) assistant_actions_preview: Vec<AssistantInitAction>,
    pub(super) docs_plan: Vec<DocsExportPlanEntry>,
    pub(super) hygiene_plan: Vec<PlannedHygieneEntry>,
    pub(super) ide_plan: Vec<PlannedIdeEntry>,
    pub(super) manifest_path: PathBuf,
    pub(super) manifest_contents: String,
    pub(super) manifest_status: ScaffoldFileStatus,
}

impl WorkspaceInitPreview {
    pub(super) fn has_docs_refresh_conflicts(&self) -> bool {
        self.docs_plan.iter().any(|entry| entry.status != DocsExportFileStatus::Create)
    }

    pub(super) fn scaffold_updates_pending(&self) -> bool {
        self.execution_status == ScaffoldFileStatus::Update
            || self.config_status == ScaffoldFileStatus::Update
            || self.local_env_template_status == ScaffoldFileStatus::Update
            || self.assistant_actions_preview.iter().any(|action| action.updated_files > 0)
    }
}

pub(super) struct InitPlannedChangesInput<'a> {
    pub(super) scope: InitConfigScope,
    pub(super) requested_domain_template_count: usize,
    pub(super) workspace: Option<&'a Path>,
    pub(super) workspace_preview: Option<&'a WorkspaceInitPreview>,
    pub(super) global_status: Option<ScaffoldFileStatus>,
    pub(super) global_config_path: Option<&'a Path>,
    pub(super) global_env_template_status: Option<ScaffoldFileStatus>,
    pub(super) global_env_template_path: Option<&'a Path>,
    pub(super) workspace_canon_selected: bool,
    pub(super) canon_init_assistant: Option<CanonInitAssistantHost>,
}

/// Collects the operator-visible `planned_changes` section for `init` after the
/// preview phase has resolved which managed files and scaffold surfaces would
/// be created or updated.
pub(super) fn build_init_planned_changes(input: InitPlannedChangesInput<'_>) -> Vec<String> {
    let InitPlannedChangesInput {
        scope,
        requested_domain_template_count,
        workspace,
        workspace_preview,
        global_status,
        global_config_path,
        global_env_template_status,
        global_env_template_path,
        workspace_canon_selected,
        canon_init_assistant,
    } = input;
    let mut planned = Vec::new();
    push_init_planned_scaffold_change(&mut planned, global_status, global_config_path);
    push_init_planned_scaffold_change(
        &mut planned,
        global_env_template_status,
        global_env_template_path,
    );

    if let Some(preview) = workspace_preview {
        push_init_planned_scaffold_change(
            &mut planned,
            Some(preview.execution_status),
            Some(preview.execution_path.as_path()),
        );
        push_init_planned_scaffold_change(
            &mut planned,
            Some(preview.config_status),
            Some(preview.local_config_path.as_path()),
        );
        push_init_planned_scaffold_change(
            &mut planned,
            Some(preview.local_env_template_status),
            Some(preview.local_env_template_path.as_path()),
        );
        push_init_planned_scaffold_change(
            &mut planned,
            Some(preview.manifest_status),
            Some(preview.manifest_path.as_path()),
        );

        if requested_domain_template_count == 0 {
            planned.push("- leave domain templates unseeded".to_string());
        } else {
            planned.push(format!("- seed {requested_domain_template_count} domain template(s)"));
        }

        if preview.assistant_assets.is_empty() {
            planned.push("- skip assistant command-pack scaffolding".to_string());
        } else {
            planned.extend(plan_assistant_setup(&preview.assistant_actions_preview));
        }
        if !preview.docs_plan.is_empty() {
            planned.extend(plan_docs_setup(&preview.docs_plan));
        }
        if !preview.ide_plan.is_empty() {
            planned.extend(preview.ide_plan.iter().map(|entry| {
                format!(
                    "- scaffold {} IDE {} ({})",
                    entry.action.ide, entry.action.setup_kind, entry.action.path
                )
            }));
        }
        if workspace_canon_selected && let Some(workspace) = workspace {
            planned.extend(canon_workspace_planned_changes(workspace, canon_init_assistant));
        }
    } else if !scope_includes_workspace(scope) {
        planned.push("- leave workspace artifacts unchanged".to_string());
    }

    planned
}

fn push_init_planned_scaffold_change(
    planned: &mut Vec<String>,
    status: Option<ScaffoldFileStatus>,
    path: Option<&Path>,
) {
    if let (Some(status), Some(path)) = (status, path)
        && status != ScaffoldFileStatus::Unchanged
    {
        planned.push(format!("- {} {}", status.label(), path.display()));
    }
}

/// Builds the workspace-scoped init preview before `execute_init` decides
/// whether to stop at preview guards or apply the scaffold changes.
///
/// Keeping this assembly in one helper makes the command flow read as phase
/// orchestration: validate, preview, guard, apply, summarize.
pub(super) fn prepare_workspace_init_preview(
    workspace: &Path,
    request: &InitRequest<'_>,
    resolved: &ResolvedInitInputs,
) -> Result<WorkspaceInitPreview, InitCommandError> {
    let store = FileConfigStore::for_workspace(workspace);
    let boundline_dir = workspace.join(BOUNDLINE_DIR_RELATIVE);
    let execution_path = boundline_dir.join(EXECUTION_PROFILE_FILE_NAME);
    let local_config_path = store.local_config_path();
    let local_env_template_path = provider_workspace_env_template_path(workspace);
    let existing_manifest = load_scaffold_manifest(workspace)?;
    let ide_setup =
        resolve_ide_setup(request.ide, request.auto_approve, existing_manifest.as_ref());
    let existing_local = store.load_local()?;
    let mut local_config = existing_local.clone().unwrap_or_default();
    apply_init_preferences(
        &mut local_config,
        &resolved.catalog,
        &resolved.effective_assistants,
        &resolved.effective_routes,
        InitPreferenceOverrides {
            seed_canon_reviewer_routes: resolved.effective_canon_mode_selection.is_some(),
            canon_mode_selection: resolved.effective_canon_mode_selection,
            risk: request.risk,
            zone: request.zone,
            owner: request.owner,
        },
    );
    apply_requested_domain_templates(
        &mut local_config.routing.domain_templates,
        resolved.requested_domain_templates.clone(),
    );
    local_config
        .routing
        .validate()
        .map_err(|source| InitCommandError::InvalidDomainTemplate(source.to_string()))?;
    let active_domains = local_config
        .routing
        .domain_templates
        .iter()
        .filter_map(
            |(family, settings)| {
                if settings.enabled.unwrap_or(false) { Some(*family) } else { None }
            },
        )
        .collect::<BTreeSet<_>>();

    let local_config_contents = toml::to_string_pretty(&local_config).map_err(|source| {
        InitCommandError::SerializeConfigPreview { path: local_config_path.clone(), source }
    })?;
    let execution_contents =
        render_execution_profile_contents(resolved.template, local_config.canon.as_ref())?;
    let execution_status = scaffold_file_status(&execution_path, &execution_contents)?;
    let config_status = match existing_local.as_ref() {
        Some(saved) if saved == &local_config => ScaffoldFileStatus::Unchanged,
        Some(_) => ScaffoldFileStatus::Update,
        None => ScaffoldFileStatus::Create,
    };
    let local_env_template_contents =
        render_provider_env_template(ProviderEnvTemplateScope::Workspace);
    let local_env_template_status =
        scaffold_file_status(&local_env_template_path, &local_env_template_contents)?;

    let assistant_assets = assets_for_assistants(&resolved.effective_assistants);
    let assistant_actions_preview = summarize_assistant_assets(workspace, &assistant_assets)?;
    let hygiene_plan = plan_workspace_hygiene_defaults(workspace, &active_domains)?;
    let ide_plan = plan_ide_setup(workspace, &ide_setup)?;
    let docs_assets = if request.export_docs {
        match request.docs_output_dir {
            Some(docs_root) => {
                docs_assets_for_assistants_under(&resolved.effective_assistants, docs_root)
            }
            None => docs_assets_for_assistants(&resolved.effective_assistants),
        }
    } else {
        Vec::new()
    };
    let docs_plan =
        if request.export_docs { plan_docs_export(workspace, &docs_assets)? } else { Vec::new() };

    let manifest_path = scaffold_manifest_path(workspace);
    let manifest = build_workspace_scaffold_manifest(
        existing_manifest.as_ref(),
        Some(resolved.template),
        &collect_workspace_scaffold_artifacts(
            workspace,
            Some((&local_config_path, local_config_contents.as_str())),
            Some((&local_env_template_path, local_env_template_contents.as_str())),
            Some((&execution_path, execution_contents.as_str())),
            &assistant_assets,
            &docs_assets,
            &hygiene_plan,
            &ide_plan,
        ),
        &ide_setup,
    );
    let manifest_contents = serialize_scaffold_manifest(&manifest, &manifest_path)?;
    let manifest_status = scaffold_file_status(&manifest_path, &manifest_contents)?;

    Ok(WorkspaceInitPreview {
        local_config,
        local_config_path,
        local_env_template_path,
        local_env_template_contents,
        local_env_template_status,
        execution_path,
        execution_contents,
        execution_status,
        config_status,
        assistant_assets,
        assistant_actions_preview,
        docs_plan,
        hygiene_plan,
        ide_plan,
        manifest_path,
        manifest_contents,
        manifest_status,
    })
}
