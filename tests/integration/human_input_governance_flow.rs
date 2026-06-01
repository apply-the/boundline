use crate::workspace_fixture::{
    run_boundline_in, temp_canon_governance_workspace, temp_optional_governance_workspace,
    terminal_text,
};

#[test]
fn capture_and_status_project_requested_governance_intent() {
    let workspace = temp_optional_governance_workspace("boundline-human-governance-status");

    let capture = run_boundline_in(
        &workspace,
        &[
            "goal",
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

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("requested_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("requested_governance_risk: high"), "{status_text}");
    assert!(status_text.contains("requested_governance_zone: payments"), "{status_text}");
    assert!(status_text.contains("requested_governance_owner: platform"), "{status_text}");
}

#[test]
fn explicit_canon_request_blocks_without_local_fallback() {
    let workspace = temp_optional_governance_workspace("boundline-human-governance-canon-block");

    assert_eq!(
        run_boundline_in(
            &workspace,
            &[
                "goal",
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
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    let plan = run_boundline_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(1), "{plan_text}");
    assert!(plan_text.contains("requested_governance_runtime: canon"), "{plan_text}");
    assert!(plan_text.contains("latest_governance_stage: plan:discovery"), "{plan_text}");
    assert!(plan_text.contains("latest_governance_state: blocked"), "{plan_text}");

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(
        run_text.contains("active session planning governance for `plan:discovery` is `blocked`"),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("requested_governance_runtime: canon"), "{text}");
    assert!(text.contains("latest_governance_stage: plan:discovery"), "{text}");
    assert!(text.contains("latest_governance_state: blocked"), "{text}");

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(3), "{inspect_text}");
    assert!(inspect_text.contains("inspect: trace read failure"), "{inspect_text}");
}

#[test]
fn goal_rejects_explicit_canon_request_missing_owner() {
    let workspace = temp_optional_governance_workspace("boundline-human-governance-missing-owner");

    let goal = run_boundline_in(
        &workspace,
        &[
            "goal",
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
    let text = terminal_text(&goal);
    assert_eq!(goal.status.code(), Some(1), "{text}");
    assert!(text.contains("failed to ingest authored brief"), "{text}");
    assert!(text.contains("owner"), "{text}");
    assert!(text.contains("canon"), "{text}");
}

#[test]
fn explicit_local_request_overrides_existing_canon_policy() {
    let workspace = temp_canon_governance_workspace("boundline-human-governance-local-override");

    assert_eq!(
        run_boundline_in(
            &workspace,
            &["goal", "--goal", "Fix the failing checkout flow", "--governance", "local",],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{text}");
    assert!(text.contains("requested_governance_runtime: local"), "{text}");
    assert!(!text.contains("latest_governance_runtime: canon"), "{text}");
}

#[test]
fn inspect_projects_requested_governance_intent_for_session_runs() {
    let workspace = temp_canon_governance_workspace("boundline-human-governance-inspect");

    assert_eq!(
        run_boundline_in(
            &workspace,
            &["goal", "--goal", "Fix the failing checkout flow", "--governance", "local",],
        )
        .status
        .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["run"]).status.code(), Some(0));

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
}
