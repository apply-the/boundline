//! Trace-side records for provider-backed lifecycle activity.

use serde::{Deserialize, Serialize};

use crate::domain::capability_provider::{
    CapabilityProviderProjection, ProviderValidationDisposition,
};

/// Trace record summarizing the latest provider-backed execution observed for a
/// run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProviderTraceRecord {
    /// Request identifier shared across the provider lifecycle.
    pub request_id: String,
    /// Additive provider projection preserved in trace summaries.
    pub projection: CapabilityProviderProjection,
    /// Final validation disposition preserved for later inspection.
    pub validation: ProviderValidationDisposition,
}
