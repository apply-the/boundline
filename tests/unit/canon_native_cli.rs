use boundline::domain::configuration::CanonPreferences;
use boundline::domain::governance::{
    ApprovalState, CanonModeSelectionPreference, GovernanceLifecycleState, GovernedDocumentRef,
    GovernedSessionLifecycle, GovernedStageRecord, PacketReadiness,
};
use boundline::{CanonMode, GovernanceIntent, GovernanceRuntimeKind};

#[test]
fn canon_mode_selection_preference_serde_roundtrip() {
    let variants = [
        (CanonModeSelectionPreference::Manual, "\"manual\""),
        (CanonModeSelectionPreference::AutoConfirm, "\"auto-confirm\""),
        (CanonModeSelectionPreference::Auto, "\"auto\""),
    ];

    for (variant, expected_json) in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        assert_eq!(serialized, expected_json, "serialization mismatch for {variant:?}");
        let deserialized: CanonModeSelectionPreference = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant, "deserialization roundtrip mismatch for {variant:?}");
    }
}

#[test]
fn canon_mode_selection_preference_default_is_auto_confirm() {
    assert_eq!(CanonModeSelectionPreference::default(), CanonModeSelectionPreference::AutoConfirm);
}

#[test]
fn canon_preferences_toml_roundtrip() {
    let prefs = CanonPreferences {
        mode_selection: CanonModeSelectionPreference::Auto,
        default_risk: Some("high".to_string()),
        default_zone: Some("payments".to_string()),
        default_owner: Some("platform".to_string()),
        default_system_context: None,
    };

    let toml_str = toml::to_string_pretty(&prefs).unwrap();
    let deserialized: CanonPreferences = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized, prefs);
}

#[test]
fn canon_preferences_toml_defaults() {
    let toml_str = "";
    let deserialized: CanonPreferences = toml::from_str(toml_str).unwrap();
    assert_eq!(deserialized.mode_selection, CanonModeSelectionPreference::AutoConfirm);
    assert!(deserialized.default_risk.is_none());
    assert!(deserialized.default_zone.is_none());
    assert!(deserialized.default_owner.is_none());
}

#[test]
fn governed_session_lifecycle_json_roundtrip() {
    let lifecycle = GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: Some(CanonMode::Requirements),
        selected_mode_sequence: vec![CanonMode::Requirements, CanonMode::Architecture],
        current_stage_index: 1,
        stage_records: vec![GovernedStageRecord {
            stage_key: "delivery:requirements".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: Some("run-001".to_string()),
            governance_attempt_id: "attempt-001".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(".canon/runs/run-001".to_string()),
            decision_ref: None,
            blocked_reason: None,
        }],
        accumulated_context: vec![GovernedDocumentRef {
            stage_key: "delivery:requirements".to_string(),
            canon_mode: CanonMode::Requirements,
            packet_ref: ".canon/runs/run-001".to_string(),
            document_path: Some(".canon/runs/run-001/requirements.md".to_string()),
            readiness: PacketReadiness::Reusable,
        }],
        terminal_reason: None,
    };

    let json = serde_json::to_string_pretty(&lifecycle).unwrap();
    let deserialized: GovernedSessionLifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, lifecycle);
}

#[test]
fn governed_session_lifecycle_json_with_opt_out() {
    let lifecycle = GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Local,
        explicit_opt_out: true,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: None,
        selected_mode_sequence: Vec::new(),
        current_stage_index: 0,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
    };

    let json = serde_json::to_string(&lifecycle).unwrap();
    let deserialized: GovernedSessionLifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, lifecycle);
    assert!(deserialized.explicit_opt_out);
    assert_eq!(deserialized.governance_runtime, GovernanceRuntimeKind::Local);
}

#[test]
fn governance_intent_with_explicit_mode_and_no_canon() {
    let intent = GovernanceIntent {
        requested: true,
        runtime_preference: Some(GovernanceRuntimeKind::Canon),
        risk: Some("high".to_string()),
        zone: Some("payments".to_string()),
        owner: Some("platform".to_string()),
        explicit_mode: Some(CanonMode::Requirements),
        explicit_no_canon: false,
    };

    let json = serde_json::to_string(&intent).unwrap();
    let deserialized: GovernanceIntent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, intent);
    assert_eq!(deserialized.explicit_mode, Some(CanonMode::Requirements));
    assert!(!deserialized.explicit_no_canon);
}

#[test]
fn governance_intent_explicit_no_canon_flag() {
    let intent = GovernanceIntent {
        requested: false,
        runtime_preference: None,
        risk: None,
        zone: None,
        owner: None,
        explicit_mode: None,
        explicit_no_canon: true,
    };

    let json = serde_json::to_string(&intent).unwrap();
    let deserialized: GovernanceIntent = serde_json::from_str(&json).unwrap();
    assert!(deserialized.explicit_no_canon);
}

#[test]
fn governance_intent_backward_compatible_deserialization() {
    // Old JSON without the new fields should still deserialize
    let old_json = r#"{
        "requested": true,
        "runtime_preference": "canon",
        "risk": "medium",
        "zone": "core",
        "owner": "platform"
    }"#;

    let intent: GovernanceIntent = serde_json::from_str(old_json).unwrap();
    assert!(intent.requested);
    assert_eq!(intent.explicit_mode, None);
    assert!(!intent.explicit_no_canon);
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4 — User Story 2: Input Assembly and Canon-Ready Inputs
// ─────────────────────────────────────────────────────────────────────────────

use boundline::domain::task::{ClarificationReasonKind, ClarificationRecord, ClarificationStatus};
use boundline::orchestrator::governance::{bounded_governance_context, governance_input_documents};

/// T035: governance_input_documents maps briefs to input_documents with correct kinds.
#[test]
fn governance_input_documents_maps_briefs_with_correct_kinds() {
    use boundline::domain::brief::{
        AuthoredBriefBundle, AuthoredBriefResolutionState, InputSourceKind, InputSourceReference,
    };
    use serde_json::json;

    let bundle = AuthoredBriefBundle {
        bundle_id: "bundle-001".to_string(),
        primary_goal_text: Some("Build a task management API".to_string()),
        sources: vec![
            InputSourceReference {
                source_id: "src-001".to_string(),
                kind: InputSourceKind::AttachedMarkdown,
                display_name: "prd.md".to_string(),
                workspace_path: Some("docs/prd.md".to_string()),
                precedence: 0,
                content: "# PRD\nProduct brief".to_string(),
            },
            InputSourceReference {
                source_id: "src-002".to_string(),
                kind: InputSourceKind::AttachedMarkdown,
                display_name: "arch.md".to_string(),
                workspace_path: Some("docs/arch.md".to_string()),
                precedence: 1,
                content: "# Architecture\nArch notes".to_string(),
            },
        ],
        deduplicated_sources: Vec::new(),
        governance_intent: None,
        resolution_state: AuthoredBriefResolutionState::Ready,
        clarification: None,
        derived_task_draft: None,
        captured_at: 1000,
    };

    let task_input = json!({ "authored_brief": bundle });
    let docs = governance_input_documents(&task_input);

    assert_eq!(docs.len(), 2);
    assert_eq!(docs[0].kind, "stage-brief");
    assert_eq!(docs[0].path, "docs/prd.md");
    assert_eq!(docs[1].kind, "authored-brief");
    assert_eq!(docs[1].path, "docs/arch.md");
}

/// T036: Clarification answers are assembled as input_documents with kind "clarification-answer".
#[test]
fn governance_input_documents_includes_clarification_answers() {
    use boundline::domain::brief::{
        AuthoredBriefBundle, AuthoredBriefResolutionState, InputSourceKind, InputSourceReference,
    };
    use serde_json::json;

    let clarification = ClarificationRecord {
        clarification_id: "clarify-001".to_string(),
        reason_kind: ClarificationReasonKind::MissingContext,
        prompt: "What authentication method should be used?".to_string(),
        missing_fields: vec!["auth_method".to_string()],
        blocking_sources: Vec::new(),
        turn_index: 1,
        status: ClarificationStatus::Answered,
    };

    let bundle = AuthoredBriefBundle {
        bundle_id: "bundle-002".to_string(),
        primary_goal_text: Some("Build auth module".to_string()),
        sources: vec![InputSourceReference {
            source_id: "src-001".to_string(),
            kind: InputSourceKind::AttachedMarkdown,
            display_name: "brief.md".to_string(),
            workspace_path: Some("docs/brief.md".to_string()),
            precedence: 0,
            content: "# Brief".to_string(),
        }],
        deduplicated_sources: Vec::new(),
        governance_intent: None,
        resolution_state: AuthoredBriefResolutionState::Ready,
        clarification: Some(clarification),
        derived_task_draft: None,
        captured_at: 2000,
    };

    let task_input = json!({ "authored_brief": bundle });
    let docs = governance_input_documents(&task_input);

    // After T040 implementation, we expect a clarification-answer entry
    let clarification_docs: Vec<_> =
        docs.iter().filter(|d| d.kind == "clarification-answer").collect();
    assert_eq!(
        clarification_docs.len(),
        1,
        "expected one clarification-answer document, got {docs:?}"
    );
    assert!(
        clarification_docs[0].path.contains("clarification"),
        "clarification doc path should reference clarification: {:?}",
        clarification_docs[0].path
    );
}

/// T037: bounded_governance_context includes reused_packets from accumulated_context.
#[test]
fn bounded_governance_context_includes_reused_packets_from_accumulated_context() {
    use boundline::domain::flow::FlowStepMetadata;
    use boundline::domain::flow::built_in_flow;
    use boundline::domain::limits::RunLimits;
    use boundline::domain::task_context::TaskContext;
    use serde_json::{Map, json};

    // Set up a task context with a prior governed stage and packet
    let mut initial_state = Map::new();
    initial_state.insert(
        "latest_governance_stage".to_string(),
        json!({
            "stage_key": "delivery:requirements",
            "runtime": "canon",
            "lifecycle_state": "governed_ready",
            "required": true,
            "autopilot_enabled": false,
            "approval_state": "not_needed",
            "canon_run_ref": "run-prior-001",
            "governance_attempt_id": "attempt-prior",
            "previous_governance_attempt_id": null,
            "packet_ref": ".canon/runs/run-prior-001",
            "decision_ref": null,
            "blocked_reason": null
        }),
    );
    initial_state.insert(
        "latest_governance_packet".to_string(),
        json!({
            "packet_ref": ".canon/runs/run-prior-001",
            "runtime": "canon",
            "canon_mode": "requirements",
            "expected_document_refs": [],
            "document_refs": [".canon/runs/run-prior-001/requirements.md"],
            "readiness": "reusable",
            "missing_sections": [],
            "headline": "Requirements produced",
            "reason_code": "packet_ready"
        }),
    );

    let context = TaskContext::new(
        "session-001".to_string(),
        "/workspace".to_string(),
        RunLimits::default(),
        initial_state,
    );

    // Use the second stage (architecture) as the downstream metadata
    let flow = built_in_flow("delivery").unwrap();
    let stage = flow.stage(1).unwrap();
    let metadata = FlowStepMetadata {
        flow_name: "delivery".to_string(),
        stage_id: stage.id.to_string(),
        stage_index: 1,
        total_stages: flow.stages.len(),
    };

    let (bounded_ctx, packet_reuse) =
        bounded_governance_context(&context, &metadata, &["/workspace/src/".to_string()]).unwrap();

    assert!(
        !bounded_ctx.reused_packets.is_empty(),
        "expected reused_packets from prior governed stage"
    );
    assert_eq!(bounded_ctx.reused_packets[0].stage_key, "delivery:requirements");
    assert_eq!(bounded_ctx.reused_packets[0].packet_ref, ".canon/runs/run-prior-001");
    assert!(packet_reuse.is_some(), "expected packet reuse binding");
}

/// T041 (supplemental): enrich_bounded_context_with_accumulated adds reused_packets from accumulated docs.
#[test]
fn enrich_bounded_context_adds_accumulated_reused_packets() {
    use boundline::adapters::governance_runtime::GovernanceBoundedContext;
    use boundline::domain::governance::{GovernedDocumentRef, PacketReadiness};
    use boundline::orchestrator::governance::enrich_bounded_context_with_accumulated;

    let mut bounded_ctx = GovernanceBoundedContext {
        read_targets: vec!["/workspace/src/".to_string()],
        stage_brief_ref: None,
        reused_packets: Vec::new(),
    };

    let accumulated = vec![
        GovernedDocumentRef {
            stage_key: "delivery:requirements".to_string(),
            canon_mode: CanonMode::Requirements,
            packet_ref: "pkt-req-001".to_string(),
            document_path: Some(".canon/runs/run-001/requirements.md".to_string()),
            readiness: PacketReadiness::Reusable,
        },
        GovernedDocumentRef {
            stage_key: "delivery:architecture".to_string(),
            canon_mode: CanonMode::Architecture,
            packet_ref: "pkt-arch-001".to_string(),
            document_path: None,
            readiness: PacketReadiness::Incomplete, // Should be skipped
        },
    ];

    enrich_bounded_context_with_accumulated(&mut bounded_ctx, &accumulated);

    assert_eq!(bounded_ctx.reused_packets.len(), 1);
    assert_eq!(bounded_ctx.reused_packets[0].packet_ref, "pkt-req-001");
    assert_eq!(bounded_ctx.reused_packets[0].stage_key, "delivery:requirements");
}

/// T042: governed_document_ref_from_response creates GovernedDocumentRef from Canon response.
#[test]
fn governed_document_ref_from_response_creates_ref_on_reusable_packet() {
    use boundline::adapters::governance_runtime::GovernanceRuntimeResponse;
    use boundline::domain::governance::{
        ApprovalState, GovernanceLifecycleState, GovernedStagePacket, PacketReadiness,
    };
    use boundline::orchestrator::governance::governed_document_ref_from_response;

    let response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::GovernedReady,
        approval_state: ApprovalState::NotNeeded,
        run_ref: Some("run-001".to_string()),
        packet: Some(GovernedStagePacket {
            packet_ref: "pkt-001".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: Some(CanonMode::Requirements),
            expected_document_refs: Vec::new(),
            document_refs: vec![".canon/runs/run-001/requirements.md".to_string()],
            readiness: PacketReadiness::Reusable,
            missing_sections: Vec::new(),
            headline: "Requirements produced".to_string(),
            reason_code: Some("packet_ready".to_string()),
        }),
        reason_code: Some("packet_ready".to_string()),
        message: "Governed requirements produced".to_string(),
    };

    let doc_ref = governed_document_ref_from_response(
        "delivery:requirements",
        CanonMode::Requirements,
        &response,
    );
    assert!(doc_ref.is_some());
    let doc_ref = doc_ref.unwrap();
    assert_eq!(doc_ref.stage_key, "delivery:requirements");
    assert_eq!(doc_ref.canon_mode, CanonMode::Requirements);
    assert_eq!(doc_ref.packet_ref, "pkt-001");
    assert_eq!(doc_ref.document_path.as_deref(), Some(".canon/runs/run-001/requirements.md"));
    assert_eq!(doc_ref.readiness, PacketReadiness::Reusable);
}

/// T042: governed_document_ref_from_response returns None for non-reusable packet.
#[test]
fn governed_document_ref_from_response_returns_none_for_incomplete_packet() {
    use boundline::adapters::governance_runtime::GovernanceRuntimeResponse;
    use boundline::domain::governance::{
        ApprovalState, GovernanceLifecycleState, GovernedStagePacket, PacketReadiness,
    };
    use boundline::orchestrator::governance::governed_document_ref_from_response;

    let response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Blocked,
        approval_state: ApprovalState::NotNeeded,
        run_ref: Some("run-002".to_string()),
        packet: Some(GovernedStagePacket {
            packet_ref: "pkt-002".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: Some(CanonMode::Requirements),
            expected_document_refs: Vec::new(),
            document_refs: Vec::new(),
            readiness: PacketReadiness::Incomplete,
            missing_sections: vec!["Stakeholders".to_string()],
            headline: "Incomplete".to_string(),
            reason_code: None,
        }),
        reason_code: None,
        message: "Incomplete document".to_string(),
    };

    let doc_ref = governed_document_ref_from_response(
        "delivery:requirements",
        CanonMode::Requirements,
        &response,
    );
    assert!(doc_ref.is_none());
}

/// T043: clarification_prompt_from_response extracts prompt for incomplete Canon response.
#[test]
fn clarification_prompt_from_incomplete_response() {
    use boundline::adapters::governance_runtime::GovernanceRuntimeResponse;
    use boundline::domain::governance::{
        ApprovalState, GovernanceLifecycleState, GovernedStagePacket, PacketReadiness,
    };
    use boundline::orchestrator::governance::clarification_prompt_from_response;

    let response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::Blocked,
        approval_state: ApprovalState::NotNeeded,
        run_ref: Some("run-003".to_string()),
        packet: Some(GovernedStagePacket {
            packet_ref: "pkt-003".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: Some(CanonMode::Requirements),
            expected_document_refs: Vec::new(),
            document_refs: Vec::new(),
            readiness: PacketReadiness::Incomplete,
            missing_sections: vec![
                "Stakeholders".to_string(),
                "Non-Functional Requirements".to_string(),
            ],
            headline: "Incomplete".to_string(),
            reason_code: None,
        }),
        reason_code: None,
        message: "Document missing required sections".to_string(),
    };

    let prompt = clarification_prompt_from_response(&response);
    assert!(prompt.is_some());
    let prompt = prompt.unwrap();
    assert!(prompt.contains("incomplete"), "prompt: {prompt}");
    assert!(prompt.contains("Stakeholders"), "prompt: {prompt}");
    assert!(prompt.contains("Non-Functional Requirements"), "prompt: {prompt}");
}

/// T043: clarification_prompt_from_response extracts prompt for pending_selection.
#[test]
fn clarification_prompt_from_pending_selection_response() {
    use boundline::adapters::governance_runtime::GovernanceRuntimeResponse;
    use boundline::domain::governance::{ApprovalState, GovernanceLifecycleState};
    use boundline::orchestrator::governance::clarification_prompt_from_response;

    let response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::PendingSelection,
        approval_state: ApprovalState::NotNeeded,
        run_ref: None,
        packet: None,
        reason_code: None,
        message: "Please select a Canon mode for this stage".to_string(),
    };

    let prompt = clarification_prompt_from_response(&response);
    assert!(prompt.is_some());
    let prompt = prompt.unwrap();
    assert!(prompt.contains("mode selection"), "prompt: {prompt}");
}

/// T044: is_awaiting_approval_response detects approval-pending state.
#[test]
fn is_awaiting_approval_detects_approval_response() {
    use boundline::adapters::governance_runtime::GovernanceRuntimeResponse;
    use boundline::domain::governance::{ApprovalState, GovernanceLifecycleState};
    use boundline::orchestrator::governance::is_awaiting_approval_response;

    let approval_response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::AwaitingApproval,
        approval_state: ApprovalState::Requested,
        run_ref: Some("run-004".to_string()),
        packet: None,
        reason_code: None,
        message: "Waiting for approval".to_string(),
    };

    assert!(is_awaiting_approval_response(&approval_response));

    let ready_response = GovernanceRuntimeResponse {
        status: GovernanceLifecycleState::GovernedReady,
        approval_state: ApprovalState::NotNeeded,
        run_ref: Some("run-005".to_string()),
        packet: None,
        reason_code: None,
        message: "Ready".to_string(),
    };

    assert!(!is_awaiting_approval_response(&ready_response));
}

/// T045: lifecycle_requires_refresh detects awaiting-approval lifecycle.
#[test]
fn lifecycle_requires_refresh_detects_approval_pending() {
    use boundline::domain::governance::GovernedSessionLifecycle;
    use boundline::domain::session::{ActiveSessionRecord, SessionStatus};
    use boundline::orchestrator::governance::lifecycle_requires_refresh;

    let base_session = ActiveSessionRecord {
        session_id: "session-refresh-test".to_string(),
        workspace_ref: "/tmp/test".to_string(),
        goal: None,
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 1000,
        updated_at: 1000,
        governance_lifecycle: None,
    };

    // No lifecycle — no refresh needed
    assert!(!lifecycle_requires_refresh(&base_session));

    // Lifecycle without terminal reason
    let mut session = base_session.clone();
    session.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: Some(CanonMode::Requirements),
        selected_mode_sequence: Vec::new(),
        current_stage_index: 0,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
    });
    assert!(!lifecycle_requires_refresh(&session));

    // Lifecycle with awaiting approval
    session.governance_lifecycle.as_mut().unwrap().terminal_reason =
        Some("awaiting approval: requires team lead sign-off".to_string());
    assert!(lifecycle_requires_refresh(&session));
}

#[test]
fn canon_surface_verification_fails_when_operations_are_missing() {
    use boundline::domain::distribution::verify_canon_surface;
    use boundline::domain::governance::{CANONICAL_MODES, CanonCapabilitySnapshot};

    let snapshot = CanonCapabilitySnapshot {
        canon_version: "0.41.0".to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["capabilities".to_string()],
        supported_modes: CANONICAL_MODES.to_vec(),
        status_values: vec!["governed_ready".to_string()],
        approval_state_values: vec!["not_needed".to_string()],
        packet_readiness_values: vec!["reusable".to_string()],
        compatibility_notes: Vec::new(),
    };

    let verification = verify_canon_surface(std::path::Path::new("/tmp/canon"), &snapshot);

    assert!(!verification.ready);
    assert!(!verification.operations_verified);
    assert_eq!(verification.missing_operations, vec!["start", "refresh"]);
    assert!(verification.repair_actions.iter().any(|action| action.contains("start")));
}

#[test]
fn canon_surface_verification_checks_all_canonical_modes() {
    use boundline::domain::distribution::verify_canon_surface;
    use boundline::domain::governance::{CANONICAL_MODES, CanonCapabilitySnapshot, CanonMode};

    let mut missing_mode_snapshot = CanonCapabilitySnapshot {
        canon_version: "0.41.0".to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["start".to_string(), "refresh".to_string()],
        supported_modes: vec![CanonMode::Requirements],
        status_values: vec!["governed_ready".to_string()],
        approval_state_values: vec!["not_needed".to_string()],
        packet_readiness_values: vec!["reusable".to_string()],
        compatibility_notes: Vec::new(),
    };

    let missing_mode =
        verify_canon_surface(std::path::Path::new("/tmp/canon"), &missing_mode_snapshot);
    assert!(!missing_mode.ready);
    assert!(!missing_mode.modes_verified);
    assert!(missing_mode.missing_modes.contains(&CanonMode::SupplyChainAnalysis));

    missing_mode_snapshot.supported_modes = CANONICAL_MODES.to_vec();
    let ready = verify_canon_surface(std::path::Path::new("/tmp/canon"), &missing_mode_snapshot);
    assert!(ready.ready);
    assert!(ready.operations_verified);
    assert!(ready.modes_verified);
}
