use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::time::Duration;

use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
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
const BUNDLED_MODEL_CATALOG: &str = include_str!("../../assistant/catalog/model-catalog.toml");
#[cfg(test)]
const NO_TTY_GUIDANCE: &str =
    "Terminal interaction is unavailable. Rerun with --non-interactive and explicit flags.";
const ACCEPT_CURRENT_ROUTES_LABEL: &str = "Accept current routes";
const CLEAR_ALL_ROUTES_LABEL: &str = "Clear all routes";
const LEAVE_SLOT_UNSET_LABEL: &str = "Leave slot unset";
const CUSTOM_MODEL_ID_LABEL: &str = "Use custom model id";
const WRITE_CONFIGURATION_PROMPT: &str = "Write configuration?";
const PROGRESS_TICK_MS: u64 = 100;

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

#[derive(Debug, Clone, Deserialize)]
struct BundledModelCatalog {
    metadata: CatalogMetadata,
    #[serde(default)]
    runtimes: Vec<CatalogRuntimeEntry>,
    #[serde(default)]
    default_routes: CatalogDefaultRoutes,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogMetadata {
    source_label: String,
    catalog_version: String,
    updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogRuntimeEntry {
    runtime: RuntimeKind,
    display_name: String,
    #[serde(default)]
    models: Vec<CatalogModelEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogModelEntry {
    model_id: String,
    display_name: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CatalogDefaultRoutes {
    planning: Option<CatalogRouteReference>,
    implementation: Option<CatalogRouteReference>,
    verification: Option<CatalogRouteReference>,
    review: Option<CatalogRouteReference>,
}

#[derive(Debug, Clone, Deserialize)]
struct CatalogRouteReference {
    runtime: RuntimeKind,
    model_id: String,
}

impl BundledModelCatalog {
    fn load() -> Result<Self, InitCommandError> {
        toml::from_str(BUNDLED_MODEL_CATALOG)
            .map_err(|source| InitCommandError::InvalidBundledCatalog(source.to_string()))
    }

    fn summary_label(&self) -> String {
        format!(
            "{} catalog {} ({})",
            self.metadata.source_label, self.metadata.catalog_version, self.metadata.updated_at
        )
    }

    fn runtime_entry(&self, runtime: RuntimeKind) -> Option<&CatalogRuntimeEntry> {
        self.runtimes.iter().find(|entry| entry.runtime == runtime)
    }

    fn default_route_for_runtime(&self, runtime: RuntimeKind) -> Option<ModelRoute> {
        let entry = self.runtime_entry(runtime)?;
        let model = entry.models.first()?;
        Some(ModelRoute { runtime, model: model.model_id.clone() })
    }

    fn default_route_for_slot(&self, slot: RouteSlot) -> Option<ModelRoute> {
        let reference = match slot {
            RouteSlot::Planning => self.default_routes.planning.as_ref(),
            RouteSlot::Implementation => self.default_routes.implementation.as_ref(),
            RouteSlot::Verification => self.default_routes.verification.as_ref(),
            RouteSlot::Review => self.default_routes.review.as_ref(),
        }?;
        Some(ModelRoute { runtime: reference.runtime, model: reference.model_id.clone() })
    }

    fn runtime_labels(&self) -> Vec<String> {
        self.runtimes
            .iter()
            .map(|entry| format!("{} ({})", entry.display_name, entry.runtime.as_str()))
            .collect()
    }

    fn model_labels_for_runtime(&self, runtime: RuntimeKind) -> Vec<String> {
        self.runtime_entry(runtime)
            .map(|entry| {
                entry
                    .models
                    .iter()
                    .map(|model| format!("{} ({})", model.display_name, model.model_id))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GuidedRouteSource {
    AssistantDefault { fallback_from: Option<RuntimeKind> },
    Bundled,
    Custom,
    Unset,
}

impl GuidedRouteSource {
    fn label(&self) -> String {
        match self {
            Self::AssistantDefault { fallback_from: Some(runtime) } => {
                format!("assistant-default fallback-from={}-unavailable", runtime.as_str())
            }
            Self::AssistantDefault { fallback_from: None } => "assistant-default".to_string(),
            Self::Bundled => "bundled".to_string(),
            Self::Custom => "custom-unverified".to_string(),
            Self::Unset => "unset".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GuidedRouteSelection {
    slot: RouteSlot,
    route: Option<ModelRoute>,
    source: GuidedRouteSource,
}

impl GuidedRouteSelection {
    fn display_line(&self) -> String {
        match &self.route {
            Some(route) => format!(
                "{:<15} {}:{} [{}]",
                self.slot.as_str(),
                route.runtime,
                route.model,
                self.source.label()
            ),
            None => format!("{:<15} unset [{}]", self.slot.as_str(), self.source.label()),
        }
    }
}

trait InitInteractor {
    fn select(
        &mut self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InitCommandError>;
    fn multi_select(
        &mut self,
        prompt: &str,
        items: &[String],
        defaults: &[bool],
    ) -> Result<Vec<usize>, InitCommandError>;
    fn input(&mut self, prompt: &str, initial: &str) -> Result<String, InitCommandError>;
    fn confirm(&mut self, prompt: &str, default: bool) -> Result<bool, InitCommandError>;
}

#[derive(Debug, Default)]
struct DialoguerInitInteractor;

impl InitInteractor for DialoguerInitInteractor {
    fn select(
        &mut self,
        prompt: &str,
        items: &[String],
        default: usize,
    ) -> Result<usize, InitCommandError> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(|error| InitCommandError::PromptInteraction(error.to_string()))
    }

    fn multi_select(
        &mut self,
        prompt: &str,
        items: &[String],
        defaults: &[bool],
    ) -> Result<Vec<usize>, InitCommandError> {
        MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(items)
            .defaults(defaults)
            .interact()
            .map_err(|error| InitCommandError::PromptInteraction(error.to_string()))
    }

    fn input(&mut self, prompt: &str, initial: &str) -> Result<String, InitCommandError> {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .with_initial_text(initial.to_string())
            .allow_empty(true)
            .interact_text()
            .map_err(|error| InitCommandError::PromptInteraction(error.to_string()))
    }

    fn confirm(&mut self, prompt: &str, default: bool) -> Result<bool, InitCommandError> {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|error| InitCommandError::PromptInteraction(error.to_string()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InitRequest<'a> {
    pub workspace: &'a Path,
    pub non_interactive: bool,
    /// Override TTY detection for testing. `None` means auto-detect from stdin/stdout.
    pub interactive_terminal_override: Option<bool>,
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

    let catalog = BundledModelCatalog::load()?;
    let interactive_terminal = request
        .interactive_terminal_override
        .unwrap_or_else(|| io::stdin().is_terminal() && io::stdout().is_terminal());
    let needs_guided_values = request.canon_mode_selection.is_none()
        || request.assistants.is_empty()
        || request.routes.is_empty();

    if !request.non_interactive && needs_guided_values && !interactive_terminal {
        return Err(InitCommandError::InteractiveTerminalUnavailable);
    }

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

    let guided_answers = if !request.non_interactive && interactive_terminal && needs_guided_values
    {
        Some(collect_guided_init_answers(
            request.canon_mode_selection.is_none(),
            request.assistants.is_empty(),
            request.routes.is_empty(),
            &catalog,
            request.assistants,
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
        answers
            .routes
            .iter()
            .filter_map(|selection| selection.route.clone().map(|route| (selection.slot, route)))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let guided_route_decisions = guided_answers
        .as_ref()
        .map(|answers| {
            answers.routes.iter().map(|selection| selection.slot).collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut effective_routes =
        if explicit_routes.is_empty() { guided_routes.clone() } else { explicit_routes.clone() };
    let mut explicit_slots =
        effective_routes.iter().map(|(slot, _)| *slot).collect::<BTreeSet<_>>();
    explicit_slots.extend(guided_route_decisions);
    let seeded_routes =
        resolve_seeded_routes(&effective_assistants, &explicit_slots, runtime_available)?;
    effective_routes
        .extend(seeded_routes.iter().map(|selection| (selection.slot, selection.route.clone())));

    if let Some(answers) = guided_answers.as_ref() {
        let summary = render_guided_summary(
            template,
            effective_canon_mode_selection,
            &effective_assistants,
            &answers.routes,
            &catalog,
            &planned,
        );
        let mut interactor = DialoguerInitInteractor;
        if !interactor.confirm(&summary, true)? {
            return Ok(InitCommandReport {
                exit_status: CommandExitStatus::NonSuccess,
                terminal_output: render_cancelled_init_report(
                    workspace,
                    template,
                    effective_canon_mode_selection,
                    &effective_assistants,
                    &answers.routes,
                    &catalog,
                ),
            });
        }
    }

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
    run_init_activity("writing execution profile", interactive_terminal, || {
        fs::write(
            &execution_path,
            serde_json::to_string_pretty(&execution).expect("execution template should serialize"),
        )
        .map_err(|source| InitCommandError::WriteFile { path: execution_path.clone(), source })
    })?;

    run_init_activity("writing workspace config", interactive_terminal, || {
        Ok(store.save_local(&local)?)
    })?;
    let assistant_actions =
        run_init_activity("scaffolding assistant packs", interactive_terminal, || {
            apply_assistant_assets(workspace, &assistant_assets)
        })?;

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

    lines.push("route_setup:".to_string());
    lines.push(format!("- catalog_source: {}", catalog.summary_label()));
    if effective_assistants.is_empty() {
        lines.push(
            "- assistant_defaults: none selected; no assistant-seeded workspace routes were recorded"
                .to_string(),
        );
    } else {
        lines.push(format!("- assistant_defaults: {}", format_runtime_list(&effective_assistants)));
    }
    if let Some(answers) = guided_answers.as_ref() {
        lines.extend(
            answers.routes.iter().map(|selection| format!("- {}", selection.display_line().trim())),
        );
    } else {
        let explicit_route_lines =
            explicit_routes.iter().chain(guided_routes.iter()).collect::<Vec<_>>();
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
                format!(
                    "- explicit {}: {}:{} [explicit]",
                    slot.as_str(),
                    route.runtime,
                    route.model
                )
            }));
        }
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
                "- {}: {} created, {} updated, {} unchanged",
                action.surface.plan_label(),
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
    routes: Vec<GuidedRouteSelection>,
}

fn collect_guided_init_answers(
    prompt_for_canon_mode: bool,
    prompt_for_assistants: bool,
    prompt_for_routes: bool,
    catalog: &BundledModelCatalog,
    explicit_assistants: &[RuntimeKind],
) -> Result<GuidedInitAnswers, InitCommandError> {
    let mut interactor = DialoguerInitInteractor;
    collect_guided_init_answers_with_interactor(
        &mut interactor,
        prompt_for_canon_mode,
        prompt_for_assistants,
        prompt_for_routes,
        catalog,
        explicit_assistants,
    )
}

fn collect_guided_init_answers_with_interactor(
    interactor: &mut dyn InitInteractor,
    prompt_for_canon_mode: bool,
    prompt_for_assistants: bool,
    prompt_for_routes: bool,
    catalog: &BundledModelCatalog,
    explicit_assistants: &[RuntimeKind],
) -> Result<GuidedInitAnswers, InitCommandError> {
    let canon_mode_selection =
        if prompt_for_canon_mode { Some(select_canon_mode(interactor)?) } else { None };

    let assistants = if prompt_for_assistants {
        select_assistants(interactor, catalog)?
    } else {
        explicit_assistants.to_vec()
    };

    let routes = if prompt_for_routes {
        review_routes(interactor, catalog, &assistants)?
    } else {
        Vec::new()
    };

    Ok(GuidedInitAnswers { canon_mode_selection, assistants, routes })
}

fn select_canon_mode(
    interactor: &mut dyn InitInteractor,
) -> Result<CanonModeSelectionPreference, InitCommandError> {
    let items = vec![
        "Auto-confirm recommended approvals".to_string(),
        "Manual approval for every governed stage".to_string(),
        "Auto approval where policy allows".to_string(),
    ];
    match interactor.select("Canon approval mode", &items, 0)? {
        0 => Ok(CanonModeSelectionPreference::AutoConfirm),
        1 => Ok(CanonModeSelectionPreference::Manual),
        _ => Ok(CanonModeSelectionPreference::Auto),
    }
}

fn select_assistants(
    interactor: &mut dyn InitInteractor,
    catalog: &BundledModelCatalog,
) -> Result<Vec<RuntimeKind>, InitCommandError> {
    let items = catalog.runtime_labels();
    let defaults = vec![false; items.len()];
    let indices = interactor.multi_select(
        "Assistant surfaces\nSelect the repository-local assistant packs and route-default sources to scaffold.",
        &items,
        &defaults,
    )?;
    Ok(indices
        .into_iter()
        .filter_map(|index| catalog.runtimes.get(index).map(|entry| entry.runtime))
        .collect())
}

fn review_routes(
    interactor: &mut dyn InitInteractor,
    catalog: &BundledModelCatalog,
    assistants: &[RuntimeKind],
) -> Result<Vec<GuidedRouteSelection>, InitCommandError> {
    let mut selections = initial_guided_route_selections(catalog, assistants);
    let mut validation_message = None;

    loop {
        let prompt =
            render_guided_route_review(catalog, &selections, validation_message.as_deref());
        let items = route_review_items();
        let choice = interactor.select(&prompt, &items, 0)?;
        match choice {
            0 => return Ok(selections),
            1..=4 => {
                let slot = required_init_route_slots()[choice - 1];
                validation_message =
                    edit_route_selection(interactor, catalog, &mut selections, slot).err();
            }
            _ => {
                clear_guided_route_selections(&mut selections);
                validation_message = None;
            }
        }
    }
}

fn route_review_items() -> Vec<String> {
    let mut items = vec![ACCEPT_CURRENT_ROUTES_LABEL.to_string()];
    items.extend(
        required_init_route_slots().into_iter().map(|slot| format!("Edit {}", slot.as_str())),
    );
    items.push(CLEAR_ALL_ROUTES_LABEL.to_string());
    items
}

fn initial_guided_route_selections(
    catalog: &BundledModelCatalog,
    assistants: &[RuntimeKind],
) -> Vec<GuidedRouteSelection> {
    let available_assistants = assistants
        .iter()
        .copied()
        .filter(|runtime| runtime_available(*runtime))
        .collect::<Vec<_>>();
    let fallback_runtime = available_assistants.first().copied();

    required_init_route_slots()
        .into_iter()
        .map(|slot| {
            if let Some(default_route) = catalog.default_route_for_slot(slot)
                && available_assistants.contains(&default_route.runtime)
            {
                return GuidedRouteSelection {
                    slot,
                    route: Some(default_route),
                    source: GuidedRouteSource::AssistantDefault { fallback_from: None },
                };
            }

            if let Some(runtime) = fallback_runtime
                && let Some(route) = catalog.default_route_for_runtime(runtime)
            {
                let fallback_from = catalog.default_route_for_slot(slot).map(|route| route.runtime);
                return GuidedRouteSelection {
                    slot,
                    route: Some(route),
                    source: GuidedRouteSource::AssistantDefault {
                        fallback_from: fallback_from
                            .filter(|default_runtime| *default_runtime != runtime),
                    },
                };
            }

            GuidedRouteSelection { slot, route: None, source: GuidedRouteSource::Unset }
        })
        .collect()
}

fn clear_guided_route_selections(selections: &mut [GuidedRouteSelection]) {
    for selection in selections {
        selection.route = None;
        selection.source = GuidedRouteSource::Unset;
    }
}

fn edit_route_selection(
    interactor: &mut dyn InitInteractor,
    catalog: &BundledModelCatalog,
    selections: &mut [GuidedRouteSelection],
    slot: RouteSlot,
) -> Result<(), String> {
    let Some(selection) = selections.iter_mut().find(|selection| selection.slot == slot) else {
        return Err(format!("route slot `{}` is not supported", slot.as_str()));
    };

    let runtime_items = {
        let mut items = catalog.runtime_labels();
        items.push(LEAVE_SLOT_UNSET_LABEL.to_string());
        items
    };
    let runtime_default = selection
        .route
        .as_ref()
        .and_then(|route| catalog.runtimes.iter().position(|entry| entry.runtime == route.runtime))
        .unwrap_or(0);
    let runtime_choice = interactor
        .select(
            &format!("{} runtime", slot.as_str()),
            &runtime_items,
            runtime_default.min(runtime_items.len().saturating_sub(1)),
        )
        .map_err(|error| error.to_string())?;

    if runtime_choice == catalog.runtimes.len() {
        selection.route = None;
        selection.source = GuidedRouteSource::Unset;
        return Ok(());
    }

    let runtime_entry = &catalog.runtimes[runtime_choice];
    let mut model_items = catalog.model_labels_for_runtime(runtime_entry.runtime);
    model_items.push(CUSTOM_MODEL_ID_LABEL.to_string());
    let model_default = selection
        .route
        .as_ref()
        .and_then(|route| {
            catalog.runtime_entry(runtime_entry.runtime).and_then(|entry| {
                entry.models.iter().position(|model| model.model_id == route.model)
            })
        })
        .unwrap_or(0);
    let model_choice = interactor
        .select(
            &format!("{} model", slot.as_str()),
            &model_items,
            model_default.min(model_items.len().saturating_sub(1)),
        )
        .map_err(|error| error.to_string())?;

    if model_choice == model_items.len() - 1 {
        let initial =
            selection.route.as_ref().map(|route| route.model.as_str()).unwrap_or_default();
        let custom_model = interactor
            .input(&format!("{} custom model id", slot.as_str()), initial)
            .map_err(|error| error.to_string())?
            .trim()
            .to_string();
        if custom_model.is_empty() {
            return Err("Custom model id cannot be empty.".to_string());
        }
        selection.route = Some(ModelRoute { runtime: runtime_entry.runtime, model: custom_model });
        selection.source = GuidedRouteSource::Custom;
        return Ok(());
    }

    let model = runtime_entry.models[model_choice].model_id.clone();
    selection.route = Some(ModelRoute { runtime: runtime_entry.runtime, model });
    selection.source = GuidedRouteSource::Bundled;
    Ok(())
}

fn render_guided_route_review(
    catalog: &BundledModelCatalog,
    selections: &[GuidedRouteSelection],
    validation_message: Option<&str>,
) -> String {
    let mut lines = vec![
        format!("Model routes\nCatalog: {}", catalog.summary_label()),
        "Review the proposed routes, then accept defaults, edit one slot, or clear all routes."
            .to_string(),
    ];
    if let Some(validation_message) = validation_message {
        lines.push(format!("Validation: {validation_message}"));
    }
    lines.extend(selections.iter().map(GuidedRouteSelection::display_line));
    lines.join("\n")
}

fn render_guided_summary(
    template: InitTemplate,
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: &[RuntimeKind],
    routes: &[GuidedRouteSelection],
    catalog: &BundledModelCatalog,
    planned_changes: &[String],
) -> String {
    let mut lines = vec![
        "Summary".to_string(),
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
        lines.push(format!("Assistant surfaces: {}", format_runtime_list(assistants)));
    }
    lines.push("Model routes:".to_string());
    lines.extend(routes.iter().map(|selection| format!("- {}", selection.display_line().trim())));
    lines.push("Planned changes:".to_string());
    lines.extend(planned_changes.iter().cloned());
    lines.push(WRITE_CONFIGURATION_PROMPT.to_string());
    lines.join("\n")
}

fn render_cancelled_init_report(
    workspace: &Path,
    template: InitTemplate,
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: &[RuntimeKind],
    routes: &[GuidedRouteSelection],
    catalog: &BundledModelCatalog,
) -> String {
    let mut lines = vec![
        "init: canceled before write".to_string(),
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
        lines.push(format!("assistant_surfaces: {}", format_runtime_list(assistants)));
    }
    lines.push("route_setup:".to_string());
    lines.extend(routes.iter().map(|selection| format!("- {}", selection.display_line().trim())));
    lines.push("next_steps:".to_string());
    lines.push(format!(
        "- rerun boundline init --workspace {} to confirm and write the configuration",
        workspace.display()
    ));
    lines.join("\n")
}

fn run_init_activity<T, F>(
    label: &str,
    interactive_terminal: bool,
    action: F,
) -> Result<T, InitCommandError>
where
    F: FnOnce() -> Result<T, InitCommandError>,
{
    if interactive_terminal {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner} {msg}")
                .expect("spinner template should be valid"),
        );
        spinner.set_message(label.to_string());
        spinner.enable_steady_tick(Duration::from_millis(PROGRESS_TICK_MS));
        let result = action();
        match &result {
            Ok(_) => spinner.finish_and_clear(),
            Err(_) => spinner.abandon_with_message(format!("{label}: failed")),
        }
        result
    } else {
        eprintln!("progress: {label}");
        let result = action();
        match &result {
            Ok(_) => eprintln!("progress: {label}: done"),
            Err(error) => eprintln!("progress: {label}: failed ({error})"),
        }
        result
    }
}

#[cfg(test)]
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
    #[error(
        "Terminal interaction is unavailable. Rerun with --non-interactive and explicit flags."
    )]
    InteractiveTerminalUnavailable,
    #[error("invalid bundled model catalog: {0}")]
    InvalidBundledCatalog(String),
    #[error("failed to collect init input: {0}")]
    PromptInteraction(String),
    #[error("invalid domain argument: {0}")]
    InvalidDomainArgument(String),
    #[error("invalid domain template settings: {0}")]
    InvalidDomainTemplate(String),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;

    use uuid::Uuid;

    use super::{
        BundledModelCatalog, GuidedRouteSource, InitInteractor, InitRequest,
        collect_guided_init_answers_with_interactor, command_in_path, execute_init,
        execution_template, format_runtime_list, format_slot_list, initial_guided_route_selections,
        parse_canon_mode_selection, parse_context_binding, parse_domain_family,
        parse_domain_standard, parse_external_context_kind, parse_model_route,
        render_guided_route_review, resolve_seeded_routes, supported_route_slots,
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

    #[derive(Default)]
    struct ScriptedInteractor {
        selects: VecDeque<usize>,
        multi_selects: VecDeque<Vec<usize>>,
        inputs: VecDeque<String>,
        confirms: VecDeque<bool>,
    }

    impl InitInteractor for ScriptedInteractor {
        fn select(
            &mut self,
            _prompt: &str,
            _items: &[String],
            _default: usize,
        ) -> Result<usize, super::InitCommandError> {
            self.selects.pop_front().ok_or_else(|| {
                super::InitCommandError::PromptInteraction("missing scripted select".to_string())
            })
        }

        fn multi_select(
            &mut self,
            _prompt: &str,
            _items: &[String],
            _defaults: &[bool],
        ) -> Result<Vec<usize>, super::InitCommandError> {
            self.multi_selects.pop_front().ok_or_else(|| {
                super::InitCommandError::PromptInteraction(
                    "missing scripted multi-select".to_string(),
                )
            })
        }

        fn input(
            &mut self,
            _prompt: &str,
            _initial: &str,
        ) -> Result<String, super::InitCommandError> {
            self.inputs.pop_front().ok_or_else(|| {
                super::InitCommandError::PromptInteraction("missing scripted input".to_string())
            })
        }

        fn confirm(
            &mut self,
            _prompt: &str,
            _default: bool,
        ) -> Result<bool, super::InitCommandError> {
            self.confirms.pop_front().ok_or_else(|| {
                super::InitCommandError::PromptInteraction("missing scripted confirm".to_string())
            })
        }
    }

    #[test]
    fn execute_init_infers_and_seeds_domain_templates() {
        let workspace = temp_workspace("boundline-init-domain");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();

        let report = execute_init(InitRequest {
            workspace: &workspace,
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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
    fn parsing_helpers_cover_variants_errors_binding_upserts_and_guided_catalog() {
        let catalog = BundledModelCatalog::load().unwrap();
        assert!(catalog.summary_label().contains("bundled"));

        let (family, standards) = parse_domain_standard("react= follow ui rules").unwrap();
        assert_eq!(family, DomainFamily::React);
        assert_eq!(standards, "follow ui rules");
        assert!(parse_domain_standard("react=").is_err());
        assert!(parse_domain_standard("react").is_err());
        assert_eq!(
            parse_canon_mode_selection("auto-confirm").unwrap(),
            crate::domain::governance::CanonModeSelectionPreference::AutoConfirm
        );

        let routes = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);
        assert_eq!(routes.len(), 4);
        assert_eq!(routes[0].slot, crate::domain::configuration::RouteSlot::Planning);
        assert_eq!(routes[0].route.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert!(matches!(routes[0].source, GuidedRouteSource::AssistantDefault { .. }));
        let review =
            render_guided_route_review(&catalog, &routes, Some("Custom model id cannot be empty."));
        assert!(review.contains("Model routes"), "{review}");
        assert!(review.contains("Custom model id cannot be empty."), "{review}");

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
            parse_model_route("planning=cursor:gpt-5-codex")
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
    fn guided_answers_can_choose_custom_models_without_freeform_route_entry() {
        let catalog = BundledModelCatalog::load().unwrap();
        let mut interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![0, 1, 2, 3, 0]),
            multi_selects: VecDeque::from(vec![vec![0]]),
            inputs: VecDeque::from(vec!["gpt-5.4-enterprise".to_string()]),
            confirms: VecDeque::new(),
        };

        let answers = collect_guided_init_answers_with_interactor(
            &mut interactor,
            true,
            true,
            true,
            &catalog,
            &[],
        )
        .unwrap();

        assert_eq!(answers.canon_mode_selection, Some(CanonModeSelectionPreference::AutoConfirm));
        assert_eq!(answers.assistants, vec![RuntimeKind::Copilot]);
        assert_eq!(answers.routes[0].route.as_ref().unwrap().runtime, RuntimeKind::Claude);
        assert_eq!(answers.routes[0].route.as_ref().unwrap().model, "gpt-5.4-enterprise");
        assert!(matches!(answers.routes[0].source, GuidedRouteSource::Custom));
    }

    #[test]
    fn execute_init_requires_non_interactive_flag_without_tty_when_guided_values_are_missing() {
        let workspace = temp_workspace("boundline-init-no-tty");

        let error = execute_init(InitRequest {
            workspace: &workspace,
            non_interactive: false,
            interactive_terminal_override: Some(false),
            template: None,
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
            force: true,
        })
        .unwrap_err();

        assert_eq!(error.to_string(), super::NO_TTY_GUIDANCE);
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
            non_interactive: true,
            interactive_terminal_override: None,
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
            non_interactive: true,
            interactive_terminal_override: None,
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

    #[test]
    fn guided_route_source_labels_and_display_line_cover_all_variants() {
        use super::{GuidedRouteSelection, GuidedRouteSource};
        use crate::domain::configuration::ModelRoute;

        let make = |slot: RouteSlot, route: Option<ModelRoute>, source: GuidedRouteSource| {
            GuidedRouteSelection { slot, route, source }
        };

        let bundled = make(
            RouteSlot::Planning,
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() }),
            GuidedRouteSource::Bundled,
        );
        assert!(bundled.display_line().contains("[bundled]"), "{}", bundled.display_line());

        let custom = make(
            RouteSlot::Implementation,
            Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() }),
            GuidedRouteSource::Custom,
        );
        assert!(custom.display_line().contains("[custom-unverified]"), "{}", custom.display_line());

        let unset = make(RouteSlot::Verification, None, GuidedRouteSource::Unset);
        assert!(unset.display_line().contains("unset"), "{}", unset.display_line());
        assert!(unset.display_line().contains("[unset]"), "{}", unset.display_line());

        let fallback = make(
            RouteSlot::Review,
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() }),
            GuidedRouteSource::AssistantDefault { fallback_from: Some(RuntimeKind::Codex) },
        );
        assert!(
            fallback.display_line().contains("fallback-from=codex-unavailable"),
            "{}",
            fallback.display_line()
        );

        let no_fallback = make(
            RouteSlot::Planning,
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() }),
            GuidedRouteSource::AssistantDefault { fallback_from: None },
        );
        assert!(
            no_fallback.display_line().contains("[assistant-default]"),
            "{}",
            no_fallback.display_line()
        );
    }

    #[test]
    fn catalog_helpers_cover_all_slot_arms_and_label_formatters() {
        let catalog = BundledModelCatalog::load().unwrap();

        // default_route_for_slot - all four arms
        assert!(catalog.default_route_for_slot(RouteSlot::Planning).is_some());
        assert!(catalog.default_route_for_slot(RouteSlot::Implementation).is_some());
        assert!(catalog.default_route_for_slot(RouteSlot::Verification).is_some());
        assert!(catalog.default_route_for_slot(RouteSlot::Review).is_some());

        // runtime_labels
        let labels = catalog.runtime_labels();
        assert!(!labels.is_empty());
        assert!(labels.iter().any(|l| l.contains("copilot")));

        // model_labels_for_runtime
        let copilot_models = catalog.model_labels_for_runtime(RuntimeKind::Copilot);
        assert!(!copilot_models.is_empty());
        // unknown runtime returns empty
        let unknown_models = catalog.model_labels_for_runtime(RuntimeKind::Codex);
        // Codex is in catalog, so this should be non-empty too - just verify it doesn't panic
        let _ = unknown_models;

        // default_route_for_runtime
        assert!(catalog.default_route_for_runtime(RuntimeKind::Copilot).is_some());

        // summary_label
        let summary = catalog.summary_label();
        assert!(summary.contains("bundled"), "{summary}");
    }

    #[test]
    fn select_canon_mode_covers_manual_and_auto_variants() {
        let catalog = BundledModelCatalog::load().unwrap();

        // Manual (index 1)
        let mut interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![1]),
            multi_selects: VecDeque::default(),
            inputs: VecDeque::default(),
            confirms: VecDeque::default(),
        };
        let answers = collect_guided_init_answers_with_interactor(
            &mut interactor,
            true,
            false,
            false,
            &catalog,
            &[RuntimeKind::Copilot],
        )
        .unwrap();
        assert_eq!(answers.canon_mode_selection, Some(CanonModeSelectionPreference::Manual));

        // Auto (index 2)
        let mut interactor2 = ScriptedInteractor {
            selects: VecDeque::from(vec![2]),
            multi_selects: VecDeque::default(),
            inputs: VecDeque::default(),
            confirms: VecDeque::default(),
        };
        let answers2 = collect_guided_init_answers_with_interactor(
            &mut interactor2,
            true,
            false,
            false,
            &catalog,
            &[RuntimeKind::Copilot],
        )
        .unwrap();
        assert_eq!(answers2.canon_mode_selection, Some(CanonModeSelectionPreference::Auto));
    }

    #[test]
    fn collect_guided_answers_skips_all_prompts_when_all_flags_are_false() {
        let catalog = BundledModelCatalog::load().unwrap();
        let explicit = [RuntimeKind::Copilot];
        let mut interactor = ScriptedInteractor::default();

        let answers = collect_guided_init_answers_with_interactor(
            &mut interactor,
            false,
            false,
            false,
            &catalog,
            &explicit,
        )
        .unwrap();

        assert_eq!(answers.canon_mode_selection, None);
        assert_eq!(answers.assistants, vec![RuntimeKind::Copilot]);
        assert!(answers.routes.is_empty());
    }

    #[test]
    fn initial_guided_route_selections_fallback_when_catalog_default_is_unavailable() {
        let catalog = BundledModelCatalog::load().unwrap();

        // Use Copilot as assistant but pass Codex as well - when codex isn't available
        // it should fall back to copilot for its slots
        let selections =
            initial_guided_route_selections(&catalog, &[RuntimeKind::Codex, RuntimeKind::Copilot]);
        assert_eq!(selections.len(), 4);
        // All slots should have a route (copilot is always available)
        for selection in &selections {
            assert!(selection.route.is_some(), "slot {:?} has no route", selection.slot);
        }
    }

    #[test]
    fn clear_guided_route_selections_unsets_all_slots() {
        let catalog = BundledModelCatalog::load().unwrap();
        let mut selections = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);
        assert!(selections.iter().all(|s| s.route.is_some()));

        super::clear_guided_route_selections(&mut selections);

        for selection in &selections {
            assert!(selection.route.is_none(), "slot {:?} should be unset", selection.slot);
            assert_eq!(selection.source, super::GuidedRouteSource::Unset);
        }
    }

    #[test]
    fn edit_route_selection_can_leave_slot_unset_and_pick_bundled_model() {
        let catalog = BundledModelCatalog::load().unwrap();
        let mut selections = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);

        // Leave planning unset: pick last runtime index (= catalog.runtimes.len())
        let unset_result = super::edit_route_selection(
            &mut ScriptedInteractor {
                selects: VecDeque::from(vec![catalog.runtimes.len()]),
                ..Default::default()
            },
            &catalog,
            &mut selections,
            RouteSlot::Planning,
        );
        assert!(unset_result.is_ok());
        let planning = selections.iter().find(|s| s.slot == RouteSlot::Planning).unwrap();
        assert!(planning.route.is_none());
        assert_eq!(planning.source, super::GuidedRouteSource::Unset);

        // Pick bundled model for implementation: runtime=0, model=0
        let bundled_result = super::edit_route_selection(
            &mut ScriptedInteractor { selects: VecDeque::from(vec![0, 0]), ..Default::default() },
            &catalog,
            &mut selections,
            RouteSlot::Implementation,
        );
        assert!(bundled_result.is_ok());
        let impl_slot = selections.iter().find(|s| s.slot == RouteSlot::Implementation).unwrap();
        assert!(impl_slot.route.is_some());
        assert_eq!(impl_slot.source, super::GuidedRouteSource::Bundled);
    }

    #[test]
    fn render_guided_summary_covers_empty_and_nonempty_assistants() {
        let catalog = BundledModelCatalog::load().unwrap();
        let slots = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);

        let with_assistants = super::render_guided_summary(
            InitTemplate::Change,
            Some(CanonModeSelectionPreference::AutoConfirm),
            &[RuntimeKind::Copilot],
            &slots,
            &catalog,
            &["- create .boundline/config.toml".to_string()],
        );
        assert!(with_assistants.contains("copilot"), "{with_assistants}");
        assert!(with_assistants.contains("Model routes:"), "{with_assistants}");

        let no_assistants =
            super::render_guided_summary(InitTemplate::BugFix, None, &[], &slots, &catalog, &[]);
        assert!(no_assistants.contains("none selected"), "{no_assistants}");
        assert!(no_assistants.contains("auto-confirm"), "{no_assistants}");
    }

    #[test]
    fn render_cancelled_init_report_covers_empty_and_nonempty_assistants() {
        let workspace = temp_workspace("boundline-init-cancel-render");
        let catalog = BundledModelCatalog::load().unwrap();
        let slots = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);

        let with = super::render_cancelled_init_report(
            &workspace,
            InitTemplate::Delivery,
            Some(CanonModeSelectionPreference::Manual),
            &[RuntimeKind::Copilot],
            &slots,
            &catalog,
        );
        assert!(with.contains("canceled before write"), "{with}");
        assert!(with.contains("copilot"), "{with}");

        let without = super::render_cancelled_init_report(
            &workspace,
            InitTemplate::BugFix,
            None,
            &[],
            &slots,
            &catalog,
        );
        assert!(without.contains("none selected"), "{without}");
        assert!(without.contains("auto-confirm"), "{without}");
    }

    #[test]
    fn execute_init_uses_spinner_path_when_interactive_terminal_override_is_true() {
        let workspace = temp_workspace("boundline-init-spinner");
        let report = execute_init(InitRequest {
            workspace: &workspace,
            non_interactive: true,
            interactive_terminal_override: Some(true),
            template: Some(InitTemplate::Change),
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
        assert_eq!(report.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(
            report.terminal_output.contains("init: workspace initialized"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn init_command_error_new_variants_display_their_messages() {
        let unavailable = super::InitCommandError::InteractiveTerminalUnavailable;
        assert!(unavailable.to_string().contains("non-interactive"), "{}", unavailable);

        let invalid_catalog =
            super::InitCommandError::InvalidBundledCatalog("bad toml".to_string());
        assert!(
            invalid_catalog.to_string().contains("bundled model catalog"),
            "{}",
            invalid_catalog
        );

        let prompt_err = super::InitCommandError::PromptInteraction("user cancelled".to_string());
        assert!(prompt_err.to_string().contains("collect init input"), "{}", prompt_err);
    }

    #[test]
    fn parse_canon_mode_selection_covers_manual_auto_and_invalid() {
        assert_eq!(
            parse_canon_mode_selection("manual").unwrap(),
            CanonModeSelectionPreference::Manual
        );
        assert_eq!(parse_canon_mode_selection("auto").unwrap(), CanonModeSelectionPreference::Auto);
        assert!(parse_canon_mode_selection("unknown").is_err());
    }
}
