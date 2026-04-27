use std::fs;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_optional_governance_workspace,
    temp_required_governance_workspace, terminal_text,
};

#[test]
fn run_surfaces_local_governance_fallback_and_packet_state() {
    let workspace = temp_optional_governance_workspace("synod-session-governance-local");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> local"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_completed: local governance packet ready for bug-fix:investigate"
        ),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: local"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: governed_ready"), "{status_text}");
    assert!(
        status_text
            .contains("latest_governance_packet_ref: .synod/governance/bug-fix-investigate/"),
        "{status_text}"
    );

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_selected: bug-fix:investigate -> local"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(
            "governance_completed: local governance packet ready for bug-fix:investigate"
        ),
        "{inspect_text}"
    );
}

#[test]
fn step_blocks_when_required_canon_governance_is_unavailable() {
    let workspace = temp_required_governance_workspace("synod-session-governance-required");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(1), "{step_text}");
    assert!(step_text.contains("latest_governance_stage: bug-fix:investigate"), "{step_text}");
    assert!(step_text.contains("latest_governance_runtime: canon"), "{step_text}");
    assert!(step_text.contains("latest_governance_state: blocked"), "{step_text}");
    assert!(
        step_text.contains(
            "latest_governance_blocked_reason: governance required Canon for bug-fix:investigate"
        ),
        "{step_text}"
    );

    let inspect =
        run_synod_in(&workspace, &["inspect", "--workspace", workspace.to_string_lossy().as_ref()]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(
        inspect_text
            .contains("governance_blocked: governance required Canon for bug-fix:investigate"),
        "{inspect_text}"
    );
}

#[test]
fn step_does_not_bypass_when_canon_approval_is_still_pending() {
    let workspace = temp_canon_approval_workspace("synod-session-governance-approval-pending");

    assert_eq!(run_synod_in(&workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(&workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(&workspace, &["plan"]).status.code(), Some(0));

    let first_step = run_synod_in(&workspace, &["step"]);
    let first_step_text = terminal_text(&first_step);
    assert_eq!(first_step.status.code(), Some(0), "{first_step_text}");
    assert!(
        first_step_text.contains("latest_governance_state: awaiting_approval"),
        "{first_step_text}"
    );

    let second_step = run_synod_in(&workspace, &["step"]);
    let second_step_text = terminal_text(&second_step);
    assert_eq!(second_step.status.code(), Some(0), "{second_step_text}");
    assert!(
        second_step_text.contains("latest_governance_state: awaiting_approval"),
        "{second_step_text}"
    );
    assert!(
        second_step_text.contains("latest_governance_decision: autopilot is waiting for approval"),
        "{second_step_text}"
    );
    assert_eq!(
        fs::read_to_string(workspace.join("src/lib.rs")).unwrap(),
        "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n"
    );
}
