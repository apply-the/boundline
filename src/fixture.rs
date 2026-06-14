//! Test fixture utilities and execution-profile loaders.
//!
//! Shared between integration tests and the legacy compatibility layer for
//! loading workspace execution profiles, bootstrapping fixture workspaces,
//! and driving the compatibility execution engine.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::cluster_store::FileClusterStore;
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::provider_runtime::{
    self, ProviderAnalysisRequest, ProviderChangeRequest, ProviderReviewDisposition,
    ProviderReviewRequest, ProviderWorkspaceFile,
};
use crate::adapters::tool::FnToolAdapter;
use crate::domain::brief::AuthoredBriefBundle;
use crate::domain::configuration::{
    ModelRoute, RoutingOverrides, resolve_effective_routing,
    resolve_effective_runtime_capabilities, resolve_effective_slot_effort_policies,
};
use crate::domain::distribution::SUPPORTED_CANON_VERSION;
use crate::domain::execution::{
    AdaptiveChangeKind, AttemptLineage, AttemptTransitionKind, ChangeEvidence, ChangeStatus,
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, ExecutionProfileError,
    PathScore, SelectionEvidence, ValidationGuidance, ValidationGuidanceConfidence,
    ValidationGuidanceSource, ValidationRecord, WorkspaceChange, WorkspaceExecutionProfile,
    WorkspaceSliceSelection,
};
use crate::domain::flow::{
    FLOW_METADATA_KEY, FlowStepMetadata, SessionFlowState, attach_stage_metadata, built_in_flow,
};
use crate::domain::goal_plan::GoalPlan;
use crate::domain::governance::CanonIntendedPersona;
use crate::domain::guidance::CapabilityPhase;
use crate::domain::limits::RunLimits;
use crate::domain::negotiation::NegotiatedDeliveryPacket;
use crate::domain::plan::Plan;
use crate::domain::reasoning::{
    CanonAdmissionPriority, CanonChallengePostureInput, IndependenceFloor, ProfileActivationRecord,
    REASONING_POSTURE_V1_CONTRACT_LINE, ReasoningActivationStatus, ReasoningActivationTrigger,
    ReasoningAdmissionEffect, ReasoningBudget, ReasoningCompatibilityWindow,
    ReasoningConfidenceContribution, ReasoningConfidenceLevel, ReasoningOutcome,
    ReasoningOutcomeKind, ReasoningProfileId,
};
use crate::domain::review::{
    ReviewOutcome, ReviewProfile, ReviewScenario, ReviewTrigger, ReviewerDefinition,
    ReviewerDisposition, ReviewerFinding, ReviewerParticipation, ReviewerParticipationStatus,
    VoteDecision, VoteResolution, VoteRuleDefinition,
};
use crate::domain::routing_decision::RoutingDecisionProjection;
use crate::domain::step::{
    ErrorInfo, Recoverability, Step, StepError, StepExecutionRequest, StepExecutionResult,
};
use crate::domain::task::TaskRunRequest;
use crate::domain::task_context::{
    LATEST_CLARIFICATION_KEY, LATEST_DERIVED_TASK_DRAFT_KEY, TASK_GOAL_KEY,
};
use crate::orchestrator::goal_planner::collect_workspace_signals;
use crate::orchestrator::governance::{bounded_reused_packets, select_packet_reuse_binding};
use crate::orchestrator::guidance_runtime::{GuidanceRuntimeEvidence, load_guidance_for_phase};
use crate::orchestrator::planner::{CallbackPlanner, Planner, PlanningError, StaticPlanner};
use crate::registry::agent_registry::{AgentRegistry, RegistryError as AgentRegistryError};
use crate::registry::tool_registry::{RegistryError as ToolRegistryError, ToolRegistry};

const EXECUTION_RELATIVE_PATH: &str = ".boundline/execution.json";
const MIN_ADAPTIVE_REPLAN_SCORE: i64 = 60;
const FIXTURE_REASONING_PROVENANCE_PREFIX: &str = "fixture:reasoning-posture:";
const FIXTURE_REASONING_ACTIVATION_REASON: &str = "fixture reasoning challenge activated";
const FIXTURE_REASONING_VERIFY_STAGE: &str = "bug-fix:verify";
const FIXTURE_REASONING_IMPLEMENT_STAGE: &str = "bug-fix:implement";
const FIXTURE_REASONING_MAX_BRANCHES: usize = 1;
const FIXTURE_REASONING_MAX_CALLS: usize = 2;
const FIXTURE_REASONING_MAX_TOKENS: usize = 8_000;
const FIXTURE_REASONING_MAX_ADJUDICATION_STEPS: usize = 1;
const FIXTURE_REASONING_BLOCKED_HEADLINE: &str = "independent pair review blocked";
const FIXTURE_REASONING_BLOCKED_DISAGREEMENT: &str = "reviewers collapsed onto one route";
const FIXTURE_REASONING_BLOCKED_NEXT_ACTION: &str = "configure distinct reviewer routes";
const FIXTURE_REASONING_BLOCKED_SUMMARY: &str =
    "reasoning independence failed; block progression until challenge distinctness is restored";
const FIXTURE_REASONING_REFLEXION_HEADLINE: &str = "bounded reflexion degraded";
const FIXTURE_REASONING_REFLEXION_NEXT_ACTION: &str =
    "run one bounded verification pass before merge";
const FIXTURE_REASONING_REFLEXION_SUMMARY: &str =
    "reflexion converged partially; continue with bounded warning semantics";
const NATIVE_GOAL_PLAN_LEGACY_SOURCE: &str = "native_goal_plan_synthesized";
const NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ID: &str = "provider-review";
const NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ROLE: &str = "Provider Review";
const NATIVE_GOAL_PLAN_PROVIDER_REVIEW_PLACEHOLDER_SUMMARY: &str =
    "provider-backed review pending runtime execution";
const PROVIDER_REVIEW_SNAPSHOT_FAILED_CODE: &str = "provider_review_snapshot_failed";
const PROVIDER_REVIEW_FAILED_CODE: &str = "provider_review_failed";
const PROVIDER_REVIEW_PRIOR_CONTEXT_KEYS: &[&str] = &[
    "latest_validation_status",
    "latest_validation_record",
    "latest_changed_files",
    "latest_change_evidence",
    "next_review_trigger",
    "latest_governance_stage",
    "last_output",
];

#[derive(Clone)]
pub struct FixtureRuntime {
    pub profile: WorkspaceExecutionProfile,
    pub planner: Arc<dyn Planner>,
    pub agents: AgentRegistry,
    pub tools: ToolRegistry,
}

#[derive(Debug, Clone)]
struct AdaptiveAttemptPlan {
    attempt: ExecutionAttemptDefinition,
    workspace_slice: WorkspaceSliceSelection,
    selection_evidence: SelectionEvidence,
    candidate_signature: String,
    attempt_lineage: AttemptLineage,
}

#[derive(Debug, Clone, Copy)]
struct AdaptiveCandidateContext<'a> {
    used_signatures: &'a BTreeSet<String>,
    previous_attempt_id: Option<&'a str>,
    previous_selected_targets: Option<&'a [String]>,
    validation_guidance: Option<&'a ValidationGuidance>,
    lineage_reason: &'a str,
}

#[derive(Debug, Clone)]
struct WorkspaceTargetSource {
    path: String,
    contents: String,
}

#[derive(Debug, Clone)]
struct GeneratedAdaptiveCandidate {
    change_kind: AdaptiveChangeKind,
    change: WorkspaceChange,
}

#[derive(Debug, Clone)]
struct RankedAdaptiveCandidate {
    change_kind: AdaptiveChangeKind,
    change: WorkspaceChange,
    signature: String,
    score: i64,
    order_index: usize,
    reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningProfileFixtureScenario {
    IndependentPairBlocked,
    BoundedReflexionWarn,
}

impl ReasoningProfileFixtureScenario {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IndependentPairBlocked => "independent_pair_blocked",
            Self::BoundedReflexionWarn => "bounded_reflexion_warn",
        }
    }
}

pub fn local_reasoning_posture_fixture() -> Result<CanonChallengePostureInput, FixtureRuntimeError>
{
    local_reasoning_posture_fixture_for_profile(
        ReasoningProfileId::IndependentPairReview,
        CanonAdmissionPriority::RequiredBeforeAcceptance,
    )
}

pub fn local_reasoning_posture_fixture_for_profile(
    profile_id: ReasoningProfileId,
    admission_priority: CanonAdmissionPriority,
) -> Result<CanonChallengePostureInput, FixtureRuntimeError> {
    let contract_line = REASONING_POSTURE_V1_CONTRACT_LINE.to_string();

    Ok(CanonChallengePostureInput {
        contract_line: contract_line.clone(),
        compatibility_window: ReasoningCompatibilityWindow {
            boundline_min: env!("CARGO_PKG_VERSION").to_string(),
            boundline_max_exclusive: fixture_next_minor_exclusive(env!("CARGO_PKG_VERSION"))?,
            canon_min: SUPPORTED_CANON_VERSION.to_string(),
            canon_max_exclusive: fixture_next_minor_exclusive(SUPPORTED_CANON_VERSION)?,
            contract_line,
        },
        required_profile_family: None,
        required_profile_id: Some(profile_id),
        minimum_independence: fixture_minimum_independence(profile_id),
        admission_priority,
        confidence_handoff_required: true,
        provenance_ref: format!("{FIXTURE_REASONING_PROVENANCE_PREFIX}{}", profile_id.as_str()),
    })
}

pub fn reasoning_profile_fixture(
    scenario: ReasoningProfileFixtureScenario,
) -> Result<ProfileActivationRecord, FixtureRuntimeError> {
    match scenario {
        ReasoningProfileFixtureScenario::IndependentPairBlocked => Ok(ProfileActivationRecord {
            activation_id: format!("fixture-{}", scenario.as_str()),
            stage_key: FIXTURE_REASONING_VERIFY_STAGE.to_string(),
            profile_id: ReasoningProfileId::IndependentPairReview,
            trigger: ReasoningActivationTrigger::OperatorPolicy,
            activation_reason: FIXTURE_REASONING_ACTIVATION_REASON.to_string(),
            status: ReasoningActivationStatus::Blocked,
            participants: Vec::new(),
            budget: fixture_reasoning_budget(ReasoningProfileId::IndependentPairReview),
            posture: Some(local_reasoning_posture_fixture()?),
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Blocked,
                headline: FIXTURE_REASONING_BLOCKED_HEADLINE.to_string(),
                disagreement_summary: Some(FIXTURE_REASONING_BLOCKED_DISAGREEMENT.to_string()),
                next_action: Some(FIXTURE_REASONING_BLOCKED_NEXT_ACTION.to_string()),
                iterations: Vec::new(),
            }),
            confidence: Some(ReasoningConfidenceContribution {
                confidence_level: ReasoningConfidenceLevel::Low,
                basis: vec!["independence=failed".to_string()],
                admission_effect: ReasoningAdmissionEffect::Gate,
                summary: FIXTURE_REASONING_BLOCKED_SUMMARY.to_string(),
            }),
        }),
        ReasoningProfileFixtureScenario::BoundedReflexionWarn => Ok(ProfileActivationRecord {
            activation_id: format!("fixture-{}", scenario.as_str()),
            stage_key: FIXTURE_REASONING_IMPLEMENT_STAGE.to_string(),
            profile_id: ReasoningProfileId::BoundedReflexion,
            trigger: ReasoningActivationTrigger::OperatorPolicy,
            activation_reason: FIXTURE_REASONING_ACTIVATION_REASON.to_string(),
            status: ReasoningActivationStatus::Degraded,
            participants: Vec::new(),
            budget: fixture_reasoning_budget(ReasoningProfileId::BoundedReflexion),
            posture: Some(local_reasoning_posture_fixture_for_profile(
                ReasoningProfileId::BoundedReflexion,
                CanonAdmissionPriority::Advisory,
            )?),
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Degraded,
                headline: FIXTURE_REASONING_REFLEXION_HEADLINE.to_string(),
                disagreement_summary: None,
                next_action: Some(FIXTURE_REASONING_REFLEXION_NEXT_ACTION.to_string()),
                iterations: Vec::new(),
            }),
            confidence: Some(ReasoningConfidenceContribution {
                confidence_level: ReasoningConfidenceLevel::Medium,
                basis: vec!["reflexion=partial_convergence".to_string()],
                admission_effect: ReasoningAdmissionEffect::Warn,
                summary: FIXTURE_REASONING_REFLEXION_SUMMARY.to_string(),
            }),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceFixture {
    pub name: String,
    #[serde(default = "default_test_command")]
    pub test_command: FixtureCommand,
    #[serde(default = "default_run_limits")]
    pub limits: RunLimits,
    #[serde(default)]
    pub file_patches: Vec<FilePatch>,
}

impl WorkspaceFixture {
    pub fn validate(&self) -> Result<(), FixtureValidationError> {
        if self.name.trim().is_empty() {
            return Err(FixtureValidationError::MissingName);
        }

        if self.test_command.program.trim().is_empty() {
            return Err(FixtureValidationError::MissingTestProgram);
        }

        if self.file_patches.is_empty() {
            return Err(FixtureValidationError::MissingFilePatches);
        }

        self.limits
            .validate()
            .map_err(|error| FixtureValidationError::InvalidRunLimits(error.to_string()))?;

        for patch in &self.file_patches {
            patch.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: String,
    pub find: String,
    pub replace: String,
}

impl FilePatch {
    fn validate(&self) -> Result<(), FixtureValidationError> {
        if self.path.trim().is_empty() {
            return Err(FixtureValidationError::MissingPatchPath);
        }

        if Path::new(&self.path).is_absolute() {
            return Err(FixtureValidationError::AbsolutePatchPath(self.path.clone()));
        }

        if self.find.is_empty() {
            return Err(FixtureValidationError::MissingFindPattern(self.path.clone()));
        }

        Ok(())
    }
}

pub fn execution_manifest_path(workspace: &Path) -> PathBuf {
    workspace.join(EXECUTION_RELATIVE_PATH)
}

fn fixture_next_minor_exclusive(version: &str) -> Result<String, FixtureRuntimeError> {
    let mut segments = version.split('.');
    let major =
        segments.next().and_then(|segment| segment.parse::<u64>().ok()).ok_or_else(|| {
            FixtureRuntimeError::InvalidReasoningFixtureVersion { version: version.to_string() }
        })?;
    let minor =
        segments.next().and_then(|segment| segment.parse::<u64>().ok()).ok_or_else(|| {
            FixtureRuntimeError::InvalidReasoningFixtureVersion { version: version.to_string() }
        })?;
    let patch_is_valid = segments.next().and_then(|segment| segment.parse::<u64>().ok()).is_some();

    if !patch_is_valid || segments.next().is_some() {
        return Err(FixtureRuntimeError::InvalidReasoningFixtureVersion {
            version: version.to_string(),
        });
    }

    Ok(format!("{major}.{}.0", minor + 1))
}

fn fixture_minimum_independence(profile_id: ReasoningProfileId) -> IndependenceFloor {
    match profile_id {
        ReasoningProfileId::BoundedSelfConsistency | ReasoningProfileId::BoundedReflexion => {
            IndependenceFloor {
                route_distinct: false,
                provider_distinct: false,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 1,
            }
        }
        ReasoningProfileId::IndependentPairReview
        | ReasoningProfileId::HeterogeneousSecurityReview => IndependenceFloor {
            route_distinct: true,
            provider_distinct: true,
            context_distinct: false,
            prompt_pattern_distinct: false,
            minimum_participants: 2,
        },
    }
}

fn fixture_reasoning_budget(profile_id: ReasoningProfileId) -> ReasoningBudget {
    let max_participants = profile_id.family().minimum_participants();
    let max_reflexion_revisions =
        if profile_id == ReasoningProfileId::BoundedReflexion { 1 } else { 0 };

    ReasoningBudget {
        max_participants,
        max_branches: FIXTURE_REASONING_MAX_BRANCHES,
        max_debate_rounds: 0,
        max_reflexion_revisions,
        max_calls: FIXTURE_REASONING_MAX_CALLS,
        max_tokens: FIXTURE_REASONING_MAX_TOKENS,
        max_adjudication_steps: FIXTURE_REASONING_MAX_ADJUDICATION_STEPS,
    }
}

pub fn load_workspace_execution_profile(
    workspace: &Path,
) -> Result<WorkspaceExecutionProfile, FixtureRuntimeError> {
    let execution_path = execution_manifest_path(workspace);
    if !execution_path.is_file() {
        return Err(FixtureRuntimeError::MissingExecutionProfile(execution_path));
    }

    let contents = fs::read_to_string(&execution_path).map_err(|source| {
        FixtureRuntimeError::ExecutionProfileRead { path: execution_path.clone(), source }
    })?;
    let profile =
        serde_json::from_str::<WorkspaceExecutionProfile>(&contents).map_err(|source| {
            FixtureRuntimeError::ExecutionProfileParse { path: execution_path.clone(), source }
        })?;
    profile.validate()?;
    Ok(profile)
}

pub fn build_fixture_plan(workspace: &Path) -> Result<Plan, FixtureRuntimeError> {
    build_fixture_plan_for_goal(workspace, None, "")
}

pub fn build_fixture_plan_for_flow(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
) -> Result<Plan, FixtureRuntimeError> {
    build_fixture_plan_for_goal(workspace, active_flow, "")
}

pub fn build_fixture_plan_for_goal(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
    goal: &str,
) -> Result<Plan, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;

    if profile.adaptive.is_some() {
        return build_adaptive_initial_plan(workspace, &profile, active_flow, goal);
    }

    build_vertical_slice_plan(&profile, active_flow, 0)
}

pub fn build_task_request(
    workspace: &Path,
    goal: impl Into<String>,
    session_id: impl Into<String>,
    authored_brief: Option<&AuthoredBriefBundle>,
    negotiation_packet: Option<&NegotiatedDeliveryPacket>,
) -> Result<TaskRunRequest, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;
    let goal = goal.into();
    let session_id = session_id.into();
    let mut input = Map::new();
    let mut initial_context = Map::new();
    input.insert("execution_profile".to_string(), json!(profile.name));
    input.insert("flow".to_string(), json!("workspace_execution"));
    initial_context.insert("goal".to_string(), json!(&goal));

    if let Some(routing_projection) = workspace_routing_projection(workspace) {
        input.insert("routing_projection".to_string(), json!(routing_projection));
        initial_context.insert("routing_projection".to_string(), json!(routing_projection));
    }

    let negotiation_packet = negotiation_packet.cloned().or_else(|| {
        authored_brief.map(|bundle| {
            NegotiatedDeliveryPacket::from_authored_brief(
                &session_id,
                &workspace.to_string_lossy(),
                &goal,
                bundle,
            )
        })
    });

    if let Some(packet) = negotiation_packet.as_ref() {
        input.insert("negotiation_goal_summary".to_string(), json!(&packet.goal_summary));
        input.insert("negotiation_resolution".to_string(), json!(packet.resolution_state.as_str()));
        input.insert(
            "negotiation_acceptance_boundary".to_string(),
            json!(&packet.acceptance_boundary.success_headline),
        );
    }

    if let Some(authored_brief) = authored_brief {
        input.insert("authored_brief".to_string(), json!(authored_brief));
        input.insert("authored_input_summary".to_string(), json!(authored_brief.summary_text()));
        input.insert(
            "authored_input_sources".to_string(),
            json!(authored_brief.ordered_source_labels()),
        );
        if !authored_brief.deduplicated_sources.is_empty() {
            input.insert(
                "authored_input_deduplicated_sources".to_string(),
                json!(authored_brief.deduplicated_source_labels()),
            );
        }
        input.insert(
            "authored_input_resolution_state".to_string(),
            json!(authored_brief.resolution_state),
        );
        if let Some(derived_task_draft) = authored_brief.derived_task_draft.as_ref() {
            input.insert("derived_task_draft".to_string(), json!(derived_task_draft));
            initial_context
                .insert(LATEST_DERIVED_TASK_DRAFT_KEY.to_string(), json!(derived_task_draft));
        }
        if let Some(clarification) = authored_brief.clarification.as_ref() {
            input.insert("clarification_record".to_string(), json!(clarification));
            input.insert("clarification_headline".to_string(), json!(clarification.headline()));
            input.insert("clarification_prompt".to_string(), json!(&clarification.prompt));
            if !clarification.missing_fields.is_empty() {
                input.insert(
                    "clarification_missing_fields".to_string(),
                    json!(&clarification.missing_fields),
                );
            }
            initial_context.insert(LATEST_CLARIFICATION_KEY.to_string(), json!(clarification));
        }
        if let Some(governance_intent) = authored_brief.governance_intent.as_ref() {
            input.insert("governance_intent".to_string(), json!(governance_intent));
            if let Some(runtime_preference) = governance_intent.runtime_preference {
                input.insert("requested_governance_runtime".to_string(), json!(runtime_preference));
            }
            if let Some(risk) = governance_intent.risk.as_ref() {
                input.insert("requested_governance_risk".to_string(), json!(risk));
            }
            if let Some(zone) = governance_intent.zone.as_ref() {
                input.insert("requested_governance_zone".to_string(), json!(zone));
            }
            if let Some(owner) = governance_intent.owner.as_ref() {
                input.insert("requested_governance_owner".to_string(), json!(owner));
            }
        }
    }

    Ok(TaskRunRequest {
        goal,
        input: Value::Object(input),
        session_id,
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: profile.limits,
        initial_context: (!initial_context.is_empty()).then_some(initial_context),
    })
}

fn workspace_routing_projection(workspace: &Path) -> Option<RoutingDecisionProjection> {
    let workspace_routing =
        FileConfigStore::for_workspace(workspace).local_routing().ok().flatten();
    let cluster_routing = FileClusterStore::for_workspace(workspace)
        .load()
        .ok()
        .flatten()
        .map(|config| config.routing);
    let global_routing = FileConfigStore::global_routing().ok().flatten();
    let effective = resolve_effective_routing(
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
    let projection = RoutingDecisionProjection::from_effective_state(
        &effective,
        &effective_capabilities,
        &effective_effort,
    );
    (!projection.is_empty()).then_some(projection)
}

pub fn build_fixture_runtime(workspace: &Path) -> Result<FixtureRuntime, FixtureRuntimeError> {
    build_fixture_runtime_for_flow(workspace, None)
}

pub fn build_fixture_runtime_for_flow(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
) -> Result<FixtureRuntime, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;
    build_fixture_runtime_from_profile(workspace, profile, active_flow)
}

pub fn build_fixture_runtime_for_goal_plan(
    workspace: &Path,
    goal_plan: &GoalPlan,
) -> Result<FixtureRuntime, FixtureRuntimeError> {
    let mut profile = synthesize_goal_plan_execution_profile(workspace, goal_plan)?;
    let routes = goal_plan_provider_routes(workspace);
    if provider_runtime::route_is_available(&routes.review) {
        profile.review = Some(synthesize_goal_plan_review_profile(&routes.review));
    }
    let mut runtime = build_fixture_runtime_from_profile(workspace, profile.clone(), None)?;
    let goal = goal_plan.goal_text.clone();

    if provider_runtime::route_is_available(&routes.planning) {
        runtime.agents.register("analyzer", {
            let workspace_ref = workspace.to_path_buf();
            let profile = profile.clone();
            let route = routes.planning.clone();
            let goal = goal.clone();
            FnAgentAdapter::new(move |request| {
                analyze_workspace_with_provider(&workspace_ref, &profile, &goal, &route, request)
            })
        })?;
    }

    if provider_runtime::route_is_available(&routes.implementation) {
        runtime.agents.register("coder", {
            let workspace_ref = workspace.to_path_buf();
            let profile = profile.clone();
            let route = routes.implementation.clone();
            let goal = goal.clone();
            FnAgentAdapter::new(move |request| {
                apply_workspace_with_provider(&workspace_ref, &profile, &goal, &route, request)
            })
        })?;
    }

    if provider_runtime::route_is_available(&routes.review) {
        runtime.agents.register("reviewer", {
            let workspace_ref = workspace.to_path_buf();
            let profile = profile.clone();
            let route = routes.review.clone();
            let goal = goal.clone();
            FnAgentAdapter::new(move |request| {
                review_workspace_with_provider(&workspace_ref, &profile, &goal, &route, request)
            })
        })?;
    }

    Ok(runtime)
}

#[derive(Debug, Clone)]
struct GoalPlanProviderRoutes {
    planning: ModelRoute,
    implementation: ModelRoute,
    review: ModelRoute,
}

fn goal_plan_provider_routes(workspace: &Path) -> GoalPlanProviderRoutes {
    let effective = workspace_effective_routing(workspace);

    GoalPlanProviderRoutes {
        planning: effective.planning.route,
        implementation: effective.implementation.route,
        review: effective.review.route,
    }
}

fn workspace_effective_routing(workspace: &Path) -> crate::domain::configuration::EffectiveRouting {
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

fn synthesize_goal_plan_review_profile(route: &ModelRoute) -> ReviewProfile {
    ReviewProfile {
        triggers: vec![ReviewTrigger::PrReady],
        reviewers: vec![ReviewerDefinition {
            reviewer_id: NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ID.to_string(),
            role: NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ROLE.to_string(),
            source: Some(provider_route_label(route)),
            weight: 1,
        }],
        vote_rule: VoteRuleDefinition::default(),
        adjudication: Default::default(),
        scenarios: vec![ReviewScenario {
            trigger: ReviewTrigger::PrReady,
            findings: vec![ReviewerFinding::new(
                NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ID.to_string(),
                ReviewerDisposition::Approve,
                NATIVE_GOAL_PLAN_PROVIDER_REVIEW_PLACEHOLDER_SUMMARY.to_string(),
            )],
            adjudication_finding: None,
        }],
    }
}

fn build_fixture_runtime_from_profile(
    workspace: &Path,
    profile: WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
) -> Result<FixtureRuntime, FixtureRuntimeError> {
    let workspace_ref = workspace.to_path_buf();
    let planner: Arc<dyn Planner> = if profile.adaptive.is_some() {
        Arc::new(CallbackPlanner::new(
            {
                let workspace_ref = workspace_ref.clone();
                let profile = profile.clone();
                let active_flow = active_flow.cloned();
                move |request, _context| {
                    build_adaptive_initial_plan(
                        &workspace_ref,
                        &profile,
                        active_flow.as_ref(),
                        &request.goal,
                    )
                    .map_err(|error| PlanningError::InvalidPlan(error.to_string()))
                }
            },
            {
                let workspace_ref = workspace_ref.clone();
                let profile = profile.clone();
                let active_flow = active_flow.cloned();
                move |task, failed_step, failure| {
                    build_adaptive_replan_steps(
                        &workspace_ref,
                        &profile,
                        active_flow.as_ref(),
                        task,
                        failed_step,
                        failure,
                    )
                }
            },
        ))
    } else {
        Arc::new(StaticPlanner::with_replans(
            build_vertical_slice_plan(&profile, active_flow, 0)?,
            build_replan_queue(&profile, active_flow)?,
        ))
    };

    let effective_routing = workspace_effective_routing(workspace);
    let provider_planning_route = explicit_provider_route(&effective_routing.planning.route);
    let provider_implementation_route =
        explicit_provider_route(&effective_routing.implementation.route);
    let provider_review_routes =
        provider_review_routes_for_profile_with_effective(&effective_routing, &profile);
    let mut agents = AgentRegistry::new();
    agents.register("analyzer", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        let provider_planning_route = provider_planning_route.clone();
        FnAgentAdapter::new(move |request| {
            analyze_workspace_with_optional_provider(
                &workspace_ref,
                &profile,
                provider_planning_route.as_ref(),
                request,
            )
        })
    })?;
    agents.register("coder", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        let provider_implementation_route = provider_implementation_route.clone();
        FnAgentAdapter::new(move |request| {
            apply_workspace_with_optional_provider(
                &workspace_ref,
                &profile,
                provider_implementation_route.as_ref(),
                request,
            )
        })
    })?;
    agents.register("reviewer", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        let provider_review_routes = provider_review_routes.clone();
        FnAgentAdapter::new(move |request| {
            review_workspace_with_optional_provider(
                &workspace_ref,
                &profile,
                &provider_review_routes,
                request,
            )
        })
    })?;

    let mut tools = ToolRegistry::new();
    tools.register("tester", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        FnToolAdapter::new(move |request| {
            verify_workspace_fixture(&workspace_ref, &profile, request)
        })
    })?;
    tools.register("replanner", {
        FnToolAdapter::new(move |_request| {
            StepExecutionResult::success(json!({
                "stdout": "replanned bounded flow",
            }))
        })
    })?;
    tools.register("review-voter", {
        let profile = profile.clone();
        FnToolAdapter::new(move |request| resolve_review_vote(&profile, request))
    })?;
    tools.register("review-finalizer", {
        let profile = profile.clone();
        FnToolAdapter::new(move |request| finalize_workspace_review(&profile, request))
    })?;

    Ok(FixtureRuntime { profile, planner, agents, tools })
}

fn synthesize_goal_plan_execution_profile(
    workspace: &Path,
    goal_plan: &GoalPlan,
) -> Result<WorkspaceExecutionProfile, FixtureRuntimeError> {
    let declared_profile = load_workspace_execution_profile(workspace).ok();
    let validation_command = declared_profile
        .as_ref()
        .map(|profile| profile.validation_command.clone())
        .unwrap_or_else(|| {
            if workspace.join("Cargo.toml").is_file() {
                ExecutionCommand {
                    program: "cargo".to_string(),
                    args: vec!["test".to_string(), "--quiet".to_string()],
                }
            } else {
                ExecutionCommand { program: "true".to_string(), args: Vec::new() }
            }
        });

    let (path, find, replace) = infer_goal_plan_change(workspace, goal_plan)?;
    let mut read_targets =
        declared_profile.map(|profile| profile.read_targets).unwrap_or_else(|| {
            goal_plan
                .tasks
                .iter()
                .filter_map(|task| {
                    let target = task.target.trim();
                    if target.is_empty() || target == "test suite" {
                        return None;
                    }
                    let candidate = workspace.join(target);
                    candidate.is_file().then(|| target.to_string())
                })
                .collect::<Vec<_>>()
        });
    if !read_targets.iter().any(|target| target == &path) {
        read_targets.push(path.clone());
    }
    if workspace.join("Cargo.toml").is_file()
        && !read_targets.iter().any(|target| target == "Cargo.toml")
    {
        read_targets.push("Cargo.toml".to_string());
    }

    Ok(WorkspaceExecutionProfile {
        name: format!("native-goal-plan-{}", goal_plan.plan_id),
        read_targets,
        validation_command,
        attempts: vec![ExecutionAttemptDefinition {
            attempt_id: "native-goal-attempt-1".to_string(),
            summary: format!("Synthetic native attempt for {}", goal_plan.goal_text),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange { path, find, replace }],
        }],
        adaptive: None,
        limits: RunLimits::default(),
        governance: None,
        review: None,
        legacy_source: Some(NATIVE_GOAL_PLAN_LEGACY_SOURCE.to_string()),
    })
}

fn infer_goal_plan_change(
    workspace: &Path,
    goal_plan: &GoalPlan,
) -> Result<(String, String, String), FixtureRuntimeError> {
    for target in goal_plan.tasks.iter().map(|task| task.target.trim()) {
        if target.is_empty() || target == "test suite" {
            continue;
        }
        let path = workspace.join(target);
        if !path.is_file() {
            continue;
        }

        let contents = fs::read_to_string(&path)
            .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
        if contents.contains("left - right") {
            return Ok((
                target.to_string(),
                "left - right".to_string(),
                "left + right".to_string(),
            ));
        }
        if contents.contains("\"todo\"") {
            return Ok((
                target.to_string(),
                "\"todo\"".to_string(),
                "\"workspace summary ready\"".to_string(),
            ));
        }
        if let Some(needle) = first_stable_line(&contents) {
            return Ok((
                target.to_string(),
                format!("__boundline_goal_plan_change_required__:{needle}"),
                format!("__boundline_goal_plan_change_applied__:{needle}"),
            ));
        }
    }

    let cargo_toml = workspace.join("Cargo.toml");
    if cargo_toml.is_file() {
        let contents = fs::read_to_string(&cargo_toml)
            .map_err(|source| FixtureRuntimeError::Io { path: cargo_toml.clone(), source })?;
        if let Some(needle) = first_stable_line(&contents) {
            return Ok((
                "Cargo.toml".to_string(),
                format!("__boundline_goal_plan_change_required__:{needle}"),
                format!("__boundline_goal_plan_change_applied__:{needle}"),
            ));
        }
    }

    Err(FixtureRuntimeError::NoSynthesizeableGoalPlanTarget {
        goal: goal_plan.goal_text.clone(),
        workspace: workspace.to_path_buf(),
    })
}

fn first_stable_line(contents: &str) -> Option<String> {
    contents.lines().map(str::trim).find(|line| !line.is_empty()).map(str::to_string)
}

fn resolve_supported_fixture_flow(
    flow_name: &str,
    context: &'static str,
) -> Result<&'static crate::domain::flow::FlowDefinition, FixtureRuntimeError> {
    // Reuse one unsupported-flow error path so direct and adaptive fixture
    // planning report the same contract shape.
    built_in_flow(flow_name).ok_or_else(|| FixtureRuntimeError::UnsupportedFixtureFlow {
        flow_name: flow_name.to_string(),
        context,
    })
}

fn build_vertical_slice_plan(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Plan, FixtureRuntimeError> {
    let Some(active_flow) = active_flow else {
        let mut steps = vec![Step::agent("analyze", "analyzer", analysis_step_input(profile))?];
        steps.extend(build_attempt_steps(profile, None, attempt_index)?);
        return Ok(Plan::new(steps)?);
    };

    let mut steps = match active_flow.flow_name.as_str() {
        "bug-fix" => {
            let flow = resolve_supported_fixture_flow(&active_flow.flow_name, "fixture planning")?;
            vec![
                Step::agent(
                    "investigate",
                    "analyzer",
                    attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
                )?,
                Step::agent(
                    "implement",
                    "coder",
                    attach_stage_metadata(
                        code_step_input(
                            profile,
                            attempt_index,
                            json!({
                                "phase": "implement",
                                "force_retry_once": profile.limits.max_retries > 0,
                            }),
                        )?,
                        flow,
                        1,
                    )?,
                )?,
                Step::tool(
                    "verify",
                    "tester",
                    attach_stage_metadata(
                        verify_step_input(
                            profile,
                            attempt_index,
                            json!({
                                "phase": "verify",
                            }),
                        )?,
                        flow,
                        2,
                    )?,
                )?,
            ]
        }
        "change" => {
            let flow = resolve_supported_fixture_flow(&active_flow.flow_name, "fixture planning")?;
            vec![
                Step::agent(
                    "understand-change",
                    "analyzer",
                    attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
                )?,
                Step::agent(
                    "implement",
                    "coder",
                    attach_stage_metadata(
                        code_step_input(profile, attempt_index, json!({"phase": "implement"}))?,
                        flow,
                        1,
                    )?,
                )?,
                Step::tool(
                    "verify",
                    "tester",
                    attach_stage_metadata(
                        verify_step_input(profile, attempt_index, json!({"phase": "verify"}))?,
                        flow,
                        2,
                    )?,
                )?,
            ]
        }
        "delivery" => {
            let flow = resolve_supported_fixture_flow(&active_flow.flow_name, "fixture planning")?;
            vec![
                Step::agent(
                    "requirements",
                    "analyzer",
                    attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
                )?,
                Step::decision(
                    "architecture",
                    attach_stage_metadata(
                        json!({
                            "phase": "architecture",
                            "output": {"architecture_ready": true},
                        }),
                        flow,
                        1,
                    )?,
                )?,
                Step::decision(
                    "backlog",
                    attach_stage_metadata(
                        json!({
                            "phase": "backlog",
                            "output": {"backlog_ready": true},
                        }),
                        flow,
                        2,
                    )?,
                )?,
                Step::agent(
                    "implementation-code",
                    "coder",
                    attach_stage_metadata(
                        code_step_input(
                            profile,
                            attempt_index,
                            json!({"phase": "implementation"}),
                        )?,
                        flow,
                        3,
                    )?,
                )?,
                Step::tool(
                    "implementation-verify",
                    "tester",
                    attach_stage_metadata(
                        verify_step_input(
                            profile,
                            attempt_index,
                            json!({"phase": "implementation"}),
                        )?,
                        flow,
                        3,
                    )?,
                )?,
            ]
        }
        other => {
            return Err(FixtureRuntimeError::UnsupportedFixtureFlow {
                flow_name: other.to_string(),
                context: "fixture planning",
            });
        }
    };

    steps.extend(build_review_steps(profile, Some(active_flow), attempt_index)?);

    Ok(Plan::new(steps)?)
}

fn build_replan_queue(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
) -> Result<Vec<Vec<Step>>, FixtureRuntimeError> {
    let mut replans = Vec::new();
    for attempt_index in 1..profile.attempts.len() {
        replans.push(build_attempt_steps(profile, active_flow, attempt_index)?);
    }
    Ok(replans)
}

fn build_adaptive_initial_plan(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    goal: &str,
) -> Result<Plan, FixtureRuntimeError> {
    let Some(candidate) = build_adaptive_candidates(
        workspace,
        profile,
        goal,
        AdaptiveCandidateContext {
            used_signatures: &BTreeSet::new(),
            previous_attempt_id: None,
            previous_selected_targets: None,
            validation_guidance: None,
            lineage_reason: "selected the initial adaptive candidate",
        },
    )?
    .into_iter()
    .next() else {
        return Err(FixtureRuntimeError::NoAdaptiveCandidate { profile: profile.name.clone() });
    };

    let Some(active_flow) = active_flow else {
        let mut steps = vec![Step::agent(
            "analyze",
            "analyzer",
            adaptive_analysis_step_input(profile, &candidate),
        )?];
        steps.extend(build_adaptive_attempt_steps(profile, None, &candidate)?);
        return Ok(Plan::new(steps)?);
    };

    let mut steps = match active_flow.flow_name.as_str() {
        "bug-fix" => {
            let flow = resolve_supported_fixture_flow(
                &active_flow.flow_name,
                "adaptive fixture planning",
            )?;
            vec![
                Step::agent(
                    "investigate",
                    "analyzer",
                    attach_stage_metadata(
                        adaptive_analysis_step_input(profile, &candidate),
                        flow,
                        0,
                    )?,
                )?,
                Step::agent(
                    "implement",
                    "coder",
                    attach_stage_metadata(
                        adaptive_code_step_input(
                            profile,
                            &candidate,
                            json!({
                                "phase": "implement",
                                "force_retry_once": profile.limits.max_retries > 0,
                            }),
                        ),
                        flow,
                        1,
                    )?,
                )?,
                Step::tool(
                    "verify",
                    "tester",
                    attach_stage_metadata(
                        adaptive_verify_step_input(profile, &candidate, json!({"phase": "verify"})),
                        flow,
                        2,
                    )?,
                )?,
            ]
        }
        "change" => {
            let flow = resolve_supported_fixture_flow(
                &active_flow.flow_name,
                "adaptive fixture planning",
            )?;
            vec![
                Step::agent(
                    "understand-change",
                    "analyzer",
                    attach_stage_metadata(
                        adaptive_analysis_step_input(profile, &candidate),
                        flow,
                        0,
                    )?,
                )?,
                Step::agent(
                    "implement",
                    "coder",
                    attach_stage_metadata(
                        adaptive_code_step_input(
                            profile,
                            &candidate,
                            json!({"phase": "implement"}),
                        ),
                        flow,
                        1,
                    )?,
                )?,
                Step::tool(
                    "verify",
                    "tester",
                    attach_stage_metadata(
                        adaptive_verify_step_input(profile, &candidate, json!({"phase": "verify"})),
                        flow,
                        2,
                    )?,
                )?,
            ]
        }
        "delivery" => {
            let flow = resolve_supported_fixture_flow(
                &active_flow.flow_name,
                "adaptive fixture planning",
            )?;
            vec![
                Step::agent(
                    "requirements",
                    "analyzer",
                    attach_stage_metadata(
                        adaptive_analysis_step_input(profile, &candidate),
                        flow,
                        0,
                    )?,
                )?,
                Step::decision(
                    "architecture",
                    attach_stage_metadata(
                        json!({
                            "phase": "architecture",
                            "output": {"architecture_ready": true},
                        }),
                        flow,
                        1,
                    )?,
                )?,
                Step::decision(
                    "backlog",
                    attach_stage_metadata(
                        json!({
                            "phase": "backlog",
                            "output": {"backlog_ready": true},
                        }),
                        flow,
                        2,
                    )?,
                )?,
                Step::agent(
                    "implementation-code",
                    "coder",
                    attach_stage_metadata(
                        adaptive_code_step_input(
                            profile,
                            &candidate,
                            json!({"phase": "implementation"}),
                        ),
                        flow,
                        3,
                    )?,
                )?,
                Step::tool(
                    "implementation-verify",
                    "tester",
                    attach_stage_metadata(
                        adaptive_verify_step_input(
                            profile,
                            &candidate,
                            json!({"phase": "implementation"}),
                        ),
                        flow,
                        3,
                    )?,
                )?,
            ]
        }
        other => {
            return Err(FixtureRuntimeError::UnsupportedFixtureFlow {
                flow_name: other.to_string(),
                context: "adaptive fixture planning",
            });
        }
    };

    steps.extend(build_review_steps_for_attempt(
        profile,
        Some(active_flow),
        candidate.attempt.attempt_id.as_str(),
    )?);

    Ok(Plan::new(steps)?)
}

fn build_adaptive_replan_steps(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    task: &crate::domain::task::Task,
    failed_step: &Step,
    failure: &StepExecutionResult,
) -> Result<Vec<Step>, PlanningError> {
    let used_signatures = adaptive_candidate_signatures_from_state(&task.context.state);
    let previous_attempt_id = latest_attempt_id_from_state(&task.context.state)
        .or_else(|| failed_step.input.get("attempt_id").and_then(Value::as_str))
        .map(str::to_string);
    let fallback_reason =
        failure.error.as_ref().map(|error| error.message.clone()).unwrap_or_else(|| {
            "adaptive validation failed and a new candidate is required".to_string()
        });
    let previous_selected_targets =
        latest_workspace_slice_from_state(&task.context.state).map(|slice| slice.selected_targets);
    let validation_guidance = build_validation_guidance(
        &task.context.state,
        &profile.read_targets,
        failure,
        &fallback_reason,
    );
    if let Some(reason) = adaptive_replan_blocker(validation_guidance.as_ref()) {
        return Err(PlanningError::ReplanUnavailable(reason));
    }
    let lineage_reason = validation_guidance
        .as_ref()
        .map(|guidance| guidance.headline.clone())
        .unwrap_or_else(|| fallback_reason.clone());

    let Some(candidate) = build_adaptive_candidates(
        workspace,
        profile,
        &task.goal,
        AdaptiveCandidateContext {
            used_signatures: &used_signatures,
            previous_attempt_id: previous_attempt_id.as_deref(),
            previous_selected_targets: previous_selected_targets.as_deref(),
            validation_guidance: validation_guidance.as_ref(),
            lineage_reason: &lineage_reason,
        },
    )
    .map_err(|error| PlanningError::Internal(error.to_string()))?
    .into_iter()
    .next() else {
        return Err(PlanningError::ReplanUnavailable(adaptive_no_candidate_reason(
            validation_guidance.as_ref(),
        )));
    };

    build_adaptive_attempt_steps(profile, active_flow, &candidate)
        .map_err(|error| PlanningError::InvalidPlan(error.to_string()))
}

fn build_adaptive_candidates(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    goal: &str,
    context: AdaptiveCandidateContext<'_>,
) -> Result<Vec<AdaptiveAttemptPlan>, FixtureRuntimeError> {
    let adaptive = profile.adaptive.as_ref().ok_or_else(|| {
        FixtureRuntimeError::MissingAdaptiveProfile { profile: profile.name.clone() }
    })?;
    let goal_hint = adaptive_goal_hint(goal, profile);
    let goal_terms = tokenize_terms(&goal_hint);
    let validation_terms = merge_terms(
        tokenize_terms(&profile.validation_command.rendered()),
        context
            .validation_guidance
            .map(|guidance| guidance.matched_terms.clone())
            .unwrap_or_default(),
    );
    let sources = load_workspace_target_sources(workspace, &profile.read_targets)?;
    let mut path_scores = sources
        .iter()
        .map(|source| {
            score_workspace_target(
                source,
                adaptive,
                &goal_terms,
                &validation_terms,
                context.validation_guidance,
            )
        })
        .collect::<Vec<_>>();
    path_scores.sort_by(|left, right| {
        right.score.cmp(&left.score).then_with(|| left.path.cmp(&right.path))
    });

    let mut seen_signatures = BTreeSet::new();
    let mut selected_sources = Vec::new();
    for scored_target in &path_scores {
        if selected_sources.len() >= adaptive.max_selected_targets {
            break;
        }

        let Some(source) = sources.iter().find(|source| source.path == scored_target.path) else {
            continue;
        };

        if adaptive_changes_for_target(&source.path, &source.contents, adaptive).is_empty() {
            continue;
        }

        selected_sources.push((source, scored_target));
    }

    let selected_targets =
        selected_sources.iter().map(|(source, _)| source.path.clone()).collect::<Vec<_>>();
    let mut ranked_candidates = Vec::new();
    let mut order_index = 0_usize;
    for (source, scored_target) in selected_sources {
        for generated in adaptive_changes_for_target(&source.path, &source.contents, adaptive) {
            let signature = workspace_change_signature(&generated.change);
            if !seen_signatures.insert(signature.clone()) {
                continue;
            }

            if context.used_signatures.contains(&signature) {
                continue;
            }

            let (score, reasons) = score_adaptive_candidate(
                scored_target,
                &generated,
                &goal_terms,
                &validation_terms,
                context.validation_guidance,
            );
            ranked_candidates.push(RankedAdaptiveCandidate {
                change_kind: generated.change_kind,
                change: generated.change,
                signature,
                score,
                order_index,
                reasons,
            });
            order_index += 1;
        }
    }

    ranked_candidates.sort_by(|left, right| {
        right.score.cmp(&left.score).then_with(|| left.order_index.cmp(&right.order_index))
    });

    if context.previous_attempt_id.is_some() {
        ranked_candidates.retain(|candidate| candidate.score >= MIN_ADAPTIVE_REPLAN_SCORE);
    }

    if ranked_candidates.len() > adaptive.max_generated_attempts {
        ranked_candidates.truncate(adaptive.max_generated_attempts);
    }

    let available_count = ranked_candidates.len();

    Ok(ranked_candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let attempt_id =
                format!("adaptive-attempt-{}", context.used_signatures.len() + index + 1);
            let workspace_slice = WorkspaceSliceSelection {
                selection_id: format!("adaptive-slice-{attempt_id}"),
                selected_targets: selected_targets.clone(),
                scored_candidates: path_scores.clone(),
                headline: adaptive_selection_headline(
                    &candidate.change.path,
                    candidate.change_kind,
                    context.validation_guidance,
                ),
            };
            let selection_evidence = SelectionEvidence {
                goal_terms: goal_terms.clone(),
                validation_terms: validation_terms.clone(),
                validation_guidance: context.validation_guidance.cloned(),
                path_scores: path_scores.clone(),
                candidate_family: Some(candidate.change_kind),
                rejected_candidates: build_rejected_candidate_summaries(
                    index,
                    &selected_targets,
                    &workspace_slice.scored_candidates,
                    available_count,
                    &candidate,
                ),
                reason: adaptive_selection_reason(
                    &candidate.change.path,
                    candidate.change_kind,
                    selected_targets.len(),
                    context.validation_guidance,
                    &candidate.reasons,
                ),
            };
            let attempt = ExecutionAttemptDefinition {
                attempt_id: attempt_id.clone(),
                summary: format!(
                    "Adaptively apply {} in {} by replacing '{}' with '{}'",
                    candidate.change_kind.as_str(),
                    candidate.change.path,
                    excerpt(&candidate.change.find),
                    excerpt(&candidate.change.replace)
                ),
                failure_mode: if index + 1 < available_count {
                    ExecutionFailureMode::Replan
                } else {
                    ExecutionFailureMode::Terminal
                },
                changes: vec![candidate.change.clone()],
            };
            let attempt_lineage = AttemptLineage {
                previous_attempt_id: context.previous_attempt_id.map(str::to_string),
                current_attempt_id: attempt_id,
                transition_kind: adaptive_transition_kind(
                    context.previous_selected_targets,
                    &selected_targets,
                ),
                reason: context.lineage_reason.to_string(),
            };

            AdaptiveAttemptPlan {
                attempt,
                workspace_slice,
                selection_evidence,
                candidate_signature: candidate.signature,
                attempt_lineage,
            }
        })
        .collect())
}

fn build_adaptive_attempt_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    candidate: &AdaptiveAttemptPlan,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    if let Some(active_flow) = active_flow {
        let code_id = format!(
            "{}-replan-{}-code",
            active_flow.current_stage_id, candidate.attempt.attempt_id
        );
        let verify_id = format!(
            "{}-replan-{}-verify",
            active_flow.current_stage_id, candidate.attempt.attempt_id
        );

        return Ok(vec![
            Step::agent(
                code_id,
                "coder",
                attach_current_stage_metadata(
                    adaptive_code_step_input(
                        profile,
                        candidate,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    ),
                    active_flow,
                ),
            )?,
            Step::tool(
                verify_id,
                "tester",
                attach_current_stage_metadata(
                    adaptive_verify_step_input(
                        profile,
                        candidate,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    ),
                    active_flow,
                ),
            )?,
        ]
        .into_iter()
        .chain(build_review_steps_for_attempt(
            profile,
            Some(active_flow),
            candidate.attempt.attempt_id.as_str(),
        )?)
        .collect());
    }

    Ok(vec![
        Step::agent(
            format!("code-{}", candidate.attempt.attempt_id),
            "coder",
            adaptive_code_step_input(profile, candidate, json!({"phase": "code"})),
        )?,
        Step::tool(
            format!("verify-{}", candidate.attempt.attempt_id),
            "tester",
            adaptive_verify_step_input(profile, candidate, json!({"phase": "verify"})),
        )?,
    ]
    .into_iter()
    .chain(build_review_steps_for_attempt(profile, None, candidate.attempt.attempt_id.as_str())?)
    .collect())
}

fn load_workspace_target_sources(
    workspace: &Path,
    targets: &[String],
) -> Result<Vec<WorkspaceTargetSource>, FixtureRuntimeError> {
    targets
        .iter()
        .map(|target| {
            let path = workspace.join(target);
            let contents = fs::read_to_string(&path)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            Ok(WorkspaceTargetSource { path: target.clone(), contents })
        })
        .collect()
}

fn score_workspace_target(
    source: &WorkspaceTargetSource,
    adaptive: &crate::domain::execution::AdaptiveExecutionProfile,
    goal_terms: &[String],
    validation_terms: &[String],
    validation_guidance: Option<&ValidationGuidance>,
) -> PathScore {
    let mut score = 0_i64;
    let mut reasons = Vec::new();
    let lower_path = source.path.to_ascii_lowercase();
    let lower_contents = source.contents.to_ascii_lowercase();

    for preference in &adaptive.path_preferences {
        if lower_path.starts_with(&preference.to_ascii_lowercase()) {
            score += 50;
            reasons.push(format!("matched path preference {}", preference));
        }
    }

    if lower_path.starts_with("src/") {
        score += 15;
        reasons.push("prioritized source file".to_string());
    }

    if lower_path.starts_with("tests/") {
        score += 5;
        reasons.push("test target remains available for evidence".to_string());
    }

    for term in goal_terms {
        if lower_path.contains(term) {
            score += 20;
            reasons.push(format!("goal term '{}' matched path", term));
        } else if lower_contents.contains(term) {
            score += 5;
            reasons.push(format!("goal term '{}' matched contents", term));
        }
    }

    for term in validation_terms {
        if lower_path.contains(term) || lower_contents.contains(term) {
            score += 3;
        }
    }

    if let Some(validation_guidance) = validation_guidance {
        for matched_path in &validation_guidance.matched_paths {
            let lower_matched_path = matched_path.to_ascii_lowercase();
            if lower_path == lower_matched_path {
                score += 200;
                reasons.push(format!("validation guidance pointed to {}", matched_path));
            } else if lower_path.ends_with(&lower_matched_path) {
                score += 80;
                reasons.push(format!("validation guidance aligned with {}", matched_path));
            }
        }

        for term in &validation_guidance.matched_terms {
            if lower_path.contains(term) {
                score += 25;
                reasons.push(format!("validation term '{}' matched path", term));
            } else if lower_contents.contains(term) {
                score += 8;
                reasons.push(format!("validation term '{}' matched contents", term));
            }
        }
    }

    let candidate_count =
        adaptive_changes_for_target(&source.path, &source.contents, adaptive).len();
    if candidate_count > 0 {
        score += 10;
        reasons.push(format!("supports {candidate_count} adaptive candidate(s)"));
    }

    PathScore { path: source.path.clone(), score, reasons }
}

fn adaptive_changes_for_target(
    path: &str,
    contents: &str,
    adaptive: &crate::domain::execution::AdaptiveExecutionProfile,
) -> Vec<GeneratedAdaptiveCandidate> {
    let mut changes = Vec::new();
    for kind in adaptive.effective_change_kinds() {
        let generated = match kind {
            AdaptiveChangeKind::ArithmeticSwap => arithmetic_swap_candidates(path, contents),
            AdaptiveChangeKind::ComparisonFlip => comparison_flip_candidates(path, contents),
            AdaptiveChangeKind::BooleanFlip => boolean_flip_candidates(path, contents),
            AdaptiveChangeKind::OrderingBoundaryFlip => {
                ordering_boundary_flip_candidates(path, contents)
            }
            AdaptiveChangeKind::ResultStatusFlip => result_status_flip_candidates(path, contents),
            AdaptiveChangeKind::NumericLiteralFlip => {
                numeric_literal_flip_candidates(path, contents)
            }
        };
        changes.extend(
            generated
                .into_iter()
                .map(|change| GeneratedAdaptiveCandidate { change_kind: kind, change }),
        );
    }
    changes
}

fn score_adaptive_candidate(
    path_score: &PathScore,
    candidate: &GeneratedAdaptiveCandidate,
    goal_terms: &[String],
    validation_terms: &[String],
    validation_guidance: Option<&ValidationGuidance>,
) -> (i64, Vec<String>) {
    let mut score = path_score.score;
    let mut reasons = Vec::new();

    if let Some(reason) = path_score.reasons.first() {
        reasons.push(reason.clone());
    }

    score += 10;
    reasons.push(format!(
        "{} remained available as a bounded local repair",
        candidate.change_kind.as_str()
    ));

    if goal_terms.iter().any(|term| {
        candidate.change.path.to_ascii_lowercase().contains(term)
            || candidate.change.find.to_ascii_lowercase().contains(term)
            || candidate.change.replace.to_ascii_lowercase().contains(term)
    }) {
        score += 10;
        reasons.push("goal terms aligned with the candidate change".to_string());
    }

    match candidate.change_kind {
        AdaptiveChangeKind::ArithmeticSwap => {
            if terms_include_any(
                validation_terms,
                &["arith", "math", "add", "sum", "multiply", "divide"],
            ) {
                score += 30;
                reasons.push("validation evidence suggested an arithmetic mismatch".to_string());
            }
        }
        AdaptiveChangeKind::ComparisonFlip => {
            if terms_include_any(validation_terms, &["equal", "match", "compar", "assert"]) {
                score += 30;
                reasons.push("validation evidence suggested an equality mismatch".to_string());
            }
        }
        AdaptiveChangeKind::BooleanFlip => {
            if terms_include_any(validation_terms, &["true", "false", "bool"]) {
                score += 30;
                reasons.push("validation evidence suggested a boolean mismatch".to_string());
            }
        }
        AdaptiveChangeKind::OrderingBoundaryFlip => {
            score += 8;
            if terms_include_any(
                validation_terms,
                &["bound", "range", "threshold", "minimum", "maximum", "greater", "less"],
            ) {
                score += 35;
                reasons.push("validation evidence suggested a boundary mismatch".to_string());
            }
        }
        AdaptiveChangeKind::ResultStatusFlip => {
            score += 8;
            if terms_include_any(
                validation_terms,
                &["error", "result", "failed", "success", "panic"],
            ) {
                score += 35;
                reasons
                    .push("validation evidence suggested an outcome-status mismatch".to_string());
            }
        }
        AdaptiveChangeKind::NumericLiteralFlip => {
            score += 5;
            if terms_include_any(
                validation_terms,
                &["zero", "one", "count", "length", "literal", "constant"],
            ) {
                score += 30;
                reasons
                    .push("validation evidence suggested a numeric literal mismatch".to_string());
            }
            if candidate.change.find.contains('0') || candidate.change.find.contains('1') {
                score += 8;
                reasons.push("candidate repairs a bounded numeric literal".to_string());
            }
        }
    }

    if let Some(validation_guidance) = validation_guidance {
        match validation_guidance.confidence {
            ValidationGuidanceConfidence::Strong => {
                score += 25;
                reasons.push("strong validation guidance supported the bounded replan".to_string());
            }
            ValidationGuidanceConfidence::Hinted => {
                score += 10;
                reasons.push("validation hints supported the bounded replan".to_string());
            }
        }
    }

    (score, reasons)
}

fn terms_include_any(terms: &[String], keywords: &[&str]) -> bool {
    terms.iter().any(|term| keywords.iter().any(|keyword| term.contains(keyword)))
}

fn arithmetic_swap_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    let patterns = [
        (" - ", [" + ", " / ", " * "]),
        (" * ", [" - ", " + ", " / "]),
        (" / ", [" * ", " + ", " - "]),
        (" + ", [" - ", " * ", " / "]),
    ];

    for (find, replacements) in patterns {
        if contents.contains(find) {
            return replacements
                .into_iter()
                .map(|replace| WorkspaceChange {
                    path: path.to_string(),
                    find: find.to_string(),
                    replace: replace.to_string(),
                })
                .collect();
        }
    }

    Vec::new()
}

fn comparison_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    if contents.contains(" != ") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: " != ".to_string(),
            replace: " == ".to_string(),
        }];
    }

    if contents.contains(" == ") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: " == ".to_string(),
            replace: " != ".to_string(),
        }];
    }

    Vec::new()
}

fn boolean_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    if contents.contains("false") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "false".to_string(),
            replace: "true".to_string(),
        }];
    }

    if contents.contains("true") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "true".to_string(),
            replace: "false".to_string(),
        }];
    }

    Vec::new()
}

fn ordering_boundary_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    let patterns = [(" >= ", " > "), (" <= ", " < "), (" > ", " >= "), (" < ", " <= ")];

    for (find, replace) in patterns {
        if contents.contains(find) {
            return vec![WorkspaceChange {
                path: path.to_string(),
                find: find.to_string(),
                replace: replace.to_string(),
            }];
        }
    }

    Vec::new()
}

fn result_status_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    if contents.contains("Err(") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "Err(".to_string(),
            replace: "Ok(".to_string(),
        }];
    }

    if contents.contains("Ok(") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "Ok(".to_string(),
            replace: "Err(".to_string(),
        }];
    }

    Vec::new()
}

fn numeric_literal_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    let patterns = [
        (" == 0", " == 1"),
        (" == 1", " == 0"),
        (" != 0", " != 1"),
        (" != 1", " != 0"),
        ("(0)", "(1)"),
        ("(1)", "(0)"),
        (" = 0;", " = 1;"),
        (" = 1;", " = 0;"),
        (" 0;", " 1;"),
        (" 1;", " 0;"),
        ("return 0;", "return 1;"),
        ("return 1;", "return 0;"),
    ];

    for (find, replace) in patterns {
        if contents.contains(find) {
            return vec![WorkspaceChange {
                path: path.to_string(),
                find: find.to_string(),
                replace: replace.to_string(),
            }];
        }
    }

    Vec::new()
}

fn tokenize_terms(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .filter_map(|segment| {
            let term = segment.trim().to_ascii_lowercase();
            if term.len() >= 3 { Some(term) } else { None }
        })
        .collect()
}

fn merge_terms(mut left: Vec<String>, right: Vec<String>) -> Vec<String> {
    let mut seen = left.iter().cloned().collect::<BTreeSet<_>>();
    for term in right {
        if seen.insert(term.clone()) {
            left.push(term);
        }
    }
    left
}

fn latest_workspace_slice_from_state(
    state: &Map<String, Value>,
) -> Option<WorkspaceSliceSelection> {
    state
        .get("latest_workspace_slice")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

fn build_validation_guidance(
    state: &Map<String, Value>,
    read_targets: &[String],
    failure: &StepExecutionResult,
    fallback_reason: &str,
) -> Option<ValidationGuidance> {
    let validation_record = state
        .get("latest_validation_record")
        .cloned()
        .and_then(|value| serde_json::from_value::<ValidationRecord>(value).ok());
    let mut evidence_segments = Vec::new();

    if let Some(record) = &validation_record {
        if !record.stderr.trim().is_empty() {
            evidence_segments.push(record.stderr.clone());
        }
        if !record.stdout.trim().is_empty() {
            evidence_segments.push(record.stdout.clone());
        }
    }

    if let Some(error) = &failure.error {
        evidence_segments.push(error.message.clone());
        if let Some(details) = &error.details {
            collect_text_segments(details, &mut evidence_segments);
        }
    }

    if evidence_segments.is_empty() {
        return None;
    }

    let combined_evidence = evidence_segments.join("\n");
    let matched_paths = guidance_paths_from_text(&combined_evidence, read_targets);
    let mut matched_terms = tokenize_terms(&combined_evidence);
    matched_terms.truncate(16);
    let source = if validation_record.is_some() {
        ValidationGuidanceSource::ValidationRecord
    } else {
        ValidationGuidanceSource::FailureMessage
    };
    let confidence = if matched_paths.is_empty() {
        ValidationGuidanceConfidence::Hinted
    } else {
        ValidationGuidanceConfidence::Strong
    };
    let headline = if matched_paths.is_empty() {
        fallback_reason.to_string()
    } else {
        format!("validation guided the next attempt toward {}", matched_paths.join(", "))
    };

    Some(ValidationGuidance { source, matched_paths, matched_terms, headline, confidence })
}

fn adaptive_replan_blocker(validation_guidance: Option<&ValidationGuidance>) -> Option<String> {
    match validation_guidance {
        None => Some(
            "adaptive planner exhausted bounded repair because validation evidence was absent"
                .to_string(),
        ),
        Some(guidance) if guidance.matched_paths.is_empty() && guidance.matched_terms.len() < 2 => {
            Some(
                "adaptive planner exhausted bounded repair because validation evidence was insufficient to justify another materially different candidate"
                    .to_string(),
            )
        }
        _ => None,
    }
}

fn adaptive_no_candidate_reason(validation_guidance: Option<&ValidationGuidance>) -> String {
    if let Some(guidance) = validation_guidance
        && !guidance.matched_paths.is_empty()
    {
        return format!(
            "adaptive planner exhausted bounded repair because no remaining candidate stayed credible after validation pointed to {}",
            guidance.matched_paths.join(", ")
        );
    }

    "adaptive planner exhausted bounded repair because no remaining candidate stayed credible"
        .to_string()
}

fn collect_text_segments(value: &Value, segments: &mut Vec<String>) {
    match value {
        Value::String(text) if !text.trim().is_empty() => segments.push(text.clone()),
        Value::String(_) => {}
        Value::Array(items) => {
            for item in items {
                collect_text_segments(item, segments);
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                collect_text_segments(value, segments);
            }
        }
        _ => {}
    }
}

fn guidance_paths_from_text(text: &str, read_targets: &[String]) -> Vec<String> {
    let lower_text = text.to_ascii_lowercase();
    let mut matches = read_targets
        .iter()
        .filter(|target| lower_text.contains(&target.to_ascii_lowercase()))
        .cloned()
        .collect::<Vec<_>>();

    if matches.is_empty() {
        for target in read_targets {
            let Some(file_name) = target.rsplit('/').next() else {
                continue;
            };
            if lower_text.contains(&file_name.to_ascii_lowercase()) {
                matches.push(target.clone());
            }
        }
    }

    if matches.iter().any(|target| target.starts_with("src/")) {
        matches.retain(|target| target.starts_with("src/"));
    }

    matches.sort();
    matches.dedup();
    matches
}

fn adaptive_selection_headline(
    path: &str,
    change_kind: AdaptiveChangeKind,
    validation_guidance: Option<&ValidationGuidance>,
) -> String {
    if let Some(validation_guidance) = validation_guidance
        && !validation_guidance.matched_paths.is_empty()
    {
        return format!(
            "selected {path} via {} for adaptive delivery after validation guidance",
            change_kind.as_str()
        );
    }

    format!("selected {path} via {} for adaptive delivery", change_kind.as_str())
}

fn adaptive_selection_reason(
    path: &str,
    change_kind: AdaptiveChangeKind,
    selected_target_count: usize,
    validation_guidance: Option<&ValidationGuidance>,
    candidate_reasons: &[String],
) -> String {
    let credibility_reason = candidate_reasons
        .first()
        .cloned()
        .unwrap_or_else(|| "it remained the most credible bounded candidate".to_string());

    if let Some(validation_guidance) = validation_guidance {
        if !validation_guidance.matched_paths.is_empty() {
            return format!(
                "selected {path} via {} from {selected_target_count} scored read target(s) after validation pointed to {} because {}",
                change_kind.as_str(),
                validation_guidance.matched_paths.join(", "),
                credibility_reason
            );
        }

        return format!(
            "selected {path} via {} from {selected_target_count} scored read target(s) after validation evidence reprioritized the bounded slice because {}",
            change_kind.as_str(),
            credibility_reason
        );
    }

    format!(
        "selected {path} via {} from {selected_target_count} scored read target(s) because {}",
        change_kind.as_str(),
        credibility_reason
    )
}

fn build_rejected_candidate_summaries(
    selected_index: usize,
    _selected_targets: &[String],
    _path_scores: &[PathScore],
    available_count: usize,
    selected_candidate: &RankedAdaptiveCandidate,
) -> Vec<String> {
    if available_count <= 1 {
        return Vec::new();
    }

    vec![format!(
        "later bounded candidates were rejected because {} on {} remained more credible",
        selected_candidate.change_kind.as_str(),
        selected_candidate.change.path
    )]
    .into_iter()
    .take(if selected_index == 0 { 1 } else { 0 })
    .collect()
}

fn adaptive_transition_kind(
    previous_selected_targets: Option<&[String]>,
    selected_targets: &[String],
) -> AttemptTransitionKind {
    let Some(previous_selected_targets) = previous_selected_targets else {
        return AttemptTransitionKind::Initial;
    };

    let previous = previous_selected_targets.iter().collect::<BTreeSet<_>>();
    let current = selected_targets.iter().collect::<BTreeSet<_>>();

    if previous == current {
        return AttemptTransitionKind::Replaced;
    }

    if current.is_subset(&previous) && current.len() < previous.len() {
        return AttemptTransitionKind::Narrowed;
    }

    if previous.is_subset(&current) && previous.len() < current.len() {
        return AttemptTransitionKind::Broadened;
    }

    AttemptTransitionKind::Replaced
}

fn adaptive_goal_hint(goal: &str, profile: &WorkspaceExecutionProfile) -> String {
    let trimmed = goal.trim();
    if trimmed.is_empty() { profile.name.clone() } else { trimmed.to_string() }
}

fn workspace_change_signature(change: &WorkspaceChange) -> String {
    format!("{}::{}=>{}", change.path, change.find, change.replace)
}

fn adaptive_candidate_signatures_from_state(state: &Map<String, Value>) -> BTreeSet<String> {
    state
        .get("adaptive_candidate_signatures")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn latest_attempt_id_from_state(state: &Map<String, Value>) -> Option<&str> {
    state.get("latest_attempt_id").and_then(Value::as_str)
}

fn adaptive_analysis_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
) -> Value {
    json!({
        "phase": "analyze",
        "execution_profile": profile.name,
        "read_targets": read_targets_for_profile(profile),
        "legacy_source": profile.legacy_source,
        "workspace_slice": candidate.workspace_slice,
        "selection_headline": &candidate.workspace_slice.headline,
        "selection_evidence": candidate.selection_evidence,
    })
}

fn adaptive_code_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
    extra: Value,
) -> Value {
    let mut input = extra.as_object().cloned().unwrap_or_default();
    input.insert("execution_profile".to_string(), json!(profile.name));
    input.insert("attempt_id".to_string(), json!(&candidate.attempt.attempt_id));
    input.insert("failure_mode".to_string(), json!(candidate.attempt.failure_mode));
    input.insert(
        "adaptive_attempt".to_string(),
        serde_json::to_value(&candidate.attempt).unwrap_or(Value::Null),
    );
    input.insert(
        "workspace_slice".to_string(),
        serde_json::to_value(&candidate.workspace_slice).unwrap_or(Value::Null),
    );
    input.insert(
        "selection_evidence".to_string(),
        serde_json::to_value(&candidate.selection_evidence).unwrap_or(Value::Null),
    );
    input.insert("selection_headline".to_string(), json!(&candidate.workspace_slice.headline));
    input.insert("candidate_signature".to_string(), json!(&candidate.candidate_signature));
    input.insert(
        "attempt_lineage".to_string(),
        serde_json::to_value(&candidate.attempt_lineage).unwrap_or(Value::Null),
    );
    Value::Object(input)
}

fn adaptive_verify_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
    extra: Value,
) -> Value {
    adaptive_code_step_input(profile, candidate, extra)
}

fn insert_adaptive_state_from_input(
    state_patch: &mut Map<String, Value>,
    input: &Value,
    existing_state: &Map<String, Value>,
) {
    if let Some(workspace_slice) = input.get("workspace_slice") {
        state_patch.insert("latest_workspace_slice".to_string(), workspace_slice.clone());
    }

    if let Some(selection_headline) = input.get("selection_headline") {
        state_patch.insert("latest_selection_headline".to_string(), selection_headline.clone());
    }

    if let Some(selection_evidence) = input.get("selection_evidence") {
        state_patch.insert("latest_selection_evidence".to_string(), selection_evidence.clone());
        if let Some(reason) = selection_evidence.get("reason") {
            state_patch.insert("latest_selection_reason".to_string(), reason.clone());
        }
        if let Some(candidate_family) = selection_evidence.get("candidate_family") {
            state_patch.insert("latest_candidate_family".to_string(), candidate_family.clone());
        }
        if let Some(rejected_candidates) = selection_evidence.get("rejected_candidates") {
            state_patch
                .insert("latest_rejected_candidates".to_string(), rejected_candidates.clone());
        }
    }

    if let Some(attempt_lineage) = input.get("attempt_lineage") {
        state_patch.insert("latest_attempt_lineage".to_string(), attempt_lineage.clone());
    }

    if let Some(candidate_signature) = input.get("candidate_signature").and_then(Value::as_str) {
        let mut signatures = adaptive_candidate_signatures_from_state(existing_state);
        signatures.insert(candidate_signature.to_string());
        state_patch.insert("latest_candidate_signature".to_string(), json!(candidate_signature));
        state_patch.insert(
            "adaptive_candidate_signatures".to_string(),
            json!(signatures.into_iter().collect::<Vec<_>>()),
        );
    }
}

fn insert_adaptive_output_from_input(rendered_output: &mut Value, input: &Value) {
    if let Some(workspace_slice) = input.get("workspace_slice") {
        rendered_output["workspace_slice"] = workspace_slice.clone();
    }

    if let Some(selection_evidence) = input.get("selection_evidence") {
        rendered_output["selection_evidence"] = selection_evidence.clone();
    }

    if let Some(selection_headline) = input.get("selection_headline") {
        rendered_output["selection_headline"] = selection_headline.clone();
    }

    if let Some(attempt_lineage) = input.get("attempt_lineage") {
        rendered_output["attempt_lineage"] = attempt_lineage.clone();
    }
}

fn build_attempt_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;

    if let Some(active_flow) = active_flow {
        let code_id =
            format!("{}-replan-{}-code", active_flow.current_stage_id, attempt.attempt_id);
        let verify_id =
            format!("{}-replan-{}-verify", active_flow.current_stage_id, attempt.attempt_id);

        return Ok(vec![
            Step::agent(
                code_id,
                "coder",
                attach_current_stage_metadata(
                    code_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    )?,
                    active_flow,
                ),
            )?,
            Step::tool(
                verify_id,
                "tester",
                attach_current_stage_metadata(
                    verify_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    )?,
                    active_flow,
                ),
            )?,
        ]
        .into_iter()
        .chain(build_review_steps(profile, Some(active_flow), attempt_index)?)
        .collect());
    }

    Ok(vec![
        Step::agent(
            format!("code-{}", attempt.attempt_id),
            "coder",
            code_step_input(profile, attempt_index, json!({"phase": "code"}))?,
        )?,
        Step::tool(
            format!("verify-{}", attempt.attempt_id),
            "tester",
            verify_step_input(profile, attempt_index, json!({"phase": "verify"}))?,
        )?,
    ]
    .into_iter()
    .chain(build_review_steps(profile, None, attempt_index)?)
    .collect())
}

fn analysis_step_input(profile: &WorkspaceExecutionProfile) -> Value {
    json!({
        "phase": "analyze",
        "execution_profile": profile.name,
        "read_targets": read_targets_for_profile(profile),
        "legacy_source": profile.legacy_source,
    })
}

fn code_step_input(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
    extra: Value,
) -> Result<Value, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;
    let mut input = extra.as_object().cloned().unwrap_or_default();
    input.insert("execution_profile".to_string(), json!(profile.name));
    input.insert("attempt_index".to_string(), json!(attempt_index));
    input.insert("attempt_id".to_string(), json!(attempt.attempt_id));
    input.insert("failure_mode".to_string(), json!(attempt.failure_mode));
    Ok(Value::Object(input))
}

fn verify_step_input(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
    extra: Value,
) -> Result<Value, FixtureRuntimeError> {
    code_step_input(profile, attempt_index, extra)
}

fn build_review_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;
    build_review_steps_for_attempt(profile, active_flow, &attempt.attempt_id)
}

fn build_review_steps_for_attempt(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_id: &str,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let Some(review) = profile.review.as_ref() else {
        return Ok(Vec::new());
    };

    let prefix = active_flow
        .map(|flow| format!("{}-review-{}", flow.current_stage_id, attempt_id))
        .unwrap_or_else(|| format!("review-{}", attempt_id));

    let mut steps = Vec::new();
    for reviewer in &review.reviewers {
        steps.push(review_agent_step(
            format!("{}-{}", prefix, reviewer.reviewer_id),
            review_step_input_for_attempt(profile, attempt_id, reviewer.reviewer_id.clone(), false),
            active_flow,
        )?);
    }

    steps.push(review_tool_step(
        format!("{}-vote", prefix),
        review_vote_step_input_for_attempt(profile, attempt_id),
        "review-voter",
        active_flow,
    )?);

    if review.adjudication.enabled {
        let Some(adjudicator_id) = review.adjudication.reviewer_id.as_ref().cloned() else {
            return Err(FixtureRuntimeError::MissingReviewAdjudicator {
                profile: profile.name.clone(),
            });
        };
        steps.push(review_agent_step(
            format!("{}-adjudicate", prefix),
            review_step_input_for_attempt(profile, attempt_id, adjudicator_id, true),
            active_flow,
        )?);
    }

    steps.push(review_tool_step(
        format!("{}-finalize", prefix),
        review_finalize_step_input_for_attempt(profile, attempt_id),
        "review-finalizer",
        active_flow,
    )?);

    Ok(steps)
}

fn review_agent_step(
    id: String,
    input: Value,
    active_flow: Option<&SessionFlowState>,
) -> Result<Step, StepError> {
    match active_flow {
        Some(active_flow) => {
            Step::agent(id, "reviewer", attach_current_stage_metadata(input, active_flow))
        }
        None => Step::agent(id, "reviewer", input),
    }
}

fn review_tool_step(
    id: String,
    input: Value,
    target_name: &str,
    active_flow: Option<&SessionFlowState>,
) -> Result<Step, StepError> {
    match active_flow {
        Some(active_flow) => {
            Step::tool(id, target_name, attach_current_stage_metadata(input, active_flow))
        }
        None => Step::tool(id, target_name, input),
    }
}

fn review_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
    reviewer_id: String,
    adjudication: bool,
) -> Value {
    json!({
        "phase": "review",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
        "reviewer_id": reviewer_id,
        "adjudication": adjudication,
        "default_review_trigger": profile.review.as_ref().and_then(default_success_review_trigger),
    })
}

fn review_vote_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
) -> Value {
    json!({
        "phase": "review-vote",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
    })
}

fn review_finalize_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
) -> Value {
    json!({
        "phase": "review-finalize",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
    })
}

fn default_success_review_trigger(review: &ReviewProfile) -> Option<ReviewTrigger> {
    review
        .triggers
        .iter()
        .copied()
        .find(|trigger| !matches!(trigger, ReviewTrigger::ValidationFailed))
}

fn attach_current_stage_metadata(input: Value, active_flow: &SessionFlowState) -> Value {
    let mut input_object = input.as_object().cloned().unwrap_or_default();
    input_object.insert(
        FLOW_METADATA_KEY.to_string(),
        json!({
            "flow_name": active_flow.flow_name,
            "stage_id": active_flow.current_stage_id,
            "stage_index": active_flow.current_stage_index,
            "total_stages": active_flow.total_stages,
        }),
    );
    Value::Object(input_object)
}

fn read_targets_for_profile(profile: &WorkspaceExecutionProfile) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for target in &profile.read_targets {
        targets.insert(target.clone());
    }

    if let Some(attempt) = profile.attempts.first() {
        for change in &attempt.changes {
            targets.insert(change.path.clone());
        }
    }

    targets.into_iter().collect()
}

fn execution_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
) -> Result<&ExecutionAttemptDefinition, FixtureRuntimeError> {
    profile.attempts.get(attempt_index).ok_or_else(|| FixtureRuntimeError::InvalidAttemptIndex {
        profile: profile.name.clone(),
        attempt_index,
    })
}

fn execution_attempt_from_request(
    profile: &WorkspaceExecutionProfile,
    request: &StepExecutionRequest,
) -> Result<ExecutionAttemptDefinition, FixtureRuntimeError> {
    if let Some(attempt) = request.input.get("adaptive_attempt") {
        return serde_json::from_value(attempt.clone()).map_err(|error| {
            FixtureRuntimeError::InvalidAdaptiveAttemptMetadata {
                profile: profile.name.clone(),
                message: error.to_string(),
            }
        });
    }

    let attempt_index =
        request.input.get("attempt_index").and_then(Value::as_u64).unwrap_or(0) as usize;
    execution_attempt(profile, attempt_index).cloned()
}

fn analyze_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    match snapshot_workspace_targets(workspace, &read_targets_for_profile(profile)) {
        Ok(snapshots) => {
            let mut state_patch = Map::new();
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "analysis_targets": snapshots,
                "legacy_source": profile.legacy_source,
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            if state_patch.is_empty() {
                StepExecutionResult::success(rendered_output)
            } else {
                insert_adaptive_output_from_input(&mut rendered_output, &request.input);
                StepExecutionResult::success_with_patch(rendered_output, state_patch)
            }
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_analysis_failed",
                format!("failed to snapshot the workspace before delivery: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn analyze_workspace_with_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    goal: &str,
    route: &ModelRoute,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let sources = match load_workspace_target_sources(workspace, &read_targets_for_profile(profile))
    {
        Ok(sources) => sources,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "provider_analysis_snapshot_failed",
                    format!("failed to snapshot workspace targets for provider analysis: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };
    let provider_files = provider_workspace_files(&sources);
    let guidance_context =
        resolve_phase_guidance(workspace, CapabilityPhase::Planning, goal, &request, &sources);
    let persona = resolve_effective_persona(workspace, &request);
    let analysis = provider_runtime::analyze_workspace(
        route,
        &ProviderAnalysisRequest {
            goal: goal.to_string(),
            phase: provider_phase_label(&request),
            files: provider_files,
            guidance_context,
            persona,
        },
    );

    match analysis {
        Ok(analysis) => {
            let mut state_patch = Map::new();
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "legacy_source": profile.legacy_source,
                "provider_route": provider_route_label(route),
                "analysis_headline": analysis.headline,
                "analysis_summary": analysis.summary,
                "analysis_risks": analysis.risks,
                "analysis_targets": provider_workspace_previews(&sources),
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            if state_patch.is_empty() {
                StepExecutionResult::success(rendered_output)
            } else {
                insert_adaptive_output_from_input(&mut rendered_output, &request.input);
                StepExecutionResult::success_with_patch(rendered_output, state_patch)
            }
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "provider_analysis_failed",
                format!(
                    "provider analysis failed for route {}: {error}",
                    provider_route_label(route)
                ),
            ),
            provider_error_recoverability(&error),
        ),
    }
}

fn analyze_workspace_with_optional_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    route: Option<&ModelRoute>,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    match route {
        Some(route) => analyze_workspace_with_provider(
            workspace,
            profile,
            &provider_goal_from_request(profile, &request),
            route,
            request,
        ),
        None => analyze_workspace_fixture(workspace, profile, request),
    }
}

fn apply_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    if request.input.get("force_retry_once").and_then(Value::as_bool).unwrap_or(false)
        && request.attempt_number == 1
    {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_retry_once",
                format!(
                    "workspace execution profile '{}' intentionally requests one retry before applying changes",
                    profile.name
                ),
            ),
            Recoverability::Retryable,
        );
    }

    let attempt = match execution_attempt_from_request(profile, &request) {
        Ok(attempt) => attempt,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_execution_attempt",
                    format!("invalid execution attempt metadata: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    match apply_execution_attempt(workspace, &attempt) {
        Ok(report) => {
            let changed_files = if report.updated_files.is_empty() {
                report.already_applied_files.clone()
            } else {
                report.updated_files.clone()
            };
            let mut state_patch = Map::new();
            state_patch.insert("latest_attempt_id".to_string(), json!(attempt.attempt_id));
            state_patch.insert("latest_changed_files".to_string(), json!(changed_files));
            state_patch.insert(
                "latest_change_evidence".to_string(),
                serde_json::to_value(&report.change_evidence).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "change_applied": true,
                "changed_files": changed_files,
                "already_applied_files": report.already_applied_files,
                "change_evidence": report.change_evidence,
            });
            insert_adaptive_output_from_input(&mut rendered_output, &request.input);
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_change_failed",
                format!("failed to apply the workspace change set: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn apply_workspace_with_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    goal: &str,
    route: &ModelRoute,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    if request.input.get("force_retry_once").and_then(Value::as_bool).unwrap_or(false)
        && request.attempt_number == 1
    {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_retry_once",
                format!(
                    "workspace execution profile '{}' intentionally requests one retry before applying changes",
                    profile.name
                ),
            ),
            Recoverability::Retryable,
        );
    }

    let allowed_paths = read_targets_for_profile(profile);
    let sources = match load_workspace_target_sources(workspace, &allowed_paths) {
        Ok(sources) => sources,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "provider_change_snapshot_failed",
                    format!("failed to snapshot workspace targets for provider changes: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    let guidance_context = resolve_phase_guidance(
        workspace,
        CapabilityPhase::Implementation,
        goal,
        &request,
        &sources,
    );
    let persona = resolve_effective_persona(workspace, &request);

    let change_response = provider_runtime::propose_workspace_changes(
        route,
        &ProviderChangeRequest {
            goal: goal.to_string(),
            phase: provider_phase_label(&request),
            allowed_paths: allowed_paths.clone(),
            files: provider_workspace_files(&sources),
            guidance_context,
            persona,
        },
    );

    let change_response = match change_response {
        Ok(change_response) => change_response,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "provider_change_failed",
                    format!(
                        "provider change generation failed for route {}: {error}",
                        provider_route_label(route)
                    ),
                ),
                provider_error_recoverability(&error),
            );
        }
    };

    if change_response.changes.is_empty() {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "provider_change_empty",
                format!(
                    "provider route {} did not return a credible bounded change set",
                    provider_route_label(route)
                ),
            ),
            Recoverability::Terminal,
        );
    }

    let attempt = ExecutionAttemptDefinition {
        attempt_id: format!("provider-{}", request.step_id),
        summary: change_response.headline.clone(),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: change_response
            .changes
            .iter()
            .map(|change| WorkspaceChange {
                path: change.path.clone(),
                find: change.find.clone(),
                replace: change.replace.clone(),
            })
            .collect(),
    };

    match apply_execution_attempt(workspace, &attempt) {
        Ok(report) => {
            let changed_files = if report.updated_files.is_empty() {
                report.already_applied_files.clone()
            } else {
                report.updated_files.clone()
            };
            let mut state_patch = Map::new();
            state_patch.insert("latest_attempt_id".to_string(), json!(attempt.attempt_id));
            state_patch.insert("latest_changed_files".to_string(), json!(changed_files));
            state_patch.insert(
                "latest_change_evidence".to_string(),
                serde_json::to_value(&report.change_evidence).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "change_applied": true,
                "provider_route": provider_route_label(route),
                "provider_headline": change_response.headline,
                "provider_summary": change_response.summary,
                "changed_files": changed_files,
                "already_applied_files": report.already_applied_files,
                "change_evidence": report.change_evidence,
            });
            insert_adaptive_output_from_input(&mut rendered_output, &request.input);
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "provider_change_apply_failed",
                format!("failed to apply provider-generated change set: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn apply_workspace_with_optional_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    route: Option<&ModelRoute>,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    match route {
        Some(route) => apply_workspace_with_provider(
            workspace,
            profile,
            &provider_goal_from_request(profile, &request),
            route,
            request,
        ),
        None => apply_workspace_fixture(workspace, profile, request),
    }
}

fn review_workspace_with_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    goal: &str,
    route: &ModelRoute,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success(json!({"review_skipped": true}));
    };

    let Some(reviewer_id) = request.input.get("reviewer_id").and_then(Value::as_str) else {
        return StepExecutionResult::failure(
            ErrorInfo::new("missing_reviewer_id", "review step is missing reviewer_id metadata"),
            Recoverability::Terminal,
        );
    };
    let adjudication = request.input.get("adjudication").and_then(Value::as_bool).unwrap_or(false);
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success(json!({
            "review_skipped": true,
            "reviewer_id": reviewer_id,
        }));
    };

    let (reviewer_role, reviewer_source) = match review.reviewer_by_id(reviewer_id) {
        Some(reviewer) => (reviewer.role.clone(), reviewer.source.clone()),
        None if adjudication => ("Adjudicator".to_string(), None),
        None => {
            return review_terminal_failure(
                "unknown_reviewer",
                format!("reviewer '{reviewer_id}' is not configured in the review council"),
                Some(trigger),
                reviewer_id,
            );
        }
    };

    let sources = match load_workspace_target_sources(workspace, &read_targets_for_profile(profile))
    {
        Ok(sources) => sources,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    PROVIDER_REVIEW_SNAPSHOT_FAILED_CODE,
                    format!("failed to snapshot workspace targets for provider review: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    let review_response = provider_runtime::review_workspace(
        route,
        &ProviderReviewRequest {
            goal: goal.to_string(),
            phase: provider_phase_label(&request),
            reviewer_id: reviewer_id.to_string(),
            reviewer_role: reviewer_role.clone(),
            attempt_id: provider_review_attempt_id(&request),
            files: provider_workspace_files(&sources),
            prior_context: provider_review_prior_context(&request),
        },
    );

    let review_response = match review_response {
        Ok(response) => response,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    PROVIDER_REVIEW_FAILED_CODE,
                    format!(
                        "provider review failed for route {}: {error}",
                        provider_route_label(route)
                    ),
                ),
                provider_error_recoverability(&error),
            );
        }
    };

    let reviewer_effective_route = Some(provider_route_label(route));
    let finding = provider_review_finding(reviewer_id, &reviewer_role, review_response);
    let mut findings = review_findings_from_state(&request);
    findings.retain(|existing| existing.reviewer_id != reviewer_id);
    findings.push(finding.clone());
    let mut reviewers = review_reviewer_ids_from_state(&request);
    if !reviewers.contains(&reviewer_id.to_string()) {
        reviewers.push(reviewer_id.to_string());
    }
    let mut participants = review_participants_from_state(&request);
    participants.retain(|participant| participant.reviewer_id != reviewer_id);
    participants.push(ReviewerParticipation {
        reviewer_id: reviewer_id.to_string(),
        status: ReviewerParticipationStatus::Completed,
        reason: None,
        effective_route: reviewer_effective_route.clone(),
    });

    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert(
        "latest_review_findings".to_string(),
        serde_json::to_value(&findings).unwrap_or(Value::Null),
    );
    state_patch.insert("latest_reviewers".to_string(), json!(reviewers));
    state_patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(&participants).unwrap_or(Value::Null),
    );
    if adjudication {
        state_patch.insert(
            "latest_review_adjudication".to_string(),
            serde_json::to_value(&finding).unwrap_or(Value::Null),
        );
    }

    let governance_context = governance_context_from_request(&request);
    let mut rendered_output = json!({
        "review_trigger": trigger,
        "reviewer_id": reviewer_id,
        "reviewer_role": reviewer_role,
        "reviewer_source": reviewer_source,
        "reviewer_effective_route": reviewer_effective_route,
        "provider_route": provider_route_label(route),
        "finding": finding,
        "adjudication": adjudication,
        "review_targets": provider_workspace_previews(&sources),
    });
    if let Some(governance_context) = governance_context {
        rendered_output["governance_context"] = governance_context;
    }

    StepExecutionResult::success_with_patch(rendered_output, state_patch)
}

fn review_workspace_with_optional_provider(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    provider_review_routes: &BTreeMap<String, ModelRoute>,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let reviewer_id = request.input.get("reviewer_id").and_then(Value::as_str).map(str::to_string);
    let route =
        reviewer_id.as_deref().and_then(|reviewer_id| provider_review_routes.get(reviewer_id));

    match route {
        Some(route) => review_workspace_with_provider(
            workspace,
            profile,
            &provider_goal_from_request(profile, &request),
            route,
            request,
        ),
        None => review_workspace_fixture(profile, request),
    }
}

fn provider_phase_label(request: &StepExecutionRequest) -> String {
    request
        .input
        .get("phase")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(request.step_id.as_str())
        .to_string()
}

fn resolve_phase_guidance(
    workspace: &Path,
    phase: CapabilityPhase,
    goal: &str,
    request: &StepExecutionRequest,
    sources: &[WorkspaceTargetSource],
) -> Vec<String> {
    let signals = collect_workspace_signals(workspace);
    let evidence = GuidanceRuntimeEvidence {
        goal_text: goal.to_string(),
        language: signals.language.clone(),
        selected_targets: sources.iter().map(|source| source.path.clone()).collect(),
        primary_inputs: provider_primary_input_refs(request),
        has_tests: signals.has_tests,
    };
    load_guidance_for_phase(workspace, phase, &evidence)
}

fn provider_primary_input_refs(request: &StepExecutionRequest) -> Vec<String> {
    request
        .input
        .get("authored_brief")
        .cloned()
        .and_then(|value| serde_json::from_value::<AuthoredBriefBundle>(value).ok())
        .map(|bundle| {
            bundle
                .sources
                .into_iter()
                .map(|source| {
                    let display_label = source.display_label();
                    source.workspace_path.unwrap_or(display_label)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Resolves the effective persona label from workspace configuration.
/// Prefers the latest governance packet's intended persona, then workspace or
/// global config, and finally the default delivery-engineer persona.
fn resolve_effective_persona(workspace: &Path, request: &StepExecutionRequest) -> Option<String> {
    if let Ok(Some(packet)) = request.task_snapshot.latest_governance_packet()
        && let Some(authority) = packet.authority_governance.as_ref()
    {
        return Some(authority.intended_persona.as_str().to_string());
    }

    let config = FileConfigStore::for_workspace(workspace).load_local().ok().flatten();
    let global = FileConfigStore::load_global().ok().flatten();
    let canon_prefs = config
        .as_ref()
        .and_then(|config| config.canon.as_ref())
        .or_else(|| global.as_ref().and_then(|config| config.canon.as_ref()));
    if let Some(owner) = canon_prefs.and_then(|prefs| prefs.default_owner.as_deref())
        && let Some(persona) = CanonIntendedPersona::canonicalize_label(owner)
    {
        return Some(persona.to_string());
    }

    Some(CanonIntendedPersona::DeliveryEngineer.as_str().to_string())
}

fn provider_route_label(route: &ModelRoute) -> String {
    format!("{}/{}", route.runtime.as_str(), route.model)
}

fn provider_workspace_files(sources: &[WorkspaceTargetSource]) -> Vec<ProviderWorkspaceFile> {
    sources
        .iter()
        .map(|source| ProviderWorkspaceFile {
            path: source.path.clone(),
            contents: source.contents.clone(),
        })
        .collect()
}

fn provider_workspace_previews(sources: &[WorkspaceTargetSource]) -> Vec<Value> {
    sources
        .iter()
        .map(|source| {
            json!({
                "path": source.path,
                "preview": excerpt(&source.contents),
            })
        })
        .collect()
}

fn provider_error_recoverability(error: &provider_runtime::ProviderRuntimeError) -> Recoverability {
    if error.is_retryable() { Recoverability::Retryable } else { Recoverability::Terminal }
}

fn provider_review_prior_context(request: &StepExecutionRequest) -> Value {
    let mut context = Map::new();
    for key in PROVIDER_REVIEW_PRIOR_CONTEXT_KEYS {
        if let Some(value) = request.task_snapshot.state.get(*key).cloned() {
            context.insert((*key).to_string(), value);
        }
    }
    Value::Object(context)
}

fn provider_goal_from_request(
    profile: &WorkspaceExecutionProfile,
    request: &StepExecutionRequest,
) -> String {
    request
        .task_snapshot
        .state
        .get(TASK_GOAL_KEY)
        .and_then(Value::as_str)
        .or_else(|| request.input.get("goal").and_then(Value::as_str))
        .or_else(|| request.input.get("negotiation_goal_summary").and_then(Value::as_str))
        .or_else(|| {
            request.task_snapshot.state.get("negotiation_goal_summary").and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(profile.name.as_str())
        .to_string()
}

fn explicit_provider_route(route: &ModelRoute) -> Option<ModelRoute> {
    if provider_runtime::route_uses_explicit_provider_namespace(route)
        && provider_runtime::route_is_available(route)
    {
        Some(route.clone())
    } else {
        None
    }
}

fn provider_review_attempt_id(request: &StepExecutionRequest) -> String {
    request
        .input
        .get("attempt_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(request.step_id.as_str())
        .to_string()
}

fn provider_review_finding(
    reviewer_id: &str,
    reviewer_role: &str,
    response: provider_runtime::ProviderReviewResponse,
) -> ReviewerFinding {
    ReviewerFinding {
        reviewer_id: reviewer_id.to_string(),
        disposition: match response.disposition {
            ProviderReviewDisposition::Approve => ReviewerDisposition::Approve,
            ProviderReviewDisposition::Concern => ReviewerDisposition::Concern,
            ProviderReviewDisposition::Block => ReviewerDisposition::Block,
        },
        summary: response.summary,
        details: response.details,
        runtime_role: Some(reviewer_role.to_string()),
        severity: None,
        required_action: response.required_action,
        confidence: None,
        evidence_refs: response.evidence_refs,
    }
}

#[cfg(test)]
fn provider_review_routes_for_profile(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
) -> BTreeMap<String, ModelRoute> {
    let effective = workspace_effective_routing(workspace);

    provider_review_routes_for_profile_with_effective(&effective, profile)
}

fn provider_review_routes_for_profile_with_effective(
    effective: &crate::domain::configuration::EffectiveRouting,
    profile: &WorkspaceExecutionProfile,
) -> BTreeMap<String, ModelRoute> {
    let Some(review) = profile.review.as_ref() else {
        return BTreeMap::new();
    };

    let mut routes = review
        .reviewers
        .iter()
        .filter_map(|reviewer| {
            provider_review_route_for_reviewer(effective, reviewer)
                .map(|route| (reviewer.reviewer_id.clone(), route))
        })
        .collect::<BTreeMap<_, _>>();

    if review.adjudication.enabled
        && let Some(adjudicator_id) = review.adjudication.reviewer_id.as_ref()
        && let Some(route) = explicit_provider_route(&effective.adjudication.route)
    {
        routes.insert(adjudicator_id.clone(), route);
    }

    routes
}

fn provider_review_route_for_reviewer(
    effective: &crate::domain::configuration::EffectiveRouting,
    reviewer: &ReviewerDefinition,
) -> Option<ModelRoute> {
    for key in reviewer_route_config_keys(reviewer) {
        if let Some(route) = effective.reviewer_roles.get(&key) {
            return explicit_provider_route(&route.route);
        }
    }

    explicit_provider_route(&effective.review.route)
}

fn reviewer_route_config_keys(reviewer: &ReviewerDefinition) -> Vec<String> {
    let mut keys = BTreeSet::new();

    for value in [&reviewer.reviewer_id, &reviewer.role] {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        keys.insert(trimmed.to_string());
        let normalized = normalize_reviewer_route_key(trimmed);
        if !normalized.is_empty() {
            keys.insert(normalized);
        }
    }

    keys.into_iter().collect()
}

fn verify_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let attempt = match execution_attempt_from_request(profile, &request) {
        Ok(attempt) => attempt,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_execution_attempt",
                    format!("invalid execution attempt metadata: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    match run_execution_command(workspace, &profile.validation_command) {
        Ok(output) if output.succeeded() => {
            let record = output.to_validation_record();
            let mut state_patch = Map::new();
            state_patch.insert("latest_validation_status".to_string(), json!("passed"));
            state_patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(&record).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            if let Some(trigger) = profile.review.as_ref().and_then(default_success_review_trigger)
            {
                state_patch.insert("next_review_trigger".to_string(), json!(trigger));
            } else {
                state_patch.insert("goal_satisfied".to_string(), json!(true));
            }
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "validation": record,
                "review_trigger": profile.review.as_ref().and_then(default_success_review_trigger),
            });
            insert_adaptive_output_from_input(&mut rendered_output, &request.input);
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }
            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Ok(output)
            if attempt.failure_mode == ExecutionFailureMode::Terminal
                && profile.review.as_ref().is_some_and(|review| {
                    review.triggers.contains(&ReviewTrigger::ValidationFailed)
                }) =>
        {
            let record = output.to_validation_record();
            let mut state_patch = Map::new();
            state_patch.insert("latest_validation_status".to_string(), json!("failed"));
            state_patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(&record).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            state_patch
                .insert("next_review_trigger".to_string(), json!(ReviewTrigger::ValidationFailed));
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "validation": record,
                "review_trigger": ReviewTrigger::ValidationFailed,
            });
            insert_adaptive_output_from_input(&mut rendered_output, &request.input);
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }
            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Ok(output) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_validation_failed",
                format!(
                    "workspace execution profile '{}' still fails validation after attempt {}",
                    profile.name, attempt.attempt_id
                ),
            )
            .with_details(output.details()),
            attempt.failure_mode.recoverability(),
        )
        .with_evidence(adaptive_failure_evidence(
            &request.input,
            &output.to_validation_record(),
            (attempt.failure_mode == ExecutionFailureMode::Terminal).then(|| {
                format!(
                    "adaptive planner exhausted bounded repair after {} because no further preselected candidate remained",
                    attempt.attempt_id
                )
            }),
        ))
        .with_state_patch({
            let mut patch = Map::new();
            patch.insert("latest_validation_status".to_string(), json!("failed"));
            patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(output.to_validation_record()).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut patch,
                &request.input,
                &request.task_snapshot.state,
            );
            if attempt.failure_mode == ExecutionFailureMode::Terminal {
                patch.insert(
                    "latest_exhaustion_reason".to_string(),
                    json!(format!(
                        "adaptive planner exhausted bounded repair after {} because no further preselected candidate remained",
                        attempt.attempt_id
                    )),
                );
            }
            patch
        }),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_verify_failed",
                format!("failed to execute the validation command: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn adaptive_failure_evidence(
    input: &Value,
    validation_record: &ValidationRecord,
    exhaustion_reason: Option<String>,
) -> Value {
    let mut evidence = json!({
        "validation_record": validation_record,
    });

    if let Some(selection_evidence) = input.get("selection_evidence") {
        evidence["selection_evidence"] = selection_evidence.clone();
    }
    if let Some(workspace_slice) = input.get("workspace_slice") {
        evidence["workspace_slice"] = workspace_slice.clone();
    }
    if let Some(attempt_lineage) = input.get("attempt_lineage") {
        evidence["attempt_lineage"] = attempt_lineage.clone();
    }
    if let Some(exhaustion_reason) = exhaustion_reason {
        evidence["exhaustion_reason"] = json!(exhaustion_reason);
    }

    evidence
}

fn review_workspace_fixture(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success(json!({"review_skipped": true}));
    };

    let Some(reviewer_id) = request.input.get("reviewer_id").and_then(Value::as_str) else {
        return StepExecutionResult::failure(
            ErrorInfo::new("missing_reviewer_id", "review step is missing reviewer_id metadata"),
            Recoverability::Terminal,
        );
    };
    let adjudication = request.input.get("adjudication").and_then(Value::as_bool).unwrap_or(false);
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success(json!({
            "review_skipped": true,
            "reviewer_id": reviewer_id,
        }));
    };

    let (reviewer_role, reviewer_source) = match review.reviewer_by_id(reviewer_id) {
        Some(reviewer) => (reviewer.role.clone(), reviewer.source.clone()),
        None if adjudication => ("Adjudicator".to_string(), None),
        None => {
            return review_terminal_failure(
                "unknown_reviewer",
                format!("reviewer '{reviewer_id}' is not configured in the review council"),
                Some(trigger),
                reviewer_id,
            );
        }
    };

    let Some(scenario) = review.scenario_for(trigger) else {
        return review_terminal_failure(
            "missing_review_scenario",
            format!("review trigger '{trigger:?}' does not define a review scenario"),
            Some(trigger),
            reviewer_id,
        );
    };

    let finding = if adjudication {
        scenario.adjudication_finding.as_ref()
    } else {
        scenario.findings.iter().find(|finding| finding.reviewer_id == reviewer_id)
    };
    let Some(finding) = finding else {
        return review_terminal_failure(
            "missing_review_finding",
            format!("reviewer '{reviewer_id}' did not produce a configured finding"),
            Some(trigger),
            reviewer_id,
        );
    };

    let reviewer_effective_routes = review_effective_route_map(review, &request);
    let reviewer_effective_route = reviewer_effective_routes.get(reviewer_id).cloned();

    let mut findings = review_findings_from_state(&request);
    findings.push(finding.clone());
    let mut reviewers = review_reviewer_ids_from_state(&request);
    if !reviewers.contains(&reviewer_id.to_string()) {
        reviewers.push(reviewer_id.to_string());
    }
    let mut participants = review_participants_from_state(&request);
    participants.push(ReviewerParticipation {
        reviewer_id: reviewer_id.to_string(),
        status: ReviewerParticipationStatus::Completed,
        reason: None,
        effective_route: reviewer_effective_route.clone(),
    });

    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert(
        "latest_review_findings".to_string(),
        serde_json::to_value(&findings).unwrap_or(Value::Null),
    );
    state_patch.insert("latest_reviewers".to_string(), json!(reviewers));
    state_patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(&participants).unwrap_or(Value::Null),
    );
    if adjudication {
        state_patch.insert(
            "latest_review_adjudication".to_string(),
            serde_json::to_value(finding).unwrap_or(Value::Null),
        );
    }

    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "reviewer_id": reviewer_id,
            "reviewer_role": reviewer_role,
            "reviewer_source": reviewer_source,
            "reviewer_effective_route": reviewer_effective_route,
            "finding": finding,
            "adjudication": adjudication,
        }),
        state_patch,
    )
}

fn governance_context_from_request(request: &StepExecutionRequest) -> Option<Value> {
    let metadata = FlowStepMetadata::from_value(request.input.get(FLOW_METADATA_KEY)?).ok()??;
    let reused_packets = bounded_reused_packets(&request.task_snapshot, &metadata).ok()?;
    if reused_packets.is_empty() {
        return None;
    }
    let reuse_binding =
        select_packet_reuse_binding(&request.task_snapshot, &metadata).ok().flatten();

    Some(json!({
        "reused_packets": reused_packets,
        "reuse_binding": reuse_binding,
    }))
}

fn resolve_review_vote(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };

    let findings = review_findings_from_state(&request);
    let effective_routes = review_effective_route_map(review, &request);
    let resolution = match review.vote_rule.resolve(
        &review.reviewers,
        &findings,
        Some(&effective_routes),
    ) {
        Ok(resolution) => resolution,
        Err(
            error @ (crate::domain::review::ReviewProfileError::MissingEffectiveReviewerRoute(_)
            | crate::domain::review::ReviewProfileError::DuplicateEffectiveReviewerRoute {
                ..
            }),
        ) => {
            return review_terminal_failure(
                "non_independent_review_council",
                format!("review council is not independent: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
        Err(error) => {
            return review_terminal_failure(
                "invalid_review_vote",
                format!("review vote could not be resolved: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
    };

    let council_profile = active_review_council_profile(review, &request);
    let stop_semantics = active_review_stop_semantics(council_profile, &request);
    let council_decision = match crate::domain::review::resolve_council_assembly(
        council_profile,
        &review.reviewers,
        &resolution.participants,
    ) {
        Ok(decision) => decision,
        Err(
            error @ (crate::domain::review::ReviewProfileError::MissingEffectiveReviewerRoute(_)
            | crate::domain::review::ReviewProfileError::DuplicateEffectiveReviewerRoute {
                ..
            }
            | crate::domain::review::ReviewProfileError::FailedReviewerIndependence(_)),
        ) => {
            return review_terminal_failure(
                "non_independent_review_council",
                format!("review council is not independent: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
        Err(
            error @ (crate::domain::review::ReviewProfileError::InsufficientCouncilReviewers {
                ..
            }
            | crate::domain::review::ReviewProfileError::MissingMandatoryReviewerRole {
                ..
            }),
        ) => {
            return review_terminal_failure(
                "required_review_council_unavailable",
                format!("required review council could not be assembled: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
        Err(error) => {
            return review_terminal_failure(
                "invalid_review_vote",
                format!("review council could not be assembled: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
    };

    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(&resolution.participants).unwrap_or(Value::Null),
    );
    state_patch.insert(
        "latest_review_vote_resolution".to_string(),
        serde_json::to_value(&resolution).unwrap_or(Value::Null),
    );
    state_patch.insert(
        "latest_review_council_resolution".to_string(),
        serde_json::to_value(&council_decision).unwrap_or(Value::Null),
    );
    state_patch
        .insert("latest_review_council_profile".to_string(), json!(council_decision.profile));
    state_patch.insert(
        "latest_review_independence_state".to_string(),
        json!(council_decision.independence_state),
    );
    state_patch.insert(
        "latest_review_selection_summary".to_string(),
        json!(council_decision.selection_summary),
    );
    state_patch.insert("latest_review_stop_semantics".to_string(), json!(stop_semantics));
    state_patch.insert("latest_review_vote".to_string(), json!(render_vote_summary(&resolution)));
    state_patch.insert("latest_review_vote_decision".to_string(), json!(resolution.decision));

    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "vote": resolution,
            "council": council_decision,
        }),
        state_patch,
    )
}

fn active_review_council_profile(
    review: &ReviewProfile,
    request: &StepExecutionRequest,
) -> crate::domain::governance::CouncilProfile {
    governed_council_profile_from_state(request).unwrap_or(match review.reviewers.len() {
        0 => crate::domain::governance::CouncilProfile::None,
        1 => crate::domain::governance::CouncilProfile::LightSingle,
        2..=4 => crate::domain::governance::CouncilProfile::YellowPair,
        _ => crate::domain::governance::CouncilProfile::RedFive,
    })
}

fn governed_council_profile_from_state(
    request: &StepExecutionRequest,
) -> Option<crate::domain::governance::CouncilProfile> {
    let packet =
        request.task_snapshot.state.get("latest_governance_packet").cloned().and_then(|value| {
            serde_json::from_value::<crate::domain::governance::GovernedStagePacket>(value).ok()
        })?;
    let authority = packet.authority_governance.as_ref()?;

    Some(authority.control_resolution_for_stage(packet.canon_mode).council_profile)
}

fn active_review_stop_semantics(
    council_profile: crate::domain::governance::CouncilProfile,
    request: &StepExecutionRequest,
) -> crate::domain::governance::StopSemantics {
    governed_stop_semantics_from_state(request).unwrap_or(match council_profile {
        crate::domain::governance::CouncilProfile::None
        | crate::domain::governance::CouncilProfile::LightSingle => {
            crate::domain::governance::StopSemantics::Proceed
        }
        crate::domain::governance::CouncilProfile::YellowPair => {
            crate::domain::governance::StopSemantics::CouncilRequired
        }
        crate::domain::governance::CouncilProfile::RedFive => {
            crate::domain::governance::StopSemantics::HumanGateRequired
        }
        crate::domain::governance::CouncilProfile::RestrictedManual => {
            crate::domain::governance::StopSemantics::HardStop
        }
    })
}

fn governed_stop_semantics_from_state(
    request: &StepExecutionRequest,
) -> Option<crate::domain::governance::StopSemantics> {
    let packet =
        request.task_snapshot.state.get("latest_governance_packet").cloned().and_then(|value| {
            serde_json::from_value::<crate::domain::governance::GovernedStagePacket>(value).ok()
        })?;
    let authority = packet.authority_governance.as_ref()?;

    Some(authority.control_resolution_for_stage(packet.canon_mode).stop_semantics)
}

fn finalize_workspace_review(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };

    let vote_decision = request
        .task_snapshot
        .state
        .get("latest_review_vote_decision")
        .cloned()
        .and_then(|value| serde_json::from_value::<VoteDecision>(value).ok());

    match vote_decision {
        Some(VoteDecision::Accepted) => review_terminal_success(trigger, ReviewOutcome::Accepted),
        Some(VoteDecision::Rejected) => review_terminal_rejection(trigger),
        Some(VoteDecision::NeedsAdjudication) if review.adjudication.enabled => {
            let adjudication = request
                .task_snapshot
                .state
                .get("latest_review_adjudication")
                .cloned()
                .and_then(|value| serde_json::from_value::<ReviewerFinding>(value).ok());
            match adjudication.map(|finding| finding.disposition) {
                Some(ReviewerDisposition::Approve) => {
                    review_terminal_success(trigger, ReviewOutcome::Accepted)
                }
                Some(ReviewerDisposition::Block) => review_terminal_rejection(trigger),
                Some(ReviewerDisposition::Concern) => review_terminal_escalation(trigger),
                None => review_terminal_failure(
                    "missing_adjudication",
                    "review required adjudication but no adjudication finding was recorded",
                    Some(trigger),
                    "review-finalizer",
                ),
            }
        }
        Some(VoteDecision::NeedsAdjudication) => review_terminal_escalation(trigger),
        None => review_terminal_failure(
            "missing_review_vote",
            "review finalizer could not find a resolved vote decision",
            Some(trigger),
            "review-finalizer",
        ),
    }
}

fn review_terminal_success(trigger: ReviewTrigger, outcome: ReviewOutcome) -> StepExecutionResult {
    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert("latest_review_outcome".to_string(), json!(outcome));
    state_patch.insert("goal_satisfied".to_string(), json!(true));
    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "review_outcome": outcome,
        }),
        state_patch,
    )
}

fn review_terminal_rejection(trigger: ReviewTrigger) -> StepExecutionResult {
    let mut patch = Map::new();
    patch.insert("latest_review_trigger".to_string(), json!(trigger));
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Rejected));
    StepExecutionResult::failure(
        ErrorInfo::new(
            "review_rejected",
            format!("review trigger '{trigger:?}' rejected the delivery result"),
        ),
        Recoverability::Terminal,
    )
    .with_state_patch(patch)
}

fn review_terminal_escalation(trigger: ReviewTrigger) -> StepExecutionResult {
    let mut patch = Map::new();
    patch.insert("latest_review_trigger".to_string(), json!(trigger));
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Escalated));
    StepExecutionResult::failure(
        ErrorInfo::new(
            "review_escalated",
            format!("review trigger '{trigger:?}' ended in escalation"),
        ),
        Recoverability::Terminal,
    )
    .with_state_patch(patch)
}

fn review_terminal_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    trigger: Option<ReviewTrigger>,
    reviewer_id: &str,
) -> StepExecutionResult {
    let message = message.into();
    let mut patch = Map::new();
    if let Some(trigger) = trigger {
        patch.insert("latest_review_trigger".to_string(), json!(trigger));
    }
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Failed));
    patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(vec![ReviewerParticipation {
            reviewer_id: reviewer_id.to_string(),
            status: ReviewerParticipationStatus::Failed,
            reason: Some(message.clone()),
            effective_route: None,
        }])
        .unwrap_or(Value::Null),
    );
    StepExecutionResult::failure(ErrorInfo::new(code, message), Recoverability::Terminal)
        .with_state_patch(patch)
}

fn active_review_trigger(
    review: &ReviewProfile,
    request: &StepExecutionRequest,
) -> Option<ReviewTrigger> {
    request
        .task_snapshot
        .state
        .get("next_review_trigger")
        .cloned()
        .or_else(|| request.input.get("default_review_trigger").cloned())
        .and_then(|value| serde_json::from_value::<ReviewTrigger>(value).ok())
        .filter(|trigger| review.triggers.contains(trigger))
}

fn review_findings_from_state(request: &StepExecutionRequest) -> Vec<ReviewerFinding> {
    request
        .task_snapshot
        .state
        .get("latest_review_findings")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerFinding>>(value).ok())
        .unwrap_or_default()
}

fn review_reviewer_ids_from_state(request: &StepExecutionRequest) -> Vec<String> {
    request
        .task_snapshot
        .state
        .get("latest_reviewers")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn review_participants_from_state(request: &StepExecutionRequest) -> Vec<ReviewerParticipation> {
    request
        .task_snapshot
        .state
        .get("latest_review_participants")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerParticipation>>(value).ok())
        .unwrap_or_default()
}

fn review_effective_route_map(
    review: &ReviewProfile,
    request: &StepExecutionRequest,
) -> BTreeMap<String, String> {
    let projection_routes = routing_projection_from_request(request)
        .map(|projection| projection_route_entries(&projection));
    let mut effective_routes = BTreeMap::new();

    for reviewer in &review.reviewers {
        let route = projection_routes
            .as_ref()
            .and_then(|routes| review_effective_route_from_entries(routes, reviewer))
            .or_else(|| {
                reviewer
                    .source
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            });

        if let Some(route) = route {
            effective_routes.insert(reviewer.reviewer_id.clone(), route);
        }
    }

    effective_routes
}

fn routing_projection_from_request(
    request: &StepExecutionRequest,
) -> Option<RoutingDecisionProjection> {
    request
        .task_snapshot
        .state
        .get("routing_projection")
        .and_then(RoutingDecisionProjection::from_value)
        .or_else(|| {
            request.input.get("routing_projection").and_then(RoutingDecisionProjection::from_value)
        })
}

fn projection_route_entries(projection: &RoutingDecisionProjection) -> BTreeMap<String, String> {
    projection
        .effective_routing
        .iter()
        .filter_map(|entry| {
            let (slot, remainder) = entry.split_once('=')?;
            let route = remainder.split_once(" [").map(|(value, _)| value).unwrap_or(remainder);
            let slot = slot.trim();
            let route = route.trim();
            if slot.is_empty() || route.is_empty() {
                return None;
            }
            Some((slot.to_string(), route.to_string()))
        })
        .collect()
}

fn review_effective_route_from_entries(
    routes: &BTreeMap<String, String>,
    reviewer: &ReviewerDefinition,
) -> Option<String> {
    for key in reviewer_route_projection_keys(reviewer) {
        if let Some(route) = routes.get(&key) {
            return Some(route.clone());
        }
    }

    routes.get("review").cloned()
}

fn reviewer_route_projection_keys(reviewer: &ReviewerDefinition) -> Vec<String> {
    let mut keys = BTreeSet::new();

    for value in [&reviewer.reviewer_id, &reviewer.role] {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        keys.insert(format!("reviewer:{trimmed}"));
        let normalized = normalize_reviewer_route_key(trimmed);
        if !normalized.is_empty() {
            keys.insert(format!("reviewer:{normalized}"));
        }
    }

    keys.into_iter().collect()
}

fn normalize_reviewer_route_key(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('-');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('-').to_string()
}

fn render_vote_summary(resolution: &VoteResolution) -> String {
    format!(
        "strategy={:?} approvals={} concerns={} blocks={} decision={:?}",
        resolution.strategy,
        resolution.approvals,
        resolution.concerns,
        resolution.blocks,
        resolution.decision
    )
}

fn snapshot_workspace_targets(
    workspace: &Path,
    targets: &[String],
) -> Result<Vec<Value>, FixtureRuntimeError> {
    targets
        .iter()
        .map(|target| {
            let path = workspace.join(target);
            let contents = fs::read_to_string(&path)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            Ok(json!({
                "path": target,
                "preview": excerpt(&contents),
            }))
        })
        .collect()
}

fn apply_execution_attempt(
    workspace: &Path,
    attempt: &ExecutionAttemptDefinition,
) -> Result<ExecutionAttemptReport, FixtureRuntimeError> {
    let mut updated_files = Vec::new();
    let mut already_applied_files = Vec::new();
    let mut change_evidence = Vec::new();

    for change in &attempt.changes {
        let path = workspace.join(&change.path);
        let contents = fs::read_to_string(&path)
            .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;

        if contents.contains(&change.find) {
            let updated = contents.replacen(&change.find, &change.replace, 1);
            fs::write(&path, updated)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            updated_files.push(change.path.clone());
            change_evidence.push(ChangeEvidence {
                path: change.path.clone(),
                change_status: ChangeStatus::Updated,
                before_excerpt: excerpt(&change.find),
                after_excerpt: excerpt(&change.replace),
                diff_preview: diff_preview(&change.find, &change.replace),
            });
            continue;
        }

        if contents.contains(&change.replace) {
            already_applied_files.push(change.path.clone());
            change_evidence.push(ChangeEvidence {
                path: change.path.clone(),
                change_status: ChangeStatus::AlreadyApplied,
                before_excerpt: excerpt(&change.find),
                after_excerpt: excerpt(&change.replace),
                diff_preview: diff_preview(&change.find, &change.replace),
            });
            continue;
        }

        return Err(FixtureRuntimeError::PatchTargetMissing { path, needle: change.find.clone() });
    }

    Ok(ExecutionAttemptReport { updated_files, already_applied_files, change_evidence })
}

#[cfg(test)]
fn apply_fixture_patches(
    workspace: &Path,
    fixture: &WorkspaceFixture,
) -> Result<PatchReport, FixtureRuntimeError> {
    let attempt = ExecutionAttemptDefinition {
        attempt_id: "legacy-attempt-1".to_string(),
        summary: format!("Legacy patch application for {}", fixture.name),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: fixture
            .file_patches
            .iter()
            .map(|patch| WorkspaceChange {
                path: patch.path.clone(),
                find: patch.find.clone(),
                replace: patch.replace.clone(),
            })
            .collect(),
    };
    let report = apply_execution_attempt(workspace, &attempt)?;

    Ok(PatchReport {
        updated_files: report.updated_files,
        already_applied_files: report.already_applied_files,
    })
}

#[cfg(test)]
#[allow(dead_code)]
fn run_fixture_command(
    workspace: &Path,
    command: &FixtureCommand,
) -> Result<FixtureCommandOutput, FixtureRuntimeError> {
    run_execution_command(
        workspace,
        &ExecutionCommand { program: command.program.clone(), args: command.args.clone() },
    )
}

fn run_execution_command(
    workspace: &Path,
    command: &ExecutionCommand,
) -> Result<FixtureCommandOutput, FixtureRuntimeError> {
    let rendered_command = render_command(command);
    let output = Command::new(&command.program)
        .args(&command.args)
        .current_dir(workspace)
        .output()
        .map_err(|source| FixtureRuntimeError::CommandLaunch {
            command: rendered_command.clone(),
            source,
        })?;

    Ok(FixtureCommandOutput {
        rendered_command,
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn render_command(command: &ExecutionCommand) -> String {
    if command.args.is_empty() {
        command.program.clone()
    } else {
        format!("{} {}", command.program, command.args.join(" "))
    }
}

fn default_test_command() -> FixtureCommand {
    FixtureCommand {
        program: "cargo".to_string(),
        args: vec!["test".to_string(), "--quiet".to_string()],
    }
}

fn default_run_limits() -> RunLimits {
    RunLimits { max_steps: 3, max_retries: 0, max_replans: 0, ..RunLimits::default() }
}

#[cfg(test)]
struct PatchReport {
    updated_files: Vec<String>,
    already_applied_files: Vec<String>,
}

struct ExecutionAttemptReport {
    updated_files: Vec<String>,
    already_applied_files: Vec<String>,
    change_evidence: Vec<ChangeEvidence>,
}

struct FixtureCommandOutput {
    rendered_command: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl FixtureCommandOutput {
    fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    fn rendered_command(&self) -> &str {
        &self.rendered_command
    }

    fn details(&self) -> Value {
        json!({
            "command": self.rendered_command,
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
        })
    }

    fn to_validation_record(&self) -> ValidationRecord {
        ValidationRecord {
            command: self.rendered_command.clone(),
            exit_code: self.exit_code.unwrap_or(-1),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            succeeded: self.succeeded(),
        }
    }
}

fn excerpt(text: &str) -> String {
    const MAX_LEN: usize = 96;
    if text.len() <= MAX_LEN {
        return text.to_string();
    }

    format!("{}...", &text[..MAX_LEN])
}

fn diff_preview(before: &str, after: &str) -> String {
    format!("- {}\n+ {}", excerpt(before), excerpt(after))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FixtureValidationError {
    #[error("workspace fixture requires a stable name")]
    MissingName,
    #[error("workspace fixture requires a test command program")]
    MissingTestProgram,
    #[error("workspace fixture requires at least one file patch")]
    MissingFilePatches,
    #[error("workspace fixture run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("workspace fixture file patch requires a path")]
    MissingPatchPath,
    #[error("workspace fixture file patch path must be relative: {0}")]
    AbsolutePatchPath(String),
    #[error("workspace fixture file patch requires a search pattern for {0}")]
    MissingFindPattern(String),
}

#[derive(Debug, Error)]
pub enum FixtureRuntimeError {
    #[error("workspace execution profile is missing at {0}")]
    MissingExecutionProfile(PathBuf),
    #[error("failed to read workspace execution profile from {path}: {source}")]
    ExecutionProfileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("workspace execution profile is invalid at {path}: {source}")]
    ExecutionProfileParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("workspace execution profile is invalid: {0}")]
    ExecutionProfileValidation(#[from] ExecutionProfileError),
    #[error("workspace fixture is invalid: {0}")]
    FixtureValidation(#[from] FixtureValidationError),
    #[error("workspace fixture flow metadata is invalid: {0}")]
    FlowValidation(#[from] crate::domain::flow::FlowValidationError),
    #[error("workspace vertical slice contains an invalid step: {0}")]
    InvalidStep(#[from] StepError),
    #[error("workspace vertical slice contains an invalid plan: {0}")]
    InvalidPlan(#[from] crate::domain::plan::PlanError),
    #[error("failed to register fixture agent: {0}")]
    AgentRegistry(#[from] AgentRegistryError),
    #[error("failed to register fixture tool: {0}")]
    ToolRegistry(#[from] ToolRegistryError),
    #[error("failed to read or write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("fixture patch search pattern was not found in {path}: {needle}")]
    PatchTargetMissing { path: PathBuf, needle: String },
    #[error("execution profile '{profile}' does not define attempt index {attempt_index}")]
    InvalidAttemptIndex { profile: String, attempt_index: usize },
    #[error("execution profile '{profile}' does not define a credible adaptive candidate")]
    NoAdaptiveCandidate { profile: String },
    #[error("execution profile '{profile}' does not define adaptive fixture metadata")]
    MissingAdaptiveProfile { profile: String },
    #[error("execution profile '{profile}' returned invalid adaptive attempt metadata: {message}")]
    InvalidAdaptiveAttemptMetadata { profile: String, message: String },
    #[error(
        "could not synthesize a native goal-plan attempt for goal '{goal}' in workspace {workspace}"
    )]
    NoSynthesizeableGoalPlanTarget { goal: String, workspace: PathBuf },
    #[error("failed to execute fixture command `{command}`: {source}")]
    CommandLaunch {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("fixture runtime does not support flow '{flow_name}' in {context}")]
    UnsupportedFixtureFlow { flow_name: String, context: &'static str },
    #[error("execution profile '{profile}' enables review adjudication without an adjudicator")]
    MissingReviewAdjudicator { profile: String },
    #[error("reasoning fixture requires a semver version in major.minor.patch form: {version}")]
    InvalidReasoningFixtureVersion { version: String },
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    use serde_json::{Map, json};
    use uuid::Uuid;

    use super::{
        AdaptiveCandidateContext, ExecutionAttemptDefinition, ExecutionCommand,
        ExecutionFailureMode, FilePatch, FixtureRuntimeError, GeneratedAdaptiveCandidate,
        RankedAdaptiveCandidate, ReasoningProfileFixtureScenario, WorkspaceChange,
        WorkspaceExecutionProfile, WorkspaceFixture, WorkspaceTargetSource,
        adaptive_failure_evidence, adaptive_no_candidate_reason, adaptive_replan_blocker,
        adaptive_selection_headline, adaptive_selection_reason, adaptive_transition_kind,
        analyze_workspace_fixture, apply_fixture_patches, apply_workspace_fixture,
        boolean_flip_candidates, build_adaptive_attempt_steps, build_adaptive_candidates,
        build_adaptive_initial_plan, build_attempt_steps, build_fixture_plan,
        build_fixture_plan_for_flow, build_fixture_runtime, build_fixture_runtime_for_goal_plan,
        build_rejected_candidate_summaries, build_review_steps_for_attempt, build_task_request,
        build_vertical_slice_plan, comparison_flip_candidates, execution_manifest_path,
        first_stable_line, fixture_minimum_independence, fixture_next_minor_exclusive,
        fixture_reasoning_budget, guidance_paths_from_text, infer_goal_plan_change,
        load_workspace_execution_profile, local_reasoning_posture_fixture,
        numeric_literal_flip_candidates, ordering_boundary_flip_candidates,
        provider_review_routes_for_profile, reasoning_profile_fixture, resolve_effective_persona,
        resolve_phase_guidance, resolve_review_vote, resolve_supported_fixture_flow,
        result_status_flip_candidates, review_workspace_fixture, run_fixture_command,
        score_adaptive_candidate, synthesize_goal_plan_execution_profile, verify_workspace_fixture,
    };
    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::brief::normalize_inputs;
    use crate::domain::configuration::{ConfigFile, ModelRoute, RoutingConfig, RuntimeKind};
    use crate::domain::execution::{
        AdaptiveChangeKind, AdaptiveExecutionProfile, AttemptTransitionKind, PathScore,
        ValidationGuidance, ValidationGuidanceConfidence, ValidationGuidanceSource,
        ValidationRecord,
    };
    use crate::domain::flow::{SessionFlowState, attach_stage_metadata, built_in_flow};
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        ApprovalState, CanonAuthorityGovernanceV1Envelope, CanonAuthorityZone, CanonChangeClass,
        CanonIntendedPersona, CanonMode, CanonRiskClass, GovernanceLifecycleState,
        GovernanceRuntimeKind, GovernedStagePacket, GovernedStageRecord, PacketReadiness,
    };
    use crate::domain::guidance::CapabilityPhase;
    use crate::domain::limits::RunLimits;
    use crate::domain::reasoning::{
        ReasoningAdmissionEffect, ReasoningConfidenceLevel, ReasoningProfileId,
    };
    use crate::domain::review::{
        ReviewProfile, ReviewScenario, ReviewTrigger, ReviewerDefinition, ReviewerDisposition,
        ReviewerFinding, VoteDecision, VoteRuleDefinition,
    };
    use crate::domain::step::{ExecutionStatus, Recoverability, StepExecutionRequest, StepKind};
    use crate::domain::task_context::TaskContext;

    fn temp_workspace() -> std::path::PathBuf {
        let workspace =
            std::env::temp_dir().join(format!("boundline-fixture-unit-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        workspace
    }

    fn write_execution_workspace(prefix: &str, source: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".boundline")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"fixture_unit\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(workspace.join("src/lib.rs"), source).unwrap();
        fs::write(
            workspace.join("tests/red_to_green.rs"),
            "#[test]\nfn red_to_green_addition() {\n    assert_eq!(fixture_unit::add(2, 2), 4);\n}\n",
        )
        .unwrap();
        workspace
    }

    fn sample_profile(validation_command: ExecutionCommand) -> WorkspaceExecutionProfile {
        WorkspaceExecutionProfile {
            name: "fixture-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
            validation_command,
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: "Replace subtraction with addition".to_string(),
                failure_mode: ExecutionFailureMode::Retry,
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
        }
    }

    fn sample_review_profile(validation_command: ExecutionCommand) -> WorkspaceExecutionProfile {
        let mut profile = sample_profile(validation_command);
        profile.review = Some(ReviewProfile {
            triggers: vec![ReviewTrigger::PrReady, ReviewTrigger::ValidationFailed],
            reviewers: vec![
                ReviewerDefinition {
                    reviewer_id: "safety".to_string(),
                    role: "Safety".to_string(),
                    source: Some("gpt".to_string()),
                    weight: 2,
                },
                ReviewerDefinition {
                    reviewer_id: "maintainability".to_string(),
                    role: "Maintainability".to_string(),
                    source: Some("claude".to_string()),
                    weight: 1,
                },
            ],
            vote_rule: VoteRuleDefinition::default(),
            adjudication: Default::default(),
            scenarios: vec![
                ReviewScenario {
                    trigger: ReviewTrigger::PrReady,
                    findings: vec![
                        ReviewerFinding::new(
                            "safety".to_string(),
                            ReviewerDisposition::Approve,
                            "No blocking issues".to_string(),
                        ),
                        ReviewerFinding::new(
                            "maintainability".to_string(),
                            ReviewerDisposition::Approve,
                            "Looks ready".to_string(),
                        ),
                    ],
                    adjudication_finding: None,
                },
                ReviewScenario {
                    trigger: ReviewTrigger::ValidationFailed,
                    findings: vec![
                        ReviewerFinding::new(
                            "safety".to_string(),
                            ReviewerDisposition::Block,
                            "Validation still fails".to_string(),
                        ),
                        ReviewerFinding::new(
                            "maintainability".to_string(),
                            ReviewerDisposition::Concern,
                            "Retry after a fix".to_string(),
                        ),
                    ],
                    adjudication_finding: None,
                },
            ],
        });
        profile
    }

    fn sample_adaptive_profile(validation_command: ExecutionCommand) -> WorkspaceExecutionProfile {
        let mut profile = sample_profile(validation_command);
        profile.attempts.clear();
        profile.read_targets = vec!["src/lib.rs".to_string()];
        profile.adaptive = Some(AdaptiveExecutionProfile {
            max_selected_targets: 1,
            max_generated_attempts: 4,
            path_preferences: vec!["src/lib.rs".to_string()],
            allowed_change_kinds: vec![
                AdaptiveChangeKind::ComparisonFlip,
                AdaptiveChangeKind::BooleanFlip,
            ],
        });
        profile
    }

    #[test]
    fn local_reasoning_posture_fixture_tracks_supported_release_window() {
        let posture = local_reasoning_posture_fixture();
        assert!(posture.is_ok(), "{posture:?}");
        let posture = posture.unwrap();

        assert!(posture.validate().is_ok());
        assert!(posture.compatibility_window.admits_versions(
            env!("CARGO_PKG_VERSION"),
            crate::domain::distribution::SUPPORTED_CANON_VERSION,
        ));
    }

    #[test]
    fn reasoning_profile_fixture_scenarios_are_deterministic() {
        let blocked =
            reasoning_profile_fixture(ReasoningProfileFixtureScenario::IndependentPairBlocked);
        assert!(blocked.is_ok(), "{blocked:?}");
        let blocked = blocked.unwrap();
        let warn = reasoning_profile_fixture(ReasoningProfileFixtureScenario::BoundedReflexionWarn);
        assert!(warn.is_ok(), "{warn:?}");
        let warn = warn.unwrap();

        assert_eq!(blocked.profile_id, ReasoningProfileId::IndependentPairReview);
        assert_eq!(
            blocked.confidence.as_ref().map(|value| value.confidence_level),
            Some(ReasoningConfidenceLevel::Low)
        );
        assert_eq!(
            blocked.confidence.as_ref().map(|value| value.admission_effect),
            Some(ReasoningAdmissionEffect::Gate)
        );
        assert_eq!(warn.profile_id, ReasoningProfileId::BoundedReflexion);
        assert_eq!(
            warn.confidence.as_ref().map(|value| value.confidence_level),
            Some(ReasoningConfidenceLevel::Medium)
        );
        assert_eq!(
            warn.confidence.as_ref().map(|value| value.admission_effect),
            Some(ReasoningAdmissionEffect::Warn)
        );
    }

    fn unsupported_flow_state() -> SessionFlowState {
        SessionFlowState {
            flow_name: "unsupported-flow".to_string(),
            current_stage_id: "invent-stage".to_string(),
            current_stage_index: 0,
            total_stages: 1,
        }
    }

    fn first_adaptive_candidate(
        workspace: &std::path::Path,
        profile: &WorkspaceExecutionProfile,
        goal: &str,
    ) -> super::AdaptiveAttemptPlan {
        let used_signatures = BTreeSet::new();
        build_adaptive_candidates(
            workspace,
            profile,
            goal,
            AdaptiveCandidateContext {
                used_signatures: &used_signatures,
                previous_attempt_id: None,
                previous_selected_targets: None,
                validation_guidance: None,
                lineage_reason: "selected adaptive candidate for test coverage",
            },
        )
        .unwrap()
        .into_iter()
        .next()
        .expect("adaptive candidate should exist")
    }

    fn request(input: serde_json::Value, attempt_number: usize) -> StepExecutionRequest {
        StepExecutionRequest {
            step_id: "code".to_string(),
            step_kind: StepKind::Agent,
            target_name: "coder".to_string(),
            input,
            task_snapshot: TaskContext::new(
                "session-1",
                "/tmp/workspace",
                RunLimits::default(),
                Map::new(),
            ),
            attempt_number,
        }
    }

    fn request_with_state(
        input: serde_json::Value,
        attempt_number: usize,
        state: Map<String, serde_json::Value>,
    ) -> StepExecutionRequest {
        StepExecutionRequest {
            step_id: "review".to_string(),
            step_kind: StepKind::Tool,
            target_name: "review-voter".to_string(),
            input,
            task_snapshot: TaskContext::new(
                "session-1",
                "/tmp/workspace",
                RunLimits::default(),
                state,
            ),
            attempt_number,
        }
    }

    #[test]
    fn loader_rejects_a_missing_manifest() {
        let workspace = temp_workspace();
        let error = load_workspace_execution_profile(&workspace).unwrap_err();

        assert!(matches!(error, FixtureRuntimeError::MissingExecutionProfile(_)));
    }

    #[test]
    fn patch_application_is_idempotent_after_the_replacement_is_present() {
        let workspace = temp_workspace();
        let source_path = workspace.join("src.txt");
        fs::write(&source_path, "red").unwrap();
        let fixture = WorkspaceFixture {
            name: "bug-fix".to_string(),
            test_command: super::default_test_command(),
            limits: super::default_run_limits(),
            file_patches: vec![FilePatch {
                path: "src.txt".to_string(),
                find: "red".to_string(),
                replace: "green".to_string(),
            }],
        };

        let first = apply_fixture_patches(&workspace, &fixture).unwrap();
        let second = apply_fixture_patches(&workspace, &fixture).unwrap();

        assert_eq!(first.updated_files, vec!["src.txt".to_string()]);
        assert_eq!(second.already_applied_files, vec!["src.txt".to_string()]);
        assert_eq!(fs::read_to_string(source_path).unwrap(), "green");
    }

    #[test]
    fn workspace_fixture_validation_rejects_missing_fields_and_invalid_patches() {
        assert!(matches!(
            WorkspaceFixture {
                name: " ".to_string(),
                test_command: super::default_test_command(),
                limits: super::default_run_limits(),
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingName
        ));
        assert!(matches!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::FixtureCommand { program: " ".to_string(), args: vec![] },
                limits: super::default_run_limits(),
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingTestProgram
        ));
        assert!(matches!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::default_test_command(),
                limits: super::default_run_limits(),
                file_patches: vec![],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingFilePatches
        ));
        assert!(matches!(
            FilePatch {
                path: " ".to_string(),
                find: "red".to_string(),
                replace: "green".to_string()
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingPatchPath
        ));
        assert!(matches!(
            FilePatch {
                path: "/tmp/outside.rs".to_string(),
                find: "red".to_string(),
                replace: "green".to_string(),
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::AbsolutePatchPath(_)
        ));
        assert!(matches!(
            FilePatch {
                path: "src/lib.rs".to_string(),
                find: "".to_string(),
                replace: "green".to_string()
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingFindPattern(_)
        ));
    }

    #[test]
    fn workspace_fixture_validation_rejects_invalid_limits_and_manifest_path_is_stable() {
        let workspace = temp_workspace();

        assert_eq!(
            execution_manifest_path(&workspace),
            workspace.join(".boundline/execution.json")
        );
        assert!(matches!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::default_test_command(),
                limits: RunLimits { max_steps: 0, ..super::default_run_limits() },
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::InvalidRunLimits(_)
        ));

        assert!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::default_test_command(),
                limits: super::default_run_limits(),
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn load_workspace_execution_profile_reports_parse_and_missing_errors() {
        let workspace = temp_workspace();
        fs::write(execution_manifest_path(&workspace), b"{not json").unwrap();
        assert!(matches!(
            load_workspace_execution_profile(&workspace).unwrap_err(),
            FixtureRuntimeError::ExecutionProfileParse { .. }
        ));

        fs::remove_file(execution_manifest_path(&workspace)).unwrap();
        assert!(matches!(
            load_workspace_execution_profile(&workspace).unwrap_err(),
            FixtureRuntimeError::MissingExecutionProfile(_)
        ));
    }

    #[test]
    fn build_vertical_slice_plan_covers_non_flow_and_all_built_in_flows() {
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let direct = build_vertical_slice_plan(&profile, None, 0).unwrap();
        assert_eq!(direct.steps[0].id, "analyze");
        assert_eq!(direct.steps[1].id, "code-fix-add");
        assert_eq!(direct.steps[2].id, "verify-fix-add");

        let bug_fix = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            bug_fix.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["investigate", "implement", "verify"]
        );

        let change = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("change").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            change.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["understand-change", "implement", "verify"]
        );

        let delivery = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("delivery").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            delivery.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "requirements",
                "architecture",
                "backlog",
                "implementation-code",
                "implementation-verify"
            ]
        );
    }

    #[test]
    fn build_fixture_plans_report_unsupported_flows() {
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let unsupported_flow = unsupported_flow_state();

        let vertical_slice_error =
            build_vertical_slice_plan(&profile, Some(&unsupported_flow), 0).unwrap_err();
        assert!(matches!(
            vertical_slice_error,
            FixtureRuntimeError::UnsupportedFixtureFlow { flow_name, context }
                if flow_name == "unsupported-flow" && context == "fixture planning"
        ));

        let workspace = write_execution_workspace(
            "boundline-fixture-unsupported-flow",
            "pub fn add(left: i32, right: i32) -> bool {\n    left != right\n}\n",
        );
        let adaptive_profile = sample_adaptive_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let adaptive_error = build_adaptive_initial_plan(
            &workspace,
            &adaptive_profile,
            Some(&unsupported_flow),
            "repair the unsupported flow",
        )
        .unwrap_err();
        assert!(matches!(
            adaptive_error,
            FixtureRuntimeError::UnsupportedFixtureFlow { flow_name, context }
                if flow_name == "unsupported-flow" && context == "adaptive fixture planning"
        ));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn adaptive_fixture_planning_requires_an_adaptive_profile() {
        let workspace = temp_workspace();
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let error =
            build_adaptive_initial_plan(&workspace, &profile, None, "repair the bug").unwrap_err();

        assert!(matches!(
            error,
            FixtureRuntimeError::MissingAdaptiveProfile { profile }
                if profile == "fixture-profile"
        ));

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_vertical_slice_plan_appends_review_steps_when_review_is_configured() {
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let direct = build_vertical_slice_plan(&profile, None, 0).unwrap();

        assert_eq!(
            direct.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "analyze",
                "code-fix-add",
                "verify-fix-add",
                "review-fix-add-safety",
                "review-fix-add-maintainability",
                "review-fix-add-vote",
                "review-fix-add-finalize",
            ]
        );
    }

    #[test]
    fn build_review_steps_for_attempt_includes_adjudication_for_flow_scoped_reviews() {
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let review = profile.review.as_mut().unwrap();
        review.adjudication.enabled = true;
        review.adjudication.reviewer_id = Some("arbiter".to_string());
        review.scenarios[0].adjudication_finding = Some(ReviewerFinding::new(
            "arbiter".to_string(),
            ReviewerDisposition::Concern,
            "Needs one more human look".to_string(),
        ));

        let steps = build_review_steps_for_attempt(
            &profile,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
            "attempt-1",
        )
        .unwrap();

        assert_eq!(
            steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "investigate-review-attempt-1-safety",
                "investigate-review-attempt-1-maintainability",
                "investigate-review-attempt-1-vote",
                "investigate-review-attempt-1-adjudicate",
                "investigate-review-attempt-1-finalize",
            ]
        );
    }

    #[test]
    fn build_review_steps_for_attempt_requires_an_adjudicator_when_enabled() {
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        profile.review.as_mut().unwrap().adjudication.enabled = true;

        let error = build_review_steps_for_attempt(&profile, None, "attempt-1").unwrap_err();

        assert!(matches!(
            error,
            FixtureRuntimeError::MissingReviewAdjudicator { profile }
                if profile == "fixture-profile"
        ));
    }

    #[test]
    fn build_adaptive_initial_plan_covers_change_and_delivery_flows() {
        let workspace = write_execution_workspace(
            "boundline-fixture-adaptive-flow-coverage",
            "pub fn add(left: i32, right: i32) -> bool {\n    left != right\n}\n",
        );
        let profile = sample_adaptive_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let change = build_adaptive_initial_plan(
            &workspace,
            &profile,
            Some(&built_in_flow("change").unwrap().initial_state()),
            "understand the bounded change",
        )
        .unwrap();
        assert_eq!(
            change.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["understand-change", "implement", "verify"]
        );

        let delivery = build_adaptive_initial_plan(
            &workspace,
            &profile,
            Some(&built_in_flow("delivery").unwrap().initial_state()),
            "deliver the bounded change",
        )
        .unwrap();
        assert_eq!(
            delivery.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "requirements",
                "architecture",
                "backlog",
                "implementation-code",
                "implementation-verify",
            ]
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_adaptive_attempt_steps_covers_flow_scoped_and_direct_variants() {
        let workspace = write_execution_workspace(
            "boundline-fixture-adaptive-attempt-coverage",
            "pub fn add(left: i32, right: i32) -> bool {\n    left != right\n}\n",
        );
        let profile = sample_adaptive_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let candidate = first_adaptive_candidate(&workspace, &profile, "repair the bounded change");

        let flow_steps = build_adaptive_attempt_steps(
            &profile,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
            &candidate,
        )
        .unwrap();
        assert_eq!(
            flow_steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "investigate-replan-adaptive-attempt-1-code",
                "investigate-replan-adaptive-attempt-1-verify",
            ]
        );

        let direct_steps = build_adaptive_attempt_steps(&profile, None, &candidate).unwrap();
        assert_eq!(
            direct_steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["code-adaptive-attempt-1", "verify-adaptive-attempt-1"]
        );

        fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn build_attempt_steps_covers_flow_scoped_and_direct_variants() {
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let flow_steps = build_attempt_steps(
            &profile,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            flow_steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["investigate-replan-fix-add-code", "investigate-replan-fix-add-verify"]
        );

        let direct_steps = build_attempt_steps(&profile, None, 0).unwrap();
        assert_eq!(
            direct_steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["code-fix-add", "verify-fix-add"]
        );
    }

    #[test]
    fn comparison_and_boolean_flip_candidates_cover_both_directions() {
        assert_eq!(
            comparison_flip_candidates("src/lib.rs", "left != right"),
            vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: " != ".to_string(),
                replace: " == ".to_string(),
            }]
        );
        assert_eq!(
            comparison_flip_candidates("src/lib.rs", "left == right"),
            vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: " == ".to_string(),
                replace: " != ".to_string(),
            }]
        );
        assert!(comparison_flip_candidates("src/lib.rs", "left < right").is_empty());

        assert_eq!(
            boolean_flip_candidates("src/lib.rs", "return false;"),
            vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "false".to_string(),
                replace: "true".to_string(),
            }]
        );
        assert_eq!(
            boolean_flip_candidates("src/lib.rs", "return true;"),
            vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "true".to_string(),
                replace: "false".to_string(),
            }]
        );
        assert!(boolean_flip_candidates("src/lib.rs", "return value;").is_empty());
    }

    #[test]
    fn review_validation_failure_is_routed_into_review_state() {
        let workspace = write_execution_workspace(
            "boundline-fixture-review-validation-failure",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        profile.attempts[0].failure_mode = ExecutionFailureMode::Terminal;

        let result =
            verify_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 1));

        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            result.state_patch.as_ref().unwrap()["next_review_trigger"],
            json!(ReviewTrigger::ValidationFailed)
        );
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("failed")
        );
    }

    #[test]
    fn review_vote_resolution_succeeds_for_pr_ready_findings() {
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(profile.review.as_ref().unwrap().scenarios[0].findings.clone())
                .unwrap(),
        );

        let result = resolve_review_vote(&profile, request_with_state(json!({}), 1, state));

        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_vote_decision"],
            json!(VoteDecision::Accepted)
        );
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_council_profile"],
            json!(crate::domain::governance::CouncilProfile::YellowPair)
        );
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_independence_state"],
            json!(crate::domain::review::ReviewerIndependenceState::Passed)
        );
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_stop_semantics"],
            json!(crate::domain::governance::StopSemantics::CouncilRequired)
        );
    }

    #[test]
    fn review_workspace_fixture_covers_skip_and_failure_paths() {
        let no_review = review_workspace_fixture(
            &sample_profile(ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            }),
            request(json!({}), 1),
        );
        assert_eq!(no_review.status, ExecutionStatus::Succeeded);
        assert_eq!(no_review.output.as_ref().unwrap()["review_skipped"], json!(true));

        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let missing_reviewer = review_workspace_fixture(&profile, request(json!({}), 1));
        assert_eq!(missing_reviewer.error.as_ref().unwrap().code, "missing_reviewer_id");

        let skipped =
            review_workspace_fixture(&profile, request(json!({"reviewer_id": "safety"}), 1));
        assert_eq!(skipped.status, ExecutionStatus::Succeeded);
        assert_eq!(skipped.output.as_ref().unwrap()["review_skipped"], json!(true));

        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        let unknown = review_workspace_fixture(
            &profile,
            request_with_state(json!({"reviewer_id": "ghost"}), 1, state.clone()),
        );
        assert_eq!(unknown.error.as_ref().unwrap().code, "unknown_reviewer");

        let mut missing_scenario_profile = profile.clone();
        missing_scenario_profile.review.as_mut().unwrap().scenarios.clear();
        let missing_scenario = review_workspace_fixture(
            &missing_scenario_profile,
            request_with_state(json!({"reviewer_id": "safety"}), 1, state.clone()),
        );
        assert_eq!(missing_scenario.error.as_ref().unwrap().code, "missing_review_scenario");

        let mut missing_finding_profile = profile.clone();
        missing_finding_profile.review.as_mut().unwrap().scenarios[0].findings =
            vec![ReviewerFinding::new(
                "maintainability".to_string(),
                ReviewerDisposition::Approve,
                "Looks ready".to_string(),
            )];
        let missing_finding = review_workspace_fixture(
            &missing_finding_profile,
            request_with_state(json!({"reviewer_id": "safety"}), 1, state),
        );
        assert_eq!(missing_finding.error.as_ref().unwrap().code, "missing_review_finding");
    }

    #[test]
    fn review_workspace_fixture_records_adjudication_results() {
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let review = profile.review.as_mut().unwrap();
        review.adjudication.enabled = true;
        review.adjudication.reviewer_id = Some("arbiter".to_string());
        review.scenarios[0].adjudication_finding = Some(ReviewerFinding::new(
            "arbiter".to_string(),
            ReviewerDisposition::Concern,
            "Needs explicit adjudication".to_string(),
        ));

        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        let result = review_workspace_fixture(
            &profile,
            request_with_state(json!({"reviewer_id": "arbiter", "adjudication": true}), 1, state),
        );

        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(result.output.as_ref().unwrap()["adjudication"], json!(true));
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_adjudication"]["reviewer_id"],
            json!("arbiter")
        );
    }

    #[test]
    fn resolve_review_vote_covers_skip_invalid_and_incomplete_paths() {
        let no_review = resolve_review_vote(
            &sample_profile(ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            }),
            request(json!({}), 1),
        );
        assert_eq!(no_review.status, ExecutionStatus::Succeeded);
        assert_eq!(no_review.output.as_ref().unwrap()["review_skipped"], json!(true));

        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let no_trigger = resolve_review_vote(&profile, request(json!({}), 1));
        assert_eq!(no_trigger.status, ExecutionStatus::Succeeded);
        assert_eq!(no_trigger.output.as_ref().unwrap()["review_skipped"], json!(true));

        let mut invalid_state = Map::new();
        invalid_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        invalid_state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(vec![ReviewerFinding::new(
                "ghost".to_string(),
                ReviewerDisposition::Approve,
                "Unknown reviewer".to_string(),
            )])
            .unwrap(),
        );
        let invalid_vote =
            resolve_review_vote(&profile, request_with_state(json!({}), 1, invalid_state));
        assert_eq!(invalid_vote.error.as_ref().unwrap().code, "invalid_review_vote");

        let mut incomplete_state = Map::new();
        incomplete_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        incomplete_state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(vec![
                profile.review.as_ref().unwrap().scenarios[0].findings[0].clone(),
            ])
            .unwrap(),
        );
        let incomplete =
            resolve_review_vote(&profile, request_with_state(json!({}), 1, incomplete_state));
        assert_eq!(incomplete.error.as_ref().unwrap().code, "required_review_council_unavailable");
    }

    #[test]
    fn resolve_review_vote_rejects_non_independent_review_council() {
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(profile.review.as_ref().unwrap().scenarios[0].findings.clone())
                .unwrap(),
        );
        state.insert(
            "routing_projection".to_string(),
            json!({
                "effective_routing": [
                    "review=claude/sonnet-4 [workspace]",
                    "reviewer:safety=claude/sonnet-4 [workspace]",
                    "reviewer:maintainability=claude/sonnet-4 [workspace]"
                ]
            }),
        );

        let result = resolve_review_vote(&profile, request_with_state(json!({}), 1, state));

        assert_eq!(result.error.as_ref().unwrap().code, "non_independent_review_council");
    }

    #[test]
    fn resolve_review_vote_rejects_missing_mandatory_role_for_pair_council() {
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        profile.review.as_mut().unwrap().reviewers.push(ReviewerDefinition {
            reviewer_id: "ux".to_string(),
            role: "UX".to_string(),
            source: Some("gemini/gemini-2.5-pro".to_string()),
            weight: 1,
        });

        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(vec![
                ReviewerFinding::new(
                    "safety".to_string(),
                    ReviewerDisposition::Approve,
                    "Looks safe".to_string(),
                ),
                ReviewerFinding::new(
                    "ux".to_string(),
                    ReviewerDisposition::Approve,
                    "Looks fine".to_string(),
                ),
            ])
            .unwrap(),
        );
        state.insert(
            "routing_projection".to_string(),
            json!({
                "effective_routing": [
                    "reviewer:safety=copilot/gpt-4.1 [workspace]",
                    "reviewer:ux=gemini/gemini-2.5-pro [workspace]"
                ]
            }),
        );

        let result = resolve_review_vote(&profile, request_with_state(json!({}), 1, state));

        assert_eq!(result.error.as_ref().unwrap().code, "required_review_council_unavailable");
    }

    #[test]
    fn finalize_workspace_review_covers_skip_and_terminal_variants() {
        let no_review = super::finalize_workspace_review(
            &sample_profile(ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            }),
            request(json!({}), 1),
        );
        assert_eq!(no_review.status, ExecutionStatus::Succeeded);
        assert_eq!(no_review.output.as_ref().unwrap()["review_skipped"], json!(true));

        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let no_trigger = super::finalize_workspace_review(&profile, request(json!({}), 1));
        assert_eq!(no_trigger.status, ExecutionStatus::Succeeded);
        assert_eq!(no_trigger.output.as_ref().unwrap()["review_skipped"], json!(true));

        let mut rejected_state = Map::new();
        rejected_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        rejected_state
            .insert("latest_review_vote_decision".to_string(), json!(VoteDecision::Rejected));
        let rejected = super::finalize_workspace_review(
            &profile,
            request_with_state(json!({}), 1, rejected_state),
        );
        assert_eq!(rejected.error.as_ref().unwrap().code, "review_rejected");

        let mut adjudicated_profile = profile.clone();
        let review = adjudicated_profile.review.as_mut().unwrap();
        review.adjudication.enabled = true;
        review.adjudication.reviewer_id = Some("arbiter".to_string());

        let mut block_state = Map::new();
        block_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        block_state.insert(
            "latest_review_vote_decision".to_string(),
            json!(VoteDecision::NeedsAdjudication),
        );
        block_state.insert(
            "latest_review_adjudication".to_string(),
            serde_json::to_value(ReviewerFinding::new(
                "arbiter".to_string(),
                ReviewerDisposition::Block,
                "Blocking concern".to_string(),
            ))
            .unwrap(),
        );
        let block = super::finalize_workspace_review(
            &adjudicated_profile,
            request_with_state(json!({}), 1, block_state),
        );
        assert_eq!(block.error.as_ref().unwrap().code, "review_rejected");

        let mut concern_state = Map::new();
        concern_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        concern_state.insert(
            "latest_review_vote_decision".to_string(),
            json!(VoteDecision::NeedsAdjudication),
        );
        concern_state.insert(
            "latest_review_adjudication".to_string(),
            serde_json::to_value(ReviewerFinding::new(
                "arbiter".to_string(),
                ReviewerDisposition::Concern,
                "Escalate for follow-up".to_string(),
            ))
            .unwrap(),
        );
        let concern = super::finalize_workspace_review(
            &adjudicated_profile,
            request_with_state(json!({}), 1, concern_state),
        );
        assert_eq!(concern.error.as_ref().unwrap().code, "review_escalated");

        let mut missing_adjudication_state = Map::new();
        missing_adjudication_state
            .insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        missing_adjudication_state.insert(
            "latest_review_vote_decision".to_string(),
            json!(VoteDecision::NeedsAdjudication),
        );
        let missing_adjudication = super::finalize_workspace_review(
            &adjudicated_profile,
            request_with_state(json!({}), 1, missing_adjudication_state),
        );
        assert_eq!(missing_adjudication.error.as_ref().unwrap().code, "missing_adjudication");

        let mut escalate_state = Map::new();
        escalate_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        escalate_state.insert(
            "latest_review_vote_decision".to_string(),
            json!(VoteDecision::NeedsAdjudication),
        );
        let escalated = super::finalize_workspace_review(
            &profile,
            request_with_state(json!({}), 1, escalate_state),
        );
        assert_eq!(escalated.error.as_ref().unwrap().code, "review_escalated");

        let mut missing_vote_state = Map::new();
        missing_vote_state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        let missing_vote = super::finalize_workspace_review(
            &profile,
            request_with_state(json!({}), 1, missing_vote_state),
        );
        assert_eq!(missing_vote.error.as_ref().unwrap().code, "missing_review_vote");
    }

    #[test]
    fn apply_workspace_fixture_covers_retry_invalid_attempt_success_and_already_applied() {
        let workspace = write_execution_workspace(
            "boundline-fixture-apply",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let retry = apply_workspace_fixture(
            &workspace,
            &profile,
            request(json!({"force_retry_once": true, "attempt_index": 0}), 1),
        );
        assert_eq!(retry.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(retry.recoverability, Recoverability::Retryable);

        let invalid_attempt =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 9}), 2));
        assert_eq!(invalid_attempt.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(invalid_attempt.recoverability, Recoverability::Terminal);

        let applied =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 2));
        assert_eq!(applied.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(applied.output.as_ref().unwrap()["changed_files"], json!(["src/lib.rs"]));
        assert_eq!(applied.state_patch.as_ref().unwrap()["latest_attempt_id"], json!("fix-add"));

        let already_applied =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 3));
        assert_eq!(already_applied.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            already_applied.output.as_ref().unwrap()["already_applied_files"],
            json!(["src/lib.rs"])
        );
    }

    #[test]
    fn verify_workspace_fixture_covers_success_failure_and_command_errors() {
        let success_workspace = write_execution_workspace(
            "boundline-fixture-verify-success",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let retry_profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let success = verify_workspace_fixture(
            &success_workspace,
            &retry_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(success.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            success.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("passed")
        );

        let failure_workspace = write_execution_workspace(
            "boundline-fixture-verify-failure",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let failure = verify_workspace_fixture(
            &failure_workspace,
            &retry_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(failure.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(failure.recoverability, Recoverability::Retryable);
        assert_eq!(
            failure.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("failed")
        );

        let error_profile = sample_profile(ExecutionCommand {
            program: "definitely-not-a-real-command".to_string(),
            args: vec![],
        });
        let command_error = verify_workspace_fixture(
            &failure_workspace,
            &error_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(command_error.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(command_error.recoverability, Recoverability::Terminal);
    }

    #[test]
    fn fixture_runtime_helpers_cover_analysis_builders_and_verify_invalid_attempts() {
        let workspace = write_execution_workspace(
            "boundline-fixture-runtime-helpers",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        fs::write(
            execution_manifest_path(&workspace),
            serde_json::to_string_pretty(&profile).unwrap(),
        )
        .unwrap();

        let plan = build_fixture_plan(&workspace).unwrap();
        assert_eq!(
            plan.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["analyze", "code-fix-add", "verify-fix-add"]
        );

        let flow_plan = build_fixture_plan_for_flow(
            &workspace,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
        )
        .unwrap();
        assert_eq!(
            flow_plan.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["investigate", "implement", "verify"]
        );

        let task_request =
            build_task_request(&workspace, "Fix the workspace", "session-1", None, None).unwrap();
        assert_eq!(task_request.input["execution_profile"], json!("fixture-profile"));
        assert_eq!(task_request.input["flow"], json!("workspace_execution"));

        let runtime = build_fixture_runtime(&workspace).unwrap();
        assert!(runtime.agents.get("analyzer").is_some());
        assert!(runtime.agents.get("coder").is_some());
        assert!(runtime.tools.get("tester").is_some());

        let goal_plan_runtime = build_fixture_runtime_for_goal_plan(
            &workspace,
            &GoalPlan::new(
                "Fix the workspace",
                vec![PlannedTask {
                    task_id: "planned-task-1".to_string(),
                    description: "Repair arithmetic".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some("tests pass".to_string()),
                    decision_type_hint: None,
                }],
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            goal_plan_runtime.profile.legacy_source.as_deref(),
            Some(super::NATIVE_GOAL_PLAN_LEGACY_SOURCE)
        );
        assert!(goal_plan_runtime.tools.get("replanner").is_some());

        let review_config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "ollama/qwen3:32b".to_string(),
                }),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        FileConfigStore::for_workspace(&workspace).save_local(&review_config).unwrap();
        let review_goal_plan_runtime = build_fixture_runtime_for_goal_plan(
            &workspace,
            &GoalPlan::new(
                "Fix the workspace",
                vec![PlannedTask {
                    task_id: "planned-task-2".to_string(),
                    description: "Repair arithmetic".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some("tests pass".to_string()),
                    decision_type_hint: None,
                }],
            )
            .unwrap(),
        )
        .unwrap();
        let review = review_goal_plan_runtime.profile.review.as_ref().unwrap();
        assert_eq!(review.reviewers.len(), 1);
        assert_eq!(review.reviewers[0].reviewer_id, super::NATIVE_GOAL_PLAN_PROVIDER_REVIEWER_ID);
        assert_eq!(review.reviewers[0].source.as_deref(), Some("copilot/ollama/qwen3:32b"));

        let analysis = analyze_workspace_fixture(&workspace, &profile, request(json!({}), 1));
        assert_eq!(analysis.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            analysis.output.as_ref().unwrap()["analysis_targets"].as_array().unwrap().len(),
            2
        );

        let invalid_attempt = verify_workspace_fixture(
            &workspace,
            &profile,
            request(json!({"attempt_index": 99}), 1),
        );
        assert_eq!(invalid_attempt.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(invalid_attempt.recoverability, Recoverability::Terminal);

        let command_output = run_fixture_command(
            &workspace,
            &super::FixtureCommand { program: "true".to_string(), args: vec![] },
        )
        .unwrap();
        assert!(command_output.succeeded());
        assert_eq!(command_output.rendered_command(), "true");
        assert_eq!(command_output.details()["command"], json!("true"));
    }

    #[test]
    fn build_task_request_copies_routing_projection_into_initial_context() {
        let workspace = write_execution_workspace(
            "boundline-fixture-routing-state",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        fs::write(
            execution_manifest_path(&workspace),
            serde_json::to_string_pretty(&profile).unwrap(),
        )
        .unwrap();
        let config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Claude,
                    model: "sonnet-4".to_string(),
                }),
                reviewer_roles: std::collections::BTreeMap::from([
                    (
                        "safety".to_string(),
                        ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4.1".to_string() },
                    ),
                    (
                        "maintainability".to_string(),
                        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
                    ),
                ]),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

        let task_request =
            build_task_request(&workspace, "Fix the workspace", "session-1", None, None).unwrap();
        let initial_context = task_request.initial_context.as_ref().unwrap();
        assert_eq!(initial_context.get("goal"), Some(&json!("Fix the workspace")));
        let projection = initial_context.get("routing_projection").unwrap();
        let effective_routing = projection["effective_routing"].as_array().unwrap();

        assert!(
            effective_routing
                .iter()
                .any(|value| { value.as_str() == Some("review=claude/sonnet-4 [workspace]") })
        );
        assert!(effective_routing.iter().any(|value| {
            value.as_str() == Some("reviewer:safety=copilot/gpt-4.1 [workspace]")
        }));
    }

    #[test]
    fn provider_review_routes_for_profile_require_explicit_namespaces() {
        let workspace = write_execution_workspace(
            "boundline-provider-review-routes",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let generic_config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-5.4".to_string(),
                }),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        FileConfigStore::for_workspace(&workspace).save_local(&generic_config).unwrap();
        let generic_routes = provider_review_routes_for_profile(&workspace, &profile);
        assert!(generic_routes.is_empty());

        let explicit_config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-5.4".to_string(),
                }),
                reviewer_roles: std::collections::BTreeMap::from([(
                    "safety".to_string(),
                    ModelRoute {
                        runtime: RuntimeKind::Copilot,
                        model: "ollama/qwen3:32b".to_string(),
                    },
                )]),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        FileConfigStore::for_workspace(&workspace).save_local(&explicit_config).unwrap();
        let explicit_routes = provider_review_routes_for_profile(&workspace, &profile);
        assert_eq!(explicit_routes.len(), 1);
        assert_eq!(
            explicit_routes.get("safety").map(|route| route.model.as_str()),
            Some("ollama/qwen3:32b")
        );
        assert!(!explicit_routes.contains_key("maintainability"));
    }

    #[test]
    fn fixture_analysis_surfaces_bounded_governance_context_for_flow_steps() {
        let workspace = write_execution_workspace(
            "boundline-fixture-governance-context",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let mut state = Map::new();
        state.insert(
            "latest_governance_stage".to_string(),
            serde_json::to_value(GovernedStageRecord {
                stage_key: "bug-fix:investigate".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: GovernanceLifecycleState::GovernedReady,
                required: false,
                autopilot_enabled: false,
                approval_state: ApprovalState::NotNeeded,
                canon_run_ref: Some("canon-run-3".to_string()),
                governance_attempt_id: "attempt-3".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: Some(".canon/runs/canon-run-3".to_string()),
                decision_ref: None,
                stage_council: None,
                blocked_reason: None,
            })
            .unwrap(),
        );
        state.insert(
            "latest_governance_packet".to_string(),
            serde_json::to_value(GovernedStagePacket {
                packet_ref: ".canon/runs/canon-run-3".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                canon_mode: Some(CanonMode::Discovery),
                expected_document_refs: vec![".canon/runs/canon-run-3/discovery.md".to_string()],
                document_refs: vec![".canon/runs/canon-run-3/discovery.md".to_string()],
                readiness: PacketReadiness::Reusable,
                missing_sections: Vec::new(),
                headline: "investigation packet ready".to_string(),
                reason_code: None,
                authority_governance: None,
                adaptive_governance: None,
                semantic_descriptor: None,
            })
            .unwrap(),
        );
        let input = attach_stage_metadata(
            json!({
                "phase": "implement",
            }),
            built_in_flow("bug-fix").unwrap(),
            1,
        )
        .unwrap();

        let result =
            analyze_workspace_fixture(&workspace, &profile, request_with_state(input, 1, state));
        let governance_context = &result.output.as_ref().unwrap()["governance_context"];

        assert_eq!(
            governance_context["reused_packets"][0]["stage_key"],
            json!("bug-fix:investigate")
        );
        assert_eq!(
            governance_context["reused_packets"][0]["packet_ref"],
            json!(".canon/runs/canon-run-3")
        );
        assert_eq!(
            governance_context["reuse_binding"]["binding_reason"],
            json!("upstream_stage_context")
        );
    }

    #[test]
    fn execution_profile_loader_prefers_the_new_manifest_when_present() {
        let workspace = temp_workspace();
        fs::write(
            workspace.join(".boundline/execution.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "preferred-profile",
                "read_targets": ["src/lib.rs"],
                "validation_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Apply the code fix",
                        "failure_mode": "terminal",
                        "changes": [
                            {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            workspace.join(".boundline/fixture.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "legacy-profile",
                "test_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "file_patches": [
                    {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let profile = load_workspace_execution_profile(&workspace).unwrap();

        assert_eq!(profile.name, "preferred-profile");
        assert_eq!(profile.legacy_source, None);
        assert_eq!(profile.attempts.len(), 1);
    }

    #[test]
    fn execution_profile_loader_requires_the_modern_manifest() {
        let workspace = temp_workspace();
        fs::write(
            workspace.join(".boundline/fixture.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "legacy-profile",
                "test_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "file_patches": [
                    {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let error = load_workspace_execution_profile(&workspace).unwrap_err();

        assert!(matches!(error, FixtureRuntimeError::MissingExecutionProfile(_)));
    }

    #[test]
    fn adaptive_helpers_cover_guidance_blockers_headlines_and_reasons() {
        let strong_guidance = ValidationGuidance {
            source: ValidationGuidanceSource::ValidationRecord,
            matched_paths: vec!["src/lib.rs".to_string()],
            matched_terms: vec!["threshold".to_string(), "boundary".to_string()],
            headline: "validation guided the next attempt toward src/lib.rs".to_string(),
            confidence: ValidationGuidanceConfidence::Strong,
        };
        let hinted_guidance = ValidationGuidance {
            source: ValidationGuidanceSource::FailureMessage,
            matched_paths: Vec::new(),
            matched_terms: vec!["hint".to_string()],
            headline: "fallback".to_string(),
            confidence: ValidationGuidanceConfidence::Hinted,
        };

        assert_eq!(
            adaptive_replan_blocker(None),
            Some(
                "adaptive planner exhausted bounded repair because validation evidence was absent"
                    .to_string()
            )
        );
        assert_eq!(
            adaptive_replan_blocker(Some(&hinted_guidance)),
            Some(
                "adaptive planner exhausted bounded repair because validation evidence was insufficient to justify another materially different candidate"
                    .to_string()
            )
        );
        assert_eq!(adaptive_replan_blocker(Some(&strong_guidance)), None);
        assert_eq!(
            adaptive_no_candidate_reason(Some(&strong_guidance)),
            "adaptive planner exhausted bounded repair because no remaining candidate stayed credible after validation pointed to src/lib.rs"
        );
        assert_eq!(
            adaptive_selection_headline(
                "src/lib.rs",
                AdaptiveChangeKind::OrderingBoundaryFlip,
                Some(&strong_guidance),
            ),
            "selected src/lib.rs via ordering_boundary_flip for adaptive delivery after validation guidance"
        );

        let guided_reason = adaptive_selection_reason(
            "src/lib.rs",
            AdaptiveChangeKind::OrderingBoundaryFlip,
            2,
            Some(&strong_guidance),
            &["boundary evidence stayed strongest".to_string()],
        );
        assert!(guided_reason.contains("validation pointed to src/lib.rs"));
        assert!(guided_reason.contains("boundary evidence stayed strongest"));

        let hinted_reason = adaptive_selection_reason(
            "src/lib.rs",
            AdaptiveChangeKind::NumericLiteralFlip,
            2,
            Some(&ValidationGuidance {
                matched_terms: vec!["literal".to_string(), "count".to_string()],
                ..hinted_guidance.clone()
            }),
            &[],
        );
        assert!(hinted_reason.contains("reprioritized the bounded slice"));
        assert!(hinted_reason.contains("it remained the most credible bounded candidate"));

        let unguided_reason = adaptive_selection_reason(
            "src/lib.rs",
            AdaptiveChangeKind::ArithmeticSwap,
            1,
            None,
            &[],
        );
        assert!(unguided_reason.contains("selected src/lib.rs via arithmetic_swap"));
    }

    #[test]
    fn adaptive_helpers_cover_path_matching_transition_rejections_and_failure_evidence() {
        let read_targets = vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()];
        assert_eq!(
            guidance_paths_from_text("lib.rs and red_to_green.rs still fail", &read_targets),
            vec!["src/lib.rs".to_string()]
        );

        let selected_targets = vec!["src/lib.rs".to_string()];
        let path_scores = vec![PathScore {
            path: "src/lib.rs".to_string(),
            score: 100,
            reasons: vec!["matched path preference src/".to_string()],
        }];
        let candidate = RankedAdaptiveCandidate {
            change_kind: AdaptiveChangeKind::OrderingBoundaryFlip,
            change: WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: " > ".to_string(),
                replace: " >= ".to_string(),
            },
            signature: "sig-ordering-boundary".to_string(),
            score: 130,
            order_index: 0,
            reasons: vec!["boundary mismatch remained strongest".to_string()],
        };

        assert_eq!(
            build_rejected_candidate_summaries(
                0,
                &selected_targets,
                &path_scores,
                2,
                &candidate,
            ),
            vec![
                "later bounded candidates were rejected because ordering_boundary_flip on src/lib.rs remained more credible"
                    .to_string()
            ]
        );
        assert!(
            build_rejected_candidate_summaries(1, &selected_targets, &path_scores, 2, &candidate,)
                .is_empty()
        );

        let previous = vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()];
        let narrowed = vec!["src/lib.rs".to_string()];
        let broadened = vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()];
        let replaced = vec!["src/helper.rs".to_string()];
        assert_eq!(adaptive_transition_kind(None, &narrowed), AttemptTransitionKind::Initial);
        assert_eq!(
            adaptive_transition_kind(Some(&previous), &narrowed),
            AttemptTransitionKind::Narrowed
        );
        assert_eq!(
            adaptive_transition_kind(Some(&narrowed), &broadened),
            AttemptTransitionKind::Broadened
        );
        assert_eq!(
            adaptive_transition_kind(Some(&narrowed), &replaced),
            AttemptTransitionKind::Replaced
        );

        let evidence = adaptive_failure_evidence(
            &json!({
                "selection_evidence": {
                    "candidate_family": "ordering_boundary_flip",
                    "reason": "selected src/lib.rs via ordering_boundary_flip"
                },
                "workspace_slice": {
                    "selection_id": "adaptive-slice-1",
                    "selected_targets": ["src/lib.rs"],
                    "scored_candidates": [],
                    "headline": "selected src/lib.rs via ordering_boundary_flip for adaptive delivery"
                },
                "attempt_lineage": {
                    "previous_attempt_id": "adaptive-attempt-1",
                    "current_attempt_id": "adaptive-attempt-2",
                    "transition_kind": "replaced",
                    "reason": "validation reprioritized the bounded slice"
                }
            }),
            &ValidationRecord {
                command: "cargo test --quiet".to_string(),
                exit_code: 101,
                stdout: String::new(),
                stderr: "threshold still fails".to_string(),
                succeeded: false,
            },
            Some("bounded recovery exhausted".to_string()),
        );

        assert_eq!(
            evidence["selection_evidence"]["candidate_family"],
            json!("ordering_boundary_flip")
        );
        assert_eq!(evidence["workspace_slice"]["selected_targets"], json!(["src/lib.rs"]));
        assert_eq!(evidence["attempt_lineage"]["current_attempt_id"], json!("adaptive-attempt-2"));
        assert_eq!(evidence["exhaustion_reason"], json!("bounded recovery exhausted"));
    }

    #[test]
    fn adaptive_helpers_cover_new_candidate_generators_and_family_scoring() {
        let ordering_candidates =
            ordering_boundary_flip_candidates("src/lib.rs", "if value <= 3 { return true; }");
        assert_eq!(ordering_candidates[0].find, " <= ");
        assert_eq!(ordering_candidates[0].replace, " < ");

        let result_error_candidates =
            result_status_flip_candidates("src/lib.rs", "fn load() -> Result<(), ()> { Err(()) }");
        assert_eq!(result_error_candidates[0].find, "Err(");
        assert_eq!(result_error_candidates[0].replace, "Ok(");

        let result_ok_candidates =
            result_status_flip_candidates("src/lib.rs", "fn load() -> Result<(), ()> { Ok(()) }");
        assert_eq!(result_ok_candidates[0].find, "Ok(");
        assert_eq!(result_ok_candidates[0].replace, "Err(");

        let numeric_candidates =
            numeric_literal_flip_candidates("src/lib.rs", "if count == 0 { return 1; }");
        assert_eq!(numeric_candidates[0].find, " == 0");
        assert_eq!(numeric_candidates[0].replace, " == 1");

        let path_score = PathScore {
            path: "src/lib.rs".to_string(),
            score: 70,
            reasons: vec!["matched path preference src/".to_string()],
        };
        let hinted_guidance = ValidationGuidance {
            source: ValidationGuidanceSource::FailureMessage,
            matched_paths: Vec::new(),
            matched_terms: vec!["result".to_string(), "error".to_string()],
            headline: "fallback".to_string(),
            confidence: ValidationGuidanceConfidence::Hinted,
        };
        let strong_guidance = ValidationGuidance {
            source: ValidationGuidanceSource::ValidationRecord,
            matched_paths: vec!["src/lib.rs".to_string()],
            matched_terms: vec!["zero".to_string(), "constant".to_string()],
            headline: "validation guided the next attempt toward src/lib.rs".to_string(),
            confidence: ValidationGuidanceConfidence::Strong,
        };

        let result_candidate = GeneratedAdaptiveCandidate {
            change_kind: AdaptiveChangeKind::ResultStatusFlip,
            change: result_error_candidates[0].clone(),
        };
        let (result_score, result_reasons) = score_adaptive_candidate(
            &path_score,
            &result_candidate,
            &[],
            &["result".to_string(), "error".to_string()],
            Some(&hinted_guidance),
        );
        assert!(result_score > path_score.score);
        assert!(result_reasons.iter().any(|reason| reason.contains("outcome-status mismatch")));
        assert!(
            result_reasons
                .iter()
                .any(|reason| reason.contains("validation hints supported the bounded replan"))
        );

        let numeric_candidate = GeneratedAdaptiveCandidate {
            change_kind: AdaptiveChangeKind::NumericLiteralFlip,
            change: numeric_candidates[0].clone(),
        };
        let (numeric_score, numeric_reasons) = score_adaptive_candidate(
            &path_score,
            &numeric_candidate,
            &["src".to_string()],
            &["zero".to_string(), "constant".to_string()],
            Some(&strong_guidance),
        );
        assert!(numeric_score > result_score);
        assert!(
            numeric_reasons
                .iter()
                .any(|reason| reason.contains("goal terms aligned with the candidate change"))
        );
        assert!(numeric_reasons.iter().any(|reason| reason.contains("numeric literal mismatch")));
        assert!(numeric_reasons.iter().any(|reason| reason.contains("bounded numeric literal")));
        assert!(numeric_reasons.iter().any(|reason| {
            reason.contains("strong validation guidance supported the bounded replan")
        }));
    }

    #[test]
    fn fixture_helper_functions_cover_versions_profiles_and_goal_plan_inference() {
        assert_eq!(fixture_next_minor_exclusive("1.2.3").unwrap(), "1.3.0");
        assert!(matches!(
            fixture_next_minor_exclusive("1.2"),
            Err(FixtureRuntimeError::InvalidReasoningFixtureVersion { .. })
        ));
        // Missing major (empty string) hits the major parse error branch.
        assert!(matches!(
            fixture_next_minor_exclusive(""),
            Err(FixtureRuntimeError::InvalidReasoningFixtureVersion { .. })
        ));
        // Non-numeric minor hits the minor parse error branch.
        assert!(matches!(
            fixture_next_minor_exclusive("1.abc.2"),
            Err(FixtureRuntimeError::InvalidReasoningFixtureVersion { .. })
        ));

        let independent_floor =
            fixture_minimum_independence(ReasoningProfileId::IndependentPairReview);
        assert!(independent_floor.route_distinct);
        assert!(independent_floor.provider_distinct);
        assert_eq!(independent_floor.minimum_participants, 2);

        let reflexion_floor = fixture_minimum_independence(ReasoningProfileId::BoundedReflexion);
        assert!(!reflexion_floor.route_distinct);
        assert!(!reflexion_floor.provider_distinct);
        assert_eq!(reflexion_floor.minimum_participants, 1);

        let reflexion_budget = fixture_reasoning_budget(ReasoningProfileId::BoundedReflexion);
        assert_eq!(reflexion_budget.max_reflexion_revisions, 1);
        let pair_budget = fixture_reasoning_budget(ReasoningProfileId::IndependentPairReview);
        assert_eq!(pair_budget.max_reflexion_revisions, 0);
        assert_eq!(pair_budget.max_participants, 2);

        assert_eq!(first_stable_line("\n\n  keep me  \n").as_deref(), Some("keep me"));

        let workspace = write_execution_workspace(
            "boundline-fixture-goal-plan-inference",
            "pub const STATUS: &str = \"todo\";\n",
        );
        let goal_plan = GoalPlan::new(
            "Summarize the workspace state",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Update the workspace summary marker".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("summary marker updated".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap();

        let (path, find, replace) = infer_goal_plan_change(&workspace, &goal_plan).unwrap();
        assert_eq!(path, "src/lib.rs");
        assert_eq!(find, "\"todo\"");
        assert_eq!(replace, "\"workspace summary ready\"");

        let synthesized = synthesize_goal_plan_execution_profile(&workspace, &goal_plan).unwrap();
        assert!(synthesized.read_targets.iter().any(|target| target == "src/lib.rs"));
        assert!(synthesized.read_targets.iter().any(|target| target == "Cargo.toml"));
        assert_eq!(synthesized.validation_command.program, "cargo");
        assert_eq!(
            synthesized.validation_command.args,
            vec!["test".to_string(), "--quiet".to_string()]
        );

        assert!(matches!(
            resolve_supported_fixture_flow("unknown-flow", "fixture planning"),
            Err(FixtureRuntimeError::UnsupportedFixtureFlow { .. })
        ));

        // A task whose target file does not exist forces infer_goal_plan_change
        // through the Cargo.toml branch (lines 899-912).
        let cargo_toml_plan = GoalPlan::new(
            "Derive from Cargo.toml",
            vec![PlannedTask {
                task_id: "planned-task-cargo".to_string(),
                description: "Nonexistent target falls through to Cargo.toml".to_string(),
                target: "nonexistent.rs".to_string(),
                expected_outcome: None,
                decision_type_hint: None,
            }],
        )
        .unwrap();
        let (cargo_path, cargo_find, _cargo_replace) =
            infer_goal_plan_change(&workspace, &cargo_toml_plan).unwrap();
        assert_eq!(cargo_path, "Cargo.toml");
        assert!(cargo_find.starts_with("__boundline_goal_plan_change_required__:"));

        fs::remove_dir_all(&workspace).unwrap();
    }

    #[test]
    fn resolve_effective_persona_prefers_latest_governance_packet() {
        let workspace = temp_workspace();
        FileConfigStore::for_workspace(&workspace)
            .save_local(&ConfigFile {
                version: 1,
                routing: RoutingConfig::default(),
                canon: Some(crate::domain::configuration::CanonPreferences {
                    mode_selection:
                        crate::domain::governance::CanonModeSelectionPreference::AutoConfirm,
                    default_owner: Some("platform".to_string()),
                    default_risk: None,
                    default_zone: None,
                    default_system_context: None,
                }),
                adapter: None,
                capability_provider: None,
            })
            .unwrap();

        let mut task_snapshot = TaskContext::new(
            "session-runtime",
            workspace.to_string_lossy(),
            RunLimits::default(),
            Map::new(),
        );
        task_snapshot
            .set_latest_governance_packet(&GovernedStagePacket {
                packet_ref: ".canon/runs/canon-run-1".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                canon_mode: Some(CanonMode::Implementation),
                expected_document_refs: vec![
                    ".canon/runs/canon-run-1/implementation.md".to_string(),
                ],
                document_refs: vec![".canon/runs/canon-run-1/implementation.md".to_string()],
                readiness: PacketReadiness::Reusable,
                missing_sections: Vec::new(),
                headline: "implementation packet ready".to_string(),
                reason_code: None,
                authority_governance: Some(CanonAuthorityGovernanceV1Envelope {
                    contract_line: "authority-governance-v1".to_string(),
                    authority_zone: CanonAuthorityZone::Green,
                    change_class: CanonChangeClass::LowImpact,
                    intended_persona: CanonIntendedPersona::SystemArchitect,
                    approval_state: ApprovalState::NotNeeded,
                    packet_readiness: PacketReadiness::Reusable,
                    risk: CanonRiskClass::LowImpact,
                    persona_anti_behaviors: Vec::new(),
                    primary_artifact: None,
                    artifact_order: Vec::new(),
                    promotion_refs: Vec::new(),
                    stage_role_hints: Vec::new(),
                }),
                adaptive_governance: None,
                semantic_descriptor: None,
            })
            .unwrap();

        let request = StepExecutionRequest {
            step_id: "implement".to_string(),
            step_kind: StepKind::Agent,
            target_name: "coder".to_string(),
            input: json!({}),
            task_snapshot,
            attempt_number: 1,
        };

        assert_eq!(
            resolve_effective_persona(&workspace, &request).as_deref(),
            Some("system-architect")
        );
    }

    #[test]
    fn resolve_effective_persona_defaults_to_delivery_engineer_for_invalid_owner() {
        let workspace = temp_workspace();
        FileConfigStore::for_workspace(&workspace)
            .save_local(&ConfigFile {
                version: 1,
                routing: RoutingConfig::default(),
                canon: Some(crate::domain::configuration::CanonPreferences {
                    mode_selection:
                        crate::domain::governance::CanonModeSelectionPreference::AutoConfirm,
                    default_owner: Some("not-a-real-persona".to_string()),
                    default_risk: None,
                    default_zone: None,
                    default_system_context: None,
                }),
                adapter: None,
                capability_provider: None,
            })
            .unwrap();

        let request = StepExecutionRequest {
            step_id: "implement".to_string(),
            step_kind: StepKind::Agent,
            target_name: "coder".to_string(),
            input: json!({}),
            task_snapshot: TaskContext::new(
                "session-runtime",
                workspace.to_string_lossy(),
                RunLimits::default(),
                Map::new(),
            ),
            attempt_number: 1,
        };

        assert_eq!(
            resolve_effective_persona(&workspace, &request).as_deref(),
            Some("delivery-engineer")
        );
    }

    #[test]
    fn resolve_phase_guidance_uses_goal_targets_and_authored_inputs() {
        let workspace = temp_workspace();
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"fixture-guidance\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .unwrap();
        fs::write(
            workspace.join("brief.md"),
            "Fix the failing Rust tests for src/lib.rs and preserve zero-panic handling.\n",
        )
        .unwrap();

        let authored_brief = normalize_inputs(
            &workspace,
            Some("Fix the failing Rust tests for src/lib.rs"),
            &[PathBuf::from("brief.md")],
        )
        .unwrap();
        let request = StepExecutionRequest {
            step_id: "implement".to_string(),
            step_kind: StepKind::Agent,
            target_name: "coder".to_string(),
            input: json!({
                "authored_brief": authored_brief,
            }),
            task_snapshot: TaskContext::new(
                "session-runtime",
                workspace.to_string_lossy(),
                RunLimits::default(),
                Map::new(),
            ),
            attempt_number: 1,
        };
        let sources = vec![WorkspaceTargetSource {
            path: "src/lib.rs".to_string(),
            contents: "pub fn add(left: i32, right: i32) -> i32 { left + right }".to_string(),
        }];

        let guidance = resolve_phase_guidance(
            &workspace,
            CapabilityPhase::Implementation,
            "Fix the failing Rust tests for src/lib.rs",
            &request,
            &sources,
        );

        assert!(
            guidance.iter().any(|entry| entry.contains("Rust") || entry.contains("cargo test")),
            "{guidance:?}"
        );
    }
}
