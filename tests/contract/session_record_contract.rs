use std::fs;

use crate::workspace_fixture::{run_boundline_in, temp_fixture_workspace, terminal_text};
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};

#[test]
fn start_creates_a_valid_workspace_scoped_session_record() {
    let workspace = temp_fixture_workspace("boundline-session-contract");
    let output = run_boundline_in(&workspace, &["start"]);
    let text = terminal_text(&output);
    let session_path = workspace.join(".boundline").join("session.json");

    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(session_path.exists(), "{text}");

    let record: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(&session_path).unwrap()).unwrap();
    record.validate().unwrap();

    assert_eq!(record.workspace_ref, workspace.canonicalize().unwrap().to_string_lossy());
    assert_eq!(record.latest_status, SessionStatus::Initialized);
    assert_eq!(record.goal, None);
    assert_eq!(record.active_task, None);
}
