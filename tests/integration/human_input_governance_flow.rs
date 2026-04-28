use crate::workspace_fixture::{
    run_synod_in, temp_canon_governance_workspace, temp_optional_governance_workspace,
    terminal_text,
};

#[test]
fn capture_and_status_project_requested_governance_intent() {
    let workspace = temp_optional_governance_workspace("synod-human-governance-status");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    let capture = run_synod_in(
        &workspace,
        &[
            "capture",
            "--goal",
            "Fix the failing checkout flow",
            "--governance",
            "canon",
            "--risk",
            "high",
            "--zone",
            "payments",
            "--owner",
            "platform",
        ],
    );
    let capture_text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(0), "{capture_text}");
    assert!(capture_text.contains("requested_governance_runtime: canon"), "{capture_text}");
    assert!(capture_text.contains("requested_governance_risk: high"), "{capture_text}");
    assert!(capture_text.contains("requested_governance_zone: payments"), "{capture_text}");
    assert!(capture_text.contains("requested_governance_owner: platform"), "{capture_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("requested_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("requested_governance_risk: high"), "{status_text}");
    assert!(status_text.contains("requested_governance_zone: payments"), "{status_text}");
    assert!(status_text.contains("requested_governance_owner: platform"), "{status_text}");
}

#[test]
fn explicit_canon_request_blocks_without_local_fallback() {
    let workspace = temp_optional_governance_workspace("synod-human-governance-canon-block");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &[
                "capture",
                "--goal",
                "Fix the failing checkout flow",
                "--governance",
                "canon",
                "--risk",
                "high",
                "--zone",
                "payments",
                "--owner",
                "platform",
            ],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let step = run_synod_in(&workspace, &["step"]);
    let text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(1), "{text}");
    assert!(text.contains("requested_governance_runtime: canon"), "{text}");
    assert!(text.contains("latest_governance_runtime: canon"), "{text}");
    assert!(text.contains("latest_governance_state: blocked"), "{text}");
    assert!(text.contains("latest_governance_blocked_reason: governance required Canon"), "{text}");
    assert!(
        text.contains(
            "governance_next_action: resolve the governance blocker, then rerun synod step"
        ),
        "{text}"
    );

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(
        inspect_text.contains(
            "governance_next_action: resolve the governance blocker, then rerun synod step"
        ),
        "{inspect_text}"
    );
}

#[test]
fn capture_rejects_explicit_canon_request_missing_owner() {
    let workspace = temp_optional_governance_workspace("synod-human-governance-missing-owner");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));

    let capture = run_synod_in(
        &workspace,
        &[
            "capture",
            "--goal",
            "Fix the failing checkout flow",
            "--governance",
            "canon",
            "--risk",
            "high",
            "--zone",
            "payments",
        ],
    );
    let text = terminal_text(&capture);
    assert_eq!(capture.status.code(), Some(1), "{text}");
    assert!(text.contains("failed to ingest authored brief"), "{text}");
    assert!(text.contains("owner"), "{text}");
    assert!(text.contains("canon"), "{text}");
}

#[test]
fn explicit_local_request_overrides_existing_canon_policy() {
    let workspace = temp_canon_governance_workspace("synod-human-governance-local-override");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Fix the failing checkout flow", "--governance", "local",],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let step = run_synod_in(&workspace, &["step"]);
    let text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{text}");
    assert!(text.contains("requested_governance_runtime: local"), "{text}");
    assert!(text.contains("latest_governance_runtime: local"), "{text}");
    assert!(!text.contains("latest_governance_runtime: canon"), "{text}");
}

#[test]
fn inspect_projects_requested_governance_intent_for_session_runs() {
    let workspace = temp_canon_governance_workspace("synod-human-governance-inspect");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(
            &workspace,
            &["capture", "--goal", "Fix the failing checkout flow", "--governance", "local",],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("requested_governance_runtime: local"), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_selected: bug-fix:investigate -> local"),
        "{inspect_text}"
    );
}
