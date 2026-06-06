use boundline::{
    CanonMode, GovernedStageCategory, governed_stage_catalog, validate_canon_capabilities_for_mode,
};

#[test]
fn governed_stage_catalog_covers_all_project_scale_canon_modes() {
    let catalog = governed_stage_catalog();
    let modes = catalog.iter().map(|entry| entry.mode).collect::<Vec<_>>();

    assert_eq!(modes.len(), 19);
    for mode in [
        CanonMode::Discovery,
        CanonMode::Requirements,
        CanonMode::SystemShaping,
        CanonMode::Architecture,
        CanonMode::Backlog,
        CanonMode::Change,
        CanonMode::Implementation,
        CanonMode::Refactor,
        CanonMode::Review,
        CanonMode::Verification,
        CanonMode::PrReview,
        CanonMode::Incident,
        CanonMode::SecurityAssessment,
        CanonMode::SystemAssessment,
        CanonMode::Migration,
        CanonMode::SupplyChainAnalysis,
        CanonMode::Brainstorming,
        CanonMode::Debugging,
        CanonMode::PolicyShaping,
    ] {
        assert!(modes.contains(&mode), "missing mode {mode}");
    }
}

#[test]
fn governed_stage_catalog_records_category_and_delivery_follow_up() {
    let catalog = governed_stage_catalog();
    let architecture = catalog
        .iter()
        .find(|entry| entry.mode == CanonMode::Architecture)
        .expect("architecture entry");
    let implementation = catalog
        .iter()
        .find(|entry| entry.mode == CanonMode::Implementation)
        .expect("implementation entry");
    let security = catalog
        .iter()
        .find(|entry| entry.mode == CanonMode::SecurityAssessment)
        .expect("security-assessment entry");

    assert_eq!(architecture.category, GovernedStageCategory::Planning);
    assert!(architecture.voting_may_be_required);
    assert!(architecture.can_lead_to_implementation_or_refactor);
    assert_eq!(implementation.category, GovernedStageCategory::ExecutionGuidance);
    assert!(implementation.recommendation_only);
    assert_eq!(security.category, GovernedStageCategory::Assessment);
    assert!(security.voting_may_be_required);
}

#[test]
fn capability_validation_rejects_unavailable_modes_explicitly() {
    let snapshot: boundline::CanonCapabilitySnapshot = serde_json::from_str(include_str!(
        "../fixtures/project_scale_delivery/canon_capabilities_missing_pr_review.json"
    ))
    .unwrap();

    let error = validate_canon_capabilities_for_mode(&snapshot, CanonMode::PrReview).unwrap_err();

    assert!(error.contains("pr-review"));
    assert!(error.contains("unsupported"));
}
