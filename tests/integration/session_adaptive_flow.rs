use crate::workspace_fixture::{run_synod_in, temp_adaptive_replanning_workspace, terminal_text};

#[test]
fn status_next_and_inspect_surface_adaptive_terminal_failure_cues() {
    let workspace = temp_adaptive_replanning_workspace("synod-session-adaptive");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Recover after the first adaptive validation fails"],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--no-flow"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");
    assert!(run_text.contains("next_command: synod inspect"), "{run_text}");

    let next = run_synod_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("next_command: synod inspect"), "{next_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_trace_ref:"), "{status_text}");
    assert!(status_text.contains("next_command: synod inspect"), "{status_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("inspection_target: session-trace-ref"), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
}
