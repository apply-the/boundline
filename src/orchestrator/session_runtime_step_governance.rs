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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use serde_json::{Map, json};
    use uuid::Uuid;

    use crate::adapters::governance_runtime::GovernanceRuntimeResponse;
    use crate::domain::execution::{ExecutionCommand, WorkspaceExecutionProfile};
    use crate::domain::flow::{FlowStepMetadata, attach_stage_metadata, built_in_flow};
    use crate::domain::governance::{
        ApprovalState, CanonMode, CanonRuntimeConfig, GovernanceLifecycleState, GovernanceProfile,
        GovernanceRuntimeKind, GovernedStagePacket, GovernedStageRecord, PacketReadiness,
        StageGovernancePolicy,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::plan::Plan;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::step::Step;
    use crate::domain::task::{Task, TaskStatus};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{ExecutionTrace, TraceEventType};
    use crate::fixture::FixtureRuntime;
    use crate::orchestrator::planner::StaticPlanner;
    use crate::registry::agent_registry::AgentRegistry;
    use crate::registry::tool_registry::ToolRegistry;

    use super::{
        GovernanceRequestKind, GovernanceStepDecision, SessionRuntime, SessionRuntimeError,
        governance_stage_key,
    };

    const BUG_FIX_FLOW_NAME: &str = "bug-fix";
    const BUG_FIX_STAGE_ID: &str = "investigate";
    const CLARIFICATION_HEADLINE: &str = "Clarification packet";
    const CLARIFICATION_MESSAGE: &str = "Need a clearer scope summary";
    const CLARIFICATION_STAGE_KEY: &str = "delivery:implement";
    const GOVERNANCE_ATTEMPT_ID: &str = "delivery-implement-attempt-1";
    const LOCAL_BLOCKED_MESSAGE: &str = "packet requires more detail";
    const LOCAL_PACKET_REF: &str = "packet-123";
    const MISSING_SECTION_SCOPE: &str = "scope";
    const REJECTED_PACKET_HEADLINE: &str = "Implementation packet rejected";
    const SESSION_GOAL: &str = "Deliver the governed task";
    const SESSION_ID: &str = "session-1";
    const STEP_ID: &str = "step-1";
    const TASK_ID: &str = "task-1";
    const TRACE_REASON_SUBSTRING: &str = "missing sections scope";
    const UPDATED_AT: u64 = 73;
    const WORKSPACE_GOAL: &str = "Govern one step";

    #[test]
    fn step_governance_apply_response_surfaces_clarification_and_persists_trace()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-step-governance-clarification")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());
        let mut task = sample_task(workspace.as_path())?;
        let step = task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);

        let error = runtime
            .apply_governance_response(
                &mut session,
                &mut task,
                &mut trace,
                &step,
                CLARIFICATION_STAGE_KEY.to_string(),
                &sample_policy(),
                GovernanceRequestKind::Start,
                GovernanceRuntimeKind::Canon,
                GOVERNANCE_ATTEMPT_ID.to_string(),
                None,
                None,
                None,
                GovernanceRuntimeResponse {
                    status: GovernanceLifecycleState::Incomplete,
                    approval_state: ApprovalState::NotNeeded,
                    run_ref: None,
                    packet: Some(GovernedStagePacket {
                        packet_ref: LOCAL_PACKET_REF.to_string(),
                        runtime: GovernanceRuntimeKind::Canon,
                        canon_mode: Some(CanonMode::Implementation),
                        expected_document_refs: vec!["implementation.md".to_string()],
                        document_refs: vec!["implementation.md".to_string()],
                        readiness: PacketReadiness::Incomplete,
                        missing_sections: vec![MISSING_SECTION_SCOPE.to_string()],
                        headline: CLARIFICATION_HEADLINE.to_string(),
                        reason_code: None,
                        authority_governance: None,
                        adaptive_governance: None,
                        semantic_descriptor: None,
                    }),
                    reason_code: None,
                    message: CLARIFICATION_MESSAGE.to_string(),
                },
            )
            .unwrap_err();

        match error {
            SessionRuntimeError::ClarificationRequired { headline, prompt } => {
                assert_eq!(headline, CLARIFICATION_HEADLINE);
                assert!(prompt.contains(MISSING_SECTION_SCOPE));
            }
            other => return Err(format!("unexpected error: {other}").into()),
        }

        assert_eq!(session.latest_status, SessionStatus::Running);
        assert!(session.latest_trace_ref.is_some());
        assert!(trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceBlocked
                && event.step_id.as_deref() == Some(STEP_ID)
        }));
        assert!(task.context.latest_governance_stage()?.is_none());

        Ok(())
    }

    #[test]
    fn step_governance_apply_response_treats_rejected_packet_as_local_block_and_continues()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-step-governance-rejected")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());
        let mut task = sample_task(workspace.as_path())?;
        let step = task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);

        let decision = runtime.apply_governance_response(
            &mut session,
            &mut task,
            &mut trace,
            &step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &sample_policy(),
            GovernanceRequestKind::Start,
            GovernanceRuntimeKind::Local,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::GovernedReady,
                approval_state: ApprovalState::NotNeeded,
                run_ref: None,
                packet: Some(GovernedStagePacket {
                    packet_ref: LOCAL_PACKET_REF.to_string(),
                    runtime: GovernanceRuntimeKind::Local,
                    canon_mode: Some(CanonMode::Implementation),
                    expected_document_refs: vec!["implementation.md".to_string()],
                    document_refs: vec!["implementation.md".to_string()],
                    readiness: PacketReadiness::Rejected,
                    missing_sections: vec![MISSING_SECTION_SCOPE.to_string()],
                    headline: REJECTED_PACKET_HEADLINE.to_string(),
                    reason_code: None,
                    authority_governance: None,
                    adaptive_governance: None,
                    semantic_descriptor: None,
                }),
                reason_code: None,
                message: LOCAL_BLOCKED_MESSAGE.to_string(),
            },
        )?;

        assert_eq!(decision, GovernanceStepDecision::Continue);
        assert_eq!(session.latest_status, SessionStatus::Running);
        assert!(session.latest_trace_ref.is_some());

        let latest_stage =
            task.context.latest_governance_stage()?.ok_or("missing latest governance stage")?;
        assert_eq!(latest_stage.stage_key, CLARIFICATION_STAGE_KEY);
        assert_eq!(latest_stage.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(latest_stage.lifecycle_state, GovernanceLifecycleState::Blocked);
        let blocked_reason = latest_stage.blocked_reason.ok_or("missing blocked reason")?;
        assert!(blocked_reason.contains(TRACE_REASON_SUBSTRING));

        let latest_packet =
            task.context.latest_governance_packet()?.ok_or("missing latest governance packet")?;
        assert_eq!(latest_packet.packet_ref, LOCAL_PACKET_REF);
        assert_eq!(latest_packet.readiness, PacketReadiness::Rejected);

        let latest_memory = task
            .context
            .latest_compacted_canon_memory()?
            .ok_or("missing latest compacted canon memory")?;
        assert_eq!(latest_memory.packet_ref.as_deref(), Some(LOCAL_PACKET_REF));

        assert!(trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernancePacketRejected
                && event.step_id.as_deref() == Some(STEP_ID)
        }));
        assert!(trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceBlocked
                && event.step_id.as_deref() == Some(STEP_ID)
        }));

        Ok(())
    }

    #[test]
    fn step_governance_ensure_stage_governance_covers_continue_guards() -> Result<(), Box<dyn Error>>
    {
        let workspace = temp_workspace("boundline-step-governance-continue-guards")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());

        let mut no_step_task = sample_task(workspace.as_path())?;
        no_step_task.plan.current_step_index = no_step_task.plan.steps.len();
        let mut no_step_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let no_governance_runtime = sample_fixture_runtime(None)?;
        assert!(matches!(
            runtime.ensure_stage_governance(
                &mut session,
                &mut no_step_task,
                &mut no_step_trace,
                &no_governance_runtime,
            )?,
            GovernanceStepDecision::Continue
        ));

        let stage_step = governance_step(BUG_FIX_FLOW_NAME, 0)?;

        let mut no_policy_task = task_with_step(workspace.as_path(), stage_step.clone())?;
        let mut no_policy_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let no_policy_runtime = sample_fixture_runtime(Some(sample_governance_profile(
            GovernanceRuntimeKind::Local,
            Vec::new(),
            None,
        )))?;
        assert!(matches!(
            runtime.ensure_stage_governance(
                &mut session,
                &mut no_policy_task,
                &mut no_policy_trace,
                &no_policy_runtime,
            )?,
            GovernanceStepDecision::Continue
        ));

        let mut disabled_policy = sample_bug_fix_policy();
        disabled_policy.enabled = false;
        let mut disabled_task = task_with_step(workspace.as_path(), stage_step.clone())?;
        let mut disabled_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let disabled_runtime = sample_fixture_runtime(Some(sample_governance_profile(
            GovernanceRuntimeKind::Local,
            vec![disabled_policy],
            None,
        )))?;
        assert!(matches!(
            runtime.ensure_stage_governance(
                &mut session,
                &mut disabled_task,
                &mut disabled_trace,
                &disabled_runtime,
            )?,
            GovernanceStepDecision::Continue
        ));

        let mut existing_stage_task = task_with_step(workspace.as_path(), stage_step)?;
        existing_stage_task.context.set_latest_governance_stage(&sample_stage_record(
            &governance_stage_key(BUG_FIX_FLOW_NAME, BUG_FIX_STAGE_ID),
            GovernanceLifecycleState::GovernedReady,
            GovernanceRuntimeKind::Local,
            GOVERNANCE_ATTEMPT_ID,
        ))?;
        let mut existing_stage_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let existing_stage_runtime = sample_fixture_runtime(Some(sample_governance_profile(
            GovernanceRuntimeKind::Local,
            vec![sample_bug_fix_policy()],
            None,
        )))?;
        assert!(matches!(
            runtime.ensure_stage_governance(
                &mut session,
                &mut existing_stage_task,
                &mut existing_stage_trace,
                &existing_stage_runtime,
            )?,
            GovernanceStepDecision::Continue
        ));

        Ok(())
    }

    #[test]
    fn step_governance_execute_governance_refresh_without_awaiting_record_continues()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-step-governance-refresh-continue")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let mut session = sample_session(workspace.as_path());
        let step = governance_step(BUG_FIX_FLOW_NAME, 0)?;
        let metadata = FlowStepMetadata::from_step(&step)?.ok_or("missing flow metadata")?;
        let mut task = task_with_step(workspace.as_path(), step.clone())?;
        let mut trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let fixture_runtime = sample_fixture_runtime(Some(sample_governance_profile(
            GovernanceRuntimeKind::Local,
            vec![sample_bug_fix_policy()],
            None,
        )))?;

        assert!(matches!(
            runtime.execute_governance_for_step(
                &mut session,
                &mut task,
                &mut trace,
                &fixture_runtime,
                &step,
                &metadata,
                fixture_runtime.profile.governance.as_ref().ok_or("missing governance profile")?,
                &sample_bug_fix_policy(),
                GovernanceRequestKind::Refresh,
            )?,
            GovernanceStepDecision::Continue
        ));

        Ok(())
    }

    #[test]
    fn step_governance_execute_governance_for_step_covers_local_fallback_and_refresh_paths()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-step-governance-local-paths")?;
        fs::create_dir_all(workspace.as_path().join("src"))?;
        fs::write(
            workspace.as_path().join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());
        let step = governance_step(BUG_FIX_FLOW_NAME, 0)?;
        let metadata = FlowStepMetadata::from_step(&step)?.ok_or("missing flow metadata")?;

        let mut fallback_policy = sample_bug_fix_policy();
        fallback_policy.autopilot = true;
        fallback_policy.runtime = Some(GovernanceRuntimeKind::Canon);
        let fallback_governance = sample_governance_profile(
            GovernanceRuntimeKind::Canon,
            vec![fallback_policy.clone()],
            Some("canon-missing-for-test"),
        );
        let mut fallback_runtime = sample_fixture_runtime(Some(fallback_governance))?;
        fallback_runtime.profile.read_targets = vec!["src/lib.rs".to_string()];

        let mut fallback_session = sample_session(workspace.as_path());
        let mut fallback_task = task_with_step(workspace.as_path(), step.clone())?;
        let mut fallback_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let fallback_decision = runtime.execute_governance_for_step(
            &mut fallback_session,
            &mut fallback_task,
            &mut fallback_trace,
            &fallback_runtime,
            &step,
            &metadata,
            fallback_runtime
                .profile
                .governance
                .as_ref()
                .ok_or("missing fallback governance profile")?,
            &fallback_policy,
            GovernanceRequestKind::Start,
        )?;
        assert!(matches!(fallback_decision, GovernanceStepDecision::Continue));
        assert!(fallback_trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceDecisionRecorded
                && event.step_id.as_deref() == Some(STEP_ID)
        }));
        assert!(fallback_trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceStarted
                && event.step_id.as_deref() == Some(STEP_ID)
        }));
        assert!(fallback_trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceCompleted
                && event.step_id.as_deref() == Some(STEP_ID)
        }));
        let fallback_stage = fallback_task
            .context
            .latest_governance_stage()?
            .ok_or("missing fallback governance stage")?;
        assert_eq!(fallback_stage.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(fallback_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);

        let mut local_runtime = sample_fixture_runtime(Some(sample_governance_profile(
            GovernanceRuntimeKind::Local,
            vec![sample_bug_fix_policy()],
            None,
        )))?;
        local_runtime.profile.read_targets = vec!["src/lib.rs".to_string()];
        let mut local_session = sample_session(workspace.as_path());
        let mut local_task = task_with_step(workspace.as_path(), step.clone())?;
        let mut local_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let local_decision = runtime.execute_governance_for_step(
            &mut local_session,
            &mut local_task,
            &mut local_trace,
            &local_runtime,
            &step,
            &metadata,
            local_runtime.profile.governance.as_ref().ok_or("missing local governance profile")?,
            &sample_bug_fix_policy(),
            GovernanceRequestKind::Start,
        )?;
        assert!(matches!(local_decision, GovernanceStepDecision::Continue));
        let local_stage = local_task
            .context
            .latest_governance_stage()?
            .ok_or("missing local governance stage")?;
        assert_eq!(local_stage.runtime, GovernanceRuntimeKind::Local);
        assert_eq!(local_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);

        let mut refresh_session = sample_session(workspace.as_path());
        let mut refresh_task = task_with_step(workspace.as_path(), step)?;
        refresh_task.context.set_latest_governance_stage(&sample_stage_record(
            &governance_stage_key(BUG_FIX_FLOW_NAME, BUG_FIX_STAGE_ID),
            GovernanceLifecycleState::AwaitingApproval,
            GovernanceRuntimeKind::Canon,
            GOVERNANCE_ATTEMPT_ID,
        ))?;
        let mut refresh_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let refresh_decision = runtime.execute_governance_for_step(
            &mut refresh_session,
            &mut refresh_task,
            &mut refresh_trace,
            &fallback_runtime,
            &governance_step(BUG_FIX_FLOW_NAME, 0)?,
            &metadata,
            fallback_runtime
                .profile
                .governance
                .as_ref()
                .ok_or("missing refresh governance profile")?,
            &fallback_policy,
            GovernanceRequestKind::Refresh,
        )?;
        assert!(matches!(refresh_decision, GovernanceStepDecision::Halt));

        Ok(())
    }

    #[test]
    fn step_governance_apply_response_covers_additional_status_outcomes()
    -> Result<(), Box<dyn Error>> {
        let workspace = temp_workspace("boundline-step-governance-status-outcomes")?;
        let runtime = SessionRuntime::for_workspace(workspace.as_path());

        let mut awaiting_session = sample_session(workspace.as_path());
        let mut awaiting_task = sample_task(workspace.as_path())?;
        let awaiting_step =
            awaiting_task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut awaiting_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let awaiting_decision = runtime.apply_governance_response(
            &mut awaiting_session,
            &mut awaiting_task,
            &mut awaiting_trace,
            &awaiting_step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &sample_policy(),
            GovernanceRequestKind::Start,
            GovernanceRuntimeKind::Canon,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::AwaitingApproval,
                approval_state: ApprovalState::Requested,
                run_ref: Some("canon-run-awaiting".to_string()),
                packet: None,
                reason_code: None,
                message: "waiting for operator approval".to_string(),
            },
        )?;
        assert!(matches!(awaiting_decision, GovernanceStepDecision::Halt));
        assert!(awaiting_trace.events.iter().any(|event| {
            event.event_type == TraceEventType::GovernanceAwaitingApproval
                && event.step_id.as_deref() == Some(STEP_ID)
        }));

        let mut required_policy = sample_policy();
        required_policy.required = true;
        let mut terminal_session = sample_session(workspace.as_path());
        let mut terminal_task = sample_task(workspace.as_path())?;
        let terminal_step =
            terminal_task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut terminal_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let terminal_decision = runtime.apply_governance_response(
            &mut terminal_session,
            &mut terminal_task,
            &mut terminal_trace,
            &terminal_step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &required_policy,
            GovernanceRequestKind::Start,
            GovernanceRuntimeKind::Canon,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::Blocked,
                approval_state: ApprovalState::Rejected,
                run_ref: None,
                packet: None,
                reason_code: None,
                message: "manual governance gate required".to_string(),
            },
        )?;
        match terminal_decision {
            GovernanceStepDecision::Terminal(response) => {
                assert!(response.terminal_reason.message.contains(CLARIFICATION_STAGE_KEY));
            }
            other => return Err(format!("expected terminal decision, found {other:?}").into()),
        }

        let mut halted_session = sample_session(workspace.as_path());
        let mut halted_task = sample_task(workspace.as_path())?;
        let halted_step = halted_task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut halted_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let halted_decision = runtime.apply_governance_response(
            &mut halted_session,
            &mut halted_task,
            &mut halted_trace,
            &halted_step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &sample_policy(),
            GovernanceRequestKind::Refresh,
            GovernanceRuntimeKind::Canon,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::Failed,
                approval_state: ApprovalState::Rejected,
                run_ref: None,
                packet: None,
                reason_code: None,
                message: "external governance failed".to_string(),
            },
        )?;
        assert!(matches!(halted_decision, GovernanceStepDecision::Halt));

        let mut completed_session = sample_session(workspace.as_path());
        let mut completed_task = sample_task(workspace.as_path())?;
        let completed_step =
            completed_task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut completed_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let completed_decision = runtime.apply_governance_response(
            &mut completed_session,
            &mut completed_task,
            &mut completed_trace,
            &completed_step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &sample_policy(),
            GovernanceRequestKind::Start,
            GovernanceRuntimeKind::Canon,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::Completed,
                approval_state: ApprovalState::NotNeeded,
                run_ref: None,
                packet: None,
                reason_code: None,
                message: "governance completed elsewhere".to_string(),
            },
        )?;
        assert!(matches!(completed_decision, GovernanceStepDecision::Continue));

        let mut rejected_session = sample_session(workspace.as_path());
        let mut rejected_task = sample_task(workspace.as_path())?;
        let rejected_step =
            rejected_task.plan.current_step().cloned().ok_or("missing current step")?;
        let mut rejected_trace = ExecutionTrace::new(TASK_ID, SESSION_ID, SESSION_GOAL);
        let rejected_decision = runtime.apply_governance_response(
            &mut rejected_session,
            &mut rejected_task,
            &mut rejected_trace,
            &rejected_step,
            CLARIFICATION_STAGE_KEY.to_string(),
            &sample_policy(),
            GovernanceRequestKind::Start,
            GovernanceRuntimeKind::Local,
            GOVERNANCE_ATTEMPT_ID.to_string(),
            None,
            None,
            None,
            GovernanceRuntimeResponse {
                status: GovernanceLifecycleState::GovernedReady,
                approval_state: ApprovalState::NotNeeded,
                run_ref: None,
                packet: Some(GovernedStagePacket {
                    packet_ref: LOCAL_PACKET_REF.to_string(),
                    runtime: GovernanceRuntimeKind::Local,
                    canon_mode: Some(CanonMode::Implementation),
                    expected_document_refs: vec!["implementation.md".to_string()],
                    document_refs: vec!["implementation.md".to_string()],
                    readiness: PacketReadiness::Rejected,
                    missing_sections: Vec::new(),
                    headline: REJECTED_PACKET_HEADLINE.to_string(),
                    reason_code: None,
                    authority_governance: None,
                    adaptive_governance: None,
                    semantic_descriptor: None,
                }),
                reason_code: None,
                message: LOCAL_BLOCKED_MESSAGE.to_string(),
            },
        )?;
        assert!(matches!(rejected_decision, GovernanceStepDecision::Continue));
        let blocked_reason = rejected_task
            .context
            .latest_governance_stage()?
            .ok_or("missing rejected governance stage")?
            .blocked_reason
            .ok_or("missing rejected blocked reason")?;
        assert!(blocked_reason.contains(LOCAL_BLOCKED_MESSAGE));

        Ok(())
    }

    fn sample_policy() -> StageGovernancePolicy {
        StageGovernancePolicy {
            flow_name: "delivery".to_string(),
            stage_id: "implement".to_string(),
            enabled: true,
            required: false,
            autopilot: false,
            require_adaptive_companion: false,
            runtime: None,
            canon_mode: Some(CanonMode::Implementation),
            reasoning_profile: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
        }
    }

    fn sample_bug_fix_policy() -> StageGovernancePolicy {
        StageGovernancePolicy {
            flow_name: BUG_FIX_FLOW_NAME.to_string(),
            stage_id: BUG_FIX_STAGE_ID.to_string(),
            enabled: true,
            required: false,
            autopilot: false,
            require_adaptive_companion: false,
            runtime: None,
            canon_mode: None,
            reasoning_profile: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
        }
    }

    fn sample_governance_profile(
        default_runtime: GovernanceRuntimeKind,
        stages: Vec<StageGovernancePolicy>,
        canon_command: Option<&str>,
    ) -> GovernanceProfile {
        GovernanceProfile {
            default_runtime,
            canon: canon_command.map(|command| CanonRuntimeConfig {
                command: command.to_string(),
                default_owner: None,
                default_risk: None,
                default_zone: None,
                default_system_context: None,
            }),
            stages,
        }
    }

    fn sample_fixture_runtime(
        governance: Option<GovernanceProfile>,
    ) -> Result<FixtureRuntime, Box<dyn Error>> {
        let planner_step = Step::decision("placeholder", json!({}))?;
        Ok(FixtureRuntime {
            profile: WorkspaceExecutionProfile {
                name: "step-governance-runtime".to_string(),
                read_targets: Vec::new(),
                validation_command: ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string(), "--quiet".to_string()],
                },
                attempts: Vec::new(),
                adaptive: None,
                limits: RunLimits::default(),
                governance,
                review: None,
                legacy_source: None,
            },
            planner: Arc::new(StaticPlanner::new(Plan::new(vec![planner_step])?)),
            agents: AgentRegistry::new(),
            tools: ToolRegistry::new(),
        })
    }

    fn governance_step(flow_name: &str, stage_index: usize) -> Result<Step, Box<dyn Error>> {
        let flow = built_in_flow(flow_name).ok_or_else(|| format!("missing flow {flow_name}"))?;
        let input = attach_stage_metadata(json!({"goal": SESSION_GOAL}), flow, stage_index)?;
        Ok(Step::agent(STEP_ID, "planner", input)?)
    }

    fn task_with_step(workspace: &Path, step: Step) -> Result<Task, Box<dyn Error>> {
        Ok(Task {
            id: TASK_ID.to_string(),
            goal: SESSION_GOAL.to_string(),
            input: json!({"goal": SESSION_GOAL}),
            context: TaskContext::new(
                SESSION_ID,
                workspace.to_string_lossy().into_owned(),
                RunLimits::default(),
                Map::new(),
            ),
            plan: Plan::new(vec![step])?,
            status: TaskStatus::Planned,
            limits: RunLimits::default(),
            terminal_reason: None,
            retry_count: 0,
            replan_count: 0,
            total_step_attempts: 0,
        })
    }

    fn sample_stage_record(
        stage_key: &str,
        lifecycle_state: GovernanceLifecycleState,
        runtime: GovernanceRuntimeKind,
        governance_attempt_id: &str,
    ) -> GovernedStageRecord {
        GovernedStageRecord {
            stage_key: stage_key.to_string(),
            runtime,
            lifecycle_state,
            required: false,
            autopilot_enabled: false,
            approval_state: if lifecycle_state == GovernanceLifecycleState::AwaitingApproval {
                ApprovalState::Requested
            } else {
                ApprovalState::NotNeeded
            },
            canon_run_ref: None,
            governance_attempt_id: governance_attempt_id.to_string(),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        }
    }

    fn sample_session(workspace: &Path) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: SESSION_ID.to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some(WORKSPACE_GOAL.to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: UPDATED_AT,
            updated_at: UPDATED_AT,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    fn sample_task(workspace: &Path) -> Result<Task, Box<dyn Error>> {
        let step = Step::agent(STEP_ID, "planner", json!({"goal": SESSION_GOAL}))?;
        let plan = Plan::new(vec![step])?;
        Ok(Task {
            id: TASK_ID.to_string(),
            goal: SESSION_GOAL.to_string(),
            input: json!({"goal": SESSION_GOAL}),
            context: TaskContext::new(
                SESSION_ID,
                workspace.to_string_lossy().into_owned(),
                RunLimits::default(),
                Map::new(),
            ),
            plan,
            status: TaskStatus::Planned,
            limits: RunLimits::default(),
            terminal_reason: None,
            retry_count: 0,
            replan_count: 0,
            total_step_attempts: 0,
        })
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
    }

    struct TestWorkspace {
        path: PathBuf,
    }

    impl TestWorkspace {
        fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
            let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
            fs::create_dir_all(&path)?;
            Ok(Self { path })
        }

        fn as_path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
