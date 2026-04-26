use crate::workspace_fixture::{run_synod_in, temp_adaptive_replanning_workspace, terminal_text};

#[test]
fn status_next_and_inspect_surface_adaptive_slice_and_lineage() {
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
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let first_step = run_synod_in(&workspace, &["step"]);
    let first_step_text = terminal_text(&first_step);
    assert_eq!(first_step.status.code(), Some(0), "{first_step_text}");
    assert!(first_step_text.contains("latest_workspace_slice: src/lib.rs"), "{first_step_text}");
    assert!(
        first_step_text
            .contains("latest_selection_headline: selected src/lib.rs for adaptive delivery"),
        "{first_step_text}"
    );

    let next = run_synod_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("latest_workspace_slice: src/lib.rs"), "{next_text}");
    assert!(next_text.contains("next_command: synod step"), "{next_text}");

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("workspace_slice: src/lib.rs"), "{run_text}");
    assert!(
        run_text.contains("attempt_lineage: adaptive-attempt-2 replaced adaptive-attempt-1"),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_workspace_slice: src/lib.rs"), "{status_text}");
    assert!(
        status_text
            .contains("latest_attempt_lineage: adaptive-attempt-2 replaced adaptive-attempt-1"),
        "{status_text}"
    );
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("adaptive slice selected src/lib.rs for adaptive delivery"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("code-adaptive-attempt-2"), "{inspect_text}");
    assert!(inspect_text.contains("validation passed"), "{inspect_text}");
}
