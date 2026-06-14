use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::cluster::execute_init;
use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{
    execute_goal_with_target, execute_plan_with_target, execute_run_with_target,
    execute_status_with_target,
};
use boundline::domain::cluster::ClusteredExecutionKind;
use boundline::domain::session::SessionStatus;

use crate::workspace_fixture::temp_cluster_workspaces;

#[test]
fn clustered_delivery_run_mutates_both_member_workspaces_under_one_session_owner() {
    let (primary, secondary) = temp_cluster_workspaces("cluster-delivery-success");

    execute_init(&primary, "cluster-1", &[primary.clone(), secondary.clone()]).unwrap();
    execute_goal_with_target(
        None,
        Some(&primary),
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan_with_target(None, Some(&primary), Some("bug-fix"), false).unwrap();

    let run = execute_run_with_target(None, Some(&primary)).unwrap();
    let status = execute_status_with_target(None, Some(&primary), None).unwrap();
    let inspect = execute_inspect(None, Some(&primary), None, false).unwrap();

    assert!(run.terminal_output.contains("terminal_status: succeeded"), "{}", run.terminal_output);
    assert!(status.terminal_output.contains("cluster_id: cluster-1"), "{}", status.terminal_output);
    assert!(
        status.terminal_output.contains("cluster_participating_workspaces:"),
        "{}",
        status.terminal_output
    );
    assert!(
        inspect.terminal_output.contains("latest_checkpoint_scope: cluster"),
        "{}",
        inspect.terminal_output
    );
    assert!(
        inspect
            .terminal_output
            .contains("execution_condition: terminal - goal satisfied after fresh proof"),
        "{}",
        inspect.terminal_output
    );
    assert_eq!(
        fs::read_to_string(primary.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n"
    );
    assert_eq!(
        fs::read_to_string(secondary.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n"
    );

    let record = FileSessionStore::for_workspace(&primary).load().unwrap().unwrap();
    assert_eq!(record.latest_status, SessionStatus::Succeeded);
    let story =
        record.active_task.as_ref().unwrap().context.cluster_delivery_story().unwrap().unwrap();
    assert_eq!(story.execution_condition.kind, ClusteredExecutionKind::Success);
    assert_eq!(story.participating_workspaces.len(), 2);
}
