use std::fs;
use std::path::PathBuf;

use boundline::domain::configuration::{AdvancedContextConfig, SemanticAccelerationPolicyState};
use boundline::domain::context_intelligence::{
    HybridOutcome, RetrievalMatchOrigin, RetrievalState, SemanticCapabilityState,
    SemanticPolicyState,
};
use boundline::domain::goal_plan::{ContextInput, ContextInputKind, ContextPackCredibility};
use boundline::orchestrator::context_intelligence::{
    AdvancedContextBuildState, build_advanced_context_projection,
};

use crate::workspace_fixture::{
    SEMANTIC_VECTOR_STATE_READY_VALUE, force_semantic_vector_state_override, temp_empty_workspace,
};

fn write_semantic_flow_workspace(prefix: &str) -> PathBuf {
    let workspace = temp_empty_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        concat!("pub fn planner() -> bool {\n", "    true\n", "}\n",),
    )
    .unwrap();
    fs::write(
        workspace.join("src/semantic.rs"),
        "pub fn reconcileConfigState() -> bool { true }\n",
    )
    .unwrap();
    workspace
}

#[test]
fn build_projection_surfaces_local_semantic_expansion_when_ready_is_forced_for_testing() {
    let _env_guard = force_semantic_vector_state_override(SEMANTIC_VECTOR_STATE_READY_VALUE);
    let workspace = write_semantic_flow_workspace("boundline-context-intelligence-semantic-flow");

    let projection = build_advanced_context_projection(
        "planner reconcile configuration state",
        &workspace,
        &[
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                source: "workspace_scan".to_string(),
                rationale: "selected bounded implementation surface".to_string(),
                primary: true,
            },
            ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/semantic.rs".to_string(),
                source: "workspace_scan".to_string(),
                rationale: "related implementation surface".to_string(),
                primary: false,
            },
        ],
        &[],
        AdvancedContextBuildState {
            credibility: ContextPackCredibility::Credible,
            staleness_reason: None,
            semantic_policy: SemanticAccelerationPolicyState::Local,
        },
        &AdvancedContextConfig::default(),
    );

    assert_eq!(projection.semantic_policy_state, SemanticPolicyState::Local);
    assert_eq!(projection.semantic_capability_state, SemanticCapabilityState::Ready);
    assert_eq!(projection.semantic_engine().as_str(), "sqlite_vec");
    assert_eq!(projection.hybrid_outcome, HybridOutcome::Expanded);
    assert_eq!(projection.retrieval_state, RetrievalState::Selected);
    assert_eq!(projection.vector_query_count(), 1);
    assert_eq!(projection.vector_candidates_returned(), 1);
    assert_eq!(projection.semantic_fallback_reason(), None);
    assert_eq!(projection.semantic_selected_count(), 1);
    assert!(projection.selected_evidence.iter().any(|candidate| {
        candidate.source_ref == "src/lib.rs" && candidate.match_origin == RetrievalMatchOrigin::Fts
    }));
    assert!(projection.selected_evidence.iter().any(|candidate| {
        candidate.source_ref == "src/semantic.rs"
            && candidate.match_origin == RetrievalMatchOrigin::SemanticExpand
            && candidate.semantic_score.is_some()
    }));
}
