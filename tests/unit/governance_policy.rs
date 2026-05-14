use std::path::Path;

use boundline::domain::brief::AuthoredBriefResolutionState;
use boundline::domain::flow::FlowStepMetadata;
use boundline::domain::governance::{
    CanonCapabilitySnapshot, CanonModeSummary, CanonResultActionSummary, CompactedCanonMemory,
    MemoryCredibilityState,
};
use boundline::domain::limits::RunLimits;
use boundline::domain::task_context::TaskContext;
use boundline::domain::task_context::{
    LATEST_GOVERNANCE_DECISION_KEY, LATEST_GOVERNANCE_PACKET_KEY,
    LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_STAGE_KEY,
};
use boundline::orchestrator::governance::{
    governance_input_documents, overlay_stage_policy_with_intent, requested_governance_intent,
};
use boundline::{
    ApprovalState, AuthoredBriefBundle, AutopilotAction, AutopilotDecisionRecord, CanonMode,
    CanonRuntimeConfig, GovernanceBoundedContext, GovernanceIntent, GovernanceLifecycleState,
    GovernanceProfile, GovernanceRuntimeKind, GovernedStagePacket, GovernedStageRecord,
    InputSourceKind, InputSourceReference, PacketReadiness, PacketReuseBinding,
    StageGovernancePolicy, SystemContextBinding, autopilot_action_text, bounded_reused_packets,
    build_autopilot_decision, classify_packet_readiness, escalation_target_stage_key,
    governance_stage_key, governance_state_patch, narrowed_bounded_context,
    select_packet_reuse_binding, selected_stage_policy, supported_canon_modes_for_stage,
};
use serde_json::json;

fn sample_record() -> GovernedStageRecord {
    GovernedStageRecord {
        stage_key: "bug-fix:investigate".to_string(),
        runtime: GovernanceRuntimeKind::Local,
        lifecycle_state: GovernanceLifecycleState::GovernedReady,
        required: false,
        autopilot_enabled: false,
        approval_state: ApprovalState::NotNeeded,
        canon_run_ref: None,
        governance_attempt_id: "attempt-1".to_string(),
        previous_governance_attempt_id: None,
        packet_ref: Some(".boundline/governance/bug-fix-investigate/attempt-1".to_string()),
        decision_ref: Some("decision-1".to_string()),
        blocked_reason: None,
    }
}

fn sample_policy() -> StageGovernancePolicy {
    StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        enabled: true,
        required: false,
        autopilot: false,
        runtime: Some(GovernanceRuntimeKind::Local),
        canon_mode: None,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
    }
}

fn sample_canon_config() -> CanonRuntimeConfig {
    CanonRuntimeConfig {
        command: "canon".to_string(),
        default_owner: Some("platform".to_string()),
        default_risk: Some("medium".to_string()),
        default_zone: Some("engineering".to_string()),
        default_system_context: Some(SystemContextBinding::Existing),
    }
}

fn sample_canon_policy(flow_name: &str, stage_id: &str, mode: CanonMode) -> StageGovernancePolicy {
    StageGovernancePolicy {
        flow_name: flow_name.to_string(),
        stage_id: stage_id.to_string(),
        enabled: true,
        required: false,
        autopilot: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(mode),
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
    }
}

fn sample_governance_intent(runtime_preference: Option<GovernanceRuntimeKind>) -> GovernanceIntent {
    GovernanceIntent {
        requested: true,
        runtime_preference,
        risk: Some("high".to_string()),
        zone: Some("payments".to_string()),
        owner: Some("platform".to_string()),
        explicit_mode: None,
        explicit_no_canon: false,
    }
}

fn sample_authored_bundle(governance_intent: Option<GovernanceIntent>) -> AuthoredBriefBundle {
    AuthoredBriefBundle {
        bundle_id: "bundle-1".to_string(),
        primary_goal_text: Some("Fix the failing checkout flow".to_string()),
        sources: vec![
            InputSourceReference {
                source_id: "source-1".to_string(),
                kind: InputSourceKind::DirectText,
                display_name: "developer goal".to_string(),
                workspace_path: None,
                precedence: 0,
                content: "Fix the failing checkout flow".to_string(),
            },
            InputSourceReference {
                source_id: "source-2".to_string(),
                kind: InputSourceKind::AttachedMarkdown,
                display_name: "brief.md".to_string(),
                workspace_path: Some("docs/brief.md".to_string()),
                precedence: 1,
                content: "# Brief".to_string(),
            },
            InputSourceReference {
                source_id: "source-3".to_string(),
                kind: InputSourceKind::ReferencedMarkdown,
                display_name: "notes.md".to_string(),
                workspace_path: Some("docs/notes.md".to_string()),
                precedence: 2,
                content: "# Notes".to_string(),
            },
        ],
        deduplicated_sources: Vec::new(),
        governance_intent,
        resolution_state: AuthoredBriefResolutionState::Ready,
        clarification: None,
        derived_task_draft: None,
        captured_at: 1,
    }
}

#[test]
fn governance_stage_helpers_select_expected_policy() {
    let policy = sample_policy();
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![policy.clone()],
    };

    assert_eq!(governance_stage_key("bug-fix", "investigate"), "bug-fix:investigate");
    assert_eq!(selected_stage_policy(Some(&profile), "bug-fix", "investigate"), Some(policy));
    assert_eq!(selected_stage_policy(Some(&profile), "bug-fix", "verify"), None);
    assert_eq!(selected_stage_policy(None, "bug-fix", "investigate"), None);
}

#[test]
fn requested_governance_intent_prefers_top_level_value_over_authored_bundle_fallback() {
    let top_level_intent = sample_governance_intent(Some(GovernanceRuntimeKind::Local));
    let bundle_intent = sample_governance_intent(Some(GovernanceRuntimeKind::Canon));
    let task_input = json!({
        "governance_intent": top_level_intent,
        "authored_brief": sample_authored_bundle(Some(bundle_intent)),
    });

    assert_eq!(
        requested_governance_intent(&task_input),
        Some(sample_governance_intent(Some(GovernanceRuntimeKind::Local)))
    );
}

#[test]
fn requested_governance_intent_falls_back_to_authored_bundle_and_overlays_policy() {
    let bundle_intent = sample_governance_intent(Some(GovernanceRuntimeKind::Canon));
    let task_input = json!({
        "authored_brief": sample_authored_bundle(Some(bundle_intent.clone())),
    });
    let mut policy = sample_policy();
    policy.enabled = false;
    policy.runtime = Some(GovernanceRuntimeKind::Local);

    let intent = requested_governance_intent(&task_input);
    let overlaid = overlay_stage_policy_with_intent(&policy, intent.as_ref());

    assert_eq!(intent, Some(bundle_intent));
    assert!(overlaid.enabled);
    assert!(overlaid.required);
    assert_eq!(overlaid.runtime, Some(GovernanceRuntimeKind::Canon));
    assert_eq!(overlaid.risk.as_deref(), Some("high"));
    assert_eq!(overlaid.zone.as_deref(), Some("payments"));
    assert_eq!(overlaid.owner.as_deref(), Some("platform"));
}

#[test]
fn governance_input_documents_uses_first_workspace_doc_as_stage_brief() {
    let task_input = json!({
        "authored_brief": sample_authored_bundle(None),
    });

    let documents = governance_input_documents(&task_input, None);

    assert_eq!(documents.len(), 2);
    assert_eq!(documents[0].kind, "stage-brief");
    assert_eq!(documents[0].path, "docs/brief.md");
    assert_eq!(documents[1].kind, "authored-brief");
    assert_eq!(documents[1].path, "docs/notes.md");
}

#[test]
fn canon_helper_summaries_render_expected_text() {
    let snapshot = CanonCapabilitySnapshot {
        canon_version: "0.51.0".to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["start".to_string(), "refresh".to_string()],
        supported_modes: vec![CanonMode::Discovery],
        status_values: vec!["governed_ready".to_string()],
        approval_state_values: vec!["not_needed".to_string()],
        packet_readiness_values: vec!["reusable".to_string()],
        compatibility_notes: Vec::new(),
    };
    let mode_summary = CanonModeSummary {
        headline: "Verification packet ready".to_string(),
        artifact_packet_summary: "Primary artifact is ready".to_string(),
        execution_posture: Some("recommendation-only".to_string()),
        primary_artifact_title: "Verification".to_string(),
        primary_artifact_path: ".canon/runs/run-1/verification.md".to_string(),
        primary_artifact_action: CanonResultActionSummary {
            label: "inspect".to_string(),
            target: ".canon/runs/run-1/verification.md".to_string(),
        },
        result_excerpt: "No contradiction found".to_string(),
        action_chip_labels: vec!["inspect".to_string()],
    };
    let memory = CompactedCanonMemory {
        headline: "Canon verification packet is still credible".to_string(),
        credibility: MemoryCredibilityState::Credible,
        stage_key: Some("change:verify".to_string()),
        run_ref: Some("run-1".to_string()),
        packet_ref: Some(".canon/runs/run-1".to_string()),
        reason_code: None,
        artifact_refs: vec![".canon/runs/run-1/verification.md".to_string()],
        mode_summary: Some(mode_summary.clone()),
        possible_actions: Vec::new(),
        recommended_next_action: None,
        evidence_summary: None,
    };

    assert_eq!(snapshot.summary_text(), "Canon 0.51.0 capabilities available");
    assert!(mode_summary.summary_text().contains("execution posture: recommendation-only"));
    assert!(memory.summary_text().contains("Canon verification packet is still credible"));
    assert_eq!(MemoryCredibilityState::Stale.as_str(), "stale");
    assert_eq!(autopilot_action_text(AutopilotAction::AwaitApproval), "await_approval");
}

#[test]
fn packet_readiness_defaults_to_incomplete_without_expected_documents() {
    let readiness = classify_packet_readiness(
        Path::new("/tmp/unused"),
        &[],
        &[],
        &[],
        PacketReadiness::Reusable,
    );

    assert_eq!(readiness, PacketReadiness::Incomplete);
}

#[test]
fn governance_state_patch_writes_all_present_entries() {
    let record = sample_record();
    let packet = GovernedStagePacket {
        packet_ref: ".boundline/governance/bug-fix-investigate/attempt-1".to_string(),
        runtime: GovernanceRuntimeKind::Local,
        canon_mode: None,
        expected_document_refs: vec!["packet/brief.md".to_string()],
        document_refs: vec!["packet/brief.md".to_string()],
        readiness: PacketReadiness::Reusable,
        missing_sections: Vec::new(),
        headline: "local packet".to_string(),
        reason_code: None,
    };
    let reuse = PacketReuseBinding {
        upstream_stage_key: "bug-fix:investigate".to_string(),
        downstream_stage_key: "bug-fix:implement".to_string(),
        packet_ref: packet.packet_ref.clone(),
        binding_reason: "immediate upstream governance packet".to_string(),
    };
    let decision = AutopilotDecisionRecord {
        decision_id: "decision-1".to_string(),
        stage_key: "bug-fix:investigate".to_string(),
        candidate_actions: vec![AutopilotAction::SelectMode, AutopilotAction::AwaitApproval],
        candidate_modes: vec![CanonMode::Discovery],
        selected_action: Some(AutopilotAction::SelectMode),
        selected_mode: Some(CanonMode::Discovery),
        selected_target_stage_key: None,
        rationale: "discovery best matches investigate".to_string(),
        blocked_reason: None,
    };

    let patch = governance_state_patch(&record, Some(&packet), Some(&reuse), Some(&decision), None)
        .unwrap();

    assert_eq!(patch[LATEST_GOVERNANCE_STAGE_KEY]["stage_key"], "bug-fix:investigate");
    assert_eq!(patch[LATEST_GOVERNANCE_PACKET_KEY]["packet_ref"], packet.packet_ref);
    assert_eq!(patch[LATEST_GOVERNANCE_PACKET_REUSE_KEY]["binding_reason"], reuse.binding_reason);
    assert_eq!(patch[LATEST_GOVERNANCE_DECISION_KEY]["decision_id"], "decision-1");
}

#[test]
fn governance_state_patch_omits_optional_entries_when_absent() {
    let patch = governance_state_patch(&sample_record(), None, None, None, None).unwrap();

    assert!(patch.contains_key(LATEST_GOVERNANCE_STAGE_KEY));
    assert!(patch[LATEST_GOVERNANCE_PACKET_KEY].is_null());
    assert!(patch[LATEST_GOVERNANCE_PACKET_REUSE_KEY].is_null());
    assert!(patch[LATEST_GOVERNANCE_DECISION_KEY].is_null());
}

#[test]
fn governance_profile_validation_rejects_duplicate_stage_policies() {
    let policy = sample_policy();
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![policy.clone(), policy],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("duplicated"));
}

#[test]
fn governance_profile_validation_rejects_unsupported_flows_and_stages() {
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![StageGovernancePolicy {
            flow_name: "unknown-flow".to_string(),
            stage_id: "investigate".to_string(),
            ..sample_policy()
        }],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("not a supported built-in flow"));

    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![StageGovernancePolicy {
            flow_name: "bug-fix".to_string(),
            stage_id: "unknown-stage".to_string(),
            ..sample_policy()
        }],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("not a supported built-in stage"));
}

#[test]
fn governance_profile_validation_rejects_existing_only_modes_with_new_context() {
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Canon,
        canon: Some(sample_canon_config()),
        stages: vec![StageGovernancePolicy {
            required: true,
            system_context: Some(SystemContextBinding::New),
            ..sample_canon_policy("bug-fix", "implement", CanonMode::Implementation)
        }],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("cannot bind system_context"));
}

#[test]
fn governance_profile_validation_rejects_disabled_required_and_autopilot_policies() {
    let mut required_policy = sample_policy();
    required_policy.enabled = false;
    required_policy.required = true;
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![required_policy],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("cannot be required unless it is enabled"));

    let mut autopilot_policy = sample_policy();
    autopilot_policy.enabled = false;
    autopilot_policy.autopilot = true;
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![autopilot_policy],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("cannot enable autopilot unless it is enabled"));
}

#[test]
fn governance_profile_validation_rejects_missing_canon_configuration_and_forbidden_mode() {
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Canon,
        canon: None,
        stages: vec![sample_canon_policy("bug-fix", "investigate", CanonMode::Discovery)],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("requires Canon configuration"));

    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Canon,
        canon: Some(sample_canon_config()),
        stages: vec![sample_canon_policy("bug-fix", "investigate", CanonMode::Implementation)],
    };

    let error = profile.validate().unwrap_err();
    assert!(error.to_string().contains("cannot bind Canon mode"));
}

#[test]
fn governance_profile_deserialization_rejects_future_canon_modes_outside_the_current_slice() {
    let profile = serde_json::from_value::<GovernanceProfile>(json!({
        "default_runtime": "canon",
        "canon": {
            "command": "canon",
            "default_owner": "platform",
            "default_risk": "medium",
            "default_zone": "engineering",
            "default_system_context": "existing"
        },
        "stages": [
            {
                "flow_name": "bug-fix",
                "stage_id": "verify",
                "enabled": true,
                "required": true,
                "autopilot": false,
                "runtime": "canon",
                "canon_mode": "supply-chain-analysis",
                "system_context": "existing",
                "risk": "medium",
                "zone": "engineering",
                "owner": "platform"
            }
        ]
    }))
    .unwrap();

    let error = profile.validate().unwrap_err();
    let message = error.to_string();
    assert!(message.contains("cannot bind Canon mode"), "{message}");
}

#[test]
fn governance_profile_validation_rejects_missing_canon_fields() {
    for missing_field in ["system_context", "risk", "zone", "owner"] {
        let mut policy = sample_canon_policy("bug-fix", "investigate", CanonMode::Discovery);
        let mut canon = sample_canon_config();
        match missing_field {
            "system_context" => {
                policy.system_context = None;
                canon.default_system_context = None;
            }
            "risk" => {
                policy.risk = None;
                canon.default_risk = None;
            }
            "zone" => {
                policy.zone = None;
                canon.default_zone = None;
            }
            "owner" => {
                policy.owner = None;
                canon.default_owner = None;
            }
            _ => unreachable!("unexpected field"),
        }

        let profile = GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Canon,
            canon: Some(canon),
            stages: vec![policy],
        };

        let error = profile.validate().unwrap_err();
        assert!(
            error.to_string().contains(&format!("missing Canon field '{missing_field}'")),
            "unexpected error for {missing_field}: {error}"
        );
    }
}

#[test]
fn governance_profile_validation_accepts_canon_defaults_for_single_mode_stage() {
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Canon,
        canon: Some(CanonRuntimeConfig {
            default_system_context: Some(SystemContextBinding::New),
            ..sample_canon_config()
        }),
        stages: vec![StageGovernancePolicy {
            flow_name: "delivery".to_string(),
            stage_id: "requirements".to_string(),
            enabled: true,
            required: false,
            autopilot: false,
            runtime: Some(GovernanceRuntimeKind::Canon),
            canon_mode: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
        }],
    };

    profile.validate().unwrap();
}

#[test]
fn canon_mode_helpers_expose_primary_documents_and_context_requirements() {
    let expectations = [
        (CanonMode::Requirements, "requirements.md", false),
        (CanonMode::Architecture, "architecture.md", false),
        (CanonMode::Backlog, "backlog.md", true),
        (CanonMode::Change, "change.md", true),
        (CanonMode::Discovery, "discovery.md", false),
        (CanonMode::Implementation, "implementation.md", true),
        (CanonMode::Verification, "verification.md", true),
        (CanonMode::SecurityAssessment, "security-assessment.md", true),
        (CanonMode::PrReview, "pr-review.md", true),
    ];

    for (mode, document_name, requires_existing_context) in expectations {
        assert_eq!(mode.primary_document_name(), document_name);
        assert_eq!(mode.requires_existing_context(), requires_existing_context);
    }

    assert_eq!(
        CanonMode::Verification.expected_document_refs(".canon/packets/verify-1"),
        vec![".canon/packets/verify-1/verification.md".to_string()]
    );
    assert_eq!(
        CanonMode::SecurityAssessment.expected_document_refs(".canon/packets/security-1"),
        vec![".canon/packets/security-1/security-assessment.md".to_string()]
    );
}

#[test]
fn supported_canon_modes_include_expected_stage_whitelist_entries() {
    let expectations = [
        ("delivery", "requirements", vec![CanonMode::Requirements]),
        ("delivery", "architecture", vec![CanonMode::Architecture]),
        ("delivery", "backlog", vec![CanonMode::Backlog]),
        ("delivery", "implementation", vec![CanonMode::Implementation]),
        ("change", "understand-change", vec![CanonMode::Change, CanonMode::Discovery]),
        ("change", "implement", vec![CanonMode::Implementation, CanonMode::Refactor]),
        (
            "change",
            "verify",
            vec![
                CanonMode::SecurityAssessment,
                CanonMode::Verification,
                CanonMode::Review,
                CanonMode::PrReview,
            ],
        ),
        (
            "bug-fix",
            "investigate",
            vec![CanonMode::Discovery, CanonMode::Change, CanonMode::Incident],
        ),
        ("bug-fix", "implement", vec![CanonMode::Implementation, CanonMode::Refactor]),
        (
            "bug-fix",
            "verify",
            vec![
                CanonMode::SecurityAssessment,
                CanonMode::Verification,
                CanonMode::Review,
                CanonMode::PrReview,
            ],
        ),
    ];

    for (flow_name, stage_id, expected_modes) in expectations {
        assert_eq!(supported_canon_modes_for_stage(flow_name, stage_id), expected_modes.as_slice());
    }

    assert!(supported_canon_modes_for_stage("delivery", "unknown").is_empty());
}

#[test]
fn autopilot_selects_security_assessment_first_for_verification_stage() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "verify".to_string(),
        stage_index: 2,
        total_stages: 3,
    };
    let policy = StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: "verify".to_string(),
        enabled: true,
        required: false,
        autopilot: true,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: None,
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
    };

    let decision = build_autopilot_decision(
        "attempt-security-assessment",
        &policy,
        GovernanceRuntimeKind::Local,
        &metadata,
        &GovernanceBoundedContext {
            read_targets: vec!["src/lib.rs".to_string()],
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        None,
        Some(ApprovalState::NotNeeded),
        Some(PacketReadiness::Reusable),
    )
    .expect("decision should exist");

    assert_eq!(decision.selected_action, Some(AutopilotAction::SelectMode));
    assert_eq!(decision.selected_mode, Some(CanonMode::SecurityAssessment));
    assert_eq!(
        decision.candidate_modes,
        vec![
            CanonMode::SecurityAssessment,
            CanonMode::Verification,
            CanonMode::Review,
            CanonMode::PrReview,
        ]
    );
}

#[test]
fn stage_governance_policy_effective_runtime_prefers_override() {
    let policy = sample_policy();
    assert_eq!(
        policy.effective_runtime(GovernanceRuntimeKind::Canon),
        GovernanceRuntimeKind::Local
    );

    let inherited = StageGovernancePolicy { runtime: None, ..sample_policy() };
    assert_eq!(
        inherited.effective_runtime(GovernanceRuntimeKind::Canon),
        GovernanceRuntimeKind::Canon
    );
}

#[test]
fn canon_runtime_config_validation_rejects_blank_command() {
    let error = CanonRuntimeConfig { command: "   ".to_string(), ..sample_canon_config() }
        .validate()
        .unwrap_err();

    assert!(error.to_string().contains("requires a Canon command"));
}

#[test]
fn governance_reuse_binding_uses_immediate_upstream_stage_context() {
    let mut context = TaskContext::new(
        "session-governance",
        "/tmp/boundline-governance",
        RunLimits::default(),
        serde_json::Map::new(),
    );
    context
        .set_latest_governance_stage(&GovernedStageRecord {
            stage_key: "bug-fix:investigate".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: false,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: Some("canon-run-1".to_string()),
            governance_attempt_id: "attempt-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(".canon/runs/canon-run-1".to_string()),
            decision_ref: None,
            blocked_reason: None,
        })
        .unwrap();
    context
        .set_latest_governance_packet(&GovernedStagePacket {
            packet_ref: ".canon/runs/canon-run-1".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: Some(CanonMode::Discovery),
            expected_document_refs: vec![".canon/runs/canon-run-1/discovery.md".to_string()],
            document_refs: vec![".canon/runs/canon-run-1/discovery.md".to_string()],
            readiness: PacketReadiness::Reusable,
            missing_sections: Vec::new(),
            headline: "investigation packet ready".to_string(),
            reason_code: None,
        })
        .unwrap();
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "implement".to_string(),
        stage_index: 1,
        total_stages: 3,
    };

    let binding =
        select_packet_reuse_binding(&context, &metadata).unwrap().expect("binding should exist");
    let reused_packets = bounded_reused_packets(&context, &metadata).unwrap();

    assert_eq!(binding.upstream_stage_key, "bug-fix:investigate");
    assert_eq!(binding.downstream_stage_key, "bug-fix:implement");
    assert_eq!(binding.binding_reason, "upstream_stage_context");
    assert_eq!(reused_packets.len(), 1);
    assert_eq!(reused_packets[0].stage_key, "bug-fix:investigate");
    assert_eq!(reused_packets[0].packet_ref, ".canon/runs/canon-run-1");
}

#[test]
fn governance_reuse_binding_supports_same_stage_rerun() {
    let mut context = TaskContext::new(
        "session-governance",
        "/tmp/boundline-governance",
        RunLimits::default(),
        serde_json::Map::new(),
    );
    context
        .set_latest_governance_stage(&GovernedStageRecord {
            stage_key: "bug-fix:implement".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: false,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: Some("canon-run-2".to_string()),
            governance_attempt_id: "attempt-2".to_string(),
            previous_governance_attempt_id: Some("attempt-1".to_string()),
            packet_ref: Some(".canon/runs/canon-run-2".to_string()),
            decision_ref: None,
            blocked_reason: None,
        })
        .unwrap();
    context
        .set_latest_governance_packet(&GovernedStagePacket {
            packet_ref: ".canon/runs/canon-run-2".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            canon_mode: Some(CanonMode::Implementation),
            expected_document_refs: vec![".canon/runs/canon-run-2/implementation.md".to_string()],
            document_refs: vec![".canon/runs/canon-run-2/implementation.md".to_string()],
            readiness: PacketReadiness::Reusable,
            missing_sections: Vec::new(),
            headline: "implementation packet ready".to_string(),
            reason_code: None,
        })
        .unwrap();
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "implement".to_string(),
        stage_index: 1,
        total_stages: 3,
    };

    let binding =
        select_packet_reuse_binding(&context, &metadata).unwrap().expect("binding should exist");

    assert_eq!(binding.binding_reason, "same_stage_rerun");
    assert_eq!(binding.upstream_stage_key, "bug-fix:implement");
}

#[test]
fn autopilot_waits_for_approval_before_reselecting_modes() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        stage_index: 0,
        total_stages: 3,
    };
    let policy = StageGovernancePolicy {
        autopilot: true,
        runtime: Some(GovernanceRuntimeKind::Canon),
        ..sample_policy()
    };
    let decision = build_autopilot_decision(
        "attempt-approval",
        &policy,
        GovernanceRuntimeKind::Local,
        &metadata,
        &GovernanceBoundedContext {
            read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        Some(GovernanceLifecycleState::AwaitingApproval),
        Some(ApprovalState::Requested),
        Some(PacketReadiness::Pending),
    )
    .expect("decision should exist");

    assert_eq!(decision.selected_action, Some(AutopilotAction::AwaitApproval));
    assert_eq!(decision.selected_mode, None);
    assert_eq!(
        decision.candidate_actions,
        vec![AutopilotAction::AwaitApproval, AutopilotAction::SelectMode]
    );
}

#[test]
fn autopilot_returns_none_when_disabled() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        stage_index: 0,
        total_stages: 3,
    };

    assert_eq!(
        build_autopilot_decision(
            "attempt-disabled",
            &sample_policy(),
            GovernanceRuntimeKind::Local,
            &metadata,
            &GovernanceBoundedContext {
                read_targets: vec!["src/lib.rs".to_string()],
                stage_brief_ref: None,
                reused_packets: Vec::new(),
            },
            None,
            None,
            None,
        ),
        None,
    );
}

#[test]
fn autopilot_prefers_narrowed_context_after_packet_rejection() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "implement".to_string(),
        stage_index: 1,
        total_stages: 3,
    };
    let policy = StageGovernancePolicy {
        autopilot: true,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(CanonMode::Implementation),
        ..sample_policy()
    };
    let bounded_context = GovernanceBoundedContext {
        read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
        stage_brief_ref: None,
        reused_packets: Vec::new(),
    };
    let decision = build_autopilot_decision(
        "attempt-rejected",
        &policy,
        GovernanceRuntimeKind::Local,
        &metadata,
        &bounded_context,
        Some(GovernanceLifecycleState::Blocked),
        Some(ApprovalState::NotNeeded),
        Some(PacketReadiness::Rejected),
    )
    .expect("decision should exist");

    assert_eq!(decision.selected_action, Some(AutopilotAction::RetryStageWithNarrowedContext));
    assert_eq!(decision.selected_mode, Some(CanonMode::Implementation));
    assert_eq!(
        narrowed_bounded_context(&bounded_context).unwrap().read_targets,
        vec!["src/lib.rs".to_string()]
    );
}

#[test]
fn autopilot_blocks_required_stage_without_a_compliant_continuation() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        stage_index: 0,
        total_stages: 3,
    };
    let policy = StageGovernancePolicy {
        autopilot: true,
        required: true,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(CanonMode::Discovery),
        ..sample_policy()
    };

    let decision = build_autopilot_decision(
        "attempt-blocked",
        &policy,
        GovernanceRuntimeKind::Local,
        &metadata,
        &GovernanceBoundedContext {
            read_targets: vec!["src/lib.rs".to_string()],
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        None,
        Some(ApprovalState::NotNeeded),
        Some(PacketReadiness::Reusable),
    )
    .expect("decision should exist");

    assert_eq!(decision.selected_action, None);
    assert_eq!(decision.selected_mode, Some(CanonMode::Discovery));
    assert!(decision.candidate_actions.contains(&AutopilotAction::BlockStage));
    assert_eq!(
        decision.blocked_reason,
        Some("no compliant governance continuation exists for bug-fix:investigate".to_string())
    );
}

#[test]
fn autopilot_escalates_pr_review_from_verification_stage() {
    let metadata = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "verify".to_string(),
        stage_index: 2,
        total_stages: 3,
    };
    let policy = StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: "verify".to_string(),
        enabled: true,
        required: false,
        autopilot: true,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(CanonMode::Verification),
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
    };

    let decision = build_autopilot_decision(
        "attempt-pr-review",
        &policy,
        GovernanceRuntimeKind::Local,
        &metadata,
        &GovernanceBoundedContext {
            read_targets: vec!["src/lib.rs".to_string()],
            stage_brief_ref: None,
            reused_packets: Vec::new(),
        },
        None,
        Some(ApprovalState::NotNeeded),
        Some(PacketReadiness::Reusable),
    )
    .expect("decision should exist");

    assert_eq!(decision.selected_action, Some(AutopilotAction::EscalatePrReview));
    assert_eq!(decision.selected_mode, Some(CanonMode::Verification));
    assert_eq!(decision.selected_target_stage_key, Some("bug-fix:verify".to_string()));
}

#[test]
fn autopilot_exposes_verification_and_pr_review_escalation_targets() {
    let implement = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "implement".to_string(),
        stage_index: 1,
        total_stages: 3,
    };
    let verify = FlowStepMetadata {
        flow_name: "bug-fix".to_string(),
        stage_id: "verify".to_string(),
        stage_index: 2,
        total_stages: 3,
    };

    assert_eq!(
        escalation_target_stage_key(&implement, AutopilotAction::EscalateVerification),
        Some("bug-fix:verify".to_string())
    );
    assert_eq!(
        escalation_target_stage_key(&verify, AutopilotAction::EscalatePrReview),
        Some("bug-fix:verify".to_string())
    );
}
