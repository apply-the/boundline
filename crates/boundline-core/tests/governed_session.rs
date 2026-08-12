//! Governed approval persists only after exact fresh proof and challenge admission.

use std::path::Path;
use std::process::Command;

use boundline_core::transaction::challenge::ChallengeDecision;
use boundline_core::transaction::evidence::{
    BoundEvidence, EvidenceKind, EvidenceLedger, GovernedStateBinding,
};
use boundline_core::transaction::governed_session::{
    GovernedSessionError, GovernedSessionRecord, GovernedSessionStore,
};
use boundline_protocol::{
    CanonOutcomeSyncStatus, ExecutorCapabilityStatus, OperatingStage, ProofFreshness,
    ProtocolVersion, PublicSessionProjection, PublicationStatus, QuarantineStatus, RecoveryStatus,
    RepositoryId, Revision, SessionId, SessionLifecycle,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}
fn git(root: &Path, args: &[&str]) -> TestResult {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    require(output.status.success(), &String::from_utf8_lossy(&output.stderr))
}
fn repository(root: &Path) -> TestResult {
    git(root, &["init", "-b", "main"])?;
    git(root, &["config", "user.name", "Test"])?;
    git(root, &["config", "user.email", "test@example.invalid"])?;
    git(root, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(root.join(".gitignore"), ".boundline/\n")?;
    git(root, &["add", ".gitignore"])?;
    git(root, &["commit", "-m", "base"])
}
fn projection() -> PublicSessionProjection {
    PublicSessionProjection {
        protocol_version: ProtocolVersion::V1,
        session_id: SessionId::new("session-1"),
        repository_id: RepositoryId::new("public-repository"),
        transaction_revision: Revision::new(5),
        lifecycle: SessionLifecycle::ApprovalPending,
        stage: OperatingStage::Verify,
        next_actions: Vec::new(),
        executor_capability: ExecutorCapabilityStatus::Admitted,
        proof_freshness: ProofFreshness::Fresh,
        publication: PublicationStatus::Pending,
        recovery: RecoveryStatus::NotRequired,
        quarantine: QuarantineStatus::Clear,
        canon_outcome_sync: CanonOutcomeSyncStatus::NotRequired,
    }
}
fn ledger(include_verification: bool) -> Result<EvidenceLedger, Box<dyn std::error::Error>> {
    let binding =
        GovernedStateBinding::new("session-1", 5, "diff", "fingerprint", ["claim"], "reviewer");
    let mut ledger = EvidenceLedger::new(binding.clone());
    ledger.record(BoundEvidence::new(EvidenceKind::Proof, binding.clone(), "proof"))?;
    if include_verification {
        ledger.record(BoundEvidence::new(EvidenceKind::Verification, binding, "verification"))?;
    }
    Ok(ledger)
}

#[test]
fn approval_is_durable_and_advances_to_publication_pending() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let repository_path = fixture.path().join("repository");
    let state = fixture.path().join("state");
    std::fs::create_dir(&repository_path)?;
    repository(&repository_path)?;
    let store = GovernedSessionStore::open(&repository_path, &state)?;
    store.save(&GovernedSessionRecord::new(
        projection(),
        ledger(true)?,
        ChallengeDecision::Satisfied,
    ))?;
    let approved = store.approve("session-1")?;
    require(
        approved.approved
            && approved.projection.lifecycle == SessionLifecycle::PublicationPending
            && approved.projection.stage == OperatingStage::Publish,
        "approval did not advance exact governed state",
    )?;
    require(store.load("session-1")?.approved, "approval was not durable")
}

#[test]
fn missing_verification_or_challenge_fails_closed() -> TestResult {
    let fixture = tempfile::tempdir()?;
    let repository_path = fixture.path().join("repository");
    let state = fixture.path().join("state");
    std::fs::create_dir(&repository_path)?;
    repository(&repository_path)?;
    let store = GovernedSessionStore::open(&repository_path, &state)?;
    store.save(&GovernedSessionRecord::new(
        projection(),
        ledger(false)?,
        ChallengeDecision::Satisfied,
    ))?;
    require(
        matches!(store.approve("session-1"), Err(GovernedSessionError::FreshEvidenceRequired)),
        "approval omitted verification",
    )?;
    store.save(&GovernedSessionRecord::new(
        projection(),
        ledger(true)?,
        ChallengeDecision::Missing {
            requirements: vec!["distinct_lineage".into()],
            automatic_override_permitted: true,
        },
    ))?;
    require(
        matches!(store.approve("session-1"), Err(GovernedSessionError::ChallengeRequired)),
        "approval omitted challenge",
    )
}
