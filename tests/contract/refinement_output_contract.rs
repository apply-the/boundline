//! Contract tests for round packet schema validation.
//!
//! Validates that round packets are correctly serialized/deserialized,
//! schema version is enforced, stop reason vocabulary is respected,
//! confidence invariants hold, and all required fields are present.

use boundline::domain::refinement::{
    Confidence, ConfidenceAdjustment, ROUND_PACKET_SCHEMA_VERSION, RoundPacket, StopReason,
};

fn make_valid_packet(round: u32, candidate_ref: &str) -> RoundPacket {
    RoundPacket {
        schema_version: ROUND_PACKET_SCHEMA_VERSION.to_string(),
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        round,
        candidate_ref: candidate_ref.into(),
        findings: vec![],
        requested_deltas: vec![],
        applied_deltas: vec![],
        critic_confidence: Confidence::Sufficient,
        effective_confidence: Confidence::Sufficient,
        confidence_adjustment_reason: None,
        stop_reason: None,
    }
}

#[test]
fn round_packet_all_required_fields_present() {
    let packet = make_valid_packet(1, "trace://plan-candidate-1");
    let json = packet.to_json_value().unwrap();
    assert!(json.get("schema_version").is_some());
    assert!(json.get("profile").is_some());
    assert!(json.get("stage").is_some());
    assert!(json.get("round").is_some());
    assert!(json.get("candidate_ref").is_some());
    assert!(json.get("findings").is_some());
    assert!(json.get("requested_deltas").is_some());
    assert!(json.get("applied_deltas").is_some());
    assert!(json.get("critic_confidence").is_some());
    assert!(json.get("effective_confidence").is_some());
    assert!(json.get("stop_reason").is_some());
}

#[test]
fn round_packet_no_inline_artifact_content() {
    let packet = make_valid_packet(1, "trace://plan-candidate-1");
    let json = packet.to_json_value().unwrap();
    let json_str = json.to_string();
    // The candidate_ref must be a trace:// reference, not inline content.
    assert!(json_str.contains("trace://plan-candidate-1"));
    // No large inline plan content should appear in the packet.
    assert!(!json_str.contains("plan_text"));
}

#[test]
fn round_packet_confidence_downgrade_only() {
    let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
    packet.critic_confidence = Confidence::Low;
    packet.effective_confidence = Confidence::High;
    assert!(packet.validate(None).is_err());
}

#[test]
fn round_packet_confidence_adjustment_visible() {
    let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
    packet.critic_confidence = Confidence::High;
    packet.effective_confidence = Confidence::Sufficient;
    packet.confidence_adjustment_reason = Some(ConfidenceAdjustment::BlockersUnresolved);
    let json = packet.to_json_value().unwrap();
    assert_eq!(
        json.get("confidence_adjustment_reason").and_then(|v| v.as_str()),
        Some("blockers_unresolved")
    );
}

#[test]
fn round_packet_stop_reason_vocabulary() {
    for reason in &[
        StopReason::NoMaterialDelta,
        StopReason::RoundLimitExhausted,
        StopReason::TimeLimitExhausted,
        StopReason::EmptyCandidate,
        StopReason::UnresolvedBlocker,
        StopReason::ProviderFailure,
        StopReason::MalformedPacket,
        StopReason::InvalidDelta,
        StopReason::InvalidConfiguration,
    ] {
        let json = serde_json::to_string(reason).unwrap();
        let back: StopReason = serde_json::from_str(&json).unwrap();
        assert_eq!(*reason, back);
    }
}

#[test]
fn round_packet_null_stop_reason_means_continuing() {
    let packet = make_valid_packet(1, "trace://plan-candidate-1");
    assert!(packet.stop_reason.is_none());
    let json = packet.to_json_value().unwrap();
    assert!(json.get("stop_reason").unwrap().is_null());
}

#[test]
fn round_packet_schema_version_mismatch_rejected() {
    let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
    packet.schema_version = "2.0".into();
    assert!(packet.validate(None).is_err());
}

#[test]
fn round_packet_malformed_version_rejected() {
    let mut packet = make_valid_packet(1, "trace://plan-candidate-1");
    packet.schema_version = "abc".into();
    assert!(packet.validate(None).is_err());
}
