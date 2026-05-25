use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
use serde_json::json;
use uuid::Uuid;

fn run_limits(max_steps: usize, max_retries: usize) -> serde_json::Value {
    json!({
        "max_steps": max_steps,
        "max_retries": max_retries,
        "max_replans": 0,
        "terminal_precedence": [
            "goal_satisfied",
            "unrecoverable_error",
            "task_not_credible",
            "retry_budget_exhausted",
            "replan_budget_exhausted",
            "step_limit_exceeded",
            "no_credible_next_step"
        ]
    })
}

fn execution_profile(name: &str, max_steps: usize, max_retries: usize) -> serde_json::Value {
    json!({
        "name": name,
        "read_targets": ["fixture-target.txt"],
        "validation_command": {
            "program": "sh",
            "args": ["-c", "grep -q green fixture-target.txt"]
        },
        "limits": run_limits(max_steps, max_retries),
        "attempts": [
            {
                "attempt_id": "fix-target",
                "summary": "Replace red with green",
                "failure_mode": "terminal",
                "changes": [
                    {"path": "fixture-target.txt", "find": "red", "replace": "green"}
                ]
            }
        ]
    })
}

fn temp_workspace(max_retries: usize) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("boundline-flow-command-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(workspace.join("fixture-target.txt"), "red\n").unwrap();
    fs::write(
        workspace.join(".boundline").join("execution.json"),
        serde_json::to_vec_pretty(&execution_profile("flow-command", 6, max_retries)).unwrap(),
    )
    .unwrap();
    workspace
}

fn run_boundline_in(workspace: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_boundline"))
        .args(args)
        .current_dir(workspace)
        .output()
        .unwrap()
}

fn terminal_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn load_session_record(workspace: &std::path::Path) -> ActiveSessionRecord {
    serde_json::from_slice(&fs::read(workspace.join(".boundline").join("session.json")).unwrap())
        .unwrap()
}

#[test]
fn flow_command_binds_bug_fix_to_the_active_session() {
    let workspace = temp_workspace(0);
    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );

    let output = run_boundline_in(&workspace, &["flow", "bug-fix"]);
    let text = terminal_text(&output);
    assert_eq!(output.status.code(), Some(0), "{text}");
    assert!(text.contains("active_flow: bug-fix"), "{text}");
    assert!(text.contains("current_stage: investigate"), "{text}");

    let record = load_session_record(&workspace);
    record.validate().unwrap();
    let active_flow = record.active_flow.expect("flow should be persisted");
    assert_eq!(active_flow.flow_name, "bug-fix");
    assert_eq!(active_flow.current_stage_id, "investigate");
    assert_eq!(active_flow.current_stage_index, 0);
    assert_eq!(active_flow.total_stages, 3);
    assert_eq!(record.latest_status, SessionStatus::GoalCaptured);
}

#[test]
fn flow_command_rejects_unknown_flow_names_with_guidance() {
    let workspace = temp_workspace(0);
    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );

    let output = run_boundline_in(&workspace, &["flow", "unknown-flow"]);
    let text = terminal_text(&output);

    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(text.contains("unknown flow `unknown-flow`"), "{text}");
    assert!(text.contains("next_command: boundline flow bug-fix"), "{text}");
}
