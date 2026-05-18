use serde::Serialize;
use serde_json::{Map, Value};

use crate::domain::reasoning::{
    IndependenceAssessmentResult, ParticipantAssignment, ProfileActivationRecord,
    ReasoningActivationStatus, ReasoningConfidenceContribution, ReasoningConfidenceLevel,
    ReasoningIterationKind, ReasoningIterationRecord, ReasoningOutcome, ReasoningOutcomeKind,
    ReasoningParticipantStatus,
};
use crate::domain::step::{ExecutionStatus, StepExecutionResult};
use crate::domain::trace::{ExecutionTrace, TraceEventType};

const REVIEW_PHASE: &str = "review";
const REVIEW_VOTE_PHASE: &str = "review-vote";
const REVIEW_FINALIZE_PHASE: &str = "review-finalize";
const UNKNOWN_REVIEWER_ID: &str = "unknown-reviewer";
const PARTICIPATION_STATUS_COMPLETED: &str = "completed";
const PARTICIPATION_STATUS_FAILED: &str = "failed";
const KEY_ADJUDICATION: &str = "adjudication";
const KEY_DEFAULT_REVIEW_TRIGGER: &str = "default_review_trigger";
const KEY_FINDING: &str = "finding";
const KEY_LATEST_REVIEW_OUTCOME: &str = "latest_review_outcome";
const KEY_LATEST_REVIEW_COUNCIL_PROFILE: &str = "latest_review_council_profile";
const KEY_LATEST_REVIEW_INDEPENDENCE_STATE: &str = "latest_review_independence_state";
const KEY_LATEST_REVIEW_SELECTION_SUMMARY: &str = "latest_review_selection_summary";
const KEY_LATEST_REVIEW_STOP_SEMANTICS: &str = "latest_review_stop_semantics";
const KEY_LATEST_REVIEW_PARTICIPANTS: &str = "latest_review_participants";
const KEY_LATEST_REVIEW_TRIGGER: &str = "latest_review_trigger";
const KEY_LATEST_REVIEW_VOTE: &str = "latest_review_vote";
const KEY_LATEST_REVIEW_VOTE_RESOLUTION: &str = "latest_review_vote_resolution";
const KEY_NEXT_REVIEW_TRIGGER: &str = "next_review_trigger";
const KEY_PHASE: &str = "phase";
const KEY_REVIEW_OUTCOME: &str = "review_outcome";
const KEY_REVIEW_TRIGGER: &str = "review_trigger";
const KEY_REVIEWER_ID: &str = "reviewer_id";
const KEY_REVIEWER_ROLE: &str = "reviewer_role";
const KEY_REVIEWER_SOURCE: &str = "reviewer_source";
const KEY_STAGE_ID: &str = "stage_id";
const KEY_SUMMARY: &str = "summary";
const KEY_VOTE: &str = "vote";
const KEY_VOTE_RESOLUTION: &str = "vote_resolution";

#[derive(Debug, Serialize)]
struct ReviewStartedPayload {
    review_trigger: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_id: Option<String>,
    adjudication: bool,
}

#[derive(Debug, Serialize)]
struct ReviewerStartedPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    review_trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_id: Option<String>,
    reviewer_id: String,
    adjudication: bool,
}

#[derive(Debug, Serialize)]
struct ReviewerCompletedPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    review_trigger: Option<String>,
    reviewer_id: String,
    participation_status: &'static str,
    adjudication: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviewer_role: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reviewer_source: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finding: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_outcome: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ReviewVoteResolvedPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    review_trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    council_profile: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    independence_state: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_semantics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vote_resolution: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_outcome: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ReviewTerminalPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    review_trigger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    council_profile: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    independence_state: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_semantics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_vote: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    participants: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReasoningProfileEventPayload {
    profile_id: String,
    stage: String,
    activation_id: String,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome_kind: Option<ReasoningOutcomeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    independence_result: Option<IndependenceAssessmentResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_posture_ref: Option<String>,
    reasoning_profile_record: ProfileActivationRecord,
}

#[derive(Debug, Clone, Serialize)]
struct ReasoningParticipantEventPayload {
    profile_id: String,
    stage: String,
    activation_id: String,
    participant_id: String,
    role: String,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_posture_ref: Option<String>,
    reasoning_profile_record: ProfileActivationRecord,
}

#[derive(Debug, Clone, Serialize)]
struct ReasoningIterationEventPayload {
    profile_id: String,
    stage: String,
    activation_id: String,
    iteration_kind: ReasoningIterationKind,
    iteration_index: usize,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome_kind: Option<ReasoningOutcomeKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_posture_ref: Option<String>,
    reasoning_profile_record: ProfileActivationRecord,
}

#[derive(Debug, Clone, Serialize)]
struct ReasoningConfidenceEventPayload {
    profile_id: String,
    stage: String,
    activation_id: String,
    confidence_level: ReasoningConfidenceLevel,
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    canon_posture_ref: Option<String>,
    reasoning_profile_record: ProfileActivationRecord,
}

fn serialize_payload<T: Serialize>(payload: &T) -> Value {
    serde_json::to_value(payload).unwrap_or(Value::Null)
}

pub(crate) fn record_reasoning_profile_events(
    trace: &mut ExecutionTrace,
    step_id: &str,
    plan_revision: usize,
    activation: &ProfileActivationRecord,
) {
    trace.record_event(
        TraceEventType::ReasoningProfileActivated,
        Some(step_id.to_string()),
        plan_revision,
        serialize_payload(&ReasoningProfileEventPayload::from_activation(activation)),
    );

    for participant in &activation.participants {
        let payload = ReasoningParticipantEventPayload::from_participant(activation, participant);
        trace.record_event(
            TraceEventType::ReasoningParticipantStarted,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&payload),
        );

        if participant_terminal_status(participant.status) {
            trace.record_event(
                TraceEventType::ReasoningParticipantCompleted,
                Some(step_id.to_string()),
                plan_revision,
                serialize_payload(&payload),
            );
        }
    }

    if let Some(outcome) = activation.outcome.as_ref() {
        trace.record_event(
            TraceEventType::ReasoningDisagreementRecorded,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&ReasoningProfileEventPayload::from_outcome(activation, outcome)),
        );

        let mut adjudication_recorded = false;
        for iteration in &outcome.iterations {
            let Some(event_type) = reasoning_iteration_event_type(iteration.iteration_kind) else {
                continue;
            };
            if event_type == TraceEventType::ReasoningAdjudicationRecorded {
                adjudication_recorded = true;
            }
            trace.record_event(
                event_type,
                Some(step_id.to_string()),
                plan_revision,
                serialize_payload(&ReasoningIterationEventPayload::from_iteration(
                    activation, outcome, iteration,
                )),
            );
        }

        if outcome.outcome_kind == ReasoningOutcomeKind::Adjudicated && !adjudication_recorded {
            trace.record_event(
                TraceEventType::ReasoningAdjudicationRecorded,
                Some(step_id.to_string()),
                plan_revision,
                serialize_payload(&ReasoningProfileEventPayload::from_outcome(activation, outcome)),
            );
        }
    }

    if let Some(confidence) = activation.confidence.as_ref() {
        trace.record_event(
            TraceEventType::ReasoningConfidenceRecorded,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&ReasoningConfidenceEventPayload::from_confidence(
                activation, confidence,
            )),
        );
    }

    if let Some(event_type) = reasoning_terminal_event_type(activation.status) {
        trace.record_event(
            event_type,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&ReasoningProfileEventPayload::from_activation(activation)),
        );
    }
}

impl ReasoningProfileEventPayload {
    fn from_activation(activation: &ProfileActivationRecord) -> Self {
        Self {
            profile_id: activation.profile_id.as_str().to_string(),
            stage: activation.stage_key.clone(),
            activation_id: activation.activation_id.clone(),
            summary: activation
                .outcome
                .as_ref()
                .map(|outcome| outcome.headline.clone())
                .unwrap_or_else(|| activation.activation_reason.clone()),
            outcome_kind: activation.outcome.as_ref().map(|outcome| outcome.outcome_kind),
            independence_result: activation.independence.as_ref().map(|value| value.result),
            next_action: activation
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.next_action.clone()),
            canon_posture_ref: reasoning_canon_posture_ref(activation),
            reasoning_profile_record: activation.clone(),
        }
    }

    fn from_outcome(activation: &ProfileActivationRecord, outcome: &ReasoningOutcome) -> Self {
        Self {
            profile_id: activation.profile_id.as_str().to_string(),
            stage: activation.stage_key.clone(),
            activation_id: activation.activation_id.clone(),
            summary: outcome
                .disagreement_summary
                .clone()
                .unwrap_or_else(|| outcome.headline.clone()),
            outcome_kind: Some(outcome.outcome_kind),
            independence_result: activation.independence.as_ref().map(|value| value.result),
            next_action: outcome.next_action.clone(),
            canon_posture_ref: reasoning_canon_posture_ref(activation),
            reasoning_profile_record: activation.clone(),
        }
    }
}

impl ReasoningParticipantEventPayload {
    fn from_participant(
        activation: &ProfileActivationRecord,
        participant: &ParticipantAssignment,
    ) -> Self {
        Self {
            profile_id: activation.profile_id.as_str().to_string(),
            stage: activation.stage_key.clone(),
            activation_id: activation.activation_id.clone(),
            participant_id: participant.participant_id.clone(),
            role: participant.role_id.clone(),
            summary: participant.result_summary.clone().unwrap_or_else(|| {
                format!(
                    "route={} status={}",
                    participant.effective_route,
                    reasoning_participant_status_text(participant.status)
                )
            }),
            canon_posture_ref: reasoning_canon_posture_ref(activation),
            reasoning_profile_record: activation.clone(),
        }
    }
}

impl ReasoningIterationEventPayload {
    fn from_iteration(
        activation: &ProfileActivationRecord,
        outcome: &ReasoningOutcome,
        iteration: &ReasoningIterationRecord,
    ) -> Self {
        Self {
            profile_id: activation.profile_id.as_str().to_string(),
            stage: activation.stage_key.clone(),
            activation_id: activation.activation_id.clone(),
            iteration_kind: iteration.iteration_kind,
            iteration_index: iteration.iteration_index,
            summary: iteration.summary.clone(),
            outcome_kind: Some(outcome.outcome_kind),
            next_action: outcome.next_action.clone(),
            canon_posture_ref: reasoning_canon_posture_ref(activation),
            reasoning_profile_record: activation.clone(),
        }
    }
}

impl ReasoningConfidenceEventPayload {
    fn from_confidence(
        activation: &ProfileActivationRecord,
        confidence: &ReasoningConfidenceContribution,
    ) -> Self {
        Self {
            profile_id: activation.profile_id.as_str().to_string(),
            stage: activation.stage_key.clone(),
            activation_id: activation.activation_id.clone(),
            confidence_level: confidence.confidence_level,
            summary: confidence.summary.clone(),
            next_action: activation
                .outcome
                .as_ref()
                .and_then(|outcome| outcome.next_action.clone()),
            canon_posture_ref: reasoning_canon_posture_ref(activation),
            reasoning_profile_record: activation.clone(),
        }
    }
}

fn reasoning_canon_posture_ref(activation: &ProfileActivationRecord) -> Option<String> {
    activation.posture.as_ref().map(|posture| posture.contract_line.clone())
}

fn participant_terminal_status(status: ReasoningParticipantStatus) -> bool {
    matches!(
        status,
        ReasoningParticipantStatus::Completed
            | ReasoningParticipantStatus::Failed
            | ReasoningParticipantStatus::Omitted
    )
}

fn reasoning_participant_status_text(status: ReasoningParticipantStatus) -> &'static str {
    match status {
        ReasoningParticipantStatus::Pending => "pending",
        ReasoningParticipantStatus::Running => "running",
        ReasoningParticipantStatus::Completed => "completed",
        ReasoningParticipantStatus::Failed => "failed",
        ReasoningParticipantStatus::Omitted => "omitted",
    }
}

fn reasoning_iteration_event_type(
    iteration_kind: ReasoningIterationKind,
) -> Option<TraceEventType> {
    match iteration_kind {
        ReasoningIterationKind::DebateRound => Some(TraceEventType::ReasoningDebateRoundCompleted),
        ReasoningIterationKind::ReflexionRevision => {
            Some(TraceEventType::ReasoningReflexionRevisionCompleted)
        }
        ReasoningIterationKind::AdjudicationStep => {
            Some(TraceEventType::ReasoningAdjudicationRecorded)
        }
        ReasoningIterationKind::Branch => None,
    }
}

fn reasoning_terminal_event_type(status: ReasoningActivationStatus) -> Option<TraceEventType> {
    match status {
        ReasoningActivationStatus::Blocked => Some(TraceEventType::ReasoningProfileBlocked),
        ReasoningActivationStatus::Interrupted => Some(TraceEventType::ReasoningProfileInterrupted),
        ReasoningActivationStatus::Escalated => Some(TraceEventType::ReasoningProfileEscalated),
        _ => None,
    }
}

pub(crate) fn record_review_step_started(
    trace: &mut ExecutionTrace,
    step_id: &str,
    step_input: &Value,
    state: &Map<String, Value>,
    plan_revision: usize,
) {
    if step_phase(step_input) != Some(REVIEW_PHASE) {
        return;
    }

    let review_trigger = review_trigger_from_state_or_input(state, step_input);
    let stage_id = stage_id(step_input);
    let adjudication = step_input.get(KEY_ADJUDICATION).and_then(Value::as_bool).unwrap_or(false);
    let reviewer_id = step_input
        .get(KEY_REVIEWER_ID)
        .and_then(Value::as_str)
        .unwrap_or(UNKNOWN_REVIEWER_ID)
        .to_string();

    if let Some(review_trigger) = review_trigger.as_deref()
        && !review_phase_active(trace)
    {
        let event_type = if review_phase_seen(trace, review_trigger, stage_id.as_deref()) {
            TraceEventType::ReviewTriggerIgnored
        } else {
            TraceEventType::ReviewStarted
        };
        trace.record_event(
            event_type,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&ReviewStartedPayload {
                review_trigger: review_trigger.to_string(),
                stage_id: stage_id.clone(),
                adjudication,
            }),
        );
    }

    trace.record_event(
        TraceEventType::ReviewerStarted,
        Some(step_id.to_string()),
        plan_revision,
        serialize_payload(&ReviewerStartedPayload {
            review_trigger,
            stage_id,
            reviewer_id,
            adjudication,
        }),
    );
}

pub(crate) fn record_review_step_completed(
    trace: &mut ExecutionTrace,
    step_id: &str,
    step_input: &Value,
    result: &StepExecutionResult,
    state_after: &Map<String, Value>,
    plan_revision: usize,
) {
    match step_phase(step_input) {
        Some(REVIEW_PHASE) => {
            let reviewer_id = result
                .output
                .as_ref()
                .and_then(|output| output.get(KEY_REVIEWER_ID))
                .and_then(Value::as_str)
                .or_else(|| step_input.get(KEY_REVIEWER_ID).and_then(Value::as_str))
                .unwrap_or(UNKNOWN_REVIEWER_ID)
                .to_string();
            let adjudication = result
                .output
                .as_ref()
                .and_then(|output| output.get(KEY_ADJUDICATION))
                .and_then(Value::as_bool)
                .or_else(|| step_input.get(KEY_ADJUDICATION).and_then(Value::as_bool))
                .unwrap_or(false);

            let payload = ReviewerCompletedPayload {
                review_trigger: review_trigger_from_output_or_state(
                    result,
                    state_after,
                    step_input,
                ),
                reviewer_id,
                participation_status: if result.status == ExecutionStatus::Succeeded {
                    PARTICIPATION_STATUS_COMPLETED
                } else {
                    PARTICIPATION_STATUS_FAILED
                },
                adjudication,
                reviewer_role: result
                    .output
                    .as_ref()
                    .and_then(|output| output.get(KEY_REVIEWER_ROLE))
                    .cloned(),
                reviewer_source: result
                    .output
                    .as_ref()
                    .and_then(|output| output.get(KEY_REVIEWER_SOURCE))
                    .cloned(),
                finding: result.output.as_ref().and_then(|output| output.get(KEY_FINDING)).cloned(),
                failure_reason: result.error.as_ref().map(|error| error.message.clone()),
                review_outcome: result
                    .error
                    .as_ref()
                    .and_then(|_| state_after.get(KEY_LATEST_REVIEW_OUTCOME))
                    .cloned(),
            };

            trace.record_event(
                TraceEventType::ReviewerCompleted,
                Some(step_id.to_string()),
                plan_revision,
                serialize_payload(&payload),
            );

            if adjudication && result.status == ExecutionStatus::Succeeded {
                trace.record_event(
                    TraceEventType::ReviewAdjudicated,
                    Some(step_id.to_string()),
                    plan_revision,
                    serialize_payload(&payload),
                );
            }

            record_review_terminal_if_present(trace, step_id, result, state_after, plan_revision);
        }
        Some(REVIEW_VOTE_PHASE) => {
            if let Some(output) = result.output.as_ref() {
                let payload = ReviewVoteResolvedPayload {
                    review_trigger: output
                        .get(KEY_REVIEW_TRIGGER)
                        .and_then(Value::as_str)
                        .or_else(|| {
                            state_after.get(KEY_LATEST_REVIEW_TRIGGER).and_then(Value::as_str)
                        })
                        .map(str::to_string),
                    summary: output
                        .get(KEY_SUMMARY)
                        .or_else(|| state_after.get(KEY_LATEST_REVIEW_VOTE))
                        .cloned(),
                    council_profile: state_after.get(KEY_LATEST_REVIEW_COUNCIL_PROFILE).cloned(),
                    independence_state: state_after
                        .get(KEY_LATEST_REVIEW_INDEPENDENCE_STATE)
                        .cloned(),
                    selection_summary: state_after
                        .get(KEY_LATEST_REVIEW_SELECTION_SUMMARY)
                        .cloned(),
                    stop_semantics: state_after.get(KEY_LATEST_REVIEW_STOP_SEMANTICS).cloned(),
                    vote_resolution: output
                        .get(KEY_VOTE_RESOLUTION)
                        .or_else(|| output.get(KEY_VOTE))
                        .or_else(|| state_after.get(KEY_LATEST_REVIEW_VOTE_RESOLUTION))
                        .cloned(),
                    review_outcome: output.get(KEY_REVIEW_OUTCOME).cloned(),
                };
                trace.record_event(
                    TraceEventType::ReviewVoteResolved,
                    Some(step_id.to_string()),
                    plan_revision,
                    serialize_payload(&payload),
                );
            }

            if result.status == ExecutionStatus::Failed {
                record_review_terminal_if_present(
                    trace,
                    step_id,
                    result,
                    state_after,
                    plan_revision,
                );
            }
        }
        Some(REVIEW_FINALIZE_PHASE) => {
            record_review_terminal_if_present(trace, step_id, result, state_after, plan_revision);
        }
        _ => {}
    }
}

fn record_review_terminal_if_present(
    trace: &mut ExecutionTrace,
    step_id: &str,
    result: &StepExecutionResult,
    state_after: &Map<String, Value>,
    plan_revision: usize,
) {
    let review_outcome =
        state_after.get(KEY_LATEST_REVIEW_OUTCOME).and_then(Value::as_str).or_else(|| {
            result
                .output
                .as_ref()
                .and_then(|output| output.get(KEY_REVIEW_OUTCOME))
                .and_then(Value::as_str)
        });
    let review_trigger =
        state_after.get(KEY_LATEST_REVIEW_TRIGGER).and_then(Value::as_str).or_else(|| {
            result
                .output
                .as_ref()
                .and_then(|output| output.get(KEY_REVIEW_TRIGGER))
                .and_then(Value::as_str)
        });

    if review_outcome.is_none() {
        return;
    }

    let payload = ReviewTerminalPayload {
        review_trigger: review_trigger.map(str::to_string),
        review_outcome: review_outcome.map(str::to_string),
        council_profile: state_after.get(KEY_LATEST_REVIEW_COUNCIL_PROFILE).cloned(),
        independence_state: state_after.get(KEY_LATEST_REVIEW_INDEPENDENCE_STATE).cloned(),
        selection_summary: state_after.get(KEY_LATEST_REVIEW_SELECTION_SUMMARY).cloned(),
        stop_semantics: state_after.get(KEY_LATEST_REVIEW_STOP_SEMANTICS).cloned(),
        review_vote: state_after.get(KEY_LATEST_REVIEW_VOTE).cloned(),
        participants: state_after.get(KEY_LATEST_REVIEW_PARTICIPANTS).cloned(),
        failure_reason: result.error.as_ref().map(|error| error.message.clone()),
    };

    let already_recorded = trace.events.iter().rev().any(|event| {
        event.event_type == TraceEventType::ReviewTerminalRecorded
            && event.step_id.as_deref() == Some(step_id)
            && event.plan_revision == plan_revision
    });
    if !already_recorded {
        trace.record_event(
            TraceEventType::ReviewTerminalRecorded,
            Some(step_id.to_string()),
            plan_revision,
            serialize_payload(&payload),
        );
    }
}

fn review_phase_active(trace: &ExecutionTrace) -> bool {
    let started = trace
        .events
        .iter()
        .filter(|event| event.event_type == TraceEventType::ReviewStarted)
        .count();
    let finished = trace
        .events
        .iter()
        .filter(|event| event.event_type == TraceEventType::ReviewTerminalRecorded)
        .count();
    started > finished
}

fn review_phase_seen(trace: &ExecutionTrace, review_trigger: &str, stage_id: Option<&str>) -> bool {
    trace.events.iter().any(|event| {
        event.event_type == TraceEventType::ReviewStarted
            && event.payload.get(KEY_REVIEW_TRIGGER).and_then(Value::as_str) == Some(review_trigger)
            && event.payload.get(KEY_STAGE_ID).and_then(Value::as_str) == stage_id
    })
}

fn review_trigger_from_state_or_input(
    state: &Map<String, Value>,
    step_input: &Value,
) -> Option<String> {
    state
        .get(KEY_NEXT_REVIEW_TRIGGER)
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            state.get(KEY_LATEST_REVIEW_TRIGGER).and_then(Value::as_str).map(str::to_string)
        })
        .or_else(|| {
            step_input.get(KEY_DEFAULT_REVIEW_TRIGGER).and_then(Value::as_str).map(str::to_string)
        })
}

fn review_trigger_from_output_or_state(
    result: &StepExecutionResult,
    state_after: &Map<String, Value>,
    step_input: &Value,
) -> Option<String> {
    result
        .output
        .as_ref()
        .and_then(|output| output.get(KEY_REVIEW_TRIGGER))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            state_after.get(KEY_LATEST_REVIEW_TRIGGER).and_then(Value::as_str).map(str::to_string)
        })
        .or_else(|| review_trigger_from_state_or_input(state_after, step_input))
}

fn step_phase(step_input: &Value) -> Option<&str> {
    step_input.get(KEY_PHASE).and_then(Value::as_str)
}

fn stage_id(step_input: &Value) -> Option<String> {
    step_input.get(KEY_STAGE_ID).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use super::{
        reasoning_iteration_event_type, reasoning_terminal_event_type,
        record_reasoning_profile_events, record_review_step_completed, record_review_step_started,
        review_trigger_from_output_or_state, review_trigger_from_state_or_input,
    };
    use crate::domain::reasoning::{
        ParticipantAssignment, ProfileActivationRecord, ReasoningActivationStatus,
        ReasoningActivationTrigger, ReasoningAdmissionEffect, ReasoningBudget,
        ReasoningConfidenceContribution, ReasoningConfidenceLevel, ReasoningIterationCondition,
        ReasoningIterationKind, ReasoningIterationRecord, ReasoningOutcome, ReasoningOutcomeKind,
        ReasoningParticipantStatus, ReasoningProfileId,
    };
    use crate::domain::step::{ErrorInfo, Recoverability, StepExecutionResult};
    use crate::domain::trace::{ExecutionTrace, TraceEventType};

    #[test]
    fn review_trace_records_started_vote_and_terminal_events() {
        let mut trace = ExecutionTrace::new("task-review-trace", "session-review-trace", "goal");
        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!("pr_ready"));
        let reviewer_input = json!({
            "phase": "review",
            "reviewer_id": "safety",
            "adjudication": false,
        });

        record_review_step_started(&mut trace, "review-safety", &reviewer_input, &state, 0);

        let mut reviewer_state = state.clone();
        reviewer_state.insert("latest_review_trigger".to_string(), json!("pr_ready"));
        let reviewer_result = StepExecutionResult::success(json!({
            "review_trigger": "pr_ready",
            "reviewer_id": "safety",
            "reviewer_role": "Safety",
            "reviewer_source": "gpt",
            "finding": {
                "reviewer_id": "safety",
                "disposition": "approve",
                "summary": "No blockers"
            },
            "adjudication": false,
        }));
        record_review_step_completed(
            &mut trace,
            "review-safety",
            &reviewer_input,
            &reviewer_result,
            &reviewer_state,
            0,
        );

        let vote_input = json!({"phase": "review-vote"});
        let mut vote_state = reviewer_state.clone();
        vote_state.insert("latest_review_council_profile".to_string(), json!("yellow_pair"));
        vote_state.insert("latest_review_independence_state".to_string(), json!("passed"));
        vote_state.insert(
            "latest_review_selection_summary".to_string(),
            json!("profile=yellow_pair quorum=met independence=passed selected_roles=Safety, Maintainability"),
        );
        vote_state.insert("latest_review_stop_semantics".to_string(), json!("council_required"));
        let vote_result = StepExecutionResult::success(json!({
            "review_trigger": "pr_ready",
            "review_outcome": "accepted",
            "summary": "strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted",
            "vote_resolution": {
                "strategy": "majority",
                "approvals": 1,
                "concerns": 0,
                "blocks": 0,
                "decision": "accepted"
            }
        }));
        record_review_step_completed(
            &mut trace,
            "review-vote",
            &vote_input,
            &vote_result,
            &vote_state,
            0,
        );

        let mut final_state = vote_state.clone();
        final_state.insert("latest_review_outcome".to_string(), json!("accepted"));
        final_state.insert(
            "latest_review_vote".to_string(),
            json!("strategy=majority approvals=1 concerns=0 blocks=0 decision=accepted"),
        );
        let finalize_input = json!({"phase": "review-finalize"});
        let finalize_result = StepExecutionResult::success(json!({
            "review_trigger": "pr_ready",
            "review_outcome": "accepted"
        }));
        record_review_step_completed(
            &mut trace,
            "review-finalize",
            &finalize_input,
            &finalize_result,
            &final_state,
            0,
        );

        assert_eq!(trace.events[0].event_type, TraceEventType::ReviewStarted);
        assert_eq!(trace.events[1].event_type, TraceEventType::ReviewerStarted);
        assert_eq!(trace.events[2].event_type, TraceEventType::ReviewerCompleted);
        assert_eq!(trace.events[3].event_type, TraceEventType::ReviewVoteResolved);
        assert_eq!(trace.events[4].event_type, TraceEventType::ReviewTerminalRecorded);
        assert_eq!(
            trace.events[3].payload.get("council_profile").and_then(|value| value.as_str()),
            Some("yellow_pair")
        );
        assert_eq!(
            trace.events[3].payload.get("stop_semantics").and_then(|value| value.as_str()),
            Some("council_required")
        );
        assert_eq!(
            trace.events[4].payload.get("independence_state").and_then(|value| value.as_str()),
            Some("passed")
        );
    }

    #[test]
    fn review_trace_marks_duplicate_trigger_after_terminal_recording() {
        let mut trace =
            ExecutionTrace::new("task-review-ignored", "session-review-ignored", "goal");
        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!("pr_ready"));
        let reviewer_input = json!({
            "phase": "review",
            "reviewer_id": "safety",
            "adjudication": false,
            "stage_id": "verify",
        });

        record_review_step_started(&mut trace, "review-safety-1", &reviewer_input, &state, 0);

        let mut final_state = Map::new();
        final_state.insert("latest_review_trigger".to_string(), json!("pr_ready"));
        final_state.insert("latest_review_outcome".to_string(), json!("accepted"));
        record_review_step_completed(
            &mut trace,
            "review-finalize",
            &json!({"phase": "review-finalize"}),
            &StepExecutionResult::success(json!({
                "review_trigger": "pr_ready",
                "review_outcome": "accepted"
            })),
            &final_state,
            0,
        );

        record_review_step_started(&mut trace, "review-safety-2", &reviewer_input, &state, 1);

        assert!(
            trace
                .events
                .iter()
                .any(|event| event.event_type == TraceEventType::ReviewTriggerIgnored)
        );
    }

    #[test]
    fn review_trace_records_adjudication_and_failed_vote_terminal_from_state() {
        let mut trace = ExecutionTrace::new("task-review-extra", "session-review-extra", "goal");
        let mut review_state = Map::new();
        review_state.insert("latest_review_trigger".to_string(), json!("pr_ready"));

        record_review_step_completed(
            &mut trace,
            "review-adjudicator",
            &json!({
                "phase": "review",
                "reviewer_id": "lead",
                "adjudication": true,
            }),
            &StepExecutionResult::success(json!({
                "reviewer_id": "lead",
                "adjudication": true,
            })),
            &review_state,
            0,
        );

        let adjudicated = trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::ReviewAdjudicated)
            .unwrap();
        assert_eq!(
            adjudicated.payload.get("review_trigger").and_then(|value| value.as_str()),
            Some("pr_ready")
        );

        let mut vote_state = review_state.clone();
        vote_state.insert("latest_review_outcome".to_string(), json!("rejected"));
        vote_state.insert("latest_review_vote".to_string(), json!("strategy=majority blocks=1"));
        vote_state.insert("latest_review_council_profile".to_string(), json!("yellow_pair"));
        vote_state.insert("latest_review_independence_state".to_string(), json!("failed"));
        vote_state.insert("latest_review_stop_semantics".to_string(), json!("council_required"));
        vote_state
            .insert("latest_review_participants".to_string(), json!([{"reviewer_id": "lead"}]));

        record_review_step_completed(
            &mut trace,
            "review-vote-failed",
            &json!({"phase": "review-vote"}),
            &StepExecutionResult::failure(
                ErrorInfo::new("review_vote_failed", "vote tally crashed"),
                Recoverability::ReplanRequired,
            ),
            &vote_state,
            1,
        );

        let terminal = trace
            .events
            .iter()
            .find(|event| {
                event.event_type == TraceEventType::ReviewTerminalRecorded
                    && event.step_id.as_deref() == Some("review-vote-failed")
            })
            .unwrap();
        assert_eq!(
            terminal.payload.get("review_trigger").and_then(|value| value.as_str()),
            Some("pr_ready")
        );
        assert_eq!(
            terminal.payload.get("review_outcome").and_then(|value| value.as_str()),
            Some("rejected")
        );
        assert_eq!(
            terminal.payload.get("failure_reason").and_then(|value| value.as_str()),
            Some("vote tally crashed")
        );
        assert_eq!(
            terminal.payload.get("council_profile").and_then(|value| value.as_str()),
            Some("yellow_pair")
        );
        assert_eq!(
            terminal.payload.get("independence_state").and_then(|value| value.as_str()),
            Some("failed")
        );
    }

    #[test]
    fn reasoning_trace_records_activation_lifecycle_and_iteration_events() {
        let mut trace =
            ExecutionTrace::new("task-reasoning-trace", "session-reasoning-trace", "goal");
        let activation = ProfileActivationRecord {
            activation_id: "attempt-1-reasoning".to_string(),
            stage_key: "bug-fix:verify".to_string(),
            profile_id: ReasoningProfileId::IndependentPairReview,
            trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
            activation_reason: "Canon posture required stronger challenge".to_string(),
            status: ReasoningActivationStatus::Escalated,
            participants: vec![ParticipantAssignment {
                role_id: "critic".to_string(),
                participant_id: "critic-1".to_string(),
                effective_route: "review:copilot:gpt-5.5".to_string(),
                provider_family: Some("copilot".to_string()),
                context_basis: "reasoning_profile_stage:bug-fix:verify".to_string(),
                prompting_pattern: "critic".to_string(),
                status: ReasoningParticipantStatus::Completed,
                result_summary: Some("critic surfaced a material disagreement".to_string()),
            }],
            budget: ReasoningBudget {
                max_participants: 2,
                max_branches: 1,
                max_debate_rounds: 2,
                max_reflexion_revisions: 1,
                max_calls: 4,
                max_tokens: 4_096,
                max_adjudication_steps: 1,
            },
            posture: None,
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Adjudicated,
                headline: "bounded debate escalated to adjudication".to_string(),
                disagreement_summary: Some(
                    "bounded debate stagnated after repeated objections".to_string(),
                ),
                next_action: Some("continue only with explicit caution".to_string()),
                iterations: vec![
                    ReasoningIterationRecord {
                        iteration_kind: ReasoningIterationKind::DebateRound,
                        iteration_index: 0,
                        participants: vec!["critic-1".to_string(), "reviewer-2".to_string()],
                        summary: "the debate round repeated the same unresolved objection"
                            .to_string(),
                        novelty: false,
                        condition: ReasoningIterationCondition::Stagnated,
                    },
                    ReasoningIterationRecord {
                        iteration_kind: ReasoningIterationKind::ReflexionRevision,
                        iteration_index: 1,
                        participants: vec!["critic-1".to_string()],
                        summary: "the reflexion revision exhausted its novelty budget".to_string(),
                        novelty: false,
                        condition: ReasoningIterationCondition::Exhausted,
                    },
                    ReasoningIterationRecord {
                        iteration_kind: ReasoningIterationKind::AdjudicationStep,
                        iteration_index: 2,
                        participants: vec!["arbiter-1".to_string()],
                        summary: "the arbiter chose the cautious route".to_string(),
                        novelty: true,
                        condition: ReasoningIterationCondition::Completed,
                    },
                ],
            }),
            confidence: Some(ReasoningConfidenceContribution {
                confidence_level: ReasoningConfidenceLevel::Medium,
                basis: vec!["debate_stagnated".to_string()],
                admission_effect: ReasoningAdmissionEffect::Escalate,
                summary: "confidence remained bounded after adjudication".to_string(),
            }),
        };

        record_reasoning_profile_events(&mut trace, "verify", 0, &activation);

        let event_types = trace.events.iter().map(|event| event.event_type).collect::<Vec<_>>();
        for event_type in [
            TraceEventType::ReasoningProfileActivated,
            TraceEventType::ReasoningParticipantStarted,
            TraceEventType::ReasoningParticipantCompleted,
            TraceEventType::ReasoningDisagreementRecorded,
            TraceEventType::ReasoningDebateRoundCompleted,
            TraceEventType::ReasoningReflexionRevisionCompleted,
            TraceEventType::ReasoningAdjudicationRecorded,
            TraceEventType::ReasoningConfidenceRecorded,
            TraceEventType::ReasoningProfileEscalated,
        ] {
            assert!(event_types.contains(&event_type));
        }
    }

    #[test]
    fn review_trace_helpers_cover_vote_fallbacks_and_terminal_dedup() -> Result<(), String> {
        let state = Map::new();
        let default_trigger_input = json!({
            "phase": "review-vote",
            "default_review_trigger": "manual_review"
        });
        if review_trigger_from_state_or_input(&state, &default_trigger_input).as_deref()
            != Some("manual_review")
        {
            return Err("default review trigger fallback should use step input".to_string());
        }

        let output_free_result = StepExecutionResult::success(json!({"summary": "kept"}));
        if review_trigger_from_output_or_state(&output_free_result, &state, &default_trigger_input)
            .as_deref()
            != Some("manual_review")
        {
            return Err("output/state fallback should reach default review trigger".to_string());
        }

        let mut trace =
            ExecutionTrace::new("task-review-fallback", "session-review-fallback", "goal");
        record_review_step_started(
            &mut trace,
            "implement",
            &json!({"phase": "implement"}),
            &state,
            0,
        );
        if !trace.events.is_empty() {
            return Err("non-review phases must not emit review events".to_string());
        }

        let mut vote_state = Map::new();
        vote_state.insert("latest_review_trigger".to_string(), json!("manual_review"));
        vote_state.insert(
            "latest_review_vote".to_string(),
            json!("strategy=unanimous decision=accepted"),
        );
        vote_state
            .insert("latest_review_vote_resolution".to_string(), json!({"decision": "accepted"}));
        let vote_result = StepExecutionResult::success(json!({
            "vote": {"decision": "accepted"}
        }));
        if review_trigger_from_output_or_state(&vote_result, &vote_state, &default_trigger_input)
            .as_deref()
            != Some("manual_review")
        {
            return Err("vote trigger should fall back to latest review trigger".to_string());
        }

        record_review_step_completed(
            &mut trace,
            "review-vote",
            &default_trigger_input,
            &vote_result,
            &vote_state,
            0,
        );
        if trace.events.len() != 1 {
            return Err(format!(
                "expected one vote event before finalize, found {}",
                trace.events.len()
            ));
        }
        let vote_event = trace
            .events
            .iter()
            .find(|event| event.event_type == TraceEventType::ReviewVoteResolved)
            .ok_or_else(|| "missing review vote resolved event".to_string())?;
        if vote_event.payload.get("summary").and_then(|value| value.as_str())
            != Some("strategy=unanimous decision=accepted")
        {
            return Err("vote summary should fall back to latest review vote".to_string());
        }
        if vote_event.payload.get("review_trigger").and_then(|value| value.as_str())
            != Some("manual_review")
        {
            return Err("vote event should retain the resolved review trigger".to_string());
        }
        if vote_event
            .payload
            .get("vote_resolution")
            .and_then(|value| value.get("decision"))
            .and_then(|value| value.as_str())
            != Some("accepted")
        {
            return Err("vote resolution should fall back to output.vote".to_string());
        }

        let mut final_state = vote_state.clone();
        final_state.insert("latest_review_outcome".to_string(), json!("accepted"));
        final_state
            .insert("latest_review_participants".to_string(), json!([{"reviewer_id": "safety"}]));
        let finalize_input = json!({
            "phase": "review-finalize",
            "default_review_trigger": "manual_review"
        });
        let finalize_result = StepExecutionResult::success(json!({
            "review_outcome": "accepted"
        }));
        record_review_step_completed(
            &mut trace,
            "review-finalize",
            &finalize_input,
            &finalize_result,
            &final_state,
            0,
        );
        record_review_step_completed(
            &mut trace,
            "review-finalize",
            &finalize_input,
            &finalize_result,
            &final_state,
            0,
        );

        let terminal_count = trace
            .events
            .iter()
            .filter(|event| {
                event.event_type == TraceEventType::ReviewTerminalRecorded
                    && event.step_id.as_deref() == Some("review-finalize")
                    && event.plan_revision == 0
            })
            .count();
        if terminal_count != 1 {
            return Err(format!("expected one terminal event after dedup, found {terminal_count}"));
        }

        Ok(())
    }

    #[test]
    fn reasoning_trace_helpers_cover_branch_and_interrupted_paths() -> Result<(), String> {
        if reasoning_iteration_event_type(ReasoningIterationKind::Branch).is_some() {
            return Err("branch iterations should not emit dedicated trace events".to_string());
        }
        if reasoning_terminal_event_type(ReasoningActivationStatus::Completed).is_some() {
            return Err(
                "completed activations should not emit terminal reasoning events".to_string()
            );
        }
        if reasoning_terminal_event_type(ReasoningActivationStatus::Blocked)
            != Some(TraceEventType::ReasoningProfileBlocked)
        {
            return Err("blocked activations should map to blocked terminal events".to_string());
        }
        if reasoning_terminal_event_type(ReasoningActivationStatus::Interrupted)
            != Some(TraceEventType::ReasoningProfileInterrupted)
        {
            return Err(
                "interrupted activations should map to interrupted terminal events".to_string()
            );
        }

        let activation = ProfileActivationRecord {
            activation_id: "attempt-2-reasoning".to_string(),
            stage_key: "bug-fix:verify".to_string(),
            profile_id: ReasoningProfileId::IndependentPairReview,
            trigger: ReasoningActivationTrigger::GovernanceEscalation,
            activation_reason: "confidence dropped during verification".to_string(),
            status: ReasoningActivationStatus::Interrupted,
            participants: vec![
                ParticipantAssignment {
                    role_id: "reviewer-pending".to_string(),
                    participant_id: "pending-1".to_string(),
                    effective_route: "review:copilot:gpt-5.5".to_string(),
                    provider_family: Some("copilot".to_string()),
                    context_basis: "reasoning_profile_stage:bug-fix:verify".to_string(),
                    prompting_pattern: "parallel".to_string(),
                    status: ReasoningParticipantStatus::Pending,
                    result_summary: None,
                },
                ParticipantAssignment {
                    role_id: "reviewer-running".to_string(),
                    participant_id: "running-1".to_string(),
                    effective_route: "review:copilot:gpt-5.5".to_string(),
                    provider_family: Some("copilot".to_string()),
                    context_basis: "reasoning_profile_stage:bug-fix:verify".to_string(),
                    prompting_pattern: "parallel".to_string(),
                    status: ReasoningParticipantStatus::Running,
                    result_summary: None,
                },
                ParticipantAssignment {
                    role_id: "reviewer-failed".to_string(),
                    participant_id: "failed-1".to_string(),
                    effective_route: "review:copilot:gpt-5.5".to_string(),
                    provider_family: Some("copilot".to_string()),
                    context_basis: "reasoning_profile_stage:bug-fix:verify".to_string(),
                    prompting_pattern: "parallel".to_string(),
                    status: ReasoningParticipantStatus::Failed,
                    result_summary: None,
                },
                ParticipantAssignment {
                    role_id: "reviewer-omitted".to_string(),
                    participant_id: "omitted-1".to_string(),
                    effective_route: "review:copilot:gpt-5.5".to_string(),
                    provider_family: Some("copilot".to_string()),
                    context_basis: "reasoning_profile_stage:bug-fix:verify".to_string(),
                    prompting_pattern: "parallel".to_string(),
                    status: ReasoningParticipantStatus::Omitted,
                    result_summary: None,
                },
            ],
            budget: ReasoningBudget {
                max_participants: 4,
                max_branches: 1,
                max_debate_rounds: 0,
                max_reflexion_revisions: 0,
                max_calls: 4,
                max_tokens: 4_096,
                max_adjudication_steps: 0,
            },
            posture: None,
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Interrupted,
                headline: "operator paused the bounded reasoning run".to_string(),
                disagreement_summary: None,
                next_action: None,
                iterations: vec![ReasoningIterationRecord {
                    iteration_kind: ReasoningIterationKind::Branch,
                    iteration_index: 0,
                    participants: vec!["pending-1".to_string()],
                    summary: "branch execution stayed local to the paused route".to_string(),
                    novelty: false,
                    condition: ReasoningIterationCondition::Exhausted,
                }],
            }),
            confidence: None,
        };

        let mut trace = ExecutionTrace::new(
            "task-reasoning-interrupted",
            "session-reasoning-interrupted",
            "goal",
        );
        record_reasoning_profile_events(&mut trace, "verify", 1, &activation);

        let participant_started = trace
            .events
            .iter()
            .filter(|event| event.event_type == TraceEventType::ReasoningParticipantStarted)
            .count();
        if participant_started != 4 {
            return Err(format!(
                "expected four participant-started events, found {participant_started}"
            ));
        }
        let participant_completed = trace
            .events
            .iter()
            .filter(|event| event.event_type == TraceEventType::ReasoningParticipantCompleted)
            .count();
        if participant_completed != 2 {
            return Err(format!(
                "expected terminal participant events only for failed and omitted, found {participant_completed}"
            ));
        }
        if !trace.events.iter().any(|event| {
            event.event_type == TraceEventType::ReasoningParticipantStarted
                && event.payload.get("summary").and_then(|value| value.as_str())
                    == Some("route=review:copilot:gpt-5.5 status=pending")
        }) {
            return Err("pending participant summary fallback was not recorded".to_string());
        }
        if !trace.events.iter().any(|event| {
            event.event_type == TraceEventType::ReasoningParticipantCompleted
                && event.payload.get("summary").and_then(|value| value.as_str())
                    == Some("route=review:copilot:gpt-5.5 status=omitted")
        }) {
            return Err("omitted participant summary fallback was not recorded".to_string());
        }
        if trace.events.iter().any(|event| {
            event.event_type == TraceEventType::ReasoningDebateRoundCompleted
                || event.event_type == TraceEventType::ReasoningReflexionRevisionCompleted
        }) {
            return Err(
                "branch iterations should not create debate or reflexion events".to_string()
            );
        }
        if !trace
            .events
            .iter()
            .any(|event| event.event_type == TraceEventType::ReasoningProfileInterrupted)
        {
            return Err("interrupted activation should emit terminal interrupted event".to_string());
        }

        Ok(())
    }
}
