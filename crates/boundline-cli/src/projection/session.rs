//! CLI projections expose execution, proof, challenge, and publication without inventing authority.

use boundline_protocol::{
    ExecutorCapabilityStatus, ProofFreshness, PublicSessionProjection, PublicationStatus,
    SessionLifecycle,
};
use serde::{Deserialize, Serialize};

/// Stable governed lifecycle operations represented by the CLI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernedSessionOperation {
    Run,
    Approve,
    Status,
    Inspect,
    SessionAbort,
    SessionCleanup,
}

/// Current independent-challenge projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeProjection {
    NotRequired,
    Missing,
    Satisfied,
    SatisfiedWithNamedOverride,
}

/// CLI-owned view derived entirely from the authoritative protocol projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedSessionProjection {
    /// Requested presentation operation.
    pub operation: GovernedSessionOperation,
    /// Complete transport-neutral authoritative session state.
    pub session: PublicSessionProjection,
    /// Challenge result admitted for this exact revision.
    pub challenge: ChallengeProjection,
    /// Whether a mutating command may proceed from this projection.
    pub mutation_admitted: bool,
    /// Stable fail-closed reasons explaining denied authority.
    pub blocking_reasons: Vec<String>,
}

impl GovernedSessionProjection {
    /// Derives a command projection without upgrading proposals or stale evidence.
    pub fn derive(
        operation: GovernedSessionOperation,
        session: PublicSessionProjection,
        challenge: ChallengeProjection,
    ) -> Self {
        let mut blocking_reasons = Vec::new();
        if session.executor_capability == ExecutorCapabilityStatus::Denied {
            blocking_reasons.push("executor_capability_denied".to_owned());
        }
        if matches!(operation, GovernedSessionOperation::Approve)
            && session.proof_freshness != ProofFreshness::Fresh
        {
            blocking_reasons.push("fresh_proof_required".to_owned());
        }
        if matches!(operation, GovernedSessionOperation::Approve)
            && matches!(challenge, ChallengeProjection::Missing)
        {
            blocking_reasons.push("challenge_required".to_owned());
        }
        if matches!(operation, GovernedSessionOperation::SessionCleanup)
            && session.lifecycle != SessionLifecycle::Terminal
        {
            blocking_reasons.push("session_retention_required".to_owned());
        }
        if session.publication == PublicationStatus::Completed
            && matches!(
                operation,
                GovernedSessionOperation::Run | GovernedSessionOperation::Approve
            )
        {
            blocking_reasons.push("publication_already_completed".to_owned());
        }
        let mutation_admitted = blocking_reasons.is_empty()
            && !matches!(
                operation,
                GovernedSessionOperation::Status | GovernedSessionOperation::Inspect
            );
        Self { operation, session, challenge, mutation_admitted, blocking_reasons }
    }
}

#[cfg(test)]
mod tests {
    use boundline_protocol::{
        CanonOutcomeSyncStatus, ExecutorCapabilityStatus, NextAction, OperatingStage,
        ProofFreshness, ProtocolVersion, PublicationStatus, QuarantineStatus, RecoveryStatus,
        RepositoryId, Revision, SessionId, SessionLifecycle,
    };

    use super::{ChallengeProjection, GovernedSessionOperation, GovernedSessionProjection};

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    fn require(condition: bool, message: &str) -> TestResult {
        if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
    }
    fn session(
        lifecycle: SessionLifecycle,
        proof: ProofFreshness,
    ) -> boundline_protocol::PublicSessionProjection {
        boundline_protocol::PublicSessionProjection {
            protocol_version: ProtocolVersion::V1,
            session_id: SessionId::new("session-1"),
            repository_id: RepositoryId::new("repository-1"),
            transaction_revision: Revision::new(7),
            lifecycle,
            stage: OperatingStage::Verify,
            next_actions: Vec::<NextAction>::new(),
            executor_capability: ExecutorCapabilityStatus::Admitted,
            proof_freshness: proof,
            publication: PublicationStatus::Pending,
            recovery: RecoveryStatus::NotRequired,
            quarantine: QuarantineStatus::Clear,
            canon_outcome_sync: CanonOutcomeSyncStatus::NotRequired,
        }
    }

    #[test]
    fn approve_requires_fresh_proof_and_challenge() -> TestResult {
        let denied = GovernedSessionProjection::derive(
            GovernedSessionOperation::Approve,
            session(SessionLifecycle::ProofPending, ProofFreshness::Stale),
            ChallengeProjection::Missing,
        );
        require(
            !denied.mutation_admitted && denied.blocking_reasons.len() == 2,
            "stale approval projected as admitted",
        )?;
        let admitted = GovernedSessionProjection::derive(
            GovernedSessionOperation::Approve,
            session(SessionLifecycle::ApprovalPending, ProofFreshness::Fresh),
            ChallengeProjection::Satisfied,
        );
        require(admitted.mutation_admitted, "fresh challenged approval was denied")
    }

    #[test]
    fn status_and_inspect_never_claim_mutation_authority() -> TestResult {
        for operation in [GovernedSessionOperation::Status, GovernedSessionOperation::Inspect] {
            let projection = GovernedSessionProjection::derive(
                operation,
                session(SessionLifecycle::Active, ProofFreshness::Fresh),
                ChallengeProjection::Satisfied,
            );
            require(
                !projection.mutation_admitted,
                "read-only projection gained mutation authority",
            )?;
        }
        Ok(())
    }

    #[test]
    fn cleanup_requires_terminal_state_while_abort_remains_explicit() -> TestResult {
        let cleanup = GovernedSessionProjection::derive(
            GovernedSessionOperation::SessionCleanup,
            session(SessionLifecycle::Active, ProofFreshness::Missing),
            ChallengeProjection::NotRequired,
        );
        require(!cleanup.mutation_admitted, "active session cleanup admitted")?;
        let abort = GovernedSessionProjection::derive(
            GovernedSessionOperation::SessionAbort,
            session(SessionLifecycle::Active, ProofFreshness::Missing),
            ChallengeProjection::NotRequired,
        );
        require(abort.mutation_admitted, "explicit abort was not projected")
    }

    #[test]
    fn denied_executor_and_completed_publication_both_block_reexecution() -> TestResult {
        let mut completed = session(SessionLifecycle::Terminal, ProofFreshness::Fresh);
        completed.executor_capability = ExecutorCapabilityStatus::Denied;
        completed.publication = PublicationStatus::Completed;
        let projection = GovernedSessionProjection::derive(
            GovernedSessionOperation::Run,
            completed,
            ChallengeProjection::Satisfied,
        );

        require(!projection.mutation_admitted, "completed denied session regained authority")?;
        require(
            projection.blocking_reasons
                == ["executor_capability_denied", "publication_already_completed"],
            "projection did not preserve both independent authority denials",
        )
    }
}
