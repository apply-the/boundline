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
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> local"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_completed: local governance packet ready for bug-fix:investigate"
        ),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: local"), "{status_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_selected: bug-fix:investigate -> local"),
        "{inspect_text}"
    );
}

#[test]
fn required_governance_workspace_blocks_on_native_goal_plan_path() {
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
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(
        run_text.contains("governance_blocked: governance required Canon for bug-fix:investigate"),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: blocked"), "{status_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
    assert!(
        inspect_text
            .contains("governance_blocked: governance required Canon for bug-fix:investigate"),
        "{inspect_text}"
    );
}

#[test]
fn approval_workspace_waits_on_investigate_governance_before_execution() {
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
    assert!(run_text.contains("terminal_status: running"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_awaiting_approval: bug-fix:investigate (requested) [canon-run-approval]"
        ),
        "{run_text}"
    );
    assert!(!run_text.contains("step investigate succeeded"), "{run_text}");
}
