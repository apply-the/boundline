#[path = "../../../src/adapters/agent.rs"]
pub mod agent;
#[path = "../../../src/adapters/audit_store.rs"]
pub mod audit_store;
#[path = "../../../src/adapters/auth_profile_store.rs"]
pub mod auth_profile_store;
#[path = "../../../src/adapters/capability_provider_runtime.rs"]
pub mod capability_provider_runtime;
#[path = "../../../src/adapters/checkpoint_store.rs"]
pub mod checkpoint_store;
#[path = "../../../src/adapters/cluster_store.rs"]
pub mod cluster_store;
#[path = "../../../src/adapters/config_store.rs"]
pub mod config_store;
#[path = "../../../src/adapters/env_layer.rs"]
pub mod env_layer;
#[path = "../../../src/adapters/github_device_flow.rs"]
pub mod github_device_flow;
#[path = "../../../src/adapters/governance_runtime.rs"]
pub mod governance_runtime;
#[path = "../../../src/adapters/provider_runtime.rs"]
pub mod provider_runtime;
#[path = "../../../src/adapters/session_store.rs"]
pub mod session_store;
#[path = "../../../src/adapters/tool.rs"]
pub mod tool;
#[path = "../../../src/adapters/trace_store.rs"]
pub mod trace_store;

pub mod framework_protocol;

pub use framework_protocol::{
    FrameworkAdapterCommand, FrameworkAdapterConfigFieldDefinition, FrameworkAdapterConfigValue,
    FrameworkAdapterDescribeResponse, FrameworkAdapterEnvelopeError, FrameworkAdapterErrorEnvelope,
    FrameworkAdapterExecuteStageRequest, FrameworkAdapterExecuteStageResponse,
    FrameworkAdapterFailureClass, FrameworkAdapterFieldValueKind,
    FrameworkAdapterHookDeliveryStatus, FrameworkAdapterImplementationStatus,
    FrameworkAdapterPlanningFinding, FrameworkAdapterPlanningFindingSeverity,
    FrameworkAdapterPlanningReadinessStatus, FrameworkAdapterPlanningRemediationSkipReason,
    FrameworkAdapterPlanningRemediationTaskOutcome, FrameworkAdapterPreflightBlockReason,
    FrameworkAdapterPreflightRequest, FrameworkAdapterPreflightResponse,
    FrameworkAdapterPreflightStatus, FrameworkAdapterRequiredFieldPolicy,
    FrameworkAdapterResponseEnvelope, FrameworkAdapterResponseEnvelopeError,
    FrameworkAdapterStageExecutionStatus, FrameworkAdapterSuccessEnvelope,
    FrameworkAdapterTransportChannel, FrameworkAdapterTransportDescriptor,
    FrameworkAdapterTransportEncoding, FrameworkAdapterTransportKind, HookEmissionRequest,
    HookEmissionResponse, format_framework_adapter_transports,
    framework_adapter_supports_v1_transport,
};

/// Single process-wide mutex serialising all tests that mutate environment
/// variables. Shared across `env_layer` and `provider_runtime` test modules so
/// they cannot race on `OPENAI_API_KEY`, `XDG_CONFIG_HOME`, etc.
#[cfg(test)]
pub(crate) static SHARED_ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> =
    std::sync::OnceLock::new();
