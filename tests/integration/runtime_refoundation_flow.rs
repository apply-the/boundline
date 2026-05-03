use crate::runtime_refoundation::temp_runtime_refoundation_workspace;
use crate::workspace_fixture::run_synod_in;
use crate::workspace_fixture::terminal_text;

#[test]
fn direct_goal_run_bootstraps_native_session_without_a_declarative_profile() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-direct-run");

    let run = run_synod_in(
        &workspace,
        &["run", "--workspace", ".", "--goal", "fix the failing add test"],
    );
    let run_text = terminal_text(&run);

    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(!run_text.contains("routing: compatibility"), "{run_text}");

    let status = run_synod_in(&workspace, &["status", "--workspace", "."]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
}

#[test]
fn session_native_runtime_path_runs_without_a_declarative_profile() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-flow");

    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]);
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let plan = run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains("routing: native (goal_plan)"), "{plan_text}");
    assert!(
        plan_text.contains(
            "execution_condition: waiting - planning is complete and execution can begin"
        ),
        "{plan_text}"
    );
    assert!(plan_text.contains("execution_path: native_goal_plan"), "{plan_text}");
    assert!(plan_text.contains("next_command: synod run"), "{plan_text}");

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(run_text.contains("execution_condition: terminal -"), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(run_text.contains("terminal_status: succeeded"), "{run_text}");
    assert!(run_text.contains("trace:"), "{run_text}");
}

#[test]
fn inspect_surfaces_route_goal_plan_and_decision_timeline_for_native_run() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-inspect");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);

    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("routing: native (goal_plan)"), "{inspect_text}");
    assert!(inspect_text.contains("execution_condition: terminal -"), "{inspect_text}");
    assert!(inspect_text.contains("goal_plan_summary:"), "{inspect_text}");
    assert!(inspect_text.contains("decision_timeline:"), "{inspect_text}");
    assert!(inspect_text.contains("selector:"), "{inspect_text}");
    assert!(
        inspect_text
            .contains("rationale: Repair bounded implementation for fix the failing add test")
            || inspect_text.contains(
                "rationale: Investigate bounded failure evidence for fix the failing add test"
            ),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("expected_outcome:"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_reason:"), "{inspect_text}");
}

#[test]
fn status_after_native_run_surfaces_latest_persisted_decision_state() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-status");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["run"]).status.code(), Some(0));

    let status = run_synod_in(&workspace, &["status", "--workspace", "."]);
    let status_text = terminal_text(&status);

    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("routing: native (goal_plan)"), "{status_text}");
    assert!(
        status_text.contains("execution_condition: terminal - work completed successfully"),
        "{status_text}"
    );
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_decision_status: verified"), "{status_text}");
    assert!(status_text.contains("latest_decision_target:"), "{status_text}");
    assert!(status_text.contains("latest_selection_headline: selector "), "{status_text}");
    assert!(status_text.contains("latest_selection_reason:"), "{status_text}");
    assert!(status_text.contains("next_command: synod inspect"), "{status_text}");
}

#[test]
fn run_guidance_for_proposed_plan_includes_confirm_action() {
    let workspace = temp_runtime_refoundation_workspace("runtime-refoundation-proposed-guidance");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);

    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("must be confirmed before execution"), "{run_text}");
    assert!(run_text.contains("synod plan --confirm"), "{run_text}");
}

#[test]
fn status_surfaces_confirmed_and_skipped_flow_states() {
    let confirmed_workspace =
        temp_runtime_refoundation_workspace("runtime-refoundation-confirmed-flow-state");

    assert_eq!(run_synod_in(&confirmed_workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&confirmed_workspace, &["capture", "--goal", "fix the failing add test"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        run_synod_in(&confirmed_workspace, &["plan", "--flow", "change"]).status.code(),
        Some(0)
    );

    let confirmed_status = run_synod_in(&confirmed_workspace, &["status"]);
    let confirmed_text = terminal_text(&confirmed_status);
    assert_eq!(confirmed_status.status.code(), Some(0), "{confirmed_text}");
    assert!(confirmed_text.contains("flow_state: confirmed (change)"), "{confirmed_text}");

    let skipped_workspace =
        temp_runtime_refoundation_workspace("runtime-refoundation-skipped-flow-state");

    assert_eq!(run_synod_in(&skipped_workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &skipped_workspace,
            &["capture", "--goal", "implement workspace summary output"]
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&skipped_workspace, &["plan", "--no-flow"]).status.code(), Some(0));

    let skipped_status = run_synod_in(&skipped_workspace, &["status"]);
    let skipped_text = terminal_text(&skipped_status);
    assert_eq!(skipped_status.status.code(), Some(0), "{skipped_text}");
    assert!(skipped_text.contains("flow_state: skipped"), "{skipped_text}");
}
