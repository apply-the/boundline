use std::fs;
use std::path::PathBuf;

use boundline::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use boundline::domain::limits::RunLimits;
use boundline::domain::plan::Plan;
use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline::domain::step::Step;
use boundline::domain::task::{Task, TaskRunRequest};
use serde_json::json;
use uuid::Uuid;

fn temp_workspace() -> PathBuf {
    let workspace =
        std::env::temp_dir().join(format!("boundline-session-store-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn build_task(workspace_ref: &str) -> Task {
    let request = TaskRunRequest {
        goal: "Deliver a session-backed CLI".to_string(),
        input: json!({"ticket": "SESSION-1"}),
        session_id: "session-1".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    };

    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "bootstrap"})).unwrap()]).unwrap();

    Task::new("task-1", &request, plan).unwrap()
}

fn build_record(workspace_ref: &str) -> ActiveSessionRecord {
    ActiveSessionRecord {
        session_id: "session-1".to_string(),
        workspace_ref: workspace_ref.to_string(),
        goal: Some("Deliver a session-backed CLI".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(format!("{workspace_ref}/.boundline/traces/task-1.json")),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
    }
}

#[test]
fn file_session_store_round_trips_a_valid_record() {
    let workspace = temp_workspace();
    let store = FileSessionStore::for_workspace(&workspace);
    let record = build_record(workspace.to_str().unwrap());

    let path = store.persist(&record).unwrap();
    assert_eq!(path, workspace.join(".boundline").join("session.json"));

    let loaded = store.load().unwrap().unwrap();
    assert_eq!(loaded, record);
}

#[test]
fn file_session_store_returns_none_when_the_session_file_is_missing() {
    let workspace = temp_workspace();
    let store = FileSessionStore::for_workspace(&workspace);

    assert_eq!(store.load().unwrap(), None);
}

#[test]
fn file_session_store_rejects_invalid_records_before_writing() {
    let workspace = temp_workspace();
    let store = FileSessionStore::for_workspace(&workspace);
    let mut record = build_record(workspace.to_str().unwrap());
    record.session_id = " ".to_string();

    match store.persist(&record).unwrap_err() {
        SessionStoreError::InvalidRecord(message) => {
            assert!(message.contains("session_id"));
        }
        other => panic!("expected invalid record error, got {other:?}"),
    }
}

#[test]
fn file_session_store_clear_removes_the_persisted_session_file() {
    let workspace = temp_workspace();
    let store = FileSessionStore::for_workspace(&workspace);
    let record = build_record(workspace.to_str().unwrap());

    store.persist(&record).unwrap();
    store.clear().unwrap();

    assert_eq!(store.load().unwrap(), None);
}

#[test]
fn file_session_store_reports_invalid_persisted_records_during_load() {
    let workspace = temp_workspace();
    let store = FileSessionStore::for_workspace(&workspace);
    let mut record = build_record(workspace.to_str().unwrap());
    record.goal = None;

    let session_path = workspace.join(".boundline").join("session.json");
    fs::create_dir_all(session_path.parent().unwrap()).unwrap();
    fs::write(&session_path, serde_json::to_vec_pretty(&record).unwrap()).unwrap();

    match store.load().unwrap_err() {
        SessionStoreError::InvalidRecord(message) => {
            assert!(message.contains("requires a goal"), "{message}");
        }
        other => panic!("expected invalid record error, got {other:?}"),
    }
}
