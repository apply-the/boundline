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
fn governance_trace_contract_native_inspect_omits_fixture_governance_wait_events() {
    let workspace = temp_canon_approval_workspace("synod-governance-trace-approval");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(!inspect_text.contains("governance_decision:"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_started:"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_awaiting_approval:"), "{inspect_text}");
}

#[test]
fn governance_trace_contract_native_inspect_omits_packet_rejection_and_block_events() {
    let workspace = temp_canon_packet_rejection_workspace("synod-governance-trace-rejected");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(!inspect_text.contains("governance_packet_rejected:"), "{inspect_text}");
    assert!(!inspect_text.contains("governance_blocked:"), "{inspect_text}");
}
