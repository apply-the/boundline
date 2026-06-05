use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
    RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
    RetrievalMatchOrigin, RetrievalMode, RetrievalSourceKind, RetrievalStalenessState,
    RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState,
    SemanticTraceEventKind, SemanticTraceRecord,
};
use boundline::domain::governance::CanonSemanticProvenanceBoundary;

fn canon_projection_contract_fixture() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-canon-semantic-contract".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        semantic_policy_state: SemanticPolicyState::Local,
        semantic_capability_state: SemanticCapabilityState::Ready,
        hybrid_outcome: HybridOutcome::BaselineOnly,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: Some(
            "semantic acceleration evaluated the bounded query but kept the V1 candidate set unchanged"
                .to_string(),
        ),
        selected_evidence: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-canon-selected".to_string(),
            source_kind: RetrievalSourceKind::CanonArtifact,
            source_ref: ".canon/planner-guidance.md".to_string(),
            authority_rank: AuthorityRank::Canon,
            match_origin: RetrievalMatchOrigin::Fts,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "matched SQLite FTS evidence for the bounded goal".to_string(),
            provenance_summary: ".canon/planner-guidance.md via canon_scan (selected governed guidance)"
                .to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: Some("v1".to_string()),
            canon_semantic_provenance_ref: Some(
                ".canon/planner-guidance.md#section:overview".to_string(),
            ),
        }],
        rejected_candidates: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-canon-rejected".to_string(),
            source_kind: RetrievalSourceKind::CanonArtifact,
            source_ref: ".canon/excluded-guidance.md".to_string(),
            authority_rank: AuthorityRank::Canon,
            match_origin: RetrievalMatchOrigin::StructuredFallback,
            selection_state: CandidateSelectionState::Rejected,
            selection_reason:
                "Canon semantic compatibility skipped the artifact: excluded by Canon semantic policy"
                    .to_string(),
            provenance_summary: ".canon/excluded-guidance.md via canon_scan (excluded governed guidance)"
                .to_string(),
            compatibility_state: RetrievalCompatibilityState::PolicyBlocked,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: Some("v1".to_string()),
            canon_semantic_provenance_ref: Some(
                ".canon/excluded-guidance.md#section:overview".to_string(),
            ),
        }],
        semantic_trace_records: vec![SemanticTraceRecord {
            record_id: "trace-canon-skip".to_string(),
            event_kind: SemanticTraceEventKind::CanonArtifactSkipped,
            candidate_ref: Some(".canon/excluded-guidance.md".to_string()),
            match_origin: None,
            compatibility_state: Some(RetrievalCompatibilityState::PolicyBlocked),
            semantic_score: None,
            canon_artifact_class: Some("stable".to_string()),
            canon_semantic_contract_line: Some("v1".to_string()),
            canon_semantic_provenance_boundary: Some(CanonSemanticProvenanceBoundary::Section),
            canon_semantic_provenance_ref: Some(
                ".canon/excluded-guidance.md#section:overview".to_string(),
            ),
            reason: "excluded by Canon semantic policy".to_string(),
        }],
        relationships: Vec::new(),
        impact_findings: Vec::new(),
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
    }
}

#[test]
fn canon_semantic_projection_contract_preserves_canon_metadata_and_skip_reason() {
    let projection = canon_projection_contract_fixture();
    let serialized = serde_json::to_value(&projection).unwrap();
    let object = serialized.as_object().unwrap();

    let selected = object
        .get("selected_evidence")
        .and_then(|value| value.as_array())
        .expect("selected evidence array");
    assert!(selected.iter().any(|candidate| {
        candidate.get("source_kind").and_then(|value| value.as_str()) == Some("canon_artifact")
            && candidate.get("source_ref").and_then(|value| value.as_str())
                == Some(".canon/planner-guidance.md")
            && candidate.get("canon_semantic_contract_line").and_then(|value| value.as_str())
                == Some("v1")
            && candidate.get("canon_semantic_provenance_ref").and_then(|value| value.as_str())
                == Some(".canon/planner-guidance.md#section:overview")
    }));

    let rejected = object
        .get("rejected_candidates")
        .and_then(|value| value.as_array())
        .expect("rejected candidates array");
    assert!(rejected.iter().any(|candidate| {
        candidate.get("source_ref").and_then(|value| value.as_str())
            == Some(".canon/excluded-guidance.md")
            && candidate.get("compatibility_state").and_then(|value| value.as_str())
                == Some("policy_blocked")
            && candidate
                .get("selection_reason")
                .and_then(|value| value.as_str())
                .is_some_and(|reason| reason.contains("excluded by Canon semantic policy"))
    }));

    let trace_records = object
        .get("semantic_trace_records")
        .and_then(|value| value.as_array())
        .expect("semantic trace records array");
    assert!(trace_records.iter().any(|record| {
        record.get("event_kind").and_then(|value| value.as_str()) == Some("canon_artifact_skipped")
            && record.get("candidate_ref").and_then(|value| value.as_str())
                == Some(".canon/excluded-guidance.md")
            && record.get("canon_artifact_class").and_then(|value| value.as_str()) == Some("stable")
            && record.get("canon_semantic_contract_line").and_then(|value| value.as_str())
                == Some("v1")
            && record.get("canon_semantic_provenance_boundary").and_then(|value| value.as_str())
                == Some("section")
            && record.get("canon_semantic_provenance_ref").and_then(|value| value.as_str())
                == Some(".canon/excluded-guidance.md#section:overview")
            && record
                .get("reason")
                .and_then(|value| value.as_str())
                .is_some_and(|reason| reason.contains("excluded by Canon semantic policy"))
    }));
}
