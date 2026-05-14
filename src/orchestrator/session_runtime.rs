use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::adapters::checkpoint_store::{
    CheckpointCaptureRequest, CheckpointStoreError, FileCheckpointStore,
};
use crate::adapters::governance_runtime::{
    CanonCliRuntime, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeRequest,
    LocalGovernanceRuntime,
};
use serde::Serialize;
use serde_json::{Map, Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::domain::checkpoint::CheckpointAuthorityScope;
use crate::domain::cluster::{
    ClusterDeliveryStory, ClusterRouteOwner, ClusterSessionProjection, ClusteredExecutionCondition,
    ClusteredExecutionKind, WorkspaceParticipationKind, WorkspaceParticipationRecord,
};
use crate::domain::configuration::{
    EffortFallbackPolicy, RouteSlot, RoutingConfig, RoutingOverrides, RuntimeKind,
    resolve_effective_routing, resolve_effective_runtime_capabilities,
    resolve_effective_slot_effort_policies,
};
use crate::domain::decision::Decision;
use crate::domain::flow::{FlowStepMetadata, built_in_flow, supported_flow_names_csv};
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::governance::{
    ApprovalState, CanonEvidenceInspectSummary, CanonPossibleActionSummary,
    CanonRecommendedActionSummary, CompactedCanonMemory, GovernanceLifecycleState,
    GovernanceRuntimeKind, GovernedStageRecord, MemoryCredibilityState, PacketReadiness,
    resolved_canon_mode,
};
use crate::domain::limits::{RunLimits, TerminalCondition};
use crate::domain::negotiation::{NegotiatedDeliveryPacket, NegotiationResolutionState};
use crate::domain::project_memory::{
    ProjectMemoryCondition, ProjectMemoryContext, ProjectMemoryStatus,
    evidence_contribution_summaries, evidence_root_for_lineage, read_project_memory,
};
use crate::domain::review::{ReviewOutcome, ReviewTrigger};
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::{
    ActiveSessionRecord, ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState,
    DelegationPacket, DelegationPacketKind, DelegationPacketState, DelegationStatusView,
    ProjectScaleSessionState, SessionCommand, SessionStatus,
};
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, Step, StepAttempt, StepExecutionRequest,
    StepExecutionResult, StepKind, StepResultSummary, StepStatus,
};
use crate::domain::task::{Task, TaskRequestError, TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEvent, TraceEventType, current_timestamp_millis};
use crate::domain::workflow::{
    ProjectScaleInput, ProjectScaleStageKind, propose_project_scale_path,
};
use crate::fixture::{
    FixtureRuntime, FixtureRuntimeError, build_fixture_plan_for_goal,
    build_fixture_runtime_for_flow, build_fixture_runtime_for_goal_plan, build_task_request,
    load_workspace_execution_profile,
};
use crate::orchestrator::decision_loop::{DecisionLoop, LoopTerminal};
use crate::orchestrator::goal_planner::{
    AuthoredInputDocument, GoalPlannerError, PlanningContextSources, build_goal_plan_with_sources,
};
use crate::orchestrator::governance::{
    GovernanceStepDecision, append_governed_document_to_lifecycle, bounded_governance_context,
    build_autopilot_decision, clarification_prompt_from_response, compacted_canon_memory_for_block,
    compacted_canon_memory_from_response, enrich_bounded_context_with_accumulated,
    governance_input_documents, governance_stage_key, governance_state_patch,
    governed_document_ref_from_response, overlay_stage_policy_with_intent,
    requested_governance_intent, runtime_command_available, selected_stage_policy,
};
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::review_trace::{record_review_step_completed, record_review_step_started};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};

#[derive(Debug, Clone)]
pub struct SessionRuntime {
    workspace_ref: PathBuf,
    checkpoint_store: FileCheckpointStore,
    session_store: FileSessionStore,
    trace_store: FileTraceStore,
}

#[derive(Debug, Clone, Serialize)]
struct DelegationTraceDetails {
    delegation: Option<DelegationStatusView>,
}

#[derive(Debug, Clone, Serialize)]
struct GoalPlanTracePayload {
    plan_id: String,
    goal: String,
    task_count: usize,
    goal_plan_state: String,
    goal_plan_revision: usize,
    flow_state: String,
    planning_rationale: Option<String>,
    verification_strategy: Option<String>,
    negotiation_goal_summary: Option<String>,
    negotiation_resolution: Option<String>,
    negotiation_acceptance_boundary: Option<String>,
    context_summary: Option<String>,
    context_credibility: Option<String>,
    context_primary_inputs: Vec<String>,
    context_provenance: Vec<String>,
    context_staleness_reason: Option<String>,
    canon_memory_summary: Option<String>,
    canon_memory_credibility: Option<String>,
    canon_memory_reason_code: Option<String>,
    canon_memory_artifact_refs: Vec<String>,
    routing_projection: RoutingDecisionProjection,
    canon_next_action: Option<String>,
    delegation: Option<DelegationStatusView>,
}

impl GoalPlanTracePayload {
    fn from_goal_plan(
        goal_plan: &GoalPlan,
        routing_projection: RoutingDecisionProjection,
        delegation: Option<DelegationStatusView>,
    ) -> Self {
        Self {
            plan_id: goal_plan.plan_id.clone(),
            goal: goal_plan.goal_text.clone(),
            task_count: goal_plan.tasks.len(),
            goal_plan_state: goal_plan.proposal_state_text().to_string(),
            goal_plan_revision: goal_plan.proposal_revision,
            flow_state: goal_plan.flow_state().summary_text(),
            planning_rationale: goal_plan.planning_rationale.clone(),
            verification_strategy: goal_plan.verification_strategy.clone(),
            negotiation_goal_summary: goal_plan.negotiation_goal_summary.clone(),
            negotiation_resolution: goal_plan.negotiation_resolution.clone(),
            negotiation_acceptance_boundary: goal_plan.negotiation_acceptance_boundary.clone(),
            context_summary: goal_plan.context_summary(),
            context_credibility: goal_plan.context_credibility(),
            context_primary_inputs: goal_plan.context_primary_inputs(),
            context_provenance: goal_plan.context_provenance_lines(),
            context_staleness_reason: goal_plan
                .context_pack
                .as_ref()
                .and_then(|pack| pack.staleness_reason.clone())
                .or_else(|| goal_plan.canon_memory_staleness_reason()),
            canon_memory_summary: goal_plan
                .compacted_canon_memory
                .as_ref()
                .map(|memory| memory.summary_text()),
            canon_memory_credibility: goal_plan
                .compacted_canon_memory
                .as_ref()
                .map(|memory| memory.credibility.as_str().to_string()),
            canon_memory_reason_code: goal_plan
                .compacted_canon_memory
                .as_ref()
                .and_then(|memory| memory.reason_code.clone()),
            canon_memory_artifact_refs: goal_plan
                .compacted_canon_memory
                .as_ref()
                .map(|memory| memory.artifact_refs.clone())
                .unwrap_or_default(),
            routing_projection,
            canon_next_action: goal_plan
                .compacted_canon_memory
                .as_ref()
                .and_then(|memory| memory.recommended_next_action.as_ref())
                .map(|action| format!("{}: {}", action.action, action.rationale)),
            delegation,
        }
    }
}

fn serialize_trace_payload<T: Serialize>(payload: &T) -> Value {
    serde_json::to_value(payload).unwrap_or(Value::Null)
}

fn delegation_trace_details(delegation: Option<DelegationStatusView>) -> Option<Value> {
    serde_json::to_value(DelegationTraceDetails { delegation }).ok()
}

fn project_scale_state_for_goal(goal: &str, next_action: &str) -> Option<ProjectScaleSessionState> {
    let input = project_scale_input_for_goal(goal)?;
    let path = propose_project_scale_path(input);
    let active_stage = path.stages.first()?;
    Some(ProjectScaleSessionState {
        active_work_unit_id: Some(format!("stage-001-{}", active_stage.kind.as_str())),
        path,
        active_stage_index: 0,
        checkpoint_refs: Vec::new(),
        trace_refs: Vec::new(),
        next_action: next_action.to_string(),
    })
}

fn project_scale_input_for_goal(goal: &str) -> Option<ProjectScaleInput> {
    let lower = goal.to_ascii_lowercase();
    let operational_entry = if lower.contains("supply chain") || lower.contains("supply-chain") {
        Some(ProjectScaleStageKind::SupplyChainAnalysis)
    } else if lower.contains("security") {
        Some(ProjectScaleStageKind::SecurityAssessment)
    } else if lower.contains("incident") {
        Some(ProjectScaleStageKind::Incident)
    } else if lower.contains("migration") || lower.contains("migrate") {
        Some(ProjectScaleStageKind::Migration)
    } else if lower.contains("system assessment") || lower.contains("assess the system") {
        Some(ProjectScaleStageKind::SystemAssessment)
    } else {
        None
    };

    let broad_goal = operational_entry.is_some()
        || lower.contains("capability")
        || lower.contains("project")
        || lower.contains("initiative")
        || lower.contains("platform")
        || lower.contains("onboarding")
        || lower.contains("architecture")
        || lower.split_whitespace().count() >= 10;

    if !broad_goal {
        return None;
    }

    let existing_system_change = operational_entry.is_none()
        && (lower.contains("existing")
            || lower.contains("modify")
            || lower.contains("change")
            || lower.contains("refactor")
            || lower.contains("fix"));

    Some(ProjectScaleInput {
        goal: goal.to_string(),
        problem_unclear: !existing_system_change,
        product_scope_unclear: !existing_system_change,
        capability_structure_unclear: lower.contains("capability")
            || lower.contains("platform")
            || lower.contains("system"),
        architecture_material: lower.contains("architecture")
            || lower.contains("audit")
            || lower.contains("auth")
            || lower.contains("schema")
            || lower.contains("integration")
            || lower.contains("capability"),
        existing_system_change,
        operational_entry,
    })
}

impl SessionRuntime {
    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        let workspace_ref = workspace_ref.as_ref().to_path_buf();
        Self {
            checkpoint_store: FileCheckpointStore::for_workspace(&workspace_ref),
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

    pub fn checkpoint_store(&self) -> &FileCheckpointStore {
        &self.checkpoint_store
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
        session.negotiation_packet = Some(session.authored_brief.as_ref().map_or_else(
            || {
                NegotiatedDeliveryPacket::from_goal(
                    &session.session_id,
                    &session.workspace_ref,
                    goal,
                )
            },
            |bundle| {
                NegotiatedDeliveryPacket::from_authored_brief(
                    &session.session_id,
                    &session.workspace_ref,
                    goal,
                    bundle,
                )
            },
        ));
        session.active_task = None;
        session.goal_plan = None;
        session.decisions.clear();
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

        if session.active_task.is_some() {
            return Err(SessionRuntimeError::FlowReplacementRequiresReset {
                current: session
                    .active_flow
                    .as_ref()
                    .map(|existing_flow| existing_flow.flow_name.clone())
                    .unwrap_or_else(|| "current-session".to_string()),
                requested: flow.name.to_string(),
            });
        }
        if session.goal_plan.is_some() {
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
        session.goal_plan = None;
        session.decisions.clear();
        session.active_flow_policy = FlowPolicy::from_builtin(flow.name).ok();
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
        if session.active_task.is_some()
            && session.goal_plan.is_none()
            && requested_flow.is_none()
            && !no_flow
        {
            return self.plan_compatibility_task(session);
        }

        let result = self.plan_goal_plan(session, requested_flow, no_flow);
        if matches!(result, Err(SessionRuntimeError::ClarificationRequired { .. }))
            && session.active_flow.is_some()
            && session.goal_plan.is_some()
            && requested_flow.is_none()
            && !no_flow
            && self.flow_selected_goal_plan_uses_compatibility_step(session)?
        {
            return self.plan_compatibility_task(session);
        }

        result
    }

    pub fn confirm_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let goal_plan = session.goal_plan.as_mut().ok_or(SessionRuntimeError::MissingGoalPlan)?;
        if goal_plan.requires_confirmation() {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
        }

        session.active_task = None;
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();
        Ok(())
    }

    pub fn prepare_cluster_run(
        &self,
        session: &mut ActiveSessionRecord,
        projection: &ClusterSessionProjection,
    ) -> Result<(), SessionRuntimeError> {
        if let Some(task) = session.active_task.as_mut() {
            task.context
                .set_cluster_session_projection(projection)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(goal_plan) = session.goal_plan.as_mut() {
            goal_plan.cluster_session_projection = Some(projection.clone());
            goal_plan.cluster_delivery_story = None;
        }

        Ok(())
    }

    pub fn uses_native_goal_plan(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        Ok(session.goal_plan.is_some())
    }

    pub fn resolve_routing_outcome(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<crate::domain::session::RoutingOutcome, SessionRuntimeError> {
        Ok(crate::domain::session::routing_outcome(session))
    }

    fn plan_compatibility_task(
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

    fn plan_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
        requested_flow: Option<&str>,
        no_flow: bool,
    ) -> Result<(), SessionRuntimeError> {
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let project_scale_state = project_scale_state_for_goal(&goal, "confirm_project_scale_path");
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
                session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
                session.decisions.clear();
                session.active_flow_policy = preserved_flow_policy.clone();
                session.latest_status = SessionStatus::GoalCaptured;
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

        self.apply_negotiation_projection(session, &goal, &mut goal_plan);
        if no_flow {
            goal_plan.mark_flow_skipped();
        }
        if requested_flow.is_some() || session.active_flow.is_some() || no_flow {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
        }

        session.active_flow = native_flow_state;
        session.active_task = None;
        session.goal_plan = Some(goal_plan);
        session.project_scale = project_scale_state;
        session.decisions.clear();
        session.active_flow_policy = preserved_flow_policy;
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
        if session.active_task.is_none() && session.latest_status.is_terminal() {
            return Err(SessionRuntimeError::MissingActiveTask);
        }
        if session.active_task.is_none()
            && self.flow_selected_goal_plan_uses_compatibility_step(session)?
        {
            self.ensure_flow_selected_compatibility_task(session)?;
        }
        let checkpoint_projection =
            self.prepare_checkpoint_for_mutation(session, SessionCommand::Step)?;
        let runtime = self.build_runtime(session)?;
        let _ = self.execute_single_step(session, &runtime)?;
        if let Some(projection) = checkpoint_projection.as_ref() {
            self.refresh_checkpoint_projection(projection)?;
        }
        Ok(())
    }

    fn flow_selected_goal_plan_uses_compatibility_step(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        if session.goal_plan.is_none() || session.active_flow.is_none() {
            return Ok(false);
        }

        match load_workspace_execution_profile(&self.workspace_ref) {
            Ok(_) => Ok(true),
            Err(FixtureRuntimeError::MissingExecutionProfile(_)) => Ok(false),
            Err(error) => Err(SessionRuntimeError::FixtureRuntime(error)),
        }
    }

    fn ensure_flow_selected_compatibility_task(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        if session.active_task.is_some() {
            return Ok(());
        }

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

        session.active_task = Some(task);
        session.decisions.clear();
        session.latest_status = SessionStatus::Planned;
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    pub fn run_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let checkpoint_projection =
            self.prepare_checkpoint_for_mutation(session, SessionCommand::Run)?;
        if session.goal_plan.is_some() {
            let response = self.run_native_goal_plan(session, checkpoint_projection.clone())?;
            if let Some(projection) = checkpoint_projection.as_ref() {
                self.refresh_checkpoint_projection(projection)?;
            }
            return Ok(response);
        }

        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                if let Some(projection) = checkpoint_projection.as_ref() {
                    self.refresh_checkpoint_projection(projection)?;
                }
                return Ok(response);
            }
        }
    }

    pub fn refresh_governance_state(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
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

            let runtime = self.build_runtime(session)?;
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

    fn planning_context_sources(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
    ) -> PlanningContextSources {
        let negotiation_packet = self.session_negotiation_packet(session, goal);
        let compacted_project_memory =
            Self::compacted_project_memory_for_workspace(&self.workspace_ref);

        PlanningContextSources {
            authored_input_summary: session
                .authored_brief
                .as_ref()
                .map(|bundle| bundle.summary_text()),
            authored_input_sources: session
                .authored_brief
                .as_ref()
                .map(|bundle| bundle.ordered_source_labels())
                .unwrap_or_default(),
            authored_input_documents: session
                .authored_brief
                .as_ref()
                .map(|bundle| {
                    bundle
                        .sources
                        .iter()
                        .map(|source| AuthoredInputDocument {
                            label: source.display_label(),
                            content: source.content.clone(),
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            execution_profile_read_targets: load_workspace_execution_profile(&self.workspace_ref)
                .map(|profile| profile.read_targets)
                .unwrap_or_default(),
            negotiation_goal_summary: negotiation_packet
                .as_ref()
                .map(|packet| packet.goal_summary.clone()),
            negotiation_resolution: negotiation_packet
                .as_ref()
                .map(|packet| packet.resolution_state.as_str().to_string()),
            negotiation_acceptance_boundary: negotiation_packet
                .as_ref()
                .map(|packet| packet.acceptance_boundary.success_headline.clone()),
            latest_trace_ref: session.latest_trace_ref.clone(),
            workflow_progress: session.workflow_progress.clone(),
            canon_capability_snapshot: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.latest_canon_capability_snapshot().ok().flatten()),
            compacted_canon_memory: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.latest_compacted_canon_memory().ok().flatten())
                .or(compacted_project_memory),
            latest_changed_files: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.state.get("latest_changed_files"))
                .and_then(|value| value.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            latest_validation_status: session
                .active_task
                .as_ref()
                .and_then(|task| task.context.state.get("latest_validation_status"))
                .and_then(|value| value.as_str().map(str::to_string)),
        }
    }

    fn compacted_project_memory_for_workspace(
        workspace_ref: &Path,
    ) -> Option<CompactedCanonMemory> {
        let context = read_project_memory(workspace_ref);
        Self::compacted_canon_memory_from_project_memory_context(workspace_ref, &context)
    }

    fn compacted_canon_memory_from_project_memory_context(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Option<CompactedCanonMemory> {
        if context.status == ProjectMemoryStatus::Absent {
            return None;
        }

        let condition = context.condition_for_workspace(workspace_ref)?;
        let artifact_refs = if context.status == ProjectMemoryStatus::Available {
            Self::project_memory_artifact_refs(workspace_ref, context)
        } else {
            Vec::new()
        };
        let contribution_summaries = if context.status == ProjectMemoryStatus::Available {
            Self::project_memory_contribution_summaries(workspace_ref, context)
        } else {
            Vec::new()
        };
        let credibility = match condition.decision() {
            crate::domain::project_memory::ProjectMemoryDecision::Proceed => {
                MemoryCredibilityState::Credible
            }
            crate::domain::project_memory::ProjectMemoryDecision::Warning => {
                MemoryCredibilityState::Stale
            }
            crate::domain::project_memory::ProjectMemoryDecision::HardStop => {
                MemoryCredibilityState::Insufficient
            }
        };
        let (possible_actions, recommended_next_action) = match condition {
            ProjectMemoryCondition::Stable => (Vec::new(), None),
            ProjectMemoryCondition::Pending => (
                vec![Self::project_memory_action(
                    "refresh",
                    "refresh project memory after Canon promotes a stable docs/project surface",
                )],
                Some(Self::project_memory_recommended_action(
                    "refresh",
                    "refresh project memory after Canon promotes a stable docs/project surface",
                )),
            ),
            ProjectMemoryCondition::EvidenceOnly => (
                vec![Self::project_memory_action(
                    "promote",
                    "publish a stable docs/project surface from Canon before reusing project memory as planning context",
                )],
                Some(Self::project_memory_recommended_action(
                    "promote",
                    "publish a stable docs/project surface from Canon before reusing project memory as planning context",
                )),
            ),
            ProjectMemoryCondition::ManualPromotion => (
                vec![Self::project_memory_action(
                    "promote",
                    "complete the manual Canon promotion step and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "promote",
                    "complete the manual Canon promotion step and refresh project memory",
                )),
            ),
            ProjectMemoryCondition::IncompleteMetadata => (
                vec![Self::project_memory_action(
                    "inspect",
                    "inspect the Canon packet metadata sidecars and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "inspect",
                    "inspect the Canon packet metadata sidecars and refresh project memory",
                )),
            ),
            ProjectMemoryCondition::BlockedGovernance => (
                vec![Self::project_memory_action(
                    "unblock",
                    "resolve the blocked Canon governance outcome before planning continues",
                )],
                Some(Self::project_memory_recommended_action(
                    "unblock",
                    "resolve the blocked Canon governance outcome before planning",
                )),
            ),
            ProjectMemoryCondition::MissingRequiredApproval => (
                vec![Self::project_memory_action(
                    "approve",
                    "complete the required Canon approval flow and refresh project memory",
                )],
                Some(Self::project_memory_recommended_action(
                    "approve",
                    "complete the required Canon approval flow before planning",
                )),
            ),
            ProjectMemoryCondition::MissingRequiredSourceArtifacts => (
                vec![Self::project_memory_action(
                    "restore",
                    "restore or republish the required Canon source artifacts before planning",
                )],
                Some(Self::project_memory_recommended_action(
                    "restore",
                    "restore or republish the required Canon source artifacts before planning",
                )),
            ),
            ProjectMemoryCondition::UnsupportedContract => (
                vec![Self::project_memory_action(
                    "update",
                    "update Canon or Boundline so both support the same project-memory contract",
                )],
                Some(Self::project_memory_recommended_action(
                    "update",
                    "update Canon or Boundline so both support the same project-memory contract before planning",
                )),
            ),
        };

        Some(CompactedCanonMemory {
            headline: condition.headline().to_string(),
            credibility,
            stage_key: None,
            run_ref: context.surfaces.iter().find_map(|surface| {
                surface.lineage.as_ref().map(|lineage| lineage.source_ref_leaf().to_string())
            }),
            packet_ref: None,
            reason_code: condition.reason_code().map(str::to_string),
            artifact_refs: artifact_refs.clone(),
            mode_summary: None,
            possible_actions,
            recommended_next_action,
            evidence_summary: (!artifact_refs.is_empty() || !contribution_summaries.is_empty())
                .then_some(CanonEvidenceInspectSummary {
                    execution_posture: None,
                    carried_forward_items: contribution_summaries,
                    artifact_provenance_links: artifact_refs,
                    closure_status: None,
                    closure_findings: Vec::new(),
                }),
        })
    }

    fn project_memory_action(action: &str, text: &str) -> CanonPossibleActionSummary {
        CanonPossibleActionSummary {
            action: action.to_string(),
            text: text.to_string(),
            target: None,
        }
    }

    fn project_memory_recommended_action(
        action: &str,
        rationale: &str,
    ) -> CanonRecommendedActionSummary {
        CanonRecommendedActionSummary {
            action: action.to_string(),
            rationale: rationale.to_string(),
            target: None,
        }
    }

    fn project_memory_artifact_refs(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Vec<String> {
        let mut refs = context
            .surfaces
            .iter()
            .map(|surface| surface.path.display().to_string())
            .collect::<Vec<_>>();

        for lineage in &context.evidence_refs {
            let evidence_root = evidence_root_for_lineage(workspace_ref, lineage);
            if evidence_root.exists() {
                let display = evidence_root
                    .strip_prefix(workspace_ref)
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| evidence_root.display().to_string());
                if !refs.contains(&display) {
                    refs.push(display);
                }
            }
        }

        refs
    }

    fn project_memory_contribution_summaries(
        workspace_ref: &Path,
        context: &ProjectMemoryContext,
    ) -> Vec<String> {
        let mut summaries = BTreeSet::new();
        for lineage in context
            .evidence_refs
            .iter()
            .chain(context.surfaces.iter().filter_map(|surface| surface.lineage.as_ref()))
        {
            for summary in evidence_contribution_summaries(workspace_ref, lineage) {
                summaries.insert(summary);
            }
        }

        summaries.into_iter().collect()
    }

    fn session_negotiation_packet(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
    ) -> Option<NegotiatedDeliveryPacket> {
        session.negotiation_packet.clone().or_else(|| {
            session
                .authored_brief
                .as_ref()
                .map(|bundle| {
                    NegotiatedDeliveryPacket::from_authored_brief(
                        &session.session_id,
                        &session.workspace_ref,
                        goal,
                        bundle,
                    )
                })
                .or_else(|| {
                    (!goal.trim().is_empty()).then(|| {
                        NegotiatedDeliveryPacket::from_goal(
                            &session.session_id,
                            &session.workspace_ref,
                            goal,
                        )
                    })
                })
        })
    }

    fn apply_negotiation_projection(
        &self,
        session: &ActiveSessionRecord,
        goal: &str,
        goal_plan: &mut GoalPlan,
    ) {
        if let Some(packet) = self.session_negotiation_packet(session, goal) {
            goal_plan.negotiation_goal_summary = Some(packet.goal_summary);
            goal_plan.negotiation_resolution = Some(packet.resolution_state.as_str().to_string());
            goal_plan.negotiation_acceptance_boundary =
                Some(packet.acceptance_boundary.success_headline);
        }
    }

    fn run_native_goal_plan(
        &self,
        session: &mut ActiveSessionRecord,
        checkpoint_projection: Option<CheckpointProjectionState>,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let Some(mut goal_plan) = session.goal_plan.clone() else {
            return Err(SessionRuntimeError::MissingGoalPlan);
        };

        if goal_plan.requires_confirmation() {
            return Err(SessionRuntimeError::PlanConfirmationRequired {
                flow_name: goal_plan.flow.as_ref().map(|flow| flow.flow_name.clone()),
            });
        }

        if let Some(delegation) = self.goal_plan_delegation_view(&goal_plan)
            && matches!(
                delegation.mode,
                DelegationContinuityMode::HandoffRequired
                    | DelegationContinuityMode::EscalationRequired
                    | DelegationContinuityMode::Stuck
                    | DelegationContinuityMode::InspectOnly
                    | DelegationContinuityMode::Exhausted
            )
        {
            let reason = session.latest_terminal_reason.clone().unwrap_or_else(|| {
                build_terminal_reason(
                    TerminalCondition::NoCredibleNextStep,
                    delegation.headline.clone(),
                    delegation_trace_details(Some(delegation.clone())),
                )
            });
            let trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
            return self.persist_native_result(
                session,
                goal_plan,
                Vec::new(),
                trace,
                NativePersistenceInput {
                    checkpoint_projection: checkpoint_projection.clone(),
                    terminal_reason: reason,
                    limits: RunLimits::default(),
                    record_terminal_event: true,
                    projected_task: None,
                },
            );
        }

        if let Some((packet, continuity)) = self.native_delegation_for_goal_plan(&goal_plan) {
            goal_plan
                .record_delegation_packet(packet, continuity)
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
            let delegation = self.goal_plan_delegation_view(&goal_plan);
            let reason = build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                delegation.as_ref().map(|view| view.headline.clone()).unwrap_or_else(|| {
                    "native goal plan reached a delegated continuity boundary".to_string()
                }),
                delegation_trace_details(delegation.clone()),
            );
            let trace = self.build_goal_plan_trace(&session.session_id, &goal_plan);
            return self.persist_native_result(
                session,
                goal_plan,
                Vec::new(),
                trace,
                NativePersistenceInput {
                    checkpoint_projection: checkpoint_projection.clone(),
                    terminal_reason: reason,
                    limits: RunLimits::default(),
                    record_terminal_event: true,
                    projected_task: None,
                },
            );
        }

        let runtime = self.build_runtime(session)?;
        let (native_governance_task, governance_events) =
            match self.prepare_native_governance_projection(session, &runtime, &goal_plan)? {
                NativeGovernanceProjection::None => (None, Vec::new()),
                NativeGovernanceProjection::Task { task, events } => (Some(*task), events),
                NativeGovernanceProjection::Terminal { response, task } => {
                    session.active_task = Some(*task);
                    session.goal_plan = Some(goal_plan);
                    session.decisions.clear();
                    return Ok(*response);
                }
            };
        let enable_flow_retry_probe = session.active_flow.is_some()
            && runtime.profile.governance.is_none()
            && runtime.profile.legacy_source.as_deref() != Some("native_goal_plan_synthesized");
        let decision_loop = DecisionLoop::new(
            runtime.agents.clone(),
            runtime.tools.clone(),
            self.trace_store.clone(),
            runtime.profile.limits.max_steps,
        );
        let (terminal, decisions, mut trace) = decision_loop
            .run_with_options(
                &goal_plan,
                session.active_flow_policy.as_ref(),
                &session.workspace_ref,
                &session.session_id,
                enable_flow_retry_probe,
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;
        let reason = self.native_terminal_reason(&terminal);
        if task_status_for_condition(reason.condition) == TaskStatus::Succeeded {
            self.propagate_cluster_delivery_changes(&goal_plan, &runtime)?;
        }
        if !governance_events.is_empty() {
            let insert_at = trace
                .events
                .iter()
                .rposition(|event| event.event_type == TraceEventType::TerminalRecorded)
                .unwrap_or(trace.events.len());
            trace.events.splice(insert_at..insert_at, governance_events);
        }
        let projected_task = native_governance_task.map(|task| {
            self.finalize_native_projected_task(
                task,
                &runtime,
                task_status_for_condition(reason.condition),
                &reason,
            )
        });

        self.persist_native_result(
            session,
            goal_plan,
            decisions,
            trace,
            NativePersistenceInput {
                checkpoint_projection,
                terminal_reason: reason,
                limits: runtime.profile.limits.clone(),
                record_terminal_event: false,
                projected_task,
            },
        )
    }

    fn persist_native_result(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: GoalPlan,
        decisions: Vec<Decision>,
        mut trace: ExecutionTrace,
        input: NativePersistenceInput,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let mut terminal_reason = input.terminal_reason;
        let mut terminal_status = task_status_for_condition(terminal_reason.condition);
        let mut goal_plan = goal_plan;
        let cluster_story = goal_plan
            .cluster_session_projection
            .as_ref()
            .map(|projection| self.build_cluster_delivery_story(projection, terminal_status));
        goal_plan.cluster_delivery_story = cluster_story.clone();
        if let Some(cluster_story) = cluster_story.as_ref()
            && cluster_story.execution_condition.kind == ClusteredExecutionKind::Failed
            && terminal_status == TaskStatus::Succeeded
        {
            terminal_reason = build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                cluster_story.execution_condition.summary.clone(),
                Some(json!({ "cluster_delivery_story": cluster_story })),
            );
            terminal_status = TaskStatus::Failed;
        }
        if input.record_terminal_event {
            trace.record_event(
                TraceEventType::TerminalRecorded,
                None,
                goal_plan.proposal_revision,
                json!({
                    "cluster_delivery_story": cluster_story,
                    "terminal_status": terminal_status,
                    "terminal_reason": terminal_reason.clone(),
                }),
            );
        } else if let Some(cluster_story) = cluster_story.clone()
            && let Some(event) = trace
                .events
                .iter_mut()
                .rev()
                .find(|event| event.event_type == TraceEventType::TerminalRecorded)
            && let Some(payload) = event.payload.as_object_mut()
        {
            payload.insert("cluster_delivery_story".to_string(), json!(cluster_story));
            payload.insert("terminal_status".to_string(), json!(terminal_status));
            payload.insert("terminal_reason".to_string(), json!(terminal_reason.clone()));
        }
        if let Some(checkpoint_projection) = input.checkpoint_projection.as_ref() {
            trace.record_event(
                TraceEventType::CheckpointCreated,
                None,
                goal_plan.proposal_revision,
                checkpoint_event_payload(checkpoint_projection),
            );
        }
        trace.finalize(terminal_status, terminal_reason.clone());
        let trace_location = self.persist_trace(&mut trace)?;
        let mut final_context =
            self.build_native_task_context(session, input.limits, &goal_plan)?;
        if let Some(checkpoint_projection) = input.checkpoint_projection.as_ref() {
            apply_checkpoint_projection_to_context(&mut final_context, checkpoint_projection);
        }
        let task_id = goal_plan.plan_id.clone();
        let plan_revision = goal_plan.proposal_revision;
        let projected_task = match input.projected_task {
            Some(task) => Some(task),
            None if cluster_story.is_some() => Some(self.synthesize_native_persisted_task(
                session,
                &goal_plan,
                &final_context,
                terminal_status,
                &terminal_reason,
            )?),
            None => None,
        };

        session.active_task = projected_task;
        session.goal_plan = Some(goal_plan);
        session.decisions = decisions;
        session.latest_status =
            if session.goal_plan.as_ref().and_then(GoalPlan::delegation_continuity).is_some() {
                SessionStatus::Planned
            } else {
                session_status_for_task_status(terminal_status)
            };
        session.latest_terminal_reason = Some(terminal_reason.clone());
        session.latest_trace_ref = Some(trace_location.clone());
        session.updated_at = current_timestamp_millis();

        Ok(TaskRunResponse {
            task_id,
            terminal_status,
            terminal_reason,
            final_context,
            plan_revision,
            trace_location,
        })
    }

    fn build_native_task_context(
        &self,
        session: &ActiveSessionRecord,
        limits: crate::domain::limits::RunLimits,
        goal_plan: &GoalPlan,
    ) -> Result<TaskContext, SessionRuntimeError> {
        let mut context = TaskContext::new(
            session.session_id.clone(),
            session.workspace_ref.clone(),
            limits,
            Map::new(),
        );
        if !goal_plan.delegation_packet_history().is_empty() {
            context
                .set_delegation_packet_history(goal_plan.delegation_packet_history())
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(continuity) = goal_plan.delegation_continuity() {
            context
                .set_delegation_continuity_state(continuity)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
            context
                .set_latest_compacted_canon_memory(memory)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(story) = goal_plan.cluster_delivery_story.as_ref() {
            context
                .set_cluster_delivery_story(story)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        Ok(context)
    }

    fn prepare_native_governance_projection(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &GoalPlan,
    ) -> Result<NativeGovernanceProjection, SessionRuntimeError> {
        let Some(active_flow) = session.active_flow.as_ref() else {
            return Ok(NativeGovernanceProjection::None);
        };
        let Some(governance) = runtime.profile.governance.as_ref() else {
            return Ok(NativeGovernanceProjection::None);
        };
        let goal = session.goal.clone().ok_or(SessionRuntimeError::MissingGoal)?;
        let request = build_task_request(
            &self.workspace_ref,
            &goal,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan = build_fixture_plan_for_goal(&self.workspace_ref, Some(active_flow), &goal)
            .map_err(SessionRuntimeError::FixtureRuntime)?;
        let mut task = Task::new(Uuid::new_v4().to_string(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;
        let mut governance_trace = self.build_goal_plan_trace(&session.session_id, goal_plan);
        let mut saw_governance = false;

        for step_index in 0..task.plan.steps.len() {
            task.plan.current_step_index = step_index;
            let step = task.plan.steps[step_index].clone();
            let Some(metadata) = FlowStepMetadata::from_step(&step)
                .map_err(|error| SessionRuntimeError::InvalidFlowState(error.to_string()))?
            else {
                continue;
            };
            let Some(policy) =
                selected_stage_policy(Some(governance), &metadata.flow_name, &metadata.stage_id)
            else {
                continue;
            };
            let governance_intent = requested_governance_intent(&task.input);
            let policy = overlay_stage_policy_with_intent(&policy, governance_intent.as_ref());
            if !policy.enabled {
                continue;
            }
            saw_governance = true;

            match self.execute_governance_for_step(
                session,
                &mut task,
                &mut governance_trace,
                runtime,
                &step,
                &metadata,
                governance,
                &policy,
                GovernanceRequestKind::Start,
            )? {
                GovernanceStepDecision::Continue => {}
                GovernanceStepDecision::Halt => {
                    let response =
                        self.build_native_governance_halt_response(session, &mut task)?;
                    return Ok(NativeGovernanceProjection::Terminal {
                        response: Box::new(response),
                        task: Box::new(task),
                    });
                }
                GovernanceStepDecision::Terminal(response) => {
                    return Ok(NativeGovernanceProjection::Terminal {
                        response: Box::new(response),
                        task: Box::new(task),
                    });
                }
            }
        }

        if !saw_governance {
            return Ok(NativeGovernanceProjection::None);
        }

        let events = governance_trace
            .events
            .into_iter()
            .filter(|event| is_governance_trace_event(event.event_type))
            .collect();
        Ok(NativeGovernanceProjection::Task { task: Box::new(task), events })
    }

    fn finalize_native_projected_task(
        &self,
        mut task: Task,
        runtime: &FixtureRuntime,
        terminal_status: TaskStatus,
        terminal_reason: &TerminalReason,
    ) -> Task {
        let changed_files = runtime
            .profile
            .attempts
            .iter()
            .flat_map(|attempt| attempt.changes.iter().map(|change| change.path.clone()))
            .collect::<Vec<_>>();
        if !changed_files.is_empty() {
            task.context.state.insert("latest_changed_files".to_string(), json!(changed_files));
        }
        task.context.state.insert(
            "latest_validation_status".to_string(),
            json!(if terminal_status == TaskStatus::Succeeded { "passed" } else { "failed" }),
        );
        if terminal_status == TaskStatus::Succeeded
            && let Some(review) = runtime.profile.review.as_ref()
            && let Some(trigger) = review
                .triggers
                .iter()
                .copied()
                .find(|trigger| !matches!(trigger, ReviewTrigger::ValidationFailed))
                .or_else(|| review.triggers.first().copied())
        {
            task.context.state.insert("latest_review_trigger".to_string(), json!(trigger));
            task.context
                .state
                .insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Accepted));
        }
        task.apply_terminal(terminal_status, terminal_reason.clone());
        task
    }

    fn synthesize_native_persisted_task(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        final_context: &TaskContext,
        terminal_status: TaskStatus,
        terminal_reason: &TerminalReason,
    ) -> Result<Task, SessionRuntimeError> {
        let request = build_task_request(
            &self.workspace_ref,
            &goal_plan.goal_text,
            session.session_id.clone(),
            session.authored_brief.as_ref(),
            session.negotiation_packet.as_ref(),
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let plan = build_fixture_plan_for_goal(
            &self.workspace_ref,
            session.active_flow.as_ref(),
            &goal_plan.goal_text,
        )
        .map_err(SessionRuntimeError::FixtureRuntime)?;
        let mut task = Task::new(goal_plan.plan_id.clone(), &request, plan)
            .map_err(SessionRuntimeError::TaskRequest)?;
        task.context = final_context.clone();
        task.apply_terminal(terminal_status, terminal_reason.clone());
        Ok(task)
    }

    fn build_native_governance_halt_response(
        &self,
        session: &ActiveSessionRecord,
        task: &mut Task,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        if matches!(task.status, TaskStatus::Planned) {
            task.mark_running();
        }
        let latest_governance = task
            .context
            .latest_governance_stage()
            .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?
            .ok_or(SessionRuntimeError::MissingGovernanceStage)?;
        let message = match latest_governance.lifecycle_state {
            GovernanceLifecycleState::AwaitingApproval => {
                format!("governance approval is still pending for {}", latest_governance.stage_key)
            }
            GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => format!(
                "governance blocked stage {}: {}",
                latest_governance.stage_key,
                latest_governance
                    .blocked_reason
                    .clone()
                    .unwrap_or_else(|| "governance review did not clear the stage".to_string())
            ),
            _ => format!("governance is still in progress for {}", latest_governance.stage_key),
        };
        let trace_location =
            session.latest_trace_ref.clone().ok_or(SessionRuntimeError::MissingTraceReference)?;

        Ok(TaskRunResponse {
            task_id: task.id.clone(),
            terminal_status: TaskStatus::Running,
            terminal_reason: build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                message,
                Some(json!({
                    "stage_key": latest_governance.stage_key,
                    "state": latest_governance.lifecycle_state,
                })),
            ),
            final_context: task.context.clone(),
            plan_revision: task.plan.revision,
            trace_location,
        })
    }

    fn native_terminal_reason(&self, terminal: &LoopTerminal) -> TerminalReason {
        match terminal {
            LoopTerminal::Success => build_terminal_reason(
                TerminalCondition::GoalSatisfied,
                "goal plan completed through the native decision loop",
                None,
            ),
            LoopTerminal::Failure(message) => {
                build_terminal_reason(TerminalCondition::UnrecoverableError, message, None)
            }
            LoopTerminal::Exhausted { steps_taken, max_steps } => build_terminal_reason(
                TerminalCondition::StepLimitExceeded,
                format!("native goal plan exhausted after {steps_taken} decision step(s)"),
                Some(json!({
                    "steps_taken": steps_taken,
                    "max_steps": max_steps,
                })),
            ),
            LoopTerminal::NoActionableState(message) => {
                build_terminal_reason(TerminalCondition::NoCredibleNextStep, message, None)
            }
        }
    }

    fn build_goal_plan_trace(&self, session_id: &str, goal_plan: &GoalPlan) -> ExecutionTrace {
        let mut trace = ExecutionTrace::new(
            goal_plan.plan_id.clone(),
            session_id.to_string(),
            goal_plan.goal_text.clone(),
        );
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            self.goal_plan_trace_payload(goal_plan),
        );
        trace
    }

    fn goal_plan_trace_payload(&self, goal_plan: &GoalPlan) -> Value {
        let payload = GoalPlanTracePayload::from_goal_plan(
            goal_plan,
            self.goal_plan_routing_projection(),
            self.goal_plan_delegation_view(goal_plan),
        );
        serialize_trace_payload(&payload)
    }

    fn goal_plan_routing_projection(&self) -> RoutingDecisionProjection {
        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
            .load()
            .ok()
            .flatten()
            .map(|config| config.routing);
        let global_routing = FileConfigStore::global_routing().ok().flatten();
        let effective_routing = resolve_effective_routing(
            &RoutingOverrides::default(),
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_capabilities = resolve_effective_runtime_capabilities(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_effort = resolve_effective_slot_effort_policies(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );

        RoutingDecisionProjection::from_effective_state(
            &effective_routing,
            &effective_capabilities,
            &effective_effort,
        )
    }

    fn goal_plan_delegation_view(&self, goal_plan: &GoalPlan) -> Option<DelegationStatusView> {
        goal_plan.delegation_continuity().and_then(|continuity| {
            DelegationStatusView::from_continuity(continuity, goal_plan.delegation_packet_history())
                .ok()
        })
    }

    fn native_delegation_for_goal_plan(
        &self,
        goal_plan: &GoalPlan,
    ) -> Option<(DelegationPacket, DelegationContinuityState)> {
        if !goal_plan.flow.as_ref().is_some_and(|flow| flow.confirmed) {
            return None;
        }

        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
            .load()
            .ok()
            .flatten()
            .map(|config| config.routing);
        let global_routing = FileConfigStore::global_routing().ok().flatten();
        let effective_routing = resolve_effective_routing(
            &RoutingOverrides::default(),
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_capabilities = resolve_effective_runtime_capabilities(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let effective_effort = resolve_effective_slot_effort_policies(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );

        let implementation_runtime = effective_routing.implementation.route.runtime;
        let assistant_runtimes = effective_assistant_runtimes(
            workspace_routing.as_ref(),
            cluster_routing.as_ref(),
            global_routing.as_ref(),
        );
        let assistant_runtime_mismatch =
            !assistant_runtimes.is_empty() && !assistant_runtimes.contains(&implementation_runtime);
        let implementation_capability = effective_capabilities.get(&implementation_runtime);
        let implementation_effort = effective_effort.get(&RouteSlot::Implementation);
        let requires_preserved_capability_handoff = implementation_capability
            .is_some_and(|capability| !capability.profile.continuation.is_supported())
            && implementation_effort
                .is_some_and(|effort| effort.policy.fallback == EffortFallbackPolicy::Preserve);

        if !requires_preserved_capability_handoff {
            if assistant_runtime_mismatch {
                let available_runtimes = assistant_runtimes
                    .iter()
                    .map(|runtime| runtime.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let evidence_summary = format!(
                    "implementation route requires {}, but available assistant runtimes are: {}",
                    implementation_runtime.as_str(),
                    available_runtimes
                );
                let packet = DelegationPacket {
                    packet_id: Uuid::new_v4().to_string(),
                    kind: DelegationPacketKind::Escalation,
                    state: DelegationPacketState::Active,
                    created_at: current_timestamp_millis(),
                    resolved_at: None,
                    source_route_owner: implementation_runtime.as_str().to_string(),
                    target_owner: "operator".to_string(),
                    continuity_reason: evidence_summary.clone(),
                    recommended_next_action: "boundline inspect".to_string(),
                    evidence_refs: Vec::new(),
                    capability_summary: Some(evidence_summary.clone()),
                    stuck_marker: None,
                    superseded_by_packet_id: None,
                };
                let continuity = DelegationContinuityState {
                    active_packet_id: Some(packet.packet_id.clone()),
                    mode: DelegationContinuityMode::EscalationRequired,
                    authority_source: ContinuityAuthority::NativeSession,
                    next_command: "boundline inspect".to_string(),
                    headline: packet.headline(),
                    evidence_summary: packet.evidence_summary(),
                };
                return Some((packet, continuity));
            }
            return None;
        }

        let implementation_capability = implementation_capability?;

        let evidence_summary = format!(
            "{} lacks continuation support for implementation",
            implementation_runtime.as_str()
        );

        if let Some(target_runtime) = assistant_runtimes.into_iter().find(|runtime| {
            effective_capabilities.get(runtime).is_some_and(|capability| {
                capability.profile.continuation.is_supported()
                    && capability.profile.handoff_target.is_supported()
            })
        }) {
            let packet = DelegationPacket {
                packet_id: Uuid::new_v4().to_string(),
                kind: DelegationPacketKind::Handoff,
                state: DelegationPacketState::Active,
                created_at: current_timestamp_millis(),
                resolved_at: None,
                source_route_owner: implementation_runtime.as_str().to_string(),
                target_owner: target_runtime.as_str().to_string(),
                continuity_reason: "implementation route cannot continue".to_string(),
                recommended_next_action: "boundline status".to_string(),
                evidence_refs: Vec::new(),
                capability_summary: Some(evidence_summary.clone()),
                stuck_marker: None,
                superseded_by_packet_id: None,
            };
            let continuity = DelegationContinuityState {
                active_packet_id: Some(packet.packet_id.clone()),
                mode: DelegationContinuityMode::HandoffRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline status".to_string(),
                headline: packet.headline(),
                evidence_summary: packet.evidence_summary(),
            };
            return Some((packet, continuity));
        }

        if implementation_capability.profile.escalation_context.is_supported() {
            let packet = DelegationPacket {
                packet_id: Uuid::new_v4().to_string(),
                kind: DelegationPacketKind::Escalation,
                state: DelegationPacketState::Active,
                created_at: current_timestamp_millis(),
                resolved_at: None,
                source_route_owner: implementation_runtime.as_str().to_string(),
                target_owner: "operator".to_string(),
                continuity_reason: "implementation route cannot continue".to_string(),
                recommended_next_action: "boundline inspect".to_string(),
                evidence_refs: Vec::new(),
                capability_summary: Some(evidence_summary),
                stuck_marker: None,
                superseded_by_packet_id: None,
            };
            let continuity = DelegationContinuityState {
                active_packet_id: Some(packet.packet_id.clone()),
                mode: DelegationContinuityMode::EscalationRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: packet.headline(),
                evidence_summary: packet.evidence_summary(),
            };
            return Some((packet, continuity));
        }

        None
    }

    fn build_cluster_delivery_story(
        &self,
        projection: &ClusterSessionProjection,
        terminal_status: TaskStatus,
    ) -> ClusterDeliveryStory {
        let blocking_workspace_ref = projection
            .member_workspace_refs
            .iter()
            .filter(|workspace_ref| *workspace_ref != &projection.primary_workspace_ref)
            .find(|workspace_ref| cluster_workspace_is_blocked(workspace_ref))
            .cloned();

        let execution_condition =
            if let Some(blocking_workspace_ref) = blocking_workspace_ref.clone() {
                ClusteredExecutionCondition {
                    kind: ClusteredExecutionKind::Failed,
                    active_workspace_ref: Some(projection.primary_workspace_ref.clone()),
                    blocking_workspace_ref: Some(blocking_workspace_ref.clone()),
                    summary: format!(
                        "cluster delivery is blocked by workspace {blocking_workspace_ref}"
                    ),
                    recovery_allowed: true,
                }
            } else {
                ClusteredExecutionCondition {
                    kind: match terminal_status {
                        TaskStatus::Succeeded => ClusteredExecutionKind::Success,
                        TaskStatus::Failed | TaskStatus::Aborted => ClusteredExecutionKind::Failed,
                        TaskStatus::Exhausted => ClusteredExecutionKind::Exhausted,
                        TaskStatus::Planned | TaskStatus::Running => ClusteredExecutionKind::Paused,
                    },
                    active_workspace_ref: Some(projection.primary_workspace_ref.clone()),
                    blocking_workspace_ref: None,
                    summary: format!(
                        "native cluster delivery executed from {}",
                        projection.primary_workspace_ref
                    ),
                    recovery_allowed: terminal_status != TaskStatus::Succeeded,
                }
            };

        let participating_workspaces = projection
            .member_workspace_refs
            .iter()
            .enumerate()
            .map(|(order, workspace_ref)| {
                let (participation_kind, latest_status, headline) =
                    if workspace_ref == &projection.primary_workspace_ref {
                        (
                            WorkspaceParticipationKind::Mutated,
                            Some(cluster_task_status_text(terminal_status).to_string()),
                            "authoritative native workspace executed the bounded goal".to_string(),
                        )
                    } else if blocking_workspace_ref.as_deref() == Some(workspace_ref.as_str()) {
                        (
                            WorkspaceParticipationKind::Blocked,
                            Some("blocked".to_string()),
                            "workspace currently blocks clustered follow-through".to_string(),
                        )
                    } else {
                        (
                            WorkspaceParticipationKind::ReadOnly,
                            Some("ready".to_string()),
                            "workspace remains aligned with the authoritative cluster route"
                                .to_string(),
                        )
                    };

                WorkspaceParticipationRecord {
                    workspace_ref: workspace_ref.clone(),
                    participation_kind,
                    order,
                    latest_trace_ref: None,
                    latest_status,
                    headline,
                    terminal_reason: None,
                }
            })
            .collect();

        ClusterDeliveryStory {
            cluster_id: projection.cluster_id.clone(),
            primary_workspace_ref: projection.primary_workspace_ref.clone(),
            authoritative_workspace_ref: projection.primary_workspace_ref.clone(),
            route_owner: ClusterRouteOwner::Native,
            member_workspace_refs: projection.member_workspace_refs.clone(),
            participating_workspaces,
            started_from_command: projection.started_from_command.clone(),
            execution_condition,
            updated_at: current_timestamp_millis(),
        }
    }

    fn propagate_cluster_delivery_changes(
        &self,
        goal_plan: &GoalPlan,
        runtime: &FixtureRuntime,
    ) -> Result<(), SessionRuntimeError> {
        let Some(projection) = goal_plan.cluster_session_projection.as_ref() else {
            return Ok(());
        };

        let changed_paths = runtime
            .profile
            .attempts
            .iter()
            .flat_map(|attempt| attempt.changes.iter().map(|change| change.path.clone()))
            .collect::<BTreeSet<_>>();
        if changed_paths.is_empty() {
            return Ok(());
        }

        for workspace_ref in projection
            .member_workspace_refs
            .iter()
            .filter(|workspace_ref| *workspace_ref != &projection.primary_workspace_ref)
        {
            if cluster_workspace_is_blocked(workspace_ref) {
                continue;
            }

            for relative_path in &changed_paths {
                let source_path = self.workspace_ref.join(relative_path);
                let target_path = Path::new(workspace_ref).join(relative_path);
                let contents = std::fs::read(&source_path).map_err(|source| {
                    SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                        path: source_path.clone(),
                        source,
                    })
                })?;
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|source| {
                        SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                            path: parent.to_path_buf(),
                            source,
                        })
                    })?;
                }
                std::fs::write(&target_path, contents).map_err(|source| {
                    SessionRuntimeError::FixtureRuntime(FixtureRuntimeError::Io {
                        path: target_path.clone(),
                        source,
                    })
                })?;
            }
        }

        Ok(())
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

        let goal_plan = session.goal_plan.as_ref();

        if let Some(goal_plan) = goal_plan
            && session.active_flow_policy.is_none()
            && session.active_workflow_progress().is_none()
        {
            return build_fixture_runtime_for_goal_plan(&self.workspace_ref, goal_plan)
                .map_err(SessionRuntimeError::FixtureRuntime);
        }

        match build_fixture_runtime_for_flow(&self.workspace_ref, session.active_flow.as_ref()) {
            Ok(runtime) => Ok(runtime),
            Err(error @ FixtureRuntimeError::MissingExecutionProfile(_)) => {
                if let Some(goal_plan) = goal_plan {
                    build_fixture_runtime_for_goal_plan(&self.workspace_ref, goal_plan)
                        .map_err(SessionRuntimeError::FixtureRuntime)
                } else {
                    Err(SessionRuntimeError::FixtureRuntime(error))
                }
            }
            Err(error) => Err(SessionRuntimeError::FixtureRuntime(error)),
        }
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
        if let Some(checkpoint_projection) = checkpoint_projection_from_context(&task.context)
            && !trace.events.iter().any(|event| {
                event.event_type == TraceEventType::CheckpointCreated
                    && event.payload.get("checkpoint_id").and_then(Value::as_str)
                        == Some(checkpoint_projection.checkpoint_id.as_str())
            })
        {
            trace.record_event(
                TraceEventType::CheckpointCreated,
                None,
                task.plan.revision,
                checkpoint_event_payload(&checkpoint_projection),
            );
        }
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
            let Some(step) = task.plan.current_step_mut() else {
                return Err(SessionRuntimeError::ExecutionInvariant(
                    "current step disappeared after scheduler validation".to_string(),
                ));
            };
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
                let Some(output) = result.output.clone() else {
                    return Err(SessionRuntimeError::ExecutionInvariant(format!(
                        "step {} reported success without output after normalization",
                        step_snapshot.id
                    )));
                };
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
                let Some(error) = result.error.clone() else {
                    return Err(SessionRuntimeError::ExecutionInvariant(format!(
                        "step {} reported failure without error details after normalization",
                        step_snapshot.id
                    )));
                };
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
            .or(existing_stage_mode);
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
            let trace_location = self.persist_trace(trace)?;
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
        let compacted_canon_memory =
            compacted_canon_memory_from_response(&stage_key, runtime_kind, &response);
        let patch = governance_state_patch(
            &record,
            response.packet.as_ref(),
            packet_reuse.as_ref(),
            decision.as_ref(),
            compacted_canon_memory.as_ref(),
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
                    "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
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
                    }),
                );
                let trace_location = self.persist_trace(trace)?;
                session.latest_status = SessionStatus::Running;
                session.latest_terminal_reason = None;
                session.latest_trace_ref = Some(trace_location);
                session.updated_at = current_timestamp_millis();
                if let Some(canon_mode) = response
                    .packet
                    .as_ref()
                    .and_then(|packet| packet.canon_mode)
                    .or_else(|| decision.as_ref().and_then(|decision| decision.selected_mode))
                {
                    let doc_ref =
                        governed_document_ref_from_response(&stage_key, canon_mode, &response);
                    append_governed_document_to_lifecycle(session, doc_ref);
                }
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
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
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
                        "packet_binding_reason": packet_reuse.as_ref().map(|binding| binding.binding_reason),
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
        let compacted_canon_memory =
            compacted_canon_memory_for_block(&block.stage_key, block.runtime, &block.reason);
        let patch = governance_state_patch(
            &record,
            None,
            None,
            decision.as_ref(),
            compacted_canon_memory.as_ref(),
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

        let Some(completed_step) = task.plan.steps.get(completed_step_index) else {
            return Err(crate::domain::flow::FlowValidationError::InvalidStageIndex {
                flow_name: active_flow.flow_name.clone(),
                stage_index: completed_step_index,
                total_stages: task.plan.steps.len(),
            });
        };
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

const LATEST_CHECKPOINT_ID_KEY: &str = "latest_checkpoint_id";
const LATEST_CHECKPOINT_SCOPE_KEY: &str = "latest_checkpoint_scope";
const LATEST_CHECKPOINT_RESTORE_COMMAND_KEY: &str = "latest_checkpoint_restore_command";
const LATEST_CHECKPOINT_WORKSPACES_KEY: &str = "latest_checkpoint_workspace_refs";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckpointCaptureScope {
    workspace_ref: String,
    authority_scope: CheckpointAuthorityScope,
    candidate_paths: Vec<String>,
    already_modified_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckpointProjectionState {
    checkpoint_id: String,
    scope: String,
    restore_command: String,
    workspace_refs: Vec<String>,
}

fn checkpoint_event_payload(projection: &CheckpointProjectionState) -> Value {
    json!({
        "checkpoint_id": projection.checkpoint_id,
        "checkpoint_scope": projection.scope,
        "checkpoint_restore_command": projection.restore_command,
        "checkpoint_workspace_refs": projection.workspace_refs,
    })
}

fn apply_checkpoint_projection_to_context(
    context: &mut TaskContext,
    projection: &CheckpointProjectionState,
) {
    context.state.insert(LATEST_CHECKPOINT_ID_KEY.to_string(), json!(projection.checkpoint_id));
    context.state.insert(LATEST_CHECKPOINT_SCOPE_KEY.to_string(), json!(projection.scope));
    context.state.insert(
        LATEST_CHECKPOINT_RESTORE_COMMAND_KEY.to_string(),
        json!(projection.restore_command),
    );
    context
        .state
        .insert(LATEST_CHECKPOINT_WORKSPACES_KEY.to_string(), json!(projection.workspace_refs));
}

fn checkpoint_projection_from_context(context: &TaskContext) -> Option<CheckpointProjectionState> {
    let checkpoint_id = context.state.get(LATEST_CHECKPOINT_ID_KEY)?.as_str()?.to_string();
    let scope = context.state.get(LATEST_CHECKPOINT_SCOPE_KEY)?.as_str()?.to_string();
    let restore_command =
        context.state.get(LATEST_CHECKPOINT_RESTORE_COMMAND_KEY)?.as_str()?.to_string();
    let workspace_refs = context
        .state
        .get(LATEST_CHECKPOINT_WORKSPACES_KEY)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(CheckpointProjectionState { checkpoint_id, scope, restore_command, workspace_refs })
}

impl SessionRuntime {
    fn prepare_checkpoint_for_mutation(
        &self,
        session: &mut ActiveSessionRecord,
        trigger_command: SessionCommand,
    ) -> Result<Option<CheckpointProjectionState>, SessionRuntimeError> {
        let scopes = self.checkpoint_capture_scopes(session)?;
        if scopes.is_empty() {
            return Ok(None);
        }

        let group_id =
            (scopes.len() > 1).then(|| format!("checkpoint-group-{}", current_timestamp_millis()));
        let restore_id = group_id
            .clone()
            .unwrap_or_else(|| format!("checkpoint-{}", current_timestamp_millis()));
        let restore_command = if scopes.len() > 1 {
            format!(
                "boundline checkpoint restore {restore_id} --cluster {}",
                self.workspace_ref.display()
            )
        } else {
            format!(
                "boundline checkpoint restore {restore_id} --workspace {}",
                self.workspace_ref.display()
            )
        };

        let task_id = session
            .active_task
            .as_ref()
            .map(|task| task.id.clone())
            .or_else(|| session.goal_plan.as_ref().map(|goal_plan| goal_plan.plan_id.clone()));
        let step_id = session
            .active_task
            .as_ref()
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone()));

        for (index, scope) in scopes.iter().enumerate() {
            let checkpoint_id = group_id
                .clone()
                .map(|group_id| format!("{group_id}-{index}"))
                .unwrap_or_else(|| restore_id.clone());
            FileCheckpointStore::for_workspace(Path::new(&scope.workspace_ref))
                .capture(CheckpointCaptureRequest {
                    checkpoint_id,
                    group_id: group_id.clone(),
                    workspace_ref: scope.workspace_ref.clone(),
                    authority_scope: scope.authority_scope,
                    trigger_command,
                    session_id: Some(session.session_id.clone()),
                    task_id: task_id.clone(),
                    step_id: step_id.clone(),
                    candidate_paths: scope.candidate_paths.clone(),
                    already_modified_paths: scope.already_modified_paths.clone(),
                })
                .map_err(SessionRuntimeError::CheckpointStore)?;
        }

        let projection = CheckpointProjectionState {
            checkpoint_id: restore_id,
            scope: if scopes.len() > 1 {
                "cluster".to_string()
            } else {
                scopes
                    .first()
                    .map(|scope| scope.authority_scope.as_str().to_string())
                    .unwrap_or_else(|| "workspace".to_string())
            },
            restore_command,
            workspace_refs: scopes.iter().map(|scope| scope.workspace_ref.clone()).collect(),
        };

        if let Some(task) = session.active_task.as_mut() {
            apply_checkpoint_projection_to_context(&mut task.context, &projection);
        }

        Ok(Some(projection))
    }

    fn refresh_checkpoint_projection(
        &self,
        projection: &CheckpointProjectionState,
    ) -> Result<(), SessionRuntimeError> {
        if projection.workspace_refs.len() > 1 {
            for workspace_ref in &projection.workspace_refs {
                let store = FileCheckpointStore::for_workspace(Path::new(workspace_ref));
                for manifest in store
                    .load_group(&projection.checkpoint_id)
                    .map_err(SessionRuntimeError::CheckpointStore)?
                {
                    store
                        .refresh_observed_state(&manifest.checkpoint_id)
                        .map_err(SessionRuntimeError::CheckpointStore)?;
                }
            }
        } else if let Some(workspace_ref) = projection.workspace_refs.first() {
            FileCheckpointStore::for_workspace(Path::new(workspace_ref))
                .refresh_observed_state(&projection.checkpoint_id)
                .map_err(SessionRuntimeError::CheckpointStore)?;
        }

        Ok(())
    }

    fn checkpoint_capture_scopes(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<Vec<CheckpointCaptureScope>, SessionRuntimeError> {
        let cluster_projection = session
            .active_task
            .as_ref()
            .and_then(|task| task.context.cluster_session_projection().ok().flatten())
            .or_else(|| {
                session
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.cluster_session_projection.clone())
            });

        if let Some(cluster_projection) = cluster_projection {
            let mut scopes = Vec::new();
            scopes.push(self.build_checkpoint_scope(
                &cluster_projection.primary_workspace_ref,
                CheckpointAuthorityScope::ClusterPrimary,
                session,
            )?);
            for member_workspace in &cluster_projection.member_workspace_refs {
                if member_workspace == &cluster_projection.primary_workspace_ref {
                    continue;
                }
                let scope = self.build_checkpoint_scope(
                    member_workspace,
                    CheckpointAuthorityScope::ClusterMember,
                    session,
                )?;
                if !scope.candidate_paths.is_empty() {
                    scopes.push(scope);
                }
            }
            return Ok(scopes
                .into_iter()
                .filter(|scope| !scope.candidate_paths.is_empty())
                .collect());
        }

        let scope = self.build_checkpoint_scope(
            &self.workspace_ref.to_string_lossy(),
            CheckpointAuthorityScope::Workspace,
            session,
        )?;
        Ok((!scope.candidate_paths.is_empty()).then_some(scope).into_iter().collect())
    }

    fn build_checkpoint_scope(
        &self,
        workspace_ref: &str,
        authority_scope: CheckpointAuthorityScope,
        session: &ActiveSessionRecord,
    ) -> Result<CheckpointCaptureScope, SessionRuntimeError> {
        let workspace = Path::new(workspace_ref);
        let mut candidate_paths = load_workspace_execution_profile(workspace)
            .map(|profile| {
                profile
                    .attempts
                    .into_iter()
                    .flat_map(|attempt| attempt.changes.into_iter().map(|change| change.path))
                    .collect::<Vec<_>>()
            })
            .or_else(|error| match error {
                FixtureRuntimeError::MissingExecutionProfile(_) => Ok(Vec::new()),
                other => Err(SessionRuntimeError::FixtureRuntime(other)),
            })?;

        let already_modified_paths = session
            .active_task
            .as_ref()
            .and_then(|task| {
                (task.context.workspace_ref == workspace_ref)
                    .then(|| task.context.state.get("latest_changed_files"))
                    .flatten()
            })
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if candidate_paths.is_empty() {
            candidate_paths = already_modified_paths.clone();
        }

        candidate_paths.sort();
        candidate_paths.dedup();

        Ok(CheckpointCaptureScope {
            workspace_ref: workspace_ref.to_string(),
            authority_scope,
            candidate_paths,
            already_modified_paths,
        })
    }
}

fn effective_assistant_runtimes(
    workspace: Option<&RoutingConfig>,
    cluster: Option<&RoutingConfig>,
    global: Option<&RoutingConfig>,
) -> Vec<RuntimeKind> {
    workspace
        .filter(|config| !config.assistant_runtimes.is_empty())
        .map(|config| config.assistant_runtimes.clone())
        .or_else(|| {
            cluster
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .or_else(|| {
            global
                .filter(|config| !config.assistant_runtimes.is_empty())
                .map(|config| config.assistant_runtimes.clone())
        })
        .unwrap_or_default()
}

struct NativePersistenceInput {
    checkpoint_projection: Option<CheckpointProjectionState>,
    terminal_reason: TerminalReason,
    limits: RunLimits,
    record_terminal_event: bool,
    projected_task: Option<Task>,
}

enum NativeGovernanceProjection {
    None,
    Task { task: Box<Task>, events: Vec<TraceEvent> },
    Terminal { response: Box<TaskRunResponse>, task: Box<Task> },
}

fn cluster_task_status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Planned => "planned",
        TaskStatus::Running => "running",
        TaskStatus::Succeeded => "succeeded",
        TaskStatus::Failed => "failed",
        TaskStatus::Exhausted => "exhausted",
        TaskStatus::Aborted => "aborted",
    }
}

fn cluster_workspace_is_blocked(workspace_ref: &str) -> bool {
    let workspace = Path::new(workspace_ref);
    let Ok(profile) = load_workspace_execution_profile(workspace) else {
        return true;
    };

    !profile.attempts.iter().any(|attempt| {
        attempt.changes.iter().all(|change| {
            let Ok(contents) = std::fs::read_to_string(workspace.join(&change.path)) else {
                return false;
            };
            contents.contains(&change.find) || contents.contains(&change.replace)
        })
    })
}

fn is_governance_trace_event(event_type: TraceEventType) -> bool {
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
    #[error("checkpoint store operation failed: {0}")]
    CheckpointStore(#[from] CheckpointStoreError),
    #[error("active session has no captured goal")]
    MissingGoal,
    #[error(
        "active session requires clarification before planning can continue: {headline}: {prompt}"
    )]
    ClarificationRequired { headline: String, prompt: String },
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
    #[error("active session has no proposed goal plan")]
    MissingGoalPlan,
    #[error("active session has a proposed plan that must be confirmed before execution")]
    PlanConfirmationRequired { flow_name: Option<String> },
    #[error("active session is missing the persisted trace reference")]
    MissingTraceReference,
    #[error("active task is missing a terminal reason")]
    MissingTerminalReason,
    #[error("fixture runtime is invalid: {0}")]
    FixtureRuntime(#[source] FixtureRuntimeError),
    #[error("task request is invalid: {0}")]
    TaskRequest(#[source] TaskRequestError),
    #[error("goal plan is invalid: {0}")]
    GoalPlan(String),
    #[error("native decision loop failed: {0}")]
    DecisionLoop(String),
    #[error("task context state is invalid: {0}")]
    TaskContext(String),
    #[error("active task is missing projected governance state")]
    MissingGovernanceStage,
    #[error("governance state patch is invalid: {0}")]
    GovernancePatch(String),
    #[error("governance runtime failed: {0}")]
    GovernanceRuntime(String),
    #[error("session runtime execution invariant failed: {0}")]
    ExecutionInvariant(String),
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
    use std::path::{Path, PathBuf};

    use serde_json::{Map, json};
    use uuid::Uuid;

    use super::{
        SessionRuntime, cluster_task_status_text, cluster_workspace_is_blocked,
        effective_assistant_runtimes, is_governance_trace_event, project_scale_input_for_goal,
        project_scale_state_for_goal, session_status_for_task_status,
    };
    use crate::adapters::checkpoint_store::FileCheckpointStore;
    use crate::adapters::trace_store::TraceStore;
    use crate::domain::brief::normalize_inputs;
    use crate::domain::cluster::{ClusterSessionProjection, ClusteredExecutionKind};
    use crate::domain::configuration::{RoutingConfig, RuntimeKind};
    use crate::domain::execution::{
        ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
        WorkspaceExecutionProfile,
    };
    use crate::domain::flow::{attach_stage_metadata, built_in_flow};
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        ApprovalState, CanonMode, CanonRuntimeConfig, GovernanceLifecycleState, GovernanceProfile,
        GovernanceRuntimeKind, GovernedStageRecord, PacketReadiness, StageGovernancePolicy,
        SystemContextBinding,
    };
    use crate::domain::limits::{RunLimits, TerminalCondition};
    use crate::domain::plan::Plan;
    use crate::domain::project_memory::{
        CompatibilityOutcome, LineageRef, ProjectMemoryContext, ProjectMemoryStatus,
        ProjectMemorySurface, PromotionStateView,
    };
    use crate::domain::session::{
        ActiveSessionRecord, ContinuityAuthority, DelegationContinuityMode,
        DelegationContinuityState, SessionCommand, SessionStatus,
    };
    use crate::domain::step::{ExecutionStatus, Recoverability, Step, StepStatus};
    use crate::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
    use crate::domain::task_context::TaskContext;
    use crate::domain::trace::{ExecutionTrace, TraceEventType};
    use crate::domain::workflow::ProjectScaleStageKind;
    use crate::fixture::FixtureRuntime;
    use crate::orchestrator::planner::StaticPlanner;
    use crate::registry::agent_registry::AgentRegistry;
    use crate::registry::tool_registry::ToolRegistry;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        workspace
    }

    fn sample_project_memory_lineage(run_ref: &str, mode: &str) -> LineageRef {
        LineageRef {
            contract_version: "v1".to_string(),
            producer: "canon".to_string(),
            source_ref: format!("canon-run:{run_ref}"),
            source_artifacts: vec!["architecture-overview.md".to_string()],
            mode: Some(mode.to_string()),
            promotion_state: "auto".to_string(),
            approval_state: Some("Completed".to_string()),
            stage: Some(mode.to_string()),
            owner: Some("Owner <owner@example.com>".to_string()),
            risk: Some("bounded-impact".to_string()),
            zone: Some("yellow".to_string()),
            promoted_at: "2026-05-13T14:30:00Z".to_string(),
            content_digest: "sha256:abc123".to_string(),
            packet_readiness: Some("complete".to_string()),
            promotion_profile: Some("project-memory".to_string()),
        }
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
            workspace.join(".boundline/execution.json"),
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
            negotiation_packet: None,
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
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        }
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
    fn planning_context_sources_include_authored_documents_and_recent_change_signals() {
        let workspace = temp_workspace("boundline-runtime-planning-context");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::write(workspace.join("src/add.rs"), "pub fn add() -> i32 { 2 }\n").unwrap();
        fs::write(
            workspace.join("brief.md"),
            "Focus on src/add.rs and tests/add.rs before broad scanning.\n",
        )
        .unwrap();

        let authored_brief = normalize_inputs(
            &workspace,
            Some("Fix the failing add test"),
            &[PathBuf::from("brief.md")],
        )
        .unwrap();
        let mut task = decision_task(workspace.to_string_lossy().as_ref(), json!({}));
        task.context.state.insert("latest_changed_files".to_string(), json!(["src/add.rs"]));
        task.context.state.insert("latest_validation_status".to_string(), json!("failed"));

        let mut session = build_session(&workspace, task);
        session.goal = Some("Fix the failing add test".to_string());
        session.authored_brief = Some(authored_brief);

        let runtime = SessionRuntime::for_workspace(&workspace);
        let sources = runtime.planning_context_sources(&session, "Fix the failing add test");

        assert!(
            sources
                .authored_input_documents
                .iter()
                .any(|document| document.label.contains("brief.md")
                    && document.content.contains("src/add.rs"))
        );
        assert_eq!(sources.latest_changed_files, vec!["src/add.rs".to_string()]);
        assert_eq!(sources.latest_validation_status.as_deref(), Some("failed"));
        assert!(sources.authored_input_sources.iter().any(|label| label.contains("brief.md")));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn project_scale_helpers_classify_broad_goals_and_operational_entries() {
        let onboarding = project_scale_input_for_goal(
            "Build a customer onboarding capability with audit logging",
        )
        .expect("broad onboarding goal should be classified");
        assert!(!onboarding.existing_system_change);
        assert!(onboarding.problem_unclear);
        assert!(onboarding.product_scope_unclear);
        assert!(onboarding.capability_structure_unclear);
        assert!(onboarding.architecture_material);
        assert_eq!(onboarding.operational_entry, None);

        let existing = project_scale_input_for_goal("Modify the existing onboarding auth flow")
            .expect("existing system change should be classified");
        assert!(existing.existing_system_change);
        assert!(!existing.problem_unclear);
        assert!(!existing.product_scope_unclear);

        let supply_chain =
            project_scale_input_for_goal("Assess supply-chain risk before migration")
                .expect("supply-chain goal should be classified");
        assert_eq!(
            supply_chain.operational_entry,
            Some(ProjectScaleStageKind::SupplyChainAnalysis)
        );

        let security = project_scale_input_for_goal("Run security review for the auth boundary")
            .expect("security goal should be classified");
        assert_eq!(security.operational_entry, Some(ProjectScaleStageKind::SecurityAssessment));

        let incident = project_scale_input_for_goal("Handle incident follow up for auth outage")
            .expect("incident goal should be classified");
        assert_eq!(incident.operational_entry, Some(ProjectScaleStageKind::Incident));

        let migration = project_scale_input_for_goal("Migrate onboarding state to the new schema")
            .expect("migration goal should be classified");
        assert_eq!(migration.operational_entry, Some(ProjectScaleStageKind::Migration));

        let system_assessment =
            project_scale_input_for_goal("Assess the system before broad refactor")
                .expect("system assessment goal should be classified");
        assert_eq!(
            system_assessment.operational_entry,
            Some(ProjectScaleStageKind::SystemAssessment)
        );

        let platform_initiative = project_scale_input_for_goal(
            "Drive a platform initiative for the billing project rollout",
        )
        .expect("platform initiative should be classified as a broad goal");
        assert!(!platform_initiative.existing_system_change);
        assert!(platform_initiative.problem_unclear);
        assert!(platform_initiative.product_scope_unclear);
        assert!(platform_initiative.capability_structure_unclear);
        assert_eq!(platform_initiative.operational_entry, None);

        let long_goal = project_scale_input_for_goal(
            "Coordinate design notes across multiple teams before locking the delivery sequence",
        )
        .expect("long goals should be classified even without a named keyword");
        assert!(!long_goal.existing_system_change);
        assert!(long_goal.problem_unclear);
        assert!(long_goal.product_scope_unclear);
        assert!(!long_goal.capability_structure_unclear);
        assert_eq!(long_goal.operational_entry, None);

        assert_eq!(project_scale_input_for_goal("Fix typo"), None);
    }

    #[test]
    fn project_scale_state_uses_first_stage_and_work_unit_id() {
        let state = project_scale_state_for_goal(
            "Build a customer onboarding capability with audit logging",
            "confirm_project_scale_path",
        )
        .expect("broad goal should produce project-scale state");

        assert_eq!(state.active_stage_index, 0);
        assert_eq!(state.active_work_unit_id.as_deref(), Some("stage-001-discovery"));
        assert_eq!(state.next_action, "confirm_project_scale_path");
        assert_eq!(state.active_stage_text().as_deref(), Some("discovery"));
        assert!(state.path.stage_names().contains("pr-review"));
        assert!(state.checkpoint_refs.is_empty());
        assert!(state.trace_refs.is_empty());

        let security = project_scale_state_for_goal(
            "Run security review for the auth boundary",
            "repair_context",
        )
        .expect("security goal should produce project-scale state");
        assert_eq!(
            security.path.stages.first().map(|stage| stage.kind),
            Some(ProjectScaleStageKind::SecurityAssessment)
        );
        assert_eq!(security.next_action, "repair_context");

        assert_eq!(project_scale_state_for_goal("Fix typo", "repair_context"), None);
    }

    #[test]
    fn planning_context_sources_include_execution_profile_read_targets() {
        let workspace = write_governed_execution_profile_workspace(
            "boundline-runtime-execution-profile-targets",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: "Replace subtraction with addition".to_string(),
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left + right".to_string(),
                }],
                failure_mode: ExecutionFailureMode::Terminal,
            }],
            vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
            None,
        );

        let mut session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({})),
        );
        session.goal = Some("Fix the failing add test".to_string());

        let runtime = SessionRuntime::for_workspace(&workspace);
        let sources = runtime.planning_context_sources(&session, "Fix the failing add test");

        assert_eq!(
            sources.execution_profile_read_targets,
            vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()]
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn planning_context_sources_fall_back_to_project_memory_surfaces() {
        let workspace = temp_workspace("boundline-runtime-project-memory");
        let project_dir = workspace.join("docs/project");
        let evidence_dir = workspace.join("docs/evidence/architecture/run-123");
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(&evidence_dir).unwrap();

        fs::write(
            project_dir.join("architecture-map.md"),
            "# Architecture Map\n\nStable Canon context.\n",
        )
        .unwrap();
        fs::write(evidence_dir.join("architecture-overview.md"), "overview\n").unwrap();
        fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "run_id": "run-123",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "profile": "project-memory",
                "promotion_state": "auto-if-approved",
                "update_strategy": "managed-blocks",
                "lineage": {
                    "contract_version": "v1",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "mode": "architecture",
                    "promotion_state": "auto-if-approved",
                    "approval_state": "Completed",
                    "packet_readiness": "complete",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "promotion_profile": "project-memory",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let mut session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({})),
        );
        session.goal = Some("Plan the next bounded change".to_string());

        let runtime = SessionRuntime::for_workspace(&workspace);
        let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
        let memory = sources
            .compacted_canon_memory
            .expect("project memory should be compacted into planning sources");

        assert_eq!(memory.credibility, crate::domain::governance::MemoryCredibilityState::Credible);
        assert_eq!(memory.run_ref.as_deref(), Some("run-123"));
        assert!(memory.artifact_refs.contains(&"docs/project/architecture-map.md".to_string()));
        assert!(memory.artifact_refs.contains(&"docs/evidence/architecture/run-123".to_string()));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn planning_context_sources_rejects_future_project_memory_contract_line() {
        let workspace = temp_workspace("boundline-runtime-project-memory-guidance");
        let project_dir = workspace.join("docs/project");
        fs::create_dir_all(&project_dir).unwrap();

        fs::write(
            project_dir.join("architecture-map.md"),
            "# Architecture Map\n\nStable Canon context.\n",
        )
        .unwrap();
        fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "run_id": "run-123",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "profile": "project-memory",
                "promotion_state": "auto-if-approved",
                "update_strategy": "managed-blocks",
                "lineage": {
                    "contract_version": "v2",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "mode": "architecture",
                    "promotion_state": "auto-if-approved",
                    "approval_state": "Completed",
                    "packet_readiness": "complete",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "promotion_profile": "project-memory",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let mut session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({})),
        );
        session.goal = Some("Plan the next bounded change".to_string());

        let runtime = SessionRuntime::for_workspace(&workspace);
        let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
        let memory = sources
            .compacted_canon_memory
            .expect("unsupported project memory should still surface repair guidance");

        assert_eq!(
            memory.credibility,
            crate::domain::governance::MemoryCredibilityState::Insufficient
        );
        assert_eq!(memory.reason_code.as_deref(), Some("project_memory_contract_incompatible"));
        assert_eq!(
            memory.recommended_next_action.as_ref().map(|action| action.action.as_str()),
            Some("update")
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn planning_context_sources_block_on_incompatible_project_memory_contract() {
        let workspace = temp_workspace("boundline-runtime-project-memory-incompatible");
        let project_dir = workspace.join("docs/project");
        fs::create_dir_all(&project_dir).unwrap();

        fs::write(
            project_dir.join("architecture-map.md"),
            "# Architecture Map\n\nIncompatible Canon context.\n",
        )
        .unwrap();
        fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&json!({
                "run_id": "run-999",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "profile": "project-memory",
                "promotion_state": "auto-if-approved",
                "update_strategy": "managed-blocks",
                "lineage": {
                    "contract_version": "v2",
                    "producer": "canon",
                    "source_ref": "canon-run:run-999",
                    "mode": "architecture",
                    "promotion_state": "auto-if-approved",
                    "approval_state": "Completed",
                    "packet_readiness": "complete",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:def456",
                    "promotion_profile": "project-memory",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let mut session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({})),
        );
        session.goal = Some("Plan the next bounded change".to_string());

        let runtime = SessionRuntime::for_workspace(&workspace);
        let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
        let memory = sources
            .compacted_canon_memory
            .expect("incompatible project memory should still surface repair guidance");

        assert_eq!(
            memory.credibility,
            crate::domain::governance::MemoryCredibilityState::Insufficient
        );
        assert_eq!(memory.reason_code.as_deref(), Some("project_memory_contract_incompatible"));
        assert_eq!(
            memory.recommended_next_action.as_ref().map(|action| action.action.as_str()),
            Some("update")
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn compacted_project_memory_maps_non_credible_states_to_actions() {
        let workspace = temp_workspace("boundline-runtime-project-memory-states");
        let cases = [
            (
                PromotionStateView::PendingOrIndex,
                "Canon project memory is pending",
                "project_memory_pending",
                "refresh",
            ),
            (
                PromotionStateView::EvidenceOnly,
                "Canon project memory is evidence-only",
                "project_memory_evidence_only",
                "promote",
            ),
            (
                PromotionStateView::Manual,
                "Canon project memory requires manual promotion",
                "project_memory_manual",
                "promote",
            ),
            (
                PromotionStateView::Unknown,
                "Canon project memory metadata is incomplete",
                "project_memory_unknown",
                "inspect",
            ),
        ];

        for (state, headline, reason_code, action) in cases {
            let context = ProjectMemoryContext {
                status: ProjectMemoryStatus::Available,
                compatibility: Some(CompatibilityOutcome::Compatible),
                surfaces: vec![ProjectMemorySurface {
                    path: PathBuf::from("docs/project/overview.md"),
                    lineage: Some(sample_project_memory_lineage("run-123", "architecture")),
                    promotion_view: state,
                    category: "overview".to_string(),
                }],
                evidence_refs: Vec::new(),
                effective_promotion_state: Some(state),
            };

            let memory = SessionRuntime::compacted_canon_memory_from_project_memory_context(
                &workspace, &context,
            )
            .expect("non-credible project memory should still compact");

            assert_eq!(memory.headline, headline);
            assert_eq!(
                memory.credibility,
                crate::domain::governance::MemoryCredibilityState::Stale
            );
            assert_eq!(memory.reason_code.as_deref(), Some(reason_code));
            assert_eq!(memory.run_ref.as_deref(), Some("run-123"));
            assert_eq!(memory.possible_actions[0].action, action);
            assert_eq!(
                memory.recommended_next_action.as_ref().map(|next| next.action.as_str()),
                Some(action)
            );
            assert_eq!(
                memory
                    .evidence_summary
                    .as_ref()
                    .map(|summary| summary.artifact_provenance_links.clone()),
                Some(vec!["docs/project/overview.md".to_string()])
            );
        }

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn compacted_project_memory_maps_hard_stop_states_to_actions() {
        let workspace = temp_workspace("boundline-runtime-project-memory-hard-stops");
        let cases = [
            (
                {
                    let mut lineage = sample_project_memory_lineage("run-awaiting", "architecture");
                    lineage.promotion_state = "auto-if-approved".to_string();
                    lineage.approval_state = Some("requested".to_string());
                    lineage.packet_readiness = Some("pending".to_string());
                    lineage
                },
                PromotionStateView::PendingOrIndex,
                "Canon project memory is waiting for required approval",
                "project_memory_missing_approval",
                "approve",
            ),
            (
                {
                    let mut lineage = sample_project_memory_lineage("run-blocked", "architecture");
                    lineage.promotion_state = "auto-if-approved".to_string();
                    lineage.approval_state = Some("rejected".to_string());
                    lineage.packet_readiness = Some("rejected".to_string());
                    lineage
                },
                PromotionStateView::PendingOrIndex,
                "Canon project memory reports blocked governance",
                "project_memory_blocked",
                "unblock",
            ),
            (
                {
                    let mut lineage =
                        sample_project_memory_lineage("run-missing-artifact", "architecture");
                    lineage.source_artifacts = vec!["architecture-overview.md".to_string()];
                    lineage
                },
                PromotionStateView::Stable,
                "Canon project memory is missing required source artifacts",
                "project_memory_missing_source_artifacts",
                "restore",
            ),
        ];

        for (lineage, state, headline, reason_code, action) in cases {
            let context = ProjectMemoryContext {
                status: ProjectMemoryStatus::Available,
                compatibility: Some(CompatibilityOutcome::Compatible),
                surfaces: vec![ProjectMemorySurface {
                    path: PathBuf::from("docs/project/overview.md"),
                    lineage: Some(lineage),
                    promotion_view: state,
                    category: "overview".to_string(),
                }],
                evidence_refs: Vec::new(),
                effective_promotion_state: Some(state),
            };

            let memory = SessionRuntime::compacted_canon_memory_from_project_memory_context(
                &workspace, &context,
            )
            .expect("hard-stop project memory should still compact");

            assert_eq!(memory.headline, headline);
            assert_eq!(
                memory.credibility,
                crate::domain::governance::MemoryCredibilityState::Insufficient
            );
            assert_eq!(memory.reason_code.as_deref(), Some(reason_code));
            assert_eq!(memory.possible_actions[0].action, action);
            assert_eq!(
                memory.recommended_next_action.as_ref().map(|next| next.action.as_str()),
                Some(action)
            );
        }

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn project_memory_artifact_refs_skip_missing_and_duplicate_evidence_roots() {
        let workspace = temp_workspace("boundline-runtime-project-memory-artifact-refs");
        fs::create_dir_all(workspace.join("docs/evidence/architecture/run-123")).unwrap();

        let existing_lineage = sample_project_memory_lineage("run-123", "architecture");
        let missing_lineage = LineageRef {
            source_ref: "canon-run:run-missing".to_string(),
            ..existing_lineage.clone()
        };

        let context = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: PathBuf::from("docs/project/architecture-map.md"),
                lineage: None,
                promotion_view: PromotionStateView::Stable,
                category: "architecture-map".to_string(),
            }],
            evidence_refs: vec![existing_lineage.clone(), existing_lineage, missing_lineage],
            effective_promotion_state: Some(PromotionStateView::Stable),
        };

        let refs = SessionRuntime::project_memory_artifact_refs(&workspace, &context);

        assert_eq!(
            refs,
            vec![
                "docs/project/architecture-map.md".to_string(),
                "docs/evidence/architecture/run-123".to_string(),
            ]
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn compacted_project_memory_carries_managed_block_attribution() {
        let workspace = temp_workspace("boundline-runtime-project-memory-managed-blocks");
        let evidence_dir = workspace.join("docs/evidence/architecture/run-123");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(
            evidence_dir.join("verification.md"),
            concat!(
                "<!-- project-memory:managed:start producer=\"canon\" source_ref=\"canon-run:run-123\" contract_version=\"v1\" -->\n",
                "Canon evidence\n",
                "<!-- project-memory:managed:end -->\n",
                "<!-- project-memory:managed:start producer=\"boundline\" source_ref=\"trace-9\" contract_version=\"v1\" -->\n",
                "Boundline evidence\n",
                "<!-- project-memory:managed:end -->\n"
            ),
        )
        .unwrap();

        let lineage = sample_project_memory_lineage("run-123", "architecture");
        let context = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: PathBuf::from("docs/project/overview.md"),
                lineage: Some(lineage.clone()),
                promotion_view: PromotionStateView::Stable,
                category: "overview".to_string(),
            }],
            evidence_refs: vec![lineage],
            effective_promotion_state: Some(PromotionStateView::Stable),
        };

        let memory = SessionRuntime::compacted_canon_memory_from_project_memory_context(
            &workspace, &context,
        )
        .expect("project memory with evidence attribution should compact");

        let carried_forward_items = memory
            .evidence_summary
            .as_ref()
            .map(|summary| summary.carried_forward_items.clone())
            .unwrap_or_default();
        assert_eq!(carried_forward_items.len(), 2);
        assert!(carried_forward_items.iter().any(|summary| summary.contains("producer=canon")));
        assert!(carried_forward_items.iter().any(|summary| summary.contains("producer=boundline")));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn execute_step_routes_agent_tool_and_decision_edge_cases() {
        let runtime = SessionRuntime::for_workspace(temp_workspace("boundline-runtime-routing"));
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
            runtime.workspace_ref().join(".boundline/session.json")
        );
        assert_eq!(runtime.trace_store().root(), runtime.workspace_ref().join(".boundline/traces"));
        assert_eq!(session_status_for_task_status(TaskStatus::Aborted), SessionStatus::Aborted);

        let mut workspace_routing = RoutingConfig {
            assistant_runtimes: vec![RuntimeKind::Copilot],
            ..RoutingConfig::default()
        };
        let mut cluster_routing = RoutingConfig {
            assistant_runtimes: vec![RuntimeKind::Codex],
            ..RoutingConfig::default()
        };
        let global_routing = RoutingConfig {
            assistant_runtimes: vec![RuntimeKind::Claude],
            ..RoutingConfig::default()
        };
        assert_eq!(
            effective_assistant_runtimes(
                Some(&workspace_routing),
                Some(&cluster_routing),
                Some(&global_routing)
            ),
            vec![RuntimeKind::Copilot]
        );
        workspace_routing.assistant_runtimes.clear();
        assert_eq!(
            effective_assistant_runtimes(
                Some(&workspace_routing),
                Some(&cluster_routing),
                Some(&global_routing)
            ),
            vec![RuntimeKind::Codex]
        );
        cluster_routing.assistant_runtimes.clear();
        assert_eq!(
            effective_assistant_runtimes(
                Some(&workspace_routing),
                Some(&cluster_routing),
                Some(&global_routing)
            ),
            vec![RuntimeKind::Claude]
        );

        let cluster_ready = write_execution_profile_workspace(
            "boundline-runtime-cluster-ready",
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
        fs::create_dir_all(cluster_ready.join("src")).unwrap();
        fs::write(cluster_ready.join("src/lib.rs"), "left - right\n").unwrap();
        assert!(!cluster_workspace_is_blocked(cluster_ready.to_string_lossy().as_ref()));
        assert!(cluster_workspace_is_blocked(
            temp_workspace("boundline-runtime-cluster-blocked").to_string_lossy().as_ref()
        ));
        assert_eq!(cluster_task_status_text(TaskStatus::Exhausted), "exhausted");
        assert!(is_governance_trace_event(TraceEventType::GovernanceBlocked));
        assert!(!is_governance_trace_event(TraceEventType::TaskStarted));
    }

    #[test]
    fn load_or_create_trace_and_flow_helpers_cover_private_flow_branches() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-flow-helpers",
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
            negotiation_packet: None,
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
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
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
    fn session_lifecycle_helpers_cover_capture_selection_planning_and_cluster_projection() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-lifecycle-helpers",
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
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "left - right\n").unwrap();
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: None,
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: vec![crate::domain::decision::Decision::new(
                crate::domain::decision::DecisionType::Analyze,
                "src/lib.rs",
                "inspect the file",
                "bounded context collected",
                Vec::new(),
            )],
            active_flow_policy: None,
            latest_status: SessionStatus::Initialized,
            latest_terminal_reason: Some(TerminalReason::new(
                TerminalCondition::TaskNotCredible,
                "stale",
                None,
            )),
            latest_trace_ref: Some("trace.json".to_string()),
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        assert!(matches!(
            runtime.capture_goal(&mut session, "   "),
            Err(super::SessionRuntimeError::MissingGoal)
        ));

        runtime.capture_goal(&mut session, "  Drive a session runtime branch  ").unwrap();
        assert_eq!(session.goal.as_deref(), Some("Drive a session runtime branch"));
        assert!(session.negotiation_packet.is_some());
        assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
        assert!(session.decisions.is_empty());
        assert!(session.latest_terminal_reason.is_none());
        assert!(session.latest_trace_ref.is_none());

        runtime.select_flow(&mut session, "bug-fix").unwrap();
        assert_eq!(session.active_flow.as_ref().unwrap().flow_name, "bug-fix");
        assert!(session.active_flow_policy.is_some());
        assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
        assert!(!runtime.uses_native_goal_plan(&session).unwrap());
        assert!(matches!(
            runtime.confirm_goal_plan(&mut session),
            Err(super::SessionRuntimeError::MissingGoalPlan)
        ));

        assert!(matches!(
            runtime.plan_task(&mut session, Some("missing"), false),
            Err(super::SessionRuntimeError::UnknownFlow { .. })
        ));

        session.active_task =
            Some(decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})));
        let previous_task_id = session.active_task.as_ref().unwrap().id.clone();
        runtime.plan_task(&mut session, None, false).unwrap();
        assert_ne!(session.active_task.as_ref().unwrap().id, previous_task_id);
        assert!(matches!(
            runtime.select_flow(&mut session, "delivery"),
            Err(super::SessionRuntimeError::FlowReplacementRequiresReset { .. })
        ));

        session.active_task = None;
        session.goal_plan = Some(
            GoalPlan::new(
                "Drive a session runtime branch",
                vec![PlannedTask {
                    task_id: "planned-task-1".to_string(),
                    description: "Repair arithmetic".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some("tests pass".to_string()),
                    decision_type_hint: None,
                }],
            )
            .unwrap(),
        );
        assert!(matches!(
            runtime.select_flow(&mut session, "delivery"),
            Err(super::SessionRuntimeError::FlowReplacementRequiresReset { .. })
        ));

        runtime.confirm_goal_plan(&mut session).unwrap();
        assert!(!session.goal_plan.as_ref().unwrap().requires_confirmation());
        assert_eq!(session.latest_status, SessionStatus::Planned);
        assert!(runtime.uses_native_goal_plan(&session).unwrap());

        session.active_task =
            Some(decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})));
        let projection = ClusterSessionProjection {
            cluster_id: "cluster-1".to_string(),
            primary_workspace_ref: workspace.to_string_lossy().into_owned(),
            member_workspace_refs: vec![workspace.to_string_lossy().into_owned()],
            started_from_command: "boundline cluster status".to_string(),
            updated_at: 10,
        };
        runtime.prepare_cluster_run(&mut session, &projection).unwrap();
        assert_eq!(
            session
                .active_task
                .as_ref()
                .unwrap()
                .context
                .cluster_session_projection()
                .unwrap()
                .unwrap(),
            projection
        );
        assert_eq!(
            session.goal_plan.as_ref().unwrap().cluster_session_projection.as_ref().unwrap(),
            &projection
        );
    }

    #[test]
    fn broad_goal_planning_persists_project_scale_state_when_context_is_insufficient() {
        let workspace = temp_workspace("boundline-runtime-project-scale-clarify");
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
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
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        runtime
            .capture_goal(&mut session, "Build a customer onboarding capability with audit logging")
            .unwrap();

        let error = runtime.plan_task(&mut session, None, false).unwrap_err();
        let rendered_error = error.to_string();
        let prompt = rendered_error
            .strip_prefix(
                "active session requires clarification before planning can continue: bounded context required before planning: ",
            )
            .expect("plan_task should return clarification-required details");
        assert!(!prompt.trim().is_empty());

        assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
        assert!(session.goal_plan.is_some());
        let project_scale = session.project_scale.expect("project scale state should be persisted");
        assert_eq!(project_scale.next_action, "repair_context");
        assert_eq!(project_scale.active_stage_text().as_deref(), Some("discovery"));
        assert!(project_scale.path.stage_names().contains("pr-review"));
    }

    #[test]
    fn execute_next_step_covers_retry_replan_and_terminal_decision_recovery() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-decision-recovery",
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

        let mut exhausted_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"terminal_failure": true})),
        );
        let max_steps = exhausted_session.active_task.as_ref().unwrap().limits.max_steps;
        exhausted_session.active_task.as_mut().unwrap().total_step_attempts = max_steps;
        let exhausted = runtime.run_to_terminal(&mut exhausted_session).unwrap();
        assert_eq!(exhausted.terminal_status, TaskStatus::Exhausted);
        assert_eq!(exhausted.terminal_reason.condition, TerminalCondition::StepLimitExceeded);

        let mut no_step_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})),
        );
        let no_step_task = no_step_session.active_task.as_mut().unwrap();
        no_step_task.plan.current_step_index = no_step_task.plan.steps.len();
        let no_step = runtime.run_to_terminal(&mut no_step_session).unwrap();
        assert_eq!(no_step.terminal_status, TaskStatus::Failed);
        assert_eq!(no_step.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);

        let mut terminal_response_session = build_session(
            &workspace,
            decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})),
        );
        let terminal_task = terminal_response_session.active_task.as_ref().unwrap().clone();
        runtime.load_or_create_trace(&mut terminal_response_session, &terminal_task).unwrap();
        terminal_response_session.active_task.as_mut().unwrap().apply_terminal(
            TaskStatus::Succeeded,
            TerminalReason::new(TerminalCondition::GoalSatisfied, "already complete", None),
        );
        let terminal_response = runtime.run_to_terminal(&mut terminal_response_session).unwrap();
        assert_eq!(terminal_response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(terminal_response.terminal_reason.message, "already complete");
    }

    #[test]
    fn execute_next_step_creates_a_compatibility_task_for_flow_selected_goal_plans() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-compatibility-goal-plan",
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
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "left - right\n").unwrap();
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut goal_plan = GoalPlan::new(
            "Drive a session runtime branch",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Repair arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap();
        goal_plan.confirm().unwrap();

        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Drive a session runtime branch".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
            active_task: None,
            goal_plan: Some(goal_plan),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        runtime.execute_next_step(&mut session).unwrap();

        assert!(session.active_task.is_some());
        assert_eq!(session.latest_status, SessionStatus::Running);
        assert!(session.goal_plan.is_some());
    }

    #[test]
    fn native_goal_plan_short_circuits_for_existing_delegation_continuity() {
        let workspace = temp_workspace("boundline-runtime-native-delegation");
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut goal_plan = GoalPlan::new(
            "Drive a delegated continuation boundary",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Inspect the delegated boundary".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status explains the continuity boundary".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap();
        goal_plan.confirm().unwrap();
        goal_plan = goal_plan
            .with_delegation_state(
                Vec::new(),
                DelegationContinuityState {
                    active_packet_id: None,
                    mode: DelegationContinuityMode::InspectOnly,
                    authority_source: ContinuityAuthority::NativeSession,
                    next_command: "boundline inspect".to_string(),
                    headline: "delegated continuity requires operator inspection".to_string(),
                    evidence_summary: "bounded continuation stopped at an inspect-only boundary"
                        .to_string(),
                },
            )
            .unwrap();

        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Drive a delegated continuation boundary".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(goal_plan),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        let response = runtime.run_to_terminal(&mut session).unwrap();

        assert_eq!(response.terminal_status, TaskStatus::Failed);
        assert_eq!(response.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);
        assert_eq!(session.latest_status, SessionStatus::Planned);
        assert!(session.goal_plan.as_ref().unwrap().delegation_continuity().is_some());
        assert!(session.active_task.is_none());
    }

    #[test]
    fn execute_next_step_falls_back_to_local_governance_when_canon_is_optional() {
        let workspace = write_governed_execution_profile_workspace(
            "boundline-runtime-governance-local-fallback",
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
                    command: "canon-missing-for-test".to_string(),
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
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
        runtime.select_flow(&mut session, "bug-fix").unwrap();
        runtime.plan_task(&mut session, None, false).unwrap();
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
    fn native_persistence_projects_cluster_story_and_copies_changes() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-cluster-primary",
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
        let ready_member = write_execution_profile_workspace(
            "boundline-runtime-cluster-ready-member",
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
        let blocked_member = write_execution_profile_workspace(
            "boundline-runtime-cluster-blocked-member",
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
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(ready_member.join("src")).unwrap();
        fs::create_dir_all(blocked_member.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "left + right\n").unwrap();
        fs::write(ready_member.join("src/lib.rs"), "left - right\n").unwrap();
        fs::write(blocked_member.join("src/lib.rs"), "unchanged\n").unwrap();

        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut goal_plan = GoalPlan::new(
            "Deliver cluster follow-through",
            vec![PlannedTask {
                task_id: "planned-task-cluster".to_string(),
                description: "Propagate the bounded change".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("cluster state records the authoritative route".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap();
        goal_plan.confirm().unwrap();
        goal_plan.cluster_session_projection = Some(ClusterSessionProjection {
            cluster_id: "cluster-1".to_string(),
            primary_workspace_ref: workspace.to_string_lossy().into_owned(),
            member_workspace_refs: vec![
                workspace.to_string_lossy().into_owned(),
                ready_member.to_string_lossy().into_owned(),
                blocked_member.to_string_lossy().into_owned(),
            ],
            started_from_command: "boundline cluster status".to_string(),
            updated_at: 10,
        });

        let mut fixture_runtime = manual_runtime();
        fixture_runtime.profile.attempts = vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left - right".to_string(),
                replace: "left + right".to_string(),
            }],
        }];
        runtime.propagate_cluster_delivery_changes(&goal_plan, &fixture_runtime).unwrap();
        assert_eq!(fs::read_to_string(ready_member.join("src/lib.rs")).unwrap(), "left + right\n");
        assert_eq!(fs::read_to_string(blocked_member.join("src/lib.rs")).unwrap(), "unchanged\n");

        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Deliver cluster follow-through".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };
        let trace = ExecutionTrace::new("task-cluster", "session-runtime", "cluster goal");
        let response = runtime
            .persist_native_result(
                &mut session,
                goal_plan,
                Vec::new(),
                trace,
                super::NativePersistenceInput {
                    checkpoint_projection: None,
                    terminal_reason: TerminalReason::new(
                        TerminalCondition::GoalSatisfied,
                        "cluster goal satisfied",
                        None,
                    ),
                    limits: RunLimits::default(),
                    record_terminal_event: true,
                    projected_task: None,
                },
            )
            .unwrap();

        assert_eq!(response.terminal_status, TaskStatus::Failed);
        assert_eq!(session.latest_status, SessionStatus::Failed);
        let cluster_story =
            session.goal_plan.as_ref().unwrap().cluster_delivery_story.as_ref().unwrap();
        assert_eq!(cluster_story.execution_condition.kind, ClusteredExecutionKind::Failed);
        assert!(cluster_story.execution_condition.summary.contains("blocked by workspace"));
    }

    #[test]
    fn cluster_story_helper_covers_success_paused_failed_and_exhausted_states() {
        let primary = write_execution_profile_workspace(
            "boundline-runtime-cluster-story-primary",
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
        let member = write_execution_profile_workspace(
            "boundline-runtime-cluster-story-member",
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
        fs::create_dir_all(member.join("src")).unwrap();
        fs::write(member.join("src/lib.rs"), "left - right\n").unwrap();

        let runtime = SessionRuntime::for_workspace(&primary);
        let projection = ClusterSessionProjection {
            cluster_id: "cluster-1".to_string(),
            primary_workspace_ref: primary.to_string_lossy().into_owned(),
            member_workspace_refs: vec![
                primary.to_string_lossy().into_owned(),
                member.to_string_lossy().into_owned(),
            ],
            started_from_command: "boundline cluster status".to_string(),
            updated_at: 10,
        };

        let success = runtime.build_cluster_delivery_story(&projection, TaskStatus::Succeeded);
        assert_eq!(success.execution_condition.kind, ClusteredExecutionKind::Success);
        assert!(!success.execution_condition.recovery_allowed);
        assert_eq!(success.participating_workspaces[0].latest_status.as_deref(), Some("succeeded"));
        assert_eq!(
            success.participating_workspaces[1].participation_kind,
            crate::domain::cluster::WorkspaceParticipationKind::ReadOnly
        );

        let paused = runtime.build_cluster_delivery_story(&projection, TaskStatus::Running);
        assert_eq!(paused.execution_condition.kind, ClusteredExecutionKind::Paused);
        assert!(paused.execution_condition.recovery_allowed);

        let exhausted = runtime.build_cluster_delivery_story(&projection, TaskStatus::Exhausted);
        assert_eq!(exhausted.execution_condition.kind, ClusteredExecutionKind::Exhausted);

        let failed = runtime.build_cluster_delivery_story(&projection, TaskStatus::Aborted);
        assert_eq!(failed.execution_condition.kind, ClusteredExecutionKind::Failed);
        assert!(failed.execution_condition.recovery_allowed);
    }

    #[test]
    fn refresh_governance_state_handles_refreshable_and_non_refreshable_records() {
        let workspace = write_governed_execution_profile_workspace(
            "boundline-runtime-governance-refresh",
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
                canon: None,
                stages: vec![StageGovernancePolicy {
                    flow_name: "bug-fix".to_string(),
                    stage_id: "investigate".to_string(),
                    enabled: true,
                    required: false,
                    autopilot: false,
                    runtime: Some(GovernanceRuntimeKind::Local),
                    canon_mode: None,
                    system_context: Some(SystemContextBinding::Existing),
                    risk: None,
                    zone: None,
                    owner: None,
                }],
            }),
        );
        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Refresh governed stage".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
            active_task: Some(
                Task::new(
                    "task-govern-refresh",
                    &build_request(workspace.to_string_lossy().as_ref()),
                    Plan::new(vec![
                        Step::agent(
                            "investigate",
                            "analyzer",
                            attach_stage_metadata(
                                json!({"phase": "investigate"}),
                                built_in_flow("bug-fix").unwrap(),
                                0,
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    ])
                    .unwrap(),
                )
                .unwrap(),
            ),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Running,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };
        session
            .active_task
            .as_mut()
            .unwrap()
            .context
            .set_latest_governance_stage(&GovernedStageRecord {
                stage_key: "bug-fix:investigate".to_string(),
                runtime: GovernanceRuntimeKind::Local,
                lifecycle_state: GovernanceLifecycleState::AwaitingApproval,
                required: false,
                autopilot_enabled: false,
                approval_state: ApprovalState::Requested,
                canon_run_ref: None,
                governance_attempt_id: "attempt-1".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: None,
                decision_ref: None,
                blocked_reason: None,
            })
            .unwrap();

        assert!(runtime.refresh_governance_state(&mut session).unwrap());
        let refreshed = session
            .active_task
            .as_ref()
            .unwrap()
            .context
            .latest_governance_stage()
            .unwrap()
            .unwrap();
        assert_eq!(refreshed.lifecycle_state, GovernanceLifecycleState::GovernedReady);

        session
            .active_task
            .as_mut()
            .unwrap()
            .context
            .set_latest_governance_stage(&GovernedStageRecord {
                stage_key: "bug-fix:investigate".to_string(),
                runtime: GovernanceRuntimeKind::Local,
                lifecycle_state: GovernanceLifecycleState::Blocked,
                required: false,
                autopilot_enabled: false,
                approval_state: ApprovalState::NotNeeded,
                canon_run_ref: None,
                governance_attempt_id: "attempt-2".to_string(),
                previous_governance_attempt_id: Some("attempt-1".to_string()),
                packet_ref: None,
                decision_ref: None,
                blocked_reason: Some("still blocked".to_string()),
            })
            .unwrap();
        assert!(!runtime.refresh_governance_state(&mut session).unwrap());
    }

    #[test]
    fn execute_next_step_blocks_when_required_canon_governance_is_unavailable() {
        let workspace = write_governed_execution_profile_workspace(
            "boundline-runtime-governance-required-canon",
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
                    command: "canon-missing-for-test".to_string(),
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
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };

        runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
        runtime.select_flow(&mut session, "bug-fix").unwrap();
        runtime.plan_task(&mut session, None, false).unwrap();
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
    fn required_canon_governance_reports_missing_configuration_and_mode() {
        let workspace_missing_config =
            temp_workspace("boundline-runtime-governance-required-config");
        let runtime_missing_config = SessionRuntime::for_workspace(&workspace_missing_config);
        let mut missing_config_session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace_missing_config.to_string_lossy().into_owned(),
            goal: Some("Drive governed bug fix".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
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
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };
        let policy = StageGovernancePolicy {
            flow_name: "bug-fix".to_string(),
            stage_id: "investigate".to_string(),
            enabled: true,
            required: true,
            autopilot: false,
            runtime: Some(GovernanceRuntimeKind::Canon),
            canon_mode: Some(CanonMode::Discovery),
            system_context: Some(SystemContextBinding::Existing),
            risk: None,
            zone: None,
            owner: None,
        };
        let governance = GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: None,
            stages: vec![policy.clone()],
        };
        let mut fixture_runtime = manual_runtime();
        fixture_runtime.profile.read_targets = vec!["README.md".to_string()];
        let step = Step::agent(
            "investigate",
            "analyzer",
            attach_stage_metadata(
                json!({"phase": "investigate"}),
                built_in_flow("bug-fix").unwrap(),
                0,
            )
            .unwrap(),
        )
        .unwrap();
        let metadata = super::FlowStepMetadata::from_step(&step).unwrap().unwrap();
        let mut task = Task::new(
            "task-governance-config",
            &build_request(workspace_missing_config.to_string_lossy().as_ref()),
            Plan::new(vec![step.clone()]).unwrap(),
        )
        .unwrap();
        let mut trace = ExecutionTrace::new("task-governance-config", "session-runtime", "goal");

        let decision = runtime_missing_config
            .execute_governance_for_step(
                &mut missing_config_session,
                &mut task,
                &mut trace,
                &fixture_runtime,
                &step,
                &metadata,
                &governance,
                &policy,
                super::GovernanceRequestKind::Start,
            )
            .unwrap();
        match decision {
            super::GovernanceStepDecision::Terminal(response) => {
                assert!(response.terminal_reason.message.contains("requires Canon configuration"));
            }
            _ => panic!("expected terminal governance block"),
        }

        let workspace_missing_mode = write_governed_execution_profile_workspace(
            "boundline-runtime-governance-required-mode",
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
                    command: "true".to_string(),
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
                    canon_mode: None,
                    system_context: Some(SystemContextBinding::Existing),
                    risk: None,
                    zone: None,
                    owner: None,
                }],
            }),
        );
        let runtime_missing_mode = SessionRuntime::for_workspace(&workspace_missing_mode);
        let mut missing_mode_session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace_missing_mode.to_string_lossy().into_owned(),
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
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
        };
        runtime_missing_mode
            .capture_goal(&mut missing_mode_session, "Drive governed bug fix")
            .unwrap();
        runtime_missing_mode.select_flow(&mut missing_mode_session, "bug-fix").unwrap();
        runtime_missing_mode.plan_task(&mut missing_mode_session, None, false).unwrap();
        runtime_missing_mode.execute_next_step(&mut missing_mode_session).unwrap();
        assert!(
            missing_mode_session
                .latest_terminal_reason
                .as_ref()
                .unwrap()
                .message
                .contains("requires an explicit Canon mode")
        );
    }

    #[test]
    fn prepare_checkpoint_for_mutation_records_workspace_projection_on_task_context() {
        let workspace = write_execution_profile_workspace(
            "boundline-runtime-checkpoint-workspace",
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
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "left - right").unwrap();

        let task = decision_task(&workspace.to_string_lossy(), json!({"decision": "checkpoint"}));
        let mut session = build_session(&workspace, task);
        let runtime = SessionRuntime::for_workspace(&workspace);

        let projection = runtime
            .prepare_checkpoint_for_mutation(&mut session, SessionCommand::Step)
            .unwrap()
            .unwrap();

        assert_eq!(projection.scope, "workspace");
        assert_eq!(projection.workspace_refs, vec![workspace.to_string_lossy().into_owned()]);
        assert_eq!(
            session
                .active_task
                .as_ref()
                .unwrap()
                .context
                .state
                .get("latest_checkpoint_id")
                .and_then(|value| value.as_str()),
            Some(projection.checkpoint_id.as_str())
        );

        fs::write(workspace.join("src/lib.rs"), "left + right").unwrap();
        runtime.refresh_checkpoint_projection(&projection).unwrap();

        let manifest = runtime.checkpoint_store().load(&projection.checkpoint_id).unwrap().unwrap();
        assert_ne!(
            manifest.captured_files[0].captured_fingerprint,
            manifest.captured_files[0].observed_after_capture_fingerprint
        );
    }

    #[test]
    fn prepare_checkpoint_for_mutation_creates_grouped_cluster_checkpoints() {
        let primary = write_execution_profile_workspace(
            "boundline-runtime-checkpoint-primary",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-primary".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "before".to_string(),
                    replace: "after".to_string(),
                }],
            }],
        );
        let member = write_execution_profile_workspace(
            "boundline-runtime-checkpoint-member",
            vec![ExecutionAttemptDefinition {
                attempt_id: "fix-member".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/member.rs".to_string(),
                    find: "before".to_string(),
                    replace: "after".to_string(),
                }],
            }],
        );
        fs::create_dir_all(primary.join("src")).unwrap();
        fs::create_dir_all(member.join("src")).unwrap();
        fs::write(primary.join("src/lib.rs"), "before").unwrap();
        fs::write(member.join("src/member.rs"), "before").unwrap();

        let mut task =
            decision_task(&primary.to_string_lossy(), json!({"decision": "cluster-checkpoint"}));
        task.context
            .set_cluster_session_projection(&ClusterSessionProjection {
                cluster_id: "cluster-a".to_string(),
                primary_workspace_ref: primary.to_string_lossy().into_owned(),
                member_workspace_refs: vec![
                    primary.to_string_lossy().into_owned(),
                    member.to_string_lossy().into_owned(),
                ],
                started_from_command: "run".to_string(),
                updated_at: 1,
            })
            .unwrap();
        let mut session = build_session(&primary, task);
        let runtime = SessionRuntime::for_workspace(&primary);

        let projection = runtime
            .prepare_checkpoint_for_mutation(&mut session, SessionCommand::Run)
            .unwrap()
            .unwrap();

        assert_eq!(projection.scope, "cluster");
        assert_eq!(projection.workspace_refs.len(), 2);

        fs::write(primary.join("src/lib.rs"), "after").unwrap();
        fs::write(member.join("src/member.rs"), "after").unwrap();
        runtime.refresh_checkpoint_projection(&projection).unwrap();

        let primary_manifests = FileCheckpointStore::for_workspace(&primary)
            .load_group(&projection.checkpoint_id)
            .unwrap();
        let member_manifests = FileCheckpointStore::for_workspace(&member)
            .load_group(&projection.checkpoint_id)
            .unwrap();
        assert_eq!(primary_manifests.len(), 1);
        assert_eq!(member_manifests.len(), 1);
    }
}
