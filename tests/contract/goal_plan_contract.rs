use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use synod::domain::goal_plan::GoalPlanStatus;
use synod::orchestrator::goal_planner::build_goal_plan;

fn temp_workspace(prefix: &str) -> PathBuf {
    let ws = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&ws).unwrap();
    ws
}

#[test]
fn goal_plan_contract_produces_non_empty_tasks_from_workspace() {
    let ws = temp_workspace("gpc-1");

    // Set up a minimal Rust workspace
    std::fs::write(ws.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"")
        .unwrap();
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::write(ws.join("src/lib.rs"), "pub fn hello() {}").unwrap();
    std::fs::create_dir(ws.join("tests")).unwrap();
    std::fs::write(ws.join("tests/basic.rs"), "#[test] fn it_works() {}").unwrap();

    let plan = build_goal_plan("implement a new feature", &ws).unwrap();

    assert_eq!(plan.status, GoalPlanStatus::Draft);
    assert!(!plan.tasks.is_empty());
    assert!(!plan.plan_id.is_empty());
    assert_eq!(plan.goal_text, "implement a new feature");
    assert!(plan.workspace_signals.language.is_some());
    assert!(plan.workspace_signals.has_tests);
    assert!(plan.validate().is_ok());
}

#[test]
fn goal_plan_contract_includes_canon_evidence_when_present() {
    let ws = temp_workspace("gpc-2");

    std::fs::create_dir_all(ws.join(".canon")).unwrap();
    std::fs::write(ws.join(".canon/governance.json"), "{}").unwrap();

    let plan = build_goal_plan("deliver a governed artifact", &ws).unwrap();

    assert!(!plan.source_evidence.is_empty());
    assert!(plan.workspace_signals.has_canon);
}
