use serde_json::Value;

use super::{
    EXPLANATION_LABEL_REASONING_CONTRIBUTION, EXPLANATION_LABEL_REASONING_FALLBACK_DISCLOSURE,
    EXPLANATION_LABEL_REASONING_SELECTION_REASON,
};
use crate::domain::reasoning::ProfileActivationRecord;

pub(crate) fn append_reasoning_profile_lines(
    lines: &mut Vec<String>,
    label_prefix: &str,
    reasoning_profile: &ProfileActivationRecord,
) {
    lines.push(format!("{label_prefix}reasoning_profile_id: {}", reasoning_profile.profile_id));
    lines.push(format!("{label_prefix}reasoning_profile_stage: {}", reasoning_profile.stage_key));
    lines.push(format!(
        "{label_prefix}reasoning_profile_status: {}",
        reasoning_profile.status.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_trigger: {}",
        reasoning_profile.trigger.as_str()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_profile_reason: {}",
        reasoning_profile.activation_reason
    ));
    lines.push(format!(
        "{label_prefix}{EXPLANATION_LABEL_REASONING_SELECTION_REASON}: {}",
        reasoning_profile.disclosure_selection_reason()
    ));
    lines.push(format!(
        "{label_prefix}reasoning_budget: participants={} branches={} calls={} adjudication_steps={}",
        reasoning_profile.budget.max_participants,
        reasoning_profile.budget.max_branches,
        reasoning_profile.budget.max_calls,
        reasoning_profile.budget.max_adjudication_steps,
    ));
    if !reasoning_profile.participants.is_empty() {
        lines.push(format!(
            "{label_prefix}reasoning_participants: {}",
            reasoning_profile
                .participants
                .iter()
                .map(|participant| format!(
                    "{}={}",
                    participant.role_id, participant.effective_route
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Some(independence) = &reasoning_profile.independence {
        lines.push(format!(
            "{label_prefix}reasoning_independence_result: {}",
            independence.result.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_independence_reason: {}", independence.reason));
    }
    if let Some(posture) = &reasoning_profile.posture {
        lines.push(format!("{label_prefix}reasoning_posture_contract: {}", posture.contract_line));
        lines.push(format!(
            "{label_prefix}reasoning_posture_admission_priority: {}",
            posture.admission_priority.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_confidence_handoff: {}",
            posture.confidence_handoff_required
        ));
        lines.push(format!(
            "{label_prefix}reasoning_posture_provenance_ref: {}",
            posture.provenance_ref
        ));
    }
    if let Some(outcome) = &reasoning_profile.outcome {
        lines.push(format!("{label_prefix}reasoning_outcome: {}", outcome.outcome_kind.as_str()));
        lines.push(format!("{label_prefix}reasoning_outcome_headline: {}", outcome.headline));
        if let Some(disagreement_summary) = &outcome.disagreement_summary {
            lines.push(format!(
                "{label_prefix}reasoning_disagreement_summary: {disagreement_summary}"
            ));
        }
        if let Some(next_action) = &outcome.next_action {
            lines.push(format!("{label_prefix}reasoning_next_action: {next_action}"));
        }
    }
    if let Some(confidence) = &reasoning_profile.confidence {
        lines.push(format!(
            "{label_prefix}reasoning_confidence_level: {}",
            confidence.confidence_level.as_str()
        ));
        lines.push(format!(
            "{label_prefix}reasoning_confidence_effect: {}",
            confidence.admission_effect.as_str()
        ));
        lines.push(format!("{label_prefix}reasoning_confidence_summary: {}", confidence.summary));
    }
    if let Some(contribution_summary) = reasoning_profile.disclosure_contribution_summary() {
        lines.push(format!(
            "{label_prefix}{EXPLANATION_LABEL_REASONING_CONTRIBUTION}: {contribution_summary}"
        ));
    }
    if let Some(fallback_disclosure) = reasoning_profile.disclosure_fallback_disclosure() {
        lines.push(format!(
            "{label_prefix}{EXPLANATION_LABEL_REASONING_FALLBACK_DISCLOSURE}: {fallback_disclosure}"
        ));
    }
}

pub(crate) fn adaptive_workspace_slice_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let slice = state.get("latest_workspace_slice")?;
    let targets = slice.get("selected_targets")?.as_array()?;
    let targets = targets.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if targets.is_empty() { None } else { Some(targets.join(", ")) }
}

pub(crate) fn adaptive_attempt_lineage_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let lineage = state.get("latest_attempt_lineage")?;
    let current = lineage.get("current_attempt_id")?.as_str()?;
    let transition = lineage.get("transition_kind")?.as_str()?;
    let previous = lineage.get("previous_attempt_id").and_then(Value::as_str);
    previous.map_or_else(
        || Some(format!("{current} ({transition})")),
        |previous| Some(format!("{current} {transition} {previous}")),
    )
}

pub(crate) fn adaptive_candidate_family_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_candidate_family")?.as_str().map(str::to_string)
}

pub(crate) fn adaptive_selection_reason_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_selection_reason")?.as_str().map(str::to_string)
}

pub(crate) fn adaptive_rejected_candidates_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    let rejected = state.get("latest_rejected_candidates")?.as_array()?;
    let rejected = rejected.iter().filter_map(|item| item.as_str()).collect::<Vec<_>>();
    if rejected.is_empty() { None } else { Some(rejected.join(" | ")) }
}

pub(crate) fn adaptive_exhaustion_reason_summary(
    state: &serde_json::Map<String, Value>,
) -> Option<String> {
    state.get("latest_exhaustion_reason")?.as_str().map(str::to_string)
}
