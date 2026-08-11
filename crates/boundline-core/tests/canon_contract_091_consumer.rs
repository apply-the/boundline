//! Registry-contract qualification for the future Canon outcome outbox owner.

use std::fmt::Debug;

use canon_contracts::{
    ApprovalDecision, AuthoritativeTimestamp, BundleDigest, BundleId, CanonContractVersion,
    ChallengeTier, Claim, CommitIdentity, DecisionMemoryDigest, Deviation, EvidenceReference,
    FinalFingerprint, OneShotOperation, OneShotRequest, OutcomeApprovalBinding,
    OutcomeAuthorityBinding, OutcomeChallengeBinding, OutcomeEventDigest, OutcomeEventId,
    OutcomeLineage, OutcomeNextAction, OutcomeSessionId, OutcomeSourceProduct,
    RecordOutcomeDisposition, RecordOutcomeRejectionReason, RecordOutcomeRequest,
    RecordOutcomeResponse, RepositoryIdentity, Revision, TerminalOutcomeStatus,
};
use serde_json::{Value, json};

const EVENT_ID: &str = "outcome-event-001";
const FINAL_REVISION: u64 = 42;
const CANON_REVISION: u64 = 19;
const GOLDEN_DIGEST: &str =
    "sha256:eb492104900410462528226f5fca56a758ef13b940d8a0c5d962110d5de2bafa";

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(condition: bool, message: impl Into<String>) -> TestResult {
    if condition { Ok(()) } else { Err(message.into().into()) }
}

fn require_eq<T: Debug + PartialEq>(actual: T, expected: T, message: &str) -> TestResult {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{message}: actual {actual:?}, expected {expected:?}").into())
    }
}

fn terminal_artifacts(
    status: TerminalOutcomeStatus,
) -> (Option<CommitIdentity>, Option<FinalFingerprint>) {
    match status {
        TerminalOutcomeStatus::Published => (
            Some(CommitIdentity::new("6b4d8ac1d1644cbd88f78d57f66aeb78550d3f42")),
            Some(FinalFingerprint::new("sha256:published-fingerprint")),
        ),
        TerminalOutcomeStatus::NoChange => {
            (None, Some(FinalFingerprint::new("sha256:unchanged-fingerprint")))
        }
        TerminalOutcomeStatus::Failed
        | TerminalOutcomeStatus::Cancelled
        | TerminalOutcomeStatus::Blocked
        | TerminalOutcomeStatus::Stale
        | TerminalOutcomeStatus::Rejected => (None, None),
    }
}

fn request_for(
    status: TerminalOutcomeStatus,
) -> Result<RecordOutcomeRequest, Box<dyn std::error::Error>> {
    let (published_commit, final_fingerprint) = terminal_artifacts(status);
    let mut request = RecordOutcomeRequest {
        event_id: OutcomeEventId::new(EVENT_ID),
        event_digest: OutcomeEventDigest::placeholder(),
        source_product: OutcomeSourceProduct::Boundline,
        source_repository_identity: RepositoryIdentity::new("git-common-dir:repo-083"),
        governance_bundle_id: BundleId::new("bundle-083-001"),
        governance_bundle_digest: BundleDigest::new("sha256:bundle-083"),
        session_id: OutcomeSessionId::new("session-083-001"),
        final_transaction_revision: Revision::new(FINAL_REVISION),
        terminal_status: status,
        published_commit,
        final_fingerprint,
        proof_references: vec![
            EvidenceReference::new("proof:cargo-test"),
            EvidenceReference::new("proof:nextest"),
        ],
        deviations: vec![Deviation::new("deviation:none")],
        terminal_claims: vec![Claim::new("claim:verified"), Claim::new("claim:published")],
        authority_binding: OutcomeAuthorityBinding {
            authority_identity: "release-owner".to_owned(),
            final_transaction_revision: Revision::new(FINAL_REVISION),
            claims: vec![Claim::new("claim:published")],
        },
        approval_binding: Some(OutcomeApprovalBinding {
            approver_identity: "release-owner".to_owned(),
            decision: ApprovalDecision::Approved,
            final_transaction_revision: Revision::new(FINAL_REVISION),
            claims: vec![Claim::new("claim:published")],
        }),
        challenge_binding: OutcomeChallengeBinding {
            tier: ChallengeTier::Tier2,
            challenger_identity: Some("independent-reviewer".to_owned()),
            challenger_invocation_id: Some("review-invocation-001".to_owned()),
            independent_context_identity: Some("context:independent-001".to_owned()),
            claims: vec![Claim::new("claim:published")],
            evidence_references: vec![EvidenceReference::new("proof:independent-review")],
            named_override: None,
        },
        lineage: OutcomeLineage {
            producer_identity: "boundline-executor".to_owned(),
            producer_invocation_id: "boundline-invocation-001".to_owned(),
            verifier_identity: Some("independent-reviewer".to_owned()),
            verifier_invocation_id: Some("review-invocation-001".to_owned()),
        },
        occurred_at: Some(AuthoritativeTimestamp::new("2026-07-29T10:15:30Z")),
    };
    request.recompute_event_digest()?;
    Ok(request)
}

#[test]
fn exact_dependency_preserves_the_additive_operation_inventory() -> TestResult {
    require_eq(serde_json::to_value(CanonContractVersion::V1)?, json!("1.0"), "V1 wire value")?;
    let operations = [
        OneShotOperation::Capabilities,
        OneShotOperation::Start,
        OneShotOperation::Refresh,
        OneShotOperation::Approve,
        OneShotOperation::Inspect,
        OneShotOperation::Publish,
        OneShotOperation::RecordOutcome,
    ];
    let actual = operations.into_iter().map(serde_json::to_value).collect::<Result<Vec<_>, _>>()?;
    let expected = serde_json::from_str::<Vec<Value>>(
        r#"["capabilities","start","refresh","approve","inspect","publish","record_outcome"]"#,
    )?;
    require_eq(actual, expected, "ordered seven-operation inventory")?;
    require(
        serde_json::to_value(OneShotOperation::Publish)?
            != serde_json::to_value(OneShotOperation::RecordOutcome)?,
        "publish and record_outcome collapsed to one operation",
    )
}

#[test]
fn exact_dependency_round_trips_outcome_request_and_response() -> TestResult {
    let request = request_for(TerminalOutcomeStatus::Published)?;
    let envelope = OneShotRequest {
        contract_version: CanonContractVersion::V1,
        request_id: EVENT_ID.to_owned(),
        operation: OneShotOperation::RecordOutcome,
        payload: request.clone(),
    };
    require_eq(
        serde_json::from_value::<OneShotRequest<RecordOutcomeRequest>>(serde_json::to_value(
            &envelope,
        )?)?,
        envelope,
        "request round trip",
    )?;
    for disposition in [RecordOutcomeDisposition::Recorded, RecordOutcomeDisposition::Replayed] {
        let response = RecordOutcomeResponse {
            event_id: request.event_id.clone(),
            event_digest: request.event_digest.clone(),
            disposition,
            decision_memory_revision: Some(Revision::new(CANON_REVISION)),
            decision_memory_digest: Some(DecisionMemoryDigest::new(
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            )),
            reason_code: None,
            next_actions: Vec::new(),
        };
        require_eq(
            serde_json::from_value::<RecordOutcomeResponse>(serde_json::to_value(&response)?)?,
            response,
            "successful response round trip",
        )?;
    }
    let rejected = RecordOutcomeResponse {
        event_id: request.event_id,
        event_digest: request.event_digest,
        disposition: RecordOutcomeDisposition::Rejected,
        decision_memory_revision: None,
        decision_memory_digest: None,
        reason_code: Some(RecordOutcomeRejectionReason::UnsupportedOperation),
        next_actions: vec![OutcomeNextAction::UpgradeContract],
    };
    require_eq(
        serde_json::from_value::<RecordOutcomeResponse>(serde_json::to_value(&rejected)?)?,
        rejected,
        "rejected response round trip",
    )
}

#[test]
fn exact_dependency_preserves_digest_and_terminal_matrix() -> TestResult {
    let published = request_for(TerminalOutcomeStatus::Published)?;
    require_eq(
        published.event_digest.clone(),
        OutcomeEventDigest::new(GOLDEN_DIGEST),
        "canonical digest fixture",
    )?;
    for status in [
        TerminalOutcomeStatus::Published,
        TerminalOutcomeStatus::NoChange,
        TerminalOutcomeStatus::Failed,
        TerminalOutcomeStatus::Cancelled,
        TerminalOutcomeStatus::Rejected,
    ] {
        request_for(status)?.validate()?;
    }
    for status in [TerminalOutcomeStatus::Blocked, TerminalOutcomeStatus::Stale] {
        require(
            request_for(status)?.validate().is_err(),
            format!("nonterminal status {status:?} was accepted"),
        )?;
    }
    Ok(())
}
