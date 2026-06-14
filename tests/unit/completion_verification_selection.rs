use boundline::domain::completion_verification::{
    ClaimConfirmationContext, ClaimConfirmationRequirement, ClaimInferenceConfidence,
    CompletionClaim, CompletionClaimKind, CompletionClaimSource, CompletionVerificationScope,
    ProofCommandRule, claim_confirmation_requirement, infer_completion_claim,
    proof_rules_for_validation_command, select_proof_command,
};

#[test]
fn proof_selection_prefers_the_narrowest_rule() {
    let claim = CompletionClaim {
        claim_id: "claim-1".to_string(),
        kind: CompletionClaimKind::TestsPass,
        scope: CompletionVerificationScope::Task,
        source: CompletionClaimSource::ExplicitMetadata,
        confidence: None,
        summary: "tests pass".to_string(),
        supporting_signals: Vec::new(),
    };
    let wide_rule = ProofCommandRule {
        claim_kind: CompletionClaimKind::TestsPass,
        command_ref: "cargo-test-all".to_string(),
        command_line: "cargo test".to_string(),
        selection_reason: "broad regression proof".to_string(),
        breadth_rank: 10,
        fully_covers_claim: true,
        documentation_relevant: false,
    };
    let narrow_rule = ProofCommandRule {
        claim_kind: CompletionClaimKind::TestsPass,
        command_ref: "cargo-test-unit".to_string(),
        command_line: "cargo test --test unit".to_string(),
        selection_reason: "narrowest falsifying proof".to_string(),
        breadth_rank: 1,
        fully_covers_claim: true,
        documentation_relevant: false,
    };

    let selection = select_proof_command(&claim, &[wide_rule, narrow_rule]);
    assert!(selection.is_ok());
    let selected = selection.ok().flatten();
    assert!(selected.is_some());
    let selected = selected.unwrap_or_else(|| unreachable!());
    assert_eq!(selected.command_ref, "cargo-test-unit");
    assert_eq!(selected.command_line, "cargo test --test unit");
}

#[test]
fn medium_confidence_claim_needs_confirmation_when_policy_forbids_silent_progress() {
    let requirement = claim_confirmation_requirement(
        Some(ClaimInferenceConfidence::Medium),
        &ClaimConfirmationContext {
            multiple_plausible_claims: false,
            proof_only_partially_covers_claim: false,
            risky_surface: false,
            conflicting_claim_signals: false,
            policy_allows_medium_without_confirmation: false,
        },
    );

    assert_eq!(requirement, ClaimConfirmationRequirement::ConfirmationRequired);
}

#[test]
fn conflicting_claim_signals_require_clarification() {
    let requirement = claim_confirmation_requirement(
        Some(ClaimInferenceConfidence::High),
        &ClaimConfirmationContext {
            multiple_plausible_claims: false,
            proof_only_partially_covers_claim: false,
            risky_surface: false,
            conflicting_claim_signals: true,
            policy_allows_medium_without_confirmation: true,
        },
    );

    assert_eq!(requirement, ClaimConfirmationRequirement::ClarificationRequired);
}

#[test]
fn inference_prefers_test_claim_when_validation_command_and_context_point_to_tests() {
    let claim = infer_completion_claim(
        "claim-1",
        "fix the failing add test",
        None,
        &["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
        Some("cargo test --quiet"),
    );

    assert!(claim.is_ok());
    let claim = claim.ok();
    assert_eq!(claim.as_ref().map(|value| value.kind), Some(CompletionClaimKind::TestsPass));
    assert!(
        claim
            .as_ref()
            .map(|value| {
                value
                    .supporting_signals
                    .iter()
                    .any(|signal| signal == "validation_command:cargo test --quiet")
            })
            .unwrap_or(false)
    );
}

#[test]
fn proof_rules_include_build_claim_for_build_commands() {
    let rules = proof_rules_for_validation_command(
        "workspace_validation_command",
        "cargo build --workspace",
    );

    assert!(rules.iter().any(|rule| {
        rule.claim_kind == CompletionClaimKind::BuildClean
            && rule.command_ref == "workspace_validation_command"
            && rule.fully_covers_claim
    }));
}

#[test]
fn medium_confidence_claim_can_proceed_when_policy_allows_it() {
    let requirement = claim_confirmation_requirement(
        Some(ClaimInferenceConfidence::Medium),
        &ClaimConfirmationContext {
            multiple_plausible_claims: false,
            proof_only_partially_covers_claim: false,
            risky_surface: false,
            conflicting_claim_signals: false,
            policy_allows_medium_without_confirmation: true,
        },
    );

    assert_eq!(requirement, ClaimConfirmationRequirement::SilentAllowed);
}
