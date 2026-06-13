use std::fs;
use std::path::Path;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_approval_workspace, temp_canon_autopilot_blocked_workspace,
    temp_canon_security_approval_workspace, temp_canon_security_assessment_workspace,
    terminal_text,
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

fn bootstrap_change(workspace: &Path) {
    assert_eq!(
        run_boundline_in(workspace, &["goal", "--goal", "Update the checkout confirmation copy"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(workspace, &["flow", "change"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(workspace, &["plan"]).status.code(), Some(0));
}

fn rewrite_governance_flow_name(workspace: &Path, flow_name: &str) {
    let path = workspace.join(".boundline/execution.json");
    let mut profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let stages = profile
        .get_mut("governance")
        .and_then(|governance| governance.get_mut("stages"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("governance stages should exist");

    for stage in stages {
        stage["flow_name"] = serde_json::Value::String(flow_name.to_string());
    }

    fs::write(path, serde_json::to_string_pretty(&profile).unwrap()).unwrap();
}

fn rewrite_governance_canon_mode(workspace: &Path, canon_mode: &str) {
    let path = workspace.join(".boundline/execution.json");
    let mut profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let stages = profile
        .get_mut("governance")
        .and_then(|governance| governance.get_mut("stages"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("governance stages should exist");

    for stage in stages {
        stage["canon_mode"] = serde_json::Value::String(canon_mode.to_string());
    }

    fs::write(path, serde_json::to_string_pretty(&profile).unwrap()).unwrap();
}

#[test]
fn governance_autopilot_flow_selects_mode_and_refreshes_after_approval() {
    let workspace = temp_canon_approval_workspace("boundline-governance-autopilot-approval");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_started: bug-fix:investigate (discovery)"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_awaiting_approval: bug-fix:investigate (requested) [canon-run-approval]"
        ),
        "{run_text}"
    );
    assert!(
        run_text.contains(
            "execution_condition: waiting - governance approval is still pending before execution can continue"
        ),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: discovery"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: awaiting_approval"), "{status_text}");
    assert!(
        status_text.contains("governance_next_action: approve: Canon is waiting for approval"),
        "{status_text}"
    );
    assert!(status_text.contains("next_command: boundline status"), "{status_text}");

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let refreshed = run_boundline_in(&workspace, &["status"]);
    let refreshed_text = terminal_text(&refreshed);
    assert_eq!(refreshed.status.code(), Some(0), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_state: governed_ready"), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_approval: granted"), "{refreshed_text}");
    assert!(
        refreshed_text.contains("latest_governance_packet_ref: .canon/runs/canon-run-approval"),
        "{refreshed_text}"
    );

    let session_id = fs::read_to_string(workspace.join(".boundline/active-session")).unwrap();
    let session = fs::read_to_string(
        workspace.join(format!(".boundline/sessions/{}/session.json", session_id.trim())),
    )
    .unwrap();
    assert!(
        session.contains("\"authority_governance\""),
        "baseline authority contract should persist after approval refresh: {session}"
    );
    assert!(
        !session.contains("\"adaptive_governance\""),
        "baseline-only Canon responses should not synthesize the optional companion: {session}"
    );
}

#[test]
fn governance_autopilot_flow_blocks_required_stage_without_a_canon_runtime() {
    let workspace =
        temp_canon_autopilot_blocked_workspace("boundline-governance-autopilot-blocked");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_text.contains("governance_blocked: governance required Canon for bug-fix:investigate, but command 'canon-missing' is unavailable"), "{run_text}");
    assert!(run_text.contains("terminal_status: failed"), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: canon"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: blocked"), "{status_text}");
    assert!(status_text.contains("next_command: boundline inspect"), "{status_text}");
}

#[test]
fn governance_autopilot_flow_routes_verify_stage_through_security_assessment() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-governance-security-assessment");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(
        run_text.contains("governance_started: bug-fix:verify (security-assessment)"),
        "{run_text}"
    );
    assert!(
        run_text.contains("governance_completed: security assessment packet ready [.canon/runs/canon-run-security]"),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_changed_files: src/lib.rs"), "{status_text}");
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_runtime: local"), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_opt_out: true"), "{status_text}");
    assert!(
        status_text.contains("governance_lifecycle_mode_selection: auto-confirm"),
        "{status_text}"
    );

    let inspect = run_boundline_in(&workspace, &["inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_started: bug-fix:verify (security-assessment)"),
        "{inspect_text}"
    );
}

#[test]
fn governance_autopilot_flow_routes_change_verify_stage_through_security_assessment() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-governance-change-security-assessment");
    rewrite_governance_flow_name(&workspace, "change");
    bootstrap_change(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("routing: native (goal_plan)"), "{run_text}");
    assert!(
        run_text.contains("governance_started: change:verify (security-assessment)"),
        "{run_text}"
    );
    assert!(
        run_text.contains(
            "governance_completed: security assessment packet ready [.canon/runs/canon-run-security]"
        ),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("latest_changed_files: src/lib.rs"), "{status_text}");
    assert!(status_text.contains("latest_validation_status: passed"), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_runtime: local"), "{status_text}");
    assert!(status_text.contains("governance_lifecycle_opt_out: true"), "{status_text}");
    assert!(
        status_text.contains("governance_lifecycle_mode_selection: auto-confirm"),
        "{status_text}"
    );
}

#[test]
fn governance_autopilot_flow_refreshes_security_assessment_approval_through_status() {
    let workspace =
        temp_canon_security_approval_workspace("boundline-governance-security-approval");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(
        run_text.contains("governance_started: bug-fix:verify (security-assessment)"),
        "{run_text}"
    );
    assert!(
        run_text.contains(
            "governance_awaiting_approval: bug-fix:verify (requested) [canon-run-security-approval]"
        ),
        "{run_text}"
    );
    assert!(
        run_text.contains(
            "execution_condition: waiting - governance approval is still pending before execution can continue"
        ),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_governance_mode: security-assessment"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: awaiting_approval"), "{status_text}");
    assert!(
        status_text
            .contains("governance_next_action: approve: Canon is waiting for security approval"),
        "{status_text}"
    );
    assert!(status_text.contains("next_command: boundline status"), "{status_text}");

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let refreshed = run_boundline_in(&workspace, &["status"]);
    let refreshed_text = terminal_text(&refreshed);
    assert_eq!(refreshed.status.code(), Some(0), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_state: governed_ready"), "{refreshed_text}");
    assert!(refreshed_text.contains("latest_governance_approval: granted"), "{refreshed_text}");
    assert!(
        refreshed_text
            .contains("latest_governance_packet_ref: .canon/runs/canon-run-security-approval"),
        "{refreshed_text}"
    );
    assert!(refreshed_text.contains("next_command: boundline step"), "{refreshed_text}");
}

#[test]
fn governance_autopilot_flow_rejects_unsupported_future_canon_mode_configuration() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-governance-unsupported-security-mode");
    rewrite_governance_canon_mode(&workspace, "supply-chain-analysis");
    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );

    let flow = run_boundline_in(&workspace, &["flow", "bug-fix"]);
    let flow_text = terminal_text(&flow);
    if flow.status.code() != Some(0) {
        assert!(flow_text.contains("session error"), "{flow_text}");
        assert!(
            flow_text
                .contains("fixture runtime is invalid: workspace execution profile is invalid"),
            "{flow_text}"
        );
        assert!(
            flow_text.contains("cannot bind Canon mode") || flow_text.contains("unknown variant"),
            "{flow_text}"
        );
        return;
    }

    let plan = run_boundline_in(&workspace, &["plan"]);
    let plan_text = terminal_text(&plan);
    if plan.status.code() != Some(0) {
        assert!(plan_text.contains("session error"), "{plan_text}");
        assert!(
            plan_text
                .contains("fixture runtime is invalid: workspace execution profile is invalid"),
            "{plan_text}"
        );
        assert!(
            plan_text.contains("cannot bind Canon mode") || plan_text.contains("unknown variant"),
            "{plan_text}"
        );
        return;
    }

    assert!(plan_text.contains("routing: native (goal_plan)"), "{plan_text}");
    assert!(plan_text.contains("execution_path: native_goal_plan"), "{plan_text}");

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_ne!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("session error"), "{run_text}");
    assert!(
        run_text.contains("fixture runtime is invalid: workspace execution profile is invalid"),
        "{run_text}"
    );
    assert!(
        run_text.contains("cannot bind Canon mode") || run_text.contains("unknown variant"),
        "{run_text}"
    );
}
