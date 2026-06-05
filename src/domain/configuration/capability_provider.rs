//! Configuration-side records for external capability-provider registrations.

use serde::{Deserialize, Serialize};

use crate::domain::capability_provider::CapabilityProviderRegistration;

/// Persisted provider block stored in workspace configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedCapabilityProviderConfiguration {
    /// Registered providers available in this workspace.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registrations: Vec<CapabilityProviderRegistration>,
    /// Active provider identifier, when one registration is authoritative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_provider_id: Option<String>,
    /// Timestamp of the latest successful activation or validation pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_validated_at: Option<u64>,
}
