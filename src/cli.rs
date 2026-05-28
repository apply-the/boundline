//! Root CLI command surface and invocation-session bookkeeping.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand};

use crate::adapters::env_layer;
use crate::domain::configuration::{
    AssistantHostKind, CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy,
    EffortLevel, IdeKind, InitConfigScope, InitTemplate, RouteSlot, RuntimeKind,
    SemanticAccelerationPolicyState, TerminalAutoApproveProfile,
};
use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};
use crate::domain::governance::{CanonMode, CanonModeSelectionPreference, GovernanceRuntimeKind};
use crate::domain::trace::current_timestamp_millis;

use super::{
    assistant_assets, checkpoint, cluster, config, diagnostics, govern, init, inspect, models_auth,
    orchestrate, output, run, session, workflow, workspace as cli_workspace,
};

/// Top-level CLI parser for the Boundline executable.
#[derive(Debug, Parser)]
#[command(
    name = "boundline",
    about = "Local delivery orchestrator for bounded engineering work",
    version
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit structured JSON host output while preserving the rendered text inside the payload"
    )]
    pub json: bool,

    #[arg(
        long,
        global = true,
        help = "Reopen detailed human-readable command output without changing JSON payloads"
    )]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<DeveloperCommand>,
}

/// Stable command names used in output rendering and session tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Doctor,
    Checkpoint,
    Orchestrate,
    Run,
    Workflow,
    Inspect,
    Goal,
    Flow,
    Plan,
    Step,
    Status,
    Next,
    Continue,
    Session,
    Govern,
    Init,
    Update,
    Assistant,
    Config,
    Cluster,
    Models,
}

impl CommandName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Checkpoint => "checkpoint",
            Self::Orchestrate => "orchestrate",
            Self::Run => "run",
            Self::Workflow => "workflow",
            Self::Inspect => "inspect",
            Self::Goal => "goal",
            Self::Flow => "flow",
            Self::Plan => "plan",
            Self::Step => "step",
            Self::Status => "status",
            Self::Next => "next",
            Self::Continue => "continue",
            Self::Session => "session",
            Self::Govern => "govern",
            Self::Init => "init",
            Self::Update => "update",
            Self::Assistant => "assistant",
            Self::Config => "config",
            Self::Cluster => "cluster",
            Self::Models => "models",
        }
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Exit-status classification used by rendered host output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExitStatus {
    Succeeded,
    NonSuccess,
    InvalidInvocation,
    TraceReadFailure,
}

/// Top-level developer commands exposed by the CLI.
#[derive(Debug, Subcommand)]
pub enum DeveloperCommand {
    Doctor {
        #[arg(long, conflicts_with = "install", required_unless_present = "install")]
        workspace: Option<PathBuf>,
        #[arg(long, conflicts_with = "workspace")]
        install: bool,
    },
    Goal {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long, conflicts_with = "new_session")]
        update: bool,
        /// Force creation of a new session even if an active non-terminal session exists.
        #[arg(long = "new", conflicts_with = "update")]
        new_session: bool,
        #[arg(long)]
        goal: Option<String>,
        /// One or more Markdown brief files (.md or .markdown) inside the workspace.
        #[arg(long = "brief")]
        brief: Vec<PathBuf>,
        #[arg(long = "governance")]
        governance: Option<GovernanceRuntimeKind>,
        #[arg(long)]
        risk: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        /// Semantic 2-4 word kebab-case session identifier derived by the AI (e.g. `rust-user-service`).
        #[arg(long)]
        slug: Option<String>,
    },
    Flow {
        name: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Plan {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long, value_name = "PATH")]
        input: Option<PathBuf>,
        #[arg(long, conflicts_with = "no_flow")]
        flow: Option<String>,
        #[arg(long, conflicts_with = "flow")]
        #[arg(long = "no-flow")]
        no_flow: bool,
        #[arg(long = "no-canon")]
        no_canon: bool,
    },
    Step {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        goal: Option<String>,
        #[arg(long)]
        compatibility: bool,
        /// One or more Markdown brief files (.md or .markdown) inside the workspace.
        #[arg(long = "brief")]
        brief: Vec<PathBuf>,
        #[arg(long = "governance")]
        governance: Option<GovernanceRuntimeKind>,
        #[arg(long)]
        risk: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        /// Explicit Canon mode to use for governed execution.
        #[arg(long = "mode", value_enum)]
        mode: Option<CanonMode>,
        /// Opt out of Canon governance even when workspace has [canon] config.
        #[arg(long = "no-canon", conflicts_with = "mode")]
        no_canon: bool,
    },
    Orchestrate {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        goal: Option<String>,
        /// One or more Markdown brief files (.md or .markdown) inside the workspace.
        #[arg(long = "brief")]
        brief: Vec<PathBuf>,
        #[arg(long)]
        flow: Option<String>,
        #[arg(long = "governance")]
        governance: Option<GovernanceRuntimeKind>,
        #[arg(long)]
        risk: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(
            long = "intent",
            visible_alias = "until",
            value_enum,
            default_value_t = orchestrate::OrchestrateIntent::ContinueUntilPhaseRequest
        )]
        intent: orchestrate::OrchestrateIntent,
        #[arg(long = "planning-stage-complete")]
        planning_stage_complete: Option<String>,
        #[arg(long = "request-id")]
        request_id: Option<String>,
        #[arg(long = "answer")]
        answer: Option<String>,
        #[arg(long = "assistant-host", value_enum)]
        assistant_host: Option<assistant_assets::AssistantHost>,
        #[arg(long = "json-stream")]
        json_stream: bool,
        #[arg(long = "no-canon")]
        no_canon: bool,
        /// Semantic 2-4 word kebab-case session identifier derived by the AI (e.g. `rust-user-service`).
        #[arg(long)]
        slug: Option<String>,
    },
    Workflow {
        #[command(subcommand)]
        command: WorkflowSubcommand,
    },
    Checkpoint {
        #[command(subcommand)]
        command: CheckpointSubcommand,
    },
    Inspect {
        #[arg(long)]
        trace: Option<PathBuf>,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        audit: bool,
    },
    Status {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
    },
    Next {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
    },
    Continue {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
    },
    Session {
        #[command(subcommand)]
        command: SessionSubcommand,
    },
    Govern {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long = "mode", value_enum)]
        mode: Option<CanonMode>,
        #[arg(long)]
        goal: Option<String>,
        #[arg(long = "brief")]
        brief: Vec<PathBuf>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        head: Option<String>,
        #[arg(long)]
        risk: Option<String>,
        #[arg(long = "structural-impact")]
        structural_impact: bool,
        #[arg(long = "public-contract-change")]
        public_contract_change: bool,
        #[arg(long = "validation-exhausted")]
        validation_exhausted: bool,
        #[arg(long = "pr-ready")]
        pr_ready: bool,
        #[arg(long = "preserved-behavior-evidence")]
        preserved_behavior_evidence: bool,
    },
    Assistant {
        #[command(subcommand)]
        command: AssistantSubcommand,
    },
    #[command(
        about = "Bootstrap install-global defaults, workspace files, and default routing",
        after_long_help = "Guided mode tips:\n  - leave --assistant unset to skip repository-local assistant packs\n  - leave guided routes blank to let selected assistants seed defaults for planning, implementation, verification, and review\n\nScope selection:\n  - --scope workspace keeps the existing repo bootstrap behavior\n  - --scope global writes install-wide defaults under the global Boundline config directory\n  - --scope both writes install-wide defaults first and then repo-local overrides\n\nWorkspace selection:\n  - omit --workspace to target the nearest initialized .boundline/ root\n  - if no .boundline/ exists, Boundline falls back to the nearest .git root\n  - use --workspace <path> only when you need to bootstrap another repository explicitly\n\nDocs export policy:\n  - --export-docs is create-only by default; existing target files stop the command\n  - use --refresh to update generated docs in place\n  - use --diff to preview docs changes without writing\n  - use --to <path> to export generated docs under another root\n\nExamples:\n  boundline init --assistant copilot\n  boundline init --scope global --assistant copilot\n  boundline init --scope both --assistant codex --assistant copilot\n  boundline init --assistant copilot --route planning=copilot:gpt-4o\n  boundline init --assistant codex --assistant copilot --route review=claude:sonnet-4\n  boundline init --export-docs\n  boundline init --export-docs --refresh\n  boundline init --workspace ../other-repo --export-docs --to docs/reference/boundline"
    )]
    Init {
        /// Configuration scope to bootstrap. `workspace` preserves existing behavior, `global` writes install-wide defaults, and `both` writes both layers.
        #[arg(long, value_enum, default_value_t = InitConfigScope::Workspace)]
        scope: InitConfigScope,
        /// Workspace directory to bootstrap. Omit it, or pass `.` to target the nearest `.boundline/` root, then the nearest `.git/` root, then the current directory. Use this flag to bootstrap a different repository explicitly.
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
        /// Disable guided terminal prompts and require explicit flag-driven input only.
        #[arg(long = "non-interactive")]
        non_interactive: bool,
        /// Optional starting template for the generated execution profile. Defaults to bug-fix.
        #[arg(long)]
        template: Option<InitTemplate>,
        /// Assistant package hosts to scaffold locally. Supported values: claude, codex, copilot, antigravity. Provider-backed hosts can also seed default routes.
        #[arg(long = "assistant")]
        assistant: Vec<AssistantHostKind>,
        /// Model route in SLOT=RUNTIME:MODEL form. Supported slots: planning, implementation, verification, review. Example: planning=copilot:gpt-4o. Leave blank in guided mode to let selected assistants seed defaults.
        #[arg(long = "route")]
        route: Vec<String>,
        /// Domain families to enable during init. When omitted, Boundline infers a bounded default from the workspace.
        #[arg(long = "domain")]
        domain: Vec<DomainFamily>,
        /// Scoped domain standards using FAMILY=TEXT.
        #[arg(long = "domain-standard")]
        domain_standard: Vec<String>,
        /// Optional external context bindings using FAMILY|KIND|REFERENCE.
        #[arg(long = "context-binding")]
        context_binding: Vec<String>,
        /// Required external context bindings using FAMILY|KIND|REFERENCE.
        #[arg(long = "required-context-binding")]
        required_context_binding: Vec<String>,
        /// Canon mode-selection preference to write to the workspace config.
        #[arg(long = "canon-mode-selection", value_enum)]
        canon_mode_selection: Option<CanonModeSelectionPreference>,
        /// Default Canon governance risk.
        #[arg(long)]
        risk: Option<String>,
        /// Default Canon governance zone.
        #[arg(long)]
        zone: Option<String>,
        /// Default Canon governance owner.
        #[arg(long)]
        owner: Option<String>,
        /// IDE setup surfaces to scaffold. Supported values: vscode, cursor, antigravity, jetbrains.
        #[arg(long = "ide")]
        ide: Vec<IdeKind>,
        /// Terminal auto-approval profile for IDEs with a stable settings schema.
        #[arg(long = "auto-approve", requires = "ide")]
        auto_approve: Option<TerminalAutoApproveProfile>,
        /// Export stable repo-local Canon and assistant reference docs under docs/boundline/.
        #[arg(long = "export-docs")]
        export_docs: bool,
        /// Refresh generated repo-local docs in place.
        #[arg(long, requires = "export_docs", conflicts_with = "diff")]
        refresh: bool,
        /// Show generated repo-local docs changes without writing files.
        #[arg(long, requires = "export_docs", conflicts_with = "refresh")]
        diff: bool,
        /// Export generated repo-local docs under a custom root instead of docs/boundline/.
        #[arg(long = "to", value_name = "PATH", requires = "export_docs")]
        to: Option<PathBuf>,
        /// Replace existing Boundline files in the workspace.
        #[arg(long)]
        force: bool,
    },
    #[command(
        about = "Preview or apply updates to the current Boundline-managed workspace scaffold",
        after_long_help = "Default behavior:\n  - `boundline update` is preview-only and does not write files\n  - rerun with `--apply` to mutate the workspace\n  - rerun with `--force --apply` to overwrite changed replace-owned scaffold files\n\nTargets:\n  - omit `--target` to refresh the default workspace-managed surfaces: config, assistant assets, and hygiene files\n  - add `--target docs` to refresh exported docs under docs/boundline/ when they are already present or when you want them created\n  - add `--target execution --template <template>` to refresh `.boundline/execution.json` from an explicit template\n\nExamples:\n  boundline update\n  boundline update --apply\n  boundline update --force --apply\n  boundline update --target docs\n  boundline update --target execution --template change --apply"
    )]
    Update {
        /// Workspace directory to update. Omit it, or pass `.` to target the nearest `.boundline/` root, then the nearest `.git/` root, then the current directory.
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
        /// Restrict the update to one or more managed scaffold surfaces.
        #[arg(long = "target", value_enum)]
        target: Vec<init::UpdateTarget>,
        /// IDE setup surfaces to refresh. If omitted, update reuses IDE setup recorded in the scaffold manifest.
        #[arg(long = "ide")]
        ide: Vec<IdeKind>,
        /// Terminal auto-approval profile for IDEs with a stable settings schema.
        #[arg(long = "auto-approve")]
        auto_approve: Option<TerminalAutoApproveProfile>,
        /// Template to use when refreshing `.boundline/execution.json`.
        #[arg(long)]
        template: Option<InitTemplate>,
        /// Show planned managed changes without writing files.
        #[arg(long)]
        diff: bool,
        /// Apply the planned workspace updates.
        #[arg(long)]
        apply: bool,
        /// Adopt conflicting untracked managed files into the scaffold manifest instead of overwriting them.
        #[arg(long)]
        adopt: bool,
        /// Remove tracked orphaned managed artifacts that are no longer desired.
        #[arg(long)]
        prune: bool,
        /// Show scaffold health, drift, adoption, and orphaned artifact status without mutating the workspace.
        #[arg(long, conflicts_with_all = ["diff", "apply", "force", "adopt", "prune", "template"])]
        status: bool,
        /// Overwrite changed replace-owned managed files during apply.
        #[arg(long)]
        force: bool,
    },
    Config {
        #[command(subcommand)]
        command: ConfigSubcommand,
    },
    Cluster {
        #[command(subcommand)]
        command: ClusterSubcommand,
    },
    Models {
        #[command(subcommand)]
        command: ModelsSubcommand,
    },
}

/// Workflow-specific subcommands.
#[derive(Debug, Subcommand)]
pub enum WorkflowSubcommand {
    List {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Run {
        name: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        goal: Option<String>,
    },
    Status {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Resume {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Inspect {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
}

/// Session history and selection subcommands.
#[derive(Debug, Subcommand)]
pub enum SessionSubcommand {
    List {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Resume {
        session_id: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
}

/// Assistant asset installation subcommands.
#[derive(Debug, Subcommand)]
pub enum AssistantSubcommand {
    Install {
        #[arg(long, value_enum)]
        host: assistant_assets::AssistantHost,
        #[arg(long, value_enum)]
        scope: assistant_assets::AssistantInstallScope,
    },
}

/// Checkpoint management subcommands.
#[derive(Debug, Subcommand)]
pub enum CheckpointSubcommand {
    List {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
    },
    Restore {
        checkpoint_id: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        force: bool,
    },
}

/// Cluster management subcommands.
#[derive(Debug, Subcommand)]
pub enum ClusterSubcommand {
    Init {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long = "cluster-id")]
        cluster_id: String,
        #[arg(long = "member")]
        member: Vec<PathBuf>,
    },
    Status {
        #[arg(long)]
        workspace: PathBuf,
    },
    Inspect {
        #[arg(long)]
        workspace: PathBuf,
    },
}

/// Configuration inspection and mutation subcommands.
#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    Show {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: Option<ConfigShowScope>,
    },
    Set {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        slot: Option<RouteSlot>,
        #[arg(
            long,
            help = "Target routing.chat for advisory conversation instead of a routed slot"
        )]
        chat: bool,
        #[arg(long)]
        reviewer: Option<String>,
        #[arg(long)]
        adjudicator: bool,
        #[arg(long)]
        runtime: RuntimeKind,
        #[arg(long)]
        model: String,
    },
    SetCapability {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        runtime: RuntimeKind,
        #[arg(long)]
        continuation: CapabilityState,
        #[arg(long)]
        resume: CapabilityState,
        #[arg(long)]
        validation: CapabilityState,
        #[arg(long = "handoff-target")]
        handoff_target: CapabilityState,
        #[arg(long = "escalation-context")]
        escalation_context: CapabilityState,
        #[arg(long)]
        notes: Option<String>,
    },
    SetCanon {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long = "mode-selection", value_enum)]
        mode_selection: CanonModeSelectionPreference,
    },
    SetSemanticAcceleration {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        policy: SemanticAccelerationPolicyState,
    },
    Unset {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        slot: Option<RouteSlot>,
        #[arg(
            long,
            help = "Target routing.chat for advisory conversation instead of a routed slot"
        )]
        chat: bool,
        #[arg(long)]
        reviewer: Option<String>,
        #[arg(long)]
        adjudicator: bool,
    },
    UnsetCapability {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        runtime: RuntimeKind,
    },
    SetEffort {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        slot: RouteSlot,
        #[arg(long)]
        level: EffortLevel,
        #[arg(long)]
        fallback: EffortFallbackPolicy,
        #[arg(long)]
        rationale: Option<String>,
    },
    UnsetEffort {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        slot: RouteSlot,
    },
    SetDomain {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        family: DomainFamily,
        #[arg(long, conflicts_with = "disable")]
        enable: bool,
        #[arg(long, conflicts_with = "enable")]
        disable: bool,
        #[arg(long)]
        standards: Option<String>,
    },
    UnsetDomain {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        family: DomainFamily,
    },
    BindContext {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        family: DomainFamily,
        #[arg(long)]
        kind: ExternalContextKind,
        #[arg(long)]
        reference: String,
        #[arg(long)]
        required: bool,
        #[arg(long)]
        notes: Option<String>,
    },
    UnbindContext {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        family: DomainFamily,
        #[arg(long)]
        kind: ExternalContextKind,
        #[arg(long)]
        reference: String,
    },
}

/// Model provider authentication and status subcommands.
#[derive(Debug, Subcommand)]
pub enum ModelsSubcommand {
    Auth {
        #[command(subcommand)]
        command: ModelsAuthSubcommand,
    },
}

/// Authentication management for model providers.
#[derive(Debug, Subcommand)]
pub enum ModelsAuthSubcommand {
    /// Authenticate with a model provider using device-flow OAuth.
    Login {
        /// Provider identifier (e.g. github-copilot).
        #[arg(long, default_value = "github-copilot")]
        provider: String,
    },
    /// Show authentication status for configured providers.
    Status,
    /// Remove stored authentication for a provider.
    Remove {
        /// Provider identifier to remove (e.g. github-copilot).
        #[arg(long)]
        provider: String,
    },
}

impl DeveloperCommand {
    /// Returns the stable name for the selected top-level command.
    pub const fn name(&self) -> CommandName {
        match self {
            Self::Doctor { .. } => CommandName::Doctor,
            Self::Checkpoint { .. } => CommandName::Checkpoint,
            Self::Orchestrate { .. } => CommandName::Orchestrate,
            Self::Goal { .. } => CommandName::Goal,
            Self::Flow { .. } => CommandName::Flow,
            Self::Plan { .. } => CommandName::Plan,
            Self::Step { .. } => CommandName::Step,
            Self::Run { .. } => CommandName::Run,
            Self::Workflow { .. } => CommandName::Workflow,
            Self::Inspect { .. } => CommandName::Inspect,
            Self::Status { .. } => CommandName::Status,
            Self::Next { .. } => CommandName::Next,
            Self::Continue { .. } => CommandName::Continue,
            Self::Session { .. } => CommandName::Session,
            Self::Govern { .. } => CommandName::Govern,
            Self::Assistant { .. } => CommandName::Assistant,
            Self::Init { .. } => CommandName::Init,
            Self::Update { .. } => CommandName::Update,
            Self::Config { .. } => CommandName::Config,
            Self::Cluster { .. } => CommandName::Cluster,
            Self::Models { .. } => CommandName::Models,
        }
    }
}

/// Session-scoped metadata captured for one CLI invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperCommandSession {
    pub command_name: CommandName,
    pub workspace_ref: Option<String>,
    pub requires_workspace_ref: bool,
    pub install_check: bool,
    pub goal: Option<String>,
    pub trace_ref: Option<String>,
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub exit_status: Option<CommandExitStatus>,
    pub trace_location: Option<String>,
}

impl DeveloperCommandSession {
    /// Builds invocation-session metadata from the parsed CLI command.
    pub fn from_command(command: &DeveloperCommand) -> Self {
        match command {
            DeveloperCommand::Doctor { workspace, install } => Self {
                command_name: CommandName::Doctor,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: *install,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Checkpoint { command } => Self {
                command_name: CommandName::Checkpoint,
                workspace_ref: match command {
                    CheckpointSubcommand::List { workspace, cluster, .. }
                    | CheckpointSubcommand::Restore { workspace, cluster, .. } => workspace
                        .as_ref()
                        .or(cluster.as_ref())
                        .map(|path| path.to_string_lossy().into_owned()),
                },
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Goal {
                workspace,
                cluster,
                update: _,
                new_session: _,
                goal,
                brief: _,
                governance: _,
                risk: _,
                zone: _,
                owner: _,
                slug: _,
            } => Self {
                command_name: CommandName::Goal,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: goal.clone(),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Orchestrate {
                workspace,
                cluster,
                goal,
                brief: _,
                flow: _,
                governance: _,
                risk: _,
                zone: _,
                owner: _,
                intent: _,
                planning_stage_complete: _,
                request_id: _,
                answer: _,
                assistant_host: _,
                json_stream: _,
                no_canon: _,
                slug: _,
            } => Self {
                command_name: CommandName::Orchestrate,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: goal.clone(),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Flow { name, workspace, cluster } => Self {
                command_name: CommandName::Flow,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: Some(name.clone()),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Plan { workspace, cluster, .. } => Self {
                command_name: CommandName::Plan,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Step { workspace, cluster } => Self {
                command_name: CommandName::Step,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Run {
                workspace,
                cluster,
                goal,
                compatibility,
                brief,
                governance,
                risk,
                zone,
                owner,
                mode,
                no_canon,
            } => Self {
                command_name: CommandName::Run,
                workspace_ref: if *compatibility || goal.is_some() || !brief.is_empty() {
                    workspace.as_ref()
                } else {
                    workspace.as_ref().or(cluster.as_ref())
                }
                .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: (*compatibility || cluster.is_some())
                    && (goal.is_some()
                        || !brief.is_empty()
                        || governance.is_some()
                        || risk.is_some()
                        || zone.is_some()
                        || owner.is_some()
                        || mode.is_some()
                        || *no_canon),
                install_check: false,
                goal: goal.clone(),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Workflow { command } => Self {
                command_name: CommandName::Workflow,
                workspace_ref: match command {
                    WorkflowSubcommand::List { workspace }
                    | WorkflowSubcommand::Run { workspace, .. }
                    | WorkflowSubcommand::Status { workspace }
                    | WorkflowSubcommand::Resume { workspace }
                    | WorkflowSubcommand::Inspect { workspace } => {
                        workspace.as_ref().map(|path| path.to_string_lossy().into_owned())
                    }
                },
                requires_workspace_ref: false,
                install_check: false,
                goal: match command {
                    WorkflowSubcommand::List { .. } => None,
                    WorkflowSubcommand::Run { name, .. } => Some(name.clone()),
                    WorkflowSubcommand::Status { .. }
                    | WorkflowSubcommand::Resume { .. }
                    | WorkflowSubcommand::Inspect { .. } => None,
                },
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Inspect { trace, workspace, cluster, .. } => Self {
                command_name: CommandName::Inspect,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: trace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Status { workspace, cluster, .. } => Self {
                command_name: CommandName::Status,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Next { workspace, cluster, .. } => Self {
                command_name: CommandName::Next,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Continue { workspace, cluster, .. } => Self {
                command_name: CommandName::Continue,
                workspace_ref: workspace
                    .as_ref()
                    .or(cluster.as_ref())
                    .map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Session { command } => Self {
                command_name: CommandName::Session,
                workspace_ref: match command {
                    SessionSubcommand::List { workspace, cluster }
                    | SessionSubcommand::Resume { workspace, cluster, .. } => workspace
                        .as_ref()
                        .or(cluster.as_ref())
                        .map(|path| path.to_string_lossy().into_owned()),
                },
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Govern { workspace, goal, .. } => Self {
                command_name: CommandName::Govern,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: goal.clone(),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Assistant { .. } => Self {
                command_name: CommandName::Assistant,
                workspace_ref: None,
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Init { workspace, .. } => Self {
                command_name: CommandName::Init,
                workspace_ref: Some(workspace.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Update { workspace, .. } => Self {
                command_name: CommandName::Update,
                workspace_ref: Some(workspace.to_string_lossy().into_owned()),
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Config { command } => Self {
                command_name: CommandName::Config,
                workspace_ref: match command {
                    ConfigSubcommand::Show { workspace, cluster, .. }
                    | ConfigSubcommand::Set { workspace, cluster, .. }
                    | ConfigSubcommand::SetCapability { workspace, cluster, .. }
                    | ConfigSubcommand::SetSemanticAcceleration { workspace, cluster, .. }
                    | ConfigSubcommand::Unset { workspace, cluster, .. }
                    | ConfigSubcommand::UnsetCapability { workspace, cluster, .. }
                    | ConfigSubcommand::SetEffort { workspace, cluster, .. }
                    | ConfigSubcommand::UnsetEffort { workspace, cluster, .. }
                    | ConfigSubcommand::SetDomain { workspace, cluster, .. }
                    | ConfigSubcommand::UnsetDomain { workspace, cluster, .. }
                    | ConfigSubcommand::BindContext { workspace, cluster, .. }
                    | ConfigSubcommand::UnbindContext { workspace, cluster, .. } => workspace
                        .as_ref()
                        .or(cluster.as_ref())
                        .map(|path| path.to_string_lossy().into_owned()),
                    ConfigSubcommand::SetCanon { workspace, .. } => {
                        workspace.as_ref().map(|path| path.to_string_lossy().into_owned())
                    }
                },
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Cluster { command } => Self {
                command_name: CommandName::Cluster,
                workspace_ref: match command {
                    ClusterSubcommand::Init { workspace, .. }
                    | ClusterSubcommand::Status { workspace }
                    | ClusterSubcommand::Inspect { workspace } => {
                        Some(workspace.to_string_lossy().into_owned())
                    }
                },
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Models { .. } => Self {
                command_name: CommandName::Models,
                workspace_ref: None,
                requires_workspace_ref: false,
                install_check: false,
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
        }
    }

    /// Validates the derived invocation-session metadata.
    pub fn validate(&self) -> Result<(), CliValidationError> {
        match self.command_name {
            CommandName::Doctor => {
                let workspace = self.workspace_ref.as_deref().unwrap_or_default();
                if !self.install_check && workspace.trim().is_empty() {
                    return Err(CliValidationError::MissingWorkspaceRef(self.command_name));
                }
            }
            CommandName::Orchestrate => {}
            CommandName::Run => {
                if self.requires_workspace_ref {
                    let workspace = self.workspace_ref.as_deref().unwrap_or_default();
                    if workspace.trim().is_empty() {
                        return Err(CliValidationError::MissingWorkspaceRef(self.command_name));
                    }
                }
            }
            CommandName::Inspect => {
                let has_trace = self.trace_ref.as_deref().map(str::trim).unwrap_or_default();
                let has_workspace =
                    self.workspace_ref.as_deref().map(str::trim).unwrap_or_default();
                if has_trace.is_empty() && has_workspace.is_empty() {
                    return Err(CliValidationError::MissingTraceSelection);
                }
            }
            CommandName::Checkpoint
            | CommandName::Goal
            | CommandName::Flow
            | CommandName::Plan
            | CommandName::Step
            | CommandName::Workflow
            | CommandName::Status
            | CommandName::Next
            | CommandName::Continue
            | CommandName::Session
            | CommandName::Govern
            | CommandName::Init
            | CommandName::Update
            | CommandName::Assistant
            | CommandName::Config
            | CommandName::Cluster
            | CommandName::Models => {}
        }

        if matches!(self.command_name, CommandName::Goal | CommandName::Orchestrate)
            && self.goal.is_some()
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(CliValidationError::MissingGoal(self.command_name));
        }

        if matches!(self.command_name, CommandName::Run)
            && self.goal.is_some()
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(CliValidationError::MissingGoal(self.command_name));
        }

        if matches!(self.command_name, CommandName::Flow)
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(CliValidationError::MissingFlowName);
        }

        Ok(())
    }

    /// Completes the invocation session and returns the rendered command exit code.
    pub fn complete(
        &mut self,
        exit_status: CommandExitStatus,
        trace_location: Option<String>,
    ) -> output::CommandExitCode {
        self.completed_at = Some(current_timestamp_millis());
        self.exit_status = Some(exit_status);
        self.trace_location = trace_location;
        output::CommandExitCode::for_status(exit_status)
    }
}

/// Validation failures for derived CLI invocation state.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum CliValidationError {
    #[error("{0} requires --workspace")]
    MissingWorkspaceRef(CommandName),
    #[error("{0} requires a non-empty --goal")]
    MissingGoal(CommandName),
    #[error("flow requires a non-empty flow name")]
    MissingFlowName,
    #[error("inspect requires --trace or --workspace")]
    MissingTraceSelection,
}

struct DispatchOutcome {
    exit_status: CommandExitStatus,
    output: String,
    host_output: Option<String>,
    stream_output: Option<String>,
    compact_output: Option<String>,
    prefer_compact_output_in_verbose: bool,
    inspection_target: Option<String>,
    trace_location: Option<String>,
    session_status: Option<crate::domain::session::SessionStatusView>,
    guidance_guardian: Option<crate::domain::guidance::GuidanceGuardianProjection>,
    trace_summary: Option<crate::domain::trace::TraceSummaryView>,
}

impl DispatchOutcome {
    fn text(
        exit_status: CommandExitStatus,
        output: impl Into<String>,
        trace_location: Option<String>,
    ) -> Self {
        Self {
            exit_status,
            output: output.into(),
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location,
            session_status: None,
            guidance_guardian: None,
            trace_summary: None,
        }
    }

    fn from_session_report(report: session::SessionCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: report.trace_location,
            session_status: report.session_status,
            guidance_guardian: report.guidance_guardian,
            trace_summary: report.trace_summary,
        }
    }

    fn from_run_report(report: run::RunCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: report.trace_location,
            session_status: report.session_status,
            guidance_guardian: None,
            trace_summary: report.trace_summary,
        }
    }

    fn from_inspect_report(report: inspect::InspectCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: report.inspection_target,
            trace_location: report.trace_location,
            session_status: None,
            guidance_guardian: None,
            trace_summary: report.trace_summary,
        }
    }

    fn from_orchestrate_report(report: orchestrate::OrchestrateCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: Some(output::render_orchestrate_stream_json(&report.events)),
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: report.trace_location,
            session_status: report.session_status,
            guidance_guardian: None,
            trace_summary: report.trace_summary,
        }
    }

    fn from_orchestrate_report_human(report: orchestrate::OrchestrateCommandReport) -> Self {
        let rendered_output = output::render_human_orchestrate_report(&report);
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: Some(rendered_output.clone()),
            stream_output: None,
            compact_output: Some(rendered_output),
            prefer_compact_output_in_verbose: true,
            inspection_target: None,
            trace_location: report.trace_location,
            session_status: report.session_status,
            guidance_guardian: None,
            trace_summary: report.trace_summary,
        }
    }

    fn from_init_report(report: init::InitCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: None,
            session_status: None,
            guidance_guardian: None,
            trace_summary: None,
        }
    }

    fn from_update_report(report: init::UpdateCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: None,
            session_status: None,
            guidance_guardian: None,
            trace_summary: None,
        }
    }

    fn render_human_output(&self, verbose: bool) -> String {
        if verbose {
            if self.prefer_compact_output_in_verbose
                && let Some(compact_output) = &self.compact_output
            {
                return compact_output.clone();
            }

            if let Some(session_status) = &self.session_status {
                let mut rendered = output::render_session_status(session_status);
                if let Some(guidance_guardian) = &self.guidance_guardian {
                    let guidance_lines =
                        output::render_guidance_projection_lines(guidance_guardian);
                    if !guidance_lines.is_empty() {
                        rendered.push('\n');
                        rendered.push_str(&guidance_lines.join("\n"));
                    }
                }
                return rendered;
            }

            return self.output.clone();
        }

        if let Some(compact_output) = &self.compact_output {
            return compact_output.clone();
        }

        if let Some(trace_summary) = &self.trace_summary {
            let next_command = self.next_command_from_output().unwrap_or_else(|| {
                if self.inspection_target.is_some() {
                    output::next_command_after_inspect(trace_summary.terminal_status)
                } else {
                    output::next_command_after_run(trace_summary.terminal_status)
                }
            });
            return output::render_trace_summary_brief(
                trace_summary,
                self.inspection_target.as_deref(),
                next_command,
            );
        }

        self.output.clone()
    }

    fn render_host_output(&self) -> &str {
        self.host_output.as_deref().unwrap_or(&self.output)
    }

    fn next_command_from_output(&self) -> Option<&str> {
        self.output.lines().find_map(|line| line.strip_prefix("next_command: "))
    }
}

/// Parses the CLI, dispatches the selected command, and returns the process exit code.
pub fn execute() -> i32 {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        let mut help = Cli::command();
        if let Err(error) = help.print_help() {
            eprintln!("{error}");
            return output::CommandExitCode::for_status(CommandExitStatus::NonSuccess).code();
        }
        println!();
        return output::CommandExitCode::for_status(CommandExitStatus::Succeeded).code();
    };

    let mut session = DeveloperCommandSession::from_command(&command);

    if let DeveloperCommand::Inspect { trace: None, workspace: None, cluster: None, .. } = &command
    {
        session.workspace_ref =
            std::env::current_dir().ok().map(|path| path.to_string_lossy().into_owned());
    }

    match session.validate() {
        Err(error) => {
            let rendered = output::validation_error_message(&error);
            let exit_code = session.complete(CommandExitStatus::InvalidInvocation, None);
            if cli.json {
                println!(
                    "{}",
                    output::render_host_command_json(
                        command.name().as_str(),
                        CommandExitStatus::InvalidInvocation,
                        &rendered,
                        None,
                        None,
                        None,
                    )
                );
            } else {
                eprintln!("{rendered}");
            }
            exit_code.code()
        }
        Ok(()) => {
            if let Err(error) = load_command_environment(Some(&command)) {
                let exit_code = session.complete(CommandExitStatus::NonSuccess, None);
                if cli.json {
                    println!(
                        "{}",
                        output::render_host_command_json(
                            command.name().as_str(),
                            CommandExitStatus::NonSuccess,
                            &error,
                            None,
                            None,
                            None,
                        )
                    );
                } else {
                    eprintln!("{error}");
                }
                return exit_code.code();
            }

            let outcome = dispatch(&command);
            let exit_code = session.complete(outcome.exit_status, outcome.trace_location.clone());
            if let Some(stream_output) = outcome.stream_output.as_ref() {
                println!("{stream_output}");
            } else if cli.json {
                println!(
                    "{}",
                    output::render_host_command_json(
                        command.name().as_str(),
                        outcome.exit_status,
                        outcome.render_host_output(),
                        outcome.trace_location.as_deref(),
                        outcome.session_status.as_ref(),
                        outcome.trace_summary.as_ref(),
                    )
                );
            } else {
                println!("{}", outcome.render_human_output(cli.verbose));
            }
            exit_code.code()
        }
    }
}

fn load_command_environment(command: Option<&DeveloperCommand>) -> Result<(), String> {
    let workspace = command.and_then(command_environment_workspace).or_else(|| {
        if command.is_none() { cli_workspace::resolve_workspace(None).ok() } else { None }
    });
    env_layer::load_provider_environment(workspace.as_deref())
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn command_environment_workspace(command: &DeveloperCommand) -> Option<PathBuf> {
    match command {
        DeveloperCommand::Doctor { workspace, install } => {
            if *install {
                None
            } else {
                resolve_command_workspace(workspace.as_deref())
            }
        }
        DeveloperCommand::Goal { workspace, .. }
        | DeveloperCommand::Flow { workspace, .. }
        | DeveloperCommand::Plan { workspace, .. }
        | DeveloperCommand::Step { workspace, .. }
        | DeveloperCommand::Orchestrate { workspace, .. }
        | DeveloperCommand::Run { workspace, .. }
        | DeveloperCommand::Inspect { workspace, .. }
        | DeveloperCommand::Status { workspace, .. }
        | DeveloperCommand::Next { workspace, .. }
        | DeveloperCommand::Continue { workspace, .. }
        | DeveloperCommand::Govern { workspace, .. } => {
            resolve_command_workspace(workspace.as_deref())
        }
        DeveloperCommand::Session { command } => match command {
            SessionSubcommand::List { workspace, cluster }
            | SessionSubcommand::Resume { workspace, cluster, .. } => {
                resolve_command_workspace(workspace.as_deref().or(cluster.as_deref()))
            }
        },
        DeveloperCommand::Workflow { command } => match command {
            WorkflowSubcommand::List { workspace }
            | WorkflowSubcommand::Run { workspace, .. }
            | WorkflowSubcommand::Status { workspace }
            | WorkflowSubcommand::Resume { workspace }
            | WorkflowSubcommand::Inspect { workspace } => {
                resolve_command_workspace(workspace.as_deref())
            }
        },
        DeveloperCommand::Checkpoint { command } => match command {
            CheckpointSubcommand::List { workspace, .. }
            | CheckpointSubcommand::Restore { workspace, .. } => {
                resolve_command_workspace(workspace.as_deref())
            }
        },
        DeveloperCommand::Init { scope, workspace, .. } => {
            if *scope == InitConfigScope::Global {
                None
            } else {
                resolve_command_workspace(Some(workspace.as_path()))
            }
        }
        DeveloperCommand::Update { workspace, .. } => {
            resolve_command_workspace(Some(workspace.as_path()))
        }
        DeveloperCommand::Config { command } => command_environment_workspace_for_config(command),
        DeveloperCommand::Cluster { command } => match command {
            ClusterSubcommand::Init { workspace, .. }
            | ClusterSubcommand::Status { workspace }
            | ClusterSubcommand::Inspect { workspace } => {
                resolve_command_workspace(Some(workspace.as_path()))
            }
        },
        DeveloperCommand::Assistant { .. } => None,
        DeveloperCommand::Models { .. } => None,
    }
}

fn command_environment_workspace_for_config(command: &ConfigSubcommand) -> Option<PathBuf> {
    match command {
        ConfigSubcommand::Show { workspace, scope, .. } => {
            if matches!(scope, Some(ConfigShowScope::Global)) {
                None
            } else {
                resolve_command_workspace(workspace.as_deref())
            }
        }
        ConfigSubcommand::Set { workspace, scope, .. }
        | ConfigSubcommand::SetCapability { workspace, scope, .. }
        | ConfigSubcommand::SetSemanticAcceleration { workspace, scope, .. }
        | ConfigSubcommand::Unset { workspace, scope, .. }
        | ConfigSubcommand::UnsetCapability { workspace, scope, .. }
        | ConfigSubcommand::SetEffort { workspace, scope, .. }
        | ConfigSubcommand::UnsetEffort { workspace, scope, .. }
        | ConfigSubcommand::SetDomain { workspace, scope, .. }
        | ConfigSubcommand::UnsetDomain { workspace, scope, .. }
        | ConfigSubcommand::BindContext { workspace, scope, .. }
        | ConfigSubcommand::UnbindContext { workspace, scope, .. } => {
            if *scope == ConfigWriteScope::Global {
                None
            } else {
                resolve_command_workspace(workspace.as_deref())
            }
        }
        ConfigSubcommand::SetCanon { workspace, .. } => {
            resolve_command_workspace(workspace.as_deref())
        }
    }
}

fn resolve_command_workspace(workspace: Option<&Path>) -> Option<PathBuf> {
    cli_workspace::resolve_workspace(workspace).ok()
}

fn dispatch(command: &DeveloperCommand) -> DispatchOutcome {
    match command {
        DeveloperCommand::Doctor { workspace, install } => {
            dispatch_doctor_command(workspace.as_deref(), *install)
        }
        DeveloperCommand::Orchestrate { .. } => dispatch_orchestrate_command(command),
        DeveloperCommand::Run { .. } => dispatch_run_command(command),
        DeveloperCommand::Workflow { command } => dispatch_workflow_command(command),
        DeveloperCommand::Checkpoint { command } => dispatch_checkpoint_command(command),
        DeveloperCommand::Inspect { trace, workspace, cluster, session, audit } => {
            dispatch_inspect_command(
                trace.as_deref(),
                workspace.as_deref(),
                cluster.as_deref(),
                session.as_deref(),
                *audit,
            )
        }
        DeveloperCommand::Session { command } => dispatch_session_history_command(command),
        DeveloperCommand::Goal { .. }
        | DeveloperCommand::Flow { .. }
        | DeveloperCommand::Plan { .. }
        | DeveloperCommand::Step { .. }
        | DeveloperCommand::Status { .. }
        | DeveloperCommand::Next { .. }
        | DeveloperCommand::Continue { .. } => dispatch_session_command(command),
        DeveloperCommand::Govern { .. } => dispatch_govern_command(command),
        DeveloperCommand::Assistant { command } => dispatch_assistant_command(command),
        DeveloperCommand::Init { .. } => dispatch_init_command(command),
        DeveloperCommand::Update { .. } => dispatch_update_command(command),
        DeveloperCommand::Config { command } => dispatch_config_command(command),
        DeveloperCommand::Cluster { command } => dispatch_cluster_command(command),
        DeveloperCommand::Models { command } => dispatch_models_command(command),
    }
}

fn dispatch_doctor_command(workspace: Option<&Path>, install: bool) -> DispatchOutcome {
    let report = if install {
        diagnostics::diagnose_installation()
    } else {
        let Some(workspace) = workspace else {
            return DispatchOutcome::text(
                CommandExitStatus::InvalidInvocation,
                output::validation_error_message(&CliValidationError::MissingWorkspaceRef(
                    CommandName::Doctor,
                )),
                None,
            );
        };
        diagnostics::diagnose_workspace_context(workspace)
    };
    DispatchOutcome::text(
        if report.ready {
            CommandExitStatus::Succeeded
        } else {
            CommandExitStatus::InvalidInvocation
        },
        output::render_diagnostics(&report),
        None,
    )
}

fn dispatch_run_command(command: &DeveloperCommand) -> DispatchOutcome {
    let DeveloperCommand::Run {
        workspace,
        cluster,
        goal,
        compatibility,
        brief,
        governance,
        risk,
        zone,
        owner,
        mode,
        no_canon,
    } = command
    else {
        return dispatch_internal_command_mismatch(CommandName::Run);
    };
    let custom = *compatibility
        || goal.is_some()
        || !brief.is_empty()
        || governance.is_some()
        || risk.is_some()
        || zone.is_some()
        || owner.is_some()
        || mode.is_some()
        || *no_canon;
    if custom {
        dispatch_custom_run(
            workspace.as_deref(),
            goal.as_deref(),
            brief,
            *compatibility,
            *governance,
            risk.as_deref(),
            zone.as_deref(),
            owner.as_deref(),
            *mode,
            *no_canon,
        )
    } else {
        dispatch_session_result(
            CommandName::Run,
            session::execute_run_with_target(workspace.as_deref(), cluster.as_deref()),
        )
    }
}

fn dispatch_orchestrate_command(command: &DeveloperCommand) -> DispatchOutcome {
    let DeveloperCommand::Orchestrate {
        workspace,
        cluster,
        goal,
        brief,
        flow,
        governance,
        risk,
        zone,
        owner,
        intent,
        planning_stage_complete,
        request_id,
        answer,
        assistant_host,
        json_stream,
        no_canon,
        slug,
    } = command
    else {
        return dispatch_internal_command_mismatch(CommandName::Orchestrate);
    };

    let is_json_stream = *json_stream;

    let result = orchestrate::execute_orchestrate(
        workspace.as_deref(),
        cluster.as_deref(),
        goal.as_deref(),
        brief,
        flow.as_deref(),
        *governance,
        risk.as_deref(),
        zone.as_deref(),
        owner.as_deref(),
        *intent,
        planning_stage_complete.as_deref(),
        request_id.as_deref(),
        answer.as_deref(),
        *assistant_host,
        *no_canon,
        slug.as_deref(),
    );

    match result {
        Ok(report) => {
            if is_json_stream {
                DispatchOutcome::from_orchestrate_report(report)
            } else {
                DispatchOutcome::from_orchestrate_report_human(report)
            }
        }
        Err(error) => {
            let msg = format!("orchestrate error: {}", error);
            if is_json_stream {
                let envelope = orchestrate::OrchestrateEventEnvelope {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    timestamp_ms: crate::domain::trace::current_timestamp_millis(),
                    event_kind: "terminal".to_string(),
                    audit: None,
                    actor_kind: None,
                    actor_name: None,
                    runtime_kind: None,
                    provider: None,
                    route_slot: None,
                    model_name: None,
                    decision_family: None,
                    review_step: None,
                    vote_summary: None,
                    adjudication_summary: None,
                    governance_mode: None,
                    session_ref: None,
                    phase_kind: None,
                    stage_key: None,
                    message: msg,
                    artifact: None,
                    phase_request: None,
                    instruction: None,
                    resume_command: None,
                    assistant_resume_command: None,
                    next_command: None,
                    assistant_next_command: None,
                    session_status: None,
                    trace_summary: None,
                };
                DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: String::new(),
                    host_output: None,
                    stream_output: serde_json::to_string(&envelope).ok().map(|s| s + "\n"),
                    compact_output: None,
                    prefer_compact_output_in_verbose: false,
                    inspection_target: None,
                    trace_location: None,
                    session_status: None,
                    guidance_guardian: None,
                    trace_summary: None,
                }
            } else {
                DispatchOutcome::text(CommandExitStatus::NonSuccess, msg, None)
            }
        }
    }
}

// The parameters mirror the `DeveloperCommand::Run` fields; no further
// grouping is warranted for a single-call CLI dispatch helper.
#[allow(clippy::too_many_arguments)]
fn dispatch_custom_run(
    workspace: Option<&Path>,
    goal: Option<&str>,
    brief: &[PathBuf],
    compatibility: bool,
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    mode: Option<CanonMode>,
    no_canon: bool,
) -> DispatchOutcome {
    let resolved_workspace = match cli_workspace::resolve_workspace(workspace) {
        Ok(workspace) => workspace,
        Err(error) => {
            return DispatchOutcome::text(
                CommandExitStatus::InvalidInvocation,
                format!("workspace resolution failed: {error}"),
                None,
            );
        }
    };
    let workspace = &resolved_workspace;
    if !workspace.is_dir() {
        return DispatchOutcome::text(
            CommandExitStatus::InvalidInvocation,
            output::validation_error_message(&CliValidationError::MissingWorkspaceRef(
                CommandName::Run,
            )),
            None,
        );
    }
    let report = if compatibility {
        diagnostics::diagnose_workspace(workspace)
    } else {
        diagnostics::diagnose_native_direct_run_workspace(workspace)
    };
    if !report.ready {
        return DispatchOutcome::text(
            CommandExitStatus::InvalidInvocation,
            output::render_diagnostics(&report),
            None,
        );
    }
    let result = if compatibility {
        run::execute_custom_run(workspace, goal, brief, governance, risk, zone, owner)
    } else {
        run::execute_native_direct_run(
            workspace, goal, brief, governance, risk, zone, owner, mode, no_canon,
        )
    };
    match result {
        Ok(report) => DispatchOutcome::from_run_report(report),
        Err(error) => {
            DispatchOutcome::text(CommandExitStatus::InvalidInvocation, error.message(), None)
        }
    }
}

fn dispatch_workflow_command(command: &WorkflowSubcommand) -> DispatchOutcome {
    let result = match command {
        WorkflowSubcommand::List { workspace } => workflow::execute_list(workspace.as_deref()),
        WorkflowSubcommand::Run { name, workspace, goal } => {
            workflow::execute_run(workspace.as_deref(), name, goal.as_deref())
        }
        WorkflowSubcommand::Status { workspace } => workflow::execute_status(workspace.as_deref()),
        WorkflowSubcommand::Resume { workspace } => workflow::execute_resume(workspace.as_deref()),
        WorkflowSubcommand::Inspect { workspace } => {
            workflow::execute_inspect(workspace.as_deref())
        }
    };
    dispatch_prefixed_result(CommandName::Workflow, result, |report| {
        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
    })
}

fn dispatch_checkpoint_command(command: &CheckpointSubcommand) -> DispatchOutcome {
    let result = match command {
        CheckpointSubcommand::List { workspace, cluster, session } => {
            checkpoint::execute_list(workspace.as_deref(), cluster.as_deref(), session.as_deref())
        }
        CheckpointSubcommand::Restore { checkpoint_id, workspace, cluster, session, force } => {
            checkpoint::execute_restore(
                checkpoint_id,
                workspace.as_deref(),
                cluster.as_deref(),
                *force,
                session.as_deref(),
            )
        }
    };
    dispatch_prefixed_result(CommandName::Checkpoint, result, |report| {
        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
    })
}

fn dispatch_session_history_command(command: &SessionSubcommand) -> DispatchOutcome {
    let result = match command {
        SessionSubcommand::List { workspace, cluster } => {
            session::execute_session_list_with_target(workspace.as_deref(), cluster.as_deref())
        }
        SessionSubcommand::Resume { session_id, workspace, cluster } => {
            session::execute_session_resume_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                session_id,
            )
        }
    };
    dispatch_session_result(CommandName::Session, result)
}

fn dispatch_inspect_command(
    trace: Option<&Path>,
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    session_id: Option<&str>,
    audit: bool,
) -> DispatchOutcome {
    let default_workspace = if trace.is_none() && workspace.is_none() && cluster.is_none() {
        std::env::current_dir().ok()
    } else {
        None
    };
    let workspace_ref = workspace.or(cluster).or(default_workspace.as_deref());
    match inspect::execute_inspect(trace, workspace_ref, session_id, audit) {
        Ok(report) => DispatchOutcome::from_inspect_report(report),
        Err(error) => DispatchOutcome::text(
            match error {
                inspect::InspectCommandError::InvalidSession(_)
                | inspect::InspectCommandError::UnknownSession(_) => CommandExitStatus::NonSuccess,
                _ => CommandExitStatus::TraceReadFailure,
            },
            inspect::render_error(trace, workspace_ref, session_id, &error),
            None,
        ),
    }
}

fn dispatch_session_command(command: &DeveloperCommand) -> DispatchOutcome {
    match command {
        DeveloperCommand::Goal {
            workspace,
            cluster,
            update,
            new_session,
            goal,
            brief,
            governance,
            risk,
            zone,
            owner,
            slug,
        } => dispatch_session_result(
            CommandName::Goal,
            if *update {
                session::execute_goal_update_with_target(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    goal.as_deref(),
                    brief,
                    *governance,
                    risk.as_deref(),
                    zone.as_deref(),
                    owner.as_deref(),
                )
            } else if *new_session {
                session::execute_goal_with_target(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    goal.as_deref(),
                    brief,
                    *governance,
                    risk.as_deref(),
                    zone.as_deref(),
                    owner.as_deref(),
                    slug.as_deref(),
                )
            } else {
                session::execute_goal_upsert_with_target(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    goal.as_deref(),
                    brief,
                    *governance,
                    risk.as_deref(),
                    zone.as_deref(),
                    owner.as_deref(),
                    slug.as_deref(),
                )
            },
        ),
        DeveloperCommand::Flow { name, workspace, cluster } => dispatch_session_result(
            CommandName::Flow,
            session::execute_flow_with_target(workspace.as_deref(), cluster.as_deref(), name),
        ),
        DeveloperCommand::Plan { workspace, cluster, input, flow, no_flow, no_canon } => {
            dispatch_session_result(
                CommandName::Plan,
                session::execute_plan_with_target_input(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    flow.as_deref(),
                    *no_flow,
                    *no_canon,
                    input.as_deref(),
                ),
            )
        }
        DeveloperCommand::Step { workspace, cluster } => dispatch_session_result(
            CommandName::Step,
            session::execute_step_with_target(workspace.as_deref(), cluster.as_deref()),
        ),
        DeveloperCommand::Status { workspace, cluster, session } => dispatch_session_result(
            CommandName::Status,
            session::execute_status_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                session.as_deref(),
            ),
        ),
        DeveloperCommand::Next { workspace, cluster, session } => dispatch_session_result(
            CommandName::Next,
            session::execute_next_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                session.as_deref(),
            ),
        ),
        DeveloperCommand::Continue { workspace, cluster, session } => dispatch_session_result(
            CommandName::Continue,
            session::execute_continue_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                session.as_deref(),
            ),
        ),
        _ => dispatch_internal_command_mismatch(command.name()),
    }
}

fn dispatch_govern_command(command: &DeveloperCommand) -> DispatchOutcome {
    let DeveloperCommand::Govern {
        workspace,
        mode,
        goal,
        brief,
        base,
        head,
        risk,
        structural_impact,
        public_contract_change,
        validation_exhausted,
        pr_ready,
        preserved_behavior_evidence,
    } = command
    else {
        return dispatch_internal_command_mismatch(CommandName::Govern);
    };
    dispatch_prefixed_result(
        CommandName::Govern,
        govern::execute_govern(govern::GovernRequest {
            workspace: workspace.as_deref(),
            mode: *mode,
            goal: goal.as_deref(),
            brief,
            base: base.as_deref(),
            head: head.as_deref(),
            risk: risk.as_deref(),
            structural_impact: *structural_impact,
            public_contract_change: *public_contract_change,
            validation_exhausted: *validation_exhausted,
            pr_ready: *pr_ready,
            preserved_behavior_evidence: *preserved_behavior_evidence,
        }),
        |report| DispatchOutcome::text(report.exit_status, report.terminal_output, None),
    )
}

fn dispatch_assistant_command(command: &AssistantSubcommand) -> DispatchOutcome {
    match command {
        AssistantSubcommand::Install { host, scope } => {
            let report = assistant_assets::install_global_assistant_package(*host, *scope);
            DispatchOutcome::text(
                CommandExitStatus::Succeeded,
                assistant_assets::render_assistant_install_report(&report),
                None,
            )
        }
    }
}

fn dispatch_init_command(command: &DeveloperCommand) -> DispatchOutcome {
    let DeveloperCommand::Init {
        scope,
        workspace,
        non_interactive,
        template,
        assistant,
        route,
        domain,
        domain_standard,
        context_binding,
        required_context_binding,
        canon_mode_selection,
        risk,
        zone,
        owner,
        ide,
        auto_approve,
        export_docs,
        refresh,
        diff,
        to,
        force,
    } = command
    else {
        return dispatch_internal_command_mismatch(CommandName::Init);
    };
    dispatch_prefixed_result(
        CommandName::Init,
        init::execute_init(init::InitRequest {
            workspace,
            scope: *scope,
            non_interactive: *non_interactive,
            interactive_terminal_override: None,
            interactor: None,
            template: *template,
            assistants: assistant,
            routes: route,
            domains: domain,
            domain_standards: domain_standard,
            context_bindings: context_binding,
            required_context_bindings: required_context_binding,
            canon_mode_selection: *canon_mode_selection,
            risk: risk.as_deref(),
            zone: zone.as_deref(),
            owner: owner.as_deref(),
            ide,
            auto_approve: *auto_approve,
            export_docs: *export_docs,
            docs_refresh: *refresh,
            docs_diff: *diff,
            docs_output_dir: to.as_deref(),
            force: *force,
        }),
        DispatchOutcome::from_init_report,
    )
}

fn dispatch_update_command(command: &DeveloperCommand) -> DispatchOutcome {
    let DeveloperCommand::Update {
        workspace,
        target,
        ide,
        auto_approve,
        template,
        diff,
        apply,
        adopt,
        prune,
        status,
        force,
    } = command
    else {
        return dispatch_internal_command_mismatch(CommandName::Update);
    };
    dispatch_prefixed_result(
        CommandName::Update,
        init::execute_update(init::UpdateRequest {
            workspace,
            targets: target,
            ide,
            auto_approve: *auto_approve,
            template: *template,
            diff: *diff,
            apply: *apply,
            adopt: *adopt,
            prune: *prune,
            status: *status,
            force: *force,
        }),
        DispatchOutcome::from_update_report,
    )
}

fn dispatch_config_command(command: &ConfigSubcommand) -> DispatchOutcome {
    let result = match command {
        ConfigSubcommand::Show { workspace, cluster, scope } => {
            config::execute_show(workspace.as_deref(), cluster.as_deref(), *scope)
        }
        ConfigSubcommand::Set {
            workspace,
            cluster,
            scope,
            slot,
            chat,
            reviewer,
            adjudicator,
            runtime,
            model,
        } => config::execute_set(config::SetConfigRequest {
            workspace: workspace.as_deref(),
            cluster: cluster.as_deref(),
            scope: *scope,
            slot: *slot,
            chat: *chat,
            reviewer: reviewer.as_deref(),
            adjudicator: *adjudicator,
            runtime: *runtime,
            model,
        }),
        ConfigSubcommand::SetCapability {
            workspace,
            cluster,
            scope,
            runtime,
            continuation,
            resume,
            validation,
            handoff_target,
            escalation_context,
            notes,
        } => config::execute_set_capability(config::SetCapabilityRequest {
            workspace: workspace.as_deref(),
            cluster: cluster.as_deref(),
            scope: *scope,
            runtime: *runtime,
            continuation: *continuation,
            resume: *resume,
            validation: *validation,
            handoff_target: *handoff_target,
            escalation_context: *escalation_context,
            notes: notes.as_deref(),
        }),
        ConfigSubcommand::SetCanon { workspace, mode_selection } => {
            let resolved_workspace = cli_workspace::resolve_workspace(workspace.as_deref())
                .map_err(|error| {
                    config::ConfigCommandError::WorkspaceResolution(error.to_string())
                });
            match resolved_workspace {
                Ok(workspace) => config::execute_set_canon(Some(&workspace), *mode_selection),
                Err(error) => Err(error),
            }
        }
        ConfigSubcommand::Unset {
            workspace,
            cluster,
            scope,
            slot,
            chat,
            reviewer,
            adjudicator,
        } => config::execute_unset(
            workspace.as_deref(),
            cluster.as_deref(),
            *scope,
            *slot,
            *chat,
            reviewer.as_deref(),
            *adjudicator,
        ),
        ConfigSubcommand::UnsetCapability { workspace, cluster, scope, runtime } => {
            config::execute_unset_capability(
                workspace.as_deref(),
                cluster.as_deref(),
                *scope,
                *runtime,
            )
        }
        ConfigSubcommand::SetSemanticAcceleration { workspace, cluster, scope, policy } => {
            config::execute_set_semantic_acceleration(config::SetSemanticAccelerationRequest {
                workspace: workspace.as_deref(),
                cluster: cluster.as_deref(),
                scope: *scope,
                policy: *policy,
            })
        }
        ConfigSubcommand::SetEffort {
            workspace,
            cluster,
            scope,
            slot,
            level,
            fallback,
            rationale,
        } => config::execute_set_effort(config::SetEffortRequest {
            workspace: workspace.as_deref(),
            cluster: cluster.as_deref(),
            scope: *scope,
            slot: *slot,
            level: *level,
            fallback: *fallback,
            rationale: rationale.as_deref(),
        }),
        ConfigSubcommand::UnsetEffort { workspace, cluster, scope, slot } => {
            config::execute_unset_effort(workspace.as_deref(), cluster.as_deref(), *scope, *slot)
        }
        ConfigSubcommand::SetDomain {
            workspace,
            cluster,
            scope,
            family,
            enable,
            disable,
            standards,
        } => config::execute_set_domain(config::SetDomainRequest {
            workspace: workspace.as_deref(),
            cluster: cluster.as_deref(),
            scope: *scope,
            family: *family,
            enable: *enable,
            disable: *disable,
            standards: standards.as_deref(),
        }),
        ConfigSubcommand::UnsetDomain { workspace, cluster, scope, family } => {
            config::execute_unset_domain(workspace.as_deref(), cluster.as_deref(), *scope, *family)
        }
        ConfigSubcommand::BindContext {
            workspace,
            cluster,
            scope,
            family,
            kind,
            reference,
            required,
            notes,
        } => config::execute_bind_context(config::BindContextRequest {
            workspace: workspace.as_deref(),
            cluster: cluster.as_deref(),
            scope: *scope,
            family: *family,
            kind: *kind,
            reference,
            required: *required,
            notes: notes.as_deref(),
        }),
        ConfigSubcommand::UnbindContext { workspace, cluster, scope, family, kind, reference } => {
            config::execute_unbind_context(
                workspace.as_deref(),
                cluster.as_deref(),
                *scope,
                *family,
                *kind,
                reference,
            )
        }
    };
    dispatch_prefixed_result(CommandName::Config, result, |report| {
        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
    })
}

fn dispatch_cluster_command(command: &ClusterSubcommand) -> DispatchOutcome {
    let result = match command {
        ClusterSubcommand::Init { workspace, cluster_id, member } => {
            cluster::execute_init(workspace, cluster_id, member)
        }
        ClusterSubcommand::Status { workspace } => cluster::execute_status(workspace),
        ClusterSubcommand::Inspect { workspace } => cluster::execute_inspect(workspace),
    };
    dispatch_prefixed_result(CommandName::Cluster, result, |report| {
        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
    })
}

fn dispatch_prefixed_result<T, E, F>(
    command_name: CommandName,
    result: Result<T, E>,
    on_success: F,
) -> DispatchOutcome
where
    E: fmt::Display,
    F: FnOnce(T) -> DispatchOutcome,
{
    match result {
        Ok(report) => on_success(report),
        Err(error) => DispatchOutcome::text(
            CommandExitStatus::NonSuccess,
            format!("{} error: {error}", command_name.as_str()),
            None,
        ),
    }
}

fn dispatch_session_result(
    command_name: CommandName,
    result: Result<session::SessionCommandReport, session::SessionCommandError>,
) -> DispatchOutcome {
    match result {
        Ok(report) => DispatchOutcome::from_session_report(report),
        Err(error) => DispatchOutcome::text(
            CommandExitStatus::NonSuccess,
            session::render_error(command_name.as_str(), &error),
            None,
        ),
    }
}

fn dispatch_internal_command_mismatch(command_name: CommandName) -> DispatchOutcome {
    DispatchOutcome::text(
        CommandExitStatus::NonSuccess,
        format!("internal dispatch mismatch for {}", command_name.as_str()),
        None,
    )
}

fn dispatch_models_command(command: &ModelsSubcommand) -> DispatchOutcome {
    let result = match command {
        ModelsSubcommand::Auth { command: auth_command } => match auth_command {
            ModelsAuthSubcommand::Login { provider } => models_auth::execute_login(provider),
            ModelsAuthSubcommand::Status => models_auth::execute_status(),
            ModelsAuthSubcommand::Remove { provider } => models_auth::execute_remove(provider),
        },
    };
    dispatch_prefixed_result(CommandName::Models, result, |report| {
        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::Parser;
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        AssistantSubcommand, CheckpointSubcommand, Cli, ClusterSubcommand, CommandExitStatus,
        CommandName, ConfigSubcommand, DeveloperCommand, DeveloperCommandSession,
        SessionSubcommand, WorkflowSubcommand, dispatch,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::adapters::session_store::{FileSessionStore, SessionStore};
    use crate::cli::assistant_assets::{AssistantHost, AssistantInstallScope};
    use crate::cli::orchestrate::OrchestrateIntent;
    use crate::domain::configuration::{
        CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
        InitConfigScope, InitTemplate, RouteSlot, RuntimeKind, SemanticAccelerationPolicyState,
    };
    use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};
    use crate::domain::governance::{CanonMode, CanonModeSelectionPreference};
    use crate::domain::guidance::GuidanceGuardianProjection;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus, SessionStatusView};
    use crate::test_support::CurrentDirGuard;

    const FIXTURE_CARGO_TOML: &str = r#"[package]
name = "dispatch_fixture"
version = "0.1.0"
edition = "2024"
"#;

    const RED_LIB_RS: &str = "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n";

    const FIXTURE_TEST_RS: &str = r#"#[test]
fn red_to_green_addition() {
    assert_eq!(dispatch_fixture::add(2, 2), 4);
}
"#;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    #[test]
    fn init_cli_defaults_workspace_to_current_directory() {
        let cli = Cli::try_parse_from([
            "boundline",
            "init",
            "--non-interactive",
            "--assistant",
            "copilot",
            "--canon-mode-selection",
            "auto-confirm",
            "--force",
        ])
        .unwrap();

        match cli.command {
            Some(DeveloperCommand::Init { scope, workspace, .. }) => {
                assert_eq!(scope, InitConfigScope::Workspace);
                assert_eq!(workspace, PathBuf::from("."));
            }
            other => panic!("expected init command, got {other:?}"),
        }
    }

    #[test]
    fn update_cli_defaults_workspace_to_current_directory() {
        let cli = Cli::try_parse_from(["boundline", "update", "--apply"]).unwrap();

        match cli.command {
            Some(DeveloperCommand::Update { workspace, apply, .. }) => {
                assert_eq!(workspace, PathBuf::from("."));
                assert!(apply);
            }
            other => panic!("expected update command, got {other:?}"),
        }
    }

    #[test]
    fn update_cli_accepts_status_and_prune_flags() {
        let cli = Cli::try_parse_from([
            "boundline",
            "update",
            "--target",
            "assistant",
            "--prune",
            "--status",
        ])
        .unwrap_err();

        assert_eq!(cli.kind(), clap::error::ErrorKind::ArgumentConflict);

        let cli = Cli::try_parse_from([
            "boundline",
            "update",
            "--target",
            "assistant",
            "--prune",
            "--apply",
            "--force",
        ])
        .unwrap();

        match cli.command {
            Some(DeveloperCommand::Update { prune, apply, force, .. }) => {
                assert!(prune);
                assert!(apply);
                assert!(force);
            }
            other => panic!("expected update command, got {other:?}"),
        }
    }

    #[test]
    fn cli_accepts_global_verbose_flag() {
        let cli = Cli::try_parse_from(["boundline", "--verbose", "status"]).unwrap();

        assert!(cli.verbose);
        assert!(!cli.json);
        match cli.command {
            Some(DeveloperCommand::Status { .. }) => {}
            other => panic!("expected status command, got {other:?}"),
        }
    }

    #[test]
    fn session_history_cli_parses_list_and_resume_subcommands() -> Result<(), String> {
        let list = Cli::try_parse_from(["boundline", "session", "list"])
            .map_err(|error| error.to_string())?;
        match list.command {
            Some(DeveloperCommand::Session { command: SessionSubcommand::List { .. } }) => {}
            other => return Err(format!("expected session list command, got {other:?}")),
        }

        let resume = Cli::try_parse_from(["boundline", "session", "resume", "session-123"])
            .map_err(|error| error.to_string())?;
        match resume.command {
            Some(DeveloperCommand::Session {
                command: SessionSubcommand::Resume { session_id, .. },
            }) => {
                if session_id != "session-123" {
                    return Err(format!("expected session-123, got {session_id}"));
                }
            }
            other => return Err(format!("expected session resume command, got {other:?}")),
        }

        Ok(())
    }

    #[test]
    fn safe_command_cli_parses_session_overrides() -> Result<(), String> {
        let status = Cli::try_parse_from(["boundline", "status", "--session", "session-123"])
            .map_err(|error| error.to_string())?;
        match status.command {
            Some(DeveloperCommand::Status { session: Some(session_id), .. }) => {
                if session_id != "session-123" {
                    return Err(format!("expected session-123 status override, got {session_id}"));
                }
            }
            other => return Err(format!("expected status session override, got {other:?}")),
        }

        let next = Cli::try_parse_from(["boundline", "next", "--session", "session-123"])
            .map_err(|error| error.to_string())?;
        match next.command {
            Some(DeveloperCommand::Next { session: Some(session_id), .. }) => {
                if session_id != "session-123" {
                    return Err(format!("expected session-123 next override, got {session_id}"));
                }
            }
            other => return Err(format!("expected next session override, got {other:?}")),
        }

        let cont = Cli::try_parse_from(["boundline", "continue", "--session", "session-123"])
            .map_err(|error| error.to_string())?;
        match cont.command {
            Some(DeveloperCommand::Continue { session: Some(session_id), .. }) => {
                if session_id != "session-123" {
                    return Err(format!(
                        "expected session-123 continue override, got {session_id}"
                    ));
                }
            }
            other => return Err(format!("expected continue session override, got {other:?}")),
        }

        let inspect = Cli::try_parse_from(["boundline", "inspect", "--session", "session-123"])
            .map_err(|error| error.to_string())?;
        match inspect.command {
            Some(DeveloperCommand::Inspect { session: Some(session_id), .. }) => {
                if session_id != "session-123" {
                    return Err(format!("expected session-123 inspect override, got {session_id}"));
                }
            }
            other => return Err(format!("expected inspect session override, got {other:?}")),
        }

        let checkpoint_list =
            Cli::try_parse_from(["boundline", "checkpoint", "list", "--session", "session-123"])
                .map_err(|error| error.to_string())?;
        match checkpoint_list.command {
            Some(DeveloperCommand::Checkpoint {
                command: CheckpointSubcommand::List { session: Some(session_id), .. },
            }) => {
                if session_id != "session-123" {
                    return Err(format!(
                        "expected session-123 checkpoint list override, got {session_id}"
                    ));
                }
            }
            other => {
                return Err(format!("expected checkpoint list session override, got {other:?}"));
            }
        }

        let checkpoint_restore = Cli::try_parse_from([
            "boundline",
            "checkpoint",
            "restore",
            "checkpoint-1",
            "--session",
            "session-123",
        ])
        .map_err(|error| error.to_string())?;
        match checkpoint_restore.command {
            Some(DeveloperCommand::Checkpoint {
                command:
                    CheckpointSubcommand::Restore { checkpoint_id, session: Some(session_id), .. },
            }) => {
                if checkpoint_id != "checkpoint-1" {
                    return Err(format!("expected checkpoint-1, got {checkpoint_id}"));
                }
                if session_id != "session-123" {
                    return Err(format!(
                        "expected session-123 checkpoint restore override, got {session_id}"
                    ));
                }
            }
            other => {
                return Err(format!("expected checkpoint restore session override, got {other:?}"));
            }
        }

        Ok(())
    }

    #[test]
    fn dispatch_outcome_reopens_verbose_session_output() {
        let outcome = super::DispatchOutcome {
            exit_status: CommandExitStatus::Succeeded,
            output: "goal: compact\nlatest_status: succeeded\nnext_command: boundline inspect"
                .to_string(),
            host_output: None,
            stream_output: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            inspection_target: None,
            trace_location: None,
            session_status: Some(SessionStatusView {
                workspace_ref: "/tmp/workspace".to_string(),
                latest_status: SessionStatus::Succeeded,
                next_command: Some("boundline inspect".to_string()),
                explanation: "completed successfully".to_string(),
                ..SessionStatusView::default()
            }),
            guidance_guardian: Some(GuidanceGuardianProjection {
                capability_resolution_summary: Some("packs loaded".to_string()),
                ..GuidanceGuardianProjection::default()
            }),
            trace_summary: None,
        };

        assert_eq!(
            outcome.render_human_output(false),
            "goal: compact\nlatest_status: succeeded\nnext_command: boundline inspect"
        );

        let verbose = outcome.render_human_output(true);
        assert!(verbose.contains("workspace_ref: /tmp/workspace"), "{verbose}");
        assert!(verbose.contains("guidance_resolution_summary: packs loaded"), "{verbose}");
    }

    #[test]
    fn dispatch_outcome_prefers_compact_trace_brief_by_default() {
        let outcome = super::DispatchOutcome {
            exit_status: CommandExitStatus::NonSuccess,
            output: concat!(
                "inspection_target: latest-workspace-trace\n",
                "trace: /tmp/workspace/.boundline/traces/task.json\n",
                "next_command: boundline inspect --trace /tmp/workspace/.boundline/traces/task.json"
            )
            .to_string(),
            host_output: None,
            stream_output: None,
            inspection_target: Some("latest-workspace-trace".to_string()),
            trace_location: Some("/tmp/workspace/.boundline/traces/task.json".to_string()),
            session_status: None,
            guidance_guardian: None,
            compact_output: None,
            prefer_compact_output_in_verbose: false,
            trace_summary: Some(crate::domain::trace::TraceSummaryView {
                trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
                goal: "Repair the checkout regression".to_string(),
                routing_summary: Some(
                    "routing: compatibility (execution_profile) - declarative manifest remains authoritative"
                        .to_string(),
                ),
                terminal_status: crate::domain::task::TaskStatus::Failed,
                terminal_reason: crate::domain::task::TerminalReason::new(
                    crate::domain::limits::TerminalCondition::TaskNotCredible,
                    "clarification is still required before execution can continue",
                    None,
                ),
                ..crate::domain::trace::TraceSummaryView::default()
            }),
        };

        let compact = outcome.render_human_output(false);

        assert!(compact.contains("goal: Repair the checkout regression"), "{compact}");
        assert!(compact.contains("inspection_target: latest-workspace-trace"), "{compact}");
        assert!(compact.contains("latest_status: failed"), "{compact}");
        assert!(
            compact.contains(
                "next_command: boundline inspect --trace /tmp/workspace/.boundline/traces/task.json"
            ),
            "{compact}"
        );
        assert!(!compact.contains("trace:"), "{compact}");
    }

    #[test]
    fn config_cli_parses_chat_route_targets() -> Result<(), String> {
        let set_cli = Cli::try_parse_from([
            "boundline",
            "config",
            "set",
            "--scope",
            "workspace",
            "--chat",
            "--runtime",
            "codex",
            "--model",
            "openai/gpt-5.4",
        ])
        .map_err(|error| error.to_string())?;

        match set_cli.command {
            Some(DeveloperCommand::Config {
                command:
                    ConfigSubcommand::Set {
                        scope,
                        slot,
                        chat,
                        reviewer,
                        adjudicator,
                        runtime,
                        model,
                        ..
                    },
            }) => {
                if scope != ConfigWriteScope::Workspace {
                    return Err(format!("expected workspace scope for config set, got {scope:?}"));
                }
                if slot.is_some() {
                    return Err(format!("expected no slot target for config set, got {slot:?}"));
                }
                if !chat {
                    return Err("expected config set --chat to set chat=true".to_string());
                }
                if reviewer.is_some() {
                    return Err(format!(
                        "expected no reviewer target for config set, got {reviewer:?}"
                    ));
                }
                if adjudicator {
                    return Err("expected config set --chat to leave adjudicator unset".to_string());
                }
                if runtime != RuntimeKind::Codex {
                    return Err(format!("expected codex runtime for config set, got {runtime:?}"));
                }
                if model != "openai/gpt-5.4" {
                    return Err(format!("expected chat model openai/gpt-5.4, got {model}"));
                }
            }
            other => {
                return Err(format!("expected config set command with chat target, got {other:?}"));
            }
        }

        let unset_cli =
            Cli::try_parse_from(["boundline", "config", "unset", "--scope", "workspace", "--chat"])
                .map_err(|error| error.to_string())?;

        match unset_cli.command {
            Some(DeveloperCommand::Config {
                command: ConfigSubcommand::Unset { scope, slot, chat, reviewer, adjudicator, .. },
            }) => {
                if scope != ConfigWriteScope::Workspace {
                    return Err(format!(
                        "expected workspace scope for config unset, got {scope:?}"
                    ));
                }
                if slot.is_some() {
                    return Err(format!("expected no slot target for config unset, got {slot:?}"));
                }
                if !chat {
                    return Err("expected config unset --chat to set chat=true".to_string());
                }
                if reviewer.is_some() {
                    return Err(format!(
                        "expected no reviewer target for config unset, got {reviewer:?}"
                    ));
                }
                if adjudicator {
                    return Err(
                        "expected config unset --chat to leave adjudicator unset".to_string()
                    );
                }
            }
            other => {
                return Err(format!(
                    "expected config unset command with chat target, got {other:?}"
                ));
            }
        }

        Ok(())
    }

    #[test]
    fn dispatch_config_set_and_unset_chat_route_from_cli() -> Result<(), String> {
        let workspace = temp_workspace("boundline-cli-config-chat-dispatch");

        let set_cli = Cli::try_parse_from(vec![
            "boundline".to_string(),
            "config".to_string(),
            "set".to_string(),
            "--workspace".to_string(),
            workspace.to_string_lossy().into_owned(),
            "--scope".to_string(),
            "workspace".to_string(),
            "--chat".to_string(),
            "--runtime".to_string(),
            "codex".to_string(),
            "--model".to_string(),
            "openai/gpt-5.4".to_string(),
        ])
        .map_err(|error| error.to_string())?;
        let Some(set_command) = set_cli.command else {
            return Err("expected parsed config set command".to_string());
        };

        let set_outcome = dispatch(&set_command);
        if set_outcome.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected config set dispatch to succeed, got {:?}: {}",
                set_outcome.exit_status, set_outcome.output
            ));
        }
        if !set_outcome.output.contains("workspace config") {
            return Err(format!(
                "expected config set dispatch output to mention workspace config, got {}",
                set_outcome.output
            ));
        }

        let local = FileConfigStore::for_workspace(&workspace)
            .load_local()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected workspace config file after config set".to_string())?;
        let chat_route = local
            .routing
            .chat
            .ok_or_else(|| "expected routing.chat after config set".to_string())?;
        if chat_route.runtime != RuntimeKind::Codex {
            return Err(format!(
                "expected routing.chat runtime codex after config set, got {:?}",
                chat_route.runtime
            ));
        }
        if chat_route.model != "openai/gpt-5.4" {
            return Err(format!(
                "expected routing.chat model openai/gpt-5.4 after config set, got {}",
                chat_route.model
            ));
        }

        let unset_cli = Cli::try_parse_from(vec![
            "boundline".to_string(),
            "config".to_string(),
            "unset".to_string(),
            "--workspace".to_string(),
            workspace.to_string_lossy().into_owned(),
            "--scope".to_string(),
            "workspace".to_string(),
            "--chat".to_string(),
        ])
        .map_err(|error| error.to_string())?;
        let Some(unset_command) = unset_cli.command else {
            return Err("expected parsed config unset command".to_string());
        };

        let unset_outcome = dispatch(&unset_command);
        if unset_outcome.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected config unset dispatch to succeed, got {:?}: {}",
                unset_outcome.exit_status, unset_outcome.output
            ));
        }
        if !unset_outcome.output.contains("workspace config") {
            return Err(format!(
                "expected config unset dispatch output to mention workspace config, got {}",
                unset_outcome.output
            ));
        }

        let local = FileConfigStore::for_workspace(&workspace)
            .load_local()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected workspace config file after config unset".to_string())?;
        if local.routing.chat.is_some() {
            return Err("expected routing.chat to be removed after config unset".to_string());
        }

        Ok(())
    }

    fn write_execution_workspace(prefix: &str) -> PathBuf {
        let workspace = temp_workspace(prefix);
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
        fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
        fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
        fs::write(
            workspace.join(".boundline/execution.json"),
            serde_json::to_string_pretty(&json!({
                "name": "dispatch-execution",
                "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
                "validation_command": {
                    "program": "cargo",
                    "args": ["test", "--quiet"]
                },
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Replace subtraction with addition",
                        "failure_mode": "terminal",
                        "changes": [
                            {
                                "path": "src/lib.rs",
                                "find": "left - right",
                                "replace": "left + right"
                            }
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn write_context_brief(workspace: &Path) -> PathBuf {
        let brief = workspace.join("brief.md");
        fs::write(
            &brief,
            "Investigate src/lib.rs and tests/red_to_green.rs before broad scanning.\n",
        )
        .unwrap();
        brief
    }

    #[test]
    fn dispatch_covers_session_error_paths() {
        let goal_workspace = temp_workspace("boundline-cli-dispatch-goal-bootstrap");
        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(goal_workspace),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("goal".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);

        let workspace = temp_workspace("boundline-cli-dispatch-error");
        let commands = [
            DeveloperCommand::Flow {
                name: "bug-fix".to_string(),
                workspace: Some(workspace.clone()),
                cluster: None,
            },
            DeveloperCommand::Plan {
                workspace: Some(workspace.clone()),
                cluster: None,
                input: None,
                flow: None,
                no_flow: false,
                no_canon: false,
            },
            DeveloperCommand::Step { workspace: Some(workspace.clone()), cluster: None },
            DeveloperCommand::Next {
                workspace: Some(workspace.clone()),
                cluster: None,
                session: None,
            },
        ];

        for command in commands {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("session error"), "{}", outcome.output);
        }

        let status = dispatch(&DeveloperCommand::Status {
            workspace: Some(workspace.clone()),
            cluster: None,
            session: None,
        });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(status.output.contains("session_bootstrap"), "{}", status.output);

        let cont = dispatch(&DeveloperCommand::Continue {
            workspace: Some(workspace.clone()),
            cluster: None,
            session: None,
        });
        assert_eq!(cont.exit_status, CommandExitStatus::Succeeded);
        assert!(cont.output.contains("chat history is not authoritative"), "{}", cont.output);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(workspace),
            cluster: None,
            session: None,
            audit: false,
        });
        assert_eq!(inspect.exit_status, CommandExitStatus::TraceReadFailure);
        assert!(inspect.output.contains("inspect: trace read failure"), "{}", inspect.output);
    }

    #[test]
    fn dispatch_covers_successful_custom_run_session_run_and_inspect_paths() {
        let custom_workspace = write_execution_workspace("boundline-cli-dispatch-success-custom");
        let session_workspace = write_execution_workspace("boundline-cli-dispatch-success-session");
        let custom_brief = write_context_brief(&custom_workspace);
        let session_brief = write_context_brief(&session_workspace);

        let custom_run = dispatch(&DeveloperCommand::Run {
            workspace: Some(custom_workspace.clone()),
            cluster: None,
            goal: Some("Fix the failing add test".to_string()),
            compatibility: false,
            brief: vec![custom_brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(custom_run.exit_status, CommandExitStatus::Succeeded);
        assert!(custom_run.output.contains("terminal_status: succeeded"), "{}", custom_run.output);
        assert!(custom_run.trace_location.is_some());

        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("Fix the failing add test".to_string()),
            brief: vec![session_brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);

        let plan = dispatch(&DeveloperCommand::Plan {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            input: None,
            flow: Some("bug-fix".to_string()),
            no_flow: false,
            no_canon: false,
        });
        assert_eq!(plan.exit_status, CommandExitStatus::Succeeded);
        assert!(plan.output.contains("execution_path: native_goal_plan"), "{}", plan.output);

        let run = dispatch(&DeveloperCommand::Run {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            goal: None,
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(run.output.contains("terminal_status: succeeded"), "{}", run.output);

        let status = dispatch(&DeveloperCommand::Status {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            session: None,
        });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);

        let next = dispatch(&DeveloperCommand::Next {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            session: None,
        });
        assert_eq!(next.exit_status, CommandExitStatus::Succeeded);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(session_workspace.clone()),
            cluster: None,
            session: None,
            audit: false,
        });
        assert_eq!(inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(inspect.output.contains("inspection_target:"), "{}", inspect.output);

        let audit_inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(session_workspace.clone()),
            cluster: None,
            session: None,
            audit: true,
        });
        assert_eq!(audit_inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(audit_inspect.output.contains("audit_timeline:"), "{}", audit_inspect.output);

        let invalid_workspace = temp_workspace("boundline-cli-dispatch-invalid");
        let invalid = dispatch(&DeveloperCommand::Run {
            workspace: Some(invalid_workspace),
            cluster: None,
            goal: Some("Fix the failing add test".to_string()),
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(invalid.exit_status, CommandExitStatus::InvalidInvocation);
        assert!(invalid.output.contains("bounded context required"), "{}", invalid.output);
    }

    #[test]
    fn dispatch_custom_run_defaults_workspace_to_current_directory() {
        let workspace = write_execution_workspace("boundline-cli-dispatch-default-custom");
        let brief = write_context_brief(&workspace);
        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);

        let run = dispatch(&DeveloperCommand::Run {
            workspace: None,
            cluster: None,
            goal: Some("Fix the failing add test".to_string()),
            compatibility: false,
            brief: vec![brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });

        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(run.output.contains("terminal_status: succeeded"), "{}", run.output);
        assert!(run.trace_location.is_some());
    }

    #[test]
    fn dispatch_session_commands_default_workspace_to_current_directory() {
        let workspace = write_execution_workspace("boundline-cli-dispatch-default-session");
        let brief = write_context_brief(&workspace);
        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);

        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: None,
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("Fix the failing add test".to_string()),
            brief: vec![brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);

        let plan = dispatch(&DeveloperCommand::Plan {
            workspace: None,
            cluster: None,
            input: None,
            flow: Some("bug-fix".to_string()),
            no_flow: false,
            no_canon: false,
        });
        assert_eq!(plan.exit_status, CommandExitStatus::Succeeded, "{}", plan.output);
        assert!(plan.output.contains("goal_plan_state: confirmed"), "{}", plan.output);
        assert!(plan.output.contains("goal_plan_revision: 1"), "{}", plan.output);

        let run = dispatch(&DeveloperCommand::Run {
            workspace: None,
            cluster: None,
            goal: None,
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(run.output.contains("terminal_status: succeeded"), "{}", run.output);

        let status =
            dispatch(&DeveloperCommand::Status { workspace: None, cluster: None, session: None });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(status.output.contains("latest_status: succeeded"), "{}", status.output);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: None,
            cluster: None,
            session: None,
            audit: false,
        });
        assert_eq!(inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(inspect.output.contains("inspection_target:"), "{}", inspect.output);
    }

    #[test]
    fn goal_upsert_creates_session_when_none_exists() {
        let workspace = write_execution_workspace("boundline-cli-goal-upsert-create");

        // Default goal (no --new, no --update) should create when no session exists.
        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("Implement user registration endpoint".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);

        let store = FileSessionStore::for_workspace(&workspace);
        let record = store.load().unwrap().unwrap();
        assert_eq!(record.latest_status, SessionStatus::GoalCaptured);
    }

    #[test]
    fn goal_upsert_updates_active_non_terminal_session() {
        let workspace = write_execution_workspace("boundline-cli-goal-upsert-update");

        // Create initial session.
        let first = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: true,
            goal: Some("First goal text".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);

        let store = FileSessionStore::for_workspace(&workspace);
        let first_record = store.load().unwrap().unwrap();
        let first_id = first_record.session_id.clone();

        // Default goal on an active non-terminal session should UPDATE (same session_id).
        let second = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("Refined goal text".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);

        let updated_record = store.load().unwrap().unwrap();
        assert_eq!(
            updated_record.session_id, first_id,
            "upsert should update existing session, not create a new one"
        );
    }

    #[test]
    fn goal_new_flag_creates_second_session() {
        let workspace = write_execution_workspace("boundline-cli-goal-new-flag");

        // Create initial session via --new.
        let first = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: true,
            goal: Some("First session".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);

        let store = FileSessionStore::for_workspace(&workspace);
        let first_id = store.load().unwrap().unwrap().session_id.clone();

        // --new should always create a second session.
        let second = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: true,
            goal: Some("Second session".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);

        let second_id = store.load().unwrap().unwrap().session_id.clone();
        assert_ne!(second_id, first_id, "--new should create a distinct session");
    }

    #[test]
    fn session_history_dispatch_covers_list_and_resume_paths() -> Result<(), String> {
        let workspace = write_execution_workspace("boundline-cli-dispatch-session-history");
        let brief = write_context_brief(&workspace);

        let first_goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: true,
            goal: Some("Fix the failing add test".to_string()),
            brief: vec![brief.clone()],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        if first_goal.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected first goal dispatch to succeed, got {:?}: {}",
                first_goal.exit_status, first_goal.output
            ));
        }
        let store = FileSessionStore::for_workspace(&workspace);
        let first_record = store
            .load()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected first persisted session".to_string())?;

        let second_goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: true,
            goal: Some("Ship the follow-up cleanup".to_string()),
            brief: vec![brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        if second_goal.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected second goal dispatch to succeed, got {:?}: {}",
                second_goal.exit_status, second_goal.output
            ));
        }

        let list = dispatch(&DeveloperCommand::Session {
            command: SessionSubcommand::List { workspace: Some(workspace.clone()), cluster: None },
        });
        if list.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected session list dispatch to succeed, got {:?}: {}",
                list.exit_status, list.output
            ));
        }
        if !list.output.contains(&first_record.session_id) {
            return Err(format!(
                "expected session list to contain {}, got {}",
                first_record.session_id, list.output
            ));
        }

        let resume = dispatch(&DeveloperCommand::Session {
            command: SessionSubcommand::Resume {
                session_id: first_record.session_id.clone(),
                workspace: Some(workspace.clone()),
                cluster: None,
            },
        });
        if resume.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected session resume dispatch to succeed, got {:?}: {}",
                resume.exit_status, resume.output
            ));
        }

        let resumed_record = store
            .load()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "expected resumed active session".to_string())?;
        if resumed_record.session_id != first_record.session_id {
            return Err(format!(
                "expected resumed session {}, got {}",
                first_record.session_id, resumed_record.session_id
            ));
        }

        Ok(())
    }

    #[test]
    fn command_names_and_dispatch_cover_remaining_command_variants() {
        for (name, expected) in [
            (CommandName::Checkpoint, "checkpoint"),
            (CommandName::Orchestrate, "orchestrate"),
            (CommandName::Workflow, "workflow"),
            (CommandName::Inspect, "inspect"),
            (CommandName::Continue, "continue"),
            (CommandName::Session, "session"),
            (CommandName::Govern, "govern"),
            (CommandName::Init, "init"),
            (CommandName::Update, "update"),
            (CommandName::Assistant, "assistant"),
            (CommandName::Config, "config"),
            (CommandName::Cluster, "cluster"),
        ] {
            assert_eq!(name.as_str(), expected);
            assert_eq!(name.to_string(), expected);
        }

        let workspace = temp_workspace("boundline-cli-dispatch-coverage");
        let default_workspace = temp_workspace("boundline-cli-dispatch-default-cwd");
        let _default_current_dir_guard = CurrentDirGuard::change_to(&default_workspace);
        for (command, expected) in [
            (
                DeveloperCommand::Checkpoint {
                    command: CheckpointSubcommand::List {
                        workspace: Some(workspace.clone()),
                        cluster: None,
                        session: None,
                    },
                },
                CommandName::Checkpoint,
            ),
            (
                DeveloperCommand::Orchestrate {
                    workspace: Some(workspace.clone()),
                    cluster: None,
                    goal: Some("Plan a delivery".to_string()),
                    brief: Vec::new(),
                    flow: None,
                    governance: None,
                    risk: None,
                    zone: None,
                    owner: None,
                    intent: OrchestrateIntent::ContinueUntilPhaseRequest,
                    planning_stage_complete: None,
                    request_id: None,
                    answer: None,
                    assistant_host: None,
                    json_stream: true,
                    no_canon: false,
                    slug: None,
                },
                CommandName::Orchestrate,
            ),
            (
                DeveloperCommand::Workflow {
                    command: WorkflowSubcommand::List { workspace: Some(workspace.clone()) },
                },
                CommandName::Workflow,
            ),
            (
                DeveloperCommand::Inspect {
                    trace: None,
                    workspace: Some(workspace.clone()),
                    cluster: None,
                    session: None,
                    audit: false,
                },
                CommandName::Inspect,
            ),
            (
                DeveloperCommand::Session {
                    command: SessionSubcommand::List {
                        workspace: Some(workspace.clone()),
                        cluster: None,
                    },
                },
                CommandName::Session,
            ),
            (
                DeveloperCommand::Status {
                    workspace: Some(workspace.clone()),
                    cluster: None,
                    session: None,
                },
                CommandName::Status,
            ),
            (
                DeveloperCommand::Next {
                    workspace: Some(workspace.clone()),
                    cluster: None,
                    session: None,
                },
                CommandName::Next,
            ),
            (
                DeveloperCommand::Init {
                    scope: InitConfigScope::Workspace,
                    workspace: workspace.clone(),
                    non_interactive: false,
                    template: None,
                    assistant: Vec::new(),
                    ide: Vec::new(),
                    auto_approve: None,
                    route: Vec::new(),
                    domain: Vec::new(),
                    domain_standard: Vec::new(),
                    context_binding: Vec::new(),
                    required_context_binding: Vec::new(),
                    canon_mode_selection: None,
                    risk: None,
                    zone: None,
                    owner: None,
                    export_docs: false,
                    refresh: false,
                    diff: false,
                    to: None,
                    force: false,
                },
                CommandName::Init,
            ),
            (
                DeveloperCommand::Update {
                    workspace: workspace.clone(),
                    target: Vec::new(),
                    ide: Vec::new(),
                    auto_approve: None,
                    template: None,
                    diff: false,
                    apply: false,
                    adopt: false,
                    prune: false,
                    status: false,
                    force: false,
                },
                CommandName::Update,
            ),
            (
                DeveloperCommand::Config {
                    command: ConfigSubcommand::Show {
                        workspace: Some(workspace.clone()),
                        cluster: None,
                        scope: Some(ConfigShowScope::Workspace),
                    },
                },
                CommandName::Config,
            ),
            (
                DeveloperCommand::Cluster {
                    command: ClusterSubcommand::Status { workspace: workspace.clone() },
                },
                CommandName::Cluster,
            ),
        ] {
            assert_eq!(command.name(), expected);
        }

        let missing = workspace.join("missing-workspace");
        let missing_member = workspace.join("missing-member");
        let file_workspace = workspace.join("workspace-file");
        fs::write(&file_workspace, "not a directory").unwrap();
        let config_workspace = temp_workspace("boundline-cli-config-dispatch");

        let checkpoint_session =
            DeveloperCommandSession::from_command(&DeveloperCommand::Checkpoint {
                command: CheckpointSubcommand::List {
                    workspace: Some(workspace.clone()),
                    cluster: None,
                    session: None,
                },
            });
        assert!(checkpoint_session.validate().is_ok());

        let session_session = DeveloperCommandSession::from_command(&DeveloperCommand::Session {
            command: SessionSubcommand::List { workspace: Some(workspace.clone()), cluster: None },
        });
        assert!(session_session.validate().is_ok());
        assert_eq!(session_session.command_name, CommandName::Session);

        let orchestrate_session =
            DeveloperCommandSession::from_command(&DeveloperCommand::Orchestrate {
                workspace: Some(workspace.clone()),
                cluster: None,
                goal: Some("Plan a delivery".to_string()),
                brief: Vec::new(),
                flow: None,
                governance: None,
                risk: None,
                zone: None,
                owner: None,
                intent: OrchestrateIntent::ContinueUntilPhaseRequest,
                planning_stage_complete: None,
                request_id: None,
                answer: None,
                assistant_host: None,
                json_stream: true,
                no_canon: false,
                slug: None,
            });
        assert!(orchestrate_session.validate().is_ok());
        assert_eq!(orchestrate_session.command_name, CommandName::Orchestrate);
        assert_eq!(
            orchestrate_session.workspace_ref,
            Some(workspace.to_string_lossy().into_owned())
        );

        let orchestrate = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Plan a delivery".to_string()),
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });
        assert_eq!(orchestrate.exit_status, CommandExitStatus::Succeeded);
        assert!(orchestrate.stream_output.is_some());
        let orchestrate_stream = orchestrate.stream_output.as_deref().unwrap_or_default();
        assert!(orchestrate_stream.contains("\"event_kind\":\"session_opened\""));
        assert!(orchestrate_stream.contains("\"event_kind\":\"phase_request\""));

        let checkpoint = dispatch(&DeveloperCommand::Checkpoint {
            command: CheckpointSubcommand::List {
                workspace: Some(workspace.clone()),
                cluster: None,
                session: None,
            },
        });
        assert_eq!(checkpoint.exit_status, CommandExitStatus::Succeeded);
        assert!(checkpoint.output.contains("checkpoint_scope: workspace"), "{}", checkpoint.output);

        let session_list = dispatch(&DeveloperCommand::Session {
            command: SessionSubcommand::List { workspace: Some(workspace.clone()), cluster: None },
        });
        assert_eq!(session_list.exit_status, CommandExitStatus::Succeeded);
        assert!(session_list.output.contains("session_history:"), "{}", session_list.output);

        assert_eq!(
            dispatch(&DeveloperCommand::Doctor {
                workspace: Some(temp_workspace("boundline-cli-doctor-invalid")),
                install: false,
            })
            .exit_status,
            CommandExitStatus::InvalidInvocation
        );
        let install_status =
            dispatch(&DeveloperCommand::Doctor { workspace: None, install: true }).exit_status;
        assert!(matches!(
            install_status,
            CommandExitStatus::Succeeded | CommandExitStatus::InvalidInvocation
        ));
        assert_eq!(
            dispatch(&DeveloperCommand::Run {
                workspace: None,
                cluster: None,
                goal: Some("Fix the failing add test".to_string()),
                compatibility: false,
                brief: Vec::new(),
                governance: None,
                risk: None,
                zone: None,
                owner: None,
                mode: None,
                no_canon: false,
            })
            .exit_status,
            CommandExitStatus::InvalidInvocation
        );

        for command in [
            DeveloperCommand::Workflow {
                command: WorkflowSubcommand::List { workspace: Some(missing.clone()) },
            },
            DeveloperCommand::Workflow {
                command: WorkflowSubcommand::Run {
                    name: "default".to_string(),
                    workspace: Some(missing.clone()),
                    goal: None,
                },
            },
            DeveloperCommand::Workflow {
                command: WorkflowSubcommand::Status { workspace: Some(missing.clone()) },
            },
            DeveloperCommand::Workflow {
                command: WorkflowSubcommand::Resume { workspace: Some(missing.clone()) },
            },
            DeveloperCommand::Workflow {
                command: WorkflowSubcommand::Inspect { workspace: Some(missing.clone()) },
            },
        ] {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("workflow error:"), "{}", outcome.output);
        }

        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: None,
            cluster: Some(missing.clone()),
            update: false,
            new_session: false,
            goal: Some("bootstrap clustered delivery".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::NonSuccess);
        assert!(goal.output.contains("session error"), "{}", goal.output);

        let init = dispatch(&DeveloperCommand::Init {
            scope: InitConfigScope::Workspace,
            workspace: file_workspace,
            non_interactive: false,
            template: None,
            assistant: Vec::new(),
            ide: Vec::new(),
            auto_approve: None,
            route: Vec::new(),
            domain: Vec::new(),
            domain_standard: Vec::new(),
            context_binding: Vec::new(),
            required_context_binding: Vec::new(),
            canon_mode_selection: None,
            risk: None,
            zone: None,
            owner: None,
            export_docs: false,
            refresh: false,
            diff: false,
            to: None,
            force: false,
        });
        assert_eq!(init.exit_status, CommandExitStatus::NonSuccess);
        assert!(init.output.contains("init error:"), "{}", init.output);

        let update = dispatch(&DeveloperCommand::Update {
            workspace: missing.clone(),
            target: Vec::new(),
            ide: Vec::new(),
            auto_approve: None,
            template: None,
            diff: false,
            apply: false,
            adopt: false,
            prune: false,
            status: false,
            force: false,
        });
        assert_eq!(update.exit_status, CommandExitStatus::NonSuccess);
        assert!(update.output.contains("update error:"), "{}", update.output);

        // Init success path: dispatch with a real temp workspace and explicit values
        let init_success_workspace = temp_workspace("boundline-cli-init-dispatch-success");
        let init_ok = dispatch(&DeveloperCommand::Init {
            scope: InitConfigScope::Workspace,
            workspace: init_success_workspace.clone(),
            non_interactive: true,
            template: Some(InitTemplate::Change),
            assistant: vec![crate::domain::configuration::AssistantHostKind::Copilot],
            ide: Vec::new(),
            auto_approve: None,
            route: Vec::new(),
            domain: Vec::new(),
            domain_standard: Vec::new(),
            context_binding: Vec::new(),
            required_context_binding: Vec::new(),
            canon_mode_selection: Some(CanonModeSelectionPreference::AutoConfirm),
            risk: None,
            zone: None,
            owner: None,
            export_docs: true,
            refresh: false,
            diff: false,
            to: None,
            force: true,
        });
        assert_eq!(init_ok.exit_status, CommandExitStatus::Succeeded, "{}", init_ok.output);
        assert!(init_ok.output.contains("init: workspace initialized"), "{}", init_ok.output);
        assert!(init_ok.output.contains("docs_export:"), "{}", init_ok.output);
        assert!(init_success_workspace.join("docs/boundline/canon.md").exists());

        let update_ok = dispatch(&DeveloperCommand::Update {
            workspace: init_success_workspace,
            target: Vec::new(),
            ide: Vec::new(),
            auto_approve: None,
            template: None,
            diff: false,
            apply: false,
            adopt: false,
            prune: false,
            status: false,
            force: false,
        });
        assert_eq!(update_ok.exit_status, CommandExitStatus::Succeeded, "{}", update_ok.output);
        assert!(update_ok.output.contains("update: preview only"), "{}", update_ok.output);

        for command in [
            DeveloperCommand::Config {
                command: ConfigSubcommand::Show {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: Some(ConfigShowScope::Workspace),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::Set {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    slot: Some(RouteSlot::Planning),
                    chat: false,
                    reviewer: None,
                    adjudicator: false,
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-4o".to_string(),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::Unset {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    slot: Some(RouteSlot::Planning),
                    chat: false,
                    reviewer: None,
                    adjudicator: false,
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::SetSemanticAcceleration {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    policy: SemanticAccelerationPolicyState::Local,
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::SetCapability {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    runtime: RuntimeKind::Codex,
                    continuation: CapabilityState::Supported,
                    resume: CapabilityState::Supported,
                    validation: CapabilityState::Supported,
                    handoff_target: CapabilityState::Supported,
                    escalation_context: CapabilityState::Supported,
                    notes: Some("supports the default route".to_string()),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::UnsetCapability {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    runtime: RuntimeKind::Codex,
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::SetEffort {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    slot: RouteSlot::Planning,
                    level: EffortLevel::High,
                    fallback: EffortFallbackPolicy::AllowLower,
                    rationale: Some("planning should stay thorough".to_string()),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::UnsetEffort {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    slot: RouteSlot::Planning,
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::SetDomain {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    family: DomainFamily::React,
                    enable: true,
                    disable: false,
                    standards: Some("workspace react rules".to_string()),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::BindContext {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    family: DomainFamily::React,
                    kind: ExternalContextKind::DesignSystem,
                    reference: "mcp:design-system".to_string(),
                    required: true,
                    notes: Some("shared system".to_string()),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::UnbindContext {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    family: DomainFamily::React,
                    kind: ExternalContextKind::DesignSystem,
                    reference: "mcp:design-system".to_string(),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::UnsetDomain {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    family: DomainFamily::React,
                },
            },
        ] {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::Succeeded);
            assert!(outcome.output.contains("config:"), "{}", outcome.output);
        }

        let config_error = dispatch(&DeveloperCommand::Config {
            command: ConfigSubcommand::Show {
                workspace: None,
                cluster: None,
                scope: Some(ConfigShowScope::Workspace),
            },
        });
        assert_eq!(config_error.exit_status, CommandExitStatus::NonSuccess);
        assert!(config_error.output.contains("config error:"), "{}", config_error.output);

        for command in [
            DeveloperCommand::Cluster {
                command: ClusterSubcommand::Init {
                    workspace: missing.clone(),
                    cluster_id: "cluster-coverage".to_string(),
                    member: vec![missing.clone(), missing_member.clone()],
                },
            },
            DeveloperCommand::Cluster {
                command: ClusterSubcommand::Status { workspace: missing.clone() },
            },
            DeveloperCommand::Cluster {
                command: ClusterSubcommand::Inspect { workspace: missing.clone() },
            },
        ] {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("cluster error:"), "{}", outcome.output);
        }
    }

    #[test]
    fn continue_govern_and_assistant_commands_cover_new_dispatch_paths() {
        let workspace = write_execution_workspace("boundline-cli-govern-assistant");

        let continue_command = DeveloperCommand::Continue {
            workspace: Some(workspace.clone()),
            cluster: None,
            session: None,
        };
        assert_eq!(continue_command.name(), CommandName::Continue);
        let continue_session = DeveloperCommandSession::from_command(&continue_command);
        assert_eq!(continue_session.command_name, CommandName::Continue);
        assert_eq!(
            continue_session.workspace_ref.as_deref(),
            Some(workspace.to_string_lossy().as_ref())
        );
        assert!(continue_session.validate().is_ok());

        let govern_command = DeveloperCommand::Govern {
            workspace: Some(workspace.clone()),
            mode: Some(CanonMode::Review),
            goal: Some("Prepare review packet".to_string()),
            brief: Vec::new(),
            base: None,
            head: None,
            risk: None,
            structural_impact: false,
            public_contract_change: false,
            validation_exhausted: false,
            pr_ready: false,
            preserved_behavior_evidence: false,
        };
        assert_eq!(govern_command.name(), CommandName::Govern);
        let govern_session = DeveloperCommandSession::from_command(&govern_command);
        assert_eq!(govern_session.command_name, CommandName::Govern);
        assert_eq!(govern_session.goal.as_deref(), Some("Prepare review packet"));
        assert!(govern_session.validate().is_ok());

        let govern_without_session = dispatch(&govern_command);
        assert_eq!(govern_without_session.exit_status, CommandExitStatus::Succeeded);
        assert!(
            govern_without_session.output.contains("govern: staged"),
            "{}",
            govern_without_session.output
        );
        assert!(
            govern_without_session.output.contains("mode: review"),
            "{}",
            govern_without_session.output
        );

        let goal = dispatch(&DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("bootstrap govern session".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        });
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);

        let govern_with_session = dispatch(&govern_command);
        assert_eq!(govern_with_session.exit_status, CommandExitStatus::Succeeded);
        assert!(
            govern_with_session.output.contains("govern: staged"),
            "{}",
            govern_with_session.output
        );
        assert!(
            govern_with_session.output.contains("mode: review"),
            "{}",
            govern_with_session.output
        );

        let assistant_command = DeveloperCommand::Assistant {
            command: AssistantSubcommand::Install {
                host: AssistantHost::Copilot,
                scope: AssistantInstallScope::User,
            },
        };
        assert_eq!(assistant_command.name(), CommandName::Assistant);
        let assistant_session = DeveloperCommandSession::from_command(&assistant_command);
        assert_eq!(assistant_session.command_name, CommandName::Assistant);
        assert!(assistant_session.workspace_ref.is_none());
        assert!(assistant_session.validate().is_ok());

        let assistant = dispatch(&assistant_command);
        assert_eq!(assistant.exit_status, CommandExitStatus::Succeeded);
        assert!(assistant.output.contains("assistant_global_package:"), "{}", assistant.output);
        assert!(assistant.output.contains("host: copilot"), "{}", assistant.output);
    }

    #[test]
    fn cli_covers_doctor_without_workspace_and_custom_run_validation_paths() {
        // Doctor without workspace → validation error (covers lines 1185-1190).
        let doctor = dispatch(&DeveloperCommand::Doctor { workspace: None, install: false });
        assert_eq!(doctor.exit_status, CommandExitStatus::InvalidInvocation);
        assert!(doctor.output.contains("workspace"), "{}", doctor.output);

        // DeveloperCommandSession::from_command for Run with cluster (no custom flags)
        // exercises the workspace_ref cluster-fallback branch (lines 764-770).
        let cluster = temp_workspace("boundline-cli-session-run-cluster");
        let session = DeveloperCommandSession::from_command(&DeveloperCommand::Run {
            workspace: None,
            cluster: Some(cluster.clone()),
            goal: None,
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(session.command_name, CommandName::Run);
        assert_eq!(session.workspace_ref.as_deref(), Some(cluster.to_string_lossy().as_ref()));

        // DeveloperCommandSession::from_command for Config SetCanon covers lines 907-913.
        let config_ws = temp_workspace("boundline-cli-session-config-setcanon");
        let canon_session = DeveloperCommandSession::from_command(&DeveloperCommand::Config {
            command: ConfigSubcommand::SetCanon {
                workspace: Some(config_ws.clone()),
                mode_selection: CanonModeSelectionPreference::AutoConfirm,
            },
        });
        assert_eq!(canon_session.command_name, CommandName::Config);
        assert_eq!(
            canon_session.workspace_ref.as_deref(),
            Some(config_ws.to_string_lossy().as_ref())
        );

        // dispatch_custom_run with a file path (not a dir) → InvalidInvocation (lines 1294-1297).
        let file_ws = temp_workspace("boundline-cli-custom-run-file");
        let file_path = file_ws.join("not-a-dir");
        std::fs::write(&file_path, "not a directory").unwrap();
        let file_run = dispatch(&DeveloperCommand::Run {
            workspace: Some(file_path),
            cluster: None,
            goal: Some("fix".to_string()),
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(file_run.exit_status, CommandExitStatus::InvalidInvocation);

        // dispatch_custom_run with workspace that fails native-direct-run diagnostics
        // (lines 1337-1342): workspace is a valid dir but has no .boundline/execution.json.
        let bare_ws = temp_workspace("boundline-cli-custom-run-bare");
        let bare_run = dispatch(&DeveloperCommand::Run {
            workspace: Some(bare_ws),
            cluster: None,
            goal: Some("fix".to_string()),
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
        });
        assert_eq!(bare_run.exit_status, CommandExitStatus::InvalidInvocation);
        assert!(bare_run.output.contains("bounded context required"), "{}", bare_run.output);
    }

    #[test]
    fn orchestrate_cli_parses_stream_intent_and_goal() {
        let cli = Cli::try_parse_from([
            "boundline",
            "orchestrate",
            "--workspace",
            "/tmp/workspace",
            "--goal",
            "Prepare architecture brief",
            "--assistant-host",
            "copilot",
            "--until",
            "phase-request",
            "--json-stream",
        ])
        .expect("orchestrate command should parse");

        let Some(DeveloperCommand::Orchestrate {
            workspace,
            goal,
            intent,
            planning_stage_complete,
            assistant_host,
            json_stream,
            ..
        }) = cli.command
        else {
            panic!("expected orchestrate command");
        };

        assert_eq!(workspace, Some(PathBuf::from("/tmp/workspace")));
        assert_eq!(goal.as_deref(), Some("Prepare architecture brief"));
        assert_eq!(intent, OrchestrateIntent::ContinueUntilPhaseRequest);
        assert!(planning_stage_complete.is_none());
        assert_eq!(assistant_host, Some(crate::cli::assistant_assets::AssistantHost::Copilot));
        assert!(json_stream);
    }

    #[test]
    fn goal_cli_parses_update_flag() {
        let cli = Cli::try_parse_from([
            "boundline",
            "goal",
            "--workspace",
            "/tmp/workspace",
            "--update",
            "--goal",
            "Refine architecture brief",
        ])
        .expect("goal update command should parse");

        let Some(DeveloperCommand::Goal { workspace, update, goal, .. }) = cli.command else {
            panic!("expected goal command");
        };

        assert_eq!(workspace, Some(PathBuf::from("/tmp/workspace")));
        assert!(update);
        assert_eq!(goal.as_deref(), Some("Refine architecture brief"));
    }

    #[test]
    fn orchestrate_cli_parses_planning_stage_completion_resume() {
        let cli = Cli::try_parse_from([
            "boundline",
            "orchestrate",
            "--workspace",
            "/tmp/workspace",
            "--planning-stage-complete",
            "plan:requirements",
            "--request-id",
            "req-session-planning-plan-requirements-review",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ])
        .expect("orchestrate planning-stage completion command should parse");

        let Some(DeveloperCommand::Orchestrate {
            workspace,
            planning_stage_complete,
            request_id,
            intent,
            json_stream,
            ..
        }) = cli.command
        else {
            panic!("expected orchestrate command");
        };

        assert_eq!(workspace, Some(PathBuf::from("/tmp/workspace")));
        assert_eq!(planning_stage_complete.as_deref(), Some("plan:requirements"));
        assert_eq!(request_id.as_deref(), Some("req-session-planning-plan-requirements-review"));
        assert_eq!(intent, OrchestrateIntent::ContinueUntilPhaseRequest);
        assert!(json_stream);
    }

    #[test]
    fn orchestrate_cli_parses_goal_clarification_answer_resume() {
        let cli = Cli::try_parse_from([
            "boundline",
            "orchestrate",
            "--workspace",
            "/tmp/workspace",
            "--request-id",
            "req-session-goal-goal-persistence-store",
            "--answer",
            "Postgres",
            "--intent",
            "continue-until-phase-request",
            "--json-stream",
        ])
        .expect("orchestrate goal clarification answer command should parse");

        let Some(DeveloperCommand::Orchestrate {
            workspace,
            request_id,
            answer,
            intent,
            json_stream,
            ..
        }) = cli.command
        else {
            panic!("expected orchestrate command");
        };

        assert_eq!(workspace, Some(PathBuf::from("/tmp/workspace")));
        assert_eq!(request_id.as_deref(), Some("req-session-goal-goal-persistence-store"));
        assert_eq!(answer.as_deref(), Some("Postgres"));
        assert_eq!(intent, OrchestrateIntent::ContinueUntilPhaseRequest);
        assert!(json_stream);
    }

    #[test]
    fn orchestrate_cli_accepts_legacy_intent_values() {
        let cli = Cli::try_parse_from([
            "boundline",
            "orchestrate",
            "--workspace",
            "/tmp/workspace",
            "--intent",
            "continue-until-terminal",
        ])
        .expect("legacy orchestrate intent value should parse");

        let Some(DeveloperCommand::Orchestrate { intent, .. }) = cli.command else {
            panic!("expected orchestrate command");
        };

        assert_eq!(intent, OrchestrateIntent::ContinueUntilTerminal);
    }

    #[test]
    fn orchestrate_dispatch_emits_phase_request_stream() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-dispatch");

        let outcome = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: Some("Prepare a bounded plan".to_string()),
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(outcome.exit_status, CommandExitStatus::Succeeded);
        let stream = outcome.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"event_kind\":\"session_opened\""), "{stream}");
        assert!(stream.contains("\"artifact_kind\":\"plan_brief\""), "{stream}");
        assert!(stream.contains("\"phase_kind\":\"execution\""), "{stream}");
    }

    #[test]
    fn orchestrate_dispatch_updates_active_session_on_follow_up_goal_input() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-goal-update");
        let brief = write_context_brief(&workspace);

        let first = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Prepare a bounded plan".to_string()),
            brief: vec![brief.clone()],
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::PlanOnly,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);
        let store = FileSessionStore::for_workspace(&workspace);
        let first_record = store.load().unwrap().unwrap();

        let second = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Refine the same bounded plan".to_string()),
            brief: vec![brief],
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);
        let second_record = store.load().unwrap().unwrap();
        assert_eq!(first_record.session_id, second_record.session_id);

        let stream = second.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"event_kind\":\"session_updated\""), "{stream}");
        assert!(
            stream.contains("updated the active session and captured the requested goal"),
            "{stream}"
        );
    }

    #[test]
    fn orchestrate_dispatch_emits_next_planning_stage_phase_request() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-governed-dispatch");
        let brief_path = workspace.join("brief.md");
        std::fs::write(
            &brief_path,
            concat!(
                "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
                "Authoritative persistence store: workspace-local .boundline/session.json.\n",
                "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
                "In-scope API operations: start, goal, plan, and orchestrate for the first slice.\n",
                "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
                "Success criteria: the first governed slice can progress through the next planning stage with reusable planning artifacts.\n",
                "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
            ),
        )
        .unwrap();

        let outcome = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: Some("Deliver a governed feature".to_string()),
            brief: vec![brief_path],
            flow: Some("delivery".to_string()),
            governance: Some(crate::domain::governance::GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(outcome.exit_status, CommandExitStatus::Succeeded);
        let stream = outcome.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"event_kind\":\"phase_request\""), "{stream}");
        assert!(stream.contains("\"stage_key\":\"plan:requirements\""), "{stream}");
        assert!(stream.contains("\"artifact_kind\":\"planning_stage_brief\""), "{stream}");
        assert!(stream.contains("\"phase_request\":{"), "{stream}");
        assert!(stream.contains("\"request_id\":\"req-"), "{stream}");
        assert!(stream.contains("\"kind\":\"clarification\""), "{stream}");
        assert!(
            stream.contains(
                "\"question\":\"Is the requirements planning brief ready to resume orchestration?\""
            ),
            "{stream}"
        );
        assert!(stream.contains("\"type\":\"suggested_choice\""), "{stream}");
        assert!(stream.contains("\"label\":\"fill using context\""), "{stream}");
        assert!(stream.contains("\"label\":\"provide reference path\""), "{stream}");
        assert!(!stream.contains("\"stage_key\":\"plan:architecture\""), "{stream}");
        assert!(
            stream.contains("--planning-stage-complete plan:requirements --until phase-request"),
            "{stream}"
        );
        assert!(stream.contains("--request-id req-"), "{stream}");
    }

    #[test]
    fn orchestrate_dispatch_advances_planning_stage_resume_cursor() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-stage-resume");
        let brief_path = workspace.join("brief.md");
        std::fs::write(
            &brief_path,
            concat!(
                "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
                "Authoritative persistence store: workspace-local .boundline/session.json.\n",
                "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
                "In-scope API operations: start, goal, plan, and orchestrate for the first slice.\n",
                "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
                "Success criteria: the first governed slice can progress through the next planning stage with reusable planning artifacts.\n",
                "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
            ),
        )
        .unwrap();

        let first = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Deliver a governed feature".to_string()),
            brief: vec![brief_path],
            flow: Some("delivery".to_string()),
            governance: Some(crate::domain::governance::GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);
        let first_stream = first.stream_output.as_deref().unwrap_or_default();
        assert!(first_stream.contains("\"stage_key\":\"plan:requirements\""), "{first_stream}");
        assert!(!first_stream.contains("\"stage_key\":\"plan:architecture\""), "{first_stream}");

        let second = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: None,
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: Some("plan:requirements".to_string()),
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);
        let second_stream = second.stream_output.as_deref().unwrap_or_default();
        assert!(
            second_stream.contains("recorded host completion for planning stage plan:requirements"),
            "{second_stream}"
        );
        assert!(second_stream.contains("\"stage_key\":\"plan:architecture\""), "{second_stream}");
        assert!(!second_stream.contains("\"stage_key\":\"plan:backlog\""), "{second_stream}");
        assert!(
            second_stream
                .contains("--planning-stage-complete plan:architecture --until phase-request"),
            "{second_stream}"
        );
    }

    #[test]
    fn orchestrate_dispatch_rejects_mismatched_planning_stage_request_id() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-stage-request-id");
        let brief_path = workspace.join("brief.md");
        std::fs::write(
            &brief_path,
            concat!(
                "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
                "Authoritative persistence store: workspace-local .boundline/session.json.\n",
                "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
                "In-scope API operations: start, goal, plan, and orchestrate for the first slice.\n",
                "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
                "Success criteria: the first governed slice can progress through the next planning stage with reusable planning artifacts.\n",
                "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
            ),
        )
        .unwrap();

        let first = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Deliver a governed feature".to_string()),
            brief: vec![brief_path],
            flow: Some("delivery".to_string()),
            governance: Some(crate::domain::governance::GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);

        let second = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: None,
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: Some("plan:requirements".to_string()),
            request_id: Some("req-stale".to_string()),
            answer: None,
            assistant_host: None,
            json_stream: false,
            no_canon: false,
            slug: None,
        });

        assert_eq!(second.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            second.output.contains("planning stage completion expected request_id"),
            "{}",
            second.output
        );
    }

    #[test]
    fn orchestrate_phase_request_uses_canon_packet_artifacts_when_memory_is_contradicted() {
        use crate::domain::governance::{
            CanonEvidenceInspectSummary, CanonRecommendedActionSummary, CompactedCanonMemory,
            MemoryCredibilityState,
        };

        let workspace = write_execution_workspace("boundline-cli-orchestrate-canon-packet");
        let brief_path = workspace.join("brief.md");
        std::fs::write(
            &brief_path,
            concat!(
                "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
                "Authoritative persistence store: workspace-local .boundline/session.json.\n",
                "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
                "In-scope API operations: goal, plan, and orchestrate for the first slice.\n",
                "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
                "Success criteria: the first governed slice can progress through the next planning stage with reusable planning artifacts.\n",
                "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
            ),
        )
        .unwrap();

        let first = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Deliver a governed feature".to_string()),
            brief: vec![brief_path],
            flow: Some("delivery".to_string()),
            governance: Some(crate::domain::governance::GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });
        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);

        let packet_dir = workspace.join(".canon/artifacts/R-20260524-019e59e7/requirements");
        std::fs::create_dir_all(&packet_dir).unwrap();
        std::fs::write(
            packet_dir.join("prd.md"),
            "# Product Requirements Document\n\nNOT CAPTURED\n",
        )
        .unwrap();
        std::fs::write(
            packet_dir.join("problem-statement.md"),
            "# Problem Statement\n\nNOT CAPTURED\n",
        )
        .unwrap();

        let store = FileSessionStore::for_workspace(&workspace);
        let mut record = store.load().unwrap().unwrap();
        let memory = CompactedCanonMemory {
            headline:
                "Requirements packet is structurally complete only and still carries 4 explicit missing-context marker(s)."
                    .to_string(),
            credibility: MemoryCredibilityState::Contradicted,
            stage_key: Some("plan:requirements".to_string()),
            run_ref: Some("R-20260524-019e59e7".to_string()),
            packet_ref: Some(".canon/artifacts/R-20260524-019e59e7/requirements".to_string()),
            reason_code: Some("rejected_packet".to_string()),
            artifact_refs: vec![
                ".canon/artifacts/R-20260524-019e59e7/requirements/problem-statement.md"
                    .to_string(),
                ".canon/artifacts/R-20260524-019e59e7/requirements/prd.md".to_string(),
            ],
            mode_summary: None,
            possible_actions: Vec::new(),
            recommended_next_action: Some(CanonRecommendedActionSummary {
                action: "replan".to_string(),
                rationale:
                    "run `R-20260524-019e59e7` is blocked because the governed packet is not reusable"
                        .to_string(),
                target: Some("R-20260524-019e59e7".to_string()),
            }),
            evidence_summary: Some(CanonEvidenceInspectSummary {
                execution_posture: None,
                carried_forward_items: Vec::new(),
                artifact_provenance_links: vec![
                    ".canon/artifacts/R-20260524-019e59e7/requirements/problem-statement.md"
                        .to_string(),
                    ".canon/artifacts/R-20260524-019e59e7/requirements/prd.md".to_string(),
                ],
                closure_status: None,
                closure_findings: Vec::new(),
            }),
            authority_provenance_lines: Vec::new(),
            adaptive_provenance_lines: vec!["adaptive_contract_line: unavailable".to_string()],
            semantic_provenance_lines: vec!["semantic_contract_line: unavailable".to_string()],
        };
        record.goal_plan.as_mut().unwrap().compacted_canon_memory = Some(memory.clone());
        store.persist(&record).unwrap();

        let second = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: None,
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);
        let stream = second.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"artifact_kind\":\"canon_packet\""), "{stream}");
        assert!(stream.contains(".canon/artifacts/R-20260524-019e59e7/requirements"), "{stream}");
        assert!(stream.contains("replan"), "{stream}");
        assert!(
            stream.contains(
                "structurally complete only and still carries 4 explicit missing-context marker(s)"
            ),
            "{stream}"
        );
    }

    #[test]
    fn orchestrate_phase_request_emits_assistant_safe_follow_up_for_copilot() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-assistant-follow-up");
        let brief_path = workspace.join("brief.md");
        std::fs::write(
            &brief_path,
            concat!(
                "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n\n",
                "Authoritative persistence store: workspace-local .boundline/session.json.\n",
                "Authentication boundary: GitHub OAuth2 stops at token validation; service authorization begins in Boundline route selection.\n",
                "In-scope API operations: goal, plan, and orchestrate for the first slice.\n",
                "Domain entities in scope: session, plan brief, run brief, and planning stage brief.\n",
                "Success criteria: the first governed slice can progress through the next planning stage with reusable planning artifacts.\n",
                "Validation target: cargo test -p boundline-cli --lib orchestrate -- --test-threads=1.\n",
            ),
        )
        .unwrap();

        let outcome = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: Some("Deliver a governed feature".to_string()),
            brief: vec![brief_path],
            flow: Some("delivery".to_string()),
            governance: Some(crate::domain::governance::GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: Some(AssistantHost::Copilot),
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(outcome.exit_status, CommandExitStatus::Succeeded);
        let stream = outcome.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"assistant_resume_command\":\"/boundline-plan\""), "{stream}");
        assert!(!stream.contains("\"assistant_next_command\":"), "{stream}");
    }

    #[test]
    fn orchestrate_dispatch_emits_structured_clarification_phase_request() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-clarification");

        let outcome = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace),
            cluster: None,
            goal: Some("Build a bounded user management microservice".to_string()),
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(outcome.exit_status, CommandExitStatus::Succeeded);
        let stream = outcome.stream_output.as_deref().unwrap_or_default();
        assert!(stream.contains("\"event_kind\":\"phase_request\""), "{stream}");
        assert!(stream.contains("\"phase_request\":{"), "{stream}");
        assert!(stream.contains("\"kind\":\"clarification\""), "{stream}");
        assert!(
            stream.contains(
                "\"question\":\"Which API operations, endpoints, or RPC methods are in scope first?\""
            ),
            "{stream}"
        );
        assert!(stream.contains("\"type\":\"suggested_choice\""), "{stream}");
        assert!(stream.contains("\"label\":\"REST CRUD\""), "{stream}");
        assert!(stream.contains("\"label\":\"gRPC\""), "{stream}");
        assert!(stream.contains("--answer \\\"<answer>\\\""), "{stream}");
    }

    #[test]
    fn orchestrate_dispatch_resumes_goal_clarification_into_planning() {
        let workspace = write_execution_workspace("boundline-cli-orchestrate-clarification-resume");

        let first = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("Build a bounded user management microservice".to_string()),
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: None,
            answer: None,
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(first.exit_status, CommandExitStatus::Succeeded);
        let first_stream = first.stream_output.as_deref().unwrap_or_default();
        let request_id = first_stream
            .split("\"request_id\":\"")
            .nth(1)
            .and_then(|value| value.split('"').next())
            .unwrap_or_default()
            .to_string();
        assert!(!request_id.is_empty(), "{first_stream}");
        assert!(
            first_stream.contains(
                "\"question\":\"Which API operations, endpoints, or RPC methods are in scope first?\""
            ),
            "{first_stream}"
        );

        let second = dispatch(&DeveloperCommand::Orchestrate {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: None,
            brief: Vec::new(),
            flow: None,
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            intent: OrchestrateIntent::ContinueUntilPhaseRequest,
            planning_stage_complete: None,
            request_id: Some(request_id),
            answer: Some("REST CRUD endpoints over HTTP".to_string()),
            assistant_host: None,
            json_stream: true,
            no_canon: false,
            slug: None,
        });

        assert_eq!(second.exit_status, CommandExitStatus::Succeeded);
        let second_stream = second.stream_output.as_deref().unwrap_or_default();
        assert!(
            second_stream.contains("applied the clarification answer to the active goal"),
            "{second_stream}"
        );
        assert!(second_stream.contains("\"phase_kind\":\"goal_capture\""), "{second_stream}");
        assert!(
            second_stream.contains(
                "\"question\":\"What measurable success criteria should prove the goal is complete?\""
            ),
            "{second_stream}"
        );
    }

    #[test]
    fn continue_error_dispatch_and_command_name_strings_cover_new_variants() {
        assert_eq!(CommandName::Continue.as_str(), "continue");
        assert_eq!(CommandName::Govern.as_str(), "govern");
        assert_eq!(CommandName::Assistant.as_str(), "assistant");

        let workspace = temp_workspace("boundline-cli-continue-error");
        FileSessionStore::for_workspace(&workspace)
            .persist(&ActiveSessionRecord {
                session_id: "session-mismatch".to_string(),
                workspace_ref: "/tmp/other-workspace".to_string(),
                goal: None,
                authored_brief: None,
                negotiation_packet: None,
                active_flow: None,
                active_task: None,
                goal_plan: None,
                workflow_progress: None,
                decisions: Vec::new(),
                active_flow_policy: None,
                latest_status: SessionStatus::Initialized,
                latest_terminal_reason: None,
                latest_trace_ref: None,
                created_at: 1,
                updated_at: 1,
                governance_lifecycle: None,
                project_scale: None,
                latest_voting: None,
                delight_feedback: None,
            })
            .unwrap();

        let outcome = dispatch(&DeveloperCommand::Continue {
            workspace: Some(workspace),
            cluster: None,
            session: None,
        });

        assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
        assert!(outcome.output.contains("continue: session error"), "{}", outcome.output);
    }
}
