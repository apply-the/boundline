//! Deterministic Tier 0-3 challenge admission encodes independence and authority.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Frozen challenge tiers ordered by governed risk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeTier {
    /// Deterministic checks only.
    Tier0,
    /// Separate fresh claim-matched verification.
    Tier1,
    /// Independent lineage and context.
    Tier2,
    /// Cross-provider or qualified human challenge.
    Tier3,
}

/// Auditable exception for Tier 2 same-lineage degradation only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedRiskOverride {
    risk_owner: String,
    justification: String,
    reference: String,
}

impl NamedRiskOverride {
    /// Creates an override only when owner, rationale, and audit reference are named.
    pub fn new(
        owner: impl Into<String>,
        justification: impl Into<String>,
        reference: impl Into<String>,
    ) -> Result<Self, ChallengeError> {
        let value = Self {
            risk_owner: owner.into(),
            justification: justification.into(),
            reference: reference.into(),
        };
        if [&value.risk_owner, &value.justification, &value.reference]
            .iter()
            .any(|field| field.trim().is_empty())
        {
            Err(ChallengeError::IncompleteOverride)
        } else {
            Ok(value)
        }
    }
}

/// Evidence supplied to the deterministic challenge policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChallengeContext {
    tier: ChallengeTier,
    deterministic_checks: bool,
    separate_invocation: bool,
    fresh_claim_matched_evidence: bool,
    distinct_lineage: bool,
    independent_context: bool,
    material_side_effect_approval: bool,
    different_provider_family: bool,
    qualified_human_challenger: bool,
    named_human_approval: bool,
    explicit_risk_acceptance: bool,
    verified_recovery_path: bool,
    shared_prompt: bool,
    shared_conversation: bool,
    shared_conclusion: bool,
    override_record: Option<NamedRiskOverride>,
}

impl ChallengeContext {
    /// Starts a context with every requirement absent.
    pub fn builder(tier: ChallengeTier) -> ChallengeContextBuilder {
        ChallengeContextBuilder {
            context: Self {
                tier,
                deterministic_checks: false,
                separate_invocation: false,
                fresh_claim_matched_evidence: false,
                distinct_lineage: false,
                independent_context: false,
                material_side_effect_approval: false,
                different_provider_family: false,
                qualified_human_challenger: false,
                named_human_approval: false,
                explicit_risk_acceptance: false,
                verified_recovery_path: false,
                shared_prompt: false,
                shared_conversation: false,
                shared_conclusion: false,
                override_record: None,
            },
        }
    }
    pub fn with_shared_prompt(mut self) -> Self {
        self.shared_prompt = true;
        self
    }
    pub fn with_shared_conversation(mut self) -> Self {
        self.shared_conversation = true;
        self
    }
    pub fn with_shared_conclusion(mut self) -> Self {
        self.shared_conclusion = true;
        self
    }
    pub const fn with_distinct_lineage(mut self, value: bool) -> Self {
        self.distinct_lineage = value;
        self
    }
    pub fn with_override(mut self, value: NamedRiskOverride) -> Self {
        self.override_record = Some(value);
        self
    }
}

/// Builder makes cumulative tier evidence explicit at the call site.
pub struct ChallengeContextBuilder {
    context: ChallengeContext,
}

macro_rules! bool_builder {
    ($($name:ident => $field:ident),+ $(,)?) => {$(
        pub const fn $name(mut self, value: bool) -> Self { self.context.$field = value; self }
    )+};
}

impl ChallengeContextBuilder {
    bool_builder!(
        deterministic_checks => deterministic_checks,
        separate_invocation => separate_invocation,
        fresh_claim_matched_evidence => fresh_claim_matched_evidence,
        distinct_lineage => distinct_lineage,
        independent_context => independent_context,
        material_side_effect_approval => material_side_effect_approval,
        different_provider_family => different_provider_family,
        qualified_human_challenger => qualified_human_challenger,
        named_human_approval => named_human_approval,
        explicit_risk_acceptance => explicit_risk_acceptance,
        verified_recovery_path => verified_recovery_path,
    );
    /// Finishes the immutable evaluation input.
    pub fn build(self) -> ChallengeContext {
        self.context
    }
}

/// Deterministic policy result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeDecision {
    Satisfied,
    SatisfiedWithOverride { risk_owner: String, reference: String },
    Missing { requirements: Vec<String>, automatic_override_permitted: bool },
    NotIndependent,
}

/// Stateless frozen challenge matrix.
pub struct ChallengePolicy;

impl ChallengePolicy {
    /// Evaluates cumulative tier requirements without invoking a reviewer.
    pub fn evaluate(context: &ChallengeContext) -> ChallengeDecision {
        if context.shared_prompt || context.shared_conversation || context.shared_conclusion {
            return ChallengeDecision::NotIndependent;
        }
        let mut missing = Vec::new();
        required(context.deterministic_checks, "deterministic_checks", &mut missing);
        if context.tier != ChallengeTier::Tier0 {
            required(context.separate_invocation, "separate_invocation", &mut missing);
            required(
                context.fresh_claim_matched_evidence,
                "fresh_claim_matched_evidence",
                &mut missing,
            );
        }
        if matches!(context.tier, ChallengeTier::Tier2 | ChallengeTier::Tier3) {
            required(context.independent_context, "independent_context", &mut missing);
            required(
                context.material_side_effect_approval,
                "material_side_effect_approval",
                &mut missing,
            );
            if !context.distinct_lineage {
                if context.tier == ChallengeTier::Tier2
                    && missing.is_empty()
                    && let Some(record) = &context.override_record
                {
                    return ChallengeDecision::SatisfiedWithOverride {
                        risk_owner: record.risk_owner.clone(),
                        reference: record.reference.clone(),
                    };
                }
                missing.push("distinct_lineage".to_owned());
            }
        }
        if context.tier == ChallengeTier::Tier3 {
            required(
                context.different_provider_family || context.qualified_human_challenger,
                "different_provider_or_human_challenger",
                &mut missing,
            );
            required(context.named_human_approval, "named_human_approval", &mut missing);
            required(context.explicit_risk_acceptance, "explicit_risk_acceptance", &mut missing);
            required(context.verified_recovery_path, "verified_recovery_path", &mut missing);
        }
        if missing.is_empty() {
            ChallengeDecision::Satisfied
        } else {
            ChallengeDecision::Missing {
                requirements: missing,
                automatic_override_permitted: context.tier != ChallengeTier::Tier3,
            }
        }
    }
}

/// Challenge record construction errors.
#[derive(Debug, Error)]
pub enum ChallengeError {
    #[error("same-lineage override requires a named owner, justification, and reference")]
    IncompleteOverride,
}

fn required(condition: bool, name: &'static str, missing: &mut Vec<String>) {
    if !condition {
        missing.push(name.to_owned());
    }
}
