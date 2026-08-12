//! Evidence freshness is bound to the exact accepted transaction state.

use boundline_core::transaction::evidence::{
    BoundEvidence, EvidenceKind, EvidenceLedger, GovernedStateBinding,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

fn binding(revision: u64, diff: &str, fingerprint: &str) -> GovernedStateBinding {
    GovernedStateBinding::new(
        "session-1",
        revision,
        diff,
        fingerprint,
        ["tests_pass", "format_clean"],
        "reviewer-1",
    )
}

#[test]
fn approval_proof_risk_and_verification_require_an_exact_binding() -> TestResult {
    let current = binding(7, "diff-a", "fingerprint-a");
    let mut ledger = EvidenceLedger::new(current.clone());
    for kind in [
        EvidenceKind::Approval,
        EvidenceKind::Proof,
        EvidenceKind::RiskAcceptance,
        EvidenceKind::Verification,
    ] {
        ledger.record(BoundEvidence::new(kind, current.clone(), "evidence-ref"))?;
    }
    require(ledger.all_fresh(), "exactly bound evidence was not fresh")?;
    let wrong_claims = GovernedStateBinding::new(
        "session-1",
        7,
        "diff-a",
        "fingerprint-a",
        ["tests_pass"],
        "reviewer-1",
    );
    require(
        ledger.record(BoundEvidence::new(EvidenceKind::Proof, wrong_claims, "bad")).is_err(),
        "different claim set admitted",
    )
}

#[test]
fn formatter_or_codegen_mutation_stales_every_prior_authority_record() -> TestResult {
    let initial = binding(7, "diff-a", "fingerprint-a");
    let mut ledger = EvidenceLedger::new(initial.clone());
    ledger.record(BoundEvidence::new(EvidenceKind::Approval, initial.clone(), "approval"))?;
    ledger.record(BoundEvidence::new(EvidenceKind::Proof, initial, "proof"))?;
    ledger.advance(binding(8, "diff-after-formatter", "fingerprint-after-formatter"));
    require(!ledger.all_fresh(), "formatter mutation preserved stale proof")?;
    require(ledger.stale_count() == 2, "not every authority record became stale")
}

#[test]
fn reviewer_lineage_is_normative_and_cannot_be_substituted() -> TestResult {
    let current = binding(3, "diff", "fingerprint");
    let ledger = EvidenceLedger::new(current.clone());
    let other = GovernedStateBinding::new(
        "session-1",
        3,
        "diff",
        "fingerprint",
        ["tests_pass", "format_clean"],
        "implementer",
    );
    require(!ledger.matches_current(&other), "different lineage matched current binding")
}
