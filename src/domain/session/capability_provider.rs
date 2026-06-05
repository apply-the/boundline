//! Session-side records for provider-backed admission, execution, and
//! validation projections.

use serde::{Deserialize, Serialize};

use crate::domain::capability_provider::CapabilityProviderProjection;

/// Additive session-visible record summarizing the latest provider-backed
/// request known to the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityProviderExecutionRecord {
    /// Request identifier shared across prepare, execute, and evidence steps.
    pub request_id: String,
    /// Compact additive provider projection for status and inspect.
    pub projection: CapabilityProviderProjection,
}
