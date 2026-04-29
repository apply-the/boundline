use std::fs;

use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};
use synod::domain::session::{ActiveSessionRecord, SessionStatus};

#[test]
fn capture_and_plan_persist_the_active_goal_and_native_goal_plan() {
    let workspace = temp_fixture_workspace("synod-session-command-contract");
    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_synod_in(&workspace, &["capture", "--goal", "Fix the failing add test"]);
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let plan = run_synod_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let session_path = workspace.join(".synod").join("session.json");
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
