use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use synod::domain::session::{ActiveSessionRecord, SessionStatus};
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("synod-session-contract-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"synod-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
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
fn start_creates_a_valid_workspace_scoped_session_record() {
    let workspace = temp_workspace();
    let output = run_synod_in(&workspace, &["start"]);
    let text = terminal_text(&output);
    let session_path = workspace.join(".synod").join("session.json");

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
