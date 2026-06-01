use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use boundline::adapters::session_store::{FileSessionStore, SessionStore};
use boundline::adapters::trace_store::{FileTraceStore, TraceStore};
use boundline::domain::governance::CanonMode;
use boundline::domain::reasoning::{
    CanonAdmissionPriority, CanonChallengePostureInput, IndependenceFloor,
    ParticipantRoleDefinition, ProfileActivationRecord, ReasoningActivationStatus,
    ReasoningActivationTrigger, ReasoningAdjudicationMode, ReasoningAdmissionEffect,
    ReasoningBudget, ReasoningCompatibilityWindow, ReasoningConfidenceContribution,
    ReasoningConfidenceLevel, ReasoningDegradationPolicy, ReasoningOutcome, ReasoningOutcomeKind,
    ReasoningParticipantRoleKind, ReasoningProfileDefinition, ReasoningProfileFamily,
    ReasoningProfileId, ReasoningRoutePreference,
};
use boundline::domain::session::ActiveSessionRecord;
use boundline::domain::trace::{ExecutionTrace, TraceEventType};
use boundline::{ConfigFile, FileConfigStore, ModelRoute, RoutingConfig, RuntimeKind};
use serde_json::Value;

use crate::workspace_fixture::{
    run_boundline_in, temp_canon_security_approval_workspace,
    temp_canon_security_assessment_workspace, terminal_text,
};

const REVIEWER_PRIMARY_ROLE_ID: &str = "reviewer_primary";
const REVIEWER_SECONDARY_ROLE_ID: &str = "reviewer_secondary";
const SELF_CONSISTENCY_PATH_ROLE_ID: &str = "self_consistency_path";
const DISTINCT_REVIEWER_ROUTES_NEXT_ACTION: &str =
    "configure distinct reviewer routes for reviewer_primary and reviewer_secondary";
const DRIFTED_REASONING_POSTURE_CONTRACT_LINE: &str = "governed_reasoning_posture_v999";
const DRIFTED_REASONING_SUMMARY: &str = "unsupported Canon reasoning posture contract line governed_reasoning_posture_v999 for the active Boundline/Canon release pair";
const DRIFTED_REASONING_GUIDANCE: &str = "align Canon to governed_reasoning_posture_v1 or update the supported Boundline/Canon release pair before rerunning";

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

fn ensure_success(
    output: &std::process::Output,
    output_text: &str,
    context: &str,
) -> Result<(), Box<dyn Error>> {
    if output.status.code() == Some(0) {
        return Ok(());
    }

    Err(fail(format!("{context} failed: {output_text}")))
}

fn ensure_contains(haystack: &str, needle: &str, context: &str) -> Result<(), Box<dyn Error>> {
    if haystack.contains(needle) {
        return Ok(());
    }

    Err(fail(format!("{context} missing expected text `{needle}` in output:\n{haystack}")))
}

fn bootstrap_bug_fix(workspace: &Path) -> Result<(), Box<dyn Error>> {
    let goal = run_boundline_in(workspace, &["goal", "--goal", "Fix the failing checkout flow"]);
    ensure_success(&goal, &terminal_text(&goal), "goal")?;

    let flow = run_boundline_in(workspace, &["flow", "bug-fix"]);
    ensure_success(&flow, &terminal_text(&flow), "flow")?;

    let plan = run_boundline_in(workspace, &["plan"]);
    ensure_success(&plan, &terminal_text(&plan), "plan")
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
            blocked_next_action: Some(DISTINCT_REVIEWER_ROUTES_NEXT_ACTION.to_string()),
        },
    }
}

fn interruptible_self_consistency_profile() -> ReasoningProfileDefinition {
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
            max_tokens: 2_048,
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

fn write_reasoning_profile_into_execution_profile(
    workspace: &Path,
    reasoning_profile: &ReasoningProfileDefinition,
) -> Result<(), Box<dyn Error>> {
    let path = workspace.join(".boundline/execution.json");
    let profile_text = fs::read_to_string(&path)?;
    let mut profile: Value = serde_json::from_str(&profile_text)?;
    profile["governance"]["stages"][0]["reasoning_profile"] =
        serde_json::to_value(reasoning_profile)?;
    fs::write(path, serde_json::to_string_pretty(&profile)?)?;
    Ok(())
}

fn save_collapsed_review_routing(workspace: &Path) -> Result<(), Box<dyn Error>> {
    FileConfigStore::for_workspace(workspace).save_local(&ConfigFile {
        version: 1,
        routing: RoutingConfig {
            review: Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() }),
            ..RoutingConfig::default()
        },
        canon: None,
        adapter: None,
    })?;

    Ok(())
}

fn load_session_record(workspace: &Path) -> Result<ActiveSessionRecord, Box<dyn Error>> {
    Ok(FileSessionStore::for_workspace(workspace)
        .load()?
        .ok_or_else(|| io::Error::other("session record should exist"))?)
}

fn load_latest_trace(workspace: &Path) -> Result<ExecutionTrace, Box<dyn Error>> {
    let record = load_session_record(workspace)?;
    let trace_ref =
        record.latest_trace_ref.ok_or_else(|| io::Error::other("trace reference should exist"))?;
    Ok(FileTraceStore::for_workspace(workspace).load(Path::new(&trace_ref))?)
}

fn ensure_trace_event(
    trace: &ExecutionTrace,
    event_type: TraceEventType,
    context: &str,
) -> Result<(), Box<dyn Error>> {
    if trace.events.iter().any(|event| event.event_type == event_type) {
        return Ok(());
    }

    Err(fail(format!(
        "{context} missing expected trace event `{}`",
        serde_json::to_string(&event_type)?
    )))
}

fn contract_drift_reasoning_profile() -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "governance-contract-drift-reasoning".to_string(),
        stage_key: "bug-fix:verify".to_string(),
        profile_id: ReasoningProfileId::BoundedSelfConsistency,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: "Canon challenge posture contract drift blocked reasoning activation"
            .to_string(),
        status: ReasoningActivationStatus::Blocked,
        participants: Vec::new(),
        budget: ReasoningBudget {
            max_participants: 1,
            max_branches: 2,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 2_048,
            max_adjudication_steps: 1,
        },
        posture: Some(CanonChallengePostureInput {
            contract_line: DRIFTED_REASONING_POSTURE_CONTRACT_LINE.to_string(),
            compatibility_window: ReasoningCompatibilityWindow {
                boundline_min: "0.62.0".to_string(),
                boundline_max_exclusive: "0.63.0".to_string(),
                canon_min: "0.59.0".to_string(),
                canon_max_exclusive: "0.61.0".to_string(),
                contract_line: DRIFTED_REASONING_POSTURE_CONTRACT_LINE.to_string(),
            },
            required_profile_family: Some(ReasoningProfileFamily::SelfConsistency),
            required_profile_id: Some(ReasoningProfileId::BoundedSelfConsistency),
            minimum_independence: IndependenceFloor {
                route_distinct: false,
                provider_distinct: false,
                context_distinct: false,
                prompt_pattern_distinct: false,
                minimum_participants: 1,
            },
            admission_priority: CanonAdmissionPriority::RequiredBeforeContinue,
            confidence_handoff_required: true,
            provenance_ref: ".canon/contracts/governed-reasoning-posture.json".to_string(),
        }),
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Blocked,
            headline: "reasoning activation blocked by Canon contract drift".to_string(),
            disagreement_summary: Some(DRIFTED_REASONING_SUMMARY.to_string()),
            next_action: Some(DRIFTED_REASONING_GUIDANCE.to_string()),
            iterations: Vec::new(),
        }),
        confidence: Some(ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::Low,
            basis: vec!["contract_drift=unsupported_contract_line".to_string()],
            admission_effect: ReasoningAdmissionEffect::Gate,
            summary: "contract drift blocked reasoning activation before execution".to_string(),
        }),
    }
}

fn inject_latest_reasoning_profile(
    workspace: &Path,
    reasoning_profile: &ProfileActivationRecord,
) -> Result<(), Box<dyn Error>> {
    let mut session = load_session_record(workspace)?;
    let lifecycle = session
        .governance_lifecycle
        .as_mut()
        .ok_or_else(|| fail("governance lifecycle should exist"))?;
    lifecycle.latest_reasoning_profile = Some(reasoning_profile.clone());
    FileSessionStore::for_workspace(workspace).persist(&session)?;

    let mut trace = load_latest_trace(workspace)?;
    let plan_revision = trace.events.last().map(|event| event.plan_revision).unwrap_or(0);
    trace.record_event(
        TraceEventType::ReasoningProfileBlocked,
        Some("verify".to_string()),
        plan_revision,
        serde_json::json!({
            "profile_id": reasoning_profile.profile_id,
            "stage": reasoning_profile.stage_key,
            "activation_id": reasoning_profile.activation_id,
            "outcome_kind": "blocked",
            "summary": reasoning_profile
                .outcome
                .as_ref()
                .map(|outcome| outcome.headline.clone())
                .unwrap_or_else(|| reasoning_profile.activation_reason.clone()),
            "next_action": reasoning_profile
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.next_action.clone()),
            "canon_posture_ref": reasoning_profile
                .posture
                .as_ref()
                .map(|posture| posture.provenance_ref.clone()),
            "reasoning_profile_record": reasoning_profile,
        }),
    );
    FileTraceStore::for_workspace(workspace).persist(&trace)?;

    Ok(())
}

#[test]
fn insufficient_independence_blocks_reasoning_profile_through_status_and_inspect()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_canon_security_assessment_workspace(
        "boundline-reasoning-profile-insufficient-independence",
    );
    write_reasoning_profile_into_execution_profile(&workspace, &independent_pair_review_profile())?;
    save_collapsed_review_routing(&workspace)?;
    bootstrap_bug_fix(&workspace)?;

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    ensure_success(&run, &run_text, "run")?;

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    ensure_success(&status, &status_text, "status")?;
    for needle in [
        "latest_reasoning_profile_id: independent_pair_review",
        "latest_reasoning_profile_status: blocked",
        "latest_reasoning_independence_result: failed",
        "latest_reasoning_outcome: blocked",
        "latest_reasoning_next_action: configure distinct reviewer routes for reviewer_primary and reviewer_secondary",
        "latest_reasoning_confidence_level: low",
    ] {
        ensure_contains(&status_text, needle, "blocked reasoning status")?;
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    if inspect.status.code() != Some(1) && inspect.status.code() != Some(0) {
        return Err(fail(format!("inspect returned unexpected code: {inspect_text}")));
    }
    for needle in [
        "reasoning_profile_id: independent_pair_review",
        "reasoning_profile_status: blocked",
        "reasoning_independence_result: failed",
        "reasoning_outcome: blocked",
        "reasoning_next_action: configure distinct reviewer routes for reviewer_primary and reviewer_secondary",
    ] {
        ensure_contains(&inspect_text, needle, "blocked reasoning inspect")?;
    }

    let trace = load_latest_trace(&workspace)?;
    ensure_trace_event(&trace, TraceEventType::ReasoningProfileBlocked, "blocked reasoning trace")?;

    Ok(())
}

#[test]
fn approval_pending_interrupts_reasoning_profile_through_status_and_inspect()
-> Result<(), Box<dyn Error>> {
    let workspace =
        temp_canon_security_approval_workspace("boundline-reasoning-profile-interrupted");
    write_reasoning_profile_into_execution_profile(
        &workspace,
        &interruptible_self_consistency_profile(),
    )?;
    bootstrap_bug_fix(&workspace)?;

    let run = run_boundline_in(&workspace, &["run"]);
    let run_text = terminal_text(&run);
    ensure_success(&run, &run_text, "run")?;

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    ensure_success(&status, &status_text, "status")?;
    for needle in [
        "latest_governance_state: awaiting_approval",
        "latest_reasoning_profile_id: bounded_self_consistency",
        "latest_reasoning_profile_status: interrupted",
        "latest_reasoning_outcome: interrupted",
        "latest_reasoning_next_action: wait for approval and rerun boundline status",
    ] {
        ensure_contains(&status_text, needle, "interrupted reasoning status")?;
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    if inspect.status.code() != Some(1) && inspect.status.code() != Some(0) {
        return Err(fail(format!("inspect returned unexpected code: {inspect_text}")));
    }
    for needle in [
        "reasoning_profile_id: bounded_self_consistency",
        "reasoning_profile_status: interrupted",
        "reasoning_outcome: interrupted",
        "reasoning_next_action: wait for approval and rerun boundline status",
    ] {
        ensure_contains(&inspect_text, needle, "interrupted reasoning inspect")?;
    }

    let trace = load_latest_trace(&workspace)?;
    ensure_trace_event(
        &trace,
        TraceEventType::ReasoningProfileInterrupted,
        "interrupted reasoning trace",
    )?;

    Ok(())
}

#[test]
fn contract_drift_blocked_outcome_surfaces_guidance_through_status_and_inspect()
-> Result<(), Box<dyn Error>> {
    let workspace =
        temp_canon_security_assessment_workspace("boundline-reasoning-profile-contract-drift");
    write_reasoning_profile_into_execution_profile(
        &workspace,
        &interruptible_self_consistency_profile(),
    )?;
    bootstrap_bug_fix(&workspace)?;

    let run = run_boundline_in(&workspace, &["run"]);
    ensure_success(&run, &terminal_text(&run), "run")?;

    inject_latest_reasoning_profile(&workspace, &contract_drift_reasoning_profile())?;

    let status = run_boundline_in(&workspace, &["status"]);
    let status_text = terminal_text(&status);
    ensure_success(&status, &status_text, "status")?;
    for needle in [
        "latest_reasoning_profile_status: blocked",
        "latest_reasoning_posture_contract: governed_reasoning_posture_v999",
        "latest_reasoning_disagreement_summary: unsupported Canon reasoning posture contract line governed_reasoning_posture_v999 for the active Boundline/Canon release pair",
        "latest_reasoning_next_action: align Canon to governed_reasoning_posture_v1 or update the supported Boundline/Canon release pair before rerunning",
        "next_best_action: align Canon to governed_reasoning_posture_v1 or update the supported Boundline/Canon release pair before rerunning",
    ] {
        ensure_contains(&status_text, needle, "contract drift reasoning status")?;
    }

    let inspect = run_boundline_in(&workspace, &["--verbose", "inspect", "--workspace", "."]);
    let inspect_text = terminal_text(&inspect);
    if inspect.status.code() != Some(1) && inspect.status.code() != Some(0) {
        return Err(fail(format!("inspect returned unexpected code: {inspect_text}")));
    }
    for needle in [
        "reasoning_profile_status: blocked",
        "reasoning_posture_contract: governed_reasoning_posture_v999",
        "reasoning_disagreement_summary: unsupported Canon reasoning posture contract line governed_reasoning_posture_v999 for the active Boundline/Canon release pair",
        "reasoning_next_action: align Canon to governed_reasoning_posture_v1 or update the supported Boundline/Canon release pair before rerunning",
    ] {
        ensure_contains(&inspect_text, needle, "contract drift reasoning inspect")?;
    }

    Ok(())
}
