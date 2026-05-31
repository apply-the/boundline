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
