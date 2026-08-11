//! Cross-boundary contract for durable terminal-outcome synchronization.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use boundline_core::publication::canon_outbox::{
    ArchiveAuthorization, CanonOutcomeOutbox, CanonOutcomeTransport, CanonSubprocessTransport,
    Clock, DeliveryAttemptResult, DeliveryControl, EnqueueOutcome, OutboxError, OutboxState,
    TransportFailure, TransportFailureKind,
};
use boundline_protocol::CanonOutcomeSyncStatus;
use canon_contracts::{
    ApprovalDecision, AuthoritativeTimestamp, BundleDigest, BundleId, ChallengeTier, Claim,
    CommitIdentity, DecisionMemoryDigest, Deviation, EvidenceReference, FinalFingerprint,
    OutcomeApprovalBinding, OutcomeAuthorityBinding, OutcomeChallengeBinding, OutcomeEventDigest,
    OutcomeEventId, OutcomeLineage, OutcomeNextAction, OutcomeSessionId, OutcomeSourceProduct,
    RecordOutcomeDisposition, RecordOutcomeRejectionReason, RecordOutcomeRequest,
    RecordOutcomeResponse, RepositoryIdentity, Revision, TerminalOutcomeStatus,
};

const EVENT_ID: &str = "outcome-event-m2bc";
const FINAL_REVISION: u64 = 42;
const CANON_REVISION: u64 = 19;
const NOW_MS: u64 = 1_785_953_730_000;

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

#[derive(Clone, Copy)]
struct FixedClock(u64);

impl Clock for FixedClock {
    fn now_millis(&self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
struct ScriptedTransport {
    calls: Arc<AtomicUsize>,
    outcomes: VecDeque<Result<RecordOutcomeResponse, TransportFailure>>,
}

impl ScriptedTransport {
    fn new(outcomes: Vec<Result<RecordOutcomeResponse, TransportFailure>>) -> Self {
        Self { calls: Arc::new(AtomicUsize::new(0)), outcomes: outcomes.into() }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl CanonOutcomeTransport for ScriptedTransport {
    fn deliver(
        &mut self,
        request: &RecordOutcomeRequest,
    ) -> Result<RecordOutcomeResponse, TransportFailure> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.outcomes.pop_front().unwrap_or_else(|| Ok(recorded_response(request)))
    }
}

fn request_for(
    event_id: &str,
    status: TerminalOutcomeStatus,
) -> Result<RecordOutcomeRequest, Box<dyn std::error::Error>> {
    let (published_commit, final_fingerprint) = match status {
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
    };
    let mut request = RecordOutcomeRequest {
        event_id: OutcomeEventId::new(event_id),
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
        proof_references: vec![EvidenceReference::new("proof:cargo-test")],
        deviations: vec![Deviation::new("deviation:none")],
        terminal_claims: vec![Claim::new("claim:verified")],
        authority_binding: OutcomeAuthorityBinding {
            authority_identity: "release-owner".to_owned(),
            final_transaction_revision: Revision::new(FINAL_REVISION),
            claims: vec![Claim::new("claim:verified")],
        },
        approval_binding: Some(OutcomeApprovalBinding {
            approver_identity: "release-owner".to_owned(),
            decision: ApprovalDecision::Approved,
            final_transaction_revision: Revision::new(FINAL_REVISION),
            claims: vec![Claim::new("claim:verified")],
        }),
        challenge_binding: OutcomeChallengeBinding {
            tier: ChallengeTier::Tier2,
            challenger_identity: Some("independent-reviewer".to_owned()),
            challenger_invocation_id: Some("review-invocation-001".to_owned()),
            independent_context_identity: Some("context:independent-001".to_owned()),
            claims: vec![Claim::new("claim:verified")],
            evidence_references: vec![EvidenceReference::new("proof:independent-review")],
            named_override: None,
        },
        lineage: OutcomeLineage {
            producer_identity: "boundline-executor".to_owned(),
            producer_invocation_id: "boundline-invocation-001".to_owned(),
            verifier_identity: Some("independent-reviewer".to_owned()),
            verifier_invocation_id: Some("review-invocation-001".to_owned()),
        },
        occurred_at: Some(AuthoritativeTimestamp::new("2026-08-05T10:15:30Z")),
    };
    request.recompute_event_digest()?;
    Ok(request)
}

fn recorded_response(request: &RecordOutcomeRequest) -> RecordOutcomeResponse {
    RecordOutcomeResponse {
        event_id: request.event_id.clone(),
        event_digest: request.event_digest.clone(),
        disposition: RecordOutcomeDisposition::Recorded,
        decision_memory_revision: Some(Revision::new(CANON_REVISION)),
        decision_memory_digest: Some(DecisionMemoryDigest::new(
            "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        )),
        reason_code: None,
        next_actions: Vec::new(),
    }
}

fn rejected_response(
    request: &RecordOutcomeRequest,
    reason: RecordOutcomeRejectionReason,
) -> RecordOutcomeResponse {
    RecordOutcomeResponse {
        event_id: request.event_id.clone(),
        event_digest: request.event_digest.clone(),
        disposition: RecordOutcomeDisposition::Rejected,
        decision_memory_revision: None,
        decision_memory_digest: None,
        reason_code: Some(reason),
        next_actions: vec![OutcomeNextAction::InspectDecisionMemory],
    }
}

#[test]
fn published_enqueue_precedes_delivery_and_ack_survives_restart() -> TestResult {
    let root = tempfile::tempdir()?;
    let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    require_eq(
        outbox.enqueue_terminal_outcome(request.clone())?,
        EnqueueOutcome::Enqueued,
        "enqueue",
    )?;
    let queued = outbox.load(EVENT_ID)?;
    require_eq(queued.state, OutboxState::Pending, "durable state before transport")?;
    require_eq(
        outbox.sync_status(Some(EVENT_ID))?,
        CanonOutcomeSyncStatus::Pending,
        "pending projection",
    )?;

    let mut transport = ScriptedTransport::new(vec![Ok(recorded_response(&request))]);
    require_eq(
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?,
        DeliveryAttemptResult::Synchronized,
        "delivery result",
    )?;
    let reopened = CanonOutcomeOutbox::open(root.path())?;
    let record = reopened.load(EVENT_ID)?;
    require_eq(record.state, OutboxState::Synchronized, "restart acknowledgement")?;
    require_eq(record.request.proof_references, request.proof_references, "proof retention")?;
    require_eq(
        reopened.sync_status(Some(EVENT_ID))?,
        CanonOutcomeSyncStatus::Synchronized,
        "sync projection",
    )
}

#[test]
fn all_terminal_outcomes_preserve_their_actual_status() -> TestResult {
    for status in [
        TerminalOutcomeStatus::NoChange,
        TerminalOutcomeStatus::Failed,
        TerminalOutcomeStatus::Cancelled,
        TerminalOutcomeStatus::Rejected,
    ] {
        let root = tempfile::tempdir()?;
        let request = request_for(EVENT_ID, status)?;
        let outbox = CanonOutcomeOutbox::open(root.path())?;
        outbox.enqueue_terminal_outcome(request.clone())?;
        let mut transport = ScriptedTransport::new(vec![Ok(recorded_response(&request))]);
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?;
        let recorded = outbox.load(EVENT_ID)?;
        require_eq(recorded.request.terminal_status, status, "terminal status changed")?;
        require(
            status != TerminalOutcomeStatus::NoChange
                || recorded.request.published_commit.is_none(),
            "no_change fabricated a commit",
        )?;
    }
    Ok(())
}

#[test]
fn nonterminal_and_local_digest_conflicts_fail_before_transport() -> TestResult {
    for status in [TerminalOutcomeStatus::Blocked, TerminalOutcomeStatus::Stale] {
        let root = tempfile::tempdir()?;
        let outbox = CanonOutcomeOutbox::open(root.path())?;
        let error = outbox.enqueue_terminal_outcome(request_for(EVENT_ID, status)?);
        require(
            matches!(
                error,
                Err(OutboxError::Rejected(RecordOutcomeRejectionReason::NonterminalOutcome))
            ),
            "nonterminal outcome was queued",
        )?;
    }

    let root = tempfile::tempdir()?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    let original = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    outbox.enqueue_terminal_outcome(original.clone())?;
    let mut changed = original;
    changed.deviations.push(Deviation::new("deviation:changed"));
    changed.recompute_event_digest()?;
    require(
        matches!(
            outbox.enqueue_terminal_outcome(changed),
            Err(OutboxError::IdentityDigestConflict)
        ),
        "changed content overwrote an event identity",
    )
}

#[test]
fn exact_duplicates_before_and_after_ack_are_idempotent() -> TestResult {
    let root = tempfile::tempdir()?;
    let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    require_eq(
        outbox.enqueue_terminal_outcome(request.clone())?,
        EnqueueOutcome::Enqueued,
        "first enqueue",
    )?;
    require_eq(
        outbox.enqueue_terminal_outcome(request.clone())?,
        EnqueueOutcome::Replayed,
        "pre-ack replay",
    )?;
    let mut transport = ScriptedTransport::new(vec![Ok(recorded_response(&request))]);
    outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?;
    require_eq(
        outbox.enqueue_terminal_outcome(request)?,
        EnqueueOutcome::Replayed,
        "post-ack replay",
    )?;
    require_eq(outbox.load(EVENT_ID)?.attempt_history.len(), 1, "duplicate delivery count")
}

#[test]
fn transient_fault_windows_are_durable_and_explicitly_retryable() -> TestResult {
    for kind in [
        TransportFailureKind::BeforeProcessStart,
        TransportFailureKind::RequestWrite,
        TransportFailureKind::AfterRequestWrite,
        TransportFailureKind::ResponseRead,
        TransportFailureKind::ProcessExit,
        TransportFailureKind::Timeout,
    ] {
        let root = tempfile::tempdir()?;
        let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
        let outbox = CanonOutcomeOutbox::open(root.path())?;
        outbox.enqueue_terminal_outcome(request.clone())?;
        let mut transport = ScriptedTransport::new(vec![
            Err(TransportFailure::new(kind, "controlled transport fault")),
            Ok(recorded_response(&request)),
        ]);
        require_eq(
            outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?,
            DeliveryAttemptResult::RetryScheduled,
            "transient classification",
        )?;
        let retry_at = outbox.load(EVENT_ID)?.next_retry_at_ms.ok_or("retry was not scheduled")?;
        require_eq(
            outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(retry_at))?,
            DeliveryAttemptResult::Synchronized,
            "explicit retry",
        )?;
    }
    Ok(())
}

#[test]
fn permanent_rejections_and_conflicts_never_auto_retry() -> TestResult {
    for reason in [
        RecordOutcomeRejectionReason::UnsupportedContractLine,
        RecordOutcomeRejectionReason::InvalidOutcome,
        RecordOutcomeRejectionReason::AuthorityBindingInvalid,
        RecordOutcomeRejectionReason::ApprovalBindingInvalid,
        RecordOutcomeRejectionReason::EvidenceBindingInvalid,
        RecordOutcomeRejectionReason::LineageInvalid,
        RecordOutcomeRejectionReason::StaleOutcome,
        RecordOutcomeRejectionReason::DecisionMemoryConflict,
    ] {
        let root = tempfile::tempdir()?;
        let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
        let outbox = CanonOutcomeOutbox::open(root.path())?;
        outbox.enqueue_terminal_outcome(request.clone())?;
        let mut transport = ScriptedTransport::new(vec![Ok(rejected_response(&request, reason))]);
        require_eq(
            outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?,
            DeliveryAttemptResult::PermanentRejected(reason),
            "permanent rejection",
        )?;
        require_eq(
            outbox.load(EVENT_ID)?.state,
            OutboxState::PermanentRejected,
            "rejection state",
        )?;
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS + 1))?;
        require_eq(transport.calls(), 1, "permanent rejection retried")?;
    }

    let root = tempfile::tempdir()?;
    let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    outbox.enqueue_terminal_outcome(request.clone())?;
    let conflict =
        rejected_response(&request, RecordOutcomeRejectionReason::IdentityDigestConflict);
    let mut transport = ScriptedTransport::new(vec![Ok(conflict)]);
    require_eq(
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?,
        DeliveryAttemptResult::Conflict,
        "conflict result",
    )?;
    require_eq(outbox.load(EVENT_ID)?.state, OutboxState::Conflict, "conflict state")
}

#[test]
fn persistence_and_acknowledgement_failures_remain_replay_safe() -> TestResult {
    let root = tempfile::tempdir()?;
    let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    outbox.enqueue_terminal_outcome(request.clone())?;
    let persistence = rejected_response(&request, RecordOutcomeRejectionReason::PersistenceFailed);
    let replay = RecordOutcomeResponse {
        disposition: RecordOutcomeDisposition::Replayed,
        ..recorded_response(&request)
    };
    let mut transport = ScriptedTransport::new(vec![Ok(persistence), Ok(replay.clone())]);
    require_eq(
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?,
        DeliveryAttemptResult::RetryScheduled,
        "Canon persistence failure",
    )?;
    let retry_at = outbox.load(EVENT_ID)?.next_retry_at_ms.ok_or("retry missing")?;
    require_eq(
        outbox.deliver_once_with_control(
            EVENT_ID,
            &mut transport,
            &FixedClock(retry_at),
            DeliveryControl::fail_before_ack(),
        )?,
        DeliveryAttemptResult::AcknowledgementPending,
        "acknowledgement fault",
    )?;
    require_eq(outbox.load(EVENT_ID)?.state, OutboxState::InFlight, "response-loss state")?;
    let mut retry_transport = ScriptedTransport::new(vec![Ok(replay)]);
    require_eq(
        outbox.deliver_once(EVENT_ID, &mut retry_transport, &FixedClock(retry_at + 1))?,
        DeliveryAttemptResult::Synchronized,
        "replayed acknowledgement",
    )
}

#[test]
fn archival_is_authorized_durable_and_non_destructive() -> TestResult {
    for synchronize in [false, true] {
        let root = tempfile::tempdir()?;
        let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
        let outbox = CanonOutcomeOutbox::open(root.path())?;
        outbox.enqueue_terminal_outcome(request.clone())?;
        let response = if synchronize {
            recorded_response(&request)
        } else {
            rejected_response(&request, RecordOutcomeRejectionReason::InvalidOutcome)
        };
        let mut transport = ScriptedTransport::new(vec![Ok(response)]);
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))?;
        outbox.archive_outbox_record(
            EVENT_ID,
            ArchiveAuthorization::new("release-owner", "retention complete", NOW_MS + 1),
        )?;
        let archived = CanonOutcomeOutbox::open(root.path())?.load(EVENT_ID)?;
        require_eq(archived.state, OutboxState::Archived, "archive state")?;
        require_eq(archived.request, request, "archive deleted event")?;
        require(archived.archive.is_some(), "archive authority missing")?;
    }

    let root = tempfile::tempdir()?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    outbox.enqueue_terminal_outcome(request_for(EVENT_ID, TerminalOutcomeStatus::Published)?)?;
    require(
        outbox
            .archive_outbox_record(
                EVENT_ID,
                ArchiveAuthorization::new("release-owner", "too early", NOW_MS),
            )
            .is_err(),
        "active record was archived",
    )
}

#[test]
fn concurrent_delivery_has_one_fenced_owner() -> TestResult {
    let root = tempfile::tempdir()?;
    let request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    outbox.enqueue_terminal_outcome(request.clone())?;
    let first_outbox = outbox.clone();
    let second_outbox = outbox.clone();
    let first_request = request.clone();
    let second_request = request;
    let first = std::thread::spawn(move || {
        let mut transport = ScriptedTransport::new(vec![Ok(recorded_response(&first_request))]);
        first_outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))
    });
    let second = std::thread::spawn(move || {
        let mut transport = ScriptedTransport::new(vec![Ok(recorded_response(&second_request))]);
        second_outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS))
    });
    let first_result = first.join().map_err(|_| "first delivery thread panicked")??;
    let second_result = second.join().map_err(|_| "second delivery thread panicked")??;
    let synchronized = [first_result, second_result]
        .into_iter()
        .filter(|result| *result == DeliveryAttemptResult::Synchronized)
        .count();
    require_eq(synchronized, 1, "concurrent acknowledgement owners")?;
    require_eq(outbox.load(EVENT_ID)?.attempt_history.len(), 1, "concurrent attempt count")
}

#[test]
fn portable_state_excludes_private_execution_material() -> TestResult {
    let root = tempfile::tempdir()?;
    let outbox = CanonOutcomeOutbox::open(root.path())?;
    outbox.enqueue_terminal_outcome(request_for(EVENT_ID, TerminalOutcomeStatus::Published)?)?;
    let portable = serde_json::to_string(&outbox.load(EVENT_ID)?)?.to_ascii_lowercase();
    for forbidden in [
        "provider_token",
        "api_key",
        "raw_prompt",
        "private_conversation",
        "process_handle",
        "environment",
        root.path().to_string_lossy().as_ref(),
    ] {
        require(!portable.contains(forbidden), format!("portable state leaked {forbidden}"))?;
    }
    Ok(())
}

#[test]
fn real_canon_process_commits_once_and_response_loss_replays() -> TestResult {
    let Some(executable) = std::env::var_os("BOUNDLINE_CANON_TEST_BINARY") else {
        return Ok(());
    };
    let workspace = tempfile::tempdir()?;
    admit_real_governance_bundle(&executable, workspace.path())?;
    let snapshot_path = workspace.path().join(".canon/decision-memory/state.json");
    let snapshot: serde_json::Value = serde_json::from_slice(&std::fs::read(&snapshot_path)?)?;
    let contract = &snapshot["snapshot"]["admitted_bundles"][0]["contract"];
    let bundle_id = contract["bundle_id"].as_str().ok_or("bundle id missing")?;
    let bundle_digest = contract["bundle_digest"].as_str().ok_or("bundle digest missing")?;

    let mut request = request_for(EVENT_ID, TerminalOutcomeStatus::Published)?;
    request.governance_bundle_id = BundleId::new(bundle_id);
    request.governance_bundle_digest = BundleDigest::new(bundle_digest);
    request.recompute_event_digest()?;
    let outbox_root = workspace.path().join("boundline-state");
    let outbox = CanonOutcomeOutbox::open(&outbox_root)?;
    outbox.enqueue_terminal_outcome(request)?;
    let mut transport = CanonSubprocessTransport::new(
        executable,
        workspace.path(),
        workspace.path(),
        Duration::from_secs(20),
    );
    require_eq(
        outbox.deliver_once_with_control(
            EVENT_ID,
            &mut transport,
            &FixedClock(NOW_MS),
            DeliveryControl::fail_before_ack(),
        )?,
        DeliveryAttemptResult::AcknowledgementPending,
        "real response-loss boundary",
    )?;
    require_eq(
        outbox.deliver_once(EVENT_ID, &mut transport, &FixedClock(NOW_MS + 1))?,
        DeliveryAttemptResult::Synchronized,
        "real Canon replay",
    )?;
    let final_snapshot: serde_json::Value = serde_json::from_slice(&std::fs::read(snapshot_path)?)?;
    let events = final_snapshot["snapshot"]["graph"]["events"]
        .as_array()
        .ok_or("decision-memory journal missing")?;
    let outcomes =
        events.iter().filter(|event| event["event"] == "outcome_recorded").collect::<Vec<_>>();
    require_eq(outcomes.len(), 1, "real Canon outcome event count")?;
    require(
        outcomes[0]["outcome"]["execution_audit"]
            == serde_json::json!({
                "process_invocations": 0,
                "network_invocations": 0,
                "provider_credential_reads": 0,
                "model_calls": 0,
                "semantic_evidence_created": 0
            }),
        "Canon outcome ingestion executed a semantic capability",
    )
}

fn admit_real_governance_bundle(
    executable: &std::ffi::OsStr,
    workspace: &std::path::Path,
) -> TestResult {
    let bundle_id = "bundle-083-001";
    let packet_id = format!("packet-{bundle_id}");
    let artifact_id = format!("artifact-{bundle_id}");
    let claim_id = format!("claim-{bundle_id}-main");
    let requirement_id = format!("requirement-{bundle_id}");
    let evidence_id = format!("evidence-{bundle_id}");
    let draft = serde_json::json!({
        "bundle_id": bundle_id,
        "profile": "discovery",
        "decision_memory_revision": 1,
        "packets": [{
            "packet_id": packet_id,
            "profile": "discovery",
            "revision": 1,
            "change_intent": "record terminal outcome",
            "scope": ["workspace"],
            "risks": ["incorrect outcome"],
            "invariants": ["no semantic execution"],
            "acceptance_criteria": ["exact outcome binding"],
            "cross_packet_references": []
        }],
        "subject_artifacts": [{
            "artifact_id": artifact_id,
            "packet_id": packet_id,
            "content": {
                "artifact_identity": "workspace",
                "revision": "git:fixture",
                "content_digest": "a".repeat(64)
            }
        }],
        "required_approvers": ["release-owner"],
        "authority_zone": "governance-release",
        "risk_tier": 2,
        "change_class": "governance-kernel",
        "context_class": "repository-local",
        "owners": ["release-owner"],
        "claims": ["claim-exact-binding"],
        "required_evidence": [{
            "requirement_id": requirement_id,
            "packet_id": packet_id,
            "claim_ids": [claim_id],
            "artifact_ids": [artifact_id],
            "kind": "external_semantic_review",
            "minimum_challenge_tier": "tier_2",
            "accepted_evidence_references": [format!("sha256:{}", "e".repeat(64))]
        }],
        "provided_evidence": [{
            "evidence_id": evidence_id,
            "packet_id": packet_id,
            "claim_ids": [claim_id],
            "artifact_ids": [artifact_id],
            "requirement_ids": [requirement_id],
            "references": [format!("sha256:{}", "e".repeat(64))],
            "lineage": "provider:challenger/executor:review/invocation:cross-repo",
            "independent_context_identity": "context-independent-cross-repo",
            "challenge_tier": "tier_2",
            "external_semantic": true,
            "fresh": true,
            "named_override": null
        }],
        "forbidden_lineages": [],
        "approvals": [{
            "approval_id": format!("approval-{bundle_id}"),
            "packet_id": packet_id,
            "claim_ids": [claim_id],
            "artifact_ids": [artifact_id],
            "evidence_ids": [evidence_id],
            "requirement_ids": [requirement_id],
            "approver": "release-owner",
            "authority_zone": "governance-release",
            "approved": true,
            "decision_memory_revision": 1,
            "valid_through_revision": 1,
            "fresh": true
        }],
        "assumptions": ["contract immutable"],
        "alternatives": ["defer"],
        "rationale": "bind actual outcome",
        "risk_acceptances": [],
        "triggers": ["governed state changes"],
        "no_change": false
    });
    let request = serde_json::json!({
        "contract_version": "1.0",
        "request_id": bundle_id,
        "operation": "start",
        "payload": {"bundle": draft}
    });
    let mut child = std::process::Command::new(executable)
        .args(["--canon-root"])
        .arg(workspace)
        .args(["--repo-root"])
        .arg(workspace)
        .args(["rpc", "--stdio"])
        .current_dir(workspace)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("Canon stdin missing")?;
    serde_json::to_writer(&mut stdin, &request)?;
    drop(stdin);
    let output = child.wait_with_output()?;
    require(
        output.status.success(),
        format!(
            "Canon governance admission failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}
