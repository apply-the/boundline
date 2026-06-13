use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::inspect::execute_inspect;
use boundline::cli::orchestrate::OrchestrateCommandReport;
use boundline::cli::output::render_human_orchestrate_report;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};
use boundline::domain::completion_verification::{
    ChildVerificationInput, CompletionRequiredAction, CompletionVerificationFinding,
    CompletionVerificationFindingKind, CompletionVerificationFindingSeverity,
    CompletionVerificationProjection, CompletionVerificationScope, CompletionVerificationState,
    aggregate_child_verification,
};
use boundline::domain::session::SessionStatus;

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn stage_and_run_surfaces_report_blocked_child_verification_without_hiding_failures() {
    let workspace = temp_fixture_workspace("completion-verification-stage-run-flow");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    execute_run(Some(&workspace)).unwrap();

    let mut session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let task = session.active_task.as_mut().expect("active task should exist after blocked run");
    let stage_projection = aggregate_child_verification(
        CompletionVerificationScope::Stage,
        &[
            ChildVerificationInput {
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
            },
            ChildVerificationInput {
                task_id: "T-014".to_string(),
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
            },
            ChildVerificationInput {
                task_id: "T-019".to_string(),
                required: true,
                deferred_reason: None,
                skipped_reason: None,
                projection: None,
            },
            ChildVerificationInput {
                task_id: "T-022".to_string(),
                required: true,
                deferred_reason: Some("explicitly deferred".to_string()),
                skipped_reason: None,
                projection: None,
            },
        ],
    );
    task.context.set_completion_verification_projection(&stage_projection).unwrap();
    session.latest_status = SessionStatus::Blocked;
    FileSessionStore::for_workspace(&workspace).persist(&session).unwrap();

    let status_report = execute_status(Some(&workspace)).unwrap();
    let inspect_report = execute_inspect(None, Some(&workspace), None, false).unwrap();
    let orchestrate_text = render_human_orchestrate_report(&OrchestrateCommandReport {
        exit_status: boundline::cli::CommandExitStatus::NonSuccess,
        terminal_output: String::new(),
        trace_location: status_report.trace_location.clone(),
        session_status: status_report.session_status.clone(),
        trace_summary: None,
        events: Vec::new(),
    });

    let status_text = status_report.terminal_output;
    assert!(status_text.contains("completion_verification_state: blocked"), "{status_text}");
    assert!(
        status_text.contains(
            "completion_verification_finding: stale_child_proof | blocking | proof is stale"
        ),
        "{status_text}"
    );
    assert!(
        status_text.contains("completion_verification_finding: missing_child_proof | blocking | required child `T-019` is missing proof"),
        "{status_text}"
    );
    assert!(
        inspect_report.terminal_output.contains("completion_verification_task_id: T-014"),
        "{}",
        inspect_report.terminal_output
    );
    assert!(
        inspect_report.terminal_output.contains("completion_verification_task_id: T-019"),
        "{}",
        inspect_report.terminal_output
    );
    assert!(
        orchestrate_text.contains("Completion Verification State: blocked"),
        "{orchestrate_text}"
    );
    assert!(orchestrate_text.contains("Required Action: rerun_proof"), "{orchestrate_text}");
}
