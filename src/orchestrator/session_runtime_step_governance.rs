use serde_json::json;

use crate::adapters::governance_runtime::GovernanceRuntime;
use crate::domain::governance::{
    CanonAuthorityZone, CanonIntendedPersona, CanonRiskClass, GovernanceLifecycleState,
    PacketReadiness,
};
use crate::domain::limits::TerminalCondition;
use crate::domain::session::SessionStatus;
use crate::orchestrator::review_trace::record_reasoning_profile_events;
use crate::orchestrator::terminal::build_terminal_reason;

use super::{
    ActiveSessionRecord, CanonCliRuntime, ExecutionTrace, FixtureRuntime, FlowStepMetadata,
    GovernanceBlockContext, GovernanceRequestKind, GovernanceRuntimeKind, GovernanceRuntimeRequest,
    GovernanceStepDecision, GovernedStageRecord, LocalGovernanceRuntime, ReasoningGateContext,
    ReasoningTraceContext, SessionRuntime, SessionRuntimeError, Step, Task, TaskRunResponse,
    TraceEventType, append_governed_document_to_lifecycle, bounded_governance_context,
    build_autopilot_decision, clarification_prompt_from_response,
    compacted_canon_memory_from_response, current_timestamp_millis, default_stage_canon_mode,
    enrich_bounded_context_with_accumulated, governance_input_documents,
    governance_projection_snapshot, governance_stage_key, governance_state_patch,
    governed_document_ref_from_response, overlay_stage_policy_with_intent,
    requested_governance_intent, resolved_canon_mode, runtime_command_available,
    selected_stage_policy, store_latest_reasoning_profile,
};

impl SessionRuntime {
    pub(super) fn ensure_stage_governance(
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
            && policy.reasoning_profile.is_none()
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
    pub(super) fn execute_governance_for_step(
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
        let existing_packet = task
            .context
            .latest_governance_packet()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        if matches!(request_kind, GovernanceRequestKind::Refresh)
            && existing_record.as_ref().is_none_or(|record| {
                record.stage_key != stage_key
                    || record.lifecycle_state != GovernanceLifecycleState::AwaitingApproval
            })
        {
            return Ok(GovernanceStepDecision::Continue);
        }

        if matches!(request_kind, GovernanceRequestKind::Start)
            && existing_record.as_ref().is_some_and(|record| {
                record.stage_key == stage_key
                    && record.lifecycle_state == GovernanceLifecycleState::GovernedReady
            })
        {
            if let Some(existing_ready_record) = existing_record.as_ref() {
                return self.apply_reasoning_profile_gate(
                    session,
                    trace,
                    ReasoningTraceContext {
                        step_id: step.id.as_str(),
                        plan_revision: task.plan.revision,
                    },
                    policy,
                    ReasoningGateContext {
                        runtime_kind: existing_ready_record.runtime,
                        governance_attempt_id: existing_ready_record.governance_attempt_id.as_str(),
                        selected_mode: existing_packet
                            .as_ref()
                            .and_then(|packet| packet.canon_mode)
                            .or_else(|| {
                                session
                                    .governance_lifecycle
                                    .as_ref()
                                    .and_then(|lifecycle| lifecycle.selected_mode)
                            }),
                    },
                );
            }

            return Ok(GovernanceStepDecision::Continue);
        }

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
        let (mut bounded_context, packet_reuse) =
            bounded_governance_context(&task.context, metadata, &runtime.profile.read_targets)
                .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        if let Some(lifecycle) = session.governance_lifecycle.as_ref() {
            enrich_bounded_context_with_accumulated(
                &mut bounded_context,
                &lifecycle.accumulated_context,
            );
        }
        let compacted_canon_memory = task
            .context
            .latest_compacted_canon_memory()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        let input_documents =
            governance_input_documents(&task.input, compacted_canon_memory.as_ref());

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
        let existing_stage_mode = existing_record
            .as_ref()
            .filter(|record| record.stage_key == stage_key)
            .and(existing_packet.as_ref().and_then(|packet| packet.canon_mode));
        let mut mode = decision
            .as_ref()
            .and_then(|record| record.selected_mode)
            .or_else(|| resolved_canon_mode(policy, governance.default_runtime))
            .or(existing_stage_mode)
            .or_else(|| default_stage_canon_mode(policy, governance.default_runtime));
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
            self.record_governance_decision_event(
                trace,
                step,
                task.plan.revision,
                selected_runtime,
                decision,
            );
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
                            "Boundline could not determine a Canon mode for governance stage {stage_key}"
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
                risk: policy.risk.clone().or_else(|| canon.default_risk.clone()).map(|risk| {
                    CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
                }),
                zone: policy.zone.clone().or_else(|| canon.default_zone.clone()).map(|zone| {
                    CanonAuthorityZone::canonicalize_label(&zone)
                        .map(str::to_string)
                        .unwrap_or(zone)
                }),
                owner: policy.owner.clone().or_else(|| canon.default_owner.clone()).map(|owner| {
                    CanonIntendedPersona::canonicalize_label(&owner)
                        .map(str::to_string)
                        .unwrap_or(owner)
                }),
                run_ref: existing_record.as_ref().and_then(|record| record.canon_run_ref.clone()),
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
                    "runtime": GovernanceRuntimeKind::Canon,
                    "canon_mode": request.mode,
                    "system_context": request.system_context,
                    "risk": request.risk,
                    "zone": request.zone,
                    "owner": request.owner,
                    "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
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
                    self.record_governance_decision_event(
                        trace,
                        step,
                        task.plan.revision,
                        GovernanceRuntimeKind::Canon,
                        record,
                    );
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
                "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
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
                self.record_governance_decision_event(
                    trace,
                    step,
                    task.plan.revision,
                    GovernanceRuntimeKind::Local,
                    record,
                );
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
        runtime_kind: GovernanceRuntimeKind,
        decision: &crate::domain::governance::AutopilotDecisionRecord,
    ) {
        trace.record_event(
            TraceEventType::GovernanceDecisionRecorded,
            Some(step.id.clone()),
            plan_revision,
            json!({
                "stage_key": decision.stage_key,
                "runtime": runtime_kind,
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
        if let Some(prompt) = clarification_prompt_from_response(&response) {
            trace.record_event(
                TraceEventType::GovernanceBlocked,
                Some(step.id.clone()),
                task.plan.revision,
                json!({
                    "stage_key": stage_key,
                    "runtime": runtime_kind,
                    "reason": prompt,
                    "packet_ref": response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                    "missing_sections": response
                        .packet
                        .as_ref()
                        .map(|packet| packet.missing_sections.clone())
                        .unwrap_or_default(),
                    "packet_source_stage": packet_reuse.as_ref().map(|binding| binding.upstream_stage_key.clone()),
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
                }),
            );
            let trace_location = self.persist_trace(&session.session_id, trace)?;
            session.latest_status = SessionStatus::Running;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = Some(trace_location);
            session.updated_at = current_timestamp_millis();
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: response
                    .packet
                    .as_ref()
                    .map(|packet| packet.headline.clone())
                    .unwrap_or_else(|| "Canon clarification required".to_string()),
                prompt,
            });
        }

        let response = crate::orchestrator::governance::fail_closed_required_authority_response(
            &stage_key,
            policy,
            runtime_kind,
            &response,
        )
        .unwrap_or(response);

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
                                let detail = if !packet.missing_sections.is_empty() {
                                    format!(
                                        ": missing sections {}",
                                        packet.missing_sections.join(", ")
                                    )
                                } else if !response.message.trim().is_empty() {
                                    format!(": {}", response.message)
                                } else {
                                    String::new()
                                };
                                format!(
                                    "governance packet was {:?} for stage {stage_key}{}",
                                    packet.readiness, detail
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
            stage_council: None,
            blocked_reason: blocked_reason.clone(),
        };
        let compacted_canon_memory =
            compacted_canon_memory_from_response(&stage_key, runtime_kind, &response);
        let projection = governance_projection_snapshot(
            &task.context,
            &stage_key,
            response.packet.as_ref(),
            response.approval_state,
        )
        .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        let patch = governance_state_patch(
            &record,
            response.packet.as_ref(),
            packet_reuse.as_ref(),
            decision.as_ref(),
            compacted_canon_memory.as_ref(),
            &projection,
        )
        .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        task.context.apply_state_patch(&patch);
        let selected_mode = response
            .packet
            .as_ref()
            .and_then(|packet| packet.canon_mode)
            .or_else(|| decision.as_ref().and_then(|decision| decision.selected_mode));

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
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
                    "latest_governance_runtime_state": projection.runtime_state,
                    "latest_governance_rollout_profile": projection.rollout_profile,
                    "latest_governance_reason": projection.reason.clone(),
                    "latest_governance_contract_lines": projection.contract_lines.clone(),
                    "latest_governance_approval_provenance": projection.approval_provenance.clone(),
                    "authority_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.authority_provenance_lines.clone()).unwrap_or_default(),
                    "adaptive_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.adaptive_provenance_lines.clone()).unwrap_or_default(),
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
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
                        "latest_governance_runtime_state": projection.runtime_state,
                        "latest_governance_rollout_profile": projection.rollout_profile,
                        "latest_governance_reason": projection.reason.clone(),
                        "latest_governance_contract_lines": projection.contract_lines.clone(),
                        "latest_governance_approval_provenance": projection.approval_provenance.clone(),
                        "authority_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.authority_provenance_lines.clone()).unwrap_or_default(),
                        "adaptive_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.adaptive_provenance_lines.clone()).unwrap_or_default(),
                    }),
                );
                let trace_location = self.persist_trace(&session.session_id, trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                if let Some(canon_mode) = selected_mode {
                    let doc_ref =
                        governed_document_ref_from_response(&stage_key, canon_mode, &response);
                    append_governed_document_to_lifecycle(session, doc_ref);
                    self.promote_governed_evidence_outputs(&stage_key, canon_mode, &response)?;
                }
                match self.apply_reasoning_profile_gate(
                    session,
                    trace,
                    ReasoningTraceContext {
                        step_id: step.id.as_str(),
                        plan_revision: task.plan.revision,
                    },
                    policy,
                    ReasoningGateContext {
                        runtime_kind,
                        governance_attempt_id: record.governance_attempt_id.as_str(),
                        selected_mode,
                    },
                )? {
                    GovernanceStepDecision::Continue => {
                        if matches!(request_kind, GovernanceRequestKind::Refresh) {
                            Ok(GovernanceStepDecision::Halt)
                        } else {
                            Ok(GovernanceStepDecision::Continue)
                        }
                    }
                    GovernanceStepDecision::Halt => Ok(GovernanceStepDecision::Halt),
                    GovernanceStepDecision::Terminal(response) => {
                        Ok(GovernanceStepDecision::Terminal(response))
                    }
                }
            }
            GovernanceLifecycleState::AwaitingApproval => {
                let interrupted_reasoning_profile = self.interrupted_reasoning_profile_for_stage(
                    stage_key.as_str(),
                    policy,
                    runtime_kind,
                    record.governance_attempt_id.as_str(),
                    response.message.as_str(),
                )?;
                if let Some(reasoning_profile) = interrupted_reasoning_profile.as_ref() {
                    store_latest_reasoning_profile(
                        session,
                        runtime_kind,
                        selected_mode,
                        reasoning_profile.clone(),
                    );
                }
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
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
                        "latest_governance_runtime_state": projection.runtime_state,
                        "latest_governance_rollout_profile": projection.rollout_profile,
                        "latest_governance_reason": projection.reason.clone(),
                        "latest_governance_contract_lines": projection.contract_lines.clone(),
                        "latest_governance_approval_provenance": projection.approval_provenance.clone(),
                        "reasoning_profile_record": interrupted_reasoning_profile,
                        "authority_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.authority_provenance_lines.clone()).unwrap_or_default(),
                        "adaptive_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.adaptive_provenance_lines.clone()).unwrap_or_default(),
                    }),
                );
                if let Some(reasoning_profile) = interrupted_reasoning_profile.as_ref() {
                    record_reasoning_profile_events(
                        trace,
                        step.id.as_str(),
                        task.plan.revision,
                        reasoning_profile,
                    );
                }
                let trace_location = self.persist_trace(&session.session_id, trace)?;
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
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
                        "latest_governance_runtime_state": projection.runtime_state,
                        "latest_governance_rollout_profile": projection.rollout_profile,
                        "latest_governance_reason": projection.reason.clone(),
                        "latest_governance_contract_lines": projection.contract_lines.clone(),
                        "latest_governance_approval_provenance": projection.approval_provenance.clone(),
                        "authority_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.authority_provenance_lines.clone()).unwrap_or_default(),
                        "adaptive_provenance_lines": compacted_canon_memory.as_ref().map(|memory| memory.adaptive_provenance_lines.clone()).unwrap_or_default(),
                    }),
                );
                let trace_location = self.persist_trace(&session.session_id, trace)?;
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
}
