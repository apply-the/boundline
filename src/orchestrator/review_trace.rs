use serde_json::{Map, Value, json};

use crate::domain::step::StepExecutionResult;
use crate::domain::trace::{ExecutionTrace, TraceEventType};

pub(crate) fn record_review_step_started(
    trace: &mut ExecutionTrace,
    step_id: &str,
    step_input: &Value,
    state: &Map<String, Value>,
    plan_revision: usize,
) {
    if step_phase(step_input) != Some("review") {
        return;
    }

    let review_trigger = review_trigger_from_state_or_input(state, step_input);
    let stage_id = stage_id(step_input);
    let adjudication = step_input.get("adjudication").and_then(Value::as_bool).unwrap_or(false);
    let reviewer_id =
        step_input.get("reviewer_id").and_then(Value::as_str).unwrap_or("unknown-reviewer");

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
            json!({
                "review_trigger": review_trigger,
                "stage_id": stage_id,
                "adjudication": adjudication,
            }),
        );
    }

    let mut payload = Map::new();
    if let Some(review_trigger) = review_trigger {
        payload.insert("review_trigger".to_string(), json!(review_trigger));
    }
    if let Some(stage_id) = stage_id {
        payload.insert("stage_id".to_string(), json!(stage_id));
    }
    payload.insert("reviewer_id".to_string(), json!(reviewer_id));
    payload.insert("adjudication".to_string(), json!(adjudication));
    trace.record_event(
        TraceEventType::ReviewerStarted,
        Some(step_id.to_string()),
        plan_revision,
        Value::Object(payload),
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
        Some("review") => {
            let reviewer_id = result
                .output
                .as_ref()
                .and_then(|output| output.get("reviewer_id"))
                .and_then(Value::as_str)
                .or_else(|| step_input.get("reviewer_id").and_then(Value::as_str))
                .unwrap_or("unknown-reviewer");
            let adjudication = result
                .output
                .as_ref()
                .and_then(|output| output.get("adjudication"))
                .and_then(Value::as_bool)
                .or_else(|| step_input.get("adjudication").and_then(Value::as_bool))
                .unwrap_or(false);

            let mut payload = Map::new();
            if let Some(review_trigger) =
                review_trigger_from_output_or_state(result, state_after, step_input)
            {
                payload.insert("review_trigger".to_string(), json!(review_trigger));
            }
            payload.insert("reviewer_id".to_string(), json!(reviewer_id));
            payload.insert(
                "participation_status".to_string(),
                json!(if result.status == crate::domain::step::ExecutionStatus::Succeeded {
                    "completed"
                } else {
                    "failed"
                }),
            );
            payload.insert("adjudication".to_string(), json!(adjudication));

            if let Some(output) = result.output.as_ref() {
                if let Some(role) = output.get("reviewer_role") {
                    payload.insert("reviewer_role".to_string(), role.clone());
                }
                if let Some(source) = output.get("reviewer_source") {
                    payload.insert("reviewer_source".to_string(), source.clone());
                }
                if let Some(finding) = output.get("finding") {
                    payload.insert("finding".to_string(), finding.clone());
                }
            }

            if let Some(error) = result.error.as_ref() {
                payload.insert("failure_reason".to_string(), json!(error.message.clone()));
                if let Some(review_outcome) = state_after.get("latest_review_outcome") {
                    payload.insert("review_outcome".to_string(), review_outcome.clone());
                }
            }

            trace.record_event(
                TraceEventType::ReviewerCompleted,
                Some(step_id.to_string()),
                plan_revision,
                Value::Object(payload.clone()),
            );

            if adjudication && result.status == crate::domain::step::ExecutionStatus::Succeeded {
                trace.record_event(
                    TraceEventType::ReviewAdjudicated,
                    Some(step_id.to_string()),
                    plan_revision,
                    Value::Object(payload),
                );
            }

            record_review_terminal_if_present(trace, step_id, result, state_after, plan_revision);
        }
        Some("review-vote") => {
            if let Some(output) = result.output.as_ref() {
                let mut payload = Map::new();
                if let Some(review_trigger) = output
                    .get("review_trigger")
                    .and_then(Value::as_str)
                    .or_else(|| state_after.get("latest_review_trigger").and_then(Value::as_str))
                {
                    payload.insert("review_trigger".to_string(), json!(review_trigger));
                }
                if let Some(summary) =
                    output.get("summary").or_else(|| state_after.get("latest_review_vote"))
                {
                    payload.insert("summary".to_string(), summary.clone());
                }
                if let Some(vote_resolution) = output
                    .get("vote_resolution")
                    .or_else(|| output.get("vote"))
                    .or_else(|| state_after.get("latest_review_vote_resolution"))
                {
                    payload.insert("vote_resolution".to_string(), vote_resolution.clone());
                }
                if let Some(review_outcome) = output.get("review_outcome") {
                    payload.insert("review_outcome".to_string(), review_outcome.clone());
                }
                trace.record_event(
                    TraceEventType::ReviewVoteResolved,
                    Some(step_id.to_string()),
                    plan_revision,
                    Value::Object(payload),
                );
            }

            if result.status == crate::domain::step::ExecutionStatus::Failed {
                record_review_terminal_if_present(
                    trace,
                    step_id,
                    result,
                    state_after,
                    plan_revision,
                );
            }
        }
        Some("review-finalize") => {
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
        state_after.get("latest_review_outcome").and_then(Value::as_str).or_else(|| {
            result
                .output
                .as_ref()
                .and_then(|output| output.get("review_outcome"))
                .and_then(Value::as_str)
        });
    let review_trigger =
        state_after.get("latest_review_trigger").and_then(Value::as_str).or_else(|| {
            result
                .output
                .as_ref()
                .and_then(|output| output.get("review_trigger"))
                .and_then(Value::as_str)
        });

    if review_outcome.is_none() {
        return;
    }

    let mut payload = Map::new();
    if let Some(review_trigger) = review_trigger {
        payload.insert("review_trigger".to_string(), json!(review_trigger));
    }
    if let Some(review_outcome) = review_outcome {
        payload.insert("review_outcome".to_string(), json!(review_outcome));
    }
    if let Some(review_vote) = state_after.get("latest_review_vote") {
        payload.insert("review_vote".to_string(), review_vote.clone());
    }
    if let Some(participants) = state_after.get("latest_review_participants") {
        payload.insert("participants".to_string(), participants.clone());
    }
    if let Some(error) = result.error.as_ref() {
        payload.insert("failure_reason".to_string(), json!(error.message.clone()));
    }

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
            Value::Object(payload),
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
            && event.payload.get("review_trigger").and_then(Value::as_str) == Some(review_trigger)
            && event.payload.get("stage_id").and_then(Value::as_str) == stage_id
    })
}

fn review_trigger_from_state_or_input(
    state: &Map<String, Value>,
    step_input: &Value,
) -> Option<String> {
    state
        .get("next_review_trigger")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| state.get("latest_review_trigger").and_then(Value::as_str).map(str::to_string))
        .or_else(|| {
            step_input.get("default_review_trigger").and_then(Value::as_str).map(str::to_string)
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
        .and_then(|output| output.get("review_trigger"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            state_after.get("latest_review_trigger").and_then(Value::as_str).map(str::to_string)
        })
        .or_else(|| review_trigger_from_state_or_input(state_after, step_input))
}

fn step_phase(step_input: &Value) -> Option<&str> {
    step_input.get("phase").and_then(Value::as_str)
}

fn stage_id(step_input: &Value) -> Option<String> {
    step_input.get("stage_id").and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use super::{record_review_step_completed, record_review_step_started};
    use crate::domain::step::StepExecutionResult;
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
            &reviewer_state,
            0,
        );

        let mut final_state = reviewer_state.clone();
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
}
