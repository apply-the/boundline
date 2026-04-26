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
    Run,
    Inspect,
    Start,
    Capture,
    Flow,
    Plan,
    Step,
    Status,
    Next,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Run => "run",
            Self::Inspect => "inspect",
            Self::Start => "start",
            Self::Capture => "capture",
            Self::Flow => "flow",
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
    Flow {
        name: String,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Plan {
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Step {
        #[arg(long)]
        workspace: Option<PathBuf>,
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
            Self::Flow { .. } => CommandName::Flow,
            Self::Plan { .. } => CommandName::Plan,
            Self::Step { .. } => CommandName::Step,
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
            DeveloperCommand::Flow { name, workspace } => Self {
                command_name: CommandName::Flow,
                workspace_ref: workspace.as_ref().map(|path| path.to_string_lossy().into_owned()),
                goal: Some(name.clone()),
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
            CommandName::Doctor => {
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
            | CommandName::Flow
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
        DeveloperCommand::Flow { name, workspace } => {
            match session::execute_flow(workspace.as_deref(), name) {
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{CommandExitStatus, DeveloperCommand, dispatch};

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
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
        fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
        fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
        fs::write(
            workspace.join(".synod/execution.json"),
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

    #[test]
    fn dispatch_covers_session_error_paths() {
        let workspace = temp_workspace("synod-cli-dispatch-error");
        let commands = [
            DeveloperCommand::Capture {
                workspace: Some(workspace.clone()),
                goal: "goal".to_string(),
            },
            DeveloperCommand::Flow {
                name: "bug-fix".to_string(),
                workspace: Some(workspace.clone()),
            },
            DeveloperCommand::Plan { workspace: Some(workspace.clone()) },
            DeveloperCommand::Step { workspace: Some(workspace.clone()) },
            DeveloperCommand::Status { workspace: Some(workspace.clone()) },
            DeveloperCommand::Next { workspace: Some(workspace.clone()) },
        ];

        for command in commands {
            let outcome = dispatch(&command);
            assert_eq!(outcome.exit_status, CommandExitStatus::NonSuccess);
            assert!(outcome.output.contains("session error"), "{}", outcome.output);
        }

        let inspect =
            dispatch(&DeveloperCommand::Inspect { trace: None, workspace: Some(workspace) });
        assert_eq!(inspect.exit_status, CommandExitStatus::TraceReadFailure);
        assert!(inspect.output.contains("inspect: trace read failure"), "{}", inspect.output);
    }

    #[test]
    fn dispatch_covers_successful_custom_run_session_run_and_inspect_paths() {
        let workspace = write_execution_workspace("synod-cli-dispatch-success");

        let custom_run = dispatch(&DeveloperCommand::Run {
            workspace: Some(workspace.clone()),
            goal: Some("Fix the failing add test".to_string()),
        });
        assert_eq!(custom_run.exit_status, CommandExitStatus::Succeeded);
        assert!(custom_run.output.contains("terminal_status: succeeded"), "{}", custom_run.output);
        assert!(custom_run.trace_location.is_some());

        let start = dispatch(&DeveloperCommand::Start { workspace: Some(workspace.clone()) });
        assert_eq!(start.exit_status, CommandExitStatus::Succeeded);

        let capture = dispatch(&DeveloperCommand::Capture {
            workspace: Some(workspace.clone()),
            goal: "Fix the failing add test".to_string(),
        });
        assert_eq!(capture.exit_status, CommandExitStatus::Succeeded);

        let plan = dispatch(&DeveloperCommand::Plan { workspace: Some(workspace.clone()) });
        assert_eq!(plan.exit_status, CommandExitStatus::Succeeded);

        let step = dispatch(&DeveloperCommand::Step { workspace: Some(workspace.clone()) });
        assert_eq!(step.exit_status, CommandExitStatus::Succeeded);

        let run =
            dispatch(&DeveloperCommand::Run { workspace: Some(workspace.clone()), goal: None });
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(run.output.contains("terminal_status: succeeded"), "{}", run.output);

        let status = dispatch(&DeveloperCommand::Status { workspace: Some(workspace.clone()) });
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);

        let next = dispatch(&DeveloperCommand::Next { workspace: Some(workspace.clone()) });
        assert_eq!(next.exit_status, CommandExitStatus::Succeeded);

        let inspect = dispatch(&DeveloperCommand::Inspect {
            trace: None,
            workspace: Some(workspace.clone()),
        });
        assert_eq!(inspect.exit_status, CommandExitStatus::Succeeded);
        assert!(inspect.output.contains("inspection_target:"), "{}", inspect.output);

        let invalid_workspace = temp_workspace("synod-cli-dispatch-invalid");
        let invalid = dispatch(&DeveloperCommand::Run {
            workspace: Some(invalid_workspace),
            goal: Some("Fix the failing add test".to_string()),
        });
        assert_eq!(invalid.exit_status, CommandExitStatus::InvalidInvocation);
        assert!(invalid.output.contains("doctor:"), "{}", invalid.output);
    }
}
