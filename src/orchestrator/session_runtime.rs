//! Workspace-scoped session orchestration for planning, execution, governance,
//! checkpoints, and persisted trace updates.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::audit_store::{
    FileSessionAuditStore, SessionAuditStore, SessionAuditStoreError,
};
use crate::adapters::checkpoint_store::{CheckpointStoreError, FileCheckpointStore};
use crate::adapters::governance_runtime::{
    CanonCliRuntime, GovernanceRequestKind, GovernanceRuntime, GovernanceRuntimeRequest,
    LocalGovernanceRuntime,
};
use crate::adapters::provider_runtime::{
    ProviderReviewDisposition, ProviderReviewRequest, ProviderRevisionRequest,
    ProviderWorkspaceFile, review_workspace, revise_artifact, route_is_available,
};
use serde::Serialize;
use serde_json::{Map, Value, json};
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::domain::audit::{
    SessionAuditActor, SessionAuditActorKind, SessionAuditAlgorithm, SessionAuditEntry,
    SessionAuditEntryKind, SessionAuditIdentity, SessionAuditOutcome, SessionAuditOutcomeStatus,
    SessionAuditPhase, SessionAuditSource, SessionAuditSourceKind,
};
use crate::domain::cluster::{
    ClusterDeliveryStory, ClusterRouteOwner, ClusterSessionProjection, ClusteredExecutionCondition,
    ClusteredExecutionKind, WorkspaceParticipationKind, WorkspaceParticipationRecord,
};
use crate::domain::configuration::{
    EffectiveRouting, EffortFallbackPolicy, ModelRoute, RouteSlot, RoutingConfig, RoutingOverrides,
    RuntimeKind, resolve_effective_routing, resolve_effective_runtime_capabilities,
    resolve_effective_slot_effort_policies,
};
use crate::domain::context_intelligence::AdvancedContextProjection;
use crate::domain::decision::{Decision, DecisionType};
use crate::domain::distribution::SUPPORTED_CANON_VERSION;
use crate::domain::flow::{FlowStepMetadata, built_in_flow, supported_flow_names_csv};
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::GoalPlan;
use crate::domain::governance::{
    ApprovalState, CanonAuthorityZone, CanonEvidenceInspectSummary, CanonIntendedPersona,
    CanonMode, CanonModeSelectionPreference, CanonPossibleActionSummary,
    CanonRecommendedActionSummary, CanonRiskClass, CompactedCanonMemory, CouncilProfile,
    GovernanceLifecycleState, GovernanceRuntimeKind, GovernedSessionLifecycle, GovernedStageRecord,
    MemoryCredibilityState, PacketReadiness, SystemContextBinding, execution_stage_key_for_mode,
    planned_canon_mode_sequence_for_flow, planning_canon_mode_for_stage_key,
    planning_canon_mode_sequence, planning_stage_brief_ref, planning_stage_key_for_mode,
    resolved_canon_mode,
};
use crate::domain::guidance::{CapabilityPhase, GuidanceGuardianProjection};
use crate::domain::limits::{RunLimits, TerminalCondition};
use crate::domain::negotiation::{NegotiatedDeliveryPacket, NegotiationResolutionState};
use crate::domain::project_memory::{
    ProjectMemoryCondition, ProjectMemoryContext, ProjectMemoryStatus,
    evidence_contribution_summaries, evidence_root_for_lineage, read_project_memory,
};
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
use crate::domain::review::{ReviewOutcome, ReviewProfile, ReviewTrigger};
use crate::domain::review::{
    ReviewerDefinition, ReviewerDisposition, ReviewerFinding, ReviewerParticipation,
    ReviewerParticipationStatus, VoteDecision, VoteRuleDefinition, resolve_council_assembly,
};
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::session::{
    ActiveSessionRecord, ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState,
    DelegationPacket, DelegationPacketKind, DelegationPacketState, DelegationStatusView,
    ProjectScaleSessionState, SessionCommand, SessionStatus, VotingSessionState,
    governance_next_action_for_state,
};
use crate::domain::stage_council::{
    StageCouncilAdjudication, StageCouncilArtifact, StageCouncilFinding,
    StageCouncilFindingDisposition, StageCouncilOutcome, StageCouncilRequest, StageCouncilStatus,
    StageCouncilVoteResolution,
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
    collect_workspace_signals,
};
use crate::orchestrator::governance::{
    GovernanceStepDecision, append_governed_document_to_lifecycle, bounded_governance_context,
    build_autopilot_decision, clarification_prompt_from_response, compacted_canon_memory_for_block,
    compacted_canon_memory_from_response, default_stage_canon_mode,
    enrich_bounded_context_with_accumulated, governance_input_documents,
    governance_projection_snapshot, governance_stage_key, governance_state_patch,
    governed_document_ref_from_response, overlay_stage_policy_with_intent,
    planning_governance_input_documents, requested_governance_intent, runtime_command_available,
    selected_stage_policy,
};
use crate::orchestrator::guidance_runtime::{
    GuardianExecutionRequest, execute_guardians_for_phase,
};
use crate::orchestrator::recovery::{RecoveryDecision, decide_recovery};
use crate::orchestrator::review_trace::{
    record_reasoning_profile_events, record_review_step_completed, record_review_step_started,
};
use crate::orchestrator::terminal::{build_terminal_reason, task_status_for_condition};

#[path = "session_runtime_checkpoint.rs"]
mod checkpoint;

#[path = "session_runtime_reasoning.rs"]
mod reasoning;

use checkpoint::{
    CheckpointProjectionState, apply_checkpoint_projection_to_context, checkpoint_event_payload,
    checkpoint_projection_from_context,
};
use reasoning::{
    GovernanceBlockContext, ReasoningGateContext, ReasoningTraceContext, is_governance_trace_event,
    reasoning_profile_block_message, store_latest_reasoning_profile,
};

#[cfg(test)]
use reasoning::{
    CURRENT_BOUNDLINE_VERSION, ReasoningIndependenceGaps, assess_reasoning_independence,
    count_distinct_participant_values, effective_routing_for_workspace,
    mark_reasoning_participants_completed, observed_reasoning_distinctness,
    reasoning_independence_reason, reasoning_outcome_for_activation,
    reasoning_participants_for_profile, reasoning_posture_for_activation, reasoning_route_for_role,
    reasoning_status_for_activation,
};

const NATIVE_REVIEWER_AGENT_NAME: &str = "reviewer";
const NATIVE_REVIEW_VOTER_TOOL_NAME: &str = "review-voter";
const NATIVE_REVIEW_FINALIZER_TOOL_NAME: &str = "review-finalizer";
const NATIVE_REVIEW_PHASE: &str = "review";
const NATIVE_REVIEW_VOTE_PHASE: &str = "review-vote";
const NATIVE_REVIEW_FINALIZE_PHASE: &str = "review-finalize";
const NATIVE_REVIEW_STEP_PREFIX: &str = "native-review";
const NATIVE_REVIEW_VOTE_STEP_ID: &str = "native-review-vote";
const NATIVE_REVIEW_FINALIZE_STEP_ID: &str = "native-review-finalize";
const LATEST_ATTEMPT_ID_KEY: &str = "latest_attempt_id";
const LATEST_CHANGED_FILES_KEY: &str = "latest_changed_files";
const LATEST_REVIEW_OUTCOME_KEY: &str = "latest_review_outcome";
const LATEST_VALIDATION_STATUS_KEY: &str = "latest_validation_status";
const NEXT_REVIEW_TRIGGER_KEY: &str = "next_review_trigger";
const VALIDATION_STATUS_PASSED: &str = "passed";
const VALIDATION_STATUS_FAILED: &str = "failed";
const PLANNING_STAGE_BRIEF_TITLE: &str = "# Canon Planning Stage Brief";
const PLANNING_STAGE_OUTPUT_LANGUAGE_HEADING: &str = "## Output Language";
const PLANNING_STAGE_OUTPUT_LANGUAGE_INSTRUCTION: &str = "Write all generated content in English, regardless of the language of the input text. Preserve proper nouns, product names, system names, and code identifiers exactly as provided in the input.";
const PLANNING_STAGE_OVERVIEW_HEADING: &str = "## Stage Overview";
const PLANNING_STAGE_CONTEXT_HEADING: &str = "## Bounded Context";
const PLANNING_STAGE_AUTHORED_INPUTS_HEADING: &str = "## Authored Inputs";
const PLANNING_STAGE_CANON_MEMORY_HEADING: &str = "## Canon Memory";
const PLANNING_STAGE_WORKFLOW_HEADING: &str = "## Workflow";
const PLANNING_DEFAULT_TARGET: &str = "workspace";
const PLANNING_UNSPECIFIED_FLOW: &str = "unspecified";
const SYSTEM_CONTEXT_NEW_TEXT: &str = "new";
const SYSTEM_CONTEXT_EXISTING_TEXT: &str = "existing";

// Well-known upstream Canon artifact file names used to enrich downstream
// planning briefs with substantive content from completed stages.
const UPSTREAM_SYSTEM_SHAPE_FILE: &str = "01-system-shape.md";
const UPSTREAM_DOMAIN_MODEL_FILE: &str = "02-domain-model.md";
const UPSTREAM_CONSTRAINTS_FILE: &str = "02-constraints.md";
const UPSTREAM_ARCHITECTURE_DECISIONS_FILE: &str = "02-architecture-decisions.md";
const UPSTREAM_PRD_FILE: &str = "07-prd.md";
const UPSTREAM_SCOPE_CUTS_FILE: &str = "05-scope-cuts.md";
const UPSTREAM_EVIDENCE_MAX_CHARS: usize = 4000;
const EXECUTION_GOVERNANCE_ROOT: &str = ".boundline/governance/execution";
const EXECUTION_STAGE_BRIEF_FILE_NAME: &str = "brief.md";
const EXECUTION_BRIEF_MAX_DECISIONS: usize = 8;

/// Computes a stable fingerprint of the inputs that determine planning
/// governance outcomes: the goal text and the authored brief content.
/// Used to skip redundant Canon invocations when the inputs have not changed.
fn compute_planning_input_fingerprint(goal: &str, session: &ActiveSessionRecord) -> String {
    let mut hasher = DefaultHasher::new();
    goal.hash(&mut hasher);
    if let Some(brief) = session.authored_brief.as_ref() {
        for source in &brief.sources {
            source.content.hash(&mut hasher);
        }
    }
    if let Some(packet) = session.negotiation_packet.as_ref() {
        packet.resolution_state.as_str().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

/// Returns `true` when the authored brief supplies enough content for Canon
/// to govern the given planning mode meaningfully.  Discovery is always
/// sufficient because its purpose is to gather information.
fn planning_brief_has_sufficient_content(
    context_sources: &PlanningContextSources,
    mode: Option<CanonMode>,
) -> bool {
    match mode {
        Some(CanonMode::Discovery) | None => true,
        Some(
            CanonMode::Requirements
            | CanonMode::SystemShaping
            | CanonMode::Architecture
            | CanonMode::Backlog,
        ) => !context_sources.authored_input_documents.is_empty(),
        Some(_) => true,
    }
}

/// Per-mode message describing what authored content Canon needs before the
/// governance stage can proceed.
fn planning_brief_insufficiency_reason(mode: Option<CanonMode>) -> String {
    match mode {
        Some(CanonMode::Requirements) => {
            "the authored brief does not contain enough content for requirements governance; \
             author a plan describing the problem statement, outcome, constraints, options, \
             tradeoffs, scope cuts, and decision checklist before proceeding"
                .to_string()
        }
        Some(CanonMode::SystemShaping) => {
            "the authored brief does not contain enough content for system-shaping governance; \
             author system context, integration boundaries, and shaping constraints before proceeding"
                .to_string()
        }
        Some(CanonMode::Architecture) => {
            "the authored brief does not contain enough content for architecture governance; \
             author the system structure, component boundaries, and technology decisions before proceeding"
                .to_string()
        }
        Some(CanonMode::Backlog) => {
            "the authored brief does not contain enough content for backlog governance; \
             author the delivery sequence, task breakdown, and acceptance criteria before proceeding"
                .to_string()
        }
        _ => {
            "the authored brief does not contain enough content for planning governance; \
             author the required sections before proceeding"
                .to_string()
        }
    }
}

/// Workspace-scoped orchestrator that coordinates session persistence,
/// checkpoint capture, trace updates, and the handoff between native and
/// compatibility execution paths.
#[derive(Debug, Clone)]
pub struct SessionRuntime {
    workspace_ref: PathBuf,
    checkpoint_store: FileCheckpointStore,
    session_store: FileSessionStore,
    trace_store: FileTraceStore,
}

#[derive(Default)]
struct NativeReviewExecution {
    events: Vec<TraceEvent>,
    terminal_reason: Option<TerminalReason>,
}

#[derive(Debug, Clone, Serialize)]
struct DelegationTraceDetails {
    delegation: Option<DelegationStatusView>,
}

#[derive(Debug, Clone)]
struct ResolvedPlanningGovernanceDefaults {
    system_context: SystemContextBinding,
    risk: String,
    zone: String,
    owner: String,
}

#[derive(Debug, Clone)]
struct PreparedPlanningGovernanceRequest {
    request: GovernanceRuntimeRequest,
    stage_council: Option<StageCouncilOutcome>,
}

#[derive(Debug, Clone)]
struct StageCouncilReviewerRoute {
    reviewer: ReviewerDefinition,
    route: ModelRoute,
}

// Persisted goal-plan projection written into trace payloads so inspect can
// reconstruct planning context, routing, and delegation without recomputing it.
#[derive(Debug, Clone, Serialize)]
struct GoalPlanTracePayload {
    plan_id: String,
    goal: String,
    task_count: usize,
    goal_plan_state: String,
    goal_plan_revision: usize,
    flow_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    advanced_context: Option<AdvancedContextProjection>,
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
            advanced_context: goal_plan
                .context_pack
                .as_ref()
                .and_then(|context_pack| context_pack.advanced_context.clone()),
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

const PROJECT_SCALE_BROAD_GOAL_CUES: &[&str] =
    &["capability", "project", "initiative", "platform", "onboarding", "architecture"];
const PROJECT_SCALE_EXISTING_SYSTEM_CHANGE_CUES: &[&str] =
    &["existing", "modify", "change", "update", "extend", "refactor", "fix"];
const PROJECT_SCALE_CONCRETE_DELIVERY_BUILD_CUES: &[&str] =
    &["implement", "build", "create", "deliver", "ship", "first slice"];
const PROJECT_SCALE_CONCRETE_DELIVERY_SHAPE_CUES: &[&str] = &[
    "microservice",
    "microservizio",
    "service",
    "api",
    "apis",
    "endpoint",
    "endpoints",
    "grpc",
    "rest",
    "oauth",
    "authorization",
    "authenticated",
    "authenticates",
    "role",
    "roles",
    "user management",
];
const PROJECT_SCALE_CAPABILITY_STRUCTURE_CUES: &[&str] = &["capability", "platform", "system"];
const PROJECT_SCALE_ARCHITECTURE_MATERIAL_CUES: &[&str] =
    &["architecture", "audit", "auth", "schema", "integration", "capability"];

fn project_scale_goal_contains_cue(lower: &str, cue: &str) -> bool {
    if cue.contains(' ') {
        return lower.contains(cue);
    }

    lower
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .any(|word| word == cue)
}

fn project_scale_goal_contains_any(lower: &str, cues: &[&str]) -> bool {
    cues.iter().any(|cue| project_scale_goal_contains_cue(lower, cue))
}

fn has_concrete_delivery_goal_shape(lower: &str) -> bool {
    project_scale_goal_contains_any(lower, PROJECT_SCALE_CONCRETE_DELIVERY_BUILD_CUES)
        && project_scale_goal_contains_any(lower, PROJECT_SCALE_CONCRETE_DELIVERY_SHAPE_CUES)
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

    let concrete_delivery_goal = has_concrete_delivery_goal_shape(&lower);

    let broad_goal = operational_entry.is_some()
        || concrete_delivery_goal
        || project_scale_goal_contains_any(&lower, PROJECT_SCALE_BROAD_GOAL_CUES)
        || lower.split_whitespace().count() >= 10;

    if !broad_goal {
        return None;
    }

    let existing_system_change = operational_entry.is_none()
        && project_scale_goal_contains_any(&lower, PROJECT_SCALE_EXISTING_SYSTEM_CHANGE_CUES);

    Some(ProjectScaleInput {
        goal: goal.to_string(),
        problem_unclear: !existing_system_change && !concrete_delivery_goal,
        product_scope_unclear: !existing_system_change,
        capability_structure_unclear: !concrete_delivery_goal
            && project_scale_goal_contains_any(&lower, PROJECT_SCALE_CAPABILITY_STRUCTURE_CUES),
        architecture_material: concrete_delivery_goal
            || project_scale_goal_contains_any(&lower, PROJECT_SCALE_ARCHITECTURE_MATERIAL_CUES),
        existing_system_change,
        operational_entry,
    })
}

impl SessionRuntime {
    /// Returns a runtime bound to one workspace and its persisted stores.
    pub fn for_workspace(workspace_ref: impl AsRef<Path>) -> Self {
        let workspace_ref = workspace_ref.as_ref().to_path_buf();
        Self {
            checkpoint_store: FileCheckpointStore::for_workspace(&workspace_ref),
            session_store: FileSessionStore::for_workspace(&workspace_ref),
            trace_store: FileTraceStore::for_workspace(&workspace_ref),
            workspace_ref,
        }
    }

    /// Returns the workspace this runtime operates on.
    pub fn workspace_ref(&self) -> &Path {
        &self.workspace_ref
    }

    /// Returns the session store used by this runtime.
    pub fn session_store(&self) -> &FileSessionStore {
        &self.session_store
    }

    /// Returns the checkpoint store used by this runtime.
    pub fn checkpoint_store(&self) -> &FileCheckpointStore {
        &self.checkpoint_store
    }

    /// Returns the trace store used by this runtime.
    pub fn trace_store(&self) -> &FileTraceStore {
        &self.trace_store
    }

    /// Loads the active workspace session, if one exists.
    pub fn load_session(&self) -> Result<Option<ActiveSessionRecord>, SessionRuntimeError> {
        self.session_store.load().map_err(SessionRuntimeError::SessionStore)
    }

    /// Persists the active session snapshot.
    pub fn persist_session(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<PathBuf, SessionRuntimeError> {
        let previous = self
            .session_store
            .load_session(&session.session_id)
            .map_err(SessionRuntimeError::SessionStore)?;
        let path =
            self.session_store.persist(session).map_err(SessionRuntimeError::SessionStore)?;
        self.sync_session_audit_lifecycle(previous.as_ref(), session)?;
        Ok(path)
    }

    /// Clears the active workspace session.
    pub fn clear_session(&self) -> Result<(), SessionRuntimeError> {
        self.session_store.clear().map_err(SessionRuntimeError::SessionStore)
    }

    fn sync_session_audit_lifecycle(
        &self,
        previous: Option<&ActiveSessionRecord>,
        session: &ActiveSessionRecord,
    ) -> Result<(), SessionRuntimeError> {
        let audit_store =
            FileSessionAuditStore::for_session(&self.workspace_ref, &session.session_id);
        let mut cursor =
            audit_store.load_cursor().map_err(SessionRuntimeError::SessionAuditStore)?;
        let session_identity = self.resolve_session_audit_identity();
        let system_actor = SessionAuditActor::system("boundline");
        let mut cursor_dirty = false;

        if !cursor.session_start_recorded {
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.created_at,
                SessionAuditEntryKind::SessionStart,
                "session started",
                session_identity.clone(),
                system_actor.clone(),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session opened"),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "workspace_ref": session.workspace_ref,
                    "goal": session.goal,
                    "latest_status": session_status_text(session.latest_status),
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.session_start_recorded = true;
            cursor_dirty = true;
        }

        let previous_status =
            previous.map(|record| session_status_text(record.latest_status).to_string());
        let current_status = session_status_text(session.latest_status).to_string();
        if cursor.latest_session_status.as_deref() != Some(current_status.as_str())
            || previous_status.as_deref() != Some(current_status.as_str())
        {
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.updated_at,
                SessionAuditEntryKind::SessionStatusChanged,
                format!("session status changed to {current_status}"),
                session_identity.clone(),
                system_actor.clone(),
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                session_audit_outcome_for_status(session.latest_status),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "previous_status": previous_status,
                    "current_status": current_status,
                    "terminal_reason": session.latest_terminal_reason,
                    "latest_trace_ref": session.latest_trace_ref,
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.latest_session_status = Some(current_status);
            cursor_dirty = true;
        }

        if session.latest_status.is_terminal() && !cursor.session_end_recorded {
            let terminal_message = session
                .latest_terminal_reason
                .as_ref()
                .map(|reason| reason.message.trim().to_string())
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| {
                    format!(
                        "session ended with status {}",
                        session_status_text(session.latest_status)
                    )
                });
            let entry = SessionAuditEntry::new_with_timestamp(
                session.session_id.clone(),
                cursor.next_sequence(),
                session.updated_at,
                SessionAuditEntryKind::SessionEnd,
                terminal_message.clone(),
                session_identity,
                system_actor,
                SessionAuditAlgorithm::new(
                    SessionAuditPhase::Session,
                    "session_runtime",
                    "persist_session",
                ),
                session_audit_outcome_for_status(session.latest_status),
                SessionAuditSource::session_lifecycle(),
                json!({
                    "latest_status": session_status_text(session.latest_status),
                    "terminal_reason": session.latest_terminal_reason,
                    "latest_trace_ref": session.latest_trace_ref,
                    "goal": session.goal,
                }),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.session_end_recorded = true;
            cursor_dirty = true;
        }

        if cursor_dirty {
            audit_store.persist_cursor(&cursor).map_err(SessionRuntimeError::SessionAuditStore)?;
        }

        Ok(())
    }

    fn resolve_session_audit_identity(&self) -> SessionAuditIdentity {
        SessionAuditIdentity {
            current_user: current_user_name(),
            git_user_name: git_config_value(&self.workspace_ref, "user.name"),
            git_user_email: git_config_value(&self.workspace_ref, "user.email"),
        }
    }

    fn project_trace_events_to_session_audit(
        &self,
        session_id: &str,
        trace_ref: &str,
        trace: &ExecutionTrace,
    ) -> Result<(), SessionRuntimeError> {
        let audit_store = FileSessionAuditStore::for_session(&self.workspace_ref, session_id);
        let mut cursor =
            audit_store.load_cursor().map_err(SessionRuntimeError::SessionAuditStore)?;
        let session_identity = self.resolve_session_audit_identity();
        let mut cursor_dirty = false;

        for event in &trace.events {
            if cursor.already_projected(&trace.task_id, &event.event_id) {
                continue;
            }

            let entry = SessionAuditEntry::new_with_timestamp(
                session_id.to_string(),
                cursor.next_sequence(),
                event.recorded_at,
                SessionAuditEntryKind::TraceEventProjected,
                trace_event_audit_message(event),
                session_identity.clone(),
                trace_event_audit_actor(event),
                trace_event_audit_algorithm(event.event_type),
                trace_event_audit_outcome(event),
                SessionAuditSource {
                    kind: SessionAuditSourceKind::TraceEvent,
                    trace_ref: Some(trace_ref.to_string()),
                    trace_event_id: Some(event.event_id.clone()),
                    trace_event_type: Some(trace_event_type_text(event.event_type)),
                    step_id: event.step_id.clone(),
                    plan_revision: Some(event.plan_revision),
                },
                event.payload.clone(),
            );
            audit_store.append(&entry).map_err(SessionRuntimeError::SessionAuditStore)?;
            cursor.mark_projected(trace.task_id.clone(), event.event_id.clone());
            cursor_dirty = true;
        }

        if cursor_dirty {
            audit_store.persist_cursor(&cursor).map_err(SessionRuntimeError::SessionAuditStore)?;
        }

        Ok(())
    }

    /// Returns the latest persisted trace for the workspace, if available.
    pub fn latest_trace(&self) -> Result<Option<PathBuf>, SessionRuntimeError> {
        self.trace_store.latest().map_err(SessionRuntimeError::TraceStore)
    }

    /// Captures a new goal into the session and resets any active execution
    /// state so planning can restart from a clean bounded snapshot.
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
        self.ensure_workspace_governance_lifecycle(session);
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
        if session.goal_plan.is_none() {
            return Err(SessionRuntimeError::MissingGoalPlan);
        }
        if let Some(stage_record) = self.unresolved_planning_governance_record(session) {
            return Err(SessionRuntimeError::PlanningGovernanceUnresolved {
                stage_key: stage_record.stage_key.clone(),
                state: stage_record.lifecycle_state,
                reason: stage_record.blocked_reason.clone().or_else(|| {
                    session.governance_lifecycle.as_ref().and_then(|l| l.terminal_reason.clone())
                }),
            });
        }

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

    /// Prepares cluster-scoped state before a clustered run starts.
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

    /// Returns true when the session is currently operating on a native goal
    /// plan instead of a compatibility task.
    pub fn uses_native_goal_plan(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<bool, SessionRuntimeError> {
        Ok(session.goal_plan.is_some())
    }

    /// Projects the effective routing outcome for the current session state.
    pub fn resolve_routing_outcome(
        &self,
        session: &ActiveSessionRecord,
    ) -> Result<crate::domain::session::RoutingOutcome, SessionRuntimeError> {
        Ok(crate::domain::session::routing_outcome(session))
    }

    // Builds a compatibility task when fixture execution remains the
    // authoritative runtime for the chosen flow.
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

    // Builds or refreshes the native goal plan, preserving partial planning
    // state when bounded context is still insufficient.
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
                session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
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
            session.project_scale = project_scale_state_for_goal(&goal, "repair_context");
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
        if requested_flow.is_some() || session.active_flow.is_some() || no_flow {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
        }

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

        session.active_flow = native_flow_state;
        session.active_task = None;
        session.goal_plan = Some(goal_plan);
        session.project_scale = project_scale_state;
        session.decisions.clear();
        session.active_flow_policy = preserved_flow_policy;
        session.latest_status =
            if planning_blocked { SessionStatus::Blocked } else { SessionStatus::Planned };
        session.latest_terminal_reason = None;
        session.latest_trace_ref = None;
        session.updated_at = current_timestamp_millis();

        Ok(())
    }

    fn ensure_workspace_governance_lifecycle(&self, session: &mut ActiveSessionRecord) {
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

        if lifecycle.planning_input_fingerprint.as_deref() == Some(new_fingerprint)
            && lifecycle
                .stage_records
                .iter()
                .any(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_some())
        {
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

    fn resolve_workspace_governance_runtime(
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

    fn sync_governed_planning_sequence(
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

    fn prepare_planning_governance_requests(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
    ) -> Result<Vec<PreparedPlanningGovernanceRequest>, SessionRuntimeError> {
        let Some(lifecycle) = session.governance_lifecycle.as_ref() else {
            return Ok(Vec::new());
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon
            || lifecycle.explicit_opt_out
        {
            return Ok(Vec::new());
        }

        planning_canon_mode_sequence(&lifecycle.selected_mode_sequence)
            .into_iter()
            .map(|mode| {
                self.build_planning_governance_request(session, goal_plan, context_sources, mode)
            })
            .collect()
    }

    fn execute_planning_governance_requests(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        requests: Vec<PreparedPlanningGovernanceRequest>,
        context_sources: &PlanningContextSources,
    ) -> Result<(), SessionRuntimeError> {
        if requests.is_empty() {
            return Ok(());
        }

        let canon = self.resolve_planning_canon_runtime();

        for (stage_index, prepared) in requests.into_iter().enumerate() {
            let mut request = prepared.request;
            if self.planning_stage_already_ready(session, &request.stage_key) {
                self.set_planning_stage_progress(session, stage_index + 1, None);
                continue;
            }

            if self.planning_stage_has_unresolved_gate(session, &request.stage_key) {
                self.set_planning_stage_progress(
                    session,
                    stage_index,
                    self.latest_planning_stage_reason(session),
                );
                break;
            }

            if !planning_brief_has_sufficient_content(context_sources, request.mode) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    planning_brief_insufficiency_reason(request.mode),
                    prepared.stage_council.clone(),
                );
                break;
            }

            self.set_planning_stage_progress(session, stage_index, None);

            if let Some(outcome) = prepared.stage_council.clone()
                && outcome.status == StageCouncilStatus::Blocked
            {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    planning_stage_council_block_reason(&request.stage_key, &outcome),
                    Some(outcome),
                );
                break;
            }

            let Some(canon) = canon.as_ref() else {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    "planning governance requires Canon initialization, but .boundline/execution.json is missing governance.canon"
                        .to_string(),
                    prepared.stage_council.clone(),
                );
                break;
            };

            enrich_bounded_context_with_accumulated(
                &mut request.bounded_context,
                &self.planning_accumulated_context(session),
            );

            if !runtime_command_available(&canon.command) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    format!(
                        "planning governance requires Canon, but command '{}' is unavailable",
                        canon.command
                    ),
                    prepared.stage_council.clone(),
                );
                break;
            }

            if let Some(reason) = canon_workspace_scope_mismatch_reason(&self.workspace_ref) {
                self.record_planning_governance_block(
                    session,
                    goal_plan,
                    &request,
                    stage_index,
                    reason,
                    prepared.stage_council.clone(),
                );
                break;
            }

            let response = CanonCliRuntime::new(canon.command.clone())
                .with_working_directory(&self.workspace_ref)
                .execute(&request)
                .map_err(|error| SessionRuntimeError::GovernanceRuntime(error.to_string()))?;

            let should_halt = self.record_planning_governance_response(
                session,
                goal_plan,
                &request,
                response,
                stage_index,
                prepared.stage_council,
            )?;
            if should_halt {
                break;
            }
        }

        Ok(())
    }

    fn resolve_planning_canon_runtime(
        &self,
    ) -> Option<crate::domain::governance::CanonRuntimeConfig> {
        load_workspace_execution_profile(&self.workspace_ref)
            .ok()
            .and_then(|profile| profile.governance.and_then(|governance| governance.canon))
    }

    fn planning_stage_already_ready(&self, session: &ActiveSessionRecord, stage_key: &str) -> bool {
        self.latest_planning_stage_record(session, stage_key).is_some_and(|record| {
            matches!(
                record.lifecycle_state,
                GovernanceLifecycleState::GovernedReady | GovernanceLifecycleState::Completed
            )
        })
    }

    fn planning_stage_has_unresolved_gate(
        &self,
        session: &ActiveSessionRecord,
        stage_key: &str,
    ) -> bool {
        self.latest_planning_stage_record(session, stage_key).is_some_and(|record| {
            matches!(
                record.lifecycle_state,
                GovernanceLifecycleState::AwaitingApproval
                    | GovernanceLifecycleState::Blocked
                    | GovernanceLifecycleState::Failed
            )
        })
    }

    fn latest_planning_stage_reason(&self, session: &ActiveSessionRecord) -> Option<String> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle
                .stage_records
                .iter()
                .rev()
                .find(|record| planning_canon_mode_for_stage_key(&record.stage_key).is_some())
                .and_then(|record| record.blocked_reason.clone())
                .or_else(|| lifecycle.terminal_reason.clone())
        })
    }

    fn planning_accumulated_context(
        &self,
        session: &ActiveSessionRecord,
    ) -> Vec<crate::domain::governance::GovernedDocumentRef> {
        session
            .governance_lifecycle
            .as_ref()
            .map(|lifecycle| lifecycle.accumulated_context.clone())
            .unwrap_or_default()
    }

    fn set_planning_stage_progress(
        &self,
        session: &mut ActiveSessionRecord,
        stage_index: usize,
        terminal_reason: Option<String>,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        lifecycle.current_stage_index = stage_index;
        lifecycle.terminal_reason = terminal_reason;
    }

    fn record_planning_governance_block(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        request: &GovernanceRuntimeRequest,
        stage_index: usize,
        reason: String,
        stage_council: Option<StageCouncilOutcome>,
    ) {
        let record = GovernedStageRecord {
            stage_key: request.stage_key.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::Blocked,
            required: true,
            autopilot_enabled: request.autopilot,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: request.governance_attempt_id.clone(),
            previous_governance_attempt_id: self
                .latest_planning_stage_record(session, &request.stage_key)
                .map(|record| record.governance_attempt_id.clone()),
            packet_ref: None,
            decision_ref: None,
            stage_council,
            blocked_reason: Some(reason.clone()),
        };

        self.upsert_planning_stage_record(session, record, stage_index, Some(reason.clone()));
        goal_plan.compacted_canon_memory = compacted_canon_memory_for_block(
            &request.stage_key,
            GovernanceRuntimeKind::Canon,
            &reason,
        );
    }

    fn record_planning_governance_response(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &mut GoalPlan,
        request: &GovernanceRuntimeRequest,
        response: crate::adapters::governance_runtime::GovernanceRuntimeResponse,
        stage_index: usize,
        stage_council: Option<StageCouncilOutcome>,
    ) -> Result<bool, SessionRuntimeError> {
        let packet_rejected = response.packet.as_ref().is_some_and(|packet| {
            matches!(packet.readiness, PacketReadiness::Incomplete | PacketReadiness::Rejected)
        });
        let effective_status =
            if packet_rejected { GovernanceLifecycleState::Blocked } else { response.status };
        let blocked_reason = if packet_rejected {
            Some(
                response
                    .packet
                    .as_ref()
                    .map(|packet| {
                        let detail = if !packet.missing_sections.is_empty() {
                            format!(": missing sections {}", packet.missing_sections.join(", "))
                        } else if !response.message.trim().is_empty() {
                            format!(": {}", response.message)
                        } else {
                            String::new()
                        };
                        format!(
                            "governance packet was {:?} for planning stage {}{}",
                            packet.readiness, request.stage_key, detail
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "governance packet was rejected for planning stage {}",
                            request.stage_key
                        )
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
            stage_key: request.stage_key.clone(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: effective_status,
            required: true,
            autopilot_enabled: request.autopilot,
            approval_state: response.approval_state,
            canon_run_ref: response.run_ref.clone(),
            governance_attempt_id: request.governance_attempt_id.clone(),
            previous_governance_attempt_id: self
                .latest_planning_stage_record(session, &request.stage_key)
                .map(|record| record.governance_attempt_id.clone()),
            packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
            decision_ref: None,
            stage_council,
            blocked_reason: blocked_reason.clone(),
        };

        self.upsert_planning_stage_record(session, record, stage_index, blocked_reason.clone());
        goal_plan.compacted_canon_memory = compacted_canon_memory_from_response(
            &request.stage_key,
            GovernanceRuntimeKind::Canon,
            &response,
        );

        if effective_status == GovernanceLifecycleState::GovernedReady
            && let Some(mode) = request.mode
        {
            let doc_ref = governed_document_ref_from_response(&request.stage_key, mode, &response);
            append_governed_document_to_lifecycle(session, doc_ref);
            self.set_planning_stage_progress(session, stage_index + 1, None);
            return Ok(false);
        }

        Ok(matches!(
            effective_status,
            GovernanceLifecycleState::AwaitingApproval
                | GovernanceLifecycleState::Blocked
                | GovernanceLifecycleState::Failed
        ))
    }

    fn latest_planning_stage_record<'a>(
        &self,
        session: &'a ActiveSessionRecord,
        stage_key: &str,
    ) -> Option<&'a GovernedStageRecord> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle.stage_records.iter().rev().find(|record| record.stage_key == stage_key)
        })
    }

    fn upsert_planning_stage_record(
        &self,
        session: &mut ActiveSessionRecord,
        record: GovernedStageRecord,
        stage_index: usize,
        terminal_reason: Option<String>,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if let Some(existing_index) = lifecycle
            .stage_records
            .iter()
            .position(|existing| existing.stage_key == record.stage_key)
        {
            lifecycle.stage_records[existing_index] = record;
        } else {
            lifecycle.stage_records.push(record);
        }
        lifecycle.current_stage_index = stage_index;
        lifecycle.terminal_reason = terminal_reason;
    }

    fn build_planning_governance_request(
        &self,
        session: &mut ActiveSessionRecord,
        goal_plan: &GoalPlan,
        context_sources: &PlanningContextSources,
        mode: CanonMode,
    ) -> Result<PreparedPlanningGovernanceRequest, SessionRuntimeError> {
        let stage_key = planning_stage_key_for_mode(mode).ok_or_else(|| {
            SessionRuntimeError::ExecutionInvariant(format!(
                "planning governance stage key is unavailable for Canon mode {}",
                mode.as_str()
            ))
        })?;
        let mut stage_brief_ref = self.materialize_planning_stage_brief(
            stage_key,
            mode,
            goal_plan,
            context_sources,
            &self.planning_accumulated_context(session),
        )?;
        let stage_council = if mode == CanonMode::Discovery {
            let council_request =
                discovery_stage_council_request(stage_key, &goal_plan.goal_text, &stage_brief_ref);
            let outcome = self.execute_discovery_stage_council(&council_request)?;
            session.latest_voting = Some(stage_council_voting_session_state(stage_key, &outcome));
            stage_brief_ref = outcome.revised_output.evidence_ref.clone();
            Some(outcome)
        } else {
            None
        };
        let defaults = self.resolve_planning_governance_defaults(session, mode)?;
        let input_documents = planning_governance_input_documents(
            session.authored_brief.as_ref(),
            &stage_brief_ref,
            goal_plan.compacted_canon_memory.as_ref(),
        );

        Ok(PreparedPlanningGovernanceRequest {
            request: GovernanceRuntimeRequest {
                request_kind: GovernanceRequestKind::Start,
                governance_attempt_id: Uuid::new_v4().to_string(),
                stage_key: stage_key.to_string(),
                goal: goal_plan.goal_text.clone(),
                workspace_ref: self.workspace_ref.to_string_lossy().into_owned(),
                autopilot: false,
                mode: Some(mode),
                system_context: Some(defaults.system_context),
                risk: Some(defaults.risk),
                zone: Some(defaults.zone),
                owner: Some(defaults.owner),
                run_ref: None,
                packet_ref: None,
                bounded_context: crate::adapters::governance_runtime::GovernanceBoundedContext {
                    read_targets: self.planning_governance_read_targets(goal_plan, context_sources),
                    stage_brief_ref: Some(stage_brief_ref),
                    reused_packets: Vec::new(),
                },
                input_documents,
            },
            stage_council,
        })
    }

    fn resolve_planning_governance_defaults(
        &self,
        session: &ActiveSessionRecord,
        mode: CanonMode,
    ) -> Result<ResolvedPlanningGovernanceDefaults, SessionRuntimeError> {
        let canon_preferences = FileConfigStore::for_workspace(&self.workspace_ref)
            .load_local()
            .ok()
            .flatten()
            .and_then(|config| config.canon);
        let governance_intent =
            session.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref());

        let system_context = canon_preferences
            .as_ref()
            .and_then(|prefs| prefs.default_system_context.as_deref())
            .and_then(parse_planning_system_context)
            .unwrap_or_else(|| default_planning_system_context(mode));
        let risk = governance_intent
            .and_then(|intent| intent.risk.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_risk.clone()))
            .map(|risk| {
                CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "risk"))?;
        let zone = governance_intent
            .and_then(|intent| intent.zone.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_zone.clone()))
            .map(|zone| {
                CanonAuthorityZone::canonicalize_label(&zone).map(str::to_string).unwrap_or(zone)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "zone"))?;
        let owner = governance_intent
            .and_then(|intent| intent.owner.clone())
            .or_else(|| canon_preferences.as_ref().and_then(|prefs| prefs.default_owner.clone()))
            .map(|owner| {
                CanonIntendedPersona::canonicalize_label(&owner)
                    .map(str::to_string)
                    .unwrap_or(owner)
            })
            .ok_or_else(|| missing_planning_governance_field(mode, "owner"))?;

        Ok(ResolvedPlanningGovernanceDefaults { system_context, risk, zone, owner })
    }

    fn planning_governance_read_targets(
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

    fn materialize_planning_stage_brief(
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

    /// Reads completed upstream Canon artifact files and renders a mode-specific
    /// section that provides substantive content for downstream planning briefs.
    ///
    /// Returns `None` when no upstream content is available (the brief remains
    /// identical to the previous behavior for backward compatibility).
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

    fn execute_discovery_stage_council(
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
        let producer_ref =
            self.write_stage_council_artifact(request, "producer", &current_artifact)?;
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

        let revised_ref =
            self.write_stage_council_artifact(request, "revised", &revised_artifact_text)?;
        let outcome = StageCouncilOutcome {
            producer_output,
            reviewer_findings: stage_findings,
            vote_resolution: StageCouncilVoteResolution {
                strategy: "bounded_majority".to_string(),
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

    fn planning_council_effective_routing(&self) -> EffectiveRouting {
        let workspace_routing =
            FileConfigStore::for_workspace(&self.workspace_ref).local_routing().ok().flatten();
        let cluster_routing = FileClusterStore::for_workspace(&self.workspace_ref)
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

    fn stage_council_blocked_outcome(
        &self,
        request: &StageCouncilRequest,
        producer_output: &StageCouncilArtifact,
        reason: &str,
        next_action: &str,
    ) -> Result<StageCouncilOutcome, SessionRuntimeError> {
        let revised_ref = self.write_stage_council_artifact(
            request,
            "blocked",
            &render_stage_council_blocked_note(reason),
        )?;
        let outcome = StageCouncilOutcome {
            producer_output: producer_output.clone(),
            reviewer_findings: Vec::new(),
            vote_resolution: StageCouncilVoteResolution {
                strategy: "bounded_majority".to_string(),
                accepted_findings: Vec::new(),
                rejected_findings: Vec::new(),
                independent_review: false,
            },
            adjudication: None,
            revised_output: StageCouncilArtifact {
                route_slot: request.producer_slot.clone(),
                evidence_ref: revised_ref,
                summary: Some("stage council blocked planning discovery".to_string()),
            },
            status: StageCouncilStatus::Blocked,
            next_action: next_action.to_string(),
        };
        outcome.validate().map_err(SessionRuntimeError::ExecutionInvariant)?;
        Ok(outcome)
    }

    fn write_stage_council_artifact(
        &self,
        request: &StageCouncilRequest,
        suffix: &str,
        contents: &str,
    ) -> Result<String, SessionRuntimeError> {
        let relative_ref =
            format!(".boundline/council/{}-{suffix}.md", request.stage_key.replace(':', "-"));
        let artifact_path = self.workspace_ref.join(&relative_ref);
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                SessionRuntimeError::GoalPlan(format!(
                    "failed to create council artifact directory {}: {error}",
                    parent.display()
                ))
            })?;
        }
        fs::write(&artifact_path, contents).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write stage council artifact {}: {error}",
                artifact_path.display()
            ))
        })?;
        Ok(relative_ref)
    }

    /// Advances the active session by exactly one bounded step.
    /// Flow-selected goal plans are bridged into compatibility tasks when a
    /// fixture execution profile remains authoritative.
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
            self.refresh_checkpoint_projection(session, projection)?;
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

    /// Continues the active session until it reaches a terminal response.
    /// Native goal-plan sessions use the native path; compatibility sessions
    /// continue one fixture step at a time until terminal.
    pub fn run_to_terminal(
        &self,
        session: &mut ActiveSessionRecord,
    ) -> Result<TaskRunResponse, SessionRuntimeError> {
        let checkpoint_projection =
            self.prepare_checkpoint_for_mutation(session, SessionCommand::Run)?;
        if session.goal_plan.is_some() {
            let response = self.run_native_goal_plan(session, checkpoint_projection.clone())?;
            if let Some(projection) = checkpoint_projection.as_ref() {
                self.refresh_checkpoint_projection(session, projection)?;
            }
            return Ok(response);
        }

        let runtime = self.build_runtime(session)?;

        loop {
            if let Some(response) = self.execute_single_step(session, &runtime)? {
                if let Some(projection) = checkpoint_projection.as_ref() {
                    self.refresh_checkpoint_projection(session, projection)?;
                }
                return Ok(response);
            }
        }
    }

    /// Refreshes governance state when a run is paused awaiting approval and a
    /// governance runtime can provide a newer answer.
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
            authority_provenance_lines: Vec::new(),
            adaptive_provenance_lines: Vec::new(),
            semantic_provenance_lines: Vec::new(),
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
        if let Some(stage_record) = self.unresolved_planning_governance_record(session) {
            return Err(SessionRuntimeError::PlanningGovernanceUnresolved {
                stage_key: stage_record.stage_key.clone(),
                state: stage_record.lifecycle_state,
                reason: stage_record.blocked_reason.clone().or_else(|| {
                    session.governance_lifecycle.as_ref().and_then(|l| l.terminal_reason.clone())
                }),
            });
        }

        if goal_plan.requires_confirmation() {
            goal_plan
                .confirm()
                .map_err(|error| SessionRuntimeError::GoalPlan(error.to_string()))?;
            session.goal_plan = Some(goal_plan.clone());
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
                    native_context: TaskContext::new(
                        session.session_id.clone(),
                        session.workspace_ref.clone(),
                        RunLimits::default(),
                        Map::new(),
                    ),
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
                    native_context: TaskContext::new(
                        session.session_id.clone(),
                        session.workspace_ref.clone(),
                        RunLimits::default(),
                        Map::new(),
                    ),
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
        let (terminal, decisions, mut trace, mut native_task_context) = decision_loop
            .run_with_options_and_context(
                &goal_plan,
                session.active_flow_policy.as_ref(),
                &session.workspace_ref,
                &session.session_id,
                enable_flow_retry_probe,
            )
            .map_err(|error| SessionRuntimeError::DecisionLoop(error.to_string()))?;
        let mut reason = self.native_terminal_reason(&terminal);
        self.backfill_native_execution_state(
            &runtime,
            &mut native_task_context,
            task_status_for_condition(reason.condition),
        );
        if task_status_for_condition(reason.condition) == TaskStatus::Succeeded {
            self.execute_post_implementation_governance(
                session,
                &runtime,
                &mut goal_plan,
                &decisions,
                &mut native_task_context,
                &mut trace,
            )?;
        }
        let native_review = if task_status_for_condition(reason.condition) == TaskStatus::Succeeded
        {
            let native_review = self.execute_native_review_sequence(
                session,
                &runtime,
                &goal_plan,
                &mut native_task_context,
            )?;
            if let Some(review_reason) = native_review.terminal_reason.clone() {
                reason = review_reason;
            }
            if task_status_for_condition(reason.condition) == TaskStatus::Succeeded {
                self.propagate_cluster_delivery_changes(&goal_plan, &runtime)?;
            }
            native_review
        } else {
            NativeReviewExecution::default()
        };
        if !native_review.events.is_empty() {
            self.insert_trace_events_before_terminal(&mut trace, native_review.events);
        }
        if !governance_events.is_empty() {
            self.insert_trace_events_before_terminal(&mut trace, governance_events);
        }
        let projected_task = native_governance_task.map(|task| {
            self.finalize_native_projected_task(
                task,
                task_status_for_condition(reason.condition),
                &reason,
                &native_task_context,
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
                native_context: native_task_context,
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
        if !trace.events.iter().any(|event| event.event_type.is_reasoning_event())
            && let Some(reasoning_profile) = session
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
        {
            record_reasoning_profile_events(
                &mut trace,
                "terminal",
                goal_plan.proposal_revision,
                reasoning_profile,
            );
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
        if let Some(guardian_request) =
            self.native_guardian_request(session, &goal_plan, decisions.as_slice())
        {
            let guardian_outcome =
                execute_guardians_for_phase(&self.workspace_ref, &guardian_request);
            Self::merge_guardian_projection(
                &mut goal_plan.guidance_guardian,
                &guardian_outcome.projection,
            );
            if let Some(event) = trace
                .events
                .iter_mut()
                .rev()
                .find(|event| event.event_type == TraceEventType::TerminalRecorded)
            {
                Self::append_guardian_projection_payload(
                    &mut event.payload,
                    &guardian_outcome.projection,
                );
            }
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
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
        let mut final_context = self.build_native_task_context(
            session,
            input.limits,
            &goal_plan,
            &input.native_context,
        )?;
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
        native_context: &TaskContext,
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
        // Carry the advanced-context retrieval story into task state so later
        // status projections remain stable after execution begins.
        if let Some(advanced_context) = goal_plan
            .context_pack
            .as_ref()
            .and_then(|context_pack| context_pack.advanced_context.as_ref())
        {
            context
                .set_latest_advanced_context(advanced_context)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        if let Some(story) = goal_plan.cluster_delivery_story.as_ref() {
            context
                .set_cluster_delivery_story(story)
                .map_err(|error| SessionRuntimeError::TaskContext(error.to_string()))?;
        }
        self.merge_native_task_context(&mut context, native_context);
        Ok(context)
    }

    fn merge_native_task_context(&self, context: &mut TaskContext, native_context: &TaskContext) {
        context.apply_state_patch(&native_context.state);
        for history_ref in &native_context.history_refs {
            context.push_history_ref(history_ref.clone());
        }
        if let Some(last_result) = native_context.last_result.clone() {
            context.set_last_result(last_result);
        }
    }

    fn backfill_native_execution_state(
        &self,
        runtime: &FixtureRuntime,
        native_context: &mut TaskContext,
        terminal_status: TaskStatus,
    ) {
        if !native_context.state.contains_key(LATEST_CHANGED_FILES_KEY) {
            let changed_files = runtime
                .profile
                .attempts
                .iter()
                .flat_map(|attempt| attempt.changes.iter().map(|change| change.path.clone()))
                .collect::<Vec<_>>();
            if !changed_files.is_empty() {
                native_context
                    .state
                    .insert(LATEST_CHANGED_FILES_KEY.to_string(), json!(changed_files));
            }
        }

        native_context.state.insert(
            LATEST_VALIDATION_STATUS_KEY.to_string(),
            json!(if terminal_status == TaskStatus::Succeeded {
                VALIDATION_STATUS_PASSED
            } else {
                VALIDATION_STATUS_FAILED
            }),
        );
    }

    fn insert_trace_events_before_terminal(
        &self,
        trace: &mut ExecutionTrace,
        events: Vec<TraceEvent>,
    ) {
        let insert_at = trace
            .events
            .iter()
            .rposition(|event| event.event_type == TraceEventType::TerminalRecorded)
            .unwrap_or(trace.events.len());
        trace.events.splice(insert_at..insert_at, events);
    }

    /// Invokes Canon governance with execution-time modes after the decision
    /// loop produces implementation artifacts. Only activates when the session
    /// has an active governance lifecycle backed by the Canon runtime and the
    /// Canon CLI command is available.
    fn execute_post_implementation_governance(
        &self,
        session: &mut ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &mut GoalPlan,
        decisions: &[Decision],
        native_context: &mut TaskContext,
        trace: &mut ExecutionTrace,
    ) -> Result<(), SessionRuntimeError> {
        let Some(lifecycle) = session.governance_lifecycle.as_ref() else {
            return Ok(());
        };
        if lifecycle.governance_runtime != GovernanceRuntimeKind::Canon {
            return Ok(());
        }
        let Some(governance) = runtime.profile.governance.as_ref() else {
            return Ok(());
        };
        let Some(canon) = governance.canon.as_ref() else {
            return Ok(());
        };
        if !runtime_command_available(&canon.command) {
            return Ok(());
        }
        if canon_workspace_scope_mismatch_reason(&self.workspace_ref).is_some() {
            return Ok(());
        }

        let goal = goal_plan.goal_text.clone();
        let execution_modes: &[CanonMode] = &[CanonMode::Implementation, CanonMode::Verification];

        for &mode in execution_modes {
            let Some(stage_key) = execution_stage_key_for_mode(mode) else {
                continue;
            };
            let stage_brief_ref = self.materialize_execution_stage_brief(
                mode,
                decisions,
                goal_plan,
                native_context,
                &runtime.profile.read_targets,
            )?;
            let governance_attempt_id = Uuid::new_v4().to_string();
            let previous_attempt_id = session.governance_lifecycle.as_ref().and_then(|lifecycle| {
                lifecycle
                    .stage_records
                    .iter()
                    .rev()
                    .find(|record| record.stage_key == stage_key)
                    .map(|record| record.governance_attempt_id.clone())
            });
            let input_documents = planning_governance_input_documents(
                session.authored_brief.as_ref(),
                &stage_brief_ref,
                goal_plan.compacted_canon_memory.as_ref(),
            );
            let read_targets =
                execution_governance_read_targets(native_context, &runtime.profile.read_targets);

            let request = GovernanceRuntimeRequest {
                request_kind: GovernanceRequestKind::Start,
                governance_attempt_id: governance_attempt_id.clone(),
                stage_key: stage_key.to_string(),
                goal: goal.clone(),
                workspace_ref: self.workspace_ref.to_string_lossy().to_string(),
                autopilot: true,
                mode: Some(mode),
                system_context: canon.default_system_context,
                risk: canon.default_risk.clone().map(|risk| {
                    CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
                }),
                zone: canon.default_zone.clone().map(|zone| {
                    CanonAuthorityZone::canonicalize_label(&zone)
                        .map(str::to_string)
                        .unwrap_or(zone)
                }),
                owner: canon.default_owner.clone().map(|owner| {
                    CanonIntendedPersona::canonicalize_label(&owner)
                        .map(str::to_string)
                        .unwrap_or(owner)
                }),
                run_ref: None,
                packet_ref: None,
                bounded_context: crate::adapters::governance_runtime::GovernanceBoundedContext {
                    read_targets: read_targets.clone(),
                    stage_brief_ref: Some(stage_brief_ref.clone()),
                    reused_packets: Vec::new(),
                },
                input_documents,
            };

            trace.record_event(
                TraceEventType::GovernanceStarted,
                None,
                goal_plan.proposal_revision,
                json!({
                    "stage_key": stage_key,
                    "runtime": GovernanceRuntimeKind::Canon,
                    "canon_mode": mode,
                    "phase": "post-implementation",
                    "stage_brief_ref": stage_brief_ref,
                    "read_targets": read_targets,
                }),
            );

            let response = match CanonCliRuntime::new(canon.command.clone())
                .with_working_directory(&self.workspace_ref)
                .execute(&request)
            {
                Ok(response) => response,
                Err(error) => {
                    trace.record_event(
                        TraceEventType::GovernanceCompleted,
                        None,
                        goal_plan.proposal_revision,
                        json!({
                            "stage_key": stage_key,
                            "canon_mode": mode,
                            "status": "error",
                            "message": error.to_string(),
                        }),
                    );
                    break;
                }
            };

            let blocked_reason = matches!(
                response.status,
                GovernanceLifecycleState::AwaitingApproval
                    | GovernanceLifecycleState::Blocked
                    | GovernanceLifecycleState::Failed
            )
            .then(|| response.message.clone());

            let record = GovernedStageRecord {
                stage_key: stage_key.to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: response.status,
                required: false,
                autopilot_enabled: true,
                approval_state: response.approval_state,
                canon_run_ref: response.run_ref.clone(),
                governance_attempt_id,
                previous_governance_attempt_id: previous_attempt_id,
                packet_ref: response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                decision_ref: None,
                stage_council: None,
                blocked_reason: blocked_reason.clone(),
            };

            let compacted_canon_memory = compacted_canon_memory_from_response(
                stage_key,
                GovernanceRuntimeKind::Canon,
                &response,
            );
            if let Some(memory) = compacted_canon_memory.clone() {
                goal_plan.compacted_canon_memory = Some(memory);
            }
            let projection = governance_projection_snapshot(
                native_context,
                stage_key,
                response.packet.as_ref(),
                response.approval_state,
            )
            .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
            let patch = governance_state_patch(
                &record,
                response.packet.as_ref(),
                None,
                None,
                compacted_canon_memory.as_ref(),
                &projection,
            )
            .map_err(|error| SessionRuntimeError::GovernancePatch(error.to_string()))?;
            native_context.apply_state_patch(&patch);

            trace.record_event(
                TraceEventType::GovernanceCompleted,
                None,
                goal_plan.proposal_revision,
                json!({
                    "stage_key": stage_key,
                    "canon_mode": mode,
                    "packet_ref": response.packet.as_ref().map(|packet| packet.packet_ref.clone()),
                    "packet_readiness": response.packet.as_ref().map(|packet| packet.readiness),
                    "document_refs": response.packet.as_ref().map(|packet| packet.document_refs.clone()).unwrap_or_default(),
                    "headline": response.packet.as_ref().map(|packet| packet.headline.clone()).unwrap_or_else(|| response.message.clone()),
                    "status": response.status,
                    "approval_state": response.approval_state,
                    "run_ref": response.run_ref,
                    "latest_governance_runtime_state": projection.runtime_state,
                    "latest_governance_rollout_profile": projection.rollout_profile,
                    "latest_governance_reason": projection.reason,
                    "latest_governance_contract_lines": projection.contract_lines,
                    "latest_governance_approval_provenance": projection.approval_provenance,
                }),
            );

            self.upsert_execution_stage_record(session, record);

            if response.status == GovernanceLifecycleState::GovernedReady
                && response.packet.is_some()
            {
                let doc_ref = governed_document_ref_from_response(stage_key, mode, &response);
                append_governed_document_to_lifecycle(session, doc_ref);
            }

            if response.status != GovernanceLifecycleState::GovernedReady {
                break;
            }
        }

        Ok(())
    }

    fn upsert_execution_stage_record(
        &self,
        session: &mut ActiveSessionRecord,
        record: GovernedStageRecord,
    ) {
        let Some(lifecycle) = session.governance_lifecycle.as_mut() else {
            return;
        };

        if let Some(existing_index) = lifecycle
            .stage_records
            .iter()
            .position(|existing| existing.stage_key == record.stage_key)
        {
            lifecycle.stage_records[existing_index] = record;
        } else {
            lifecycle.stage_records.push(record);
        }
    }

    fn materialize_execution_stage_brief(
        &self,
        mode: CanonMode,
        decisions: &[Decision],
        goal_plan: &GoalPlan,
        native_context: &TaskContext,
        fallback_targets: &[String],
    ) -> Result<String, SessionRuntimeError> {
        let stage_brief_ref = format!(
            "{}/{}/{}",
            EXECUTION_GOVERNANCE_ROOT,
            mode.as_str(),
            EXECUTION_STAGE_BRIEF_FILE_NAME
        );
        let stage_brief_path = self.workspace_ref.join(&stage_brief_ref);
        let Some(parent) = stage_brief_path.parent() else {
            return Err(SessionRuntimeError::ExecutionInvariant(format!(
                "execution stage brief path has no parent for mode {}",
                mode.as_str()
            )));
        };
        fs::create_dir_all(parent).map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to create execution stage brief directory for {}: {error}",
                mode.as_str()
            ))
        })?;
        fs::write(
            &stage_brief_path,
            render_execution_stage_brief(
                mode,
                goal_plan,
                decisions,
                native_context,
                fallback_targets,
            ),
        )
        .map_err(|error| {
            SessionRuntimeError::GoalPlan(format!(
                "failed to write execution stage brief for {}: {error}",
                mode.as_str()
            ))
        })?;
        Ok(stage_brief_ref)
    }

    fn execute_native_review_sequence(
        &self,
        session: &ActiveSessionRecord,
        runtime: &FixtureRuntime,
        goal_plan: &GoalPlan,
        native_context: &mut TaskContext,
    ) -> Result<NativeReviewExecution, SessionRuntimeError> {
        let Some(review) = runtime.profile.review.as_ref() else {
            return Ok(NativeReviewExecution::default());
        };
        let Some(trigger) = Self::native_review_trigger(review) else {
            return Ok(NativeReviewExecution::default());
        };

        native_context.state.insert(NEXT_REVIEW_TRIGGER_KEY.to_string(), json!(trigger));
        let attempt_id = native_context
            .state
            .get(LATEST_ATTEMPT_ID_KEY)
            .and_then(Value::as_str)
            .unwrap_or(goal_plan.plan_id.as_str())
            .to_string();
        let mut review_trace = ExecutionTrace::new(
            goal_plan.plan_id.clone(),
            session.session_id.clone(),
            goal_plan.goal_text.clone(),
        );

        for reviewer in &review.reviewers {
            let mut step = Step::agent(
                format!("{NATIVE_REVIEW_STEP_PREFIX}-{}", reviewer.reviewer_id),
                NATIVE_REVIEWER_AGENT_NAME,
                json!({
                    "phase": NATIVE_REVIEW_PHASE,
                    "attempt_id": attempt_id.clone(),
                    "reviewer_id": reviewer.reviewer_id.clone(),
                    "adjudication": false,
                    "default_review_trigger": trigger,
                }),
            )
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
            let result = self.execute_native_follow_up_step(
                runtime,
                native_context,
                &mut review_trace,
                &mut step,
                goal_plan.proposal_revision,
            )?;
            if result.status == ExecutionStatus::Failed {
                return Ok(NativeReviewExecution {
                    events: review_trace.events,
                    terminal_reason: Self::native_review_terminal_reason(
                        native_context,
                        result.error.as_ref().map(|error| error.message.as_str()),
                    ),
                });
            }
        }

        let mut vote_step = Step::tool(
            NATIVE_REVIEW_VOTE_STEP_ID,
            NATIVE_REVIEW_VOTER_TOOL_NAME,
            json!({
                "phase": NATIVE_REVIEW_VOTE_PHASE,
                "attempt_id": attempt_id.clone(),
            }),
        )
        .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let vote_result = self.execute_native_follow_up_step(
            runtime,
            native_context,
            &mut review_trace,
            &mut vote_step,
            goal_plan.proposal_revision,
        )?;
        if vote_result.status == ExecutionStatus::Failed {
            return Ok(NativeReviewExecution {
                events: review_trace.events,
                terminal_reason: Self::native_review_terminal_reason(
                    native_context,
                    vote_result.error.as_ref().map(|error| error.message.as_str()),
                ),
            });
        }

        if review.adjudication.enabled {
            let adjudicator_id = review.adjudication.reviewer_id.as_ref().ok_or_else(|| {
                SessionRuntimeError::ExecutionInvariant(
                    "native review adjudication is enabled without an adjudicator".to_string(),
                )
            })?;
            let mut step = Step::agent(
                format!("{NATIVE_REVIEW_STEP_PREFIX}-adjudicate"),
                NATIVE_REVIEWER_AGENT_NAME,
                json!({
                    "phase": NATIVE_REVIEW_PHASE,
                    "attempt_id": attempt_id.clone(),
                    "reviewer_id": adjudicator_id,
                    "adjudication": true,
                    "default_review_trigger": trigger,
                }),
            )
            .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
            let result = self.execute_native_follow_up_step(
                runtime,
                native_context,
                &mut review_trace,
                &mut step,
                goal_plan.proposal_revision,
            )?;
            if result.status == ExecutionStatus::Failed {
                return Ok(NativeReviewExecution {
                    events: review_trace.events,
                    terminal_reason: Self::native_review_terminal_reason(
                        native_context,
                        result.error.as_ref().map(|error| error.message.as_str()),
                    ),
                });
            }
        }

        let mut finalize_step = Step::tool(
            NATIVE_REVIEW_FINALIZE_STEP_ID,
            NATIVE_REVIEW_FINALIZER_TOOL_NAME,
            json!({
                "phase": NATIVE_REVIEW_FINALIZE_PHASE,
                "attempt_id": attempt_id,
            }),
        )
        .map_err(|error| SessionRuntimeError::ExecutionInvariant(error.to_string()))?;
        let finalize_result = self.execute_native_follow_up_step(
            runtime,
            native_context,
            &mut review_trace,
            &mut finalize_step,
            goal_plan.proposal_revision,
        )?;

        Ok(NativeReviewExecution {
            events: review_trace.events,
            terminal_reason: Self::native_review_terminal_reason(
                native_context,
                finalize_result.error.as_ref().map(|error| error.message.as_str()),
            ),
        })
    }

    fn execute_native_follow_up_step(
        &self,
        runtime: &FixtureRuntime,
        native_context: &mut TaskContext,
        trace: &mut ExecutionTrace,
        step: &mut Step,
        plan_revision: usize,
    ) -> Result<StepExecutionResult, SessionRuntimeError> {
        step.mark_running();
        let started_at = current_timestamp_millis();
        let mut attempt = StepAttempt::new(step.id.clone(), step.input.clone(), started_at);
        trace.record_event(
            TraceEventType::StepStarted,
            Some(step.id.clone()),
            plan_revision,
            json!({
                "attempt_number": step.attempt_count,
                "input": step.input.clone(),
                "step_kind": step.kind,
            }),
        );
        record_review_step_started(
            trace,
            &step.id,
            &step.input,
            &native_context.state,
            plan_revision,
        );

        let result = self.normalize_result(self.execute_step(runtime, step, native_context), step);
        attempt.complete(&result, current_timestamp_millis());
        native_context.push_history_ref(attempt.attempt_id.clone());

        match result.status {
            ExecutionStatus::Succeeded => {
                let output = result.output.clone().ok_or_else(|| {
                    SessionRuntimeError::ExecutionInvariant(format!(
                        "native review step {} reported success without output",
                        step.id
                    ))
                })?;
                step.mark_succeeded(output.clone());
                native_context.apply_success_output(&step.id, &output, result.state_patch.as_ref());
                native_context.set_last_result(StepResultSummary::from_step(step));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step.id.clone()),
                    plan_revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "succeeded",
                        "output": output,
                        "evidence": result.evidence,
                    }),
                );
            }
            ExecutionStatus::Failed => {
                let error = result.error.clone().ok_or_else(|| {
                    SessionRuntimeError::ExecutionInvariant(format!(
                        "native review step {} reported failure without error",
                        step.id
                    ))
                })?;
                step.mark_failed(error.clone(), result.recoverability);
                native_context.apply_failure_error(&step.id, &error);
                if let Some(state_patch) = result.state_patch.as_ref() {
                    native_context.apply_state_patch(state_patch);
                }
                native_context.set_last_result(StepResultSummary::from_step(step));
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step.id.clone()),
                    plan_revision,
                    json!({
                        "attempt_id": attempt.attempt_id,
                        "status": "failed",
                        "error": error,
                        "recoverability": result.recoverability,
                        "evidence": result.evidence,
                    }),
                );
            }
        }

        record_review_step_completed(
            trace,
            &step.id,
            &step.input,
            &result,
            &native_context.state,
            plan_revision,
        );

        Ok(result)
    }

    fn native_review_trigger(review: &ReviewProfile) -> Option<ReviewTrigger> {
        review
            .triggers
            .iter()
            .copied()
            .find(|trigger| !matches!(trigger, ReviewTrigger::ValidationFailed))
            .or_else(|| review.triggers.first().copied())
    }

    fn native_review_terminal_reason(
        native_context: &TaskContext,
        failure_message: Option<&str>,
    ) -> Option<TerminalReason> {
        let outcome = native_context
            .state
            .get(LATEST_REVIEW_OUTCOME_KEY)
            .cloned()
            .and_then(|value| serde_json::from_value::<ReviewOutcome>(value).ok());
        let mut details = Map::new();
        for key in [
            "latest_review_trigger",
            "latest_review_findings",
            "latest_review_participants",
            "latest_review_vote_resolution",
            "latest_review_vote",
        ] {
            if let Some(value) = native_context.state.get(key).cloned() {
                details.insert(key.to_string(), value);
            }
        }
        let details = (!details.is_empty()).then_some(Value::Object(details));

        match outcome {
            Some(ReviewOutcome::Accepted) => None,
            Some(ReviewOutcome::Rejected) => Some(build_terminal_reason(
                TerminalCondition::TaskNotCredible,
                failure_message.unwrap_or("native review rejected the delivery result"),
                details,
            )),
            Some(ReviewOutcome::Escalated) => Some(build_terminal_reason(
                TerminalCondition::NoCredibleNextStep,
                failure_message.unwrap_or("native review escalated and requires follow-up"),
                details,
            )),
            Some(ReviewOutcome::Failed) | None => Some(build_terminal_reason(
                TerminalCondition::UnrecoverableError,
                failure_message.unwrap_or("native review failed"),
                details,
            )),
        }
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
        let mut task = if let Some(active_task) = session
            .active_task
            .as_ref()
            .filter(|task| task.goal == goal && !task.status.is_terminal())
        {
            active_task.clone()
        } else {
            let plan = build_fixture_plan_for_goal(&self.workspace_ref, Some(active_flow), &goal)
                .map_err(SessionRuntimeError::FixtureRuntime)?;
            Task::new(Uuid::new_v4().to_string(), &request, plan)
                .map_err(SessionRuntimeError::TaskRequest)?
        };
        let mut governance_trace = self.build_goal_plan_trace(&session.session_id, goal_plan);
        let mut saw_governance = false;
        let start_step_index = task.plan.current_step_index;

        for step_index in start_step_index..task.plan.steps.len() {
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
        terminal_status: TaskStatus,
        terminal_reason: &TerminalReason,
        native_context: &TaskContext,
    ) -> Task {
        task.context.apply_state_patch(&native_context.state);
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
        if let Some(reasoning_profile) = session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
            .filter(|record| {
                record.stage_key == latest_governance.stage_key
                    && record.status.halts_outer_workflow()
            })
        {
            let trace_location = session
                .latest_trace_ref
                .clone()
                .ok_or(SessionRuntimeError::MissingTraceReference)?;

            return Ok(TaskRunResponse {
                task_id: task.id.clone(),
                terminal_status: TaskStatus::Running,
                terminal_reason: build_terminal_reason(
                    TerminalCondition::TaskNotCredible,
                    reasoning_profile_block_message(reasoning_profile),
                    Some(json!({
                        "stage_key": reasoning_profile.stage_key,
                        "profile_id": reasoning_profile.profile_id,
                        "status": reasoning_profile.status,
                    })),
                ),
                final_context: task.context.clone(),
                plan_revision: task.plan.revision,
                trace_location,
            });
        }
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
        let trace_location = self.persist_trace(&session.session_id, trace)?;
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
                let guardian_phase = Self::guardian_phase_for_step(session, step_index);
                let guardian_request = self.guardian_request_for_step(
                    session,
                    task,
                    &step_snapshot,
                    guardian_phase,
                    &result,
                );
                let guardian_outcome =
                    execute_guardians_for_phase(&self.workspace_ref, &guardian_request);
                if let Some(goal_plan) = session.goal_plan.as_mut() {
                    Self::merge_guardian_projection(
                        &mut goal_plan.guidance_guardian,
                        &guardian_outcome.projection,
                    );
                }
                let mut step_payload = json!({
                    "attempt_id": attempt.attempt_id,
                    "status": "succeeded",
                    "output": output,
                    "evidence": result.evidence,
                });
                Self::append_guardian_projection_payload(
                    &mut step_payload,
                    &guardian_outcome.projection,
                );
                trace.record_event(
                    TraceEventType::StepCompleted,
                    Some(step_snapshot.id.clone()),
                    task.plan.revision,
                    step_payload,
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
                let trace_location = self.persist_trace(&session.session_id, trace)?;
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
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
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
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
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
                        let trace_location = self.persist_trace(&session.session_id, trace)?;
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

    // Builds the guardian request from a fixture-style step result, preferring
    // normalized changed-file state and then backfilling explicit evidence refs.
    fn guardian_request_for_step(
        &self,
        session: &ActiveSessionRecord,
        task: &Task,
        step: &Step,
        phase: CapabilityPhase,
        result: &StepExecutionResult,
    ) -> GuardianExecutionRequest {
        let goal_text = session.goal.clone().unwrap_or_else(|| task.goal.clone());
        let target_ref = step
            .target_name
            .clone()
            .or_else(|| {
                session
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.tasks.get(task.plan.current_step_index))
                    .map(|planned| planned.target.clone())
            })
            .unwrap_or_else(|| "workspace".to_string());
        let changed_files = Self::changed_files_for_guardian(task, result, step, &target_ref);
        let mut evidence_refs = changed_files.clone();
        if let Some(target_name) = step.target_name.as_ref()
            && !evidence_refs.iter().any(|reference| reference == target_name)
        {
            evidence_refs.push(target_name.clone());
        }
        if let Some(evidence) = result.evidence.as_ref() {
            evidence_refs.push(evidence.to_string());
        }

        GuardianExecutionRequest {
            goal_text,
            target_ref,
            phase,
            evidence_refs,
            changed_files,
            workspace_signals: collect_workspace_signals(&self.workspace_ref),
        }
    }

    // Reconstructs the same guardian request shape for native runs, where the
    // authoritative evidence lives in persisted decisions instead of step payloads.
    fn native_guardian_request(
        &self,
        session: &ActiveSessionRecord,
        goal_plan: &GoalPlan,
        decisions: &[Decision],
    ) -> Option<GuardianExecutionRequest> {
        // Native runs do not emit fixture step payloads, so reuse the guardian
        // executor by deriving the same request shape from persisted decisions.
        let phase = Self::guardian_phase_for_decisions(decisions)?;
        let mut changed_files = decisions
            .iter()
            .filter(|decision| {
                matches!(
                    decision.decision_type,
                    DecisionType::Code | DecisionType::Fix | DecisionType::Test
                )
            })
            .map(|decision| decision.target.trim().to_string())
            .filter(|target| !target.is_empty())
            .collect::<Vec<_>>();
        if changed_files.is_empty() {
            changed_files = goal_plan
                .tasks
                .iter()
                .map(|task| task.target.trim().to_string())
                .filter(|target| !target.is_empty())
                .collect();
        }
        if changed_files.is_empty() {
            return None;
        }
        let mut unique_files = BTreeSet::new();
        changed_files.retain(|target| unique_files.insert(target.clone()));

        let target_ref = changed_files.first().cloned()?;
        let mut seen_refs = BTreeSet::new();
        let mut evidence_refs = Vec::new();
        for changed_file in &changed_files {
            if seen_refs.insert(changed_file.clone()) {
                evidence_refs.push(changed_file.clone());
            }
        }
        for decision in decisions {
            if seen_refs.insert(decision.target.clone()) {
                evidence_refs.push(decision.target.clone());
            }
            for evidence in &decision.evidence_inputs {
                if seen_refs.insert(evidence.reference.clone()) {
                    evidence_refs.push(evidence.reference.clone());
                }
            }
            if let Some(tool_result) = decision.tool_result.as_ref()
                && seen_refs.insert(tool_result.invocation.clone())
            {
                evidence_refs.push(tool_result.invocation.clone());
            }
        }

        Some(GuardianExecutionRequest {
            goal_text: session.goal.clone().unwrap_or_else(|| goal_plan.goal_text.clone()),
            target_ref,
            phase,
            evidence_refs,
            changed_files,
            workspace_signals: collect_workspace_signals(&self.workspace_ref),
        })
    }

    // Maps the planned step hint to the lifecycle phase used to resolve and run
    // guidance or guardians after that step finishes.
    fn guardian_phase_for_step(
        session: &ActiveSessionRecord,
        step_index: usize,
    ) -> CapabilityPhase {
        match session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.tasks.get(step_index))
            .and_then(|planned| planned.decision_type_hint)
        {
            Some(crate::domain::decision::DecisionType::Analyze) => CapabilityPhase::Planning,
            Some(crate::domain::decision::DecisionType::Code)
            | Some(crate::domain::decision::DecisionType::Fix) => CapabilityPhase::Implementation,
            Some(crate::domain::decision::DecisionType::Test) => CapabilityPhase::Verification,
            Some(crate::domain::decision::DecisionType::Replan) => CapabilityPhase::Review,
            None => CapabilityPhase::Implementation,
        }
    }

    // Native flows do not have an explicit step cursor, so infer the guardian
    // phase from the latest persisted decision that materially changed the run.
    fn guardian_phase_for_decisions(decisions: &[Decision]) -> Option<CapabilityPhase> {
        decisions
            .iter()
            .rev()
            .map(|decision| match decision.decision_type {
                DecisionType::Analyze => Some(CapabilityPhase::Planning),
                DecisionType::Code | DecisionType::Fix => Some(CapabilityPhase::Implementation),
                DecisionType::Test => Some(CapabilityPhase::Verification),
                DecisionType::Replan => Some(CapabilityPhase::Review),
            })
            .next()
            .flatten()
    }

    fn changed_files_for_guardian(
        task: &Task,
        result: &StepExecutionResult,
        step: &Step,
        fallback_target: &str,
    ) -> Vec<String> {
        // Successful bounded work normalizes changed files under
        // latest_changed_files; fall back only when that normalized view is absent.
        for state_key in ["latest_changed_files", "changed_files"] {
            if let Some(changed_files) = task.context.state.get(state_key).and_then(Value::as_array)
            {
                let files = changed_files
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();
                if !files.is_empty() {
                    return files;
                }
            }
        }

        if let Some(changed_files) = result
            .evidence
            .as_ref()
            .and_then(|value| value.get("changed_files"))
            .and_then(Value::as_array)
        {
            let files = changed_files
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            if !files.is_empty() {
                return files;
            }
        }

        step.target_name
            .clone()
            .map(|target| vec![target])
            .unwrap_or_else(|| vec![fallback_target.to_string()])
    }

    // Merge execution output into the flattened read-side projection while
    // keeping planning-time guidance selection stable across later phases.
    fn merge_guardian_projection(
        projection: &mut GuidanceGuardianProjection,
        update: &GuidanceGuardianProjection,
    ) {
        // Planning guidance stays stable once selected, while execution-phase
        // guardian output should reflect the latest authoritative verification pass.
        if projection.capability_resolution_summary.is_none() {
            projection.capability_resolution_summary = update.capability_resolution_summary.clone();
        }
        if projection.loaded_guidance_sources.is_empty() {
            projection.loaded_guidance_sources = update.loaded_guidance_sources.clone();
        }
        if projection.skipped_guidance_sources.is_empty() {
            projection.skipped_guidance_sources = update.skipped_guidance_sources.clone();
        }
        projection.loaded_guardian_sources = update.loaded_guardian_sources.clone();
        projection.skipped_guardian_sources = update.skipped_guardian_sources.clone();
        projection.guardian_timeline = update.guardian_timeline.clone();
        projection.guardian_findings_summary = update.guardian_findings_summary.clone();
        projection.guardian_findings = update.guardian_findings.clone();
        projection.guardian_degradations = update.guardian_degradations.clone();
        projection.guardian_blocking_outcome = update.guardian_blocking_outcome.clone();
    }

    // Mirror the flattened projection into trace payloads so `inspect` can
    // hydrate the same operator story without recomputing runtime resolution.
    fn append_guardian_projection_payload(
        payload: &mut Value,
        projection: &GuidanceGuardianProjection,
    ) {
        let Some(object) = payload.as_object_mut() else {
            return;
        };
        if let Some(summary) = projection.capability_resolution_summary.as_ref() {
            object.insert(
                "capability_resolution_summary".to_string(),
                Value::String(summary.clone()),
            );
        }
        if !projection.loaded_guidance_sources.is_empty() {
            object.insert(
                "loaded_guidance_sources".to_string(),
                serde_json::to_value(&projection.loaded_guidance_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.skipped_guidance_sources.is_empty() {
            object.insert(
                "skipped_guidance_sources".to_string(),
                serde_json::to_value(&projection.skipped_guidance_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.loaded_guardian_sources.is_empty() {
            object.insert(
                "loaded_guardian_sources".to_string(),
                serde_json::to_value(&projection.loaded_guardian_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.skipped_guardian_sources.is_empty() {
            object.insert(
                "skipped_guardian_sources".to_string(),
                serde_json::to_value(&projection.skipped_guardian_sources).unwrap_or(Value::Null),
            );
        }
        if !projection.guardian_timeline.is_empty() {
            object.insert(
                "guardian_timeline".to_string(),
                serde_json::to_value(&projection.guardian_timeline).unwrap_or(Value::Null),
            );
        }
        if let Some(summary) = projection.guardian_findings_summary.as_ref() {
            object.insert("guardian_findings_summary".to_string(), Value::String(summary.clone()));
        }
        if !projection.guardian_findings.is_empty() {
            object.insert(
                "guardian_findings".to_string(),
                serde_json::to_value(&projection.guardian_findings).unwrap_or(Value::Null),
            );
        }
        if !projection.guardian_degradations.is_empty() {
            object.insert(
                "guardian_degradations".to_string(),
                serde_json::to_value(&projection.guardian_degradations).unwrap_or(Value::Null),
            );
        }
        if let Some(outcome) = projection.guardian_blocking_outcome.as_ref() {
            object.insert("guardian_blocking_outcome".to_string(), Value::String(outcome.clone()));
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

    // Loads the current trace when present; otherwise creates a new trace and
    // records the initial task and flow-selection events.
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
        let trace_location = self.persist_trace(&session.session_id, &mut trace)?;
        session.latest_trace_ref = Some(trace_location);

        Ok(trace)
    }

    fn unresolved_planning_governance_record<'a>(
        &self,
        session: &'a ActiveSessionRecord,
    ) -> Option<&'a GovernedStageRecord> {
        session.governance_lifecycle.as_ref().and_then(|lifecycle| {
            lifecycle.stage_records.iter().rev().find(|record| {
                planning_canon_mode_for_stage_key(&record.stage_key).is_some()
                    && matches!(
                        record.lifecycle_state,
                        GovernanceLifecycleState::AwaitingApproval
                            | GovernanceLifecycleState::Blocked
                            | GovernanceLifecycleState::Failed
                    )
            })
        })
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

    // Applies terminal state to task, trace, and session in one place so the
    // persisted snapshot stays aligned across all operator surfaces.
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
        if !trace.events.iter().any(|event| event.event_type.is_reasoning_event())
            && let Some(reasoning_profile) = session
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
        {
            let step_id =
                task.plan.current_step().map(|step| step.id.as_str()).unwrap_or("terminal");
            record_reasoning_profile_events(trace, step_id, task.plan.revision, reasoning_profile);
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
        let trace_location = self.persist_trace(&session.session_id, trace)?;

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

    // Persist twice so the stored trace payload also contains its own final
    // trace location for downstream inspect and status rendering.
    fn persist_trace(
        &self,
        session_id: &str,
        trace: &mut ExecutionTrace,
    ) -> Result<String, SessionRuntimeError> {
        let trace_store = FileTraceStore::for_session(&self.workspace_ref, session_id);
        let path = trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        let trace_location = path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        trace_store.persist(trace).map_err(SessionRuntimeError::TraceStore)?;
        self.project_trace_events_to_session_audit(session_id, &trace_location, trace)?;
        Ok(trace_location)
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
    native_context: TaskContext,
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

fn canon_workspace_scope_mismatch_reason(workspace: &Path) -> Option<String> {
    let workspace = workspace.canonicalize().unwrap_or_else(|_| workspace.to_path_buf());
    let git_root = nearest_git_root(&workspace)?;
    if git_root == workspace {
        return None;
    }

    Some(format!(
        "planning governance requires a Canon workspace root, but Canon would target git root {} instead of workspace {}; use the repository root as the Boundline workspace or initialize a dedicated nested repository first",
        git_root.display(),
        workspace.display()
    ))
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn current_user_name() -> Option<String> {
    env::var("USER")
        .ok()
        .or_else(|| env::var("USERNAME").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn git_config_value(workspace: &Path, key: &str) -> Option<String> {
    let output =
        Command::new("git").current_dir(workspace).args(["config", "--get", key]).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn session_audit_outcome_for_status(status: SessionStatus) -> SessionAuditOutcome {
    match status {
        SessionStatus::Initialized => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "session initialized")
        }
        SessionStatus::GoalCaptured => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Recorded,
            "goal captured for active session",
        ),
        SessionStatus::Planned => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "session planned")
        }
        SessionStatus::Blocked => {
            let mut outcome =
                SessionAuditOutcome::new(SessionAuditOutcomeStatus::Blocked, "session blocked");
            outcome.blocking = true;
            outcome
        }
        SessionStatus::Running => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "session running")
        }
        SessionStatus::Succeeded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "session succeeded")
        }
        SessionStatus::Failed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session failed")
        }
        SessionStatus::Exhausted => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Failed,
            "session exhausted its execution budget",
        ),
        SessionStatus::Aborted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session aborted")
        }
        SessionStatus::Invalid => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, "session invalid")
        }
    }
}

fn trace_event_audit_algorithm(event_type: TraceEventType) -> SessionAuditAlgorithm {
    match event_type {
        TraceEventType::GoalPlanCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Plan,
            "goal_planner",
            "build_goal_plan_with_sources",
        ),
        TraceEventType::FlowInferred => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Plan, "session_runtime", "plan_goal_plan")
        }
        TraceEventType::ProjectScalePathProposed
        | TraceEventType::ProjectScaleStageTransitioned => SessionAuditAlgorithm::new(
            SessionAuditPhase::Goal,
            "workflow",
            "propose_project_scale_path",
        ),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "decision_loop",
            "run_with_options_and_context",
        ),
        TraceEventType::ReviewVoteResolved | TraceEventType::VotingDecisionRecorded => {
            SessionAuditAlgorithm::new(
                SessionAuditPhase::Review,
                "review_vote",
                "VoteRuleDefinition::resolve",
            )
        }
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewerStarted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Review,
            "review_trace",
            "record_review_step_completed",
        ),
        TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ReasoningProfileBlocked
        | TraceEventType::ReasoningProfileInterrupted
        | TraceEventType::ReasoningProfileEscalated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Reasoning,
            "reasoning_profile",
            "record_reasoning_profile_events",
        ),
        TraceEventType::GovernanceDecisionRecorded => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "build_autopilot_decision",
        ),
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => SessionAuditAlgorithm::new(
            SessionAuditPhase::Governance,
            "governance",
            "execute_governance_for_step",
        ),
        TraceEventType::RetryScheduled
        | TraceEventType::StageRetryScheduled
        | TraceEventType::Replanned
        | TraceEventType::StageReplanned
        | TraceEventType::StageFailed => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Recovery, "recovery", "decide_recovery")
        }
        TraceEventType::CheckpointCreated => SessionAuditAlgorithm::new(
            SessionAuditPhase::Run,
            "checkpoint",
            "prepare_checkpoint_for_mutation",
        ),
        TraceEventType::TerminalRecorded => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "finalize_task")
        }
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::StageTransitioned
        | TraceEventType::StepStarted
        | TraceEventType::StepCompleted => {
            SessionAuditAlgorithm::new(SessionAuditPhase::Run, "session_runtime", "advance_task")
        }
    }
}

fn trace_event_audit_outcome(event: &TraceEvent) -> SessionAuditOutcome {
    match event.event_type {
        TraceEventType::TaskStarted
        | TraceEventType::FlowSelected
        | TraceEventType::GoalPlanCreated
        | TraceEventType::FlowInferred
        | TraceEventType::ProjectScalePathProposed
        | TraceEventType::StageTransitioned
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewerStarted
        | TraceEventType::StepStarted
        | TraceEventType::DecisionCreated
        | TraceEventType::ReasoningProfileActivated
        | TraceEventType::ReasoningParticipantStarted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Started, "activity started")
        }
        TraceEventType::DecisionDispatched
        | TraceEventType::CheckpointCreated
        | TraceEventType::VotingDecisionRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Recorded, "activity recorded")
        }
        TraceEventType::DecisionVerified | TraceEventType::GovernanceCompleted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Succeeded, "activity succeeded")
        }
        TraceEventType::StepCompleted
        | TraceEventType::ReviewerCompleted
        | TraceEventType::ReviewVoteResolved
        | TraceEventType::ReviewAdjudicated
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::DecisionRecovered
        | TraceEventType::ReasoningParticipantCompleted
        | TraceEventType::ReasoningDisagreementRecorded
        | TraceEventType::ReasoningDebateRoundCompleted
        | TraceEventType::ReasoningReflexionRevisionCompleted
        | TraceEventType::ReasoningAdjudicationRecorded
        | TraceEventType::ReasoningConfidenceRecorded
        | TraceEventType::ProjectScaleStageTransitioned
        | TraceEventType::TerminalRecorded => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Completed, "activity completed")
        }
        TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::ReasoningProfileInterrupted => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Awaiting, "awaiting follow-up")
        }
        TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected
        | TraceEventType::ReasoningProfileBlocked => {
            let mut outcome = SessionAuditOutcome::new(
                SessionAuditOutcomeStatus::Blocked,
                trace_event_summary(event),
            );
            outcome.blocking = true;
            outcome
        }
        TraceEventType::DecisionFailed | TraceEventType::ReasoningProfileEscalated => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
        TraceEventType::RetryScheduled | TraceEventType::StageRetryScheduled => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Retried, trace_event_summary(event))
        }
        TraceEventType::Replanned | TraceEventType::StageReplanned => SessionAuditOutcome::new(
            SessionAuditOutcomeStatus::Replanned,
            trace_event_summary(event),
        ),
        TraceEventType::StageFailed => {
            SessionAuditOutcome::new(SessionAuditOutcomeStatus::Failed, trace_event_summary(event))
        }
    }
}

fn trace_event_audit_message(event: &TraceEvent) -> String {
    let event_label = trace_event_type_text(event.event_type).replace('_', " ");
    let summary = trace_event_summary(event);
    if summary == event_label { event_label } else { format!("{event_label}: {summary}") }
}

fn trace_event_summary(event: &TraceEvent) -> String {
    payload_string(event.payload.get("summary"))
        .or_else(|| payload_string(event.payload.get("reason")))
        .or_else(|| payload_string(event.payload.get("message")))
        .or_else(|| payload_string(event.payload.get("headline")))
        .or_else(|| {
            payload_string(event.payload.get("target")).map(|target| format!("target {target}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("stage_key"))
                .map(|stage_key| format!("stage {stage_key}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("reviewer_id"))
                .map(|reviewer_id| format!("reviewer {reviewer_id}"))
        })
        .or_else(|| {
            payload_string(event.payload.get("participant_id"))
                .map(|participant_id| format!("participant {participant_id}"))
        })
        .unwrap_or_else(|| trace_event_type_text(event.event_type).replace('_', " "))
}

fn trace_event_audit_actor(event: &TraceEvent) -> SessionAuditActor {
    match event.event_type {
        TraceEventType::ReviewerStarted | TraceEventType::ReviewerCompleted => {
            reviewer_audit_actor(&event.payload)
        }
        TraceEventType::ReviewAdjudicated => reviewer_audit_actor(&event.payload),
        TraceEventType::ReviewVoteResolved => review_council_audit_actor(&event.payload),
        TraceEventType::ReasoningParticipantStarted
        | TraceEventType::ReasoningParticipantCompleted => {
            reasoning_participant_audit_actor(&event.payload)
        }
        TraceEventType::GovernanceSelected
        | TraceEventType::GovernanceStarted
        | TraceEventType::GovernanceDecisionRecorded
        | TraceEventType::GovernanceAwaitingApproval
        | TraceEventType::GovernanceCompleted
        | TraceEventType::GovernanceBlocked
        | TraceEventType::GovernancePacketRejected => governance_audit_actor(&event.payload),
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => SessionAuditActor {
            kind: SessionAuditActorKind::Agent,
            id: "boundline-decision-loop".to_string(),
            display_name: Some("Boundline Decision Loop".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        TraceEventType::ReviewStarted
        | TraceEventType::ReviewTriggerIgnored
        | TraceEventType::ReviewTerminalRecorded
        | TraceEventType::VotingDecisionRecorded => SessionAuditActor {
            kind: SessionAuditActorKind::Reviewer,
            id: "review-council".to_string(),
            display_name: Some("Review Council".to_string()),
            role: None,
            runtime_kind: None,
            provider: None,
            route_slot: None,
            model_name: None,
            participant_routes: Vec::new(),
            mixed_routes: false,
        },
        _ => SessionAuditActor::system("boundline"),
    }
}

fn reviewer_audit_actor(payload: &Value) -> SessionAuditActor {
    let reviewer_id = payload_string(payload.get("reviewer_id"))
        .unwrap_or_else(|| "unknown-reviewer".to_string());
    let reviewer_role = payload_string(payload.get("reviewer_role"));
    let reviewer_source = payload_string(payload.get("reviewer_source"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: reviewer_id.clone(),
        display_name: Some(reviewer_id),
        role: reviewer_role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };
    if let Some(source) = reviewer_source.as_deref() {
        apply_route_text_to_actor(&mut actor, source);
    }
    actor
}

fn review_council_audit_actor(payload: &Value) -> SessionAuditActor {
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::Reviewer,
        id: "review-council".to_string(),
        display_name: Some("Review Council".to_string()),
        role: None,
        runtime_kind: None,
        provider: None,
        route_slot: Some("review".to_string()),
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    let participants = payload
        .get("vote_resolution")
        .and_then(|value| value.get("participants"))
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerParticipation>>(value).ok())
        .unwrap_or_default();

    let completed_routes = participants
        .iter()
        .filter(|participant| participant.status == ReviewerParticipationStatus::Completed)
        .filter_map(|participant| participant.effective_route.as_deref())
        .map(str::trim)
        .filter(|route| !route.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    actor.participant_routes = completed_routes.clone();
    actor.mixed_routes = completed_routes.len() > 1;

    if let Some(route) = completed_routes.first() {
        apply_route_text_to_actor(&mut actor, route);
    }

    if actor.mixed_routes {
        actor.role = Some("multi-reviewer".to_string());
    }

    actor
}

fn reasoning_participant_audit_actor(payload: &Value) -> SessionAuditActor {
    let participant_id = payload_string(payload.get("participant_id"))
        .unwrap_or_else(|| "unknown-participant".to_string());
    let role = payload_string(payload.get("role"));
    let mut actor = SessionAuditActor {
        kind: SessionAuditActorKind::ReasoningParticipant,
        id: participant_id.clone(),
        display_name: Some(participant_id.clone()),
        role,
        runtime_kind: None,
        provider: None,
        route_slot: None,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    };

    if let Some(record) = payload
        .get("reasoning_profile_record")
        .cloned()
        .and_then(|value| serde_json::from_value::<ProfileActivationRecord>(value).ok())
        && let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id == participant_id)
    {
        actor.provider = participant.provider_family.clone();
        apply_route_text_to_actor(&mut actor, &participant.effective_route);
    }

    actor
}

fn governance_audit_actor(payload: &Value) -> SessionAuditActor {
    let runtime = payload_string(payload.get("runtime"))
        .or_else(|| payload_string(payload.get("selected_runtime")))
        .unwrap_or_else(|| "governance".to_string());
    let route_slot = payload_string(payload.get("stage_key"))
        .as_deref()
        .and_then(governance_route_slot_for_stage_key)
        .map(str::to_string);
    SessionAuditActor {
        kind: SessionAuditActorKind::GovernanceRuntime,
        id: runtime.clone(),
        display_name: Some(runtime.clone()),
        role: payload_string(payload.get("stage_key")),
        runtime_kind: Some(runtime),
        provider: payload_string(payload.get("runtime"))
            .or_else(|| payload_string(payload.get("selected_runtime"))),
        route_slot,
        model_name: None,
        participant_routes: Vec::new(),
        mixed_routes: false,
    }
}

fn governance_route_slot_for_stage_key(stage_key: &str) -> Option<&'static str> {
    let stage_key = stage_key.trim();
    if stage_key.is_empty() {
        return None;
    }

    if stage_key.starts_with("plan:") {
        return Some("planning");
    }

    Some("implementation")
}

fn apply_route_text_to_actor(actor: &mut SessionAuditActor, route_text: &str) {
    if let Some((route_slot, runtime, model)) = parse_three_segment_route(route_text) {
        actor.route_slot = Some(route_slot);
        actor.runtime_kind = Some(runtime.clone());
        actor.provider.get_or_insert(runtime);
        actor.model_name = Some(model);
        return;
    }

    if let Some((runtime, model)) = route_text.split_once('/') {
        let runtime = runtime.trim();
        let model = model.trim();
        if !runtime.is_empty() {
            actor.runtime_kind = Some(runtime.to_string());
            actor.provider.get_or_insert(runtime.to_string());
        }
        if !model.is_empty() {
            actor.model_name = Some(model.to_string());
        }
    }
}

fn parse_three_segment_route(route_text: &str) -> Option<(String, String, String)> {
    let mut parts = route_text.splitn(3, ':');
    let route_slot = parts.next()?.trim();
    let runtime = parts.next()?.trim();
    let model = parts.next()?.trim();
    if route_slot.is_empty() || runtime.is_empty() || model.is_empty() {
        return None;
    }
    Some((route_slot.to_string(), runtime.to_string(), model.to_string()))
}

fn payload_string(value: Option<&Value>) -> Option<String> {
    let value = value?;
    match value {
        Value::Null => None,
        Value::String(text) => Some(text.clone()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => serde_json::to_string(value).ok(),
    }
}

fn trace_event_type_text(event_type: TraceEventType) -> String {
    serde_json::to_value(event_type)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Errors surfaced while orchestrating session-native planning, execution,
/// governance, checkpoints, and persisted trace/session updates.
#[derive(Debug, Error)]
pub enum SessionRuntimeError {
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("trace store operation failed: {0}")]
    TraceStore(#[from] TraceStoreError),
    #[error("session audit store operation failed: {0}")]
    SessionAuditStore(#[from] SessionAuditStoreError),
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
    #[error(
        "active session planning governance for `{stage_key}` is `{state}` and must be resolved before confirmation or execution can continue"
    )]
    PlanningGovernanceUnresolved {
        stage_key: String,
        state: GovernanceLifecycleState,
        reason: Option<String>,
    },
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

fn default_planning_system_context(mode: CanonMode) -> SystemContextBinding {
    if mode.requires_existing_context() {
        SystemContextBinding::Existing
    } else {
        SystemContextBinding::New
    }
}

fn parse_planning_system_context(raw: &str) -> Option<SystemContextBinding> {
    match raw.trim() {
        SYSTEM_CONTEXT_NEW_TEXT => Some(SystemContextBinding::New),
        SYSTEM_CONTEXT_EXISTING_TEXT => Some(SystemContextBinding::Existing),
        _ => None,
    }
}

/// Reads a well-known artifact file from a Canon packet directory and returns
/// its content capped at `UPSTREAM_EVIDENCE_MAX_CHARS`. Returns `None` when the
/// file does not exist or is empty, ensuring graceful degradation when upstream
/// stages produced fewer artifacts than expected.
fn read_upstream_artifact_capped(packet_dir: &Path, file_name: &str) -> Option<String> {
    let path = packet_dir.join(file_name);
    let content = fs::read_to_string(&path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().count() <= UPSTREAM_EVIDENCE_MAX_CHARS {
        return Some(trimmed.to_string());
    }
    Some(truncate_with_ellipsis_marker(trimmed, UPSTREAM_EVIDENCE_MAX_CHARS))
}

fn truncate_with_ellipsis_marker(text: &str, max_chars: usize) -> String {
    let Some((end_index, _)) = text.char_indices().nth(max_chars) else {
        return text.to_string();
    };
    let mut truncated = text[..end_index].to_string();
    truncated.push_str("\n\n[truncated]");
    truncated
}

fn execution_governance_read_targets(
    native_context: &TaskContext,
    fallback_targets: &[String],
) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for state_key in [LATEST_CHANGED_FILES_KEY, "changed_files"] {
        if let Some(changed_files) = native_context.state.get(state_key).and_then(Value::as_array) {
            for changed_file in changed_files.iter().filter_map(Value::as_str) {
                if !changed_file.trim().is_empty() {
                    targets.insert(changed_file.to_string());
                }
            }
        }
    }

    if targets.is_empty() {
        for target in fallback_targets {
            if !target.trim().is_empty() {
                targets.insert(target.clone());
            }
        }
    }

    targets.into_iter().collect()
}

fn missing_planning_governance_field(mode: CanonMode, field: &'static str) -> SessionRuntimeError {
    SessionRuntimeError::GoalPlan(format!(
        "planning governance for Canon mode {} requires field '{field}'",
        mode.as_str()
    ))
}

fn render_planning_stage_brief(
    stage_key: &str,
    mode: CanonMode,
    goal_plan: &GoalPlan,
    context_sources: &PlanningContextSources,
) -> String {
    let flow_name = goal_plan
        .flow
        .as_ref()
        .map(|flow| flow.flow_name.as_str())
        .unwrap_or(PLANNING_UNSPECIFIED_FLOW);
    let target_summary = goal_plan
        .context_pack
        .as_ref()
        .filter(|context_pack| !context_pack.selected_targets.is_empty())
        .map(|context_pack| context_pack.selected_targets.join(", "))
        .unwrap_or_else(|| PLANNING_DEFAULT_TARGET.to_string());
    let context_summary = goal_plan
        .context_summary()
        .unwrap_or_else(|| "no bounded context summary recorded".to_string());
    let primary_inputs = goal_plan.context_primary_inputs();
    let primary_inputs =
        if primary_inputs.is_empty() { "none".to_string() } else { primary_inputs.join(", ") };
    let authored_inputs = if context_sources.authored_input_sources.is_empty() {
        "none".to_string()
    } else {
        context_sources.authored_input_sources.join(", ")
    };

    let mut brief = format!(
        concat!(
            "{title}\n\n",
            "{output_lang_heading}\n",
            "- instruction: {output_lang_instruction}\n\n",
            "{overview}\n",
            "- stage_key: {stage_key}\n",
            "- canon_mode: {mode}\n",
            "- flow: {flow_name}\n",
            "- goal: {goal}\n",
            "- targets: {targets}\n\n",
            "{workflow}\n",
            "- planning_rationale: {planning_rationale}\n",
            "- verification_strategy: {verification_strategy}\n\n",
            "{context}\n",
            "- summary: {context_summary}\n",
            "- primary_inputs: {primary_inputs}\n\n",
            "{authored}\n",
            "- authored_input_summary: {authored_input_summary}\n",
            "- authored_input_sources: {authored_inputs}\n"
        ),
        title = PLANNING_STAGE_BRIEF_TITLE,
        output_lang_heading = PLANNING_STAGE_OUTPUT_LANGUAGE_HEADING,
        output_lang_instruction = PLANNING_STAGE_OUTPUT_LANGUAGE_INSTRUCTION,
        overview = PLANNING_STAGE_OVERVIEW_HEADING,
        workflow = PLANNING_STAGE_WORKFLOW_HEADING,
        context = PLANNING_STAGE_CONTEXT_HEADING,
        authored = PLANNING_STAGE_AUTHORED_INPUTS_HEADING,
        stage_key = stage_key,
        mode = mode.as_str(),
        flow_name = flow_name,
        goal = goal_plan.goal_text,
        targets = target_summary,
        planning_rationale = goal_plan.planning_rationale.as_deref().unwrap_or("none"),
        verification_strategy = goal_plan.verification_strategy.as_deref().unwrap_or("none"),
        context_summary = context_summary,
        primary_inputs = primary_inputs,
        authored_input_summary =
            context_sources.authored_input_summary.as_deref().unwrap_or("none"),
        authored_inputs = authored_inputs,
    );

    if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
        brief.push_str("\n\n");
        brief.push_str(PLANNING_STAGE_CANON_MEMORY_HEADING);
        brief.push('\n');
        brief.push_str("- summary: ");
        brief.push_str(&memory.summary_text());
        brief.push('\n');
        brief.push_str("- credibility: ");
        brief.push_str(memory.credibility.as_str());
        brief.push('\n');
    }

    brief.push_str("\n\n## Problem Domain\n");
    brief.push_str("- domain: ");
    brief.push_str(&planning_problem_domain(goal_plan));
    brief.push('\n');

    brief.push_str("\n## Known Facts\n");
    brief.push_str("- goal: ");
    brief.push_str(&goal_plan.goal_text);
    brief.push('\n');
    brief.push_str("- selected_targets: ");
    brief.push_str(&target_summary);
    brief.push('\n');
    brief.push_str("- primary_inputs: ");
    brief.push_str(&primary_inputs);
    brief.push('\n');
    brief.push_str("- authored_inputs: ");
    brief.push_str(&authored_inputs);
    brief.push('\n');

    // Insert the structured goal decomposition between Known Facts and Unknowns.
    // This section gives Canon templates the authored body sections they need
    // (Problem, Outcome, Constraints, Entities, Operations, Validation) so they
    // produce substantive content instead of "NOT CAPTURED" placeholder stubs.
    if let Some(decomposition_section) = render_goal_decomposition_section(&goal_plan.goal_text) {
        brief.push_str(&decomposition_section);
    }

    brief.push_str("\n## Unknowns\n");
    for unknown in planning_unknown_markers(
        &goal_plan.goal_text,
        goal_plan.verification_strategy.as_deref(),
        !context_sources.authored_input_sources.is_empty(),
    ) {
        brief.push_str("- ");
        brief.push_str(&unknown);
        brief.push('\n');
    }

    brief.push_str("\n## Assumptions\n");
    for assumption in planning_assumptions(goal_plan) {
        brief.push_str("- ");
        brief.push_str(&assumption);
        brief.push('\n');
    }

    brief.push_str("\n## Validation Targets\n");
    brief.push_str("- strategy: ");
    brief.push_str(goal_plan.verification_strategy.as_deref().unwrap_or(
        "operator must provide validation command or acceptance evidence before execution",
    ));
    brief.push('\n');

    brief.push_str("\n## Confidence Levels\n");
    brief.push_str("- context_pack: ");
    brief.push_str(
        goal_plan
            .context_pack
            .as_ref()
            .map(|context_pack| context_pack.credibility.as_str())
            .unwrap_or("unavailable"),
    );
    brief.push('\n');
    brief.push_str("- authored_input: ");
    brief.push_str(if context_sources.authored_input_summary.is_some() {
        "operator_authored"
    } else {
        "not_provided"
    });
    brief.push('\n');

    brief.push_str("\n## Discovery Handoff\n");
    brief.push_str("- handoff: use known facts as bounded evidence, preserve unknowns as questions, and reject the packet if discovery cannot convert assumptions into actionable requirements.\n");

    brief
}

fn render_execution_stage_brief(
    mode: CanonMode,
    goal_plan: &GoalPlan,
    decisions: &[Decision],
    native_context: &TaskContext,
    fallback_targets: &[String],
) -> String {
    let changed_files = execution_governance_read_targets(native_context, fallback_targets);
    let validation_status = native_context
        .state
        .get(LATEST_VALIDATION_STATUS_KEY)
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let mut brief = format!(
        concat!(
            "# Execution Governance Brief\n\n",
            "## Overview\n",
            "- canon_mode: {mode}\n",
            "- goal: {goal}\n",
            "- plan_revision: {plan_revision}\n"
        ),
        mode = mode.as_str(),
        goal = goal_plan.goal_text,
        plan_revision = goal_plan.proposal_revision,
    );

    brief.push_str("\n## Changed Files\n");
    if changed_files.is_empty() {
        brief.push_str("- no bounded file targets were recorded\n");
    } else {
        for changed_file in &changed_files {
            brief.push_str("- ");
            brief.push_str(changed_file);
            brief.push('\n');
        }
    }

    brief.push_str("\n## Validation\n");
    brief.push_str("- status: ");
    brief.push_str(validation_status);
    brief.push('\n');

    if let Some(memory) = goal_plan.compacted_canon_memory.as_ref() {
        brief.push_str("\n## Canon Memory\n");
        brief.push_str("- summary: ");
        brief.push_str(&memory.summary_text());
        brief.push('\n');
        brief.push_str("- credibility: ");
        brief.push_str(memory.credibility.as_str());
        brief.push('\n');
    }

    brief.push_str("\n## Decision Summary\n");
    let mut rendered_any_decision = false;
    for decision in decisions
        .iter()
        .filter(|decision| decision.status.is_terminal())
        .take(EXECUTION_BRIEF_MAX_DECISIONS)
    {
        let decision_type = match decision.decision_type {
            DecisionType::Analyze => "analyze",
            DecisionType::Code => "code",
            DecisionType::Test => "test",
            DecisionType::Fix => "fix",
            DecisionType::Replan => "replan",
        };
        let decision_status = match decision.status {
            crate::domain::decision::DecisionStatus::Pending => "pending",
            crate::domain::decision::DecisionStatus::Dispatched => "dispatched",
            crate::domain::decision::DecisionStatus::Verified => "verified",
            crate::domain::decision::DecisionStatus::Failed => "failed",
            crate::domain::decision::DecisionStatus::Recovered => "recovered",
        };
        brief.push_str("- ");
        brief.push_str(decision_type);
        brief.push_str(": ");
        brief.push_str(&decision.target);
        brief.push_str(" (status: ");
        brief.push_str(decision_status);
        brief.push_str(") -> ");
        brief.push_str(&decision.expected_outcome);
        brief.push('\n');
        rendered_any_decision = true;
    }

    if !rendered_any_decision {
        brief.push_str("- no terminal decisions were recorded\n");
    }

    brief
}

fn planning_problem_domain(goal_plan: &GoalPlan) -> String {
    let lower = goal_plan.goal_text.to_ascii_lowercase();
    if lower.contains("user") || lower.contains("oauth") || lower.contains("auth") {
        "user management and authentication".to_string()
    } else if lower.contains("api") || lower.contains("grpc") || lower.contains("service") {
        "service/API delivery".to_string()
    } else {
        "bounded delivery target from captured goal".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Goal Decomposition
// ─────────────────────────────────────────────────────────────────────────────

/// Structured decomposition of a goal string into the semantic sections that
/// Canon governance templates expect as authored body content.
///
/// # Why this exists
///
/// Canon's requirements template generates multiple artifacts (prd.md,
/// tradeoffs.md, constraints.md, etc.) by reading structured sections from the
/// planning brief. When the brief contains only a flat goal string under
/// `## Known Facts`, Canon cannot locate a `## Problem`, `## Outcome`, or
/// `## Constraints` section and emits "NOT CAPTURED" placeholder stubs for
/// every missing section in every output artifact.
///
/// `decompose_goal_text` performs best-effort deterministic parsing of the
/// goal string to extract these sections. The decomposition is keyword-based
/// and does NOT invoke an external LLM; it splits on well-known structural
/// markers ("Persistence:", "Auth:", "Intended outcome:", entity/operation
/// patterns) to produce content Canon can reference as authored body.
///
/// # Degradation contract
///
/// If a section cannot be extracted, its field is `None` (or empty vec).
/// The brief renderer writes only the sections that have content, so an
/// empty decomposition produces output identical to the previous behavior.
/// This guarantees backward compatibility with goals that don't follow
/// recognizable patterns.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GoalDecomposition {
    /// The core problem statement: what is being built and why.
    /// Extracted from text preceding structural markers like "Persistence:"
    /// or "Intended outcome:".
    pub problem: Option<String>,

    /// The desired deliverable outcome.
    /// Extracted from text following "Intended outcome:" or "Desired outcome:".
    pub outcome: Option<String>,

    /// Technical constraints binding the implementation.
    /// Each entry is a single constraint (e.g. "Persistence: in-memory",
    /// "Auth: OAuth2 JWT at service level").
    pub constraints: Vec<String>,

    /// Domain entities with their attributes.
    /// Extracted from "Users: first name, last name, ..." or similar patterns.
    pub entities: Vec<String>,

    /// API operations, endpoints, or RPC methods in scope.
    /// Extracted from comma-separated lists of operation names or CRUD
    /// expansion patterns.
    pub operations: Vec<String>,

    /// The validation strategy or acceptance criteria.
    /// Extracted from "Validation:" markers or test command references.
    pub validation: Option<String>,
}

impl GoalDecomposition {
    /// Returns `true` when the decomposition extracted at least one
    /// substantive section that Canon templates can consume.
    pub fn has_content(&self) -> bool {
        self.problem.is_some()
            || self.outcome.is_some()
            || !self.constraints.is_empty()
            || !self.entities.is_empty()
            || !self.operations.is_empty()
            || self.validation.is_some()
    }
}

/// Performs best-effort structured decomposition of a goal string.
///
/// Parses the goal text for recognizable patterns and extracts semantic
/// sections that align with Canon template expectations. The extraction is
/// deterministic and keyword-driven; no external LLM call is made.
///
/// # Recognized patterns
///
/// | Pattern | Extracted as |
/// |---------|-------------|
/// | Text before first structural marker | `problem` |
/// | "Intended outcome:" / "Desired outcome:" | `outcome` |
/// | "Persistence:" clause | constraint |
/// | "Auth:" / "OAuth2" clause | constraint |
/// | "edition YYYY" / framework mentions | constraint |
/// | "Users:" or entity-attribute lists | entity |
/// | Comma-separated PascalCase names | operations |
/// | "CRUD" keyword expansion | operations |
/// | "Validation:" / test command | `validation` |
///
/// # Examples
///
/// ```text
/// goal = "Rust microservice (edition 2024), Axum + gRPC, user management
///         service. Users: first name, last name, email, role (Admin | User).
///         Persistence: in-memory. Auth: OAuth2 JWT. gRPC operations:
///         CreateUser, GetUser, ListUsers, UpdateUser, DeleteUser.
///         Intended outcome: a complete Cargo workspace with unit tests.
///         Validation: shell script with curl/grpcurl smoke tests."
///
/// result.problem = Some("Rust microservice (edition 2024), Axum + gRPC, user management service")
/// result.outcome = Some("a complete Cargo workspace with unit tests")
/// result.constraints = ["Persistence: in-memory store with no external database...",
///                       "Auth: OAuth2 JWT validated at service level",
///                       "Rust edition 2024", "Axum HTTP framework", "gRPC RPC surface"]
/// result.entities = ["Users: first name, last name, email, role (Admin | User)"]
/// result.operations = ["CreateUser", "GetUser", "ListUsers", "UpdateUser", "DeleteUser"]
/// result.validation = Some("shell script with curl/grpcurl smoke tests against running server")
/// ```
pub fn decompose_goal_text(goal: &str) -> GoalDecomposition {
    let mut decomposition = GoalDecomposition::default();
    let goal_trimmed = goal.trim();
    if goal_trimmed.is_empty() {
        return decomposition;
    }

    // ── Extract outcome (text after "Intended outcome:" or "Desired outcome:") ──
    let outcome_markers = ["intended outcome:", "desired outcome:"];
    let lower = goal_trimmed.to_ascii_lowercase();
    for marker in &outcome_markers {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let outcome_text =
                after.split_once('.').map(|(sentence, _)| sentence.trim()).unwrap_or(after.trim());
            if !outcome_text.is_empty() {
                decomposition.outcome = Some(outcome_text.to_string());
            }
            break;
        }
    }

    // ── Extract validation (text after "Validation:") ──
    let validation_markers = ["validation:", "acceptance:"];
    for marker in &validation_markers {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let validation_text =
                after.split_once('.').map(|(sentence, _)| sentence.trim()).unwrap_or(after.trim());
            if !validation_text.is_empty() {
                decomposition.validation = Some(validation_text.to_string());
            }
            break;
        }
    }
    // Fallback: detect test commands as validation
    if decomposition.validation.is_none() {
        let test_commands = ["cargo test", "npm test", "pytest", "go test"];
        for cmd in &test_commands {
            if lower.contains(cmd) {
                decomposition.validation = Some(format!("{cmd} (detected from goal text)"));
                break;
            }
        }
    }

    // ── Extract constraints ──
    // Persistence clause
    if let Some(pos) = lower.find("persistence:") {
        let after = &goal_trimmed[pos + "persistence:".len()..];
        let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
        if !clause.is_empty() {
            decomposition.constraints.push(format!("Persistence: {clause}"));
        }
    }
    // Auth clause
    if let Some(pos) = lower.find("auth:") {
        let after = &goal_trimmed[pos + "auth:".len()..];
        let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
        if !clause.is_empty() {
            decomposition.constraints.push(format!("Auth: {clause}"));
        }
    }
    // Edition constraint
    if lower.contains("edition 2024") || lower.contains("edition 2021") {
        let edition = if lower.contains("edition 2024") { "2024" } else { "2021" };
        decomposition.constraints.push(format!("Rust edition {edition}"));
    }
    // Framework constraints
    if lower.contains("axum") {
        decomposition.constraints.push("Axum HTTP framework".to_string());
    }
    if lower.contains("grpc") {
        decomposition.constraints.push("gRPC RPC surface".to_string());
    }
    if lower.contains("actix") {
        decomposition.constraints.push("Actix-web HTTP framework".to_string());
    }
    if lower.contains("tonic") {
        decomposition.constraints.push("Tonic gRPC framework".to_string());
    }

    // ── Extract entities ──
    // Pattern: "Users: attr, attr, attr" or "Entity: attr, attr"
    let entity_markers = ["users:", "user:", "entities:", "entity:"];
    for marker in &entity_markers {
        if let Some(pos) = lower.find(marker) {
            let capitalized_marker = &goal_trimmed[pos..pos + marker.len()];
            let after = &goal_trimmed[pos + marker.len()..];
            let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
            if !clause.is_empty() {
                decomposition.entities.push(format!("{capitalized_marker} {clause}"));
            }
        }
    }

    // ── Extract operations ──
    // Pattern: comma-separated PascalCase names (e.g. CreateUser, GetUser, ...)
    let operation_patterns =
        ["operations:", "operations in scope:", "rpcs:", "endpoints:", "methods:"];
    for marker in &operation_patterns {
        if let Some(pos) = lower.find(marker) {
            let after = &goal_trimmed[pos + marker.len()..];
            let clause = after.split_once('.').map(|(s, _)| s.trim()).unwrap_or(after.trim());
            for op in clause.split(',') {
                let op = op.trim();
                if !op.is_empty() && op.len() < 60 {
                    decomposition.operations.push(op.to_string());
                }
            }
            break;
        }
    }
    // Fallback: detect PascalCase comma-separated lists like "CreateUser, GetUser, ..."
    if decomposition.operations.is_empty() {
        let pascal_ops: Vec<&str> = goal_trimmed
            .split(',')
            .map(|s| s.trim())
            .filter(|s| {
                s.len() > 3
                    && s.len() < 40
                    && s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && s.chars().any(|c| c.is_ascii_lowercase())
                    && s.chars().all(|c| c.is_alphanumeric())
            })
            .collect();
        // Only use if we found 3+ consecutive PascalCase items (likely operations)
        if pascal_ops.len() >= 3 {
            decomposition.operations = pascal_ops.into_iter().map(|s| s.to_string()).collect();
        }
    }

    // ── Extract problem (text before the first structural marker) ──
    let structural_markers = [
        "persistence:",
        "auth:",
        "intended outcome:",
        "desired outcome:",
        "validation:",
        "acceptance:",
    ];
    let first_marker_pos = structural_markers.iter().filter_map(|marker| lower.find(marker)).min();
    if let Some(pos) = first_marker_pos {
        let problem_text = goal_trimmed[..pos].trim().trim_end_matches('.');
        if !problem_text.is_empty() {
            decomposition.problem = Some(problem_text.to_string());
        }
    } else if decomposition.outcome.is_none() {
        // No structural markers at all — use the first sentence as the problem
        let first_sentence =
            goal_trimmed.split_once('.').map(|(s, _)| s.trim()).unwrap_or(goal_trimmed);
        if !first_sentence.is_empty() {
            decomposition.problem = Some(first_sentence.to_string());
        }
    }

    decomposition
}

/// Renders the `## Structured Goal Decomposition` section for the planning brief.
///
/// This section exists to satisfy Canon template expectations for authored body
/// content. Canon's requirements mode generates artifacts (prd.md, tradeoffs.md,
/// etc.) by reading structured `### Problem`, `### Desired Outcome`, etc.
/// subsections. Without them, Canon emits "NOT CAPTURED" placeholder stubs.
///
/// The section is only included when `decompose_goal_text` extracts at least one
/// substantive field. An empty decomposition produces no output (backward compat).
fn render_goal_decomposition_section(goal_text: &str) -> Option<String> {
    let decomposition = decompose_goal_text(goal_text);
    if !decomposition.has_content() {
        return None;
    }

    let mut section = String::from("\n## Structured Goal Decomposition\n");

    if let Some(problem) = &decomposition.problem {
        section.push_str("### Problem\n");
        section.push_str(problem);
        section.push_str("\n\n");
    }

    if let Some(outcome) = &decomposition.outcome {
        section.push_str("### Desired Outcome\n");
        section.push_str(outcome);
        section.push_str("\n\n");
    }

    if !decomposition.constraints.is_empty() {
        section.push_str("### Constraints\n");
        for constraint in &decomposition.constraints {
            section.push_str("- ");
            section.push_str(constraint);
            section.push('\n');
        }
        section.push('\n');
    }

    if !decomposition.entities.is_empty() {
        section.push_str("### Domain Entities\n");
        for entity in &decomposition.entities {
            section.push_str("- ");
            section.push_str(entity);
            section.push('\n');
        }
        section.push('\n');
    }

    if !decomposition.operations.is_empty() {
        section.push_str("### Operations In Scope\n");
        for operation in &decomposition.operations {
            section.push_str("- ");
            section.push_str(operation);
            section.push('\n');
        }
        section.push('\n');
    }

    if let Some(validation) = &decomposition.validation {
        section.push_str("### Validation Criteria\n");
        section.push_str(validation);
        section.push('\n');
    }

    Some(section)
}

fn plain_goal_requires_planning_clarification(
    goal: &str,
    context_sources: &PlanningContextSources,
) -> bool {
    if !context_sources.authored_input_sources.is_empty()
        || !context_sources.execution_profile_read_targets.is_empty()
        || context_sources.latest_trace_ref.is_some()
        || !context_sources.latest_changed_files.is_empty()
        || context_sources.compacted_canon_memory.is_some()
    {
        return false;
    }

    let lower = goal.to_ascii_lowercase();
    let broad_delivery = lower.contains("build ")
        || lower.contains("deliver ")
        || lower.contains("capability")
        || lower.contains("microservice")
        || lower.contains("microservizio")
        || lower.contains("service")
        || lower.contains("api");
    let has_validation = lower.contains("cargo test")
        || lower.contains("validation")
        || lower.contains("acceptance")
        || lower.contains("verify");

    broad_delivery && !has_validation
}

fn plain_goal_planning_clarification_prompt() -> String {
    "Answer these planning questions before Boundline can continue planning: What exact outcome should Boundline deliver? Which domain entities and relationships are in scope? Which API operations, endpoints, or RPC methods are in scope? What persistence and OAuth/security assumptions are binding? Which validation command or acceptance evidence should prove the slice?".to_string()
}

pub fn planning_unknown_markers(
    goal_text: &str,
    verification_strategy: Option<&str>,
    has_authored_inputs: bool,
) -> Vec<String> {
    let lower = goal_text.to_ascii_lowercase();
    let mut unknowns = Vec::new();
    if !lower.contains("validation")
        && !lower.contains("cargo test")
        && !lower.contains("acceptance")
        && verification_strategy.unwrap_or("none") == "none"
    {
        unknowns.push("validation_target requires operator confirmation".to_string());
    }
    if !lower.contains("database")
        && !lower.contains("postgres")
        && !lower.contains("sqlite")
        && !lower.contains("persist")
    {
        unknowns.push("persistence assumptions require operator confirmation".to_string());
    }
    if !has_authored_inputs {
        unknowns.push("authored source provenance is unavailable".to_string());
    }

    // Flag gaps detected by the structured goal decomposition so Canon can
    // explicitly mark them as requiring operator input rather than guessing.
    let decomposition = decompose_goal_text(goal_text);
    if decomposition.outcome.is_none() {
        unknowns.push("desired outcome could not be extracted from goal text and requires operator confirmation".to_string());
    }
    if decomposition.operations.is_empty() && lower.contains("service") {
        unknowns.push("API operations or endpoints in scope could not be identified and require operator specification".to_string());
    }
    if decomposition.entities.is_empty() && (lower.contains("user") || lower.contains("entity")) {
        unknowns.push(
            "domain entities and their attributes could not be parsed from goal text".to_string(),
        );
    }

    if unknowns.is_empty() {
        unknowns
            .push("no explicit unknown markers were detected from the captured brief".to_string());
    }
    unknowns
}

fn planning_assumptions(goal_plan: &GoalPlan) -> Vec<String> {
    let lower = goal_plan.goal_text.to_ascii_lowercase();
    let mut assumptions = Vec::new();
    if lower.contains("rust") {
        assumptions.push("language/runtime: Rust".to_string());
    }
    if lower.contains("axum") {
        assumptions.push("HTTP framework: Axum".to_string());
    }
    if lower.contains("grpc") {
        assumptions.push("RPC surface: gRPC".to_string());
    }
    if lower.contains("oauth") {
        assumptions.push("security: OAuth2 protected surface".to_string());
    }

    if assumptions.is_empty() {
        assumptions.push("no concrete technical assumptions were captured".to_string());
    }
    assumptions
}

fn discovery_stage_council_request(
    stage_key: &str,
    goal: &str,
    stage_brief_ref: &str,
) -> StageCouncilRequest {
    StageCouncilRequest {
        stage_key: stage_key.to_string(),
        goal: goal.to_string(),
        producer_slot: RouteSlot::Planning.as_str().to_string(),
        phase: "planning-discovery".to_string(),
        target_refs: vec![stage_brief_ref.to_string()],
        current_artifact_ref: Some(stage_brief_ref.to_string()),
        constraints: vec![
            "use independent reviewer routes when available".to_string(),
            "do not promote discovery planning when council independence collapses".to_string(),
        ],
    }
}

fn discovery_stage_council_reviewers(routing: &EffectiveRouting) -> Vec<StageCouncilReviewerRoute> {
    let configured = routing
        .reviewer_roles
        .iter()
        .take(2)
        .map(|(reviewer_id, reviewer_route)| StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: reviewer_id.clone(),
                role: reviewer_id.replace(['_', '-'], " "),
                source: Some(model_route_label(&reviewer_route.route)),
                weight: 1,
            },
            route: reviewer_route.route.clone(),
        })
        .collect::<Vec<_>>();
    if configured.len() == 2 {
        return configured;
    }

    let fallback_route = routing.review.route.clone();
    vec![
        StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: configured
                    .first()
                    .map(|route| route.reviewer.reviewer_id.clone())
                    .unwrap_or_else(|| "reviewer-a".to_string()),
                role: configured
                    .first()
                    .map(|route| route.reviewer.role.clone())
                    .unwrap_or_else(|| "discovery challenger a".to_string()),
                source: configured
                    .first()
                    .and_then(|route| route.reviewer.source.clone())
                    .or_else(|| Some(model_route_label(&fallback_route))),
                weight: 1,
            },
            route: configured
                .first()
                .map(|route| route.route.clone())
                .unwrap_or_else(|| fallback_route.clone()),
        },
        StageCouncilReviewerRoute {
            reviewer: ReviewerDefinition {
                reviewer_id: configured
                    .get(1)
                    .map(|route| route.reviewer.reviewer_id.clone())
                    .unwrap_or_else(|| "reviewer-b".to_string()),
                role: configured
                    .get(1)
                    .map(|route| route.reviewer.role.clone())
                    .unwrap_or_else(|| "discovery challenger b".to_string()),
                source: configured
                    .get(1)
                    .and_then(|route| route.reviewer.source.clone())
                    .or_else(|| Some(model_route_label(&fallback_route))),
                weight: 1,
            },
            route: configured.get(1).map(|route| route.route.clone()).unwrap_or(fallback_route),
        },
    ]
}

fn model_route_label(route: &ModelRoute) -> String {
    format!("{}/{}", route.runtime.as_str(), route.model)
}

fn reviewer_disposition_from_provider(
    disposition: ProviderReviewDisposition,
) -> ReviewerDisposition {
    match disposition {
        ProviderReviewDisposition::Approve => ReviewerDisposition::Approve,
        ProviderReviewDisposition::Concern => ReviewerDisposition::Concern,
        ProviderReviewDisposition::Block => ReviewerDisposition::Block,
    }
}

fn stage_council_disposition_from_provider(
    disposition: ProviderReviewDisposition,
) -> StageCouncilFindingDisposition {
    match disposition {
        ProviderReviewDisposition::Approve => StageCouncilFindingDisposition::Approve,
        ProviderReviewDisposition::Concern => StageCouncilFindingDisposition::Concern,
        ProviderReviewDisposition::Block => StageCouncilFindingDisposition::Block,
    }
}

fn provider_review_disposition_text(disposition: ProviderReviewDisposition) -> &'static str {
    match disposition {
        ProviderReviewDisposition::Approve => "approve",
        ProviderReviewDisposition::Concern => "concern",
        ProviderReviewDisposition::Block => "block",
    }
}

fn planning_stage_council_block_reason(stage_key: &str, outcome: &StageCouncilOutcome) -> String {
    let summary = outcome
        .reviewer_findings
        .iter()
        .find(|finding| finding.disposition == StageCouncilFindingDisposition::Block)
        .map(|finding| finding.summary.as_str())
        .unwrap_or(outcome.next_action.as_str());
    format!("{stage_key} stage council blocked planning: {summary}")
}

fn stage_council_voting_session_state(
    stage_key: &str,
    outcome: &StageCouncilOutcome,
) -> VotingSessionState {
    VotingSessionState {
        trigger: format!("stage_council:{stage_key}"),
        reviewed_evidence_ref: Some(outcome.producer_output.evidence_ref.clone()),
        result: stage_council_status_text(outcome.status).to_string(),
        reviewer_findings: outcome
            .reviewer_findings
            .iter()
            .map(|finding| {
                format!(
                    "{} [{}]: {}",
                    finding.reviewer_id, finding.effective_route, finding.summary
                )
            })
            .collect(),
        adjudication_result: outcome
            .adjudication
            .as_ref()
            .map(|adjudication| format!("{}: {}", adjudication.decision, adjudication.rationale)),
        blocking: outcome.status == StageCouncilStatus::Blocked,
        next_action: outcome.next_action.clone(),
    }
}

fn stage_council_status_text(status: StageCouncilStatus) -> &'static str {
    match status {
        StageCouncilStatus::Proceed => "proceed",
        StageCouncilStatus::Blocked => "blocked",
        StageCouncilStatus::Degraded => "degraded",
    }
}

fn render_stage_council_blocked_markdown(
    request: &StageCouncilRequest,
    findings: &[StageCouncilFinding],
    accepted_findings: &[String],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Discovery Stage Council Blocked\n\n");
    markdown.push_str(&format!("- stage: {}\n", request.stage_key));
    markdown.push_str("- outcome: blocked\n");
    if findings.is_empty() {
        markdown.push_str("- findings: no provider-backed reviewer findings were recorded\n");
    } else {
        markdown.push_str("\n## Findings\n\n");
        for finding in findings {
            let accepted = if accepted_findings.contains(&finding.reviewer_id) {
                "accepted"
            } else {
                "rejected"
            };
            markdown.push_str(&format!(
                "- {} [{}] {}: {}\n",
                finding.reviewer_id, finding.effective_route, accepted, finding.summary
            ));
        }
    }
    markdown.push_str(
        "\nRepair the discovery inputs or reviewer routing, then rerun `boundline plan`.\n",
    );
    markdown
}

fn render_stage_council_blocked_note(reason: &str) -> String {
    format!(
        "# Discovery Stage Council Blocked\n\n- reason: {reason}\n\nRerun `boundline plan` after restoring independent provider-backed council execution.\n"
    )
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
#[path = "session_runtime_tests.rs"]
mod tests;
