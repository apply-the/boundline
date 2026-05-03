use crate::workspace_fixture::{
    run_boundline, run_boundline_in, temp_adaptive_guided_replanning_workspace,
    temp_adaptive_replanning_workspace, terminal_text,
};

#[test]
fn status_next_and_inspect_surface_adaptive_terminal_failure_cues() {
    let workspace = temp_adaptive_replanning_workspace("boundline-session-adaptive");

    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(
            &workspace,
            &["capture", "--goal", "Recover after the first adaptive validation fails"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan", "--no-flow"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");
    assert!(run_text.contains("next_command: boundline inspect"), "{run_text}");

    let next = run_boundline_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: boundline inspect"), "{next_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref:"), "{status_text}");
    assert!(status_text.contains("next_command: boundline inspect"), "{status_text}");

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
}

#[test]
fn inspect_surfaces_validation_guided_adaptive_recovery_on_compatibility_route() {
    let workspace = temp_adaptive_guided_replanning_workspace("boundline-session-adaptive-guided");

    let run = run_boundline(&[
        "run",
        "--goal",
        "Recover after validation points to helper.rs",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("workspace_slice: src/helper.rs"), "{run_text}");

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: latest-workspace-trace"), "{inspect_text}");
    assert!(
        inspect_text.contains(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
        ),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(
            "adaptive slice selected src/helper.rs via arithmetic_swap for adaptive delivery after validation guidance"
        ),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");

    let next = run_boundline_in(
        &workspace,
        &["next", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("continuity_authority: compatibility_trace"), "{next_text}");
    assert!(next_text.contains("routing: compatibility (execution_profile)"), "{next_text}");
    assert!(next_text.contains("execution_condition: terminal -"), "{next_text}");
}
