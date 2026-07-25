//! Contract tests for the frozen Boundline protocol V1 data shapes.

use boundline_protocol::{
    AcceptedDiffDigest, AdapterDescriptor, AdapterIdentity, AdapterProposal, AdapterTransport,
    CanonOutcomeSyncStatus, CapabilityDescriptor, CapabilityDisposition, Claim, ContractLine,
    EvidenceBinding, EvidenceFreshness, EvidenceReference, ExecutorCapabilityStatus, Fingerprint,
    FrameworkAdapterAuthority, LineageDescriptor, MutationOutcome, MutationProposal,
    MutationRequestEnvelope, MutationResultEnvelope, NextAction, OperatingStage,
    OperationDescriptor, OperationId, ProofFreshness, ProtocolVersion, PublicSessionProjection,
    PublicationStatus, QuarantineStatus, ReasonCode, RecoveryStatus, RepositoryId, RequestDigest,
    RequestId, Revision, RouteDescriptor, SessionId, SessionLifecycle, StageId, TraceReference,
    canonical_json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt::Debug;

const CONTRACT_LINE: &str = "boundline.protocol";
const OPERATION: &str = "session.advance";
const REQUEST_ID: &str = "req-001";
const SESSION_ID: &str = "session-001";
const REPOSITORY_ID: &str = "repo-001";
const REQUEST_DIGEST: &str = "sha256:request";
const DIFF_DIGEST: &str = "sha256:diff";
const FINGERPRINT: &str = "sha256:worktree";
const REVISION_BEFORE: u64 = 7;
const REVISION_AFTER: u64 = 8;

fn check(condition: bool, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

fn check_eq<T: Debug + PartialEq>(
    actual: T,
    expected: T,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual == expected {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("{message}: actual {actual:?}, expected {expected:?}"))
            .into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AdvancePayload {
    stage: StageId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AdvanceResult {
    accepted: bool,
}

fn request() -> MutationRequestEnvelope<AdvancePayload> {
    MutationRequestEnvelope {
        protocol_version: ProtocolVersion::V1,
        contract_line: ContractLine::new(CONTRACT_LINE),
        operation: OperationId::new(OPERATION),
        request_id: RequestId::new(REQUEST_ID),
        canonical_request_digest: RequestDigest::new(REQUEST_DIGEST),
        expected_state_revision: Revision::new(REVISION_BEFORE),
        payload: AdvancePayload { stage: StageId::new("verification") },
    }
}

#[test]
fn mutation_envelopes_round_trip_with_stable_field_names() -> Result<(), Box<dyn std::error::Error>>
{
    let request = request();
    let encoded = serde_json::to_value(&request)?;

    check_eq(encoded["protocol_version"].clone(), json!("1.0"), "protocol version")?;
    check_eq(encoded["contract_line"].clone(), json!(CONTRACT_LINE), "contract line")?;
    check_eq(encoded["operation"].clone(), json!(OPERATION), "operation")?;
    check_eq(encoded["request_id"].clone(), json!(REQUEST_ID), "request ID")?;
    check_eq(encoded["canonical_request_digest"].clone(), json!(REQUEST_DIGEST), "request digest")?;
    check_eq(
        encoded["expected_state_revision"].clone(),
        json!(REVISION_BEFORE),
        "expected revision",
    )?;
    check_eq(
        serde_json::from_value::<MutationRequestEnvelope<AdvancePayload>>(encoded)?,
        request,
        "request round trip",
    )?;

    let result = MutationResultEnvelope {
        protocol_version: ProtocolVersion::V1,
        request_id: RequestId::new(REQUEST_ID),
        canonical_request_digest: RequestDigest::new(REQUEST_DIGEST),
        previous_revision: Revision::new(REVISION_BEFORE),
        resulting_revision: Revision::new(REVISION_AFTER),
        outcome: MutationOutcome::Accepted(AdvanceResult { accepted: true }),
        evidence: vec![EvidenceReference::new("evidence-001")],
        traces: vec![TraceReference::new("trace-001")],
    };
    let result_json = serde_json::to_value(&result)?;

    check_eq(result_json["outcome"]["status"].clone(), json!("accepted"), "accepted outcome")?;
    check_eq(
        serde_json::from_value::<MutationResultEnvelope<AdvanceResult>>(result_json)?,
        result,
        "result round trip",
    )?;
    Ok(())
}

#[test]
fn unsupported_protocol_versions_and_unknown_authority_values_fail_closed()
-> Result<(), Box<dyn std::error::Error>> {
    let mut unsupported = serde_json::to_value(request()).unwrap_or(Value::Null);
    unsupported["protocol_version"] = json!("2.0");
    check(
        serde_json::from_value::<MutationRequestEnvelope<AdvancePayload>>(unsupported).is_err(),
        "unsupported protocol version was accepted",
    )?;

    let unknown_authority = json!({
        "invocation_id": "inv-001",
        "authority": "may_publish",
        "summary": "proposal",
        "mutations": [],
        "artifacts": [],
        "evidence": [],
        "diagnostics": [],
        "next_actions": []
    });
    check(
        serde_json::from_value::<AdapterProposal>(unknown_authority).is_err(),
        "unknown adapter authority was accepted",
    )
}

#[test]
fn additive_fields_are_ignored_by_v1_consumers() -> Result<(), Box<dyn std::error::Error>> {
    let mut encoded = serde_json::to_value(request())?;
    encoded["future_optional_field"] = json!({"introduced_in": "1.1"});

    let decoded: MutationRequestEnvelope<AdvancePayload> = serde_json::from_value(encoded)?;
    check_eq(decoded, request(), "additive request decode")?;
    Ok(())
}

#[test]
fn canonical_json_is_stable_across_object_key_order() -> Result<(), Box<dyn std::error::Error>> {
    let left = json!({"z": [{"b": 2, "a": 1}], "a": {"d": 4, "c": 3}});
    let right = json!({"a": {"c": 3, "d": 4}, "z": [{"a": 1, "b": 2}]});

    check_eq(canonical_json(&left)?, canonical_json(&right)?, "canonical key ordering")?;
    check_eq(
        canonical_json(&left)?,
        r#"{"a":{"c":3,"d":4},"z":[{"a":1,"b":2}]}"#.to_owned(),
        "canonical bytes",
    )?;
    Ok(())
}

#[test]
fn evidence_and_projection_cover_frozen_public_state() -> Result<(), Box<dyn std::error::Error>> {
    let lineage = LineageDescriptor {
        executor_id: "reviewer-001".to_owned(),
        provider_family: Some("provider-family".to_owned()),
        model: Some("model-pin".to_owned()),
        invocation_id: "invocation-001".to_owned(),
    };
    let evidence = EvidenceBinding {
        session_id: SessionId::new(SESSION_ID),
        transaction_revision: Revision::new(REVISION_AFTER),
        accepted_diff_digest: AcceptedDiffDigest::new(DIFF_DIGEST),
        worktree_fingerprint: Fingerprint::new(FINGERPRINT),
        claim_set: vec![Claim::new("tests-pass")],
        reviewer_lineage: lineage.clone(),
        evidence_references: vec![EvidenceReference::new("evidence-001")],
        freshness: EvidenceFreshness::Fresh,
    };
    let projection = PublicSessionProjection {
        protocol_version: ProtocolVersion::V1,
        session_id: SessionId::new(SESSION_ID),
        repository_id: RepositoryId::new(REPOSITORY_ID),
        transaction_revision: Revision::new(REVISION_AFTER),
        lifecycle: SessionLifecycle::ProofPending,
        stage: OperatingStage::Verify,
        next_actions: vec![NextAction {
            operation: OperationId::new("approve"),
            summary: "Approve fresh proof".to_owned(),
        }],
        executor_capability: ExecutorCapabilityStatus::Admitted,
        proof_freshness: ProofFreshness::Fresh,
        publication: PublicationStatus::NotStarted,
        recovery: RecoveryStatus::NotRequired,
        quarantine: QuarantineStatus::Clear,
        canon_outcome_sync: CanonOutcomeSyncStatus::NotRequired,
    };

    check_eq(
        serde_json::from_value::<EvidenceBinding>(serde_json::to_value(&evidence)?)?,
        evidence,
        "evidence round trip",
    )?;
    check_eq(
        serde_json::from_value::<PublicSessionProjection>(serde_json::to_value(&projection)?)?,
        projection,
        "projection round trip",
    )?;
    Ok(())
}

#[test]
fn stable_reason_codes_have_exact_wire_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (ReasonCode::IdempotencyConflict, "idempotency_conflict"),
        (ReasonCode::StateRevisionMismatch, "state_revision_mismatch"),
        (ReasonCode::ExecutorAlreadyInFlight, "executor_already_in_flight"),
        (ReasonCode::ExecutorFencingTokenStale, "executor_fencing_token_stale"),
        (ReasonCode::ExecutorTerminationUnconfirmed, "executor_termination_unconfirmed"),
        (ReasonCode::ExecutorCapabilityDenied, "executor_capability_denied"),
        (ReasonCode::ExecutorBoundaryViolated, "executor_boundary_violated"),
        (
            ReasonCode::UncommittedCandidateRequiresValidation,
            "uncommitted_candidate_requires_validation",
        ),
        (ReasonCode::ApprovalStale, "approval_stale"),
        (ReasonCode::ProofStale, "proof_stale"),
        (ReasonCode::RepositoryIdentityMismatch, "repository_identity_mismatch"),
        (ReasonCode::AuthoritativeWorktreeDirty, "authoritative_worktree_dirty"),
        (ReasonCode::PublicationLockHeld, "publication_lock_held"),
        (ReasonCode::PublicationRebaseRequired, "publication_rebase_required"),
        (ReasonCode::PublicationPreconditionFailed, "publication_precondition_failed"),
        (ReasonCode::PublicationRecoveryRequired, "publication_recovery_required"),
        (ReasonCode::RepositoryQuarantined, "repository_quarantined"),
        (ReasonCode::CanonOutcomeSyncPending, "canon_outcome_sync_pending"),
        (ReasonCode::CanonOutcomeSyncFailed, "canon_outcome_sync_failed"),
        (ReasonCode::UnsupportedGitOrFilesystemState, "unsupported_git_or_filesystem_state"),
    ];

    for (reason, wire_value) in cases {
        check_eq(serde_json::to_value(reason)?, json!(wire_value), "stable reason code")?;
    }
    Ok(())
}

#[test]
fn public_adapter_descriptors_remain_proposal_only() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor = AdapterDescriptor {
        protocol_version: ProtocolVersion::V1,
        identity: AdapterIdentity {
            adapter_id: "speckit".to_owned(),
            version: "0.90.0".to_owned(),
            executable_digest: "sha256:adapter".to_owned(),
        },
        protocol_line: ContractLine::new("framework-adapter-v1"),
        transport: AdapterTransport::OneShotLocalSubprocess,
        operations: vec![OperationDescriptor {
            operation: OperationId::new("execute_stage"),
            mutating: true,
        }],
        stages: vec![StageId::new("requirements")],
        requested_capabilities: vec![CapabilityDescriptor {
            capability: "workspace_write".to_owned(),
            disposition: CapabilityDisposition::RequiresAdmission,
        }],
    };
    let proposal = AdapterProposal {
        invocation_id: "inv-001".to_owned(),
        authority: FrameworkAdapterAuthority::ProposalOnly,
        summary: "Proposed requirements packet".to_owned(),
        mutations: vec![MutationProposal {
            path: "spec.md".to_owned(),
            content_digest: "sha256:content".to_owned(),
        }],
        artifacts: vec![],
        evidence: vec![],
        diagnostics: vec![],
        next_actions: vec![],
    };
    let route = RouteDescriptor {
        route_id: "route-001".to_owned(),
        stage: StageId::new("requirements"),
        executor: descriptor.identity.adapter_id.clone(),
        lineage: LineageDescriptor {
            executor_id: "speckit".to_owned(),
            provider_family: None,
            model: None,
            invocation_id: "inv-001".to_owned(),
        },
    };

    check_eq(
        serde_json::from_value::<AdapterDescriptor>(serde_json::to_value(&descriptor)?)?,
        descriptor,
        "adapter descriptor round trip",
    )?;
    check_eq(
        serde_json::to_value(&proposal)?["authority"].clone(),
        json!("proposal_only"),
        "proposal authority",
    )?;
    check_eq(
        serde_json::from_value::<RouteDescriptor>(serde_json::to_value(&route)?)?,
        route,
        "route round trip",
    )?;
    Ok(())
}
