use std::path::{Path, PathBuf};

use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::cli::CommandExitStatus;
use crate::cli::output;
use crate::cli::session::{self, SessionCommandError};
use crate::domain::brief::{
    BriefIngestionError, normalize_governance_intent, normalize_inputs_with_governance,
};
use crate::domain::governance::GovernanceRuntimeKind;
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingOutcome, RoutingSource, SessionStatus,
};
use crate::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType};
use crate::fixture::{FixtureRuntimeError, build_fixture_runtime, build_task_request};
use crate::orchestrator::engine::{Orchestrator, OrchestratorError};
use crate::orchestrator::flow_inference::infer_flow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
}

pub fn execute_native_direct_run(
    workspace: &Path,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<RunCommandReport, RunCommandError> {
    ensure_native_direct_run_can_bootstrap(workspace)?;

    session::execute_start(Some(workspace)).map_err(RunCommandError::SessionCommand)?;
    session::execute_capture(Some(workspace), goal, briefs, governance, risk, zone, owner)
        .map_err(RunCommandError::SessionCommand)?;

    let session_store = FileSessionStore::for_workspace(workspace);
    let record = session_store
        .load()
        .map_err(RunCommandError::SessionStore)?
        .ok_or(SessionCommandError::MissingActiveSession)
        .map_err(RunCommandError::SessionCommand)?;

    if native_direct_run_requires_clarification(&record) {
        let report =
            session::execute_status(Some(workspace)).map_err(RunCommandError::SessionCommand)?;
        return Ok(RunCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: report.terminal_output,
            trace_location: record.latest_trace_ref.clone(),
        });
    }

    let inferred_flow = record.goal.as_deref().and_then(infer_flow).map(|flow| flow.flow_name);
    session::execute_plan(Some(workspace), inferred_flow.as_deref(), inferred_flow.is_none())
        .map_err(RunCommandError::SessionCommand)?;
    let report = session::execute_run(Some(workspace)).map_err(RunCommandError::SessionCommand)?;
    let trace_location = session_store
        .load()
        .map_err(RunCommandError::SessionStore)?
        .and_then(|record| record.latest_trace_ref);

    Ok(RunCommandReport {
        exit_status: report.exit_status,
        terminal_output: report.terminal_output,
        trace_location,
    })
}

pub fn execute_custom_run(
    workspace: &Path,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<RunCommandReport, RunCommandError> {
    let governance_intent = normalize_governance_intent(governance, risk, zone, owner)?;
    let bundle = normalize_inputs_with_governance(workspace, goal, briefs, governance_intent)?;
    let goal = bundle.render_goal_text();
    let store = FileTraceStore::for_workspace(workspace);
    let trace_reader = store.clone();
    let request = build_task_request(
        workspace,
        goal,
        format!("run-{}", crate::domain::trace::current_timestamp_millis()),
        Some(&bundle),
        None,
    )?;

    if let Some(clarification) = bundle.clarification.as_ref() {
        let terminal_reason = TerminalReason::new(
            TerminalCondition::TaskNotCredible,
            clarification.prompt.clone(),
            Some(json!({
                "clarification_required": true,
                "clarification_headline": clarification.headline(),
                "clarification_missing_fields": clarification.missing_fields,
            })),
        );
        let mut trace = ExecutionTrace::new(
            Uuid::new_v4().to_string(),
            request.session_id.clone(),
            request.goal.clone(),
        );
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            0,
            json!({
                "goal": request.goal,
                "input": request.input,
                "limits": request.limits,
            }),
        );
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            0,
            json!({
                "terminal_status": TaskStatus::Failed,
                "terminal_reason": terminal_reason,
            }),
        );
        trace.finalize(TaskStatus::Failed, terminal_reason.clone());
        let trace_path = store.persist(&trace).map_err(RunCommandError::TraceStore)?;
        let trace_location = trace_path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        store.persist(&trace).map_err(RunCommandError::TraceStore)?;
        let loaded_trace = trace_reader.load(Path::new(&trace_location)).ok();
        let response = TaskRunResponse {
            task_id: trace.task_id.clone(),
            terminal_status: TaskStatus::Failed,
            terminal_reason,
            final_context: TaskContext::new(
                request.session_id,
                request.workspace_ref,
                request.limits,
                request.initial_context.unwrap_or_default(),
            ),
            plan_revision: 0,
            trace_location: trace_location.clone(),
        };
        let terminal_output = compatibility_terminal_output(output::render_run_trace(
            "run",
            loaded_trace.as_ref(),
            &response,
            "/synod-inspect",
        ));
        return Ok(RunCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output,
            trace_location: Some(trace_location),
        });
    }

    let runtime = build_fixture_runtime(workspace)?;
    let orchestrator = Orchestrator::new(runtime.planner, runtime.agents, runtime.tools, store)
        .with_governance(runtime.profile.read_targets.clone(), runtime.profile.governance.clone());
    let response = orchestrator.run(request)?;
    let trace = trace_reader.load(Path::new(&response.trace_location)).ok();
    let exit_status = if response.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };
    let terminal_output = compatibility_terminal_output(output::render_run_trace(
        "run",
        trace.as_ref(),
        &response,
        output::next_command_after_run(response.terminal_status),
    ));

    Ok(RunCommandReport {
        exit_status,
        terminal_output,
        trace_location: Some(response.trace_location),
    })
}

#[derive(Debug, Error)]
pub enum RunCommandError {
    #[error("failed to ingest authored brief: {0}")]
    BriefIngestion(#[from] BriefIngestionError),
    #[error(
        "active session already contains meaningful work; continue it or reset the workspace session before using direct native run"
    )]
    ActiveSessionConflict,
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session command failed: {0}")]
    SessionCommand(#[from] SessionCommandError),
    #[error("failed to prepare the fixture-backed vertical slice: {0}")]
    FixtureRuntime(#[from] FixtureRuntimeError),
    #[error("failed to persist the clarification trace: {0}")]
    TraceStore(#[from] crate::adapters::trace_store::TraceStoreError),
    #[error("failed to run the orchestrator vertical slice: {0}")]
    Orchestrator(#[from] OrchestratorError),
}

fn compatibility_terminal_output(body: String) -> String {
    let routing = output::render_route_outcome(&RoutingOutcome {
        mode: RoutingMode::Compatibility,
        source: RoutingSource::ExecutionProfile,
        reason: "compatibility mode was chosen deliberately from the declarative execution path"
            .to_string(),
    });

    format!("{routing}\nexecution_path: fixture_compatibility\n{body}")
}

fn ensure_native_direct_run_can_bootstrap(workspace: &Path) -> Result<(), RunCommandError> {
    let Some(record) =
        FileSessionStore::for_workspace(workspace).load().map_err(RunCommandError::SessionStore)?
    else {
        return Ok(());
    };

    if active_session_has_meaningful_state(&record) {
        return Err(RunCommandError::ActiveSessionConflict);
    }

    Ok(())
}

fn active_session_has_meaningful_state(record: &ActiveSessionRecord) -> bool {
    record.goal.as_deref().map(str::trim).is_some_and(|goal| !goal.is_empty())
        || record.authored_brief.is_some()
        || record.negotiation_packet.is_some()
        || record.active_flow.is_some()
        || record.active_task.is_some()
        || record.goal_plan.is_some()
        || !record.decisions.is_empty()
        || record.latest_trace_ref.is_some()
        || !matches!(record.latest_status, SessionStatus::Initialized)
}

fn native_direct_run_requires_clarification(record: &ActiveSessionRecord) -> bool {
    record.authored_brief.as_ref().and_then(|bundle| bundle.clarification.as_ref()).is_some()
        || record.negotiation_packet.as_ref().is_some_and(|packet| {
            packet.resolution_state
                != crate::domain::negotiation::NegotiationResolutionState::Credible
        })
}
