#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use uuid::Uuid;

pub type DashboardTestResult = Result<(), Box<dyn Error>>;

pub fn dashboard_workspace(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline").join("traces"))?;
    Ok(workspace)
}

pub fn write_session(
    workspace: &Path,
    status: &str,
    session_revision: u64,
) -> Result<PathBuf, Box<dyn Error>> {
    let session_path = workspace.join(".boundline").join("session.json");
    let latest_status = match status {
        "initialized" => SessionStatus::Initialized,
        "goal_captured" => SessionStatus::GoalCaptured,
        _ => SessionStatus::GoalCaptured,
    };
    let session = ActiveSessionRecord {
        session_id: format!("session-{session_revision}"),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Fix the failing checkout flow".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 1,
        updated_at: session_revision,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    };
    fs::write(&session_path, serde_json::to_vec_pretty(&session)?)?;
    Ok(session_path)
}

pub fn require(condition: bool, message: &str) -> DashboardTestResult {
    if condition { Ok(()) } else { Err(message.to_string().into()) }
}

pub fn require_eq<T>(actual: T, expected: T, label: &str) -> DashboardTestResult
where
    T: std::fmt::Debug + PartialEq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected:?}, got {actual:?}").into())
    }
}

pub fn require_contains(haystack: &str, needle: &str, label: &str) -> DashboardTestResult {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!("{label}: missing {needle:?} in {haystack:?}").into())
    }
}
