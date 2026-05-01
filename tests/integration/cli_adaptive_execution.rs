use crate::workspace_fixture::{
    extract_trace_path, run_synod, temp_adaptive_fixture_workspace,
    temp_adaptive_guided_replanning_workspace, temp_adaptive_ordering_boundary_workspace,
    temp_adaptive_replanning_workspace, terminal_text,
};

#[test]
fn custom_run_executes_an_adaptive_profile_without_authored_attempts() {
    let workspace = temp_adaptive_fixture_workspace("synod-cli-adaptive-run");
    let output = run_synod(&[
        "run",
        "--goal",
        "Fix the failing add test",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("code-adaptive-attempt-1"), "{text}");
    assert!(text.contains("verify-adaptive-attempt-1"), "{text}");
    assert!(text.contains("changed_files: src/lib.rs"), "{text}");
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");
}

#[test]
fn custom_run_replans_an_adaptive_candidate_after_failed_validation() {
    let workspace = temp_adaptive_replanning_workspace("synod-cli-adaptive-replan");
    let output = run_synod(&[
        "run",
        "--goal",
        "Recover after the first adaptive validation fails",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("replan after verify-adaptive-attempt-1"), "{text}");
    assert!(text.contains("code-adaptive-attempt-2"), "{text}");
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}

#[test]
fn custom_run_uses_validation_guidance_to_shift_the_adaptive_target() {
    let workspace = temp_adaptive_guided_replanning_workspace("synod-cli-adaptive-guided");
    let output = run_synod(&[
        "run",
        "--goal",
        "Recover after validation points to helper.rs",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("replan after verify-adaptive-attempt-1"), "{text}");
    assert!(text.contains("changed_files: src/helper.rs"), "{text}");
    assert!(text.contains("workspace_slice: src/helper.rs"), "{text}");
    assert!(
        text.contains("attempt_lineage: adaptive-attempt-2 replaced adaptive-attempt-1"),
        "{text}"
    );
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}

#[test]
fn custom_run_can_repair_an_ordering_boundary_with_a_broader_family() {
    let workspace =
        temp_adaptive_ordering_boundary_workspace("synod-cli-adaptive-ordering-boundary");
    let output = run_synod(&[
        "run",
        "--goal",
        "Fix the inclusive threshold boundary",
        "--workspace",
        workspace.to_string_lossy().as_ref(),
    ]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("candidate_family: ordering_boundary_flip"), "{text}");
    assert!(text.contains("changed_files: src/lib.rs"), "{text}");
    assert!(text.contains("validation: passed"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
}
