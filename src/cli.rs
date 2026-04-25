use std::{fmt, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::domain::trace::current_timestamp_millis;

pub mod diagnostics;
pub mod inspect;
pub mod output;
pub mod run;
pub mod session;

#[derive(Debug, Parser)]
#[command(name = "synod", about = "Developer CLI for the Synod orchestrator core")]
pub struct Cli {
    #[command(subcommand)]
    pub command: DeveloperCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Doctor,
    Demo,
    Run,
    Inspect,
    Start,
    Capture,
    Plan,
    Step,
    Status,
    Next,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Demo => "demo",
            Self::Run => "run",
            Self::Inspect => "inspect",
            Self::Start => "start",
            Self::Capture => "capture",
            Self::Plan => "plan",
            Self::Step => "step",
            Self::Status => "status",
            Self::Next => "next",
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
        #[arg(long)]
        workspace: PathBuf,
    },
    Start {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Capture {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        goal: String,
    },
    Plan {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Step {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Demo {
        #[arg(long)]
        workspace: PathBuf,
    },
    Run {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        goal: Option<String>,
    },
    Inspect {
        #[arg(long)]
        trace: Option<PathBuf>,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Status {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Next {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
}

impl DeveloperCommand {
    pub const fn name(&self) -> CommandName {
        match self {
            Self::Doctor { .. } => CommandName::Doctor,
            Self::Start { .. } => CommandName::Start,
            Self::Capture { .. } => CommandName::Capture,
            Self::Plan { .. } => CommandName::Plan,
            Self::Step { .. } => CommandName::Step,
            Self::Demo { .. } => CommandName::Demo,
            Self::Run { .. } => CommandName::Run,
            Self::Inspect { .. } => CommandName::Inspect,
            Self::Status { .. } => CommandName::Status,
            Self::Next { .. } => CommandName::Next,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeveloperCommandSession {
    pub command_name: CommandName,
    pub workspace_ref: Option<String>,
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
            DeveloperCommand::Doctor { workspace } => Self {
                command_name: CommandName::Doctor,
                workspace_ref: Some(workspace.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Start { workspace } => Self {
                command_name: CommandName::Start,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Capture { workspace, goal } => Self {
                command_name: CommandName::Capture,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: Some(goal.clone()),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Plan { workspace } => Self {
                command_name: CommandName::Plan,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Step { workspace } => Self {
                command_name: CommandName::Step,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Demo { workspace } => Self {
                command_name: CommandName::Demo,
                workspace_ref: Some(workspace.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Run { workspace, goal } => Self {
                command_name: CommandName::Run,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: goal.clone(),
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Inspect { trace, workspace } => Self {
                command_name: CommandName::Inspect,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: trace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Status { workspace } => Self {
                command_name: CommandName::Status,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: None,
                trace_ref: None,
                started_at: current_timestamp_millis(),
                completed_at: None,
                exit_status: None,
                trace_location: None,
            },
            DeveloperCommand::Next { workspace } => Self {
                command_name: CommandName::Next,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
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
            CommandName::Doctor | CommandName::Demo => {
                let workspace = self.workspace_ref.as_deref().unwrap_or_default();
                if workspace.trim().is_empty() {
                    return Err(CliValidationError::MissingWorkspaceRef(self.command_name));
                }
            }
            CommandName::Run => {
                if self.goal.is_some() {
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
            | CommandName::Capture
            | CommandName::Plan
            | CommandName::Step
            | CommandName::Status
            | CommandName::Next => {}
        }

        if matches!(self.command_name, CommandName::Capture)
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
        DeveloperCommand::Doctor { workspace } => {
            let report = diagnostics::diagnose_workspace(workspace);
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
        DeveloperCommand::Demo { workspace } => {
            let report = diagnostics::diagnose_workspace(workspace);
            if !report.ready {
                return DispatchOutcome {
                    exit_status: CommandExitStatus::InvalidInvocation,
                    output: output::render_diagnostics(&report),
                    trace_location: None,
                };
            }

            match run::execute_demo(workspace) {
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
        }
        DeveloperCommand::Run { workspace, goal } => match goal {
            Some(goal) => {
                let workspace = workspace
                    .as_ref()
                    .expect("validated run invocations with --goal must include --workspace");
                let report = diagnostics::diagnose_workspace(workspace);
                if !report.ready {
                    return DispatchOutcome {
                        exit_status: CommandExitStatus::InvalidInvocation,
                        output: output::render_diagnostics(&report),
                        trace_location: None,
                    };
                }

                match run::execute_custom_run(workspace, goal) {
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
            }
            None => match session::execute_run(workspace.as_deref()) {
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
            },
        },
        DeveloperCommand::Inspect { trace, workspace } => {
            match inspect::execute_inspect(trace.as_deref(), workspace.as_deref()) {
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
                    output: inspect::render_error(trace.as_deref(), workspace.as_deref(), &error),
                    trace_location: None,
                },
            }
        }
        DeveloperCommand::Start { workspace } => match session::execute_start(workspace.as_deref())
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
        },
        DeveloperCommand::Capture { workspace, goal } => {
            match session::execute_capture(workspace.as_deref(), goal) {
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
        DeveloperCommand::Plan { workspace } => match session::execute_plan(workspace.as_deref()) {
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
        },
        DeveloperCommand::Step { workspace } => match session::execute_step(workspace.as_deref()) {
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
        },
        DeveloperCommand::Status { workspace } => {
            match session::execute_status(workspace.as_deref()) {
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
        DeveloperCommand::Next { workspace } => match session::execute_next(workspace.as_deref()) {
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
        },
    }
}
