//! Budget enforcement adapter for Boundline.
//!
//! Wraps [`SessionBudgetProjection`] and provides the I/O boundary
//! for reserving budget before provider calls and reconciling spend
//! after calls complete.

use boundline_core::domain::inference_economics::{
    AdmissionDecision, BudgetError, BudgetState, InvocationCostRecord, MonetaryAmount,
    ReconciledCostQuality, SessionBudgetProjection,
};

/// Orchestrates budget enforcement for a single session.
///
/// Delegates state-machine logic to [`SessionBudgetProjection`] and
/// adds the I/O-context methods that the provider dispatch path calls.
pub struct BudgetEnforcer {
    projection: SessionBudgetProjection,
}

impl BudgetEnforcer {
    /// Create a new enforcer backed by the given projection.
    #[must_use]
    pub fn new(projection: SessionBudgetProjection) -> Self {
        Self { projection }
    }

    /// Reserve budget before a provider call is dispatched.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError`] when the reservation would exceed the
    /// remaining budget or enforcement is disabled.
    pub fn reserve_before_call(
        &mut self,
        amount: MonetaryAmount,
        snapshot_id: &str,
        snapshot_age_secs: u64,
    ) -> Result<(), BudgetError> {
        self.projection.reserve(amount, snapshot_id, snapshot_age_secs)
    }

    /// Reconcile spend after a provider call completes.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError`] when there is no matching reservation.
    pub fn reconcile_after_call(&mut self, cost: InvocationCostRecord) -> Result<(), BudgetError> {
        self.projection.reconcile(cost)
    }

    /// Check whether a call can be admitted under the current budget.
    #[must_use]
    pub fn check_admission(
        &self,
        amount: MonetaryAmount,
        cost_quality: ReconciledCostQuality,
        authority_zone: u8,
    ) -> AdmissionDecision {
        self.projection.check_admission(amount, cost_quality, authority_zone)
    }

    /// Return a reference to the current projection for status output.
    #[must_use]
    pub fn projection(&self) -> &SessionBudgetProjection {
        &self.projection
    }

    /// Return whether budget enforcement is active.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.projection.budget_state != BudgetState::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boundline_core::domain::inference_economics::{Currency, SessionBudgetConfig};
    use std::str::FromStr;

    fn test_config(limit: &str) -> SessionBudgetConfig {
        SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str(limit).expect("valid"),
        }
    }

    #[test]
    fn new_enforcer_is_enabled_with_budget() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let enforcer = BudgetEnforcer::new(proj);
        assert!(enforcer.is_enabled());
    }

    #[test]
    fn new_enforcer_is_disabled_without_budget() {
        let proj = SessionBudgetProjection::new(None);
        let enforcer = BudgetEnforcer::new(proj);
        assert!(!enforcer.is_enabled());
    }

    #[test]
    fn reserve_before_call_succeeds() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let mut enforcer = BudgetEnforcer::new(proj);
        let amount = MonetaryAmount::from_str("3.00").expect("valid");
        enforcer.reserve_before_call(amount, "snap-1", 0).expect("ok");
        assert_eq!(enforcer.projection().reserved, amount);
    }

    #[test]
    fn reserve_before_call_fails_when_exhausted() {
        let cfg = test_config("1.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let mut enforcer = BudgetEnforcer::new(proj);
        let amount = MonetaryAmount::from_str("1.00").expect("valid");
        enforcer.reserve_before_call(amount, "snap-1", 0).expect("ok");
        let err = enforcer
            .reserve_before_call(MonetaryAmount::from_str("0.01").expect("valid"), "snap-2", 0)
            .unwrap_err();
        assert_eq!(err, BudgetError::BudgetExhausted);
    }

    #[test]
    fn reconcile_after_call_updates_projection() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let mut enforcer = BudgetEnforcer::new(proj);
        let reservation = MonetaryAmount::from_str("3.00").expect("valid");
        enforcer.reserve_before_call(reservation, "snap-1", 0).expect("ok");
        let record = InvocationCostRecord {
            call_id: "c1".into(),
            provider_id: "p".into(),
            model_id: "m".into(),
            reservation_amount: reservation,
            final_amount: MonetaryAmount::from_str("2.50").expect("valid"),
            native_currency: None,
            normalized_currency: Currency::Usd,
            conversion_source: None,
            conversion_timestamp: None,
            pricing_snapshot_id: "snap-1".into(),
            snapshot_age_secs: 0,
            cost_quality: ReconciledCostQuality::Exact,
            reservation_confidence:
                boundline_core::domain::inference_economics::ReservationCostQuality::CurrentEstimate,
        };
        enforcer.reconcile_after_call(record).expect("ok");
        assert_eq!(
            enforcer.projection().known_spent,
            MonetaryAmount::from_str("2.50").expect("valid")
        );
    }

    #[test]
    fn check_admission_in_bounds() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let enforcer = BudgetEnforcer::new(proj);
        let decision = enforcer.check_admission(
            MonetaryAmount::from_str("5.00").expect("valid"),
            ReconciledCostQuality::Estimated,
            0,
        );
        assert_eq!(decision, AdmissionDecision::InBounds);
    }

    #[test]
    fn check_admission_blocked_when_exhausted() {
        let cfg = test_config("1.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let mut enforcer = BudgetEnforcer::new(proj);
        enforcer
            .reserve_before_call(MonetaryAmount::from_str("1.00").expect("valid"), "snap-1", 0)
            .expect("ok");
        let decision = enforcer.check_admission(
            MonetaryAmount::from_str("0.01").expect("valid"),
            ReconciledCostQuality::Exact,
            0,
        );
        assert_eq!(decision, AdmissionDecision::Blocked);
    }

    #[test]
    fn projection_returns_reference() {
        let cfg = test_config("5.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let enforcer = BudgetEnforcer::new(proj);
        assert_eq!(
            enforcer.projection().budget_limit,
            MonetaryAmount::from_str("5.00").expect("valid")
        );
    }
}
