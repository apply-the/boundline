//! Projection and payload-folding helpers for inspect trace summaries.

use serde_json::Value;

use crate::domain::guidance::{GuardianFinding, GuidanceGuardianProjection};
use crate::domain::trace::TraceEventType;

use super::governance_timeline_line;

pub(super) fn merge_guidance_projection_from_payload(
    projection: &mut GuidanceGuardianProjection,
    payload: &Value,
) {
    let Some(object) = payload.as_object() else {
        return;
    };

    let loaded_packs = string_array_field(object, "loaded_packs");
    if !loaded_packs.is_empty() {
        projection.loaded_packs = loaded_packs;
    }

    if projection.capability_resolution_summary.is_none() {
        projection.capability_resolution_summary = object
            .get("capability_resolution_summary")
            .and_then(|value| value.as_str().map(str::to_string));
    }
    if projection.catalog_validation_findings.is_empty() {
        projection.catalog_validation_findings =
            string_array_field(object, "catalog_validation_findings");
    }
    if projection.loaded_guidance_sources.is_empty() {
        projection.loaded_guidance_sources = string_array_field(object, "loaded_guidance_sources");
    }
    if projection.skipped_guidance_sources.is_empty() {
        projection.skipped_guidance_sources =
            string_array_field(object, "skipped_guidance_sources");
    }

    let loaded_guardian_sources = string_array_field(object, "loaded_guardian_sources");
    if !loaded_guardian_sources.is_empty() {
        projection.loaded_guardian_sources = loaded_guardian_sources;
    }

    let skipped_guardian_sources = string_array_field(object, "skipped_guardian_sources");
    if !skipped_guardian_sources.is_empty() {
        projection.skipped_guardian_sources = skipped_guardian_sources;
    }

    let guardian_timeline = string_array_field(object, "guardian_timeline");
    if !guardian_timeline.is_empty() {
        projection.guardian_timeline = guardian_timeline;
    }

    if let Some(summary) =
        object.get("guardian_findings_summary").and_then(|value| value.as_str().map(str::to_string))
    {
        projection.guardian_findings_summary = Some(summary);
    }

    if let Some(findings) = object
        .get("guardian_findings")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<GuardianFinding>>(value).ok())
        && !findings.is_empty()
    {
        projection.guardian_findings = findings;
    }

    let guardian_degradations = string_array_field(object, "guardian_degradations");
    if !guardian_degradations.is_empty() {
        projection.guardian_degradations = guardian_degradations;
    }

    if let Some(outcome) =
        object.get("guardian_blocking_outcome").and_then(|value| value.as_str().map(str::to_string))
    {
        projection.guardian_blocking_outcome = Some(outcome);
    }
}

#[derive(Debug, Default)]
pub(super) struct TraceInputProjection {
    pub(super) authored_input_summary: Option<String>,
    pub(super) authored_input_sources: Vec<String>,
    pub(super) authored_input_deduplicated_sources: Vec<String>,
    pub(super) clarification_headline: Option<String>,
    pub(super) clarification_prompt: Option<String>,
    pub(super) clarification_missing_fields: Vec<String>,
    pub(super) requested_governance_runtime: Option<String>,
    pub(super) requested_governance_risk: Option<String>,
    pub(super) requested_governance_zone: Option<String>,
    pub(super) requested_governance_owner: Option<String>,
    pub(super) negotiation_goal_summary: Option<String>,
    pub(super) negotiation_resolution: Option<String>,
    pub(super) negotiation_acceptance_boundary: Option<String>,
}

impl TraceInputProjection {
    pub(super) fn merge_task_started_payload(&mut self, payload: &Value) {
        if self.authored_input_summary.is_none() {
            self.authored_input_summary =
                nested_payload_string(payload, "input", "authored_input_summary");
        }
        if self.authored_input_sources.is_empty() {
            self.authored_input_sources =
                nested_payload_string_array(payload, "input", "authored_input_sources");
        }
        if self.authored_input_deduplicated_sources.is_empty() {
            self.authored_input_deduplicated_sources = nested_payload_string_array(
                payload,
                "input",
                "authored_input_deduplicated_sources",
            );
        }
        if self.clarification_headline.is_none() {
            self.clarification_headline =
                nested_payload_string(payload, "input", "clarification_headline");
        }
        if self.clarification_prompt.is_none() {
            self.clarification_prompt =
                nested_payload_string(payload, "input", "clarification_prompt");
        }
        if self.clarification_missing_fields.is_empty() {
            self.clarification_missing_fields =
                nested_payload_string_array(payload, "input", "clarification_missing_fields");
        }
        if self.requested_governance_runtime.is_none() {
            self.requested_governance_runtime =
                nested_payload_string(payload, "input", "requested_governance_runtime");
        }
        if self.requested_governance_risk.is_none() {
            self.requested_governance_risk =
                nested_payload_string(payload, "input", "requested_governance_risk");
        }
        if self.requested_governance_zone.is_none() {
            self.requested_governance_zone =
                nested_payload_string(payload, "input", "requested_governance_zone");
        }
        if self.requested_governance_owner.is_none() {
            self.requested_governance_owner =
                nested_payload_string(payload, "input", "requested_governance_owner");
        }
        if self.negotiation_goal_summary.is_none() {
            self.negotiation_goal_summary =
                nested_payload_string(payload, "input", "negotiation_goal_summary");
        }
        if self.negotiation_resolution.is_none() {
            self.negotiation_resolution =
                nested_payload_string(payload, "input", "negotiation_resolution");
        }
        if self.negotiation_acceptance_boundary.is_none() {
            self.negotiation_acceptance_boundary =
                nested_payload_string(payload, "input", "negotiation_acceptance_boundary");
        }
    }

    pub(super) fn merge_goal_plan_payload(&mut self, payload: &Value) {
        if self.negotiation_goal_summary.is_none() {
            self.negotiation_goal_summary = payload_string(payload, "negotiation_goal_summary");
        }
        if self.negotiation_resolution.is_none() {
            self.negotiation_resolution = payload_string(payload, "negotiation_resolution");
        }
        if self.negotiation_acceptance_boundary.is_none() {
            self.negotiation_acceptance_boundary =
                payload_string(payload, "negotiation_acceptance_boundary");
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct TraceContextProjection {
    pub(super) summary: Option<String>,
    pub(super) credibility: Option<String>,
    pub(super) primary_inputs: Vec<String>,
    pub(super) provenance: Vec<String>,
    pub(super) staleness_reason: Option<String>,
}

impl TraceContextProjection {
    pub(super) fn merge_task_started_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = nested_payload_string(payload, "input", "context_summary");
        }
        if self.credibility.is_none() {
            self.credibility = nested_payload_string(payload, "input", "context_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs =
                nested_payload_string_array(payload, "input", "context_primary_inputs");
        }
        if self.provenance.is_empty() {
            self.provenance = nested_payload_string_array(payload, "input", "context_provenance");
        }
        if self.staleness_reason.is_none() {
            self.staleness_reason =
                nested_payload_string(payload, "input", "context_staleness_reason");
        }
    }

    pub(super) fn merge_goal_plan_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = payload_string(payload, "context_summary");
        }
        if self.credibility.is_none() {
            self.credibility = payload_string(payload, "context_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs = payload_string_array(payload, "context_primary_inputs");
        }
        if self.provenance.is_empty() {
            self.provenance = payload_string_array(payload, "context_provenance");
        }
        if self.staleness_reason.is_none() {
            self.staleness_reason = payload_string(payload, "context_staleness_reason");
        }
    }

    pub(super) fn merge_governance_payload(&mut self, payload: &Value) {
        if self.summary.is_none() {
            self.summary = payload_string(payload, "canon_memory_summary");
        }
        if self.credibility.is_none() {
            self.credibility = payload_string(payload, "canon_memory_credibility");
        }
        if self.primary_inputs.is_empty() {
            self.primary_inputs = payload_string_array(payload, "document_refs");
        }

        self.push_optional_line(
            payload_string(payload, "canon_memory_summary")
                .map(|value| format!("canon_memory: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_compatibility")
                .map(|value| format!("canon_memory_compatibility: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_run_ref")
                .or_else(|| payload_string(payload, "run_ref"))
                .map(|value| format!("canon_memory_run_ref: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_packet_ref")
                .or_else(|| payload_string(payload, "packet_ref"))
                .map(|value| format!("canon_memory_packet: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_memory_reason_code")
                .map(|value| format!("canon_memory_reason: {value}")),
        );
        self.push_optional_line(
            payload_string(payload, "canon_next_action")
                .map(|value| format!("canon_memory_next_action: {value}")),
        );

        for line in payload_string_array(payload, "authority_provenance_lines") {
            self.push_line(line);
        }
        for line in payload_string_array(payload, "adaptive_provenance_lines") {
            self.push_line(line);
        }

        if self.staleness_reason.is_none()
            && payload_string(payload, "canon_memory_credibility")
                .is_some_and(|credibility| credibility != "credible")
        {
            self.staleness_reason = payload_string(payload, "reason")
                .or_else(|| payload_string(payload, "canon_memory_summary"));
        }
    }

    fn push_optional_line(&mut self, line: Option<String>) {
        if let Some(line) = line {
            self.push_line(line);
        }
    }

    fn push_line(&mut self, line: String) {
        if !self.provenance.contains(&line) {
            self.provenance.push(line);
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct TraceGovernanceProjection {
    pub(super) latest_state: Option<String>,
    pub(super) next_action: Option<String>,
    pub(super) runtime_state: Option<String>,
    pub(super) rollout_profile: Option<String>,
    pub(super) reason: Option<String>,
    pub(super) approval_provenance: Option<String>,
    pub(super) timeline: Vec<String>,
}

impl TraceGovernanceProjection {
    pub(super) fn merge_event(
        &mut self,
        event_type: TraceEventType,
        payload: &Value,
        context_projection: &mut TraceContextProjection,
    ) {
        match event_type {
            TraceEventType::GovernanceAwaitingApproval => {
                self.latest_state = Some("awaiting_approval".to_string());
            }
            TraceEventType::GovernanceCompleted => {
                self.latest_state = Some("governed_ready".to_string());
            }
            TraceEventType::GovernanceBlocked | TraceEventType::GovernancePacketRejected => {
                self.latest_state = Some("blocked".to_string());
            }
            _ => {}
        }

        context_projection.merge_governance_payload(payload);

        if self.next_action.is_none() {
            self.next_action = payload_string(payload, "canon_next_action");
        }
        if self.runtime_state.is_none() {
            self.runtime_state = payload_string(payload, "latest_governance_runtime_state");
        }
        if self.rollout_profile.is_none() {
            self.rollout_profile = payload_string(payload, "latest_governance_rollout_profile");
        }
        if self.reason.is_none() {
            self.reason = payload_string(payload, "latest_governance_reason");
        }
        if self.approval_provenance.is_none() {
            self.approval_provenance =
                payload_string(payload, "latest_governance_approval_provenance");
        }
        if let Some(line) = governance_timeline_line(event_type, payload) {
            self.timeline.push(line);
        }
    }
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload.get(key).and_then(|value| value.as_str().map(str::to_string))
}

fn payload_string_array(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn nested_payload_string(payload: &Value, container: &str, key: &str) -> Option<String> {
    payload
        .get(container)
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str().map(str::to_string))
}

fn nested_payload_string_array(payload: &Value, container: &str, key: &str) -> Vec<String> {
    payload
        .get(container)
        .and_then(|value| value.get(key))
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

// Extract a string array from a JSON payload without failing the overall
// inspection path when older or partial payloads omit the key.
pub(super) fn string_array_field(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Vec<String> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
