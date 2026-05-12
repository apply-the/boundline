use std::{fmt, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::domain::configuration::{
    CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
    InitTemplate, RouteSlot, RuntimeKind,
};
use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};
use crate::domain::governance::{CanonMode, CanonModeSelectionPreference, GovernanceRuntimeKind};
use crate::domain::trace::current_timestamp_millis;

use super::{
    assistant_assets, checkpoint, cluster, config, diagnostics, govern, init, inspect, output, run,
    session, workflow, workspace as cli_workspace,
};

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

    #[command(subcommand)]
    pub command: DeveloperCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Doctor,
    Checkpoint,
    Run,
    Workflow,
    Inspect,
    Start,
    Capture,
    Flow,
    Plan,
    Step,
    Status,
    Next,
    Continue,
    Govern,
    Init,
    Assistant,
    Config,
    Cluster,
}

impl CommandName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Checkpoint => "checkpoint",
            Self::Run => "run",
            Self::Workflow => "workflow",
            Self::Inspect => "inspect",
            Self::Start => "start",
            Self::Capture => "capture",
            Self::Flow => "flow",
            Self::Plan => "plan",
            Self::Step => "step",
            Self::Status => "status",
            Self::Next => "next",
            Self::Continue => "continue",
            Self::Govern => "govern",
            Self::Init => "init",
            Self::Assistant => "assistant",
            Self::Config => "config",
            Self::Cluster => "cluster",
        }
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExitStatus {
    Succeeded,
    NonSuccess,
    InvalidInvocation,
    TraceReadFailure,
}

#[derive(Debug, Subcommand)]
pub enum DeveloperCommand {
    Doctor {
        #[arg(long, conflicts_with = "install", required_unless_present = "install")]
        workspace: Option<PathBuf>,
        #[arg(long, conflicts_with = "workspace")]
        install: bool,
    },
    Start {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Capture {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
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
        #[arg(long, conflicts_with_all = ["no_flow", "confirm"])]
        flow: Option<String>,
        #[arg(long, conflicts_with_all = ["flow", "confirm"])]
        #[arg(long = "no-flow")]
        no_flow: bool,
        #[arg(long, conflicts_with_all = ["flow", "no_flow"])]
        confirm: bool,
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
    },
    Status {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Next {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Continue {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
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
        about = "Bootstrap Boundline files, assistant packs, and default routing for a workspace",
        after_long_help = "Guided mode tips:\n  - leave --assistant unset to skip repository-local assistant packs\n  - leave guided routes blank to let selected assistants seed defaults for planning, implementation, verification, and review\n\nDocs export policy:\n  - --export-docs is create-only by default; existing target files stop the command\n  - use --refresh to update generated docs in place\n  - use --diff to preview docs changes without writing\n  - use --to <path> to export generated docs under another root\n\nExamples:\n  boundline init --workspace . --assistant copilot\n  boundline init --workspace . --assistant copilot --route planning=copilot:gpt-5.4\n  boundline init --workspace . --assistant codex --assistant copilot --route review=claude:sonnet-4\n  boundline init --workspace . --export-docs\n  boundline init --workspace . --export-docs --refresh\n  boundline init --workspace . --export-docs --to docs/reference/boundline"
    )]
    Init {
        /// Workspace directory to bootstrap. Defaults to the current directory.
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
        /// Disable guided terminal prompts and require explicit flag-driven input only.
        #[arg(long = "non-interactive")]
        non_interactive: bool,
        /// Optional starting template for the generated execution profile. Defaults to bug-fix.
        #[arg(long)]
        template: Option<InitTemplate>,
        /// Assistant runtimes to record in the local workspace config and use for seeded defaults. Supported values: claude, codex, copilot, gemini.
        #[arg(long = "assistant")]
        assistant: Vec<RuntimeKind>,
        /// Model route in SLOT=RUNTIME:MODEL form. Supported slots: planning, implementation, verification, review. Example: planning=copilot:gpt-5.4. Leave blank in guided mode to let selected assistants seed defaults.
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
    Config {
        #[command(subcommand)]
        command: ConfigSubcommand,
    },
    Cluster {
        #[command(subcommand)]
        command: ClusterSubcommand,
    },
}

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

#[derive(Debug, Subcommand)]
pub enum AssistantSubcommand {
    Install {
        #[arg(long, value_enum)]
        host: assistant_assets::AssistantHost,
        #[arg(long, value_enum)]
        scope: assistant_assets::AssistantInstallScope,
    },
}

#[derive(Debug, Subcommand)]
pub enum CheckpointSubcommand {
    List {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
    },
    Restore {
        checkpoint_id: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
}

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
    Unset {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        cluster: Option<PathBuf>,
        #[arg(long)]
        scope: ConfigWriteScope,
        #[arg(long)]
        slot: Option<RouteSlot>,
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

impl DeveloperCommand {
    pub const fn name(&self) -> CommandName {
        match self {
            Self::Doctor { .. } => CommandName::Doctor,
            Self::Checkpoint { .. } => CommandName::Checkpoint,
            Self::Start { .. } => CommandName::Start,
            Self::Capture { .. } => CommandName::Capture,
            Self::Flow { .. } => CommandName::Flow,
            Self::Plan { .. } => CommandName::Plan,
            Self::Step { .. } => CommandName::Step,
            Self::Run { .. } => CommandName::Run,
            Self::Workflow { .. } => CommandName::Workflow,
            Self::Inspect { .. } => CommandName::Inspect,
            Self::Status { .. } => CommandName::Status,
            Self::Next { .. } => CommandName::Next,
            Self::Continue { .. } => CommandName::Continue,
            Self::Govern { .. } => CommandName::Govern,
            Self::Assistant { .. } => CommandName::Assistant,
            Self::Init { .. } => CommandName::Init,
            Self::Config { .. } => CommandName::Config,
            Self::Cluster { .. } => CommandName::Cluster,
        }
    }
}

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
            DeveloperCommand::Start { workspace, cluster } => Self {
                command_name: CommandName::Start,
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
            DeveloperCommand::Checkpoint { command } => Self {
                command_name: CommandName::Checkpoint,
                workspace_ref: match command {
                    CheckpointSubcommand::List { workspace, cluster }
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
            DeveloperCommand::Capture {
                workspace,
                cluster,
                goal,
                brief: _,
                governance: _,
                risk: _,
                zone: _,
                owner: _,
            } => Self {
                command_name: CommandName::Capture,
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
            DeveloperCommand::Inspect { trace, workspace, cluster } => Self {
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
            DeveloperCommand::Status { workspace, cluster } => Self {
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
            DeveloperCommand::Next { workspace, cluster } => Self {
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
            DeveloperCommand::Continue { workspace, cluster } => Self {
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
            DeveloperCommand::Config { command } => Self {
                command_name: CommandName::Config,
                workspace_ref: match command {
                    ConfigSubcommand::Show { workspace, cluster, .. }
                    | ConfigSubcommand::Set { workspace, cluster, .. }
                    | ConfigSubcommand::SetCapability { workspace, cluster, .. }
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
        }
    }

    pub fn validate(&self) -> Result<(), CliValidationError> {
        match self.command_name {
            CommandName::Doctor => {
                let workspace = self.workspace_ref.as_deref().unwrap_or_default();
                if !self.install_check && workspace.trim().is_empty() {
                    return Err(CliValidationError::MissingWorkspaceRef(self.command_name));
                }
            }
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
            CommandName::Start
            | CommandName::Checkpoint
            | CommandName::Capture
            | CommandName::Flow
            | CommandName::Plan
            | CommandName::Step
            | CommandName::Workflow
            | CommandName::Status
            | CommandName::Next
            | CommandName::Continue
            | CommandName::Govern
            | CommandName::Init
            | CommandName::Assistant
            | CommandName::Config
            | CommandName::Cluster => {}
        }

        if matches!(self.command_name, CommandName::Capture)
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
    trace_location: Option<String>,
    session_status: Option<crate::domain::session::SessionStatusView>,
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
            trace_location,
            session_status: None,
            trace_summary: None,
        }
    }

    fn from_session_report(report: session::SessionCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            trace_location: report.trace_location,
            session_status: report.session_status,
            trace_summary: report.trace_summary,
        }
    }

    fn from_run_report(report: run::RunCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            trace_location: report.trace_location,
            session_status: report.session_status,
            trace_summary: report.trace_summary,
        }
    }

    fn from_inspect_report(report: inspect::InspectCommandReport) -> Self {
        Self {
            exit_status: report.exit_status,
            output: report.terminal_output,
            trace_location: report.trace_location,
            session_status: None,
            trace_summary: report.trace_summary,
        }
    }
}

pub fn execute() -> i32 {
    let cli = Cli::parse();
    let mut session = DeveloperCommandSession::from_command(&cli.command);

    if let DeveloperCommand::Inspect { trace: None, workspace: None, cluster: None } = &cli.command
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
                        cli.command.name().as_str(),
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
            let outcome = dispatch(&cli.command);
            let exit_code = session.complete(outcome.exit_status, outcome.trace_location.clone());
            if cli.json {
                println!(
                    "{}",
                    output::render_host_command_json(
                        cli.command.name().as_str(),
                        outcome.exit_status,
                        &outcome.output,
                        outcome.trace_location.as_deref(),
                        outcome.session_status.as_ref(),
                        outcome.trace_summary.as_ref(),
                    )
                );
            } else {
                println!("{}", outcome.output);
            }
            exit_code.code()
        }
    }
}

fn dispatch(command: &DeveloperCommand) -> DispatchOutcome {
    match command {
        DeveloperCommand::Doctor { workspace, install } => {
            let report = if *install {
                diagnostics::diagnose_installation()
            } else {
                let Some(workspace) = workspace.as_ref() else {
                    return DispatchOutcome::text(
                        CommandExitStatus::InvalidInvocation,
                        output::validation_error_message(&CliValidationError::MissingWorkspaceRef(
                            CommandName::Doctor,
                        )),
                        None,
                    );
                };
                diagnostics::diagnose_workspace(workspace)
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
        } => {
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
                let resolved_workspace =
                    match cli_workspace::resolve_workspace(workspace.as_deref()) {
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
                let report = if *compatibility {
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

                let result = if *compatibility {
                    run::execute_custom_run(
                        workspace,
                        goal.as_deref(),
                        brief,
                        *governance,
                        risk.as_deref(),
                        zone.as_deref(),
                        owner.as_deref(),
                    )
                } else {
                    run::execute_native_direct_run(
                        workspace,
                        goal.as_deref(),
                        brief,
                        *governance,
                        risk.as_deref(),
                        zone.as_deref(),
                        owner.as_deref(),
                        *mode,
                        *no_canon,
                    )
                };

                match result {
                    Ok(report) => DispatchOutcome::from_run_report(report),
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::InvalidInvocation,
                        error.to_string(),
                        None,
                    ),
                }
            } else {
                match session::execute_run_with_target(workspace.as_deref(), cluster.as_deref()) {
                    Ok(report) => DispatchOutcome::from_session_report(report),
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        session::render_error(command.name().as_str(), &error),
                        None,
                    ),
                }
            }
        }
        DeveloperCommand::Workflow { command } => match command {
            WorkflowSubcommand::List { workspace } => {
                match workflow::execute_list(workspace.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("workflow error: {error}"),
                        None,
                    ),
                }
            }
            WorkflowSubcommand::Run { name, workspace, goal } => {
                match workflow::execute_run(workspace.as_deref(), name, goal.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("workflow error: {error}"),
                        None,
                    ),
                }
            }
            WorkflowSubcommand::Status { workspace } => {
                match workflow::execute_status(workspace.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("workflow error: {error}"),
                        None,
                    ),
                }
            }
            WorkflowSubcommand::Resume { workspace } => {
                match workflow::execute_resume(workspace.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("workflow error: {error}"),
                        None,
                    ),
                }
            }
            WorkflowSubcommand::Inspect { workspace } => {
                match workflow::execute_inspect(workspace.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("workflow error: {error}"),
                        None,
                    ),
                }
            }
        },
        DeveloperCommand::Checkpoint { command } => match command {
            CheckpointSubcommand::List { workspace, cluster } => {
                match checkpoint::execute_list(workspace.as_deref(), cluster.as_deref()) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("checkpoint error: {error}"),
                        None,
                    ),
                }
            }
            CheckpointSubcommand::Restore { checkpoint_id, workspace, cluster, force } => {
                match checkpoint::execute_restore(
                    checkpoint_id,
                    workspace.as_deref(),
                    cluster.as_deref(),
                    *force,
                ) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("checkpoint error: {error}"),
                        None,
                    ),
                }
            }
        },
        DeveloperCommand::Inspect { trace, workspace, cluster } => {
            let default_workspace = if trace.is_none() && workspace.is_none() && cluster.is_none() {
                std::env::current_dir().ok()
            } else {
                None
            };
            let workspace_ref =
                workspace.as_deref().or(cluster.as_deref()).or(default_workspace.as_deref());
            match inspect::execute_inspect(trace.as_deref(), workspace_ref) {
                Ok(report) => DispatchOutcome::from_inspect_report(report),
                Err(error) => DispatchOutcome::text(
                    match error {
                        inspect::InspectCommandError::InvalidSession(_) => {
                            CommandExitStatus::NonSuccess
                        }
                        _ => CommandExitStatus::TraceReadFailure,
                    },
                    inspect::render_error(trace.as_deref(), workspace_ref, &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Start { workspace, cluster } => {
            match session::execute_start_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Capture {
            workspace,
            cluster,
            goal,
            brief,
            governance,
            risk,
            zone,
            owner,
        } => {
            match session::execute_capture_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                goal.as_deref(),
                brief,
                *governance,
                risk.as_deref(),
                zone.as_deref(),
                owner.as_deref(),
            ) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Flow { name, workspace, cluster } => {
            match session::execute_flow_with_target(workspace.as_deref(), cluster.as_deref(), name)
            {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Plan { workspace, cluster, flow, no_flow, confirm } => {
            match session::execute_plan_with_target(
                workspace.as_deref(),
                cluster.as_deref(),
                flow.as_deref(),
                *no_flow,
                *confirm,
            ) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Step { workspace, cluster } => {
            match session::execute_step_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Status { workspace, cluster } => {
            match session::execute_status_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Next { workspace, cluster } => {
            match session::execute_next_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Continue { workspace, cluster } => {
            match session::execute_continue_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome::from_session_report(report),
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    session::render_error(command.name().as_str(), &error),
                    None,
                ),
            }
        }
        DeveloperCommand::Govern {
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
        } => {
            match govern::execute_govern(govern::GovernRequest {
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
            }) {
                Ok(report) => {
                    DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                }
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    format!("govern error: {error}"),
                    None,
                ),
            }
        }
        DeveloperCommand::Assistant { command } => match command {
            AssistantSubcommand::Install { host, scope } => {
                let report = assistant_assets::install_global_assistant_package(*host, *scope);
                DispatchOutcome::text(
                    CommandExitStatus::Succeeded,
                    assistant_assets::render_assistant_install_report(&report),
                    None,
                )
            }
        },
        DeveloperCommand::Init {
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
            export_docs,
            refresh,
            diff,
            to,
            force,
        } => {
            match init::execute_init(init::InitRequest {
                workspace,
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
                export_docs: *export_docs,
                docs_refresh: *refresh,
                docs_diff: *diff,
                docs_output_dir: to.as_deref(),
                force: *force,
            }) {
                Ok(report) => {
                    DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                }
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    format!("init error: {error}"),
                    None,
                ),
            }
        }
        DeveloperCommand::Config { command } => {
            let result = match command {
                ConfigSubcommand::Show { workspace, cluster, scope } => {
                    config::execute_show(workspace.as_deref(), cluster.as_deref(), *scope)
                }
                ConfigSubcommand::Set {
                    workspace,
                    cluster,
                    scope,
                    slot,
                    reviewer,
                    adjudicator,
                    runtime,
                    model,
                } => config::execute_set(config::SetConfigRequest {
                    workspace: workspace.as_deref(),
                    cluster: cluster.as_deref(),
                    scope: *scope,
                    slot: *slot,
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
                        Ok(workspace) => {
                            config::execute_set_canon(Some(&workspace), *mode_selection)
                        }
                        Err(error) => Err(error),
                    }
                }
                ConfigSubcommand::Unset {
                    workspace,
                    cluster,
                    scope,
                    slot,
                    reviewer,
                    adjudicator,
                } => config::execute_unset(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    *scope,
                    *slot,
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
                    config::execute_unset_effort(
                        workspace.as_deref(),
                        cluster.as_deref(),
                        *scope,
                        *slot,
                    )
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
                    config::execute_unset_domain(
                        workspace.as_deref(),
                        cluster.as_deref(),
                        *scope,
                        *family,
                    )
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
                ConfigSubcommand::UnbindContext {
                    workspace,
                    cluster,
                    scope,
                    family,
                    kind,
                    reference,
                } => config::execute_unbind_context(
                    workspace.as_deref(),
                    cluster.as_deref(),
                    *scope,
                    *family,
                    *kind,
                    reference,
                ),
            };

            match result {
                Ok(report) => {
                    DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                }
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    format!("config error: {error}"),
                    None,
                ),
            }
        }
        DeveloperCommand::Cluster { command } => match command {
            ClusterSubcommand::Init { workspace, cluster_id, member } => {
                match cluster::execute_init(workspace, cluster_id, member) {
                    Ok(report) => {
                        DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                    }
                    Err(error) => DispatchOutcome::text(
                        CommandExitStatus::NonSuccess,
                        format!("cluster error: {error}"),
                        None,
                    ),
                }
            }
            ClusterSubcommand::Status { workspace } => match cluster::execute_status(workspace) {
                Ok(report) => {
                    DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                }
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    format!("cluster error: {error}"),
                    None,
                ),
            },
            ClusterSubcommand::Inspect { workspace } => match cluster::execute_inspect(workspace) {
                Ok(report) => {
                    DispatchOutcome::text(report.exit_status, report.terminal_output, None)
                }
                Err(error) => DispatchOutcome::text(
                    CommandExitStatus::NonSuccess,
                    format!("cluster error: {error}"),
                    None,
                ),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{LazyLock, Mutex, MutexGuard};

    use clap::Parser;
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        AssistantSubcommand, CheckpointSubcommand, Cli, ClusterSubcommand, CommandExitStatus,
        CommandName, ConfigSubcommand, DeveloperCommand, DeveloperCommandSession,
        WorkflowSubcommand, dispatch,
    };
    use crate::adapters::session_store::{FileSessionStore, SessionStore};
    use crate::cli::assistant_assets::{AssistantHost, AssistantInstallScope};
    use crate::domain::configuration::{
        CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
        InitTemplate, RouteSlot, RuntimeKind,
    };
    use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};
    use crate::domain::governance::{CanonMode, CanonModeSelectionPreference};
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};

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

    static CURRENT_DIR_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct CurrentDirGuard {
        original: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl CurrentDirGuard {
        fn change_to(path: &Path) -> Self {
            let lock = CURRENT_DIR_LOCK.lock().unwrap();
            let original = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { original, _lock: lock }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).unwrap();
        }
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
            DeveloperCommand::Init { workspace, .. } => {
                assert_eq!(workspace, PathBuf::from("."));
            }
            other => panic!("expected init command, got {other:?}"),
        }
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
        let workspace = temp_workspace("boundline-cli-dispatch-error");
        let commands = [
            DeveloperCommand::Capture {
                workspace: Some(workspace.clone()),
                cluster: None,
                goal: Some("goal".to_string()),
                brief: Vec::new(),
                governance: None,
                risk: None,
                zone: None,
                owner: None,
            },
            DeveloperCommand::Flow {
                name: "bug-fix".to_string(),
                workspace: Some(workspace.clone()),
                cluster: None,
            },
            DeveloperCommand::Plan {
                workspace: Some(workspace.clone()),
                cluster: None,
                flow: None,
                no_flow: false,
                confirm: false,
            },
            DeveloperCommand::Step { workspace: Some(workspace.clone()), cluster: None },
            DeveloperCommand::Next { workspace: Some(workspace.clone()), cluster: None },
        ];

        for command in commands {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("session error"), "{}", outcome.output);
        }

        let status = dispatch(&DeveloperCommand::Status {
            workspace: Some(workspace.clone()),
            cluster: None,
        });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(status.output.contains("session_bootstrap"), "{}", status.output);

        let cont = dispatch(&DeveloperCommand::Continue {
            workspace: Some(workspace.clone()),
            cluster: None,
        });
        assert_eq!(cont.exit_status, CommandExitStatus::Succeeded);
        assert!(cont.output.contains("chat history is not authoritative"), "{}", cont.output);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(workspace),
            cluster: None,
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

        let start = dispatch(&DeveloperCommand::Start {
            workspace: Some(session_workspace.clone()),
            cluster: None,
        });
        assert_eq!(start.exit_status, CommandExitStatus::Succeeded);

        let capture = dispatch(&DeveloperCommand::Capture {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            goal: Some("Fix the failing add test".to_string()),
            brief: vec![session_brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
        });
        assert_eq!(capture.exit_status, CommandExitStatus::Succeeded);

        let plan = dispatch(&DeveloperCommand::Plan {
            workspace: Some(session_workspace.clone()),
            cluster: None,
            flow: Some("bug-fix".to_string()),
            no_flow: false,
            confirm: false,
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
        });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);

        let next = dispatch(&DeveloperCommand::Next {
            workspace: Some(session_workspace.clone()),
            cluster: None,
        });
        assert_eq!(next.exit_status, CommandExitStatus::Succeeded);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(session_workspace.clone()),
            cluster: None,
        });
        assert_eq!(inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(inspect.output.contains("inspection_target:"), "{}", inspect.output);

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

        let start = dispatch(&DeveloperCommand::Start { workspace: None, cluster: None });
        assert_eq!(start.exit_status, CommandExitStatus::Succeeded);

        let capture = dispatch(&DeveloperCommand::Capture {
            workspace: None,
            cluster: None,
            goal: Some("Fix the failing add test".to_string()),
            brief: vec![brief],
            governance: None,
            risk: None,
            zone: None,
            owner: None,
        });
        assert_eq!(capture.exit_status, CommandExitStatus::Succeeded);

        let plan = dispatch(&DeveloperCommand::Plan {
            workspace: None,
            cluster: None,
            flow: Some("bug-fix".to_string()),
            no_flow: false,
            confirm: false,
        });
        assert_eq!(plan.exit_status, CommandExitStatus::Succeeded, "{}", plan.output);
        assert!(plan.output.contains("execution_path: native_goal_plan"), "{}", plan.output);

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

        let status = dispatch(&DeveloperCommand::Status { workspace: None, cluster: None });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(status.output.contains("latest_status: succeeded"), "{}", status.output);

        let inspect =
            dispatch(&DeveloperCommand::Inspect { trace: None, workspace: None, cluster: None });
        assert_eq!(inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(inspect.output.contains("inspection_target:"), "{}", inspect.output);
    }

    #[test]
    fn command_names_and_dispatch_cover_remaining_command_variants() {
        for (name, expected) in [
            (CommandName::Checkpoint, "checkpoint"),
            (CommandName::Workflow, "workflow"),
            (CommandName::Inspect, "inspect"),
            (CommandName::Continue, "continue"),
            (CommandName::Govern, "govern"),
            (CommandName::Init, "init"),
            (CommandName::Assistant, "assistant"),
            (CommandName::Config, "config"),
            (CommandName::Cluster, "cluster"),
        ] {
            assert_eq!(name.as_str(), expected);
            assert_eq!(name.to_string(), expected);
        }

        let workspace = temp_workspace("boundline-cli-dispatch-coverage");
        for (command, expected) in [
            (
                DeveloperCommand::Checkpoint {
                    command: CheckpointSubcommand::List {
                        workspace: Some(workspace.clone()),
                        cluster: None,
                    },
                },
                CommandName::Checkpoint,
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
                },
                CommandName::Inspect,
            ),
            (
                DeveloperCommand::Status { workspace: Some(workspace.clone()), cluster: None },
                CommandName::Status,
            ),
            (
                DeveloperCommand::Next { workspace: Some(workspace.clone()), cluster: None },
                CommandName::Next,
            ),
            (
                DeveloperCommand::Init {
                    workspace: workspace.clone(),
                    non_interactive: false,
                    template: None,
                    assistant: Vec::new(),
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
                },
            });
        assert!(checkpoint_session.validate().is_ok());

        let checkpoint = dispatch(&DeveloperCommand::Checkpoint {
            command: CheckpointSubcommand::List {
                workspace: Some(workspace.clone()),
                cluster: None,
            },
        });
        assert_eq!(checkpoint.exit_status, CommandExitStatus::Succeeded);
        assert!(checkpoint.output.contains("checkpoint_scope: workspace"), "{}", checkpoint.output);

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

        let start =
            dispatch(&DeveloperCommand::Start { workspace: None, cluster: Some(missing.clone()) });
        assert_eq!(start.exit_status, CommandExitStatus::NonSuccess);
        assert!(start.output.contains("session error"), "{}", start.output);

        let init = dispatch(&DeveloperCommand::Init {
            workspace: file_workspace,
            non_interactive: false,
            template: None,
            assistant: Vec::new(),
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

        // Init success path: dispatch with a real temp workspace and explicit values
        let init_success_workspace = temp_workspace("boundline-cli-init-dispatch-success");
        let init_ok = dispatch(&DeveloperCommand::Init {
            workspace: init_success_workspace.clone(),
            non_interactive: true,
            template: Some(InitTemplate::Change),
            assistant: vec![RuntimeKind::Copilot],
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
                    reviewer: None,
                    adjudicator: false,
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-5.4".to_string(),
                },
            },
            DeveloperCommand::Config {
                command: ConfigSubcommand::Unset {
                    workspace: Some(config_workspace.clone()),
                    cluster: None,
                    scope: ConfigWriteScope::Workspace,
                    slot: Some(RouteSlot::Planning),
                    reviewer: None,
                    adjudicator: false,
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

        let continue_command =
            DeveloperCommand::Continue { workspace: Some(workspace.clone()), cluster: None };
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
        assert_eq!(govern_without_session.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            govern_without_session.output.contains(".boundline/session.json is missing"),
            "{}",
            govern_without_session.output
        );

        let start = dispatch(&DeveloperCommand::Start {
            workspace: Some(workspace.clone()),
            cluster: None,
        });
        assert_eq!(start.exit_status, CommandExitStatus::Succeeded);

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
            })
            .unwrap();

        let outcome =
            dispatch(&DeveloperCommand::Continue { workspace: Some(workspace), cluster: None });

        assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
        assert!(outcome.output.contains("continue: session error"), "{}", outcome.output);
    }
}
