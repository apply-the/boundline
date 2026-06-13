use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::CommandExitStatus;
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};
use boundline::domain::completion_verification::{
    CompletionRequiredAction, CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationFindingSeverity, CompletionVerificationProjection,
    CompletionVerificationScope, CompletionVerificationState,
};
use boundline::domain::session::SessionStatus;
use std::fs;

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn run_executes_a_fresh_proof_command_before_task_closeout() {
    let workspace = temp_fixture_workspace("completion-verification-integration-proof-required");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run_report = execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();
    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();

    assert_eq!(run_report.exit_status, CommandExitStatus::Succeeded);
    assert_eq!(session.latest_status, SessionStatus::Succeeded);
    assert_eq!(
        session.active_task.as_ref().map(|task| task.status),
        Some(boundline::domain::task::TaskStatus::Succeeded)
    );
    assert!(
        run_report.terminal_output.contains("completion_verification_state: ready"),
        "{}",
        run_report.terminal_output
    );
    assert!(
        run_report.terminal_output.contains("completion_evidence_refs:"),
        "{}",
        run_report.terminal_output
    );
    assert!(
        status_report.terminal_output.contains("completion_claim_kind: tests_pass"),
        "{}",
        status_report.terminal_output
    );
}

#[test]
fn run_blocks_task_closeout_when_no_proving_command_matches_the_claim() {
    let workspace = temp_fixture_workspace("completion-verification-integration-no-proof-command");

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
    let persisted = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let projection = persisted
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten())
        .expect("blocked projection should persist on the task");

    assert_eq!(run_report.exit_status, CommandExitStatus::NonSuccess);
    assert_eq!(persisted.latest_status, SessionStatus::Blocked);
    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Blocked);
    assert!(
        projection
            .completion_verification_findings
            .iter()
            .any(|finding| finding.message.contains("no proving command exists"))
    );
    assert!(
        run_report.terminal_output.contains(
            "completion_verification_finding: missing_proof | blocking | no proving command exists"
        ),
        "{}",
        run_report.terminal_output
    );
}

#[test]
fn rerun_executes_selected_proof_and_unblocks_closeout_when_the_command_passes() {
    let workspace = temp_fixture_workspace("completion-verification-integration-proof-pass");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let first_run = execute_run(Some(&workspace)).unwrap();
    let persisted = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();
    let projection = persisted
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten())
        .expect("ready projection should persist on the task");

    assert_eq!(first_run.exit_status, CommandExitStatus::Succeeded);
    assert_eq!(persisted.latest_status, SessionStatus::Succeeded);
    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Ready);
    assert!(!projection.completion_evidence_refs.is_empty());
    assert!(
        status_report
            .session_status
            .as_ref()
            .and_then(|view| view.latest_governance_packet_ref.as_ref())
            .is_none()
    );
    assert!(
        status_report
            .session_status
            .as_ref()
            .and_then(|view| view.latest_governance_approval.as_ref())
            .is_none()
    );
    assert!(
        projection
            .completion_verification_findings
            .iter()
            .all(|finding| finding.kind != CompletionVerificationFindingKind::FailedProof)
    );
    assert!(
        first_run.terminal_output.contains("completion_verification_state: ready"),
        "{}",
        first_run.terminal_output
    );
}

#[test]
fn run_keeps_closeout_blocked_when_the_fresh_proof_command_fails() {
    let workspace = temp_fixture_workspace("completion-verification-integration-proof-fail");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let first_run = execute_run(Some(&workspace)).unwrap();
    assert_eq!(first_run.exit_status, CommandExitStatus::Succeeded);

    let mut session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let task = session.active_task.as_mut().expect("task should persist after a successful proof");
    let claim = task
        .context
        .completion_claim()
        .ok()
        .flatten()
        .expect("claim should persist after a successful proof");
    let mut selection = task
        .context
        .completion_proof_selection()
        .ok()
        .flatten()
        .expect("proof selection should persist after a successful proof");
    selection.command_line = "/bin/sh -c false".to_string();
    task.set_completion_proof_selection(&selection).unwrap();
    task.set_completion_verification_projection(&CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::ProofRequired,
        scope: CompletionVerificationScope::Task,
        claim: Some(claim.clone()),
        completion_blocked_claims: vec![claim.kind],
        completion_evidence_refs: Vec::new(),
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::MissingProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: "fresh proof command has not been executed in the current workspace state"
                .to_string(),
            proof_ref: Some(selection.command_ref.clone()),
            task_id: Some(task.id.clone()),
            changed_paths: Vec::new(),
            required_action: CompletionRequiredAction::RunProof,
        }],
        child_summary: None,
    })
    .unwrap();
    FileSessionStore::for_workspace(&workspace).persist(&session).unwrap();

    let run_report = execute_run(Some(&workspace)).unwrap();
    let persisted = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let projection_value = persisted
        .active_task
        .as_ref()
        .and_then(|task| task.context.state.get("completion_verification_projection"))
        .cloned()
        .expect("failed proof projection should persist on the task");
    let projection: CompletionVerificationProjection =
        serde_json::from_value(projection_value).expect("projection should deserialize");

    assert_eq!(run_report.exit_status, CommandExitStatus::NonSuccess);
    assert_eq!(persisted.latest_status, SessionStatus::Blocked);
    assert_eq!(projection.completion_verification_state, CompletionVerificationState::Failed);
    assert!(
        projection
            .completion_verification_findings
            .iter()
            .any(|finding| { finding.kind == CompletionVerificationFindingKind::FailedProof })
    );
    assert!(
        run_report.terminal_output.contains("completion_verification_state: failed"),
        "{}",
        run_report.terminal_output
    );
}

#[test]
fn medium_confidence_claim_blocks_closeout_until_the_operator_confirms_the_inference()
-> Result<(), String> {
    let workspace =
        temp_fixture_workspace("completion-verification-integration-confirm-medium-claim");

    execute_goal(Some(&workspace), Some("keep the build clean"), &[], None, None, None, None)
        .map_err(|error| error.to_string())?;
    write_validation_command(&workspace, "cargo", &["build", "--quiet"])?;
    execute_plan(Some(&workspace), Some("bug-fix"), false).map_err(|error| error.to_string())?;

    let run_report = execute_run(Some(&workspace)).map_err(|error| error.to_string())?;
    let persisted = FileSessionStore::for_workspace(&workspace)
        .load()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "missing persisted session".to_string())?;
    let projection = persisted
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten())
        .ok_or_else(|| "missing completion verification projection".to_string())?;

    if run_report.exit_status != CommandExitStatus::NonSuccess {
        return Err(format!(
            "expected non-success closeout while confirmation is required, got {:?}",
            run_report.exit_status
        ));
    }
    if persisted.latest_status != SessionStatus::Blocked {
        return Err(format!("expected blocked session, got {:?}", persisted.latest_status));
    }
    if projection.completion_verification_state != CompletionVerificationState::Blocked {
        return Err(format!(
            "expected blocked projection, got {:?}",
            projection.completion_verification_state
        ));
    }
    if !run_report.terminal_output.contains("completion_claim_source: runtime_inference") {
        return Err(run_report.terminal_output);
    }
    if !run_report
        .terminal_output
        .contains("completion_verification_required_action: confirm_claim")
    {
        return Err(run_report.terminal_output);
    }
    if !run_report.terminal_output.contains("selected proof command `cargo build --quiet`") {
        return Err(run_report.terminal_output);
    }

    Ok(())
}

#[test]
fn conflicting_goal_and_expected_outcome_signals_block_closeout_for_resolution()
-> Result<(), String> {
    let workspace = temp_fixture_workspace("completion-verification-integration-claim-conflict");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .map_err(|error| error.to_string())?;
    execute_plan(Some(&workspace), Some("bug-fix"), false).map_err(|error| error.to_string())?;

    let mut session = FileSessionStore::for_workspace(&workspace)
        .load()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "missing planned session".to_string())?;
    let goal_plan = session.goal_plan.as_mut().ok_or_else(|| "missing goal plan".to_string())?;
    let task = goal_plan.tasks.last_mut().ok_or_else(|| "missing verification task".to_string())?;
    task.expected_outcome = Some("migration remains valid".to_string());
    FileSessionStore::for_workspace(&workspace)
        .persist(&session)
        .map_err(|error| error.to_string())?;

    let run_report = execute_run(Some(&workspace)).map_err(|error| error.to_string())?;
    let persisted = FileSessionStore::for_workspace(&workspace)
        .load()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "missing persisted session".to_string())?;
    let projection = persisted
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten())
        .ok_or_else(|| "missing completion verification projection".to_string())?;

    if run_report.exit_status != CommandExitStatus::NonSuccess {
        return Err(format!(
            "expected non-success closeout for conflicting claim signals, got {:?}",
            run_report.exit_status
        ));
    }
    if persisted.latest_status != SessionStatus::Blocked {
        return Err(format!("expected blocked session, got {:?}", persisted.latest_status));
    }
    if projection.completion_verification_state != CompletionVerificationState::Blocked {
        return Err(format!(
            "expected blocked projection, got {:?}",
            projection.completion_verification_state
        ));
    }
    if !projection
        .completion_verification_findings
        .iter()
        .any(|finding| finding.kind == CompletionVerificationFindingKind::ClaimConflict)
    {
        return Err(format!(
            "expected claim_conflict finding, got {:?}",
            projection.completion_verification_findings
        ));
    }
    if !run_report
        .terminal_output
        .contains("completion_verification_required_action: resolve_conflict")
    {
        return Err(run_report.terminal_output);
    }

    Ok(())
}

fn write_validation_command(
    workspace: &std::path::Path,
    program: &str,
    args: &[&str],
) -> Result<(), String> {
    let execution_path = workspace.join(".boundline/execution.json");
    let current = fs::read_to_string(&execution_path).map_err(|error| error.to_string())?;
    let mut value: serde_json::Value =
        serde_json::from_str(&current).map_err(|error| error.to_string())?;
    value["validation_command"] = serde_json::json!({
        "program": program,
        "args": args,
    });
    fs::write(
        execution_path,
        serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}
