//! Timeline and headline formatting helpers for inspect trace summaries.

use serde_json::Value;

use crate::domain::limits::TerminalCondition;
use crate::domain::session::governance_packet_provenance_text;
use crate::domain::task::TerminalReason;
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::TraceEventType;

use super::{
    KEY_ACTION_RESULT, KEY_FAILURE_REASON, KEY_FINDING, KEY_REVIEW_OUTCOME, KEY_REVIEW_TRIGGER,
    KEY_REVIEWER_ID, KEY_REVIEWER_ROLE, KEY_SUMMARY, KEY_TARGET, KEY_VOTE_RESOLUTION,
    UNKNOWN_DECISION_ID, UNKNOWN_TARGET, UNKNOWN_VALIDATION_EXIT_CODE,
};

// Normalize decision-loop events into human-readable timeline lines while
// preserving the persisted decision id, target, rationale, and evidence refs.
pub(super) fn decision_timeline_lines(
    event_type: TraceEventType,
    decision_id: Option<&str>,
    payload: &Value,
) -> Vec<String> {
    let decision_id = decision_id.unwrap_or(UNKNOWN_DECISION_ID);
    let status = payload.get("status").and_then(|value| value.as_str()).unwrap_or("unknown");

    match event_type {
        TraceEventType::DecisionCreated => {
            let selector = payload.get("selector").and_then(|value| value.as_str());
            let decision_type =
                payload.get("decision_type").and_then(|value| value.as_str()).unwrap_or("unknown");
            let target =
                payload.get(KEY_TARGET).and_then(|value| value.as_str()).unwrap_or(UNKNOWN_TARGET);
            let mut lines = vec![match selector {
                Some(selector) => {
                    format!(
                        "decision: {decision_id} {selector} ({decision_type}) -> {target} [{status}]"
                    )
                }
                None => format!("decision: {decision_id} {decision_type} -> {target} [{status}]"),
            }];

            if let Some(selector) = selector {
                lines.push(format!("selector: {selector}"));
            }

            if let Some(rationale) = payload.get("rationale").and_then(|value| value.as_str()) {
                lines.push(format!("rationale: {rationale}"));
            }
            if let Some(expected_outcome) =
                payload.get("expected_outcome").and_then(|value| value.as_str())
            {
                lines.push(format!("expected_outcome: {expected_outcome}"));
                lines.push(format!("verification_intent: {expected_outcome}"));
            }
            if let Some(inputs) = payload.get("evidence_inputs").and_then(|value| value.as_array())
            {
                let inputs = inputs.iter().filter_map(format_evidence_input).collect::<Vec<_>>();
                if !inputs.is_empty() {
                    lines.push(format!("evidence_inputs: {}", inputs.join(", ")));
                }
            }

            lines
        }
        TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed => {
            vec![format!("decision_status: {decision_id} {status}")]
        }
        TraceEventType::DecisionRecovered => {
            let recovery_decision_id = payload
                .get("recovery_decision_id")
                .and_then(|value| value.as_str())
                .unwrap_or(UNKNOWN_DECISION_ID);
            vec![format!("decision_status: {decision_id} {status} via {recovery_decision_id}")]
        }
        _ => Vec::new(),
    }
}

pub(super) fn decision_failure_evidence(
    decision_id: Option<&str>,
    payload: &Value,
) -> Option<String> {
    let decision_id = decision_id.unwrap_or(UNKNOWN_DECISION_ID);
    let target = payload.get(KEY_TARGET).and_then(|value| value.as_str()).unwrap_or(UNKNOWN_TARGET);
    let action_result = payload.get(KEY_ACTION_RESULT)?;
    let typed_result = serde_json::from_value::<ToolResult>(action_result.clone()).ok();
    let message = typed_result
        .as_ref()
        .and_then(|tool_result| {
            first_non_empty(&[Some(tool_result.stderr.as_str()), Some(tool_result.stdout.as_str())])
        })
        .or_else(|| {
            first_non_empty(&[
                action_result.get("stderr").and_then(|value| value.as_str()),
                action_result.get("stdout").and_then(|value| value.as_str()),
            ])
        })?;

    Some(format!("{decision_id} {target}: {message}"))
}

pub(super) fn review_timeline_line(event_type: TraceEventType, payload: &Value) -> Option<String> {
    match event_type {
        TraceEventType::ReviewStarted => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger: {trigger}")),
        TraceEventType::ReviewTriggerIgnored => payload
            .get(KEY_REVIEW_TRIGGER)
            .and_then(|value| value.as_str())
            .map(|trigger| format!("review_trigger_ignored: {trigger}")),
        TraceEventType::ReviewerCompleted => reviewer_line(payload),
        TraceEventType::ReviewCouncilAssembled => payload
            .get("selection_summary")
            .and_then(|value| value.as_str())
            .map(|summary| format!("review_council: {summary}"))
            .or_else(|| {
                payload
                    .get("council_profile")
                    .and_then(|value| value.as_str())
                    .map(|profile| format!("review_council: {profile}"))
            }),
        TraceEventType::ReviewStopSemanticsRecorded => payload
            .get("stop_semantics")
            .and_then(|value| value.as_str())
            .map(|stop_semantics| format!("review_stop_semantics: {stop_semantics}")),
        TraceEventType::ReviewVoteResolved => payload
            .get(KEY_SUMMARY)
            .and_then(|value| value.as_str())
            .map(|summary| format!("review_vote: {summary}"))
            .or_else(|| {
                payload.get(KEY_VOTE_RESOLUTION).map(|resolution| {
                    format!(
                        "review_vote: {}",
                        serde_json::to_string(resolution).unwrap_or_default()
                    )
                })
            }),
        TraceEventType::ReviewAdjudicated => {
            reviewer_line(payload).map(|line| format!("review_adjudication: {line}"))
        }
        TraceEventType::ReviewTerminalRecorded => payload
            .get(KEY_REVIEW_OUTCOME)
            .and_then(|value| value.as_str())
            .map(|outcome| format!("review_outcome: {outcome}"))
            .or_else(|| {
                payload
                    .get(KEY_FAILURE_REASON)
                    .and_then(|value| value.as_str())
                    .map(|reason| format!("review_reason: {reason}"))
            }),
        _ => None,
    }
}

pub(super) fn governance_timeline_line(
    event_type: TraceEventType,
    payload: &Value,
) -> Option<String> {
    match event_type {
        TraceEventType::GovernanceSelected => Some(format!(
            "governance_selected: {} -> {}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload
                .get("selected_runtime")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown-runtime")
        )),
        TraceEventType::GovernanceStarted => Some(format!(
            "governance_started: {}{}{}{}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload
                .get("canon_mode")
                .and_then(|value| value.as_str())
                .map(|mode| format!(" ({mode})"))
                .unwrap_or_default(),
            payload
                .get("run_ref")
                .and_then(|value| value.as_str())
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceDecisionRecorded => payload
            .get("selected_action")
            .and_then(|value| value.as_str())
            .map(|selected_action| format!("governance_decision: {selected_action}"))
            .or_else(|| {
                payload
                    .get("blocked_reason")
                    .and_then(|value| value.as_str())
                    .map(|reason| format!("governance_decision_blocked: {reason}"))
            }),
        TraceEventType::GovernanceAwaitingApproval => Some(format!(
            "governance_awaiting_approval: {} ({}){}{}",
            payload.get("stage_key").and_then(|value| value.as_str()).unwrap_or("unknown-stage"),
            payload.get("approval_state").and_then(|value| value.as_str()).unwrap_or("unknown"),
            payload
                .get("run_ref")
                .and_then(|value| value.as_str())
                .map(|run_ref| format!(" [{run_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceCompleted => Some(format!(
            "governance_completed: {}{}{}",
            payload
                .get("headline")
                .and_then(|value| value.as_str())
                .unwrap_or("governed packet ready"),
            payload
                .get("packet_ref")
                .and_then(|value| value.as_str())
                .map(|packet_ref| format!(" [{packet_ref}]"))
                .unwrap_or_default(),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernanceBlocked => Some(format!(
            "governance_blocked: {}{}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("blocked"),
            governance_packet_provenance_suffix(payload)
        )),
        TraceEventType::GovernancePacketRejected => Some(format!(
            "governance_packet_rejected: {}{}",
            payload.get("reason").and_then(|value| value.as_str()).unwrap_or("packet rejected"),
            governance_packet_provenance_suffix(payload)
        )),
        _ => None,
    }
}

pub(super) fn reviewer_line(payload: &Value) -> Option<String> {
    let reviewer_id = payload.get(KEY_REVIEWER_ID).and_then(|value| value.as_str())?;

    if let Some(finding) = payload.get(KEY_FINDING) {
        let disposition =
            finding.get("disposition").and_then(|value| value.as_str()).unwrap_or("unknown");
        let summary =
            finding.get("summary").and_then(|value| value.as_str()).unwrap_or("review finding");
        let role = payload.get(KEY_REVIEWER_ROLE).and_then(|value| value.as_str());
        return Some(match role {
            Some(role) => format!("reviewer {reviewer_id} ({role}) {disposition}: {summary}"),
            None => format!("reviewer {reviewer_id} {disposition}: {summary}"),
        });
    }

    payload
        .get(KEY_FAILURE_REASON)
        .and_then(|value| value.as_str())
        .map(|reason| format!("reviewer {reviewer_id} failed: {reason}"))
}

pub(super) fn synthesized_in_progress_reason(
    latest_governance_state: Option<&str>,
) -> TerminalReason {
    let message = match latest_governance_state {
        Some("awaiting_approval") => "governance approval is still pending",
        Some("blocked") => "governed work is blocked pending intervention",
        Some("governed_ready") => "governed work is ready for the next bounded step",
        _ => "trace is still in progress",
    };

    TerminalReason::new(TerminalCondition::NoCredibleNextStep, message, None)
}

pub(super) fn success_headline(payload: &Value, attempts: usize) -> String {
    let selection_reason = adaptive_selection_reason(payload);
    if let Some(headline) = payload
        .get("output")
        .and_then(|output| output.get("workspace_slice"))
        .and_then(|slice| slice.get("headline"))
        .and_then(|value| value.as_str())
    {
        return selection_reason.map_or_else(
            || format!("adaptive slice {headline}"),
            |reason| format!("adaptive slice {headline}: {reason}"),
        );
    }

    if let Some(change) = payload
        .get("output")
        .and_then(|output| output.get("change_evidence"))
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
    {
        let path = change.get("path").and_then(|value| value.as_str()).unwrap_or("workspace");
        let before =
            change.get("before_excerpt").and_then(|value| value.as_str()).unwrap_or("before");
        let after = change.get("after_excerpt").and_then(|value| value.as_str()).unwrap_or("after");
        return format!("updated {path} from {before} to {after} after {attempts} attempt(s)");
    }

    if let Some(changed_files) = payload
        .get("output")
        .and_then(|output| output.get("changed_files"))
        .and_then(|value| value.as_array())
    {
        let changed_files =
            changed_files.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
        if !changed_files.is_empty() {
            return format!("updated {} after {attempts} attempt(s)", changed_files.join(", "));
        }
    }

    if let Some(validation) = payload
        .get("output")
        .and_then(|output| output.get("validation"))
        .or_else(|| payload.get("evidence").and_then(|evidence| evidence.get("validation_record")))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let succeeded =
            validation.get("succeeded").and_then(|value| value.as_bool()).unwrap_or(false);
        return format!(
            "validation {} after {attempts} attempt(s) via {command}",
            if succeeded { "passed" } else { "failed" }
        );
    }

    format!("succeeded after {attempts} attempt(s)")
}

pub(super) fn failure_headline(payload: &Value, attempts: usize) -> String {
    if let Some(exhaustion_reason) = payload
        .get("evidence")
        .and_then(|evidence| evidence.get("exhaustion_reason"))
        .and_then(|value| value.as_str())
    {
        return format!(
            "adaptive repair exhausted after {attempts} attempt(s): {exhaustion_reason}"
        );
    }

    if let Some(validation) =
        payload.get("evidence").and_then(|evidence| evidence.get("validation_record"))
    {
        let command =
            validation.get("command").and_then(|value| value.as_str()).unwrap_or("validation");
        let exit_code = validation
            .get("exit_code")
            .and_then(|value| value.as_i64())
            .unwrap_or(UNKNOWN_VALIDATION_EXIT_CODE);
        return adaptive_selection_reason(payload).map_or_else(
            || {
                format!(
                    "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code})"
                )
            },
            |reason| {
                format!(
                    "validation failed after {attempts} attempt(s) via {command} (exit_code={exit_code}) while {reason}"
                )
            },
        );
    }

    format!("failed after {attempts} attempt(s)")
}

pub(super) fn adaptive_evidence_lines(payload: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    let selection =
        payload.get("output").and_then(|output| output.get("selection_evidence")).or_else(|| {
            payload.get("evidence").and_then(|evidence| evidence.get("selection_evidence"))
        });

    if let Some(selection) = selection {
        if let Some(candidate_family) =
            selection.get("candidate_family").and_then(|value| value.as_str())
        {
            lines.push(format!("candidate_family: {candidate_family}"));
        }

        if let Some(reason) = selection.get("reason").and_then(|value| value.as_str()) {
            lines.push(format!("selection_reason: {reason}"));
        }

        if let Some(rejected_candidates) =
            selection.get("rejected_candidates").and_then(|value| value.as_array())
        {
            lines.extend(
                rejected_candidates
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(|item| format!("rejected_candidate: {item}")),
            );
        }
    }

    if let Some(exhaustion_reason) = payload
        .get("evidence")
        .and_then(|evidence| evidence.get("exhaustion_reason"))
        .and_then(|value| value.as_str())
    {
        lines.push(format!("adaptive_exhaustion: {exhaustion_reason}"));
    }

    lines
}

fn adaptive_selection_reason(payload: &Value) -> Option<String> {
    payload
        .get("output")
        .and_then(|output| output.get("selection_evidence"))
        .or_else(|| payload.get("evidence").and_then(|evidence| evidence.get("selection_evidence")))
        .and_then(|selection| selection.get("reason"))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn governance_packet_provenance_suffix(payload: &Value) -> String {
    governance_packet_provenance_text(
        payload.get("packet_source_stage").and_then(|value| value.as_str()),
        payload.get("packet_binding_reason").and_then(|value| value.as_str()),
    )
    .map(|provenance| format!(" from {provenance}"))
    .unwrap_or_default()
}

fn first_non_empty<'a>(values: &[Option<&'a str>]) -> Option<&'a str> {
    values.iter().filter_map(|value| *value).find(|value| !value.trim().is_empty())
}

fn format_evidence_input(value: &Value) -> Option<String> {
    let kind = value.get("kind")?.as_str()?;
    let reference = value.get("reference")?.as_str()?;
    Some(format!("{kind}:{reference}"))
}
