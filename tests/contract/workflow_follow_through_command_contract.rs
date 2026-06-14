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

#[test]
fn workflow_follow_through_contract_surfaces_govern_pause_with_actionable_guidance() {
    let workspace =
        temp_workflow_follow_through_approval_workspace("workflow-follow-through-contract-pending");

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
    assert!(text.contains("execution_condition: waiting - governance approval is still pending before execution can continue"), "{text}");
    assert!(text.contains("next_command: boundline workflow resume --workspace "), "{text}");
    assert!(!text.contains("not yet executable from the workflow command surface"), "{text}");
}

#[test]
fn workflow_follow_through_contract_completes_review_and_govern_without_static_blockers() {
    let workspace =
        temp_workflow_follow_through_workspace("workflow-follow-through-contract-ready");

    let output = run_boundline_in(
        &workspace,
        &["workflow", "run", "governed-delivery", "--goal", "Fix the failing checkout flow"],
    );
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("workflow_phase: inspect"), "{text}");
    assert!(text.contains("governance_lifecycle_mode_selection: auto-confirm"), "{text}");
    assert!(text.contains("execution_condition: terminal - work completed successfully"), "{text}");
}
