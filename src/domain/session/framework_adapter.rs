//! Session-side records for framework-adapter lifecycle execution state.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::framework_adapter::{
    AdapterExecutionSource, AdapterFailureClass, AdapterLifecycleStageKey,
    FrameworkAdapterStageOutcomeDetails, LifecycleStageExecutionStatus, StageClaimState,
};
use crate::domain::task::TerminalReason;

/// Persisted execution summary for one lifecycle stage during a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleStageExecutionRecord {
    /// Stable run identifier.
    pub run_id: String,
    /// Lifecycle stage being summarized.
    pub stage_key: AdapterLifecycleStageKey,
    /// Effective stage execution source selected by the host.
    pub execution_source: AdapterExecutionSource,
    /// Adapter identifier when the stage was adapter-owned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_id: Option<String>,
    /// Terminal status recorded for the stage.
    pub status: LifecycleStageExecutionStatus,
    /// Whether operator intervention is required before continuing.
    #[serde(default)]
    pub intervention_required: bool,
    /// Failure class when the stage did not complete cleanly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<AdapterFailureClass>,
    /// Artifacts or refs produced by the stage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub produced_artifacts: Vec<String>,
    /// Optional adapter-owned detail payload preserved with the stage outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<FrameworkAdapterStageOutcomeDetails>,
    /// Stage start timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    /// Stage end timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<u64>,
}

/// Structured terminal details recorded when an adapter-owned lifecycle stage
/// fails after the adapter has claimed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkAdapterStageFailureDetails {
    /// Durable execution record for the claimed stage outcome.
    pub execution: LifecycleStageExecutionRecord,
    /// Claim state captured by the host when the stage failed.
    pub claim_state: StageClaimState,
    /// Compact operator-facing summary of the failure classification.
    pub summary: String,
    /// Optional detailed diagnostic string preserved for operator output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Protocol error code when the claimed stage failed with a protocol-valid
    /// error envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_error_code: Option<String>,
}

impl FrameworkAdapterStageFailureDetails {
    /// Deserializes claimed-stage adapter failure details from a terminal-reason
    /// details payload.
    pub fn from_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok()
    }

    /// Extracts claimed-stage adapter failure details from a terminal reason.
    pub fn from_terminal_reason(reason: &TerminalReason) -> Option<Self> {
        reason.details.as_ref().and_then(Self::from_value)
    }
}
