use boundline::domain::completion_verification::{
    capture_workspace_fingerprint, compare_workspace_fingerprints,
};

use crate::completion_verification::{completion_verification_workspace, write_workspace_file};

#[test]
fn source_changes_invalidate_a_passing_proof() {
    let workspace = completion_verification_workspace("completion-verification-source");
    let before = capture_workspace_fingerprint(workspace.path(), false);
    assert!(before.is_ok());
    let before = before.unwrap_or_else(|_| unreachable!());

    let write_result = write_workspace_file(
        workspace.path(),
        "src/lib.rs",
        "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
    );
    assert!(write_result.is_ok(), "{write_result:?}");

    let after = capture_workspace_fingerprint(workspace.path(), false);
    assert!(after.is_ok());
    let after = after.unwrap_or_else(|_| unreachable!());

    let diff = compare_workspace_fingerprints(&before, &after);
    assert_eq!(diff.changed_paths, vec!["src/lib.rs".to_string()]);
    assert!(!diff.truncated);
}

#[test]
fn boundline_runtime_artifacts_do_not_invalidate_the_proof_that_wrote_them() {
    let workspace = completion_verification_workspace("completion-verification-boundline");
    let before = capture_workspace_fingerprint(workspace.path(), false);
    assert!(before.is_ok());
    let before = before.unwrap_or_else(|_| unreachable!());

    let write_result = write_workspace_file(
        workspace.path(),
        ".boundline/traces/session-1/events.jsonl",
        "{\"event\":\"proof\"}\n",
    );
    assert!(write_result.is_ok(), "{write_result:?}");

    let after = capture_workspace_fingerprint(workspace.path(), false);
    assert!(after.is_ok());
    let after = after.unwrap_or_else(|_| unreachable!());

    let diff = compare_workspace_fingerprints(&before, &after);
    assert!(diff.changed_paths.is_empty(), "{diff:?}");
}

#[test]
fn documentation_only_changes_are_ignored_when_docs_are_not_claim_relevant() {
    let workspace = completion_verification_workspace("completion-verification-docs-off");
    let before = capture_workspace_fingerprint(workspace.path(), false);
    assert!(before.is_ok());
    let before = before.unwrap_or_else(|_| unreachable!());

    let write_result = write_workspace_file(
        workspace.path(),
        "docs/release-readiness.md",
        "# Release Readiness\n\nUpdated operator note.\n",
    );
    assert!(write_result.is_ok(), "{write_result:?}");

    let after = capture_workspace_fingerprint(workspace.path(), false);
    assert!(after.is_ok());
    let after = after.unwrap_or_else(|_| unreachable!());

    let diff = compare_workspace_fingerprints(&before, &after);
    assert!(diff.changed_paths.is_empty(), "{diff:?}");
}

#[test]
fn documentation_only_changes_invalidate_when_docs_are_claim_relevant() {
    let workspace = completion_verification_workspace("completion-verification-docs-on");
    let before = capture_workspace_fingerprint(workspace.path(), true);
    assert!(before.is_ok());
    let before = before.unwrap_or_else(|_| unreachable!());

    let write_result = write_workspace_file(
        workspace.path(),
        "docs/release-readiness.md",
        "# Release Readiness\n\nUpdated release note.\n",
    );
    assert!(write_result.is_ok(), "{write_result:?}");

    let after = capture_workspace_fingerprint(workspace.path(), true);
    assert!(after.is_ok());
    let after = after.unwrap_or_else(|_| unreachable!());

    let diff = compare_workspace_fingerprints(&before, &after);
    assert_eq!(diff.changed_paths, vec!["docs/release-readiness.md".to_string()]);
}

#[test]
fn large_change_sets_are_capped_and_marked_truncated() {
    let workspace = completion_verification_workspace("completion-verification-many");
    let before = capture_workspace_fingerprint(workspace.path(), false);
    assert!(before.is_ok());
    let before = before.unwrap_or_else(|_| unreachable!());

    for index in 0..12 {
        let relative_path = format!("src/generated_{index}.rs");
        let contents = format!("pub const VALUE_{index}: i32 = {index};\n");
        let write_result = write_workspace_file(workspace.path(), &relative_path, &contents);
        assert!(write_result.is_ok(), "{write_result:?}");
    }

    let after = capture_workspace_fingerprint(workspace.path(), false);
    assert!(after.is_ok());
    let after = after.unwrap_or_else(|_| unreachable!());

    let diff = compare_workspace_fingerprints(&before, &after);
    assert_eq!(diff.changed_paths.len(), 10);
    assert!(diff.truncated);
}
