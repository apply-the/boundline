use std::fs;
use std::path::{Path, PathBuf};

use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{
    HybridOutcome, RetrievalMatchOrigin, RetrievalMode, RetrievalState,
};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection,
};
use serde::Deserialize;

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_READY_VALUE, force_semantic_vector_state_override, temp_empty_workspace,
};
const MIN_SEMANTIC_RECALL_THRESHOLD: f64 = 1.0;

#[derive(Debug, Deserialize)]
struct SemanticRecallCase {
    name: String,
    goal: String,
    expected_semantic_ref: String,
    files: Vec<SemanticRecallFile>,
}

#[derive(Debug, Deserialize)]
struct SemanticRecallFile {
    path: String,
    content: String,
    primary: bool,
    rationale: String,
}

fn semantic_recall_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/context_intelligence_semantic_eval/cases.json")
}

fn load_semantic_recall_cases() -> Vec<SemanticRecallCase> {
    serde_json::from_str(&fs::read_to_string(semantic_recall_fixture_path()).unwrap()).unwrap()
}

fn write_case_workspace(prefix: &str, files: &[SemanticRecallFile]) -> PathBuf {
    let workspace = temp_empty_workspace(prefix);
    for file in files {
        let path = workspace.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, &file.content).unwrap();
    }
    workspace
}

fn build_case_projection(
    case: &SemanticRecallCase,
    workspace: &Path,
) -> boundline::domain::context_intelligence::AdvancedContextProjection {
    let inputs = case
        .files
        .iter()
        .map(|file| ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: file.path.clone(),
            rationale: file.rationale.clone(),
            source: "workspace_scan".to_string(),
            primary: file.primary,
        })
        .collect::<Vec<_>>();

    build_advanced_context_projection(
        &case.goal,
        workspace,
        &inputs,
        &[],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    )
}

#[test]
fn semantic_recall_corpus_meets_curated_threshold() {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let cases = load_semantic_recall_cases();
    let mut matched_cases = 0usize;

    for case in &cases {
        let workspace = write_case_workspace(
            &format!("boundline-context-intelligence-semantic-recall-{}", case.name),
            &case.files,
        );
        let projection = build_case_projection(case, &workspace);

        assert_eq!(projection.retrieval_mode, RetrievalMode::Local, "{}", case.name);
        assert_eq!(projection.retrieval_state, RetrievalState::Selected, "{}", case.name);
        assert_eq!(projection.hybrid_outcome, HybridOutcome::Expanded, "{}", case.name);
        if projection.selected_evidence.iter().any(|candidate| {
            candidate.source_ref == case.expected_semantic_ref
                && candidate.match_origin == RetrievalMatchOrigin::SemanticExpand
        }) {
            matched_cases += 1;
        }
    }

    let recall = matched_cases as f64 / cases.len() as f64;
    assert!(
        recall >= MIN_SEMANTIC_RECALL_THRESHOLD,
        "semantic recall {recall:.3} fell below threshold {MIN_SEMANTIC_RECALL_THRESHOLD:.3}"
    );
}
