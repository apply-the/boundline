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
fn governance_trace_contract_renders_approval_wait_with_mode_and_run_ref() {
    let workspace = temp_canon_approval_workspace("synod-governance-trace-approval");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");

    std::fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let refresh_run = run_synod_in(&workspace, &["run"]);
    let refresh_run_text = terminal_text(&refresh_run);
    assert_eq!(refresh_run.status.code(), Some(0), "{refresh_run_text}");

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("governance_decision: select_mode"), "{inspect_text}");
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
}

#[test]
fn governance_trace_contract_records_packet_rejection_and_block() {
    let workspace = temp_canon_packet_rejection_workspace("synod-governance-trace-rejected");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("latest_governance_state: blocked"), "{step_text}");

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("governance_packet_rejected:"), "{inspect_text}");
    assert!(inspect_text.contains("governance_blocked:"), "{inspect_text}");
    assert!(inspect_text.contains("governance packet was Rejected"), "{inspect_text}");
}
