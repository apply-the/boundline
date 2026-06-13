use boundline::cli::output::render_session_status;
use boundline::domain::completion_verification::{
    CompletionClaim, CompletionClaimKind, CompletionClaimSource, CompletionRequiredAction,
    CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationFindingSeverity, CompletionVerificationScope,
    CompletionVerificationState,
};
use boundline::domain::session::SessionStatusView;

#[test]
fn session_status_renders_completion_verification_projection() {
    let output = render_session_status(&SessionStatusView {
        session_id: "session-1".to_string(),
        workspace_ref: "/tmp/workspace".to_string(),
        completion_verification_state: Some(CompletionVerificationState::Blocked),
        completion_claim: Some(CompletionClaim {
            claim_id: "claim-1".to_string(),
            kind: CompletionClaimKind::BugFixed,
            scope: CompletionVerificationScope::Task,
            source: CompletionClaimSource::RuntimeInference,
            confidence: None,
            summary: "bug fix is ready for closeout".to_string(),
            supporting_signals: vec!["changed_files".to_string()],
        }),
        completion_blocked_claims: Some(vec![CompletionClaimKind::BugFixed]),
        completion_evidence_refs: Some(vec!["trace:proof-1".to_string()]),
        completion_verification_findings: Some(vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::StaleProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: "proof is stale".to_string(),
            proof_ref: Some("proof-1".to_string()),
            task_id: None,
            changed_paths: vec!["src/lib.rs".to_string()],
            required_action: CompletionRequiredAction::RerunProof,
        }]),
        ..SessionStatusView::default()
    });

    assert!(output.contains("completion_verification_state: blocked"), "{output}");
    assert!(output.contains("completion_claim_kind: bug_fixed"), "{output}");
    assert!(output.contains("completion_blocked_claims: bug_fixed"), "{output}");
    assert!(output.contains("completion_evidence_refs: trace:proof-1"), "{output}");
    assert!(output.contains("completion_verification_required_action: rerun_proof"), "{output}");
}
