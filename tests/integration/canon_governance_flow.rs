use std::path::Path;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_governance_workspace, temp_canon_packet_rejection_workspace,
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
fn canon_governance_flow_reuses_upstream_packet_for_implement_stage() {
    let workspace = temp_canon_governance_workspace("synod-canon-governance-flow");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> canon"), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:implement -> canon"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_completed: implementation packet ready [.canon/runs/canon-run-implement]"
        ),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(
        status_text.contains("latest_governance_packet_source_stage: bug-fix:investigate"),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_packet_binding_reason: upstream_stage_context"),
        "{status_text}"
    );
}

#[test]
fn canon_governance_flow_halts_when_packet_is_not_ready() {
    let workspace = temp_canon_packet_rejection_workspace("synod-canon-governance-rejected");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("latest_governance_runtime: canon"), "{step_text}");
    assert!(step_text.contains("latest_governance_state: blocked"), "{step_text}");
    assert!(step_text.contains("governance packet was Rejected"), "{step_text}");

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("governance_packet_rejected:"), "{inspect_text}");
}
