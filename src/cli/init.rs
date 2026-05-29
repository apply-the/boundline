#[path = "init/preview.rs"]
mod preview;

#[path = "init/report.rs"]
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::ValueEnum;
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use thiserror::Error;

use super::assistant_assets::{
    AssistantAsset, AssistantSurface, DocsExportAsset, DocsExportSurface, assets_for_assistants,
    docs_assets_for_assistants, docs_assets_for_assistants_under,
};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::adapters::env_layer::{
    ProviderEnvTemplateScope, global_env_template_path as provider_global_env_template_path,
    render_provider_env_template,
    workspace_env_template_path as provider_workspace_env_template_path,
};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{
    AssistantHostKind, CanonPreferences, ConfigFile, IdeKind, InitConfigScope, InitTemplate,
    ModelRoute, RouteSlot, RoutingOverrides, RuntimeKind, TerminalAutoApproveProfile,
    built_in_default_route, resolve_effective_routing, seeded_routes_for_assistants,
};
use crate::domain::distribution::CanonInstallStatus;
#[cfg(not(test))]
use crate::domain::distribution::evaluate_canon_install;
use crate::domain::domain_templates::{
    DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
    detect_domain_families,
};
use crate::domain::governance::CanonModeSelectionPreference;
use crate::domain::governance::CanonRiskClass;
use crate::domain::project_index::{ProjectDocRoots, resolve_project_doc_roots};
use crate::domain::scaffold_manifest::{
    IdeSetupSelection, SCAFFOLD_MANIFEST_FILE_NAME, ScaffoldManifest, ScaffoldManifestEntry,
    ScaffoldOwnershipMode, ScaffoldTarget,
};
use crate::domain::workspace_hygiene::{merge_hygiene_content, plan_hygiene_defaults};

use self::preview::{
    InitPlannedChangesInput, WorkspaceInitPreview, build_init_planned_changes,
    prepare_workspace_init_preview,
};
use self::report::{
    render_cancelled_init_report, render_canon_workspace_bootstrap_failure_report,
    render_guided_summary, render_init_canon_bootstrap_blocked_report,
    render_init_preview_only_report, render_successful_init_report,
    render_update_adopt_required_report, render_update_applied_report, render_update_diff_report,
    render_update_force_required_report, render_update_preview_report, render_update_status_report,
};

const INIT_ROUTE_EXAMPLE: &str = "planning=copilot:gpt-4.1";
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
const DEFAULT_CANON_COMMAND: &str = "canon";
const DEFAULT_CANON_RISK: &str = CanonRiskClass::BoundedImpact.as_str();
const DEFAULT_CANON_ZONE: &str = "engineering";
const DEFAULT_CANON_OWNER: &str = "platform";
const DEFAULT_CANON_SYSTEM_CONTEXT: &str = "existing";
const BOUNDLINE_DIR_RELATIVE: &str = ".boundline";
const EXECUTION_PROFILE_FILE_NAME: &str = "execution.json";
const BOUNDLINE_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_MEMORY_ROOT_README: &str = "# Project Memory\n\nUse this folder for stable repo-visible inputs that Boundline and Canon can reuse across planning and delivery. Keep curated project context here, such as domain language, architecture maps, operating constraints, and other maintained reference material.\n\nDo not store transient runtime artifacts here. Boundline runtime state stays under `.boundline/`, and raw Canon packets stay under `.canon/`.\n";
const EVIDENCE_ROOT_README: &str = "# Evidence\n\nUse this folder for consolidated repo-visible feature outputs and evidence bundles that should remain readable after a delivery cycle completes. A typical layout is `docs/evidence/<feature-slug>/...`.\n\nDo not treat this folder as raw runtime storage. Boundline keeps transient governance artifacts under `.boundline/`, and Canon keeps raw run packets under `.canon/`.\n";
#[cfg(not(test))]
const CANON_INIT_SUBCOMMAND: &str = "init";
#[cfg(not(test))]
const CANON_AI_FLAG: &str = "--ai";
#[cfg(not(test))]
const CANON_OUTPUT_FLAG: &str = "--output";
#[cfg(not(test))]
const CANON_OUTPUT_JSON: &str = "json";
const CANON_WORKSPACE_ROOT_RELATIVE: &str = ".canon";
const CANON_AGENT_SKILLS_RELATIVE: &str = ".agents/skills";
const CANON_BOOTSTRAP_NOTE_LABEL: &str = "canon_workspace_bootstrap";
const CANON_SAFETY_REVIEWER_ROLE_ID: &str = "safety";
const CANON_MAINTAINABILITY_REVIEWER_ROLE_ID: &str = "maintainability";
const CANON_REVIEWER_ROUTE_REPAIR_ACTION: &str =
    "set distinct routing.reviewer_roles entries for safety and maintainability";
const CANON_SAFETY_REVIEWER_SLOT_ORDER: [RouteSlot; 4] =
    [RouteSlot::Verification, RouteSlot::Review, RouteSlot::Planning, RouteSlot::Implementation];
const CANON_MAINTAINABILITY_REVIEWER_SLOT_ORDER: [RouteSlot; 4] =
    [RouteSlot::Review, RouteSlot::Planning, RouteSlot::Verification, RouteSlot::Implementation];

#[cfg(test)]
static CANON_INSTALL_STATUS_OVERRIDE: std::sync::OnceLock<
    std::sync::Mutex<Option<CanonInstallStatus>>,
> = std::sync::OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonBootstrapReadiness {
    ready: bool,
    state: &'static str,
    detail: String,
    repair_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonReviewerRouteReadiness {
    ready: bool,
    detail: String,
    repair_actions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonInitAssistantHost {
    Codex,
    Copilot,
    Claude,
}

impl CanonInitAssistantHost {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Copilot => "copilot",
            Self::Claude => "claude",
        }
    }

    const fn from_runtime(runtime: RuntimeKind) -> Option<Self> {
        match runtime {
            RuntimeKind::Codex => Some(Self::Codex),
            RuntimeKind::Copilot => Some(Self::Copilot),
            RuntimeKind::Claude => Some(Self::Claude),
            RuntimeKind::Gemini => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CanonWorkspaceBootstrapReport {
    repo_root: PathBuf,
    canon_root: PathBuf,
    methods_materialized: usize,
    policies_materialized: usize,
    skills_materialized: usize,
    claude_md_created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub compact_output: String,
}

impl InitCommandReport {
    fn new(exit_status: CommandExitStatus, terminal_output: impl Into<String>) -> Self {
        let terminal_output = terminal_output.into();
        let compact_output = render_init_brief(exit_status, &terminal_output);
        Self { exit_status, terminal_output, compact_output }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

impl UpdateCommandReport {
    fn new(exit_status: CommandExitStatus, terminal_output: impl Into<String>) -> Self {
        Self { exit_status, terminal_output: terminal_output.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum UpdateTarget {
    Config,
    Execution,
    Assistant,
    Docs,
    Hygiene,
    Ide,
}

impl UpdateTarget {
    const fn label(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Execution => "execution",
            Self::Assistant => "assistant",
            Self::Docs => "docs",
            Self::Hygiene => "hygiene",
            Self::Ide => "ide",
        }
    }
}

fn render_init_brief(exit_status: CommandExitStatus, terminal_output: &str) -> String {
    let lines = terminal_output.lines().collect::<Vec<_>>();
    let mut compact = Vec::new();

    if let Some(outcome) = lines.first().copied().filter(|line| !line.trim().is_empty()) {
        compact.push(outcome.to_string());
    }

    for label in ["scope", "template"] {
        if let Some(value) = init_value(&lines, label) {
            compact.push(format!("{label}: {value}"));
        }
    }

    if let Some(summary_line) = init_summary_brief_line(&lines) {
        compact.push(summary_line);
    }

    if let Some(artifacts_line) = init_artifacts_brief_line(&lines) {
        compact.push(artifacts_line);
    }

    if let Some(governance_line) = init_governance_brief_line(&lines) {
        compact.push(governance_line);
    }

    compact.push(format!("latest_status: {}", init_latest_status(exit_status, &lines)));

    if let Some(next_command) = init_next_command(&lines) {
        compact.push(format!("next_command: {next_command}"));
    }

    compact.join("\n")
}

fn init_summary_brief_line(lines: &[&str]) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(canon_mode_selection) = init_value(lines, "canon_mode_selection") {
        parts.push(format!("canon_mode_selection={canon_mode_selection}"));
    }

    if lines.contains(&"assistant_setup:") {
        parts.push("assistant_setup=repo-local".to_string());
    } else if let Some(assistant_setup) = init_value(lines, "assistant_setup") {
        parts.push(format!("assistant_setup={assistant_setup}"));
    }

    if lines.contains(&"docs_export:") {
        parts.push("docs_export=exported".to_string());
    } else if let Some(docs_export) = init_value(lines, "docs_export") {
        parts.push(format!("docs_export={docs_export}"));
    }

    if lines.contains(&"workspace_hygiene:") {
        parts.push("workspace_hygiene=applied".to_string());
    } else if let Some(workspace_hygiene) = init_value(lines, "workspace_hygiene") {
        parts.push(format!("workspace_hygiene={workspace_hygiene}"));
    }

    if let Some(workspace_artifacts) = init_value(lines, "workspace_artifacts") {
        parts.push(format!("workspace_artifacts={workspace_artifacts}"));
    }

    (!parts.is_empty()).then(|| format!("summary: {}", parts.join("; ")))
}

fn init_artifacts_brief_line(lines: &[&str]) -> Option<String> {
    let mut parts = Vec::new();

    for label in [
        "global_config",
        "global_provider_env_template",
        "execution_profile",
        "workspace_config",
        "workspace_provider_env_template",
    ] {
        if let Some(value) = init_value(lines, label) {
            parts.push(format!("{label}={value}"));
        }
    }

    if let Some(canon_root) =
        init_section_bullet_value(lines, CANON_BOOTSTRAP_NOTE_LABEL, "canon_root")
    {
        parts.push(format!("canon_root={canon_root}"));
    }

    (!parts.is_empty()).then(|| format!("artifacts: {}", parts.join("; ")))
}

fn init_governance_brief_line(lines: &[&str]) -> Option<String> {
    let mut parts = Vec::new();

    for label in ["canon_mode_selection", "canon_bootstrap", "canon_surface"] {
        if let Some(value) = init_value(lines, label) {
            parts.push(format!("{label}={value}"));
        }
    }

    (!parts.is_empty()).then(|| format!("governance: {}", parts.join("; ")))
}

fn init_latest_status(exit_status: CommandExitStatus, lines: &[&str]) -> &'static str {
    if let Some(outcome) = lines.first().copied() {
        if outcome.starts_with("init: blocked") || outcome.starts_with("init: preview only") {
            return "blocked";
        }
        if outcome.starts_with("init: canceled") {
            return "canceled";
        }
    }

    match exit_status {
        CommandExitStatus::Succeeded => "succeeded",
        CommandExitStatus::NonSuccess | CommandExitStatus::InvalidInvocation => "failed",
        CommandExitStatus::TraceReadFailure => "failed",
    }
}

fn init_next_command<'a>(lines: &'a [&str]) -> Option<&'a str> {
    init_section_bullets(lines, "next_steps")
        .into_iter()
        .next()
        .or_else(|| init_value(lines, "repair_command"))
}

fn init_value<'a>(lines: &'a [&str], label: &str) -> Option<&'a str> {
    lines.iter().find_map(|line| line.strip_prefix(&format!("{label}: ")))
}

fn init_section_bullet_value<'a>(
    lines: &'a [&str],
    section_label: &str,
    bullet_label: &str,
) -> Option<&'a str> {
    init_section_bullets(lines, section_label)
        .into_iter()
        .find_map(|line| line.strip_prefix(&format!("{bullet_label}: ")))
}

fn init_section_bullets<'a>(lines: &'a [&str], section_label: &str) -> Vec<&'a str> {
    let Some(section_index) =
        lines.iter().position(|line| *line == format!("{section_label}:").as_str())
    else {
        return Vec::new();
    };

    let mut bullets = Vec::new();
    for line in lines.iter().skip(section_index + 1) {
        if let Some(bullet) = line.strip_prefix("- ") {
            bullets.push(bullet);
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        if line.contains(':') {
            break;
        }
    }
    bullets
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

    fn model_routes_for_runtime(&self, runtime: RuntimeKind) -> Vec<ModelRoute> {
        self.runtime_entry(runtime)
            .map(|entry| {
                entry
                    .models
                    .iter()
                    .map(|model| ModelRoute { runtime, model: model.model_id.clone() })
                    .collect()
            })
            .unwrap_or_default()
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

pub trait InitInteractor: std::fmt::Debug {
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

#[derive(Debug)]
pub struct InitRequest<'a> {
    pub workspace: &'a Path,
    pub scope: InitConfigScope,
    pub non_interactive: bool,
    /// Override TTY detection for testing. `None` means auto-detect from stdin/stdout.
    pub interactive_terminal_override: Option<bool>,
    /// Inject a custom interactor for testing. `None` uses `DialoguerInitInteractor`.
    pub interactor: Option<Box<dyn InitInteractor>>,
    pub template: Option<InitTemplate>,
    pub assistants: &'a [AssistantHostKind],
    pub routes: &'a [String],
    pub domains: &'a [DomainFamily],
    pub domain_standards: &'a [String],
    pub context_bindings: &'a [String],
    pub required_context_bindings: &'a [String],
    pub canon_mode_selection: Option<CanonModeSelectionPreference>,
    pub risk: Option<&'a str>,
    pub zone: Option<&'a str>,
    pub owner: Option<&'a str>,
    pub ide: &'a [IdeKind],
    pub auto_approve: Option<TerminalAutoApproveProfile>,
    pub export_docs: bool,
    pub docs_refresh: bool,
    pub docs_diff: bool,
    pub docs_output_dir: Option<&'a Path>,
    pub force: bool,
}

#[derive(Debug)]
pub struct UpdateRequest<'a> {
    pub workspace: &'a Path,
    pub targets: &'a [UpdateTarget],
    pub ide: &'a [IdeKind],
    pub auto_approve: Option<TerminalAutoApproveProfile>,
    pub template: Option<InitTemplate>,
    pub diff: bool,
    pub apply: bool,
    pub adopt: bool,
    pub prune: bool,
    pub status: bool,
    pub force: bool,
}

#[derive(Debug, Clone)]
struct ResolvedInitInputs {
    catalog: BundledModelCatalog,
    template: InitTemplate,
    interactive_terminal: bool,
    guided_answers: Option<GuidedInitAnswers>,
    effective_canon_mode_selection: Option<CanonModeSelectionPreference>,
    effective_assistants: Vec<AssistantHostKind>,
    explicit_routes: Vec<(RouteSlot, ModelRoute)>,
    guided_routes: Vec<(RouteSlot, ModelRoute)>,
    seeded_routes: Vec<SeededRouteSelection>,
    effective_routes: Vec<(RouteSlot, ModelRoute)>,
    requested_domain_templates: BTreeMap<DomainFamily, DomainTemplateSettings>,
}

#[derive(Debug, Clone, Default)]
struct StoredInitDefaults {
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    assistants: Vec<AssistantHostKind>,
    routes: Vec<(RouteSlot, ModelRoute)>,
}

const INIT_ASSISTANT_HOSTS: [AssistantHostKind; 4] = [
    AssistantHostKind::Claude,
    AssistantHostKind::Codex,
    AssistantHostKind::Copilot,
    AssistantHostKind::Antigravity,
];

fn assistant_host_labels() -> Vec<String> {
    INIT_ASSISTANT_HOSTS
        .iter()
        .map(|host| match host {
            AssistantHostKind::Claude => "Claude Code (claude)".to_string(),
            AssistantHostKind::Codex => "Codex (codex)".to_string(),
            AssistantHostKind::Copilot => "Copilot (copilot)".to_string(),
            AssistantHostKind::Antigravity => "Antigravity (antigravity)".to_string(),
        })
        .collect()
}

fn assistant_runtimes_for_hosts(assistants: &[AssistantHostKind]) -> Vec<RuntimeKind> {
    let mut runtimes = Vec::new();
    for host in assistants {
        if let Some(runtime) = host.default_runtime()
            && !runtimes.contains(&runtime)
        {
            runtimes.push(runtime);
        }
    }
    runtimes
}

fn configured_assistant_hosts(config: &ConfigFile) -> Vec<AssistantHostKind> {
    if !config.routing.assistant_hosts.is_empty() {
        return config.routing.assistant_hosts.clone();
    }

    let mut assistants = Vec::new();
    for runtime in &config.routing.assistant_runtimes {
        if let Some(host) = AssistantHostKind::from_runtime(*runtime)
            && !assistants.contains(&host)
        {
            assistants.push(host);
        }
    }
    assistants
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannedHygieneEntry {
    action: HygieneInitAction,
    final_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannedIdeEntry {
    action: IdeInitAction,
    artifact: RenderedManagedArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedManagedArtifact {
    path: PathBuf,
    target: ScaffoldTarget,
    ownership: ScaffoldOwnershipMode,
    contents: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdatePlanAction {
    Create,
    Replace,
    Merge,
    Remove,
    Adopt,
    AdoptCurrent,
    Orphaned,
    Unchanged,
    Conflict,
}

impl UpdatePlanAction {
    const fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Replace => "replace",
            Self::Merge => "merge",
            Self::Remove => "remove",
            Self::Adopt => "adopt",
            Self::AdoptCurrent => "adopt-current",
            Self::Orphaned => "orphaned",
            Self::Unchanged => "unchanged",
            Self::Conflict => "conflict",
        }
    }

    const fn is_change(self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdatePlanEntry {
    path: String,
    target: ScaffoldTarget,
    ownership: ScaffoldOwnershipMode,
    action: UpdatePlanAction,
    detail: String,
    tracked: bool,
    requires_force: bool,
    requires_adopt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdatePlan {
    entries: Vec<UpdatePlanEntry>,
    manifest_after_apply: ScaffoldManifest,
    manifest_present: bool,
}

impl UpdatePlan {
    fn has_changes(&self) -> bool {
        self.entries.iter().any(|entry| entry.action.is_change())
    }

    fn requires_force(&self) -> bool {
        self.entries.iter().any(|entry| entry.requires_force)
    }

    fn requires_adopt(&self) -> bool {
        self.entries.iter().any(|entry| entry.requires_adopt)
    }

    fn count_by_action(&self, action: UpdatePlanAction) -> usize {
        self.entries.iter().filter(|entry| entry.action == action).count()
    }
}

#[derive(Debug)]
struct InitApplyInputs<'a> {
    scope: InitConfigScope,
    template: InitTemplate,
    interactive_terminal: bool,
    workspace: Option<&'a Path>,
    planned_changes: &'a [String],
    workspace_canon_selected: bool,
    canon_init_assistant: Option<CanonInitAssistantHost>,
    global_env_template_path: Option<&'a Path>,
    global_env_template_contents: Option<&'a str>,
    global_config: Option<&'a ConfigFile>,
    execution_path: Option<&'a Path>,
    execution_contents: Option<&'a str>,
    local_config: Option<&'a ConfigFile>,
    local_env_template_path: Option<&'a Path>,
    local_env_template_contents: Option<&'a str>,
    assistant_assets: &'a [AssistantAsset],
    docs_plan: &'a [DocsExportPlanEntry],
    hygiene_plan: &'a [PlannedHygieneEntry],
    ide_plan: &'a [PlannedIdeEntry],
    manifest_path: Option<&'a Path>,
    manifest_contents: Option<&'a str>,
}

#[derive(Debug, Default)]
struct InitApplyOutcome {
    project_doc_roots: Option<ProjectDocRoots>,
    hygiene_actions: Vec<HygieneInitAction>,
    assistant_actions: Vec<AssistantInitAction>,
    docs_actions: Vec<DocsInitAction>,
    ide_actions: Vec<IdeInitAction>,
    canon_workspace_bootstrap: Option<CanonWorkspaceBootstrapReport>,
}

enum InitApplyExit {
    Applied(InitApplyOutcome),
    Report(InitCommandReport),
}

struct InitSuccessReportInputs<'a> {
    scope: InitConfigScope,
    export_docs: bool,
    docs_output_dir: Option<&'a Path>,
    resolved: &'a ResolvedInitInputs,
    workspace: Option<&'a Path>,
    global_config_path: Option<&'a Path>,
    global_env_template_path: Option<&'a Path>,
    execution_path: Option<&'a Path>,
    local_config_path: Option<&'a Path>,
    local_env_template_path: Option<&'a Path>,
    manifest_path: Option<&'a Path>,
    project_doc_roots: Option<&'a ProjectDocRoots>,
    local_config: Option<&'a ConfigFile>,
    global_config: Option<&'a ConfigFile>,
    canon_bootstrap: Option<&'a CanonBootstrapReadiness>,
    canon_workspace_bootstrap: Option<&'a CanonWorkspaceBootstrapReport>,
    canon_init_assistant: Option<CanonInitAssistantHost>,
    assistant_actions: &'a [AssistantInitAction],
    docs_actions: &'a [DocsInitAction],
    ide_actions: &'a [IdeInitAction],
    hygiene_actions: &'a [HygieneInitAction],
}

/// Applies the scaffold changes after `execute_init` has already completed all
/// preview-only and blocked guards.
///
/// The helper returns either the collected action summaries for final output or
/// an already-rendered stop report when Canon workspace bootstrap fails.
fn apply_init_scaffold_changes(
    inputs: InitApplyInputs<'_>,
) -> Result<InitApplyExit, InitCommandError> {
    let InitApplyInputs {
        scope,
        template,
        interactive_terminal,
        workspace,
        planned_changes,
        workspace_canon_selected,
        canon_init_assistant,
        global_env_template_path,
        global_env_template_contents,
        global_config,
        execution_path,
        execution_contents,
        local_config,
        local_env_template_path,
        local_env_template_contents,
        assistant_assets,
        docs_plan,
        hygiene_plan,
        ide_plan,
        manifest_path,
        manifest_contents,
    } = inputs;
    let mut outcome = InitApplyOutcome::default();

    if let Some(workspace) = workspace
        && workspace_canon_selected
    {
        let bootstrap_result =
            run_init_activity("bootstrapping Canon workspace", interactive_terminal, || {
                materialize_canon_workspace(workspace, canon_init_assistant)
                    .map_err(InitCommandError::CanonWorkspaceBootstrap)
            });
        match bootstrap_result {
            Ok(report) => outcome.canon_workspace_bootstrap = Some(report),
            Err(InitCommandError::CanonWorkspaceBootstrap(detail)) => {
                return Ok(InitApplyExit::Report(render_canon_workspace_bootstrap_failure_report(
                    scope,
                    workspace,
                    template,
                    &detail,
                    planned_changes,
                )));
            }
            Err(error) => return Err(error),
        }
    }

    if let (Some(path), Some(contents)) = (global_env_template_path, global_env_template_contents) {
        run_init_activity("writing global provider env template", interactive_terminal, || {
            write_scaffold_file(path, contents)
        })?;
    }

    if let Some(config) = global_config {
        run_init_activity("writing global config", interactive_terminal, || {
            FileConfigStore::save_global(config)?;
            Ok(())
        })?;
    }

    if let Some(workspace) = workspace {
        let boundline_dir = workspace.join(BOUNDLINE_DIR_RELATIVE);
        fs::create_dir_all(&boundline_dir).map_err(|source| InitCommandError::WriteFile {
            path: boundline_dir.clone(),
            source,
        })?;
        outcome.project_doc_roots = Some(ensure_workspace_project_doc_roots(workspace)?);
        outcome.hygiene_actions = apply_workspace_hygiene_plan(workspace, hygiene_plan)?;

        if let (Some(path), Some(contents)) = (execution_path, execution_contents) {
            run_init_activity("writing execution profile", interactive_terminal, || {
                fs::write(path, contents).map_err(|source| InitCommandError::WriteFile {
                    path: path.to_path_buf(),
                    source,
                })
            })?;
        }

        if let Some(local) = local_config {
            let store = FileConfigStore::for_workspace(workspace);
            run_init_activity("writing workspace config", interactive_terminal, || {
                store.save_local(local)?;
                Ok(())
            })?;
        }

        if let (Some(path), Some(contents)) = (local_env_template_path, local_env_template_contents)
        {
            run_init_activity(
                "writing workspace provider env template",
                interactive_terminal,
                || write_scaffold_file(path, contents),
            )?;
        }

        outcome.assistant_actions =
            run_init_activity("scaffolding assistant packs", interactive_terminal, || {
                apply_assistant_assets(workspace, assistant_assets)
            })?;
        outcome.docs_actions =
            run_init_activity("exporting repo-local docs", interactive_terminal, || {
                apply_docs_plan(workspace, docs_plan)
            })?;
        outcome.ide_actions =
            run_init_activity("scaffolding IDE setup", interactive_terminal, || {
                apply_ide_setup(workspace, ide_plan)
            })?;

        if let (Some(path), Some(contents)) = (manifest_path, manifest_contents) {
            run_init_activity("writing scaffold manifest", interactive_terminal, || {
                write_scaffold_file(path, contents)
            })?;
        }
    }

    Ok(InitApplyExit::Applied(outcome))
}

pub fn execute_init(mut request: InitRequest<'_>) -> Result<InitCommandReport, InitCommandError> {
    validate_docs_export_options(&request)?;

    validate_init_scope_options(&request)?;

    let workspace_root = if scope_includes_workspace(request.scope) {
        let root = resolve_workspace_root(request.workspace)?;
        fs::create_dir_all(&root)
            .map_err(|source| InitCommandError::CreateWorkspace { path: root.clone(), source })?;
        Some(root)
    } else {
        None
    };
    let workspace = workspace_root.as_deref();
    let resolved = resolve_init_inputs(&mut request, workspace)?;

    if let Some(preflight_report) = canon_surface_preflight(&request, &resolved, workspace)? {
        return Ok(preflight_report);
    }

    let global_config_path =
        scope_includes_global(request.scope).then(FileConfigStore::global_config_path);
    let global_env_template_path =
        scope_includes_global(request.scope).then(provider_global_env_template_path);
    let global_env_template_contents = scope_includes_global(request.scope)
        .then(|| render_provider_env_template(ProviderEnvTemplateScope::Global));
    let (global_status, global_config) = if scope_includes_global(request.scope) {
        let existing = FileConfigStore::load_global()?;
        let mut config = existing.clone().unwrap_or_default();
        apply_init_preferences(
            &mut config,
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
        let status = match existing.as_ref() {
            Some(saved) if saved == &config => ScaffoldFileStatus::Unchanged,
            Some(_) => ScaffoldFileStatus::Update,
            None => ScaffoldFileStatus::Create,
        };
        (Some(status), Some(config))
    } else {
        (None, None)
    };
    let global_env_template_status = if let (Some(path), Some(contents)) =
        (global_env_template_path.as_ref(), global_env_template_contents.as_ref())
    {
        Some(scaffold_file_status(path, contents)?)
    } else {
        None
    };

    let mut local_config = None;
    let mut local_config_path = None;
    let mut local_env_template_path = None;
    let mut local_env_template_contents = None;
    let mut execution_path = None;
    let mut execution_contents = None;
    let mut assistant_assets = Vec::new();
    let mut docs_plan = Vec::new();
    let mut hygiene_plan = Vec::new();
    let mut ide_plan = Vec::new();
    let mut manifest_path = None;
    let mut manifest_contents = None;
    let mut workspace_preview = None;

    if let Some(workspace) = workspace {
        let preview = prepare_workspace_init_preview(workspace, &request, &resolved)?;

        if request.docs_diff {
            return Ok(InitCommandReport::new(
                CommandExitStatus::Succeeded,
                render_docs_export_diff_report(request.docs_output_dir, &preview.docs_plan),
            ));
        }

        if !request.docs_refresh && !request.force && preview.has_docs_refresh_conflicts() {
            return Ok(InitCommandReport::new(
                CommandExitStatus::NonSuccess,
                render_docs_export_conflict_report(request.docs_output_dir, &preview.docs_plan),
            ));
        }

        workspace_preview = Some(preview);
    }

    let workspace_canon_selected = scope_includes_workspace(request.scope)
        && canon_selected_for_init(
            workspace_preview.as_ref().map(|preview| &preview.local_config),
            global_config.as_ref(),
        );
    let canon_init_assistant = workspace_canon_selected
        .then(|| {
            preferred_canon_init_assistant(
                &assistant_runtimes_for_hosts(&resolved.effective_assistants),
                &resolved.effective_routes,
            )
        })
        .flatten();

    let planned = build_init_planned_changes(InitPlannedChangesInput {
        scope: request.scope,
        requested_domain_template_count: resolved.requested_domain_templates.len(),
        workspace,
        workspace_preview: workspace_preview.as_ref(),
        global_status,
        global_config_path: global_config_path.as_deref(),
        global_env_template_status,
        global_env_template_path: global_env_template_path.as_deref(),
        workspace_canon_selected,
        canon_init_assistant,
    });

    let canon_bootstrap = canon_bootstrap_readiness(
        workspace_preview.as_ref().map(|preview| &preview.local_config),
        global_config.as_ref(),
    );
    if let Some(canon_bootstrap) = canon_bootstrap.as_ref()
        && !canon_bootstrap.ready
    {
        return Ok(render_init_canon_bootstrap_blocked_report(
            &request,
            workspace,
            resolved.template,
            canon_bootstrap,
            &planned,
        ));
    }

    let scaffold_updates_pending = global_status == Some(ScaffoldFileStatus::Update)
        || global_env_template_status == Some(ScaffoldFileStatus::Update)
        || workspace_preview.as_ref().is_some_and(WorkspaceInitPreview::scaffold_updates_pending);

    if scaffold_updates_pending && !request.force {
        return Ok(render_init_preview_only_report(
            &request,
            workspace,
            resolved.template,
            &planned,
        ));
    }

    if let Some(preview) = workspace_preview {
        let WorkspaceInitPreview {
            local_config: preview_local_config,
            local_config_path: preview_local_config_path,
            local_env_template_path: preview_local_env_template_path,
            local_env_template_contents: preview_local_env_template_contents,
            local_env_template_status: _,
            execution_path: preview_execution_path,
            execution_contents: preview_execution_contents,
            execution_status: _,
            config_status: _,
            assistant_assets: preview_assistant_assets,
            assistant_actions_preview: _,
            docs_plan: preview_docs_plan,
            hygiene_plan: preview_hygiene_plan,
            ide_plan: preview_ide_plan,
            manifest_path: preview_manifest_path,
            manifest_contents: preview_manifest_contents,
            manifest_status: _,
        } = preview;

        local_config = Some(preview_local_config);
        local_config_path = Some(preview_local_config_path);
        local_env_template_path = Some(preview_local_env_template_path);
        local_env_template_contents = Some(preview_local_env_template_contents);
        execution_path = Some(preview_execution_path);
        execution_contents = Some(preview_execution_contents);
        assistant_assets = preview_assistant_assets;
        docs_plan = preview_docs_plan;
        hygiene_plan = preview_hygiene_plan;
        ide_plan = preview_ide_plan;
        manifest_path = Some(preview_manifest_path);
        manifest_contents = Some(preview_manifest_contents);
    }

    let mut default_interactor: Box<dyn InitInteractor> = Box::new(DialoguerInitInteractor);
    let interactor: &mut dyn InitInteractor = match request.interactor.as_mut() {
        Some(i) => i.as_mut(),
        None => default_interactor.as_mut(),
    };
    if let Some(answers) = resolved.guided_answers.as_ref() {
        let summary = render_guided_summary(
            request.scope,
            resolved.template,
            resolved.effective_canon_mode_selection,
            &resolved.effective_assistants,
            &answers.routes,
            &resolved.catalog,
            &planned,
        );
        if !interactor.confirm(&summary, true)? {
            return Ok(InitCommandReport::new(
                CommandExitStatus::NonSuccess,
                render_cancelled_init_report(
                    request.scope,
                    workspace,
                    resolved.template,
                    resolved.effective_canon_mode_selection,
                    &resolved.effective_assistants,
                    &answers.routes,
                    &resolved.catalog,
                ),
            ));
        }
    }

    let InitApplyOutcome {
        project_doc_roots: apply_project_doc_roots,
        hygiene_actions,
        assistant_actions,
        docs_actions,
        ide_actions,
        canon_workspace_bootstrap,
    } = match apply_init_scaffold_changes(InitApplyInputs {
        scope: request.scope,
        template: resolved.template,
        interactive_terminal: resolved.interactive_terminal,
        workspace,
        planned_changes: &planned,
        workspace_canon_selected,
        canon_init_assistant,
        global_env_template_path: global_env_template_path.as_deref(),
        global_env_template_contents: global_env_template_contents.as_deref(),
        global_config: global_config.as_ref(),
        execution_path: execution_path.as_deref(),
        execution_contents: execution_contents.as_deref(),
        local_config: local_config.as_ref(),
        local_env_template_path: local_env_template_path.as_deref(),
        local_env_template_contents: local_env_template_contents.as_deref(),
        assistant_assets: &assistant_assets,
        docs_plan: &docs_plan,
        hygiene_plan: &hygiene_plan,
        ide_plan: &ide_plan,
        manifest_path: manifest_path.as_deref(),
        manifest_contents: manifest_contents.as_deref(),
    })? {
        InitApplyExit::Applied(outcome) => outcome,
        InitApplyExit::Report(report) => return Ok(report),
    };
    let project_doc_roots = apply_project_doc_roots;

    Ok(render_successful_init_report(InitSuccessReportInputs {
        scope: request.scope,
        export_docs: request.export_docs,
        docs_output_dir: request.docs_output_dir,
        resolved: &resolved,
        workspace,
        global_config_path: global_config_path.as_deref(),
        global_env_template_path: global_env_template_path.as_deref(),
        execution_path: execution_path.as_deref(),
        local_config_path: local_config_path.as_deref(),
        local_env_template_path: local_env_template_path.as_deref(),
        manifest_path: manifest_path.as_deref(),
        project_doc_roots: project_doc_roots.as_ref(),
        local_config: local_config.as_ref(),
        global_config: global_config.as_ref(),
        canon_bootstrap: canon_bootstrap.as_ref(),
        canon_workspace_bootstrap: canon_workspace_bootstrap.as_ref(),
        canon_init_assistant,
        assistant_actions: &assistant_actions,
        docs_actions: &docs_actions,
        ide_actions: &ide_actions,
        hygiene_actions: &hygiene_actions,
    }))
}

pub fn execute_update(request: UpdateRequest<'_>) -> Result<UpdateCommandReport, InitCommandError> {
    let workspace = resolve_workspace_root(request.workspace)?;
    let store = FileConfigStore::for_workspace(&workspace);
    let existing_local = store.load_local()?.ok_or_else(|| {
        InitCommandError::UpdateWorkspaceNotInitialized { workspace: workspace.clone() }
    })?;
    let existing_manifest = load_scaffold_manifest(&workspace)?;
    let catalog = BundledModelCatalog::load()?;
    let execution_path = workspace.join(BOUNDLINE_DIR_RELATIVE).join(EXECUTION_PROFILE_FILE_NAME);
    let effective_template = resolve_workspace_template(
        request.template,
        existing_manifest.as_ref(),
        &execution_path,
        existing_local.canon.as_ref(),
    )?;
    let selected_targets = resolve_update_targets(
        &workspace,
        &existing_local,
        existing_manifest.as_ref(),
        request.targets,
        request.ide,
        request.auto_approve,
        effective_template,
        request.template,
    )?;
    let ide_setup = if selected_targets.contains(&UpdateTarget::Ide) {
        resolve_ide_setup(request.ide, request.auto_approve, existing_manifest.as_ref())
    } else {
        existing_manifest.as_ref().map(|manifest| manifest.ide_setup.clone()).unwrap_or_default()
    };

    let mut desired_config = existing_local.clone();
    let assistants = configured_assistant_hosts(&desired_config);
    let routes = stored_routes(Some(&existing_local), None);
    let canon_mode_selection = desired_config.canon.as_ref().map(|canon| canon.mode_selection);
    let canon_risk = desired_config.canon.as_ref().and_then(|canon| canon.default_risk.clone());
    let canon_zone = desired_config.canon.as_ref().and_then(|canon| canon.default_zone.clone());
    let canon_owner = desired_config.canon.as_ref().and_then(|canon| canon.default_owner.clone());
    let seed_canon_reviewer_routes = desired_config.canon.is_some();
    apply_init_preferences(
        &mut desired_config,
        &catalog,
        &assistants,
        &routes,
        InitPreferenceOverrides {
            seed_canon_reviewer_routes,
            canon_mode_selection,
            risk: canon_risk.as_deref(),
            zone: canon_zone.as_deref(),
            owner: canon_owner.as_deref(),
        },
    );

    let config_path = store.local_config_path();
    let config_contents = toml::to_string_pretty(&desired_config).map_err(|source| {
        InitCommandError::SerializeConfigPreview { path: config_path.clone(), source }
    })?;
    let env_template_path = provider_workspace_env_template_path(&workspace);
    let env_template_contents = render_provider_env_template(ProviderEnvTemplateScope::Workspace);
    let execution_contents = effective_template
        .map(|template| render_execution_profile_contents(template, desired_config.canon.as_ref()))
        .transpose()?;
    let assistant_assets = if selected_targets.contains(&UpdateTarget::Assistant) {
        assets_for_assistants(&assistants)
    } else {
        Vec::new()
    };
    let docs_assets = if selected_targets.contains(&UpdateTarget::Docs) {
        docs_assets_for_assistants(&assistants)
    } else {
        Vec::new()
    };
    let active_domains = desired_config
        .routing
        .domain_templates
        .iter()
        .filter_map(
            |(family, settings)| {
                if settings.enabled.unwrap_or(false) { Some(*family) } else { None }
            },
        )
        .collect::<BTreeSet<_>>();

    let hygiene_plan = if selected_targets.contains(&UpdateTarget::Hygiene) {
        plan_workspace_hygiene_defaults(&workspace, &active_domains)?
    } else {
        Vec::new()
    };
    let ide_plan = if selected_targets.contains(&UpdateTarget::Ide) {
        plan_ide_setup(&workspace, &ide_setup)?
    } else {
        Vec::new()
    };
    let manifest_template = effective_template
        .or(existing_manifest.as_ref().and_then(|manifest| manifest.workspace_template));
    let desired_artifacts = collect_workspace_scaffold_artifacts(
        &workspace,
        selected_targets
            .contains(&UpdateTarget::Config)
            .then_some((&config_path, config_contents.as_str())),
        selected_targets
            .contains(&UpdateTarget::Config)
            .then_some((&env_template_path, env_template_contents.as_str())),
        if selected_targets.contains(&UpdateTarget::Execution) {
            execution_contents.as_deref().map(|contents| (execution_path.as_path(), contents))
        } else {
            None
        },
        &assistant_assets,
        &docs_assets,
        &hygiene_plan,
        &ide_plan,
    );
    let plan = build_update_plan(
        &workspace,
        existing_manifest.as_ref(),
        &desired_artifacts,
        manifest_template,
        &ide_setup,
        &selected_targets,
        request.adopt,
        request.prune,
    );
    let plan = plan?;
    let manifest_path = scaffold_manifest_path(&workspace);
    let manifest_contents =
        serialize_scaffold_manifest(&plan.manifest_after_apply, &manifest_path)?;
    let manifest_status = scaffold_file_status(&manifest_path, &manifest_contents)?;

    if request.status {
        return Ok(UpdateCommandReport::new(
            CommandExitStatus::Succeeded,
            render_update_status_report(
                &workspace,
                &selected_targets,
                existing_manifest.as_ref(),
                &plan,
                &manifest_path,
                manifest_status,
            ),
        ));
    }

    if request.diff {
        return Ok(UpdateCommandReport::new(
            CommandExitStatus::Succeeded,
            render_update_diff_report(
                &workspace,
                &selected_targets,
                &plan,
                &manifest_path,
                manifest_status,
            ),
        ));
    }

    if !request.apply {
        return Ok(UpdateCommandReport::new(
            CommandExitStatus::Succeeded,
            render_update_preview_report(
                &workspace,
                &selected_targets,
                &plan,
                &manifest_path,
                manifest_status,
            ),
        ));
    }

    if plan.requires_adopt() {
        return Ok(UpdateCommandReport::new(
            CommandExitStatus::NonSuccess,
            render_update_adopt_required_report(
                &workspace,
                &selected_targets,
                &plan,
                &manifest_path,
                manifest_status,
            ),
        ));
    }

    if plan.requires_force() && !request.force {
        return Ok(UpdateCommandReport::new(
            CommandExitStatus::NonSuccess,
            render_update_force_required_report(
                &workspace,
                &selected_targets,
                &plan,
                &manifest_path,
                manifest_status,
            ),
        ));
    }

    ensure_workspace_project_doc_roots(&workspace)?;
    apply_update_plan(&workspace, &plan, &desired_artifacts)?;
    write_scaffold_file(&manifest_path, &manifest_contents)?;

    Ok(UpdateCommandReport::new(
        CommandExitStatus::Succeeded,
        render_update_applied_report(
            &workspace,
            &selected_targets,
            &plan,
            &manifest_path,
            manifest_status,
        ),
    ))
}

#[allow(clippy::too_many_arguments)]
fn resolve_update_targets(
    workspace: &Path,
    config: &ConfigFile,
    existing_manifest: Option<&ScaffoldManifest>,
    requested: &[UpdateTarget],
    requested_ide: &[IdeKind],
    auto_approve: Option<TerminalAutoApproveProfile>,
    effective_template: Option<InitTemplate>,
    explicit_template: Option<InitTemplate>,
) -> Result<BTreeSet<UpdateTarget>, InitCommandError> {
    let mut targets = if requested.is_empty() {
        let mut defaults =
            BTreeSet::from([UpdateTarget::Config, UpdateTarget::Assistant, UpdateTarget::Hygiene]);
        if workspace_has_exported_docs(workspace, &configured_assistant_hosts(config))
            || existing_manifest_has_target(existing_manifest, ScaffoldTarget::Docs)
        {
            defaults.insert(UpdateTarget::Docs);
        }
        if existing_manifest_has_target(existing_manifest, ScaffoldTarget::Ide) {
            defaults.insert(UpdateTarget::Ide);
        }
        defaults
    } else {
        requested.iter().copied().collect::<BTreeSet<_>>()
    };

    if !requested_ide.is_empty() || auto_approve.is_some() {
        if requested.is_empty() {
            targets.insert(UpdateTarget::Ide);
        } else if !targets.contains(&UpdateTarget::Ide) {
            return Err(InitCommandError::UpdateIdeOptionsRequireIdeTarget);
        }
    }

    if explicit_template.is_some() {
        if requested.is_empty() {
            targets.insert(UpdateTarget::Execution);
        } else if !targets.contains(&UpdateTarget::Execution) {
            return Err(InitCommandError::UpdateTemplateRequiresExecutionTarget);
        }
    }

    if targets.contains(&UpdateTarget::Execution) && effective_template.is_none() {
        return Err(InitCommandError::UpdateExecutionTemplateRequired);
    }

    Ok(targets)
}

fn existing_manifest_has_target(
    existing_manifest: Option<&ScaffoldManifest>,
    target: ScaffoldTarget,
) -> bool {
    existing_manifest
        .is_some_and(|manifest| manifest.entries.iter().any(|entry| entry.target == target))
}

fn workspace_has_exported_docs(workspace: &Path, assistants: &[AssistantHostKind]) -> bool {
    docs_assets_for_assistants(assistants)
        .iter()
        .map(|asset| docs_target_path(workspace, &asset.relative_path))
        .any(|path| path.is_file())
}

fn apply_update_plan(
    workspace: &Path,
    plan: &UpdatePlan,
    desired_artifacts: &[RenderedManagedArtifact],
) -> Result<(), InitCommandError> {
    let desired_by_path = desired_artifacts
        .iter()
        .filter_map(|artifact| workspace_relative_path(&artifact.path).map(|path| (path, artifact)))
        .collect::<BTreeMap<_, _>>();

    for entry in &plan.entries {
        match entry.action {
            UpdatePlanAction::Create | UpdatePlanAction::Replace | UpdatePlanAction::Merge => {
                let artifact = desired_by_path.get(&entry.path).ok_or_else(|| {
                    InitCommandError::WorkspaceResolution(format!(
                        "missing desired artifact for update plan path {}",
                        entry.path
                    ))
                })?;
                write_scaffold_file(&workspace.join(&entry.path), &artifact.contents)?;
            }
            UpdatePlanAction::Remove => {
                let target = workspace.join(&entry.path);
                if target.is_dir() {
                    fs::remove_dir_all(&target)
                        .map_err(|source| InitCommandError::WriteFile { path: target, source })?;
                } else if target.is_file() {
                    fs::remove_file(&target)
                        .map_err(|source| InitCommandError::WriteFile { path: target, source })?;
                }
            }
            UpdatePlanAction::Adopt
            | UpdatePlanAction::AdoptCurrent
            | UpdatePlanAction::Orphaned
            | UpdatePlanAction::Unchanged
            | UpdatePlanAction::Conflict => {}
        }
    }

    Ok(())
}

fn render_execution_profile_contents(
    template: InitTemplate,
    canon: Option<&CanonPreferences>,
) -> Result<String, InitCommandError> {
    serde_json::to_string_pretty(&execution_template(template, canon))
        .map_err(|source| InitCommandError::InvalidExecutionProfile(source.to_string()))
}

fn resolve_workspace_template(
    explicit_template: Option<InitTemplate>,
    existing_manifest: Option<&ScaffoldManifest>,
    execution_path: &Path,
    canon: Option<&CanonPreferences>,
) -> Result<Option<InitTemplate>, InitCommandError> {
    if explicit_template.is_some() {
        return Ok(explicit_template);
    }

    if let Some(template) = existing_manifest.and_then(|manifest| manifest.workspace_template) {
        return Ok(Some(template));
    }

    if !execution_path.is_file() {
        return Ok(None);
    }

    let existing = fs::read_to_string(execution_path).map_err(|source| {
        InitCommandError::ReadFile { path: execution_path.to_path_buf(), source }
    })?;
    for template in [InitTemplate::BugFix, InitTemplate::Change, InitTemplate::Delivery] {
        if render_execution_profile_contents(template, canon)? == existing {
            return Ok(Some(template));
        }
    }

    Ok(None)
}

fn scaffold_manifest_path(workspace: &Path) -> PathBuf {
    workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME)
}

fn load_scaffold_manifest(workspace: &Path) -> Result<Option<ScaffoldManifest>, InitCommandError> {
    let path = scaffold_manifest_path(workspace);
    if !path.is_file() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)
        .map_err(|source| InitCommandError::ReadFile { path: path.clone(), source })?;
    Ok(serde_json::from_str(&contents).ok())
}

fn serialize_scaffold_manifest(
    manifest: &ScaffoldManifest,
    path: &Path,
) -> Result<String, InitCommandError> {
    serde_json::to_string_pretty(manifest).map_err(|source| {
        InitCommandError::SerializeScaffoldManifest { path: path.to_path_buf(), source }
    })
}

fn build_workspace_scaffold_manifest(
    existing_manifest: Option<&ScaffoldManifest>,
    workspace_template: Option<InitTemplate>,
    artifacts: &[RenderedManagedArtifact],
    ide_setup: &[IdeSetupSelection],
) -> ScaffoldManifest {
    let entries = artifacts.iter().filter_map(rendered_artifact_manifest_entry).collect::<Vec<_>>();
    scaffold_manifest_from_entries(existing_manifest, workspace_template, entries, ide_setup)
}

fn scaffold_manifest_from_entries(
    existing_manifest: Option<&ScaffoldManifest>,
    workspace_template: Option<InitTemplate>,
    entries: Vec<ScaffoldManifestEntry>,
    ide_setup: &[IdeSetupSelection],
) -> ScaffoldManifest {
    let now = current_timestamp_millis();
    let mut manifest =
        ScaffoldManifest::new(BOUNDLINE_VERSION, workspace_template, now, now, entries);
    manifest.ide_setup = ide_setup.to_vec();

    if let Some(existing_manifest) = existing_manifest {
        manifest.created_at_ms = existing_manifest.created_at_ms;
        if existing_manifest.tracks_same_state(&manifest) {
            manifest.updated_at_ms = existing_manifest.updated_at_ms;
        }
    }

    manifest
}

fn rendered_artifact_manifest_entry(
    artifact: &RenderedManagedArtifact,
) -> Option<ScaffoldManifestEntry> {
    workspace_relative_path(&artifact.path).map(|path| {
        ScaffoldManifestEntry::new(path, artifact.target, artifact.ownership, &artifact.contents)
    })
}

#[allow(clippy::too_many_arguments)]
fn collect_workspace_scaffold_artifacts(
    workspace: &Path,
    config_artifact: Option<(&Path, &str)>,
    env_template_artifact: Option<(&Path, &str)>,
    execution_artifact: Option<(&Path, &str)>,
    assistant_assets: &[AssistantAsset],
    docs_assets: &[DocsExportAsset],
    hygiene_plan: &[PlannedHygieneEntry],
    ide_plan: &[PlannedIdeEntry],
) -> Vec<RenderedManagedArtifact> {
    let mut artifacts = Vec::new();

    if let Some((config_path, config_contents)) = config_artifact
        && let Some(relative_path) = workspace_scaffold_relative_path(workspace, config_path)
    {
        artifacts.push(RenderedManagedArtifact {
            path: relative_path,
            target: ScaffoldTarget::Config,
            ownership: ScaffoldOwnershipMode::Replace,
            contents: config_contents.to_string(),
        });
    }

    if let Some((env_template_path, env_template_contents)) = env_template_artifact
        && let Some(relative_path) = workspace_scaffold_relative_path(workspace, env_template_path)
    {
        artifacts.push(RenderedManagedArtifact {
            path: relative_path,
            target: ScaffoldTarget::Config,
            ownership: ScaffoldOwnershipMode::Replace,
            contents: env_template_contents.to_string(),
        });
    }

    if let Some((execution_path, execution_contents)) = execution_artifact
        && let Some(relative_path) = workspace_scaffold_relative_path(workspace, execution_path)
    {
        artifacts.push(RenderedManagedArtifact {
            path: relative_path,
            target: ScaffoldTarget::Execution,
            ownership: ScaffoldOwnershipMode::Replace,
            contents: execution_contents.to_string(),
        });
    }

    artifacts.extend(assistant_assets.iter().map(|asset| RenderedManagedArtifact {
        path: PathBuf::from(asset.relative_path.as_ref()),
        target: ScaffoldTarget::Assistant,
        ownership: ScaffoldOwnershipMode::Replace,
        contents: materialize_assistant_asset_contents(workspace, asset),
    }));
    artifacts.extend(docs_assets.iter().filter_map(|asset| {
        workspace_scaffold_relative_path(workspace, Path::new(&asset.relative_path)).map(
            |relative_path| RenderedManagedArtifact {
                path: relative_path,
                target: ScaffoldTarget::Docs,
                ownership: ScaffoldOwnershipMode::Replace,
                contents: asset.contents.to_string(),
            },
        )
    }));
    artifacts.extend(hygiene_plan.iter().filter(|entry| entry.action.status != "skipped").map(
        |entry| RenderedManagedArtifact {
            path: PathBuf::from(&entry.action.path),
            target: ScaffoldTarget::Hygiene,
            ownership: ScaffoldOwnershipMode::Merge,
            contents: entry.final_content.clone(),
        },
    ));
    artifacts.extend(ide_plan.iter().map(|entry| entry.artifact.clone()));

    artifacts
}

fn workspace_scaffold_relative_path(workspace: &Path, path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        path.strip_prefix(workspace).ok().map(PathBuf::from)
    } else {
        Some(path.to_path_buf())
    }
}

fn workspace_relative_path(path: &Path) -> Option<String> {
    path.to_str().map(|value| value.replace('\\', "/"))
}

fn update_target_for_scaffold_target(target: ScaffoldTarget) -> UpdateTarget {
    match target {
        ScaffoldTarget::Config => UpdateTarget::Config,
        ScaffoldTarget::Execution => UpdateTarget::Execution,
        ScaffoldTarget::Assistant => UpdateTarget::Assistant,
        ScaffoldTarget::Docs => UpdateTarget::Docs,
        ScaffoldTarget::Hygiene => UpdateTarget::Hygiene,
        ScaffoldTarget::Ide => UpdateTarget::Ide,
    }
}

fn read_optional_file_contents(path: &Path) -> Result<Option<String>, InitCommandError> {
    if !path.is_file() {
        return Ok(None);
    }

    fs::read_to_string(path)
        .map(Some)
        .map_err(|source| InitCommandError::ReadFile { path: path.to_path_buf(), source })
}

#[allow(clippy::too_many_arguments)]
fn build_update_plan(
    workspace: &Path,
    existing_manifest: Option<&ScaffoldManifest>,
    desired_artifacts: &[RenderedManagedArtifact],
    workspace_template: Option<InitTemplate>,
    ide_setup: &[IdeSetupSelection],
    selected_targets: &BTreeSet<UpdateTarget>,
    adopt: bool,
    prune: bool,
) -> Result<UpdatePlan, InitCommandError> {
    let mut existing_entries = existing_manifest
        .map(|manifest| {
            manifest
                .entries
                .iter()
                .cloned()
                .map(|entry| (entry.path.clone(), entry))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut next_manifest_entries = BTreeMap::<String, ScaffoldManifestEntry>::new();
    let mut entries = Vec::new();

    for artifact in desired_artifacts {
        let path = workspace_relative_path(&artifact.path)
            .unwrap_or_else(|| artifact.path.display().to_string());
        let desired_entry = rendered_artifact_manifest_entry(artifact).unwrap_or_else(|| {
            ScaffoldManifestEntry::new(
                path.clone(),
                artifact.target,
                artifact.ownership,
                &artifact.contents,
            )
        });
        let manifest_entry = existing_entries.remove(&path);
        let current_path = workspace.join(&path);
        let current_contents = read_optional_file_contents(&current_path)?;
        let current_fingerprint =
            current_contents.as_deref().map(crate::domain::scaffold_manifest::fingerprint_text);
        let tracked = manifest_entry.is_some();

        let (action, detail, requires_force, requires_adopt, next_entry) = match (
            artifact.ownership,
            current_contents.as_deref(),
            manifest_entry.as_ref(),
        ) {
            (_, None, _) => (
                UpdatePlanAction::Create,
                "managed scaffold file is missing and will be created".to_string(),
                false,
                false,
                Some(desired_entry.clone()),
            ),
            (ScaffoldOwnershipMode::Merge, Some(current), _) => {
                if current == artifact.contents {
                    let detail = if tracked {
                        "merge-owned file already matches the desired managed content".to_string()
                    } else {
                        "merge-owned file already matches the desired managed content and will be tracked".to_string()
                    };
                    (
                        if tracked { UpdatePlanAction::Unchanged } else { UpdatePlanAction::Adopt },
                        detail,
                        false,
                        false,
                        Some(desired_entry.clone()),
                    )
                } else {
                    (
                        UpdatePlanAction::Merge,
                        "merge-owned file will add managed patterns while preserving custom lines"
                            .to_string(),
                        false,
                        false,
                        Some(desired_entry.clone()),
                    )
                }
            }
            (ScaffoldOwnershipMode::Replace, Some(current), Some(existing_entry)) => {
                if current == artifact.contents {
                    (
                        UpdatePlanAction::Unchanged,
                        "tracked file already matches the desired managed content".to_string(),
                        false,
                        false,
                        Some(desired_entry.clone()),
                    )
                } else if current_fingerprint.as_deref()
                    == Some(existing_entry.fingerprint.as_str())
                {
                    (
                        UpdatePlanAction::Replace,
                        "tracked file differs from the desired managed content".to_string(),
                        false,
                        false,
                        Some(desired_entry.clone()),
                    )
                } else {
                    (
                        UpdatePlanAction::Replace,
                        "tracked file has local drift and requires --force before replacement"
                            .to_string(),
                        true,
                        false,
                        Some(desired_entry.clone()),
                    )
                }
            }
            (ScaffoldOwnershipMode::Replace, Some(current), None) => {
                if current == artifact.contents {
                    (
                            UpdatePlanAction::Adopt,
                            "untracked file already matches the desired managed content and will be tracked"
                                .to_string(),
                            false,
                            false,
                            Some(desired_entry.clone()),
                        )
                } else if adopt {
                    let adopted_entry = ScaffoldManifestEntry::new(
                        path.clone(),
                        artifact.target,
                        artifact.ownership,
                        current,
                    );
                    (
                            UpdatePlanAction::AdoptCurrent,
                            "untracked file differs from the desired managed content and will be adopted as the current baseline"
                                .to_string(),
                            true,
                            false,
                            Some(adopted_entry),
                        )
                } else {
                    (
                            UpdatePlanAction::Conflict,
                            "untracked file differs from the desired managed content; rerun with --adopt --force to baseline it"
                                .to_string(),
                            false,
                            true,
                            None,
                        )
                }
            }
        };

        if let Some(next_entry) = next_entry {
            next_manifest_entries.insert(path.clone(), next_entry);
        }
        entries.push(UpdatePlanEntry {
            path,
            target: artifact.target,
            ownership: artifact.ownership,
            action,
            detail,
            tracked,
            requires_force,
            requires_adopt,
        });
    }

    for (path, entry) in existing_entries {
        if !selected_targets.contains(&update_target_for_scaffold_target(entry.target)) {
            next_manifest_entries.insert(path, entry);
            continue;
        }

        let current_path = workspace.join(&entry.path);
        if !current_path.is_file() {
            continue;
        }

        if prune {
            entries.push(UpdatePlanEntry {
                path: entry.path.clone(),
                target: entry.target,
                ownership: entry.ownership,
                action: UpdatePlanAction::Remove,
                detail: "tracked artifact is no longer desired and will be pruned".to_string(),
                tracked: true,
                requires_force: false,
                requires_adopt: false,
            });
        } else {
            next_manifest_entries.insert(path.clone(), entry.clone());
            entries.push(UpdatePlanEntry {
                path,
                target: entry.target,
                ownership: entry.ownership,
                action: UpdatePlanAction::Orphaned,
                detail: "tracked artifact is no longer desired; rerun with --prune to remove it"
                    .to_string(),
                tracked: true,
                requires_force: false,
                requires_adopt: false,
            });
        }
    }

    let manifest_after_apply = scaffold_manifest_from_entries(
        existing_manifest,
        workspace_template,
        next_manifest_entries.into_values().collect(),
        ide_setup,
    );

    entries.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(UpdatePlan { entries, manifest_after_apply, manifest_present: existing_manifest.is_some() })
}

fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn scope_includes_global(scope: InitConfigScope) -> bool {
    matches!(scope, InitConfigScope::Global | InitConfigScope::Both)
}

fn scope_includes_workspace(scope: InitConfigScope) -> bool {
    matches!(scope, InitConfigScope::Workspace | InitConfigScope::Both)
}

fn validate_init_scope_options(request: &InitRequest<'_>) -> Result<(), InitCommandError> {
    if request.scope != InitConfigScope::Global {
        return Ok(());
    }

    let mut invalid_arguments = Vec::new();
    if !request.domains.is_empty() {
        invalid_arguments.push("--domain");
    }
    if !request.domain_standards.is_empty() {
        invalid_arguments.push("--domain-standard");
    }
    if !request.context_bindings.is_empty() {
        invalid_arguments.push("--context-binding");
    }
    if !request.required_context_bindings.is_empty() {
        invalid_arguments.push("--required-context-binding");
    }
    if request.export_docs {
        invalid_arguments.push("--export-docs");
    }
    if request.docs_refresh {
        invalid_arguments.push("--refresh");
    }
    if request.docs_diff {
        invalid_arguments.push("--diff");
    }
    if request.docs_output_dir.is_some() {
        invalid_arguments.push("--to");
    }
    if !request.ide.is_empty() {
        invalid_arguments.push("--ide");
    }
    if request.auto_approve.is_some() {
        invalid_arguments.push("--auto-approve");
    }

    if invalid_arguments.is_empty() {
        Ok(())
    } else {
        Err(InitCommandError::InvalidScopeArgument(format!(
            "{} can only be used with --scope workspace or --scope both",
            invalid_arguments.join(", ")
        )))
    }
}

fn load_stored_init_defaults(
    scope: InitConfigScope,
    workspace: Option<&Path>,
) -> Result<StoredInitDefaults, InitCommandError> {
    let workspace_config = if scope_includes_workspace(scope) {
        workspace
            .map(|path| FileConfigStore::for_workspace(path).load_local())
            .transpose()?
            .flatten()
    } else {
        None
    };
    let global_config =
        if scope_includes_global(scope) { FileConfigStore::load_global()? } else { None };

    Ok(StoredInitDefaults {
        canon_mode_selection: workspace_config
            .as_ref()
            .and_then(stored_canon_mode_selection)
            .or_else(|| global_config.as_ref().and_then(stored_canon_mode_selection)),
        assistants: stored_assistants(workspace_config.as_ref(), global_config.as_ref()),
        routes: stored_routes(workspace_config.as_ref(), global_config.as_ref()),
    })
}

fn stored_canon_mode_selection(config: &ConfigFile) -> Option<CanonModeSelectionPreference> {
    config.canon.as_ref().map(|canon| canon.mode_selection)
}

fn stored_assistants(
    workspace_config: Option<&ConfigFile>,
    global_config: Option<&ConfigFile>,
) -> Vec<AssistantHostKind> {
    workspace_config
        .map(configured_assistant_hosts)
        .filter(|assistants| !assistants.is_empty())
        .or_else(|| {
            global_config
                .map(configured_assistant_hosts)
                .filter(|assistants| !assistants.is_empty())
        })
        .unwrap_or_default()
}

fn stored_routes(
    workspace_config: Option<&ConfigFile>,
    global_config: Option<&ConfigFile>,
) -> Vec<(RouteSlot, ModelRoute)> {
    required_init_route_slots()
        .into_iter()
        .filter_map(|slot| {
            workspace_config
                .and_then(|config| stored_route_for_slot(config, slot))
                .or_else(|| global_config.and_then(|config| stored_route_for_slot(config, slot)))
                .map(|route| (slot, route))
        })
        .collect()
}

fn stored_route_for_slot(config: &ConfigFile, slot: RouteSlot) -> Option<ModelRoute> {
    match slot {
        RouteSlot::Planning => config.routing.planning.clone(),
        RouteSlot::Implementation => config.routing.implementation.clone(),
        RouteSlot::Verification => config.routing.verification.clone(),
        RouteSlot::Review => config.routing.review.clone(),
    }
}

fn resolve_init_inputs(
    request: &mut InitRequest<'_>,
    workspace: Option<&Path>,
) -> Result<ResolvedInitInputs, InitCommandError> {
    let catalog = BundledModelCatalog::load()?;
    let interactive_terminal = request
        .interactive_terminal_override
        .unwrap_or_else(|| io::stdin().is_terminal() && io::stdout().is_terminal());
    let stored_defaults = load_stored_init_defaults(request.scope, workspace)?;
    let explicit_routes = request
        .routes
        .iter()
        .map(|raw_route| parse_model_route(raw_route))
        .collect::<Result<Vec<_>, _>>()?;
    let prompt_for_canon_mode =
        request.canon_mode_selection.or(stored_defaults.canon_mode_selection).is_none();
    let prompt_for_assistants = request.assistants.is_empty()
        && stored_defaults.assistants.is_empty()
        && explicit_routes.is_empty()
        && stored_defaults.routes.is_empty();
    let prompt_for_routes = explicit_routes.is_empty() && stored_defaults.routes.is_empty();
    let needs_guided_values = prompt_for_canon_mode || prompt_for_assistants || prompt_for_routes;

    if !request.non_interactive && needs_guided_values && !interactive_terminal {
        return Err(InitCommandError::InteractiveTerminalUnavailable);
    }

    let mut default_interactor: Box<dyn InitInteractor> = Box::new(DialoguerInitInteractor);
    let interactor: &mut dyn InitInteractor = match request.interactor.as_mut() {
        Some(i) => i.as_mut(),
        None => default_interactor.as_mut(),
    };

    let guided_answers = if !request.non_interactive && interactive_terminal && needs_guided_values
    {
        Some(collect_guided_init_answers_with_interactor(
            interactor,
            prompt_for_canon_mode,
            prompt_for_assistants,
            prompt_for_routes,
            &catalog,
            if request.assistants.is_empty() {
                stored_defaults.assistants.as_slice()
            } else {
                request.assistants
            },
        )?)
    } else {
        None
    };

    let effective_canon_mode_selection = request
        .canon_mode_selection
        .or(stored_defaults.canon_mode_selection)
        .or_else(|| guided_answers.as_ref().and_then(|answers| answers.canon_mode_selection));
    let effective_assistants = if request.assistants.is_empty() {
        guided_answers
            .as_ref()
            .map(|answers| answers.assistants.clone())
            .unwrap_or_else(|| stored_defaults.assistants.clone())
    } else {
        request.assistants.to_vec()
    };
    let effective_assistant_runtimes = assistant_runtimes_for_hosts(&effective_assistants);

    let guided_routes = if explicit_routes.is_empty()
        && prompt_for_routes
        && let Some(answers) = guided_answers.as_ref()
    {
        answers
            .routes
            .iter()
            .filter_map(|selection| selection.route.clone().map(|route| (selection.slot, route)))
            .collect::<Vec<_>>()
    } else {
        stored_defaults.routes.clone()
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
        resolve_seeded_routes(&effective_assistant_runtimes, &explicit_slots, runtime_available)?;
    effective_routes
        .extend(seeded_routes.iter().map(|selection| (selection.slot, selection.route.clone())));

    let requested_domain_templates = if let Some(workspace) = workspace {
        requested_domain_templates(
            workspace,
            request.domains,
            request.domain_standards,
            request.context_bindings,
            request.required_context_bindings,
        )?
    } else {
        BTreeMap::new()
    };

    Ok(ResolvedInitInputs {
        catalog,
        template: request.template.unwrap_or(InitTemplate::BugFix),
        interactive_terminal,
        guided_answers,
        effective_canon_mode_selection,
        effective_assistants,
        explicit_routes,
        guided_routes,
        seeded_routes,
        effective_routes,
        requested_domain_templates,
    })
}

#[derive(Clone, Copy)]
struct InitPreferenceOverrides<'a> {
    seed_canon_reviewer_routes: bool,
    canon_mode_selection: Option<CanonModeSelectionPreference>,
    risk: Option<&'a str>,
    zone: Option<&'a str>,
    owner: Option<&'a str>,
}

fn apply_init_preferences(
    config: &mut ConfigFile,
    catalog: &BundledModelCatalog,
    assistants: &[AssistantHostKind],
    routes: &[(RouteSlot, ModelRoute)],
    overrides: InitPreferenceOverrides<'_>,
) {
    let assistant_runtimes = assistant_runtimes_for_hosts(assistants);
    config.routing.assistant_hosts = assistants.to_vec();
    config.routing.assistant_runtimes = assistant_runtimes.clone();
    for (slot, route) in routes {
        config.routing.set_slot(*slot, route.clone());
    }

    if overrides.canon_mode_selection.is_some()
        || overrides.risk.is_some()
        || overrides.zone.is_some()
        || overrides.owner.is_some()
    {
        let mut canon = config.canon.clone().unwrap_or(CanonPreferences {
            mode_selection: overrides.canon_mode_selection.unwrap_or_default(),
            default_risk: None,
            default_zone: None,
            default_owner: None,
            default_system_context: None,
        });
        if let Some(mode_selection) = overrides.canon_mode_selection {
            canon.mode_selection = mode_selection;
        }
        if let Some(risk) = overrides.risk.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_risk = Some(risk.to_string());
        }
        if let Some(zone) = overrides.zone.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_zone = Some(zone.to_string());
        }
        if let Some(owner) = overrides.owner.map(str::trim).filter(|value| !value.is_empty()) {
            canon.default_owner = Some(owner.to_string());
        }
        apply_missing_canon_defaults(&mut canon);
        config.canon = Some(canon);
    }

    if overrides.seed_canon_reviewer_routes {
        seed_missing_canon_reviewer_routes(config, catalog, &assistant_runtimes, routes);
    }
}

fn seed_missing_canon_reviewer_routes(
    config: &mut ConfigFile,
    catalog: &BundledModelCatalog,
    assistants: &[RuntimeKind],
    routes: &[(RouteSlot, ModelRoute)],
) {
    let routes_by_slot = routes.iter().cloned().collect::<BTreeMap<_, _>>();
    let existing_safety = config.routing.reviewer_roles.get(CANON_SAFETY_REVIEWER_ROLE_ID).cloned();
    let existing_maintainability =
        config.routing.reviewer_roles.get(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID).cloned();

    let safety = existing_safety.clone().or_else(|| {
        select_canon_reviewer_route(
            catalog,
            assistants,
            &routes_by_slot,
            &CANON_SAFETY_REVIEWER_SLOT_ORDER,
            existing_maintainability.as_ref(),
        )
    });
    if let Some(route) = safety.clone() {
        config
            .routing
            .reviewer_roles
            .entry(CANON_SAFETY_REVIEWER_ROLE_ID.to_string())
            .or_insert(route);
    }

    let maintainability = existing_maintainability.clone().or_else(|| {
        select_canon_reviewer_route(
            catalog,
            assistants,
            &routes_by_slot,
            &CANON_MAINTAINABILITY_REVIEWER_SLOT_ORDER,
            safety.as_ref(),
        )
    });
    if let Some(route) = maintainability {
        config
            .routing
            .reviewer_roles
            .entry(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID.to_string())
            .or_insert(route);
    }
}

fn select_canon_reviewer_route(
    catalog: &BundledModelCatalog,
    assistants: &[RuntimeKind],
    routes_by_slot: &BTreeMap<RouteSlot, ModelRoute>,
    preferred_slots: &[RouteSlot],
    conflicting_route: Option<&ModelRoute>,
) -> Option<ModelRoute> {
    canon_reviewer_route_candidates(catalog, assistants, routes_by_slot, preferred_slots)
        .into_iter()
        .find(|candidate| conflicting_route.is_none_or(|existing| candidate != existing))
}

fn canon_reviewer_route_candidates(
    catalog: &BundledModelCatalog,
    assistants: &[RuntimeKind],
    routes_by_slot: &BTreeMap<RouteSlot, ModelRoute>,
    preferred_slots: &[RouteSlot],
) -> Vec<ModelRoute> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for slot in preferred_slots {
        if let Some(route) = routes_by_slot.get(slot) {
            push_canon_reviewer_route_candidate(&mut candidates, &mut seen, route.clone());
        }
    }

    for runtime in assistants.iter().copied().filter(|runtime| runtime_available(*runtime)) {
        for route in catalog.model_routes_for_runtime(runtime) {
            push_canon_reviewer_route_candidate(&mut candidates, &mut seen, route);
        }
    }

    for slot in preferred_slots {
        if let Some(route) = catalog.default_route_for_slot(*slot) {
            push_canon_reviewer_route_candidate(&mut candidates, &mut seen, route);
        }
    }

    candidates
}

fn push_canon_reviewer_route_candidate(
    candidates: &mut Vec<ModelRoute>,
    seen: &mut BTreeSet<String>,
    route: ModelRoute,
) {
    let route_key = model_route_label(&route);
    if seen.insert(route_key) {
        candidates.push(route);
    }
}

fn model_route_label(route: &ModelRoute) -> String {
    format!("{}:{}", route.runtime, route.model)
}

fn apply_missing_canon_defaults(canon: &mut CanonPreferences) {
    if canon.default_risk.as_deref().map(str::trim).filter(|value| !value.is_empty()).is_none() {
        canon.default_risk = Some(DEFAULT_CANON_RISK.to_string());
    }
    if canon.default_zone.as_deref().map(str::trim).filter(|value| !value.is_empty()).is_none() {
        canon.default_zone = Some(DEFAULT_CANON_ZONE.to_string());
    }
    if canon.default_owner.as_deref().map(str::trim).filter(|value| !value.is_empty()).is_none() {
        canon.default_owner = Some(DEFAULT_CANON_OWNER.to_string());
    }
    if canon
        .default_system_context
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        canon.default_system_context = Some(DEFAULT_CANON_SYSTEM_CONTEXT.to_string());
    }
}

fn canon_preference_or_default<'a>(value: Option<&'a str>, fallback: &'static str) -> &'a str {
    value.map(str::trim).filter(|entry| !entry.is_empty()).unwrap_or(fallback)
}

/// Early pre-flight check for the Canon surface. Runs immediately after guided
/// prompts complete (inside `resolve_init_inputs`) but before any config loading
/// or asset computation. Returns a blocking report if Canon is selected but the
/// binary or surface is not ready; returns `None` when Canon is not selected or
/// the surface is healthy.
fn canon_surface_preflight(
    request: &InitRequest<'_>,
    resolved: &ResolvedInitInputs,
    workspace: Option<&Path>,
) -> Result<Option<InitCommandReport>, InitCommandError> {
    if resolved.effective_canon_mode_selection.is_none() {
        return Ok(None);
    }

    let current_exe = match std::env::current_exe() {
        Ok(current_exe) => current_exe,
        Err(error) => {
            let detail = format!(
                "Boundline could not determine the current executable before checking Canon: {error}"
            );
            return Ok(Some(render_canon_surface_preflight_failure(
                request,
                workspace,
                resolved,
                "blocked",
                &detail,
                std::slice::from_ref(&detail),
            )));
        }
    };

    let status = evaluate_init_canon_install(&current_exe);
    let surface_ready = status.surface_verification.as_ref().is_some_and(|surface| surface.ready);
    if surface_ready {
        return Ok(None);
    }

    let repair_actions = if let Some(surface) = status.surface_verification.as_ref() {
        if surface.repair_actions.is_empty() {
            status.suggested_actions.clone()
        } else {
            surface.repair_actions.clone()
        }
    } else if status.suggested_actions.is_empty() {
        vec![status.message.clone()]
    } else {
        status.suggested_actions.clone()
    };

    Ok(Some(render_canon_surface_preflight_failure(
        request,
        workspace,
        resolved,
        "blocked",
        &status.message,
        &repair_actions,
    )))
}

fn render_canon_surface_preflight_failure(
    request: &InitRequest<'_>,
    workspace: Option<&Path>,
    resolved: &ResolvedInitInputs,
    state: &str,
    detail: &str,
    repair_actions: &[String],
) -> InitCommandReport {
    let mut lines = vec![
        "init: blocked - Canon surface not ready".to_string(),
        format!("scope: {}", request.scope),
    ];
    if scope_includes_workspace(request.scope) {
        lines.push(format!("template: {}", template_label(resolved.template)));
    }
    lines.push(format!("canon_bootstrap: {state}"));
    lines.push(format!("canon_surface: {detail}"));
    lines.push("repair_actions:".to_string());
    if repair_actions.is_empty() {
        lines.push("- verify the Canon installation and rerun init".to_string());
    } else {
        lines.extend(repair_actions.iter().map(|action| format!("- {action}")));
    }
    lines.push("planned_changes:".to_string());
    lines.push("- none (blocked before planning)".to_string());
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

fn canon_bootstrap_readiness(
    local_config: Option<&ConfigFile>,
    global_config: Option<&ConfigFile>,
) -> Option<CanonBootstrapReadiness> {
    if !canon_selected_for_init(local_config, global_config) {
        return None;
    }

    let current_exe = match std::env::current_exe() {
        Ok(current_exe) => current_exe,
        Err(error) => {
            let repair = format!(
                "Boundline could not determine the current executable before checking Canon: {error}"
            );
            return Some(CanonBootstrapReadiness {
                ready: false,
                state: "blocked",
                detail: repair.clone(),
                repair_actions: vec![repair],
            });
        }
    };

    let status = evaluate_init_canon_install(&current_exe);
    let surface_ready = status.surface_verification.as_ref().is_some_and(|surface| surface.ready);
    if !surface_ready {
        let repair_actions = if let Some(surface) = status.surface_verification.as_ref() {
            if surface.repair_actions.is_empty() {
                status.suggested_actions.clone()
            } else {
                surface.repair_actions.clone()
            }
        } else if status.suggested_actions.is_empty() {
            vec![status.message.clone()]
        } else {
            status.suggested_actions.clone()
        };

        return Some(CanonBootstrapReadiness {
            ready: false,
            state: "blocked",
            detail: status.message,
            repair_actions,
        });
    }

    let reviewer_routes = canon_reviewer_route_readiness(local_config, global_config);
    if !reviewer_routes.ready {
        return Some(CanonBootstrapReadiness {
            ready: false,
            state: "blocked",
            detail: reviewer_routes.detail,
            repair_actions: reviewer_routes.repair_actions,
        });
    }

    Some(CanonBootstrapReadiness {
        ready: true,
        state: "ready",
        detail: status.message,
        repair_actions: Vec::new(),
    })
}

fn canon_reviewer_route_readiness(
    local_config: Option<&ConfigFile>,
    global_config: Option<&ConfigFile>,
) -> CanonReviewerRouteReadiness {
    let routing = resolve_effective_routing(
        &RoutingOverrides::default(),
        local_config.map(|config| &config.routing),
        None,
        global_config.map(|config| &config.routing),
    );
    let safety = routing.reviewer_roles.get(CANON_SAFETY_REVIEWER_ROLE_ID);
    let maintainability = routing.reviewer_roles.get(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID);

    let mut missing_roles = Vec::new();
    if safety.is_none() {
        missing_roles.push(CANON_SAFETY_REVIEWER_ROLE_ID);
    }
    if maintainability.is_none() {
        missing_roles.push(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID);
    }

    if !missing_roles.is_empty() {
        let repair_actions = missing_roles
            .iter()
            .map(|role_id| format!("set routing.reviewer_roles.{role_id} to a runtime:model route"))
            .chain(std::iter::once(CANON_REVIEWER_ROUTE_REPAIR_ACTION.to_string()))
            .collect();
        return CanonReviewerRouteReadiness {
            ready: false,
            detail: format!(
                "missing mandatory Canon reviewer routes: {}",
                missing_roles.join(", ")
            ),
            repair_actions,
        };
    }

    let Some(safety) = safety else {
        return CanonReviewerRouteReadiness {
            ready: false,
            detail: "safety reviewer route unexpectedly missing".to_string(),
            repair_actions: vec![CANON_REVIEWER_ROUTE_REPAIR_ACTION.to_string()],
        };
    };
    let Some(maintainability) = maintainability else {
        return CanonReviewerRouteReadiness {
            ready: false,
            detail: "maintainability reviewer route unexpectedly missing".to_string(),
            repair_actions: vec![CANON_REVIEWER_ROUTE_REPAIR_ACTION.to_string()],
        };
    };
    let safety_route = &safety.route;
    let maintainability_route = &maintainability.route;
    if safety_route == maintainability_route {
        return CanonReviewerRouteReadiness {
            ready: false,
            detail: format!(
                "Canon reviewer routes collapse onto {} for safety and maintainability",
                model_route_label(safety_route)
            ),
            repair_actions: vec![
                CANON_REVIEWER_ROUTE_REPAIR_ACTION.to_string(),
                format!(
                    "change either routing.reviewer_roles.{} or routing.reviewer_roles.{} to a different runtime:model route",
                    CANON_SAFETY_REVIEWER_ROLE_ID, CANON_MAINTAINABILITY_REVIEWER_ROLE_ID
                ),
            ],
        };
    }

    CanonReviewerRouteReadiness {
        ready: true,
        detail: "distinct Canon reviewer routes configured".to_string(),
        repair_actions: Vec::new(),
    }
}

fn canon_selected_for_init(
    local_config: Option<&ConfigFile>,
    global_config: Option<&ConfigFile>,
) -> bool {
    local_config.and_then(|config| config.canon.as_ref()).is_some()
        || global_config.and_then(|config| config.canon.as_ref()).is_some()
}

fn preferred_canon_init_assistant(
    assistants: &[RuntimeKind],
    routes: &[(RouteSlot, ModelRoute)],
) -> Option<CanonInitAssistantHost> {
    assistants.iter().copied().find_map(CanonInitAssistantHost::from_runtime).or_else(|| {
        routes.iter().find_map(|(_, route)| CanonInitAssistantHost::from_runtime(route.runtime))
    })
}

fn canon_workspace_planned_changes(
    workspace: &Path,
    assistant: Option<CanonInitAssistantHost>,
) -> Vec<String> {
    let mut planned = Vec::new();
    let canon_root = workspace.join(CANON_WORKSPACE_ROOT_RELATIVE);
    if !canon_root.is_dir() {
        planned.push(format!("- create {}", canon_root.display()));
    }

    if assistant.is_some() {
        let skills_root = workspace.join(CANON_AGENT_SKILLS_RELATIVE);
        if !skills_root.is_dir() {
            planned.push(format!("- create {} (Canon AI scaffolding)", skills_root.display()));
        }
    }

    planned
}

fn materialize_canon_workspace(
    workspace: &Path,
    assistant: Option<CanonInitAssistantHost>,
) -> Result<CanonWorkspaceBootstrapReport, String> {
    let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());

    if let Some(git_root) = nearest_git_root(&workspace)
        && git_root != workspace
    {
        return Err(format!(
            "Canon init would target git root {} instead of the requested workspace {}; use the repository root as the Boundline workspace or initialize a dedicated nested repository first",
            git_root.display(),
            workspace.display()
        ));
    }

    #[cfg(test)]
    {
        materialize_test_canon_workspace(&workspace, assistant)
    }

    #[cfg(not(test))]
    {
        let current_exe = std::env::current_exe().map_err(|error| {
            format!(
                "Boundline could not resolve the current executable before running Canon init: {error}"
            )
        })?;
        let status = evaluate_init_canon_install(&current_exe);
        let canon_path = status.location.ok_or_else(|| {
            "Boundline could not resolve the authoritative Canon binary before running `canon init`"
                .to_string()
        })?;

        let mut command = Command::new(&canon_path);
        command.current_dir(&workspace).arg(CANON_INIT_SUBCOMMAND);
        if let Some(assistant) = assistant {
            command.arg(CANON_AI_FLAG).arg(assistant.as_str());
        }
        command.arg(CANON_OUTPUT_FLAG).arg(CANON_OUTPUT_JSON);

        let output = command.output().map_err(|error| {
            format!(
                "failed to execute `{}` in {}: {error}",
                canon_path.display(),
                workspace.display()
            )
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
            let summary = if detail.is_empty() {
                format!(
                    "`{} {}` exited with code {}",
                    canon_path.display(),
                    CANON_INIT_SUBCOMMAND,
                    output.status.code().unwrap_or(-1)
                )
            } else {
                detail.to_string()
            };
            return Err(format!(
                "Canon workspace bootstrap failed in {}: {summary}",
                workspace.display()
            ));
        }

        serde_json::from_slice::<CanonWorkspaceBootstrapReport>(&output.stdout).map_err(|source| {
            format!("failed to parse Canon init output from {}: {source}", canon_path.display())
        })
    }
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
fn materialize_test_canon_workspace(
    workspace: &Path,
    assistant: Option<CanonInitAssistantHost>,
) -> Result<CanonWorkspaceBootstrapReport, String> {
    let canon_root = workspace.join(CANON_WORKSPACE_ROOT_RELATIVE);
    let canon_preexisting = canon_root.is_dir();
    fs::create_dir_all(&canon_root).map_err(|error| {
        format!("failed to create Canon workspace root at {}: {error}", canon_root.display())
    })?;

    let skills_root = workspace.join(CANON_AGENT_SKILLS_RELATIVE);
    let skills_preexisting = skills_root.is_dir();
    let skills_materialized = if assistant.is_some() && !skills_preexisting {
        fs::create_dir_all(&skills_root).map_err(|error| {
            format!(
                "failed to create Canon assistant scaffolding at {}: {error}",
                skills_root.display()
            )
        })?;
        36
    } else {
        0
    };

    Ok(CanonWorkspaceBootstrapReport {
        repo_root: workspace.to_path_buf(),
        canon_root,
        methods_materialized: if canon_preexisting { 0 } else { 16 },
        policies_materialized: if canon_preexisting { 0 } else { 5 },
        skills_materialized,
        claude_md_created: matches!(assistant, Some(CanonInitAssistantHost::Claude)),
    })
}

fn evaluate_init_canon_install(executable_path: &Path) -> CanonInstallStatus {
    #[cfg(test)]
    {
        let _ = executable_path;
        let overrides = CANON_INSTALL_STATUS_OVERRIDE.get_or_init(|| std::sync::Mutex::new(None));
        let overridden = match overrides.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        };
        overridden.unwrap_or_else(default_test_canon_install_status)
    }

    #[cfg(not(test))]
    {
        evaluate_canon_install(executable_path)
    }
}

#[cfg(test)]
fn replace_test_canon_install_status_override(
    status: Option<CanonInstallStatus>,
) -> Option<CanonInstallStatus> {
    let overrides = CANON_INSTALL_STATUS_OVERRIDE.get_or_init(|| std::sync::Mutex::new(None));
    match overrides.lock() {
        Ok(mut guard) => std::mem::replace(&mut *guard, status),
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            std::mem::replace(&mut *guard, status)
        }
    }
}

#[cfg(test)]
fn default_test_canon_install_status() -> CanonInstallStatus {
    use crate::domain::distribution::{
        CanonSurfaceVerification, CompanionState, SUPPORTED_CANON_VERSION,
    };

    let canon_path = std::env::temp_dir().join("boundline-test-canon");
    CanonInstallStatus {
        state: CompanionState::AlreadySatisfied,
        version: Some(SUPPORTED_CANON_VERSION.to_string()),
        location: Some(canon_path.clone()),
        bundled_with_boundline: false,
        message: format!(
            "Canon {SUPPORTED_CANON_VERSION} is already available on PATH at {} with verified governance surface",
            canon_path.display()
        ),
        suggested_actions: Vec::new(),
        surface_verification: Some(CanonSurfaceVerification {
            canon_path,
            version_compatible: true,
            operations_verified: true,
            missing_operations: Vec::new(),
            modes_verified: true,
            missing_modes: Vec::new(),
            unsupported_modes: Vec::new(),
            capability_snapshot: None,
            ready: true,
            repair_actions: Vec::new(),
        }),
    }
}

fn route_setup_lines(
    catalog: &BundledModelCatalog,
    effective_assistants: &[AssistantHostKind],
    guided_answers: Option<&GuidedInitAnswers>,
    explicit_routes: &[(RouteSlot, ModelRoute)],
    guided_routes: &[(RouteSlot, ModelRoute)],
    seeded_routes: &[SeededRouteSelection],
    inspect_command: String,
) -> Vec<String> {
    let mut lines = vec![format!("- catalog_source: {}", catalog.summary_label())];
    if effective_assistants.is_empty() {
        lines.push(
            "- assistant_defaults: none selected; no assistant-seeded routes were recorded"
                .to_string(),
        );
    } else {
        lines.push(format!(
            "- assistant_defaults: {}",
            format_assistant_host_list(effective_assistants)
        ));
    }

    if let Some(answers) = guided_answers {
        lines.extend(
            answers.routes.iter().map(|selection| format!("- {}", selection.display_line().trim())),
        );
    } else {
        let explicit_route_lines =
            explicit_routes.iter().chain(guided_routes.iter()).collect::<Vec<_>>();
        if seeded_routes.is_empty() && explicit_route_lines.is_empty() {
            lines.push(
                "- routes: none recorded; add --assistant or --route later to pin defaults"
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
    lines
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
struct IdeInitAction {
    ide: IdeKind,
    setup_kind: &'static str,
    status: &'static str,
    path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AssistantInitAction {
    surface: AssistantSurface,
    status: &'static str,
    created_files: usize,
    updated_files: usize,
    unchanged_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScaffoldFileStatus {
    Create,
    Update,
    Unchanged,
}

impl ScaffoldFileStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocsInitAction {
    surface: DocsExportSurface,
    status: &'static str,
    created_files: usize,
    updated_files: usize,
    unchanged_files: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocsExportFileStatus {
    Create,
    Update,
    Unchanged,
}

impl DocsExportFileStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocsExportPlanEntry {
    surface: DocsExportSurface,
    path: String,
    status: DocsExportFileStatus,
    contents: &'static str,
}

fn plan_assistant_setup(assistant_actions: &[AssistantInitAction]) -> Vec<String> {
    assistant_actions
        .iter()
        .filter(|action| action.created_files > 0 || action.updated_files > 0)
        .map(|action| {
            let verb = if action.updated_files > 0 { "refresh" } else { "scaffold" };
            let changed_files = action.created_files + action.updated_files;
            format!("- {verb} {} ({} file(s))", action.surface.plan_label(), changed_files)
        })
        .collect()
}

fn summarize_assistant_assets(
    workspace: &Path,
    assistant_assets: &[AssistantAsset],
) -> Result<Vec<AssistantInitAction>, InitCommandError> {
    let mut grouped = BTreeMap::<AssistantSurface, AssistantInitAction>::new();

    for asset in assistant_assets {
        let target = workspace.join(asset.relative_path.as_ref());
        let rendered_contents = materialize_assistant_asset_contents(workspace, asset);
        let file_status = scaffold_file_status(&target, &rendered_contents)?;
        let entry = grouped.entry(asset.surface).or_insert(AssistantInitAction {
            surface: asset.surface,
            status: "unchanged",
            created_files: 0,
            updated_files: 0,
            unchanged_files: 0,
        });
        match file_status {
            ScaffoldFileStatus::Create => entry.created_files += 1,
            ScaffoldFileStatus::Update => entry.updated_files += 1,
            ScaffoldFileStatus::Unchanged => entry.unchanged_files += 1,
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

fn apply_assistant_assets(
    workspace: &Path,
    assistant_assets: &[AssistantAsset],
) -> Result<Vec<AssistantInitAction>, InitCommandError> {
    let mut grouped = BTreeMap::<AssistantSurface, AssistantInitAction>::new();

    for asset in assistant_assets {
        let target = workspace.join(asset.relative_path.as_ref());
        let rendered_contents = materialize_assistant_asset_contents(workspace, asset);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| InitCommandError::WriteFile {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let file_status = if target.is_file() {
            let existing = fs::read_to_string(&target)
                .map_err(|source| InitCommandError::ReadFile { path: target.clone(), source })?;
            if existing == rendered_contents {
                "unchanged"
            } else {
                fs::write(&target, &rendered_contents).map_err(|source| {
                    InitCommandError::WriteFile { path: target.clone(), source }
                })?;
                "updated"
            }
        } else {
            fs::write(&target, &rendered_contents)
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

fn materialize_assistant_asset_contents(workspace: &Path, asset: &AssistantAsset) -> String {
    if !asset.relative_path.as_ref().starts_with(".github/prompts/") {
        return asset.contents.to_string();
    }

    let guidance_path = workspace.join("assistant/README.md");
    let rewritten_guidance = format!("Shared guidance: `{}`", guidance_path.display());
    asset.contents.replace("Shared guidance: `assistant/README.md`", &rewritten_guidance)
}

fn scaffold_file_status(
    path: &Path,
    expected_contents: &str,
) -> Result<ScaffoldFileStatus, InitCommandError> {
    if !path.is_file() {
        return Ok(ScaffoldFileStatus::Create);
    }

    let existing = fs::read_to_string(path)
        .map_err(|source| InitCommandError::ReadFile { path: path.to_path_buf(), source })?;
    if existing == expected_contents {
        Ok(ScaffoldFileStatus::Unchanged)
    } else {
        Ok(ScaffoldFileStatus::Update)
    }
}

fn write_scaffold_file(path: &Path, contents: &str) -> Result<(), InitCommandError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| InitCommandError::WriteFile { path: parent.to_path_buf(), source })?;
    }

    fs::write(path, contents)
        .map_err(|source| InitCommandError::WriteFile { path: path.to_path_buf(), source })
}

fn ensure_workspace_project_doc_roots(
    workspace: &Path,
) -> Result<crate::domain::project_index::ProjectDocRoots, InitCommandError> {
    let doc_roots = resolve_project_doc_roots(workspace).unwrap_or_default();
    for root in [doc_roots.project_memory_dir(workspace), doc_roots.evidence_dir(workspace)] {
        fs::create_dir_all(&root)
            .map_err(|source| InitCommandError::WriteFile { path: root.clone(), source })?;
    }

    write_scaffold_file_if_missing(
        &doc_roots.project_memory_dir(workspace).join("README.md"),
        PROJECT_MEMORY_ROOT_README,
    )?;
    write_scaffold_file_if_missing(
        &doc_roots.evidence_dir(workspace).join("README.md"),
        EVIDENCE_ROOT_README,
    )?;

    Ok(doc_roots)
}

fn write_scaffold_file_if_missing(path: &Path, contents: &str) -> Result<(), InitCommandError> {
    if path.exists() {
        return Ok(());
    }

    write_scaffold_file(path, contents)
}

fn resolve_workspace_root(workspace: &Path) -> Result<PathBuf, InitCommandError> {
    if workspace.is_absolute() {
        return Ok(workspace.to_path_buf());
    }

    let resolve_from = |root: &Path| -> Result<PathBuf, InitCommandError> {
        if workspace == Path::new(".") {
            return Ok(root.canonicalize().unwrap_or_else(|_| root.to_path_buf()));
        }

        Ok(join_workspace_root(root, workspace))
    };

    match std::env::current_dir() {
        Ok(current_dir) => resolve_from(&current_dir),
        Err(source) => {
            if let Some(pwd) = std::env::var_os("PWD") {
                let pwd = PathBuf::from(pwd);
                if pwd.is_absolute() && pwd.is_dir() {
                    return resolve_from(&pwd);
                }
            }

            Err(InitCommandError::CurrentDirectoryUnavailable {
                workspace: workspace.to_path_buf(),
                source,
            })
        }
    }
}

fn join_workspace_root(root: &Path, workspace: &Path) -> PathBuf {
    if workspace == Path::new(".") { root.to_path_buf() } else { root.join(workspace) }
}

fn docs_target_path(workspace: &Path, export_path: &str) -> PathBuf {
    let export_path = Path::new(export_path);
    if export_path.is_absolute() { export_path.to_path_buf() } else { workspace.join(export_path) }
}

fn plan_docs_export(
    workspace: &Path,
    docs_assets: &[DocsExportAsset],
) -> Result<Vec<DocsExportPlanEntry>, InitCommandError> {
    let mut plan = Vec::with_capacity(docs_assets.len());

    for asset in docs_assets {
        let target = docs_target_path(workspace, &asset.relative_path);
        let status = if target.is_file() {
            let existing = fs::read_to_string(&target)
                .map_err(|source| InitCommandError::ReadFile { path: target.clone(), source })?;
            if existing == asset.contents {
                DocsExportFileStatus::Unchanged
            } else {
                DocsExportFileStatus::Update
            }
        } else {
            DocsExportFileStatus::Create
        };
        plan.push(DocsExportPlanEntry {
            surface: asset.surface,
            path: asset.relative_path.clone(),
            status,
            contents: asset.contents,
        });
    }

    Ok(plan)
}

fn plan_docs_setup(docs_plan: &[DocsExportPlanEntry]) -> Vec<String> {
    let mut grouped = BTreeMap::<DocsExportSurface, Vec<&DocsExportPlanEntry>>::new();
    for entry in docs_plan {
        grouped.entry(entry.surface).or_default().push(entry);
    }

    grouped
        .into_iter()
        .map(|(surface, entries)| {
            let action = if entries.iter().any(|entry| entry.status != DocsExportFileStatus::Create)
            {
                "refresh"
            } else {
                "scaffold"
            };
            format!("- {action} {} ({} file(s))", surface.plan_label(), entries.len())
        })
        .collect()
}

fn apply_docs_plan(
    workspace: &Path,
    docs_plan: &[DocsExportPlanEntry],
) -> Result<Vec<DocsInitAction>, InitCommandError> {
    let mut grouped = BTreeMap::<DocsExportSurface, DocsInitAction>::new();

    for entry in docs_plan {
        let target = docs_target_path(workspace, &entry.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| InitCommandError::WriteFile {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let file_status = match entry.status {
            DocsExportFileStatus::Create => {
                fs::write(&target, entry.contents).map_err(|source| {
                    InitCommandError::WriteFile { path: target.clone(), source }
                })?;
                "created"
            }
            DocsExportFileStatus::Update => {
                fs::write(&target, entry.contents).map_err(|source| {
                    InitCommandError::WriteFile { path: target.clone(), source }
                })?;
                "updated"
            }
            DocsExportFileStatus::Unchanged => "unchanged",
        };

        let action = grouped.entry(entry.surface).or_insert(DocsInitAction {
            surface: entry.surface,
            status: "unchanged",
            created_files: 0,
            updated_files: 0,
            unchanged_files: 0,
        });
        match file_status {
            "created" => action.created_files += 1,
            "updated" => action.updated_files += 1,
            _ => action.unchanged_files += 1,
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

fn docs_export_root_display(docs_output_dir: Option<&Path>) -> String {
    docs_output_dir.unwrap_or_else(|| Path::new("docs/boundline")).display().to_string()
}

fn render_docs_export_conflict_report(
    docs_output_dir: Option<&Path>,
    docs_plan: &[DocsExportPlanEntry],
) -> String {
    let mut lines = vec![
        "init: documentation export blocked".to_string(),
        format!("docs_export_root: {}", docs_export_root_display(docs_output_dir)),
        "conflicting_paths:".to_string(),
    ];
    lines.extend(
        docs_plan
            .iter()
            .filter(|entry| entry.status != DocsExportFileStatus::Create)
            .map(|entry| format!("- {} ({})", entry.path, entry.status.label())),
    );
    lines.push("choose:".to_string());
    lines.push("- rerun with --refresh to update generated docs in place".to_string());
    lines.push("- rerun with --diff to preview docs changes without writing".to_string());
    lines.push("- rerun with --to <path> to export generated docs elsewhere".to_string());
    lines.push("- rerun with --force to overwrite generated docs without a prompt".to_string());
    lines.join("\n")
}

fn render_docs_export_diff_report(
    docs_output_dir: Option<&Path>,
    docs_plan: &[DocsExportPlanEntry],
) -> String {
    let changed = docs_plan
        .iter()
        .filter(|entry| entry.status != DocsExportFileStatus::Unchanged)
        .collect::<Vec<_>>();

    let mut lines = vec![
        "init: documentation export diff".to_string(),
        format!("docs_export_root: {}", docs_export_root_display(docs_output_dir)),
    ];
    if changed.is_empty() {
        lines.push("docs_export_diff: no content changes".to_string());
    } else {
        lines.push("docs_export_diff:".to_string());
        lines.extend(
            changed.into_iter().map(|entry| format!("- {} {}", entry.status.label(), entry.path)),
        );
    }
    lines.push("next_steps:".to_string());
    lines.push("- rerun with --export-docs to create any missing generated docs".to_string());
    lines
        .push("- rerun with --export-docs --refresh to update existing generated docs".to_string());
    lines.join("\n")
}

fn validate_docs_export_options(request: &InitRequest<'_>) -> Result<(), InitCommandError> {
    if !request.export_docs
        && (request.docs_refresh || request.docs_diff || request.docs_output_dir.is_some())
    {
        return Err(InitCommandError::InvalidDocsExportArgument(
            "--refresh, --diff, and --to require --export-docs".to_string(),
        ));
    }
    if request.docs_refresh && request.docs_diff {
        return Err(InitCommandError::InvalidDocsExportArgument(
            "--refresh and --diff cannot be used together".to_string(),
        ));
    }
    Ok(())
}

fn resolve_ide_setup(
    requested_ide: &[IdeKind],
    auto_approve: Option<TerminalAutoApproveProfile>,
    existing_manifest: Option<&ScaffoldManifest>,
) -> Vec<IdeSetupSelection> {
    let mut selected = BTreeMap::<IdeKind, Option<TerminalAutoApproveProfile>>::new();

    if requested_ide.is_empty() {
        if let Some(existing_manifest) = existing_manifest {
            for setup in &existing_manifest.ide_setup {
                selected.insert(setup.ide, setup.auto_approve);
            }
        }
    } else {
        for ide in requested_ide {
            let profile = if *ide == IdeKind::VsCode {
                Some(auto_approve.unwrap_or(TerminalAutoApproveProfile::ReadOnly))
            } else {
                None
            };
            selected.insert(*ide, profile);
        }
    }

    selected
        .into_iter()
        .map(|(ide, auto_approve)| IdeSetupSelection { ide, auto_approve })
        .collect()
}

fn plan_ide_setup(
    workspace: &Path,
    selections: &[IdeSetupSelection],
) -> Result<Vec<PlannedIdeEntry>, InitCommandError> {
    let mut entries = Vec::new();
    for selection in selections {
        let (path, setup_kind, contents) = match selection.ide {
            IdeKind::VsCode => (
                PathBuf::from(".vscode/settings.json"),
                "managed-settings",
                render_vscode_settings(
                    workspace,
                    selection.auto_approve.unwrap_or(TerminalAutoApproveProfile::ReadOnly),
                )?,
            ),
            IdeKind::Cursor => (
                PathBuf::from(".cursor/rules/boundline.md"),
                "manual-guidance",
                render_cursor_ide_guidance(),
            ),
            IdeKind::Antigravity => (
                PathBuf::from(".boundline/ide/antigravity.md"),
                "manual-guidance",
                render_antigravity_ide_guidance(),
            ),
            IdeKind::JetBrains => (
                PathBuf::from(".boundline/ide/jetbrains.md"),
                "manual-guidance",
                render_jetbrains_ide_guidance(),
            ),
        };
        let target = workspace.join(&path);
        let status = scaffold_file_status(&target, &contents)?.label();
        let path_text =
            workspace_relative_path(&path).unwrap_or_else(|| path.display().to_string());
        entries.push(PlannedIdeEntry {
            action: IdeInitAction { ide: selection.ide, setup_kind, status, path: path_text },
            artifact: RenderedManagedArtifact {
                path,
                target: ScaffoldTarget::Ide,
                ownership: ScaffoldOwnershipMode::Merge,
                contents,
            },
        });
    }
    Ok(entries)
}

fn render_vscode_settings(
    workspace: &Path,
    profile: TerminalAutoApproveProfile,
) -> Result<String, InitCommandError> {
    let settings_path = workspace.join(".vscode/settings.json");
    let mut root = if settings_path.is_file() {
        let contents = fs::read_to_string(&settings_path)
            .map_err(|source| InitCommandError::ReadFile { path: settings_path.clone(), source })?;
        serde_json::from_str::<Value>(&contents).map_err(|source| {
            InitCommandError::InvalidIdeSettings {
                path: settings_path.clone(),
                detail: format!(
                    "fix the JSON syntax before Boundline can merge IDE settings: {source}"
                ),
            }
        })?
    } else {
        Value::Object(Map::new())
    };

    let root_object = root.as_object_mut().ok_or_else(|| InitCommandError::InvalidIdeSettings {
        path: settings_path.clone(),
        detail: "fix the JSON syntax: the top-level VS Code settings value must be an object"
            .to_string(),
    })?;
    let auto_value = root_object
        .entry("chat.tools.terminal.autoApprove".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !auto_value.is_object() {
        *auto_value = Value::Object(Map::new());
    }
    let auto = auto_value.as_object_mut().ok_or_else(|| InitCommandError::InvalidIdeSettings {
        path: settings_path.clone(),
        detail: "terminal auto-approve value could not be converted to object after forced reset"
            .to_string(),
    })?;
    remove_boundline_auto_approve_entries(auto);
    match profile {
        TerminalAutoApproveProfile::ReadOnly => apply_read_only_auto_approve_entries(auto),
        TerminalAutoApproveProfile::SessionSafe => apply_session_safe_auto_approve_entries(auto),
        TerminalAutoApproveProfile::Trusted => {
            auto.insert("boundline".to_string(), Value::Bool(true));
            auto.insert("canon".to_string(), Value::Bool(true));
        }
    }

    serde_json::to_string_pretty(&root)
        .map(|mut rendered| {
            rendered.push('\n');
            rendered
        })
        .map_err(|source| InitCommandError::InvalidIdeSettings {
            path: settings_path,
            detail: format!("failed to render VS Code settings: {source}"),
        })
}

fn remove_boundline_auto_approve_entries(auto: &mut Map<String, Value>) {
    for key in [
        "boundline",
        "canon",
        "/^boundline (doctor|status|next|inspect|orchestrate)\\b/",
        "/^boundline goal\\b/",
        "/^boundline plan\\b/",
        "/^boundline run\\b/",
        "/^boundline config show\\b/",
        "/^boundline workflow (list|status|inspect)\\b/",
        "/^boundline update\\b(?!.*\\s--(apply|force|adopt|prune)\\b)/",
        "/^boundline (init|run|step|orchestrate|workflow (run|resume)|config (set|unset|bind-context|unbind-context)|cluster init)\\b/",
        "/^boundline (init|run|step|workflow (run|resume)|config (set|unset|bind-context|unbind-context)|cluster init)\\b/",
        "/^boundline init\\b/",
        "/^boundline workflow (run|resume)\\b/",
        "/^boundline config (set|unset|bind-context|unbind-context)\\b/",
        "/^boundline cluster init\\b/",
    ] {
        auto.remove(key);
    }
}

fn apply_read_only_auto_approve_entries(auto: &mut Map<String, Value>) {
    auto.insert("boundline".to_string(), Value::Bool(false));
    auto.insert("canon".to_string(), Value::Bool(false));
    for pattern in [
        "/^boundline (doctor|status|next|inspect|orchestrate)\\b/",
        "/^boundline config show\\b/",
        "/^boundline workflow (list|status|inspect)\\b/",
        "/^boundline update\\b(?!.*\\s--(apply|force|adopt|prune)\\b)/",
    ] {
        auto.insert(pattern.to_string(), auto_approve_rule(true));
    }
    auto.insert(
        "/^boundline (init|run|step|workflow (run|resume)|config (set|unset|bind-context|unbind-context)|cluster init)\\b/".to_string(),
        auto_approve_rule(false),
    );
}

fn apply_session_safe_auto_approve_entries(auto: &mut Map<String, Value>) {
    auto.insert("boundline".to_string(), Value::Bool(false));
    auto.insert("canon".to_string(), Value::Bool(false));
    for pattern in [
        "/^boundline (doctor|status|next|inspect|orchestrate)\\b/",
        "/^boundline goal\\b/",
        "/^boundline plan\\b/",
        "/^boundline run\\b/",
        "/^boundline config show\\b/",
        "/^boundline workflow (list|status|inspect)\\b/",
        "/^boundline update\\b(?!.*\\s--(apply|force|adopt|prune)\\b)/",
    ] {
        auto.insert(pattern.to_string(), auto_approve_rule(true));
    }
    for pattern in [
        "/^boundline init\\b/",
        "/^boundline workflow (run|resume)\\b/",
        "/^boundline config (set|unset|bind-context|unbind-context)\\b/",
        "/^boundline cluster init\\b/",
    ] {
        auto.insert(pattern.to_string(), auto_approve_rule(false));
    }
}

fn auto_approve_rule(approve: bool) -> Value {
    json!({
        "approve": approve,
        "matchCommandLine": true
    })
}

fn render_cursor_ide_guidance() -> String {
    "# Boundline Cursor Guidance\n\nCursor support is managed as guidance because Boundline does not currently claim a stable Cursor terminal auto-approval settings schema. Use `boundline init --assistant codex` or another supported assistant pack for repo-local commands, keep the CLI output authoritative, and configure any Cursor auto-run policy manually in Cursor.\n"
        .to_string()
}

fn render_antigravity_ide_guidance() -> String {
    "# Boundline Antigravity Guidance\n\nAntigravity support is repo-local and CLI-first. Start with `boundline init --assistant antigravity`, refresh with `boundline update --workspace <workspace> --target assistant --apply`, and configure Antigravity terminal execution policy manually when your local installation supports it.\n"
        .to_string()
}

fn render_jetbrains_ide_guidance() -> String {
    "# Boundline JetBrains Guidance\n\nJetBrains support is documentation-only in this Boundline release. Use the installed `boundline` CLI from the JetBrains terminal and avoid generated terminal auto-approval settings until JetBrains exposes a stable project-scoped schema for AI terminal command approval.\n"
        .to_string()
}

fn summarize_ide_setup(ide_plan: &[PlannedIdeEntry]) -> Vec<IdeInitAction> {
    ide_plan.iter().map(|entry| entry.action.clone()).collect()
}

fn apply_ide_setup(
    workspace: &Path,
    ide_plan: &[PlannedIdeEntry],
) -> Result<Vec<IdeInitAction>, InitCommandError> {
    for entry in ide_plan {
        write_scaffold_file(&workspace.join(&entry.artifact.path), &entry.artifact.contents)?;
    }
    Ok(summarize_ide_setup(ide_plan))
}

fn plan_workspace_hygiene_defaults(
    workspace: &Path,
    domains: &BTreeSet<DomainFamily>,
) -> Result<Vec<PlannedHygieneEntry>, InitCommandError> {
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
            "created"
        } else if merged.added_patterns.is_empty() {
            "unchanged"
        } else {
            "updated"
        };

        actions.push(PlannedHygieneEntry {
            action: HygieneInitAction {
                path: plan.path.to_string(),
                status,
                added_patterns: merged.added_patterns.len(),
                preserved_custom_lines: merged.preserved_custom_lines,
                sources: plan.packs.into_iter().map(|pack| pack.provenance).collect(),
            },
            final_content: merged.content,
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
            actions.push(PlannedHygieneEntry {
                action: HygieneInitAction {
                    path: path.to_string(),
                    status: "skipped",
                    added_patterns: 0,
                    preserved_custom_lines: 0,
                    sources: vec!["not-relevant".to_string()],
                },
                final_content: String::new(),
            });
        }
    }

    Ok(actions)
}

fn apply_workspace_hygiene_plan(
    workspace: &Path,
    plan: &[PlannedHygieneEntry],
) -> Result<Vec<HygieneInitAction>, InitCommandError> {
    let mut actions = Vec::new();
    for entry in plan {
        let target = workspace.join(&entry.action.path);
        if entry.action.status != "unchanged" && entry.action.status != "skipped" {
            let contents = entry.final_content.as_str();
            fs::write(&target, contents)
                .map_err(|source| InitCommandError::WriteFile { path: target.clone(), source })?;
        }
        actions.push(entry.action.clone());
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

fn format_assistant_host_list(assistants: &[AssistantHostKind]) -> String {
    assistants.iter().map(|assistant| assistant.as_str()).collect::<Vec<_>>().join(", ")
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

fn init_global_inspect_command() -> String {
    "boundline config show --scope global".to_string()
}

fn init_doctor_command(workspace: &Path) -> String {
    format!("boundline doctor --workspace {}", workspace.display())
}

fn init_install_doctor_command() -> String {
    "boundline doctor --install".to_string()
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
    assistants: Vec<AssistantHostKind>,
    routes: Vec<GuidedRouteSelection>,
}

fn collect_guided_init_answers_with_interactor(
    interactor: &mut dyn InitInteractor,
    prompt_for_canon_mode: bool,
    prompt_for_assistants: bool,
    prompt_for_routes: bool,
    catalog: &BundledModelCatalog,
    explicit_assistants: &[AssistantHostKind],
) -> Result<GuidedInitAnswers, InitCommandError> {
    let canon_mode_selection =
        if prompt_for_canon_mode { Some(select_canon_mode(interactor)?) } else { None };

    let assistants = if prompt_for_assistants {
        select_assistants(interactor)?
    } else {
        explicit_assistants.to_vec()
    };

    let routes = if prompt_for_routes {
        review_routes(interactor, catalog, &assistant_runtimes_for_hosts(&assistants))?
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
) -> Result<Vec<AssistantHostKind>, InitCommandError> {
    let items = assistant_host_labels();
    let defaults = vec![false; items.len()];
    let indices = interactor.multi_select(
        "Assistant surfaces\nSelect the repository-local assistant packs to scaffold. Provider-backed hosts can also seed route defaults.",
        &items,
        &defaults,
    )?;
    Ok(indices.into_iter().filter_map(|index| INIT_ASSISTANT_HOSTS.get(index).copied()).collect())
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
    if assistants.is_empty() {
        return required_init_route_slots()
            .into_iter()
            .map(|slot| match catalog.default_route_for_slot(slot) {
                Some(route) => GuidedRouteSelection {
                    slot,
                    route: Some(route),
                    source: GuidedRouteSource::Bundled,
                },
                None => {
                    GuidedRouteSelection { slot, route: None, source: GuidedRouteSource::Unset }
                }
            })
            .collect();
    }

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
    let bundled_default = catalog.default_route_for_slot(slot);
    let runtime_default = selection
        .route
        .as_ref()
        .map(|route| route.runtime)
        .or_else(|| bundled_default.as_ref().map(|route| route.runtime))
        .and_then(|runtime| catalog.runtimes.iter().position(|entry| entry.runtime == runtime))
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
        .or_else(|| {
            bundled_default.as_ref().and_then(|route| {
                if route.runtime != runtime_entry.runtime {
                    return None;
                }
                catalog.runtime_entry(runtime_entry.runtime).and_then(|entry| {
                    entry.models.iter().position(|model| model.model_id == route.model)
                })
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

fn run_init_activity<T, F>(
    label: &str,
    interactive_terminal: bool,
    action: F,
) -> Result<T, InitCommandError>
where
    F: FnOnce() -> Result<T, InitCommandError>,
{
    if interactive_terminal {
        let spinner_running = Arc::new(AtomicBool::new(true));
        let spinner_signal = Arc::clone(&spinner_running);
        let spinner_label = label.to_string();
        let spinner_thread = thread::spawn(move || {
            let frames = ['|', '/', '-', '\\'];
            let mut index = 0usize;

            while spinner_signal.load(Ordering::Relaxed) {
                eprint!("\r{} {}", frames[index % frames.len()], spinner_label);
                let _ = io::stderr().flush();
                thread::sleep(Duration::from_millis(PROGRESS_TICK_MS));
                index += 1;
            }
        });
        let result = action();
        spinner_running.store(false, Ordering::Relaxed);
        let _ = spinner_thread.join();
        clear_progress_line();
        match &result {
            Ok(_) => {}
            Err(_) => eprintln!("{label}: failed"),
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

fn clear_progress_line() {
    eprint!("\r\x1b[2K");
    let _ = io::stderr().flush();
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

    if let Some(canon) = canon {
        let default_risk =
            canon_preference_or_default(canon.default_risk.as_deref(), DEFAULT_CANON_RISK);
        let default_zone =
            canon_preference_or_default(canon.default_zone.as_deref(), DEFAULT_CANON_ZONE);
        let default_owner =
            canon_preference_or_default(canon.default_owner.as_deref(), DEFAULT_CANON_OWNER);
        let default_system_context = canon_preference_or_default(
            canon.default_system_context.as_deref(),
            DEFAULT_CANON_SYSTEM_CONTEXT,
        );
        let (flow_name, stage_id, canon_mode) = match template {
            InitTemplate::BugFix => ("bug-fix", "investigate", "discovery"),
            InitTemplate::Change => ("change", "understand-change", "change"),
            InitTemplate::Delivery => ("delivery", "requirements", "requirements"),
        };
        execution["governance"] = json!({
            "default_runtime": "canon",
            "canon": {
                "command": DEFAULT_CANON_COMMAND,
                "default_owner": default_owner,
                "default_risk": default_risk,
                "default_zone": default_zone,
                "default_system_context": default_system_context
            },
            "stages": [{
                "flow_name": flow_name,
                "stage_id": stage_id,
                "enabled": true,
                "required": true,
                "autopilot": false,
                "runtime": "canon",
                "canon_mode": canon_mode,
                "system_context": default_system_context,
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

fn assistant_host_capability_line(assistant: AssistantHostKind) -> &'static str {
    match assistant.default_runtime() {
        Some(runtime) => runtime_capability_line(runtime),
        None => "scaffolds repo-local assets without a provider runtime default",
    }
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
    #[error(
        "current working directory is unavailable while resolving workspace {workspace}: {source}. Rerun from an existing directory or pass --workspace /absolute/path"
    )]
    CurrentDirectoryUnavailable { workspace: PathBuf, source: std::io::Error },
    #[error("failed to resolve the init workspace: {0}")]
    WorkspaceResolution(String),
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
    #[error("failed to serialize execution profile: {0}")]
    InvalidExecutionProfile(String),
    #[error("failed to bootstrap Canon workspace: {0}")]
    CanonWorkspaceBootstrap(String),
    #[error("failed to collect init input: {0}")]
    PromptInteraction(String),
    #[error("invalid docs export argument: {0}")]
    InvalidDocsExportArgument(String),
    #[error("invalid IDE settings at {path}: {detail}")]
    InvalidIdeSettings { path: PathBuf, detail: String },
    #[error("invalid init scope argument: {0}")]
    InvalidScopeArgument(String),
    #[error("invalid domain argument: {0}")]
    InvalidDomainArgument(String),
    #[error("invalid domain template settings: {0}")]
    InvalidDomainTemplate(String),
    #[error("failed to serialize managed config preview at {path}: {source}")]
    SerializeConfigPreview { path: PathBuf, source: toml::ser::Error },
    #[error("workspace {workspace} is not initialized; run `boundline init` first")]
    UpdateWorkspaceNotInitialized { workspace: PathBuf },
    #[error("updating `.boundline/execution.json` requires `--template <template>`")]
    UpdateExecutionTemplateRequired,
    #[error(
        "`--template` can only be used together with `--target execution` or no explicit targets"
    )]
    UpdateTemplateRequiresExecutionTarget,
    #[error(
        "`--ide` and `--auto-approve` can only be used together with `--target ide` or no explicit targets"
    )]
    UpdateIdeOptionsRequireIdeTarget,
    #[error("failed to serialize scaffold manifest at {path}: {source}")]
    SerializeScaffoldManifest { path: PathBuf, source: serde_json::Error },
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    use uuid::Uuid;

    use super::{
        BOUNDLINE_DIR_RELATIVE, BOUNDLINE_VERSION, BundledModelCatalog,
        CANON_MAINTAINABILITY_REVIEWER_ROLE_ID, CANON_SAFETY_REVIEWER_ROLE_ID,
        EXECUTION_PROFILE_FILE_NAME, GuidedRouteSource, InitCommandError, InitInteractor,
        InitRequest, UpdateRequest, UpdateTarget, canon_reviewer_route_readiness,
        collect_guided_init_answers_with_interactor, command_in_path,
        ensure_workspace_project_doc_roots, execute_init, execute_update, execution_template,
        format_runtime_list, format_slot_list, initial_guided_route_selections,
        parse_canon_mode_selection, parse_context_binding, parse_domain_family,
        parse_domain_standard, parse_external_context_kind, parse_model_route,
        render_guided_route_review, resolve_seeded_routes, resolve_workspace_root,
        supported_route_slots, supported_runtime_choices, template_label, upsert_binding,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::cli::CommandExitStatus;
    use crate::domain::configuration::{
        CanonPreferences, ConfigFile, InitConfigScope, InitTemplate, ModelRoute, RouteSlot,
        RoutingConfig, RuntimeKind,
    };
    use crate::domain::domain_templates::{
        DomainFamily, ExternalContextBinding, ExternalContextKind,
    };
    use crate::domain::governance::CanonModeSelectionPreference;
    use crate::domain::scaffold_manifest::{
        INITIAL_SCAFFOLD_EPOCH, SCAFFOLD_MANIFEST_FILE_NAME, SCAFFOLD_MANIFEST_VERSION,
        ScaffoldManifest,
    };
    use crate::test_support::CurrentDirGuard;

    static GLOBAL_CONFIG_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    // Keeps per-test overrides focused on the behavior under test instead of
    // repeating the default workspace init boilerplate in every case.
    fn base_init_request<'a>(workspace: &'a Path) -> InitRequest<'a> {
        InitRequest {
            workspace,
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
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
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
            force: true,
        }
    }

    fn copilot_canon_init_request<'a>(
        workspace: &'a Path,
        template: InitTemplate,
    ) -> InitRequest<'a> {
        InitRequest {
            template: Some(template),
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            ..base_init_request(workspace)
        }
    }

    // Mirrors `base_init_request` for update tests so each case can override
    // only the few flags that matter to the scenario under test.
    fn base_update_request<'a>(workspace: &'a Path) -> UpdateRequest<'a> {
        UpdateRequest {
            workspace,
            targets: &[],
            ide: &[],
            auto_approve: None,
            template: None,
            diff: false,
            apply: false,
            adopt: false,
            prune: false,
            status: false,
            force: false,
        }
    }

    fn initialize_workspace_for_update(workspace: &Path) {
        execute_init(copilot_canon_init_request(workspace, InitTemplate::BugFix)).unwrap();
    }

    fn load_workspace_scaffold_manifest(workspace: &Path) -> ScaffoldManifest {
        let manifest_path =
            workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME);
        let contents = fs::read_to_string(manifest_path).unwrap();
        serde_json::from_str(&contents).unwrap()
    }

    struct PwdEnvGuard {
        original: Option<OsString>,
    }

    impl PwdEnvGuard {
        fn set(path: &Path) -> Self {
            let original = std::env::var_os("PWD");
            unsafe {
                std::env::set_var("PWD", path);
            }
            Self { original }
        }

        fn remove() -> Self {
            let original = std::env::var_os("PWD");
            unsafe {
                std::env::remove_var("PWD");
            }
            Self { original }
        }
    }

    impl Drop for PwdEnvGuard {
        fn drop(&mut self) {
            match self.original.as_ref() {
                Some(value) => unsafe {
                    std::env::set_var("PWD", value);
                },
                None => unsafe {
                    std::env::remove_var("PWD");
                },
            }
        }
    }

    struct GlobalConfigEnvGuard<'a> {
        old_xdg: Option<OsString>,
        old_home: Option<OsString>,
        _lock: MutexGuard<'a, ()>,
    }

    impl Drop for GlobalConfigEnvGuard<'_> {
        fn drop(&mut self) {
            unsafe {
                match &self.old_xdg {
                    Some(value) => std::env::set_var("XDG_CONFIG_HOME", value),
                    None => std::env::remove_var("XDG_CONFIG_HOME"),
                }
                match &self.old_home {
                    Some(value) => std::env::set_var("HOME", value),
                    None => std::env::remove_var("HOME"),
                }
            }
        }
    }

    fn with_global_config_env<T>(
        xdg_home: Option<&Path>,
        home: Option<&Path>,
        action: impl FnOnce() -> T,
    ) -> T {
        let lock = GLOBAL_CONFIG_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
        let restore = GlobalConfigEnvGuard {
            old_xdg: std::env::var_os("XDG_CONFIG_HOME"),
            old_home: std::env::var_os("HOME"),
            _lock: lock,
        };

        unsafe {
            match xdg_home {
                Some(path) => std::env::set_var("XDG_CONFIG_HOME", path),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
            match home {
                Some(path) => std::env::set_var("HOME", path),
                None => std::env::remove_var("HOME"),
            }
        }

        let result = action();
        drop(restore);
        result
    }

    struct CanonInstallOverrideGuard {
        previous: Option<crate::domain::distribution::CanonInstallStatus>,
    }

    impl Drop for CanonInstallOverrideGuard {
        fn drop(&mut self) {
            let previous = self.previous.take();
            let _ = super::replace_test_canon_install_status_override(previous);
        }
    }

    fn with_canon_install_override<T>(
        status: crate::domain::distribution::CanonInstallStatus,
        action: impl FnOnce() -> T,
    ) -> T {
        let previous = super::replace_test_canon_install_status_override(Some(status));
        let guard = CanonInstallOverrideGuard { previous };
        let result = action();
        drop(guard);
        result
    }

    #[derive(Debug, Default)]
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

    #[derive(Debug)]
    struct CurrentDirChangingInteractor {
        inner: ScriptedInteractor,
        confirm_dir: PathBuf,
    }

    impl InitInteractor for CurrentDirChangingInteractor {
        fn select(
            &mut self,
            prompt: &str,
            items: &[String],
            default: usize,
        ) -> Result<usize, super::InitCommandError> {
            self.inner.select(prompt, items, default)
        }

        fn multi_select(
            &mut self,
            prompt: &str,
            items: &[String],
            defaults: &[bool],
        ) -> Result<Vec<usize>, super::InitCommandError> {
            self.inner.multi_select(prompt, items, defaults)
        }

        fn input(
            &mut self,
            prompt: &str,
            initial: &str,
        ) -> Result<String, super::InitCommandError> {
            self.inner.input(prompt, initial)
        }

        fn confirm(
            &mut self,
            prompt: &str,
            default: bool,
        ) -> Result<bool, super::InitCommandError> {
            std::env::set_current_dir(&self.confirm_dir).unwrap();
            self.inner.confirm(prompt, default)
        }
    }

    #[test]
    fn execute_init_infers_and_seeds_domain_templates() {
        let workspace = temp_workspace("boundline-init-domain");
        fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#)
            .unwrap();
        fs::create_dir_all(workspace.join("design")).unwrap();

        let report = execute_init(InitRequest {
            domain_standards: &["react=workspace react rules".to_string()],
            context_bindings: &["react|design_system|mcp:design-system".to_string()],
            required_context_bindings: &["react|design_reference|design/reference.md".to_string()],
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            ..base_init_request(&workspace)
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
            context_bindings: &["react|design_system".to_string()],
            ..base_init_request(&workspace)
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
            template: Some(InitTemplate::Delivery),
            force: false,
            ..base_init_request(&workspace)
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
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
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
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
            force: true,
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("template: change"));
        assert!(report.terminal_output.contains("domain_templates: none"));
        assert!(report.terminal_output.contains("canon_bootstrap: ready"));
        assert!(report.terminal_output.contains("canon_workspace_bootstrap:"));
        assert!(workspace.join(".canon").is_dir());

        let execution_profile =
            fs::read_to_string(workspace.join(".boundline/execution.json")).unwrap();
        assert!(execution_profile.contains("init-change"));
        assert!(execution_profile.contains("\"governance\""), "{execution_profile}");
        assert!(
            execution_profile.contains("\"default_runtime\": \"canon\""),
            "{execution_profile}"
        );
        assert!(
            execution_profile.contains("\"default_risk\": \"bounded-impact\""),
            "{execution_profile}"
        );
        assert!(
            execution_profile.contains("\"default_zone\": \"engineering\""),
            "{execution_profile}"
        );
        assert!(
            execution_profile.contains("\"default_owner\": \"platform\""),
            "{execution_profile}"
        );
    }

    fn blocked_canon_install_status(
        message: &str,
        repair_action: &str,
    ) -> crate::domain::distribution::CanonInstallStatus {
        let mut blocked_status = super::default_test_canon_install_status();
        blocked_status.state = crate::domain::distribution::CompanionState::RepairNeeded;
        blocked_status.message = message.to_string();
        blocked_status.suggested_actions = vec![repair_action.to_string()];
        if let Some(surface) = blocked_status.surface_verification.as_mut() {
            surface.ready = false;
            surface.repair_actions = vec![repair_action.to_string()];
        }
        blocked_status
    }

    #[test]
    fn execute_init_blocks_when_canon_surface_is_unavailable() {
        let workspace = temp_workspace("boundline-init-canon-blocked");
        let mut blocked_status = blocked_canon_install_status(
            "Canon governance surface is unavailable",
            "install or repair Canon 0.62.0 before rerunning init",
        );
        if let Some(surface) = blocked_status.surface_verification.as_mut() {
            surface.operations_verified = false;
            surface.missing_operations = vec!["start".to_string(), "refresh".to_string()];
        }

        let report = with_canon_install_override(blocked_status, || {
            execute_init(InitRequest {
                workspace: &workspace,
                scope: InitConfigScope::Workspace,
                non_interactive: true,
                interactive_terminal_override: None,
                interactor: None,
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
                ide: &[],
                auto_approve: None,
                export_docs: false,
                docs_refresh: false,
                docs_diff: false,
                docs_output_dir: None,
                force: true,
            })
            .unwrap()
        });

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            report.terminal_output.contains("init: blocked - Canon surface not ready"),
            "{}",
            report.terminal_output
        );
        assert!(
            report.terminal_output.contains("canon_bootstrap: blocked"),
            "{}",
            report.terminal_output
        );
        assert!(
            report.terminal_output.contains("install or repair Canon 0.62.0 before rerunning init"),
            "{}",
            report.terminal_output
        );
        assert!(!workspace.join(".boundline/execution.json").exists());
    }

    #[test]
    fn execute_init_fails_fast_before_planning_when_canon_surface_is_unavailable() {
        // Verifies that the Canon surface pre-flight fires immediately after guided
        // prompts but before any config loading or asset computation: the output
        // must say "blocked before planning" and no workspace files must exist.
        let workspace = temp_workspace("boundline-init-canon-preflight");
        let mut blocked_status = blocked_canon_install_status(
            "Canon 0.10.0 is present but version 0.62.0 is required",
            "upgrade Canon to 0.62.0 or later",
        );
        if let Some(surface) = blocked_status.surface_verification.as_mut() {
            surface.version_compatible = false;
        }

        let report = with_canon_install_override(blocked_status, || {
            execute_init(InitRequest {
                workspace: &workspace,
                scope: InitConfigScope::Workspace,
                non_interactive: true,
                interactive_terminal_override: None,
                interactor: None,
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
                ide: &[],
                auto_approve: None,
                export_docs: false,
                docs_refresh: false,
                docs_diff: false,
                docs_output_dir: None,
                force: true,
            })
            .unwrap()
        });

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            report.terminal_output.contains("init: blocked - Canon surface not ready"),
            "{}",
            report.terminal_output
        );
        assert!(
            report.terminal_output.contains("blocked before planning"),
            "expected 'blocked before planning' in output to confirm fail-fast path;\n{}",
            report.terminal_output
        );
        assert!(report.terminal_output.contains("0.62.0"), "{}", report.terminal_output);
        // No workspace files must be created by the blocked run.
        assert!(!workspace.join(".boundline").exists());
    }

    #[test]
    fn execute_init_seeds_missing_routes_from_selected_assistant_defaults() {
        let workspace = temp_workspace("boundline-init-default-routes");

        let report = execute_init(InitRequest {
            workspace: &workspace,
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
            template: None,
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
            force: true,
        })
        .unwrap();

        assert!(report.terminal_output.contains("route_setup:"));
        assert!(report.terminal_output.contains("assistant_defaults: copilot"));
        assert!(
            report.terminal_output.contains("seeded planning: copilot:gpt-4.1 [assistant-default]")
        );
        assert!(
            report.terminal_output.contains("seeded review: copilot:gpt-4.1 [assistant-default]")
        );
        assert!(
            report.terminal_output.contains("inspect_or_edit: boundline config show --workspace")
        );

        let saved = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(saved.routing.planning.unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(saved.routing.implementation.unwrap().model, "gpt-4.1");
        assert_eq!(saved.routing.verification.unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(saved.routing.review.unwrap().runtime, RuntimeKind::Copilot);
        assert!(saved.routing.reviewer_roles.contains_key(CANON_SAFETY_REVIEWER_ROLE_ID));
        assert!(saved.routing.reviewer_roles.contains_key(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID));
        assert_ne!(
            saved.routing.reviewer_roles.get(CANON_SAFETY_REVIEWER_ROLE_ID),
            saved.routing.reviewer_roles.get(CANON_MAINTAINABILITY_REVIEWER_ROLE_ID)
        );
    }

    #[test]
    fn canon_reviewer_route_readiness_rejects_duplicate_mandatory_routes() {
        let local = ConfigFile {
            routing: RoutingConfig {
                reviewer_roles: BTreeMap::from([
                    (
                        CANON_SAFETY_REVIEWER_ROLE_ID.to_string(),
                        ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() },
                    ),
                    (
                        CANON_MAINTAINABILITY_REVIEWER_ROLE_ID.to_string(),
                        ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() },
                    ),
                ]),
                ..RoutingConfig::default()
            },
            canon: Some(CanonPreferences {
                mode_selection: CanonModeSelectionPreference::AutoConfirm,
                default_risk: None,
                default_zone: None,
                default_owner: None,
                default_system_context: None,
            }),
            ..ConfigFile::default()
        };

        let readiness = canon_reviewer_route_readiness(Some(&local), None);

        assert!(!readiness.ready);
        assert!(readiness.detail.contains("collapse onto copilot:gpt-5.4"));
        assert!(
            readiness
                .repair_actions
                .iter()
                .any(|action| action.contains("routing.reviewer_roles.safety"))
        );
    }

    #[test]
    fn execute_init_preserves_explicit_routes_while_seeding_remaining_slots() {
        let workspace = temp_workspace("boundline-init-partial-routes");
        let explicit = ["planning=copilot:gpt-4o".to_string()];

        let report = execute_init(InitRequest {
            workspace: &workspace,
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
            template: None,
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            routes: &explicit,
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
            force: true,
        })
        .unwrap();

        assert!(report.terminal_output.contains("route_setup:"));
        assert!(report.terminal_output.contains("explicit planning: copilot:gpt-4o [explicit]"));
        assert!(
            report
                .terminal_output
                .contains("seeded verification: copilot:gpt-4.1 [assistant-default]")
        );

        let saved = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(saved.routing.planning.unwrap().model, "gpt-4o");
        assert_eq!(saved.routing.implementation.unwrap().model, "gpt-4.1");
        assert_eq!(saved.routing.review.unwrap().model, "gpt-4.1");
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
            parse_model_route("planning-codex-o4-mini")
                .unwrap_err()
                .to_string()
                .contains("SLOT=RUNTIME:MODEL")
        );
        assert!(
            parse_model_route("plan=codex:o4-mini")
                .unwrap_err()
                .to_string()
                .contains(&supported_route_slots())
        );
        assert!(
            parse_model_route("planning=cursor:o4-mini")
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
        let claude_custom_model_index =
            catalog.runtime_entry(RuntimeKind::Claude).map(|entry| entry.models.len()).unwrap();
        let mut interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![0, 1, 2, claude_custom_model_index, 0]),
            multi_selects: VecDeque::from(vec![vec![0]]),
            inputs: VecDeque::from(vec!["gpt-4o-enterprise".to_string()]),
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
        assert_eq!(
            answers.assistants,
            vec![crate::domain::configuration::AssistantHostKind::Claude]
        );
        assert_eq!(answers.routes[0].route.as_ref().unwrap().runtime, RuntimeKind::Claude);
        assert_eq!(answers.routes[0].route.as_ref().unwrap().model, "gpt-4o-enterprise");
        assert!(matches!(answers.routes[0].source, GuidedRouteSource::Custom));
    }

    #[test]
    fn execute_init_requires_non_interactive_flag_without_tty_when_guided_values_are_missing() {
        let workspace = temp_workspace("boundline-init-no-tty");

        let error = execute_init(InitRequest {
            workspace: &workspace,
            scope: InitConfigScope::Workspace,
            non_interactive: false,
            interactive_terminal_override: Some(false),
            interactor: None,
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
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
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
        assert!(error.to_string().contains("--route planning=copilot:gpt-4.1"));
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
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
            template: None,
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
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
            scope: InitConfigScope::Workspace,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
            template: None,
            assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
            routes: &[],
            domains: &[],
            domain_standards: &[],
            context_bindings: &[],
            required_context_bindings: &[],
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            ide: &[],
            auto_approve: None,
            export_docs: false,
            docs_refresh: false,
            docs_diff: false,
            docs_output_dir: None,
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
            default_risk: Some("bounded-impact".to_string()),
            default_zone: Some("engineering".to_string()),
            default_owner: Some("platform".to_string()),
            default_system_context: None,
        };
        let governed_delivery = execution_template(InitTemplate::Delivery, Some(&canon));
        assert_eq!(governed_delivery["governance"]["default_runtime"], "canon");
        assert_eq!(governed_delivery["governance"]["stages"][0]["canon_mode"], "requirements");

        let partial_canon = CanonPreferences {
            mode_selection: CanonModeSelectionPreference::AutoConfirm,
            default_risk: None,
            default_zone: None,
            default_owner: None,
            default_system_context: None,
        };
        let fallback_delivery = execution_template(InitTemplate::Delivery, Some(&partial_canon));
        assert_eq!(fallback_delivery["governance"]["canon"]["command"], "canon");
        assert_eq!(fallback_delivery["governance"]["canon"]["default_risk"], "bounded-impact");
        assert_eq!(fallback_delivery["governance"]["canon"]["default_zone"], "engineering");
        assert_eq!(fallback_delivery["governance"]["canon"]["default_owner"], "platform");
        assert_eq!(fallback_delivery["governance"]["canon"]["default_system_context"], "existing");

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
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() }),
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
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() }),
            GuidedRouteSource::AssistantDefault { fallback_from: Some(RuntimeKind::Codex) },
        );
        assert!(
            fallback.display_line().contains("fallback-from=codex-unavailable"),
            "{}",
            fallback.display_line()
        );

        let no_fallback = make(
            RouteSlot::Planning,
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() }),
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
        assert!(
            copilot_models.iter().any(|label| label.contains("Sonnet 4.6")),
            "{copilot_models:?}"
        );
        assert!(
            copilot_models.iter().any(|label| label.contains("Gemini 2.5 Pro")),
            "{copilot_models:?}"
        );
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
            &[crate::domain::configuration::AssistantHostKind::Copilot],
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
            &[crate::domain::configuration::AssistantHostKind::Copilot],
        )
        .unwrap();
        assert_eq!(answers2.canon_mode_selection, Some(CanonModeSelectionPreference::Auto));
    }

    #[test]
    fn collect_guided_answers_skips_all_prompts_when_all_flags_are_false() {
        let catalog = BundledModelCatalog::load().unwrap();
        let explicit = [crate::domain::configuration::AssistantHostKind::Copilot];
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
        assert_eq!(
            answers.assistants,
            vec![crate::domain::configuration::AssistantHostKind::Copilot]
        );
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
    fn collect_guided_init_answers_wrapper_and_parse_model_route_cover_success_paths() {
        let catalog = BundledModelCatalog::load().unwrap();

        let answers = collect_guided_init_answers_with_interactor(
            &mut ScriptedInteractor::default(),
            false,
            false,
            false,
            &catalog,
            &[crate::domain::configuration::AssistantHostKind::Copilot],
        )
        .unwrap();
        assert_eq!(answers.canon_mode_selection, None);
        assert_eq!(
            answers.assistants,
            vec![crate::domain::configuration::AssistantHostKind::Copilot]
        );
        assert!(answers.routes.is_empty());

        let (slot, route) = parse_model_route("planning=copilot:gpt-4o").unwrap();
        assert_eq!(slot, RouteSlot::Planning);
        assert_eq!(route.runtime, RuntimeKind::Copilot);
        assert_eq!(route.model, "gpt-4o");

        assert!(parse_model_route("planning=copilot: ").is_err());
    }

    #[test]
    fn select_assistants_filters_indices_that_are_not_in_the_catalog() {
        let mut interactor = ScriptedInteractor {
            multi_selects: VecDeque::from(vec![vec![0, 999]]),
            ..Default::default()
        };

        let assistants = super::select_assistants(&mut interactor).unwrap();
        assert_eq!(assistants.len(), 1);
    }

    #[test]
    fn assistant_asset_plan_and_apply_cover_created_updated_and_unchanged_states() {
        let workspace = temp_workspace("boundline-init-assistant-asset-states");
        let assistant_assets = super::assets_for_assistants(&[
            crate::domain::configuration::AssistantHostKind::Copilot,
        ]);
        assert!(!assistant_assets.is_empty());
        let multi_file_surface_asset = assistant_assets
            .iter()
            .find(|candidate| {
                assistant_assets
                    .iter()
                    .filter(|asset| asset.surface == candidate.surface)
                    .nth(1)
                    .is_some()
            })
            .cloned()
            .unwrap();

        let initial_plan = super::plan_assistant_setup(
            &super::summarize_assistant_assets(&workspace, &assistant_assets).unwrap(),
        );
        assert!(initial_plan.iter().all(|line| line.contains("scaffold")), "{initial_plan:?}");

        let created = super::apply_assistant_assets(&workspace, &assistant_assets).unwrap();
        assert!(created.iter().all(|action| action.status == "created"), "{created:?}");
        assert_eq!(
            created
                .iter()
                .map(|action| action.created_files + action.updated_files + action.unchanged_files)
                .sum::<usize>(),
            assistant_assets.len()
        );

        let projected_prompt =
            fs::read_to_string(workspace.join(".github/prompts/boundline-goal.prompt.md")).unwrap();
        assert!(
            projected_prompt.contains(&format!(
                "Shared guidance: `{}`",
                workspace.join("assistant/README.md").display()
            )),
            "{projected_prompt:?}"
        );

        let refresh_plan = super::plan_assistant_setup(
            &super::summarize_assistant_assets(&workspace, &assistant_assets).unwrap(),
        );
        assert!(refresh_plan.is_empty(), "{refresh_plan:?}");

        let unchanged = super::apply_assistant_assets(&workspace, &assistant_assets).unwrap();
        assert!(unchanged.iter().all(|action| action.status == "unchanged"), "{unchanged:?}");

        fs::remove_file(workspace.join(multi_file_surface_asset.relative_path.as_ref())).unwrap();
        let recreated = super::apply_assistant_assets(&workspace, &assistant_assets).unwrap();
        assert!(recreated.iter().any(|action| action.status == "updated"), "{recreated:?}");
        assert_eq!(recreated.iter().map(|action| action.created_files).sum::<usize>(), 1);

        fs::write(workspace.join(multi_file_surface_asset.relative_path.as_ref()), "stale")
            .unwrap();
        let updated = super::apply_assistant_assets(&workspace, &assistant_assets).unwrap();
        assert!(updated.iter().any(|action| action.updated_files > 0), "{updated:?}");
    }

    #[test]
    fn docs_export_plan_apply_conflict_diff_and_custom_root() {
        let workspace = temp_workspace("boundline-init-docs-export-states");
        let docs_root = Path::new("docs/reference/boundline");
        let docs_assets = super::docs_assets_for_assistants_under(
            &[crate::domain::configuration::AssistantHostKind::Copilot],
            docs_root,
        );
        assert!(!docs_assets.is_empty());
        assert!(
            docs_assets
                .iter()
                .any(|asset| asset.relative_path == "docs/reference/boundline/canon.md")
        );
        assert!(
            docs_assets
                .iter()
                .all(|asset| asset.relative_path.starts_with("docs/reference/boundline/"))
        );
        assert!(docs_assets.iter().all(|asset| !asset.relative_path.contains("session-")));
        let multi_file_surface_asset = docs_assets
            .iter()
            .find(|candidate| {
                docs_assets
                    .iter()
                    .filter(|asset| asset.surface == candidate.surface)
                    .nth(1)
                    .is_some()
            })
            .unwrap();

        let initial_plan = super::plan_docs_export(&workspace, &docs_assets).unwrap();
        let initial_summary = super::plan_docs_setup(&initial_plan);
        assert!(
            initial_summary.iter().all(|line| line.contains("scaffold")),
            "{initial_summary:?}"
        );

        let created = super::apply_docs_plan(&workspace, &initial_plan).unwrap();
        assert!(created.iter().all(|action| action.status == "created"), "{created:?}");
        assert_eq!(
            created
                .iter()
                .map(|action| action.created_files + action.updated_files + action.unchanged_files)
                .sum::<usize>(),
            docs_assets.len()
        );

        let refresh_plan = super::plan_docs_export(&workspace, &docs_assets).unwrap();
        let refresh_summary = super::plan_docs_setup(&refresh_plan);
        assert!(refresh_summary.iter().all(|line| line.contains("refresh")), "{refresh_summary:?}");

        let unchanged = super::apply_docs_plan(&workspace, &refresh_plan).unwrap();
        assert!(unchanged.iter().all(|action| action.status == "unchanged"), "{unchanged:?}");

        fs::remove_file(workspace.join(&multi_file_surface_asset.relative_path)).unwrap();
        let recreated = super::apply_docs_plan(
            &workspace,
            &super::plan_docs_export(&workspace, &docs_assets).unwrap(),
        )
        .unwrap();
        assert!(recreated.iter().any(|action| action.status == "updated"), "{recreated:?}");
        assert_eq!(recreated.iter().map(|action| action.created_files).sum::<usize>(), 1);

        fs::write(workspace.join(&multi_file_surface_asset.relative_path), "stale").unwrap();
        let update_plan = super::plan_docs_export(&workspace, &docs_assets).unwrap();
        let conflict_report =
            super::render_docs_export_conflict_report(Some(docs_root), &update_plan);
        assert!(conflict_report.contains("documentation export blocked"), "{conflict_report}");
        assert!(conflict_report.contains("--refresh"), "{conflict_report}");
        assert!(conflict_report.contains("--diff"), "{conflict_report}");
        assert!(conflict_report.contains("--to <path>"), "{conflict_report}");
        let diff_report = super::render_docs_export_diff_report(Some(docs_root), &update_plan);
        assert!(
            diff_report.contains("update docs/reference/boundline/assistant/"),
            "{diff_report}"
        );
        let updated = super::apply_docs_plan(&workspace, &update_plan).unwrap();
        assert!(updated.iter().any(|action| action.updated_files > 0), "{updated:?}");
    }

    #[test]
    fn review_routes_covers_clear_validation_retries_and_accept() {
        let catalog = BundledModelCatalog::load().unwrap();
        let review_items = super::route_review_items();
        assert_eq!(review_items.first().unwrap(), super::ACCEPT_CURRENT_ROUTES_LABEL);
        assert_eq!(review_items.last().unwrap(), super::CLEAR_ALL_ROUTES_LABEL);

        let mut missing_selections: Vec<super::GuidedRouteSelection> = Vec::new();
        let missing_slot_error = super::edit_route_selection(
            &mut ScriptedInteractor::default(),
            &catalog,
            &mut missing_selections,
            RouteSlot::Planning,
        )
        .unwrap_err();
        assert!(missing_slot_error.contains("planning"), "{missing_slot_error}");

        let custom_model_choice =
            catalog.model_labels_for_runtime(catalog.runtimes[0].runtime).len();
        let clear_choice = review_items.len() - 1;
        let mut interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![clear_choice, 1, 0, custom_model_choice, 1, 0, 0, 0]),
            inputs: VecDeque::from(vec!["   ".to_string()]),
            ..Default::default()
        };

        let routes =
            super::review_routes(&mut interactor, &catalog, &[RuntimeKind::Copilot]).unwrap();
        let planning =
            routes.iter().find(|selection| selection.slot == RouteSlot::Planning).unwrap();
        assert_eq!(planning.source, super::GuidedRouteSource::Bundled);
        assert_eq!(planning.route.as_ref().unwrap().runtime, catalog.runtimes[0].runtime);
        assert!(
            routes
                .iter()
                .filter(|selection| selection.slot != RouteSlot::Planning)
                .all(|selection| selection.route.is_none())
        );
    }

    #[test]
    fn parser_helpers_cover_remaining_route_runtime_and_binding_variants() {
        assert_eq!(super::parse_route_slot("implementation").unwrap(), RouteSlot::Implementation);
        assert_eq!(super::parse_route_slot("verification").unwrap(), RouteSlot::Verification);
        assert_eq!(super::parse_route_slot("review").unwrap(), RouteSlot::Review);

        assert_eq!(super::parse_runtime_kind("claude").unwrap(), RuntimeKind::Claude);
        assert_eq!(super::parse_runtime_kind("gemini").unwrap(), RuntimeKind::Gemini);

        assert_eq!(parse_domain_family("systems").unwrap(), DomainFamily::Systems);
        assert_eq!(
            parse_external_context_kind("design-system").unwrap(),
            ExternalContextKind::DesignSystem
        );

        assert!(super::parse_context_binding("react", false).is_err());
        assert!(super::parse_context_binding("react|design_system", false).is_err());
    }

    #[test]
    fn initial_guided_route_selections_use_bundled_defaults_without_assistants() {
        let catalog = BundledModelCatalog::load().unwrap();

        let selections = initial_guided_route_selections(&catalog, &[]);

        assert_eq!(selections.len(), 4);
        let planning =
            selections.iter().find(|selection| selection.slot == RouteSlot::Planning).unwrap();
        let implementation = selections
            .iter()
            .find(|selection| selection.slot == RouteSlot::Implementation)
            .unwrap();
        let verification =
            selections.iter().find(|selection| selection.slot == RouteSlot::Verification).unwrap();
        let review =
            selections.iter().find(|selection| selection.slot == RouteSlot::Review).unwrap();

        assert_eq!(planning.source, super::GuidedRouteSource::Bundled);
        assert_eq!(planning.route.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(planning.route.as_ref().unwrap().model, "gpt-5.4");

        assert_eq!(implementation.source, super::GuidedRouteSource::Bundled);
        assert_eq!(implementation.route.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(implementation.route.as_ref().unwrap().model, "opus-4.6");

        assert_eq!(verification.source, super::GuidedRouteSource::Bundled);
        assert_eq!(verification.route.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(verification.route.as_ref().unwrap().model, "sonnet-4.6");

        assert_eq!(review.source, super::GuidedRouteSource::Bundled);
        assert_eq!(review.route.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(review.route.as_ref().unwrap().model, "gpt-5.4");
    }

    #[test]
    fn execute_init_guided_flow_without_assistants_uses_bundled_catalog_defaults() {
        let workspace = temp_workspace("boundline-init-guided-no-assistants");

        let interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![0, 0]),
            multi_selects: VecDeque::from(vec![vec![]]),
            inputs: VecDeque::new(),
            confirms: VecDeque::from(vec![true]),
        };

        let report = execute_init(InitRequest {
            non_interactive: false,
            interactive_terminal_override: Some(true),
            interactor: Some(Box::new(interactor)),
            template: Some(InitTemplate::BugFix),
            ..base_init_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(workspace.join(".boundline/execution.json").is_file());
        assert!(workspace.join(".boundline/config.toml").is_file());

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(local.routing.assistant_runtimes, Vec::<RuntimeKind>::new());
        assert_eq!(local.routing.planning.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(local.routing.planning.as_ref().unwrap().model, "gpt-5.4");
        assert_eq!(local.routing.implementation.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(local.routing.implementation.as_ref().unwrap().model, "opus-4.6");
        assert_eq!(local.routing.verification.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(local.routing.verification.as_ref().unwrap().model, "sonnet-4.6");
        assert_eq!(local.routing.review.as_ref().unwrap().runtime, RuntimeKind::Copilot);
        assert_eq!(local.routing.review.as_ref().unwrap().model, "gpt-5.4");
    }

    #[test]
    fn execute_init_global_scope_writes_install_defaults_without_workspace_artifacts() {
        let workspace = temp_workspace("boundline-init-global-scope");
        let xdg_home = temp_workspace("boundline-init-global-xdg");

        with_global_config_env(Some(&xdg_home), None, || {
            let report = execute_init(InitRequest {
                scope: InitConfigScope::Global,
                assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
                canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
                ..base_init_request(&workspace)
            })
            .unwrap();

            assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
            assert!(report.terminal_output.contains("init: global configuration initialized"));
            assert!(report.terminal_output.contains("scope: global"));
            assert!(report.terminal_output.contains("global_config:"));
            assert!(
                report.terminal_output.contains("workspace_artifacts: skipped in global scope")
            );
            assert!(!workspace.join(".boundline/config.toml").exists());
            assert!(!workspace.join(".boundline/execution.json").exists());

            let saved = FileConfigStore::load_global().unwrap().unwrap();
            assert_eq!(saved.routing.planning.as_ref().unwrap().runtime, RuntimeKind::Copilot);
            assert!(FileConfigStore::global_config_path().is_file());
        });
    }

    #[test]
    fn execute_init_both_scope_writes_global_defaults_and_workspace_overrides() {
        let workspace = temp_workspace("boundline-init-both-scope");
        let xdg_home = temp_workspace("boundline-init-both-xdg");

        with_global_config_env(Some(&xdg_home), None, || {
            let report = execute_init(InitRequest {
                scope: InitConfigScope::Both,
                template: Some(InitTemplate::Change),
                assistants: &[crate::domain::configuration::AssistantHostKind::Copilot],
                canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
                ..base_init_request(&workspace)
            })
            .unwrap();

            assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
            assert!(report.terminal_output.contains("scope: both"));
            assert!(report.terminal_output.contains("global_config:"));
            assert!(report.terminal_output.contains("workspace_config:"));
            assert!(workspace.join(".boundline/config.toml").is_file());
            assert!(workspace.join(".boundline/execution.json").is_file());
            assert!(FileConfigStore::global_config_path().is_file());

            let global = FileConfigStore::load_global().unwrap().unwrap();
            let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
            assert_eq!(global.routing.planning, local.routing.planning);
            assert_eq!(global.routing.review, local.routing.review);
        });
    }

    #[test]
    fn scripted_interactor_reports_missing_values_for_each_prompt_type() {
        let mut interactor = ScriptedInteractor::default();

        let select_error = interactor.select("prompt", &[], 0).unwrap_err();
        assert!(select_error.to_string().contains("missing scripted select"));

        let multi_select_error = interactor.multi_select("prompt", &[], &[]).unwrap_err();
        assert!(multi_select_error.to_string().contains("missing scripted multi-select"));

        let input_error = interactor.input("prompt", "").unwrap_err();
        assert!(input_error.to_string().contains("missing scripted input"));

        let confirm_error = interactor.confirm("prompt", false).unwrap_err();
        assert!(confirm_error.to_string().contains("missing scripted confirm"));
    }

    #[test]
    fn run_init_activity_covers_interactive_failure_path() {
        let result: Result<(), _> = super::run_init_activity("failing activity", true, || {
            Err(super::InitCommandError::InvalidBundledCatalog("broken catalog".to_string()))
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bundled model catalog"));

        let non_interactive_result: Result<(), _> =
            super::run_init_activity("failing activity", false, || {
                Err(super::InitCommandError::InvalidBundledCatalog("broken catalog".to_string()))
            });
        assert!(non_interactive_result.is_err());
    }

    #[test]
    fn render_guided_summary_covers_empty_and_nonempty_assistants() {
        let catalog = BundledModelCatalog::load().unwrap();
        let slots = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);

        let with_assistants = super::render_guided_summary(
            InitConfigScope::Workspace,
            InitTemplate::Change,
            Some(CanonModeSelectionPreference::AutoConfirm),
            &[crate::domain::configuration::AssistantHostKind::Copilot],
            &slots,
            &catalog,
            &["- create .boundline/config.toml".to_string()],
        );
        assert!(with_assistants.contains("copilot"), "{with_assistants}");
        assert!(with_assistants.contains("Model routes:"), "{with_assistants}");

        let no_assistants = super::render_guided_summary(
            InitConfigScope::Workspace,
            InitTemplate::BugFix,
            None,
            &[],
            &slots,
            &catalog,
            &[],
        );
        assert!(no_assistants.contains("none selected"), "{no_assistants}");
        assert!(no_assistants.contains("auto-confirm"), "{no_assistants}");
    }

    #[test]
    fn render_cancelled_init_report_covers_empty_and_nonempty_assistants() {
        let workspace = temp_workspace("boundline-init-cancel-render");
        let catalog = BundledModelCatalog::load().unwrap();
        let slots = initial_guided_route_selections(&catalog, &[RuntimeKind::Copilot]);

        let with = super::render_cancelled_init_report(
            InitConfigScope::Workspace,
            Some(workspace.as_path()),
            InitTemplate::Delivery,
            Some(CanonModeSelectionPreference::Manual),
            &[crate::domain::configuration::AssistantHostKind::Copilot],
            &slots,
            &catalog,
        );
        assert!(with.contains("canceled before write"), "{with}");
        assert!(with.contains("copilot"), "{with}");

        let without = super::render_cancelled_init_report(
            InitConfigScope::Workspace,
            Some(workspace.as_path()),
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
            interactive_terminal_override: Some(true),
            ..copilot_canon_init_request(&workspace, InitTemplate::Change)
        })
        .unwrap();
        assert_eq!(report.exit_status, crate::cli::CommandExitStatus::Succeeded);
        assert!(
            report.terminal_output.contains("init: workspace initialized"),
            "{}",
            report.terminal_output
        );
        assert!(
            report.terminal_output.contains("assistant_package_scope: repo-local"),
            "{}",
            report.terminal_output
        );
        assert!(
            report
                .terminal_output
                .contains("assistant_global_bootstrap: use `boundline assistant install --host <host> --scope user` before workspace init"),
            "{}",
            report.terminal_output
        );
        assert!(report.compact_output.contains("latest_status: succeeded"));
    }

    #[test]
    fn init_command_report_new_adds_compact_brief() {
        let report = super::InitCommandReport::new(
            CommandExitStatus::NonSuccess,
            concat!(
                "init: blocked - Canon surface not ready\n",
                "scope: workspace\n",
                "template: change\n",
                "workspace_config: /tmp/workspace/.boundline/config.toml\n",
                "execution_profile: /tmp/workspace/.boundline/execution.json\n",
                "canon_mode_selection: auto-confirm\n",
                "canon_bootstrap: blocked\n",
                "canon_surface: Canon binary missing\n",
                "assistant_setup: none\n",
                "workspace_hygiene: none\n",
                "next_steps:\n",
                "- verify workspace: boundline doctor --workspace /tmp/workspace\n",
                "- verify install: boundline doctor --install\n"
            ),
        );

        assert!(report.compact_output.contains("init: blocked - Canon surface not ready"));
        assert!(report.compact_output.contains("scope: workspace"));
        assert!(report.compact_output.contains("template: change"));
        assert!(report.compact_output.contains("summary: canon_mode_selection=auto-confirm; assistant_setup=none; workspace_hygiene=none"), "{}", report.compact_output);
        assert!(report.compact_output.contains("artifacts: execution_profile=/tmp/workspace/.boundline/execution.json; workspace_config=/tmp/workspace/.boundline/config.toml"), "{}", report.compact_output);
        assert!(report.compact_output.contains("governance: canon_mode_selection=auto-confirm; canon_bootstrap=blocked; canon_surface=Canon binary missing"), "{}", report.compact_output);
        assert!(report.compact_output.contains("latest_status: blocked"));
        assert!(
            report.compact_output.contains(
                "next_command: verify workspace: boundline doctor --workspace /tmp/workspace"
            ),
            "{}",
            report.compact_output
        );
        assert!(!report.compact_output.contains("verify install"), "{}", report.compact_output);
    }

    #[test]
    fn init_command_error_new_variants_display_their_messages() {
        let unavailable = super::InitCommandError::InteractiveTerminalUnavailable;
        assert!(unavailable.to_string().contains("non-interactive"), "{}", unavailable);

        let cwd_unavailable = super::InitCommandError::CurrentDirectoryUnavailable {
            workspace: PathBuf::from("."),
            source: std::io::Error::from(std::io::ErrorKind::NotFound),
        };
        assert!(cwd_unavailable.to_string().contains("current working directory is unavailable"));

        let invalid_catalog =
            super::InitCommandError::InvalidBundledCatalog("bad toml".to_string());
        assert!(
            invalid_catalog.to_string().contains("bundled model catalog"),
            "{}",
            invalid_catalog
        );

        let prompt_err = super::InitCommandError::PromptInteraction("user cancelled".to_string());
        assert!(prompt_err.to_string().contains("collect init input"), "{}", prompt_err);

        let docs_arg = super::InitCommandError::InvalidDocsExportArgument(
            "requires --export-docs".to_string(),
        );
        assert!(docs_arg.to_string().contains("docs export argument"), "{}", docs_arg);
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

    #[test]
    fn execute_init_guided_flow_via_injected_interactor_confirm_true() {
        let workspace = temp_workspace("boundline-init-guided-confirm");
        let catalog = BundledModelCatalog::load().unwrap();
        let copilot_models = catalog.model_labels_for_runtime(RuntimeKind::Copilot).len();

        // Scripted answers:
        //   select_canon_mode          → 0 (AutoConfirm)
        //   select_assistants          → [0] (Claude, index 0 in INIT_ASSISTANT_HOSTS)
        //   review_routes accept       → 0 (Accept current routes)
        //   confirm summary            → true (write)
        let interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![0, 0]),
            multi_selects: VecDeque::from(vec![vec![0]]),
            inputs: VecDeque::new(),
            confirms: VecDeque::from(vec![true]),
        };

        let report = execute_init(InitRequest {
            non_interactive: false,
            interactive_terminal_override: Some(true),
            interactor: Some(Box::new(interactor)),
            template: Some(InitTemplate::BugFix),
            ..base_init_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            report.terminal_output.contains("init: workspace initialized"),
            "{}",
            report.terminal_output
        );
        assert!(report.terminal_output.contains("claude"), "{}", report.terminal_output);
        // Suppress unused variable warning — copilot_models is needed to check catalog health
        let _ = copilot_models;
    }

    #[test]
    fn execute_init_reuses_existing_workspace_defaults_without_prompting() {
        let workspace = temp_workspace("boundline-init-reuse-workspace-defaults");
        let store = FileConfigStore::for_workspace(&workspace);
        let mut existing = ConfigFile::default();
        existing.routing.planning =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() });
        existing.routing.implementation =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "opus-4.6".to_string() });
        existing.routing.verification =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "sonnet-4.6".to_string() });
        existing.routing.review =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() });
        existing.canon = Some(CanonPreferences {
            mode_selection: CanonModeSelectionPreference::AutoConfirm,
            default_risk: None,
            default_zone: None,
            default_owner: None,
            default_system_context: None,
        });
        store.save_local(&existing).unwrap();

        let report = execute_init(InitRequest {
            non_interactive: false,
            interactive_terminal_override: Some(true),
            interactor: Some(Box::new(ScriptedInteractor::default())),
            template: Some(InitTemplate::BugFix),
            ..base_init_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("canon_bootstrap: ready"));
        assert!(report.terminal_output.contains("canon_workspace_bootstrap:"));
        assert!(workspace.join(".canon").is_dir());
        assert!(workspace.join(".agents/skills").is_dir());

        let local = store.load_local().unwrap().unwrap();
        assert_eq!(local.canon.unwrap().mode_selection, CanonModeSelectionPreference::AutoConfirm);

        let execution_profile =
            fs::read_to_string(workspace.join(".boundline/execution.json")).unwrap();
        assert!(execution_profile.contains("\"governance\""), "{execution_profile}");
        assert!(
            execution_profile.contains("\"default_runtime\": \"canon\""),
            "{execution_profile}"
        );
    }

    #[test]
    fn execute_init_blocks_canon_bootstrap_for_nested_git_workspace() {
        let repo_root = temp_workspace("boundline-init-nested-canon-root");
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let workspace = repo_root.join("tmp");
        fs::create_dir_all(&workspace).unwrap();

        let report =
            execute_init(copilot_canon_init_request(&workspace, InitTemplate::BugFix)).unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("Canon workspace bootstrap failed"));
        assert!(report.terminal_output.contains(repo_root.display().to_string().as_str()));
        assert_init_workspace_artifacts_absent(&workspace);
    }

    #[test]
    fn execute_init_with_dot_workspace_does_not_promote_nested_git_directory_to_repo_root() {
        let repo_root = temp_workspace("boundline-init-dot-nested-execute-root");
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let workspace = repo_root.join("tmp");
        fs::create_dir_all(&workspace).unwrap();
        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);

        let report =
            execute_init(copilot_canon_init_request(Path::new("."), InitTemplate::BugFix)).unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("Canon workspace bootstrap failed"));
        assert!(report.terminal_output.contains(repo_root.display().to_string().as_str()));
        assert!(report.terminal_output.contains(workspace.display().to_string().as_str()));
        assert_init_workspace_artifacts_absent(&repo_root);
        assert_init_workspace_artifacts_absent(&workspace);
    }

    #[test]
    fn resolve_workspace_root_keeps_dot_in_nested_git_directory() {
        let repo_root = temp_workspace("boundline-init-dot-nested-root");
        fs::create_dir_all(repo_root.join(".git")).unwrap();
        let workspace = repo_root.join("tmp");
        fs::create_dir_all(&workspace).unwrap();
        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);

        let resolved = resolve_workspace_root(Path::new(".")).unwrap();

        assert_eq!(resolved, workspace.canonicalize().unwrap());
    }

    #[test]
    fn resolve_workspace_root_uses_pwd_when_current_directory_is_unavailable() {
        let fallback_workspace = temp_workspace("boundline-init-pwd-fallback");
        let broken_workspace = temp_workspace("boundline-init-broken-cwd");
        let _current_dir_guard = CurrentDirGuard::change_to(&broken_workspace);
        fs::remove_dir_all(&broken_workspace).unwrap();
        let _pwd_guard = PwdEnvGuard::set(&fallback_workspace);

        let resolved = resolve_workspace_root(Path::new(".")).unwrap();

        let expected = fallback_workspace.canonicalize().unwrap_or(fallback_workspace);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_workspace_root_reports_unavailable_current_directory_without_valid_pwd() {
        let broken_workspace = temp_workspace("boundline-init-broken-cwd-error");
        let _current_dir_guard = CurrentDirGuard::change_to(&broken_workspace);
        fs::remove_dir_all(&broken_workspace).unwrap();
        let _pwd_guard = PwdEnvGuard::remove();

        let error = resolve_workspace_root(Path::new(".")).unwrap_err();

        match error {
            InitCommandError::CurrentDirectoryUnavailable { workspace, .. } => {
                assert_eq!(workspace, PathBuf::from("."));
            }
            other => panic!("expected current-directory error, got {other:?}"),
        }
    }

    #[test]
    fn execute_init_guided_flow_with_dot_workspace_survives_current_dir_change() {
        let workspace = temp_workspace("boundline-init-guided-dot");
        let diverted_workspace = temp_workspace("boundline-init-guided-diverted");
        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);

        let interactor = CurrentDirChangingInteractor {
            inner: ScriptedInteractor {
                selects: VecDeque::from(vec![0, 0]),
                multi_selects: VecDeque::from(vec![vec![]]),
                inputs: VecDeque::new(),
                confirms: VecDeque::from(vec![true]),
            },
            confirm_dir: diverted_workspace.clone(),
        };

        let report = execute_init(InitRequest {
            non_interactive: false,
            interactive_terminal_override: Some(true),
            interactor: Some(Box::new(interactor)),
            template: Some(InitTemplate::BugFix),
            ..base_init_request(Path::new("."))
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(workspace.join(".boundline/execution.json").is_file());
        assert!(workspace.join(".boundline/config.toml").is_file());
        assert!(!diverted_workspace.join(".boundline/execution.json").exists());
        assert!(!diverted_workspace.join(".boundline/config.toml").exists());
    }

    #[test]
    fn execute_init_guided_flow_via_injected_interactor_confirm_false_cancels() {
        let workspace = temp_workspace("boundline-init-guided-cancel");

        // Same guided flow but confirm → false (cancel before write)
        let interactor = ScriptedInteractor {
            selects: VecDeque::from(vec![0, 0]),
            multi_selects: VecDeque::from(vec![vec![0]]),
            inputs: VecDeque::new(),
            confirms: VecDeque::from(vec![false]),
        };

        let report = execute_init(InitRequest {
            non_interactive: false,
            interactive_terminal_override: Some(true),
            interactor: Some(Box::new(interactor)),
            template: Some(InitTemplate::BugFix),
            ..base_init_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            report.terminal_output.contains("canceled before write"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn execute_update_preview_reports_changes_without_writing_files() {
        let workspace = temp_workspace("boundline-update-preview");
        initialize_workspace_for_update(&workspace);

        let config_path = workspace.join(".boundline/config.toml");
        let original = fs::read_to_string(&config_path).unwrap();
        fs::write(&config_path, format!("{original}\n# local drift\n")).unwrap();

        let report = execute_update(base_update_request(&workspace)).unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            report
                .terminal_output
                .contains("update: preview only - workspace-managed scaffold changes detected"),
            "{}",
            report.terminal_output
        );
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            format!("{original}\n# local drift\n")
        );
    }

    #[test]
    fn execute_update_apply_requires_force_for_replace_owned_changes() {
        let workspace = temp_workspace("boundline-update-force");
        initialize_workspace_for_update(&workspace);

        let config_path = workspace.join(".boundline/config.toml");
        let original = fs::read_to_string(&config_path).unwrap();
        fs::write(&config_path, format!("{original}\n# local drift\n")).unwrap();

        let report =
            execute_update(UpdateRequest { apply: true, ..base_update_request(&workspace) })
                .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("require --force"), "{}", report.terminal_output);
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            format!("{original}\n# local drift\n")
        );
    }

    #[test]
    fn execute_update_execution_target_requires_template() {
        let workspace = temp_workspace("boundline-update-execution-template");
        initialize_workspace_for_update(&workspace);
        fs::remove_file(workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME))
            .unwrap();
        fs::write(workspace.join(BOUNDLINE_DIR_RELATIVE).join(EXECUTION_PROFILE_FILE_NAME), "{}")
            .unwrap();

        let error = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Execution],
            ..base_update_request(&workspace)
        })
        .unwrap_err();

        assert!(matches!(error, InitCommandError::UpdateExecutionTemplateRequired));
    }

    fn assert_workspace_project_doc_roots_exist(workspace: &Path) {
        assert!(workspace.join("docs/project").is_dir());
        assert!(workspace.join("docs/evidence").is_dir());
        assert!(workspace.join("docs/project/README.md").is_file());
        assert!(workspace.join("docs/evidence/README.md").is_file());
    }

    fn assert_init_workspace_artifacts_absent(workspace: &Path) {
        assert!(!workspace.join(".boundline/execution.json").exists());
        assert!(!workspace.join(".boundline/config.toml").exists());
        assert!(!workspace.join(".env.template").exists());
        assert!(!workspace.join(".canon").exists());
    }

    #[test]
    fn execute_init_writes_workspace_scaffold_manifest() {
        let workspace = temp_workspace("boundline-init-manifest");

        let report =
            execute_init(copilot_canon_init_request(&workspace, InitTemplate::Change)).unwrap();

        let manifest = load_workspace_scaffold_manifest(&workspace);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert_eq!(manifest.version, SCAFFOLD_MANIFEST_VERSION);
        assert_eq!(manifest.scaffold_epoch, INITIAL_SCAFFOLD_EPOCH);
        assert_eq!(manifest.boundline_version, BOUNDLINE_VERSION);
        assert_eq!(manifest.workspace_template, Some(InitTemplate::Change));
        assert!(manifest.entries.iter().any(|entry| entry.path == ".boundline/config.toml"));
        assert!(manifest.entries.iter().any(|entry| entry.path == ".boundline/execution.json"));
        assert!(manifest.entries.iter().any(|entry| entry.path == "assistant/README.md"));
        assert_workspace_project_doc_roots_exist(&workspace);
        assert!(report.terminal_output.contains("project_memory_root: docs/project"));
        assert!(report.terminal_output.contains("evidence_root: docs/evidence"));
    }

    #[test]
    fn execute_update_recreates_project_doc_roots_when_missing() {
        let workspace = temp_workspace("boundline-update-project-doc-roots");
        initialize_workspace_for_update(&workspace);
        fs::remove_dir_all(workspace.join("docs/project")).unwrap();
        fs::remove_dir_all(workspace.join("docs/evidence")).unwrap();

        let report =
            execute_update(UpdateRequest { apply: true, ..base_update_request(&workspace) })
                .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert_workspace_project_doc_roots_exist(&workspace);
    }

    #[test]
    fn ensure_workspace_project_doc_roots_preserves_existing_readmes() {
        let workspace = temp_workspace("boundline-project-doc-roots-existing-readmes");
        let project_root = workspace.join("docs/project");
        let evidence_root = workspace.join("docs/evidence");
        fs::create_dir_all(&project_root).unwrap();
        fs::create_dir_all(&evidence_root).unwrap();
        let project_readme = project_root.join("README.md");
        let evidence_readme = evidence_root.join("README.md");
        fs::write(&project_readme, "# Existing Project Memory\n").unwrap();
        fs::write(&evidence_readme, "# Existing Evidence\n").unwrap();

        let doc_roots = ensure_workspace_project_doc_roots(&workspace).unwrap();

        assert_eq!(doc_roots.project_memory_dir(&workspace), project_root);
        assert_eq!(doc_roots.evidence_dir(&workspace), evidence_root);
        assert_eq!(fs::read_to_string(&project_readme).unwrap(), "# Existing Project Memory\n");
        assert_eq!(fs::read_to_string(&evidence_readme).unwrap(), "# Existing Evidence\n");
    }

    #[test]
    fn execute_update_recreates_scaffold_manifest_without_force() {
        let workspace = temp_workspace("boundline-update-manifest-recreate");
        initialize_workspace_for_update(&workspace);
        let manifest_path =
            workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME);
        fs::remove_file(&manifest_path).unwrap();

        let preview = execute_update(base_update_request(&workspace)).unwrap();
        assert!(
            preview.terminal_output.contains(SCAFFOLD_MANIFEST_FILE_NAME),
            "{}",
            preview.terminal_output
        );

        let report =
            execute_update(UpdateRequest { apply: true, ..base_update_request(&workspace) })
                .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(manifest_path.is_file());
    }

    #[test]
    fn execute_update_infers_execution_template_from_existing_profile() {
        let workspace = temp_workspace("boundline-update-execution-inference");
        execute_init(copilot_canon_init_request(&workspace, InitTemplate::Delivery)).unwrap();
        fs::remove_file(workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME))
            .unwrap();

        let report = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Execution],
            apply: true,
            ..base_update_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            load_workspace_scaffold_manifest(&workspace).workspace_template
                == Some(InitTemplate::Delivery)
        );
    }

    #[test]
    fn execute_update_applies_execution_refresh_when_template_is_provided() {
        let workspace = temp_workspace("boundline-update-execution-apply");
        initialize_workspace_for_update(&workspace);

        let config = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        let expected = serde_json::to_string_pretty(&execution_template(
            InitTemplate::Change,
            config.canon.as_ref(),
        ))
        .unwrap();

        let report = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Execution],
            template: Some(InitTemplate::Change),
            apply: true,
            force: true,
            ..base_update_request(&workspace)
        })
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert_eq!(
            fs::read_to_string(workspace.join(".boundline/execution.json")).unwrap(),
            expected
        );
    }

    #[test]
    fn execute_update_apply_requires_adopt_for_untracked_divergent_replace_owned_file() {
        let workspace = temp_workspace("boundline-update-adopt-required");
        initialize_workspace_for_update(&workspace);

        let config_path = workspace.join(".boundline/config.toml");
        let original = fs::read_to_string(&config_path).unwrap();
        let drifted = format!("{original}\n# local drift\n");
        fs::write(&config_path, &drifted).unwrap();
        fs::remove_file(workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME))
            .unwrap();

        let preview = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Config],
            diff: true,
            ..base_update_request(&workspace)
        })
        .unwrap();
        assert!(
            preview.terminal_output.contains("[conflict] .boundline/config.toml"),
            "{}",
            preview.terminal_output
        );

        let blocked = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Config],
            apply: true,
            ..base_update_request(&workspace)
        })
        .unwrap();
        assert_eq!(blocked.exit_status, CommandExitStatus::NonSuccess);
        assert!(blocked.terminal_output.contains("require --adopt"), "{}", blocked.terminal_output);
        assert_eq!(fs::read_to_string(&config_path).unwrap(), drifted);
    }

    #[test]
    fn execute_update_adopt_force_baselines_untracked_divergent_replace_owned_file() {
        let workspace = temp_workspace("boundline-update-adopt-force");
        initialize_workspace_for_update(&workspace);

        let config_path = workspace.join(".boundline/config.toml");
        let original = fs::read_to_string(&config_path).unwrap();
        let drifted = format!("{original}\n# local drift\n");
        fs::write(&config_path, &drifted).unwrap();
        fs::remove_file(workspace.join(BOUNDLINE_DIR_RELATIVE).join(SCAFFOLD_MANIFEST_FILE_NAME))
            .unwrap();

        let report = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Config],
            apply: true,
            adopt: true,
            force: true,
            ..base_update_request(&workspace)
        })
        .unwrap();

        let manifest = load_workspace_scaffold_manifest(&workspace);
        let config_entry =
            manifest.entries.iter().find(|entry| entry.path == ".boundline/config.toml").unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert_eq!(fs::read_to_string(&config_path).unwrap(), drifted);
        assert_eq!(
            config_entry.fingerprint,
            crate::domain::scaffold_manifest::fingerprint_text(&drifted)
        );
    }

    #[test]
    fn execute_update_status_and_prune_surface_orphaned_managed_artifacts() {
        let workspace = temp_workspace("boundline-update-orphans");
        initialize_workspace_for_update(&workspace);

        let store = FileConfigStore::for_workspace(&workspace);
        let mut config = store.load_local().unwrap().unwrap();
        config.routing.assistant_runtimes.clear();
        config.routing.assistant_hosts.clear();
        store.save_local(&config).unwrap();

        let status = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Assistant],
            status: true,
            ..base_update_request(&workspace)
        })
        .unwrap();
        assert!(
            status.terminal_output.contains("update_status: orphaned"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("assistant/README.md"),
            "{}",
            status.terminal_output
        );

        let report = execute_update(UpdateRequest {
            targets: &[UpdateTarget::Assistant],
            apply: true,
            prune: true,
            ..base_update_request(&workspace)
        })
        .unwrap();

        let manifest = load_workspace_scaffold_manifest(&workspace);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(!workspace.join("assistant/README.md").exists());
        assert!(!manifest.entries.iter().any(|entry| entry.path == "assistant/README.md"));
    }
}
