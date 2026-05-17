use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ImpactAnalysisFinding,
    ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus, RelationshipCredibilityState,
    RelationshipKind, RelationshipProjection, RemoteTransmissionPolicyState,
    RetrievalCompatibilityState, RetrievalIndexState, RetrievalMode, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate,
};

/// Builds one selected evidence candidate for projection validation tests.
fn selected_candidate() -> RetrievedEvidenceCandidate {
    RetrievedEvidenceCandidate {
        candidate_id: "candidate-1".to_string(),
        source_kind: RetrievalSourceKind::WorkspaceFile,
        source_ref: "src/lib.rs".to_string(),
        authority_rank: AuthorityRank::Structured,
        selection_state: CandidateSelectionState::Selected,
        selection_reason: "goal keyword matched the implementation surface".to_string(),
        provenance_summary: "workspace file selected through local retrieval".to_string(),
        compatibility_state: RetrievalCompatibilityState::Compatible,
        staleness_state: RetrievalStalenessState::Fresh,
    }
}

/// Builds one supported relationship for projection validation tests.
fn projected_relationship() -> RelationshipProjection {
    RelationshipProjection {
        relationship_id: "relationship-1".to_string(),
        subject_ref: "src/lib.rs".to_string(),
        relationship_kind: RelationshipKind::ExercisesTest,
        credibility_state: RelationshipCredibilityState::Credible,
        explanation: "the matching test file names the same target".to_string(),
        supporting_candidate_ids: vec!["candidate-1".to_string()],
    }
}

/// Builds one impact finding for projection validation tests.
fn projected_finding() -> ImpactAnalysisFinding {
    ImpactAnalysisFinding {
        finding_id: "finding-1".to_string(),
        finding_kind: ImpactFindingKind::MissingTest,
        subject_ref: "tests/lib.rs".to_string(),
        status: ImpactFindingStatus::Open,
        severity: ImpactFindingSeverity::Medium,
        recommended_follow_up: "add or refresh the focused regression test".to_string(),
        supporting_relationship_ids: vec!["relationship-1".to_string()],
    }
}

#[test]
fn advanced_context_projection_accepts_selected_local_evidence() {
    let projection = AdvancedContextProjection {
        query_id: "query-1".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: vec![selected_candidate()],
        rejected_candidates: Vec::new(),
        relationships: vec![projected_relationship()],
        impact_findings: vec![projected_finding()],
    };

    assert_eq!(projection.authority_order_text(), "structured>canon>workspace_override>semantic");
    assert_eq!(projection.selected_evidence_count(), 1);
    assert_eq!(projection.impact_finding_count(), 1);
    assert!(projection.validate().is_ok());
}

#[test]
fn advanced_context_projection_requires_terminal_reason_for_non_selected_state() {
    let projection = AdvancedContextProjection {
        query_id: "query-2".to_string(),
        retrieval_mode: RetrievalMode::Disabled,
        retrieval_state: RetrievalState::Degraded,
        retrieval_index_state: RetrievalIndexState::Insufficient,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::Blocked,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: Vec::new(),
        rejected_candidates: Vec::new(),
        relationships: Vec::new(),
        impact_findings: Vec::new(),
    };

    let error = projection.validate().unwrap_err();
    assert_eq!(
        error.to_string(),
        "advanced context projection requires a terminal reason for non-selected states"
    );
}
