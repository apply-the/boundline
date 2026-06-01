//! Trace-side records for framework-adapter hook dispatch activity.

use serde::{Deserialize, Serialize};

use crate::domain::framework_adapter::{
    AdapterHookKey, AdapterLifecycleStageKey, HookDispatchStatus,
};

/// Trace record for one hook emission attempt targeting an active adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookEventDispatchRecord {
    /// Stable run identifier.
    pub run_id: String,
    /// Hook identifier emitted by the host.
    pub hook_key: AdapterHookKey,
    /// Lifecycle stage associated with the emitted hook.
    pub stage_key: AdapterLifecycleStageKey,
    /// Adapter identifier receiving the hook.
    pub adapter_id: String,
    /// Whether the adapter had already claimed the associated stage.
    #[serde(default)]
    pub stage_claimed: bool,
    /// Ref or path to the emitted payload envelope.
    pub payload_ref: String,
    /// Hook dispatch result.
    pub dispatch_status: HookDispatchStatus,
    /// Short human-facing summary of the dispatch outcome.
    pub summary: String,
    /// Timestamp of the hook dispatch event.
    pub recorded_at: u64,
}
