use boundline::domain::completion_verification::{
    ChildVerificationInput, ClaimInferenceConfidence, CompletionClaim, CompletionClaimKind,
    CompletionClaimSource, CompletionRequiredAction, CompletionVerificationError,
    CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationFindingSeverity, CompletionVerificationProjection,
    CompletionVerificationScope, CompletionVerificationState, aggregate_child_verification,
};

#[test]
fn completion_verification_vocabulary_stays_stable() {
    assert_eq!(CompletionVerificationState::Ready.as_str(), "ready");
    assert_eq!(CompletionVerificationState::ProofRequired.as_str(), "proof_required");
    assert_eq!(CompletionClaimKind::BuildClean.as_str(), "build_clean");
    assert_eq!(CompletionClaimSource::OperatorOverride.as_str(), "operator_override");
    assert_eq!(CompletionRequiredAction::ResolveConflict.as_str(), "resolve_conflict");
    assert_eq!(CompletionVerificationFindingKind::FailedChildProof.as_str(), "failed_child_proof");
    assert_eq!(CompletionVerificationFindingSeverity::Warning.as_str(), "warning");
}

#[test]
fn explicit_claims_reject_inference_confidence() {
    let claim = CompletionClaim {
        claim_id: "claim-explicit".to_string(),
        kind: CompletionClaimKind::TestsPass,
        scope: CompletionVerificationScope::Task,
        source: CompletionClaimSource::ExplicitMetadata,
        confidence: Some(ClaimInferenceConfidence::High),
        summary: "tests passed".to_string(),
        supporting_signals: Vec::new(),
    };

    let result = claim.validate();
    assert_eq!(
        result,
        Err(CompletionVerificationError::InvalidClaim(
            "explicit_metadata claims must not carry inference confidence".to_string()
        ))
    );
}

#[test]
fn runtime_inference_claims_require_confidence() {
    let claim = CompletionClaim {
        claim_id: "claim-inferred".to_string(),
        kind: CompletionClaimKind::BugFixed,
        scope: CompletionVerificationScope::Task,
        source: CompletionClaimSource::RuntimeInference,
        confidence: None,
        summary: "bug fix is ready".to_string(),
        supporting_signals: vec!["changed_files".to_string()],
    };

    let result = claim.validate();
    assert_eq!(
        result,
        Err(CompletionVerificationError::InvalidClaim(
            "runtime_inference claims require confidence".to_string()
        ))
    );
}

#[test]
fn blocked_projection_requires_a_finding() {
    let projection = CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::Blocked,
        scope: CompletionVerificationScope::Task,
        claim: None,
        completion_blocked_claims: vec![CompletionClaimKind::BuildClean],
        completion_evidence_refs: Vec::new(),
        completion_verification_findings: Vec::new(),
        child_summary: None,
    };

    let result = projection.validate();
    assert_eq!(
        result,
        Err(CompletionVerificationError::InvalidProjection(
            "blocking states require at least one finding".to_string()
        ))
    );
}

#[test]
fn ready_projection_rejects_blocking_findings() {
    let projection = CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::Ready,
        scope: CompletionVerificationScope::Task,
        claim: None,
        completion_blocked_claims: Vec::new(),
        completion_evidence_refs: vec!["trace:proof-1".to_string()],
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::StaleProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: "proof is stale".to_string(),
            proof_ref: Some("proof-1".to_string()),
            task_id: None,
            changed_paths: vec!["src/lib.rs".to_string()],
            required_action: CompletionRequiredAction::RerunProof,
        }],
        child_summary: None,
    };

    let result = projection.validate();
    assert_eq!(
        result,
        Err(CompletionVerificationError::InvalidProjection(
            "ready projections must not carry blocking findings".to_string()
        ))
    );
}

#[test]
fn parent_aggregation_blocks_when_required_children_are_stale_or_missing() {
    let ready_child = ChildVerificationInput {
        task_id: "T-001".to_string(),
        required: true,
        deferred_reason: None,
        skipped_reason: None,
        projection: Some(CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::Ready,
            scope: CompletionVerificationScope::Task,
            claim: None,
            completion_blocked_claims: Vec::new(),
            completion_evidence_refs: vec!["evidence-1".to_string()],
            completion_verification_findings: Vec::new(),
            child_summary: None,
        }),
    };
    let stale_child = ChildVerificationInput {
        task_id: "T-014".to_string(),
        required: true,
        deferred_reason: None,
        skipped_reason: None,
        projection: Some(CompletionVerificationProjection {
            completion_verification_state: CompletionVerificationState::ProofRequired,
            scope: CompletionVerificationScope::Task,
            claim: None,
            completion_blocked_claims: vec![CompletionClaimKind::TestsPass],
            completion_evidence_refs: Vec::new(),
            completion_verification_findings: vec![CompletionVerificationFinding {
                kind: CompletionVerificationFindingKind::StaleProof,
                severity: CompletionVerificationFindingSeverity::Blocking,
                message: "proof is stale".to_string(),
                proof_ref: Some("proof-14".to_string()),
                task_id: Some("T-014".to_string()),
                changed_paths: vec!["src/lib.rs".to_string()],
                required_action: CompletionRequiredAction::RerunProof,
            }],
            child_summary: None,
        }),
    };
    let missing_child = ChildVerificationInput {
        task_id: "T-019".to_string(),
        required: true,
        deferred_reason: None,
        skipped_reason: None,
        projection: None,
    };

    let projection = aggregate_child_verification(
        CompletionVerificationScope::Stage,
        &[ready_child, stale_child, missing_child],
    );

    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Blocked);
    assert_eq!(projection.scope, CompletionVerificationScope::Stage);
    let summary = projection.child_summary.expect("parent aggregation should emit summary");
    assert_eq!(summary.ready_children, 1);
    assert_eq!(summary.blocked_children, 2);
    assert_eq!(summary.stale_children, 1);
    assert_eq!(summary.missing_proof_children, 1);
    assert!(projection.completion_verification_findings.iter().any(|finding| {
        finding.kind == CompletionVerificationFindingKind::StaleChildProof
            && finding.task_id.as_deref() == Some("T-014")
    }));
    assert!(projection.completion_verification_findings.iter().any(|finding| {
        finding.kind == CompletionVerificationFindingKind::MissingChildProof
            && finding.task_id.as_deref() == Some("T-019")
    }));
}

#[test]
fn parent_aggregation_ignores_optional_skipped_and_deferred_children() {
    let optional_child = ChildVerificationInput {
        task_id: "T-optional".to_string(),
        required: false,
        deferred_reason: None,
        skipped_reason: Some("outside current slice".to_string()),
        projection: None,
    };
    let deferred_child = ChildVerificationInput {
        task_id: "T-deferred".to_string(),
        required: true,
        deferred_reason: Some("awaiting external migration window".to_string()),
        skipped_reason: None,
        projection: None,
    };

    let projection = aggregate_child_verification(
        CompletionVerificationScope::Run,
        &[optional_child, deferred_child],
    );

    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Ready);
    let summary = match projection.child_summary {
        Some(summary) => summary,
        None => panic!("expected child summary"),
    };
    assert_eq!(summary.skipped_children, 1);
    assert_eq!(summary.deferred_children, 1);
    assert_eq!(summary.blocked_children, 0);
}
