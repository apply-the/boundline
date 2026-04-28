use serde_json::{Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::governance_runtime::{
    CanonCliRuntime, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeRequest,
    LocalGovernanceRuntime,
};
use crate::adapters::trace_store::{TraceStore, TraceStoreError};
use crate::domain::flow::FlowStepMetadata;
use crate::domain::governance::{
    ApprovalState, GovernanceLifecycleState, GovernanceProfile, GovernanceRuntimeKind,
    GovernedStageRecord, PacketReadiness, resolved_canon_mode,
};
use crate::domain::limits::TerminalCondition;
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionRequest,
    StepExecutionResult, StepKind, StepResultSummary, StepStatus,
};
use crate::domain::task::{Task, TaskRequestError, TaskRunRequest, TaskRunResponse};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType, current_timestamp_millis};
use crate::orchestrator::governance::{
    GovernanceStepDecision, bounded_governance_context, build_autopilot_decision,
    governance_input_documents, governance_stage_key, governance_state_patch,
    overlay_stage_policy_with_intent, requested_governance_intent, runtime_command_available,
    selected_stage_policy,
};
use crate::orchestrator::planner::{Planner, PlanningError};
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::review_trace::{record_review_step_completed, record_review_step_started};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

pub struct Orchestrator<P, S> {
    planner: P,
    agents: AgentRegistry,
    tools: ToolRegistry,
    trace_store: S,
    read_targets: Vec<String>,
    governance: Option<GovernanceProfile>,
}

impl<P, S> Orchestrator<P, S>
where
    P: Planner,
    S: TraceStore,
{
    pub fn new(planner: P, agents: AgentRegistry, tools: ToolRegistry, trace_store: S) -> Self {
        Self { planner, agents, tools, trace_store, read_targets: Vec::new(), governance: None }
    }

    pub fn with_governance(
        mut self,
        read_targets: Vec<String>,
        governance: Option<GovernanceProfile>,
    ) -> Self {
        self.read_targets = read_targets;
        self.governance = governance;
        self
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

            match self.ensure_stage_governance(&mut task, &mut trace)? {
                GovernanceStepDecision::Continue => {}
                GovernanceStepDecision::Halt => {
                    unreachable!("engine governance never halts non-terminally")
                }
                GovernanceStepDecision::Terminal(trace_location) => break trace_location,
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
            record_review_step_started(
                &mut trace,
                &step_snapshot.id,
                &step_snapshot.input,
                &task.context.state,
                task.plan.revision,
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
                    record_review_step_completed(
                        &mut trace,
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
                        break self.finalize_task(&mut task, &mut trace, reason)?;
                    }

                    task.plan.advance();
                    self.persist_trace(&mut trace)?;
                }
                ExecutionStatus::Failed => {
                    let error = result.error.clone().expect("failed results are validated");
                    task.plan.steps[step_index].mark_failed(error.clone(), result.recoverability);
                    task.context.apply_failure_error(&step_snapshot.id, &error);
                    if let Some(state_patch) = result.state_patch.as_ref() {
                        task.context.apply_state_patch(state_patch);
                    }
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
                    record_review_step_completed(
                        &mut trace,
                        &step_snapshot.id,
                        &step_snapshot.input,
                        &result,
                        &task.context.state,
                        task.plan.revision,
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

    fn ensure_stage_governance(
        &self,
        task: &mut Task,
        trace: &mut ExecutionTrace,
    ) -> Result<GovernanceStepDecision<String>, OrchestratorError> {
        let Some(step) = task.plan.current_step().cloned() else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let Some(metadata) = FlowStepMetadata::from_step(&step)
            .map_err(|error| OrchestratorError::InvalidFlowState(error.to_string()))?
        else {
            return Ok(GovernanceStepDecision::Continue);
        };
        let Some(governance) = self.governance.as_ref() else {
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
            .map_err(|error| OrchestratorError::TaskContext(error.to_string()))?
            && existing_record.stage_key == stage_key
        {
            return Ok(GovernanceStepDecision::Continue);
        }

        let existing_packet = task
            .context
            .latest_governance_packet()
            .map_err(|error| OrchestratorError::TaskContext(error.to_string()))?;
        let governance_attempt_id =
            format!("{}-attempt-{}", stage_key.replace(':', "-"), task.plan.revision);
        let (bounded_context, packet_reuse) =
            bounded_governance_context(&task.context, &metadata, &self.read_targets)
                .map_err(|error| OrchestratorError::GovernancePatch(error.to_string()))?;
        let input_documents = governance_input_documents(&task.input);

        let requested_runtime = policy.effective_runtime(governance.default_runtime);
        let canon_available = governance
            .canon
            .as_ref()
            .is_some_and(|canon| runtime_command_available(&canon.command));
        let mut decision = if requested_runtime == GovernanceRuntimeKind::Canon {
            build_autopilot_decision(
                &governance_attempt_id,
                &policy,
                governance.default_runtime,
                &metadata,
                &bounded_context,
                None,
                None,
                existing_packet.as_ref().map(|packet| packet.readiness),
            )
        } else {
            None
        };
        let mut mode = decision
            .as_ref()
            .and_then(|record| record.selected_mode)
            .or_else(|| resolved_canon_mode(&policy, governance.default_runtime))
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
            self.record_governance_decision_event(trace, &step, task.plan.revision, decision);
        }

        if requested_runtime == GovernanceRuntimeKind::Canon
            && selected_runtime == GovernanceRuntimeKind::Canon
        {
            let Some(canon) = governance.canon.as_ref() else {
                return self.handle_governance_block(
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
                request_kind: GovernanceRequestKind::Start,
                governance_attempt_id: governance_attempt_id.clone(),
                stage_key: stage_key.clone(),
                goal: task.goal.clone(),
                workspace_ref: task.context.workspace_ref.clone(),
                autopilot: policy.autopilot,
                mode: Some(mode_value),
                system_context: policy.system_context.or(canon.default_system_context),
                risk: policy.risk.clone().or_else(|| canon.default_risk.clone()),
                zone: policy.zone.clone().or_else(|| canon.default_zone.clone()),
                owner: policy.owner.clone().or_else(|| canon.default_owner.clone()),
                run_ref: None,
                packet_ref: existing_packet.as_ref().map(|packet| packet.packet_ref.clone()),
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
                }),
            );
            let response = CanonCliRuntime::new(canon.command.clone())
                .execute(&request)
                .map_err(|error| OrchestratorError::GovernanceRuntime(error.to_string()))?;
            let decision = if decision.is_some() {
                decision
            } else {
                let decision = build_autopilot_decision(
                    &governance_attempt_id,
                    &policy,
                    governance.default_runtime,
                    &metadata,
                    &request.bounded_context,
                    Some(response.status),
                    Some(response.approval_state),
                    response.packet.as_ref().map(|packet| packet.readiness),
                );
                if let Some(record) = &decision {
                    self.record_governance_decision_event(trace, &step, task.plan.revision, record);
                }
                decision
            };

            return self.apply_governance_response(
                task,
                trace,
                &step,
                stage_key,
                &policy,
                GovernanceRuntimeKind::Canon,
                governance_attempt_id,
                packet_reuse,
                decision,
                response,
            );
        }

        let request = GovernanceRuntimeRequest {
            request_kind: GovernanceRequestKind::Start,
            governance_attempt_id: governance_attempt_id.clone(),
            stage_key: stage_key.clone(),
            goal: task.goal.clone(),
            workspace_ref: task.context.workspace_ref.clone(),
            autopilot: policy.autopilot,
            mode: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
            run_ref: None,
            packet_ref: existing_packet.as_ref().map(|packet| packet.packet_ref.clone()),
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
            }),
        );
        let response = LocalGovernanceRuntime
            .execute(&request)
            .map_err(|error| OrchestratorError::GovernanceRuntime(error.to_string()))?;
        let decision = if decision.is_some() {
            decision
        } else {
            let decision = build_autopilot_decision(
                &governance_attempt_id,
                &policy,
                governance.default_runtime,
                &metadata,
                &request.bounded_context,
                Some(response.status),
                Some(response.approval_state),
                response.packet.as_ref().map(|packet| packet.readiness),
            );
            if let Some(record) = &decision {
                self.record_governance_decision_event(trace, &step, task.plan.revision, record);
            }
            decision
        };

        self.apply_governance_response(
            task,
            trace,
            &step,
            stage_key,
            &policy,
            GovernanceRuntimeKind::Local,
            governance_attempt_id,
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
        task: &mut Task,
        trace: &mut ExecutionTrace,
        step: &Step,
        stage_key: String,
        policy: &crate::domain::governance::StageGovernancePolicy,
        runtime_kind: GovernanceRuntimeKind,
        governance_attempt_id: String,
        packet_reuse: Option<crate::domain::governance::PacketReuseBinding>,
        decision: Option<crate::domain::governance::AutopilotDecisionRecord>,
        response: crate::adapters::governance_runtime::GovernanceRuntimeResponse,
    ) -> Result<GovernanceStepDecision<String>, OrchestratorError> {
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
            previous_governance_attempt_id: None,
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
        .map_err(|error| OrchestratorError::GovernancePatch(error.to_string()))?;
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
                    }),
                );
                self.persist_trace(trace)?;
                Ok(GovernanceStepDecision::Continue)
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
                    }),
                );
                let reason = build_terminal_reason(
                    TerminalCondition::TaskNotCredible,
                    format!("governance awaiting approval for stage {stage_key}"),
                    Some(json!({
                        "stage_key": stage_key,
                        "runtime": runtime_kind,
                        "approval_state": response.approval_state,
                    })),
                );
                self.finalize_task(task, trace, reason).map(GovernanceStepDecision::Terminal)
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
                    }),
                );

                if policy.required || runtime_kind == GovernanceRuntimeKind::Canon {
                    let terminal_reason = build_terminal_reason(
                        TerminalCondition::TaskNotCredible,
                        format!("governance blocked stage {stage_key}: {reason}"),
                        Some(json!({
                            "stage_key": stage_key,
                            "runtime": runtime_kind,
                            "required": policy.required,
                        })),
                    );
                    let trace_location = self.finalize_task(task, trace, terminal_reason)?;
                    Ok(GovernanceStepDecision::Terminal(trace_location))
                } else {
                    self.persist_trace(trace)?;
                    Ok(GovernanceStepDecision::Continue)
                }
            }
            _ => Ok(GovernanceStepDecision::Continue),
        }
    }

    fn handle_governance_block(
        &self,
        task: &mut Task,
        trace: &mut ExecutionTrace,
        block: GovernanceBlockContext,
        decision: Option<crate::domain::governance::AutopilotDecisionRecord>,
    ) -> Result<GovernanceStepDecision<String>, OrchestratorError> {
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
            .map_err(|error| OrchestratorError::GovernancePatch(error.to_string()))?;
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
            let trace_location = self.finalize_task(task, trace, terminal_reason)?;
            Ok(GovernanceStepDecision::Terminal(trace_location))
        } else {
            self.persist_trace(trace)?;
            Ok(GovernanceStepDecision::Continue)
        }
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
    #[error("flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("task context state is invalid: {0}")]
    TaskContext(String),
    #[error("governance state patch is invalid: {0}")]
    GovernancePatch(String),
    #[error("governance runtime failed: {0}")]
    GovernanceRuntime(String),
}

struct GovernanceBlockContext {
    step_id: String,
    stage_key: String,
    required: bool,
    autopilot_enabled: bool,
    runtime: GovernanceRuntimeKind,
    reason: String,
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
    use crate::domain::flow::{attach_stage_metadata, built_in_flow};
    use crate::domain::governance::{
        CanonMode, CanonRuntimeConfig, GovernanceLifecycleState, StageGovernancePolicy,
        SystemContextBinding,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::{Plan, PlanStatus};
    use crate::domain::task::TaskStatus;

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

    const MISSING_CANON_COMMAND: &str = "/definitely/missing/canon";

    fn build_governance_profile(required: bool) -> GovernanceProfile {
        GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: Some(CanonRuntimeConfig {
                command: MISSING_CANON_COMMAND.to_string(),
                default_owner: Some("team-synod".to_string()),
                default_risk: Some("medium".to_string()),
                default_zone: Some("core".to_string()),
                default_system_context: Some(SystemContextBinding::Existing),
            }),
            stages: vec![StageGovernancePolicy {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                enabled: true,
                required,
                autopilot: false,
                runtime: Some(GovernanceRuntimeKind::Canon),
                canon_mode: Some(CanonMode::Discovery),
                system_context: Some(SystemContextBinding::Existing),
                risk: Some("medium".to_string()),
                zone: Some("core".to_string()),
                owner: Some("team-synod".to_string()),
            }],
        }
    }

    fn build_governed_plan() -> Plan {
        let flow = built_in_flow("bug-fix").expect("bug-fix flow must exist");
        let input = attach_stage_metadata(
            json!({
                "output": {"governed": true},
                "state_patch": {"goal_satisfied": true}
            }),
            flow,
            0,
        )
        .expect("stage metadata should attach");

        Plan::new(vec![Step::decision("investigate", input).unwrap()]).unwrap()
    }

    fn build_governed_orchestrator(required: bool) -> Orchestrator<TestPlanner, TestTraceStore> {
        build_orchestrator(TestPlanner::from_plan(build_governed_plan())).with_governance(
            vec!["README.md".to_string()],
            Some(build_governance_profile(required)),
        )
    }

    fn build_governed_task() -> Task {
        Task::new("task-governed", &build_request(RunLimits::default()), build_governed_plan())
            .unwrap()
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
    fn run_executes_optional_local_governance_for_flow_aware_steps() {
        let orchestrator = build_governed_orchestrator(false);

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();
        let governance = response
            .final_context
            .latest_governance_stage()
            .unwrap()
            .expect("governance state should be recorded");

        assert_eq!(response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(governance.stage_key, "bug-fix:investigate");
        assert_eq!(governance.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(governance.lifecycle_state, GovernanceLifecycleState::GovernedReady);
        assert_eq!(governance.blocked_reason, None);
    }

    #[test]
    fn run_blocks_required_canon_governance_for_flow_aware_steps() {
        let orchestrator = build_governed_orchestrator(true);

        let response = orchestrator.run(build_request(RunLimits::default())).unwrap();
        let governance = response
            .final_context
            .latest_governance_stage()
            .unwrap()
            .expect("governance state should be recorded");

        assert_eq!(response.terminal_status, TaskStatus::Failed);
        assert_eq!(response.terminal_reason.condition, TerminalCondition::TaskNotCredible);
        assert_eq!(governance.stage_key, "bug-fix:investigate");
        assert_eq!(governance.runtime, GovernanceRuntimeKind::Canon);
        assert_eq!(governance.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert_eq!(
            governance.blocked_reason,
            Some(format!(
                "governance required Canon for bug-fix:investigate, but command '{}' is unavailable",
                MISSING_CANON_COMMAND,
            )),
        );
    }

    #[test]
    fn record_governance_decision_event_records_full_decision_payload() {
        let orchestrator = build_governed_orchestrator(false);
        let step = build_governed_plan().steps[0].clone();
        let mut trace = ExecutionTrace::new("task-governed", "session-engine", "goal");
        let decision = crate::domain::governance::AutopilotDecisionRecord {
            decision_id: "decision-1".to_string(),
            stage_key: "bug-fix:investigate".to_string(),
            candidate_actions: vec![crate::domain::governance::AutopilotAction::SelectMode],
            candidate_modes: vec![CanonMode::Discovery],
            selected_action: Some(crate::domain::governance::AutopilotAction::SelectMode),
            selected_mode: Some(CanonMode::Discovery),
            selected_target_stage_key: Some("bug-fix:verify".to_string()),
            rationale: "selected discovery".to_string(),
            blocked_reason: Some("narrow the context".to_string()),
        };

        orchestrator.record_governance_decision_event(&mut trace, &step, 7, &decision);

        let event = trace.events.last().unwrap();
        assert_eq!(event.event_type, TraceEventType::GovernanceDecisionRecorded);
        assert_eq!(event.step_id.as_deref(), Some("investigate"));
        assert_eq!(event.payload["selected_target_stage_key"], json!("bug-fix:verify"));
        assert_eq!(event.payload["blocked_reason"], json!("narrow the context"));
    }

    #[test]
    fn apply_governance_response_continues_after_optional_local_packet_rejection() {
        let orchestrator = build_governed_orchestrator(false);
        let mut task = build_governed_task();
        let mut trace = ExecutionTrace::new("task-governed", "session-engine", "goal");
        let step = task.plan.steps[0].clone();
        let policy = build_governance_profile(false).stages[0].clone();
        let decision = crate::domain::governance::AutopilotDecisionRecord {
            decision_id: "decision-2".to_string(),
            stage_key: "bug-fix:investigate".to_string(),
            candidate_actions: vec![crate::domain::governance::AutopilotAction::BlockStage],
            candidate_modes: vec![CanonMode::Discovery],
            selected_action: None,
            selected_mode: Some(CanonMode::Discovery),
            selected_target_stage_key: None,
            rationale: "packet rejected".to_string(),
            blocked_reason: Some("packet rejected by governance".to_string()),
        };
        let response = crate::adapters::governance_runtime::GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::GovernedReady,
            approval_state: ApprovalState::NotNeeded,
            run_ref: None,
            packet: Some(crate::domain::governance::GovernedStagePacket {
                packet_ref: ".synod/governance/bug-fix-investigate/attempt-1".to_string(),
                runtime: GovernanceRuntimeKind::Local,
                canon_mode: None,
                expected_document_refs: vec!["packet/brief.md".to_string()],
                document_refs: vec!["packet/brief.md".to_string()],
                readiness: PacketReadiness::Rejected,
                missing_sections: vec!["substantive_body".to_string()],
                headline: "rejected packet".to_string(),
            }),
            message: "local governance evaluated bug-fix:investigate".to_string(),
        };

        let result = orchestrator
            .apply_governance_response(
                &mut task,
                &mut trace,
                &step,
                "bug-fix:investigate".to_string(),
                &policy,
                GovernanceRuntimeKind::Local,
                "attempt-1".to_string(),
                None,
                Some(decision),
                response,
            )
            .unwrap();

        assert!(matches!(
            result,
            crate::orchestrator::governance::GovernanceStepDecision::Continue
        ));
        let record = task.context.latest_governance_stage().unwrap().unwrap();
        assert_eq!(record.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert_eq!(record.blocked_reason.as_deref(), Some("packet rejected by governance"));
        assert!(
            trace
                .events
                .iter()
                .any(|event| event.event_type == TraceEventType::GovernancePacketRejected)
        );
        assert!(
            trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceBlocked)
        );
    }

    #[test]
    fn apply_governance_response_terminalizes_when_approval_is_pending() {
        let orchestrator = build_governed_orchestrator(true);
        let mut task = build_governed_task();
        let mut trace = ExecutionTrace::new("task-governed", "session-engine", "goal");
        let step = task.plan.steps[0].clone();
        let policy = build_governance_profile(true).stages[0].clone();
        let response = crate::adapters::governance_runtime::GovernanceRuntimeResponse {
            status: GovernanceLifecycleState::AwaitingApproval,
            approval_state: ApprovalState::Requested,
            run_ref: Some("canon-run-1".to_string()),
            packet: None,
            message: "waiting for approval".to_string(),
        };

        let result = orchestrator
            .apply_governance_response(
                &mut task,
                &mut trace,
                &step,
                "bug-fix:investigate".to_string(),
                &policy,
                GovernanceRuntimeKind::Canon,
                "attempt-2".to_string(),
                None,
                None,
                response,
            )
            .unwrap();

        match result {
            crate::orchestrator::governance::GovernanceStepDecision::Terminal(trace_location) => {
                assert_eq!(trace_location, "/tmp/task-governed.json");
            }
            other => panic!("expected terminal decision, got {other:?}"),
        }
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(
            task.terminal_reason.as_ref().unwrap().condition,
            TerminalCondition::TaskNotCredible
        );
        assert!(
            trace
                .events
                .iter()
                .any(|event| event.event_type == TraceEventType::GovernanceAwaitingApproval)
        );
    }

    #[test]
    fn handle_governance_block_continues_for_optional_stages() {
        let orchestrator = build_governed_orchestrator(false);
        let mut task = build_governed_task();
        let mut trace = ExecutionTrace::new("task-governed", "session-engine", "goal");
        let decision = crate::domain::governance::AutopilotDecisionRecord {
            decision_id: "decision-3".to_string(),
            stage_key: "bug-fix:investigate".to_string(),
            candidate_actions: vec![crate::domain::governance::AutopilotAction::SelectMode],
            candidate_modes: vec![CanonMode::Discovery],
            selected_action: Some(crate::domain::governance::AutopilotAction::SelectMode),
            selected_mode: Some(CanonMode::Discovery),
            selected_target_stage_key: None,
            rationale: "selected discovery".to_string(),
            blocked_reason: Some("canon unavailable".to_string()),
        };

        let result = orchestrator
            .handle_governance_block(
                &mut task,
                &mut trace,
                GovernanceBlockContext {
                    step_id: "investigate".to_string(),
                    stage_key: "bug-fix:investigate".to_string(),
                    required: false,
                    autopilot_enabled: true,
                    runtime: GovernanceRuntimeKind::Canon,
                    reason: "canon unavailable".to_string(),
                },
                Some(decision),
            )
            .unwrap();

        assert!(matches!(
            result,
            crate::orchestrator::governance::GovernanceStepDecision::Continue
        ));
        let record = task.context.latest_governance_stage().unwrap().unwrap();
        assert_eq!(record.lifecycle_state, GovernanceLifecycleState::Blocked);
        assert_eq!(record.decision_ref.as_deref(), Some("decision-3"));
        assert!(
            trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceBlocked)
        );
        assert_eq!(task.status, TaskStatus::Planned);
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
