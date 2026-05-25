use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use boundline::domain::decision::DecisionType;
use boundline::domain::goal_plan::ContextPackCredibility;
use boundline::orchestrator::goal_planner::{
    AuthoredInputDocument, GoalPlannerError, PlanningContextSources, build_context_pack,
    build_goal_plan, build_goal_plan_with_sources, collect_workspace_signals, derive_tasks,
    scan_canon_artifacts,
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
    std::fs::create_dir_all(ws.join(".boundline")).unwrap();
    std::fs::write(ws.join(".boundline/config.toml"), "").unwrap();

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
fn collect_workspace_signals_detects_python_and_go_projects_and_skips_hidden_dirs() {
    let python_ws = temp_workspace("gp-python");
    std::fs::write(python_ws.join("pyproject.toml"), "[project]\nname='test'\n").unwrap();
    std::fs::write(python_ws.join("README.md"), "workspace\n").unwrap();
    std::fs::create_dir_all(python_ws.join(".git")).unwrap();
    std::fs::create_dir_all(python_ws.join("target")).unwrap();
    std::fs::write(python_ws.join(".git/ignored.txt"), "ignored\n").unwrap();
    std::fs::write(python_ws.join("target/generated.txt"), "generated\n").unwrap();

    let python_signals = collect_workspace_signals(&python_ws);
    assert_eq!(python_signals.language.as_deref(), Some("python"));
    assert_eq!(python_signals.file_count, 2);

    let go_ws = temp_workspace("gp-go");
    std::fs::write(go_ws.join("go.mod"), "module example.com/test\n").unwrap();

    let go_signals = collect_workspace_signals(&go_ws);
    assert_eq!(go_signals.language.as_deref(), Some("go"));
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
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::create_dir(ws.join("tests")).unwrap();
    std::fs::write(ws.join("src/lib.rs"), "pub fn render_feature() {}\n").unwrap();
    std::fs::write(ws.join("tests/basic.rs"), "#[test]\nfn it_works() {}\n").unwrap();

    let plan = build_goal_plan_with_sources(
        "implement a feature",
        &ws,
        &PlanningContextSources {
            authored_input_documents: vec![AuthoredInputDocument {
                label: "brief.md".to_string(),
                content: "Focus on src/lib.rs for the bounded feature work.".to_string(),
            }],
            authored_input_summary: Some("Need the feature target".to_string()),
            authored_input_sources: vec!["brief.md".to_string()],
            ..PlanningContextSources::default()
        },
        None,
    )
    .unwrap();
    assert_eq!(plan.goal_text, "implement a feature");
    assert!(!plan.tasks.is_empty());
    assert!(plan.workspace_signals.language.is_some());
    assert!(plan.context_pack.is_some());
    assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
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
    assert!(
        plan.context_primary_inputs()
            .iter()
            .any(|item| item.contains(".canon") || item.contains("rules.md"))
    );
}

#[test]
fn build_goal_plan_prefers_source_target_over_test_file_for_fix_goals() {
    let ws = temp_workspace("gp-fix-target");
    std::fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"gp_fix_target\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::create_dir_all(ws.join("tests")).unwrap();
    std::fs::write(
        ws.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left - right }",
    )
    .unwrap();
    std::fs::write(
        ws.join("tests/addition.rs"),
        "#[test]\nfn red_to_green_addition() { assert_eq!(gp_fix_target::add(2, 2), 4); }",
    )
    .unwrap();

    let plan = build_goal_plan("fix the failing add test", &ws).unwrap();

    assert_eq!(plan.tasks[0].target, "src/lib.rs");
}

#[test]
fn build_context_pack_uses_authored_sources_and_workspace_files() {
    let ws = temp_workspace("gp-context");
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::write(
        ws.join("src/context_router.rs"),
        "pub fn build_context_router() {}\npub struct ContextSummary;",
    )
    .unwrap();

    let pack = build_context_pack(
        "build a context router",
        &ws,
        &PlanningContextSources {
            authored_input_summary: Some("Need a bounded context router".to_string()),
            authored_input_sources: vec!["brief.md".to_string()],
            authored_input_documents: vec![AuthoredInputDocument {
                label: "brief.md".to_string(),
                content: "Focus on src/context_router.rs for the bounded router work.".to_string(),
            }],
            execution_profile_read_targets: Vec::new(),
            negotiation_goal_summary: Some("ship the context router slice".to_string()),
            negotiation_resolution: Some("credible".to_string()),
            negotiation_acceptance_boundary: None,
            latest_trace_ref: Some(".boundline/traces/last.json".to_string()),
            workflow_progress: None,
            canon_capability_snapshot: None,
            compacted_canon_memory: None,
            latest_changed_files: Vec::new(),
            latest_validation_status: None,
        },
    );

    assert_eq!(pack.credibility, ContextPackCredibility::Credible);
    assert!(pack.selected_targets.iter().any(|item| item == "src/context_router.rs"));
    assert!(pack.inputs.iter().any(|item| item.reference == "Need a bounded context router"));
    assert!(pack.inputs.iter().any(|item| item.reference == ".boundline/traces/last.json"));
}

#[test]
fn build_goal_plan_with_sources_fails_when_context_is_insufficient() {
    let ws = temp_workspace("gp-insufficient");

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
            assert_eq!(
                goal_plan.context_pack.as_ref().map(|pack| pack.credibility),
                Some(ContextPackCredibility::Insufficient)
            );
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn build_goal_plan_supports_greenfield_constructive_goal_in_empty_workspace() {
    let ws = temp_workspace("gp-greenfield");

    let plan = build_goal_plan("build a react dashboard", &ws).unwrap();

    assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
    assert!(
        plan.context_summary()
            .as_deref()
            .is_some_and(|summary| summary.contains("greenfield goal seed"))
    );
    assert!(plan.context_pack.as_ref().is_some_and(|pack| {
        pack.selected_targets.iter().any(|target| target.starts_with("goal:"))
    }));
    assert!(plan.context_primary_inputs().iter().any(|input| input == "build a react dashboard"));
}

#[test]
fn build_goal_plan_inferrs_bug_fix_from_source_and_test_evidence_without_bug_keywords() {
    let ws = temp_workspace("gp-evidence-bug-fix");
    std::fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"gp_evidence_bug_fix\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::create_dir_all(ws.join("tests")).unwrap();
    std::fs::write(
        ws.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left - right }",
    )
    .unwrap();
    std::fs::write(
        ws.join("tests/red_to_green.rs"),
        "#[test]\nfn red_to_green_addition() { assert_eq!(gp_evidence_bug_fix::add(2, 2), 4); }",
    )
    .unwrap();

    let plan = build_goal_plan_with_sources(
        "investigate arithmetic path",
        &ws,
        &PlanningContextSources {
            authored_input_documents: vec![AuthoredInputDocument {
                label: "brief.md".to_string(),
                content:
                    "Inspect src/lib.rs and tests/red_to_green.rs for the bounded arithmetic issue."
                        .to_string(),
            }],
            authored_input_summary: Some("Need the arithmetic source and test".to_string()),
            authored_input_sources: vec!["brief.md".to_string()],
            ..PlanningContextSources::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(plan.flow.as_ref().map(|flow| flow.flow_name.as_str()), Some("bug-fix"));
    assert!(
        plan.flow
            .as_ref()
            .unwrap()
            .confidence_reason
            .contains("selected targets span existing tests and source files")
    );
    assert_eq!(plan.tasks[1].decision_type_hint, Some(DecisionType::Fix));
    assert!(plan.verification_strategy.as_deref().unwrap().contains("tests/red_to_green.rs"));
}

#[test]
fn build_goal_plan_inferrs_change_from_source_focused_evidence_without_bug_keywords() {
    let ws = temp_workspace("gp-evidence-change");
    std::fs::write(
        ws.join("Cargo.toml"),
        "[package]\nname = \"gp_evidence_change\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::write(
        ws.join("src/dashboard.rs"),
        "pub fn render_dashboard() {}\npub struct DashboardState;",
    )
    .unwrap();

    let plan = build_goal_plan_with_sources(
        "shape dashboard surface",
        &ws,
        &PlanningContextSources {
            authored_input_documents: vec![AuthoredInputDocument {
                label: "brief.md".to_string(),
                content: "Shape src/dashboard.rs for the bounded dashboard surface.".to_string(),
            }],
            authored_input_summary: Some("Need the dashboard surface".to_string()),
            authored_input_sources: vec!["brief.md".to_string()],
            ..PlanningContextSources::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(plan.flow.as_ref().map(|flow| flow.flow_name.as_str()), Some("change"));
    assert!(
        plan.flow
            .as_ref()
            .unwrap()
            .confidence_reason
            .contains("selected targets focus on implementation files")
    );
    assert_eq!(plan.tasks[1].decision_type_hint, Some(DecisionType::Code));
    assert!(
        plan.verification_strategy
            .as_deref()
            .unwrap()
            .contains("review bounded workspace evidence")
    );
}
