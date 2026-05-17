use std::fs;
use std::path::PathBuf;

use boundline::domain::context_intelligence::{
    ImpactFindingKind, RelationshipKind, RetrievalMode, RetrievalState,
};
use boundline::orchestrator::goal_planner::{PlanningContextSources, build_context_pack};
use uuid::Uuid;

/// Creates a temporary workspace root for bounded context-intelligence tests.
fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

#[test]
fn build_context_pack_projects_selected_local_evidence_for_workspace_targets() {
    let workspace = temp_workspace("boundline-context-intelligence-local");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("tests/lib.rs"),
        "#[test]\nfn add_works() { assert_eq!(crate::add(2, 2), 4); }\n",
    )
    .unwrap();

    let context_pack = build_context_pack(
        "Fix the add implementation",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Selected);
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| candidate.source_ref == "src/lib.rs")
    );
    assert!(advanced_context.relationships.iter().any(|relationship| {
        relationship.relationship_kind == RelationshipKind::ExercisesTest
            && relationship.subject_ref == "src/lib.rs"
    }));
}

#[test]
fn build_context_pack_records_missing_test_findings_for_uncovered_targets() {
    let workspace = temp_workspace("boundline-context-intelligence-impact");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/engine.rs"), "pub fn reconcile_plan() -> bool { true }\n")
        .unwrap();

    let context_pack = build_context_pack(
        "Reconcile the planner engine",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert!(advanced_context.relationships.iter().any(|relationship| {
        relationship.relationship_kind == RelationshipKind::RequiresEvidence
            && relationship.subject_ref == "src/engine.rs"
    }));
    assert!(advanced_context.impact_findings.iter().any(|finding| {
        finding.finding_kind == ImpactFindingKind::MissingTest
            && finding.subject_ref == "tests/engine.rs"
    }));
}

#[test]
fn build_context_pack_projects_advanced_context_from_authored_input_without_files() {
    let workspace = temp_workspace("boundline-context-intelligence-insufficient");
    let context_pack = build_context_pack(
        "Plan a bounded change from authored input",
        &workspace,
        &PlanningContextSources {
            authored_input_summary: Some(
                "operator notes: refresh the bounded reconciliation flow".to_string(),
            ),
            ..PlanningContextSources::default()
        },
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Selected);
    assert!(!advanced_context.selected_evidence.is_empty());
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| candidate.source_ref.contains("operator notes:"))
    );
}

#[test]
fn build_context_pack_respects_disabled_advanced_context_policy() {
    let workspace = temp_workspace("boundline-context-intelligence-disabled");
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join(".boundline/config.toml"),
        "version = 1\n\n[routing.advanced_context]\nretrieval_mode = \"disabled\"\nremote_policy = \"blocked\"\n",
    )
    .unwrap();

    let context_pack = build_context_pack(
        "Fix the add implementation",
        &workspace,
        &PlanningContextSources::default(),
    );
    let advanced_context = context_pack.advanced_context.expect("advanced context projection");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Disabled);
    assert_eq!(advanced_context.retrieval_state, RetrievalState::Insufficient);
    assert!(advanced_context.selected_evidence.is_empty());
    assert!(
        advanced_context
            .terminal_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("disabled by configuration"))
    );
}
