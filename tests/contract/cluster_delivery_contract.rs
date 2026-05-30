use boundline::cli::cluster::execute_init;
use boundline::cli::inspect::execute_inspect;
use boundline::cli::session::{
    execute_goal_with_target, execute_plan_with_target, execute_run_with_target,
    execute_status_with_target,
};

use crate::workspace_fixture::{temp_broken_fixture_workspace, temp_cluster_workspaces};

#[test]
fn clustered_success_surfaces_name_authority_and_participation() {
    let (primary, secondary) = temp_cluster_workspaces("cluster-delivery-contract-success");

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

    for output in [&run.terminal_output, &status.terminal_output, &inspect.terminal_output] {
        assert!(output.contains("cluster_id: cluster-1"), "{output}");
        assert!(output.contains("cluster_route_owner: native"), "{output}");
        assert!(output.contains("cluster_authoritative_workspace:"), "{output}");
        assert!(output.contains("cluster_participating_workspaces:"), "{output}");
    }
}

#[test]
fn clustered_failure_surfaces_name_the_blocking_workspace() {
    let (primary, _) = temp_cluster_workspaces("cluster-delivery-contract-primary");
    let blocked = temp_broken_fixture_workspace("cluster-delivery-contract-blocked");

    execute_init(&primary, "cluster-1", &[primary.clone(), blocked.clone()]).unwrap();
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

    for output in [&run.terminal_output, &status.terminal_output, &inspect.terminal_output] {
        assert!(output.contains("cluster_blocking_workspace:"), "{output}");
        assert!(output.contains(blocked.to_string_lossy().as_ref()), "{output}");
    }
}
