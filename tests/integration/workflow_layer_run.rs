use std::fs;

use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::workflow::WorkflowPhase;

use crate::workspace_fixture::{
    temp_invalid_workflow_layer_workspace, temp_workflow_layer_workspace, terminal_text,
};

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
fn workflow_run_creates_a_session_and_persists_workflow_progress() {
    let workspace = temp_workflow_layer_workspace("workflow-layer-run");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");

    let record = load_session_record(&workspace);
    record.validate().unwrap();
    let workflow_progress = record.workflow_progress.expect("workflow progress should be stored");
    assert_eq!(workflow_progress.workflow_name, "default");
    assert_eq!(workflow_progress.current_phase, Some(WorkflowPhase::Inspect));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Capture));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Plan));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Run));
    assert_eq!(record.latest_status, SessionStatus::Succeeded);
    assert!(record.latest_trace_ref.is_some());
}

#[test]
fn workflow_run_blocks_invalid_definitions_without_creating_a_session() {
    let workspace = temp_invalid_workflow_layer_workspace("workflow-layer-run-invalid");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "invalid-flow", "--goal", "Fix the failing add test"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(!workspace.join(".boundline").join("session.json").exists());
}
