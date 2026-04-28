use std::fs;
use std::path::PathBuf;

use synod::domain::brief::{
    BriefIngestionError, GovernanceIntent, normalize_governance_intent,
    normalize_inputs_with_governance,
};
use synod::domain::governance::GovernanceRuntimeKind;
use uuid::Uuid;

fn temp_workspace(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn normalize_governance_intent_preserves_explicit_canon_business_fields() {
    let intent = normalize_governance_intent(
        Some(GovernanceRuntimeKind::Canon),
        Some(" high "),
        Some(" payments "),
        Some(" platform "),
    )
    .unwrap();

    assert_eq!(
        intent,
        Some(GovernanceIntent {
            requested: true,
            runtime_preference: Some(GovernanceRuntimeKind::Canon),
            risk: Some("high".to_string()),
            zone: Some("payments".to_string()),
            owner: Some("platform".to_string()),
        })
    );
}

#[test]
fn normalize_governance_intent_marks_request_when_only_business_fields_are_present() {
    let intent = normalize_governance_intent(None, Some("medium"), Some("core"), None).unwrap();

    assert_eq!(
        intent,
        Some(GovernanceIntent {
            requested: true,
            runtime_preference: None,
            risk: Some("medium".to_string()),
            zone: Some("core".to_string()),
            owner: None,
        })
    );
}

#[test]
fn normalize_inputs_with_governance_persists_intent_inside_the_bundle() {
    let workspace = temp_workspace("synod-human-governance-bundle");
    let intent = normalize_governance_intent(
        Some(GovernanceRuntimeKind::Local),
        Some("low"),
        Some("developer-experience"),
        Some("platform"),
    )
    .unwrap();

    let bundle = normalize_inputs_with_governance(
        &workspace,
        Some("Fix the failing checkout flow"),
        &[],
        intent.clone(),
    )
    .unwrap();

    assert_eq!(bundle.governance_intent, intent);
}

#[test]
fn normalize_governance_intent_rejects_explicit_canon_requests_missing_owner() {
    let error = normalize_governance_intent(
        Some(GovernanceRuntimeKind::Canon),
        Some("high"),
        Some("payments"),
        None,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        BriefIngestionError::MissingGovernanceField { field, runtime }
            if field == "owner" && runtime == GovernanceRuntimeKind::Canon
    ));
}
