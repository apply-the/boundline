use std::fs;

use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::workflow::{WorkflowLifecycleState, WorkflowPhase};

use crate::workspace_fixture::{temp_workflow_follow_through_blocked_workspace, terminal_text};

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    serde_json::from_slice(&fs::read(workspace.join(".boundline").join("session.json")).unwrap())
        .unwrap()
}

#[test]
fn workflow_run_blocks_when_govern_phase_has_no_bounded_governance_evidence() {
    let workspace =
        temp_workflow_follow_through_blocked_workspace("workflow-follow-through-blocked");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "blocked-delivery", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("workflow: blocked-delivery"), "{text}");
    assert!(text.contains("workflow_phase: govern"), "{text}");
    assert!(text.contains("execution_condition: blocked - workflow govern phase requires governance evidence from the active session"), "{text}");
    assert!(text.contains("next_command: boundline workflow inspect --workspace "), "{text}");

    let record = load_session_record(&workspace);
    let progress = record.workflow_progress.expect("workflow progress should exist");
    assert_eq!(progress.lifecycle_state, WorkflowLifecycleState::Blocked);
    assert_eq!(progress.current_phase, Some(WorkflowPhase::Govern));
}
