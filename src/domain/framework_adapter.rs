//! Shared vocabulary for external framework-adapter domain records.

use serde::{Deserialize, Serialize};

/// Stable protocol line identifier for the initial framework-adapter contract.
pub const FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1: &str = "framework-adapter-v1";

/// Operator-selected activation mode for the workspace adapter slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterSelectionMode {
    /// No external adapter is selected; built-in behavior remains active.
    None,
    /// A host-shipped known profile is selected.
    KnownProfile,
    /// A fully custom external adapter command is selected.
    Custom,
}

/// Origin that wrote or migrated the adapter selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterRegistrationSource {
    /// Selection created by `boundline adapter add`.
    AdapterAdd,
    /// Selection created during `boundline init`.
    Init,
    /// Selection migrated from an older config shape.
    ConfigMigration,
}

/// Discovery outcome recorded for the selected adapter command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterDiscoveryState {
    /// The operator explicitly provided the command or path.
    ExplicitCommand,
    /// The host found a candidate binary on `PATH` and used it only after explicit selection.
    DiscoveredOnPath,
    /// The command could not be resolved yet.
    Unresolved,
}

/// Typed value kinds used by adapter config fields and stored values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterValueKind {
    /// Free-form string value.
    String,
    /// Filesystem path stored as a string ref.
    Path,
    /// Boolean value.
    Boolean,
    /// Integer value.
    Integer,
    /// Bounded string chosen from a known set.
    Enum,
}

/// Source that resolved one stored adapter config value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterValueSource {
    /// Guided prompt answered by the operator.
    OperatorPrompt,
    /// CLI flag or explicit `--set` value.
    CliFlag,
    /// Host-known default for a selected profile.
    KnownProfileDefault,
    /// Value carried forward from an older config shape.
    MigratedConfig,
}

/// Resolution state for one stored adapter config value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredAdapterConfigValueState {
    /// A valid value is present.
    Present,
    /// The value is missing.
    Missing,
    /// The value exists but is invalid for the current schema.
    Invalid,
}

/// Overall completeness of the stored adapter config set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterConfigCompletenessState {
    /// All required fields are valid.
    Complete,
    /// One or more required fields are still missing.
    MissingRequired,
    /// One or more stored values are invalid.
    Invalid,
}

/// Host-known lifecycle stages available for adapter claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterLifecycleStageKey {
    /// Goal-definition stage.
    Goal,
    /// Planning stage.
    Plan,
    /// Execution stage.
    Run,
    /// Review stage.
    Review,
}

impl AdapterLifecycleStageKey {
    /// Returns the stable serialized identifier for the lifecycle stage key.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Goal => "goal",
            Self::Plan => "plan",
            Self::Run => "run",
            Self::Review => "review",
        }
    }
}

/// Host-known hook identifiers available to adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterHookKey {
    /// Fired after one stage completes successfully.
    StageCompleted,
    /// Fired after one stage fails.
    StageFailed,
}

impl AdapterHookKey {
    /// Returns the stable serialized identifier for the hook key.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StageCompleted => "stage_completed",
            Self::StageFailed => "stage_failed",
        }
    }
}

/// Validation state of one adapter capability snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterCapabilitySnapshotState {
    /// Capability metadata validated successfully.
    Validated,
    /// Capability metadata blocked activation before stage ownership.
    Blocked,
    /// Capability metadata was malformed.
    InvalidManifest,
    /// Capability metadata was syntactically valid but incompatible.
    Incompatible,
}

/// Host-owned compatibility result for one adapter snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompatibilityState {
    /// Protocol line and version range are compatible.
    Compatible,
    /// Protocol line or version range is incompatible.
    Incompatible,
    /// Boundline version is outside the adapter-supported range.
    UnsupportedBoundline,
}

/// Effective stage execution source recorded by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterExecutionSource {
    /// Built-in Canon-aware behavior handled the stage.
    BuiltIn,
    /// The external adapter handled the stage.
    Adapter,
}

impl AdapterExecutionSource {
    /// Returns the stable serialized identifier for the execution source.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BuiltIn => "built_in",
            Self::Adapter => "adapter",
        }
    }
}

/// Host reason for routing a stage to built-in or adapter execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageRoutingDecisionReason {
    /// No adapter was selected for the workspace.
    NoAdapterSelected,
    /// The selected adapter did not declare this stage.
    UndeclaredStage,
    /// The selected adapter explicitly declared this stage.
    DeclaredOverride,
    /// Preflight blocked the adapter before stage claim.
    PreflightBlocked,
    /// The capability snapshot was invalid.
    InvalidManifest,
    /// Compatibility validation blocked the adapter.
    CompatibilityBlocked,
}

impl StageRoutingDecisionReason {
    /// Returns the stable serialized identifier for the routing decision reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoAdapterSelected => "no_adapter_selected",
            Self::UndeclaredStage => "undeclared_stage",
            Self::DeclaredOverride => "declared_override",
            Self::PreflightBlocked => "preflight_blocked",
            Self::InvalidManifest => "invalid_manifest",
            Self::CompatibilityBlocked => "compatibility_blocked",
        }
    }
}

/// Claim state recorded for one routed stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageClaimState {
    /// The adapter never claimed the stage.
    NotClaimed,
    /// The adapter claimed the stage and execution started.
    Claimed,
    /// The claimed stage completed successfully.
    Completed,
    /// The adapter failed after claiming the stage.
    FailedAfterClaim,
}

impl StageClaimState {
    /// Returns the stable serialized identifier for the claim state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotClaimed => "not_claimed",
            Self::Claimed => "claimed",
            Self::Completed => "completed",
            Self::FailedAfterClaim => "failed_after_claim",
        }
    }
}

/// Failure classes surfaced in adapter execution records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterFailureClass {
    /// Failure happened during preflight validation.
    Preflight,
    /// Failure happened while parsing or validating the manifest.
    Manifest,
    /// Failure was caused by missing required config.
    MissingConfig,
    /// Failure occurred inside adapter-owned stage execution.
    AdapterRuntime,
    /// Failure was caused by protocol or version incompatibility.
    Compatibility,
    /// Failure occurred because the adapter returned a protocol-valid error envelope.
    ProtocolError,
    /// Failure occurred because the adapter exchange failed at the transport boundary.
    TransportFailure,
    /// Failure was limited to a non-owning hook warning.
    HookWarningOnly,
}

impl AdapterFailureClass {
    /// Returns the stable serialized identifier for the failure class.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Manifest => "manifest",
            Self::MissingConfig => "missing_config",
            Self::AdapterRuntime => "adapter_runtime",
            Self::Compatibility => "compatibility",
            Self::ProtocolError => "protocol_error",
            Self::TransportFailure => "transport_failure",
            Self::HookWarningOnly => "hook_warning_only",
        }
    }
}

/// Terminal status recorded for one lifecycle stage execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleStageExecutionStatus {
    /// The stage succeeded.
    Succeeded,
    /// The stage failed.
    Failed,
    /// The stage blocked awaiting operator action.
    Blocked,
    /// The stage did not run.
    Skipped,
}

impl LifecycleStageExecutionStatus {
    /// Returns the stable serialized identifier for the stage execution status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Skipped => "skipped",
        }
    }
}

/// Final readiness posture recorded for adapter-owned planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningReadinessStatus {
    /// Planning is ready to continue.
    Ready,
    /// Planning remains blocked.
    Blocked,
}

impl PlanningReadinessStatus {
    /// Returns the stable serialized identifier for the readiness posture.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
        }
    }
}

/// Severity attached to one planning finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningFindingSeverity {
    /// The finding blocks plan completion.
    Blocking,
    /// The finding is advisory only.
    NonBlocking,
}

impl PlanningFindingSeverity {
    /// Returns the stable serialized identifier for the finding severity.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::NonBlocking => "non_blocking",
        }
    }
}

/// One typed planning finding preserved in session state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningFinding {
    /// Stable finding identifier.
    pub finding_id: String,
    /// Operator-facing summary.
    pub summary: String,
    /// Severity used by the planning gate.
    pub severity: PlanningFindingSeverity,
}

/// Skip reason recorded for one remediation task that did not execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningRemediationSkipReason {
    /// The remediation is outside the active feature scope.
    OutOfScope,
    /// The remediation would be unsafe to execute automatically.
    Unsafe,
    /// The remediation requires operator input.
    RequiresOperatorInput,
    /// The remediation is not deterministic enough for automatic execution.
    NonDeterministic,
    /// The remediation did not include an executable command.
    MissingCommand,
}

impl PlanningRemediationSkipReason {
    /// Returns the stable serialized identifier for the skip reason.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OutOfScope => "out_of_scope",
            Self::Unsafe => "unsafe",
            Self::RequiresOperatorInput => "requires_operator_input",
            Self::NonDeterministic => "non_deterministic",
            Self::MissingCommand => "missing_command",
        }
    }
}

/// Outcome record for one attempted or skipped remediation task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningRemediationTaskOutcome {
    /// Stable remediation task identifier.
    pub task_id: String,
    /// Operator-facing task summary.
    pub summary: String,
    /// Findings addressed by the task.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_ids: Vec<String>,
    /// Skip reason when the remediation did not run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<PlanningRemediationSkipReason>,
}

/// Final implementation posture recorded for adapter-owned run stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImplementationStatus {
    /// The implementation workflow completed.
    Completed,
    /// The implementation workflow blocked.
    Blocked,
    /// The implementation workflow failed.
    Failed,
}

impl ImplementationStatus {
    /// Returns the stable serialized identifier for the implementation posture.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

/// Optional adapter-owned stage detail payload persisted for status and inspect output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FrameworkAdapterStageOutcomeDetails {
    /// Workflow identifier executed for the stage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    /// Commands or bridge steps executed during the stage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executed_commands: Vec<String>,
    /// Planning findings surfaced by the adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planning_findings: Vec<PlanningFinding>,
    /// Remediation tasks attempted during planning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_tasks_attempted: Vec<PlanningRemediationTaskOutcome>,
    /// Remediation tasks completed successfully.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_tasks_completed: Vec<PlanningRemediationTaskOutcome>,
    /// Remediation tasks skipped with explicit reasons.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remediation_tasks_skipped: Vec<PlanningRemediationTaskOutcome>,
    /// Blocking planning findings that remain unresolved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remaining_blocking_findings: Vec<PlanningFinding>,
    /// Final planning-readiness posture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_planning_readiness_status: Option<PlanningReadinessStatus>,
    /// Number of analyze passes observed during planning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analyze_pass_count: Option<usize>,
    /// Number of remediation cycles used during planning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remediation_cycles_used: Option<usize>,
    /// Final implementation-stage posture for adapter-owned `run`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_status: Option<ImplementationStatus>,
    /// Validation or evidence refs reported by the adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_refs: Vec<String>,
}

impl FrameworkAdapterStageOutcomeDetails {
    /// Returns whether the detail payload contains any persisted adapter-owned data.
    pub fn is_empty(&self) -> bool {
        self.workflow_id.is_none()
            && self.executed_commands.is_empty()
            && self.planning_findings.is_empty()
            && self.remediation_tasks_attempted.is_empty()
            && self.remediation_tasks_completed.is_empty()
            && self.remediation_tasks_skipped.is_empty()
            && self.remaining_blocking_findings.is_empty()
            && self.final_planning_readiness_status.is_none()
            && self.analyze_pass_count.is_none()
            && self.remediation_cycles_used.is_none()
            && self.implementation_status.is_none()
            && self.validation_refs.is_empty()
    }
}

/// Result of one hook dispatch attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookDispatchStatus {
    /// The hook was delivered successfully.
    Delivered,
    /// The hook was ignored because it was not active.
    Ignored,
    /// The hook completed with warning-only semantics.
    Warning,
    /// The hook failed.
    Failed,
}

impl HookDispatchStatus {
    /// Returns the stable serialized identifier for the hook dispatch status.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Ignored => "ignored",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}
