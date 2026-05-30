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

const TRACE_BRIEF_CHAR_LIMIT: usize = 120;
const TRACE_BRIEF_ITEM_LIMIT: usize = 3;
const TRACE_AUDIT_TIMELINE_LIMIT: usize = 10;

pub fn render_trace_summary_brief(
    summary: &TraceSummaryView,
    inspection_target: Option<&str>,
    next_command: &str,
) -> String {
    let mut lines = Vec::new();

    if let Some(inspection_target) = inspection_target {
        lines.push(format!("inspection_target: {inspection_target}"));
    }

    lines.push(format!("goal: {}", preview_trace_brief_text(&summary.goal)));

    lines.extend(trace_input_brief_lines(summary));

    if let Some(routing_summary) = &summary.routing_summary {
        lines.push(routing_summary.clone());
    } else {
        lines.push(format!("route_owner: {}", trace_route_owner(summary)));
    }

    lines.push(render_trace_execution_condition(summary));

    if let Some(summary_line) = trace_summary_brief_line(summary) {
        lines.push(summary_line);
    }

    if let Some(artifacts_line) = trace_artifacts_brief_line(summary) {
        lines.push(artifacts_line);
    }

    lines.extend(trace_step_brief_lines(summary));

    if let Some(clarification_line) = trace_clarification_brief_line(summary) {
        lines.push(clarification_line);
    }

    if let Some(clarification_headline) = &summary.clarification_headline {
        lines.push(format!(
            "clarification_headline: {}",
            preview_trace_brief_text(clarification_headline)
        ));
    }

    if let Some(clarification_prompt) = &summary.clarification_prompt {
        lines.push(format!(
            "clarification_prompt: {}",
            preview_trace_brief_text(clarification_prompt)
        ));
    }

    if !summary.clarification_missing_fields.is_empty() {
        lines.push(format!(
            "clarification_missing_fields: {}",
            preview_trace_brief_items(&summary.clarification_missing_fields)
        ));
    }

    if let Some(review_line) = trace_review_brief_line(summary) {
        lines.push(review_line);
    }

    if let Some(audit_line) = trace_audit_brief_line(summary) {
        lines.push(audit_line);
    }

    if let Some(governance_line) = trace_governance_brief_line(summary) {
        lines.push(governance_line);
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

    if let Some(reasoning_line) = trace_reasoning_brief_line(summary) {
        lines.push(reasoning_line);
    }

    let explanation_projection = explanation_projection_for_trace_summary(summary, next_command);
    lines.extend(explanation_projection_lines(&explanation_projection));

    if !summary.terminal_reason.message.trim().is_empty() {
        lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    }

    lines.push(format!("terminal_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("latest_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("next_command: {next_command}"));
    lines.join("\n")
}

fn trace_input_brief_lines(summary: &TraceSummaryView) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(authored_input_summary) = &summary.authored_input_summary {
        lines.push(format!(
            "authored_input_summary: {}",
            preview_trace_brief_text(authored_input_summary)
        ));
    }

    if !summary.authored_input_sources.is_empty() {
        lines.push(format!(
            "authored_input_sources: {}",
            preview_trace_brief_items(&summary.authored_input_sources)
        ));
    }

    if !summary.authored_input_deduplicated_sources.is_empty() {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            preview_trace_brief_items(&summary.authored_input_deduplicated_sources)
        ));
    }

    lines
}

fn trace_summary_brief_line(summary: &TraceSummaryView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(goal_plan_summary) = &summary.goal_plan_summary {
        parts.push(format!("goal_plan_summary={}", preview_trace_brief_text(goal_plan_summary)));
    }
    if let Some(negotiation_goal_summary) = &summary.negotiation_goal_summary {
        parts.push(format!(
            "negotiation_goal_summary={}",
            preview_trace_brief_text(negotiation_goal_summary)
        ));
    }
    if let Some(negotiation_resolution) = &summary.negotiation_resolution {
        parts.push(format!(
            "negotiation_resolution={}",
            preview_trace_brief_text(negotiation_resolution)
        ));
    }
    if let Some(negotiation_acceptance_boundary) = &summary.negotiation_acceptance_boundary {
        parts.push(format!(
            "negotiation_acceptance_boundary={}",
            preview_trace_brief_text(negotiation_acceptance_boundary)
        ));
    }
    if let Some(latest_step) = summary.executed_steps.last() {
        parts.push(format!(
            "latest_step={} ({}) {}",
            latest_step.step_id,
            step_status_text(latest_step.final_status),
            preview_trace_brief_text(&latest_step.headline)
        ));
    }
    if !summary.terminal_reason.message.trim().is_empty() {
        parts.push(format!(
            "terminal_reason={}",
            preview_trace_brief_text(&summary.terminal_reason.message)
        ));
    }
    if let Some(duration) = summary.duration {
        parts.push(format!("duration_ms={duration}"));
    }

    (!parts.is_empty()).then(|| format!("summary: {}", parts.join("; ")))
}

fn trace_artifacts_brief_line(summary: &TraceSummaryView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(goal_brief_ref) = &summary.goal_brief_ref {
        parts.push(format!("goal_brief_ref={goal_brief_ref}"));
    }
    if let Some(session_plan_brief_ref) = &summary.session_plan_brief_ref {
        parts.push(format!("session_plan_brief_ref={session_plan_brief_ref}"));
    }
    if let Some(run_brief_ref) = &summary.run_brief_ref {
        parts.push(format!("run_brief_ref={run_brief_ref}"));
    }

    parts.push(format!("trace={}", summary.trace_ref));

    if let Some(checkpoint_id) = &summary.latest_checkpoint_id {
        match &summary.latest_checkpoint_scope {
            Some(scope) => parts.push(format!("latest_checkpoint_id={checkpoint_id} ({scope})")),
            None => parts.push(format!("latest_checkpoint_id={checkpoint_id}")),
        }
    }

    if let Some(delegation) = &summary.delegation
        && let Some(packet_id) = &delegation.packet_id
    {
        parts.push(format!("delegation_packet_id={packet_id}"));
    }

    Some(format!("artifacts: {}", parts.join("; ")))
}

fn trace_step_brief_lines(summary: &TraceSummaryView) -> Vec<String> {
    let mut lines = summary
        .executed_steps
        .iter()
        .take(TRACE_BRIEF_ITEM_LIMIT)
        .map(|step| {
            format!(
                "step {} ({}) {} - {}",
                step.step_id,
                step_kind_text(step.step_kind),
                step_status_text(step.final_status),
                preview_trace_brief_text(&step.headline)
            )
        })
        .collect::<Vec<_>>();

    let remaining = summary.executed_steps.len().saturating_sub(lines.len());
    if remaining > 0 {
        lines.push(format!("steps_remaining: {remaining}"));
    }

    lines
}

fn trace_clarification_brief_line(summary: &TraceSummaryView) -> Option<String> {
    if let Some(clarification_prompt) = &summary.clarification_prompt {
        return Some(format!("clarification: {}", preview_trace_brief_text(clarification_prompt)));
    }

    if let Some(clarification_headline) = &summary.clarification_headline {
        return Some(format!(
            "clarification: {}",
            preview_trace_brief_text(clarification_headline)
        ));
    }

    if !summary.clarification_missing_fields.is_empty() {
        return Some(format!(
            "clarification_missing_fields: {}",
            preview_trace_brief_items(&summary.clarification_missing_fields)
        ));
    }

    None
}

fn trace_review_brief_line(summary: &TraceSummaryView) -> Option<String> {
    summary
        .review_timeline
        .last()
        .map(|latest_review| format!("review: {}", preview_trace_brief_text(latest_review)))
}

fn trace_governance_brief_line(summary: &TraceSummaryView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(governance_runtime_state) = &summary.governance_runtime_state {
        parts.push(format!("governance_runtime_state={governance_runtime_state}"));
    }
    if let Some(governance_reason) = &summary.governance_reason {
        parts.push(format!("governance_reason={}", preview_trace_brief_text(governance_reason)));
    }
    if let Some(governance_next_action) = &summary.governance_next_action {
        parts.push(format!(
            "governance_next_action={}",
            preview_trace_brief_text(governance_next_action)
        ));
    }
    if let Some(governance_approval_provenance) = &summary.governance_approval_provenance {
        parts.push(format!(
            "governance_approval_provenance={}",
            preview_trace_brief_text(governance_approval_provenance)
        ));
    }

    if parts.is_empty() {
        summary.governance_timeline.last().map(|latest_governance| {
            format!("governance: {}", preview_trace_brief_text(latest_governance))
        })
    } else {
        Some(format!("governance: {}", parts.join("; ")))
    }
}

fn trace_audit_brief_line(summary: &TraceSummaryView) -> Option<String> {
    let audit = summary.session_audit.as_ref()?;
    if audit.entry_count == 0 {
        return None;
    }

    let latest = audit.entries.last()?;
    let actor_attribution = audit_actor_attribution_fields(&latest.actor);
    Some(format!(
        "audit: count={}; event={}; algorithm={}; actor={}{}; outcome={}",
        audit.entry_count,
        latest.event_label(),
        preview_trace_brief_text(&latest.algorithm.rollup_key()),
        latest.actor.rollup_key(),
        actor_attribution,
        latest.outcome.status.as_str(),
    ))
}

fn audit_actor_attribution_fields(actor: &crate::domain::audit::SessionAuditActor) -> String {
    let mut parts = Vec::new();
    if !actor.participant_routes.is_empty() {
        parts.push(format!("participant_routes={}", actor.participant_routes.join(", ")));
    }
    if actor.mixed_routes {
        parts.push("mixed_routes=true".to_string());
    }

    if parts.is_empty() { String::new() } else { format!("; {}", parts.join("; ")) }
}

fn push_session_audit_lines(
    lines: &mut Vec<String>,
    session_audit: &crate::domain::audit::SessionAuditProjection,
    timeline_limit: Option<usize>,
) {
    lines.push(format!("audit_entry_count: {}", session_audit.entry_count));
    lines.push(format!("audit_session_ref: {}", session_audit.session_id));
    if !session_audit.actor_rollups.is_empty() {
        lines.push(format!(
            "audit_actors: {}",
            preview_trace_brief_items(
                &session_audit
                    .actor_rollups
                    .iter()
                    .map(|rollup| format!("{} ({})", rollup.key, rollup.count))
                    .collect::<Vec<_>>()
            )
        ));
    }
    if !session_audit.algorithm_rollups.is_empty() {
        lines.push(format!(
            "audit_algorithms: {}",
            preview_trace_brief_items(
                &session_audit
                    .algorithm_rollups
                    .iter()
                    .map(|rollup| format!("{} ({})", rollup.key, rollup.count))
                    .collect::<Vec<_>>()
            )
        ));
    }
    if !session_audit.outcome_rollups.is_empty() {
        lines.push(format!(
            "audit_outcomes: {}",
            preview_trace_brief_items(
                &session_audit
                    .outcome_rollups
                    .iter()
                    .map(|rollup| format!("{} ({})", rollup.key, rollup.count))
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(latest) = session_audit.entries.last() {
        let actor_attribution = audit_actor_attribution_fields(&latest.actor);
        lines.push(format!(
            "audit_latest: event={} algorithm={} actor={}{} outcome={} message={}",
            latest.event_label(),
            latest.algorithm.rollup_key(),
            latest.actor.rollup_key(),
            actor_attribution,
            latest.outcome.status.as_str(),
            latest.message
        ));
    }
    lines.push("audit_timeline:".to_string());

    match timeline_limit {
        Some(limit) => {
            let visible_entries =
                session_audit.entries.iter().rev().take(limit).collect::<Vec<_>>();
            for entry in visible_entries.into_iter().rev() {
                let actor_attribution = audit_actor_attribution_fields(&entry.actor);
                lines.push(format!(
                    "{} event={} algorithm={} actor={}{} outcome={} message={}",
                    entry.timestamp,
                    entry.event_label(),
                    entry.algorithm.rollup_key(),
                    entry.actor.rollup_key(),
                    actor_attribution,
                    entry.outcome.status.as_str(),
                    entry.message,
                ));
            }
        }
        None => {
            for entry in &session_audit.entries {
                let actor_attribution = audit_actor_attribution_fields(&entry.actor);
                lines.push(format!(
                    "{} event={} algorithm={} actor={}{} outcome={} message={}",
                    entry.timestamp,
                    entry.event_label(),
                    entry.algorithm.rollup_key(),
                    entry.actor.rollup_key(),
                    actor_attribution,
                    entry.outcome.status.as_str(),
                    entry.message,
                ));
            }
        }
    }
}

fn trace_reasoning_brief_line(summary: &TraceSummaryView) -> Option<String> {
    let reasoning_profile = summary.reasoning_profile.as_ref()?;
    let mut parts = vec![
        format!("reasoning_profile_id={}", reasoning_profile.profile_id),
        format!("reasoning_status={}", reasoning_profile.status.as_str()),
    ];

    if let Some(outcome) = &reasoning_profile.outcome {
        parts.push(format!(
            "reasoning_contribution={}",
            preview_trace_brief_text(&outcome.headline)
        ));
        if let Some(next_action) = &outcome.next_action {
            parts.push(format!("reasoning_next_action={}", preview_trace_brief_text(next_action)));
        }
    }

    Some(format!("reasoning: {}", parts.join("; ")))
}

fn preview_trace_brief_text(text: &str) -> String {
    let trimmed = text.trim();
    let char_count = trimmed.chars().count();
    if char_count <= TRACE_BRIEF_CHAR_LIMIT {
        return trimmed.to_string();
    }

    let preview = trimmed.chars().take(TRACE_BRIEF_CHAR_LIMIT - 3).collect::<String>();
    format!("{preview}...")
}

fn preview_trace_brief_items(items: &[String]) -> String {
    let visible = items
        .iter()
        .take(TRACE_BRIEF_ITEM_LIMIT)
        .map(|item| preview_trace_brief_text(item))
        .collect::<Vec<_>>();
    let remaining = items.len().saturating_sub(visible.len());
    if remaining == 0 {
        visible.join(", ")
    } else {
        format!("{} (+{remaining} more)", visible.join(", "))
    }
}

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

    if let Some(goal_brief_ref) = &summary.goal_brief_ref {
        lines.push(format!("goal_brief_ref: {goal_brief_ref}"));
    }

    if let Some(session_plan_brief_ref) = &summary.session_plan_brief_ref {
        lines.push(format!("session_plan_brief_ref: {session_plan_brief_ref}"));
    }

    if let Some(run_brief_ref) = &summary.run_brief_ref {
        lines.push(format!("run_brief_ref: {run_brief_ref}"));
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

    if let Some(session_audit) = &summary.session_audit
        && session_audit.entry_count > 0
    {
        push_session_audit_lines(&mut lines, session_audit, Some(TRACE_AUDIT_TIMELINE_LIMIT));
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

pub fn render_trace_audit_summary(
    summary: &TraceSummaryView,
    inspection_target: &str,
    next_command: &str,
) -> String {
    let mut lines = vec![
        format!("inspection_target: {inspection_target}"),
        format!("trace: {}", summary.trace_ref),
        format!("goal: {}", summary.goal),
    ];

    match &summary.session_audit {
        Some(session_audit) if session_audit.entry_count > 0 => {
            push_session_audit_lines(&mut lines, session_audit, None);
        }
        _ => {
            lines.push("audit_entry_count: 0".to_string());
            lines.push("audit: no session audit entries found".to_string());
        }
    }

    lines.push(format!("latest_status: {}", task_status_text(summary.terminal_status)));
    lines.push(format!("terminal_reason: {}", summary.terminal_reason.message));
    lines.push(format!("next_command: {next_command}"));

    lines.join("\n")
}
