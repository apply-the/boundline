use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::json;
use uuid::Uuid;

fn run_limits(max_steps: usize) -> serde_json::Value {
    json!({
        "max_steps": max_steps,
        "max_retries": 0,
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

fn execution_profile(name: &str, max_steps: usize) -> serde_json::Value {
    json!({
        "name": name,
        "read_targets": ["fixture-target.txt"],
        "validation_command": {
            "program": "sh",
            "args": ["-c", "grep -q green fixture-target.txt"]
        },
        "limits": run_limits(max_steps),
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

fn temp_workspace() -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("boundline-flow-status-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = \"boundline-fixture\"\nversion = \"0.4.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(workspace.join("fixture-target.txt"), "red\n").unwrap();
    fs::write(
        workspace.join(".boundline").join("execution.json"),
        serde_json::to_vec_pretty(&execution_profile("flow-status", 8)).unwrap(),
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

#[test]
fn change_flow_status_and_next_include_flow_stage_fields() {
    let workspace = temp_workspace();
    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Apply the pricing change"]).status.code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["flow", "change"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let status_output = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status_output);
    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("active_flow: change"), "{status_text}");
    assert!(status_text.contains("current_stage: understand-change"), "{status_text}");
    assert!(status_text.contains("stage_progress: 1/3"), "{status_text}");
    assert!(status_text.contains("execution_path: native_goal_plan"), "{status_text}");
    assert!(status_text.contains("next_command: boundline run"), "{status_text}");
    assert!(!status_text.contains("current_step_id:"), "{status_text}");

    let next_output = run_boundline_in(&workspace, &["next"]);
    let next_text = terminal_text(&next_output);
    assert_eq!(next_output.status.code(), Some(0), "{next_text}");
    assert!(next_text.contains("active_flow: change"), "{next_text}");
    assert!(next_text.contains("current_stage: understand-change"), "{next_text}");
    assert!(next_text.contains("stage_progress: 1/3"), "{next_text}");
    assert!(next_text.contains("next_command: boundline run"), "{next_text}");
}

#[test]
fn status_without_a_flow_omits_flow_specific_fields() {
    let workspace = temp_workspace();
    assert_eq!(
        run_boundline_in(&workspace, &["goal", "--goal", "Fix the checkout flow"]).status.code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(&workspace, &["plan"]).status.code(), Some(0));

    let status_output = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status_output);
    assert_eq!(status_output.status.code(), Some(0), "{status_text}");
    assert!(!status_text.contains("active_flow:"), "{status_text}");
    assert!(!status_text.contains("current_stage:"), "{status_text}");
    assert!(!status_text.contains("stage_progress:"), "{status_text}");
}
