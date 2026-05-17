use std::path::PathBuf;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::cli::session::{execute_capture, execute_plan, execute_start};
use boundline::domain::context_intelligence::{
    HybridOutcome, RemoteTransmissionPolicyState, RetrievalMatchOrigin, RetrievalMode,
    SemanticCapabilityState, SemanticPolicyState,
};

use crate::workspace_fixture::temp_fixture_workspace;

// Seed a small Rust workspace so the planner can persist a stable local-only
// advanced-context projection for consumer-facing contract checks.
fn write_local_context_workspace(prefix: &str) -> PathBuf {
    temp_fixture_workspace(prefix)
}

#[test]
fn advanced_context_consumer_contract_persists_local_only_projection_shape() {
    let workspace = write_local_context_workspace("boundline-context-consumer-contract");

    execute_start(Some(&workspace)).unwrap();
    execute_capture(
        Some(&workspace),
        Some("fix the failing add path"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    execute_plan(Some(&workspace), Some("bug-fix"), false, false).unwrap();

    let session = FileSessionStore::for_workspace(&workspace).load().unwrap().unwrap();
    let advanced_context = session
        .goal_plan
        .as_ref()
        .and_then(|goal_plan| goal_plan.context_pack.as_ref())
        .and_then(|context_pack| context_pack.advanced_context.as_ref())
        .expect("advanced context projection should persist with the goal plan");

    assert_eq!(advanced_context.retrieval_mode, RetrievalMode::Local);
    assert_eq!(advanced_context.remote_policy_state, RemoteTransmissionPolicyState::LocalOnly);
    assert_eq!(advanced_context.semantic_policy_state, SemanticPolicyState::Disabled);
    assert_eq!(advanced_context.semantic_capability_state, SemanticCapabilityState::Unsupported);
    assert_eq!(advanced_context.hybrid_outcome, HybridOutcome::BaselineOnly);
    assert!(!advanced_context.used_remote);
    assert!(
        advanced_context
            .selected_evidence
            .iter()
            .any(|candidate| { candidate.source_ref == "src/lib.rs" })
    );
    assert!(
        advanced_context
            .relationships
            .iter()
            .any(|relationship| relationship.subject_ref == "src/lib.rs")
    );
    assert!(!advanced_context.impact_findings.is_empty());

    let serialized = serde_json::to_value(advanced_context).unwrap();
    let object = serialized.as_object().unwrap();
    assert_eq!(object.get("retrieval_mode").unwrap().as_str(), Some("local"));
    assert_eq!(object.get("semantic_policy_state").unwrap().as_str(), Some("disabled"));
    assert_eq!(object.get("semantic_capability_state").unwrap().as_str(), Some("unsupported"));
    assert_eq!(object.get("hybrid_outcome").unwrap().as_str(), Some("baseline_only"));
    assert!(object.contains_key("selected_evidence"));
    let selected_evidence = object
        .get("selected_evidence")
        .and_then(|value| value.as_array())
        .expect("selected evidence array");
    assert!(selected_evidence.iter().any(|candidate| {
        candidate.get("match_origin").and_then(|value| value.as_str())
            == Some(RetrievalMatchOrigin::Fts.as_str())
    }));
    assert!(object.contains_key("relationships"));
}
