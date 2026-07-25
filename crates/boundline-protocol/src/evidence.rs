//! Public evidence and lineage descriptors bind proof to an exact governed state.

use serde::{Deserialize, Serialize};

use crate::{
    AcceptedDiffDigest, Claim, EvidenceReference, Fingerprint, Revision, SessionId, StageId,
};

/// Public executor or reviewer lineage needed to evaluate independence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageDescriptor {
    /// Stable identity of the executor or reviewer.
    pub executor_id: String,
    /// Provider family when the executor is model-assisted.
    pub provider_family: Option<String>,
    /// Pinned model identity when the executor is model-assisted.
    pub model: Option<String>,
    /// Invocation that produced the result.
    pub invocation_id: String,
}

/// Freshness of evidence relative to the accepted transaction state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFreshness {
    /// Evidence matches the accepted revision, diff, fingerprint, and claims.
    Fresh,
    /// A later mutation invalidated at least one binding dimension.
    Stale,
}

/// Proof binding exposed without its persistence or invalidation implementation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceBinding {
    /// Governed session supported by the evidence.
    pub session_id: SessionId,
    /// Exact transaction revision supported by the evidence.
    pub transaction_revision: Revision,
    /// Exact accepted complete diff supported by the evidence.
    pub accepted_diff_digest: AcceptedDiffDigest,
    /// Exact worktree fingerprint supported by the evidence.
    pub worktree_fingerprint: Fingerprint,
    /// Claims evaluated by the reviewer.
    pub claim_set: Vec<Claim>,
    /// Reviewer lineage used to evaluate independence requirements.
    pub reviewer_lineage: LineageDescriptor,
    /// Immutable records containing the actual proof.
    pub evidence_references: Vec<EvidenceReference>,
    /// Current freshness projection.
    pub freshness: EvidenceFreshness,
}

/// Public route projection linking a stage to its selected executor lineage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDescriptor {
    /// Stable route identity.
    pub route_id: String,
    /// Stage for which the route was selected.
    pub stage: StageId,
    /// Selected executor identity.
    pub executor: String,
    /// Invocation lineage attached to the route.
    pub lineage: LineageDescriptor,
}
