use synod::{
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
        runtime,
        canon_mode,
        system_context: None,
        risk: None,
        zone: None,
        owner: None,
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
        &[CanonMode::Change]
    );
    assert_eq!(
        supported_canon_modes_for_stage("change", "implement"),
        &[CanonMode::Implementation]
    );
    assert_eq!(
        supported_canon_modes_for_stage("change", "verify"),
        &[CanonMode::Verification, CanonMode::PrReview]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "investigate"),
        &[CanonMode::Discovery, CanonMode::Change]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "implement"),
        &[CanonMode::Implementation]
    );
    assert_eq!(
        supported_canon_modes_for_stage("bug-fix", "verify"),
        &[CanonMode::Verification, CanonMode::PrReview]
    );
    assert!(supported_canon_modes_for_stage("bug-fix", "missing").is_empty());
}

#[test]
fn canon_stage_mapping_derives_candidates_from_stage_support_order() {
    let policy = canon_policy("investigate", Some(GovernanceRuntimeKind::Canon), None);

    assert_eq!(
        candidate_canon_modes(&policy, GovernanceRuntimeKind::Local),
        vec![CanonMode::Discovery, CanonMode::Change]
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
    assert_eq!(autopilot_action_text(synod::AutopilotAction::SelectMode), "select_mode");
    assert_eq!(
        autopilot_action_text(synod::AutopilotAction::RetryStageWithNarrowedContext),
        "retry_stage_with_narrowed_context"
    );
    assert_eq!(autopilot_action_text(synod::AutopilotAction::BlockStage), "block_stage");
}
