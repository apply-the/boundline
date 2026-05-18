use std::error::Error;
use std::fs;

use boundline::domain::reasoning::{
    ReasoningActivationStatus, ReasoningOutcomeKind, ReasoningParticipantRoleKind,
    ReasoningProfileId,
};

const RUNTIME_CONTRACT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/061-reasoning-profile-contracts/contracts/reasoning-profile-runtime-contract.md"
);
const EXPECTED_PROFILE_IDS: [&str; 4] = [
    "bounded_self_consistency",
    "independent_pair_review",
    "heterogeneous_security_review",
    "bounded_reflexion",
];
const EXPECTED_STATUSES: [&str; 8] =
    ["pending", "active", "completed", "degraded", "blocked", "interrupted", "escalated", "failed"];
const EXPECTED_OUTCOME_KINDS: [&str; 8] = [
    "converged",
    "disagreed",
    "adjudicated",
    "degraded",
    "blocked",
    "interrupted",
    "escalated",
    "failed",
];
const EXPECTED_PARTICIPANT_ROLES: [&str; 6] = [
    "independent_path",
    "blind_reviewer",
    "heterogeneous_reviewer",
    "critic",
    "reviser",
    "arbiter",
];
const UNPUBLISHED_PARTICIPANT_ROLE: &str = "observer";

fn read_text(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

fn assert_contains(document: &str, expected: &str, context: &str) {
    assert!(document.contains(expected), "{context}: expected to find `{expected}`");
}

#[test]
fn reasoning_profile_runtime_contract_lists_supported_profile_ids() -> Result<(), Box<dyn Error>> {
    let contract = read_text(RUNTIME_CONTRACT_PATH)?;
    let actual = [
        ReasoningProfileId::BoundedSelfConsistency.as_str(),
        ReasoningProfileId::IndependentPairReview.as_str(),
        ReasoningProfileId::HeterogeneousSecurityReview.as_str(),
        ReasoningProfileId::BoundedReflexion.as_str(),
    ];

    for profile_id in EXPECTED_PROFILE_IDS {
        assert_contains(
            &contract,
            profile_id,
            "runtime contract should publish the supported profile ids",
        );
    }
    assert_eq!(actual, EXPECTED_PROFILE_IDS);

    Ok(())
}

#[test]
fn reasoning_profile_runtime_contract_lists_supported_status_values() -> Result<(), Box<dyn Error>>
{
    let contract = read_text(RUNTIME_CONTRACT_PATH)?;
    let actual = [
        ReasoningActivationStatus::Pending.as_str(),
        ReasoningActivationStatus::Active.as_str(),
        ReasoningActivationStatus::Completed.as_str(),
        ReasoningActivationStatus::Degraded.as_str(),
        ReasoningActivationStatus::Blocked.as_str(),
        ReasoningActivationStatus::Interrupted.as_str(),
        ReasoningActivationStatus::Escalated.as_str(),
        ReasoningActivationStatus::Failed.as_str(),
    ];

    for status in EXPECTED_STATUSES {
        assert_contains(
            &contract,
            status,
            "runtime contract should publish the supported status values",
        );
    }
    assert_eq!(actual, EXPECTED_STATUSES);

    Ok(())
}

#[test]
fn reasoning_profile_runtime_contract_lists_supported_outcome_kinds() -> Result<(), Box<dyn Error>>
{
    let contract = read_text(RUNTIME_CONTRACT_PATH)?;
    let actual = [
        ReasoningOutcomeKind::Converged.as_str(),
        ReasoningOutcomeKind::Disagreed.as_str(),
        ReasoningOutcomeKind::Adjudicated.as_str(),
        ReasoningOutcomeKind::Degraded.as_str(),
        ReasoningOutcomeKind::Blocked.as_str(),
        ReasoningOutcomeKind::Interrupted.as_str(),
        ReasoningOutcomeKind::Escalated.as_str(),
        ReasoningOutcomeKind::Failed.as_str(),
    ];

    for outcome_kind in EXPECTED_OUTCOME_KINDS {
        assert_contains(
            &contract,
            outcome_kind,
            "runtime contract should publish the supported outcome kinds",
        );
    }
    assert_eq!(actual, EXPECTED_OUTCOME_KINDS);

    Ok(())
}

#[test]
fn reasoning_profile_runtime_contract_lists_supported_participant_roles()
-> Result<(), Box<dyn Error>> {
    let contract = read_text(RUNTIME_CONTRACT_PATH)?;
    let actual = [
        ReasoningParticipantRoleKind::IndependentPath.as_str(),
        ReasoningParticipantRoleKind::BlindReviewer.as_str(),
        ReasoningParticipantRoleKind::HeterogeneousReviewer.as_str(),
        ReasoningParticipantRoleKind::Critic.as_str(),
        ReasoningParticipantRoleKind::Reviser.as_str(),
        ReasoningParticipantRoleKind::Arbiter.as_str(),
    ];

    for role in EXPECTED_PARTICIPANT_ROLES {
        assert_contains(
            &contract,
            role,
            "runtime contract should publish the supported participant roles",
        );
    }
    assert!(!contract.contains(UNPUBLISHED_PARTICIPANT_ROLE));
    assert_eq!(actual, EXPECTED_PARTICIPANT_ROLES);
    assert!(
        serde_json::from_str::<ReasoningParticipantRoleKind>("\"observer\"").is_err(),
        "runtime vocabulary should reject unpublished participant roles"
    );

    Ok(())
}
