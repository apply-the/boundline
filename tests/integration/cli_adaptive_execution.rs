use crate::workspace_fixture::{
    extract_trace_path, run_boundline, temp_adaptive_fixture_workspace,
    temp_adaptive_guided_replanning_workspace, temp_adaptive_ordering_boundary_workspace,
    temp_adaptive_replanning_workspace, terminal_text,
};

#[test]
fn custom_run_executes_an_adaptive_profile_without_authored_attempts() {
    let workspace = temp_adaptive_fixture_workspace("boundline-cli-adaptive-run");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("code-adaptive-attempt-1"), "{text}");
    assert!(text.contains("verify-adaptive-attempt-1"), "{text}");
    assert!(
        text.contains(
            "execution_condition: terminal - goal satisfied after step verify-adaptive-attempt-1"
        ),
        "{text}"
    );
    assert!(text.contains("adaptive slice selected src/lib.rs via arithmetic_swap"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");
}

#[test]
fn custom_run_replans_an_adaptive_candidate_after_failed_validation() {
    let workspace = temp_adaptive_replanning_workspace("boundline-cli-adaptive-replan");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Recover after the first adaptive validation fails",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("verify-adaptive-attempt-1 (tool) failed"), "{text}");
    assert!(text.contains("latest_step=verify-adaptive-attempt-2 (succeeded)"), "{text}");
    assert!(text.contains("steps_remaining: 2"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}

#[test]
fn custom_run_uses_validation_guidance_to_shift_the_adaptive_target() {
    let workspace = temp_adaptive_guided_replanning_workspace("boundline-cli-adaptive-guided");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Recover after validation points to helper.rs",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("latest_step=verify-adaptive-attempt-2 (succeeded)"), "{text}");
    assert!(text.contains("adaptive slice selected src/helper.rs via arithmetic_swap"), "{text}");
    assert!(text.contains("steps_remaining: 2"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}

#[test]
fn custom_run_can_repair_an_ordering_boundary_with_a_broader_family() {
    let workspace =
        temp_adaptive_ordering_boundary_workspace("boundline-cli-adaptive-ordering-boundary");
    let output = run_boundline(&[
        "run",
        "--goal",
        "Fix the inclusive threshold boundary",
        "--compatibility",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("ordering_boundary_flip"), "{text}");
    assert!(text.contains("verify-adaptive-attempt-1"), "{text}");
    assert!(
        text.contains("adaptive slice selected src/lib.rs via ordering_boundary_flip"),
        "{text}"
    );
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}
