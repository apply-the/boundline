use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use serde_json::json;
use thiserror::Error;

use super::assistant_assets::{AssistantAsset, AssistantSurface, assets_for_assistants};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{
    CanonPreferences, InitTemplate, ModelRoute, RouteSlot, RuntimeKind, built_in_default_route,
    seeded_routes_for_assistants,
};
use crate::domain::domain_templates::{
    DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
    detect_domain_families,
};
use crate::domain::governance::CanonModeSelectionPreference;
use crate::domain::workspace_hygiene::{merge_hygiene_content, plan_hygiene_defaults};

const INIT_ROUTE_EXAMPLE: &str = "planning=copilot:gpt-5.4";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SeededRouteSelection {
    slot: RouteSlot,
    route: ModelRoute,
    fallback_from_unavailable: Option<RuntimeKind>,
}

#[derive(Debug, Clone, Copy)]
pub struct InitRequest<'a> {
    pub workspace: &'a Path,
    pub template: Option<InitTemplate>,
    pub assistants: &'a [RuntimeKind],
    pub routes: &'a [String],
    pub domains: &'a [DomainFamily],
    pub domain_standards: &'a [String],
    pub context_bindings: &'a [String],
    pub required_context_bindings: &'a [String],
    pub canon_mode_selection: Option<CanonModeSelectionPreference>,
    pub risk: Option<&'a str>,
    pub zone: Option<&'a str>,
    pub owner: Option<&'a str>,
    pub force: bool,
}

pub fn execute_init(request: InitRequest<'_>) -> Result<InitCommandReport, InitCommandError> {
    let workspace = request.workspace;
    fs::create_dir_all(workspace).map_err(|source| InitCommandError::CreateWorkspace {
        path: workspace.to_path_buf(),
        source,
    })?;

    let template = request.template.unwrap_or(InitTemplate::BugFix);
    let requested_domain_templates = requested_domain_templates(
        workspace,
        request.domains,
        request.domain_standards,
        request.context_bindings,
        request.required_context_bindings,
    )?;
    let store = FileConfigStore::for_workspace(workspace);
    let execution_path = workspace.join(".boundline/execution.json");
    let local_config_path = store.local_config_path();

    let guided_answers = if io::stdin().is_terminal()
        && (request.canon_mode_selection.is_none()
            || request.assistants.is_empty()
            || request.routes.is_empty())
    {
        Some(collect_guided_init_answers(
            request.canon_mode_selection.is_none(),
            request.assistants.is_empty(),
            request.routes.is_empty(),
        )?)
    } else {
        None
    };
    let effective_canon_mode_selection = request
        .canon_mode_selection
        .or_else(|| guided_answers.as_ref().and_then(|answers| answers.canon_mode_selection));
    let effective_assistants = if request.assistants.is_empty() {
        guided_answers.as_ref().map(|answers| answers.assistants.clone()).unwrap_or_default()
    } else {
        request.assistants.to_vec()
    };
    let assistant_assets = assets_for_assistants(&effective_assistants);

    let mut planned = Vec::new();
    let execution_exists = execution_path.is_file();
    let config_exists = local_config_path.is_file();

    planned.push(if execution_exists {
        format!("- update {}", execution_path.display())
    } else {
        format!("- create {}", execution_path.display())
    });
    planned.push(if config_exists {
        format!("- update {}", local_config_path.display())
    } else {
        format!("- create {}", local_config_path.display())
    });
    if requested_domain_templates.is_empty() {
        planned.push("- leave domain templates unseeded".to_string());
    } else {
        planned.push(format!("- seed {} domain template(s)", requested_domain_templates.len()));
    }

    if assistant_assets.is_empty() {
        planned.push("- skip assistant command-pack scaffolding".to_string());
    } else {
        planned.extend(plan_assistant_setup(workspace, &assistant_assets));
    }

    let assistant_assets_exist =
        assistant_assets.iter().any(|asset| workspace.join(asset.relative_path).exists());

    if (execution_exists || config_exists || assistant_assets_exist) && !request.force {
        let inspect_command = init_inspect_command(workspace);
        let mut lines = vec![
            "init: preview only - existing Boundline files would be updated".to_string(),
            format!("template: {}", template_label(template)),
            "why_stopped:".to_string(),
            "- existing .boundline files or assistant assets are already present".to_string(),
            "- rerun the same command with --force to apply updates".to_string(),
            "planned_changes:".to_string(),
        ];
        lines.extend(planned);
        lines.push("next_steps:".to_string());
        lines.push("- rerun the same command with --force".to_string());
        lines.push(format!("- inspect current config: {inspect_command}"));
        return Ok(InitCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: lines.join("\n"),
        });
    }
    let explicit_routes = request
        .routes
        .iter()
        .map(|raw_route| parse_model_route(raw_route))
        .collect::<Result<Vec<_>, _>>()?;
    let guided_routes = if explicit_routes.is_empty()
        && let Some(answers) = guided_answers.as_ref()
    {
        answers.routes.clone()
    } else {
        Vec::new()
    };
    let mut effective_routes =
        if explicit_routes.is_empty() { guided_routes.clone() } else { explicit_routes.clone() };
    let explicit_slots = effective_routes.iter().map(|(slot, _)| *slot).collect::<BTreeSet<_>>();
    let seeded_routes =
        resolve_seeded_routes(&effective_assistants, &explicit_slots, runtime_available)?;
    effective_routes
        .extend(seeded_routes.iter().map(|selection| (selection.slot, selection.route.clone())));

    if let Some(parent) = execution_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| InitCommandError::WriteFile { path: parent.to_path_buf(), source })?;
    }

    let mut local = store.load_local()?.unwrap_or_default();
    local.routing.assistant_runtimes = effective_assistants.clone();
    for (slot, route) in &effective_routes {
        local.routing.set_slot(*slot, route.clone());
    }
    if effective_canon_mode_selection.is_some()
        || request.risk.is_some()
        || request.zone.is_some()
        || request.owner.is_some()
    {
        let mut canon = local.canon.unwrap_or(CanonPreferences {
            mode_selection: effective_canon_mode_selection.unwrap_or_default(),
            default_risk: None,
            default_zone: None,
            default_owner: None,
            default_system_context: None,
        });
        if let Some(mode_selection) = effective_canon_mode_selection {
            canon.mode_selection = mode_selection;
        }
        if let Some(risk) = request.risk.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_risk = Some(risk.to_string());
        }
        if let Some(zone) = request.zone.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_zone = Some(zone.to_string());
        }
        if let Some(owner) = request.owner.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_owner = Some(owner.to_string());
        }
        local.canon = Some(canon);
    }
    apply_requested_domain_templates(
        &mut local.routing.domain_templates,
        requested_domain_templates,
    );
    local
        .routing
        .validate()
        .map_err(|source| InitCommandError::InvalidDomainTemplate(source.to_string()))?;
    let active_domains = local
        .routing
        .domain_templates
        .iter()
        .filter_map(
            |(family, settings)| {
                if settings.enabled.unwrap_or(false) { Some(*family) } else { None }
            },
        )
        .collect::<BTreeSet<_>>();
    let hygiene_actions = apply_workspace_hygiene_defaults(workspace, &active_domains)?;

    let execution = execution_template(template, local.canon.as_ref());
    fs::write(
        &execution_path,
        serde_json::to_string_pretty(&execution).expect("execution template should serialize"),
    )
    .map_err(|source| InitCommandError::WriteFile { path: execution_path.clone(), source })?;

    store.save_local(&local)?;
    let assistant_actions = apply_assistant_assets(workspace, &assistant_assets)?;

    let capabilities = effective_assistants
        .iter()
        .map(|runtime| format!("- {}: {}", runtime.as_str(), runtime_capability_line(*runtime)))
        .collect::<Vec<_>>();
    let inspect_command = init_inspect_command(workspace);
    let doctor_command = init_doctor_command(workspace);

    let mut lines = vec![
        "init: workspace initialized".to_string(),
        format!("template: {}", template_label(template)),
        format!("execution_profile: {}", execution_path.display()),
        format!("workspace_config: {}", local_config_path.display()),
    ];

    if !capabilities.is_empty() {
        lines.push("runtime_capabilities:".to_string());
        lines.extend(capabilities);
    }

    let explicit_route_lines =
        explicit_routes.iter().chain(guided_routes.iter()).collect::<Vec<_>>();
    lines.push("route_setup:".to_string());
    if effective_assistants.is_empty() {
        lines.push(
            "- assistant_defaults: none selected; no assistant-seeded workspace routes were recorded"
                .to_string(),
        );
    } else {
        lines.push(format!("- assistant_defaults: {}", format_runtime_list(&effective_assistants)));
    }
    if seeded_routes.is_empty() && explicit_route_lines.is_empty() {
        lines.push(
            "- workspace-local routes: none recorded; add --assistant or --route later to pin workspace-specific defaults"
                .to_string(),
        );
    } else {
        lines.extend(seeded_routes.iter().map(|selection| {
            let provenance = match selection.fallback_from_unavailable {
                Some(runtime) => {
                    format!("assistant-default fallback-from={runtime}-unavailable")
                }
                None => "assistant-default".to_string(),
            };
            format!(
                "- seeded {}: {}:{} [{provenance}]",
                selection.slot.as_str(),
                selection.route.runtime,
                selection.route.model
            )
        }));
        lines.extend(explicit_route_lines.iter().map(|(slot, route)| {
            format!("- explicit {}: {}:{} [explicit]", slot.as_str(), route.runtime, route.model)
        }));
    }
    lines.push(format!("- inspect_or_edit: {inspect_command}"));

    if let Some(canon) = local.canon.as_ref() {
        lines.push(format!("canon_mode_selection: {}", canon.mode_selection));
    }

    if assistant_actions.is_empty() {
        lines.push("assistant_setup: none".to_string());
    } else {
        lines.push("assistant_setup:".to_string());
        lines.extend(assistant_actions.iter().map(|action| {
            format!(
                "- {}: {} created={} updated={} unchanged={}",
                action.surface.plan_label(),
                action.status,
                action.created_files,
                action.updated_files,
                action.unchanged_files
            )
        }));
    }

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
    }

    lines.push("next_steps:".to_string());
    lines.push(format!("- inspect effective config: {inspect_command}"));
    lines.push(format!("- verify workspace: {doctor_command}"));

    Ok(InitCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: lines.join("\n"),
    })
}

fn requested_domain_templates(
    workspace: &Path,
    domains: &[DomainFamily],
    domain_standards: &[String],
    context_bindings: &[String],
    required_context_bindings: &[String],
) -> Result<BTreeMap<DomainFamily, DomainTemplateSettings>, InitCommandError> {
    let mut templates = BTreeMap::<DomainFamily, DomainTemplateSettings>::new();

    let requested_families =
        if domains.is_empty() { detect_domain_families(workspace, None) } else { domains.to_vec() };
    for family in requested_families {
        templates.entry(family).or_default().enabled = Some(true);
    }

    for raw in domain_standards {
        let (family, standards) = parse_domain_standard(raw)?;
        let settings = templates.entry(family).or_default();
        settings.enabled.get_or_insert(true);
        settings.standards = Some(standards);
    }

    for raw in context_bindings {
        let (family, binding) = parse_context_binding(raw, false)?;
        let settings = templates.entry(family).or_default();
        settings.enabled.get_or_insert(true);
        upsert_binding(&mut settings.external_context_bindings, binding);
    }
    for raw in required_context_bindings {
        let (family, binding) = parse_context_binding(raw, true)?;
        let settings = templates.entry(family).or_default();
        settings.enabled.get_or_insert(true);
        upsert_binding(&mut settings.external_context_bindings, binding);
    }

    for settings in templates.values() {
        settings
            .validate()
            .map_err(|source| InitCommandError::InvalidDomainTemplate(source.to_string()))?;
    }

    Ok(templates)
}

fn apply_requested_domain_templates(
    existing: &mut BTreeMap<DomainFamily, DomainTemplateSettings>,
    requested: BTreeMap<DomainFamily, DomainTemplateSettings>,
) {
    for (family, settings) in requested {
        let target = existing.entry(family).or_default();
        if let Some(enabled) = settings.enabled {
            target.enabled = Some(enabled);
        }
        if let Some(standards) = settings.standards {
            target.standards = Some(standards);
        }
        for binding in settings.external_context_bindings {
            upsert_binding(&mut target.external_context_bindings, binding);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HygieneInitAction {
    path: String,
    status: &'static str,
    added_patterns: usize,
    preserved_custom_lines: usize,
    sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AssistantInitAction {
    surface: AssistantSurface,
    status: &'static str,
    created_files: usize,
    updated_files: usize,
    unchanged_files: usize,
}

fn plan_assistant_setup(workspace: &Path, assistant_assets: &[&AssistantAsset]) -> Vec<String> {
    let mut grouped = BTreeMap::<AssistantSurface, Vec<&AssistantAsset>>::new();
    for asset in assistant_assets {
        let asset = *asset;
        grouped.entry(asset.surface).or_default().push(asset);
    }

    grouped
        .into_iter()
        .map(|(surface, assets)| {
            let action = if assets.iter().any(|asset| workspace.join(asset.relative_path).exists())
            {
                "refresh"
            } else {
                "scaffold"
            };
            format!("- {action} {} ({} file(s))", surface.plan_label(), assets.len())
        })
        .collect()
}

fn apply_assistant_assets(
    workspace: &Path,
    assistant_assets: &[&AssistantAsset],
) -> Result<Vec<AssistantInitAction>, InitCommandError> {
    let mut grouped = BTreeMap::<AssistantSurface, AssistantInitAction>::new();

    for asset in assistant_assets {
        let asset = *asset;
        let target = workspace.join(asset.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| InitCommandError::WriteFile {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let file_status = if target.is_file() {
            let existing = fs::read_to_string(&target)
                .map_err(|source| InitCommandError::ReadFile { path: target.clone(), source })?;
            if existing == asset.contents {
                "unchanged"
            } else {
                fs::write(&target, asset.contents).map_err(|source| {
                    InitCommandError::WriteFile { path: target.clone(), source }
                })?;
                "updated"
            }
        } else {
            fs::write(&target, asset.contents)
                .map_err(|source| InitCommandError::WriteFile { path: target.clone(), source })?;
            "created"
        };

        let entry = grouped.entry(asset.surface).or_insert(AssistantInitAction {
            surface: asset.surface,
            status: "unchanged",
            created_files: 0,
            updated_files: 0,
            unchanged_files: 0,
        });
        match file_status {
            "created" => entry.created_files += 1,
            "updated" => entry.updated_files += 1,
            _ => entry.unchanged_files += 1,
        }
    }

    let mut actions = grouped.into_values().collect::<Vec<_>>();
    for action in &mut actions {
        action.status = if action.updated_files > 0
            || (action.created_files > 0 && action.unchanged_files > 0)
        {
            "updated"
        } else if action.created_files > 0 {
            "created"
        } else {
            "unchanged"
        };
    }
    Ok(actions)
}

fn apply_workspace_hygiene_defaults(
    workspace: &Path,
    domains: &BTreeSet<DomainFamily>,
) -> Result<Vec<HygieneInitAction>, InitCommandError> {
    let mut actions = Vec::new();
    let mut planned_paths = BTreeSet::new();

    for plan in plan_hygiene_defaults(workspace, domains) {
        planned_paths.insert(plan.path.to_string());
        let target = workspace.join(plan.path);
        let existing =
            if target.is_file() {
                Some(fs::read_to_string(&target).map_err(|source| InitCommandError::ReadFile {
                    path: target.clone(),
                    source,
                })?)
            } else {
                None
            };
        let existed = existing.is_some();
        let merged = merge_hygiene_content(existing.as_deref(), &plan);
        let status = if !existed {
            fs::write(&target, &merged.content)
                .map_err(|source| InitCommandError::WriteFile { path: target.clone(), source })?;
            "created"
        } else if merged.added_patterns.is_empty() {
            "unchanged"
        } else {
            fs::write(&target, &merged.content)
                .map_err(|source| InitCommandError::WriteFile { path: target.clone(), source })?;
            "updated"
        };

        actions.push(HygieneInitAction {
            path: plan.path.to_string(),
            status,
            added_patterns: merged.added_patterns.len(),
            preserved_custom_lines: merged.preserved_custom_lines,
            sources: plan.packs.into_iter().map(|pack| pack.provenance).collect(),
        });
    }

    for path in [
        ".gitignore",
        ".dockerignore",
        ".eslintignore",
        ".prettierignore",
        ".terraformignore",
        ".helmignore",
    ] {
        if !planned_paths.contains(path) {
            actions.push(HygieneInitAction {
                path: path.to_string(),
                status: "skipped",
                added_patterns: 0,
                preserved_custom_lines: 0,
                sources: vec!["not-relevant".to_string()],
            });
        }
    }

    Ok(actions)
}

fn resolve_seeded_routes(
    assistants: &[RuntimeKind],
    explicit_slots: &BTreeSet<RouteSlot>,
    availability: impl Fn(RuntimeKind) -> bool,
) -> Result<Vec<SeededRouteSelection>, InitCommandError> {
    let missing_slots = required_init_route_slots()
        .into_iter()
        .filter(|slot| !explicit_slots.contains(slot))
        .collect::<Vec<_>>();
    if missing_slots.is_empty() || assistants.is_empty() {
        return Ok(Vec::new());
    }

    let mut available_assistants = Vec::new();
    let mut unavailable_assistants = BTreeSet::new();
    for runtime in assistants.iter().copied() {
        if availability(runtime) {
            available_assistants.push(runtime);
        } else {
            unavailable_assistants.insert(runtime);
        }
    }

    if available_assistants.is_empty() {
        return Err(InitCommandError::NoAvailableAssistantDefaults {
            assistants: format_runtime_list(assistants),
            slots: format_slot_list(&missing_slots),
            example: INIT_ROUTE_EXAMPLE,
        });
    }

    let seeded = seeded_routes_for_assistants(&available_assistants);
    Ok(missing_slots
        .into_iter()
        .filter_map(|slot| {
            seeded.get(&slot).cloned().map(|route| SeededRouteSelection {
                slot,
                route,
                fallback_from_unavailable: unavailable_assistants
                    .contains(&built_in_default_route(slot).runtime)
                    .then_some(built_in_default_route(slot).runtime),
            })
        })
        .collect())
}

fn required_init_route_slots() -> [RouteSlot; 4] {
    [RouteSlot::Planning, RouteSlot::Implementation, RouteSlot::Verification, RouteSlot::Review]
}

fn format_runtime_list(runtimes: &[RuntimeKind]) -> String {
    runtimes.iter().map(|runtime| runtime.as_str()).collect::<Vec<_>>().join(", ")
}

fn format_slot_list(slots: &[RouteSlot]) -> String {
    slots.iter().map(|slot| slot.as_str()).collect::<Vec<_>>().join(", ")
}

fn supported_runtime_choices() -> String {
    format_runtime_list(&[
        RuntimeKind::Claude,
        RuntimeKind::Codex,
        RuntimeKind::Copilot,
        RuntimeKind::Gemini,
    ])
}

fn supported_route_slots() -> String {
    format_slot_list(&required_init_route_slots())
}

fn init_inspect_command(workspace: &Path) -> String {
    format!("boundline config show --workspace {}", workspace.display())
}

fn init_doctor_command(workspace: &Path) -> String {
    format!("boundline doctor --workspace {}", workspace.display())
}

fn guided_canon_mode_prompt() -> String {
    "Canon mode-selection [manual|auto-confirm|auto]\n  Default: auto-confirm\n  Choose how Canon approval is requested for governed stages.\n> ".to_string()
}

fn guided_assistant_prompt() -> String {
    format!(
        "Assistant surfaces [{}], comma-separated\n  Leave blank to skip repository-local assistant packs and assistant-seeded routes.\n  Example: copilot,codex\n> ",
        supported_runtime_choices()
    )
}

fn guided_route_prompt() -> String {
    format!(
        "Model routes [{}], comma-separated as SLOT=RUNTIME:MODEL\n  Optional: leave blank to let selected assistants seed missing slots.\n  Example: {}\n> ",
        supported_route_slots(),
        INIT_ROUTE_EXAMPLE
    )
}

fn invalid_route_shape_message(raw: &str) -> String {
    format!(
        "model route `{raw}` must use SLOT=RUNTIME:MODEL with slots {}. Example: {}",
        supported_route_slots(),
        INIT_ROUTE_EXAMPLE
    )
}

fn parse_domain_standard(raw: &str) -> Result<(DomainFamily, String), InitCommandError> {
    let (family, standards) = raw.split_once('=').ok_or_else(|| {
        InitCommandError::InvalidDomainArgument("domain standards must use FAMILY=TEXT".to_string())
    })?;
    let family = parse_domain_family(family)?;
    let standards = standards.trim();
    if standards.is_empty() {
        return Err(InitCommandError::InvalidDomainArgument(
            "domain standards text cannot be empty".to_string(),
        ));
    }
    Ok((family, standards.to_string()))
}

fn parse_model_route(raw: &str) -> Result<(RouteSlot, ModelRoute), InitCommandError> {
    let (slot_raw, route_raw) = raw
        .split_once('=')
        .ok_or_else(|| InitCommandError::InvalidDomainArgument(invalid_route_shape_message(raw)))?;
    let (runtime_raw, model_raw) = route_raw
        .split_once(':')
        .ok_or_else(|| InitCommandError::InvalidDomainArgument(invalid_route_shape_message(raw)))?;
    let slot = parse_route_slot(slot_raw)?;
    let runtime = parse_runtime_kind(runtime_raw)?;
    let route = ModelRoute { runtime, model: model_raw.trim().to_string() };
    route
        .validate()
        .map_err(|source| InitCommandError::InvalidDomainArgument(source.to_string()))?;
    Ok((slot, route))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuidedInitAnswers {
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: Vec<RuntimeKind>,
    routes: Vec<(RouteSlot, ModelRoute)>,
}

fn collect_guided_init_answers(
    prompt_for_canon_mode: bool,
    prompt_for_assistants: bool,
    prompt_for_routes: bool,
) -> Result<GuidedInitAnswers, InitCommandError> {
    let canon_mode_selection = if prompt_for_canon_mode {
        let mode = prompt_line(&guided_canon_mode_prompt())?;
        Some(parse_canon_mode_selection(if mode.trim().is_empty() {
            "auto-confirm"
        } else {
            mode.trim()
        })?)
    } else {
        None
    };

    let assistants = if prompt_for_assistants {
        let assistants = prompt_line(&guided_assistant_prompt())?;
        parse_guided_assistants(&assistants)?
    } else {
        Vec::new()
    };

    let routes = if prompt_for_routes {
        let routes = prompt_line(&guided_route_prompt())?;
        parse_guided_routes(&routes)?
    } else {
        Vec::new()
    };

    Ok(GuidedInitAnswers { canon_mode_selection, assistants, routes })
}

fn prompt_line(prompt: &str) -> Result<String, InitCommandError> {
    print!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| InitCommandError::InvalidDomainArgument(error.to_string()))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| InitCommandError::InvalidDomainArgument(error.to_string()))?;
    Ok(line.trim().to_string())
}

fn parse_guided_assistants(raw: &str) -> Result<Vec<RuntimeKind>, InitCommandError> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(parse_runtime_kind)
        .collect()
}

fn parse_guided_routes(raw: &str) -> Result<Vec<(RouteSlot, ModelRoute)>, InitCommandError> {
    raw.split(',').map(str::trim).filter(|value| !value.is_empty()).map(parse_model_route).collect()
}

fn parse_canon_mode_selection(raw: &str) -> Result<CanonModeSelectionPreference, InitCommandError> {
    match raw.trim() {
        "manual" => Ok(CanonModeSelectionPreference::Manual),
        "auto-confirm" => Ok(CanonModeSelectionPreference::AutoConfirm),
        "auto" => Ok(CanonModeSelectionPreference::Auto),
        other => Err(InitCommandError::InvalidDomainArgument(format!(
            "Canon mode-selection `{other}` is not supported; use manual, auto-confirm, or auto"
        ))),
    }
}

fn parse_route_slot(raw: &str) -> Result<RouteSlot, InitCommandError> {
    match raw.trim() {
        "planning" => Ok(RouteSlot::Planning),
        "implementation" => Ok(RouteSlot::Implementation),
        "verification" => Ok(RouteSlot::Verification),
        "review" => Ok(RouteSlot::Review),
        other => Err(InitCommandError::InvalidDomainArgument(format!(
            "route slot `{other}` is not supported; use {}",
            supported_route_slots()
        ))),
    }
}

fn parse_runtime_kind(raw: &str) -> Result<RuntimeKind, InitCommandError> {
    match raw.trim() {
        "claude" => Ok(RuntimeKind::Claude),
        "codex" => Ok(RuntimeKind::Codex),
        "copilot" => Ok(RuntimeKind::Copilot),
        "gemini" => Ok(RuntimeKind::Gemini),
        other => Err(InitCommandError::InvalidDomainArgument(format!(
            "runtime `{other}` is not supported; use {}",
            supported_runtime_choices()
        ))),
    }
}

fn parse_context_binding(
    raw: &str,
    required: bool,
) -> Result<(DomainFamily, ExternalContextBinding), InitCommandError> {
    let mut parts = raw.splitn(3, '|');
    let family = parts.next().ok_or_else(|| {
        InitCommandError::InvalidDomainArgument(
            "context bindings must use FAMILY|KIND|REFERENCE".to_string(),
        )
    })?;
    let kind = parts.next().ok_or_else(|| {
        InitCommandError::InvalidDomainArgument(
            "context bindings must use FAMILY|KIND|REFERENCE".to_string(),
        )
    })?;
    let reference = parts.next().ok_or_else(|| {
        InitCommandError::InvalidDomainArgument(
            "context bindings must use FAMILY|KIND|REFERENCE".to_string(),
        )
    })?;

    let family = parse_domain_family(family)?;
    let kind = parse_external_context_kind(kind)?;
    let binding = ExternalContextBinding {
        kind,
        reference: reference.trim().to_string(),
        required,
        notes: None,
    };
    binding
        .validate()
        .map_err(|source| InitCommandError::InvalidDomainTemplate(source.to_string()))?;
    Ok((family, binding))
}

fn parse_domain_family(raw: &str) -> Result<DomainFamily, InitCommandError> {
    match raw.trim() {
        "systems" => Ok(DomainFamily::Systems),
        "jvm_service" | "jvm-service" => Ok(DomainFamily::JvmService),
        "dotnet_service" | "dotnet-service" => Ok(DomainFamily::DotNetService),
        "python_service" | "python-service" => Ok(DomainFamily::PythonService),
        "node_service" | "node-service" => Ok(DomainFamily::NodeService),
        "web_ui" | "web-ui" => Ok(DomainFamily::WebUi),
        "react" => Ok(DomainFamily::React),
        "vue" => Ok(DomainFamily::Vue),
        "angular" => Ok(DomainFamily::Angular),
        "ruby" => Ok(DomainFamily::Ruby),
        "php" => Ok(DomainFamily::Php),
        "data" => Ok(DomainFamily::Data),
        "mobile" => Ok(DomainFamily::Mobile),
        other => {
            Err(InitCommandError::InvalidDomainArgument(format!("unknown domain family `{other}`")))
        }
    }
}

fn parse_external_context_kind(raw: &str) -> Result<ExternalContextKind, InitCommandError> {
    match raw.trim() {
        "design_reference" | "design-reference" => Ok(ExternalContextKind::DesignReference),
        "design_system" | "design-system" => Ok(ExternalContextKind::DesignSystem),
        "design_tokens" | "design-tokens" => Ok(ExternalContextKind::DesignTokens),
        "platform_guidance" | "platform-guidance" => Ok(ExternalContextKind::PlatformGuidance),
        "api_contract" | "api-contract" => Ok(ExternalContextKind::ApiContract),
        "custom" => Ok(ExternalContextKind::Custom),
        other => Err(InitCommandError::InvalidDomainArgument(format!(
            "unknown external context kind `{other}`"
        ))),
    }
}

fn upsert_binding(bindings: &mut Vec<ExternalContextBinding>, binding: ExternalContextBinding) {
    if let Some(existing) = bindings
        .iter_mut()
        .find(|existing| existing.kind == binding.kind && existing.reference == binding.reference)
    {
        *existing = binding;
    } else {
        bindings.push(binding);
    }
}

fn execution_template(
    template: InitTemplate,
    canon: Option<&CanonPreferences>,
) -> serde_json::Value {
    let (name, attempt_id, summary) = match template {
        InitTemplate::BugFix => ("init-bug-fix", "apply-bug-fix", "Apply a bounded bug fix"),
        InitTemplate::Change => ("init-change", "apply-change", "Apply a bounded change"),
        InitTemplate::Delivery => {
            ("init-delivery", "apply-delivery", "Apply a bounded delivery update")
        }
    };

    let mut execution = json!({
        "name": name,
        "read_targets": ["src/", "tests/"],
        "validation_command": {
            "program": "cargo",
            "args": ["test", "--quiet"]
        },
        "attempts": [
            {
                "attempt_id": attempt_id,
                "summary": summary,
                "failure_mode": "replan",
                "changes": [
                    {
                        "path": "README.md",
                        "find": "TODO(init)",
                        "replace": "TODO(init)"
                    }
                ]
            }
        ]
    });

    if let Some(canon) = canon
        && let (Some(default_risk), Some(default_zone), Some(default_owner)) = (
            canon.default_risk.as_deref(),
            canon.default_zone.as_deref(),
            canon.default_owner.as_deref(),
        )
    {
        let (flow_name, stage_id, canon_mode) = match template {
            InitTemplate::BugFix => ("bug-fix", "investigate", "discovery"),
            InitTemplate::Change => ("change", "understand-change", "change"),
            InitTemplate::Delivery => ("delivery", "requirements", "requirements"),
        };
        execution["governance"] = json!({
            "default_runtime": "canon",
            "canon": {
                "command": "canon",
                "default_owner": default_owner,
                "default_risk": default_risk,
                "default_zone": default_zone,
                "default_system_context": "existing"
            },
            "stages": [{
                "flow_name": flow_name,
                "stage_id": stage_id,
                "enabled": true,
                "required": true,
                "autopilot": false,
                "runtime": "canon",
                "canon_mode": canon_mode,
                "system_context": "existing",
                "risk": default_risk,
                "zone": default_zone,
                "owner": default_owner
            }]
        });
    }

    execution
}

fn template_label(template: InitTemplate) -> &'static str {
    match template {
        InitTemplate::BugFix => "bug-fix",
        InitTemplate::Change => "change",
        InitTemplate::Delivery => "delivery",
    }
}

fn runtime_capability_line(runtime: RuntimeKind) -> &'static str {
    if runtime_available(runtime) { "available" } else { "missing from PATH or extension surface" }
}

pub fn runtime_available(runtime: RuntimeKind) -> bool {
    match runtime {
        RuntimeKind::Copilot => true,
        RuntimeKind::Claude => command_in_path("claude"),
        RuntimeKind::Codex => command_in_path("codex"),
        RuntimeKind::Gemini => command_in_path("gemini"),
    }
}

fn command_in_path(command: &str) -> bool {
    let path_var = match std::env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };

    for entry in std::env::split_paths(&path_var) {
        let candidate = entry.join(command);
        if candidate.is_file() {
            return true;
        }
    }

    false
}

#[derive(Debug, Error)]
pub enum InitCommandError {
    #[error("failed to create workspace directory {path}: {source}")]
    CreateWorkspace { path: PathBuf, source: std::io::Error },
    #[error("failed to write file {path}: {source}")]
    WriteFile { path: PathBuf, source: std::io::Error },
    #[error("failed to read file {path}: {source}")]
    ReadFile { path: PathBuf, source: std::io::Error },
    #[error("failed to persist config: {0}")]
    ConfigStore(#[from] ConfigStoreError),
    #[error(
        "no available assistant defaults remain for slots {slots}; selected assistants {assistants} are missing from PATH or extension surface. Install one of them, choose an available assistant, or rerun with explicit --route overrides such as --route {example}"
    )]
    NoAvailableAssistantDefaults { assistants: String, slots: String, example: &'static str },
    #[error("invalid domain argument: {0}")]
    InvalidDomainArgument(String),
    #[error("invalid domain template settings: {0}")]
    InvalidDomainTemplate(String),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{
        InitRequest, command_in_path, execute_init, execution_template, format_runtime_list,
        format_slot_list, guided_assistant_prompt, guided_canon_mode_prompt, guided_route_prompt,
        parse_canon_mode_selection, parse_context_binding, parse_domain_family,
        parse_domain_standard, parse_external_context_kind, parse_guided_assistants,
        parse_guided_routes, parse_model_route, resolve_seeded_routes, supported_route_slots,
        supported_runtime_choices, template_label, upsert_binding,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::cli::CommandExitStatus;
    use crate::domain::configuration::{CanonPreferences, InitTemplate, RouteSlot, RuntimeKind};
    use crate::domain::domain_templates::{
        DomainFamily, ExternalContextBinding, ExternalContextKind,
    };
    use crate::domain::governance::CanonModeSelectionPreference;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    #[test]
    fn execute_init_infers_and_seeds_domain_templates() {
        let workspace = temp_workspace("boundline-init-domain");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[RuntimeKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &["react=workspace react rules".to_string()],
            context_bindings: &["react|design_system|mcp:design-system".to_string()],
            required_context_bindings: &["react|design_reference|design/reference.md".to_string()],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("domain_templates:"));
        assert!(report.terminal_output.contains("- react: enabled=true"));

        let saved = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(saved.routing.domain_templates.contains_key(&DomainFamily::React));
        assert!(saved.routing.domain_templates.contains_key(&DomainFamily::WebUi));
        let react = saved.routing.domain_templates.get(&DomainFamily::React).unwrap();
        assert_eq!(react.standards.as_deref(), Some("workspace react rules"));
        assert_eq!(react.external_context_bindings.len(), 2);
    }

    #[test]
    fn execute_init_rejects_invalid_domain_binding_format() {
        let workspace = temp_workspace("boundline-init-domain-invalid");

        let error = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &["react|design_system".to_string()],
            required_context_bindings: &[],
            canon_mode_selection: None,
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap_err();

        assert!(error.to_string().contains("context bindings must use FAMILY|KIND|REFERENCE"));
    }

    #[test]
    fn execute_init_previews_existing_files_without_force() {
        let workspace = temp_workspace("boundline-init-preview");
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(workspace.join(".boundline/execution.json"), "{}\n").unwrap();
        FileConfigStore::for_workspace(&workspace).save_local(&Default::default()).unwrap();

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: Some(InitTemplate::Delivery),
            assistants: &[],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: None,
            risk: None,
            zone: None,
            owner: None,
            force: false,
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("init: preview only"));
        assert!(report.terminal_output.contains("template: delivery"));
        assert!(report.terminal_output.contains("planned_changes:"));
        assert!(report.terminal_output.contains("- update"));
        assert!(report.terminal_output.contains("next_steps:"));
    }

    #[test]
    fn execute_init_reports_empty_domain_templates_when_no_detection_matches() {
        let workspace = temp_workspace("boundline-init-empty-domain");

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: Some(InitTemplate::Change),
            assistants: &[],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("template: change"));
        assert!(report.terminal_output.contains("domain_templates: none"));

        let execution_profile =
            fs::read_to_string(workspace.join(".boundline/execution.json")).unwrap();
        assert!(execution_profile.contains("init-change"));
    }

    #[test]
    fn execute_init_seeds_missing_routes_from_selected_assistant_defaults() {
        let workspace = temp_workspace("boundline-init-default-routes");

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[RuntimeKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap();

        assert!(report.terminal_output.contains("route_setup:"));
        assert!(report.terminal_output.contains("assistant_defaults: copilot"));
        assert!(
            report.terminal_output.contains("seeded planning: copilot:gpt-5.4 [assistant-default]")
        );
        assert!(
            report.terminal_output.contains("seeded review: copilot:gpt-5.4 [assistant-default]")
        );
        assert!(
            report.terminal_output.contains("inspect_or_edit: boundline config show --workspace")
        );

        let saved = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(saved.routing.planning.unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(saved.routing.implementation.unwrap().model, "gpt-5.4");
        assert_eq!(saved.routing.verification.unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(saved.routing.review.unwrap().runtime, RuntimeKind::Copilot);
    }

    #[test]
    fn execute_init_preserves_explicit_routes_while_seeding_remaining_slots() {
        let workspace = temp_workspace("boundline-init-partial-routes");
        let explicit = ["planning=copilot:gpt-4o".to_string()];

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[RuntimeKind::Copilot],
            routes: &explicit,
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap();

        assert!(report.terminal_output.contains("route_setup:"));
        assert!(report.terminal_output.contains("explicit planning: copilot:gpt-4o [explicit]"));
        assert!(
            report
                .terminal_output
                .contains("seeded verification: copilot:gpt-5.4 [assistant-default]")
        );

        let saved = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(saved.routing.planning.unwrap().model, "gpt-4o");
        assert_eq!(saved.routing.implementation.unwrap().model, "gpt-5.4");
        assert_eq!(saved.routing.review.unwrap().model, "gpt-5.4");
    }

    #[test]
    fn parsing_helpers_cover_variants_errors_and_binding_upserts() {
        let canon_prompt = guided_canon_mode_prompt();
        assert!(canon_prompt.contains("manual|auto-confirm|auto"));
        assert!(canon_prompt.contains("Default: auto-confirm"));

        let assistant_prompt = guided_assistant_prompt();
        assert!(assistant_prompt.contains("Assistant surfaces [claude, codex, copilot, gemini]"));
        assert!(assistant_prompt.contains("Leave blank to skip repository-local assistant packs"));

        let route_prompt = guided_route_prompt();
        assert!(route_prompt.contains(&supported_route_slots()));
        assert!(
            route_prompt
                .contains("Optional: leave blank to let selected assistants seed missing slots")
        );
        assert!(route_prompt.contains("planning=copilot:gpt-5.4"));

        let (family, standards) = parse_domain_standard("react= follow ui rules").unwrap();
        assert_eq!(family, DomainFamily::React);
        assert_eq!(standards, "follow ui rules");
        assert!(parse_domain_standard("react=").is_err());
        assert!(parse_domain_standard("react").is_err());
        assert_eq!(
            parse_canon_mode_selection("auto-confirm").unwrap(),
            crate::domain::governance::CanonModeSelectionPreference::AutoConfirm
        );
        assert_eq!(
            parse_guided_assistants("codex, copilot").unwrap(),
            vec![RuntimeKind::Codex, RuntimeKind::Copilot]
        );
        let routes = parse_guided_routes("planning=codex:gpt-5-codex").unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].0, crate::domain::configuration::RouteSlot::Planning);
        assert_eq!(routes[0].1.runtime, RuntimeKind::Codex);
        assert_eq!(routes[0].1.model, "gpt-5-codex");
        assert!(
            parse_model_route("planning-codex-gpt-5-codex")
                .unwrap_err()
                .to_string()
                .contains("SLOT=RUNTIME:MODEL")
        );
        assert!(
            parse_model_route("plan=codex:gpt-5-codex")
                .unwrap_err()
                .to_string()
                .contains(&supported_route_slots())
        );
        assert!(
            parse_guided_assistants("cursor")
                .unwrap_err()
                .to_string()
                .contains(&supported_runtime_choices())
        );

        assert_eq!(parse_domain_family("jvm-service").unwrap(), DomainFamily::JvmService);
        assert_eq!(parse_domain_family("dotnet_service").unwrap(), DomainFamily::DotNetService);
        assert_eq!(parse_domain_family("python-service").unwrap(), DomainFamily::PythonService);
        assert_eq!(parse_domain_family("node_service").unwrap(), DomainFamily::NodeService);
        assert_eq!(parse_domain_family("web-ui").unwrap(), DomainFamily::WebUi);
        assert_eq!(parse_domain_family("vue").unwrap(), DomainFamily::Vue);
        assert_eq!(parse_domain_family("angular").unwrap(), DomainFamily::Angular);
        assert_eq!(parse_domain_family("ruby").unwrap(), DomainFamily::Ruby);
        assert_eq!(parse_domain_family("php").unwrap(), DomainFamily::Php);
        assert_eq!(parse_domain_family("data").unwrap(), DomainFamily::Data);
        assert_eq!(parse_domain_family("mobile").unwrap(), DomainFamily::Mobile);
        assert!(parse_domain_family("unknown").is_err());

        assert_eq!(
            parse_external_context_kind("design-reference").unwrap(),
            ExternalContextKind::DesignReference
        );
        assert_eq!(
            parse_external_context_kind("design_tokens").unwrap(),
            ExternalContextKind::DesignTokens
        );
        assert_eq!(
            parse_external_context_kind("platform-guidance").unwrap(),
            ExternalContextKind::PlatformGuidance
        );
        assert_eq!(
            parse_external_context_kind("api_contract").unwrap(),
            ExternalContextKind::ApiContract
        );
        assert_eq!(parse_external_context_kind("custom").unwrap(), ExternalContextKind::Custom);
        assert!(parse_external_context_kind("unknown-kind").is_err());

        let (family, binding) =
            parse_context_binding("react|design-system|mcp:design-system", true).unwrap();
        assert_eq!(family, DomainFamily::React);
        assert_eq!(binding.kind, ExternalContextKind::DesignSystem);
        assert!(binding.required);
        assert!(parse_context_binding("react||ref", false).is_err());

        let mut bindings = vec![ExternalContextBinding {
            kind: ExternalContextKind::DesignSystem,
            reference: "mcp:design-system".to_string(),
            required: false,
            notes: Some("old".to_string()),
        }];
        upsert_binding(
            &mut bindings,
            ExternalContextBinding {
                kind: ExternalContextKind::DesignSystem,
                reference: "mcp:design-system".to_string(),
                required: true,
                notes: Some("new".to_string()),
            },
        );
        upsert_binding(
            &mut bindings,
            ExternalContextBinding {
                kind: ExternalContextKind::ApiContract,
                reference: "api/openapi.yaml".to_string(),
                required: false,
                notes: None,
            },
        );
        assert_eq!(bindings.len(), 2);
        assert!(bindings[0].required);
        assert_eq!(bindings[0].notes.as_deref(), Some("new"));
    }

    #[test]
    fn resolve_seeded_routes_marks_fallbacks_when_selected_preferred_runtime_is_unavailable() {
        let seeded = resolve_seeded_routes(
            &[RuntimeKind::Codex, RuntimeKind::Copilot],
            &BTreeSet::new(),
            |runtime| runtime == RuntimeKind::Copilot,
        )
        .unwrap();

        let planning =
            seeded.iter().find(|selection| selection.slot == RouteSlot::Planning).unwrap();
        assert_eq!(planning.route.runtime, RuntimeKind::Copilot);
        assert_eq!(planning.fallback_from_unavailable, Some(RuntimeKind::Codex));

        let verification =
            seeded.iter().find(|selection| selection.slot == RouteSlot::Verification).unwrap();
        assert_eq!(verification.route.runtime, RuntimeKind::Copilot);
        assert_eq!(verification.fallback_from_unavailable, None);
    }

    #[test]
    fn resolve_seeded_routes_errors_when_no_selected_runtime_can_fill_missing_slots() {
        let error =
            resolve_seeded_routes(&[RuntimeKind::Codex], &BTreeSet::new(), |_| false).unwrap_err();

        assert!(error.to_string().contains("no available assistant defaults remain"));
        assert!(error.to_string().contains("planning, implementation, verification, review"));
        assert!(error.to_string().contains("--route planning=copilot:gpt-5.4"));
    }

    #[test]
    fn resolve_seeded_routes_allows_unavailable_assistants_when_explicit_routes_cover_every_slot() {
        let explicit_slots = [
            RouteSlot::Planning,
            RouteSlot::Implementation,
            RouteSlot::Verification,
            RouteSlot::Review,
        ]
        .into_iter()
        .collect();

        let seeded =
            resolve_seeded_routes(&[RuntimeKind::Codex], &explicit_slots, |_| false).unwrap();

        assert!(seeded.is_empty());
        assert_eq!(format_runtime_list(&[RuntimeKind::Codex]), "codex");
        assert_eq!(format_slot_list(&[RouteSlot::Planning, RouteSlot::Review]), "planning, review");
    }

    #[test]
    fn execute_init_reports_hygiene_unchanged_when_files_already_contain_patterns() {
        let workspace = temp_workspace("boundline-init-hygiene-unchanged");
        fs::create_dir_all(workspace.join(".git")).unwrap();
        fs::write(
            workspace.join(".gitignore"),
            "# Boundline universal defaults\n.boundline/traces/\n.boundline/checkpoints/\n",
        )
        .unwrap();

        let report = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[RuntimeKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        })
        .unwrap();

        assert!(
            report.terminal_output.contains(".gitignore: unchanged"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    #[cfg(unix)]
    fn execute_init_returns_read_file_error_when_hygiene_file_is_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let workspace = temp_workspace("boundline-init-hygiene-unreadable");
        fs::create_dir_all(workspace.join(".git")).unwrap();
        let gitignore = workspace.join(".gitignore");
        fs::write(&gitignore, "custom/\n").unwrap();
        fs::set_permissions(&gitignore, fs::Permissions::from_mode(0o000)).unwrap();

        let result = execute_init(InitRequest {
            workspace: &workspace,
            template: None,
            assistants: &[RuntimeKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            force: true,
        });

        // Restore permissions so temp dir cleanup can succeed
        fs::set_permissions(&gitignore, fs::Permissions::from_mode(0o644)).unwrap();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to read file"));
    }

    #[test]
    fn helper_functions_cover_templates_and_runtime_detection_paths() {
        assert_eq!(template_label(InitTemplate::BugFix), "bug-fix");
        assert_eq!(template_label(InitTemplate::Change), "change");
        assert_eq!(template_label(InitTemplate::Delivery), "delivery");

        let delivery_template = execution_template(InitTemplate::Delivery, None);
        assert_eq!(delivery_template["name"], "init-delivery");
        let change_template = execution_template(InitTemplate::Change, None);
        assert_eq!(change_template["attempts"][0]["attempt_id"], "apply-change");

        let canon = CanonPreferences {
            mode_selection: CanonModeSelectionPreference::AutoConfirm,
            default_risk: Some("medium".to_string()),
            default_zone: Some("engineering".to_string()),
            default_owner: Some("platform".to_string()),
            default_system_context: None,
        };
        let governed_delivery = execution_template(InitTemplate::Delivery, Some(&canon));
        assert_eq!(governed_delivery["governance"]["default_runtime"], "canon");
        assert_eq!(governed_delivery["governance"]["stages"][0]["canon_mode"], "requirements");

        assert!(super::runtime_available(RuntimeKind::Copilot));
        let _ = super::runtime_available(RuntimeKind::Claude);
        let _ = super::runtime_available(RuntimeKind::Codex);
        let _ = super::runtime_available(RuntimeKind::Gemini);
        assert!(!command_in_path("boundline-command-that-should-not-exist"));
    }
}
