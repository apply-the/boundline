use std::path::Path;

use thiserror::Error;

use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::domain::task::TaskStatus;
use crate::fixture::{FixtureRuntimeError, build_fixture_runtime, build_task_request};
use crate::orchestrator::engine::{Orchestrator, OrchestratorError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
}

pub fn execute_custom_run(
    workspace: &Path,
    goal: impl Into<String>,
) -> Result<RunCommandReport, RunCommandError> {
    let goal = goal.into();
    let runtime = build_fixture_runtime(workspace)?;
    let store = FileTraceStore::for_workspace(workspace);
    let trace_reader = store.clone();
    let request = build_task_request(
        workspace,
        goal,
        format!("run-{}", crate::domain::trace::current_timestamp_millis()),
    )?;
    let orchestrator = Orchestrator::new(runtime.planner, runtime.agents, runtime.tools, store)
        .with_governance(runtime.profile.read_targets.clone(), runtime.profile.governance.clone());
    let response = orchestrator.run(request)?;
    let trace = trace_reader.load(Path::new(&response.trace_location)).ok();
    let exit_status = if response.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };
    let terminal_output = output::render_run_trace(
        "run",
        trace.as_ref(),
        &response,
        output::next_command_after_run(response.terminal_status),
    );

    Ok(RunCommandReport {
        exit_status,
        terminal_output,
        trace_location: Some(response.trace_location),
    })
}

#[derive(Debug, Error)]
pub enum RunCommandError {
    #[error("failed to prepare the fixture-backed vertical slice: {0}")]
    FixtureRuntime(#[from] FixtureRuntimeError),
    #[error("failed to run the orchestrator vertical slice: {0}")]
    Orchestrator(#[from] OrchestratorError),
}
