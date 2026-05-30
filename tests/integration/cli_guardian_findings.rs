use std::fs;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_goal, execute_plan, execute_run, execute_status};

use crate::workspace_fixture::temp_fixture_workspace;

#[test]
fn run_persists_guardian_findings_and_status_projection() {
    let workspace = temp_fixture_workspace("boundline-cli-guardian-findings");
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 {\n    let total = Some(left + right).unwrap();\n    total - right\n}\n",
    )
    .unwrap();

    execute_goal(Some(&workspace), Some("fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
    let run_report = execute_run(Some(&workspace)).unwrap();
    let status_report = execute_status(Some(&workspace)).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let plan = session.goal_plan.expect("goal plan should remain persisted");

    assert!(plan.guidance_guardian.guardian_findings_summary.is_some());
    assert!(
        plan.guidance_guardian
            .guardian_findings
            .iter()
            .any(|finding| finding.summary.contains("unwrap/expect shortcut detected"))
    );
    assert!(
        plan.guidance_guardian
            .guardian_timeline
            .iter()
            .any(|line| line.contains("rust-language-safety: completed"))
    );
    assert!(
        status_report.terminal_output.contains("guardian_findings_summary:"),
        "{}",
        status_report.terminal_output
    );
    assert!(
        status_report.terminal_output.contains("guardian_timeline:"),
        "{}",
        status_report.terminal_output
    );
    assert!(run_report.terminal_output.contains("trace:"), "{}", run_report.terminal_output);
}
