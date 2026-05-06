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
    checkpoint, cluster, config, diagnostics, init, inspect, output, run, session, workflow,
    workspace as cli_workspace,
};

#[derive(Debug, Parser)]
#[command(name = "boundline", about = "Local delivery orchestrator for bounded engineering work")]
pub struct Cli {
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
    Init,
    Config,
    Cluster,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
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
            Self::Init => "init",
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
    Init {
        /// Workspace directory to bootstrap.
        #[arg(long)]
        workspace: PathBuf,
        /// Optional starting template for the generated execution profile. Defaults to bug-fix.
        #[arg(long)]
        template: Option<InitTemplate>,
        /// Assistant runtimes to record in the local workspace config.
        #[arg(long = "assistant")]
        assistant: Vec<RuntimeKind>,
        /// Model route in SLOT=RUNTIME:MODEL form, e.g. planning=copilot:gpt-4o.
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
            | CommandName::Init
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
}

pub fn execute() -> i32 {
    let cli = Cli::parse();
    let mut session = DeveloperCommandSession::from_command(&cli.command);

    match session.validate() {
        Err(error) => {
            let exit_code = session.complete(CommandExitStatus::InvalidInvocation, None);
            eprintln!("{}", output::validation_error_message(&error));
            exit_code.code()
        }
        Ok(()) => {
            let outcome = dispatch(&cli.command);
            let exit_code = session.complete(outcome.exit_status, outcome.trace_location);
            println!("{}", outcome.output);
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
                    return DispatchOutcome {
                        exit_status: CommandExitStatus::InvalidInvocation,
                        output: output::validation_error_message(
                            &CliValidationError::MissingWorkspaceRef(CommandName::Doctor),
                        ),
                        trace_location: None,
                    };
                };
                diagnostics::diagnose_workspace(workspace)
            };
            DispatchOutcome {
                exit_status: if report.ready {
                    CommandExitStatus::Succeeded
                } else {
                    CommandExitStatus::InvalidInvocation
                },
                output: output::render_diagnostics(&report),
                trace_location: None,
            }
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
                            return DispatchOutcome {
                                exit_status: CommandExitStatus::InvalidInvocation,
                                output: format!("workspace resolution failed: {error}"),
                                trace_location: None,
                            };
                        }
                    };
                let workspace = &resolved_workspace;
                if !workspace.is_dir() {
                    return DispatchOutcome {
                        exit_status: CommandExitStatus::InvalidInvocation,
                        output: output::validation_error_message(
                            &CliValidationError::MissingWorkspaceRef(CommandName::Run),
                        ),
                        trace_location: None,
                    };
                }
                let report = if *compatibility {
                    diagnostics::diagnose_workspace(workspace)
                } else {
                    diagnostics::diagnose_native_direct_run_workspace(workspace)
                };
                if !report.ready {
                    return DispatchOutcome {
                        exit_status: CommandExitStatus::InvalidInvocation,
                        output: output::render_diagnostics(&report),
                        trace_location: None,
                    };
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
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: report.trace_location,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::InvalidInvocation,
                        output: error.to_string(),
                        trace_location: None,
                    },
                }
            } else {
                match session::execute_run_with_target(workspace.as_deref(), cluster.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: session::render_error(command.name().as_str(), &error),
                        trace_location: None,
                    },
                }
            }
        }
        DeveloperCommand::Workflow { command } => match command {
            WorkflowSubcommand::List { workspace } => {
                match workflow::execute_list(workspace.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("workflow error: {error}"),
                        trace_location: None,
                    },
                }
            }
            WorkflowSubcommand::Run { name, workspace, goal } => {
                match workflow::execute_run(workspace.as_deref(), name, goal.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("workflow error: {error}"),
                        trace_location: None,
                    },
                }
            }
            WorkflowSubcommand::Status { workspace } => {
                match workflow::execute_status(workspace.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("workflow error: {error}"),
                        trace_location: None,
                    },
                }
            }
            WorkflowSubcommand::Resume { workspace } => {
                match workflow::execute_resume(workspace.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("workflow error: {error}"),
                        trace_location: None,
                    },
                }
            }
            WorkflowSubcommand::Inspect { workspace } => {
                match workflow::execute_inspect(workspace.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("workflow error: {error}"),
                        trace_location: None,
                    },
                }
            }
        },
        DeveloperCommand::Checkpoint { command } => match command {
            CheckpointSubcommand::List { workspace, cluster } => {
                match checkpoint::execute_list(workspace.as_deref(), cluster.as_deref()) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("checkpoint error: {error}"),
                        trace_location: None,
                    },
                }
            }
            CheckpointSubcommand::Restore { checkpoint_id, workspace, cluster, force } => {
                match checkpoint::execute_restore(
                    checkpoint_id,
                    workspace.as_deref(),
                    cluster.as_deref(),
                    *force,
                ) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("checkpoint error: {error}"),
                        trace_location: None,
                    },
                }
            }
        },
        DeveloperCommand::Inspect { trace, workspace, cluster } => {
            match inspect::execute_inspect(
                trace.as_deref(),
                workspace.as_deref().or(cluster.as_deref()),
            ) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: match error {
                        inspect::InspectCommandError::InvalidSession(_) => {
                            CommandExitStatus::NonSuccess
                        }
                        _ => CommandExitStatus::TraceReadFailure,
                    },
                    output: inspect::render_error(
                        trace.as_deref(),
                        workspace.as_deref().or(cluster.as_deref()),
                        &error,
                    ),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Start { workspace, cluster } => {
            match session::execute_start_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
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
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Flow { name, workspace, cluster } => {
            match session::execute_flow_with_target(workspace.as_deref(), cluster.as_deref(), name)
            {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
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
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Step { workspace, cluster } => {
            match session::execute_step_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Status { workspace, cluster } => {
            match session::execute_status_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Next { workspace, cluster } => {
            match session::execute_next_with_target(workspace.as_deref(), cluster.as_deref()) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: session::render_error(command.name().as_str(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Init {
            workspace,
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
            force,
        } => {
            match init::execute_init(init::InitRequest {
                workspace,
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
                force: *force,
            }) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: format!("init error: {error}"),
                    trace_location: None,
                },
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
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: format!("config error: {error}"),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Cluster { command } => match command {
            ClusterSubcommand::Init { workspace, cluster_id, member } => {
                match cluster::execute_init(workspace, cluster_id, member) {
                    Ok(report) => DispatchOutcome {
                        exit_status: report.exit_status,
                        output: report.terminal_output,
                        trace_location: None,
                    },
                    Err(error) => DispatchOutcome {
                        exit_status: CommandExitStatus::NonSuccess,
                        output: format!("cluster error: {error}"),
                        trace_location: None,
                    },
                }
            }
            ClusterSubcommand::Status { workspace } => match cluster::execute_status(workspace) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: format!("cluster error: {error}"),
                    trace_location: None,
                },
            },
            ClusterSubcommand::Inspect { workspace } => match cluster::execute_inspect(workspace) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::NonSuccess,
                    output: format!("cluster error: {error}"),
                    trace_location: None,
                },
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        CheckpointSubcommand, ClusterSubcommand, CommandExitStatus, CommandName, ConfigSubcommand,
        DeveloperCommand, DeveloperCommandSession, WorkflowSubcommand, dispatch,
    };
    use crate::domain::configuration::{
        CapabilityState, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy, EffortLevel,
        RouteSlot, RuntimeKind,
    };
    use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};

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
            DeveloperCommand::Status { workspace: Some(workspace.clone()), cluster: None },
            DeveloperCommand::Next { workspace: Some(workspace.clone()), cluster: None },
        ];

        for command in commands {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("session error"), "{}", outcome.output);
        }

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
        assert!(invalid.output.contains("doctor:"), "{}", invalid.output);
    }

    #[test]
    fn command_names_and_dispatch_cover_remaining_command_variants() {
        for (name, expected) in [
            (CommandName::Checkpoint, "checkpoint"),
            (CommandName::Workflow, "workflow"),
            (CommandName::Inspect, "inspect"),
            (CommandName::Init, "init"),
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
            force: false,
        });
        assert_eq!(init.exit_status, CommandExitStatus::NonSuccess);
        assert!(init.output.contains("init error:"), "{}", init.output);

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
}
