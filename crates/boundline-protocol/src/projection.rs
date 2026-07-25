//! Read-only session projections expose authoritative state without mutation powers.

use serde::{Deserialize, Serialize};

use crate::{OperationId, ProtocolVersion, RepositoryId, Revision, SessionId};

/// Stable high-level stages used by public Boundline projections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingStage {
    /// Establish the governed goal.
    Goal,
    /// Admit an execution plan.
    Plan,
    /// Execute or delegate the admitted change.
    Execute,
    /// Verify claims against fresh evidence.
    Verify,
    /// Publish the verified candidate.
    Publish,
    /// Synchronize the terminal governance outcome.
    SynchronizeOutcome,
}

/// Stable lifecycle projection for a governed session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionLifecycle {
    /// Session can continue executing admitted work.
    Active,
    /// Session was deliberately paused.
    Paused,
    /// Session cannot advance until a visible condition is resolved.
    Blocked,
    /// Session awaits an approval bound to the current revision.
    ApprovalPending,
    /// Session awaits fresh claim-matched proof.
    ProofPending,
    /// One executor currently owns the session lease.
    ExecutorInFlight,
    /// Crash-produced delta awaits revalidation.
    UncommittedCandidate,
    /// Verified candidate awaits repository publication.
    PublicationPending,
    /// Journaled publication requires recovery.
    RecoveryRequired,
    /// Session reached a durably finalized terminal state.
    Terminal,
}

/// A typed action that may advance the current projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction {
    /// Operation the caller may request.
    pub operation: OperationId,
    /// Human-readable description derived from authoritative state.
    pub summary: String,
}

/// Aggregate executor capability admission state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutorCapabilityStatus {
    /// No executor capability decision is currently needed.
    NotRequired,
    /// All required capabilities were admitted.
    Admitted,
    /// One or more required capabilities were denied.
    Denied,
}

/// Claim-proof freshness visible to every transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofFreshness {
    /// No proof has been admitted for the current state.
    Missing,
    /// Proof matches the current state and claims.
    Fresh,
    /// A later mutation invalidated the proof.
    Stale,
}

/// Public publication lifecycle, distinct from internal journal states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationStatus {
    /// Publication has not started.
    NotStarted,
    /// A verified candidate is ready for publication.
    Pending,
    /// Publication completed durably.
    Completed,
}

/// Public recovery requirement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryStatus {
    /// No journaled publication requires recovery.
    NotRequired,
    /// A journal fully explains state that requires recovery.
    Required,
}

/// Public repository quarantine projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineStatus {
    /// Repository is not quarantined.
    Clear,
    /// Unexplained state is preserved and blocks mutation.
    Quarantined,
}

/// Terminal outcome synchronization state with Canon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonOutcomeSyncStatus {
    /// No terminal outcome is ready for synchronization.
    NotRequired,
    /// Terminal outcome is durably queued.
    Pending,
    /// Canon recorded the terminal outcome.
    Synchronized,
    /// Terminal outcome synchronization failed visibly.
    Failed,
}

/// Transport-neutral authoritative projection of a governed session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicSessionProjection {
    /// Schema version used to encode the projection.
    pub protocol_version: ProtocolVersion,
    /// Projected session identity.
    pub session_id: SessionId,
    /// Clone-local repository identity.
    pub repository_id: RepositoryId,
    /// Current transaction revision.
    pub transaction_revision: Revision,
    /// Current lifecycle.
    pub lifecycle: SessionLifecycle,
    /// Current operating stage.
    pub stage: OperatingStage,
    /// Actions admitted by the current authoritative state.
    pub next_actions: Vec<NextAction>,
    /// Aggregate executor capability decision.
    pub executor_capability: ExecutorCapabilityStatus,
    /// Current proof freshness.
    pub proof_freshness: ProofFreshness,
    /// Current publication state.
    pub publication: PublicationStatus,
    /// Whether journal-driven recovery is required.
    pub recovery: RecoveryStatus,
    /// Whether unexplained state has quarantined the repository.
    pub quarantine: QuarantineStatus,
    /// Current terminal outcome synchronization state.
    pub canon_outcome_sync: CanonOutcomeSyncStatus,
}
