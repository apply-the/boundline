use std::path::Path;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_canon_packet_rejection_workspace,
    terminal_text,
};

fn bootstrap_bug_fix(workspace: &Path) {
    assert_eq!(run_synod_in(workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_synod_in(workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_synod_in(workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_synod_in(workspace, &["plan"]).status.code(), Some(0));
}

#[test]
fn governance_trace_contract_native_inspect_surfaces_fixture_governance_wait_events() {
    let workspace = temp_canon_approval_workspace("synod-governance-trace-approval");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:investigate (discovery)"), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
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
    assert!(inspect_text.contains("terminal_status: running"), "{inspect_text}");
}

#[test]
fn governance_trace_contract_native_inspect_surfaces_packet_rejection_and_block_events() {
    let workspace = temp_canon_packet_rejection_workspace("synod-governance-trace-rejected");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(
        run_text.contains(
            "governance_blocked: governance packet was Rejected for stage bug-fix:investigate"
        ),
        "{run_text}"
    );

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
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
