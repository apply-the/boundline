use std::fs;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_optional_governance_workspace,
    temp_required_governance_workspace, terminal_text,
};

#[test]
fn run_in_optional_governance_workspace_uses_native_goal_plan_path() {
    let workspace = temp_optional_governance_workspace("synod-session-governance-local");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(!run_text.contains("governance_selected:"), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(!status_text.contains("latest_governance_stage:"), "{status_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_selected:"), "{inspect_text}");
}

#[test]
fn required_governance_workspace_still_runs_on_native_goal_plan_path() {
    let workspace = temp_required_governance_workspace("synod-session-governance-required");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_blocked:"), "{inspect_text}");
}

#[test]
fn approval_workspace_runs_without_waiting_on_fixture_governance_step() {
    let workspace = temp_canon_approval_workspace("synod-session-governance-approval-pending");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert_eq!(
        fs::read_to_string(workspace.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n"
    );
}
