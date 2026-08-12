//! Frozen Tier 0-3 challenge rules reject self-verification and silent degradation.

use boundline_core::transaction::challenge::{
    ChallengeContext, ChallengeDecision, ChallengePolicy, ChallengeTier, NamedRiskOverride,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn require(condition: bool, message: &str) -> TestResult {
    if condition { Ok(()) } else { Err(std::io::Error::other(message).into()) }
}

fn complete(tier: ChallengeTier) -> ChallengeContext {
    ChallengeContext::builder(tier)
        .deterministic_checks(true)
        .separate_invocation(true)
        .fresh_claim_matched_evidence(true)
        .distinct_lineage(true)
        .independent_context(true)
        .material_side_effect_approval(true)
        .different_provider_family(true)
        .named_human_approval(true)
        .explicit_risk_acceptance(true)
        .verified_recovery_path(true)
        .build()
}

#[test]
fn every_tier_enforces_its_cumulative_requirements() -> TestResult {
    for tier in
        [ChallengeTier::Tier0, ChallengeTier::Tier1, ChallengeTier::Tier2, ChallengeTier::Tier3]
    {
        require(
            matches!(ChallengePolicy::evaluate(&complete(tier)), ChallengeDecision::Satisfied),
            "complete tier rejected",
        )?;
    }
    require(
        matches!(
            ChallengePolicy::evaluate(
                &ChallengeContext::builder(ChallengeTier::Tier1).deterministic_checks(true).build()
            ),
            ChallengeDecision::Missing { .. }
        ),
        "Tier 1 admitted without separate fresh verification",
    )?;
    require(
        matches!(
            ChallengePolicy::evaluate(
                &ChallengeContext::builder(ChallengeTier::Tier3).deterministic_checks(true).build()
            ),
            ChallengeDecision::Missing { automatic_override_permitted: false, .. }
        ),
        "Tier 3 missing challenge allowed override",
    )
}

#[test]
fn shared_prompt_conversation_or_conclusion_is_not_independent() -> TestResult {
    for context in [
        complete(ChallengeTier::Tier2).with_shared_prompt(),
        complete(ChallengeTier::Tier2).with_shared_conversation(),
        complete(ChallengeTier::Tier2).with_shared_conclusion(),
    ] {
        require(
            matches!(ChallengePolicy::evaluate(&context), ChallengeDecision::NotIndependent),
            "shared implementer context counted as independent",
        )?;
    }
    Ok(())
}

#[test]
fn same_lineage_tier_two_requires_named_owned_override() -> TestResult {
    let degraded = complete(ChallengeTier::Tier2).with_distinct_lineage(false);
    require(
        matches!(ChallengePolicy::evaluate(&degraded), ChallengeDecision::Missing { .. }),
        "same lineage silently admitted",
    )?;
    let override_record =
        NamedRiskOverride::new("risk-owner", "only qualified offline verifier", "RISK-42")?;
    require(
        matches!(
            ChallengePolicy::evaluate(&degraded.with_override(override_record)),
            ChallengeDecision::SatisfiedWithOverride { .. }
        ),
        "named override rejected",
    )
}
