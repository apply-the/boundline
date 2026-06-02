use crate::adapters::agent::{FrameworkAdapterHost, FrameworkAdapterHostError};
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::framework_adapter::{
    AdapterExecutionSource, AdapterFailureClass, AdapterLifecycleStageKey,
    LifecycleStageExecutionStatus, StageClaimState, StageRoutingDecisionReason,
};
use crate::domain::session::{FrameworkAdapterStageFailureDetails, LifecycleStageExecutionRecord};

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use serde_json::json;
use uuid::Uuid;

use super::{
    ActiveSessionRecord, BacklogQualityAssessment, BacklogQualityState, CanonMode,
    CanonModeSelectionPreference, ContextPackCredibility, CouncilProfile, FileConfigStore,
    FlowPolicy, FrameworkAdapterStageFailedTracePayload, GoalPlan, GoalPlannerError,
    GovernanceLifecycleState, GovernanceRuntimeKind, GovernedSessionLifecycle,
    NegotiationResolutionState, PLAN_QUALITY_BLOCKED_DEFAULT_PROMPT, PLAN_QUALITY_BLOCKED_HEADLINE,
    PLAN_QUALITY_CLARIFICATION_DEFAULT_PROMPT, PLAN_QUALITY_CLARIFICATION_HEADLINE,
    PLAN_QUALITY_CLARIFICATION_PROMPT_PREFIX, PlanQualityAssessment, PlanQualityState,
    PlanningAnalysisProjection, PlanningAnalysisState, PlanningContextSources,
    ProviderReviewDisposition, ProviderReviewRequest, ProviderRevisionRequest,
    ProviderWorkspaceFile, ReviewerFinding, ReviewerParticipation, ReviewerParticipationStatus,
    SessionRuntime, SessionRuntimeError, SessionStatus, StageCouncilAdjudication,
    StageCouncilArtifact, StageCouncilFinding, StageCouncilFindingDisposition, StageCouncilOutcome,
    StageCouncilRequest, StageCouncilStatus, StageCouncilVoteResolution, Task, TraceEventType,
    UPSTREAM_ARCHITECTURE_DECISIONS_FILE, UPSTREAM_CONSTRAINTS_FILE, UPSTREAM_DOMAIN_MODEL_FILE,
    UPSTREAM_PRD_FILE, UPSTREAM_SCOPE_CUTS_FILE, UPSTREAM_SYSTEM_SHAPE_FILE, VoteDecision,
    VoteRuleDefinition, backlog_quality_snapshot_for_lifecycle, build_fixture_plan_for_goal,
    build_goal_plan_with_sources, build_task_request, build_terminal_reason, built_in_flow,
    compute_planning_input_fingerprint, configured_framework_adapter_binding,
    current_timestamp_millis, discovery_stage_council_reviewers,
    framework_adapter_stage_failure_terminal_condition,
    framework_adapter_stage_outcome_details_from_response,
    framework_adapter_stage_routing_trace_payload, framework_adapter_stage_routing_value,
    load_workspace_execution_profile, map_framework_adapter_failure_class, model_route_label,
    plain_goal_planning_clarification_prompt, plain_goal_requires_planning_clarification,
    planned_canon_mode_sequence_for_flow, planning_canon_mode_for_stage_key,
    planning_stage_brief_ref, project_scale_state_for_goal, protocol_error_code_from_host_error,
    provider_review_disposition_text, read_upstream_artifact_capped, render_planning_stage_brief,
    render_stage_council_blocked_markdown, resolve_council_assembly, review_workspace,
    reviewer_disposition_from_provider, revise_artifact, route_is_available,
    session_status_for_task_status, stage_council_disposition_from_provider,
    supported_flow_names_csv, task_status_for_condition,
};
use crate::domain::limits::TerminalCondition;

const PROJECT_SCALE_CONFIRM_PATH: &str = "confirm_project_scale_path";
const PROJECT_SCALE_REPAIR_CONTEXT_PATH: &str = "repair_context";
const ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY: &str = "unavailable_binary";
const ADAPTER_FALLBACK_REASON_UNSUPPORTED_TRANSPORT: &str = "unsupported_transport";
const ADAPTER_FALLBACK_REASON_PREFLIGHT_BLOCKED: &str = "preflight_blocked";
const STAGE_COUNCIL_PRODUCER_ARTIFACT_SUFFIX: &str = "producer";
const STAGE_COUNCIL_REVISED_ARTIFACT_SUFFIX: &str = "revised";
const STAGE_COUNCIL_VOTE_STRATEGY_BOUNDED_MAJORITY: &str = "bounded_majority";

enum FrameworkAdapterPlanStageOutcome {
    NotClaimed,
    ClaimedSucceeded(StageRoutingDecisionRecord),
    ClaimedBlocked(FrameworkAdapterStageFailureDetails),
    ClaimedFailed(FrameworkAdapterStageFailureDetails),
}

impl SessionRuntime {
    // Builds a compatibility task when fixture execution remains the
    // authoritative runtime for the chosen flow.
    pub(super) fn plan_compatibility_task(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        if let Some(active_flow) = &session.active_flow {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }

        let request = build_task_request(
            &self.workspace_ref,
            &goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan =
            build_fixture_plan_for_goal(&self.workspace_ref, session.active_flow.as_ref(), &goal)
                .map_err(SessionRuntimeError::FixtureRuntime)?;
        let task = Task::new(Uuid::new_v4().to_string(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;

        session.goal_plan = None;
        session.active_task = Some(task);
        session.decisions.clear();
        session.active_flow_policy = session
            .active_flow
            .as_ref()
            .and_then(|flow| FlowPolicy::from_builtin(&flow.flow_name).ok());
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    // Builds or refreshes the native goal plan, preserving partial planning
    // state when bounded context is still insufficient.
    pub(super) fn plan_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let project_scale_state = project_scale_state_for_goal(&goal, PROJECT_SCALE_CONFIRM_PATH);
        if !no_flow
            && requested_flow.is_none()
            && let Some(active_flow) = &session.active_flow
        {
            active_flow
                .validate()
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?;
        }
        if let Some(flow_name) = requested_flow {
            built_in_flow(flow_name).ok_or_else(|| SessionRuntimeError::UnknownFlow {
                requested: flow_name.to_string(),
                supported: supported_flow_names_csv(),
            })?;
        }

        if let Some(packet) = self.session_negotiation_packet(session, &goal)
            && packet.resolution_state == NegotiationResolutionState::PendingClarification
        {
            session.active_task = None;
            session.goal_plan = None;
            session.project_scale = project_scale_state.clone();
            session.decisions.clear();
            session.latest_status = SessionStatus::GoalCaptured;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();
            let prompt = packet
                .constraints
                .iter()
                .find(|constraint| constraint.blocks_planning)
                .map(|constraint| constraint.summary.clone())
                .unwrap_or_else(|| {
                    "resolve the blocking clarification before planning can continue".to_string()
                });
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: packet
                    .clarification_headline
                    .unwrap_or_else(|| "clarification required before planning".to_string()),
                prompt,
            });
        }

        if let Some(authored_brief) = session.authored_brief.as_ref()
            && authored_brief.clarification.is_some()
        {
            session.active_task = None;
            session.goal_plan = None;
            session.project_scale = project_scale_state.clone();
            session.decisions.clear();
            session.latest_status = SessionStatus::GoalCaptured;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();
            return Err(SessionRuntimeError::ClarificationRequired {
                headline: authored_brief
                    .clarification_headline()
                    .unwrap_or_else(|| "bounded context required before planning".to_string()),
                prompt: authored_brief.clarification_prompt().unwrap_or_else(|| {
                    "capture a narrower goal before planning can continue".to_string()
                }),
            });
        }

        let context_sources = self.planning_context_sources(session, &goal);
        let native_flow_state = if no_flow {
            None
        } else if let Some(flow_name) = requested_flow {
            built_in_flow(flow_name).map(|flow| flow.initial_state())
        } else {
            session.active_flow.clone()
        };
        let preserved_flow_policy =
            if native_flow_state.is_some() { session.active_flow_policy.clone() } else { None };
        let preferred_flow = native_flow_state.as_ref().map(|flow| flow.flow_name.as_str());
        let mut goal_plan = match build_goal_plan_with_sources(
            &goal,
            &self.workspace_ref,
            &context_sources,
            preferred_flow,
        ) {
            Ok(goal_plan) => goal_plan,
            Err(GoalPlannerError::MissingGoal) => return Err(SessionRuntimeError::MissingGoal),
            Err(GoalPlannerError::InsufficientContext { summary, goal_plan }) => {
                let mut goal_plan = *goal_plan;
                self.apply_negotiation_projection(session, &goal, &mut goal_plan);
                if no_flow {
                    goal_plan.mark_flow_skipped();
                }

                session.active_flow = native_flow_state.clone();
                session.active_task = None;
                session.goal_plan = Some(goal_plan);
                session.project_scale =
                    project_scale_state_for_goal(&goal, PROJECT_SCALE_REPAIR_CONTEXT_PATH);
                session.decisions.clear();
                session.active_flow_policy = preserved_flow_policy.clone();
                session.latest_status = SessionStatus::Blocked;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = None;
                session.updated_at = current_timestamp_millis();

                return Err(SessionRuntimeError::ClarificationRequired {
                    headline: "bounded context required before planning".to_string(),
                    prompt: summary,
                });
            }
            Err(GoalPlannerError::PlanCreation(error)) => {
                return Err(SessionRuntimeError::GoalPlan(error.to_string()));
            }
        };
        if session.authored_brief.is_none()
            && plain_goal_requires_planning_clarification(&goal, &context_sources)
        {
            self.apply_negotiation_projection(session, &goal, &mut goal_plan);
            if no_flow {
                goal_plan.mark_flow_skipped();
            }

            session.active_flow = native_flow_state.clone();
            session.active_task = None;
            session.goal_plan = Some(goal_plan);
            session.project_scale =
                project_scale_state_for_goal(&goal, PROJECT_SCALE_REPAIR_CONTEXT_PATH);
            session.decisions.clear();
            session.active_flow_policy = preserved_flow_policy.clone();
            session.latest_status = SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();

            return Err(SessionRuntimeError::ClarificationRequired {
                headline: "bounded context required before planning".to_string(),
                prompt: plain_goal_planning_clarification_prompt(),
            });
        }

        if let Some(previous_goal_plan) = session.goal_plan.as_ref() {
            let previous_revision = previous_goal_plan.proposal_revision;
            goal_plan.proposal_revision = previous_goal_plan.next_revision();
            goal_plan.planning_rationale = Some(match goal_plan.planning_rationale.take() {
                Some(rationale) => format!(
                    "{rationale}; supersedes revision {previous_revision} because workspace evidence changed or the operator requested a fresh plan"
                ),
                None => format!(
                    "supersedes revision {previous_revision} because workspace evidence changed or the operator requested a fresh plan"
                ),
            });
        }

        let planned_governed_flow_name = if no_flow {
            None
        } else {
            native_flow_state
                .as_ref()
                .map(|flow| flow.flow_name.clone())
                .or_else(|| goal_plan.flow.as_ref().map(|flow| flow.flow_name.clone()))
        };

        self.apply_negotiation_projection(session, &goal, &mut goal_plan);
        if no_flow {
            goal_plan.mark_flow_skipped();
        }
        let plan_quality = goal_plan.plan_quality_assessment();
        if !matches!(plan_quality.state, PlanQualityState::Ready) {
            let (headline, prompt) = Self::plan_quality_gate_details(&goal_plan, &plan_quality);

            session.active_flow = native_flow_state.clone();
            session.active_task = None;
            session.goal_plan = Some(goal_plan);
            session.project_scale =
                project_scale_state_for_goal(&goal, PROJECT_SCALE_REPAIR_CONTEXT_PATH);
            session.decisions.clear();
            session.active_flow_policy = preserved_flow_policy.clone();
            session.latest_status = SessionStatus::Blocked;
            session.latest_terminal_reason = None;
            session.latest_trace_ref = None;
            session.updated_at = current_timestamp_millis();

            return Err(SessionRuntimeError::ClarificationRequired { headline, prompt });
        }
        let should_confirm_goal_plan =
            requested_flow.is_some() || session.active_flow.is_some() || no_flow;

        self.ensure_workspace_governance_lifecycle(session);
        let planning_fingerprint = compute_planning_input_fingerprint(&goal, session);
        self.reset_planning_governance_state(session, &planning_fingerprint);
        self.sync_governed_planning_sequence(session, planned_governed_flow_name.as_deref());
        let planning_requests =
            self.prepare_planning_governance_requests(session, &goal_plan, &context_sources)?;
        self.execute_planning_governance_requests(
            session,
            &mut goal_plan,
            planning_requests,
            &context_sources,
        )?;
        if let Some(lifecycle) = session.governance_lifecycle.as_mut() {
            lifecycle.planning_input_fingerprint = Some(planning_fingerprint);
        }
        let planning_blocked = self.unresolved_planning_governance_record(session).is_some();
        goal_plan.planning_analysis = if planning_blocked {
            None
        } else {
            self.planning_analysis_projection(session, &goal_plan)
        };
        let planning_analysis_blocked = goal_plan
            .planning_analysis
            .as_ref()
            .is_some_and(|projection| matches!(projection.state, PlanningAnalysisState::Blocked));

        let mut framework_adapter_trace_ref = None;
        let mut framework_adapter_blocked_reason = None;
        match self.maybe_apply_framework_adapter_plan_stage(&mut goal_plan)? {
            FrameworkAdapterPlanStageOutcome::NotClaimed => {
                if should_confirm_goal_plan {
                    goal_plan
                        .confirm()
                        .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
                }
            }
            FrameworkAdapterPlanStageOutcome::ClaimedSucceeded(routing_record) => {
                if should_confirm_goal_plan {
                    goal_plan
                        .confirm()
                        .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
                }
                let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
                trace.record_event(
                    TraceEventType::StageRouted,
                    None,
                    goal_plan.proposal_revision,
                    framework_adapter_stage_routing_value(
                        framework_adapter_stage_routing_trace_payload(routing_record),
                    )?,
                );
                framework_adapter_trace_ref =
                    Some(self.persist_trace(&session.session_id, &mut trace)?);
            }
            FrameworkAdapterPlanStageOutcome::ClaimedBlocked(blocked) => {
                let terminal_reason = build_terminal_reason(
                    TerminalCondition::NoCredibleNextStep,
                    blocked.summary.clone(),
                    Some(serde_json::to_value(&blocked).map_err(|error| {
                        SessionRuntimeError::ExecutionInvariant(format!(
                            "failed to serialize framework-adapter plan-stage blocked details: {error}"
                        ))
                    })?),
                );
                let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
                trace.record_event(
                    TraceEventType::StageRouted,
                    None,
                    goal_plan.proposal_revision,
                    framework_adapter_stage_routing_value(
                        framework_adapter_stage_routing_trace_payload(
                            plan_stage_routing_record_from_blocked(&blocked),
                        ),
                    )?,
                );
                framework_adapter_trace_ref =
                    Some(self.persist_trace(&session.session_id, &mut trace)?);
                framework_adapter_blocked_reason = Some(terminal_reason);
            }
            FrameworkAdapterPlanStageOutcome::ClaimedFailed(failure) => {
                let terminal_reason = build_terminal_reason(
                    framework_adapter_stage_failure_terminal_condition(&failure),
                    failure.summary.clone(),
                    Some(serde_json::to_value(&failure).map_err(|error| {
                        SessionRuntimeError::ExecutionInvariant(format!(
                            "failed to serialize framework-adapter plan-stage failure details: {error}"
                        ))
                    })?),
                );
                let terminal_status = task_status_for_condition(terminal_reason.condition);
                let mut trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
                trace.record_event(
                    TraceEventType::StageRouted,
                    None,
                    goal_plan.proposal_revision,
                    framework_adapter_stage_routing_value(
                        framework_adapter_stage_routing_trace_payload(
                            plan_stage_routing_record_from_failure(&failure),
                        ),
                    )?,
                );
                trace.record_event(
                    TraceEventType::StageFailed,
                    None,
                    goal_plan.proposal_revision,
                    serde_json::to_value(&FrameworkAdapterStageFailedTracePayload {
                        stage_id: AdapterLifecycleStageKey::Plan.as_str().to_string(),
                        stage_key: AdapterLifecycleStageKey::Plan,
                        reason: failure.summary.clone(),
                        summary: failure.summary.clone(),
                        framework_adapter_stage_failure: failure.clone(),
                    })
                    .map_err(|error| {
                        SessionRuntimeError::ExecutionInvariant(format!(
                            "failed to serialize framework-adapter plan-stage trace payload: {error}"
                        ))
                    })?,
                );
                trace.record_event(
                    TraceEventType::TerminalRecorded,
                    None,
                    goal_plan.proposal_revision,
                    json!({
                        "cluster_delivery_story": serde_json::Value::Null,
                        "terminal_status": terminal_status,
                        "terminal_reason": terminal_reason.clone(),
                    }),
                );
                trace.finalize(terminal_status, terminal_reason.clone());
                let trace_location = self.persist_trace(&session.session_id, &mut trace)?;

                session.active_flow = native_flow_state;
                session.active_task = None;
                session.goal_plan = Some(goal_plan);
                session.project_scale = project_scale_state;
                session.decisions.clear();
                session.active_flow_policy = preserved_flow_policy;
                session.latest_status = session_status_for_task_status(terminal_status);
                session.latest_terminal_reason = Some(terminal_reason);
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();

                return Err(SessionRuntimeError::ExecutionInvariant(format!(
                    "framework-adapter plan stage execution failed after claim: {}",
                    failure.summary
                )));
            }
        }

        session.active_flow = native_flow_state;
        session.active_task = None;
        session.goal_plan = Some(goal_plan);
        session.project_scale = project_scale_state;
        session.decisions.clear();
        session.active_flow_policy = preserved_flow_policy;
        session.latest_status = if framework_adapter_blocked_reason.is_some()
            || planning_blocked
            || planning_analysis_blocked
        {
            SessionStatus::Blocked
        } else {
            SessionStatus::Planned
        };
        session.latest_terminal_reason = framework_adapter_blocked_reason;
        session.latest_trace_ref = framework_adapter_trace_ref;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    fn plan_quality_gate_details(
        goal_plan: &GoalPlan,
        assessment: &PlanQualityAssessment,
    ) -> (String, String) {
        if matches!(assessment.state, PlanQualityState::Blocked) {
            if let Some(context_pack) = goal_plan.context_pack.as_ref() {
                match context_pack.credibility {
                    ContextPackCredibility::Insufficient => {
                        return (
                            PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                            context_pack.summary.clone(),
                        );
                    }
                    ContextPackCredibility::Stale => {
                        return (
                            PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                            context_pack
                                .staleness_reason
                                .clone()
                                .unwrap_or_else(|| context_pack.summary.clone()),
                        );
                    }
                    ContextPackCredibility::Credible => {}
                }
            }

            return (
                PLAN_QUALITY_BLOCKED_HEADLINE.to_string(),
                PLAN_QUALITY_BLOCKED_DEFAULT_PROMPT.to_string(),
            );
        }

        if assessment.findings.is_empty() {
            return (
                PLAN_QUALITY_CLARIFICATION_HEADLINE.to_string(),
                PLAN_QUALITY_CLARIFICATION_DEFAULT_PROMPT.to_string(),
            );
        }

        (
            PLAN_QUALITY_CLARIFICATION_HEADLINE.to_string(),
            format!("{PLAN_QUALITY_CLARIFICATION_PROMPT_PREFIX}{}", assessment.findings.join(", ")),
        )
    }

    fn planning_analysis_projection(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
    ) -> Option<PlanningAnalysisProjection> {
        let backlog_snapshot = session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            backlog_quality_snapshot_for_lifecycle(lifecycle, &self.workspace_ref)
        });

        if let Some(snapshot) = backlog_snapshot {
            if !matches!(snapshot.assessment.state, BacklogQualityState::Ready) {
                return None;
            }

            return Some(
                goal_plan
                    .planning_analysis_projection(&snapshot.assessment, &snapshot.document_bodies),
            );
        }

        Some(
            goal_plan.planning_analysis_projection(
                &Self::default_planning_analysis_backlog_quality(),
                &[],
            ),
        )
    }

    fn default_planning_analysis_backlog_quality() -> BacklogQualityAssessment {
        BacklogQualityAssessment {
            state: BacklogQualityState::Ready,
            findings: Vec::new(),
            task_count: None,
            mvp_scope: None,
            unmapped_items: Vec::new(),
        }
    }

    fn maybe_apply_framework_adapter_plan_stage(
        &self,
        goal_plan: &mut GoalPlan,
    ) -> Result<FrameworkAdapterPlanStageOutcome, SessionRuntimeError> {
        let binding =
            configured_framework_adapter_binding(&self.workspace_ref).map_err(|error| {
                SessionRuntimeError::ExecutionInvariant(format!(
                    "failed to load framework-adapter runtime binding: {error}"
                ))
            })?;
        let Some(binding) = binding else {
            return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
        };
        let adapter_id = binding.selection.selection.adapter_id.clone();

        let describe = match binding.host.describe() {
            Ok(describe) => describe,
            Err(_) => {
                append_adapter_fallback_reason(
                    goal_plan,
                    ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY,
                );
                return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
            }
        };

        if !describe
            .declared_stage_overrides
            .contains(&crate::orchestrator::FrameworkStageKey::Plan)
        {
            return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
        }

        if !crate::adapters::framework_adapter_supports_v1_transport(&describe.supported_transports)
        {
            append_adapter_fallback_reason(
                goal_plan,
                ADAPTER_FALLBACK_REASON_UNSUPPORTED_TRANSPORT,
            );
            return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
        }

        let config_values = framework_adapter_config_values(&binding.selection);
        let preflight =
            match binding.host.preflight(&crate::adapters::FrameworkAdapterPreflightRequest {
                boundline_version: env!("CARGO_PKG_VERSION").to_string(),
                workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
                non_interactive: true,
                config_values: config_values.clone(),
            }) {
                Ok(preflight) => preflight,
                Err(_) => {
                    append_adapter_fallback_reason(
                        goal_plan,
                        ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY,
                    );
                    return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
                }
            };

        if preflight.status == crate::adapters::FrameworkAdapterPreflightStatus::Blocked {
            append_adapter_fallback_reason(goal_plan, ADAPTER_FALLBACK_REASON_PREFLIGHT_BLOCKED);
            return Ok(FrameworkAdapterPlanStageOutcome::NotClaimed);
        }

        let runtime_config_values = if preflight.normalized_config_values.is_empty() {
            config_values
        } else {
            preflight.normalized_config_values.clone()
        };

        let run_id = Uuid::new_v4();
        match binding.host.execute_stage(&crate::adapters::FrameworkAdapterExecuteStageRequest {
            run_id,
            stage_key: crate::orchestrator::FrameworkStageKey::Plan,
            stage_attempt: 1,
            workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
            adapter_id,
            config_values: runtime_config_values,
            context_artifacts: Vec::new(),
        }) {
            Ok(response)
                if response.status
                    == crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded =>
            {
                let response_details =
                    framework_adapter_stage_outcome_details_from_response(&response);
                Ok(FrameworkAdapterPlanStageOutcome::ClaimedSucceeded(
                    plan_stage_routing_record_from_success(
                        run_id,
                        binding.selection.selection.adapter_id.clone(),
                        response.produced_artifacts,
                        response_details,
                    ),
                ))
            }
            Ok(response)
                if response.status
                    == crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked =>
            {
                Ok(FrameworkAdapterPlanStageOutcome::ClaimedBlocked(
                    plan_stage_blocked_from_execute_response(
                        run_id,
                        binding.selection.selection.adapter_id.clone(),
                        response,
                    ),
                ))
            }
            Ok(response) => Ok(FrameworkAdapterPlanStageOutcome::ClaimedFailed(
                plan_stage_failure_from_execute_response(
                    run_id,
                    binding.selection.selection.adapter_id.clone(),
                    response,
                ),
            )),
            Err(error) => Ok(FrameworkAdapterPlanStageOutcome::ClaimedFailed(
                plan_stage_failure_from_host_error(
                    run_id,
                    binding.selection.selection.adapter_id.clone(),
                    error,
                ),
            )),
        }
    }

    pub(super) fn ensure_workspace_governance_lifecycle(&self, session: &mut ActiveSessionRecord) {
        if session.governance_lifecycle.is_some() {
            return;
        }

        let Some(governance_runtime) = self.resolve_workspace_governance_runtime(session) else {
            return;
        };

        session.governance_lifecycle = Some(GovernedSessionLifecycle {
            governance_runtime,
            explicit_opt_out: governance_runtime == GovernanceRuntimeKind::Local,
            mode_selection_preference: self.resolve_workspace_mode_selection_preference(),
            selected_mode: session
                .authored_brief
                .as_ref()
                .and_then(|bundle| bundle.governance_intent.as_ref())
                .and_then(|intent| intent.explicit_mode),
            selected_mode_sequence: Vec::new(),
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: Vec::new(),
            accumulated_context: Vec::new(),
            terminal_reason: None,
            planning_input_fingerprint: None,
        });
    }

    fn reset_planning_governance_state(
        &self,
        session: &mut ActiveSessionRecord,
        new_fingerprint: &str,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if lifecycle.planning_input_fingerprint.as_deref() == Some(new_fingerprint) {
            let all_planning_stages_clear = lifecycle
                .stage_records
                .iter()
                .filter(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_some())
                .all(|record| {
                    matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::GovernedReady
                            | GovernanceLifecycleState::Completed
                    )
                });
            if all_planning_stages_clear {
                return;
            }
            for record in lifecycle.stage_records.iter_mut() {
                if planning_canon_mode_for_stage_key(&record.stage_key).is_some()
                    && !matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::GovernedReady
                            | GovernanceLifecycleState::Completed
                    )
                {
                    record.lifecycle_state = GovernanceLifecycleState::PendingSelection;
                    record.blocked_reason = None;
                }
            }
            lifecycle.current_stage_index = 0;
            lifecycle.terminal_reason = None;
            return;
        }

        lifecycle
            .stage_records
            .retain(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_none());
        lifecycle
            .accumulated_context
            .retain(|reference| planning_canon_mode_for_stage_key(&reference.stage_key).is_none());
        lifecycle.current_stage_index = 0;
        lifecycle.terminal_reason = None;
    }

    pub(super) fn resolve_workspace_governance_runtime(
        &self,
        session: &ActiveSessionRecord,
    ) -> Option<GovernanceRuntimeKind> {
        if let Some(governance_intent) =
            session.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref())
        {
            if governance_intent.explicit_no_canon {
                return Some(GovernanceRuntimeKind::Local);
            }
            if let Some(runtime_preference) = governance_intent.runtime_preference {
                return Some(runtime_preference);
            }
        }

        let local_config =
            FileConfigStore::for_workspace(&self.workspace_ref).load_local().ok().flatten();
        let global_config = FileConfigStore::load_global().ok().flatten();

        load_workspace_execution_profile(&self.workspace_ref)
            .ok()
            .and_then(|profile| profile.governance.map(|governance| governance.default_runtime))
            .or_else(|| {
                (local_config.as_ref().and_then(|config| config.canon.as_ref()).is_some()
                    || global_config.as_ref().and_then(|config| config.canon.as_ref()).is_some())
                .then_some(GovernanceRuntimeKind::Canon)
            })
    }

    fn resolve_workspace_mode_selection_preference(&self) -> CanonModeSelectionPreference {
        let local_config =
            FileConfigStore::for_workspace(&self.workspace_ref).load_local().ok().flatten();
        let global_config = FileConfigStore::load_global().ok().flatten();

        local_config
            .and_then(|config| config.canon.map(|canon| canon.mode_selection))
            .or_else(|| {
                global_config.and_then(|config| config.canon.map(|canon| canon.mode_selection))
            })
            .unwrap_or_default()
    }

    pub(super) fn sync_governed_planning_sequence(
        &self,
        session: &mut ActiveSessionRecord,
        flow_name: Option<&str>,
    ) {
        let Some(flow_name) = flow_name else {
            return;
        };
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon
            || lifecycle.explicit_opt_out
            || !lifecycle.selected_mode_sequence.is_empty()
        {
            return;
        }

        let planned_sequence = planned_canon_mode_sequence_for_flow(flow_name);
        if planned_sequence.is_empty() {
            return;
        }

        if lifecycle.selected_mode.is_none() {
            lifecycle.selected_mode = planned_sequence.first().copied();
        }
        lifecycle.selected_mode_sequence = planned_sequence;
    }

    pub(super) fn planning_governance_read_targets(
        &self,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
    ) -> Vec<String> {
        let mut read_targets = Vec::new();
        let mut seen = BTreeSet::new();

        for target in goal_plan
            .context_pack
            .as_ref()
            .map(|context_pack| context_pack.selected_targets.as_slice())
            .unwrap_or_default()
        {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }
        for target in &context_sources.execution_profile_read_targets {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }
        for target in &context_sources.latest_changed_files {
            if seen.insert(target.clone()) {
                read_targets.push(target.clone());
            }
        }

        read_targets
    }

    pub(super) fn materialize_planning_stage_brief(
        &self,
        stage_key: &str,
        mode: CanonMode,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Result<String, SessionRuntimeError> {
        let stage_brief_ref = planning_stage_brief_ref(stage_key).ok_or_else(|| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to resolve planning stage brief path for {stage_key}"
            ))
        })?;
        let stage_brief_path = self.workspace_ref.join(&stage_brief_ref);
        let stage_directory = stage_brief_path.parent().ok_or_else(|| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to resolve planning stage brief directory for {stage_key}"
            ))
        })?;
        fs::create_dir_all(stage_directory).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to create planning governance directory for {stage_key}: {error}"
            ))
        })?;

        let mut brief_content =
            render_planning_stage_brief(stage_key, mode, goal_plan, context_sources);

        if let Some(upstream_section) =
            self.render_upstream_evidence_for_mode(mode, accumulated_context)
        {
            brief_content.push_str(&upstream_section);
        }

        fs::write(&stage_brief_path, &brief_content).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write planning stage brief for {stage_key}: {error}"
            ))
        })?;

        Ok(stage_brief_ref)
    }

    fn render_upstream_evidence_for_mode(
        &self,
        mode: CanonMode,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        match mode {
            CanonMode::Architecture => {
                self.render_architecture_upstream_evidence(accumulated_context)
            }
            CanonMode::Backlog => self.render_backlog_upstream_evidence(accumulated_context),
            _ => None,
        }
    }

    fn render_architecture_upstream_evidence(
        &self,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        let system_shaping_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::SystemShaping);
        let requirements_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Requirements);

        let mut section = String::new();
        let mut has_content = false;

        if let Some(doc_ref) = system_shaping_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_SYSTEM_SHAPE_FILE)
            {
                section.push_str("\n\n## Boundaries\n\n### System Context\n\n");
                section.push_str(&content);
                has_content = true;
            }
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_DOMAIN_MODEL_FILE)
            {
                section.push_str("\n\n### Domain Model\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if let Some(doc_ref) = requirements_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_CONSTRAINTS_FILE)
            {
                section.push_str("\n\n### Constraints\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if has_content { Some(section) } else { None }
    }

    fn render_backlog_upstream_evidence(
        &self,
        accumulated_context: &[crate::domain::governance::GovernedDocumentRef],
    ) -> Option<String> {
        let architecture_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Architecture);
        let requirements_ref =
            accumulated_context.iter().find(|doc| doc.canon_mode == CanonMode::Requirements);

        let mut section = String::new();
        let mut has_content = false;

        if let Some(doc_ref) = architecture_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_ARCHITECTURE_DECISIONS_FILE)
            {
                section.push_str("\n\n## Planning Scope\n\n### Architecture Decisions\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if let Some(doc_ref) = requirements_ref {
            let packet_dir = self.workspace_ref.join(&doc_ref.packet_ref);
            if let Some(content) = read_upstream_artifact_capped(&packet_dir, UPSTREAM_PRD_FILE) {
                let heading = if has_content {
                    "\n\n### Product Scope\n\n"
                } else {
                    "\n\n## Planning Scope\n\n### Product Scope\n\n"
                };
                section.push_str(heading);
                section.push_str(&content);
                has_content = true;
            }
            if let Some(content) =
                read_upstream_artifact_capped(&packet_dir, UPSTREAM_SCOPE_CUTS_FILE)
            {
                section.push_str("\n\n### Scope Cuts\n\n");
                section.push_str(&content);
                has_content = true;
            }
        }

        if has_content { Some(section) } else { None }
    }

    pub(super) fn execute_discovery_stage_council(
        &self,
        request: &StageCouncilRequest,
    ) -> Result<StageCouncilOutcome, SessionRuntimeError> {
        let current_artifact_ref = request.current_artifact_ref.as_ref().ok_or_else(|| {
            SessionRuntimeError::ExecutionInvariant(
                "stage council requires current_artifact_ref for discovery planning".to_string(),
            )
        })?;
        let current_artifact_path = self.workspace_ref.join(current_artifact_ref);
        let current_artifact = fs::read_to_string(&current_artifact_path).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to read discovery stage artifact {}: {error}",
                current_artifact_path.display()
            ))
        })?;
        let producer_ref = self.write_stage_council_artifact(
            request,
            STAGE_COUNCIL_PRODUCER_ARTIFACT_SUFFIX,
            &current_artifact,
        )?;
        let producer_output = StageCouncilArtifact {
            route_slot: request.producer_slot.clone(),
            evidence_ref: producer_ref.clone(),
            summary: Some("planner produced the discovery artifact for council review".to_string()),
        };
        let routing = self.planning_council_effective_routing();
        let reviewer_routes = discovery_stage_council_reviewers(&routing);
        let reviewers =
            reviewer_routes.iter().map(|route| route.reviewer.clone()).collect::<Vec<_>>();
        let participants = reviewer_routes
            .iter()
            .map(|route| {
                let available = route_is_available(&route.route);
                ReviewerParticipation {
                    reviewer_id: route.reviewer.reviewer_id.clone(),
                    status: if available {
                        ReviewerParticipationStatus::Completed
                    } else {
                        ReviewerParticipationStatus::Omitted
                    },
                    reason: (!available).then(|| {
                        format!(
                            "route {} is unavailable for provider-backed council review",
                            route
                                .reviewer
                                .source
                                .clone()
                                .unwrap_or_else(|| model_route_label(&route.route))
                        )
                    }),
                    effective_route: route.reviewer.source.clone(),
                }
            })
            .collect::<Vec<_>>();

        if let Err(error) =
            resolve_council_assembly(CouncilProfile::YellowPair, &reviewers, &participants)
        {
            return self.stage_council_blocked_outcome(
                request,
                &producer_output,
                &error.to_string(),
                "configure distinct provider-backed reviewer routes before rerunning boundline plan",
            );
        }

        let artifact_file = ProviderWorkspaceFile {
            path: producer_ref.clone(),
            contents: current_artifact.clone(),
        };
        let prior_context = json!({
            "stage_key": request.stage_key,
            "target_refs": request.target_refs,
            "constraints": request.constraints,
            "current_artifact_ref": current_artifact_ref,
        });
        let mut effective_routes = BTreeMap::new();
        let mut review_findings = Vec::new();
        let mut stage_findings = Vec::new();

        for reviewer_route in &reviewer_routes {
            let effective_route = reviewer_route
                .reviewer
                .source
                .clone()
                .unwrap_or_else(|| model_route_label(&reviewer_route.route));
            effective_routes
                .insert(reviewer_route.reviewer.reviewer_id.clone(), effective_route.clone());
            let response = match review_workspace(
                &reviewer_route.route,
                &ProviderReviewRequest {
                    goal: request.goal.clone(),
                    phase: request.phase.clone(),
                    reviewer_id: reviewer_route.reviewer.reviewer_id.clone(),
                    reviewer_role: reviewer_route.reviewer.role.clone(),
                    attempt_id: format!(
                        "{}-{}",
                        request.stage_key.replace(':', "-"),
                        reviewer_route.reviewer.reviewer_id
                    ),
                    files: vec![artifact_file.clone()],
                    prior_context: prior_context.clone(),
                },
            ) {
                Ok(response) => response,
                Err(error) => {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        &format!(
                            "reviewer {} failed: {error}",
                            reviewer_route.reviewer.reviewer_id
                        ),
                        "restore provider review availability before rerunning boundline plan",
                    );
                }
            };

            let mut finding = ReviewerFinding::new(
                reviewer_route.reviewer.reviewer_id.clone(),
                reviewer_disposition_from_provider(response.disposition),
                response.summary.clone(),
            );
            finding.details = response.details.clone();
            finding.runtime_role = Some(reviewer_route.reviewer.role.clone());
            finding.required_action = response.required_action.clone();
            finding.evidence_refs = if response.evidence_refs.is_empty() {
                vec![producer_ref.clone()]
            } else {
                response.evidence_refs.clone()
            };
            review_findings.push(finding);

            stage_findings.push(StageCouncilFinding {
                reviewer_id: reviewer_route.reviewer.reviewer_id.clone(),
                effective_route,
                disposition: stage_council_disposition_from_provider(response.disposition),
                summary: response.summary,
                accepted: false,
            });
        }

        let vote_resolution = VoteRuleDefinition::default()
            .resolve(&reviewers, &review_findings, Some(&effective_routes))
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let mut accepted_findings = stage_findings
            .iter()
            .filter(|finding| finding.disposition != StageCouncilFindingDisposition::Approve)
            .map(|finding| finding.reviewer_id.clone())
            .collect::<Vec<_>>();
        let mut rejected_findings = stage_findings
            .iter()
            .filter(|finding| finding.disposition == StageCouncilFindingDisposition::Approve)
            .map(|finding| finding.reviewer_id.clone())
            .collect::<Vec<_>>();
        let mut adjudication = None;
        let mut blocking = stage_findings
            .iter()
            .any(|finding| finding.disposition == StageCouncilFindingDisposition::Block)
            || vote_resolution.decision == VoteDecision::Rejected;

        if vote_resolution.decision == VoteDecision::NeedsAdjudication {
            if !route_is_available(&routing.adjudication.route) {
                return self.stage_council_blocked_outcome(
                    request,
                    &producer_output,
                    "adjudication was required but the adjudication route is unavailable",
                    "configure an adjudication route before rerunning boundline plan",
                );
            }

            let adjudication_response = match review_workspace(
                &routing.adjudication.route,
                &ProviderReviewRequest {
                    goal: request.goal.clone(),
                    phase: format!("{}-adjudication", request.phase),
                    reviewer_id: "arbiter".to_string(),
                    reviewer_role: "discovery adjudicator".to_string(),
                    attempt_id: format!("{}-arbiter", request.stage_key.replace(':', "-")),
                    files: vec![artifact_file.clone()],
                    prior_context: json!({
                        "review_findings": review_findings.clone(),
                        "stage_findings": stage_findings.clone(),
                    }),
                },
            ) {
                Ok(response) => response,
                Err(error) => {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        &format!("adjudication failed: {error}"),
                        "restore adjudication availability before rerunning boundline plan",
                    );
                }
            };

            adjudication = Some(StageCouncilAdjudication {
                adjudicator_route: model_route_label(&routing.adjudication.route),
                decision: provider_review_disposition_text(adjudication_response.disposition)
                    .to_string(),
                rationale: adjudication_response.summary.clone(),
            });

            match adjudication_response.disposition {
                ProviderReviewDisposition::Approve => {
                    accepted_findings.clear();
                    rejected_findings = stage_findings
                        .iter()
                        .filter(|finding| {
                            finding.disposition != StageCouncilFindingDisposition::Approve
                        })
                        .map(|finding| finding.reviewer_id.clone())
                        .collect();
                    blocking = false;
                }
                ProviderReviewDisposition::Concern => {
                    blocking = false;
                }
                ProviderReviewDisposition::Block => {
                    blocking = true;
                }
            }
        }

        for finding in &mut stage_findings {
            finding.accepted = accepted_findings.contains(&finding.reviewer_id);
        }

        let mut revised_summary = Some(
            "reviser preserved the producer artifact because no council findings were accepted"
                .to_string(),
        );
        let revised_artifact_text = if blocking {
            revised_summary = Some("stage council blocked planning discovery".to_string());
            render_stage_council_blocked_markdown(request, &stage_findings, &accepted_findings)
        } else {
            let accepted_feedback = stage_findings
                .iter()
                .filter(|finding| finding.accepted)
                .map(|finding| format!("{}: {}", finding.reviewer_id, finding.summary))
                .collect::<Vec<_>>();
            if accepted_feedback.is_empty() {
                current_artifact.clone()
            } else {
                if !route_is_available(&routing.planning.route) {
                    return self.stage_council_blocked_outcome(
                        request,
                        &producer_output,
                        "reviser route is unavailable for provider-backed council revision",
                        "configure a planning route before rerunning boundline plan",
                    );
                }
                match revise_artifact(
                    &routing.planning.route,
                    &ProviderRevisionRequest {
                        goal: request.goal.clone(),
                        phase: request.phase.clone(),
                        reviser_id: "reviser".to_string(),
                        target_refs: request.target_refs.clone(),
                        current_artifact: current_artifact.clone(),
                        accepted_feedback,
                        prior_context: json!({
                            "review_findings": stage_findings.clone(),
                            "adjudication": adjudication.clone(),
                        }),
                    },
                ) {
                    Ok(response) => {
                        revised_summary = Some(response.summary);
                        response.revised_artifact
                    }
                    Err(error) => {
                        return self.stage_council_blocked_outcome(
                            request,
                            &producer_output,
                            &format!("reviser failed: {error}"),
                            "restore revision availability before rerunning boundline plan",
                        );
                    }
                }
            }
        };

        let revised_ref = self.write_stage_council_artifact(
            request,
            STAGE_COUNCIL_REVISED_ARTIFACT_SUFFIX,
            &revised_artifact_text,
        )?;
        let outcome = StageCouncilOutcome {
            producer_output,
            reviewer_findings: stage_findings,
            vote_resolution: StageCouncilVoteResolution {
                strategy: STAGE_COUNCIL_VOTE_STRATEGY_BOUNDED_MAJORITY.to_string(),
                accepted_findings,
                rejected_findings,
                independent_review: true,
            },
            adjudication,
            revised_output: StageCouncilArtifact {
                route_slot: request.producer_slot.clone(),
                evidence_ref: revised_ref,
                summary: revised_summary,
            },
            status: if blocking {
                StageCouncilStatus::Blocked
            } else {
                StageCouncilStatus::Proceed
            },
            next_action: if blocking {
                "repair discovery inputs and rerun boundline plan".to_string()
            } else {
                "continue planning discovery".to_string()
            },
        };
        outcome.validate().map_err(SessionRuntimeError::ExecutionInvariant)?;
        Ok(outcome)
    }
}

pub(super) fn plan_stage_failure_from_execute_response(
    run_id: Uuid,
    adapter_id: String,
    response: crate::adapters::FrameworkAdapterExecuteStageResponse,
) -> FrameworkAdapterStageFailureDetails {
    let finished_at = current_timestamp_millis();
    let failure_class = response
        .failure_class
        .map(map_framework_adapter_failure_class)
        .or(Some(AdapterFailureClass::AdapterRuntime));
    let response_details = framework_adapter_stage_outcome_details_from_response(&response);
    let status = match response.status {
        crate::adapters::FrameworkAdapterStageExecutionStatus::Succeeded => {
            LifecycleStageExecutionStatus::Succeeded
        }
        crate::adapters::FrameworkAdapterStageExecutionStatus::Blocked => {
            LifecycleStageExecutionStatus::Blocked
        }
        crate::adapters::FrameworkAdapterStageExecutionStatus::Failed => {
            LifecycleStageExecutionStatus::Failed
        }
    };

    FrameworkAdapterStageFailureDetails {
        execution: LifecycleStageExecutionRecord {
            run_id: run_id.to_string(),
            stage_key: AdapterLifecycleStageKey::Plan,
            execution_source: AdapterExecutionSource::Adapter,
            adapter_id: Some(adapter_id),
            status,
            intervention_required: true,
            failure_class,
            produced_artifacts: response.produced_artifacts,
            details: response_details,
            started_at: Some(finished_at),
            finished_at: Some(finished_at),
        },
        claim_state: StageClaimState::FailedAfterClaim,
        summary: response.summary,
        detail: None,
        protocol_error_code: None,
    }
}

pub(super) fn plan_stage_failure_from_host_error(
    run_id: Uuid,
    adapter_id: String,
    error: FrameworkAdapterHostError,
) -> FrameworkAdapterStageFailureDetails {
    let (failure_class, protocol_error_code) = match &error {
        FrameworkAdapterHostError::DeserializeResponse { .. }
        | FrameworkAdapterHostError::InvalidEnvelope { .. }
        | FrameworkAdapterHostError::ProtocolError { .. } => {
            (AdapterFailureClass::ProtocolError, protocol_error_code_from_host_error(&error))
        }
        FrameworkAdapterHostError::EmptyCommand
        | FrameworkAdapterHostError::SerializeRequest { .. }
        | FrameworkAdapterHostError::Spawn { .. }
        | FrameworkAdapterHostError::WriteRequest { .. }
        | FrameworkAdapterHostError::Wait { .. }
        | FrameworkAdapterHostError::ProcessFailed { .. } => {
            (AdapterFailureClass::TransportFailure, None)
        }
    };
    let summary = match failure_class {
        AdapterFailureClass::ProtocolError => {
            let code_suffix = protocol_error_code
                .as_deref()
                .map(|code| format!(" code={code}"))
                .unwrap_or_default();
            format!(
                "framework-adapter returned a protocol error after claiming plan stage{code_suffix}"
            )
        }
        AdapterFailureClass::TransportFailure => {
            "framework-adapter transport failed after claiming plan stage".to_string()
        }
        _ => "framework-adapter plan stage failed after claim".to_string(),
    };
    let finished_at = current_timestamp_millis();

    FrameworkAdapterStageFailureDetails {
        execution: LifecycleStageExecutionRecord {
            run_id: run_id.to_string(),
            stage_key: AdapterLifecycleStageKey::Plan,
            execution_source: AdapterExecutionSource::Adapter,
            adapter_id: Some(adapter_id),
            status: LifecycleStageExecutionStatus::Failed,
            intervention_required: true,
            failure_class: Some(failure_class),
            produced_artifacts: Vec::new(),
            details: None,
            started_at: Some(finished_at),
            finished_at: Some(finished_at),
        },
        claim_state: StageClaimState::FailedAfterClaim,
        summary,
        detail: Some(error.to_string()),
        protocol_error_code,
    }
}

pub(super) fn plan_stage_routing_record_from_failure(
    failure: &FrameworkAdapterStageFailureDetails,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: failure.execution.run_id.clone(),
        stage_key: failure.execution.stage_key,
        execution_source: failure.execution.execution_source,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state: failure.claim_state,
        adapter_id: failure.execution.adapter_id.clone(),
        stage_status: Some(failure.execution.status),
        produced_artifacts: failure.execution.produced_artifacts.clone(),
        details: failure.execution.details.clone(),
        recorded_at: current_timestamp_millis(),
    }
}

pub(super) fn plan_stage_routing_record_from_success(
    run_id: Uuid,
    adapter_id: String,
    produced_artifacts: Vec<String>,
    details: Option<crate::domain::framework_adapter::FrameworkAdapterStageOutcomeDetails>,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: run_id.to_string(),
        stage_key: AdapterLifecycleStageKey::Plan,
        execution_source: AdapterExecutionSource::Adapter,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state: StageClaimState::Completed,
        adapter_id: Some(adapter_id),
        stage_status: Some(LifecycleStageExecutionStatus::Succeeded),
        produced_artifacts,
        details,
        recorded_at: current_timestamp_millis(),
    }
}

pub(super) fn plan_stage_routing_record_from_blocked(
    blocked: &FrameworkAdapterStageFailureDetails,
) -> StageRoutingDecisionRecord {
    StageRoutingDecisionRecord {
        run_id: blocked.execution.run_id.clone(),
        stage_key: blocked.execution.stage_key,
        execution_source: blocked.execution.execution_source,
        decision_reason: StageRoutingDecisionReason::DeclaredOverride,
        claim_state: blocked.claim_state,
        adapter_id: blocked.execution.adapter_id.clone(),
        stage_status: Some(blocked.execution.status),
        produced_artifacts: blocked.execution.produced_artifacts.clone(),
        details: blocked.execution.details.clone(),
        recorded_at: current_timestamp_millis(),
    }
}

pub(super) fn plan_stage_blocked_from_execute_response(
    run_id: Uuid,
    adapter_id: String,
    response: crate::adapters::FrameworkAdapterExecuteStageResponse,
) -> FrameworkAdapterStageFailureDetails {
    let finished_at = current_timestamp_millis();
    let response_details = framework_adapter_stage_outcome_details_from_response(&response);

    FrameworkAdapterStageFailureDetails {
        execution: LifecycleStageExecutionRecord {
            run_id: run_id.to_string(),
            stage_key: AdapterLifecycleStageKey::Plan,
            execution_source: AdapterExecutionSource::Adapter,
            adapter_id: Some(adapter_id),
            status: LifecycleStageExecutionStatus::Blocked,
            intervention_required: true,
            failure_class: None,
            produced_artifacts: response.produced_artifacts,
            details: response_details,
            started_at: Some(finished_at),
            finished_at: Some(finished_at),
        },
        claim_state: StageClaimState::Claimed,
        summary: response.summary,
        detail: response.next_action,
        protocol_error_code: None,
    }
}

pub(super) fn append_adapter_fallback_reason(goal_plan: &mut GoalPlan, reason: &str) {
    let note = format!("adapter_fallback_reason: {reason}");
    goal_plan.planning_rationale = Some(match goal_plan.planning_rationale.take() {
        Some(existing) if existing.contains(&note) => existing,
        Some(existing) => format!("{existing}; {note}"),
        None => note,
    });
}

pub(super) fn framework_adapter_config_values(
    selection: &crate::domain::configuration::PersistedAdapterConfiguration,
) -> Vec<crate::adapters::FrameworkAdapterConfigValue> {
    selection
        .values
        .iter()
        .map(|value| crate::adapters::FrameworkAdapterConfigValue {
            field_key: value.field_key.clone(),
            value_kind: match value.value_kind {
                crate::domain::framework_adapter::AdapterValueKind::String => {
                    crate::adapters::FrameworkAdapterFieldValueKind::String
                }
                crate::domain::framework_adapter::AdapterValueKind::Path => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Path
                }
                crate::domain::framework_adapter::AdapterValueKind::Boolean => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Boolean
                }
                crate::domain::framework_adapter::AdapterValueKind::Integer => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Integer
                }
                crate::domain::framework_adapter::AdapterValueKind::Enum => {
                    crate::adapters::FrameworkAdapterFieldValueKind::Enum
                }
            },
            string_value: value.string_value.clone(),
            path_value: value.path_value.clone(),
            bool_value: value.bool_value,
            int_value: value.int_value,
        })
        .collect()
}

#[cfg(all(test, unix))]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use crate::adapters::config_store::FileConfigStore;
    use crate::adapters::trace_store::TraceStore;
    use crate::adapters::{
        FrameworkAdapterDescribeResponse, FrameworkAdapterExecuteStageResponse,
        FrameworkAdapterPreflightResponse, FrameworkAdapterStageExecutionStatus,
    };
    use crate::domain::brief::normalize_inputs;
    use crate::domain::configuration::{
        AdapterSelectionRecord, ConfigFile, PersistedAdapterConfiguration,
    };
    use crate::domain::framework_adapter::{
        AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterFailureClass,
        AdapterRegistrationSource, AdapterSelectionMode, LifecycleStageExecutionStatus,
        StageClaimState, StageRoutingDecisionReason,
    };
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::trace::TraceEventType;
    use crate::fixture::{
        sample_framework_adapter_describe_response,
        sample_framework_adapter_execute_stage_failed_response,
        sample_framework_adapter_execute_stage_success_response,
        sample_framework_adapter_preflight_blocked_response,
        sample_framework_adapter_preflight_ready_response,
        sample_framework_adapter_success_envelope,
    };
    use crate::orchestrator::{FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, FrameworkStageKey};

    use super::{
        ADAPTER_FALLBACK_REASON_PREFLIGHT_BLOCKED, ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY,
        ADAPTER_FALLBACK_REASON_UNSUPPORTED_TRANSPORT, FrameworkAdapterPlanStageOutcome,
        SessionRuntime,
    };

    const ADAPTER_COMMAND_MISSING: &str = "definitely-missing-boundline-adapter";
    const ADAPTER_DISPLAY_NAME: &str = "Speckit";
    const ADAPTER_ID: &str = "speckit";
    const EXECUTE_RESPONSE_FILE_NAME: &str = "execute-stage-response.json";
    const FRAMEWORK_ADAPTER_SCRIPT_FILE_NAME: &str = "framework-adapter.sh";
    const GOAL_TEXT: &str = "Drive the planning runtime";
    const PLANNED_TASK_DESCRIPTION: &str = "Repair arithmetic";
    const PLANNED_TASK_ID: &str = "planned-task-1";
    const PLANNED_TASK_TARGET: &str = "src/lib.rs";
    const PREFLIGHT_RESPONSE_FILE_NAME: &str = "preflight-response.json";
    const SCHEMA_FINGERPRINT: &str = "schema-v1";
    const SUCCESS_ARTIFACT: &str = "specs/066-agentic-framework-integration/plan.md";
    const UPDATED_AT: u64 = 42;

    #[test]
    fn planning_runtime_framework_adapter_not_claimed_paths_cover_binding_and_preflight()
    -> Result<(), Box<dyn Error>> {
        let missing_selection_workspace = temp_workspace("boundline-plan-stage-no-selection")?;
        let missing_selection_runtime =
            SessionRuntime::for_workspace(missing_selection_workspace.as_path());
        let mut missing_selection_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            missing_selection_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut missing_selection_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert_eq!(missing_selection_goal_plan.planning_rationale, None);

        let missing_binary_workspace = temp_workspace("boundline-plan-stage-missing-binary")?;
        save_local_adapter(
            missing_binary_workspace.as_path(),
            sample_adapter_selection(ADAPTER_COMMAND_MISSING),
        )?;
        let missing_binary_runtime =
            SessionRuntime::for_workspace(missing_binary_workspace.as_path());
        let mut missing_binary_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            missing_binary_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut missing_binary_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert!(has_fallback_reason(
            &missing_binary_goal_plan,
            ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY,
        ));

        let undeclared_workspace = temp_workspace("boundline-plan-stage-undeclared")?;
        let mut undeclared_describe = sample_framework_adapter_describe_response();
        undeclared_describe
            .declared_stage_overrides
            .retain(|stage| *stage != FrameworkStageKey::Plan);
        let undeclared_script = write_framework_adapter_script(
            undeclared_workspace.as_path(),
            &undeclared_describe,
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            undeclared_workspace.as_path(),
            sample_adapter_selection(undeclared_script.as_str()),
        )?;
        let undeclared_runtime = SessionRuntime::for_workspace(undeclared_workspace.as_path());
        let mut undeclared_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            undeclared_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut undeclared_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert_eq!(undeclared_goal_plan.planning_rationale, None);

        let unsupported_workspace = temp_workspace("boundline-plan-stage-unsupported-transport")?;
        let mut unsupported_describe = sample_framework_adapter_describe_response();
        unsupported_describe.supported_transports = Vec::new();
        let unsupported_script = write_framework_adapter_script(
            unsupported_workspace.as_path(),
            &unsupported_describe,
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            unsupported_workspace.as_path(),
            sample_adapter_selection(unsupported_script.as_str()),
        )?;
        let unsupported_runtime = SessionRuntime::for_workspace(unsupported_workspace.as_path());
        let mut unsupported_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            unsupported_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut unsupported_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert!(has_fallback_reason(
            &unsupported_goal_plan,
            ADAPTER_FALLBACK_REASON_UNSUPPORTED_TRANSPORT,
        ));

        let preflight_failure_workspace = temp_workspace("boundline-plan-stage-preflight-failure")?;
        let preflight_failure_script = write_framework_adapter_script(
            preflight_failure_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::ProcessFailure,
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            preflight_failure_workspace.as_path(),
            sample_adapter_selection(preflight_failure_script.as_str()),
        )?;
        let preflight_failure_runtime =
            SessionRuntime::for_workspace(preflight_failure_workspace.as_path());
        let mut preflight_failure_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            preflight_failure_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut preflight_failure_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert!(has_fallback_reason(
            &preflight_failure_goal_plan,
            ADAPTER_FALLBACK_REASON_UNAVAILABLE_BINARY,
        ));

        let preflight_blocked_workspace = temp_workspace("boundline-plan-stage-preflight-blocked")?;
        let preflight_blocked_script = write_framework_adapter_script(
            preflight_blocked_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_blocked_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            preflight_blocked_workspace.as_path(),
            sample_adapter_selection(preflight_blocked_script.as_str()),
        )?;
        let preflight_blocked_runtime =
            SessionRuntime::for_workspace(preflight_blocked_workspace.as_path());
        let mut preflight_blocked_goal_plan = sample_goal_plan()?;
        assert!(matches!(
            preflight_blocked_runtime
                .maybe_apply_framework_adapter_plan_stage(&mut preflight_blocked_goal_plan)?,
            FrameworkAdapterPlanStageOutcome::NotClaimed
        ));
        assert!(has_fallback_reason(
            &preflight_blocked_goal_plan,
            ADAPTER_FALLBACK_REASON_PREFLIGHT_BLOCKED,
        ));

        Ok(())
    }

    #[test]
    fn planning_runtime_framework_adapter_claimed_paths_cover_execute_outcomes()
    -> Result<(), Box<dyn Error>> {
        let success_workspace = temp_workspace("boundline-plan-stage-claimed-success")?;
        let success_script = write_framework_adapter_script(
            success_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            success_workspace.as_path(),
            sample_adapter_selection(success_script.as_str()),
        )?;
        let success_runtime = SessionRuntime::for_workspace(success_workspace.as_path());
        let mut success_goal_plan = sample_goal_plan()?;
        match success_runtime.maybe_apply_framework_adapter_plan_stage(&mut success_goal_plan)? {
            FrameworkAdapterPlanStageOutcome::ClaimedSucceeded(routing_record) => {
                assert_eq!(routing_record.claim_state, StageClaimState::Completed);
                assert_eq!(
                    routing_record.stage_status,
                    Some(LifecycleStageExecutionStatus::Succeeded)
                );
                assert_eq!(
                    routing_record.decision_reason,
                    StageRoutingDecisionReason::DeclaredOverride
                );
                assert_eq!(
                    routing_record.produced_artifacts,
                    vec![
                        SUCCESS_ARTIFACT.to_string(),
                        "specs/066-agentic-framework-integration/tasks.md".to_string()
                    ]
                );
            }
            _ => return Err("expected claimed success outcome".into()),
        }

        let blocked_workspace = temp_workspace("boundline-plan-stage-claimed-blocked")?;
        let blocked_script = write_framework_adapter_script(
            blocked_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(blocked_execute_stage_response()),
        )?;
        save_local_adapter(
            blocked_workspace.as_path(),
            sample_adapter_selection(blocked_script.as_str()),
        )?;
        let blocked_runtime = SessionRuntime::for_workspace(blocked_workspace.as_path());
        let mut blocked_goal_plan = sample_goal_plan()?;
        match blocked_runtime.maybe_apply_framework_adapter_plan_stage(&mut blocked_goal_plan)? {
            FrameworkAdapterPlanStageOutcome::ClaimedBlocked(blocked) => {
                assert_eq!(blocked.claim_state, StageClaimState::Claimed);
                assert_eq!(blocked.execution.status, LifecycleStageExecutionStatus::Blocked);
                assert_eq!(blocked.detail.as_deref(), Some("resume planning"));
                let details =
                    blocked.execution.details.as_ref().ok_or("missing blocked details")?;
                assert_eq!(details.workflow_id.as_deref(), Some("speckit-planning"));
                assert_eq!(
                    details.final_planning_readiness_status,
                    Some(crate::domain::framework_adapter::PlanningReadinessStatus::Blocked)
                );
                assert_eq!(details.analyze_pass_count, Some(2));
                assert_eq!(details.remediation_cycles_used, Some(1));
                assert_eq!(details.remaining_blocking_findings.len(), 1);
            }
            _ => return Err("expected claimed blocked outcome".into()),
        }

        let failed_workspace = temp_workspace("boundline-plan-stage-claimed-failed")?;
        let failed_script = write_framework_adapter_script(
            failed_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_failed_response()),
        )?;
        save_local_adapter(
            failed_workspace.as_path(),
            sample_adapter_selection(failed_script.as_str()),
        )?;
        let failed_runtime = SessionRuntime::for_workspace(failed_workspace.as_path());
        let mut failed_goal_plan = sample_goal_plan()?;
        match failed_runtime.maybe_apply_framework_adapter_plan_stage(&mut failed_goal_plan)? {
            FrameworkAdapterPlanStageOutcome::ClaimedFailed(failure) => {
                assert_eq!(failure.claim_state, StageClaimState::FailedAfterClaim);
                assert_eq!(failure.execution.status, LifecycleStageExecutionStatus::Failed);
                assert_eq!(
                    failure.execution.failure_class,
                    Some(AdapterFailureClass::AdapterRuntime)
                );
                assert_eq!(failure.detail, None);
            }
            _ => return Err("expected claimed failed outcome".into()),
        }

        let transport_failure_workspace =
            temp_workspace("boundline-plan-stage-claimed-transport-failure")?;
        let transport_failure_script = write_framework_adapter_script(
            transport_failure_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::ProcessFailure,
        )?;
        save_local_adapter(
            transport_failure_workspace.as_path(),
            sample_adapter_selection(transport_failure_script.as_str()),
        )?;
        let transport_failure_runtime =
            SessionRuntime::for_workspace(transport_failure_workspace.as_path());
        let mut transport_failure_goal_plan = sample_goal_plan()?;
        match transport_failure_runtime
            .maybe_apply_framework_adapter_plan_stage(&mut transport_failure_goal_plan)?
        {
            FrameworkAdapterPlanStageOutcome::ClaimedFailed(failure) => {
                assert_eq!(failure.claim_state, StageClaimState::FailedAfterClaim);
                assert_eq!(
                    failure.execution.failure_class,
                    Some(AdapterFailureClass::TransportFailure)
                );
                assert!(failure.summary.contains("transport failed"));
            }
            _ => return Err("expected claimed transport-failure outcome".into()),
        }

        Ok(())
    }

    #[test]
    fn planning_runtime_plan_task_framework_adapter_persists_success_and_blocked_traces()
    -> Result<(), Box<dyn Error>> {
        let success_workspace = temp_workspace("boundline-plan-task-adapter-success")?;
        seed_planning_workspace(success_workspace.as_path())?;
        let success_script = write_framework_adapter_script(
            success_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_success_response()),
        )?;
        save_local_adapter(
            success_workspace.as_path(),
            sample_adapter_selection(success_script.as_str()),
        )?;
        let success_runtime = SessionRuntime::for_workspace(success_workspace.as_path());
        let mut success_session = sample_planning_session(success_workspace.as_path());

        success_runtime.capture_goal(&mut success_session, GOAL_TEXT)?;
        success_session.authored_brief = Some(sample_authored_brief(success_workspace.as_path())?);
        success_runtime.select_flow(&mut success_session, "bug-fix")?;
        success_runtime.plan_task(&mut success_session, None, false)?;

        assert_eq!(success_session.latest_status, SessionStatus::Planned);
        assert!(success_session.latest_terminal_reason.is_none());
        assert!(
            success_session.goal_plan.as_ref().is_some_and(|plan| !plan.requires_confirmation())
        );
        let success_trace_ref =
            success_session.latest_trace_ref.as_deref().ok_or("missing success trace ref")?;
        let success_trace = success_runtime.trace_store().load(Path::new(success_trace_ref))?;
        let success_stage_routed = success_trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::StageRouted)
            .ok_or("missing success stage routed event")?;
        assert_eq!(
            success_stage_routed.payload["framework_adapter_stage_routing"]["stage_status"],
            serde_json::json!("succeeded")
        );

        let blocked_workspace = temp_workspace("boundline-plan-task-adapter-blocked")?;
        seed_planning_workspace(blocked_workspace.as_path())?;
        let blocked_script = write_framework_adapter_script(
            blocked_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(blocked_execute_stage_response()),
        )?;
        save_local_adapter(
            blocked_workspace.as_path(),
            sample_adapter_selection(blocked_script.as_str()),
        )?;
        let blocked_runtime = SessionRuntime::for_workspace(blocked_workspace.as_path());
        let mut blocked_session = sample_planning_session(blocked_workspace.as_path());

        blocked_runtime.capture_goal(&mut blocked_session, GOAL_TEXT)?;
        blocked_session.authored_brief = Some(sample_authored_brief(blocked_workspace.as_path())?);
        blocked_runtime.select_flow(&mut blocked_session, "bug-fix")?;
        blocked_runtime.plan_task(&mut blocked_session, None, false)?;

        assert_eq!(blocked_session.latest_status, SessionStatus::Blocked);
        assert_eq!(
            blocked_session.latest_terminal_reason.as_ref().map(|reason| reason.condition),
            Some(TerminalCondition::NoCredibleNextStep)
        );
        let blocked_trace_ref =
            blocked_session.latest_trace_ref.as_deref().ok_or("missing blocked trace ref")?;
        let blocked_trace = blocked_runtime.trace_store().load(Path::new(blocked_trace_ref))?;
        let blocked_stage_routed = blocked_trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::StageRouted)
            .ok_or("missing blocked stage routed event")?;
        assert_eq!(
            blocked_stage_routed.payload["framework_adapter_stage_routing"]["stage_status"],
            serde_json::json!("blocked")
        );

        Ok(())
    }

    #[test]
    fn planning_runtime_plan_task_framework_adapter_persists_failed_trace_and_status()
    -> Result<(), Box<dyn Error>> {
        let failed_workspace = temp_workspace("boundline-plan-task-adapter-failed")?;
        seed_planning_workspace(failed_workspace.as_path())?;
        let failed_script = write_framework_adapter_script(
            failed_workspace.as_path(),
            &sample_framework_adapter_describe_response(),
            PreflightMode::Response(sample_framework_adapter_preflight_ready_response()),
            ExecuteMode::Response(sample_framework_adapter_execute_stage_failed_response()),
        )?;
        save_local_adapter(
            failed_workspace.as_path(),
            sample_adapter_selection(failed_script.as_str()),
        )?;
        let failed_runtime = SessionRuntime::for_workspace(failed_workspace.as_path());
        let mut failed_session = sample_planning_session(failed_workspace.as_path());

        failed_runtime.capture_goal(&mut failed_session, GOAL_TEXT)?;
        failed_session.authored_brief = Some(sample_authored_brief(failed_workspace.as_path())?);
        failed_runtime.select_flow(&mut failed_session, "bug-fix")?;
        let error = failed_runtime.plan_task(&mut failed_session, None, false).unwrap_err();
        assert!(
            error.to_string().contains("framework-adapter plan stage execution failed after claim")
        );

        assert_eq!(failed_session.latest_status, SessionStatus::Failed);
        assert_eq!(
            failed_session.latest_terminal_reason.as_ref().map(|reason| reason.condition),
            Some(TerminalCondition::TaskNotCredible)
        );
        let failed_trace_ref =
            failed_session.latest_trace_ref.as_deref().ok_or("missing failed trace ref")?;
        let failed_trace = failed_runtime.trace_store().load(Path::new(failed_trace_ref))?;
        let failed_stage = failed_trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::StageFailed)
            .ok_or("missing failed stage event")?;
        assert_eq!(
            failed_stage.payload["framework_adapter_stage_failure"]["summary"],
            serde_json::json!("Speckit could not complete the claimed stage")
        );
        let terminal_event = failed_trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::TerminalRecorded)
            .ok_or("missing terminal event")?;
        assert_eq!(terminal_event.payload["terminal_status"], serde_json::json!("failed"));

        Ok(())
    }

    #[derive(Clone)]
    enum PreflightMode {
        Response(FrameworkAdapterPreflightResponse),
        ProcessFailure,
    }

    #[derive(Clone)]
    #[allow(clippy::large_enum_variant)]
    enum ExecuteMode {
        Response(FrameworkAdapterExecuteStageResponse),
        ProcessFailure,
    }

    fn blocked_execute_stage_response() -> FrameworkAdapterExecuteStageResponse {
        FrameworkAdapterExecuteStageResponse {
            status: FrameworkAdapterStageExecutionStatus::Blocked,
            summary: "adapter blocked planning".to_string(),
            produced_artifacts: vec![SUCCESS_ARTIFACT.to_string()],
            workflow_id: Some("speckit-planning".to_string()),
            executed_commands: vec!["speckit.analyze".to_string()],
            planning_findings: vec![crate::adapters::FrameworkAdapterPlanningFinding {
                finding_id: "F-001".to_string(),
                summary: "Blocking planning finding".to_string(),
                severity: crate::adapters::FrameworkAdapterPlanningFindingSeverity::Blocking,
            }],
            remediation_tasks_attempted: vec![
                crate::adapters::FrameworkAdapterPlanningRemediationTaskOutcome {
                    task_id: "R-001".to_string(),
                    summary: "Attempt remediation".to_string(),
                    finding_ids: vec!["F-001".to_string()],
                    skip_reason: None,
                },
            ],
            remediation_tasks_completed: Vec::new(),
            remediation_tasks_skipped: vec![
                crate::adapters::FrameworkAdapterPlanningRemediationTaskOutcome {
                    task_id: "R-002".to_string(),
                    summary: "Needs operator input".to_string(),
                    finding_ids: vec!["F-001".to_string()],
                    skip_reason: Some(
                        crate::adapters::FrameworkAdapterPlanningRemediationSkipReason::RequiresOperatorInput,
                    ),
                },
            ],
            remaining_blocking_findings: vec![crate::adapters::FrameworkAdapterPlanningFinding {
                finding_id: "F-001".to_string(),
                summary: "Blocking planning finding".to_string(),
                severity: crate::adapters::FrameworkAdapterPlanningFindingSeverity::Blocking,
            }],
            final_planning_readiness_status: Some(
                crate::adapters::FrameworkAdapterPlanningReadinessStatus::Blocked,
            ),
            analyze_pass_count: Some(2),
            remediation_cycles_used: Some(1),
            implementation_status: None,
            validation_refs: vec!["specs/066-agentic-framework-integration/analysis.md".to_string()],
            next_action: Some("resume planning".to_string()),
            failure_class: None,
        }
    }

    fn has_fallback_reason(goal_plan: &GoalPlan, reason: &str) -> bool {
        goal_plan.planning_rationale.as_deref().is_some_and(|rationale| {
            rationale.contains(&format!("adapter_fallback_reason: {reason}"))
        })
    }

    fn path_string(path: &Path) -> String {
        PathBuf::from(path).to_string_lossy().into_owned()
    }

    fn sample_planning_session(workspace: &Path) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: format!("session-{}", Uuid::new_v4()),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
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
            delight_feedback: None,
            latest_voting: None,
        }
    }

    fn sample_adapter_selection(command: &str) -> PersistedAdapterConfiguration {
        PersistedAdapterConfiguration {
            selection: AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::KnownProfile,
                adapter_id: ADAPTER_ID.to_string(),
                display_name: ADAPTER_DISPLAY_NAME.to_string(),
                command: command.to_string(),
                args: Vec::new(),
                registration_source: AdapterRegistrationSource::AdapterAdd,
                discovery_state: AdapterDiscoveryState::ExplicitCommand,
                compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                updated_at: UPDATED_AT,
            },
            schema_fingerprint: SCHEMA_FINGERPRINT.to_string(),
            completeness_state: AdapterConfigCompletenessState::Complete,
            interactive_resolution: false,
            last_validated_at: Some(UPDATED_AT),
            value_count: 0,
            values: Vec::new(),
        }
    }

    fn sample_goal_plan() -> Result<GoalPlan, Box<dyn Error>> {
        GoalPlan::new(
            GOAL_TEXT,
            vec![PlannedTask {
                task_id: PLANNED_TASK_ID.to_string(),
                description: PLANNED_TASK_DESCRIPTION.to_string(),
                target: PLANNED_TASK_TARGET.to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .map_err(Into::into)
    }

    fn sample_authored_brief(
        workspace: &Path,
    ) -> Result<crate::domain::brief::AuthoredBriefBundle, Box<dyn Error>> {
        normalize_inputs(workspace, Some(GOAL_TEXT), &[PathBuf::from("brief.md")])
            .map_err(Into::into)
    }

    fn save_local_adapter(
        workspace: &Path,
        adapter: PersistedAdapterConfiguration,
    ) -> Result<(), Box<dyn Error>> {
        FileConfigStore::for_workspace(workspace)
            .save_local(&ConfigFile { adapter: Some(adapter), ..ConfigFile::default() })?;
        Ok(())
    }

    fn seed_planning_workspace(workspace: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(workspace.join("src"))?;
        fs::create_dir_all(workspace.join("tests"))?;
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left - right }\n",
        )?;
        fs::write(
            workspace.join("tests/add.rs"),
            "#[test]\nfn adds() { assert_eq!(2 + 2, 4); }\n",
        )?;
        fs::write(
            workspace.join("brief.md"),
            "Focus on src/lib.rs and tests/add.rs before broad scanning. Repair the arithmetic bug and keep the change narrowly scoped.\n",
        )?;
        Ok(())
    }

    fn temp_workspace(prefix: &str) -> Result<TestWorkspace, Box<dyn Error>> {
        TestWorkspace::new(prefix)
    }

    fn write_framework_adapter_script(
        workspace: &Path,
        describe: &FrameworkAdapterDescribeResponse,
        preflight_mode: PreflightMode,
        execute_mode: ExecuteMode,
    ) -> Result<String, Box<dyn Error>> {
        let describe_path = workspace.join("describe-response.json");
        let preflight_path = workspace.join(PREFLIGHT_RESPONSE_FILE_NAME);
        let execute_path = workspace.join(EXECUTE_RESPONSE_FILE_NAME);
        let script_path = workspace.join(FRAMEWORK_ADAPTER_SCRIPT_FILE_NAME);

        fs::write(
            &describe_path,
            serde_json::to_string(&sample_framework_adapter_success_envelope(describe.clone()))?,
        )?;

        let preflight_block = match preflight_mode {
            PreflightMode::Response(response) => {
                fs::write(
                    &preflight_path,
                    serde_json::to_string(&sample_framework_adapter_success_envelope(response))?,
                )?;
                format!("cat >/dev/null\n  cat '{}'", preflight_path.to_string_lossy())
            }
            PreflightMode::ProcessFailure => {
                "cat >/dev/null\n  echo 'preflight failed' >&2\n  exit 1".to_string()
            }
        };

        let execute_block = match execute_mode {
            ExecuteMode::Response(response) => {
                fs::write(
                    &execute_path,
                    serde_json::to_string(&sample_framework_adapter_success_envelope(response))?,
                )?;
                format!("cat >/dev/null\n  cat '{}'", execute_path.to_string_lossy())
            }
            ExecuteMode::ProcessFailure => {
                "cat >/dev/null\n  echo 'transport failed' >&2\n  exit 1".to_string()
            }
        };

        fs::write(
            &script_path,
            format!(
                "#!/bin/sh\ncase \"$1\" in\ndescribe)\n  cat '{}'\n  ;;\npreflight)\n  {}\n  ;;\nexecute-stage)\n  {}\n  ;;\n*)\n  exit 1\n  ;;\nesac\n",
                describe_path.to_string_lossy(),
                preflight_block,
                execute_block,
            ),
        )?;

        let mut permissions = fs::metadata(&script_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)?;
        Ok(path_string(&script_path))
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
