use std::fs;

use crate::workspace_fixture::{
    temp_workflow_governed_stage_approval_workspace, temp_workflow_governed_stage_workspace,
    terminal_text,
};

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

#[test]
fn workflow_status_projects_governed_stage_lineage_after_native_run() {
    let workspace = temp_workflow_governed_stage_workspace("workflow-governed-stage-lineage");

    let run = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    assert_eq!(run.status.code(), Some(0), "{}", terminal_text(&run));

    let status = run_boundline_in(&workspace, &["workflow", "status", "--workspace", "."]);
    let status_text = terminal_text(&status);

    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("workflow: default"), "{status_text}");
    assert!(status_text.contains("workflow_phase: inspect"), "{status_text}");
    assert!(status_text.contains("completion_verification_state: ready"), "{status_text}");
    assert!(
        status_text.contains("governance_lifecycle_mode_selection: auto-confirm"),
        "{status_text}"
    );

    let inspect = run_boundline_in(&workspace, &["workflow", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:investigate (discovery)"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains("governance_started: bug-fix:implement (implementation)"),
        "{inspect_text}"
    );
}

#[test]
fn workflow_status_refreshes_governed_investigate_approval_before_resume() {
    let workspace =
        temp_workflow_governed_stage_approval_workspace("workflow-governed-stage-approval-refresh");

    let run = run_boundline_in(
        &workspace,
        &["workflow", "run", "default", "--goal", "Fix the failing add test"],
    );
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("workflow: default"), "{run_text}");
    assert!(run_text.contains("latest_governance_stage: bug-fix:investigate"), "{run_text}");
    assert!(run_text.contains("latest_governance_state: awaiting_approval"), "{run_text}");

    let status = run_boundline_in(&workspace, &["workflow", "status", "--workspace", "."]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("workflow_phase: run"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: awaiting_approval"), "{status_text}");
    assert!(
        status_text.contains("next_command: boundline workflow resume --workspace "),
        "{status_text}"
    );

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let refreshed = run_boundline_in(&workspace, &["workflow", "status", "--workspace", "."]);
    let refreshed_text = terminal_text(&refreshed);
    assert_eq!(refreshed.status.code(), Some(0), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_state: governed_ready"), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_approval: granted"), "{refreshed_text}");

    let resume = run_boundline_in(&workspace, &["workflow", "resume", "--workspace", "."]);
    let resume_text = terminal_text(&resume);
    assert_eq!(resume.status.code(), Some(0), "{resume_text}");
    assert!(resume_text.contains("workflow_phase: inspect"), "{resume_text}");
    assert!(resume_text.contains("latest_governance_state: governed_ready"), "{resume_text}");
    assert!(
        resume_text.contains("next_command: boundline workflow inspect --workspace "),
        "{resume_text}"
    );

    let inspect = run_boundline_in(&workspace, &["workflow", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("workflow: default"), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:investigate (discovery)"),
        "{inspect_text}"
    );
}
