use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
    RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
    RetrievalMatchOrigin, RetrievalMode, RetrievalScore, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState,
    SemanticPolicyState,
};

const SEMANTIC_FALLBACK_REASON: &str = "semantic acceleration is enabled but sqlite-vec support is unavailable; using baseline structured retrieval";

fn hybrid_projection_contract_fixture() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-semantic-projection-contract".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        semantic_policy_state: SemanticPolicyState::Local,
        semantic_capability_state: SemanticCapabilityState::Ready,
        hybrid_outcome: HybridOutcome::Expanded,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: Some(
            "semantic acceleration expanded the V1 candidate set with additional local evidence"
                .to_string(),
        ),
        selected_evidence: vec![
            RetrievedEvidenceCandidate {
                candidate_id: "candidate-selected-fts".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                source_ref: "src/lib.rs".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::Fts,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "selected through lexical bounded retrieval".to_string(),
                provenance_summary: "workspace file selected via lexical retrieval".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: RetrievalScore::from_raw(3.115),
                semantic_score: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            },
            RetrievedEvidenceCandidate {
                candidate_id: "candidate-selected-semantic".to_string(),
                source_kind: RetrievalSourceKind::WorkspaceFile,
                source_ref: "src/semantic.rs".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::SemanticExpand,
                selection_state: CandidateSelectionState::Selected,
                selection_reason:
                    "semantic similarity expanded the V1 candidate set with bounded local evidence"
                        .to_string(),
                provenance_summary: "workspace file selected via semantic expansion".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: RetrievalScore::from_raw(0.944),
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            },
        ],
        rejected_candidates: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-rejected-semantic".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/alternate.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::SemanticExpand,
            selection_state: CandidateSelectionState::Rejected,
            selection_reason:
                "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
                    .to_string(),
            provenance_summary: "workspace file rejected after semantic expansion".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: RetrievalScore::from_raw(0.812),
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }],
        semantic_trace_records: Vec::new(),
        relationships: Vec::new(),
        impact_findings: Vec::new(),
    }
}

fn fallback_projection_contract_fixture() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-semantic-fallback-contract".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        semantic_policy_state: SemanticPolicyState::Local,
        semantic_capability_state: SemanticCapabilityState::Unavailable,
        hybrid_outcome: HybridOutcome::Skipped,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: Some(SEMANTIC_FALLBACK_REASON.to_string()),
        selected_evidence: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-selected-baseline".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/lib.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::Fts,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "selected through lexical bounded retrieval".to_string(),
            provenance_summary: "workspace file selected via lexical retrieval".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: RetrievalScore::from_raw(3.115),
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }],
        rejected_candidates: Vec::new(),
        semantic_trace_records: Vec::new(),
        relationships: Vec::new(),
        impact_findings: Vec::new(),
    }
}

#[test]
fn semantic_projection_contract_serializes_hybrid_candidate_lineage() {
    let projection = hybrid_projection_contract_fixture();
    let serialized = serde_json::to_value(&projection).unwrap();
    let object = serialized.as_object().unwrap();

    assert_eq!(object.get("semantic_policy_state").unwrap().as_str(), Some("local"));
    assert_eq!(object.get("semantic_capability_state").unwrap().as_str(), Some("ready"));
    assert_eq!(object.get("semantic_engine").unwrap().as_str(), Some("sqlite_vec"));
    assert_eq!(object.get("hybrid_outcome").unwrap().as_str(), Some("expanded"));
    assert_eq!(object.get("vector_query_count").unwrap().as_u64(), Some(1));
    assert_eq!(object.get("vector_candidates_returned").unwrap().as_u64(), Some(2));
    assert_eq!(object.get("semantic_fallback_reason"), None);

    let selected = object
        .get("selected_evidence")
        .and_then(|value| value.as_array())
        .expect("selected evidence array");
    assert!(selected.iter().any(|candidate| {
        candidate.get("source_ref").and_then(|value| value.as_str()) == Some("src/lib.rs")
            && candidate.get("match_origin").and_then(|value| value.as_str()) == Some("fts")
            && candidate.get("selection_state").and_then(|value| value.as_str()) == Some("selected")
    }));
    assert!(selected.iter().any(|candidate| {
        candidate.get("source_ref").and_then(|value| value.as_str()) == Some("src/semantic.rs")
            && candidate.get("match_origin").and_then(|value| value.as_str())
                == Some("semantic_expand")
            && candidate.get("selection_state").and_then(|value| value.as_str()) == Some("selected")
            && candidate.get("collapsed_from_chunk_count").and_then(|value| value.as_u64())
                == Some(1)
            && candidate.get("semantic_score").and_then(|value| value.as_f64()).is_some()
    }));

    let rejected = object
        .get("rejected_candidates")
        .and_then(|value| value.as_array())
        .expect("rejected candidates array");
    assert!(rejected.iter().any(|candidate| {
        candidate.get("source_ref").and_then(|value| value.as_str()) == Some("src/alternate.rs")
            && candidate.get("match_origin").and_then(|value| value.as_str())
                == Some("semantic_expand")
            && candidate.get("selection_state").and_then(|value| value.as_str()) == Some("rejected")
            && candidate.get("collapsed_from_chunk_count").and_then(|value| value.as_u64())
                == Some(1)
            && candidate.get("semantic_score").and_then(|value| value.as_f64()).is_some()
    }));
}

#[test]
fn semantic_projection_contract_serializes_v1_fallback_state() {
    let projection = fallback_projection_contract_fixture();
    let serialized = serde_json::to_value(&projection).unwrap();
    let object = serialized.as_object().unwrap();

    assert_eq!(object.get("semantic_policy_state").unwrap().as_str(), Some("local"));
    assert_eq!(object.get("semantic_capability_state").unwrap().as_str(), Some("missing"));
    assert_eq!(object.get("semantic_engine").unwrap().as_str(), Some("baseline_json"));
    assert_eq!(object.get("hybrid_outcome").unwrap().as_str(), Some("skipped"));
    assert_eq!(object.get("vector_query_count").unwrap().as_u64(), Some(0));
    assert_eq!(object.get("vector_candidates_returned").unwrap().as_u64(), Some(0));
    assert_eq!(
        object.get("semantic_fallback_reason").unwrap().as_str(),
        Some(SEMANTIC_FALLBACK_REASON)
    );
    assert_eq!(object.get("terminal_reason").unwrap().as_str(), Some(SEMANTIC_FALLBACK_REASON));
    assert_eq!(
        object.get("selected_evidence").and_then(|value| value.as_array()).map(Vec::len),
        Some(1)
    );
    assert_eq!(
        object.get("rejected_candidates").and_then(|value| value.as_array()).map(Vec::len),
        None
    );
}
