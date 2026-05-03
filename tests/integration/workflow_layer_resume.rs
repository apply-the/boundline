use std::fs;

use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::workflow::WorkflowPhase;

use crate::workspace_fixture::{temp_workflow_layer_workspace, terminal_text};

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
fn workflow_status_reports_paused_capture_state_and_resume_guidance() {
    let workspace = temp_workflow_layer_workspace("workflow-layer-resume-status");

    let start = run_boundline_in(&workspace, &["workflow", "run", "default"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let output = run_boundline_in(&workspace, &["workflow", "status", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: capture"), "{text}");
    assert!(
		text.contains(
			"execution_condition: waiting - workflow is waiting for a captured goal before it can continue"
		),
		"{text}"
	);
    assert!(
        text.contains("next_command: boundline capture --workspace ")
            && text.contains("--goal <goal>"),
        "{text}"
    );
}

#[test]
fn workflow_resume_continues_after_goal_capture_without_replaying_completed_phases() {
    let workspace = temp_workflow_layer_workspace("workflow-layer-resume-run");

    let start = run_boundline_in(&workspace, &["workflow", "run", "default"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_boundline_in(
        &workspace,
        &["capture", "--workspace", ".", "--goal", "Fix the failing add test"],
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let output = run_boundline_in(&workspace, &["workflow", "resume", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("execution_condition: terminal - work completed successfully"), "{text}");

    let record = load_session_record(&workspace);
    record.validate().unwrap();
    let workflow_progress = record.workflow_progress.expect("workflow progress should exist");
    assert_eq!(workflow_progress.current_phase, Some(WorkflowPhase::Inspect));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Capture));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Plan));
    assert!(workflow_progress.completed_phases.contains(&WorkflowPhase::Run));
    assert_eq!(record.latest_status, SessionStatus::Succeeded);
}

#[test]
fn workflow_inspect_includes_workflow_projection_and_trace_summary() {
    let workspace = temp_workflow_layer_workspace("workflow-layer-inspect");

    let run = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    assert_eq!(run.status.code(), Some(0), "{}", terminal_text(&run));

    let output = run_boundline_in(&workspace, &["workflow", "inspect", "--workspace", "."]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow: default"), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("inspection_target:"), "{text}");
}
