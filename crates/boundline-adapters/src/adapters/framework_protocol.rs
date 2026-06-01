//! Typed JSON payloads for the framework-adapter subprocess protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::orchestrator::framework_catalog::{
    FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, FrameworkHookKey, FrameworkStageKey,
};

/// Supported one-shot commands in the initial framework-adapter protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrameworkAdapterCommand {
    /// Returns adapter capabilities and the declared config schema.
    Describe,
    /// Validates a proposed config set before any stage claim occurs.
    Preflight,
    /// Executes one claimed lifecycle stage.
    ExecuteStage,
    /// Delivers one observable hook event to the adapter.
    EmitHook,
}

impl FrameworkAdapterCommand {
    /// Returns the stable command token used by the adapter binary.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Describe => "describe",
            Self::Preflight => "preflight",
            Self::ExecuteStage => "execute-stage",
            Self::EmitHook => "emit-hook",
        }
    }
}

/// Supported transport families the adapter may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterTransportKind {
    /// One-shot stdio transport.
    Stdio,
}

impl FrameworkAdapterTransportKind {
    /// Returns the stable transport token used on the wire.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
        }
    }
}

/// Supported request and response encodings the adapter may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterTransportEncoding {
    /// UTF-8 JSON payloads.
    Json,
}

impl FrameworkAdapterTransportEncoding {
    /// Returns the stable encoding token used on the wire.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
}

/// Supported channels used by a declared transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterTransportChannel {
    /// Standard input.
    Stdin,
    /// Standard output.
    Stdout,
}

impl FrameworkAdapterTransportChannel {
    /// Returns the stable channel token used on the wire.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdin => "stdin",
            Self::Stdout => "stdout",
        }
    }
}

/// One declared transport supported by the adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterTransportDescriptor {
    /// Transport family.
    pub transport: FrameworkAdapterTransportKind,
    /// Message encoding.
    pub encoding: FrameworkAdapterTransportEncoding,
    /// Channel used for requests.
    pub request_channel: FrameworkAdapterTransportChannel,
    /// Channel used for responses.
    pub response_channel: FrameworkAdapterTransportChannel,
}

impl FrameworkAdapterTransportDescriptor {
    /// Returns the v1-supported stdio JSON transport declaration.
    pub const fn stdio_json() -> Self {
        Self {
            transport: FrameworkAdapterTransportKind::Stdio,
            encoding: FrameworkAdapterTransportEncoding::Json,
            request_channel: FrameworkAdapterTransportChannel::Stdin,
            response_channel: FrameworkAdapterTransportChannel::Stdout,
        }
    }

    /// Returns whether this declaration matches the host-supported v1 stdio contract.
    pub const fn is_v1_compatible(&self) -> bool {
        matches!(
            (self.transport, self.encoding, self.request_channel, self.response_channel,),
            (
                FrameworkAdapterTransportKind::Stdio,
                FrameworkAdapterTransportEncoding::Json,
                FrameworkAdapterTransportChannel::Stdin,
                FrameworkAdapterTransportChannel::Stdout,
            )
        )
    }

    /// Renders one operator-facing transport summary tuple.
    pub fn summary(&self) -> String {
        format!(
            "{}/{}/{}->{}",
            self.transport.as_str(),
            self.encoding.as_str(),
            self.request_channel.as_str(),
            self.response_channel.as_str(),
        )
    }
}

/// Returns whether the declared transports include the v1-supported stdio JSON path.
pub fn framework_adapter_supports_v1_transport(
    transports: &[FrameworkAdapterTransportDescriptor],
) -> bool {
    transports.iter().any(FrameworkAdapterTransportDescriptor::is_v1_compatible)
}

/// Renders declared transports for operator-facing diagnostics.
pub fn format_framework_adapter_transports(
    transports: &[FrameworkAdapterTransportDescriptor],
) -> String {
    transports
        .iter()
        .map(FrameworkAdapterTransportDescriptor::summary)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Supported value kinds for adapter-declared configuration fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterFieldValueKind {
    /// Free-form string value.
    String,
    /// Path-like string value.
    Path,
    /// Boolean value.
    Boolean,
    /// Integer value.
    Integer,
    /// One string selected from a fixed bounded set.
    Enum,
}

/// Non-interactive behavior requested for one adapter-declared field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterRequiredFieldPolicy {
    /// Fail when the field is required and missing.
    Fail,
    /// Use the declared default when available.
    UseDefault,
    /// Skip the field when no adapter-owned work depends on it.
    SkipIfUnowned,
}

/// Failure classes surfaced by adapter stage execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterFailureClass {
    /// The adapter blocked during preflight validation.
    Preflight,
    /// The adapter emitted an invalid or incomplete manifest.
    Manifest,
    /// One or more required fields were missing.
    MissingConfig,
    /// The claimed stage failed during adapter execution.
    AdapterRuntime,
    /// The adapter is incompatible with the host protocol or version range.
    Compatibility,
}

/// Status values returned by the adapter preflight command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterPreflightStatus {
    /// The adapter may proceed to claimed stage execution.
    Ready,
    /// The adapter cannot proceed until the operator resolves a blocking issue.
    Blocked,
}

/// Blocking reasons returned during adapter preflight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterPreflightBlockReason {
    /// One or more required config fields are missing.
    MissingRequiredConfig,
    /// The adapter reported incompatible protocol metadata.
    IncompatibleProtocol,
    /// The adapter rejected the proposed config values.
    InvalidConfig,
    /// The adapter depends on a local resource that is unavailable.
    UnavailableResource,
}

/// Status values returned by a claimed stage execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterStageExecutionStatus {
    /// The claimed stage completed successfully.
    Succeeded,
    /// The claimed stage blocked and requires operator action.
    Blocked,
    /// The claimed stage failed and should surface failure semantics to the host.
    Failed,
}

/// Status values returned by hook delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameworkAdapterHookDeliveryStatus {
    /// The hook was delivered and processed successfully.
    Delivered,
    /// The hook was ignored because it is not active in the current run.
    Ignored,
    /// The hook completed with a warning-only outcome.
    Warning,
    /// The hook failed.
    Failed,
}

/// One adapter-declared required or optional configuration field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterConfigFieldDefinition {
    /// Stable field identifier.
    pub field_key: String,
    /// Operator-facing prompt label.
    pub display_label: String,
    /// Field value kind.
    pub value_kind: FrameworkAdapterFieldValueKind,
    /// Whether the field must exist before adapter-owned execution starts.
    pub required: bool,
    /// Whether the field must be redacted from operator-facing projections.
    pub secret: bool,
    /// Optional text default surfaced by a profile or adapter.
    pub default_value_text: Option<String>,
    /// Prompt copy shown during guided setup.
    pub prompt_text: String,
    /// Recovery guidance shown when setup blocks.
    pub help_text: String,
    /// Non-interactive behavior for this field.
    pub non_interactive_policy: FrameworkAdapterRequiredFieldPolicy,
}

/// One resolved adapter configuration value sent through the protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterConfigValue {
    /// Stable field identifier.
    pub field_key: String,
    /// Typed value kind for the stored payload.
    pub value_kind: FrameworkAdapterFieldValueKind,
    /// String or enum value when applicable.
    pub string_value: Option<String>,
    /// Path-like value when applicable.
    pub path_value: Option<String>,
    /// Boolean value when applicable.
    pub bool_value: Option<bool>,
    /// Integer value when applicable.
    pub int_value: Option<i64>,
}

/// Typed response emitted by `describe`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterDescribeResponse {
    /// Stable protocol line identifier.
    pub protocol_line: String,
    /// Stable adapter identity reported by the binary.
    pub adapter_id: String,
    /// Semantic version of the adapter build.
    pub adapter_version: String,
    /// Supported Boundline version range.
    pub supported_boundline_range: String,
    /// Explicit transports supported by this adapter build.
    #[serde(default)]
    pub supported_transports: Vec<FrameworkAdapterTransportDescriptor>,
    /// Host-known stages claimed by the adapter.
    pub declared_stage_overrides: Vec<FrameworkStageKey>,
    /// Host-known hooks observed by the adapter.
    pub declared_hook_subscriptions: Vec<FrameworkHookKey>,
    /// Required or optional config fields declared by the adapter.
    pub required_config_fields: Vec<FrameworkAdapterConfigFieldDefinition>,
}

impl FrameworkAdapterDescribeResponse {
    /// Returns a bootstrap-ready empty describe response for one adapter identity.
    pub fn bootstrap(adapter_id: impl Into<String>, adapter_version: impl Into<String>) -> Self {
        Self {
            protocol_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
            adapter_id: adapter_id.into(),
            adapter_version: adapter_version.into(),
            supported_boundline_range: String::new(),
            supported_transports: vec![FrameworkAdapterTransportDescriptor::stdio_json()],
            declared_stage_overrides: Vec::new(),
            declared_hook_subscriptions: Vec::new(),
            required_config_fields: Vec::new(),
        }
    }
}

/// Protocol-valid error payload returned inside a standard error envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameworkAdapterEnvelopeError {
    /// Stable machine-readable error code.
    pub code: String,
    /// Human-readable error summary.
    pub message: String,
    /// Optional structured details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

/// Standard success envelope used by every adapter response on stdout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameworkAdapterSuccessEnvelope<T> {
    /// Indicates this envelope contains a successful protocol exchange.
    pub success: bool,
    /// Command-specific response payload.
    pub data: T,
}

/// Standard error envelope used by every adapter response on stdout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameworkAdapterErrorEnvelope {
    /// Indicates this envelope contains a protocol-valid error outcome.
    pub success: bool,
    /// Structured error details.
    pub error: FrameworkAdapterEnvelopeError,
}

/// Standard stdout response envelope for all protocol commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameworkAdapterResponseEnvelope<T> {
    /// A successful protocol exchange.
    Success(FrameworkAdapterSuccessEnvelope<T>),
    /// A protocol-valid error exchange.
    Error(FrameworkAdapterErrorEnvelope),
}

impl<T> FrameworkAdapterResponseEnvelope<T> {
    /// Wraps a command payload in the standard success envelope.
    pub fn success(data: T) -> Self {
        Self::Success(FrameworkAdapterSuccessEnvelope { success: true, data })
    }

    /// Converts the parsed response envelope into a command payload or a
    /// protocol-level envelope error.
    pub fn into_result(self) -> Result<T, FrameworkAdapterResponseEnvelopeError> {
        match self {
            Self::Success(envelope) => {
                if envelope.success {
                    Ok(envelope.data)
                } else {
                    Err(FrameworkAdapterResponseEnvelopeError::InvalidEnvelope {
                        detail: "success envelope must set success=true".to_string(),
                    })
                }
            }
            Self::Error(envelope) => {
                if envelope.success {
                    Err(FrameworkAdapterResponseEnvelopeError::InvalidEnvelope {
                        detail: "error envelope must set success=false".to_string(),
                    })
                } else {
                    Err(FrameworkAdapterResponseEnvelopeError::Protocol {
                        code: envelope.error.code,
                        message: envelope.error.message,
                        details: envelope.error.details,
                    })
                }
            }
        }
    }
}

/// Errors surfaced while unwrapping a parsed response envelope.
#[derive(Debug, Clone, PartialEq)]
pub enum FrameworkAdapterResponseEnvelopeError {
    /// The envelope shape parsed but violated the success/error rules.
    InvalidEnvelope { detail: String },
    /// The adapter returned a protocol-valid error envelope.
    Protocol { code: String, message: String, details: Option<Value> },
}

/// Typed request sent to the adapter `preflight` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterPreflightRequest {
    /// Current Boundline version string.
    pub boundline_version: String,
    /// Workspace path or reference used for the run.
    pub workspace_ref: String,
    /// Whether the current host surface forbids prompts.
    pub non_interactive: bool,
    /// Proposed config values to validate.
    pub config_values: Vec<FrameworkAdapterConfigValue>,
}

/// Typed response emitted by the adapter `preflight` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterPreflightResponse {
    /// Overall readiness state for adapter-owned execution.
    pub status: FrameworkAdapterPreflightStatus,
    /// Config values normalized by the adapter.
    pub normalized_config_values: Vec<FrameworkAdapterConfigValue>,
    /// Warning messages surfaced without blocking execution.
    pub warnings: Vec<String>,
    /// Blocking reason when `status = blocked`.
    pub reason: Option<FrameworkAdapterPreflightBlockReason>,
    /// Field keys still missing when the adapter cannot proceed.
    pub missing_fields: Vec<String>,
    /// Recovery guidance for operator-visible output.
    pub recovery: Option<String>,
}

/// Typed request sent to the adapter `execute-stage` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterExecuteStageRequest {
    /// Lifecycle run identifier.
    pub run_id: Uuid,
    /// Host-known stage identifier.
    pub stage_key: FrameworkStageKey,
    /// Attempt index for the current stage.
    pub stage_attempt: u32,
    /// Workspace path or reference used for the run.
    pub workspace_ref: String,
    /// Stable adapter identifier the host selected.
    pub adapter_id: String,
    /// Resolved config values available to the adapter.
    pub config_values: Vec<FrameworkAdapterConfigValue>,
    /// Artifact references that provide additional execution context.
    pub context_artifacts: Vec<String>,
}

/// Typed response emitted by the adapter `execute-stage` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterExecuteStageResponse {
    /// Terminal stage state emitted by the adapter.
    pub status: FrameworkAdapterStageExecutionStatus,
    /// Operator-readable summary for the claimed stage.
    pub summary: String,
    /// Artifact references produced by the adapter.
    pub produced_artifacts: Vec<String>,
    /// Optional next-action guidance when the stage blocks or fails.
    pub next_action: Option<String>,
    /// Failure classification when the stage does not succeed.
    pub failure_class: Option<FrameworkAdapterFailureClass>,
}

/// Typed request sent to the adapter `emit-hook` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEmissionRequest {
    /// Lifecycle run identifier.
    pub run_id: Uuid,
    /// Hook identifier owned by the host catalog.
    pub hook_key: FrameworkHookKey,
    /// Stage that was active when the hook fired.
    pub stage_key: FrameworkStageKey,
    /// Whether the adapter already owned the current stage.
    pub stage_claimed: bool,
    /// Workspace path or reference used for the run.
    pub workspace_ref: String,
    /// Trace or artifact reference that carries the hook payload.
    pub payload_ref: String,
}

/// Typed response emitted by the adapter `emit-hook` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEmissionResponse {
    /// Delivery result for the hook invocation.
    pub status: FrameworkAdapterHookDeliveryStatus,
    /// Operator-readable summary for trace and status projections.
    pub summary: String,
}
