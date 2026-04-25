use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use synod::domain::session::{ActiveSessionRecord, SessionStatus};
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace =
        std::env::temp_dir().join(format!("synod-session-command-contract-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.5.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    workspace
}

fn run_synod_in(workspace: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_synod")).args(args).current_dir(workspace).output().unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn capture_and_plan_persist_the_active_goal_and_planned_task_snapshot() {
    let workspace = temp_workspace();
    let start = run_synod_in(&workspace, &["start"]);
    assert_eq!(start.status.code(), Some(0), "{}", terminal_text(&start));

    let capture = run_synod_in(
        &workspace,
        &["capture", "--goal", "Summarize the current bounded developer flow"],
    );
    assert_eq!(capture.status.code(), Some(0), "{}", terminal_text(&capture));

    let plan = run_synod_in(&workspace, &["plan"]);
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let session_path = workspace.join(".synod").join("session.json");
    let record: ActiveSessionRecord =
        serde_json::from_slice(&fs::read(&session_path).unwrap()).unwrap();
    record.validate().unwrap();

    assert_eq!(record.goal.as_deref(), Some("Summarize the current bounded developer flow"));
    assert_eq!(record.latest_status, SessionStatus::Planned);
    assert!(record.active_task.is_some());
    assert_eq!(record.active_task.as_ref().unwrap().plan.current_step_index, 0);
    assert_eq!(record.latest_trace_ref, None);
}
