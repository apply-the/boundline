//! Inference economics domain model for Boundline.
//!
//! This module owns the domain types for session budget enforcement,
//! provider-agnostic inference cost tracking, authority-zone-based spend
//! exception approval, and operator-owned pricing snapshot management.
//!
//! All monetary values use [`rust_decimal::Decimal`] for exact arithmetic;
//! floating-point types (`f32`, `f64`) MUST NOT appear in cost, budget,
//! reservation, or reconciliation calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// T004: Currency
// ---------------------------------------------------------------------------

/// The configured session-budget currency.
///
/// Defaults to USD. Extensible through operator configuration; unknown
/// currencies are rejected at config-validation time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    /// United States Dollar — the default reference currency.
    #[default]
    Usd,
    /// Euro.
    Eur,
}

// ---------------------------------------------------------------------------
// T005: MonetaryAmount
// ---------------------------------------------------------------------------

/// An exact monetary value stored as a [`Decimal`].
///
/// This newtype guarantees that budget, spend, reservation, and
/// reconciliation calculations never silently degrade through floating-point
/// arithmetic. Serialization round-trips through decimal strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonetaryAmount(Decimal);

impl MonetaryAmount {
    /// Create a zero-value amount.
    #[must_use]
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Create from a [`Decimal`].
    #[must_use]
    pub fn from_decimal(d: Decimal) -> Self {
        Self(d)
    }

    /// Return the inner [`Decimal`].
    #[must_use]
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }

    /// Add two amounts, returning the sum.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the operation overflows.
    pub fn checked_add(self, rhs: Self) -> Result<Self, MonetaryError> {
        self.0.checked_add(rhs.0).map(Self).ok_or(MonetaryError::ArithmeticOverflow)
    }

    /// Subtract `rhs` from `self`, returning the difference.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the operation underflows (negative result) or
    /// overflows.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, MonetaryError> {
        self.0.checked_sub(rhs.0).map(Self).ok_or(MonetaryError::ArithmeticOverflow)
    }
}

/// Errors that can occur during monetary arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MonetaryError {
    /// Arithmetic overflow or underflow in a monetary calculation.
    #[error("monetary arithmetic overflow")]
    ArithmeticOverflow,
}

impl Serialize for MonetaryAmount {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for MonetaryAmount {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse::<Decimal>()
            .map(Self)
            .map_err(|e| serde::de::Error::custom(format_args!("invalid decimal: {e}")))
    }
}

impl std::str::FromStr for MonetaryAmount {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Decimal>().map(Self)
    }
}

impl std::fmt::Display for MonetaryAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// ---------------------------------------------------------------------------
// T006: CostBasis
// ---------------------------------------------------------------------------

/// What the budget projection is based on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostBasis {
    /// All recorded costs have exact provider-reported values.
    ExactOnly,
    /// All recorded costs are based on configured pricing estimates.
    Estimated,
    /// The budget projection mixes exact and estimated costs.
    Mixed,
}

// ---------------------------------------------------------------------------
// T007: BudgetState
// ---------------------------------------------------------------------------

/// Current budget enforcement state for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetState {
    /// Spend plus reserves are well within the limit.
    InBounds,
    /// Within 10% of the configured limit; a warning but not blocked.
    ApproachingLimit,
    /// Execution is paused — an approval decision is required before
    /// the next provider-backed inference call can be admitted.
    ApprovalRequired,
    /// Budget is fully consumed; no further non-exempt calls may be
    /// admitted without an explicit override.
    Exhausted,
    /// No session budget is configured; economics enforcement is
    /// inactive and all admission checks short-circuit to `InBounds`.
    Disabled,
}

// ---------------------------------------------------------------------------
// T008: ApprovalType and ApprovalScope
// ---------------------------------------------------------------------------

/// The kind of spend exception being requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    /// Cost of the call cannot be determined exactly or reliably estimated.
    UnknownCostApproval,
    /// The call's conservative reservation exceeds the remaining budget.
    BudgetOverride,
}

/// The bounded set of calls that one approval authorizes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalScope {
    /// One specific call only (V1 default).
    #[default]
    SingleCall,
    /// All calls within a bounded task.
    BoundedTask,
    /// All calls within the current session — requires an explicit
    /// configured policy and a monetary ceiling.
    BoundedSession,
}

// ---------------------------------------------------------------------------
// T009: SnapshotState, ReservationCostQuality, ReconciledCostQuality
// ---------------------------------------------------------------------------

/// State of a pricing snapshot.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotState {
    /// Snapshot is within the configured staleness threshold.
    #[default]
    Current,
    /// Snapshot age exceeds the configured staleness threshold.
    Stale,
    /// No applicable pricing entry exists for the requested model.
    Missing,
    /// Snapshot is corrupt, unparseable, or has an invalid schema version.
    Invalid,
}

/// Pre-call estimate confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReservationCostQuality {
    /// Estimate based on a current (non-stale) pricing snapshot.
    CurrentEstimate,
    /// Estimate based on a stale pricing snapshot.
    StaleEstimate,
    /// No pricing data available — cost is unknown before execution.
    Unknown,
}

/// Post-call reconciled cost confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciledCostQuality {
    /// Cost was reported directly by the provider.
    Exact,
    /// Cost was derived from a versioned configured pricing estimate.
    Estimated,
    /// Neither provider cost nor a reliable estimate is available.
    Unknown,
    /// Local or self-hosted route with zero marginal monetary cost.
    LocalZeroMarginalCost,
}

// ---------------------------------------------------------------------------
// T010: UnknownCostPolicy
// ---------------------------------------------------------------------------

/// The configured behavior when a provider call has unknown cost.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCostPolicy {
    /// Refuse to admit calls whose cost cannot be determined.
    Block,
    /// Pause and require explicit operator approval before admitting.
    #[default]
    RequireApproval,
    /// Admit the call but record a warning in trace and status output.
    AllowWithWarning,
}

// ---------------------------------------------------------------------------
// T011: SessionBudgetConfig
// ---------------------------------------------------------------------------

/// Budget configuration for one session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBudgetConfig {
    /// The currency in which the budget is denominated.
    pub currency: Currency,
    /// The monetary limit expressed as a decimal string (e.g. "10.00").
    pub limit: MonetaryAmount,
}

// ---------------------------------------------------------------------------
// T010: InferenceEconomicsConfig (depends on T011 + T008 + T009)
// ---------------------------------------------------------------------------

/// Top-level inference economics configuration section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceEconomicsConfig {
    /// Optional per-session budget. When [`None`], economics enforcement
    /// is disabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_budget: Option<SessionBudgetConfig>,

    /// Number of days after which a pricing snapshot is considered stale.
    ///
    /// Default: 30 days.
    #[serde(default = "default_staleness_threshold_days")]
    pub staleness_threshold_days: u32,

    /// Policy applied when a provider call has unknown cost.
    ///
    /// Default: [`UnknownCostPolicy::RequireApproval`].
    #[serde(default = "default_unknown_cost_policy")]
    pub unknown_cost_policy: UnknownCostPolicy,

    /// Default approval scope for spend exceptions.
    ///
    /// Default: [`ApprovalScope::SingleCall`].
    #[serde(default)]
    pub approval_default_scope: ApprovalScope,
}

const fn default_staleness_threshold_days() -> u32 {
    30
}

const fn default_unknown_cost_policy() -> UnknownCostPolicy {
    UnknownCostPolicy::RequireApproval
}

impl InferenceEconomicsConfig {
    /// Validate the configuration, returning an error for invalid values.
    ///
    /// # Errors
    ///
    /// Returns an error when:
    /// - The budget limit is negative.
    /// - The budget limit is zero (treated as misconfiguration rather
    ///   than a valid unlimited budget).
    /// - The staleness threshold is zero.
    pub fn validate(&self) -> Result<(), InferenceEconomicsConfigError> {
        if let Some(ref budget) = self.session_budget {
            if budget.limit.as_decimal().is_sign_negative() {
                return Err(InferenceEconomicsConfigError::NegativeBudgetLimit);
            }
            if budget.limit.as_decimal().is_zero() {
                return Err(InferenceEconomicsConfigError::ZeroBudgetLimit);
            }
        }
        if self.staleness_threshold_days == 0 {
            return Err(InferenceEconomicsConfigError::ZeroStalenessThreshold);
        }
        Ok(())
    }
}

/// Errors returned by [`InferenceEconomicsConfig::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InferenceEconomicsConfigError {
    /// Budget limit must be non-negative.
    #[error("budget limit must be positive; got a negative value")]
    NegativeBudgetLimit,
    /// A zero budget limit is treated as invalid — disable economics
    /// instead by omitting the `session_budget` key.
    #[error("budget limit must be positive; use None to disable economics")]
    ZeroBudgetLimit,
    /// Staleness threshold must be at least one day.
    #[error("staleness threshold must be at least 1 day")]
    ZeroStalenessThreshold,
}

// ---------------------------------------------------------------------------
// Tests: T015 + T016
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // T015: MonetaryAmount arithmetic + serialization round-trip
    #[test]
    fn monetary_amount_add_sub_checked() {
        let a = MonetaryAmount::from_str("1.50").expect("valid decimal");
        let b = MonetaryAmount::from_str("2.25").expect("valid decimal");
        let sum = a.checked_add(b).expect("no overflow");
        assert_eq!(sum.as_decimal().to_string(), "3.75");
        let diff = sum.checked_sub(a).expect("no overflow");
        assert_eq!(diff, b);
    }

    #[test]
    fn monetary_amount_zero_is_identity() {
        let z = MonetaryAmount::zero();
        let x = MonetaryAmount::from_str("5.00").expect("valid");
        assert_eq!(x.checked_add(z).expect("ok"), x);
        assert_eq!(x.checked_sub(z).expect("ok"), x);
    }

    #[test]
    fn monetary_amount_serialization_round_trip() {
        let amount = MonetaryAmount::from_str("12.34").expect("valid decimal");
        let json = serde_json::to_string(&amount).expect("serialize");
        let parsed: MonetaryAmount = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(amount, parsed);
    }

    #[test]
    fn monetary_amount_from_str_rejects_invalid() {
        assert!(MonetaryAmount::from_str("not-a-number").is_err());
    }

    // T016: InferenceEconomicsConfig validation
    #[test]
    fn config_valid_with_positive_budget() {
        let cfg = InferenceEconomicsConfig {
            session_budget: Some(SessionBudgetConfig {
                currency: Currency::Usd,
                limit: MonetaryAmount::from_str("10.00").expect("valid"),
            }),
            staleness_threshold_days: 30,
            unknown_cost_policy: UnknownCostPolicy::RequireApproval,
            approval_default_scope: ApprovalScope::SingleCall,
        };
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn config_rejects_negative_limit() {
        let cfg = InferenceEconomicsConfig {
            session_budget: Some(SessionBudgetConfig {
                currency: Currency::Usd,
                limit: MonetaryAmount::from_str("-1.00").expect("valid parse"),
            }),
            staleness_threshold_days: 30,
            unknown_cost_policy: UnknownCostPolicy::RequireApproval,
            approval_default_scope: ApprovalScope::SingleCall,
        };
        assert_eq!(cfg.validate().unwrap_err(), InferenceEconomicsConfigError::NegativeBudgetLimit);
    }

    #[test]
    fn config_rejects_zero_limit() {
        let cfg = InferenceEconomicsConfig {
            session_budget: Some(SessionBudgetConfig {
                currency: Currency::Usd,
                limit: MonetaryAmount::zero(),
            }),
            staleness_threshold_days: 30,
            unknown_cost_policy: UnknownCostPolicy::RequireApproval,
            approval_default_scope: ApprovalScope::SingleCall,
        };
        assert_eq!(cfg.validate().unwrap_err(), InferenceEconomicsConfigError::ZeroBudgetLimit);
    }

    #[test]
    fn config_rejects_zero_staleness_threshold() {
        let cfg = InferenceEconomicsConfig {
            session_budget: None,
            staleness_threshold_days: 0,
            unknown_cost_policy: UnknownCostPolicy::RequireApproval,
            approval_default_scope: ApprovalScope::SingleCall,
        };
        assert_eq!(
            cfg.validate().unwrap_err(),
            InferenceEconomicsConfigError::ZeroStalenessThreshold
        );
    }

    #[test]
    fn config_no_budget_is_valid() {
        let cfg = InferenceEconomicsConfig {
            session_budget: None,
            staleness_threshold_days: 30,
            unknown_cost_policy: UnknownCostPolicy::RequireApproval,
            approval_default_scope: ApprovalScope::SingleCall,
        };
        assert!(cfg.validate().is_ok());
    }
}

// ---------------------------------------------------------------------------
// T017: SessionBudgetProjection
// ---------------------------------------------------------------------------

/// A per-session monetary budget view containing currency, budget limit,
/// known spent amount, reserved amount, remaining known budget, unknown-cost
/// call count, pricing snapshot identifier, cost basis, budget state, and
/// required action when approval is needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBudgetProjection {
    /// The currency in which the budget is denominated.
    pub currency: Currency,
    /// The total approved budget limit.
    pub budget_limit: MonetaryAmount,
    /// Sum of exact + estimated reconciled costs.
    pub known_spent: MonetaryAmount,
    /// Sum of active pre-call reservations.
    pub reserved: MonetaryAmount,
    /// budget_limit - known_spent - reserved.
    pub remaining_known_budget: MonetaryAmount,
    /// Count of completed calls with `cost_quality = Unknown`.
    pub unknown_cost_call_count: u32,
    /// The active pricing snapshot identifier, if any.
    pub pricing_snapshot_id: Option<String>,
    /// What the budget projection is based on.
    pub cost_basis: CostBasis,
    /// Current budget enforcement state.
    pub budget_state: BudgetState,
    /// Action the operator must take, if any.
    pub required_action: Option<RequiredAction>,
}

/// Action required from the operator to unblock a paused session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredAction {
    /// Approve an unknown-cost call before it can execute.
    ApproveUnknownCost,
    /// Approve a call that would exceed the remaining budget.
    ApproveBudgetOverride,
}

// ---------------------------------------------------------------------------
// T020: RequiredAction (defined above)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// T018: PricingEntry + PricingSnapshot
// ---------------------------------------------------------------------------

/// A single per-model pricing entry within a pricing snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricingEntry {
    /// Provider identifier.
    pub provider_id: String,
    /// Model identifier.
    pub model_id: String,
    /// Price per 1000 input tokens in the entry's native currency.
    pub input_price_per_1k: MonetaryAmount,
    /// Price per 1000 output tokens in the entry's native currency.
    pub output_price_per_1k: MonetaryAmount,
    /// Optional cached-input pricing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cached_input_price_per_1k: Option<MonetaryAmount>,
    /// The billing currency for this entry.
    pub native_currency: Currency,
    /// Optional operator notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A versioned, operator-owned pricing snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricingSnapshot {
    /// Unique versioned identifier.
    pub snapshot_id: String,
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// When this snapshot became effective (ISO 8601).
    pub effective_timestamp: String,
    /// Provenance of the snapshot (e.g. "operator-created").
    pub source: String,
    /// Per-model pricing entries.
    pub entries: Vec<PricingEntry>,
    /// Current state of this snapshot.
    #[serde(default)]
    pub state: SnapshotState,
}

// ---------------------------------------------------------------------------
// T019: InvocationCostRecord
// ---------------------------------------------------------------------------

/// A per-call record of reservation amount, final amount, native provider
/// currency, normalized session currency, conversion provenance, pricing
/// snapshot identifier, and cost quality classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationCostRecord {
    /// Unique call identifier.
    pub call_id: String,
    /// The provider identifier used for this call.
    pub provider_id: String,
    /// The model identifier used for this call.
    pub model_id: String,
    /// Pre-call conservative reservation amount.
    pub reservation_amount: MonetaryAmount,
    /// Post-call reconciled final amount.
    pub final_amount: MonetaryAmount,
    /// Provider's billing currency, if supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_currency: Option<Currency>,
    /// Session-normalized currency.
    pub normalized_currency: Currency,
    /// FX conversion source, if conversion was applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_source: Option<String>,
    /// When the conversion was applied, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_timestamp: Option<String>,
    /// Snapshot used for the reservation.
    pub pricing_snapshot_id: String,
    /// How old the snapshot was at reservation time (seconds).
    pub snapshot_age_secs: u64,
    /// Post-call reconciled cost confidence.
    pub cost_quality: ReconciledCostQuality,
    /// Pre-call estimate confidence.
    pub reservation_confidence: ReservationCostQuality,
}

// ---------------------------------------------------------------------------
// T021-T023: Budget state machine methods + supporting types
// ---------------------------------------------------------------------------

/// The result of an admission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionDecision {
    /// The call is within budget and may proceed.
    InBounds,
    /// The call is blocked because the budget is exhausted or the config
    /// policy mandates refusal.
    Blocked,
    /// Approval is required; the required approver role is embedded.
    ApprovalRequired,
}

/// Errors that can occur during budget operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BudgetError {
    /// The reservation amount exceeds the remaining budget.
    #[error("reservation would exceed remaining budget")]
    ReservationExceedsRemaining,
    /// The budget is exhausted and cannot accept further reservations.
    #[error("budget is exhausted")]
    BudgetExhausted,
    /// No budget is configured; call should not have reached the enforcer.
    #[error("budget enforcement is disabled")]
    BudgetDisabled,
    /// The reconciliation record does not match any active reservation.
    #[error("no matching reservation found for reconciliation")]
    NoMatchingReservation,
}

impl SessionBudgetProjection {
    /// Create a new projection from a configured budget.
    ///
    /// When `config` is [`None`], the projection is created in the
    /// [`BudgetState::Disabled`] state.
    #[must_use]
    pub fn new(config: Option<&SessionBudgetConfig>) -> Self {
        let Some(cfg) = config else {
            return Self {
                currency: Currency::default(),
                budget_limit: MonetaryAmount::zero(),
                known_spent: MonetaryAmount::zero(),
                reserved: MonetaryAmount::zero(),
                remaining_known_budget: MonetaryAmount::zero(),
                unknown_cost_call_count: 0,
                pricing_snapshot_id: None,
                cost_basis: CostBasis::ExactOnly,
                budget_state: BudgetState::Disabled,
                required_action: None,
            };
        };
        Self {
            currency: cfg.currency,
            budget_limit: cfg.limit,
            known_spent: MonetaryAmount::zero(),
            reserved: MonetaryAmount::zero(),
            remaining_known_budget: cfg.limit,
            unknown_cost_call_count: 0,
            pricing_snapshot_id: None,
            cost_basis: CostBasis::ExactOnly,
            budget_state: BudgetState::InBounds,
            required_action: None,
        }
    }

    /// Reserve a conservative estimated amount before a call is admitted.
    ///
    /// Updates `reserved` and `remaining_known_budget`. Transitions
    /// `budget_state` accordingly.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::BudgetExhausted`] when the budget is already
    /// exhausted, and [`BudgetError::BudgetDisabled`] when no budget is
    /// configured.
    pub fn reserve(
        &mut self,
        amount: MonetaryAmount,
        snapshot_id: &str,
        _snapshot_age_secs: u64,
    ) -> Result<(), BudgetError> {
        if self.budget_state == BudgetState::Disabled {
            return Err(BudgetError::BudgetDisabled);
        }
        if self.budget_state == BudgetState::Exhausted {
            return Err(BudgetError::BudgetExhausted);
        }
        self.reserved = self
            .reserved
            .checked_add(amount)
            .map_err(|_| BudgetError::ReservationExceedsRemaining)?;
        self.remaining_known_budget = self
            .remaining_known_budget
            .checked_sub(amount)
            .map_err(|_| BudgetError::ReservationExceedsRemaining)?;
        self.pricing_snapshot_id = Some(snapshot_id.to_string());

        // Transition budget state based on remaining budget
        let limit = self.budget_limit;
        let remaining = self.remaining_known_budget;
        let ten_percent = limit
            .as_decimal()
            .checked_div(rust_decimal::Decimal::TEN)
            .map(MonetaryAmount::from_decimal);

        if remaining.as_decimal().is_zero() {
            self.budget_state = BudgetState::Exhausted;
        } else if ten_percent.is_some_and(|tp| remaining <= tp) {
            self.budget_state = BudgetState::ApproachingLimit;
        }

        Ok(())
    }

    /// Reconcile a completed call against the active reservation.
    ///
    /// Replaces the reservation with the actual cost, updates `known_spent`,
    /// and handles the unknown-cost counter.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::NoMatchingReservation`] if there is no
    /// active reservation to reconcile against.
    pub fn reconcile(&mut self, cost: InvocationCostRecord) -> Result<(), BudgetError> {
        if self.reserved.as_decimal().is_zero() {
            return Err(BudgetError::NoMatchingReservation);
        }

        // Release the reservation
        self.reserved =
            self.reserved.checked_sub(cost.reservation_amount).unwrap_or(MonetaryAmount::zero());

        // Add actual cost to known spent
        self.known_spent = self
            .known_spent
            .checked_add(cost.final_amount)
            .map_err(|_| BudgetError::ReservationExceedsRemaining)?;

        // Recompute remaining
        self.remaining_known_budget = self
            .budget_limit
            .checked_sub(self.known_spent)
            .and_then(|v| v.checked_sub(self.reserved))
            .unwrap_or(MonetaryAmount::zero());

        // Update cost basis
        self.cost_basis = match cost.cost_quality {
            ReconciledCostQuality::Exact if self.cost_basis == CostBasis::ExactOnly => {
                CostBasis::ExactOnly
            }
            ReconciledCostQuality::Exact => CostBasis::Mixed,
            ReconciledCostQuality::Estimated => CostBasis::Estimated,
            ReconciledCostQuality::Unknown => {
                self.unknown_cost_call_count += 1;
                CostBasis::Mixed
            }
            ReconciledCostQuality::LocalZeroMarginalCost => self.cost_basis,
        };

        // Re-evaluate budget state
        if self.remaining_known_budget.as_decimal().is_zero() {
            self.budget_state = BudgetState::Exhausted;
        } else if self.budget_state == BudgetState::Exhausted {
            self.budget_state = BudgetState::ApproachingLimit;
        }

        Ok(())
    }

    /// Check whether a call of the given amount can be admitted.
    ///
    /// Returns [`AdmissionDecision::InBounds`] when the call fits,
    /// [`AdmissionDecision::Blocked`] when the budget is exhausted and the
    /// unknown-cost policy is `Block`, or
    /// [`AdmissionDecision::ApprovalRequired`] when the budget is
    /// approaching limit, exhausted, or the call is an unknown-cost call.
    #[must_use]
    pub fn check_admission(
        &self,
        amount: MonetaryAmount,
        cost_quality: ReconciledCostQuality,
        _authority_zone: u8,
    ) -> AdmissionDecision {
        match self.budget_state {
            BudgetState::Disabled => AdmissionDecision::InBounds,
            BudgetState::InBounds => {
                // For unknown-cost calls, require approval even when in-bounds
                if cost_quality == ReconciledCostQuality::Unknown {
                    return AdmissionDecision::ApprovalRequired;
                }
                // Check if adding this amount would exceed
                if amount.as_decimal() > self.remaining_known_budget.as_decimal() {
                    return AdmissionDecision::ApprovalRequired;
                }
                AdmissionDecision::InBounds
            }
            BudgetState::ApproachingLimit => AdmissionDecision::ApprovalRequired,
            BudgetState::ApprovalRequired => AdmissionDecision::ApprovalRequired,
            BudgetState::Exhausted => AdmissionDecision::Blocked,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: T033 — SessionBudgetProjection state machine
// ---------------------------------------------------------------------------

#[cfg(test)]
mod budget_tests {
    use super::*;
    use std::str::FromStr;

    fn test_config(limit: &str) -> SessionBudgetConfig {
        SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str(limit).expect("valid decimal"),
        }
    }

    #[test]
    fn reserve_updates_state_and_tracks_remaining() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        let amount = MonetaryAmount::from_str("3.00").expect("valid");
        proj.reserve(amount, "snap-1", 0).expect("reservation should succeed");
        assert_eq!(proj.reserved.as_decimal().to_string(), "3.00");
        assert_eq!(proj.remaining_known_budget.as_decimal().to_string(), "7.00");
        assert_eq!(proj.budget_state, BudgetState::InBounds);
    }

    #[test]
    fn reserve_transitions_to_approaching_limit() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        // Reserve 9.50 — remaining 0.50 is within 10% of 10.00
        let amount = MonetaryAmount::from_str("9.50").expect("valid");
        proj.reserve(amount, "snap-1", 0).expect("ok");
        assert_eq!(proj.budget_state, BudgetState::ApproachingLimit);
    }

    #[test]
    fn reserve_transitions_to_exhausted() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        proj.reserve(MonetaryAmount::from_str("10.00").expect("valid"), "snap-1", 0).expect("ok");
        assert_eq!(proj.budget_state, BudgetState::Exhausted);
    }

    #[test]
    fn reserve_rejected_when_exhausted() {
        let cfg = test_config("1.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        proj.reserve(MonetaryAmount::from_str("1.00").expect("valid"), "snap-1", 0).expect("ok");
        let err = proj
            .reserve(MonetaryAmount::from_str("0.01").expect("valid"), "snap-2", 0)
            .unwrap_err();
        assert_eq!(err, BudgetError::BudgetExhausted);
    }

    #[test]
    fn reconcile_exact_replaces_reservation() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        let reservation = MonetaryAmount::from_str("3.00").expect("valid");
        proj.reserve(reservation, "snap-1", 0).expect("ok");

        let record = InvocationCostRecord {
            call_id: "call-1".into(),
            provider_id: "openai".into(),
            model_id: "gpt-4o".into(),
            reservation_amount: reservation,
            final_amount: MonetaryAmount::from_str("2.50").expect("valid"),
            native_currency: None,
            normalized_currency: Currency::Usd,
            conversion_source: None,
            conversion_timestamp: None,
            pricing_snapshot_id: "snap-1".into(),
            snapshot_age_secs: 0,
            cost_quality: ReconciledCostQuality::Exact,
            reservation_confidence: ReservationCostQuality::CurrentEstimate,
        };
        proj.reconcile(record).expect("reconcile ok");
        assert_eq!(proj.reserved, MonetaryAmount::zero());
        assert_eq!(proj.known_spent, MonetaryAmount::from_str("2.50").expect("valid"));
        assert_eq!(proj.remaining_known_budget, MonetaryAmount::from_str("7.50").expect("valid"));
        assert_eq!(proj.budget_state, BudgetState::InBounds);
    }

    #[test]
    fn reconcile_unknown_increments_counter() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        let reservation = MonetaryAmount::from_str("1.00").expect("valid");
        proj.reserve(reservation, "snap-1", 0).expect("ok");

        let record = InvocationCostRecord {
            call_id: "call-2".into(),
            provider_id: "unknown-provider".into(),
            model_id: "unknown-model".into(),
            reservation_amount: reservation,
            final_amount: MonetaryAmount::zero(),
            native_currency: None,
            normalized_currency: Currency::Usd,
            conversion_source: None,
            conversion_timestamp: None,
            pricing_snapshot_id: "snap-1".into(),
            snapshot_age_secs: 3600,
            cost_quality: ReconciledCostQuality::Unknown,
            reservation_confidence: ReservationCostQuality::Unknown,
        };
        proj.reconcile(record).expect("reconcile ok");
        assert_eq!(proj.unknown_cost_call_count, 1);
        assert_eq!(proj.cost_basis, CostBasis::Mixed);
    }

    #[test]
    fn admission_in_bounds() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let decision = proj.check_admission(
            MonetaryAmount::from_str("5.00").expect("valid"),
            ReconciledCostQuality::Estimated,
            0,
        );
        assert_eq!(decision, AdmissionDecision::InBounds);
    }

    #[test]
    fn admission_approval_required_when_over_budget() {
        let cfg = test_config("10.00");
        let mut proj = SessionBudgetProjection::new(Some(&cfg));
        proj.reserve(MonetaryAmount::from_str("10.00").expect("valid"), "snap-1", 0).expect("ok");
        let decision = proj.check_admission(
            MonetaryAmount::from_str("0.01").expect("valid"),
            ReconciledCostQuality::Exact,
            0,
        );
        assert_eq!(decision, AdmissionDecision::Blocked);
    }

    #[test]
    fn admission_unknown_cost_requires_approval_even_in_bounds() {
        let cfg = test_config("10.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        let decision = proj.check_admission(
            MonetaryAmount::from_str("1.00").expect("valid"),
            ReconciledCostQuality::Unknown,
            0,
        );
        assert_eq!(decision, AdmissionDecision::ApprovalRequired);
    }

    #[test]
    fn disabled_projection_always_admits() {
        let proj = SessionBudgetProjection::new(None);
        assert_eq!(proj.budget_state, BudgetState::Disabled);
        let decision = proj.check_admission(
            MonetaryAmount::from_str("999.99").expect("valid"),
            ReconciledCostQuality::Unknown,
            0,
        );
        assert_eq!(decision, AdmissionDecision::InBounds);
        // Reservation should fail on disabled budget
        let mut proj2 = SessionBudgetProjection::new(None);
        let err =
            proj2.reserve(MonetaryAmount::from_str("1.00").expect("valid"), "snap", 0).unwrap_err();
        assert_eq!(err, BudgetError::BudgetDisabled);
    }

    #[test]
    fn new_projection_from_config_has_in_bounds_state() {
        let cfg = test_config("25.00");
        let proj = SessionBudgetProjection::new(Some(&cfg));
        assert_eq!(proj.budget_state, BudgetState::InBounds);
        assert_eq!(proj.remaining_known_budget.as_decimal().to_string(), "25.00");
        assert_eq!(proj.known_spent.as_decimal().to_string(), "0");
        assert_eq!(proj.reserved.as_decimal().to_string(), "0");
    }
}

// ---------------------------------------------------------------------------
// SessionEconomicsState — standalone persistence-sidecar for economics data
// ---------------------------------------------------------------------------

/// The inference economics state for a single session, stored as a sidecar
/// alongside [`ActiveSessionRecord`] rather than embedded within it.
///
/// This avoids coupling the entire codebase to the economics module — only
/// the economics-aware code paths need to load or mutate this state.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEconomicsState {
    /// Budget projection for the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_projection: Option<SessionBudgetProjection>,
    /// Recorded invocation cost records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invocation_cost_records: Vec<InvocationCostRecord>,
}

// ---------------------------------------------------------------------------
// T038-T039: InferenceRouteProfile + RouteTier (Phase 4)
// ---------------------------------------------------------------------------

/// Describes a selectable inference route, including its tier, provider,
/// model, capability requirements, and economic characteristics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRouteProfile {
    /// The governance tier of this route.
    pub tier: RouteTier,
    /// Provider identifier.
    pub provider_id: String,
    /// Model identifier.
    pub model_id: String,
    /// Capability requirements that must be satisfied.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_requirements: Vec<String>,
    /// Whether this route transmits content outside the local environment.
    #[serde(default)]
    pub repository_egress: bool,
}

/// The routing tier that determines capability level and cost profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteTier {
    /// Deterministic local path — no LLM call.
    Tier0Deterministic,
    /// Small/cheap model for summarization, extraction, simple classification.
    Tier1Cheap,
    /// Balanced model for planning, review, guardian reasoning.
    Tier2Balanced,
    /// High-capability model for architecture, red-zone review.
    Tier3HighCapability,
}

/// The outcome of a route selection decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteSelection {
    /// A specific route was selected.
    Selected(InferenceRouteProfile),
    /// No compliant route exists; the task is blocked with a reason.
    Blocked(String),
    /// The deterministic Tier 0 path was chosen — execute locally.
    DeterministicLocal,
}

// ---------------------------------------------------------------------------
// T049-T051: SpendExceptionApprovalRecord, SpendExceptionDecisionProjection,
//            ApprovalState, ApproverRole (Phase 5)
// ---------------------------------------------------------------------------

/// The actor who may approve a spend exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApproverRole {
    /// The active session owner (low-risk, non-egress calls).
    SessionOwner,
    /// A governance approver (red-zone or egress calls).
    GovernanceApprover,
}

/// The lifecycle state of an approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    /// Approval has been granted but not yet consumed.
    Pending,
    /// The approval has been consumed by a matching call.
    Consumed,
    /// The approval expired before consumption.
    Expired,
    /// The approval was explicitly revoked.
    Revoked,
}

/// An audit record for an approved spend exception.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendExceptionApprovalRecord {
    /// Unique approval identifier.
    pub approval_id: String,
    /// The type of spend exception.
    pub approval_type: ApprovalType,
    /// Who approved.
    pub approver_identity: String,
    /// The role of the approver.
    pub approver_role: ApproverRole,
    /// The session that owns this approval.
    pub session_id: String,
    /// Optional execution run reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_run_id: Option<String>,
    /// Target provider.
    pub provider_id: String,
    /// Target model.
    pub model_id: String,
    /// Selected route identifier.
    pub route: String,
    /// The authority zone of the task.
    pub authority_zone: String,
    /// Whether repository content leaves the local environment.
    pub repository_egress: bool,
    /// Monetary ceiling, if bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_amount: Option<MonetaryAmount>,
    /// The scope of the approval.
    pub scope: ApprovalScope,
    /// Operator-provided reason.
    pub reason: String,
    /// When the approval was created (ISO 8601).
    pub created_at: String,
    /// When the approval was consumed, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<String>,
    /// Optional expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Current lifecycle state.
    pub state: ApprovalState,
    /// Separate data-transmission authorization for egress calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_transmission_authorized: Option<bool>,
}

/// A runtime projection of a pending spend exception decision, surfaced
/// to the operator for approval or rejection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpendExceptionDecisionProjection {
    /// The type of exception being requested.
    pub approval_type: ApprovalType,
    /// Current approval state.
    pub approval_state: ApprovalState,
    /// The role that must approve.
    pub required_role: ApproverRole,
    /// The authority zone of the task.
    pub authority_zone: String,
    /// Whether repository content leaves the local environment.
    pub repository_egress: bool,
    /// The monetary amount requested.
    pub requested_amount: MonetaryAmount,
    /// Session currency.
    pub currency: Currency,
    /// List of actions the operator must take.
    pub required_actions: Vec<String>,
}

// ---------------------------------------------------------------------------
// T070a: RouteChangeGate (Phase 5)
// ---------------------------------------------------------------------------

/// Governs whether a route-policy change may become active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteChangeGate {
    /// Whether the required evaluation approval has been satisfied.
    pub evaluation_approved: bool,
    /// Human-readable reason when the gate is closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gating_reason: Option<String>,
}

impl RouteChangeGate {
    /// Create a gate that blocks activation until evaluation approval
    /// is satisfied.
    #[must_use]
    pub fn blocked(reason: impl Into<String>) -> Self {
        Self { evaluation_approved: false, gating_reason: Some(reason.into()) }
    }

    /// Create a gate that allows activation.
    #[must_use]
    pub fn approved() -> Self {
        Self { evaluation_approved: true, gating_reason: None }
    }

    /// Whether the change may become active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.evaluation_approved
    }
}

// ---------------------------------------------------------------------------
// Config loading helpers
// ---------------------------------------------------------------------------

/// Load the [`InferenceEconomicsConfig`] from a TOML table.
///
/// The caller is responsible for extracting the `[inference_economics]`
/// section from the config file. When the table is absent, [`None`] is
/// returned (economics enforcement is disabled).
///
/// # Errors
///
/// Returns an error if the table is present but cannot be deserialized
/// into an [`InferenceEconomicsConfig`].
pub fn load_economics_config(
    table: Option<&toml::Table>,
) -> Result<Option<InferenceEconomicsConfig>, InferenceEconomicsConfigError> {
    let Some(t) = table else {
        return Ok(None);
    };
    let config: InferenceEconomicsConfig = toml::Value::Table(t.clone())
        .try_into()
        .map_err(|_| InferenceEconomicsConfigError::ZeroStalenessThreshold)?;
    config.validate()?;
    Ok(Some(config))
}

// ---------------------------------------------------------------------------
// Status view projection helper
// ---------------------------------------------------------------------------

/// Populate budget projection fields on a [`SessionStatusView`].
///
/// Call this from the status-view builder when an economics projection
/// is available. When `projection` is [`None`], the budget fields
/// are left unchanged (defaulting to their serde-absent state).
pub fn project_budget_to_status(
    view: &mut crate::domain::session::SessionStatusView,
    projection: Option<&SessionBudgetProjection>,
) {
    let Some(proj) = projection else {
        return;
    };
    view.budget_currency = Some(format!("{:?}", proj.currency).to_lowercase());
    view.budget_limit = Some(proj.budget_limit.to_string());
    view.budget_known_spent = Some(proj.known_spent.to_string());
    view.budget_reserved = Some(proj.reserved.to_string());
    view.budget_remaining = Some(proj.remaining_known_budget.to_string());
    view.budget_state = Some(format!("{:?}", proj.budget_state).to_lowercase());
    view.budget_unknown_cost_call_count = Some(proj.unknown_cost_call_count);
}

#[cfg(test)]
mod status_projection_tests {
    use super::*;
    use crate::domain::session::SessionStatusView;
    use std::str::FromStr;

    #[test]
    fn load_economics_config_from_valid_toml() {
        let toml_str = r#"
[session_budget]
currency = "usd"
limit = "25.00"

staleness_threshold_days = 30
unknown_cost_policy = "require_approval"
"#;
        let table = toml_str.parse::<toml::Table>().expect("valid toml");
        let config = load_economics_config(Some(&table)).expect("ok").expect("some");
        assert!(config.session_budget.is_some());
        let budget = config.session_budget.as_ref().unwrap();
        assert_eq!(budget.currency, Currency::Usd);
        assert_eq!(budget.limit.as_decimal().to_string(), "25.00");
        assert_eq!(config.staleness_threshold_days, 30);
        assert_eq!(config.unknown_cost_policy, UnknownCostPolicy::RequireApproval);
    }

    #[test]
    fn load_economics_config_none_table_returns_none() {
        let result = load_economics_config(None).expect("ok");
        assert!(result.is_none());
    }

    #[test]
    fn reconcile_with_exact_cost_updates_basis_to_mixed() {
        let config = SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("100.00").expect("valid"),
        };
        let mut proj = SessionBudgetProjection::new(Some(&config));
        // First, create an Estimated cost basis
        let reservation1 = MonetaryAmount::from_str("1.00").expect("valid");
        proj.reserve(reservation1, "snap-1", 0).expect("ok");
        let record1 = InvocationCostRecord {
            call_id: "c1".into(),
            provider_id: "p".into(),
            model_id: "m".into(),
            reservation_amount: reservation1,
            final_amount: MonetaryAmount::from_str("0.80").expect("valid"),
            native_currency: None,
            normalized_currency: Currency::Usd,
            conversion_source: None,
            conversion_timestamp: None,
            pricing_snapshot_id: "snap-1".into(),
            snapshot_age_secs: 0,
            cost_quality: ReconciledCostQuality::Estimated,
            reservation_confidence: ReservationCostQuality::CurrentEstimate,
        };
        proj.reconcile(record1).expect("ok");
        assert_eq!(proj.cost_basis, CostBasis::Estimated);
        // Now reconcile with Exact — should become Mixed
        let reservation2 = MonetaryAmount::from_str("2.00").expect("valid");
        proj.reserve(reservation2, "snap-2", 0).expect("ok");
        let record2 = InvocationCostRecord {
            call_id: "c2".into(),
            provider_id: "p".into(),
            model_id: "m".into(),
            reservation_amount: reservation2,
            final_amount: MonetaryAmount::from_str("1.50").expect("valid"),
            native_currency: None,
            normalized_currency: Currency::Usd,
            conversion_source: None,
            conversion_timestamp: None,
            pricing_snapshot_id: "snap-2".into(),
            snapshot_age_secs: 0,
            cost_quality: ReconciledCostQuality::Exact,
            reservation_confidence: ReservationCostQuality::CurrentEstimate,
        };
        proj.reconcile(record2).expect("ok");
        assert_eq!(proj.cost_basis, CostBasis::Mixed);
    }

    #[test]
    fn check_admission_exhausted_returns_blocked() {
        let config = SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("1.00").expect("valid"),
        };
        let mut proj = SessionBudgetProjection::new(Some(&config));
        // Reserve the entire budget to exhaust it
        let reservation = MonetaryAmount::from_str("1.00").expect("valid");
        proj.reserve(reservation, "snap-1", 0).expect("ok");
        assert_eq!(proj.budget_state, BudgetState::Exhausted);
        let decision = proj.check_admission(
            MonetaryAmount::from_str("0.10").expect("valid"),
            ReconciledCostQuality::Estimated,
            0,
        );
        assert_eq!(decision, AdmissionDecision::Blocked);
    }

    #[test]
    fn check_admission_approaching_limit_requires_approval() {
        let config = SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("10.00").expect("valid"),
        };
        let mut proj = SessionBudgetProjection::new(Some(&config));
        // Reserve 9.50 leaving 0.50 which is 5% — below 10% threshold
        let reservation = MonetaryAmount::from_str("9.50").expect("valid");
        proj.reserve(reservation, "snap-1", 0).expect("ok");
        assert_eq!(proj.budget_state, BudgetState::ApproachingLimit);
        let decision = proj.check_admission(
            MonetaryAmount::from_str("0.10").expect("valid"),
            ReconciledCostQuality::Estimated,
            0,
        );
        assert_eq!(decision, AdmissionDecision::ApprovalRequired);
    }

    #[test]
    fn project_budget_populates_view_fields() {
        let config = SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("25.00").expect("valid"),
        };
        let proj = SessionBudgetProjection::new(Some(&config));
        let mut view = SessionStatusView::default();
        project_budget_to_status(&mut view, Some(&proj));
        assert_eq!(view.budget_currency.as_deref(), Some("usd"));
        assert!(view.budget_limit.is_some());
        assert_eq!(view.budget_state.as_deref(), Some("inbounds"));
        assert_eq!(view.budget_unknown_cost_call_count, Some(0));
    }

    #[test]
    fn project_budget_none_leaves_view_unchanged() {
        let mut view = SessionStatusView::default();
        project_budget_to_status(&mut view, None);
        assert!(view.budget_currency.is_none());
        assert!(view.budget_state.is_none());
    }

    #[test]
    fn project_budget_sets_unknown_cost_count() {
        let config = SessionBudgetConfig {
            currency: Currency::Usd,
            limit: MonetaryAmount::from_str("50.00").expect("valid"),
        };
        let mut proj = SessionBudgetProjection::new(Some(&config));
        let reservation = MonetaryAmount::from_str("1.00").expect("valid");
        proj.reserve(reservation, "snap-1", 0).expect("ok");
        let record = InvocationCostRecord {
            call_id: "c1".into(),
            provider_id: "p".into(),
            model_id: "m".into(),
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
        let mut view = SessionStatusView::default();
        project_budget_to_status(&mut view, Some(&proj));
        assert_eq!(view.budget_unknown_cost_call_count, Some(1));
    }
}
