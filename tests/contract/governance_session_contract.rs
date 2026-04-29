use std::path::Path;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_canon_governance_workspace, terminal_text,
};

fn bootstrap_bug_fix(workspace: &Path) {
    assert_eq!(run_synod_in(workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(workspace, &["plan"]).status.code(), Some(0));
}

#[test]
fn governance_session_contract_native_run_does_not_project_fixture_governance_fields() {
    let workspace = temp_canon_governance_workspace("synod-governance-session-contract");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(!run_text.contains("governance_selected:"), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(!status_text.contains("latest_governance_runtime:"), "{status_text}");
    assert!(!status_text.contains("latest_governance_mode:"), "{status_text}");
    assert!(!status_text.contains("latest_governance_run_ref:"), "{status_text}");
}

#[test]
fn governance_session_contract_native_planned_sessions_require_run_instead_of_step() {
    let workspace = temp_canon_approval_workspace("synod-governance-approval-session");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_ne!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("active session has no planned task"), "{step_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("next_command: synod run"), "{status_text}");
    assert!(!status_text.contains("latest_governance_state:"), "{status_text}");
}
