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
fn canon_governance_workspace_projects_governed_stage_lineage_on_native_goal_plan_path() {
    let workspace = temp_canon_governance_workspace("synod-canon-governance-flow");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> canon"), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:investigate (discovery)"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_completed: discovery packet ready [.canon/runs/canon-run-investigate]"
        ),
        "{run_text}"
    );
    assert!(run_text.contains("governance_started: bug-fix:implement (implementation) from bug-fix:investigate (upstream_stage_context)"), "{run_text}");
    assert!(run_text.contains("governance_completed: implementation packet ready [.canon/runs/canon-run-implement] from bug-fix:investigate (upstream_stage_context)"), "{run_text}");

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_changed_files: src/lib.rs"), "{status_text}");
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:implement"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: implementation"), "{status_text}");
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

    let inspect = run_synod_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:investigate (discovery)"),
        "{inspect_text}"
    );
    assert!(
        inspect_text.contains("governance_started: bug-fix:implement (implementation)"),
        "{inspect_text}"
    );
}

#[test]
fn canon_governance_rejected_workspace_surfaces_explicit_governance_block_state() {
    let workspace = temp_canon_packet_rejection_workspace("synod-canon-governance-rejected");
    bootstrap_bug_fix(&workspace);

    let run = run_synod_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:investigate (discovery)"), "{run_text}");
    assert!(run_text.contains("governance_packet_rejected: governance packet was Rejected for stage bug-fix:investigate"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_blocked: governance packet was Rejected for stage bug-fix:investigate"
        ),
        "{run_text}"
    );

    let status = run_synod_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: running"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: blocked"), "{status_text}");
    assert!(status_text.contains("latest_governance_blocked_reason: governance packet was Rejected for stage bug-fix:investigate"), "{status_text}");

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
}
