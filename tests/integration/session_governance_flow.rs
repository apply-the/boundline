use std::fs;
use std::path::Path;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_approval_workspace, temp_optional_governance_workspace,
    temp_required_governance_workspace, terminal_text,
};

fn rewrite_approval_stub_with_governed_adaptive_companion(workspace: &Path) {
    fs::write(
        workspace.join(".boundline/canon-stub.sh"),
        r#"#!/bin/sh
cat >/dev/null
state=$(cat .canon/approval-state.txt 2>/dev/null | tr -d '\n')
if [ "$state" = "granted" ]; then
    printf '{"status":"governed_ready","run_ref":"canon-run-approval","packet_ref":".canon/runs/canon-run-approval","expected_document_refs":[".canon/runs/canon-run-approval/discovery.md"],"document_refs":[".canon/runs/canon-run-approval/discovery.md"],"approval_state":"granted","packet_readiness":"reusable","missing_sections":[],"authority_governance":{"contract_line":"authority-governance-v1","authority_zone":"green","change_class":"low-impact","intended_persona":"delivery-engineer","approval_state":"granted","packet_readiness":"reusable","risk":"low-impact"},"adaptive_governance":{"contract_line":"adaptive-governance-v1","governance_state":"rule","rollout_profile":"governed"},"headline":"approval granted packet ready","message":"Canon approval granted"}'
else
    printf '{"status":"awaiting_approval","run_ref":"canon-run-approval","packet_ref":".canon/runs/canon-run-approval","expected_document_refs":[".canon/runs/canon-run-approval/discovery.md"],"document_refs":[],"approval_state":"requested","packet_readiness":"pending","missing_sections":[],"authority_governance":{"contract_line":"authority-governance-v1","authority_zone":"green","change_class":"low-impact","intended_persona":"delivery-engineer","approval_state":"requested","packet_readiness":"pending","risk":"low-impact"},"adaptive_governance":{"contract_line":"adaptive-governance-v1","governance_state":"rule","rollout_profile":"governed"},"headline":"awaiting approval","message":"Canon is waiting for approval"}'
fi
"#,
    )
    .unwrap();
}

#[test]
fn run_in_optional_governance_workspace_uses_native_goal_plan_path() {
    let workspace = temp_optional_governance_workspace("boundline-session-governance-local");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("governance_selected: bug-fix:investigate -> local"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_completed: local governance packet ready for bug-fix:investigate"
        ),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: succeeded"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime: local"), "{status_text}");
    assert!(status_text.contains("latest_governance_runtime_state: advisory"), "{status_text}");
    assert!(status_text.contains("latest_governance_rollout_profile: minimal"), "{status_text}");
    assert!(
        status_text.contains(
            "latest_governance_reason: startup posture defaulted locally for low-trust surface"
        ),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_governance_approval_provenance: approval not required"),
        "{status_text}"
    );

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: succeeded"), "{inspect_text}");
    assert!(
        inspect_text.contains("governance_selected: bug-fix:investigate -> local"),
        "{inspect_text}"
    );
    assert!(inspect_text.contains("governance_runtime_state: advisory"), "{inspect_text}");
    assert!(inspect_text.contains("governance_rollout_profile: minimal"), "{inspect_text}");
}

#[test]
fn required_governance_workspace_blocks_on_native_goal_plan_path() {
    let workspace = temp_required_governance_workspace("boundline-session-governance-required");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(
        run_text.contains("governance_blocked: governance required Canon for bug-fix:investigate"),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: failed"), "{status_text}");
    assert!(status_text.contains("latest_governance_stage: bug-fix:investigate"), "{status_text}");
    assert!(status_text.contains("latest_governance_state: blocked"), "{status_text}");

    let inspect = run_boundline_in(
        &workspace,
        &["inspect", "--workspace", workspace.to_string_lossy().as_ref()],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: failed"), "{inspect_text}");
    assert!(
        inspect_text
            .contains("governance_blocked: governance required Canon for bug-fix:investigate"),
        "{inspect_text}"
    );
}

#[test]
fn approval_workspace_waits_on_investigate_governance_before_execution() {
    let workspace = temp_canon_approval_workspace("boundline-session-governance-approval-pending");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(run_text.contains("terminal_status: running"), "{run_text}");
    assert!(
        run_text.contains(
            "governance_awaiting_approval: bug-fix:investigate (requested) [canon-run-approval]"
        ),
        "{run_text}"
    );
    assert!(!run_text.contains("step investigate succeeded"), "{run_text}");
}

#[test]
fn approval_workspace_run_resumes_after_operator_grant() {
    let workspace = temp_canon_approval_workspace("boundline-session-governance-approval-resume");

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let initial_run = run_boundline_in(&workspace, &["run"]);
    let initial_run_text = terminal_text(&initial_run);
    assert_eq!(initial_run.status.code(), Some(0), "{initial_run_text}");
    assert!(
        initial_run_text.contains(
            "governance_awaiting_approval: bug-fix:investigate (requested) [canon-run-approval]"
        ),
        "{initial_run_text}"
    );

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let resumed_run = run_boundline_in(&workspace, &["run"]);
    let resumed_run_text = terminal_text(&resumed_run);
    assert_eq!(resumed_run.status.code(), Some(0), "{resumed_run_text}");
    assert!(resumed_run_text.contains("terminal_status: succeeded"), "{resumed_run_text}");
    assert!(
        !resumed_run_text.contains("refreshed governance approval state and returned"),
        "{resumed_run_text}"
    );
}

#[test]
fn approval_workspace_next_refreshes_and_step_graduates_requested_adaptive_posture() {
    let workspace =
        temp_canon_approval_workspace("boundline-session-governance-adaptive-graduation");
    rewrite_approval_stub_with_governed_adaptive_companion(&workspace);

    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let initial_run = run_boundline_in(&workspace, &["run"]);
    let initial_run_text = terminal_text(&initial_run);
    assert_eq!(initial_run.status.code(), Some(0), "{initial_run_text}");
    assert!(initial_run_text.contains("terminal_status: running"), "{initial_run_text}");

    let pending_status = run_boundline_in(&workspace, &["status"]);
    let pending_status_text = terminal_text(&pending_status);
    assert_eq!(pending_status.status.code(), Some(0), "{pending_status_text}");
    assert!(
        pending_status_text.contains("latest_governance_runtime_state: advisory"),
        "{pending_status_text}"
    );
    assert!(
        pending_status_text.contains("latest_governance_rollout_profile: minimal"),
        "{pending_status_text}"
    );
    assert!(
        pending_status_text.contains(
            "latest_governance_reason: startup posture defaulted locally for low-trust surface"
        ),
        "{pending_status_text}"
    );
    assert!(
        pending_status_text.contains(
            "latest_governance_approval_provenance: stronger posture remained inactive because operator approval is still requested"
        ),
        "{pending_status_text}"
    );
    assert!(
        pending_status_text.contains("next_command: boundline status"),
        "{pending_status_text}"
    );

    fs::write(workspace.join(".canon/approval-state.txt"), "granted\n").unwrap();

    let next = run_boundline_in(&workspace, &["next"]);
    let next_text = terminal_text(&next);
    assert_eq!(next.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("latest_governance_state: governed_ready"), "{next_text}");
    assert!(next_text.contains("latest_governance_runtime_state: rule"), "{next_text}");
    assert!(next_text.contains("latest_governance_rollout_profile: governed"), "{next_text}");
    assert!(
        next_text.contains(
            "latest_governance_reason: startup posture activated approved adaptive companion"
        ),
        "{next_text}"
    );
    assert!(
        next_text.contains(
            "latest_governance_approval_provenance: operator approval activated the requested stronger posture"
        ),
        "{next_text}"
    );
    assert!(next_text.contains("next_command: boundline step"), "{next_text}");

    let step = run_boundline_in(&workspace, &["step"]);
    let step_text = terminal_text(&step);
    assert_eq!(step.status.code(), Some(0), "{step_text}");
    assert!(
        !step_text.contains(
            "refreshed governance approval state and returned without executing another step"
        ),
        "{step_text}"
    );
    assert!(step_text.contains("current_stage: implement"), "{step_text}");
}
