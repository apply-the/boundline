use std::fs;

use synod::adapters::session_store::{FileSessionStore, SessionStore};
use synod::cli::cluster::execute_init;
use synod::cli::inspect::execute_inspect;
use synod::cli::session::{
    execute_capture_with_target, execute_plan_with_target, execute_run_with_target,
    execute_start_with_target, execute_status_with_target,
};
use synod::domain::cluster::ClusteredExecutionKind;
use synod::domain::session::SessionStatus;

use crate::workspace_fixture::{temp_broken_fixture_workspace, temp_fixture_workspace};

#[test]
fn clustered_delivery_run_names_the_blocking_workspace_when_a_member_cannot_continue() {
    let primary = temp_fixture_workspace("cluster-delivery-blocked-primary");
    let secondary = temp_broken_fixture_workspace("cluster-delivery-blocked-secondary");

    execute_init(&primary, "cluster-1", &[primary.clone(), secondary.clone()]).unwrap();
    execute_start_with_target(None, Some(&primary)).unwrap();
    execute_capture_with_target(
        None,
        Some(&primary),
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan_with_target(None, Some(&primary), Some("bug-fix"), false).unwrap();

    let run = execute_run_with_target(None, Some(&primary)).unwrap();
    let status = execute_status_with_target(None, Some(&primary)).unwrap();
    let inspect = execute_inspect(None, Some(&primary)).unwrap();

    assert!(run.terminal_output.contains("cluster_blocking_workspace:"), "{}", run.terminal_output);
    assert!(
        run.terminal_output.contains(secondary.to_string_lossy().as_ref()),
        "{}",
        run.terminal_output
    );
    assert!(
        status.terminal_output.contains("cluster_execution_condition: failed"),
        "{}",
        status.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("cluster_blocking_workspace:"),
        "{}",
        inspect.terminal_output
    );

    assert_eq!(
        fs::read_to_string(primary.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n"
    );
    assert_eq!(
        fs::read_to_string(secondary.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n"
    );

    let record = FileSessionStore::for_workspace(&primary).load().unwrap().unwrap();
    assert_eq!(record.latest_status, SessionStatus::Failed);
    let expected_secondary = fs::canonicalize(&secondary).unwrap();
    let story =
        record.active_task.as_ref().unwrap().context.cluster_delivery_story().unwrap().unwrap();
    assert_eq!(story.execution_condition.kind, ClusteredExecutionKind::Failed);
    assert_eq!(
        story.execution_condition.blocking_workspace_ref.as_deref(),
        Some(expected_secondary.to_string_lossy().as_ref())
    );
}
