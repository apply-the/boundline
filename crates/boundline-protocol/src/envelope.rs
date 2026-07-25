//! Versioned mutation envelopes define the stable replay and revision boundary.

use serde::{Deserialize, Serialize};

use crate::{
    ContractLine, EvidenceReference, OperationId, RequestDigest, RequestId, Revision,
    TraceReference,
};

/// Frozen protocol schema version accepted by the 1.0 contract line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolVersion {
    /// Boundline public protocol version 1.0.
    #[serde(rename = "1.0")]
    V1,
}

/// Typed mutation request carrying replay identity and revision precondition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationRequestEnvelope<T> {
    /// Schema version used to decode the envelope.
    pub protocol_version: ProtocolVersion,
    /// Contract namespace in which the operation and request ID are scoped.
    pub contract_line: ContractLine,
    /// Requested mutation operation.
    pub operation: OperationId,
    /// Idempotency identifier for this mutation request.
    pub request_id: RequestId,
    /// Canonical digest used to detect request-ID payload conflicts.
    pub canonical_request_digest: RequestDigest,
    /// State revision required for a request not already recorded.
    pub expected_state_revision: Revision,
    /// Operation-specific typed payload.
    pub payload: T,
}

/// Mutation outcome represented without exposing internal transaction records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum MutationOutcome<T> {
    /// Mutation was admitted and produced the typed value.
    Accepted(T),
    /// Mutation failed closed with a stable public reason.
    Rejected(ReasonCode),
}

/// Typed mutation result with the exact revision transition and proof references.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResultEnvelope<T> {
    /// Schema version used to encode the result.
    pub protocol_version: ProtocolVersion,
    /// Request identifier copied from the admitted request.
    pub request_id: RequestId,
    /// Canonical request digest copied from the admitted request.
    pub canonical_request_digest: RequestDigest,
    /// Revision observed before the recorded mutation.
    pub previous_revision: Revision,
    /// Revision after the recorded mutation.
    pub resulting_revision: Revision,
    /// Typed accepted value or stable rejection reason.
    pub outcome: MutationOutcome<T>,
    /// Immutable evidence references supporting the result.
    pub evidence: Vec<EvidenceReference>,
    /// Immutable trace references supporting the result.
    pub traces: Vec<TraceReference>,
}

/// Stable fail-closed reason codes for the Boundline 1.0 contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    /// A request ID was reused with a different canonical digest.
    IdempotencyConflict,
    /// A new mutation request did not target the current revision.
    StateRevisionMismatch,
    /// Another executor already owns the session execution lease.
    ExecutorAlreadyInFlight,
    /// An executor attempted to write with a superseded fencing token.
    ExecutorFencingTokenStale,
    /// Reconciliation cannot begin while executor termination is unconfirmed.
    ExecutorTerminationUnconfirmed,
    /// The requested executor capability was not admitted.
    ExecutorCapabilityDenied,
    /// The executor escaped its admitted mutation or capability boundary.
    ExecutorBoundaryViolated,
    /// A crash delta must be revalidated before ownership can be assigned.
    UncommittedCandidateRequiresValidation,
    /// An approval no longer matches the accepted state.
    ApprovalStale,
    /// Proof no longer matches the accepted state.
    ProofStale,
    /// Repository identity does not match the admitted local instance.
    RepositoryIdentityMismatch,
    /// The authoritative worktree contains unexplained changes.
    AuthoritativeWorktreeDirty,
    /// Another publication owns the repository-scoped lock.
    PublicationLockHeld,
    /// The admitted base is no longer the publication target.
    PublicationRebaseRequired,
    /// An authoritative path or metadata precondition changed.
    PublicationPreconditionFailed,
    /// A started publication requires journal-driven recovery.
    PublicationRecoveryRequired,
    /// Repository state is preserved pending explicit operator resolution.
    RepositoryQuarantined,
    /// Publication succeeded but terminal Canon synchronization remains pending.
    CanonOutcomeSyncPending,
    /// Canon permanently rejected or could not accept the terminal outcome.
    CanonOutcomeSyncFailed,
    /// The Git or filesystem state is outside the qualified stable boundary.
    UnsupportedGitOrFilesystemState,
}
