use std::fs;

use crate::workspace_fixture::{run_synod_in, temp_fixture_workspace, terminal_text};
use synod::domain::session::{ActiveSessionRecord, SessionStatus};

#[test]
fn capture_and_plan_persist_the_active_goal_and_planned_task_snapshot() {
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
    assert!(record.active_task.is_some());
    assert_eq!(record.active_task.as_ref().unwrap().plan.current_step_index, 0);
    assert_eq!(record.latest_trace_ref, None);
}
