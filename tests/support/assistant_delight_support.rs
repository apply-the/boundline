use std::error::Error;
use std::fs;

use boundline::domain::limits::TerminalCondition;
use boundline::domain::reasoning::{
    ProfileActivationRecord, ReasoningActivationStatus, ReasoningActivationTrigger,
    ReasoningAdmissionEffect, ReasoningBudget, ReasoningConfidenceContribution,
    ReasoningConfidenceLevel, ReasoningOutcome, ReasoningOutcomeKind, ReasoningProfileId,
};
use boundline::domain::session::{
    DelightFeedbackSignal, DelightNextActionOutcome, DelightSurface, SessionStatus,
    SessionStatusView,
};
use boundline::domain::task::{TaskStatus, TerminalReason};
use boundline::domain::trace::{ExecutionTrace, TraceSummaryView};

const DELIGHT_TRACE_FIXTURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/063-assistant-delight-followthrough/trace.json"
);

pub const ACTIVE_SELECTION_REASON: &str =
    "Canon governance activated stronger challenge before verification";
pub const ACTIVE_CONTRIBUTION: &str =
    "bounded reflexion clarified the highest-risk validation path";
pub const ACTIVE_CONFIDENCE_SUMMARY: &str =
    "bounded reflexion converged on the highest-risk validation path";
pub const ACTIVE_NEXT_ACTION: &str = "run the focused validation path before wider rollout";
pub const DEGRADED_SELECTION_REASON: &str =
    "Canon governance activated stronger challenge but critique capacity degraded";
pub const DEGRADED_CONTRIBUTION: &str = "bounded reflexion degraded after one non-novel revision";
pub const DEGRADED_CONFIDENCE_SUMMARY: &str =
    "bounded reflexion remained useful but confidence stayed degraded";
pub const DEGRADED_NEXT_ACTION: &str =
    "fallback to a bounded validation pass and preserve explicit caution";
pub const DEGRADED_FALLBACK_DISCLOSURE: &str = "reasoning profile is degraded: Canon governance activated stronger challenge but critique capacity degraded";
pub const DELIGHT_STARTED_AT: u64 = 1_716_115_200_000;
pub const DELIGHT_TIME_TO_FIRST_USEFUL_ANSWER_MS: u64 = 315_000;
pub const DELIGHT_FIRST_USEFUL_ANSWER_AT: u64 =
    DELIGHT_STARTED_AT + DELIGHT_TIME_TO_FIRST_USEFUL_ANSWER_MS;

pub fn load_delight_trace_fixture() -> Result<ExecutionTrace, Box<dyn Error>> {
    let fixture_text = fs::read_to_string(DELIGHT_TRACE_FIXTURE_PATH)?;
    Ok(serde_json::from_str(&fixture_text)?)
}

pub fn delight_feedback_signal() -> DelightFeedbackSignal {
    DelightFeedbackSignal {
        first_useful_answer_at: Some(DELIGHT_FIRST_USEFUL_ANSWER_AT),
        first_useful_answer_command: Some(DelightSurface::ExplainPlan),
        total_explanations: 1,
        attributed_explanations: 1,
        accepted_next_actions: 1,
        overridden_next_actions: 0,
        next_action_outcome: DelightNextActionOutcome::Accepted,
        override_reason: None,
        captured_at: Some(DELIGHT_FIRST_USEFUL_ANSWER_AT),
    }
}

fn reasoning_budget() -> ReasoningBudget {
    ReasoningBudget {
        max_participants: 2,
        max_branches: 1,
        max_debate_rounds: 0,
        max_reflexion_revisions: 1,
        max_calls: 2,
        max_tokens: 6_000,
        max_adjudication_steps: 1,
    }
}

pub fn active_reasoning_profile() -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "reasoning-delight-active".to_string(),
        stage_key: "bug-fix:verify".to_string(),
        profile_id: ReasoningProfileId::BoundedReflexion,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: ACTIVE_SELECTION_REASON.to_string(),
        status: ReasoningActivationStatus::Completed,
        participants: Vec::new(),
        budget: reasoning_budget(),
        posture: None,
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Converged,
            headline: ACTIVE_CONTRIBUTION.to_string(),
            disagreement_summary: None,
            next_action: Some(ACTIVE_NEXT_ACTION.to_string()),
            iterations: Vec::new(),
        }),
        confidence: Some(ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::High,
            basis: vec!["reasoning_profile=bounded_reflexion".to_string()],
            admission_effect: ReasoningAdmissionEffect::None,
            summary: ACTIVE_CONFIDENCE_SUMMARY.to_string(),
        }),
    }
}

pub fn degraded_reasoning_profile() -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "reasoning-delight-degraded".to_string(),
        stage_key: "bug-fix:verify".to_string(),
        profile_id: ReasoningProfileId::BoundedReflexion,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: DEGRADED_SELECTION_REASON.to_string(),
        status: ReasoningActivationStatus::Degraded,
        participants: Vec::new(),
        budget: reasoning_budget(),
        posture: None,
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Degraded,
            headline: DEGRADED_CONTRIBUTION.to_string(),
            disagreement_summary: Some(
                "bounded reflexion repeated the prior critique without adding new evidence"
                    .to_string(),
            ),
            next_action: Some(DEGRADED_NEXT_ACTION.to_string()),
            iterations: Vec::new(),
        }),
        confidence: Some(ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::Medium,
            basis: vec!["reasoning_status=degraded".to_string()],
            admission_effect: ReasoningAdmissionEffect::Warn,
            summary: DEGRADED_CONFIDENCE_SUMMARY.to_string(),
        }),
    }
}

pub fn active_trace_summary() -> TraceSummaryView {
    TraceSummaryView {
        trace_ref: "/tmp/workspace/.boundline/traces/assistant-delight-active-trace.json"
            .to_string(),
        goal: "Explain the bounded runtime change".to_string(),
        trace_started_at: Some(DELIGHT_STARTED_AT),
        goal_plan_summary: Some(
            "goal plan keeps the explanation bounded to the active verification slice".to_string(),
        ),
        context_summary: Some("bounded runtime evidence is available".to_string()),
        governance_timeline: vec!["governance_selected: bug-fix:verify -> canon".to_string()],
        governance_next_action: Some("review the focused validation slice".to_string()),
        reasoning_profile: Some(active_reasoning_profile()),
        delight_feedback: Some(delight_feedback_signal()),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "bounded explanation closed successfully",
            None,
        ),
        ..TraceSummaryView::default()
    }
}

pub fn degraded_trace_summary() -> TraceSummaryView {
    TraceSummaryView {
        trace_ref: "/tmp/workspace/.boundline/traces/assistant-delight-degraded-trace.json"
            .to_string(),
        goal: "Explain the bounded runtime change".to_string(),
        trace_started_at: Some(DELIGHT_STARTED_AT),
        goal_plan_summary: Some(
            "goal plan keeps the explanation bounded to the active verification slice".to_string(),
        ),
        context_summary: Some("bounded runtime evidence is available".to_string()),
        governance_timeline: vec!["governance_selected: bug-fix:verify -> canon".to_string()],
        governance_next_action: Some(DEGRADED_NEXT_ACTION.to_string()),
        reasoning_profile: Some(degraded_reasoning_profile()),
        delight_feedback: Some(delight_feedback_signal()),
        terminal_status: TaskStatus::Failed,
        terminal_reason: TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "bounded explanation remains degraded but inspectable",
            None,
        ),
        ..TraceSummaryView::default()
    }
}

pub fn active_session_status() -> SessionStatusView {
    SessionStatusView {
        session_id: "session-assistant-delight-active".to_string(),
        workspace_ref: "/tmp/workspace".to_string(),
        session_started_at: Some(DELIGHT_STARTED_AT),
        goal: Some("Explain the bounded runtime change".to_string()),
        active_flow: Some("bug-fix".to_string()),
        flow_state: Some("verify".to_string()),
        latest_status: SessionStatus::Running,
        latest_governance_runtime: Some("canon".to_string()),
        latest_governance_packet_ref: Some(".canon/runs/run-42".to_string()),
        latest_reasoning_profile: Some(active_reasoning_profile()),
        delight_feedback: Some(delight_feedback_signal()),
        next_command: Some("boundline inspect".to_string()),
        explanation: "bounded explanation remains inspectable".to_string(),
        ..SessionStatusView::default()
    }
}

pub fn degraded_session_status() -> SessionStatusView {
    SessionStatusView {
        session_id: "session-assistant-delight-degraded".to_string(),
        workspace_ref: "/tmp/workspace".to_string(),
        session_started_at: Some(DELIGHT_STARTED_AT),
        goal: Some("Explain the bounded runtime change".to_string()),
        active_flow: Some("bug-fix".to_string()),
        flow_state: Some("verify".to_string()),
        latest_status: SessionStatus::Failed,
        latest_governance_runtime: Some("canon".to_string()),
        latest_governance_packet_ref: Some(".canon/runs/run-42".to_string()),
        latest_reasoning_profile: Some(degraded_reasoning_profile()),
        delight_feedback: Some(delight_feedback_signal()),
        next_command: Some("boundline status".to_string()),
        explanation: "bounded explanation remains inspectable".to_string(),
        ..SessionStatusView::default()
    }
}
