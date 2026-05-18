use std::error::Error;
use std::fs;

use boundline::domain::trace::TraceEventType;

const TRACE_CONTRACT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/specs/061-reasoning-profile-contracts/contracts/reasoning-trace-contract.md"
);
const EXPECTED_EVENT_FAMILIES: [(&str, TraceEventType, &str); 10] = [
    (
        "profile activation",
        TraceEventType::ReasoningProfileActivated,
        "reasoning_profile_activated",
    ),
    (
        "participant started",
        TraceEventType::ReasoningParticipantStarted,
        "reasoning_participant_started",
    ),
    (
        "participant completed",
        TraceEventType::ReasoningParticipantCompleted,
        "reasoning_participant_completed",
    ),
    (
        "convergence or disagreement recorded",
        TraceEventType::ReasoningDisagreementRecorded,
        "reasoning_disagreement_recorded",
    ),
    (
        "debate round completed",
        TraceEventType::ReasoningDebateRoundCompleted,
        "reasoning_debate_round_completed",
    ),
    (
        "reflexion revision completed",
        TraceEventType::ReasoningReflexionRevisionCompleted,
        "reasoning_reflexion_revision_completed",
    ),
    (
        "adjudication recorded",
        TraceEventType::ReasoningAdjudicationRecorded,
        "reasoning_adjudication_recorded",
    ),
    (
        "confidence contribution recorded",
        TraceEventType::ReasoningConfidenceRecorded,
        "reasoning_confidence_recorded",
    ),
    (
        "profile blocked or escalated",
        TraceEventType::ReasoningProfileBlocked,
        "reasoning_profile_blocked",
    ),
    (
        "profile blocked or escalated",
        TraceEventType::ReasoningProfileEscalated,
        "reasoning_profile_escalated",
    ),
];
const EXPECTED_PAYLOAD_FIELDS: [&str; 12] = [
    "profile_id",
    "stage",
    "activation_id",
    "participant_id",
    "role",
    "iteration_kind",
    "iteration_index",
    "outcome_kind",
    "independence_result",
    "confidence_level",
    "summary",
    "next_action",
];

fn read_text(path: &str) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

fn assert_contains(document: &str, expected: &str, context: &str) {
    assert!(document.contains(expected), "{context}: expected to find `{expected}`");
}

#[test]
fn reasoning_trace_contract_lists_required_event_families() -> Result<(), Box<dyn Error>> {
    let contract = read_text(TRACE_CONTRACT_PATH)?;

    for (document_phrase, event_type, serialized_name) in EXPECTED_EVENT_FAMILIES {
        assert_contains(
            &contract,
            document_phrase,
            "reasoning trace contract should publish the required event families",
        );
        assert_eq!(serde_json::to_string(&event_type)?, format!("\"{serialized_name}\""));
    }

    Ok(())
}

#[test]
fn reasoning_trace_contract_lists_required_payload_fields() -> Result<(), Box<dyn Error>> {
    let contract = read_text(TRACE_CONTRACT_PATH)?;

    for field in EXPECTED_PAYLOAD_FIELDS {
        assert_contains(
            &contract,
            field,
            "reasoning trace contract should publish the additive payload fields",
        );
    }

    Ok(())
}
