use boundline::domain::completion_verification::{
    ChildVerificationInput, CompletionRequiredAction, CompletionVerificationFinding,
    CompletionVerificationFindingKind, CompletionVerificationFindingSeverity,
    CompletionVerificationProjection, CompletionVerificationScope, CompletionVerificationState,
    aggregate_child_verification,
};

#[test]
fn parent_scope_projection_uses_blocked_child_findings_without_canon_readiness_fields() {
    let projection = aggregate_child_verification(
        CompletionVerificationScope::Run,
        &[ChildVerificationInput {
            task_id: "T-019".to_string(),
            required: true,
            deferred_reason: None,
            skipped_reason: None,
            projection: Some(CompletionVerificationProjection {
                completion_verification_state: CompletionVerificationState::ProofRequired,
                scope: CompletionVerificationScope::Task,
                claim: None,
                completion_blocked_claims: Vec::new(),
                completion_evidence_refs: Vec::new(),
                completion_verification_findings: vec![CompletionVerificationFinding {
                    kind: CompletionVerificationFindingKind::MissingProof,
                    severity: CompletionVerificationFindingSeverity::Blocking,
                    message: "no proving command exists".to_string(),
                    proof_ref: None,
                    task_id: Some("T-019".to_string()),
                    changed_paths: Vec::new(),
                    required_action: CompletionRequiredAction::RunProof,
                }],
                child_summary: None,
            }),
        }],
    );

    assert_eq!(projection.scope, CompletionVerificationScope::Run);
    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Blocked);
    assert!(projection.completion_verification_findings.iter().any(|finding| {
        finding.kind == CompletionVerificationFindingKind::MissingChildProof
            && finding.task_id.as_deref() == Some("T-019")
    }));

    let serialized = serde_json::to_value(&projection).expect("projection should serialize");
    assert!(serialized.get("approval_state").is_none(), "{serialized}");
    assert!(serialized.get("packet_readiness").is_none(), "{serialized}");
    assert!(serialized.get("readiness").is_none(), "{serialized}");
}
