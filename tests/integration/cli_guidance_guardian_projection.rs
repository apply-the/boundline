use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};

#[test]
fn verification_guardian_projection_is_phase_gated_and_reused_by_status_and_inspect() {
    let workspace = temp_fixture_workspace("boundline-cli-guidance-guardian-projection");
    fs::create_dir_all(workspace.join(".boundline/guardians")).unwrap();
    fs::write(
        workspace.join(".boundline/guardians/verification-only.toml"),
        "[guardians.verification-only]\ntitle = \"Verification Only\"\nkind = \"deterministic\"\napplies_to = [\"verification\"]\nrules = [\"verification_evidence\"]\nseverity_floor = \"warn\"\ncommand = \"builtin:validation-evidence\"\n",
    )
    .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    let plan_report = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

    let planned_session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let planned_goal_plan = planned_session.goal_plan.expect("goal plan should be persisted");
    assert!(
        !planned_goal_plan
            .guidance_guardian
            .loaded_guardian_sources
            .iter()
            .any(|source| source == ".boundline/guardians/verification-only.toml")
    );
    assert!(
        !plan_report.terminal_output.contains("verification-only"),
        "{}",
        plan_report.terminal_output
    );

    execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();
    let inspect_output = run_boundline_in(&workspace, &["--verbose", "inspect"]);
    let inspect_text = terminal_text(&inspect_output);

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let goal_plan = session.goal_plan.expect("goal plan should remain persisted");

    assert!(
        goal_plan
            .guidance_guardian
            .loaded_guardian_sources
            .iter()
            .any(|source| { source == ".boundline/guardians/verification-only.toml" })
    );
    assert!(
        goal_plan
            .guidance_guardian
            .guardian_timeline
            .iter()
            .any(|line| { line.contains("verification-only: completed") })
    );
    assert!(
        status_report.terminal_output.contains("verification-only: completed"),
        "{}",
        status_report.terminal_output
    );
    assert!(inspect_output.status.success(), "{}", inspect_text);
    assert!(inspect_text.contains("verification-only: completed"), "{}", inspect_text);
}
