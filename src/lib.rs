pub mod assistant_plugin_validation;

pub use boundline_adapters::adapters;
pub use boundline_adapters::fixture;
pub use boundline_adapters::orchestrator;
pub use boundline_adapters::registry;
pub use boundline_cli::cli;
pub use boundline_core::domain;

pub use adapters::agent::FnAgentAdapter;
pub use adapters::config_store::{ConfigStoreError, FileConfigStore};
pub use adapters::governance_runtime::{
    CanonCliRuntime, GovernanceBoundedContext, GovernanceInputDocument, GovernanceRequestKind,
    GovernanceRuntime, GovernanceRuntimeError, GovernanceRuntimeRequest, GovernanceRuntimeResponse,
    LocalGovernanceRuntime, ReusedPacketInput,
};
pub use adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
pub use adapters::tool::FnToolAdapter;
pub use adapters::trace_store::FileTraceStore;
pub use boundline_adapters::orchestrator::SessionRuntimeError;
pub use domain::brief::{
    AuthoredBriefBundle, BriefIngestionError, GovernanceIntent, InputSourceKind,
    InputSourceReference, MAX_BRIEF_BYTES, MAX_BRIEF_SOURCES, normalize_governance_intent,
    normalize_inputs as normalize_brief_inputs, normalize_inputs_with_governance,
};
pub use domain::cluster::{
    ClusterAuthorityKind, ClusterConfigFile, ClusterDeliveryStory, ClusterError,
    ClusterFollowUpAuthority, ClusterInspectReport, ClusterMemberRegistration, ClusterMemberRole,
    ClusterMemberState, ClusterMemberStatusView, ClusterRouteOwner, ClusterSessionProjection,
    ClusteredExecutionCondition, ClusteredExecutionKind, WorkspaceCluster,
    WorkspaceParticipationKind, WorkspaceParticipationRecord,
};
pub use domain::configuration::{
    ConfigFile, ConfigShowScope, ConfigWriteScope, EffectiveRouting, InitTemplate, ModelRoute,
    RouteSlot, RoutingConfig, RoutingOverrides, RuntimeKind, SourcedRoute, ValueSource,
    resolve_effective_routing,
};
pub use domain::distribution::{
    CompanionState, DistributionChannel, SUPPORTED_CANON_VERSION, evaluate_canon_install,
    supported_distribution_channels,
};
pub use domain::execution::{
    ChangeEvidence, ChangeStatus, ExecutionAttemptDefinition, ExecutionCommand,
    ExecutionFailureMode, ValidationRecord, WorkspaceChange, WorkspaceExecutionProfile,
};
pub use domain::flow::{SessionFlowState, built_in_flow, supported_flow_names};
pub use domain::follow_through::FollowThroughProjection;
pub use domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan,
};
pub use domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, CanonMode, CanonRuntimeConfig,
    GovernanceLifecycleState, GovernanceProfile, GovernanceProfileError, GovernanceRuntimeKind,
    GovernedStagePacket, GovernedStageRecord, PacketReadiness, PacketReuseBinding,
    StageGovernancePolicy, SystemContextBinding, autopilot_action_text, candidate_canon_modes,
    classify_packet_readiness, resolved_canon_mode, supported_canon_modes_for_stage,
};
pub use domain::limits::{RunLimits, TerminalCondition};
pub use domain::negotiation::{
    AcceptanceBoundary, NegotiatedDeliveryPacket, NegotiationConstraint, NegotiationConstraintKind,
    NegotiationConstraintSource, NegotiationConstraintState, NegotiationResolutionState,
    TradeoffDecision,
};
pub use domain::plan::Plan;
pub use domain::review::{
    AdjudicationDefinition, ReviewOutcome, ReviewProfile, ReviewScenario, ReviewTrigger,
    ReviewerDefinition, ReviewerDisposition, ReviewerFinding, ReviewerParticipation,
    ReviewerParticipationStatus, VoteDecision, VoteResolution, VoteRuleDefinition, VoteStrategy,
};
pub use domain::routing_decision::RoutingDecisionProjection;
pub use domain::step::{
    ErrorInfo, Recoverability, Step, StepExecutionRequest, StepExecutionResult, StepKind,
    StepStatus,
};
pub use domain::task::{TaskRunRequest, TaskRunResponse, TaskStatus, TerminalReason};
pub use domain::workflow::{
    ConditionalWorkflowPhase, WorkflowAvailabilityState, WorkflowConditionKind, WorkflowDefinition,
    WorkflowDefinitionError, WorkflowDiscoveryEntry, WorkflowGoalSource, WorkflowLifecycleState,
    WorkflowOutputPreferences, WorkflowPhase, WorkflowProgressState, WorkflowRegistry,
};
pub use domain::workspace_hygiene::{
    HygieneFilePlan, HygieneMergeResult, HygienePatternPack, merge_hygiene_content,
    plan_hygiene_defaults,
};
pub use orchestrator::governance::{
    GovernanceStatePatchError, GovernanceStateSelectionError, GovernanceStepDecision,
    append_governed_document_to_lifecycle, bounded_governance_context, bounded_reused_packets,
    build_autopilot_decision, clarification_prompt_from_response,
    enrich_bounded_context_with_accumulated, escalation_target_stage_key, governance_stage_key,
    governance_state_patch, governed_document_ref_from_response, is_awaiting_approval_response,
    lifecycle_requires_refresh, narrowed_bounded_context, runtime_command_available,
    select_packet_reuse_binding, selected_stage_policy, set_lifecycle_awaiting_approval,
};
pub use orchestrator::planner::{Planner, StaticPlanner};
pub use orchestrator::{Orchestrator, OrchestratorError, SessionRuntime};
pub use registry::agent_registry::AgentRegistry;
pub use registry::tool_registry::ToolRegistry;
