use synod::adapters::session_store::{FileSessionStore, SessionStore};
use synod::cli::run::execute_custom_run;
use synod::cli::session::{
    execute_capture, execute_next, execute_plan, execute_run, execute_start,
};

use crate::runtime_refoundation::{
    temp_runtime_refoundation_compat_workspace, temp_runtime_refoundation_governed_workspace,
};

#[test]
fn confirmed_goal_plan_takes_precedence_over_execution_profile_for_session_run() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
    assert!(run.terminal_output.contains("routing: native (goal_plan)"), "{}", run.terminal_output);
    assert!(!run.terminal_output.contains("routing: compatibility"), "{}", run.terminal_output);
}

#[test]
fn explicit_compatibility_run_is_visible_and_preserves_native_session_state() {
    let workspace = temp_runtime_refoundation_compat_workspace("runtime-routing-contract-compat");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let before = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let report = execute_custom_run(
        &workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(
        report.terminal_output.contains("routing: compatibility"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("execution_path: fixture_compatibility"),
        "{}",
        report.terminal_output
    );

    let after = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    assert_eq!(after.goal_plan, before.goal_plan);
    assert_eq!(after.decisions, before.decisions);
}

#[test]
fn native_and_compatibility_follow_up_keep_shared_routing_and_execution_condition_labels() {
    let compatibility_workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-shared-summary");

    let compatibility_next = execute_next(Some(&compatibility_workspace)).unwrap_err();
    assert!(compatibility_next.to_string().contains("no active session found"));

    let compatibility_run = execute_custom_run(
        &compatibility_workspace,
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(
        compatibility_run.terminal_output.contains("routing: compatibility"),
        "{}",
        compatibility_run.terminal_output
    );
    assert!(
        compatibility_run.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        compatibility_run.terminal_output
    );

    let compatibility_follow_up = execute_next(Some(&compatibility_workspace)).unwrap();
    assert!(
        compatibility_follow_up
            .terminal_output
            .contains("routing: compatibility (execution_profile)"),
        "{}",
        compatibility_follow_up.terminal_output
    );
    assert!(
        compatibility_follow_up.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        compatibility_follow_up.terminal_output
    );

    let native_workspace =
        temp_runtime_refoundation_compat_workspace("runtime-routing-contract-native-summary");
    execute_start(Some(&native_workspace)).unwrap();
    execute_capture(
        Some(&native_workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&native_workspace), Some("bug-fix"), false).unwrap();

    let native_run = execute_run(Some(&native_workspace)).unwrap();
    assert!(
        native_run.terminal_output.contains("routing: native (goal_plan)"),
        "{}",
        native_run.terminal_output
    );
    assert!(
        native_run.terminal_output.contains("execution_condition: terminal -"),
        "{}",
        native_run.terminal_output
    );
}

#[test]
fn canon_artifacts_remain_bounded_evidence_for_native_runs() {
    let workspace =
        temp_runtime_refoundation_governed_workspace("runtime-routing-contract-governed");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), None, true).unwrap();

    let run = execute_run(Some(&workspace)).unwrap();
    assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
    assert!(!run.terminal_output.contains("governance_selected:"), "{}", run.terminal_output);
}
