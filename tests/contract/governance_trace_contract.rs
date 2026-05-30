use std::path::Path;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_approval_workspace, temp_canon_packet_rejection_workspace,
    temp_canon_unsupported_companion_workspace, terminal_text,
};

fn bootstrap_bug_fix(workspace: &Path) {
    assert_eq!(
        run_boundline_in(workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(workspace, &["plan"]).status.code(), Some(0));
}

#[test]
fn governance_trace_contract_native_inspect_surfaces_fixture_governance_wait_events() {
    let workspace = temp_canon_approval_workspace("boundline-governance-trace-approval");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:investigate (discovery)"), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:investigate (discovery)"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(
            "governance_awaiting_approval: bug-fix:investigate (requested) [canon-run-approval]"
        ),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("governance_runtime_state: advisory"), "{inspect_text}");
    assert!(inspect_text.contains("governance_rollout_profile: minimal"), "{inspect_text}");
    assert!(
        inspect_text
            .contains("governance_reason: startup posture defaulted locally for low-trust surface"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains(
            "governance_approval_provenance: stronger posture remained inactive because operator approval is still requested"
        ),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("terminal_status: running"), "{inspect_text}");
}

#[test]
fn governance_trace_contract_native_inspect_surfaces_packet_rejection_and_block_events() {
    let workspace = temp_canon_packet_rejection_workspace("boundline-governance-trace-rejected");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(
        run_text.contains(
            "governance_blocked: governance packet was Rejected for stage bug-fix:investigate"
        ),
        "{run_text}"
    );

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("governance_packet_rejected: governance packet was Rejected for stage bug-fix:investigate"), "{inspect_text}");
    assert!(
        inspect_text.contains(
            "governance_blocked: governance packet was Rejected for stage bug-fix:investigate"
        ),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains("terminal_reason: governed work is blocked pending intervention"),
        "{inspect_text}"
    );
}

#[test]
fn governance_trace_contract_ignores_unsupported_companion_contract_lines() {
    let workspace =
        temp_canon_unsupported_companion_workspace("boundline-governance-trace-unsupported");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_completed: discovery packet ready"), "{run_text}");

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:investigate (discovery)"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("governance_runtime_state: advisory"), "{inspect_text}");
    assert!(inspect_text.contains("governance_rollout_profile: minimal"), "{inspect_text}");
    assert!(
        inspect_text
            .contains("governance_reason: startup posture defaulted locally for low-trust surface"),
        "{inspect_text}"
    );
    assert!(
        !inspect_text.contains("adaptive-governance-v2"),
        "unsupported companion contract should not surface in inspect output: {inspect_text}"
    );
}
