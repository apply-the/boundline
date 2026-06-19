//! Integration tests for inference economics domain model.

use boundline_core::domain::inference_economics::{
    AdmissionDecision, ApprovalScope, BudgetState, Currency, InferenceEconomicsConfig,
    InvocationCostRecord, MonetaryAmount, ReconciledCostQuality, ReservationCostQuality,
    SessionBudgetConfig, SessionBudgetProjection, load_economics_config,
};
use std::str::FromStr;

#[test]
fn config_to_budget_round_trip() {
    let config = InferenceEconomicsConfig {
        session_budget: Some(SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("10.00").expect("valid"),
        }),
        staleness_threshold_days: 30,
        unknown_cost_policy: Default::default(),
        approval_default_scope: ApprovalScope::SingleCall,
    };
    assert!(config.validate().is_ok());
    let proj = SessionBudgetProjection::new(config.session_budget.as_ref());
    assert_eq!(proj.budget_state, BudgetState::InBounds);
    assert_eq!(proj.unknown_cost_call_count, 0);
}

#[test]
fn budget_exhaustion_blocks_calls() {
    let config = SessionBudgetConfig {
        currency: Currency::Usd,
        limit: MonetaryAmount::from_str("1.00").expect("valid"),
    };
    let mut proj = SessionBudgetProjection::new(Some(&config));
    proj.reserve(MonetaryAmount::from_str("1.00").expect("valid"), "snap-1", 0).expect("ok");
    assert_eq!(proj.budget_state, BudgetState::Exhausted);
    let decision = proj.check_admission(
        MonetaryAmount::from_str("0.01").expect("valid"),
        ReconciledCostQuality::Exact,
        0,
    );
    assert_eq!(decision, AdmissionDecision::Blocked);
}

#[test]
fn defaults_preserved_when_economics_disabled() {
    let proj = SessionBudgetProjection::new(None);
    assert_eq!(proj.budget_state, BudgetState::Disabled);
    let decision = proj.check_admission(
        MonetaryAmount::from_str("999.99").expect("valid"),
        ReconciledCostQuality::Unknown,
        0,
    );
    assert_eq!(decision, AdmissionDecision::InBounds);
}

#[test]
fn unknown_cost_call_increments_counter() {
    let config = SessionBudgetConfig {
        currency: Currency::Usd,
        limit: MonetaryAmount::from_str("50.00").expect("valid"),
    };
    let mut proj = SessionBudgetProjection::new(Some(&config));
    let reservation = MonetaryAmount::from_str("3.00").expect("valid");
    proj.reserve(reservation, "snap-1", 0).expect("ok");
    let record = InvocationCostRecord {
        call_id: "call-x".into(),
        provider_id: "u".into(),
        model_id: "u".into(),
        reservation_amount: reservation,
        final_amount: MonetaryAmount::zero(),
        native_currency: None,
        normalized_currency: Currency::Usd,
        conversion_source: None,
        conversion_timestamp: None,
        pricing_snapshot_id: "snap-1".into(),
        snapshot_age_secs: 0,
        cost_quality: ReconciledCostQuality::Unknown,
        reservation_confidence: ReservationCostQuality::Unknown,
    };
    proj.reconcile(record).expect("ok");
    assert_eq!(proj.unknown_cost_call_count, 1);
}

#[test]
fn config_loader_validates() {
    let raw: toml::Table =
        toml::from_str(r#"session_budget = { currency = "usd", limit = "10.00" }"#).expect("parse");
    let result = load_economics_config(Some(&raw)).expect("ok");
    assert!(result.is_some());
}
