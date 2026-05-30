use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::workflow::{WorkflowLifecycleState, WorkflowPhase};

use crate::workspace_fixture::{
    temp_workflow_follow_through_approval_workspace, temp_workflow_follow_through_workspace,
    terminal_text,
};

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    FileSessionStore::for_workspace(workspace).load().unwrap().unwrap()
}

#[test]
fn workflow_run_completes_review_and_govern_when_follow_through_is_ready() {
    let workspace = temp_workflow_follow_through_workspace("workflow-follow-through-success");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "governed-delivery", "--goal", "Fix the failing checkout flow"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: governed-delivery"), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("latest_review_outcome: accepted"), "{text}");
    assert!(text.contains("latest_governance_state: governed_ready"), "{text}");
    assert!(text.contains("execution_condition: terminal - work completed successfully"), "{text}");

    let record = load_session_record(&workspace);
    let progress = record.workflow_progress.expect("workflow progress should exist");
    assert_eq!(progress.lifecycle_state, WorkflowLifecycleState::Completed);
    assert_eq!(progress.current_phase, Some(WorkflowPhase::Inspect));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Run));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Review));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Govern));
    assert_eq!(record.latest_status, SessionStatus::Succeeded);
}

#[test]
fn workflow_run_advances_review_and_pauses_at_govern_when_approval_is_pending() {
    let workspace =
        temp_workflow_follow_through_approval_workspace("workflow-follow-through-pending");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "governed-delivery", "--goal", "Fix the failing checkout flow"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: governed-delivery"), "{text}");
    assert!(text.contains("workflow_phase: govern"), "{text}");
    assert!(
        text.contains("routing: native (goal_plan) - goal plan is ready for native execution"),
        "{text}"
    );
    assert!(text.contains("latest_governance_state: awaiting_approval"), "{text}");
    assert!(text.contains("execution_condition: waiting - governance approval is still pending before execution can continue"), "{text}");
    assert!(text.contains("next_command: boundline workflow resume --workspace "), "{text}");

    let record = load_session_record(&workspace);
    let progress = record.workflow_progress.expect("workflow progress should exist");
    assert_eq!(progress.lifecycle_state, WorkflowLifecycleState::Paused);
    assert_eq!(progress.current_phase, Some(WorkflowPhase::Govern));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Run));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Review));
    assert!(!progress.completed_phases.contains(&WorkflowPhase::Govern));
}

#[test]
fn workflow_resume_finishes_after_governance_approval_refresh() {
    let workspace =
        temp_workflow_follow_through_approval_workspace("workflow-follow-through-resume");

    let start = run_boundline_in(
        &workspace,
        &["workflow", "run", "governed-delivery", "--goal", "Fix the failing checkout flow"],
    );
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let output = run_boundline_in(&workspace, &["workflow", "resume", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: governed-delivery"), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("latest_governance_state: governed_ready"), "{text}");
    assert!(text.contains("execution_condition: terminal - work completed successfully"), "{text}");

    let record = load_session_record(&workspace);
    let progress = record.workflow_progress.expect("workflow progress should exist");
    assert_eq!(progress.lifecycle_state, WorkflowLifecycleState::Completed);
    assert_eq!(progress.current_phase, Some(WorkflowPhase::Inspect));
    assert!(progress.completed_phases.contains(&WorkflowPhase::Govern));
    assert_eq!(record.latest_status, SessionStatus::Succeeded);
}
