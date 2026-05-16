use std::path::Path;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_approval_workspace, temp_canon_governance_workspace,
    temp_canon_security_assessment_workspace, terminal_text,
};

fn bootstrap_bug_fix(workspace: &Path) {
    assert_eq!(run_boundline_in(workspace, &["start"]).status.code(), Some(0));
    assert_eq!(
        run_boundline_in(workspace, &["capture", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(workspace, &["plan"]).status.code(), Some(0));
}

#[test]
fn governance_session_contract_native_run_projects_fixture_governance_fields() {
    let workspace = temp_canon_governance_workspace("boundline-governance-session-contract");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> canon"), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:implement (implementation) from bug-fix:investigate (upstream_stage_context)"), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_changed_files: src/lib.rs"), "{status_text}");
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:implement"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: implementation"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime_state: advisory"), "{status_text}");
    assert!(status_text.contains("latest_governance_rollout_profile: minimal"), "{status_text}");
    assert!(
        status_text
            .contains("latest_governance_reason: startup posture seeded from adaptive companion"),
        "{status_text}"
    );
    assert!(
        status_text.contains(
            "latest_governance_contract_lines: authority_contract_line: authority-governance-v1 | adaptive_contract_line: adaptive-governance-v1"
        ),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_approval_provenance: approval not required"),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_run_ref: canon-run-implement"),
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
fn governance_session_contract_native_planned_sessions_require_run_instead_of_step() {
    let workspace = temp_canon_approval_workspace("boundline-governance-approval-session");
    bootstrap_bug_fix(&workspace);

    let step = run_boundline_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_ne!(step.status.code(), Some(0), "{step_text}");
    assert!(step_text.contains("active session has no planned task"), "{step_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("next_command: boundline run"), "{status_text}");
    assert!(!status_text.contains("latest_governance_state:"), "{status_text}");
}

#[test]
fn governance_session_contract_surfaces_security_assessment_fields_on_native_route() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-governance-security-session-contract");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(
        run_text.contains("governance_started: bug-fix:verify (security-assessment)"),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_changed_files: src/lib.rs"), "{status_text}");
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:verify"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: security-assessment"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime_state: advisory"), "{status_text}");
    assert!(status_text.contains("latest_governance_rollout_profile: minimal"), "{status_text}");
    assert!(
        status_text.contains(
            "latest_governance_reason: startup posture defaulted locally for low-trust surface"
        ),
        "{status_text}"
    );
    assert!(
        status_text.contains(
            "latest_governance_contract_lines: authority_contract_line: authority-governance-v1 | adaptive_contract_line: unavailable"
        ),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_packet_ref: .canon/runs/canon-run-security"),
        "{status_text}"
    );
}
