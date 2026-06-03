//! Update-report rendering helpers for the init/update CLI surface.

use std::collections::BTreeSet;
use std::path::Path;

use super::{
    AssistantHostKind, BundledModelCatalog, CANON_BOOTSTRAP_NOTE_LABEL, CanonBootstrapReadiness,
    CanonModeSelectionPreference, CommandExitStatus, GuidedRouteSelection, InitCommandReport,
    InitRequest, InitSuccessReportInputs, InitTemplate, RouteSetupLineInput, ScaffoldFileStatus,
    ScaffoldManifest, UpdatePlan, UpdatePlanAction, UpdatePlanEntry, UpdateTarget,
    WRITE_CONFIGURATION_PROMPT, assistant_host_capability_line, docs_export_root_display,
    format_assistant_host_list, init_doctor_command, init_global_inspect_command,
    init_inspect_command, init_install_doctor_command, route_setup_lines, scope_includes_global,
    scope_includes_workspace, template_label,
};

const DERIVED_INDEX_HYGIENE_SOURCE: &str = "boundline:derived_index";

fn collect_update_entries(plan: &UpdatePlan, action: UpdatePlanAction) -> Vec<&UpdatePlanEntry> {
    plan.entries.iter().filter(|entry| entry.action == action).collect()
}

fn render_update_plan_entry(entry: &UpdatePlanEntry) -> String {
    format!("- [{}] {}: {}", entry.action.label(), entry.path, entry.detail)
}

fn append_update_summary(lines: &mut Vec<String>, plan: &UpdatePlan) {
    lines.push("summary:".to_string());
    let mut emitted = false;
    for action in [
        UpdatePlanAction::Create,
        UpdatePlanAction::Replace,
        UpdatePlanAction::Merge,
        UpdatePlanAction::Adopt,
        UpdatePlanAction::AdoptCurrent,
        UpdatePlanAction::Orphaned,
        UpdatePlanAction::Remove,
        UpdatePlanAction::Conflict,
    ] {
        let count = plan.count_by_action(action);
        if count > 0 {
            emitted = true;
            lines.push(format!("- {}: {}", action.label(), count));
        }
    }
    if !emitted {
        lines.push("- unchanged: all managed scaffold artifacts are current".to_string());
    }
}

fn append_update_section(lines: &mut Vec<String>, heading: &str, entries: &[&UpdatePlanEntry]) {
    if entries.is_empty() {
        return;
    }

    lines.push(format!("{heading}:"));
    lines.extend(entries.iter().map(|entry| render_update_plan_entry(entry)));
}

fn render_update_manifest_state(
    existing_manifest: Option<&ScaffoldManifest>,
    plan: &UpdatePlan,
) -> &'static str {
    if existing_manifest.is_none() {
        "missing"
    } else if plan.requires_adopt() {
        "adoption_required"
    } else if plan.requires_force() {
        "drifted"
    } else if plan.count_by_action(UpdatePlanAction::Orphaned) > 0 {
        "orphaned"
    } else {
        "present"
    }
}

pub(super) fn render_update_preview_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        if plan.has_changes() || manifest_status != ScaffoldFileStatus::Unchanged {
            "update: preview only - workspace-managed scaffold changes detected".to_string()
        } else {
            "update: preview only - workspace-managed scaffold is current".to_string()
        },
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
        format!("manifest: {}", if plan.manifest_present { "present" } else { "missing" }),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    let change_entries =
        plan.entries.iter().filter(|entry| entry.action.is_change()).collect::<Vec<_>>();
    append_update_section(&mut lines, "planned_changes", &change_entries);
    lines.push("next_steps:".to_string());
    if plan.requires_adopt() {
        lines.push(
            "- rerun with --adopt --force --apply to baseline conflicting untracked managed files"
                .to_string(),
        );
    } else if plan.requires_force() {
        lines.push(
            "- rerun with --force --apply to overwrite changed tracked managed files".to_string(),
        );
    } else if plan.has_changes() || manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("- rerun with --apply to write the planned updates".to_string());
        if plan.count_by_action(UpdatePlanAction::Orphaned) > 0 {
            lines.push(
                "- rerun with --prune --apply to remove orphaned managed artifacts".to_string(),
            );
        }
    } else {
        lines.push("- no action required".to_string());
    }
    lines.join("\n")
}

pub(super) fn render_update_status_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    existing_manifest: Option<&ScaffoldManifest>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        format!("update_status: {}", render_update_manifest_state(existing_manifest, plan)),
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
        format!("manifest: {}", if plan.manifest_present { "present" } else { "missing" }),
        format!(
            "tracked_artifacts: {}",
            existing_manifest.map_or(0, |manifest| manifest.entries.len())
        ),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    append_update_section(
        &mut lines,
        "adoptions",
        &[
            collect_update_entries(plan, UpdatePlanAction::Adopt),
            collect_update_entries(plan, UpdatePlanAction::AdoptCurrent),
        ]
        .concat(),
    );
    append_update_section(
        &mut lines,
        "updates",
        &[
            collect_update_entries(plan, UpdatePlanAction::Create),
            collect_update_entries(plan, UpdatePlanAction::Replace),
            collect_update_entries(plan, UpdatePlanAction::Merge),
        ]
        .concat(),
    );
    append_update_section(
        &mut lines,
        "orphaned_artifacts",
        &[
            collect_update_entries(plan, UpdatePlanAction::Orphaned),
            collect_update_entries(plan, UpdatePlanAction::Remove),
        ]
        .concat(),
    );
    append_update_section(
        &mut lines,
        "conflicts",
        &collect_update_entries(plan, UpdatePlanAction::Conflict),
    );
    lines.push("next_steps:".to_string());
    if plan.requires_adopt() {
        lines.push(
            "- rerun with --adopt --force --apply to baseline conflicting untracked managed files"
                .to_string(),
        );
    } else if plan.requires_force() {
        lines.push(
            "- rerun with --force --apply to overwrite changed tracked managed files".to_string(),
        );
    } else if plan.count_by_action(UpdatePlanAction::Orphaned) > 0 {
        lines.push("- rerun with --prune --apply to remove orphaned managed artifacts".to_string());
    } else if plan.has_changes() || manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("- rerun with --apply to write the planned updates".to_string());
    } else {
        lines.push("- no action required".to_string());
    }
    lines.join("\n")
}

pub(super) fn render_update_adopt_required_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        "update: blocked - conflicting untracked managed files require --adopt".to_string(),
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    append_update_section(
        &mut lines,
        "conflicts",
        &collect_update_entries(plan, UpdatePlanAction::Conflict),
    );
    lines.push("next_steps:".to_string());
    lines.push(
        "- rerun with --adopt --force --apply to baseline conflicting untracked managed files"
            .to_string(),
    );
    lines.push("- rerun with --status to inspect the full scaffold health report".to_string());
    lines.join("\n")
}

pub(super) fn render_update_force_required_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        "update: blocked - changed replace-owned managed files require --force".to_string(),
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    append_update_section(
        &mut lines,
        "force_required",
        &plan.entries.iter().filter(|entry| entry.requires_force).collect::<Vec<_>>(),
    );
    lines.push("next_steps:".to_string());
    lines.push(
        "- rerun with --force --apply to overwrite changed managed scaffold files".to_string(),
    );
    lines.push("- rerun without --apply to inspect the preview again".to_string());
    lines.join("\n")
}

pub(super) fn render_update_applied_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        "update: workspace scaffold refreshed".to_string(),
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    append_update_section(
        &mut lines,
        "applied_changes",
        &plan
            .entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.action,
                    UpdatePlanAction::Create
                        | UpdatePlanAction::Replace
                        | UpdatePlanAction::Merge
                        | UpdatePlanAction::Adopt
                        | UpdatePlanAction::AdoptCurrent
                        | UpdatePlanAction::Remove
                )
            })
            .collect::<Vec<_>>(),
    );
    lines.join("\n")
}

pub(super) fn render_update_diff_report(
    workspace: &Path,
    targets: &BTreeSet<UpdateTarget>,
    plan: &UpdatePlan,
    manifest_path: &Path,
    manifest_status: ScaffoldFileStatus,
) -> String {
    let mut lines = vec![
        "update: workspace scaffold diff".to_string(),
        format!("workspace: {}", workspace.display()),
        format!("targets: {}", render_update_targets(targets)),
    ];
    append_update_summary(&mut lines, plan);
    if manifest_status != ScaffoldFileStatus::Unchanged {
        lines.push("manifest_changes:".to_string());
        lines.push(format!("- [{}] {}", manifest_status.label(), manifest_path.display()));
    }
    append_update_section(
        &mut lines,
        "workspace_diff",
        &plan.entries.iter().filter(|entry| entry.action.is_change()).collect::<Vec<_>>(),
    );
    lines.push("next_steps:".to_string());
    if plan.requires_adopt() {
        lines.push(
            "- rerun with --adopt --force --apply to baseline conflicting untracked managed files"
                .to_string(),
        );
    } else if plan.requires_force() {
        lines.push(
            "- rerun with --force --apply to overwrite changed tracked managed files".to_string(),
        );
    } else {
        lines.push("- rerun with --apply to write the planned updates".to_string());
    }
    lines.join("\n")
}

fn render_update_targets(targets: &BTreeSet<UpdateTarget>) -> String {
    targets.iter().map(|target| target.label()).collect::<Vec<_>>().join(", ")
}

/// Renders the late Canon bootstrap block, after preview planning has already
/// computed the changes that would have been applied.
pub(super) fn render_init_canon_bootstrap_blocked_report(
    request: &InitRequest<'_>,
    workspace: Option<&Path>,
    template: InitTemplate,
    canon_bootstrap: &CanonBootstrapReadiness,
    planned_changes: &[String],
) -> InitCommandReport {
    let mut lines = vec![
        "init: blocked - Canon surface not ready".to_string(),
        format!("scope: {}", request.scope),
    ];
    if scope_includes_workspace(request.scope) {
        lines.push(format!("template: {}", template_label(template)));
    }
    lines.push(format!("canon_bootstrap: {}", canon_bootstrap.state));
    lines.push(format!("canon_surface: {}", canon_bootstrap.detail));
    lines.push("repair_actions:".to_string());
    if canon_bootstrap.repair_actions.is_empty() {
        lines.push("- verify the Canon installation and rerun init".to_string());
    } else {
        lines.extend(canon_bootstrap.repair_actions.iter().map(|action| format!("- {action}")));
    }
    lines.push("planned_changes:".to_string());
    if planned_changes.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(planned_changes.iter().cloned());
    }
    lines.push("next_steps:".to_string());
    lines.push("- repair Canon and rerun the same init command".to_string());
    if let Some(workspace) = workspace {
        lines.push(format!("- verify workspace: {}", init_doctor_command(workspace)));
    }
    if scope_includes_global(request.scope) {
        lines.push(format!("- verify install: {}", init_install_doctor_command()));
    }
    InitCommandReport::new(CommandExitStatus::NonSuccess, lines.join("\n"))
}

/// Renders the preview-only stop when managed files already exist and the
/// operator did not pass `--force`.
pub(super) fn render_init_preview_only_report(
    request: &InitRequest<'_>,
    workspace: Option<&Path>,
    template: InitTemplate,
    planned_changes: &[String],
) -> InitCommandReport {
    let mut lines = vec![
        "init: preview only - existing Boundline files would be updated".to_string(),
        format!("scope: {}", request.scope),
        "why_stopped:".to_string(),
        "- existing Boundline configuration or selected scaffold outputs are already present"
            .to_string(),
        "- rerun the same command with --force to apply updates".to_string(),
        "planned_changes:".to_string(),
    ];
    if scope_includes_workspace(request.scope) {
        lines.insert(2, format!("template: {}", template_label(template)));
    }
    lines.extend(planned_changes.iter().cloned());
    lines.push("next_steps:".to_string());
    lines.push("- rerun the same command with --force".to_string());
    if scope_includes_workspace(request.scope)
        && let Some(workspace) = workspace
    {
        lines.push(format!("- inspect current config: {}", init_inspect_command(workspace)));
    }
    if scope_includes_global(request.scope) {
        lines.push(format!("- inspect global config: {}", init_global_inspect_command()));
    }
    InitCommandReport::new(CommandExitStatus::NonSuccess, lines.join("\n"))
}

/// Renders the final success report once `execute_init` has finished applying
/// every selected scaffold surface.
pub(super) fn render_successful_init_report(
    inputs: InitSuccessReportInputs<'_>,
) -> InitCommandReport {
    let InitSuccessReportInputs {
        scope,
        export_docs,
        docs_output_dir,
        resolved,
        workspace,
        global_config_path,
        global_env_template_path,
        execution_path,
        local_config_path,
        local_env_template_path,
        manifest_path,
        project_doc_roots,
        local_config,
        global_config,
        canon_bootstrap,
        canon_workspace_bootstrap,
        canon_init_assistant,
        assistant_actions,
        docs_actions,
        ide_actions,
        hygiene_actions,
    } = inputs;
    let capabilities = resolved
        .effective_assistants
        .iter()
        .map(|assistant| {
            format!("- {}: {}", assistant.as_str(), assistant_host_capability_line(*assistant))
        })
        .collect::<Vec<_>>();

    let summary_line = match scope {
        super::InitConfigScope::Global => "init: global configuration initialized",
        super::InitConfigScope::Workspace => "init: workspace initialized",
        super::InitConfigScope::Both => "init: configuration initialized",
    };
    let mut lines = vec![summary_line.to_string(), format!("scope: {scope}")];
    if scope_includes_workspace(scope) {
        lines.push(format!("template: {}", template_label(resolved.template)));
    }
    if let Some(path) = global_config_path {
        lines.push(format!("global_config: {}", path.display()));
    }
    if let Some(path) = global_env_template_path {
        lines.push(format!("global_provider_env_template: {}", path.display()));
    }
    if let Some(path) = execution_path {
        lines.push(format!("execution_profile: {}", path.display()));
    }
    if let Some(path) = local_config_path {
        lines.push(format!("workspace_config: {}", path.display()));
    }
    if let Some(path) = local_env_template_path {
        lines.push(format!("workspace_provider_env_template: {}", path.display()));
    }
    if let Some(path) = manifest_path {
        lines.push(format!("workspace_scaffold_manifest: {}", path.display()));
    }
    if let Some(doc_roots) = project_doc_roots {
        lines.push(format!(
            "project_memory_root: {}",
            doc_roots.project_memory.to_string_lossy().replace('\\', "/")
        ));
        lines.push(format!(
            "evidence_root: {}",
            doc_roots.evidence.to_string_lossy().replace('\\', "/")
        ));
    }

    if !capabilities.is_empty() {
        lines.push("runtime_capabilities:".to_string());
        lines.extend(capabilities);
    }

    lines.push("route_setup:".to_string());
    lines.extend(route_setup_lines(RouteSetupLineInput {
        catalog: &resolved.catalog,
        ollama_profile: resolved.ollama_profile,
        ollama_profile_routes: &resolved.ollama_profile_routes,
        effective_assistants: &resolved.effective_assistants,
        guided_answers: resolved.guided_answers.as_ref(),
        explicit_routes: &resolved.explicit_routes,
        guided_routes: &resolved.guided_routes,
        seeded_routes: &resolved.seeded_routes,
        inspect_command: scope_includes_workspace(scope)
            .then(|| workspace.map(init_inspect_command))
            .flatten()
            .unwrap_or_else(init_global_inspect_command),
    }));

    if let Some(canon) = local_config
        .and_then(|config| config.canon.as_ref())
        .or_else(|| global_config.and_then(|config| config.canon.as_ref()))
    {
        lines.push(format!("canon_mode_selection: {}", canon.mode_selection));
        if let Some(canon_bootstrap) = canon_bootstrap {
            lines.push(format!("canon_bootstrap: {}", canon_bootstrap.state));
            lines.push(format!("canon_surface: {}", canon_bootstrap.detail));
        }
        if let Some(canon_workspace_bootstrap) = canon_workspace_bootstrap {
            lines.push(format!("{CANON_BOOTSTRAP_NOTE_LABEL}:"));
            lines.push(format!("- canon_root: {}", canon_workspace_bootstrap.canon_root.display()));
            if let Some(assistant) = canon_init_assistant {
                lines.push(format!("- ai: {}", assistant.as_str()));
            }
            lines.push(format!(
                "- methods_materialized: {}",
                canon_workspace_bootstrap.methods_materialized
            ));
            lines.push(format!(
                "- policies_materialized: {}",
                canon_workspace_bootstrap.policies_materialized
            ));
            lines.push(format!(
                "- skills_materialized: {}",
                canon_workspace_bootstrap.skills_materialized
            ));
            lines.push(format!(
                "- claude_md_created: {}",
                canon_workspace_bootstrap.claude_md_created
            ));
        }
    }

    if scope_includes_workspace(scope) {
        if assistant_actions.is_empty() {
            lines.push("assistant_setup: none".to_string());
        } else {
            lines.push("assistant_package_scope: repo-local".to_string());
            lines.push(
                "assistant_global_bootstrap: use `boundline assistant install --host <host> --scope user` before workspace init"
                    .to_string(),
            );
            lines.push("assistant_setup:".to_string());
            lines.extend(assistant_actions.iter().map(|action| {
                format!(
                    "- {}: {} created, {} updated, {} unchanged",
                    action.surface.plan_label(),
                    action.created_files,
                    action.updated_files,
                    action.unchanged_files
                )
            }));
        }

        if export_docs {
            if docs_actions.is_empty() {
                lines.push("docs_export: none".to_string());
            } else {
                lines.push("docs_export:".to_string());
                lines.push(format!("- root: {}", docs_export_root_display(docs_output_dir)));
                lines.extend(docs_actions.iter().map(|action| {
                    format!(
                        "- {}: {} created, {} updated, {} unchanged",
                        action.surface.plan_label(),
                        action.created_files,
                        action.updated_files,
                        action.unchanged_files
                    )
                }));
            }
        }

        if ide_actions.is_empty() {
            lines.push("ide_setup: none".to_string());
        } else {
            lines.push("ide_setup:".to_string());
            lines.extend(ide_actions.iter().map(|action| {
                format!(
                    "- {}: {} {} path={}",
                    action.ide, action.setup_kind, action.status, action.path
                )
            }));
        }

        if let Some(local) = local_config {
            if local.routing.domain_templates.is_empty() {
                lines.push("domain_templates: none".to_string());
            } else {
                lines.push("domain_templates:".to_string());
                for (family, settings) in &local.routing.domain_templates {
                    lines.push(format!(
                        "- {}: enabled={}",
                        family.as_str(),
                        settings.enabled.unwrap_or(false)
                    ));
                    if let Some(standards) = settings.standards.as_deref().map(str::trim)
                        && !standards.is_empty()
                    {
                        lines.push(format!("  standards: {standards}"));
                    }
                    if !settings.external_context_bindings.is_empty() {
                        lines.push(format!(
                            "  external_context_bindings: {}",
                            settings.external_context_bindings.len()
                        ));
                    }
                }
            }
        }

        if hygiene_actions.is_empty() {
            lines.push("workspace_hygiene: none".to_string());
        } else {
            lines.push("workspace_hygiene:".to_string());
            lines.extend(hygiene_actions.iter().map(|action| {
                let sources = if action.sources.is_empty() {
                    "none".to_string()
                } else {
                    action.sources.join(",")
                };
                format!(
                    "- {}: {} added={} preserved={} sources={}",
                    action.path,
                    action.status,
                    action.added_patterns,
                    action.preserved_custom_lines,
                    sources
                )
            }));
            if hygiene_actions.iter().any(|action| {
                action.sources.iter().any(|source| source == DERIVED_INDEX_HYGIENE_SOURCE)
            }) {
                lines.push(
                    "derived_index_hygiene: disposable retrieval DB, manifest, and SQLite WAL/SHM sidecars stay ignored"
                        .to_string(),
                );
            }
        }
    } else {
        lines.push("workspace_artifacts: skipped in global scope".to_string());
    }

    lines.push("next_steps:".to_string());
    if let Some(workspace) = workspace {
        lines.push(format!("- inspect effective config: {}", init_inspect_command(workspace)));
        lines.push(format!("- verify workspace: {}", init_doctor_command(workspace)));
    }
    if scope_includes_global(scope) {
        lines.push(format!("- inspect global config: {}", init_global_inspect_command()));
        lines.push(format!("- verify install: {}", init_install_doctor_command()));
    }

    InitCommandReport::new(CommandExitStatus::Succeeded, lines.join("\n"))
}

pub(super) fn render_canon_workspace_bootstrap_failure_report(
    scope: super::InitConfigScope,
    workspace: &Path,
    template: InitTemplate,
    detail: &str,
    planned_changes: &[String],
) -> InitCommandReport {
    let mut lines = vec![
        "init: blocked - Canon workspace bootstrap failed".to_string(),
        format!("scope: {scope}"),
    ];
    if scope_includes_workspace(scope) {
        lines.push(format!("template: {}", template_label(template)));
    }
    lines.push("canon_bootstrap: blocked".to_string());
    lines.push(format!("{CANON_BOOTSTRAP_NOTE_LABEL}: {detail}"));
    lines.push("planned_changes:".to_string());
    if planned_changes.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(planned_changes.iter().cloned());
    }
    lines.push("next_steps:".to_string());
    lines.push(
        "- inspect `canon init --output json` in the workspace and rerun the same init command"
            .to_string(),
    );
    lines.push(format!("- verify workspace: {}", init_doctor_command(workspace)));
    InitCommandReport::new(CommandExitStatus::NonSuccess, lines.join("\n"))
}

pub(super) fn render_guided_summary(
    scope: super::InitConfigScope,
    template: InitTemplate,
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: &[AssistantHostKind],
    routes: &[GuidedRouteSelection],
    catalog: &BundledModelCatalog,
    planned_changes: &[String],
) -> String {
    let mut lines = vec![
        "Summary".to_string(),
        format!("Scope: {}", scope),
        format!("Template: {}", template_label(template)),
        format!("Catalog: {}", catalog.summary_label()),
        format!(
            "Canon approval mode: {}",
            canon_mode_selection.unwrap_or(CanonModeSelectionPreference::AutoConfirm)
        ),
    ];
    if assistants.is_empty() {
        lines.push("Assistant surfaces: none selected".to_string());
    } else {
        lines.push(format!("Assistant surfaces: {}", format_assistant_host_list(assistants)));
    }
    lines.push("Model routes:".to_string());
    lines.extend(routes.iter().map(|selection| format!("- {}", selection.display_line().trim())));
    lines.push("Planned changes:".to_string());
    lines.extend(planned_changes.iter().cloned());
    lines.push(WRITE_CONFIGURATION_PROMPT.to_string());
    lines.join("\n")
}

pub(super) fn render_cancelled_init_report(
    scope: super::InitConfigScope,
    workspace: Option<&Path>,
    template: InitTemplate,
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: &[AssistantHostKind],
    routes: &[GuidedRouteSelection],
    catalog: &BundledModelCatalog,
) -> String {
    let mut lines = vec![
        "init: canceled before write".to_string(),
        format!("scope: {}", scope),
        format!("template: {}", template_label(template)),
        format!("catalog: {}", catalog.summary_label()),
        format!(
            "canon_mode_selection: {}",
            canon_mode_selection.unwrap_or(CanonModeSelectionPreference::AutoConfirm)
        ),
    ];
    if assistants.is_empty() {
        lines.push("assistant_surfaces: none selected".to_string());
    } else {
        lines.push(format!("assistant_surfaces: {}", format_assistant_host_list(assistants)));
    }
    lines.push("route_setup:".to_string());
    lines.extend(routes.iter().map(|selection| format!("- {}", selection.display_line().trim())));
    lines.push("next_steps:".to_string());
    let mut rerun = format!("boundline init --scope {scope}");
    if scope_includes_workspace(scope) {
        let workspace = workspace.unwrap_or_else(|| Path::new("."));
        rerun.push_str(&format!(" --workspace {}", workspace.display()));
    }
    lines.push(format!("- rerun {rerun} to confirm and write the configuration"));
    lines.join("\n")
}
