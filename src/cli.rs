use std::{fmt, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::domain::trace::current_timestamp_millis;

pub mod diagnostics;
pub mod inspect;
pub mod output;
pub mod run;

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
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Demo => "demo",
            Self::Run => "run",
            Self::Inspect => "inspect",
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
    Demo {
        #[arg(long)]
        workspace: PathBuf,
    },
    Run {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        goal: String,
    },
    Inspect {
        #[arg(long)]
        trace: Option<PathBuf>,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
}

impl DeveloperCommand {
    pub const fn name(&self) -> CommandName {
        match self {
            Self::Doctor { .. } => CommandName::Doctor,
            Self::Demo { .. } => CommandName::Demo,
            Self::Run { .. } => CommandName::Run,
            Self::Inspect { .. } => CommandName::Inspect,
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
                workspace_ref: Some(workspace.to_string_lossy().into_owned()),
                goal: Some(goal.clone()),
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
        }
    }

    pub fn validate(&self) -> Result<(), CliValidationError> {
        match self.command_name {
            CommandName::Doctor | CommandName::Demo | CommandName::Run => {
                let workspace = self.workspace_ref.as_deref().unwrap_or_default();
                if workspace.trim().is_empty() {
                    return Err(CliValidationError::MissingWorkspaceRef(self.command_name));
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
        }

        if matches!(self.command_name, CommandName::Run)
            && self.goal.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(CliValidationError::MissingGoal);
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
    #[error("run requires a non-empty --goal")]
    MissingGoal,
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
        DeveloperCommand::Run { workspace, goal } => {
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
        DeveloperCommand::Inspect { trace, workspace } => {
            match inspect::execute_inspect(trace.as_deref(), workspace.as_deref()) {
                Ok(report) => DispatchOutcome {
                    exit_status: report.exit_status,
                    output: report.terminal_output,
                    trace_location: None,
                },
                Err(error) => DispatchOutcome {
                    exit_status: CommandExitStatus::TraceReadFailure,
                    output: inspect::render_error(trace.as_deref(), workspace.as_deref(), &error),
                    trace_location: None,
                },
            }
        }
    }
}
