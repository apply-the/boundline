use std::fs;
use std::path::Path;

use crate::workspace_fixture::{
    run_synod_in, temp_canon_approval_workspace, temp_canon_governance_workspace, terminal_text,
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
fn governance_session_contract_projects_canon_reuse_and_run_ref_fields() {
    let workspace = temp_canon_governance_workspace("synod-governance-session-contract");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> canon"), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:implement -> canon"), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: implementation"), "{status_text}");
    assert!(
        status_text.contains("latest_governance_run_ref: canon-run-implement"),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_packet_ref: .canon/runs/canon-run-implement"),
        "{status_text}"
    );
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
fn governance_session_contract_refreshes_approval_state_during_status() {
    let workspace = temp_canon_approval_workspace("synod-governance-approval-session");
    bootstrap_bug_fix(&workspace);

    let step = run_synod_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("latest_governance_state: awaiting_approval"), "{step_text}");
    assert!(step_text.contains("latest_governance_mode: discovery"), "{step_text}");
    assert!(step_text.contains("latest_governance_run_ref: canon-run-approval"), "{step_text}");
    assert!(step_text.contains("latest_governance_candidates: select_mode"), "{step_text}");

    let pending_status = run_synod_in(&workspace, &["status"]);
    let pending_status_text = terminal_text(&pending_status);
    assert_eq!(pending_status.status.code(), Some(0), "{pending_status_text}");
    assert!(
        pending_status_text
            .contains("governance_next_action: wait for approval and rerun synod status"),
        "{pending_status_text}"
    );

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_state: governed_ready"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: discovery"), "{status_text}");
    assert!(status_text.contains("latest_governance_run_ref: canon-run-approval"), "{status_text}");
    assert!(status_text.contains("next_command: synod step"), "{status_text}");
}
