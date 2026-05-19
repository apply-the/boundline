use super::cluster::render_cluster_story_lines;
use super::context::{push_advanced_context_lines, push_context_projection_lines};
use super::delight::{append_delight_feedback_lines, inspect_closure_lines};
use super::explanation::{
    explanation_cognitive_projection_for_trace_summary, explanation_cognitive_projection_lines,
    explanation_projection_for_trace_summary, explanation_projection_lines,
};
use super::routing::{
    render_route_config_projection, render_trace_execution_condition,
    route_config_projection_for_trace_summary, trace_route_owner,
};
use super::runtime::append_reasoning_profile_lines;
use super::support::{push_governance_display_lines, render_guidance_projection_lines};
use super::*;
use crate::domain::follow_through::FollowThroughProjection;

/// Renders the persisted trace summary as operator-facing text without
/// recomputing planning, routing, or guidance state.
pub fn render_trace_summary(
    summary: &TraceSummaryView,
    inspection_target: &str,
    next_command: &str,
) -> String {
    let mut lines = vec![
        format!("inspection_target: {inspection_target}"),
        format!("trace: {}", summary.trace_ref),
        format!("goal: {}", summary.goal),
    ];

    if let Some(cluster_story) = &summary.cluster_delivery_story {
        lines.extend(render_cluster_story_lines(cluster_story));
    }

    if let Some(routing_summary) = &summary.routing_summary {
        lines.push(routing_summary.clone());
    }

    lines.push(format!("route_owner: {}", trace_route_owner(summary)));
    if let Some(route_config_projection) =
        render_route_config_projection(route_config_projection_for_trace_summary(summary))
    {
        lines.push(route_config_projection);
    }

    lines.push(render_trace_execution_condition(summary));

    if let Some(goal_plan_summary) = &summary.goal_plan_summary {
        lines.push(format!("goal_plan_summary: {goal_plan_summary}"));
    }

    if let Some(negotiation_goal_summary) = &summary.negotiation_goal_summary {
        lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
    }

    if let Some(negotiation_resolution) = &summary.negotiation_resolution {
        lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
    }

    if let Some(negotiation_acceptance_boundary) = &summary.negotiation_acceptance_boundary {
        lines.push(format!("negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"));
    }

    if let Some(authored_input_summary) = &summary.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if !summary.authored_input_sources.is_empty() {
        lines
            .push(format!("authored_input_sources: {}", summary.authored_input_sources.join(", ")));
    }

    if !summary.authored_input_deduplicated_sources.is_empty() {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            summary.authored_input_deduplicated_sources.join(", ")
        ));
    }

    push_context_projection_lines(
        &mut lines,
        summary.context_summary.as_deref(),
        summary.context_credibility.as_deref(),
        &summary.context_primary_inputs,
        &summary.context_provenance,
        summary.context_staleness_reason.as_deref(),
    );

    push_advanced_context_lines(&mut lines, summary.advanced_context.as_ref());

    lines.extend(render_guidance_projection_lines(&summary.guidance_guardian));

    if let Some(clarification_headline) = &summary.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &summary.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if !summary.clarification_missing_fields.is_empty() {
        lines.push(format!(
            "clarification_missing_fields: {}",
            summary.clarification_missing_fields.join(", ")
        ));
    }

    push_governance_display_lines(
        &mut lines,
        summary.requested_governance_runtime.as_deref(),
        summary.requested_governance_risk.as_deref(),
        summary.requested_governance_zone.as_deref(),
        summary.requested_governance_owner.as_deref(),
    );

    if !summary.decision_timeline.is_empty() {
        lines.push("decision_timeline:".to_string());
        lines.extend(summary.decision_timeline.iter().cloned());
    }

    if !summary.failure_evidence.is_empty() {
        lines.push("failure_evidence:".to_string());
        lines.extend(summary.failure_evidence.iter().cloned());
    }

    if !summary.adaptive_evidence.is_empty() {
        lines.push("adaptive_evidence:".to_string());
        lines.extend(summary.adaptive_evidence.iter().cloned());
    }

    if let Some(latest_checkpoint_id) = &summary.latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }

    if let Some(latest_checkpoint_scope) = &summary.latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }

    if let Some(latest_checkpoint_restore_command) = &summary.latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    for step in &summary.executed_steps {
        lines.push(format!(
            "step {} ({}) {} [{} attempt(s)] - {}",
            step.step_id,
            step_kind_text(step.step_kind),
            step_status_text(step.final_status),
            step.attempts,
            step.headline,
        ));
    }

    for recovery in &summary.recovery_events {
        let label = match recovery.event_type {
            TraceEventType::RetryScheduled => "retry",
            TraceEventType::StageRetryScheduled => "stage_retry",
            TraceEventType::Replanned => "replan",
            TraceEventType::StageReplanned => "stage_replan",
            TraceEventType::FlowSelected => "flow",
            TraceEventType::StageTransitioned => "stage",
            TraceEventType::StageFailed => "stage_failure",
            _ => "recovery",
        };
        lines.push(format!("{label}: {}", recovery.trigger));
    }

    lines.extend(summary.governance_timeline.iter().cloned());

    if let Some(governance_runtime_state) = &summary.governance_runtime_state {
        lines.push(format!("governance_runtime_state: {governance_runtime_state}"));
    }

    if let Some(governance_rollout_profile) = &summary.governance_rollout_profile {
        lines.push(format!("governance_rollout_profile: {governance_rollout_profile}"));
    }

    if let Some(governance_reason) = &summary.governance_reason {
        lines.push(format!("governance_reason: {governance_reason}"));
    }

    if let Some(governance_approval_provenance) = &summary.governance_approval_provenance {
        lines.push(format!("governance_approval_provenance: {governance_approval_provenance}"));
    }

    if let Some(governance_next_action) = &summary.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }

    if let Some(reasoning_profile) = &summary.reasoning_profile {
        append_reasoning_profile_lines(&mut lines, "", reasoning_profile);
    }

    if let Some(delegation) = &summary.delegation {
        lines.push(format!("delegation_mode: {}", delegation.mode.as_str()));
        if let Some(packet_id) = &delegation.packet_id {
            lines.push(format!("delegation_packet_id: {packet_id}"));
        }
        if let Some(packet_kind) = delegation.packet_kind {
            lines.push(format!("delegation_packet_kind: {}", packet_kind.as_str()));
        }
        if let Some(packet_state) = delegation.packet_state {
            lines.push(format!("delegation_packet_state: {}", packet_state.as_str()));
        }
        if let Some(target_owner) = &delegation.target_owner {
            lines.push(format!("delegation_target_owner: {target_owner}"));
        }
        lines.push(format!("delegation_headline: {}", delegation.headline));
        lines.push(format!("delegation_evidence_summary: {}", delegation.evidence_summary));
    }

    let explanation_projection = explanation_projection_for_trace_summary(summary, next_command);
    lines.extend(explanation_projection_lines(&explanation_projection));
    lines.extend(explanation_cognitive_projection_lines(
        &explanation_cognitive_projection_for_trace_summary(
            summary,
            next_command,
            &explanation_projection.fallback_disclosure,
        ),
    ));
    append_delight_feedback_lines(
        &mut lines,
        summary.delight_feedback.as_ref(),
        summary.trace_started_at,
    );

    let follow_through = FollowThroughProjection::from_trace_summary(summary, Some(next_command));
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }

    if let Some(inspect_context) = &summary.inspect_context {
        lines.extend(inspect_closure_lines(inspect_context));
    }
    if let Some(inspect_council) = &summary.inspect_council {
        lines.extend(inspect_closure_lines(inspect_council));
    }
    if let Some(inspect_timeline) = &summary.inspect_timeline {
        lines.extend(inspect_closure_lines(inspect_timeline));
    }

    lines.extend(summary.review_timeline.iter().cloned());

    lines.push(format!("terminal_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    lines.push(format!("next_command: {next_command}"));

    if let Some(duration) = summary.duration {
        lines.push(format!("duration_ms: {duration}"));
    }

    lines.join("\n")
}
