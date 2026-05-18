use std::error::Error;
use std::io;

use boundline::cli::inspect::summarize_trace;
use boundline::cli::output::render_trace_summary;
use boundline::domain::limits::TerminalCondition;
use boundline::domain::reasoning::{
    ProfileActivationRecord, ReasoningActivationStatus, ReasoningActivationTrigger,
    ReasoningAdmissionEffect, ReasoningBudget, ReasoningConfidenceContribution,
    ReasoningConfidenceLevel, ReasoningIterationCondition, ReasoningIterationKind,
    ReasoningIterationRecord, ReasoningOutcome, ReasoningOutcomeKind, ReasoningProfileId,
};
use boundline::domain::task::{TaskStatus, TerminalReason};
use boundline::domain::trace::{ExecutionTrace, TraceEventType};
use serde_json::json;

fn fail(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

fn ensure_contains(haystack: &str, needle: &str, context: &str) -> Result<(), Box<dyn Error>> {
    if haystack.contains(needle) {
        return Ok(());
    }

    Err(fail(format!("{context} missing expected text `{needle}` in output:\n{haystack}")))
}

fn terminal_trace() -> ExecutionTrace {
    let mut trace = ExecutionTrace::new(
        "task-reasoning-trace",
        "session-reasoning-trace",
        "Inspect reasoning trace",
    );
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason = Some(TerminalReason::new(
        TerminalCondition::UnrecoverableError,
        "reasoning trace failed",
        None,
    ));
    trace.ended_at = Some(trace.started_at + 1);
    trace
}

fn base_budget() -> ReasoningBudget {
    ReasoningBudget {
        max_participants: 2,
        max_branches: 1,
        max_debate_rounds: 2,
        max_reflexion_revisions: 1,
        max_calls: 4,
        max_tokens: 4_096,
        max_adjudication_steps: 1,
    }
}

fn reflexion_record() -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "attempt-1-reasoning".to_string(),
        stage_key: "bug-fix:verify".to_string(),
        profile_id: ReasoningProfileId::BoundedReflexion,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: "Canon posture required bounded reflexion before verification"
            .to_string(),
        status: ReasoningActivationStatus::Degraded,
        participants: Vec::new(),
        budget: base_budget(),
        posture: None,
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Degraded,
            headline: "bounded reflexion degraded after one non-novel revision".to_string(),
            disagreement_summary: Some(
                "bounded reflexion exhausted its revision budget without new evidence".to_string(),
            ),
            next_action: Some("run one bounded verification pass before merge".to_string()),
            iterations: vec![ReasoningIterationRecord {
                iteration_kind: ReasoningIterationKind::ReflexionRevision,
                iteration_index: 0,
                participants: vec!["critic-1".to_string(), "reviser-1".to_string()],
                summary: "the reviser repeated the prior patch rationale without adding evidence"
                    .to_string(),
                novelty: false,
                condition: ReasoningIterationCondition::Exhausted,
            }],
        }),
        confidence: Some(ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::Medium,
            basis: vec!["iteration_condition=exhausted".to_string(), "novelty=false".to_string()],
            admission_effect: ReasoningAdmissionEffect::Warn,
            summary: "reflexion converged partially; continue with bounded warning semantics"
                .to_string(),
        }),
    }
}

fn debate_stagnation_record() -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "attempt-2-reasoning".to_string(),
        stage_key: "bug-fix:verify".to_string(),
        profile_id: ReasoningProfileId::IndependentPairReview,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: "Canon posture required stronger challenge before verification".to_string(),
        status: ReasoningActivationStatus::Degraded,
        participants: Vec::new(),
        budget: base_budget(),
        posture: None,
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Degraded,
            headline: "bounded debate stagnated without new evidence".to_string(),
            disagreement_summary: Some(
                "two bounded debate rounds repeated the same objections without improving confidence"
                    .to_string(),
            ),
            next_action: Some("escalate to adjudication or continue with explicit caution".to_string()),
            iterations: vec![ReasoningIterationRecord {
                iteration_kind: ReasoningIterationKind::DebateRound,
                iteration_index: 1,
                participants: vec!["reviewer-a".to_string(), "reviewer-b".to_string()],
                summary: "the second debate round repeated the prior disagreement and stagnated"
                    .to_string(),
                novelty: false,
                condition: ReasoningIterationCondition::Stagnated,
            }],
        }),
        confidence: Some(ReasoningConfidenceContribution {
            confidence_level: ReasoningConfidenceLevel::Medium,
            basis: vec![
                "iteration_condition=stagnated".to_string(),
                "confidence_source=bounded_debate".to_string(),
            ],
            admission_effect: ReasoningAdmissionEffect::Warn,
            summary: "debate disagreement remained unresolved; continue only with explicit caution"
                .to_string(),
        }),
    }
}

#[test]
fn summarize_trace_preserves_bounded_reflexion_iterations_from_reasoning_events()
-> Result<(), Box<dyn Error>> {
    let mut trace = terminal_trace();
    let record = reflexion_record();
    trace.record_event(
        TraceEventType::ReasoningProfileActivated,
        Some("verify".to_string()),
        1,
        json!({ "reasoning_profile_record": record.clone() }),
    );
    trace.record_event(
        TraceEventType::ReasoningReflexionRevisionCompleted,
        Some("verify".to_string()),
        1,
        json!({ "reasoning_profile_record": record }),
    );

    let summary = summarize_trace("/tmp/reasoning-trace.json", &trace)?;
    let reasoning_profile =
        summary.reasoning_profile.ok_or_else(|| fail("expected reasoning profile projection"))?;
    let outcome =
        reasoning_profile.outcome.ok_or_else(|| fail("expected reasoning outcome projection"))?;
    let revision =
        outcome.iterations.first().ok_or_else(|| fail("expected bounded reflexion iteration"))?;

    if reasoning_profile.profile_id != ReasoningProfileId::BoundedReflexion {
        return Err(fail("expected bounded_reflexion profile id"));
    }
    if outcome.outcome_kind != ReasoningOutcomeKind::Degraded {
        return Err(fail("expected degraded outcome for bounded reflexion"));
    }
    if revision.iteration_kind != ReasoningIterationKind::ReflexionRevision {
        return Err(fail("expected reflexion revision iteration kind"));
    }
    if revision.condition != ReasoningIterationCondition::Exhausted {
        return Err(fail("expected exhausted reflexion iteration condition"));
    }

    Ok(())
}

#[test]
fn render_trace_summary_surfaces_debate_stagnation_and_confidence_from_reasoning_events()
-> Result<(), Box<dyn Error>> {
    let mut trace = terminal_trace();
    let record = debate_stagnation_record();
    trace.record_event(
        TraceEventType::ReasoningDebateRoundCompleted,
        Some("verify".to_string()),
        1,
        json!({ "reasoning_profile_record": record.clone() }),
    );
    trace.record_event(
        TraceEventType::ReasoningConfidenceRecorded,
        Some("verify".to_string()),
        1,
        json!({ "reasoning_profile_record": record }),
    );

    let summary = summarize_trace("/tmp/reasoning-trace.json", &trace)?;
    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    for needle in [
        "reasoning_profile_id: independent_pair_review",
        "reasoning_outcome: degraded",
        "reasoning_confidence_level: medium",
        "reasoning_confidence_summary: debate disagreement remained unresolved; continue only with explicit caution",
        "reasoning_next_action: escalate to adjudication or continue with explicit caution",
    ] {
        ensure_contains(&rendered, needle, "reasoning trace summary")?;
    }

    Ok(())
}
