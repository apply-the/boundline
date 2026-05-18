use std::str::FromStr;

use boundline::{
    CanonMode, GovernanceRuntimeKind, StageGovernancePolicy, autopilot_action_text,
    candidate_canon_modes, resolved_canon_mode, supported_canon_modes_for_stage,
};

fn canon_policy(
    stage_id: &str,
    runtime: Option<GovernanceRuntimeKind>,
    canon_mode: Option<CanonMode>,
) -> StageGovernancePolicy {
    StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: stage_id.to_string(),
        enabled: true,
        required: false,
        autopilot: true,
        require_adaptive_companion: false,
        runtime,
        canon_mode,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
        reasoning_profile: None,
    }
}

#[test]
fn first_slice_canon_stage_mapping_matches_supported_flows() {
    assert_eq!(
        supported_canon_modes_for_stage("delivery", "requirements"),
        &[CanonMode::Requirements]
    );
    assert_eq!(
        supported_canon_modes_for_stage("delivery", "architecture"),
        &[CanonMode::Architecture]
    );
    assert_eq!(supported_canon_modes_for_stage("delivery", "backlog"), &[CanonMode::Backlog]);
    assert_eq!(
        supported_canon_modes_for_stage("delivery", "implementation"),
        &[CanonMode::Implementation]
    );
    assert_eq!(
        supported_canon_modes_for_stage("change", "understand-change"),
        &[CanonMode::Change, CanonMode::Discovery]
    );
    assert_eq!(
        supported_canon_modes_for_stage("change", "implement"),
        &[CanonMode::Implementation, CanonMode::Refactor]
    );
    assert_eq!(
        supported_canon_modes_for_stage("change", "verify"),
        &[
            CanonMode::SecurityAssessment,
            CanonMode::Verification,
            CanonMode::Review,
            CanonMode::PrReview,
        ]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "investigate"),
        &[CanonMode::Discovery, CanonMode::Change, CanonMode::Incident]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "implement"),
        &[CanonMode::Implementation, CanonMode::Refactor]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "verify"),
        &[
            CanonMode::SecurityAssessment,
            CanonMode::Verification,
            CanonMode::Review,
            CanonMode::PrReview,
        ]
    );
    assert!(supported_canon_modes_for_stage("bug-fix", "missing").is_empty());
}

#[test]
fn canon_mode_wire_format_matches_canon_and_accepts_legacy_snake_case() {
    assert_eq!(
        serde_json::to_string(&CanonMode::SecurityAssessment).unwrap(),
        "\"security-assessment\""
    );
    assert_eq!(serde_json::to_string(&CanonMode::PrReview).unwrap(), "\"pr-review\"");
    assert_eq!(
        serde_json::from_str::<CanonMode>("\"security_assessment\"").unwrap(),
        CanonMode::SecurityAssessment
    );
    assert_eq!(serde_json::from_str::<CanonMode>("\"pr_review\"").unwrap(), CanonMode::PrReview);
}

#[test]
fn expanded_canon_modes_round_trip_and_map_to_primary_documents() {
    let expectations = [
        (CanonMode::SystemShaping, "system-shaping", "system-shaping.md"),
        (CanonMode::Refactor, "refactor", "refactor.md"),
        (CanonMode::Review, "review", "review.md"),
        (CanonMode::Incident, "incident", "incident.md"),
        (CanonMode::SystemAssessment, "system-assessment", "system-assessment.md"),
        (CanonMode::Migration, "migration", "migration.md"),
        (CanonMode::SupplyChainAnalysis, "supply-chain-analysis", "supply-chain-analysis.md"),
    ];

    for (mode, wire, primary_document) in expectations {
        assert_eq!(mode.to_string(), wire);
        assert_eq!(CanonMode::from_str(wire).unwrap(), mode);
        assert_eq!(serde_json::to_string(&mode).unwrap(), format!("\"{wire}\""));
        assert_eq!(serde_json::from_str::<CanonMode>(&format!("\"{wire}\"")).unwrap(), mode);
        assert_eq!(mode.primary_document_name(), primary_document);
    }
}

#[test]
fn canon_stage_mapping_prefers_security_assessment_for_verify_stage_autopilot_candidates() {
    let policy = canon_policy("verify", Some(GovernanceRuntimeKind::Canon), None);

    assert_eq!(
        candidate_canon_modes(&policy, GovernanceRuntimeKind::Local),
        vec![
            CanonMode::SecurityAssessment,
            CanonMode::Verification,
            CanonMode::Review,
            CanonMode::PrReview,
        ]
    );
    assert_eq!(resolved_canon_mode(&policy, GovernanceRuntimeKind::Local), None);
}

#[test]
fn canon_stage_mapping_derives_candidates_from_stage_support_order() {
    let policy = canon_policy("investigate", Some(GovernanceRuntimeKind::Canon), None);

    assert_eq!(
        candidate_canon_modes(&policy, GovernanceRuntimeKind::Local),
        vec![CanonMode::Discovery, CanonMode::Change, CanonMode::Incident]
    );
    assert_eq!(resolved_canon_mode(&policy, GovernanceRuntimeKind::Local), None);
}

#[test]
fn canon_stage_mapping_preserves_explicit_mode_and_local_defaults() {
    let explicit =
        canon_policy("investigate", Some(GovernanceRuntimeKind::Canon), Some(CanonMode::Change));
    let local = canon_policy("investigate", Some(GovernanceRuntimeKind::Local), None);

    assert!(candidate_canon_modes(&explicit, GovernanceRuntimeKind::Local).is_empty());
    assert_eq!(
        resolved_canon_mode(&explicit, GovernanceRuntimeKind::Local),
        Some(CanonMode::Change)
    );
    assert!(candidate_canon_modes(&local, GovernanceRuntimeKind::Local).is_empty());
    assert_eq!(resolved_canon_mode(&local, GovernanceRuntimeKind::Local), None);
}

#[test]
fn canon_stage_mapping_exposes_cli_action_labels() {
    assert_eq!(autopilot_action_text(boundline::AutopilotAction::SelectMode), "select_mode");
    assert_eq!(
        autopilot_action_text(boundline::AutopilotAction::RetryStageWithNarrowedContext),
        "retry_stage_with_narrowed_context"
    );
    assert_eq!(autopilot_action_text(boundline::AutopilotAction::BlockStage), "block_stage");
}
