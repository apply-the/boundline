use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::demo::endpoints::{DemoRuntime, DemoRuntimeError, build_demo_runtime};
use crate::demo::profile::{DemoProfileError, DemoRunProfile};
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{ActiveSessionRecord, SessionStatus};
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionRequest,
    StepExecutionResult, StepKind, StepResultSummary, StepStatus,
};
use crate::domain::task::{Task, TaskRequestError, TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::orchestrator::planner::Planner;
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};

#[derive(Debug, Clone)]
pub struct SessionRuntime {
    workspace_ref: PathBuf,
    session_store: FileSessionStore,
    trace_store: FileTraceStore,
}

impl SessionRuntime {
    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        let workspace_ref = workspace_ref.as_ref().to_path_buf();
        Self {
            session_store: FileSessionStore::for_workspace(&workspace_ref),
            trace_store: FileTraceStore::for_workspace(&workspace_ref),
            workspace_ref,
        }
    }

    pub fn workspace_ref(&self) -> &Path {
        &self.workspace_ref
    }

    pub fn session_store(&self) -> &FileSessionStore {
        &self.session_store
    }

    pub fn trace_store(&self) -> &FileTraceStore {
        &self.trace_store
    }

    pub fn load_session(&self) -> Result<Option<ActiveSessionRecord>, SessionRuntimeError> {
        self.session_store.load().map_err(SessionRuntimeError::SessionStore)
    }

    pub fn persist_session(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<PathBuf, SessionRuntimeError> {
        self.session_store.persist(session).map_err(SessionRuntimeError::SessionStore)
    }

    pub fn clear_session(&self) -> Result<(), SessionRuntimeError> {
        self.session_store.clear().map_err(SessionRuntimeError::SessionStore)
    }

    pub fn latest_trace(&self) -> Result<Option<PathBuf>, SessionRuntimeError> {
        self.trace_store.latest().map_err(SessionRuntimeError::TraceStore)
    }

    pub fn capture_goal(
        &self,
        session: &mut ActiveSessionRecord,
        goal: &str,
    ) -> Result<(), SessionRuntimeError> {
        let goal = goal.trim();
        if goal.is_empty() {
            return Err(SessionRuntimeError::MissingGoal);
        }

        session.goal = Some(goal.to_string());
        session.active_task = None;
        session.latest_status = SessionStatus::GoalCaptured;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn plan_task(&self, session: &mut ActiveSessionRecord) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let profile = DemoRunProfile::default_run(goal);
        let request = profile.to_task_request(
            self.workspace_ref.to_string_lossy().into_owned(),
            session.session_id.clone(),
        );
        let plan = profile.to_plan().map_err(SessionRuntimeError::DemoProfile)?;
        let task = Task::new(Uuid::new_v4().to_string(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;

        session.active_task = Some(task);
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn execute_next_step(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let runtime = self.build_runtime(session)?;
        let _ = self.execute_single_step(session, &runtime)?;
        Ok(())
    }

    pub fn run_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                return Ok(response);
            }
        }
    }

    fn build_runtime(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<DemoRuntime, SessionRuntimeError> {
        let goal = session
            .goal
            .as_deref()
            .or_else(|| session.active_task.as_ref().map(|task| task.goal.as_str()))
            .unwrap_or_default()
            .trim()
            .to_string();

        if goal.is_empty() {
            return Err(SessionRuntimeError::MissingGoal);
        }

        build_demo_runtime(DemoRunProfile::default_run(goal))
            .map_err(SessionRuntimeError::DemoRuntime)
    }

    fn execute_single_step(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &DemoRuntime,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        let mut task = session.active_task.take().ok_or(SessionRuntimeError::MissingActiveTask)?;

        if task.status.is_terminal() {
            let response = self.existing_terminal_response(session, &task)?;
            session.latest_status = session_status_for_task_status(task.status);
            session.latest_terminal_reason = task.terminal_reason.clone();
            session.updated_at = current_timestamp_millis();
            session.active_task = Some(task);
            return Ok(Some(response));
        }

        if matches!(task.status, TaskStatus::Planned) {
            task.mark_running();
        }

        session.latest_status = SessionStatus::Running;
        session.latest_terminal_reason = None;

        let mut trace = self.load_or_create_trace(session, &task)?;
        let response = self.advance_task(session, &mut task, &mut trace, runtime)?;
        session.active_task = Some(task);

        Ok(response)
    }

    fn advance_task(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        runtime: &DemoRuntime,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        if task.total_step_attempts >= task.limits.max_steps {
            let reason = build_terminal_reason(
                TerminalCondition::StepLimitExceeded,
                "maximum step attempts reached",
                Some(json!({
                    "attempts": task.total_step_attempts,
                    "max_steps": task.limits.max_steps,
                })),
            );
            return self.finalize_task(session, task, trace, reason).map(Some);
        }

        if task.plan.current_step().is_none() {
            let reason = build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                "no executable next step remains in the current plan",
                Some(json!({
                    "plan_revision": task.plan.revision,
                })),
            );
            return self.finalize_task(session, task, trace, reason).map(Some);
        }

        let step_index = task.plan.current_step_index;
        let step_snapshot = {
            let step = task
                .plan
                .current_step_mut()
                .expect("current step was checked before step execution");
            step.mark_running();
            step.clone()
        };
        task.total_step_attempts += 1;

        let started_at = current_timestamp_millis();
        let mut attempt =
            StepAttempt::new(step_snapshot.id.clone(), step_snapshot.input.clone(), started_at);
        trace.record_event(
            TraceEventType::StepStarted,
            Some(step_snapshot.id.clone()),
            task.plan.revision,
            json!({
                "attempt_number": step_snapshot.attempt_count,
                "input": step_snapshot.input,
                "step_kind": step_snapshot.kind,
            }),
        );
        let trace_location = self.persist_trace(trace)?;
        session.latest_trace_ref = Some(trace_location);

        let result = self.execute_step(runtime, &step_snapshot, &task.context);
        let result = self.normalize_result(result, &step_snapshot);
        attempt.complete(&result, current_timestamp_millis());
        task.context.push_history_ref(attempt.attempt_id.clone());

        match result.status {
            ExecutionStatus::Succeeded => {
                let output = result.output.clone().expect("successful results are validated");
                task.plan.steps[step_index].mark_succeeded(output.clone());
                task.context.apply_success_output(
                    &step_snapshot.id,
                    &output,
                    result.state_patch.as_ref(),
                );
                task.context
                    .set_last_result(StepResultSummary::from_step(&task.plan.steps[step_index]));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step_snapshot.id.clone()),
                    task.plan.revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "succeeded",
                        "output": output,
                        "evidence": result.evidence,
                    }),
                );

                let goal_satisfied = task
                    .context
                    .state
                    .get("goal_satisfied")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    || task.plan.current_step_index + 1 >= task.plan.steps.len();

                if goal_satisfied {
                    task.plan.advance();
                    let reason = build_terminal_reason(
                        TerminalCondition::GoalSatisfied,
                        format!("goal satisfied after step {}", step_snapshot.id),
                        Some(json!({
                            "step_id": step_snapshot.id,
                        })),
                    );
                    return self.finalize_task(session, task, trace, reason).map(Some);
                }

                task.plan.advance();
                let trace_location = self.persist_trace(trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                Ok(None)
            }
            ExecutionStatus::Failed => {
                let error = result.error.clone().expect("failed results are validated");
                task.plan.steps[step_index].mark_failed(error.clone(), result.recoverability);
                task.context.apply_failure_error(&step_snapshot.id, &error);
                task.context
                    .set_last_result(StepResultSummary::from_step(&task.plan.steps[step_index]));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step_snapshot.id.clone()),
                    task.plan.revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "failed",
                        "error": error,
                        "recoverability": result.recoverability,
                        "evidence": result.evidence,
                    }),
                );

                match decide_recovery(task, &task.plan.steps[step_index], &result) {
                    RecoveryDecision::Continue => {
                        let trace_location = self.persist_trace(trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Retry { reason } => {
                        task.retry_count += 1;
                        let step = &mut task.plan.steps[step_index];
                        step.status = StepStatus::Pending;
                        trace.record_event(
                            TraceEventType::RetryScheduled,
                            Some(step_snapshot.id.clone()),
                            task.plan.revision,
                            json!({
                                "reason": reason,
                                "retry_count": task.retry_count,
                            }),
                        );
                        let trace_location = self.persist_trace(trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Replan { reason } => {
                        let replacements = match runtime.planner.replan(
                            task,
                            &task.plan.steps[step_index],
                            &result,
                        ) {
                            Ok(replacements) => replacements,
                            Err(error) => {
                                let reason = build_terminal_reason(
                                    TerminalCondition::TaskNotCredible,
                                    "planner could not produce a credible replacement plan",
                                    Some(json!({"error": error.to_string()})),
                                );
                                return self.finalize_task(session, task, trace, reason).map(Some);
                            }
                        };

                        task.replan_count += 1;
                        let revision = match task.plan.replace_remaining_steps(replacements) {
                            Ok(revision) => revision,
                            Err(error) => {
                                let reason = build_terminal_reason(
                                    TerminalCondition::TaskNotCredible,
                                    "replacement plan did not provide a credible next step",
                                    Some(json!({"error": error.to_string()})),
                                );
                                return self.finalize_task(session, task, trace, reason).map(Some);
                            }
                        };

                        trace.record_event(
                            TraceEventType::Replanned,
                            Some(step_snapshot.id.clone()),
                            revision.to_revision,
                            json!({
                                "reason": reason,
                                "replan_count": task.replan_count,
                                "from_revision": revision.from_revision,
                                "to_revision": revision.to_revision,
                                "replaced_step_ids": revision.replaced_step_ids,
                                "added_step_ids": revision.added_step_ids,
                            }),
                        );
                        let trace_location = self.persist_trace(trace)?;
                        session.latest_status = SessionStatus::Running;
                        session.latest_terminal_reason = None;
                        session.latest_trace_ref = Some(trace_location);
                        session.updated_at = current_timestamp_millis();
                        Ok(None)
                    }
                    RecoveryDecision::Terminate(reason) => {
                        self.finalize_task(session, task, trace, reason).map(Some)
                    }
                }
            }
        }
    }

    fn load_or_create_trace(
        &self,
        session: &mut ActiveSessionRecord,
        task: &Task,
    ) -> Result<ExecutionTrace, SessionRuntimeError> {
        if let Some(trace_ref) = &session.latest_trace_ref {
            return self
                .trace_store
                .load(Path::new(trace_ref))
                .map_err(SessionRuntimeError::TraceStore);
        }

        let mut trace = ExecutionTrace::new(
            task.id.clone(),
            task.context.session_id.clone(),
            task.goal.clone(),
        );
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            task.plan.revision,
            json!({
                "goal": task.goal,
                "input": task.input,
                "limits": task.limits,
            }),
        );
        let trace_location = self.persist_trace(&mut trace)?;
        session.latest_trace_ref = Some(trace_location);

        Ok(trace)
    }

    fn existing_terminal_response(
        &self,
        session: &ActiveSessionRecord,
        task: &Task,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let trace_location =
            session.latest_trace_ref.clone().ok_or(SessionRuntimeError::MissingTraceReference)?;
        let terminal_reason =
            task.terminal_reason.clone().ok_or(SessionRuntimeError::MissingTerminalReason)?;

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: task.status,
            terminal_reason,
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    fn execute_step(
        &self,
        runtime: &DemoRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        match step.kind {
            StepKind::Agent => self.execute_agent(runtime, step, context),
            StepKind::Tool => self.execute_tool(runtime, step, context),
            StepKind::Decision => self.execute_decision(step),
        }
    }

    fn execute_agent(
        &self,
        runtime: &DemoRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "agent step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(agent) = runtime.agents.get(&target_name) else {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "unknown_agent",
                    format!("agent '{}' is not registered", target_name),
                ),
                Recoverability::Terminal,
            );
        };

        agent.execute(StepExecutionRequest {
            step_id: step.id.clone(),
            step_kind: step.kind,
            target_name,
            input: step.input.clone(),
            task_snapshot: context.clone(),
            attempt_number: step.attempt_count,
        })
    }

    fn execute_tool(
        &self,
        runtime: &DemoRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "tool step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(tool) = runtime.tools.get(&target_name) else {
            return StepExecutionResult::failure(
                ErrorInfo::new("unknown_tool", format!("tool '{}' is not registered", target_name)),
                Recoverability::Terminal,
            );
        };

        tool.execute(StepExecutionRequest {
            step_id: step.id.clone(),
            step_kind: step.kind,
            target_name,
            input: step.input.clone(),
            task_snapshot: context.clone(),
            attempt_number: step.attempt_count,
        })
    }

    fn execute_decision(&self, step: &Step) -> StepExecutionResult {
        let Some(object) = step.input.as_object() else {
            return StepExecutionResult::success(step.input.clone());
        };

        if object.get("retryable_failure").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_retry", "decision step requested a retry"),
                Recoverability::Retryable,
            );
        }

        if object.get("replan_required").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_replan", "decision step requested a replan"),
                Recoverability::ReplanRequired,
            );
        }

        if object.get("terminal_failure").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_terminal", "decision step requested terminal failure"),
                Recoverability::Terminal,
            );
        }

        let output = object.get("output").cloned().unwrap_or_else(|| step.input.clone());
        let state_patch = object.get("state_patch").and_then(Value::as_object).cloned();

        match state_patch {
            Some(patch) => StepExecutionResult::success_with_patch(output, patch),
            None => StepExecutionResult::success(output),
        }
    }

    fn normalize_result(&self, result: StepExecutionResult, step: &Step) -> StepExecutionResult {
        match result.validate() {
            Ok(()) => result,
            Err(error) => StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_endpoint_result",
                    format!("step {} produced an invalid result: {}", step.id, error),
                )
                .with_details(json!({"step_id": step.id})),
                Recoverability::Terminal,
            ),
        }
    }

    fn finalize_task(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        reason: TerminalReason,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let terminal_status = task_status_for_condition(reason.condition);
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            task.plan.revision,
            json!({
                "terminal_status": terminal_status,
                "terminal_reason": reason,
            }),
        );
        task.apply_terminal(terminal_status, reason.clone());
        trace.finalize(terminal_status, reason.clone());
        let trace_location = self.persist_trace(trace)?;

        session.latest_status = session_status_for_task_status(terminal_status);
        session.latest_terminal_reason = Some(reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status,
            terminal_reason: reason,
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    fn persist_trace(&self, trace: &mut ExecutionTrace) -> Result<String, SessionRuntimeError> {
        let path = self.trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        let trace_location = path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        self.trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        Ok(trace_location)
    }
}

#[derive(Debug, Error)]
pub enum SessionRuntimeError {
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("trace store operation failed: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("active session has no captured goal")]
    MissingGoal,
    #[error("active session has no planned task")]
    MissingActiveTask,
    #[error("active session is missing the persisted trace reference")]
    MissingTraceReference,
    #[error("active task is missing a terminal reason")]
    MissingTerminalReason,
    #[error("demo profile is invalid: {0}")]
    DemoProfile(#[source] DemoProfileError),
    #[error("demo runtime is invalid: {0}")]
    DemoRuntime(#[source] DemoRuntimeError),
    #[error("task request is invalid: {0}")]
    TaskRequest(#[source] TaskRequestError),
}

fn session_status_for_task_status(status: TaskStatus) -> SessionStatus {
    match status {
        TaskStatus::Planned => SessionStatus::Planned,
        TaskStatus::Running => SessionStatus::Running,
        TaskStatus::Succeeded => SessionStatus::Succeeded,
        TaskStatus::Failed => SessionStatus::Failed,
        TaskStatus::Exhausted => SessionStatus::Exhausted,
        TaskStatus::Aborted => SessionStatus::Aborted,
    }
}
