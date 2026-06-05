use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ContextIntelligenceError,
    DerivedIndexManifest, HybridOutcome, ImpactAnalysisFinding, ImpactFindingKind,
    ImpactFindingSeverity, ImpactFindingStatus, IndexDoctorCheck, IndexDoctorConsistencyState,
    IndexDoctorReport, IndexDoctorStatus, IndexMaintenanceCommand, IndexMaintenanceOperation,
    IndexMaintenanceTrigger, IndexRefreshReason, IndexStaleReason, ManifestFtsState,
    RelationshipCredibilityState, RelationshipKind, RelationshipProjection,
    RemoteTransmissionPolicyState, RetrievalBudgets, RetrievalCompatibilityState,
    RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode, RetrievalScore, RetrievalSourceKind,
    RetrievalStalenessState, RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState,
    SemanticChunkRecord, SemanticChunkState, SemanticEngine, SemanticPolicyState,
    SemanticVectorRecord, SemanticVectorState, SourceDigestCompatibilityState, SourceDigestRecord,
    SourcePresenceState, VectorExtensionState,
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

fn derived_index_manifest() -> DerivedIndexManifest {
    DerivedIndexManifest {
        schema_version: "retrieval-index-v3".to_string(),
        workspace_root: "workspace".to_string(),
        git_branch: Some("main".to_string()),
        git_head: Some("abc123".to_string()),
        last_seen_head: Some("abc123".to_string()),
        index_status: RetrievalIndexState::Ready,
        last_refresh_at: Some("2026-05-30 12:00:00".to_string()),
        last_refresh_reason: Some(IndexRefreshReason::ManualRefresh),
        stale_reason: None,
        file_count: 2,
        chunk_count: 2,
        fts5_state: ManifestFtsState::Ready,
        sqlite_vec_state: VectorExtensionState::Ready,
        semantic_engine: SemanticEngine::SqliteVec,
        workspace_fingerprint: "fnv64:0000000000000001".to_string(),
        config_fingerprint: "fnv64:0000000000000002".to_string(),
        chunker_fingerprint: "fnv64:0000000000000003".to_string(),
        embedding_model_fingerprint: "fnv64:0000000000000004".to_string(),
    }
}

fn source_digest_record() -> SourceDigestRecord {
    SourceDigestRecord {
        source_ref: "src/lib.rs".to_string(),
        source_kind: RetrievalSourceKind::WorkspaceFile,
        content_hash: "fnv64:feedbeef".to_string(),
        compatibility_state: SourceDigestCompatibilityState::Compatible,
        authority_rank: AuthorityRank::Structured,
        last_indexed_at: Some("2026-05-30 12:00:00".to_string()),
        chunk_count: 1,
        source_presence_state: SourcePresenceState::Present,
    }
}

fn semantic_chunk_record() -> SemanticChunkRecord {
    SemanticChunkRecord {
        chunk_id: "semantic:src/lib.rs:0".to_string(),
        source_ref: "src/lib.rs".to_string(),
        chunk_ordinal: 0,
        chunk_range: "1-4".to_string(),
        provenance_boundary: "workspace_file".to_string(),
        provenance_ref: "src/lib.rs#L1".to_string(),
        content_hash: "fnv64:chunkhash".to_string(),
        chunk_state: SemanticChunkState::Ready,
        embedding_dimensions: 48,
        canon_semantic_contract_line: None,
        semantic_labels: vec!["workspace_file".to_string(), "rs".to_string()],
    }
}

fn semantic_vector_record() -> SemanticVectorRecord {
    SemanticVectorRecord {
        chunk_id: "semantic:src/lib.rs:0".to_string(),
        vector_schema_line: "boundline.semantic_chunk.v1".to_string(),
        embedding_dimensions: 48,
        write_generation: 1,
        vector_state: SemanticVectorState::Ready,
    }
}

fn index_maintenance_operation() -> IndexMaintenanceOperation {
    IndexMaintenanceOperation {
        operation_id: "operation-1".to_string(),
        command_name: IndexMaintenanceCommand::Refresh,
        trigger: IndexMaintenanceTrigger::Manual,
        pre_state: RetrievalIndexState::Ready,
        post_state: RetrievalIndexState::Ready,
        sources_scanned: 2,
        chunks_upserted: 2,
        chunks_deleted: 0,
        vector_rows_written: 2,
        fallback_reason: None,
        recommended_action: Some("none".to_string()),
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
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
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
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
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
        (RetrievalIndexState::Missing, "missing"),
        (RetrievalIndexState::Incompatible, "incompatible"),
        (RetrievalIndexState::Degraded, "degraded"),
        (RetrievalIndexState::Corrupt, "corrupt"),
        (RetrievalIndexState::SemanticUnavailable, "semantic_unavailable"),
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
        (SemanticCapabilityState::Corrupt, "corrupt"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (engine, label) in [
        (SemanticEngine::Disabled, "disabled"),
        (SemanticEngine::BaselineJson, "baseline_json"),
        (SemanticEngine::SqliteVec, "sqlite_vec"),
    ] {
        assert_eq!(engine.as_str(), label);
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
        (SemanticChunkState::Deleted, "deleted"),
        (SemanticChunkState::MissingVector, "missing_vector"),
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
        (VectorExtensionState::Degraded, "degraded"),
        (VectorExtensionState::Corrupt, "corrupt"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in [
        (ManifestFtsState::Ready, "ready"),
        (ManifestFtsState::Missing, "missing"),
        (ManifestFtsState::Corrupt, "corrupt"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (reason, label) in [
        (IndexRefreshReason::ManualRefresh, "manual_refresh"),
        (IndexRefreshReason::Rebuild, "rebuild"),
        (IndexRefreshReason::SchemaChange, "schema_change"),
        (IndexRefreshReason::BranchChange, "branch_change"),
        (IndexRefreshReason::ConfigChange, "config_change"),
        (IndexRefreshReason::ChunkerChange, "chunker_change"),
        (IndexRefreshReason::CapabilityChange, "capability_change"),
        (IndexRefreshReason::DoctorRepair, "doctor_repair"),
    ] {
        assert_eq!(reason.as_str(), label);
    }

    for (reason, label) in [
        (IndexStaleReason::GitHeadChanged, "git_head_changed"),
        (IndexStaleReason::BranchCheckout, "branch_checkout"),
        (IndexStaleReason::Merge, "merge"),
        (IndexStaleReason::PullWithMerge, "pull_with_merge"),
        (IndexStaleReason::Rebase, "rebase"),
        (IndexStaleReason::PostRewrite, "post_rewrite"),
        (IndexStaleReason::HookMarkedStale, "hook_marked_stale"),
    ] {
        assert_eq!(reason.as_str(), label);
    }

    for (state, label) in [
        (SourcePresenceState::Present, "present"),
        (SourcePresenceState::Deleted, "deleted"),
        (SourcePresenceState::Skipped, "skipped"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in [
        (SourceDigestCompatibilityState::Compatible, "compatible"),
        (SourceDigestCompatibilityState::Excluded, "excluded"),
        (SourceDigestCompatibilityState::Unsupported, "unsupported"),
        (SourceDigestCompatibilityState::Blocked, "blocked"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (state, label) in [
        (SemanticVectorState::Ready, "ready"),
        (SemanticVectorState::Missing, "missing"),
        (SemanticVectorState::Stale, "stale"),
        (SemanticVectorState::DimensionMismatch, "dimension_mismatch"),
        (SemanticVectorState::Corrupt, "corrupt"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    for (command, label) in [
        (IndexMaintenanceCommand::Status, "status"),
        (IndexMaintenanceCommand::Refresh, "refresh"),
        (IndexMaintenanceCommand::Rebuild, "rebuild"),
        (IndexMaintenanceCommand::Clean, "clean"),
        (IndexMaintenanceCommand::Doctor, "doctor"),
    ] {
        assert_eq!(command.as_str(), label);
    }

    for (trigger, label) in [
        (IndexMaintenanceTrigger::Manual, "manual"),
        (IndexMaintenanceTrigger::PostCheckout, "post_checkout"),
        (IndexMaintenanceTrigger::PostMerge, "post_merge"),
        (IndexMaintenanceTrigger::PostRewrite, "post_rewrite"),
        (IndexMaintenanceTrigger::SchemaChange, "schema_change"),
        (IndexMaintenanceTrigger::ConfigChange, "config_change"),
        (IndexMaintenanceTrigger::CapabilityChange, "capability_change"),
    ] {
        assert_eq!(trigger.as_str(), label);
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
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
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

#[test]
fn derived_index_models_validate_new_lifecycle_invariants() {
    let manifest = derived_index_manifest();
    assert!(manifest.validate().is_ok());

    let mut invalid_fts_manifest = manifest.clone();
    invalid_fts_manifest.fts5_state = ManifestFtsState::Missing;
    assert_eq!(
        invalid_fts_manifest.validate().unwrap_err(),
        ContextIntelligenceError::InvalidReadyManifestFtsState {
            fts5_state: ManifestFtsState::Missing,
        }
    );

    let mut invalid_vector_manifest = manifest;
    invalid_vector_manifest.sqlite_vec_state = VectorExtensionState::Missing;
    assert_eq!(
        invalid_vector_manifest.validate().unwrap_err(),
        ContextIntelligenceError::InvalidSqliteVecManifestState {
            sqlite_vec_state: VectorExtensionState::Missing,
        }
    );

    let mut deleted_source = source_digest_record();
    deleted_source.source_presence_state = SourcePresenceState::Deleted;
    assert_eq!(
        deleted_source.validate().unwrap_err(),
        ContextIntelligenceError::InvalidDeletedSourceChunkCount {
            source_ref: "src/lib.rs".to_string(),
            chunk_count: 1,
        }
    );

    let mut invalid_chunk = semantic_chunk_record();
    invalid_chunk.embedding_dimensions = 0;
    assert_eq!(
        invalid_chunk.validate().unwrap_err(),
        ContextIntelligenceError::InvalidSemanticChunkDimensions {
            chunk_id: "semantic:src/lib.rs:0".to_string(),
        }
    );

    let mut invalid_vector = semantic_vector_record();
    invalid_vector.embedding_dimensions = 0;
    assert_eq!(
        invalid_vector.validate().unwrap_err(),
        ContextIntelligenceError::InvalidSemanticVectorDimensions {
            chunk_id: "semantic:src/lib.rs:0".to_string(),
        }
    );

    let mut missing_action = index_maintenance_operation();
    missing_action.post_state = RetrievalIndexState::Degraded;
    missing_action.recommended_action = None;
    assert_eq!(
        missing_action.validate().unwrap_err(),
        ContextIntelligenceError::MissingIndexOperationRecommendedAction {
            operation_id: "operation-1".to_string(),
        }
    );

    let mut invalid_status = index_maintenance_operation();
    invalid_status.command_name = IndexMaintenanceCommand::Status;
    invalid_status.pre_state = RetrievalIndexState::Missing;
    invalid_status.post_state = RetrievalIndexState::Ready;
    assert_eq!(
        invalid_status.validate().unwrap_err(),
        ContextIntelligenceError::InvalidStatusOperationStateTransition
    );

    let doctor_report = IndexDoctorReport {
        status: IndexDoctorStatus::Advisory,
        checks: vec![IndexDoctorCheck {
            check_name: "manifest_consistency".to_string(),
            result: IndexDoctorStatus::Advisory,
            detail: "derived index manifest is missing".to_string(),
            suggested_fix: "boundline index refresh --workspace workspace".to_string(),
        }],
        tracked_index_files: Vec::new(),
        missing_ignore_rules: vec![
            ".boundline/context-intelligence/retrieval-index.sqlite3".to_string(),
        ],
        wal_sidecars_present: false,
        manifest_consistency: IndexDoctorConsistencyState::Missing,
        vector_schema_consistency: IndexDoctorConsistencyState::Consistent,
    };
    assert!(doctor_report.validate().is_ok());

    let mut invalid_doctor_check = doctor_report.clone();
    invalid_doctor_check.checks[0].suggested_fix = " ".to_string();
    assert_eq!(
        invalid_doctor_check.validate().unwrap_err(),
        ContextIntelligenceError::MissingIndexDoctorCheckSuggestedFix {
            check_name: "manifest_consistency".to_string(),
        }
    );
}

#[test]
fn derived_index_manifest_detects_stale_heads_and_rebuild_triggers() {
    let manifest = derived_index_manifest();
    assert!(!manifest.head_is_stale());
    assert_eq!(manifest.effective_stale_reason(), None);

    let mut stale_manifest = manifest.clone();
    stale_manifest.last_seen_head = Some("def456".to_string());
    assert!(stale_manifest.head_is_stale());
    assert_eq!(stale_manifest.effective_stale_reason(), Some(IndexStaleReason::GitHeadChanged));

    let mut rebuild_manifest = manifest.clone();
    rebuild_manifest.chunker_fingerprint = "fnv64:0000000000009999".to_string();
    assert!(manifest.requires_rebuild_against(&rebuild_manifest));

    let mut compatible_manifest = manifest.clone();
    compatible_manifest.git_head = Some("def456".to_string());
    compatible_manifest.last_seen_head = Some("def456".to_string());
    assert!(!manifest.requires_rebuild_against(&compatible_manifest));
}

#[test]
fn semantic_chunk_record_uses_stable_chunk_ids() {
    assert_eq!(SemanticChunkRecord::stable_chunk_id("src/lib.rs", 0), "semantic:src/lib.rs:0");

    let mut invalid_chunk = semantic_chunk_record();
    invalid_chunk.chunk_id = "semantic:src/lib.rs".to_string();
    assert_eq!(
        invalid_chunk.validate().unwrap_err(),
        ContextIntelligenceError::InvalidSemanticChunkId {
            chunk_id: "semantic:src/lib.rs".to_string(),
            expected_chunk_id: "semantic:src/lib.rs:0".to_string(),
        }
    );
}

#[test]
fn derived_index_manifest_rejects_missing_required_fields_and_reports_hook_marked_stale() {
    for (manifest, expected_error) in [
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.schema_version = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestSchemaVersion,
        ),
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.workspace_root = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestWorkspaceRoot,
        ),
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.workspace_fingerprint = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestWorkspaceFingerprint,
        ),
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.config_fingerprint = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestConfigFingerprint,
        ),
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.chunker_fingerprint = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestChunkerFingerprint,
        ),
        (
            {
                let mut manifest = derived_index_manifest();
                manifest.embedding_model_fingerprint = " ".to_string();
                manifest
            },
            ContextIntelligenceError::MissingManifestEmbeddingFingerprint,
        ),
    ] {
        assert_eq!(manifest.validate().unwrap_err(), expected_error);
    }

    let mut stale_manifest = derived_index_manifest();
    stale_manifest.index_status = RetrievalIndexState::Stale;
    stale_manifest.git_head = None;
    stale_manifest.last_seen_head = None;
    assert_eq!(stale_manifest.effective_stale_reason(), Some(IndexStaleReason::HookMarkedStale));
}

#[test]
fn source_digest_semantic_chunk_and_vector_records_reject_missing_fields() {
    for (record, expected_error) in [
        (
            {
                let mut record = source_digest_record();
                record.source_ref = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSourceDigestRef,
        ),
        (
            {
                let mut record = source_digest_record();
                record.content_hash = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSourceDigestHash {
                source_ref: "src/lib.rs".to_string(),
            },
        ),
    ] {
        assert_eq!(record.validate().unwrap_err(), expected_error);
    }

    for (record, expected_error) in [
        (
            {
                let mut record = semantic_chunk_record();
                record.chunk_id = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkId,
        ),
        (
            {
                let mut record = semantic_chunk_record();
                record.source_ref = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkSourceRef {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
        (
            {
                let mut record = semantic_chunk_record();
                record.chunk_range = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkRange {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
        (
            {
                let mut record = semantic_chunk_record();
                record.provenance_boundary = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkBoundary {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
        (
            {
                let mut record = semantic_chunk_record();
                record.provenance_ref = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkProvenanceRef {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
        (
            {
                let mut record = semantic_chunk_record();
                record.content_hash = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticChunkHash {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
    ] {
        assert_eq!(record.validate().unwrap_err(), expected_error);
    }

    for (record, expected_error) in [
        (
            {
                let mut record = semantic_vector_record();
                record.chunk_id = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticVectorChunkId,
        ),
        (
            {
                let mut record = semantic_vector_record();
                record.vector_schema_line = " ".to_string();
                record
            },
            ContextIntelligenceError::MissingSemanticVectorSchemaLine {
                chunk_id: "semantic:src/lib.rs:0".to_string(),
            },
        ),
    ] {
        assert_eq!(record.validate().unwrap_err(), expected_error);
    }
}

#[test]
fn index_doctor_and_lifecycle_reports_validate_new_error_paths() {
    for (status, label) in [
        (IndexDoctorStatus::Passed, "passed"),
        (IndexDoctorStatus::Advisory, "advisory"),
        (IndexDoctorStatus::Failed, "failed"),
    ] {
        assert_eq!(status.as_str(), label);
    }

    for (state, label) in [
        (IndexDoctorConsistencyState::Consistent, "consistent"),
        (IndexDoctorConsistencyState::Missing, "missing"),
        (IndexDoctorConsistencyState::Corrupt, "corrupt"),
        (IndexDoctorConsistencyState::Invalid, "invalid"),
    ] {
        assert_eq!(state.as_str(), label);
    }

    let mut operation = index_maintenance_operation();
    operation.operation_id = " ".to_string();
    assert_eq!(
        operation.validate().unwrap_err(),
        ContextIntelligenceError::MissingIndexOperationId
    );

    for (report, expected_error) in [
        (
            {
                let mut report = boundline::domain::context_intelligence::IndexLifecycleReport {
                    command: IndexMaintenanceCommand::Status,
                    workspace_root: "workspace".to_string(),
                    operation_id: "operation-1".to_string(),
                    pre_state: RetrievalIndexState::Ready,
                    post_state: RetrievalIndexState::Ready,
                    recommended_action: "none".to_string(),
                    stale_reason: None,
                    warnings: Vec::new(),
                    manifest: Some(derived_index_manifest()),
                };
                report.workspace_root = " ".to_string();
                report
            },
            ContextIntelligenceError::MissingIndexLifecycleWorkspaceRoot,
        ),
        (
            {
                let mut report = boundline::domain::context_intelligence::IndexLifecycleReport {
                    command: IndexMaintenanceCommand::Status,
                    workspace_root: "workspace".to_string(),
                    operation_id: "operation-1".to_string(),
                    pre_state: RetrievalIndexState::Ready,
                    post_state: RetrievalIndexState::Ready,
                    recommended_action: "none".to_string(),
                    stale_reason: None,
                    warnings: Vec::new(),
                    manifest: Some(derived_index_manifest()),
                };
                report.operation_id = " ".to_string();
                report
            },
            ContextIntelligenceError::MissingIndexLifecycleOperationId,
        ),
        (
            {
                let mut report = boundline::domain::context_intelligence::IndexLifecycleReport {
                    command: IndexMaintenanceCommand::Status,
                    workspace_root: "workspace".to_string(),
                    operation_id: "operation-1".to_string(),
                    pre_state: RetrievalIndexState::Ready,
                    post_state: RetrievalIndexState::Ready,
                    recommended_action: "none".to_string(),
                    stale_reason: None,
                    warnings: Vec::new(),
                    manifest: Some(derived_index_manifest()),
                };
                report.recommended_action = " ".to_string();
                report
            },
            ContextIntelligenceError::MissingIndexLifecycleRecommendedAction,
        ),
        (
            {
                let mut report = boundline::domain::context_intelligence::IndexLifecycleReport {
                    command: IndexMaintenanceCommand::Status,
                    workspace_root: "workspace".to_string(),
                    operation_id: "operation-1".to_string(),
                    pre_state: RetrievalIndexState::Ready,
                    post_state: RetrievalIndexState::Ready,
                    recommended_action: "none".to_string(),
                    stale_reason: None,
                    warnings: Vec::new(),
                    manifest: Some(derived_index_manifest()),
                };
                report.post_state = RetrievalIndexState::Stale;
                report
            },
            ContextIntelligenceError::MissingIndexLifecycleStaleReason,
        ),
    ] {
        assert_eq!(report.validate().unwrap_err(), expected_error);
    }

    let mut check = IndexDoctorCheck {
        check_name: "manifest_consistency".to_string(),
        result: IndexDoctorStatus::Advisory,
        detail: "missing manifest".to_string(),
        suggested_fix: "refresh".to_string(),
    };
    check.check_name = " ".to_string();
    assert_eq!(
        check.validate().unwrap_err(),
        ContextIntelligenceError::MissingIndexDoctorCheckName
    );

    let mut check = IndexDoctorCheck {
        check_name: "manifest_consistency".to_string(),
        result: IndexDoctorStatus::Advisory,
        detail: "missing manifest".to_string(),
        suggested_fix: "refresh".to_string(),
    };
    check.detail = " ".to_string();
    assert_eq!(
        check.validate().unwrap_err(),
        ContextIntelligenceError::MissingIndexDoctorCheckDetail {
            check_name: "manifest_consistency".to_string(),
        }
    );

    let report = IndexDoctorReport {
        status: IndexDoctorStatus::Failed,
        checks: Vec::new(),
        tracked_index_files: Vec::new(),
        missing_ignore_rules: Vec::new(),
        wal_sidecars_present: false,
        manifest_consistency: IndexDoctorConsistencyState::Missing,
        vector_schema_consistency: IndexDoctorConsistencyState::Missing,
    };
    assert_eq!(report.validate().unwrap_err(), ContextIntelligenceError::MissingIndexDoctorChecks);
}

#[test]
fn semantic_projection_serialization_covers_custom_candidate_and_projection_paths() {
    let mut candidate = selected_candidate();
    candidate.match_origin = RetrievalMatchOrigin::SemanticExpand;
    candidate.selection_state = CandidateSelectionState::Rejected;
    candidate.semantic_score = RetrievalScore::from_raw(0.91);
    candidate.canon_semantic_contract_line = Some("v1".to_string());
    candidate.canon_semantic_provenance_ref = Some(".canon/provenance.md#overview".to_string());

    let serialized_candidate = serde_json::to_value(&candidate).unwrap();
    assert_eq!(serialized_candidate["collapsed_from_chunk_count"], 1);
    assert_eq!(serialized_candidate["canon_semantic_contract_line"], "v1");

    let projection = AdvancedContextProjection {
        query_id: "query-semantic-fallback".to_string(),
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
        rejected_candidates: vec![candidate],
        semantic_trace_records: Vec::new(),
        relationships: Vec::new(),
        impact_findings: Vec::new(),
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
    };

    let serialized_projection = serde_json::to_value(&projection).unwrap();
    assert_eq!(serialized_projection["semantic_capability_state"], "corrupt");
    assert_eq!(serialized_projection["semantic_engine"], "baseline_json");
    assert_eq!(
        serialized_projection["semantic_fallback_reason"],
        "derived index is corrupt; using bounded fallback"
    );
}
