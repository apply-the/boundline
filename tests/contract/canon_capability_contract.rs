use boundline::{
    CanonCapabilitySnapshot, CanonMode, governed_stage_catalog,
    validate_canon_capabilities_for_mode,
};

#[test]
fn canon_045_capability_snapshot_accepts_every_governed_stage_mode() {
    let snapshot: CanonCapabilitySnapshot = serde_json::from_str(include_str!(
        "../fixtures/project_scale_delivery/canon_capabilities_045_full.json"
    ))
    .unwrap();

    for entry in governed_stage_catalog() {
        validate_canon_capabilities_for_mode(&snapshot, entry.mode)
            .unwrap_or_else(|error| panic!("{} should be supported: {error}", entry.mode.as_str()));
    }
}

#[test]
fn canon_capability_validation_rejects_unavailable_mode_without_fallback() {
    let snapshot: CanonCapabilitySnapshot = serde_json::from_str(include_str!(
        "../fixtures/project_scale_delivery/canon_capabilities_missing_pr_review.json"
    ))
    .unwrap();

    let error = validate_canon_capabilities_for_mode(&snapshot, CanonMode::PrReview).unwrap_err();
    assert!(error.contains("unsupported"), "{error}");
    assert!(error.contains("pr-review"), "{error}");
}
