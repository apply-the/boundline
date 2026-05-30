use boundline::cli::output::render_trace_summary;
use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
    RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
    RetrievalMatchOrigin, RetrievalMode, RetrievalScore, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState,
    SemanticPolicyState, SemanticTraceEventKind, SemanticTraceRecord,
};
use boundline::domain::governance::CanonSemanticProvenanceBoundary;
use boundline::domain::trace::TraceSummaryView;

fn semantic_trace_projection() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-semantic-contract".to_string(),
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
            "semantic acceleration expanded the V1 candidate set with 1 additional bounded match(es)"
                .to_string(),
        ),
        selected_evidence: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-selected-1".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/context_router.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::SemanticExpand,
            selection_state: CandidateSelectionState::Selected,
            selection_reason:
                "semantic similarity expanded the V1 candidate set with bounded local evidence"
                    .to_string(),
            provenance_summary: "workspace file selected through semantic expansion".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: RetrievalScore::from_raw(0.944),
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }],
        rejected_candidates: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-rejected-1".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/semantic.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::SemanticExpand,
            selection_state: CandidateSelectionState::Rejected,
            selection_reason:
                "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
                    .to_string(),
            provenance_summary: "workspace file evaluated through semantic expansion".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: RetrievalScore::from_raw(0.812),
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }],
        semantic_trace_records: vec![SemanticTraceRecord {
            record_id: "trace-semantic-outcome".to_string(),
            event_kind: SemanticTraceEventKind::HybridOutcomeRecorded,
            candidate_ref: Some("src/context_router.rs".to_string()),
            match_origin: Some(RetrievalMatchOrigin::SemanticExpand),
            compatibility_state: Some(RetrievalCompatibilityState::Compatible),
            semantic_score: RetrievalScore::from_raw(0.944),
            canon_artifact_class: Some("stable".to_string()),
            canon_semantic_contract_line: Some("v1".to_string()),
            canon_semantic_provenance_boundary: Some(CanonSemanticProvenanceBoundary::Section),
            canon_semantic_provenance_ref: Some(
                ".canon/context-router.md#section:overview".to_string(),
            ),
            reason: "semantic retrieval ended with retrieval state selected with hybrid outcome expanded"
                .to_string(),
        }],
        relationships: Vec::new(),
        impact_findings: Vec::new(),
    }
}

#[test]
fn trace_summary_contract_surfaces_semantic_rejection_details() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/semantic-contract-trace.json".to_string(),
        goal: "Recover semantically related bounded evidence".to_string(),
        advanced_context: Some(semantic_trace_projection()),
        ..TraceSummaryView::default()
    };

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    assert!(rendered.contains("semantic_policy_state: local"), "{rendered}");
    assert!(rendered.contains("semantic_capability_state: ready"), "{rendered}");
    assert!(rendered.contains("semantic_engine: sqlite_vec"), "{rendered}");
    assert!(rendered.contains("hybrid_outcome: expanded"), "{rendered}");
    assert!(rendered.contains("vector_query_count: 1"), "{rendered}");
    assert!(rendered.contains("vector_candidates_returned: 2"), "{rendered}");
    assert!(
        rendered.contains(
            "semantic_trace: hybrid_outcome_recorded ref=src/context_router.rs origin=semantic_expand compatibility=compatible semantic_score=0.944 canon_artifact_class=stable canon_contract=v1 canon_boundary=section canon_provenance=.canon/context-router.md#section:overview semantic retrieval ended with retrieval state selected with hybrid outcome expanded"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "selected_evidence: src/context_router.rs [workspace_file] origin=semantic_expand semantic_score=0.944 semantic similarity expanded the V1 candidate set with bounded local evidence"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "rejected_candidate: src/semantic.rs [workspace_file] origin=semantic_expand semantic_score=0.812 semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
        ),
        "{rendered}"
    );
}

#[test]
fn trace_summary_contract_surfaces_recovery_guidance_for_corrupt_index() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/semantic-corrupt-trace.json".to_string(),
        goal: "Recover guidance for a corrupt derived index".to_string(),
        advanced_context: Some(AdvancedContextProjection {
            query_id: "query-semantic-corrupt".to_string(),
            retrieval_mode: RetrievalMode::Local,
            retrieval_state: RetrievalState::Degraded,
            retrieval_index_state: RetrievalIndexState::Corrupt,
            semantic_policy_state: SemanticPolicyState::Local,
            semantic_capability_state: SemanticCapabilityState::Corrupt,
            hybrid_outcome: HybridOutcome::Fallback,
            budgets: Default::default(),
            remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
            used_remote: false,
            terminal_reason: Some("derived index is corrupt; using bounded fallback".to_string()),
            selected_evidence: Vec::new(),
            rejected_candidates: Vec::new(),
            semantic_trace_records: Vec::new(),
            relationships: Vec::new(),
            impact_findings: Vec::new(),
        }),
        ..TraceSummaryView::default()
    };

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    assert!(
        rendered.contains(
            "retrieval_recovery_guidance: run boundline index doctor to inspect vector capability, hooks, and tracked-file hygiene"
        ),
        "{rendered}"
    );
}
