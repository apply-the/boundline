pub mod adapters;
pub mod cli;
pub mod domain;
pub mod fixture;
pub mod orchestrator;
pub mod registry;

pub use adapters::agent::FnAgentAdapter;
pub use adapters::governance_runtime::{
    CanonCliRuntime, GovernanceBoundedContext, GovernanceInputDocument, GovernanceRequestKind,
    GovernanceRuntime, GovernanceRuntimeError, GovernanceRuntimeRequest, GovernanceRuntimeResponse,
    LocalGovernanceRuntime, ReusedPacketInput,
};
pub use adapters::tool::FnToolAdapter;
pub use adapters::trace_store::FileTraceStore;
pub use domain::brief::{
    AuthoredBriefBundle, BriefIngestionError, GovernanceIntent, InputSourceKind,
    InputSourceReference, MAX_BRIEF_BYTES, MAX_BRIEF_SOURCES, normalize_governance_intent,
    normalize_inputs as normalize_brief_inputs, normalize_inputs_with_governance,
};
pub use domain::execution::{
    ChangeEvidence, ChangeStatus, ExecutionAttemptDefinition, ExecutionCommand,
    ExecutionFailureMode, ValidationRecord, WorkspaceChange, WorkspaceExecutionProfile,
};
pub use domain::flow::{SessionFlowState, built_in_flow, supported_flow_names};
pub use domain::governance::{
    ApprovalState, AutopilotAction, AutopilotDecisionRecord, CanonMode, CanonRuntimeConfig,
    GovernanceLifecycleState, GovernanceProfile, GovernanceProfileError, GovernanceRuntimeKind,
    GovernedStagePacket, GovernedStageRecord, PacketReadiness, PacketReuseBinding,
    StageGovernancePolicy, SystemContextBinding, autopilot_action_text, candidate_canon_modes,
    classify_packet_readiness, resolved_canon_mode, supported_canon_modes_for_stage,
};
pub use domain::limits::{RunLimits, TerminalCondition};
pub use domain::plan::Plan;
pub use domain::review::{
    AdjudicationDefinition, ReviewOutcome, ReviewProfile, ReviewScenario, ReviewTrigger,
    ReviewerDefinition, ReviewerDisposition, ReviewerFinding, ReviewerParticipation,
    ReviewerParticipationStatus, VoteDecision, VoteResolution, VoteRuleDefinition, VoteStrategy,
};
pub use domain::step::{
    ErrorInfo, Recoverability, Step, StepExecutionRequest, StepExecutionResult, StepKind,
    StepStatus,
};
pub use domain::task::{TaskRunRequest, TaskRunResponse, TaskStatus, TerminalReason};
pub use orchestrator::governance::{
    GovernanceStatePatchError, GovernanceStateSelectionError, GovernanceStepDecision,
    bounded_governance_context, bounded_reused_packets, build_autopilot_decision,
    escalation_target_stage_key, governance_stage_key, governance_state_patch,
    narrowed_bounded_context, runtime_command_available, select_packet_reuse_binding,
    selected_stage_policy,
};
pub use orchestrator::planner::{Planner, StaticPlanner};
pub use orchestrator::{Orchestrator, OrchestratorError};
pub use registry::agent_registry::AgentRegistry;
pub use registry::tool_registry::ToolRegistry;
