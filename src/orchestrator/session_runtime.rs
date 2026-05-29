//! Workspace-scoped session orchestration for planning, execution, governance,
//! checkpoints, and persisted trace updates.

use std::collections::{BTreeMap, BTreeSet};
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
    GovernanceRuntimeResponse, LocalGovernanceRuntime,
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
use crate::domain::brief::AuthoredBriefBundle;
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
use crate::domain::follow_through::FollowThroughProjection;
use crate::domain::goal_plan::{
    ContextPackCredibility, GoalPlan, PlanQualityAssessment, PlanQualityState,
    PlanningAnalysisProjection, PlanningAnalysisState,
};
use crate::domain::governance::{
    ApprovalState, BacklogQualityAssessment, BacklogQualityState, CanonAuthorityZone,
    CanonEvidenceInspectSummary, CanonIntendedPersona, CanonMode, CanonModeSelectionPreference,
    CanonPossibleActionSummary, CanonRecommendedActionSummary, CanonRiskClass,
    CompactedCanonMemory, CouncilProfile, GovernanceLifecycleState, GovernanceRuntimeKind,
    GovernedSessionLifecycle, GovernedStageRecord, MemoryCredibilityState, PacketReadiness,
    SystemContextBinding, backlog_quality_snapshot_for_lifecycle, execution_stage_key_for_mode,
    planned_canon_mode_sequence_for_flow, planning_canon_mode_for_stage_key,
    planning_canon_mode_sequence, planning_stage_brief_ref, planning_stage_key_for_mode,
    resolved_canon_mode,
};
use crate::domain::guidance::{CapabilityPhase, GuidanceGuardianProjection};
use crate::domain::limits::{RunLimits, TerminalCondition};
use crate::domain::negotiation::{NegotiatedDeliveryPacket, NegotiationResolutionState};
use crate::domain::project_memory::{
    GovernedEvidencePromotionRequest, ProjectMemoryCondition, ProjectMemoryContext,
    ProjectMemoryStatus, evidence_contribution_summaries, evidence_root_for_lineage,
    promote_governed_evidence_bundle as promote_project_evidence_bundle, read_project_memory,
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

const PLAN_QUALITY_BLOCKED_HEADLINE: &str = "bounded context required before planning";
const PLAN_QUALITY_BLOCKED_DEFAULT_PROMPT: &str =
    "refresh the bounded planning context before planning can continue";
const PLAN_QUALITY_CLARIFICATION_HEADLINE: &str = "planning clarification required";
const PLAN_QUALITY_CLARIFICATION_PROMPT_PREFIX: &str = "planning is missing the following inputs: ";
const PLAN_QUALITY_CLARIFICATION_DEFAULT_PROMPT: &str =
    "capture the missing planning inputs before planning can continue";

#[path = "session_runtime_checkpoint.rs"]
mod checkpoint;

#[path = "session_runtime_briefs.rs"]
mod briefs;

#[path = "session_runtime_planning_governance.rs"]
mod planning_governance;

#[path = "session_runtime_planning_runtime.rs"]
mod planning_runtime;

#[path = "session_runtime_planning_council.rs"]
mod planning_council;

#[path = "session_runtime_planning_context.rs"]
mod planning_context;

#[path = "session_runtime_runtime_support.rs"]
mod runtime_support;

#[path = "session_runtime_surface.rs"]
mod surface;

#[path = "session_runtime_planning_canon.rs"]
mod planning_canon;

#[path = "session_runtime_native_execution.rs"]
mod native_execution;

#[path = "session_runtime_native_governance.rs"]
mod native_governance;

#[path = "session_runtime_native_goal_plan.rs"]
mod native_goal_plan;

#[path = "session_runtime_native_review.rs"]
mod native_review;

#[path = "session_runtime_guardians.rs"]
mod guardians;

#[path = "session_runtime_finalization.rs"]
mod finalization;

#[path = "session_runtime_flow_trace.rs"]
mod flow_trace;

#[path = "session_runtime_execution_core.rs"]
mod execution_core;

#[path = "session_runtime_run_control.rs"]
mod run_control;

#[path = "session_runtime_step_execution.rs"]
mod step_execution;

#[path = "session_runtime_step_governance.rs"]
mod step_governance;

#[path = "session_runtime_reasoning.rs"]
mod reasoning;

pub use briefs::{GoalDecomposition, decompose_goal_text, planning_unknown_markers};

use briefs::{
    plain_goal_planning_clarification_prompt, plain_goal_requires_planning_clarification,
    render_execution_stage_brief, render_planning_stage_brief,
    render_stage_council_blocked_markdown, render_stage_council_blocked_note,
};
use checkpoint::{
    CheckpointProjectionState, apply_checkpoint_projection_to_context, checkpoint_event_payload,
    checkpoint_projection_from_context,
};
use planning_governance::{
    discovery_stage_council_request, discovery_stage_council_reviewers, model_route_label,
    planning_stage_council_block_reason, provider_review_disposition_text,
    reviewer_disposition_from_provider, stage_council_disposition_from_provider,
    stage_council_voting_session_state,
};
use reasoning::{
    GovernanceBlockContext, ReasoningGateContext, ReasoningTraceContext, is_governance_trace_event,
    reasoning_profile_block_message, store_latest_reasoning_profile,
};
use runtime_support::*;

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
/// to govern the given planning mode meaningfully. Discovery is always
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

#[cfg(test)]
#[path = "session_runtime_tests.rs"]
mod tests;
