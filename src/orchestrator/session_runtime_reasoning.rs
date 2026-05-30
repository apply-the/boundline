use std::collections::BTreeSet;
use std::path::Path;

use serde_json::json;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::domain::configuration::{EffectiveRouting, RoutingOverrides, resolve_effective_routing};
use crate::domain::distribution::SUPPORTED_CANON_VERSION;
use crate::domain::governance::{
    CanonModeSelectionPreference, GovernanceLifecycleState, GovernanceRuntimeKind,
    GovernedSessionLifecycle,
};
use crate::domain::limits::TerminalCondition;
use crate::domain::reasoning::{
    CanonAdmissionPriority, CanonChallengePostureInput, IndependenceAssessment,
    IndependenceAssessmentResult, IndependenceFloor, ParticipantAssignment,
    ParticipantRoleDefinition, ProfileActivationRecord, REASONING_POSTURE_V1_CONTRACT_LINE,
    ReasoningActivationStatus, ReasoningActivationTrigger, ReasoningAdmissionEffect,
    ReasoningCompatibilityWindow, ReasoningConfidenceContribution, ReasoningConfidenceLevel,
    ReasoningIterationCondition, ReasoningIterationKind, ReasoningIterationRecord,
    ReasoningObservedDistinctness, ReasoningOutcome, ReasoningOutcomeKind,
    ReasoningParticipantRoleKind, ReasoningParticipantStatus, ReasoningProfileDefinition,
    ReasoningRoutePreference,
};
use crate::domain::session::SessionStatus;
use crate::orchestrator::governance::governance_state_patch;
use crate::orchestrator::review_trace::record_reasoning_profile_events;
use crate::orchestrator::terminal::build_terminal_reason;

use super::{
    ActiveSessionRecord, ApprovalState, CanonMode, ExecutionTrace, GovernanceStepDecision,
    GovernedStageRecord, SessionRuntime, SessionRuntimeError, Task, TaskRunResponse,
    TraceEventType, compacted_canon_memory_for_block, current_timestamp_millis,
    governance_next_action_for_state, governance_projection_snapshot,
};

pub(super) struct ReasoningTraceContext<'a> {
    pub(super) step_id: &'a str,
    pub(super) plan_revision: usize,
}

pub(super) struct ReasoningGateContext<'a> {
    pub(super) runtime_kind: GovernanceRuntimeKind,
    pub(super) governance_attempt_id: &'a str,
    pub(super) selected_mode: Option<CanonMode>,
}

pub(super) const CURRENT_BOUNDLINE_VERSION: &str = env!("CARGO_PKG_VERSION");

impl SessionRuntime {
    pub(super) fn apply_reasoning_profile_gate(
        &self,
        session: &mut ActiveSessionRecord,
        trace: &mut ExecutionTrace,
        trace_context: ReasoningTraceContext<'_>,
        policy: &crate::domain::governance::StageGovernancePolicy,
        gate_context: ReasoningGateContext<'_>,
    ) -> Result<GovernanceStepDecision<TaskRunResponse>, SessionRuntimeError> {
        let stage_key = policy.stage_key();
        let Some(activation) = self.activate_reasoning_profile_for_stage(
            session,
            stage_key.as_str(),
            policy,
            gate_context.runtime_kind,
            gate_context.governance_attempt_id,
            gate_context.selected_mode,
        )?
        else {
            return Ok(GovernanceStepDecision::Continue);
        };

        if activation.status != ReasoningActivationStatus::Blocked {
            if let Some(event) = trace.events.iter_mut().rev().find(|event| {
                event.step_id.as_deref() == Some(trace_context.step_id)
                    && event.event_type == TraceEventType::GovernanceCompleted
            }) {
                if let Some(payload) = event.payload.as_object_mut() {
                    payload
                        .insert("reasoning_profile_record".to_string(), json!(activation.clone()));
                }
            } else {
                trace.record_event(
                    TraceEventType::GovernanceCompleted,
                    Some(trace_context.step_id.to_string()),
                    trace_context.plan_revision,
                    json!({
                        "stage_key": stage_key,
                        "runtime": gate_context.runtime_kind,
                        "headline": format!(
                            "reasoning profile {} {}",
                            activation.profile_id,
                            activation.status.as_str()
                        ),
                        "reasoning_profile_record": activation.clone(),
                    }),
                );
            }
            self.persist_reasoning_gate_state(
                session,
                trace,
                trace_context.step_id,
                trace_context.plan_revision,
                &activation,
            )?;
            return Ok(GovernanceStepDecision::Continue);
        }

        trace.record_event(
            TraceEventType::GovernanceBlocked,
            Some(trace_context.step_id.to_string()),
            trace_context.plan_revision,
            json!({
                "stage_key": stage_key,
                "runtime": gate_context.runtime_kind,
                "required": policy.required,
                "reason": reasoning_profile_block_message(&activation),
                "reasoning_profile": activation.profile_id,
                "reasoning_status": activation.status,
                "reasoning_activation_id": activation.activation_id,
                "reasoning_profile_record": activation.clone(),
            }),
        );
        self.persist_reasoning_gate_state(
            session,
            trace,
            trace_context.step_id,
            trace_context.plan_revision,
            &activation,
        )?;

        Ok(GovernanceStepDecision::Halt)
    }

    pub(super) fn interrupted_reasoning_profile_for_stage(
        &self,
        stage_key: &str,
        policy: &crate::domain::governance::StageGovernancePolicy,
        runtime_kind: GovernanceRuntimeKind,
        governance_attempt_id: &str,
        interruption_reason: &str,
    ) -> Result<Option<ProfileActivationRecord>, SessionRuntimeError> {
        let Some(definition) = policy.reasoning_profile.as_ref() else {
            return Ok(None);
        };
        if !definition.degradation_policy.interruptible {
            return Ok(None);
        }

        let posture =
            reasoning_posture_for_activation(definition, runtime_kind, governance_attempt_id)?;
        let mut basis = vec!["interruption=awaiting_approval".to_string()];
        if let Some(posture) = posture.as_ref() {
            basis.push(format!("posture_contract={}", posture.contract_line));
        }
        let next_action = governance_next_action_for_state(Some("awaiting_approval"));
        let activation = ProfileActivationRecord {
            activation_id: format!("{governance_attempt_id}-reasoning"),
            stage_key: stage_key.to_string(),
            profile_id: definition.profile_id,
            trigger: if runtime_kind == GovernanceRuntimeKind::Canon {
                ReasoningActivationTrigger::CanonRequiredChallenge
            } else {
                ReasoningActivationTrigger::OperatorPolicy
            },
            activation_reason: reasoning_activation_reason(stage_key, definition, runtime_kind),
            status: ReasoningActivationStatus::Interrupted,
            participants: Vec::new(),
            budget: definition.limits.clone(),
            posture,
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Interrupted,
                headline: format!(
                    "reasoning profile {} interrupted at {}",
                    definition.profile_id, stage_key
                ),
                disagreement_summary: Some(interruption_reason.to_string()),
                next_action,
                iterations: Vec::new(),
            }),
            confidence: Some(ReasoningConfidenceContribution {
                confidence_level: ReasoningConfidenceLevel::Low,
                basis,
                admission_effect: ReasoningAdmissionEffect::Gate,
                summary: "reasoning profile interrupted while governance approval is pending"
                    .to_string(),
            }),
        };
        activation
            .validate_against(definition)
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;

        Ok(Some(activation))
    }

    pub(super) fn activate_reasoning_profile_for_stage(
        &self,
        session: &mut ActiveSessionRecord,
        stage_key: &str,
        policy: &crate::domain::governance::StageGovernancePolicy,
        runtime_kind: GovernanceRuntimeKind,
        governance_attempt_id: &str,
        selected_mode: Option<CanonMode>,
    ) -> Result<Option<ProfileActivationRecord>, SessionRuntimeError> {
        let Some(definition) = policy.reasoning_profile.as_ref() else {
            return Ok(None);
        };

        let routing = effective_routing_for_workspace(&self.workspace_ref);
        let mut participants = reasoning_participants_for_profile(stage_key, definition, &routing);
        let independence = assess_reasoning_independence(stage_key, definition, &participants);
        let outcome =
            reasoning_outcome_for_activation(stage_key, definition, &participants, &independence);
        let status = reasoning_status_for_activation(&independence, outcome.as_ref());
        if status == ReasoningActivationStatus::Completed {
            mark_reasoning_participants_completed(&mut participants);
        }
        let trigger = if runtime_kind == GovernanceRuntimeKind::Canon {
            ReasoningActivationTrigger::CanonRequiredChallenge
        } else {
            ReasoningActivationTrigger::OperatorPolicy
        };
        let posture =
            reasoning_posture_for_activation(definition, runtime_kind, governance_attempt_id)?;
        let confidence = Some(reasoning_confidence_for_activation(
            runtime_kind,
            &independence,
            posture.as_ref(),
        ));
        let activation = ProfileActivationRecord {
            activation_id: format!("{governance_attempt_id}-reasoning"),
            stage_key: stage_key.to_string(),
            profile_id: definition.profile_id,
            trigger,
            activation_reason: reasoning_activation_reason(stage_key, definition, runtime_kind),
            status,
            participants,
            budget: definition.limits.clone(),
            posture,
            independence: Some(independence),
            outcome,
            confidence,
        };

        activation
            .validate_against(definition)
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;

        store_latest_reasoning_profile(session, runtime_kind, selected_mode, activation.clone());

        Ok(Some(activation))
    }

    pub(super) fn handle_governance_block(
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
            stage_council: None,
            packet_ref: None,
            decision_ref: decision.as_ref().map(|decision| decision.decision_id.clone()),
            blocked_reason: Some(block.reason.clone()),
        };
        let compacted_canon_memory =
            compacted_canon_memory_for_block(&block.stage_key, block.runtime, &block.reason);
        let projection = governance_projection_snapshot(
            &task.context,
            &block.stage_key,
            None,
            ApprovalState::NotNeeded,
        )
        .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
        let patch = governance_state_patch(
            &record,
            None,
            None,
            decision.as_ref(),
            compacted_canon_memory.as_ref(),
            &projection,
        )
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
                "latest_governance_runtime_state": projection.runtime_state,
                "latest_governance_rollout_profile": projection.rollout_profile,
                "latest_governance_reason": projection.reason.clone(),
                "latest_governance_contract_lines": projection.contract_lines.clone(),
                "latest_governance_approval_provenance": projection.approval_provenance.clone(),
            }),
        );
        let trace_location = self.persist_trace(&session.session_id, trace)?;
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

    fn persist_reasoning_gate_state(
        &self,
        session: &mut ActiveSessionRecord,
        trace: &mut ExecutionTrace,
        step_id: &str,
        plan_revision: usize,
        activation: &ProfileActivationRecord,
    ) -> Result<(), SessionRuntimeError> {
        record_reasoning_profile_events(trace, step_id, plan_revision, activation);
        let trace_location = self.persist_trace(&session.session_id, trace)?;
        session.latest_status = SessionStatus::Running;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = Some(trace_location);
        session.updated_at = current_timestamp_millis();
        Ok(())
    }
}

pub(super) fn is_governance_trace_event(event_type: TraceEventType) -> bool {
    matches!(
        event_type,
        TraceEventType::GovernanceSelected
            | TraceEventType::GovernanceStarted
            | TraceEventType::GovernanceDecisionRecorded
            | TraceEventType::GovernanceAwaitingApproval
            | TraceEventType::GovernanceCompleted
            | TraceEventType::GovernanceBlocked
            | TraceEventType::GovernancePacketRejected
    )
}

const REASONING_CONTEXT_BASIS_PREFIX: &str = "governance_stage";

pub(super) fn effective_routing_for_workspace(workspace: &Path) -> EffectiveRouting {
    let workspace_routing =
        FileConfigStore::for_workspace(workspace).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();

    resolve_effective_routing(
        &RoutingOverrides::default(),
        workspace_routing.as_ref(),
        cluster_routing.as_ref(),
        global_routing.as_ref(),
    )
}

pub(super) fn reasoning_participants_for_profile(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    routing: &EffectiveRouting,
) -> Vec<ParticipantAssignment> {
    let mut selected_roles = Vec::new();
    for role in &definition.participant_roles {
        if role.required {
            selected_roles.push(role);
        }
    }
    for role in &definition.participant_roles {
        if !role.required && selected_roles.len() < definition.limits.max_participants {
            selected_roles.push(role);
        }
    }

    let mut review_role_ordinal = 0usize;
    selected_roles
        .into_iter()
        .map(|role| {
            let assignment = participant_assignment_for_role(
                stage_key,
                role,
                routing,
                definition,
                review_role_ordinal,
            );
            if role_uses_configured_reviewer_routes(role) {
                review_role_ordinal += 1;
            }
            assignment
        })
        .collect()
}

fn participant_assignment_for_role(
    stage_key: &str,
    role: &ParticipantRoleDefinition,
    routing: &EffectiveRouting,
    definition: &ReasoningProfileDefinition,
    review_role_ordinal: usize,
) -> ParticipantAssignment {
    let (effective_route, provider_family) =
        reasoning_route_for_role(role, routing, review_role_ordinal);
    ParticipantAssignment {
        role_id: role.role_id.clone(),
        participant_id: format!("{}-{}", definition.profile_id.as_str(), role.role_id),
        effective_route,
        provider_family,
        context_basis: format!("{REASONING_CONTEXT_BASIS_PREFIX}:{stage_key}"),
        prompting_pattern: role.role_kind.as_str().to_string(),
        status: ReasoningParticipantStatus::Pending,
        result_summary: None,
    }
}

fn role_uses_configured_reviewer_routes(role: &ParticipantRoleDefinition) -> bool {
    role.preferred_slot == ReasoningRoutePreference::Review
        && matches!(
            role.role_kind,
            ReasoningParticipantRoleKind::BlindReviewer
                | ReasoningParticipantRoleKind::HeterogeneousReviewer
                | ReasoningParticipantRoleKind::Critic
                | ReasoningParticipantRoleKind::Reviser
        )
}

pub(super) fn reasoning_route_for_role(
    role: &ParticipantRoleDefinition,
    routing: &EffectiveRouting,
    review_role_ordinal: usize,
) -> (String, Option<String>) {
    if role.role_kind == ReasoningParticipantRoleKind::Arbiter {
        let route = &routing.adjudication.route;
        return (
            format!(
                "{}:{}:{}",
                ReasoningRoutePreference::Adjudication.as_str(),
                route.runtime,
                route.model
            ),
            Some(route.runtime.as_str().to_string()),
        );
    }

    if role.preferred_slot == ReasoningRoutePreference::Review {
        if let Some(route) = routing.reviewer_roles.get(&role.role_id) {
            return (
                format!(
                    "reviewer_roles.{}:{}:{}",
                    role.role_id, route.route.runtime, route.route.model
                ),
                Some(route.route.runtime.as_str().to_string()),
            );
        }

        if role_uses_configured_reviewer_routes(role)
            && let Some((reviewer_role_id, route)) =
                routing.reviewer_roles.iter().nth(review_role_ordinal)
        {
            return (
                format!(
                    "reviewer_roles.{}:{}:{}",
                    reviewer_role_id, route.route.runtime, route.route.model
                ),
                Some(route.route.runtime.as_str().to_string()),
            );
        }
    }

    let route = match role.preferred_slot {
        ReasoningRoutePreference::Planning => &routing.planning.route,
        ReasoningRoutePreference::Implementation => &routing.implementation.route,
        ReasoningRoutePreference::Verification => &routing.verification.route,
        ReasoningRoutePreference::Review => &routing.review.route,
        ReasoningRoutePreference::Adjudication => &routing.adjudication.route,
    };

    (
        format!("{}:{}:{}", role.preferred_slot.as_str(), route.runtime, route.model),
        Some(route.runtime.as_str().to_string()),
    )
}

fn requested_independence_floor(definition: &ReasoningProfileDefinition) -> IndependenceFloor {
    let mut roles = definition.participant_roles.iter();
    let Some(first_role) = roles.next() else {
        return IndependenceFloor {
            route_distinct: false,
            provider_distinct: false,
            context_distinct: false,
            prompt_pattern_distinct: false,
            minimum_participants: 1,
        };
    };
    let mut floor = first_role.independence_requirements.clone();
    for role in roles {
        floor.route_distinct |= role.independence_requirements.route_distinct;
        floor.provider_distinct |= role.independence_requirements.provider_distinct;
        floor.context_distinct |= role.independence_requirements.context_distinct;
        floor.prompt_pattern_distinct |= role.independence_requirements.prompt_pattern_distinct;
        floor.minimum_participants =
            floor.minimum_participants.max(role.independence_requirements.minimum_participants);
    }

    floor
}

pub(super) fn reasoning_posture_for_activation(
    definition: &ReasoningProfileDefinition,
    runtime_kind: GovernanceRuntimeKind,
    governance_attempt_id: &str,
) -> Result<Option<CanonChallengePostureInput>, SessionRuntimeError> {
    if runtime_kind != GovernanceRuntimeKind::Canon {
        return Ok(None);
    }

    let posture = CanonChallengePostureInput {
        contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
        compatibility_window: ReasoningCompatibilityWindow {
            boundline_min: CURRENT_BOUNDLINE_VERSION.to_string(),
            boundline_max_exclusive: next_minor_exclusive(CURRENT_BOUNDLINE_VERSION)?,
            canon_min: SUPPORTED_CANON_VERSION.to_string(),
            canon_max_exclusive: next_minor_exclusive(SUPPORTED_CANON_VERSION)?,
            contract_line: REASONING_POSTURE_V1_CONTRACT_LINE.to_string(),
        },
        required_profile_family: Some(definition.family),
        required_profile_id: Some(definition.profile_id),
        minimum_independence: requested_independence_floor(definition),
        admission_priority: CanonAdmissionPriority::RequiredBeforeContinue,
        confidence_handoff_required: true,
        provenance_ref: format!("governance_attempt:{governance_attempt_id}"),
    };

    posture
        .validate()
        .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;

    Ok(Some(posture))
}

fn reasoning_confidence_for_activation(
    runtime_kind: GovernanceRuntimeKind,
    independence: &IndependenceAssessment,
    posture: Option<&CanonChallengePostureInput>,
) -> ReasoningConfidenceContribution {
    let (confidence_level, admission_effect, summary) = match independence.result {
        IndependenceAssessmentResult::Passed if runtime_kind == GovernanceRuntimeKind::Canon => (
            ReasoningConfidenceLevel::High,
            ReasoningAdmissionEffect::None,
            "reasoning independence passed under the Canon-governed challenge posture"
                .to_string(),
        ),
        IndependenceAssessmentResult::Passed => (
            ReasoningConfidenceLevel::Medium,
            ReasoningAdmissionEffect::None,
            "reasoning independence passed under the requested participant topology"
                .to_string(),
        ),
        IndependenceAssessmentResult::Degraded => (
            ReasoningConfidenceLevel::Medium,
            ReasoningAdmissionEffect::Warn,
            "reasoning independence degraded; continue only with explicit caution"
                .to_string(),
        ),
        IndependenceAssessmentResult::Failed => (
            ReasoningConfidenceLevel::Low,
            ReasoningAdmissionEffect::Gate,
            "reasoning independence failed; block progression until challenge distinctness is restored"
                .to_string(),
        ),
    };

    let mut basis = vec![format!("independence={}", independence.result.as_str())];
    if let Some(posture) = posture {
        basis.push(format!("posture_contract={}", posture.contract_line));
    }

    ReasoningConfidenceContribution { confidence_level, basis, admission_effect, summary }
}

fn next_minor_exclusive(version: &str) -> Result<String, SessionRuntimeError> {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|value| value.parse::<u64>().ok()).ok_or_else(|| {
        SessionRuntimeError::ExecutionInvariant(format!(
            "invalid semantic version '{version}' for reasoning posture window"
        ))
    })?;
    let minor = parts.next().and_then(|value| value.parse::<u64>().ok()).ok_or_else(|| {
        SessionRuntimeError::ExecutionInvariant(format!(
            "invalid semantic version '{version}' for reasoning posture window"
        ))
    })?;
    let _patch = parts.next().and_then(|value| value.parse::<u64>().ok()).ok_or_else(|| {
        SessionRuntimeError::ExecutionInvariant(format!(
            "invalid semantic version '{version}' for reasoning posture window"
        ))
    })?;

    if parts.next().is_some() {
        return Err(SessionRuntimeError::ExecutionInvariant(format!(
            "invalid semantic version '{version}' for reasoning posture window"
        )));
    }

    Ok(format!("{major}.{}.0", minor + 1))
}

pub(super) fn assess_reasoning_independence(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    participants: &[ParticipantAssignment],
) -> IndependenceAssessment {
    let requested_floor = requested_independence_floor(definition);
    let observed_distinctions = observed_reasoning_distinctness(participants);
    let minimum = requested_floor.minimum_participants;
    let gaps = ReasoningIndependenceGaps::from_observed(
        &requested_floor,
        participants.len(),
        &observed_distinctions,
    );
    let result = gaps.result(
        definition.degradation_policy.allow_reduced_participants,
        definition.degradation_policy.allow_degraded_independence,
    );
    let reason = reasoning_independence_reason(
        stage_key,
        definition,
        participants.len(),
        minimum,
        &observed_distinctions,
        gaps,
        result,
    );

    IndependenceAssessment { requested_floor, observed_distinctions, result, reason }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReasoningIndependenceGaps {
    participant_gap: bool,
    route_gap: bool,
    provider_gap: bool,
    context_gap: bool,
    prompt_gap: bool,
}

impl ReasoningIndependenceGaps {
    pub(super) fn from_observed(
        requested_floor: &IndependenceFloor,
        participant_count: usize,
        observed_distinctions: &ReasoningObservedDistinctness,
    ) -> Self {
        let minimum = requested_floor.minimum_participants;

        Self {
            participant_gap: participant_count < minimum,
            route_gap: requested_floor.route_distinct
                && observed_distinctions.distinct_routes < minimum,
            provider_gap: requested_floor.provider_distinct
                && observed_distinctions.distinct_providers < minimum,
            context_gap: requested_floor.context_distinct
                && observed_distinctions.distinct_contexts < minimum,
            prompt_gap: requested_floor.prompt_pattern_distinct
                && observed_distinctions.distinct_prompt_patterns < minimum,
        }
    }

    const fn has_missing_distinctness(self) -> bool {
        self.route_gap || self.provider_gap || self.context_gap || self.prompt_gap
    }

    pub(super) fn result(
        self,
        allow_reduced_participants: bool,
        allow_degraded_independence: bool,
    ) -> IndependenceAssessmentResult {
        if !self.participant_gap && !self.has_missing_distinctness() {
            IndependenceAssessmentResult::Passed
        } else if (!self.participant_gap || allow_reduced_participants)
            && (!self.has_missing_distinctness() || allow_degraded_independence)
        {
            IndependenceAssessmentResult::Degraded
        } else {
            IndependenceAssessmentResult::Failed
        }
    }

    pub(super) fn missing_dimensions(
        self,
        participant_count: usize,
        minimum: usize,
        observed_distinctions: &ReasoningObservedDistinctness,
    ) -> Vec<String> {
        let mut missing = Vec::new();

        if self.participant_gap {
            missing.push(format!("participants={participant_count} < required={minimum}"));
        }
        if self.route_gap {
            missing.push(format!(
                "distinct_routes={} < required={minimum}",
                observed_distinctions.distinct_routes
            ));
        }
        if self.provider_gap {
            missing.push(format!(
                "distinct_providers={} < required={minimum}",
                observed_distinctions.distinct_providers
            ));
        }
        if self.context_gap {
            missing.push(format!(
                "distinct_contexts={} < required={minimum}",
                observed_distinctions.distinct_contexts
            ));
        }
        if self.prompt_gap {
            missing.push(format!(
                "distinct_prompt_patterns={} < required={minimum}",
                observed_distinctions.distinct_prompt_patterns
            ));
        }

        missing
    }
}

pub(super) fn observed_reasoning_distinctness(
    participants: &[ParticipantAssignment],
) -> ReasoningObservedDistinctness {
    ReasoningObservedDistinctness {
        distinct_routes: count_distinct_participant_values(participants, |participant| {
            Some(participant.effective_route.as_str())
        }),
        distinct_providers: count_distinct_participant_values(participants, |participant| {
            participant.provider_family.as_deref()
        }),
        distinct_contexts: count_distinct_participant_values(participants, |participant| {
            Some(participant.context_basis.as_str())
        }),
        distinct_prompt_patterns: count_distinct_participant_values(participants, |participant| {
            Some(participant.prompting_pattern.as_str())
        }),
    }
}

pub(super) fn count_distinct_participant_values<'a, F>(
    participants: &'a [ParticipantAssignment],
    selector: F,
) -> usize
where
    F: Fn(&'a ParticipantAssignment) -> Option<&'a str>,
{
    participants.iter().filter_map(selector).collect::<BTreeSet<_>>().len()
}

pub(super) fn reasoning_independence_reason(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    participant_count: usize,
    minimum: usize,
    observed_distinctions: &ReasoningObservedDistinctness,
    gaps: ReasoningIndependenceGaps,
    result: IndependenceAssessmentResult,
) -> String {
    if result == IndependenceAssessmentResult::Passed {
        format!(
            "reasoning profile {} satisfies the requested independence for {}",
            definition.profile_id, stage_key
        )
    } else {
        let missing = gaps.missing_dimensions(participant_count, minimum, observed_distinctions);
        format!(
            "reasoning profile {} cannot satisfy the requested independence for {}: {}",
            definition.profile_id,
            stage_key,
            missing.join(", ")
        )
    }
}

pub(super) fn reasoning_outcome_for_activation(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    participants: &[ParticipantAssignment],
    independence: &IndependenceAssessment,
) -> Option<ReasoningOutcome> {
    match independence.result {
        IndependenceAssessmentResult::Passed => {
            successful_reasoning_outcome(stage_key, definition, participants)
        }
        IndependenceAssessmentResult::Degraded => Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Degraded,
            headline: format!(
                "reasoning profile {} degraded at {}",
                definition.profile_id, stage_key
            ),
            disagreement_summary: Some(independence.reason.clone()),
            next_action: definition.degradation_policy.blocked_next_action.clone(),
            iterations: Vec::new(),
        }),
        IndependenceAssessmentResult::Failed => Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Blocked,
            headline: format!(
                "reasoning profile {} blocked at {}",
                definition.profile_id, stage_key
            ),
            disagreement_summary: Some(independence.reason.clone()),
            next_action: definition.degradation_policy.blocked_next_action.clone(),
            iterations: Vec::new(),
        }),
    }
}

fn successful_reasoning_outcome(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    participants: &[ParticipantAssignment],
) -> Option<ReasoningOutcome> {
    match definition.profile_id {
        crate::domain::reasoning::ReasoningProfileId::IndependentPairReview => {
            Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Adjudicated,
                headline: format!(
                    "reasoning profile {} completed at {}",
                    definition.profile_id, stage_key
                ),
                disagreement_summary: Some(
                    "independent blind review completed and governance adjudication accepted the bounded outcome"
                        .to_string(),
                ),
                next_action: None,
                iterations: Vec::new(),
            })
        }
        crate::domain::reasoning::ReasoningProfileId::HeterogeneousSecurityReview => {
            Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Converged,
                headline: format!(
                    "reasoning profile {} completed at {}",
                    definition.profile_id, stage_key
                ),
                disagreement_summary: Some(
                    "heterogeneous security review converged on a bounded approval-ready outcome"
                        .to_string(),
                ),
                next_action: None,
                iterations: Vec::new(),
            })
        }
        crate::domain::reasoning::ReasoningProfileId::BoundedReflexion => Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Converged,
            headline: format!(
                "reasoning profile {} completed at {}",
                definition.profile_id, stage_key
            ),
            disagreement_summary: Some(
                "bounded reflexion completed one critique-and-revise cycle and converged"
                    .to_string(),
            ),
            next_action: None,
            iterations: vec![ReasoningIterationRecord {
                iteration_kind: ReasoningIterationKind::ReflexionRevision,
                iteration_index: 0,
                participants: participants
                    .iter()
                    .map(|participant| participant.participant_id.clone())
                    .collect(),
                summary:
                    "critic challenged the proposed fix and reviser produced a bounded revision"
                        .to_string(),
                novelty: true,
                condition: ReasoningIterationCondition::Completed,
            }],
        }),
        _ => None,
    }
}

pub(super) fn reasoning_status_for_activation(
    independence: &IndependenceAssessment,
    outcome: Option<&ReasoningOutcome>,
) -> ReasoningActivationStatus {
    match independence.result {
        IndependenceAssessmentResult::Passed if outcome.is_some() => {
            ReasoningActivationStatus::Completed
        }
        IndependenceAssessmentResult::Passed => ReasoningActivationStatus::Active,
        IndependenceAssessmentResult::Degraded => ReasoningActivationStatus::Degraded,
        IndependenceAssessmentResult::Failed => ReasoningActivationStatus::Blocked,
    }
}

pub(super) fn mark_reasoning_participants_completed(participants: &mut [ParticipantAssignment]) {
    for participant in participants {
        participant.status = ReasoningParticipantStatus::Completed;
        if participant.result_summary.is_none() {
            participant.result_summary =
                Some(format!("completed via {}", participant.effective_route));
        }
    }
}

fn reasoning_activation_reason(
    stage_key: &str,
    definition: &ReasoningProfileDefinition,
    runtime_kind: GovernanceRuntimeKind,
) -> String {
    if runtime_kind == GovernanceRuntimeKind::Canon {
        format!(
            "Canon governance activated reasoning profile {} for {}",
            definition.profile_id, stage_key
        )
    } else {
        format!(
            "stage governance activated reasoning profile {} for {}",
            definition.profile_id, stage_key
        )
    }
}

pub(super) fn store_latest_reasoning_profile(
    session: &mut ActiveSessionRecord,
    runtime_kind: GovernanceRuntimeKind,
    selected_mode: Option<CanonMode>,
    activation: ProfileActivationRecord,
) {
    if let Some(lifecycle) = session.governance_lifecycle.as_mut() {
        lifecycle.governance_runtime = runtime_kind;
        if lifecycle.selected_mode.is_none() {
            lifecycle.selected_mode = selected_mode;
        }
        if let Some(mode) = selected_mode
            && !lifecycle.selected_mode_sequence.contains(&mode)
        {
            lifecycle.selected_mode_sequence.push(mode);
        }
        lifecycle.latest_reasoning_profile = Some(activation);
        return;
    }

    session.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: runtime_kind,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::default(),
        selected_mode,
        selected_mode_sequence: selected_mode.into_iter().collect(),
        latest_reasoning_profile: Some(activation),
        current_stage_index: 0,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    });
}

pub(super) fn reasoning_profile_block_message(record: &ProfileActivationRecord) -> String {
    let detail = record
        .outcome
        .as_ref()
        .and_then(|outcome| outcome.disagreement_summary.clone())
        .unwrap_or_else(|| record.activation_reason.clone());
    if let Some(next_action) =
        record.outcome.as_ref().and_then(|outcome| outcome.next_action.as_ref())
    {
        format!(
            "reasoning profile {} blocked stage {}: {}. next action: {}",
            record.profile_id, record.stage_key, detail, next_action
        )
    } else {
        format!(
            "reasoning profile {} blocked stage {}: {}",
            record.profile_id, record.stage_key, detail
        )
    }
}

pub(super) struct GovernanceBlockContext {
    pub(super) step_id: String,
    pub(super) stage_key: String,
    pub(super) required: bool,
    pub(super) autopilot_enabled: bool,
    pub(super) runtime: GovernanceRuntimeKind,
    pub(super) reason: String,
}
