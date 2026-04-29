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

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(!run_text.contains("latest_governance_state:"), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(!status_text.contains("latest_governance_state:"), "{status_text}");
}

#[test]
fn governance_autopilot_flow_blocks_required_stage_without_a_canon_runtime() {
    let workspace = temp_canon_autopilot_blocked_workspace("synod-governance-autopilot-blocked");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("decision "), "{run_text}");
    assert!(!run_text.contains("latest_governance_runtime:"), "{run_text}");
    assert!(!run_text.contains("latest_governance_state:"), "{run_text}");
}
