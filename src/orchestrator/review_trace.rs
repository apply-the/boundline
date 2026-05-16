use serde::Serialize;
use serde_json::{Map, Value};

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

fn serialize_payload<T: Serialize>(payload: &T) -> Value {
    serde_json::to_value(payload).unwrap_or(Value::Null)
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

    use super::{record_review_step_completed, record_review_step_started};
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
}
