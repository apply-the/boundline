use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn pr_ready_triggers_voting_but_low_risk_refactor_with_evidence_skips_it() {
    let workspace = temp_fixture_workspace("boundline-voting-pr-ready");
    assert_eq!(run_boundline_in(&workspace, &["start"]).status.code(), Some(0));

    let pr_ready = run_boundline_in(
        &workspace,
        &[
            "govern",
            "--mode",
            "pr-review",
            "--goal",
            "Review the merge-ready onboarding diff",
            "--pr-ready",
        ],
    );
    let pr_text = terminal_text(&pr_ready);
    assert_eq!(pr_ready.status.code(), Some(0), "{pr_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert!(status_text.contains("latest_voting_trigger: pr_ready"), "{status_text}");
    assert!(status_text.contains("latest_voting_blocking: true"), "{status_text}");

    let refactor = run_boundline_in(
        &workspace,
        &[
            "govern",
            "--mode",
            "refactor",
            "--goal",
            "Refactor the local helper without behavior changes",
            "--risk",
            "low",
            "--preserved-behavior-evidence",
        ],
    );
    let refactor_text = terminal_text(&refactor);
    assert_eq!(refactor.status.code(), Some(0), "{refactor_text}");

    let skipped = run_boundline_in(&workspace, &["status"]);
    let skipped_text = terminal_text(&skipped);
    assert!(
        skipped_text.contains("latest_voting_trigger: low_risk_preserved_behavior"),
        "{skipped_text}"
    );
    assert!(skipped_text.contains("latest_voting_result: skipped"), "{skipped_text}");
    assert!(skipped_text.contains("latest_voting_blocking: false"), "{skipped_text}");
}
