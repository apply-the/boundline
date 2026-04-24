use std::path::Path;

use thiserror::Error;

use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::demo::endpoints::{DemoRuntimeError, build_demo_runtime};
use crate::demo::profile::DemoRunProfile;
use crate::domain::task::TaskStatus;
use crate::orchestrator::engine::{Orchestrator, OrchestratorError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
}

pub fn execute_demo(workspace: &Path) -> Result<RunCommandReport, RunCommandError> {
    execute_profile("demo", DemoRunProfile::guided_demo(), workspace)
}

pub fn execute_custom_run(
    workspace: &Path,
    goal: impl Into<String>,
) -> Result<RunCommandReport, RunCommandError> {
    execute_profile("run", DemoRunProfile::default_run(goal), workspace)
}

fn execute_profile(
    command_name: &str,
    profile: DemoRunProfile,
    workspace: &Path,
) -> Result<RunCommandReport, RunCommandError> {
    let runtime = build_demo_runtime(profile)?;
    let store = FileTraceStore::for_workspace(workspace);
    let trace_reader = store.clone();
    let request = runtime.profile.to_task_request(
        workspace.to_string_lossy().into_owned(),
        format!("{command_name}-{}", crate::domain::trace::current_timestamp_millis()),
    );
    let orchestrator = Orchestrator::new(runtime.planner, runtime.agents, runtime.tools, store);
    let response = orchestrator.run(request)?;
    let trace = trace_reader.load(Path::new(&response.trace_location)).ok();
    let exit_status = if response.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };
    let terminal_output = output::render_run_trace(command_name, trace.as_ref(), &response);

    Ok(RunCommandReport {
        exit_status,
        terminal_output,
        trace_location: Some(response.trace_location),
    })
}

#[derive(Debug, Error)]
pub enum RunCommandError {
    #[error("failed to build the developer demo runtime: {0}")]
    DemoRuntime(#[from] DemoRuntimeError),
    #[error("failed to run the orchestrator demo: {0}")]
    Orchestrator(#[from] OrchestratorError),
}
