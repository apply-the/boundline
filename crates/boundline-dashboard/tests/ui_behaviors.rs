use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use boundline_core::domain::dashboard::{
    DashboardAuthority, DashboardBrandMark, DashboardColorProfile, DashboardPanels,
    DashboardSessionView, DashboardSnapshot, ExecutionCondition,
};
use boundline_core::domain::session::{ActiveSessionRecord, SessionStatus};
use boundline_dashboard::app::{DashboardCli, run};
use boundline_dashboard::input::{DashboardInput, DashboardInputState, DashboardPanelFocus};
use boundline_dashboard::state::{DashboardAppState, TerminalCapabilities};
use uuid::Uuid;

type TestResult = Result<(), Box<dyn Error>>;

static CURRENT_DIR_LOCK: Mutex<()> = Mutex::new(());

fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(message.to_string().into()) }
}

fn require_eq<T>(actual: T, expected: T, label: &str) -> TestResult
where
    T: std::fmt::Debug + PartialEq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected:?}, got {actual:?}").into())
    }
}

fn require_contains(haystack: &str, needle: &str, label: &str) -> TestResult {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!("{label}: missing {needle:?} in {haystack:?}").into())
    }
}

fn temp_workspace(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline").join("traces"))?;
    Ok(workspace)
}

fn write_goal_captured_session(workspace: &Path) -> Result<(), Box<dyn Error>> {
    let session = ActiveSessionRecord {
        session_id: "session-ui".to_string(),
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
        latest_status: SessionStatus::GoalCaptured,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 1,
        updated_at: 2,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
    };
    let session_path = workspace.join(".boundline").join("session.json");
    fs::write(session_path, serde_json::to_vec_pretty(&session)?)?;
    Ok(())
}

fn sample_snapshot() -> DashboardSnapshot {
    DashboardSnapshot {
        snapshot_id: "snapshot-ui".to_string(),
        workspace_ref: "/workspace".to_string(),
        captured_at: "unix-ms:1".to_string(),
        authority: DashboardAuthority::SessionNative,
        session_revision: Some(2),
        session: Some(DashboardSessionView {
            session_id: "session-ui".to_string(),
            goal: "Fix the failing checkout flow".to_string(),
            route_kind: "session_native".to_string(),
            route_owner: "runtime".to_string(),
            active_flow: None,
            flow_state: None,
            goal_plan_state: None,
            goal_plan_revision: None,
            current_stage: None,
            current_step_id: None,
            current_step_index: None,
            execution_condition: ExecutionCondition::Ready,
            latest_status: "goal_captured".to_string(),
            next_action_label: "Plan".to_string(),
            next_command: "boundline plan --workspace /workspace".to_string(),
            blocking_reason: None,
            compatibility_context: None,
        }),
        timeline: Vec::new(),
        panels: DashboardPanels::empty(),
        actions: Vec::new(),
        degraded_state: None,
        branding: DashboardBrandMark {
            wordmark_lines: vec!["boundline".to_string()],
            color_profile: DashboardColorProfile::Color,
            min_width: 20,
            fallback_label: "boundline".to_string(),
        },
    }
}

struct CurrentDirGuard {
    original: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Result<Self, Box<dyn Error>> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { original })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

#[test]
fn dashboard_input_navigation_wraps_forward_and_backward() -> TestResult {
    let mut state = DashboardInputState::default();

    for expected in [
        DashboardPanelFocus::GoalPlan,
        DashboardPanelFocus::Evidence,
        DashboardPanelFocus::Findings,
        DashboardPanelFocus::Checkpoints,
        DashboardPanelFocus::GovernedReferences,
        DashboardPanelFocus::Diagnostics,
        DashboardPanelFocus::Summary,
    ] {
        state.apply_navigation(DashboardInput::FocusNext);
        require_eq(state.focus, expected, "focus next")?;
    }

    state.apply_navigation(DashboardInput::Refresh);
    require_eq(state.focus, DashboardPanelFocus::Summary, "refresh does not move focus")?;

    for expected in [
        DashboardPanelFocus::Diagnostics,
        DashboardPanelFocus::GovernedReferences,
        DashboardPanelFocus::Checkpoints,
        DashboardPanelFocus::Findings,
        DashboardPanelFocus::Evidence,
        DashboardPanelFocus::GoalPlan,
        DashboardPanelFocus::Summary,
    ] {
        state.apply_navigation(DashboardInput::FocusPrevious);
        require_eq(state.focus, expected, "focus previous")?;
    }

    Ok(())
}

#[test]
fn dashboard_input_and_app_state_allow_only_one_action_at_a_time() -> TestResult {
    let mut input_state = DashboardInputState::default();
    require(input_state.begin_action(), "first input action should start")?;
    require(!input_state.begin_action(), "second input action should be rejected")?;
    input_state.finish_action();
    require_eq(input_state.action_in_progress, false, "input action reset")?;

    let mut app_state = DashboardAppState::default();
    app_state.replace_snapshot(sample_snapshot());
    require(app_state.snapshot.is_some(), "snapshot should be stored in app state")?;
    require(app_state.begin_action(), "first app action should start")?;
    require(!app_state.begin_action(), "second app action should be rejected")?;
    app_state.finish_action();
    require_eq(app_state.action_in_progress, false, "app action reset")
}

#[test]
fn terminal_capabilities_and_run_cover_active_degraded_and_snapshot_modes() -> TestResult {
    require_eq(TerminalCapabilities::detect(false).color, true, "color mode")?;
    require_eq(TerminalCapabilities::detect(true).color, false, "no-color mode")?;
    require_eq(TerminalCapabilities::detect(true).interactive, true, "interactive mode")?;

    let workspace = temp_workspace("dashboard-ui-active")?;
    write_goal_captured_session(&workspace)?;

    let interactive = run(DashboardCli {
        workspace: Some(workspace.clone()),
        no_color: false,
        snapshot_json: false,
    })?;
    require_contains(&interactive, "mode: interactive", "interactive render mode")?;

    let monochrome = run(DashboardCli {
        workspace: Some(workspace.clone()),
        no_color: true,
        snapshot_json: false,
    })?;
    require_contains(&monochrome, "mode: monochrome", "monochrome render mode")?;

    let json = run(DashboardCli {
        workspace: Some(workspace.clone()),
        no_color: true,
        snapshot_json: true,
    })?;
    require_contains(&json, "\"snapshot_id\"", "snapshot json")?;
    require_contains(&json, "\"session\"", "session json")?;

    let degraded_workspace = temp_workspace("dashboard-ui-degraded")?;
    let degraded = run(DashboardCli {
        workspace: Some(degraded_workspace.clone()),
        no_color: true,
        snapshot_json: false,
    })?;
    require_contains(&degraded, "mode: degraded", "degraded render mode")?;
    require_contains(&degraded, "missing_active_session", "degraded reason")?;

    let _current_dir_lock = CURRENT_DIR_LOCK.lock().map_err(|error| error.to_string())?;
    let _guard = CurrentDirGuard::change_to(&workspace)?;
    let from_current_dir =
        run(DashboardCli { workspace: None, no_color: false, snapshot_json: false })?;
    require_contains(&from_current_dir, "mode: interactive", "current-dir workspace resolution")
}

#[test]
fn binary_entrypoint_emits_snapshot_json_for_no_session_workspace() -> TestResult {
    let workspace = temp_workspace("dashboard-ui-binary")?;
    let output = Command::new(env!("CARGO_BIN_EXE_boundline-dashboard"))
        .args([
            "--workspace",
            workspace.to_string_lossy().as_ref(),
            "--snapshot-json",
            "--no-color",
        ])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    require_eq(output.status.code(), Some(0), "binary exit code")?;
    require_contains(&stdout, "\"degraded_state\"", "binary snapshot json")
}

#[test]
fn binary_entrypoint_supports_version_flag() -> TestResult {
    let output =
        Command::new(env!("CARGO_BIN_EXE_boundline-dashboard")).arg("--version").output()?;

    require_eq(output.status.code(), Some(0), "version exit code")?;
    let stdout = String::from_utf8(output.stdout)?;
    require_contains(&stdout, "boundline-dashboard", "version binary name")?;
    require_contains(&stdout, env!("CARGO_PKG_VERSION"), "version string")
}

#[test]
fn binary_entrypoint_reports_failure_for_session_store_errors() -> TestResult {
    let workspace = temp_workspace("dashboard-ui-binary-error")?;
    fs::create_dir_all(workspace.join(".boundline").join("session.json"))?;

    let output = Command::new(env!("CARGO_BIN_EXE_boundline-dashboard"))
        .args(["--workspace", workspace.to_string_lossy().as_ref()])
        .output()?;

    let stderr = String::from_utf8(output.stderr)?;
    require_eq(output.status.code(), Some(1), "binary error exit code")?;
    require_contains(&stderr, "session store error", "binary stderr")
}

#[test]
fn run_reports_error_for_session_store_failures() -> TestResult {
    let workspace = temp_workspace("dashboard-ui-run-error")?;
    fs::create_dir_all(workspace.join(".boundline").join("session.json"))?;

    let error =
        run(DashboardCli { workspace: Some(workspace), no_color: false, snapshot_json: false })
            .err()
            .ok_or("broken session store must return an error")?;

    require_contains(&error, "session store error", "run session store error")
}

#[test]
fn run_reports_error_when_workspace_is_omitted_and_current_directory_is_unavailable() -> TestResult
{
    let workspace = temp_workspace("dashboard-ui-missing-cwd")?;
    let _current_dir_lock = CURRENT_DIR_LOCK.lock().map_err(|error| error.to_string())?;
    let _guard = CurrentDirGuard::change_to(&workspace)?;
    fs::remove_dir_all(&workspace)?;

    let error = run(DashboardCli { workspace: None, no_color: false, snapshot_json: false })
        .err()
        .ok_or("missing current directory must return an error")?;

    require_contains(&error, "No such file", "missing current directory error")
}
