use serde_json::{Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::trace_store::{TraceStore, TraceStoreError};
use crate::domain::limits::TerminalCondition;
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionRequest,
    StepExecutionResult, StepKind, StepResultSummary, StepStatus,
};
use crate::domain::task::{Task, TaskRequestError, TaskRunRequest, TaskRunResponse};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::orchestrator::planner::{Planner, PlanningError};
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

pub struct Orchestrator<P, S> {
    planner: P,
    agents: AgentRegistry,
    tools: ToolRegistry,
    trace_store: S,
}

impl<P, S> Orchestrator<P, S>
where
    P: Planner,
    S: TraceStore,
{
    pub fn new(planner: P, agents: AgentRegistry, tools: ToolRegistry, trace_store: S) -> Self {
        Self { planner, agents, tools, trace_store }
    }

    pub fn run(&self, request: TaskRunRequest) -> Result<TaskRunResponse, OrchestratorError> {
        request.validate().map_err(OrchestratorError::InvalidRequest)?;

        let bootstrap_context = TaskContext::new(
            request.session_id.clone(),
            request.workspace_ref.clone(),
            request.limits.clone(),
            request.initial_context.clone().unwrap_or_default(),
        );

        let plan = self
            .planner
            .create_initial_plan(&request, &bootstrap_context)
            .map_err(OrchestratorError::Planning)?;

        let task_id = Uuid::new_v4().to_string();
        let mut task = Task::new(task_id.clone(), &request, plan)
            .map_err(OrchestratorError::InvalidRequest)?;
        task.mark_running();

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
        self.persist_trace(&mut trace)?;

        let trace_location = loop {
            if task.total_step_attempts >= task.limits.max_steps {
                let reason = build_terminal_reason(
                    TerminalCondition::StepLimitExceeded,
                    "maximum step attempts reached",
                    Some(json!({
                        "attempts": task.total_step_attempts,
                        "max_steps": task.limits.max_steps,
                    })),
                );
                break self.finalize_task(&mut task, &mut trace, reason)?;
            }

            if task.plan.current_step().is_none() {
                let reason = build_terminal_reason(
                    TerminalCondition::NoCredibleNextStep,
                    "no executable next step remains in the current plan",
                    Some(json!({
                        "plan_revision": task.plan.revision,
                    })),
                );
                break self.finalize_task(&mut task, &mut trace, reason)?;
            }

            let step_index = task.plan.current_step_index;
            let step_snapshot = {
                let step = task
                    .plan
                    .current_step_mut()
                    .expect("current step was checked before entering the loop body");
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
            self.persist_trace(&mut trace)?;

            let result = self.execute_step(&step_snapshot, &task.context);
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
                    task.context.set_last_result(StepResultSummary::from_step(
                        &task.plan.steps[step_index],
                    ));
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
                        break self.finalize_task(&mut task, &mut trace, reason)?;
                    }

                    task.plan.advance();
                    self.persist_trace(&mut trace)?;
                }
                ExecutionStatus::Failed => {
                    let error = result.error.clone().expect("failed results are validated");
                    task.plan.steps[step_index].mark_failed(error.clone(), result.recoverability);
                    task.context.apply_failure_error(&step_snapshot.id, &error);
                    task.context.set_last_result(StepResultSummary::from_step(
                        &task.plan.steps[step_index],
                    ));
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

                    match decide_recovery(&task, &task.plan.steps[step_index], &result) {
                        RecoveryDecision::Continue => {
                            self.persist_trace(&mut trace)?;
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
                            self.persist_trace(&mut trace)?;
                        }
                        RecoveryDecision::Replan { reason } => {
                            let replacements = match self.planner.replan(
                                &task,
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
                                    break self.finalize_task(&mut task, &mut trace, reason)?;
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
                                    break self.finalize_task(&mut task, &mut trace, reason)?;
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
                            self.persist_trace(&mut trace)?;
                        }
                        RecoveryDecision::Terminate(reason) => {
                            break self.finalize_task(&mut task, &mut trace, reason)?;
                        }
                    }
                }
            }
        };

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: task.status,
            terminal_reason: task
                .terminal_reason
                .clone()
                .expect("run loop always finalizes the task before returning"),
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    fn execute_step(&self, step: &Step, context: &TaskContext) -> StepExecutionResult {
        match step.kind {
            StepKind::Agent => self.execute_agent(step, context),
            StepKind::Tool => self.execute_tool(step, context),
            StepKind::Decision => self.execute_decision(step),
        }
    }

    fn execute_agent(&self, step: &Step, context: &TaskContext) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "agent step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(agent) = self.agents.get(&target_name) else {
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

    fn execute_tool(&self, step: &Step, context: &TaskContext) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "tool step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(tool) = self.tools.get(&target_name) else {
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
        task: &mut Task,
        trace: &mut ExecutionTrace,
        reason: crate::domain::task::TerminalReason,
    ) -> Result<String, OrchestratorError> {
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
        trace.finalize(terminal_status, reason);
        self.persist_trace(trace)
    }

    fn persist_trace(&self, trace: &mut ExecutionTrace) -> Result<String, OrchestratorError> {
        let path = self.trace_store.persist(trace).map_err(OrchestratorError::TraceStore)?;
        let trace_location = path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        self.trace_store.persist(trace).map_err(OrchestratorError::TraceStore)?;
        Ok(trace_location)
    }
}

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("invalid task request: {0}")]
    InvalidRequest(TaskRequestError),
    #[error("planning failed: {0}")]
    Planning(PlanningError),
    #[error("trace persistence failed: {0}")]
    TraceStore(TraceStoreError),
}

#[cfg(test)]
mod tests {
    use std::io::Error;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use serde_json::Map;

    use super::*;
    use crate::adapters::agent::FnAgentAdapter;
    use crate::adapters::tool::FnToolAdapter;
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::{Plan, PlanStatus};

    #[derive(Clone)]
    struct TestPlanner {
        initial_plan: Result<Plan, PlanningError>,
        replan_result: Result<Vec<Step>, PlanningError>,
    }

    impl TestPlanner {
        fn from_plan(plan: Plan) -> Self {
            Self { initial_plan: Ok(plan), replan_result: Ok(Vec::new()) }
        }

        fn failing_initial_plan(error: PlanningError) -> Self {
            Self { initial_plan: Err(error), replan_result: Ok(Vec::new()) }
        }

        fn with_replan_result(plan: Plan, replan_result: Result<Vec<Step>, PlanningError>) -> Self {
            Self { initial_plan: Ok(plan), replan_result }
        }
    }

    impl Planner for TestPlanner {
        fn create_initial_plan(
            &self,
            _request: &TaskRunRequest,
            _context: &TaskContext,
        ) -> Result<Plan, PlanningError> {
            self.initial_plan.clone()
        }

        fn replan(
            &self,
            _task: &Task,
            _failed_step: &Step,
            _failure: &StepExecutionResult,
        ) -> Result<Vec<Step>, PlanningError> {
            self.replan_result.clone()
        }
    }

    struct TestTraceStore {
        fail_on_call: Option<usize>,
        calls: Mutex<usize>,
    }

    impl TestTraceStore {
        fn succeeding() -> Self {
            Self { fail_on_call: None, calls: Mutex::new(0) }
        }

        fn fail_on(call: usize) -> Self {
            Self { fail_on_call: Some(call), calls: Mutex::new(0) }
        }
    }

    impl TraceStore for TestTraceStore {
        fn persist(&self, trace: &ExecutionTrace) -> Result<PathBuf, TraceStoreError> {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;

            if self.fail_on_call == Some(*calls) {
                return Err(TraceStoreError::Write(Error::other("forced trace store failure")));
            }

            Ok(PathBuf::from(format!("/tmp/{}.json", trace.task_id)))
        }

        fn load(&self, _path: &std::path::Path) -> Result<ExecutionTrace, TraceStoreError> {
            Err(TraceStoreError::Read(Error::other("test trace store does not load traces")))
        }

        fn latest(&self) -> Result<Option<PathBuf>, TraceStoreError> {
            Ok(None)
        }
    }

    fn build_request(limits: RunLimits) -> TaskRunRequest {
        TaskRunRequest {
            goal: "Exercise engine branches".to_string(),
            input: json!({"ticket": "BUG-12"}),
            session_id: "session-engine".to_string(),
            workspace_ref: "/tmp/synod-engine".to_string(),
            limits,
            initial_context: None,
        }
    }

    fn build_context() -> TaskContext {
        TaskContext::new("session-engine", "/tmp/synod-engine", RunLimits::default(), Map::new())
    }

    fn build_orchestrator(planner: TestPlanner) -> Orchestrator<TestPlanner, TestTraceStore> {
        Orchestrator::new(
            planner,
            AgentRegistry::new(),
            ToolRegistry::new(),
            TestTraceStore::succeeding(),
        )
    }

    #[test]
    fn execute_decision_covers_success_retry_replan_and_terminal_paths() {
        let orchestrator = build_orchestrator(TestPlanner::from_plan(
            Plan::new(vec![Step::decision("noop", json!({"output": true})).unwrap()]).unwrap(),
        ));

        let non_object = Step::decision("raw", json!("value")).unwrap();
        let raw_result = orchestrator.execute_decision(&non_object);
        assert_eq!(raw_result.status, ExecutionStatus::Succeeded);
        assert_eq!(raw_result.output, Some(json!("value")));

        let retry_step = Step::decision("retry", json!({"retryable_failure": true})).unwrap();
        assert_eq!(
            orchestrator.execute_decision(&retry_step).recoverability,
            Recoverability::Retryable
        );

        let replan_step = Step::decision("replan", json!({"replan_required": true})).unwrap();
        assert_eq!(
            orchestrator.execute_decision(&replan_step).recoverability,
            Recoverability::ReplanRequired
        );

        let terminal_step = Step::decision("terminal", json!({"terminal_failure": true})).unwrap();
        assert_eq!(
            orchestrator.execute_decision(&terminal_step).recoverability,
            Recoverability::Terminal
        );

        let patch_step = Step::decision(
            "patch",
            json!({
                "output": {"done": true},
                "state_patch": {"goal_satisfied": true}
            }),
        )
        .unwrap();
        let patch_result = orchestrator.execute_decision(&patch_step);
        assert_eq!(patch_result.output, Some(json!({"done": true})));
        assert_eq!(patch_result.state_patch.unwrap()["goal_satisfied"], json!(true));
    }

    #[test]
    fn execute_agent_and_tool_cover_missing_and_unknown_targets() {
        let orchestrator = build_orchestrator(TestPlanner::from_plan(
            Plan::new(vec![Step::decision("noop", json!({})).unwrap()]).unwrap(),
        ));
        let context = build_context();

        let missing_agent = Step {
            id: "agent-missing".to_string(),
            kind: StepKind::Agent,
            target_name: None,
            input: json!({}),
            status: StepStatus::Pending,
            attempt_count: 0,
            output: None,
            error: None,
            recoverability: None,
        };
        let missing_agent_result = orchestrator.execute_agent(&missing_agent, &context);
        assert_eq!(missing_agent_result.error.unwrap().code, "missing_target");

        let unknown_agent = Step::agent("agent-unknown", "ghost-agent", json!({})).unwrap();
        let unknown_agent_result = orchestrator.execute_agent(&unknown_agent, &context);
        assert_eq!(unknown_agent_result.error.unwrap().code, "unknown_agent");

        let missing_tool = Step {
            id: "tool-missing".to_string(),
            kind: StepKind::Tool,
            target_name: None,
            input: json!({}),
            status: StepStatus::Pending,
            attempt_count: 0,
            output: None,
            error: None,
            recoverability: None,
        };
        let missing_tool_result = orchestrator.execute_tool(&missing_tool, &context);
        assert_eq!(missing_tool_result.error.unwrap().code, "missing_target");

        let unknown_tool = Step::tool("tool-unknown", "ghost-tool", json!({})).unwrap();
        let unknown_tool_result = orchestrator.execute_tool(&unknown_tool, &context);
        assert_eq!(unknown_tool_result.error.unwrap().code, "unknown_tool");
    }

    #[test]
    fn normalize_result_converts_invalid_payloads_into_terminal_failures() {
        let orchestrator = build_orchestrator(TestPlanner::from_plan(
            Plan::new(vec![Step::decision("noop", json!({})).unwrap()]).unwrap(),
        ));
        let step = Step::decision("normalize", json!({})).unwrap();
        let invalid_result = StepExecutionResult {
            status: ExecutionStatus::Succeeded,
            output: None,
            error: None,
            recoverability: Recoverability::Terminal,
            evidence: None,
            state_patch: None,
        };

        let normalized = orchestrator.normalize_result(invalid_result, &step);
        assert_eq!(normalized.status, ExecutionStatus::Failed);
        assert_eq!(normalized.recoverability, Recoverability::Terminal);
        assert_eq!(normalized.error.unwrap().code, "invalid_endpoint_result");
    }

    #[test]
    fn run_returns_planning_errors_from_initial_plan_creation() {
        let orchestrator = build_orchestrator(TestPlanner::failing_initial_plan(
            PlanningError::InvalidPlan("planner rejected request".to_string()),
        ));

        match orchestrator.run(build_request(RunLimits::default())).unwrap_err() {
            OrchestratorError::Planning(PlanningError::InvalidPlan(message)) => {
                assert!(message.contains("planner rejected request"));
            }
            other => panic!("expected planning error, got {other:?}"),
        }
    }

    #[test]
    fn run_stops_with_step_limit_exceeded_after_retrying() {
        let plan =
            Plan::new(vec![Step::decision("retry", json!({"retryable_failure": true})).unwrap()])
                .unwrap();
        let orchestrator = build_orchestrator(TestPlanner::from_plan(plan));

        let response = orchestrator
            .run(build_request(RunLimits { max_steps: 1, max_retries: 3, ..RunLimits::default() }))
            .unwrap();

        assert_eq!(response.terminal_status, crate::domain::task::TaskStatus::Exhausted);
        assert_eq!(response.terminal_reason.condition, TerminalCondition::StepLimitExceeded);
    }

    #[test]
    fn run_stops_when_current_plan_has_no_executable_next_step() {
        let plan = Plan {
            revision: 0,
            steps: vec![Step::decision("done", json!({})).unwrap()],
            current_step_index: 1,
            status: PlanStatus::Active,
        };
        let orchestrator = build_orchestrator(TestPlanner::from_plan(plan));

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();

        assert_eq!(response.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);
    }

    #[test]
    fn run_marks_task_not_credible_when_planner_cannot_replan() {
        let plan =
            Plan::new(vec![Step::decision("replan", json!({"replan_required": true})).unwrap()])
                .unwrap();
        let orchestrator = build_orchestrator(TestPlanner::with_replan_result(
            plan,
            Err(PlanningError::Internal("planner blew up".to_string())),
        ));

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();

        assert_eq!(response.terminal_reason.condition, TerminalCondition::TaskNotCredible);
        assert!(response.terminal_reason.message.contains("planner could not produce"));
    }

    #[test]
    fn run_marks_task_not_credible_when_replacement_plan_has_no_next_step() {
        let plan =
            Plan::new(vec![Step::decision("replan", json!({"replan_required": true})).unwrap()])
                .unwrap();
        let orchestrator =
            build_orchestrator(TestPlanner::with_replan_result(plan, Ok(Vec::new())));

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();

        assert_eq!(response.terminal_reason.condition, TerminalCondition::TaskNotCredible);
        assert!(response.terminal_reason.message.contains("replacement plan did not provide"));
    }

    #[test]
    fn run_normalizes_invalid_agent_results_into_unrecoverable_failures() {
        let plan = Plan::new(vec![Step::agent("code", "coder", json!({"phase": "code"})).unwrap()])
            .unwrap();
        let planner = TestPlanner::from_plan(plan);
        let mut agents = AgentRegistry::new();
        agents
            .register(
                "coder",
                FnAgentAdapter::new(|_| StepExecutionResult {
                    status: ExecutionStatus::Succeeded,
                    output: None,
                    error: None,
                    recoverability: Recoverability::Terminal,
                    evidence: None,
                    state_patch: None,
                }),
            )
            .unwrap();
        let orchestrator =
            Orchestrator::new(planner, agents, ToolRegistry::new(), TestTraceStore::succeeding());

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();

        assert_eq!(response.terminal_reason.condition, TerminalCondition::UnrecoverableError);
        assert_eq!(
            response.final_context.state["last_error"]["code"],
            json!("invalid_endpoint_result")
        );
    }

    #[test]
    fn persist_trace_surfaces_store_failures_on_first_and_second_writes() {
        let planner = TestPlanner::from_plan(
            Plan::new(vec![Step::decision("noop", json!({})).unwrap()]).unwrap(),
        );

        let first_fail = Orchestrator::new(
            planner.clone(),
            AgentRegistry::new(),
            ToolRegistry::new(),
            TestTraceStore::fail_on(1),
        );
        let second_fail = Orchestrator::new(
            planner,
            AgentRegistry::new(),
            ToolRegistry::new(),
            TestTraceStore::fail_on(2),
        );

        let mut first_trace = ExecutionTrace::new("task-1", "session-1", "goal");
        let mut second_trace = ExecutionTrace::new("task-2", "session-2", "goal");

        assert!(matches!(
            first_fail.persist_trace(&mut first_trace),
            Err(OrchestratorError::TraceStore(_))
        ));
        assert!(matches!(
            second_fail.persist_trace(&mut second_trace),
            Err(OrchestratorError::TraceStore(_))
        ));
    }

    #[test]
    fn execute_step_dispatches_registered_agent_and_tool_endpoints() {
        let plan = Plan::new(vec![Step::decision("noop", json!({})).unwrap()]).unwrap();
        let planner = TestPlanner::from_plan(plan);

        let mut agents = AgentRegistry::new();
        agents
            .register(
                "analyzer",
                FnAgentAdapter::new(|request| {
                    StepExecutionResult::success(json!({
                        "target": request.target_name,
                        "attempt": request.attempt_number,
                    }))
                }),
            )
            .unwrap();
        let mut tools = ToolRegistry::new();
        tools
            .register(
                "tester",
                FnToolAdapter::new(|request| {
                    StepExecutionResult::success(json!({
                        "target": request.target_name,
                        "attempt": request.attempt_number,
                    }))
                }),
            )
            .unwrap();

        let orchestrator = Orchestrator::new(planner, agents, tools, TestTraceStore::succeeding());
        let context = build_context();
        let mut agent_step = Step::agent("analyze", "analyzer", json!({})).unwrap();
        let mut tool_step = Step::tool("verify", "tester", json!({})).unwrap();
        agent_step.mark_running();
        tool_step.mark_running();

        let agent_result = orchestrator.execute_step(&agent_step, &context);
        let tool_result = orchestrator.execute_step(&tool_step, &context);

        assert_eq!(agent_result.output.unwrap()["target"], json!("analyzer"));
        assert_eq!(tool_result.output.unwrap()["target"], json!("tester"));
    }
}
