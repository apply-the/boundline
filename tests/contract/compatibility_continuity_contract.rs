use boundline::cli::run::{RunCommandError, execute_custom_run, execute_native_direct_run};
use boundline::cli::session::{execute_goal, execute_next, execute_plan, execute_status};

use crate::runtime_refoundation::temp_runtime_refoundation_compat_workspace;

#[test]
fn status_contract_preserves_native_snapshot_and_surfaces_compatibility_follow_up() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("compatibility-continuity-contract-mixed-route");

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    execute_custom_run(&workspace, Some("Fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("routing: native (goal_plan)"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("continuity_authority: compatibility_trace"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("compatibility_follow_up: inspect_only"),
        "{}",
        status.terminal_output
    );
    assert!(
        status
            .terminal_output
            .contains("compatibility_follow_up_command: boundline inspect --workspace "),
        "{}",
        status.terminal_output
    );
}

#[test]
fn next_contract_without_active_session_uses_latest_compatibility_trace_as_authority() {
    let workspace =
        temp_runtime_refoundation_compat_workspace("compatibility-continuity-contract-no-session");

    execute_custom_run(&workspace, Some("Fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let next = execute_next(Some(&workspace)).unwrap();
    assert!(
        next.terminal_output.contains("continuity_authority: compatibility_trace"),
        "{}",
        next.terminal_output
    );
    assert!(
        next.terminal_output.contains("compatibility_follow_up: inspect_only"),
        "{}",
        next.terminal_output
    );
    assert!(
        next.terminal_output.contains("next_command: boundline inspect --workspace "),
        "{}",
        next.terminal_output
    );
}

#[test]
fn native_direct_run_contract_refuses_to_replace_meaningful_active_session_state() {
    let workspace = temp_runtime_refoundation_compat_workspace(
        "compatibility-continuity-contract-active-session",
    );

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();

    let error = execute_native_direct_run(
        &workspace,
        Some("Ship the checkout change"),
        &[],
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .unwrap_err();

    assert!(matches!(error, RunCommandError::ActiveSessionConflict));
}
