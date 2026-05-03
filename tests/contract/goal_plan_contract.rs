use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use boundline::domain::goal_plan::GoalPlanStatus;
use boundline::orchestrator::goal_planner::{
    GoalPlannerError, PlanningContextSources, build_goal_plan, build_goal_plan_with_sources,
};

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
    assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
    assert!(plan.context_pack.is_some());
    assert!(!plan.context_primary_inputs().is_empty());
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

#[test]
fn goal_plan_contract_reports_insufficient_context_for_empty_workspace() {
    let ws = temp_workspace("gpc-empty-context");

    let err = build_goal_plan_with_sources(
        "investigate a thing",
        &ws,
        &PlanningContextSources::default(),
        None,
    )
    .unwrap_err();

    match err {
        GoalPlannerError::InsufficientContext { summary, goal_plan } => {
            assert!(summary.contains("no credible bounded context"));
            assert_eq!(goal_plan.status, GoalPlanStatus::Draft);
            assert_eq!(goal_plan.context_credibility().as_deref(), Some("insufficient"));
        }
        other => panic!("unexpected planner error: {other}"),
    }
}
