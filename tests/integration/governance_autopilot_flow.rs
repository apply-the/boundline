use std::fs;
use std::path::Path;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_canon_autopilot_blocked_workspace,
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
fn governance_autopilot_flow_selects_mode_and_refreshes_after_approval() {
    let workspace = temp_canon_approval_workspace("synod-governance-autopilot-approval");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("latest_governance_state: awaiting_approval"), "{step_text}");
    assert!(
        step_text.contains("latest_governance_decision: autopilot selected Canon mode Discovery"),
        "{step_text}"
    );
    assert!(step_text.contains("latest_governance_candidates: select_mode"), "{step_text}");
    assert!(step_text.contains("latest_governance_mode: discovery"), "{step_text}");

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_state: governed_ready"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: discovery"), "{status_text}");
    assert!(status_text.contains("latest_governance_run_ref: canon-run-approval"), "{status_text}");
}

#[test]
fn governance_autopilot_flow_blocks_required_stage_without_a_canon_runtime() {
    let workspace = temp_canon_autopilot_blocked_workspace("synod-governance-autopilot-blocked");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(1), "{step_text}");
    assert!(step_text.contains("latest_governance_runtime: canon"), "{step_text}");
    assert!(step_text.contains("latest_governance_state: blocked"), "{step_text}");
    assert!(
        step_text.contains("latest_governance_decision: autopilot selected Canon mode Discovery"),
        "{step_text}"
    );
    assert!(step_text.contains("latest_governance_candidates: select_mode"), "{step_text}");
    assert!(step_text.contains("command 'canon-missing' is unavailable"), "{step_text}");
}
