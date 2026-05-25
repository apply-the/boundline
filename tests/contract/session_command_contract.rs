use std::fs;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};

#[test]
fn goal_and_plan_persist_the_active_goal_and_native_goal_plan() {
    let workspace = temp_fixture_workspace("boundline-session-command-contract");
    let goal = run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing add test"]);
    assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

    let plan = run_boundline_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let session_path = workspace.join(".boundline").join("session.json");
    let record: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(&session_path).unwrap()).unwrap();
    record.validate().unwrap();

    assert_eq!(record.goal.as_deref(), Some("Fix the failing add test"));
    assert_eq!(record.latest_status, SessionStatus::Planned);
    assert!(record.active_task.is_none());
    let goal_plan =
        record.goal_plan.as_ref().expect("goal plan should be present after native planning");
    assert!(!goal_plan.tasks.is_empty());
    let flow = goal_plan.flow.as_ref().expect("bug-fix flow proposal should be persisted");
    assert_eq!(flow.flow_name, "bug-fix");
    assert!(!flow.confirmed);
    assert_eq!(record.latest_trace_ref, None);
}
