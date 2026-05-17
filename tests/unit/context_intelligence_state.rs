use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ContextIntelligenceError,
    HybridOutcome, ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity,
    ImpactFindingStatus, RelationshipCredibilityState, RelationshipKind, RelationshipProjection,
    RemoteTransmissionPolicyState, RetrievalBudgets, RetrievalCompatibilityState,
    RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState,
    SemanticChunkState, SemanticPolicyState, VectorExtensionState,
};

/// Builds one selected evidence candidate for projection validation tests.
fn selected_candidate() -> RetrievedEvidenceCandidate {
    RetrievedEvidenceCandidate {
        candidate_id: "candidate-1".to_string(),
        source_kind: RetrievalSourceKind::WorkspaceFile,
        source_ref: "src/lib.rs".to_string(),
        authority_rank: AuthorityRank::Structured,
        match_origin: RetrievalMatchOrigin::Fts,
        selection_state: CandidateSelectionState::Selected,
        selection_reason: "goal keyword matched the implementation surface".to_string(),
        provenance_summary: "workspace file selected through local retrieval".to_string(),
        compatibility_state: RetrievalCompatibilityState::Compatible,
        staleness_state: RetrievalStalenessState::Fresh,
        lexical_score: None,
        semantic_score: None,
        canon_semantic_contract_line: None,
        canon_semantic_provenance_ref: None,
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
        semantic_policy_state: SemanticPolicyState::Disabled,
        semantic_capability_state: SemanticCapabilityState::Unsupported,
        hybrid_outcome: HybridOutcome::BaselineOnly,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: vec![selected_candidate()],
        rejected_candidates: Vec::new(),
        semantic_trace_records: Vec::new(),
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
        semantic_policy_state: SemanticPolicyState::Disabled,
        semantic_capability_state: SemanticCapabilityState::Unsupported,
        hybrid_outcome: HybridOutcome::BaselineOnly,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::Blocked,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: Vec::new(),
        rejected_candidates: Vec::new(),
        semantic_trace_records: Vec::new(),
        relationships: Vec::new(),
        impact_findings: Vec::new(),
    };

    let error = projection.validate().unwrap_err();
    assert_eq!(
        error.to_string(),
        "advanced context projection requires a terminal reason for non-selected states"
    );
}

#[test]
fn context_intelligence_variants_expose_stable_labels() {
    for (mode, label) in [
        (RetrievalMode::Disabled, "disabled"),
        (RetrievalMode::Local, "local"),
        (RetrievalMode::Remote, "remote"),
    ] {
        assert_eq!(mode.as_str(), label);
    }

    for (state, label, selected) in [
        (RetrievalState::Selected, "selected", true),
        (RetrievalState::Degraded, "degraded", false),
        (RetrievalState::Insufficient, "insufficient", false),
        (RetrievalState::Exhausted, "exhausted", false),
        (RetrievalState::Unavailable, "unavailable", false),
    ] {
        assert_eq!(state.as_str(), label);
        assert_eq!(state.is_selected(), selected);
    }

    for (state, label) in [
        (RetrievalIndexState::Ready, "ready"),
        (RetrievalIndexState::Stale, "stale"),
        (RetrievalIndexState::Building, "building"),
        (RetrievalIndexState::Insufficient, "insufficient"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in
        [(SemanticPolicyState::Disabled, "disabled"), (SemanticPolicyState::Local, "local")]
    {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in [
        (SemanticCapabilityState::Ready, "ready"),
        (SemanticCapabilityState::Unavailable, "unavailable"),
        (SemanticCapabilityState::Unsupported, "unsupported"),
        (SemanticCapabilityState::Degraded, "degraded"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (outcome, label) in [
        (HybridOutcome::BaselineOnly, "baseline_only"),
        (HybridOutcome::Expanded, "expanded"),
        (HybridOutcome::Reranked, "reranked"),
        (HybridOutcome::Skipped, "skipped"),
        (HybridOutcome::Fallback, "fallback"),
    ] {
        assert_eq!(outcome.as_str(), label);
    }

    for (state, label) in [
        (SemanticChunkState::Pending, "pending"),
        (SemanticChunkState::Ready, "ready"),
        (SemanticChunkState::Stale, "stale"),
        (SemanticChunkState::Blocked, "blocked"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (origin, label) in [
        (RetrievalMatchOrigin::Fts, "fts"),
        (RetrievalMatchOrigin::SemanticExpand, "semantic_expand"),
        (RetrievalMatchOrigin::SemanticRerank, "semantic_rerank"),
        (RetrievalMatchOrigin::StructuredFallback, "structured_fallback"),
    ] {
        assert_eq!(origin.as_str(), label);
    }
    for (state, label) in [
        (VectorExtensionState::Ready, "ready"),
        (VectorExtensionState::Missing, "missing"),
        (VectorExtensionState::Unsupported, "unsupported"),
        (VectorExtensionState::Stale, "stale"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (rank, label) in [
        (AuthorityRank::Structured, "structured"),
        (AuthorityRank::Canon, "canon"),
        (AuthorityRank::WorkspaceOverride, "workspace_override"),
        (AuthorityRank::Semantic, "semantic"),
    ] {
        assert_eq!(rank.as_str(), label);
    }

    for (kind, label) in [
        (RetrievalSourceKind::WorkspaceFile, "workspace_file"),
        (RetrievalSourceKind::ProjectMemory, "project_memory"),
        (RetrievalSourceKind::Trace, "trace"),
        (RetrievalSourceKind::ReviewFinding, "review_finding"),
        (RetrievalSourceKind::VerificationEvidence, "verification_evidence"),
        (RetrievalSourceKind::CanonArtifact, "canon_artifact"),
    ] {
        assert_eq!(kind.as_str(), label);
    }

    for (state, requires_reason) in [
        (CandidateSelectionState::Discovered, false),
        (CandidateSelectionState::Selected, true),
        (CandidateSelectionState::Downgraded, true),
        (CandidateSelectionState::Rejected, true),
        (CandidateSelectionState::Expired, false),
    ] {
        assert_eq!(state.requires_reason(), requires_reason);
    }

    for (state, label) in [
        (RetrievalCompatibilityState::Compatible, "compatible"),
        (RetrievalCompatibilityState::UnsupportedContract, "unsupported_contract"),
        (RetrievalCompatibilityState::MissingMetadata, "missing_metadata"),
        (RetrievalCompatibilityState::PolicyBlocked, "policy_blocked"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in
        [(RetrievalStalenessState::Fresh, "fresh"), (RetrievalStalenessState::Stale, "stale")]
    {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in [
        (RemoteTransmissionPolicyState::Blocked, "blocked"),
        (RemoteTransmissionPolicyState::LocalOnly, "local_only"),
        (RemoteTransmissionPolicyState::RemoteAllowed, "remote_allowed"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (kind, label) in [
        (RelationshipKind::AffectsSystem, "affects_system"),
        (RelationshipKind::AffectsDomain, "affects_domain"),
        (RelationshipKind::ExercisesTest, "exercises_test"),
        (RelationshipKind::ExposesContract, "exposes_contract"),
        (RelationshipKind::SuggestsReviewer, "suggests_reviewer"),
        (RelationshipKind::SupportsRisk, "supports_risk"),
        (RelationshipKind::RequiresEvidence, "requires_evidence"),
    ] {
        assert_eq!(kind.as_str(), label);
    }

    for (state, label) in [
        (RelationshipCredibilityState::Credible, "credible"),
        (RelationshipCredibilityState::Tentative, "tentative"),
        (RelationshipCredibilityState::Insufficient, "insufficient"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (kind, label) in [
        (ImpactFindingKind::AffectedSystem, "affected_system"),
        (ImpactFindingKind::AffectedDomain, "affected_domain"),
        (ImpactFindingKind::MissingTest, "missing_test"),
        (ImpactFindingKind::ContractExposure, "contract_exposure"),
        (ImpactFindingKind::ReviewerGap, "reviewer_gap"),
        (ImpactFindingKind::EvidenceGap, "evidence_gap"),
    ] {
        assert_eq!(kind.as_str(), label);
    }

    for (status, label) in [
        (ImpactFindingStatus::Open, "open"),
        (ImpactFindingStatus::Acknowledged, "acknowledged"),
        (ImpactFindingStatus::Resolved, "resolved"),
        (ImpactFindingStatus::Invalidated, "invalidated"),
    ] {
        assert_eq!(status.as_str(), label);
    }

    for (severity, label) in [
        (ImpactFindingSeverity::Low, "low"),
        (ImpactFindingSeverity::Medium, "medium"),
        (ImpactFindingSeverity::High, "high"),
    ] {
        assert_eq!(severity.as_str(), label);
    }
}

#[test]
fn retrieved_candidate_requires_semantic_score_for_semantic_match_origins() {
    let mut candidate = selected_candidate();
    candidate.match_origin = RetrievalMatchOrigin::SemanticExpand;

    assert_eq!(
        candidate.validate().unwrap_err(),
        ContextIntelligenceError::MissingSemanticScore {
            candidate_id: "candidate-1".to_string(),
            match_origin: RetrievalMatchOrigin::SemanticExpand,
        }
    );
}

#[test]
fn retrieved_candidate_rejects_partial_canon_semantic_metadata() {
    let mut candidate = selected_candidate();
    candidate.canon_semantic_contract_line = Some("v1".to_string());

    assert_eq!(
        candidate.validate().unwrap_err(),
        ContextIntelligenceError::IncompleteCanonSemanticMetadata {
            candidate_id: "candidate-1".to_string(),
        }
    );
}

#[test]
fn retrieval_models_reject_missing_required_fields() {
    for (label, budgets) in [
        (
            "refinement_budget",
            RetrievalBudgets { refinement_budget: 0, ..RetrievalBudgets::default() },
        ),
        ("refresh_budget", RetrievalBudgets { refresh_budget: 0, ..RetrievalBudgets::default() }),
        ("depth_limit", RetrievalBudgets { depth_limit: 0, ..RetrievalBudgets::default() }),
        ("expansion_limit", RetrievalBudgets { expansion_limit: 0, ..RetrievalBudgets::default() }),
        ("traversal_limit", RetrievalBudgets { traversal_limit: 0, ..RetrievalBudgets::default() }),
        ("evidence_limit", RetrievalBudgets { evidence_limit: 0, ..RetrievalBudgets::default() }),
    ] {
        assert_eq!(
            budgets.validate().unwrap_err(),
            ContextIntelligenceError::InvalidBudget(label.to_string())
        );
    }

    let mut candidate = selected_candidate();
    candidate.candidate_id = " ".to_string();
    assert_eq!(candidate.validate().unwrap_err(), ContextIntelligenceError::MissingCandidateId);

    let mut candidate = selected_candidate();
    candidate.source_ref = " ".to_string();
    assert_eq!(
        candidate.validate().unwrap_err(),
        ContextIntelligenceError::MissingSourceRef { candidate_id: "candidate-1".to_string() }
    );

    let mut candidate = selected_candidate();
    candidate.selection_reason = " ".to_string();
    assert_eq!(
        candidate.validate().unwrap_err(),
        ContextIntelligenceError::MissingSelectionReason {
            candidate_id: "candidate-1".to_string(),
        }
    );

    let mut candidate = selected_candidate();
    candidate.provenance_summary = " ".to_string();
    assert_eq!(
        candidate.validate().unwrap_err(),
        ContextIntelligenceError::MissingProvenanceSummary {
            candidate_id: "candidate-1".to_string(),
        }
    );

    let mut relationship = projected_relationship();
    relationship.relationship_id = " ".to_string();
    assert_eq!(
        relationship.validate().unwrap_err(),
        ContextIntelligenceError::MissingRelationshipId
    );

    let mut relationship = projected_relationship();
    relationship.subject_ref = " ".to_string();
    assert_eq!(
        relationship.validate().unwrap_err(),
        ContextIntelligenceError::MissingRelationshipSubject {
            relationship_id: "relationship-1".to_string(),
        }
    );

    let mut relationship = projected_relationship();
    relationship.explanation = " ".to_string();
    assert_eq!(
        relationship.validate().unwrap_err(),
        ContextIntelligenceError::MissingRelationshipExplanation {
            relationship_id: "relationship-1".to_string(),
        }
    );

    let mut relationship = projected_relationship();
    relationship.supporting_candidate_ids.clear();
    assert_eq!(
        relationship.validate().unwrap_err(),
        ContextIntelligenceError::MissingRelationshipSupport {
            relationship_id: "relationship-1".to_string(),
        }
    );

    let mut finding = projected_finding();
    finding.finding_id = " ".to_string();
    assert_eq!(finding.validate().unwrap_err(), ContextIntelligenceError::MissingFindingId);

    let mut finding = projected_finding();
    finding.subject_ref = " ".to_string();
    assert_eq!(
        finding.validate().unwrap_err(),
        ContextIntelligenceError::MissingFindingSubject { finding_id: "finding-1".to_string() }
    );

    let mut finding = projected_finding();
    finding.recommended_follow_up = " ".to_string();
    assert_eq!(
        finding.validate().unwrap_err(),
        ContextIntelligenceError::MissingFindingFollowUp { finding_id: "finding-1".to_string() }
    );

    let mut finding = projected_finding();
    finding.supporting_relationship_ids.clear();
    assert_eq!(
        finding.validate().unwrap_err(),
        ContextIntelligenceError::MissingFindingSupport { finding_id: "finding-1".to_string() }
    );
}

#[test]
fn advanced_context_projection_rejects_invalid_runtime_state() {
    let projection = AdvancedContextProjection {
        query_id: "query-3".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        semantic_policy_state: SemanticPolicyState::Disabled,
        semantic_capability_state: SemanticCapabilityState::Unsupported,
        hybrid_outcome: HybridOutcome::BaselineOnly,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: vec![selected_candidate()],
        rejected_candidates: Vec::new(),
        semantic_trace_records: Vec::new(),
        relationships: vec![projected_relationship()],
        impact_findings: vec![projected_finding()],
    };

    let mut missing_selected_evidence = projection.clone();
    missing_selected_evidence.selected_evidence.clear();
    assert_eq!(
        missing_selected_evidence.validate().unwrap_err(),
        ContextIntelligenceError::MissingSelectedEvidence
    );

    let mut unexpected_remote_usage = projection.clone();
    unexpected_remote_usage.used_remote = true;
    assert_eq!(
        unexpected_remote_usage.validate().unwrap_err(),
        ContextIntelligenceError::UnexpectedRemoteUsage
    );

    let mut blocked_remote_usage = projection.clone();
    blocked_remote_usage.retrieval_mode = RetrievalMode::Remote;
    blocked_remote_usage.remote_policy_state = RemoteTransmissionPolicyState::Blocked;
    blocked_remote_usage.used_remote = true;
    assert_eq!(
        blocked_remote_usage.validate().unwrap_err(),
        ContextIntelligenceError::BlockedRemoteUsage
    );

    let mut disabled_policy_with_expansion = projection.clone();
    disabled_policy_with_expansion.hybrid_outcome = HybridOutcome::Expanded;
    assert_eq!(
        disabled_policy_with_expansion.validate().unwrap_err(),
        ContextIntelligenceError::InvalidSemanticHybridOutcome {
            policy_state: SemanticPolicyState::Disabled,
            hybrid_outcome: HybridOutcome::Expanded,
        }
    );

    let mut missing_semantic_terminal_reason = projection.clone();
    missing_semantic_terminal_reason.hybrid_outcome = HybridOutcome::Skipped;
    missing_semantic_terminal_reason.selected_evidence.clear();
    assert_eq!(
        missing_semantic_terminal_reason.validate().unwrap_err(),
        ContextIntelligenceError::MissingSemanticTerminalReason {
            hybrid_outcome: HybridOutcome::Skipped,
        }
    );

    let mut invalid_nested_candidate = projection;
    invalid_nested_candidate.rejected_candidates = vec![RetrievedEvidenceCandidate {
        selection_reason: " ".to_string(),
        ..selected_candidate()
    }];
    assert_eq!(
        invalid_nested_candidate.validate().unwrap_err(),
        ContextIntelligenceError::MissingSelectionReason {
            candidate_id: "candidate-1".to_string(),
        }
    );
}
