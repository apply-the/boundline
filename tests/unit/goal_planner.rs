use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use synod::domain::decision::DecisionType;
use synod::orchestrator::goal_planner::{
    build_goal_plan, collect_workspace_signals, derive_tasks, scan_canon_artifacts,
};

fn temp_workspace(prefix: &str) -> PathBuf {
    let ws = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&ws).unwrap();
    ws
}

#[test]
fn collect_workspace_signals_detects_rust_project() {
    let ws = temp_workspace("gp-rust");
    std::fs::write(ws.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    std::fs::create_dir(ws.join("tests")).unwrap();

    let signals = collect_workspace_signals(&ws);
    assert_eq!(signals.language.as_deref(), Some("rust"));
    assert!(signals.has_tests);
    assert!(!signals.has_canon);
    assert!(!signals.has_config);
}

#[test]
fn collect_workspace_signals_detects_javascript_project() {
    let ws = temp_workspace("gp-js");
    std::fs::write(ws.join("package.json"), "{}").unwrap();

    let signals = collect_workspace_signals(&ws);
    assert_eq!(signals.language.as_deref(), Some("javascript"));
}

#[test]
fn collect_workspace_signals_detects_canon_and_config() {
    let ws = temp_workspace("gp-canon");
    std::fs::create_dir_all(ws.join(".canon")).unwrap();
    std::fs::create_dir_all(ws.join(".synod")).unwrap();
    std::fs::write(ws.join(".synod/config.toml"), "").unwrap();

    let signals = collect_workspace_signals(&ws);
    assert!(signals.has_canon);
    assert!(signals.has_config);
}

#[test]
fn collect_workspace_signals_counts_files() {
    let ws = temp_workspace("gp-count");
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::write(ws.join("src/a.rs"), "").unwrap();
    std::fs::write(ws.join("src/b.rs"), "").unwrap();
    std::fs::write(ws.join("README.md"), "").unwrap();

    let signals = collect_workspace_signals(&ws);
    assert!(signals.file_count >= 3);
}

#[test]
fn derive_tasks_always_starts_with_analyze() {
    let ws = temp_workspace("gp-analyze");
    let signals = collect_workspace_signals(&ws);
    let tasks = derive_tasks("add a feature", &ws, &signals);

    assert!(!tasks.is_empty());
    assert_eq!(tasks[0].decision_type_hint, Some(DecisionType::Analyze));
}

#[test]
fn derive_tasks_uses_fix_for_bug_keywords() {
    let ws = temp_workspace("gp-fix");
    let signals = collect_workspace_signals(&ws);
    let tasks = derive_tasks("fix the broken login", &ws, &signals);

    let fix_tasks: Vec<_> =
        tasks.iter().filter(|t| t.decision_type_hint == Some(DecisionType::Fix)).collect();
    assert!(!fix_tasks.is_empty());
}

#[test]
fn derive_tasks_uses_code_for_non_bug_goals() {
    let ws = temp_workspace("gp-code");
    let signals = collect_workspace_signals(&ws);
    let tasks = derive_tasks("implement a dashboard", &ws, &signals);

    let code_tasks: Vec<_> =
        tasks.iter().filter(|t| t.decision_type_hint == Some(DecisionType::Code)).collect();
    assert!(!code_tasks.is_empty());
}

#[test]
fn derive_tasks_adds_test_step_when_tests_exist() {
    let ws = temp_workspace("gp-test");
    std::fs::create_dir(ws.join("tests")).unwrap();

    let signals = collect_workspace_signals(&ws);
    let tasks = derive_tasks("add a feature", &ws, &signals);

    let test_tasks: Vec<_> =
        tasks.iter().filter(|t| t.decision_type_hint == Some(DecisionType::Test)).collect();
    assert!(!test_tasks.is_empty());
}

#[test]
fn scan_canon_artifacts_returns_evidence_refs() {
    let ws = temp_workspace("gp-scan-canon");
    std::fs::create_dir_all(ws.join(".canon")).unwrap();
    std::fs::write(ws.join(".canon/artifact.json"), "{}").unwrap();

    let evidence = scan_canon_artifacts(&ws);
    assert_eq!(evidence.len(), 1);
    assert!(evidence[0].reference.contains("artifact.json"));
}

#[test]
fn scan_canon_artifacts_returns_empty_when_no_canon_dir() {
    let ws = temp_workspace("gp-no-canon");
    let evidence = scan_canon_artifacts(&ws);
    assert!(evidence.is_empty());
}

#[test]
fn build_goal_plan_produces_valid_plan() {
    let ws = temp_workspace("gp-build");
    std::fs::write(ws.join("Cargo.toml"), "[package]").unwrap();
    std::fs::create_dir(ws.join("tests")).unwrap();

    let plan = build_goal_plan("implement a feature", &ws).unwrap();
    assert_eq!(plan.goal_text, "implement a feature");
    assert!(!plan.tasks.is_empty());
    assert!(plan.workspace_signals.language.is_some());
    assert!(plan.validate().is_ok());
}

#[test]
fn build_goal_plan_rejects_empty_goal() {
    let ws = temp_workspace("gp-empty");
    let result = build_goal_plan("", &ws);
    assert!(result.is_err());
}

#[test]
fn build_goal_plan_includes_canon_evidence() {
    let ws = temp_workspace("gp-canon-ev");
    std::fs::create_dir_all(ws.join(".canon")).unwrap();
    std::fs::write(ws.join(".canon/rules.md"), "# rules").unwrap();

    let plan = build_goal_plan("add a feature", &ws).unwrap();
    assert!(!plan.source_evidence.is_empty());
}
