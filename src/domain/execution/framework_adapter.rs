//! Execution-side records for adapter capability validation and stage routing.

use serde::{Deserialize, Serialize};

use crate::domain::framework_adapter::{
    AdapterCapabilitySnapshotState, AdapterExecutionSource, AdapterHookKey,
    AdapterLifecycleStageKey, LifecycleStageExecutionStatus, ProtocolCompatibilityState,
    StageClaimState, StageRoutingDecisionReason,
};

/// Validated adapter capabilities captured before a lifecycle run begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCapabilitySnapshot {
    /// Stable run identifier.
    pub run_id: String,
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Protocol line declared by the adapter.
    pub protocol_line: String,
    /// Adapter-reported version string.
    pub adapter_version: String,
    /// Adapter-declared Boundline compatibility range.
    pub supported_boundline_range: String,
    /// Lifecycle stages the adapter declared it can own.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_stage_overrides: Vec<AdapterLifecycleStageKey>,
    /// Hook families the adapter wants to receive.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_hook_subscriptions: Vec<AdapterHookKey>,
    /// Fingerprint of the config schema validated during preflight.
    pub config_schema_fingerprint: String,
    /// Result of capability validation.
    pub snapshot_state: AdapterCapabilitySnapshotState,
}

/// Host-owned compatibility result derived from capability validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolCompatibilityRecord {
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Protocol line checked by the host.
    pub protocol_line: String,
    /// Adapter-declared Boundline compatibility range.
    pub supported_boundline_range: String,
    /// Compatibility verdict.
    pub state: ProtocolCompatibilityState,
    /// Timestamp of the compatibility evaluation.
    pub evaluated_at: u64,
}

/// Host-owned routing decision for one lifecycle stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageRoutingDecisionRecord {
    /// Stable run identifier.
    pub run_id: String,
    /// Lifecycle stage being routed.
    pub stage_key: AdapterLifecycleStageKey,
    /// Effective execution source selected by the host.
    pub execution_source: AdapterExecutionSource,
    /// Host reason for the routing choice.
    pub decision_reason: StageRoutingDecisionReason,
    /// Claim status for the routed stage.
    pub claim_state: StageClaimState,
    /// Adapter identifier when the route references an external adapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    /// Durable stage status when the claimed-stage outcome is already known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_status: Option<LifecycleStageExecutionStatus>,
    /// Produced artifacts captured from the authoritative adapter response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub produced_artifacts: Vec<String>,
    /// Timestamp of the routing decision.
    pub recorded_at: u64,
}
