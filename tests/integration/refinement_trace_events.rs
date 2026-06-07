//! Integration tests for refinement trace event emission.
//!
//! Validates that a 3-round refinement loop produces exactly 3 trace
//! events of type RefinementRoundCompleted, each with all required
//! fields present and no inline artifact content.

use std::time::Duration;

use boundline::domain::refinement::{RefinementProfile, RefinementRoles, RoundPacket};
use boundline::orchestrator::refinement::{ResolvedRefinementRoles, execute_refinement_loop};

fn make_profile(max_rounds: u32) -> RefinementProfile {
    RefinementProfile {
        profile: "plan_refinement".into(),
        stage: "plan".into(),
        enabled: true,
        max_rounds,
        max_elapsed_time_seconds: 300,
        roles: RefinementRoles {
            planner_provider_id: "t".into(),
            critic_provider_id: "t".into(),
            finalizer_provider_id: "t".into(),
        },
    }
}

fn make_roles() -> ResolvedRefinementRoles {
    ResolvedRefinementRoles {
        planner_provider_id: "t".into(),
        critic_provider_id: "t".into(),
        finalizer_provider_id: "t".into(),
    }
}

#[test]
fn three_round_loop_produces_three_trace_events() {
    let profile = make_profile(3);
    let roles = make_roles();
    let mut packets: Vec<RoundPacket> = vec![];
    let _outcome = execute_refinement_loop(&profile, &roles, Duration::from_secs(300), |p| {
        packets.push(p.clone());
    })
    .unwrap();
    assert_eq!(packets.len(), 3);
}

#[test]
fn each_trace_event_has_all_required_fields() {
    let profile = make_profile(1);
    let roles = make_roles();
    let mut packets: Vec<RoundPacket> = vec![];
    execute_refinement_loop(&profile, &roles, Duration::from_secs(300), |p| {
        packets.push(p.clone());
    })
    .unwrap();
    for packet in &packets {
        assert!(!packet.schema_version.is_empty());
        assert!(!packet.profile.is_empty());
        assert!(!packet.stage.is_empty());
        assert!(packet.round >= 1);
        assert!(packet.candidate_ref.starts_with("trace://"));
        // All required fields are present by construction (RoundPacket struct).
    }
}

#[test]
fn no_trace_event_contains_inline_artifact_content() {
    let profile = make_profile(2);
    let roles = make_roles();
    let mut packets: Vec<RoundPacket> = vec![];
    execute_refinement_loop(&profile, &roles, Duration::from_secs(300), |p| {
        packets.push(p.clone());
    })
    .unwrap();
    for packet in &packets {
        let json = packet.to_json_value().unwrap();
        let json_str = json.to_string();
        assert!(!json_str.contains("plan_text"));
        assert!(!json_str.contains("inline_content"));
    }
}
