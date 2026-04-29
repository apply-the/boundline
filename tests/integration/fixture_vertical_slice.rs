use std::process::Command;

use crate::workspace_fixture::{
    extract_trace_path, run_synod_in, temp_fixture_workspace, terminal_text,
};

#[test]
fn fixture_vertical_slice_drives_a_failing_test_to_green() {
    let workspace = temp_fixture_workspace("synod-cli-vertical-slice");
    let initial =
        Command::new("cargo").args(["test", "--quiet"]).current_dir(&workspace).output().unwrap();
    assert!(!initial.status.success(), "{}", terminal_text(&initial));

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing add test"]).status.code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["plan", "--flow", "bug-fix"]).status.code(), Some(0));

    let output = run_synod_in(&workspace, &["run"]);
    let text = terminal_text(&output);
    let trace_path = extract_trace_path(&text);

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("created: Analyze"), "{text}");
    assert!(text.contains("created: Fix"), "{text}");
    assert!(text.contains("created: Test"), "{text}");
    assert!(text.contains("terminal_status: succeeded"), "{text}");
    assert!(trace_path.as_ref().is_some_and(|path| path.exists()), "{text}");

    let final_run =
        Command::new("cargo").args(["test", "--quiet"]).current_dir(&workspace).output().unwrap();
    assert!(final_run.status.success(), "{}", terminal_text(&final_run));
}
