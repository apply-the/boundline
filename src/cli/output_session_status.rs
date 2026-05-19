use super::cluster::render_cluster_story_lines;
use super::compatibility::render_compatibility_follow_up_lines;
use super::context::{push_advanced_context_lines, push_context_projection_lines};
use super::delight::append_delight_feedback_lines;
use super::explanation::{
    explanation_cognitive_projection_for_session_status, explanation_cognitive_projection_lines,
    explanation_projection_for_session_status, explanation_projection_lines,
};
use super::routing::{
    render_route_config_projection, render_session_projection_prefix,
    route_config_projection_for_status_view, session_route_owner,
};
use super::runtime::append_reasoning_profile_lines;
use super::*;
use crate::domain::follow_through::FollowThroughProjection;

/// Renders the persisted session view as the operator-facing status surface.
pub fn render_session_status(view: &SessionStatusView) -> String {
    let mut lines = vec![
        format!("session_id: {}", view.session_id),
        format!("workspace_ref: {}", view.workspace_ref),
    ];

    if let Some(goal) = &view.goal {
        lines.push(format!("goal: {goal}"));
    }

    if let Some(negotiation_goal_summary) = &view.negotiation_goal_summary {
        lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
    }

    if let Some(negotiation_resolution) = &view.negotiation_resolution {
        lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
    }

    if let Some(negotiation_acceptance_boundary) = &view.negotiation_acceptance_boundary {
        lines.push(format!("negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"));
    }

    lines.extend(render_session_projection_prefix(view).lines().map(str::to_string));
    lines.push(format!("route_owner: {}", session_route_owner(view)));

    if let Some(route_config_projection) =
        render_route_config_projection(route_config_projection_for_status_view(view))
    {
        lines.push(route_config_projection);
    }

    if let Some(continuity_authority) = view.continuity_authority {
        lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    }

    if let Some(delegation) = &view.delegation {
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

    if let Some(compatibility_follow_up) = &view.compatibility_follow_up {
        lines.extend(render_compatibility_follow_up_lines(
            compatibility_follow_up,
            "compatibility_routing",
            "compatibility_execution_condition",
            "compatibility_follow_up_command",
        ));
    }

    if let Some(cluster_story) = &view.cluster_delivery_story {
        lines.extend(render_cluster_story_lines(cluster_story));
    }

    if let Some(authored_input_summary) = &view.authored_input_summary {
        lines.push(format!("authored_input_summary: {authored_input_summary}"));
    }

    if let Some(authored_input_sources) = &view.authored_input_sources
        && !authored_input_sources.is_empty()
    {
        lines.push(format!("authored_input_sources: {}", authored_input_sources.join(", ")));
    }

    if let Some(authored_input_deduplicated_sources) = &view.authored_input_deduplicated_sources
        && !authored_input_deduplicated_sources.is_empty()
    {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            authored_input_deduplicated_sources.join(", ")
        ));
    }

    push_context_projection_lines(
        &mut lines,
        view.context_summary.as_deref(),
        view.context_credibility.as_deref(),
        view.context_primary_inputs.as_deref().unwrap_or(&[]),
        view.context_provenance.as_deref().unwrap_or(&[]),
        view.context_staleness_reason.as_deref(),
    );

    push_advanced_context_lines(&mut lines, view.advanced_context.as_ref());

    if let Some(clarification_headline) = &view.clarification_headline {
        lines.push(format!("clarification_headline: {clarification_headline}"));
    }

    if let Some(clarification_prompt) = &view.clarification_prompt {
        lines.push(format!("clarification_prompt: {clarification_prompt}"));
    }

    if let Some(clarification_missing_fields) = &view.clarification_missing_fields
        && !clarification_missing_fields.is_empty()
    {
        lines.push(format!(
            "clarification_missing_fields: {}",
            clarification_missing_fields.join(", ")
        ));
    }

    if let Some(requested_governance_runtime) = &view.requested_governance_runtime {
        lines.push(format!("requested_governance_runtime: {requested_governance_runtime}"));
    }

    if let Some(requested_governance_risk) = &view.requested_governance_risk {
        lines.push(format!("requested_governance_risk: {requested_governance_risk}"));
    }

    if let Some(requested_governance_zone) = &view.requested_governance_zone {
        lines.push(format!("requested_governance_zone: {requested_governance_zone}"));
    }

    if let Some(requested_governance_owner) = &view.requested_governance_owner {
        lines.push(format!("requested_governance_owner: {requested_governance_owner}"));
    }

    if let Some(active_flow) = &view.active_flow {
        lines.push(format!("active_flow: {active_flow}"));
    }

    if let Some(flow_state) = &view.flow_state {
        lines.push(format!("flow_state: {flow_state}"));
    }

    if let Some(goal_plan_state) = &view.goal_plan_state {
        lines.push(format!("goal_plan_state: {goal_plan_state}"));
    }

    if let Some(goal_plan_revision) = view.goal_plan_revision {
        lines.push(format!("goal_plan_revision: {goal_plan_revision}"));
    }

    if let Some(planning_rationale) = &view.planning_rationale {
        lines.push(format!("planning_rationale: {planning_rationale}"));
    }

    if let Some(verification_strategy) = &view.verification_strategy {
        lines.push(format!("verification_strategy: {verification_strategy}"));
    }

    if let Some(active_workflow) = &view.active_workflow {
        lines.push(format!("workflow: {active_workflow}"));
    }

    if let Some(workflow_phase) = &view.workflow_phase {
        lines.push(format!("workflow_phase: {workflow_phase}"));
    }

    if let Some(current_stage_id) = &view.current_stage_id {
        lines.push(format!("current_stage: {current_stage_id}"));
    }

    if let (Some(current_stage_index), Some(total_stages)) =
        (view.current_stage_index, view.total_stages)
    {
        lines.push(format!("stage_progress: {}/{}", current_stage_index + 1, total_stages));
    }

    if let Some(plan_revision) = view.plan_revision {
        lines.push(format!("plan_revision: {plan_revision}"));
    }

    if let Some(current_step_index) = view.current_step_index {
        lines.push(format!("current_step_index: {current_step_index}"));
    }

    if let Some(current_step_id) = &view.current_step_id {
        lines.push(format!("current_step_id: {current_step_id}"));
    }

    lines.push(format!("latest_status: {}", session_status_text(view.latest_status)));

    if let Some(execution_path) = &view.execution_path {
        lines.push(format!("execution_path: {execution_path}"));
    }

    if let Some(latest_trace_ref) = &view.latest_trace_ref {
        lines.push(format!("latest_trace_ref: {latest_trace_ref}"));
    }

    if let Some(latest_decision_status) = &view.latest_decision_status {
        lines.push(format!("latest_decision_status: {latest_decision_status}"));
    }

    if let Some(latest_decision_target) = &view.latest_decision_target {
        lines.push(format!("latest_decision_target: {latest_decision_target}"));
    }

    if let Some(latest_changed_files) = &view.latest_changed_files
        && !latest_changed_files.is_empty()
    {
        lines.push(format!("latest_changed_files: {}", latest_changed_files.join(", ")));
    }

    if let Some(latest_checkpoint_id) = &view.latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }

    if let Some(latest_checkpoint_scope) = &view.latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }

    if let Some(latest_checkpoint_restore_command) = &view.latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    if let Some(latest_workspace_slice) = &view.latest_workspace_slice {
        lines.push(format!("latest_workspace_slice: {latest_workspace_slice}"));
    }

    if let Some(latest_selection_headline) = &view.latest_selection_headline {
        lines.push(format!("latest_selection_headline: {latest_selection_headline}"));
    }

    if let Some(latest_candidate_family) = &view.latest_candidate_family {
        lines.push(format!("latest_candidate_family: {latest_candidate_family}"));
    }

    if let Some(latest_selection_reason) = &view.latest_selection_reason {
        lines.push(format!("latest_selection_reason: {latest_selection_reason}"));
    }

    if let Some(latest_rejected_candidates) = &view.latest_rejected_candidates
        && !latest_rejected_candidates.is_empty()
    {
        lines.push(format!(
            "latest_rejected_candidates: {}",
            latest_rejected_candidates.join(" | ")
        ));
    }

    if let Some(latest_attempt_lineage) = &view.latest_attempt_lineage {
        lines.push(format!("latest_attempt_lineage: {latest_attempt_lineage}"));
    }

    if let Some(latest_validation_status) = &view.latest_validation_status {
        lines.push(format!("latest_validation_status: {latest_validation_status}"));
    }

    if let Some(latest_exhaustion_reason) = &view.latest_exhaustion_reason {
        lines.push(format!("latest_exhaustion_reason: {latest_exhaustion_reason}"));
    }

    if let Some(latest_review_trigger) = &view.latest_review_trigger {
        lines.push(format!("latest_review_trigger: {latest_review_trigger}"));
    }

    if let Some(latest_review_vote) = &view.latest_review_vote {
        lines.push(format!("latest_review_vote: {latest_review_vote}"));
    }

    if let Some(latest_review_outcome) = &view.latest_review_outcome {
        lines.push(format!("latest_review_outcome: {latest_review_outcome}"));
    }

    if let Some(latest_review_council_profile) = &view.latest_review_council_profile {
        lines.push(format!("latest_review_council_profile: {latest_review_council_profile}"));
    }

    if let Some(latest_review_independence_state) = &view.latest_review_independence_state {
        lines.push(format!("latest_review_independence_state: {latest_review_independence_state}"));
    }

    if let Some(latest_review_stop_semantics) = &view.latest_review_stop_semantics {
        lines.push(format!("latest_review_stop_semantics: {latest_review_stop_semantics}"));
    }

    if let Some(latest_review_selection_summary) = &view.latest_review_selection_summary {
        lines.push(format!("latest_review_selection_summary: {latest_review_selection_summary}"));
    }

    if let Some(latest_review_headline) = &view.latest_review_headline {
        lines.push(format!("latest_review_headline: {latest_review_headline}"));
    }

    if let Some(latest_governance_stage) = &view.latest_governance_stage {
        lines.push(format!("latest_governance_stage: {latest_governance_stage}"));
    }

    if let Some(latest_governance_runtime) = &view.latest_governance_runtime {
        lines.push(format!("latest_governance_runtime: {latest_governance_runtime}"));
    }

    if let Some(latest_governance_mode) = &view.latest_governance_mode {
        lines.push(format!("latest_governance_mode: {latest_governance_mode}"));
    }

    if let Some(latest_governance_run_ref) = &view.latest_governance_run_ref {
        lines.push(format!("latest_governance_run_ref: {latest_governance_run_ref}"));
    }

    if let Some(latest_governance_state) = &view.latest_governance_state {
        lines.push(format!("latest_governance_state: {latest_governance_state}"));
    }

    if let Some(latest_governance_runtime_state) = &view.latest_governance_runtime_state {
        lines.push(format!("latest_governance_runtime_state: {latest_governance_runtime_state}"));
    }

    if let Some(latest_governance_rollout_profile) = &view.latest_governance_rollout_profile {
        lines.push(format!(
            "latest_governance_rollout_profile: {latest_governance_rollout_profile}"
        ));
    }

    if let Some(latest_governance_reason) = &view.latest_governance_reason {
        lines.push(format!("latest_governance_reason: {latest_governance_reason}"));
    }

    if let Some(latest_governance_contract_lines) = &view.latest_governance_contract_lines
        && !latest_governance_contract_lines.is_empty()
    {
        lines.push(format!(
            "latest_governance_contract_lines: {}",
            latest_governance_contract_lines.join(" | ")
        ));
    }

    if let Some(latest_governance_approval_provenance) = &view.latest_governance_approval_provenance
    {
        lines.push(format!(
            "latest_governance_approval_provenance: {latest_governance_approval_provenance}"
        ));
    }

    if let Some(latest_governance_blocked_reason) = &view.latest_governance_blocked_reason {
        lines.push(format!("latest_governance_blocked_reason: {latest_governance_blocked_reason}"));
    }

    if let Some(latest_governance_packet_ref) = &view.latest_governance_packet_ref {
        lines.push(format!("latest_governance_packet_ref: {latest_governance_packet_ref}"));
    }

    if let Some(latest_governance_packet_source_stage) = &view.latest_governance_packet_source_stage
    {
        lines.push(format!(
            "latest_governance_packet_source_stage: {latest_governance_packet_source_stage}"
        ));
    }

    if let Some(latest_governance_packet_binding_reason) =
        &view.latest_governance_packet_binding_reason
    {
        lines.push(format!(
            "latest_governance_packet_binding_reason: {latest_governance_packet_binding_reason}"
        ));
    }

    if let Some(latest_governance_approval) = &view.latest_governance_approval {
        lines.push(format!("latest_governance_approval: {latest_governance_approval}"));
    }

    if let Some(latest_governance_decision) = &view.latest_governance_decision {
        lines.push(format!("latest_governance_decision: {latest_governance_decision}"));
    }

    if let Some(latest_governance_candidates) = &view.latest_governance_candidates
        && !latest_governance_candidates.is_empty()
    {
        lines.push(format!(
            "latest_governance_candidates: {}",
            latest_governance_candidates.join(", ")
        ));
    }

    if let Some(project_scale_path) = &view.project_scale_path {
        lines.push(format!("project_scale_path: {project_scale_path}"));
    }
    if let Some(project_scale_current_stage) = &view.project_scale_current_stage {
        lines.push(format!("project_scale_current_stage: {project_scale_current_stage}"));
    }
    if let Some(project_scale_next_action) = &view.project_scale_next_action {
        lines.push(format!("project_scale_next_action: {project_scale_next_action}"));
    }
    if let Some(project_scale_checkpoint_refs) = &view.project_scale_checkpoint_refs
        && !project_scale_checkpoint_refs.is_empty()
    {
        lines.push(format!(
            "project_scale_checkpoint_refs: {}",
            project_scale_checkpoint_refs.join(", ")
        ));
    }
    if let Some(latest_voting_trigger) = &view.latest_voting_trigger {
        lines.push(format!("latest_voting_trigger: {latest_voting_trigger}"));
    }
    if let Some(latest_voting_result) = &view.latest_voting_result {
        lines.push(format!("latest_voting_result: {latest_voting_result}"));
    }
    if let Some(latest_voting_adjudication) = &view.latest_voting_adjudication {
        lines.push(format!("latest_voting_adjudication: {latest_voting_adjudication}"));
    }
    if let Some(latest_voting_reviewed_evidence) = &view.latest_voting_reviewed_evidence {
        lines.push(format!("latest_voting_reviewed_evidence: {latest_voting_reviewed_evidence}"));
    }
    if let Some(latest_voting_blocking) = view.latest_voting_blocking {
        lines.push(format!("latest_voting_blocking: {latest_voting_blocking}"));
    }
    if let Some(latest_voting_next_action) = &view.latest_voting_next_action {
        lines.push(format!("latest_voting_next_action: {latest_voting_next_action}"));
    }

    if let Some(governance_next_action) = &view.governance_next_action {
        lines.push(format!("governance_next_action: {governance_next_action}"));
    }
    if let Some(governance_lifecycle_runtime) = &view.governance_lifecycle_runtime {
        lines.push(format!("governance_lifecycle_runtime: {governance_lifecycle_runtime}"));
    }
    if let Some(governance_lifecycle_opt_out) = view.governance_lifecycle_opt_out {
        lines.push(format!("governance_lifecycle_opt_out: {governance_lifecycle_opt_out}"));
    }
    if let Some(governance_lifecycle_mode_selection) = &view.governance_lifecycle_mode_selection {
        lines.push(format!(
            "governance_lifecycle_mode_selection: {governance_lifecycle_mode_selection}"
        ));
    }
    if let Some(governance_lifecycle_selected_mode) = &view.governance_lifecycle_selected_mode {
        lines.push(format!(
            "governance_lifecycle_selected_mode: {governance_lifecycle_selected_mode}"
        ));
    }
    if let Some(reasoning_profile) = &view.latest_reasoning_profile {
        append_reasoning_profile_lines(&mut lines, "latest_", reasoning_profile);
    }

    let follow_through = FollowThroughProjection::from_session_view(view);
    if !follow_through.is_empty() {
        lines.extend(follow_through.projection_lines());
    }

    let explanation_projection = explanation_projection_for_session_status(view);
    lines.extend(explanation_projection_lines(&explanation_projection));
    lines.extend(explanation_cognitive_projection_lines(
        &explanation_cognitive_projection_for_session_status(
            view,
            &explanation_projection.fallback_disclosure,
        ),
    ));
    append_delight_feedback_lines(
        &mut lines,
        view.delight_feedback.as_ref(),
        view.session_started_at,
    );

    if let Some(next_command) = view.next_command.as_ref().or(view.workflow_next_action.as_ref()) {
        lines.push(format!("next_command: {next_command}"));
    }

    lines.push(format!("explanation: {}", view.explanation));
    lines.join("\n")
}
