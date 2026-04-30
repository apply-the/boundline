use std::path::{Path, PathBuf};

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::governance_runtime::{
    CanonCliRuntime, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeRequest,
    LocalGovernanceRuntime,
};
use crate::adapters::tool::FnToolAdapter;
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::domain::flow::{FlowStepMetadata, built_in_flow, supported_flow_names_csv};
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::InferredFlow;
use crate::domain::governance::{
    ApprovalState, CanonMode, GovernanceLifecycleState, GovernanceRuntimeKind, GovernedStageRecord,
    PacketReadiness, resolved_canon_mode, supported_canon_modes_for_stage,
};
use crate::domain::limits::{RunLimits, TerminalCondition};
use crate::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingOutcome, SessionStatus, routing_outcome,
};
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionRequest,
    StepExecutionResult, StepKind, StepResultSummary, StepStatus,
};
use crate::domain::task::{Task, TaskRequestError, TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::fixture::{
    FixtureRuntime, FixtureRuntimeError, build_fixture_plan_for_goal,
    build_fixture_runtime_for_flow, build_task_request, load_workspace_execution_profile,
};
use crate::orchestrator::decision_loop::{DecisionLoop, LoopTerminal};
use crate::orchestrator::flow_inference::infer_flow;
use crate::orchestrator::goal_planner::{GoalPlannerError, build_goal_plan};
use crate::orchestrator::governance::{
    GovernanceStepDecision, bounded_governance_context, build_autopilot_decision,
    governance_input_documents, governance_stage_key, governance_state_patch,
    overlay_stage_policy_with_intent, requested_governance_intent, runtime_command_available,
    selected_stage_policy,
};
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::review_trace::{record_review_step_completed, record_review_step_started};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

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
        session.goal_plan = None;
        session.decisions.clear();
        session.active_flow_policy = None;
        session.latest_status = SessionStatus::GoalCaptured;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn select_flow(
        &self,
        session: &mut ActiveSessionRecord,
        flow_name: &str,
    ) -> Result<(), SessionRuntimeError> {
        let flow = built_in_flow(flow_name).ok_or_else(|| SessionRuntimeError::UnknownFlow {
            requested: flow_name.to_string(),
            supported: supported_flow_names_csv(),
        })?;

        if session.active_task.is_some()
            || session.goal_plan.is_some()
            || !session.decisions.is_empty()
        {
            return Err(SessionRuntimeError::FlowReplacementRequiresReset {
                current: session
                    .active_flow
                    .as_ref()
                    .map(|existing_flow| existing_flow.flow_name.clone())
                    .unwrap_or_else(|| "current-session".to_string()),
                requested: flow.name.to_string(),
            });
        }

        session.active_flow = Some(flow.initial_state());
        session.active_task = None;
        session.latest_trace_ref = None;
        session.latest_terminal_reason = None;
        session.latest_status = if session.goal.is_some() {
            SessionStatus::GoalCaptured
        } else {
            SessionStatus::Initialized
        };
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn plan_task(
        &self,
        session: &mut ActiveSessionRecord,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        if let Some(bundle) = session.authored_brief.as_ref()
            && let Some(clarification) = bundle.clarification.as_ref()
        {
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: clarification.headline(),
                prompt: clarification.prompt.clone(),
            });
        }
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        let mut goal_plan = build_goal_plan(&goal, &self.workspace_ref)
            .map_err(SessionRuntimeError::GoalPlanner)?;
        self.apply_planning_flow_selection(session, &mut goal_plan, requested_flow, no_flow)?;
        goal_plan
            .confirm()
            .map_err(|error| SessionRuntimeError::InvalidGoalPlan(error.to_string()))?;

        session.active_task = None;
        session.goal_plan = Some(goal_plan);
        session.decisions.clear();
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    fn apply_planning_flow_selection(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut crate::domain::goal_plan::GoalPlan,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        if no_flow {
            session.active_flow = None;
            session.active_flow_policy = None;
            goal_plan.mark_flow_skipped();
            return Ok(());
        }

        if let Some(flow_name) = requested_flow {
            self.apply_confirmed_flow(
                session,
                goal_plan,
                flow_name,
                "operator confirmed flow during planning",
            )?;
            return Ok(());
        }

        if let Some(active_flow) = &session.active_flow {
            let flow_name = active_flow.flow_name.clone();
            self.apply_confirmed_flow(
                session,
                goal_plan,
                &flow_name,
                "operator selected flow before planning",
            )?;
            return Ok(());
        }

        session.active_flow = None;
        session.active_flow_policy = None;
        goal_plan.flow_skipped = false;
        goal_plan.flow = infer_flow(&goal_plan.goal_text);
        Ok(())
    }

    fn apply_confirmed_flow(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut crate::domain::goal_plan::GoalPlan,
        flow_name: &str,
        confidence_reason: &str,
    ) -> Result<(), SessionRuntimeError> {
        let flow = built_in_flow(flow_name).ok_or_else(|| SessionRuntimeError::UnknownFlow {
            requested: flow_name.to_string(),
            supported: supported_flow_names_csv(),
        })?;
        let policy = FlowPolicy::from_builtin(flow_name)
            .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;

        session.active_flow = Some(flow.initial_state());
        session.active_flow_policy = Some(policy);
        goal_plan.flow = Some(InferredFlow {
            flow_name: flow.name.to_string(),
            confidence_reason: confidence_reason.to_string(),
            confirmed: true,
        });

        Ok(())
    }

    pub fn resolve_routing_outcome(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<RoutingOutcome, SessionRuntimeError> {
        if let Some(policy) = session.active_flow_policy.as_ref() {
            policy
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        Ok(routing_outcome(session))
    }

    pub fn uses_native_goal_plan(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if session.active_task.is_some() {
            return Ok(false);
        }

        let outcome = self.resolve_routing_outcome(session)?;

        if outcome.mode == RoutingMode::Blocked
            && let Some(goal_plan) = session.goal_plan.as_ref()
            && let Some(flow) = goal_plan.flow.as_ref()
            && !flow.confirmed
        {
            return Err(SessionRuntimeError::FlowConfirmationRequired {
                flow_name: flow.flow_name.clone(),
            });
        }

        Ok(outcome.mode == RoutingMode::Native)
    }

    fn should_materialize_security_assessment_task(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if session.active_task.is_some() || session.goal_plan.is_none() {
            return Ok(false);
        }

        let flow_name =
            session.active_flow.as_ref().map(|flow| flow.flow_name.as_str()).or_else(|| {
                session
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.flow.as_ref())
                    .filter(|flow| flow.confirmed)
                    .map(|flow| flow.flow_name.as_str())
            });
        let Some(flow_name) = flow_name else {
            return Ok(false);
        };

        let profile = match load_workspace_execution_profile(&self.workspace_ref) {
            Ok(profile) => profile,
            Err(FixtureRuntimeError::MissingExecutionProfile(_)) => return Ok(false),
            Err(error) => return Err(SessionRuntimeError::FixtureRuntime(error)),
        };
        let Some(governance) = profile.governance.as_ref() else {
            return Ok(false);
        };

        Ok(governance.stages.iter().any(|policy| {
            policy.enabled
                && policy.flow_name == flow_name
                && policy.effective_runtime(governance.default_runtime)
                    == GovernanceRuntimeKind::Canon
                && supported_canon_modes_for_stage(&policy.flow_name, &policy.stage_id)
                    .contains(&CanonMode::SecurityAssessment)
        }))
    }

    fn materialize_security_assessment_task(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if !self.should_materialize_security_assessment_task(session)? {
            return Ok(false);
        }

        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let request = build_task_request(
            &self.workspace_ref,
            goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan = build_fixture_plan_for_goal(
            &self.workspace_ref,
            session.active_flow.as_ref(),
            session.goal.as_deref().unwrap_or_default(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;

        session.active_task = Some(
            Task::new(format!("task-{}", session.session_id), &request, plan)
                .map_err(SessionRuntimeError::TaskRequest)?,
        );
        session.updated_at = current_timestamp_millis();
        Ok(true)
    }

    fn native_flow_policy(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<Option<FlowPolicy>, SessionRuntimeError> {
        if let Some(policy) = session.active_flow_policy.as_ref() {
            policy
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
            return Ok(Some(policy.clone()));
        }

        let Some(goal_plan) = session.goal_plan.as_ref() else {
            return Ok(None);
        };
        let Some(flow) = goal_plan.flow.as_ref() else {
            return Ok(None);
        };
        if !flow.confirmed {
            return Ok(None);
        }

        let policy = FlowPolicy::from_builtin(&flow.flow_name)
            .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        Ok(Some(policy))
    }

    fn native_adapter_registries(
        &self,
    ) -> Result<(AgentRegistry, ToolRegistry), SessionRuntimeError> {
        let mut agents = AgentRegistry::new();
        let analyzer_workspace = self.workspace_ref.clone();
        agents
            .register(
                "analyzer",
                FnAgentAdapter::new(move |request| {
                    native_analyze_workspace(&analyzer_workspace, request)
                }),
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;

        let coder_workspace = self.workspace_ref.clone();
        agents
            .register(
                "coder",
                FnAgentAdapter::new(move |request| {
                    native_apply_workspace_change(&coder_workspace, request)
                }),
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;

        let mut tools = ToolRegistry::new();
        let tester_workspace = self.workspace_ref.clone();
        tools
            .register(
                "tester",
                FnToolAdapter::new(move |request| {
                    native_run_validation(&tester_workspace, request)
                }),
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;

        tools
            .register("replanner", FnToolAdapter::new(native_replan_step))
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;

        Ok((agents, tools))
    }

    fn run_goal_plan_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let goal_plan = session.goal_plan.clone().ok_or(SessionRuntimeError::MissingActiveTask)?;
        let flow_policy = self.native_flow_policy(session)?;
        let (agents, tools) = self.native_adapter_registries()?;
        let loop_runner = DecisionLoop::new(
            agents,
            tools,
            self.trace_store.clone(),
            RunLimits::default().max_steps,
        );
        let (terminal, decisions, mut trace) = loop_runner
            .run(&goal_plan, flow_policy.as_ref(), &session.workspace_ref, &session.session_id)
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;
        let (condition, message, details) = native_terminal_outcome(&terminal);
        let terminal_reason = build_terminal_reason(condition, message, details);
        let terminal_status = task_status_for_condition(condition);

        trace.finalize(terminal_status, terminal_reason.clone());
        let trace_path =
            self.trace_store.persist(&trace).map_err(SessionRuntimeError::TraceStore)?;
        let trace_location = trace_path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        self.trace_store.persist(&trace).map_err(SessionRuntimeError::TraceStore)?;

        session.active_task = None;
        session.decisions = decisions;
        session.latest_status = session_status_for_task_status(terminal_status);
        session.latest_terminal_reason = Some(terminal_reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id: session.session_id.clone(),
            terminal_status,
            terminal_reason,
            final_context: TaskContext::new(
                session.session_id.clone(),
                session.workspace_ref.clone(),
                RunLimits::default(),
                Map::new(),
            ),
            plan_revision: 0,
            trace_location,
        })
    }

    pub fn execute_next_step(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let _ = self.materialize_security_assessment_task(session)?;
        let runtime = self.build_runtime(session)?;
        let _ = self.execute_single_step(session, &runtime)?;
        Ok(())
    }

    pub fn run_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let _ = self.materialize_security_assessment_task(session)?;

        if self.uses_native_goal_plan(session)? {
            return self.run_goal_plan_to_terminal(session);
        }

        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                return Ok(response);
            }

            if let Some(response) = self.paused_governance_response(session)? {
                return Ok(response);
            }
        }
    }

    fn paused_governance_response(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<Option<TaskRunResponse>, SessionRuntimeError> {
        let Some(task) = session.active_task.as_ref() else {
            return Ok(None);
        };
        let Some(record) = task
            .context
            .latest_governance_stage()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
        else {
            return Ok(None);
        };

        let message = match record.lifecycle_state {
            GovernanceLifecycleState::AwaitingApproval => {
                format!("governance approval is still pending for {}", record.stage_key)
            }
            GovernanceLifecycleState::Blocked => record
                .blocked_reason
                .clone()
                .unwrap_or_else(|| format!("governance blocked stage {}", record.stage_key)),
            GovernanceLifecycleState::Failed => record
                .blocked_reason
                .clone()
                .unwrap_or_else(|| format!("governance failed for stage {}", record.stage_key)),
            _ => return Ok(None),
        };

        let trace_location =
            session.latest_trace_ref.clone().ok_or(SessionRuntimeError::MissingTraceReference)?;

        Ok(Some(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: task.status,
            terminal_reason: build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                message,
                Some(json!({
                    "stage_key": record.stage_key,
                    "governance_state": record.lifecycle_state,
                })),
            ),
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        }))
    }

    pub fn refresh_governance_state(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if session.active_task.is_none() {
            return Ok(false);
        }

        let runtime = self.build_runtime(session)?;
        let Some(mut task) = session.active_task.take() else {
            return Ok(false);
        };
        let result = (|| {
            let Some(record) = task
                .context
                .latest_governance_stage()
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
            else {
                return Ok(false);
            };
            if record.lifecycle_state != GovernanceLifecycleState::AwaitingApproval {
                return Ok(false);
            }

            let mut trace = self.load_or_create_trace(session, &task)?;
            let step =
                task.plan.current_step().cloned().ok_or(SessionRuntimeError::MissingActiveTask)?;
            let metadata = FlowStepMetadata::from_step(&step)
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
                .ok_or_else(|| {
                    SessionRuntimeError::InvalidFlowState(
                        "governance refresh requires flow metadata".to_string(),
                    )
                })?;
            let Some(governance) = runtime.profile.governance.as_ref() else {
                return Ok(false);
            };
            let Some(policy) =
                selected_stage_policy(Some(governance), &metadata.flow_name, &metadata.stage_id)
            else {
                return Ok(false);
            };
            let governance_intent = requested_governance_intent(&task.input);
            let policy = overlay_stage_policy_with_intent(&policy, governance_intent.as_ref());

            let decision = self.execute_governance_for_step(
                session,
                &mut task,
                &mut trace,
                &runtime,
                &step,
                &metadata,
                governance,
                &policy,
                GovernanceRequestKind::Refresh,
            )?;

            Ok(!matches!(decision, GovernanceStepDecision::Continue))
        })();
        session.active_task = Some(task);
        result
    }

    fn build_runtime(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<FixtureRuntime, SessionRuntimeError> {
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

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

        build_fixture_runtime_for_flow(&self.workspace_ref, session.active_flow.as_ref())
            .map_err(SessionRuntimeError::FixtureRuntime)
    }

    fn execute_single_step(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
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
        runtime: &FixtureRuntime,
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

        match self.ensure_stage_governance(session, task, trace, runtime)? {
            GovernanceStepDecision::Continue => {}
            GovernanceStepDecision::Halt => return Ok(None),
            GovernanceStepDecision::Terminal(response) => return Ok(Some(response)),
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
        record_review_step_started(
            trace,
            &step_snapshot.id,
            &step_snapshot.input,
            &task.context.state,
            task.plan.revision,
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
                record_review_step_completed(
                    trace,
                    &step_snapshot.id,
                    &step_snapshot.input,
                    &result,
                    &task.context.state,
                    task.plan.revision,
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

                if let Some((from_stage, to_stage)) = self
                    .advance_session_flow(session, task, step_index)
                    .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
                {
                    trace.record_event(
                        TraceEventType::StageTransitioned,
                        Some(step_snapshot.id.clone()),
                        task.plan.revision,
                        json!({
                            "flow_name": from_stage.flow_name,
                            "from_stage_id": from_stage.stage_id,
                            "to_stage_id": to_stage.stage_id,
                            "from_stage_index": from_stage.stage_index,
                            "to_stage_index": to_stage.stage_index,
                        }),
                    );
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
                if let Some(state_patch) = result.state_patch.as_ref() {
                    task.context.apply_state_patch(state_patch);
                }
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
                record_review_step_completed(
                    trace,
                    &step_snapshot.id,
                    &step_snapshot.input,
                    &result,
                    &task.context.state,
                    task.plan.revision,
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
                        let flow_payload =
                            self.flow_payload_for_step(&step_snapshot).map_err(|error| {
                                SessionRuntimeError::InvalidFlowState(error.to_string())
                            })?;
                        let mut payload = json!({
                            "reason": reason,
                            "retry_count": task.retry_count,
                        });
                        if let Some(flow_payload) = flow_payload.clone()
                            && let Some(object) = payload.as_object_mut()
                        {
                            object.insert("flow".to_string(), flow_payload);
                        }
                        trace.record_event(
                            if flow_payload.is_some() {
                                TraceEventType::StageRetryScheduled
                            } else {
                                TraceEventType::RetryScheduled
                            },
                            Some(step_snapshot.id.clone()),
                            task.plan.revision,
                            payload,
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

                        let flow_payload =
                            self.flow_payload_for_step(&step_snapshot).map_err(|error| {
                                SessionRuntimeError::InvalidFlowState(error.to_string())
                            })?;
                        let mut payload = json!({
                            "reason": reason,
                            "replan_count": task.replan_count,
                            "from_revision": revision.from_revision,
                            "to_revision": revision.to_revision,
                            "replaced_step_ids": revision.replaced_step_ids,
                            "added_step_ids": revision.added_step_ids,
                        });
                        if let Some(flow_payload) = flow_payload.clone()
                            && let Some(object) = payload.as_object_mut()
                        {
                            object.insert("flow".to_string(), flow_payload);
                        }
                        trace.record_event(
                            if flow_payload.is_some() {
                                TraceEventType::StageReplanned
                            } else {
                                TraceEventType::Replanned
                            },
                            Some(step_snapshot.id.clone()),
                            revision.to_revision,
                            payload,
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

    fn ensure_stage_governance(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        runtime: &FixtureRuntime,
    ) -> Result<GovernanceStepDecision<TaskRunResponse>, SessionRuntimeError> {
        let Some(step) = task.plan.current_step().cloned() else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let Some(metadata) = FlowStepMetadata::from_step(&step)
            .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
        else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let Some(governance) = runtime.profile.governance.as_ref() else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let Some(policy) =
            selected_stage_policy(Some(governance), &metadata.flow_name, &metadata.stage_id)
        else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let governance_intent = requested_governance_intent(&task.input);
        let policy = overlay_stage_policy_with_intent(&policy, governance_intent.as_ref());
        if !policy.enabled {
            return Ok(GovernanceStepDecision::Continue);
        }

        let stage_key = governance_stage_key(&metadata.flow_name, &metadata.stage_id);
        if let Some(existing_record) = task
            .context
            .latest_governance_stage()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
            && existing_record.stage_key == stage_key
        {
            return Ok(GovernanceStepDecision::Continue);
        }

        self.execute_governance_for_step(
            session,
            task,
            trace,
            runtime,
            &step,
            &metadata,
            governance,
            &policy,
            GovernanceRequestKind::Start,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_governance_for_step(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        runtime: &FixtureRuntime,
        step: &Step,
        metadata: &FlowStepMetadata,
        governance: &crate::domain::governance::GovernanceProfile,
        policy: &crate::domain::governance::StageGovernancePolicy,
        request_kind: GovernanceRequestKind,
    ) -> Result<GovernanceStepDecision<TaskRunResponse>, SessionRuntimeError> {
        let stage_key = governance_stage_key(&metadata.flow_name, &metadata.stage_id);
        let existing_record = task
            .context
            .latest_governance_stage()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        if matches!(request_kind, GovernanceRequestKind::Refresh)
            && existing_record.as_ref().is_none_or(|record| {
                record.stage_key != stage_key
                    || record.lifecycle_state != GovernanceLifecycleState::AwaitingApproval
            })
        {
            return Ok(GovernanceStepDecision::Continue);
        }

        let existing_packet = task
            .context
            .latest_governance_packet()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        let governance_attempt_id = existing_record
            .as_ref()
            .filter(|_| matches!(request_kind, GovernanceRequestKind::Refresh))
            .map(|record| record.governance_attempt_id.clone())
            .unwrap_or_else(|| {
                format!("{}-attempt-{}", stage_key.replace(':', "-"), task.plan.revision)
            });
        let previous_attempt_id = if matches!(request_kind, GovernanceRequestKind::Refresh) {
            existing_record
                .as_ref()
                .and_then(|record| record.previous_governance_attempt_id.clone())
        } else {
            existing_record
                .as_ref()
                .filter(|record| record.stage_key == stage_key)
                .map(|record| record.governance_attempt_id.clone())
        };
        let (bounded_context, packet_reuse) =
            bounded_governance_context(&task.context, metadata, &runtime.profile.read_targets)
                .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        let input_documents = governance_input_documents(&task.input);

        let requested_runtime = policy.effective_runtime(governance.default_runtime);
        let canon_available = governance
            .canon
            .as_ref()
            .is_some_and(|canon| runtime_command_available(&canon.command));
        let mut decision = if requested_runtime == GovernanceRuntimeKind::Canon {
            build_autopilot_decision(
                &governance_attempt_id,
                policy,
                governance.default_runtime,
                metadata,
                &bounded_context,
                existing_record.as_ref().map(|record| record.lifecycle_state),
                existing_record.as_ref().map(|record| record.approval_state),
                existing_packet.as_ref().map(|packet| packet.readiness),
            )
        } else {
            None
        };
        let mut mode = decision
            .as_ref()
            .and_then(|record| record.selected_mode)
            .or_else(|| resolved_canon_mode(policy, governance.default_runtime))
            .or(existing_packet.as_ref().and_then(|packet| packet.canon_mode));
        let mut selected_runtime = requested_runtime;
        if requested_runtime == GovernanceRuntimeKind::Canon
            && (mode.is_none() || !canon_available)
            && !policy.required
        {
            selected_runtime = GovernanceRuntimeKind::Local;
            decision = None;
        }

        trace.record_event(
            TraceEventType::GovernanceSelected,
            Some(step.id.clone()),
            task.plan.revision,
            json!({
                "stage_key": stage_key,
                "required": policy.required,
                "autopilot_enabled": policy.autopilot,
                "requested_runtime": requested_runtime,
                "selected_runtime": selected_runtime,
            }),
        );

        if let Some(decision) = &decision {
            self.record_governance_decision_event(trace, step, task.plan.revision, decision);
        }

        if requested_runtime == GovernanceRuntimeKind::Canon
            && selected_runtime == GovernanceRuntimeKind::Canon
        {
            let Some(canon) = governance.canon.as_ref() else {
                return self.handle_governance_block(
                    session,
                    task,
                    trace,
                    GovernanceBlockContext {
                        step_id: step.id.clone(),
                        stage_key: stage_key.clone(),
                        required: policy.required,
                        autopilot_enabled: policy.autopilot,
                        runtime: GovernanceRuntimeKind::Canon,
                        reason: format!(
                            "governance stage {stage_key} requires Canon configuration"
                        ),
                    },
                    decision.clone(),
                );
            };
            if !canon_available {
                return self.handle_governance_block(
                    session,
                    task,
                    trace,
                    GovernanceBlockContext {
                        step_id: step.id.clone(),
                        stage_key: stage_key.clone(),
                        required: policy.required,
                        autopilot_enabled: policy.autopilot,
                        runtime: GovernanceRuntimeKind::Canon,
                        reason: format!(
                            "governance required Canon for {stage_key}, but command '{}' is unavailable",
                            canon.command
                        ),
                    },
                    decision.clone(),
                );
            }
            let Some(mode_value) = mode.take() else {
                return self.handle_governance_block(
                    session,
                    task,
                    trace,
                    GovernanceBlockContext {
                        step_id: step.id.clone(),
                        stage_key: stage_key.clone(),
                        required: policy.required,
                        autopilot_enabled: policy.autopilot,
                        runtime: GovernanceRuntimeKind::Canon,
                        reason: format!(
                            "governance stage {stage_key} requires an explicit Canon mode"
                        ),
                    },
                    decision.clone(),
                );
            };

            let request = GovernanceRuntimeRequest {
                request_kind,
                governance_attempt_id: governance_attempt_id.clone(),
                stage_key: stage_key.clone(),
                goal: task.goal.clone(),
                workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
                autopilot: policy.autopilot,
                mode: Some(mode_value),
                system_context: policy.system_context.or(canon.default_system_context),
                risk: policy.risk.clone().or_else(|| canon.default_risk.clone()),
                zone: policy.zone.clone().or_else(|| canon.default_zone.clone()),
                owner: policy.owner.clone().or_else(|| canon.default_owner.clone()),
                run_ref: existing_record.as_ref().and_then(|record| record.canon_run_ref.clone()),
                packet_ref: existing_record
                    .as_ref()
                    .and_then(|record| record.packet_ref.clone())
                    .or_else(|| existing_packet.as_ref().map(|packet| packet.packet_ref.clone())),
                bounded_context,
                input_documents: input_documents.clone(),
            };
            trace.record_event(
                TraceEventType::GovernanceStarted,
                Some(step.id.clone()),
                task.plan.revision,
                json!({
                    "stage_key": stage_key,
                    "runtime": GovernanceRuntimeKind::Canon,
                    "canon_mode": request.mode,
                    "system_context": request.system_context,
                    "risk": request.risk,
                    "zone": request.zone,
                    "owner": request.owner,
                    "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
                }),
            );
            let response = CanonCliRuntime::new(canon.command.clone())
                .with_working_directory(&self.workspace_ref)
                .execute(&request)
                .map_err(|error| SessionRuntimeError::GovernanceRuntime(error.to_string()))?;
            let decision = if decision.is_some() {
                decision
            } else {
                let decision = build_autopilot_decision(
                    &governance_attempt_id,
                    policy,
                    governance.default_runtime,
                    metadata,
                    &request.bounded_context,
                    Some(response.status),
                    Some(response.approval_state),
                    response.packet.as_ref().map(|packet| packet.readiness),
                );
                if let Some(record) = &decision {
                    self.record_governance_decision_event(trace, step, task.plan.revision, record);
                }
                decision
            };

            return self.apply_governance_response(
                session,
                task,
                trace,
                step,
                stage_key,
                policy,
                request_kind,
                GovernanceRuntimeKind::Canon,
                governance_attempt_id,
                previous_attempt_id,
                packet_reuse,
                decision,
                response,
            );
        }

        let request = GovernanceRuntimeRequest {
            request_kind,
            governance_attempt_id: governance_attempt_id.clone(),
            stage_key: stage_key.clone(),
            goal: task.goal.clone(),
            workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
            autopilot: policy.autopilot,
            mode: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
            run_ref: None,
            packet_ref: existing_record
                .as_ref()
                .and_then(|record| record.packet_ref.clone())
                .or_else(|| existing_packet.as_ref().map(|packet| packet.packet_ref.clone())),
            bounded_context,
            input_documents,
        };

        trace.record_event(
            TraceEventType::GovernanceStarted,
            Some(step.id.clone()),
            task.plan.revision,
            json!({
                "stage_key": stage_key,
                "runtime": GovernanceRuntimeKind::Local,
                "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
            }),
        );
        let response = LocalGovernanceRuntime
            .execute(&request)
            .map_err(|error| SessionRuntimeError::GovernanceRuntime(error.to_string()))?;

        let decision = if decision.is_some() {
            decision
        } else {
            let decision = build_autopilot_decision(
                &governance_attempt_id,
                policy,
                governance.default_runtime,
                metadata,
                &request.bounded_context,
                Some(response.status),
                Some(response.approval_state),
                response.packet.as_ref().map(|packet| packet.readiness),
            );
            if let Some(record) = &decision {
                self.record_governance_decision_event(trace, step, task.plan.revision, record);
            }
            decision
        };

        self.apply_governance_response(
            session,
            task,
            trace,
            step,
            stage_key,
            policy,
            request_kind,
            GovernanceRuntimeKind::Local,
            governance_attempt_id,
            previous_attempt_id,
            packet_reuse,
            decision,
            response,
        )
    }

    fn record_governance_decision_event(
        &self,
        trace: &mut ExecutionTrace,
        step: &Step,
        plan_revision: usize,
        decision: &crate::domain::governance::AutopilotDecisionRecord,
    ) {
        trace.record_event(
            TraceEventType::GovernanceDecisionRecorded,
            Some(step.id.clone()),
            plan_revision,
            json!({
                "stage_key": decision.stage_key,
                "candidate_actions": decision.candidate_actions,
                "candidate_modes": decision.candidate_modes,
                "selected_action": decision.selected_action,
                "selected_mode": decision.selected_mode,
                "selected_target_stage_key": decision.selected_target_stage_key,
                "reason": decision.rationale,
                "blocked_reason": decision.blocked_reason,
            }),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_governance_response(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        step: &Step,
        stage_key: String,
        policy: &crate::domain::governance::StageGovernancePolicy,
        request_kind: GovernanceRequestKind,
        runtime_kind: GovernanceRuntimeKind,
        governance_attempt_id: String,
        previous_attempt_id: Option<String>,
        packet_reuse: Option<crate::domain::governance::PacketReuseBinding>,
        decision: Option<crate::domain::governance::AutopilotDecisionRecord>,
        response: crate::adapters::governance_runtime::GovernanceRuntimeResponse,
    ) -> Result<GovernanceStepDecision<TaskRunResponse>, SessionRuntimeError> {
        let packet_rejected = response.packet.as_ref().is_some_and(|packet| {
            matches!(packet.readiness, PacketReadiness::Incomplete | PacketReadiness::Rejected)
        });
        let effective_status =
            if packet_rejected { GovernanceLifecycleState::Blocked } else { response.status };
        let blocked_reason = if packet_rejected {
            Some(
                decision
                    .as_ref()
                    .and_then(|decision| decision.blocked_reason.clone())
                    .unwrap_or_else(|| {
                        response
                            .packet
                            .as_ref()
                            .map(|packet| {
                                format!(
                                    "governance packet was {:?} for stage {stage_key}",
                                    packet.readiness
                                )
                            })
                            .unwrap_or_else(|| {
                                format!("governance packet was rejected for stage {stage_key}")
                            })
                    }),
            )
        } else {
            matches!(
                response.status,
                GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed
            )
            .then(|| response.message.clone())
        };
        let record = GovernedStageRecord {
            stage_key: stage_key.clone(),
            runtime: runtime_kind,
            lifecycle_state: effective_status,
            required: policy.required,
            autopilot_enabled: policy.autopilot,
            approval_state: response.approval_state,
            canon_run_ref: response.run_ref.clone(),
            governance_attempt_id,
            previous_governance_attempt_id: previous_attempt_id,
            packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
            decision_ref: decision.as_ref().map(|decision| decision.decision_id.clone()),
            blocked_reason: blocked_reason.clone(),
        };
        let patch = governance_state_patch(
            &record,
            response.packet.as_ref(),
            packet_reuse.as_ref(),
            decision.as_ref(),
        )
        .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        task.context.apply_state_patch(&patch);

        if let Some(packet) = response.packet.as_ref()
            && packet_rejected
        {
            trace.record_event(
                TraceEventType::GovernancePacketRejected,
                Some(step.id.clone()),
                task.plan.revision,
                json!({
                    "stage_key": stage_key,
                    "packet_ref": packet.packet_ref,
                    "packet_readiness": packet.readiness,
                    "missing_sections": packet.missing_sections,
                    "reason": blocked_reason.as_deref().unwrap_or(&response.message),
                    "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
                }),
            );
        }

        match effective_status {
            GovernanceLifecycleState::GovernedReady => {
                trace.record_event(
                    TraceEventType::GovernanceCompleted,
                    Some(step.id.clone()),
                    task.plan.revision,
                    json!({
                        "stage_key": stage_key,
                        "runtime": runtime_kind,
                        "packet_ref": response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                        "packet_readiness": response.packet.as_ref().map(|packet| packet.readiness),
                        "document_refs": response.packet.as_ref().map(|packet| packet.document_refs.clone()).unwrap_or_default(),
                        "headline": response.packet.as_ref().map(|packet| packet.headline.clone()).unwrap_or_else(|| response.message.clone()),
                        "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
                    }),
                );
                let trace_location = self.persist_trace(trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                if matches!(request_kind, GovernanceRequestKind::Refresh) {
                    Ok(GovernanceStepDecision::Halt)
                } else {
                    Ok(GovernanceStepDecision::Continue)
                }
            }
            GovernanceLifecycleState::AwaitingApproval => {
                trace.record_event(
                    TraceEventType::GovernanceAwaitingApproval,
                    Some(step.id.clone()),
                    task.plan.revision,
                    json!({
                        "stage_key": stage_key,
                        "runtime": runtime_kind,
                        "approval_state": response.approval_state,
                        "run_ref": response.run_ref,
                        "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
                    }),
                );
                let trace_location = self.persist_trace(trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                Ok(GovernanceStepDecision::Halt)
            }
            GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => {
                let reason = blocked_reason.unwrap_or(response.message.clone());
                trace.record_event(
                    TraceEventType::GovernanceBlocked,
                    Some(step.id.clone()),
                    task.plan.revision,
                    json!({
                        "stage_key": stage_key,
                        "runtime": runtime_kind,
                        "required": policy.required,
                        "reason": reason,
                        "packet_ref": response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                        "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason.clone()),
                    }),
                );
                let trace_location = self.persist_trace(trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();

                if policy.required {
                    let terminal_reason = build_terminal_reason(
                        TerminalCondition::TaskNotCredible,
                        format!("governance blocked stage {stage_key}: {reason}"),
                        Some(json!({
                            "stage_key": stage_key,
                            "runtime": runtime_kind,
                            "required": policy.required,
                        })),
                    );
                    self.finalize_task(session, task, trace, terminal_reason)
                        .map(GovernanceStepDecision::Terminal)
                } else if runtime_kind == GovernanceRuntimeKind::Local
                    && matches!(request_kind, GovernanceRequestKind::Start)
                {
                    Ok(GovernanceStepDecision::Continue)
                } else {
                    Ok(GovernanceStepDecision::Halt)
                }
            }
            _ => Ok(GovernanceStepDecision::Continue),
        }
    }

    fn handle_governance_block(
        &self,
        session: &mut ActiveSessionRecord,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        block: GovernanceBlockContext,
        decision: Option<crate::domain::governance::AutopilotDecisionRecord>,
    ) -> Result<GovernanceStepDecision<TaskRunResponse>, SessionRuntimeError> {
        let record = GovernedStageRecord {
            stage_key: block.stage_key.clone(),
            runtime: block.runtime,
            lifecycle_state: GovernanceLifecycleState::Blocked,
            required: block.required,
            autopilot_enabled: block.autopilot_enabled,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: format!(
                "{}-blocked-{}",
                block.stage_key.replace(':', "-"),
                task.plan.revision
            ),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: decision.as_ref().map(|decision| decision.decision_id.clone()),
            blocked_reason: Some(block.reason.clone()),
        };
        let patch = governance_state_patch(&record, None, None, decision.as_ref())
            .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        task.context.apply_state_patch(&patch);
        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some(block.step_id.clone()),
            task.plan.revision,
            json!({
                "stage_key": block.stage_key,
                "runtime": block.runtime,
                "required": block.required,
                "reason": block.reason,
            }),
        );
        let trace_location = self.persist_trace(trace)?;
        session.latest_trace_ref = Some(trace_location);
        session.updated_at = current_timestamp_millis();

        if block.required {
            let terminal_reason = build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                format!("governance blocked stage {}: {}", block.stage_key, block.reason),
                Some(json!({
                    "stage_key": block.stage_key,
                    "runtime": block.runtime,
                    "required": block.required,
                })),
            );
            self.finalize_task(session, task, trace, terminal_reason)
                .map(GovernanceStepDecision::Terminal)
        } else {
            session.latest_status = SessionStatus::Running;
            session.latest_terminal_reason = None;
            Ok(GovernanceStepDecision::Halt)
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
        if let Some(active_flow) = &session.active_flow {
            trace.record_event(
                TraceEventType::FlowSelected,
                None,
                task.plan.revision,
                json!({
                    "flow_name": active_flow.flow_name,
                    "current_stage_id": active_flow.current_stage_id,
                    "current_stage_index": active_flow.current_stage_index,
                    "total_stages": active_flow.total_stages,
                }),
            );
        }
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

    fn advance_session_flow(
        &self,
        session: &mut ActiveSessionRecord,
        task: &Task,
        completed_step_index: usize,
    ) -> Result<
        Option<(FlowStepMetadata, FlowStepMetadata)>,
        crate::domain::flow::FlowValidationError,
    > {
        let Some(active_flow) = session.active_flow.as_mut() else {
            return Ok(None);
        };

        let completed_step =
            task.plan.steps.get(completed_step_index).expect("completed step index is valid");
        let Some(completed_metadata) = FlowStepMetadata::from_step(completed_step)? else {
            return Ok(None);
        };

        if completed_metadata.flow_name != active_flow.flow_name {
            return Err(crate::domain::flow::FlowValidationError::StageIdMismatch {
                flow_name: active_flow.flow_name.clone(),
                expected: active_flow.current_stage_id.clone(),
                actual: completed_metadata.stage_id,
            });
        }

        if let Some(next_step) = task.plan.steps.get(completed_step_index + 1)
            && let Some(next_metadata) = FlowStepMetadata::from_step(next_step)?
            && next_metadata.stage_index != active_flow.current_stage_index
        {
            active_flow.current_stage_index = next_metadata.stage_index;
            active_flow.current_stage_id = next_metadata.stage_id.clone();
            return Ok(Some((completed_metadata, next_metadata)));
        }

        Ok(None)
    }

    fn flow_payload_for_step(
        &self,
        step: &Step,
    ) -> Result<Option<Value>, crate::domain::flow::FlowValidationError> {
        let Some(metadata) = FlowStepMetadata::from_step(step)? else {
            return Ok(None);
        };

        Ok(Some(json!({
            "flow_name": metadata.flow_name,
            "stage_id": metadata.stage_id,
            "stage_index": metadata.stage_index,
            "total_stages": metadata.total_stages,
        })))
    }

    fn record_stage_failure(
        &self,
        trace: &mut ExecutionTrace,
        session: &ActiveSessionRecord,
        step_id: &str,
        plan_revision: usize,
        reason: &TerminalReason,
    ) {
        let Some(active_flow) = &session.active_flow else {
            return;
        };

        trace.record_event(
            TraceEventType::StageFailed,
            Some(step_id.to_string()),
            plan_revision,
            json!({
                "flow_name": active_flow.flow_name,
                "stage_id": active_flow.current_stage_id,
                "stage_index": active_flow.current_stage_index,
                "reason": reason.message,
            }),
        );
    }

    fn execute_step(
        &self,
        runtime: &FixtureRuntime,
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
        runtime: &FixtureRuntime,
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
        runtime: &FixtureRuntime,
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
        if terminal_status != TaskStatus::Succeeded {
            let step_id = task
                .plan
                .current_step()
                .map(|step| step.id.clone())
                .unwrap_or_else(|| "terminal".to_string());
            self.record_stage_failure(trace, session, &step_id, task.plan.revision, &reason);
        }
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

struct GovernanceBlockContext {
    step_id: String,
    stage_key: String,
    required: bool,
    autopilot_enabled: bool,
    runtime: GovernanceRuntimeKind,
    reason: String,
}

#[derive(Debug, Error)]
pub enum SessionRuntimeError {
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("trace store operation failed: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("active session has no captured goal")]
    MissingGoal,
    #[error("{headline}: {prompt}")]
    ClarificationRequired { headline: String, prompt: String },
    #[error("goal planning failed: {0}")]
    GoalPlanner(#[from] GoalPlannerError),
    #[error("goal plan state is invalid: {0}")]
    InvalidGoalPlan(String),
    #[error(
        "native execution cannot continue until the proposed `{flow_name}` flow is confirmed or skipped"
    )]
    FlowConfirmationRequired { flow_name: String },
    #[error("unknown flow `{requested}`; supported flows: {supported}")]
    UnknownFlow { requested: String, supported: String },
    #[error(
        "cannot replace active flow `{current}` with `{requested}` while work is still present"
    )]
    FlowReplacementRequiresReset { current: String, requested: String },
    #[error("active session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("active session has no planned task")]
    MissingActiveTask,
    #[error("active session is missing the persisted trace reference")]
    MissingTraceReference,
    #[error("active task is missing a terminal reason")]
    MissingTerminalReason,
    #[error("fixture runtime is invalid: {0}")]
    FixtureRuntime(#[source] FixtureRuntimeError),
    #[error("task request is invalid: {0}")]
    TaskRequest(#[source] TaskRequestError),
    #[error("task context state is invalid: {0}")]
    TaskContext(String),
    #[error("governance state patch is invalid: {0}")]
    GovernancePatch(String),
    #[error("governance runtime failed: {0}")]
    GovernanceRuntime(String),
    #[error("decision loop failed: {0}")]
    DecisionLoop(String),
}

fn native_terminal_outcome(terminal: &LoopTerminal) -> (TerminalCondition, String, Option<Value>) {
    match terminal {
        LoopTerminal::Success => (
            TerminalCondition::GoalSatisfied,
            "goal plan completed through the native decision loop".to_string(),
            None,
        ),
        LoopTerminal::Failure(message) => {
            (TerminalCondition::UnrecoverableError, message.clone(), None)
        }
        LoopTerminal::Exhausted { steps_taken, max_steps } => (
            TerminalCondition::StepLimitExceeded,
            "native decision loop exhausted its step budget".to_string(),
            Some(json!({
                "steps_taken": steps_taken,
                "max_steps": max_steps,
            })),
        ),
        LoopTerminal::NoActionableState(message) => {
            (TerminalCondition::NoCredibleNextStep, message.clone(), None)
        }
    }
}

fn native_analyze_workspace(
    workspace_ref: &Path,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let (target, path) = match request_target_path(workspace_ref, &request) {
        Ok(target) => target,
        Err(error) => return StepExecutionResult::failure(error, Recoverability::ReplanRequired),
    };

    if path.is_dir() {
        let entries = std::fs::read_dir(&path).map_err(|error| error.to_string());
        return match entries {
            Ok(entries) => {
                let listing = entries
                    .flatten()
                    .map(|entry| entry.file_name().to_string_lossy().to_string())
                    .collect::<Vec<_>>();
                StepExecutionResult::success_with_patch(
                    json!({
                        "target": target,
                        "stdout": listing.join("\n"),
                        "entry_count": listing.len(),
                    }),
                    Map::from_iter([(
                        "latest_selection_headline".to_string(),
                        json!(format!("analyzed directory {target}")),
                    )]),
                )
                .with_evidence(json!({"kind": "directory", "target": target}))
            }
            Err(error) => StepExecutionResult::failure(
                ErrorInfo::new(
                    "directory_read_failed",
                    format!("failed to read {target}: {error}"),
                ),
                Recoverability::ReplanRequired,
            ),
        };
    }

    match std::fs::read_to_string(&path) {
        Ok(contents) => StepExecutionResult::success_with_patch(
            json!({
                "target": target,
                "stdout": contents,
            }),
            Map::from_iter([(
                "latest_selection_headline".to_string(),
                json!(format!("analyzed {target}")),
            )]),
        )
        .with_evidence(json!({"kind": "file", "target": target})),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new("file_read_failed", format!("failed to read {target}: {error}")),
            Recoverability::ReplanRequired,
        ),
    }
}

fn native_apply_workspace_change(
    workspace_ref: &Path,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let (target, path) = match request_target_path(workspace_ref, &request) {
        Ok(target) => target,
        Err(error) => return StepExecutionResult::failure(error, Recoverability::ReplanRequired),
    };

    let original = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new("file_read_failed", format!("failed to read {target}: {error}")),
                Recoverability::ReplanRequired,
            );
        }
    };

    let (updated, diff, summary) = if original.contains("left - right") {
        (
            original.replacen("left - right", "left + right", 1),
            "- left - right\n+ left + right".to_string(),
            format!("applied deterministic arithmetic fix to {target}"),
        )
    } else if original.contains("left / right") {
        (
            original.replacen("left / right", "left + right", 1),
            "- left / right\n+ left + right".to_string(),
            format!("replaced unsafe arithmetic in {target}"),
        )
    } else {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "native_change_unavailable",
                format!("no deterministic native change is available for {target}"),
            ),
            Recoverability::ReplanRequired,
        );
    };

    if let Err(error) = std::fs::write(&path, &updated) {
        return StepExecutionResult::failure(
            ErrorInfo::new("file_write_failed", format!("failed to write {target}: {error}")),
            Recoverability::Terminal,
        );
    }

    StepExecutionResult::success_with_patch(
        json!({
            "target": target,
            "stdout": summary,
            "diff": diff,
            "changed_files": [target.clone()],
        }),
        Map::from_iter([
            ("latest_changed_files".to_string(), json!([target.clone()])),
            ("latest_selection_headline".to_string(), json!(format!("applied change to {target}"))),
        ]),
    )
}

fn native_run_validation(
    workspace_ref: &Path,
    _request: StepExecutionRequest,
) -> StepExecutionResult {
    match std::process::Command::new("cargo")
        .arg("test")
        .arg("--quiet")
        .current_dir(workspace_ref)
        .output()
    {
        Ok(output) => {
            let payload = json!({
                "command": "cargo test --quiet",
                "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
                "exit_code": output.status.code().unwrap_or(-1),
            });
            let state_patch = Map::from_iter([(
                "latest_validation_status".to_string(),
                json!(if output.status.success() { "passed" } else { "failed" }),
            )]);

            if output.status.success() {
                StepExecutionResult::success_with_patch(payload.clone(), state_patch)
                    .with_evidence(payload)
            } else {
                StepExecutionResult::failure(
                    ErrorInfo::new("validation_failed", "cargo test --quiet reported failures")
                        .with_details(payload.clone()),
                    Recoverability::ReplanRequired,
                )
                .with_evidence(payload)
                .with_state_patch(state_patch)
            }
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new("validation_command_failed", error.to_string()),
            Recoverability::Terminal,
        ),
    }
}

fn native_replan_step(request: StepExecutionRequest) -> StepExecutionResult {
    let target = request.input.get("target").and_then(Value::as_str).unwrap_or("current-task");
    StepExecutionResult::success_with_patch(
        json!({
            "target": target,
            "stdout": format!("recorded a recovery decision for {target}"),
        }),
        Map::from_iter([(
            "latest_selection_headline".to_string(),
            json!(format!("recorded recovery decision for {target}")),
        )]),
    )
}

fn request_target_path(
    workspace_ref: &Path,
    request: &StepExecutionRequest,
) -> Result<(String, PathBuf), ErrorInfo> {
    let target = request
        .input
        .get("target")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|target| !target.is_empty())
        .ok_or_else(|| ErrorInfo::new("missing_target", "request did not include a target path"))?
        .to_string();

    Ok((target.clone(), workspace_ref.join(&target)))
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use serde_json::{Map, json};
    use uuid::Uuid;

    use super::{SessionRuntime, session_status_for_task_status};
    use crate::adapters::trace_store::TraceStore;
    use crate::domain::execution::{
        ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
        WorkspaceExecutionProfile,
    };
    use crate::domain::flow::{attach_stage_metadata, built_in_flow};
    use crate::domain::governance::{
        CanonMode, CanonRuntimeConfig, GovernanceLifecycleState, GovernanceProfile,
        GovernanceRuntimeKind, PacketReadiness, StageGovernancePolicy, SystemContextBinding,
    };
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::{ExecutionStatus, Recoverability, Step, StepStatus};
    use crate::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{ExecutionTrace, TraceEventType};
    use crate::fixture::{FixtureRuntime, build_fixture_plan_for_goal, build_task_request};
    use crate::orchestrator::planner::StaticPlanner;
    use crate::registry::agent_registry::AgentRegistry;
    use crate::registry::tool_registry::ToolRegistry;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        workspace
    }

    fn write_execution_profile_workspace(
        prefix: &str,
        attempts: Vec<ExecutionAttemptDefinition>,
    ) -> PathBuf {
        write_governed_execution_profile_workspace(prefix, attempts, Vec::new(), None)
    }

    fn write_governed_execution_profile_workspace(
        prefix: &str,
        attempts: Vec<ExecutionAttemptDefinition>,
        read_targets: Vec<String>,
        governance: Option<GovernanceProfile>,
    ) -> PathBuf {
        let workspace = temp_workspace(prefix);
        fs::write(
            workspace.join(".synod/execution.json"),
            serde_json::to_string_pretty(&WorkspaceExecutionProfile {
                name: "session-runtime-profile".to_string(),
                read_targets,
                validation_command: ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string(), "--quiet".to_string()],
                },
                attempts,
                adaptive: None,
                limits: RunLimits::default(),
                governance,
                review: None,
                legacy_source: None,
            })
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn build_request(workspace_ref: &str) -> TaskRunRequest {
        TaskRunRequest {
            goal: "Drive a session runtime branch".to_string(),
            input: json!({"ticket": "SESSION-RUNTIME"}),
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace_ref.to_string(),
            limits: RunLimits::default(),
            initial_context: None,
        }
    }

    fn decision_task(workspace_ref: &str, input: serde_json::Value) -> Task {
        let plan = Plan::new(vec![Step::decision("decide", input).unwrap()]).unwrap();
        Task::new("task-runtime", &build_request(workspace_ref), plan).unwrap()
    }

    fn build_session(workspace: &Path, task: Task) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Drive a session runtime branch".to_string()),
            authored_brief: None,
            active_flow: None,
            active_task: Some(task),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        }
    }

    fn build_fixture_session_task(workspace: &Path, session: &ActiveSessionRecord) -> Task {
        let request = build_task_request(
            workspace,
            session.goal.clone().unwrap_or_else(|| "Drive a session runtime branch".to_string()),
            session.session_id.clone(),
            session.authored_brief.as_ref(),
        )
        .unwrap();
        let plan = build_fixture_plan_for_goal(
            workspace,
            session.active_flow.as_ref(),
            session.goal.as_deref().unwrap_or_default(),
        )
        .unwrap();

        Task::new("task-runtime", &request, plan).unwrap()
    }

    fn manual_runtime() -> FixtureRuntime {
        FixtureRuntime {
            profile: WorkspaceExecutionProfile {
                name: "manual-runtime".to_string(),
                read_targets: Vec::new(),
                validation_command: ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string(), "--quiet".to_string()],
                },
                attempts: vec![ExecutionAttemptDefinition {
                    attempt_id: "fix-add".to_string(),
                    summary: String::new(),
                    failure_mode: ExecutionFailureMode::Terminal,
                    changes: vec![WorkspaceChange {
                        path: "src/lib.rs".to_string(),
                        find: "left - right".to_string(),
                        replace: "left + right".to_string(),
                    }],
                }],
                adaptive: None,
                limits: RunLimits::default(),
                governance: None,
                review: None,
                legacy_source: None,
            },
            planner: std::sync::Arc::new(StaticPlanner::new(
                Plan::new(vec![Step::decision("placeholder", json!({})).unwrap()]).unwrap(),
            )),
            agents: AgentRegistry::new(),
            tools: ToolRegistry::new(),
        }
    }

    fn context() -> TaskContext {
        TaskContext::new("session-runtime", "/tmp/workspace", RunLimits::default(), Map::new())
    }

    #[test]
    fn execute_step_routes_agent_tool_and_decision_edge_cases() {
        let runtime = SessionRuntime::for_workspace(temp_workspace("synod-runtime-routing"));
        let fixture_runtime = manual_runtime();
        let context = context();

        let mut missing_agent_target = Step::agent("agent", "coder", json!({})).unwrap();
        missing_agent_target.target_name = None;
        let missing_agent = runtime.execute_step(&fixture_runtime, &missing_agent_target, &context);
        assert_eq!(missing_agent.status, ExecutionStatus::Failed);
        assert_eq!(missing_agent.recoverability, Recoverability::Terminal);

        let unknown_agent = runtime.execute_step(
            &fixture_runtime,
            &Step::agent("agent", "unknown", json!({})).unwrap(),
            &context,
        );
        assert_eq!(unknown_agent.status, ExecutionStatus::Failed);

        let mut missing_tool_target = Step::tool("tool", "tester", json!({})).unwrap();
        missing_tool_target.target_name = None;
        let missing_tool = runtime.execute_step(&fixture_runtime, &missing_tool_target, &context);
        assert_eq!(missing_tool.status, ExecutionStatus::Failed);

        let unknown_tool = runtime.execute_step(
            &fixture_runtime,
            &Step::tool("tool", "unknown", json!({})).unwrap(),
            &context,
        );
        assert_eq!(unknown_tool.status, ExecutionStatus::Failed);

        let plain_decision =
            runtime.execute_decision(&Step::decision("plain", json!("ok")).unwrap());
        assert_eq!(plain_decision.status, ExecutionStatus::Succeeded);

        let retry_decision = runtime.execute_decision(
            &Step::decision("retry", json!({"retryable_failure": true})).unwrap(),
        );
        assert_eq!(retry_decision.recoverability, Recoverability::Retryable);

        let replan_decision = runtime
            .execute_decision(&Step::decision("replan", json!({"replan_required": true})).unwrap());
        assert_eq!(replan_decision.recoverability, Recoverability::ReplanRequired);

        let terminal_decision = runtime.execute_decision(
            &Step::decision("terminal", json!({"terminal_failure": true})).unwrap(),
        );
        assert_eq!(terminal_decision.recoverability, Recoverability::Terminal);

        let patched_decision = runtime.execute_decision(
            &Step::decision(
                "patched",
                json!({"output": {"ok": true}, "state_patch": {"goal_satisfied": true}}),
            )
            .unwrap(),
        );
        assert_eq!(patched_decision.status, ExecutionStatus::Succeeded);
        assert_eq!(patched_decision.state_patch.as_ref().unwrap()["goal_satisfied"], json!(true));

        assert_eq!(
            runtime.session_store().path(),
            runtime.workspace_ref().join(".synod/session.json")
        );
        assert_eq!(runtime.trace_store().root(), runtime.workspace_ref().join(".synod/traces"));
        assert_eq!(session_status_for_task_status(TaskStatus::Aborted), SessionStatus::Aborted);
    }

    #[test]
    fn load_or_create_trace_and_flow_helpers_cover_private_flow_branches() {
        let workspace = write_execution_profile_workspace(
            "synod-runtime-flow-helpers",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
        );
        let runtime = SessionRuntime::for_workspace(&workspace);

        let flow = built_in_flow("bug-fix").unwrap();
        let stage0 = Step::agent(
            "investigate",
            "analyzer",
            attach_stage_metadata(json!({"phase": "investigate"}), flow, 0).unwrap(),
        )
        .unwrap();
        let stage1 = Step::agent(
            "implement",
            "coder",
            attach_stage_metadata(json!({"phase": "implement"}), flow, 1).unwrap(),
        )
        .unwrap();
        let request = build_request(workspace.to_string_lossy().as_ref());
        let task = Task::new(
            "task-flow",
            &request,
            Plan::new(vec![stage0.clone(), stage1.clone()]).unwrap(),
        )
        .unwrap();
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Drive a session runtime branch".to_string()),
            authored_brief: None,
            active_flow: Some(flow.initial_state()),
            active_task: Some(task.clone()),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        let created = runtime.load_or_create_trace(&mut session, &task).unwrap();
        assert_eq!(created.events[0].event_type, TraceEventType::TaskStarted);
        assert_eq!(created.events[1].event_type, TraceEventType::FlowSelected);

        let reused = runtime.load_or_create_trace(&mut session, &task).unwrap();
        assert_eq!(reused.goal, created.goal);

        let transition = runtime.advance_session_flow(&mut session, &task, 0).unwrap().unwrap();
        assert_eq!(transition.0.stage_id, "investigate");
        assert_eq!(transition.1.stage_id, "implement");
        assert_eq!(session.active_flow.as_ref().unwrap().current_stage_id, "implement");

        let payload = runtime.flow_payload_for_step(&stage0).unwrap().unwrap();
        assert_eq!(payload["stage_id"], json!("investigate"));
        assert_eq!(
            runtime.flow_payload_for_step(&Step::decision("plain", json!({})).unwrap()).unwrap(),
            None
        );

        let mut trace = ExecutionTrace::new("task-flow", "session-runtime", "goal");
        runtime.record_stage_failure(
            &mut trace,
            &session,
            "implement",
            0,
            &TerminalReason::new(TerminalCondition::UnrecoverableError, "failed", None),
        );
        assert_eq!(trace.events[0].event_type, TraceEventType::StageFailed);
    }

    #[test]
    fn execute_next_step_covers_retry_replan_and_terminal_decision_recovery() {
        let workspace = write_execution_profile_workspace(
            "synod-runtime-decision-recovery",
            vec![
                ExecutionAttemptDefinition {
                    attempt_id: "bad-fix".to_string(),
                    summary: String::new(),
                    failure_mode: ExecutionFailureMode::Replan,
                    changes: vec![WorkspaceChange {
                        path: "src/lib.rs".to_string(),
                        find: "left - right".to_string(),
                        replace: "left / right".to_string(),
                    }],
                },
                ExecutionAttemptDefinition {
                    attempt_id: "good-fix".to_string(),
                    summary: String::new(),
                    failure_mode: ExecutionFailureMode::Terminal,
                    changes: vec![WorkspaceChange {
                        path: "src/lib.rs".to_string(),
                        find: "left / right".to_string(),
                        replace: "left + right".to_string(),
                    }],
                },
            ],
        );
        let runtime = SessionRuntime::for_workspace(&workspace);

        let mut retry_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"retryable_failure": true})),
        );
        runtime.execute_next_step(&mut retry_session).unwrap();
        assert_eq!(retry_session.active_task.as_ref().unwrap().retry_count, 1);
        assert_eq!(
            retry_session.active_task.as_ref().unwrap().plan.steps[0].status,
            StepStatus::Pending
        );

        let mut replan_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"replan_required": true})),
        );
        runtime.execute_next_step(&mut replan_session).unwrap();
        assert_eq!(replan_session.active_task.as_ref().unwrap().replan_count, 1);
        assert_eq!(replan_session.active_task.as_ref().unwrap().plan.revision, 1);

        let mut terminal_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"terminal_failure": true})),
        );
        runtime.execute_next_step(&mut terminal_session).unwrap();
        assert_eq!(terminal_session.latest_status, SessionStatus::Failed);
        assert!(terminal_session.latest_terminal_reason.is_some());
    }

    #[test]
    fn execute_next_step_falls_back_to_local_governance_when_canon_is_optional() {
        let workspace = write_governed_execution_profile_workspace(
            "synod-runtime-governance-local-fallback",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            vec!["README.md".to_string()],
            Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Local,
                canon: Some(CanonRuntimeConfig {
                    command: "/definitely/missing/canon".to_string(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: vec![StageGovernancePolicy {
                    flow_name: "bug-fix".to_string(),
                    stage_id: "investigate".to_string(),
                    enabled: true,
                    required: false,
                    autopilot: false,
                    runtime: Some(GovernanceRuntimeKind::Canon),
                    canon_mode: Some(CanonMode::Discovery),
                    system_context: Some(SystemContextBinding::Existing),
                    risk: Some("medium".to_string()),
                    zone: Some("engineering".to_string()),
                    owner: Some("platform".to_string()),
                }],
            }),
        );
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
        runtime.select_flow(&mut session, "bug-fix").unwrap();
        session.active_task = Some(build_fixture_session_task(&workspace, &session));
        session.latest_status = SessionStatus::Planned;
        runtime.execute_next_step(&mut session).unwrap();

        let task = session.active_task.as_ref().unwrap();
        let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
        let governed_packet = task.context.latest_governance_packet().unwrap().unwrap();
        assert_eq!(governed_stage.stage_key, "bug-fix:investigate");
        assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);
        assert_eq!(governed_packet.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(governed_packet.readiness, PacketReadiness::Reusable);
        assert!(!governed_packet.document_refs.is_empty());

        let trace = runtime
            .trace_store()
            .load(Path::new(session.latest_trace_ref.as_ref().unwrap()))
            .unwrap();
        assert!(
            trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceSelected),
            "{:?}",
            trace.events
        );
        assert!(
            trace
                .events
                .iter()
                .any(|event| event.event_type == TraceEventType::GovernanceCompleted),
            "{:?}",
            trace.events
        );
    }

    #[test]
    fn execute_next_step_blocks_when_required_canon_governance_is_unavailable() {
        let workspace = write_governed_execution_profile_workspace(
            "synod-runtime-governance-required-canon",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            vec!["README.md".to_string()],
            Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Local,
                canon: Some(CanonRuntimeConfig {
                    command: "/definitely/missing/canon".to_string(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: vec![StageGovernancePolicy {
                    flow_name: "bug-fix".to_string(),
                    stage_id: "investigate".to_string(),
                    enabled: true,
                    required: true,
                    autopilot: false,
                    runtime: Some(GovernanceRuntimeKind::Canon),
                    canon_mode: Some(CanonMode::Discovery),
                    system_context: Some(SystemContextBinding::Existing),
                    risk: Some("medium".to_string()),
                    zone: Some("engineering".to_string()),
                    owner: Some("platform".to_string()),
                }],
            }),
        );
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
        runtime.select_flow(&mut session, "bug-fix").unwrap();
        session.active_task = Some(build_fixture_session_task(&workspace, &session));
        session.latest_status = SessionStatus::Planned;
        runtime.execute_next_step(&mut session).unwrap();

        let task = session.active_task.as_ref().unwrap();
        let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
        assert_eq!(session.latest_status, SessionStatus::Failed);
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(governed_stage.stage_key, "bug-fix:investigate");
        assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Canon);
        assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert!(task.context.latest_governance_packet().unwrap().is_none());
        assert!(
            session
                .latest_terminal_reason
                .as_ref()
                .unwrap()
                .message
                .contains("governance blocked stage bug-fix:investigate")
        );
        assert_eq!(task.plan.current_step_index, 0);
        assert_eq!(task.plan.steps[0].status, StepStatus::Pending);

        let trace = runtime
            .trace_store()
            .load(Path::new(session.latest_trace_ref.as_ref().unwrap()))
            .unwrap();
        assert!(
            trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceBlocked),
            "{:?}",
            trace.events
        );
    }

    #[test]
    fn plan_task_persists_goal_plan_and_inferred_flow_without_creating_fixture_task() {
        let workspace = temp_workspace("synod-runtime-native-plan");
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "fix the broken add function").unwrap();
        runtime.plan_task(&mut session, None, false).unwrap();

        assert_eq!(session.latest_status, SessionStatus::Planned);
        assert!(session.active_task.is_none());
        assert!(session.decisions.is_empty());
        assert!(session.active_flow.is_none());
        assert!(session.active_flow_policy.is_none());

        let goal_plan = session.goal_plan.as_ref().unwrap();
        assert_eq!(goal_plan.status, crate::domain::goal_plan::GoalPlanStatus::Confirmed);
        assert!(!goal_plan.tasks.is_empty());
        assert_eq!(goal_plan.flow.as_ref().unwrap().flow_name, "bug-fix");
        assert!(!goal_plan.flow.as_ref().unwrap().confirmed);
        session.validate().unwrap();
    }

    #[test]
    fn plan_task_confirms_explicit_flow_during_native_planning() {
        let workspace = temp_workspace("synod-runtime-native-plan-explicit-flow");
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "implement workspace summary output").unwrap();
        runtime.plan_task(&mut session, Some("change"), false).unwrap();

        assert_eq!(session.active_flow.as_ref().unwrap().flow_name, "change");
        assert_eq!(session.active_flow_policy.as_ref().unwrap().flow_name, "change");
        assert!(session.goal_plan.as_ref().unwrap().flow.as_ref().unwrap().confirmed);
        session.validate().unwrap();
    }

    #[test]
    fn run_to_terminal_uses_native_goal_plan_route_when_present() {
        let workspace = temp_workspace("synod-runtime-native-run");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"native-run\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left - right }\n",
        )
        .unwrap();

        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "fix the broken add function").unwrap();
        runtime.plan_task(&mut session, Some("bug-fix"), false).unwrap();
        let response = runtime.run_to_terminal(&mut session).unwrap();

        assert_eq!(response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(session.latest_status, SessionStatus::Succeeded);
        assert!(session.active_task.is_none());
        assert!(!session.decisions.is_empty());

        let trace = runtime
            .trace_store()
            .load(Path::new(session.latest_trace_ref.as_ref().unwrap()))
            .unwrap();
        assert!(
            trace.events.iter().any(|event| event.event_type == TraceEventType::DecisionCreated),
            "{:?}",
            trace.events
        );
    }

    #[test]
    fn run_to_terminal_materializes_security_assessment_task_without_dropping_native_routing() {
        let workspace = temp_workspace("synod-runtime-security-assessment-route");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".canon/runs/canon-run-security")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"security-route\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        )
        .unwrap();
        fs::write(
            workspace.join("tests/red_to_green.rs"),
            "use security_route::add;\n\n#[test]\nfn synod_drives_red_to_green() {\n    assert_eq!(add(2, 2), 4);\n}\n",
        )
        .unwrap();
        fs::write(
            workspace.join(".canon/runs/canon-run-security/security-assessment.md"),
            "# Security Assessment\n\nValidated the bounded security review for the verify stage.\n",
        )
        .unwrap();
        fs::write(
            workspace.join(".synod/canon-stub.sh"),
            "#!/bin/sh\ncat >/dev/null\nprintf '%s' '{\"status\":\"governed_ready\",\"run_ref\":\"canon-run-security\",\"packet_ref\":\".canon/runs/canon-run-security\",\"expected_document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"document_refs\":[\".canon/runs/canon-run-security/security-assessment.md\"],\"approval_state\":\"not_needed\",\"packet_readiness\":\"reusable\",\"missing_sections\":[],\"headline\":\"security assessment packet ready\",\"message\":\"Canon completed the governed security assessment\"}'\n",
        )
        .unwrap();
        let mut permissions =
            fs::metadata(workspace.join(".synod/canon-stub.sh")).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(workspace.join(".synod/canon-stub.sh"), permissions).unwrap();
        fs::write(
            workspace.join(".synod/execution.json"),
            serde_json::to_string_pretty(&WorkspaceExecutionProfile {
                name: "session-runtime-profile".to_string(),
                read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
                validation_command: ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string(), "--quiet".to_string()],
                },
                attempts: vec![ExecutionAttemptDefinition {
                    attempt_id: "fix-add".to_string(),
                    summary: String::new(),
                    failure_mode: ExecutionFailureMode::Terminal,
                    changes: vec![WorkspaceChange {
                        path: "src/lib.rs".to_string(),
                        find: "left - right".to_string(),
                        replace: "left + right".to_string(),
                    }],
                }],
                adaptive: None,
                limits: RunLimits::default(),
                governance: Some(GovernanceProfile {
                    default_runtime: GovernanceRuntimeKind::Local,
                    canon: Some(CanonRuntimeConfig {
                        command: workspace
                            .join(".synod/canon-stub.sh")
                            .to_string_lossy()
                            .into_owned(),
                        default_owner: Some("platform".to_string()),
                        default_risk: Some("medium".to_string()),
                        default_zone: Some("engineering".to_string()),
                        default_system_context: Some(SystemContextBinding::Existing),
                    }),
                    stages: vec![StageGovernancePolicy {
                        flow_name: "bug-fix".to_string(),
                        stage_id: "verify".to_string(),
                        enabled: true,
                        required: true,
                        autopilot: true,
                        runtime: Some(GovernanceRuntimeKind::Canon),
                        canon_mode: None,
                        system_context: Some(SystemContextBinding::Existing),
                        risk: Some("medium".to_string()),
                        zone: Some("engineering".to_string()),
                        owner: Some("platform".to_string()),
                    }],
                }),
                review: None,
                legacy_source: None,
            })
            .unwrap(),
        )
        .unwrap();

        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
        };

        runtime.capture_goal(&mut session, "fix the broken add function").unwrap();
        runtime.plan_task(&mut session, Some("bug-fix"), false).unwrap();

        let response = runtime.run_to_terminal(&mut session).unwrap();
        let routing = runtime.resolve_routing_outcome(&session).unwrap();
        let task = session.active_task.as_ref().expect("governed task should persist");
        let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
        let governed_packet = task.context.latest_governance_packet().unwrap().unwrap();
        let trace = runtime
            .trace_store()
            .load(Path::new(session.latest_trace_ref.as_ref().unwrap()))
            .unwrap();

        assert_eq!(response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(session.latest_status, SessionStatus::Succeeded);
        assert_eq!(routing.mode, crate::domain::session::RoutingMode::Native);
        assert_eq!(governed_stage.stage_key, "bug-fix:verify");
        assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Canon);
        assert_eq!(governed_packet.canon_mode, Some(CanonMode::SecurityAssessment));
        assert_eq!(governed_packet.packet_ref, ".canon/runs/canon-run-security");
        assert!(trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceStarted
                && event.payload.get("canon_mode") == Some(&json!("security-assessment"))
        }));
        assert!(
            trace
                .events
                .iter()
                .any(|event| { event.event_type == TraceEventType::GovernanceCompleted })
        );
    }
}
