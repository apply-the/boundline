use std::error::Error;
use std::fs;
use std::path::Path;

use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::domain::governance::CanonMode;
use boundline::domain::reasoning::{
    IndependenceFloor, ParticipantRoleDefinition, ReasoningAdjudicationMode, ReasoningBudget,
    ReasoningDegradationPolicy, ReasoningParticipantRoleKind, ReasoningProfileDefinition,
    ReasoningProfileFamily, ReasoningProfileId, ReasoningRoutePreference,
};
use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::trace::TraceEventType;
use boundline::{ConfigFile, FileConfigStore, ModelRoute, RoutingConfig, RuntimeKind};
use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_security_assessment_workspace, terminal_text,
};

const SELF_CONSISTENCY_PATH_ROLE_ID: &str = "self_consistency_path";
const REVIEWER_PRIMARY_ROLE_ID: &str = "reviewer_primary";
const REVIEWER_SECONDARY_ROLE_ID: &str = "reviewer_secondary";
const CRITIC_ROLE_ID: &str = "critic";
const REVISER_ROLE_ID: &str = "reviser";
const LATEST_SELF_CONSISTENCY_PARTICIPANT_LINE: &str =
    "latest_reasoning_participants: self_consistency_path=verification:";
const SELF_CONSISTENCY_PARTICIPANT_LINE: &str =
    "reasoning_participants: self_consistency_path=verification:";

fn bootstrap_bug_fix(workspace: &Path) {
    assert_eq!(
        run_boundline_in(workspace, &["goal", "--goal", "Fix the failing checkout flow"])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(run_boundline_in(workspace, &["flow", "bug-fix"]).status.code(), Some(0));
    assert_eq!(run_boundline_in(workspace, &["plan"]).status.code(), Some(0));
}

fn verification_reasoning_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::BoundedSelfConsistency,
        family: ReasoningProfileFamily::SelfConsistency,
        allowed_stages: vec![CanonMode::Verification, CanonMode::SecurityAssessment],
        limits: ReasoningBudget {
            max_participants: 1,
            max_branches: 2,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 2048,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![ParticipantRoleDefinition {
            role_id: SELF_CONSISTENCY_PATH_ROLE_ID.to_string(),
            role_kind: ReasoningParticipantRoleKind::IndependentPath,
            preferred_slot: ReasoningRoutePreference::Verification,
            independence_requirements: IndependenceFloor {
                route_distinct: false,
                provider_distinct: false,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 1,
            },
            required: true,
        }],
        adjudication_mode: ReasoningAdjudicationMode::None,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some(
                "rerun verification with bounded self-consistency disabled".to_string(),
            ),
        },
    }
}

fn independent_pair_review_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::IndependentPairReview,
        family: ReasoningProfileFamily::BlindReview,
        allowed_stages: vec![CanonMode::Verification, CanonMode::SecurityAssessment],
        limits: ReasoningBudget {
            max_participants: 2,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 8_000,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![
            ParticipantRoleDefinition {
                role_id: REVIEWER_PRIMARY_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::BlindReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: true,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
            ParticipantRoleDefinition {
                role_id: REVIEWER_SECONDARY_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::BlindReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: true,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
        ],
        adjudication_mode: ReasoningAdjudicationMode::GovernanceReview,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some(
                "configure distinct reviewer routes for reviewer_primary and reviewer_secondary"
                    .to_string(),
            ),
        },
    }
}

fn heterogeneous_security_review_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::HeterogeneousSecurityReview,
        family: ReasoningProfileFamily::HeterogeneousReview,
        allowed_stages: vec![CanonMode::Verification, CanonMode::SecurityAssessment],
        limits: ReasoningBudget {
            max_participants: 2,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 8_000,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![
            ParticipantRoleDefinition {
                role_id: REVIEWER_PRIMARY_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::HeterogeneousReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: false,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
            ParticipantRoleDefinition {
                role_id: REVIEWER_SECONDARY_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::HeterogeneousReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: false,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
        ],
        adjudication_mode: ReasoningAdjudicationMode::None,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some(
                "configure heterogeneous reviewer routes with distinct provider families"
                    .to_string(),
            ),
        },
    }
}

fn bounded_reflexion_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::BoundedReflexion,
        family: ReasoningProfileFamily::Reflexion,
        allowed_stages: vec![CanonMode::Verification, CanonMode::SecurityAssessment],
        limits: ReasoningBudget {
            max_participants: 2,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 1,
            max_calls: 2,
            max_tokens: 4_096,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![
            ParticipantRoleDefinition {
                role_id: CRITIC_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::Critic,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: false,
                    provider_distinct: false,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 1,
                },
                required: true,
            },
            ParticipantRoleDefinition {
                role_id: REVISER_ROLE_ID.to_string(),
                role_kind: ReasoningParticipantRoleKind::Reviser,
                preferred_slot: ReasoningRoutePreference::Verification,
                independence_requirements: IndependenceFloor {
                    route_distinct: false,
                    provider_distinct: false,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 1,
                },
                required: true,
            },
        ],
        adjudication_mode: ReasoningAdjudicationMode::None,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some(
                "rerun bounded reflexion after restoring critique or revise capacity".to_string(),
            ),
        },
    }
}

fn write_selected_reasoning_profile_into_execution_profile(
    workspace: &Path,
    reasoning_profile: &ReasoningProfileDefinition,
) {
    let path = workspace.join(".boundline/execution.json");
    let mut profile: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    profile["governance"]["stages"][0]["reasoning_profile"] =
        serde_json::to_value(reasoning_profile).unwrap();
    fs::write(path, serde_json::to_string_pretty(&profile).unwrap()).unwrap();
}

fn write_reasoning_profile_into_execution_profile(workspace: &Path) {
    write_selected_reasoning_profile_into_execution_profile(
        workspace,
        &verification_reasoning_profile(),
    );
}

fn save_distinct_review_routing(workspace: &Path) {
    FileConfigStore::for_workspace(workspace)
        .save_local(&ConfigFile {
            version: 1,
            routing: RoutingConfig {
                reviewer_roles: std::collections::BTreeMap::from([
                    (
                        REVIEWER_PRIMARY_ROLE_ID.to_string(),
                        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
                    ),
                    (
                        REVIEWER_SECONDARY_ROLE_ID.to_string(),
                        ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4.1".to_string() },
                    ),
                ]),
                ..RoutingConfig::default()
            },
            canon: None,
        })
        .unwrap();
}

fn load_session_record(workspace: &Path) -> Result<ActiveSessionRecord, Box<dyn Error>> {
    let session_path = workspace.join(".boundline/session.json");
    Ok(serde_json::from_str(&fs::read_to_string(session_path)?)?)
}

fn load_latest_trace(
    workspace: &Path,
) -> Result<boundline::domain::trace::ExecutionTrace, Box<dyn Error>> {
    let record = load_session_record(workspace)?;
    let trace_ref = record
        .latest_trace_ref
        .ok_or_else(|| std::io::Error::other("trace reference should exist"))?;
    let trace_path = workspace.join(trace_ref);
    Ok(FileTraceStore::for_workspace(workspace).load(&trace_path)?)
}

#[test]
fn verification_stage_activation_surfaces_reasoning_profile_lines() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-profile-activation");
    write_reasoning_profile_into_execution_profile(&workspace);
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");
    assert!(
        run_text.contains("governance_started: bug-fix:verify (security-assessment)"),
        "{run_text}"
    );

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(
        status_text.contains("latest_reasoning_profile_id: bounded_self_consistency"),
        "{status_text}"
    );
    assert!(
        status_text.contains("latest_reasoning_profile_stage: bug-fix:verify"),
        "{status_text}"
    );

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(
        inspect_text.contains("reasoning_profile_id: bounded_self_consistency"),
        "{inspect_text}"
    );
}

#[test]
fn status_and_inspect_surface_reasoning_profile_summary_details() {
    let workspace = temp_canon_security_assessment_workspace("boundline-reasoning-profile-summary");
    write_reasoning_profile_into_execution_profile(&workspace);
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    for needle in [
        "latest_reasoning_profile_status: active",
        LATEST_SELF_CONSISTENCY_PARTICIPANT_LINE,
        "latest_reasoning_posture_contract: governed_reasoning_posture_v1",
        "latest_reasoning_confidence_level: high",
        "latest_reasoning_confidence_summary: reasoning independence passed under the Canon-governed challenge posture",
    ] {
        assert!(status_text.contains(needle), "{status_text}");
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    for needle in [
        "reasoning_profile_status: active",
        SELF_CONSISTENCY_PARTICIPANT_LINE,
        "reasoning_posture_contract: governed_reasoning_posture_v1",
        "reasoning_confidence_level: high",
        "reasoning_confidence_summary: reasoning independence passed under the Canon-governed challenge posture",
    ] {
        assert!(inspect_text.contains(needle), "{inspect_text}");
    }
}

#[test]
fn verification_stage_trace_records_reasoning_activation_and_confidence_events()
-> Result<(), Box<dyn Error>> {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-trace-activation");
    write_reasoning_profile_into_execution_profile(&workspace);
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let trace = load_latest_trace(&workspace)?;

    assert!(
        trace
            .events
            .iter()
            .any(|event| event.event_type == TraceEventType::ReasoningProfileActivated),
        "expected reasoning activation event in persisted trace"
    );
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.event_type == TraceEventType::ReasoningConfidenceRecorded),
        "expected reasoning confidence event in persisted trace"
    );

    Ok(())
}

#[test]
fn verification_stage_without_reasoning_profile_preserves_unchanged_projection_path() {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-profile-no-profile");
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(!status_text.contains("latest_reasoning_profile_id:"), "{status_text}");

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    assert!(!inspect_text.contains("reasoning_profile_id:"), "{inspect_text}");
}

#[test]
fn independent_pair_review_positive_path_surfaces_terminal_reasoning_outcome()
-> Result<(), Box<dyn Error>> {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-profile-independent-pair");
    save_distinct_review_routing(&workspace);
    write_selected_reasoning_profile_into_execution_profile(
        &workspace,
        &independent_pair_review_profile(),
    );
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    for needle in [
        "latest_reasoning_profile_id: independent_pair_review",
        "latest_reasoning_profile_status: completed",
        "latest_reasoning_independence_result: passed",
        "latest_reasoning_outcome: adjudicated",
        "latest_reasoning_confidence_level: high",
    ] {
        assert!(status_text.contains(needle), "{status_text}");
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    for needle in [
        "reasoning_profile_id: independent_pair_review",
        "reasoning_profile_status: completed",
        "reasoning_independence_result: passed",
        "reasoning_outcome: adjudicated",
    ] {
        assert!(inspect_text.contains(needle), "{inspect_text}");
    }

    let trace = load_latest_trace(&workspace)?;
    assert!(
        trace
            .events
            .iter()
            .any(|event| event.event_type == TraceEventType::ReasoningAdjudicationRecorded),
        "expected reasoning adjudication event in persisted trace"
    );

    Ok(())
}

#[test]
fn heterogeneous_security_review_positive_path_surfaces_terminal_reasoning_outcome()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_canon_security_assessment_workspace(
        "boundline-reasoning-profile-heterogeneous-security-review",
    );
    save_distinct_review_routing(&workspace);
    write_selected_reasoning_profile_into_execution_profile(
        &workspace,
        &heterogeneous_security_review_profile(),
    );
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    for needle in [
        "latest_reasoning_profile_id: heterogeneous_security_review",
        "latest_reasoning_profile_status: completed",
        "latest_reasoning_independence_result: passed",
        "latest_reasoning_outcome: converged",
        "latest_reasoning_confidence_level: high",
    ] {
        assert!(status_text.contains(needle), "{status_text}");
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    for needle in [
        "reasoning_profile_id: heterogeneous_security_review",
        "reasoning_profile_status: completed",
        "reasoning_independence_result: passed",
        "reasoning_outcome: converged",
    ] {
        assert!(inspect_text.contains(needle), "{inspect_text}");
    }

    Ok(())
}

#[test]
fn bounded_reflexion_positive_path_surfaces_terminal_reasoning_outcome()
-> Result<(), Box<dyn Error>> {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-profile-bounded-reflexion");
    write_selected_reasoning_profile_into_execution_profile(
        &workspace,
        &bounded_reflexion_profile(),
    );
    bootstrap_bug_fix(&workspace);

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(0), "{run_text}");

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    for needle in [
        "latest_reasoning_profile_id: bounded_reflexion",
        "latest_reasoning_profile_status: completed",
        "latest_reasoning_independence_result: passed",
        "latest_reasoning_outcome: converged",
        "latest_reasoning_confidence_level: high",
    ] {
        assert!(status_text.contains(needle), "{status_text}");
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(0), "{inspect_text}");
    for needle in [
        "reasoning_profile_id: bounded_reflexion",
        "reasoning_profile_status: completed",
        "reasoning_independence_result: passed",
        "reasoning_outcome: converged",
    ] {
        assert!(inspect_text.contains(needle), "{inspect_text}");
    }

    let trace = load_latest_trace(&workspace)?;
    assert!(
        trace.events.iter().any(|event| {
            event.event_type == TraceEventType::ReasoningReflexionRevisionCompleted
        }),
        "expected reasoning reflexion revision event in persisted trace"
    );

    Ok(())
}
