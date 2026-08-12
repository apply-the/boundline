//! Exact state bindings invalidate authority and proof after every mutation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Exact state and lineage dimensions supported by an authority record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedStateBinding {
    session_id: String,
    transaction_revision: u64,
    accepted_diff_digest: String,
    worktree_fingerprint: String,
    claim_set: BTreeSet<String>,
    reviewer_lineage: String,
}

impl GovernedStateBinding {
    /// Creates the full normative binding without volatile exclusions.
    pub fn new<I, S>(
        session_id: impl Into<String>,
        transaction_revision: u64,
        accepted_diff_digest: impl Into<String>,
        worktree_fingerprint: impl Into<String>,
        claims: I,
        reviewer_lineage: impl Into<String>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            session_id: session_id.into(),
            transaction_revision,
            accepted_diff_digest: accepted_diff_digest.into(),
            worktree_fingerprint: worktree_fingerprint.into(),
            claim_set: claims.into_iter().map(Into::into).collect(),
            reviewer_lineage: reviewer_lineage.into(),
        }
    }
}

/// Authority-bearing evidence classes governed by identical freshness rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceKind {
    /// Named human authority decision.
    Approval,
    /// Claim-matched completion proof.
    Proof,
    /// Named acceptance of a recorded risk.
    RiskAcceptance,
    /// Independent verification result.
    Verification,
}

/// One immutable evidence reference and its exact governed-state binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundEvidence {
    kind: EvidenceKind,
    binding: GovernedStateBinding,
    reference: String,
    stale: bool,
}

impl BoundEvidence {
    /// Creates evidence awaiting admission against the current state.
    pub fn new(
        kind: EvidenceKind,
        binding: GovernedStateBinding,
        reference: impl Into<String>,
    ) -> Self {
        Self { kind, binding, reference: reference.into(), stale: false }
    }
}

/// In-memory domain service for deterministic freshness propagation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLedger {
    current: GovernedStateBinding,
    records: Vec<BoundEvidence>,
}

impl EvidenceLedger {
    /// Starts a ledger at one admitted transaction state.
    pub fn new(current: GovernedStateBinding) -> Self {
        Self { current, records: Vec::new() }
    }

    /// Admits evidence only when every binding dimension matches.
    pub fn record(&mut self, evidence: BoundEvidence) -> Result<(), EvidenceBindingError> {
        if evidence.binding != self.current {
            return Err(EvidenceBindingError::StateMismatch);
        }
        self.records.push(evidence);
        Ok(())
    }

    /// Advances after mutation and makes every previous record stale.
    pub fn advance(&mut self, current: GovernedStateBinding) {
        if self.current != current {
            for record in &mut self.records {
                record.stale = true;
            }
            self.current = current;
        }
    }

    /// Reports whether every admitted record supports the current state.
    pub fn all_fresh(&self) -> bool {
        self.records.iter().all(|record| !record.stale && record.binding == self.current)
    }

    /// Counts stale records retained for audit.
    pub fn stale_count(&self) -> usize {
        self.records.iter().filter(|record| record.stale).count()
    }

    /// Reports fresh evidence of one required authority class.
    pub fn has_fresh(&self, kind: EvidenceKind) -> bool {
        self.records
            .iter()
            .any(|record| record.kind == kind && !record.stale && record.binding == self.current)
    }

    /// Checks a prospective binding against current authority state.
    pub fn matches_current(&self, binding: &GovernedStateBinding) -> bool {
        binding == &self.current
    }
}

/// Evidence admission failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceBindingError {
    /// At least one exact binding dimension differs.
    #[error("evidence does not match the current transaction state")]
    StateMismatch,
}
