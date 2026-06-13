use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::inspect::execute_inspect;
use boundline::cli::orchestrate::OrchestrateCommandReport;
use boundline::cli::output::render_human_orchestrate_report;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};
use boundline::domain::completion_verification::{
    CompletionClaimKind, CompletionVerificationFindingKind, CompletionVerificationState,
};

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn blocked_closeout_projects_proof_required_status_fields() {
    let workspace = temp_fixture_workspace("completion-verification-contract-proof-required");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run_report = execute_run(Some(&workspace)).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right + 1\n}\n",
    )
    .unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();
    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let status_view = status_report
        .session_status
        .clone()
        .expect("status command should return a session projection");

    assert_eq!(run_report.exit_status, boundline::cli::CommandExitStatus::Succeeded);
    assert_eq!(session.latest_status, boundline::domain::session::SessionStatus::Blocked);
    assert_eq!(
        status_view.completion_verification_state,
        Some(CompletionVerificationState::ProofRequired)
    );
    assert_eq!(
        status_view.completion_claim.as_ref().map(|claim| claim.kind),
        Some(CompletionClaimKind::TestsPass)
    );
    assert_eq!(
        status_view.completion_blocked_claims.as_ref(),
        Some(&vec![CompletionClaimKind::TestsPass])
    );
    assert!(status_view.completion_verification_findings.as_ref().is_some_and(|findings| {
        findings.iter().any(|finding| finding.kind == CompletionVerificationFindingKind::StaleProof)
    }));
    let serialized = serde_json::to_value(&status_view).expect("status view should serialize");
    let completion_claim = serialized["completion_claim"].clone();
    assert!(completion_claim.get("approval_state").is_none(), "{serialized}");
    assert!(completion_claim.get("packet_readiness").is_none(), "{serialized}");
    assert!(
        status_view
            .next_command
            .as_deref()
            .is_some_and(|command| command.contains("cargo test --quiet"))
    );
}

#[test]
fn blocked_closeout_projects_unsupported_claim_findings() {
    let workspace = temp_fixture_workspace("completion-verification-contract-unsupported-claim");

    execute_goal(
        Some(&workspace),
        Some("validate the production migration"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let mut session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let goal_plan = session.goal_plan.as_mut().expect("goal plan should exist");
    let task = goal_plan.tasks.last_mut().expect("goal plan should contain a verification task");
    task.expected_outcome = Some("migration remains valid".to_string());
    FileSessionStore::for_workspace(&workspace).persist(&session).unwrap();

    let run_report = execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();
    let status_view = status_report
        .session_status
        .clone()
        .expect("status command should return a session projection");

    assert_eq!(run_report.exit_status, boundline::cli::CommandExitStatus::NonSuccess);
    assert_eq!(
        status_view.completion_verification_state,
        Some(CompletionVerificationState::Blocked)
    );
    assert_eq!(
        status_view.completion_claim.as_ref().map(|claim| claim.kind),
        Some(CompletionClaimKind::MigrationValid)
    );
    assert!(status_view.completion_verification_findings.as_ref().is_some_and(|findings| {
        findings.iter().any(|finding| {
            finding.kind == CompletionVerificationFindingKind::MissingProof
                && finding.message.contains("no proving command exists")
        })
    }));
}

#[test]
fn inspect_and_orchestrate_surfaces_preserve_completion_verification_projection() {
    let workspace = temp_fixture_workspace("completion-verification-contract-inspect-orchestrate");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    execute_run(Some(&workspace)).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right + 1\n}\n",
    )
    .unwrap();

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

    assert!(
        inspect_report.terminal_output.contains("completion_verification_state: proof_required"),
        "{}",
        inspect_report.terminal_output
    );
    assert!(
        inspect_report
            .terminal_output
            .contains("completion_verification_required_action: rerun_proof"),
        "{}",
        inspect_report.terminal_output
    );
    assert!(
        orchestrate_text.contains("Completion Verification State: proof_required"),
        "{orchestrate_text}"
    );
    assert!(orchestrate_text.contains("Blocked Claims: tests_pass"), "{orchestrate_text}");
}
