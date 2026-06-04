use std::fs;
use std::path::Path;

use boundline::domain::brief::{AuthoredBriefResolutionState, GoalQualityAssessment};
use boundline::domain::flow::FlowStepMetadata;
use boundline::domain::governance::{
    CanonCapabilitySnapshot, CanonModeSelectionPreference, CanonModeSummary,
    CanonResultActionSummary, CompactedCanonMemory, GovernedDocumentRef, GovernedSessionLifecycle,
    MemoryCredibilityState, PacketReuseBindingReason, backlog_quality_snapshot_for_lifecycle,
    planning_stage_key_for_mode,
};
use boundline::domain::limits::RunLimits;
use boundline::domain::reasoning::{
    IndependenceFloor, ParticipantRoleDefinition, ReasoningAdjudicationMode, ReasoningBudget,
    ReasoningDegradationPolicy, ReasoningParticipantRoleKind, ReasoningProfileDefinition,
    ReasoningProfileFamily, ReasoningProfileId, ReasoningRoutePreference,
};
use boundline::domain::task_context::TaskContext;
use boundline::domain::task_context::{
    LATEST_GOVERNANCE_CONTRACT_LINES_KEY, LATEST_GOVERNANCE_DECISION_KEY,
    LATEST_GOVERNANCE_PACKET_KEY, LATEST_GOVERNANCE_PACKET_REUSE_KEY, LATEST_GOVERNANCE_REASON_KEY,
    LATEST_GOVERNANCE_STAGE_KEY,
};
use boundline::orchestrator::governance::{
    GovernanceProjectionSnapshot, default_stage_canon_mode, governance_input_documents,
    overlay_stage_policy_with_intent, requested_governance_intent,
};
use boundline::{
    ApprovalState, AuthoredBriefBundle, AutopilotAction, AutopilotDecisionRecord, CanonMode,
    CanonRuntimeConfig, GovernanceBoundedContext, GovernanceDegradationMode, GovernanceIntent,
    GovernanceLifecycleState, GovernanceProfile, GovernanceRolloutProfile, GovernanceRuntimeKind,
    GovernanceRuntimeState, GovernanceStartupContext, GovernanceTransitionDirection,
    GovernedStagePacket, GovernedStageRecord, InputSourceKind, InputSourceReference,
    PacketReadiness, PacketReuseBinding, SUPPORTED_CANON_VERSION, StageGovernancePolicy,
    StopSemantics, SystemContextBinding, assess_backlog_quality, autopilot_action_text,
    bounded_reused_packets, build_autopilot_decision, classify_packet_readiness,
    escalation_target_stage_key, governance_stage_key, governance_state_patch,
    narrowed_bounded_context, resolve_governance_startup_posture, select_packet_reuse_binding,
    selected_stage_policy, supported_canon_modes_for_stage,
};
use serde_json::json;
use uuid::Uuid;

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
        stage_council: None,
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
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Local),
        canon_mode: None,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
        reasoning_profile: None,
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
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(mode),
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
        reasoning_profile: None,
    }
}

fn sample_reasoning_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::BoundedSelfConsistency,
        family: ReasoningProfileFamily::SelfConsistency,
        allowed_stages: vec![CanonMode::Verification],
        limits: ReasoningBudget {
            max_participants: 1,
            max_branches: 2,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 2048,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![ParticipantRoleDefinition {
            role_id: "self_consistency_path".to_string(),
            role_kind: ReasoningParticipantRoleKind::IndependentPath,
            preferred_slot: ReasoningRoutePreference::Verification,
            independence_requirements: IndependenceFloor {
                route_distinct: false,
                provider_distinct: false,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 1,
            },
            required: true,
        }],
        adjudication_mode: ReasoningAdjudicationMode::None,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some("rerun bounded verification".to_string()),
        },
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
        goal_quality: GoalQualityAssessment::default(),
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
fn selected_stage_policy_preserves_reasoning_profile_attachment() {
    let mut policy = sample_canon_policy("bug-fix", "verify", CanonMode::Verification);
    policy.reasoning_profile = Some(sample_reasoning_profile());
    let profile = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Canon,
        canon: Some(sample_canon_config()),
        stages: vec![policy],
    };

    assert!(profile.validate().is_ok());

    let selected = selected_stage_policy(Some(&profile), "bug-fix", "verify")
        .and_then(|stage| stage.reasoning_profile);

    assert_eq!(
        selected.as_ref().map(|definition| definition.profile_id),
        Some(ReasoningProfileId::BoundedSelfConsistency)
    );
    assert_eq!(
        selected.as_ref().and_then(|definition| definition.allowed_stages.first().copied()),
        Some(CanonMode::Verification)
    );
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
        canon_version: SUPPORTED_CANON_VERSION.to_string(),
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
        authority_provenance_lines: Vec::new(),
        adaptive_provenance_lines: Vec::new(),
        semantic_provenance_lines: Vec::new(),
    };

    assert_eq!(
        snapshot.summary_text(),
        format!("Canon {SUPPORTED_CANON_VERSION} capabilities available")
    );
    assert!(mode_summary.summary_text().contains("execution posture: recommendation-only"));
    assert!(memory.summary_text().contains("Canon verification packet is still credible"));
    assert_eq!(MemoryCredibilityState::Stale.as_str(), "stale");
    assert_eq!(autopilot_action_text(AutopilotAction::AwaitApproval), "await_approval");
}

#[test]
fn governance_runtime_state_transition_helpers_classify_direction() {
    assert_eq!(
        GovernanceRuntimeState::Advisory.transition_direction_to(GovernanceRuntimeState::Rule),
        GovernanceTransitionDirection::Promote
    );
    assert_eq!(
        GovernanceRuntimeState::Hook.transition_direction_to(GovernanceRuntimeState::Catch),
        GovernanceTransitionDirection::Downgrade
    );
    assert_eq!(
        GovernanceRuntimeState::Catch.transition_direction_to(GovernanceRuntimeState::Catch),
        GovernanceTransitionDirection::NoChange
    );
}

#[test]
fn governance_rollout_profile_baseline_state_matches_maturity() {
    assert_eq!(
        GovernanceRolloutProfile::Minimal.baseline_runtime_state(),
        GovernanceRuntimeState::Advisory
    );
    assert_eq!(
        GovernanceRolloutProfile::Guided.baseline_runtime_state(),
        GovernanceRuntimeState::Catch
    );
    assert_eq!(
        GovernanceRolloutProfile::Governed.baseline_runtime_state(),
        GovernanceRuntimeState::Rule
    );
    assert_eq!(
        GovernanceRolloutProfile::Strict.baseline_runtime_state(),
        GovernanceRuntimeState::Hook
    );
}

#[test]
fn governance_degradation_modes_map_to_existing_stop_semantics() {
    assert_eq!(
        GovernanceDegradationMode::AdvisoryFallback.mapped_stop_semantics(),
        StopSemantics::ProceedWithAdvisory
    );
    assert_eq!(
        GovernanceDegradationMode::SmallerCouncil.mapped_stop_semantics(),
        StopSemantics::CouncilRequired
    );
    assert_eq!(
        GovernanceDegradationMode::HumanGate.mapped_stop_semantics(),
        StopSemantics::HumanGateRequired
    );
    assert_eq!(
        GovernanceDegradationMode::ReducedAutonomy.mapped_stop_semantics(),
        StopSemantics::ProceedWithWarning
    );
    assert_eq!(
        GovernanceDegradationMode::VerificationOnly.mapped_stop_semantics(),
        StopSemantics::DegradedProceed
    );
    assert_eq!(
        GovernanceDegradationMode::ExecutionBlock.mapped_stop_semantics(),
        StopSemantics::HardStop
    );
}

#[test]
fn governance_startup_posture_defaults_to_advisory_for_low_trust_surfaces() {
    let resolution = resolve_governance_startup_posture(GovernanceStartupContext {
        current_state: GovernanceRuntimeState::Advisory,
        requested_profile: None,
        operator_approved_profile: None,
        low_trust_surface: true,
    });

    assert_eq!(resolution.rollout_profile, GovernanceRolloutProfile::Minimal);
    assert_eq!(resolution.runtime_state, GovernanceRuntimeState::Advisory);
    assert_eq!(resolution.transition_direction, GovernanceTransitionDirection::NoChange);
}

#[test]
fn governance_startup_posture_keeps_requested_stronger_profile_advisory_without_approval() {
    let resolution = resolve_governance_startup_posture(GovernanceStartupContext {
        current_state: GovernanceRuntimeState::Advisory,
        requested_profile: Some(GovernanceRolloutProfile::Governed),
        operator_approved_profile: None,
        low_trust_surface: false,
    });

    assert_eq!(resolution.rollout_profile, GovernanceRolloutProfile::Minimal);
    assert_eq!(resolution.runtime_state, GovernanceRuntimeState::Advisory);
    assert_eq!(resolution.transition_direction, GovernanceTransitionDirection::NoChange);
}

#[test]
fn governance_startup_posture_promotes_when_stronger_profile_is_explicitly_approved() {
    let resolution = resolve_governance_startup_posture(GovernanceStartupContext {
        current_state: GovernanceRuntimeState::Advisory,
        requested_profile: Some(GovernanceRolloutProfile::Governed),
        operator_approved_profile: Some(GovernanceRolloutProfile::Governed),
        low_trust_surface: false,
    });

    assert_eq!(resolution.rollout_profile, GovernanceRolloutProfile::Governed);
    assert_eq!(resolution.runtime_state, GovernanceRuntimeState::Rule);
    assert_eq!(resolution.transition_direction, GovernanceTransitionDirection::Promote);
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
fn backlog_quality_blocks_closure_limited_backlog_packets() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Reusable),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
        ],
        &[],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nrisk-only-packet\n\n",
                "## Execution Handoff\n\nhandoff withheld for closure reasons\n"
            )
            .to_string(),
            "# Planning Risks\n\n- Closure evidence is incomplete.\n".to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "blocked");
    assert_eq!(assessment.findings, vec!["closure_limited_backlog_packet".to_string()]);
    assert_eq!(assessment.task_count, None);
}

#[test]
fn backlog_quality_requires_execution_handoff_for_full_packets() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Reusable),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/02-epic-tree.md".to_string(),
            ".canon/artifacts/run/backlog/03-capability-to-epic-map.md".to_string(),
            ".canon/artifacts/run/backlog/04-dependency-map.md".to_string(),
            ".canon/artifacts/run/backlog/05-delivery-slices.md".to_string(),
            ".canon/artifacts/run/backlog/06-sequencing-plan.md".to_string(),
            ".canon/artifacts/run/backlog/07-acceptance-anchors.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
        ],
        &[],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nfull-packet\n\n",
                "## Execution Handoff\n\nhandoff unavailable\n"
            )
            .to_string(),
            "# Epic Tree\n\n- Epic AUTH-SESSION: harden revocation boundaries.\n".to_string(),
            "# Capability To Epic Map\n\n- Auth session revocation -> AUTH-SESSION\n".to_string(),
            "# Dependency Map\n\n- [SLICE-AUTH-001] depends on rollback guard rails.\n".to_string(),
            concat!(
                "# Delivery Slices\n\n",
                "- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n",
                "- [SLICE-AUTH-002] Surface revocation evidence for operators.\n"
            )
            .to_string(),
            concat!(
                "# Sequencing Plan\n\n",
                "1. [SLICE-AUTH-001] first because it hardens the mutation boundary.\n",
                "2. [SLICE-AUTH-002] after the revoke path is stable.\n"
            )
            .to_string(),
            concat!(
                "# Acceptance Anchors\n\n",
                "- [SLICE-AUTH-001] Revocation stays rollback-safe.\n",
                "- [SLICE-AUTH-002] Evidence is externally reviewable.\n"
            )
            .to_string(),
            "# Planning Risks\n\n- Verification posture is still underspecified.\n".to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "clarification_required");
    assert_eq!(assessment.findings, vec!["missing_execution_handoff".to_string()]);
    assert_eq!(assessment.task_count, Some(2));
    assert_eq!(assessment.mvp_scope.as_deref(), Some("SLICE-AUTH-001"));
}

#[test]
fn backlog_quality_accepts_valid_backlog_document() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Reusable),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/02-epic-tree.md".to_string(),
            ".canon/artifacts/run/backlog/03-capability-to-epic-map.md".to_string(),
            ".canon/artifacts/run/backlog/04-dependency-map.md".to_string(),
            ".canon/artifacts/run/backlog/05-delivery-slices.md".to_string(),
            ".canon/artifacts/run/backlog/06-sequencing-plan.md".to_string(),
            ".canon/artifacts/run/backlog/07-acceptance-anchors.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
            ".canon/artifacts/run/backlog/09-execution-handoff.md".to_string(),
        ],
        &[],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nfull-packet\n\n",
                "## Execution Handoff\n\ngoverned execution handoff is available\n",
            )
            .to_string(),
            "# Epic Tree\n\n- Epic AUTH-SESSION: harden revocation boundaries.\n".to_string(),
            "# Capability To Epic Map\n\n- Auth session revocation -> AUTH-SESSION\n".to_string(),
            "# Dependency Map\n\n- [SLICE-AUTH-002] depends on [SLICE-AUTH-001].\n".to_string(),
            concat!(
                "# Delivery Slices\n\n",
                "- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n",
                "- [SLICE-AUTH-002] Surface revocation evidence for operators.\n",
            )
            .to_string(),
            concat!(
                "# Sequencing Plan\n\n",
                "1. [SLICE-AUTH-001] first because it hardens the mutation boundary.\n",
                "2. [SLICE-AUTH-002] after the revoke path is stable.\n"
            )
            .to_string(),
            concat!(
                "# Acceptance Anchors\n\n",
                "- [SLICE-AUTH-001] Revocation stays rollback-safe.\n",
                "- [SLICE-AUTH-002] Evidence is externally reviewable.\n",
                "Unmapped: post-launch adoption metric\n"
            )
            .to_string(),
            "# Planning Risks\n\n- Hidden coupling can widen rollback scope.\n".to_string(),
            concat!(
                "# Execution Handoff\n\n",
                "## Selected Slice\n\nSLICE-AUTH-001\n\n",
                "## Implementation Artifact References\n\n",
                "- src/auth/session.rs\n",
                "- tech-docs/changes/auth-session.md\n\n",
                "## Dependency Prerequisites\n\n",
                "- rollback guard rails remain intact.\n\n",
                "## Independent Verification Anchors\n\n",
                "- contract test proves session revoke remains bounded.\n",
            )
            .to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "ready");
    assert_eq!(assessment.findings, Vec::<String>::new());
    assert_eq!(assessment.task_count, Some(2));
    assert_eq!(assessment.mvp_scope.as_deref(), Some("SLICE-AUTH-001"));
    assert_eq!(assessment.unmapped_items, vec!["post-launch adoption metric".to_string()]);
}

#[test]
fn backlog_quality_reports_rejected_packet_missing_sections_and_handoff_content_gaps() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Rejected),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/02-epic-tree.md".to_string(),
            ".canon/artifacts/run/backlog/03-capability-to-epic-map.md".to_string(),
            ".canon/artifacts/run/backlog/05-delivery-slices.md".to_string(),
            ".canon/artifacts/run/backlog/06-sequencing-plan.md".to_string(),
            ".canon/artifacts/run/backlog/07-acceptance-anchors.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
            ".canon/artifacts/run/backlog/09-execution-handoff.md".to_string(),
        ],
        &["Dependency Map".to_string()],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nfull-packet\n\n",
                "## Execution Handoff\n\ngoverned execution handoff is available\n",
            )
            .to_string(),
            "# Epic Tree\n\n- Epic AUTH-SESSION: harden revocation boundaries.\n".to_string(),
            "# Capability To Epic Map\n\n- Auth session revocation -> AUTH-SESSION\n".to_string(),
            concat!(
                "# Delivery Slices\n\n",
                "- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n",
                "- [SLICE-AUTH-002] Surface revocation evidence for operators.\n",
            )
            .to_string(),
            concat!(
                "# Sequencing Plan\n\n",
                "1. [SLICE-AUTH-001] first because it hardens the mutation boundary.\n",
                "2. [SLICE-AUTH-002] after the revoke path is stable.\n"
            )
            .to_string(),
            concat!(
                "# Acceptance Anchors\n\n",
                "- [SLICE-AUTH-001] Revocation stays rollback-safe.\n",
                "- [SLICE-AUTH-002] Evidence is externally reviewable.\n",
            )
            .to_string(),
            "# Planning Risks\n\n- Hidden coupling can widen rollback scope.\n".to_string(),
            concat!(
                "# Execution Handoff\n\n",
                "## Dependency Prerequisites\n\n",
                "- rollback guard rails remain intact.\n"
            )
            .to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "blocked");
    assert!(assessment.findings.contains(&"backlog_packet_not_reusable".to_string()));
    assert!(assessment.findings.contains(&"missing_section:dependency_map".to_string()));
    assert!(assessment.findings.contains(&"missing_dependency_order".to_string()));
    assert!(assessment.findings.contains(&"invalid_selected_slice_id".to_string()));
    assert!(assessment.findings.contains(&"missing_implementation_artifact_refs".to_string()));
    assert!(assessment.findings.contains(&"missing_independent_verification_anchors".to_string()));
}

#[test]
fn backlog_quality_reports_invalid_selected_slice_when_handoff_points_outside_packet() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Reusable),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/02-epic-tree.md".to_string(),
            ".canon/artifacts/run/backlog/03-capability-to-epic-map.md".to_string(),
            ".canon/artifacts/run/backlog/04-dependency-map.md".to_string(),
            ".canon/artifacts/run/backlog/05-delivery-slices.md".to_string(),
            ".canon/artifacts/run/backlog/06-sequencing-plan.md".to_string(),
            ".canon/artifacts/run/backlog/07-acceptance-anchors.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
            ".canon/artifacts/run/backlog/09-execution-handoff.md".to_string(),
        ],
        &[],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nfull-packet\n\n",
                "## Execution Handoff\n\ngoverned execution handoff is available\n",
            )
            .to_string(),
            "# Epic Tree\n\n- Epic AUTH-SESSION: harden revocation boundaries.\n".to_string(),
            "# Capability To Epic Map\n\n- Auth session revocation -> AUTH-SESSION\n".to_string(),
            "# Dependency Map\n\n- [SLICE-AUTH-002] depends on [SLICE-AUTH-001].\n".to_string(),
            concat!(
                "# Delivery Slices\n\n",
                "- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n",
                "- [SLICE-AUTH-002] Surface revocation evidence for operators.\n",
            )
            .to_string(),
            concat!(
                "# Sequencing Plan\n\n",
                "1. [SLICE-AUTH-001] first because it hardens the mutation boundary.\n",
                "2. [SLICE-AUTH-002] after the revoke path is stable.\n"
            )
            .to_string(),
            concat!(
                "# Acceptance Anchors\n\n",
                "- [SLICE-AUTH-001] Revocation stays rollback-safe.\n",
                "- [SLICE-AUTH-002] Evidence is externally reviewable.\n",
            )
            .to_string(),
            "# Planning Risks\n\n- Hidden coupling can widen rollback scope.\n".to_string(),
            concat!(
                "# Execution Handoff\n\n",
                "## Selected Slice\n\nSLICE-AUTH-999\n\n",
                "## Implementation Artifact References\n\n",
                "- src/auth/session.rs\n\n",
                "## Independent Verification Anchors\n\n",
                "- contract test proves session revoke remains bounded.\n",
            )
            .to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "blocked");
    assert_eq!(assessment.findings, vec!["invalid_selected_slice_id".to_string()]);
}

#[test]
fn backlog_quality_closure_limited_packet_skips_missing_execution_handoff() {
    let assessment = assess_backlog_quality(
        Some(PacketReadiness::Pending),
        &[
            ".canon/artifacts/run/backlog/01-backlog-overview.md".to_string(),
            ".canon/artifacts/run/backlog/05-delivery-slices.md".to_string(),
            ".canon/artifacts/run/backlog/08-planning-risks.md".to_string(),
        ],
        &[],
        &[
            concat!(
                "# Backlog Overview\n\n",
                "## Decomposition Posture\n\nrisk-only-packet\n\n",
                "## Execution Handoff\n\nhandoff withheld for closure reasons\n"
            )
            .to_string(),
            "# Delivery Slices\n\n- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n"
                .to_string(),
            "# Planning Risks\n\n- Closure evidence is incomplete.\n".to_string(),
        ],
    );

    assert_eq!(assessment.state.as_str(), "blocked");
    assert!(assessment.findings.contains(&"backlog_packet_pending".to_string()));
    assert!(assessment.findings.contains(&"closure_limited_backlog_packet".to_string()));
    assert!(!assessment.findings.contains(&"missing_execution_handoff".to_string()));
}

#[test]
fn backlog_quality_snapshot_expands_expected_packet_refs_from_workspace()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace =
        std::env::temp_dir().join(format!("boundline-backlog-snapshot-{}", Uuid::new_v4()));
    let packet_ref = ".canon/artifacts/run/backlog";
    let packet_dir = workspace.join(packet_ref);
    fs::create_dir_all(&packet_dir)?;
    fs::write(
        packet_dir.join("backlog-overview.md"),
        "# Backlog Overview\n\n## Decomposition Posture\n\nfull-packet\n\n## Execution Handoff\n\ngoverned execution handoff is available\n",
    )?;
    fs::write(
        packet_dir.join("epic-tree.md"),
        "# Epic Tree\n\n- Epic AUTH-SESSION: harden revocation boundaries.\n",
    )?;
    fs::write(
        packet_dir.join("capability-to-epic-map.md"),
        "# Capability To Epic Map\n\n- Auth session revocation -> AUTH-SESSION\n",
    )?;
    fs::write(
        packet_dir.join("dependency-map.md"),
        "# Dependency Map\n\n- [SLICE-AUTH-002] depends on [SLICE-AUTH-001].\n",
    )?;
    fs::write(
        packet_dir.join("delivery-slices.md"),
        concat!(
            "# Delivery Slices\n\n",
            "- [SLICE-AUTH-001] Harden rollback-safe session revocation.\n",
            "- [SLICE-AUTH-002] Surface revocation evidence for operators.\n",
        ),
    )?;
    fs::write(
        packet_dir.join("sequencing-plan.md"),
        concat!(
            "# Sequencing Plan\n\n",
            "1. [SLICE-AUTH-001] first because it hardens the mutation boundary.\n",
            "2. [SLICE-AUTH-002] after the revoke path is stable.\n",
        ),
    )?;
    fs::write(
        packet_dir.join("acceptance-anchors.md"),
        concat!(
            "# Acceptance Anchors\n\n",
            "- [SLICE-AUTH-001] Revocation stays rollback-safe.\n",
            "- [SLICE-AUTH-002] Evidence is externally reviewable.\n",
        ),
    )?;
    fs::write(
        packet_dir.join("planning-risks.md"),
        "# Planning Risks\n\n- Hidden coupling can widen rollback scope.\n",
    )?;
    fs::write(
        packet_dir.join("execution-handoff.md"),
        concat!(
            "# Execution Handoff\n\n",
            "## Selected Slice\n\nSLICE-AUTH-001\n\n",
            "## Implementation Artifact References\n\n",
            "- src/auth/session.rs\n\n",
            "## Independent Verification Anchors\n\n",
            "- contract test proves session revoke remains bounded.\n",
        ),
    )?;

    let backlog_stage_key = planning_stage_key_for_mode(CanonMode::Backlog)
        .ok_or("backlog mode should map to a planning stage")?;
    let lifecycle = GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::Auto,
        selected_mode: Some(CanonMode::Backlog),
        selected_mode_sequence: vec![CanonMode::Backlog],
        latest_reasoning_profile: None,
        current_stage_index: 0,
        stage_records: vec![GovernedStageRecord {
            stage_key: backlog_stage_key.to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::GovernedReady,
            required: false,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: Some("run-1".to_string()),
            governance_attempt_id: "attempt-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(packet_ref.to_string()),
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        }],
        accumulated_context: vec![GovernedDocumentRef {
            stage_key: backlog_stage_key.to_string(),
            canon_mode: CanonMode::Backlog,
            packet_ref: packet_ref.to_string(),
            document_path: None,
            readiness: PacketReadiness::Reusable,
        }],
        terminal_reason: None,
        planning_input_fingerprint: None,
    };

    let snapshot = backlog_quality_snapshot_for_lifecycle(&lifecycle, &workspace)
        .ok_or("expected backlog quality snapshot to exist")?;

    assert_eq!(snapshot.assessment.state.as_str(), "ready");
    assert_eq!(snapshot.document_bodies.len(), 9);
    assert_eq!(snapshot.assessment.task_count, Some(2));

    fs::remove_dir_all(&workspace)?;
    Ok(())
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
        authority_governance: None,
        adaptive_governance: None,
        semantic_descriptor: None,
    };
    let reuse = PacketReuseBinding {
        upstream_stage_key: "bug-fix:investigate".to_string(),
        downstream_stage_key: "bug-fix:implement".to_string(),
        packet_ref: packet.packet_ref.clone(),
        binding_reason: PacketReuseBindingReason::UpstreamStageContext,
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
    let projection = GovernanceProjectionSnapshot {
        runtime_state: GovernanceRuntimeState::Advisory,
        rollout_profile: GovernanceRolloutProfile::Minimal,
        reason: "startup posture defaulted locally for low-trust surface".to_string(),
        contract_lines: vec![
            "authority_contract_line: unavailable".to_string(),
            "adaptive_contract_line: unavailable".to_string(),
        ],
        approval_provenance: "approval not required".to_string(),
    };

    let patch = governance_state_patch(
        &record,
        Some(&packet),
        Some(&reuse),
        Some(&decision),
        None,
        &projection,
    )
    .unwrap();

    assert_eq!(patch[LATEST_GOVERNANCE_STAGE_KEY]["stage_key"], "bug-fix:investigate");
    assert_eq!(patch[LATEST_GOVERNANCE_PACKET_KEY]["packet_ref"], packet.packet_ref);
    assert_eq!(
        patch[LATEST_GOVERNANCE_PACKET_REUSE_KEY]["binding_reason"],
        serde_json::json!(reuse.binding_reason)
    );
    assert_eq!(patch[LATEST_GOVERNANCE_DECISION_KEY]["decision_id"], "decision-1");
    assert_eq!(patch[LATEST_GOVERNANCE_REASON_KEY], json!(projection.reason));
    assert_eq!(patch[LATEST_GOVERNANCE_CONTRACT_LINES_KEY], json!(projection.contract_lines));
}

#[test]
fn governance_state_patch_omits_optional_entries_when_absent() {
    let projection = GovernanceProjectionSnapshot {
        runtime_state: GovernanceRuntimeState::Advisory,
        rollout_profile: GovernanceRolloutProfile::Minimal,
        reason: "startup posture defaulted locally for low-trust surface".to_string(),
        contract_lines: vec![
            "authority_contract_line: unavailable".to_string(),
            "adaptive_contract_line: unavailable".to_string(),
        ],
        approval_provenance: "approval not required".to_string(),
    };
    let patch =
        governance_state_patch(&sample_record(), None, None, None, None, &projection).unwrap();

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
            require_adaptive_companion: false,
            runtime: Some(GovernanceRuntimeKind::Canon),
            canon_mode: None,
            system_context: None,
            risk: None,
            zone: None,
            owner: None,
            reasoning_profile: None,
        }],
    };

    profile.validate().unwrap();
}

#[test]
fn canon_mode_helpers_expose_primary_documents_and_context_requirements() {
    let expectations = [
        (CanonMode::Requirements, "requirements.md", false),
        (CanonMode::Architecture, "architecture.md", false),
        (CanonMode::Backlog, "backlog-overview.md", true),
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
        CanonMode::Backlog.expected_document_refs(".canon/packets/backlog-1"),
        vec![
            ".canon/packets/backlog-1/backlog-overview.md".to_string(),
            ".canon/packets/backlog-1/epic-tree.md".to_string(),
            ".canon/packets/backlog-1/capability-to-epic-map.md".to_string(),
            ".canon/packets/backlog-1/dependency-map.md".to_string(),
            ".canon/packets/backlog-1/delivery-slices.md".to_string(),
            ".canon/packets/backlog-1/sequencing-plan.md".to_string(),
            ".canon/packets/backlog-1/acceptance-anchors.md".to_string(),
            ".canon/packets/backlog-1/planning-risks.md".to_string(),
        ]
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
        ("delivery", "system-shaping", vec![CanonMode::SystemShaping]),
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
fn default_stage_canon_mode_uses_authoritative_stage_candidate_order() {
    let investigate_policy = StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        enabled: true,
        required: true,
        autopilot: false,
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: None,
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
        reasoning_profile: None,
    };
    assert_eq!(
        default_stage_canon_mode(&investigate_policy, GovernanceRuntimeKind::Local),
        Some(CanonMode::Discovery)
    );

    let verify_policy =
        StageGovernancePolicy { stage_id: "verify".to_string(), ..investigate_policy.clone() };
    assert_eq!(
        default_stage_canon_mode(&verify_policy, GovernanceRuntimeKind::Local),
        Some(CanonMode::SecurityAssessment)
    );

    let inherited_canon_policy =
        StageGovernancePolicy { runtime: None, ..investigate_policy.clone() };
    assert_eq!(
        default_stage_canon_mode(&inherited_canon_policy, GovernanceRuntimeKind::Canon),
        Some(CanonMode::Discovery)
    );
    assert_eq!(
        default_stage_canon_mode(&inherited_canon_policy, GovernanceRuntimeKind::Local),
        None
    );
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
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: None,
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
        reasoning_profile: None,
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
            stage_council: None,
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
            authority_governance: None,
            adaptive_governance: None,
            semantic_descriptor: None,
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
    assert_eq!(binding.binding_reason, PacketReuseBindingReason::UpstreamStageContext);
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
            stage_council: None,
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
            authority_governance: None,
            adaptive_governance: None,
            semantic_descriptor: None,
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

    assert_eq!(binding.binding_reason, PacketReuseBindingReason::SameStageRerun);
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
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(CanonMode::Verification),
        system_context: Some(SystemContextBinding::Existing),
        risk: Some("medium".to_string()),
        zone: Some("engineering".to_string()),
        owner: Some("platform".to_string()),
        reasoning_profile: None,
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
