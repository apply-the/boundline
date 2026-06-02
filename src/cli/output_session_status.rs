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
    route_config_projection_for_status_view, session_execution_condition_parts,
    session_route_owner,
};
use super::runtime::{
    append_reasoning_profile_lines, framework_adapter_hook_dispatch_lines,
    framework_adapter_stage_failure_lines, framework_adapter_stage_routing_lines,
    framework_adapter_status_lines,
};
use super::support::push_governance_display_lines;
use super::{SessionStatusView, session_status_text};
use crate::domain::follow_through::FollowThroughProjection;
use crate::domain::governance::planning_stage_brief_ref;

const SESSION_BRIEF_CHAR_LIMIT: usize = 120;
const SESSION_BRIEF_ITEM_LIMIT: usize = 3;

/// Renders the default concise session brief for human-readable CLI output.
pub fn render_session_status_brief(view: &SessionStatusView) -> String {
    let mut lines = Vec::new();

    push_session_overview_brief_lines(&mut lines, view);

    lines.extend(session_input_brief_lines(view));

    push_context_projection_lines(
        &mut lines,
        view.context_summary.as_deref(),
        view.context_credibility.as_deref(),
        view.context_primary_inputs.as_deref().unwrap_or(&[]),
        &[],
        view.context_staleness_reason.as_deref(),
    );

    push_advanced_context_lines(&mut lines, view.advanced_context.as_ref());

    lines.extend(render_session_projection_prefix(view).lines().map(str::to_string));
    lines.extend(framework_adapter_status_lines(&view.workspace_ref));
    lines.extend(framework_adapter_stage_routing_lines(
        view.latest_framework_adapter_stage_routing.as_ref(),
    ));
    lines.extend(framework_adapter_hook_dispatch_lines(
        view.latest_framework_adapter_hook_dispatch.as_ref(),
    ));
    lines.extend(framework_adapter_stage_failure_lines(
        view.latest_framework_adapter_stage_failure.as_ref(),
    ));

    push_session_progress_brief_lines(&mut lines, view);

    if let Some(summary_line) = session_summary_brief_line(view) {
        lines.push(summary_line);
    }

    if let Some(artifacts_line) = session_artifacts_brief_line(view) {
        lines.push(artifacts_line);
    }

    lines.extend(session_clarification_brief_lines(view));

    if let Some(review_line) = session_review_brief_line(view) {
        lines.push(review_line);
    }

    if let Some(governance_line) = session_governance_brief_line(view) {
        lines.push(governance_line);
    }

    push_session_governance_detail_lines(&mut lines, view);

    if let Some(reasoning_line) = session_reasoning_brief_line(view) {
        lines.push(reasoning_line);
    }

    push_session_footer_brief_lines(&mut lines, view);
    lines.join("\n")
}

fn push_session_overview_brief_lines(lines: &mut Vec<String>, view: &SessionStatusView) {
    push_optional_preview_text_line(lines, "goal", view.goal.as_deref());
    push_optional_preview_text_line(
        lines,
        "negotiation_goal_summary",
        view.negotiation_goal_summary.as_deref(),
    );
    push_optional_preview_text_line(
        lines,
        "negotiation_resolution",
        view.negotiation_resolution.as_deref(),
    );
    push_optional_preview_text_line(
        lines,
        "negotiation_acceptance_boundary",
        view.negotiation_acceptance_boundary.as_deref(),
    );
}

fn push_session_progress_brief_lines(lines: &mut Vec<String>, view: &SessionStatusView) {
    if let Some(continuity_authority) = view.continuity_authority {
        lines.push(format!("continuity_authority: {}", continuity_authority.as_str()));
    }

    if let Some(compatibility_follow_up) = &view.compatibility_follow_up {
        lines.extend(render_compatibility_follow_up_lines(
            compatibility_follow_up,
            "compatibility_routing",
            "compatibility_follow_up",
            "compatibility_follow_up_command",
        ));
    }

    if let Some(cluster_story) = &view.cluster_delivery_story {
        lines.extend(render_cluster_story_lines(cluster_story));
    }

    push_optional_line(lines, "active_flow", view.active_flow.as_deref());
    push_optional_line(lines, "current_stage", view.current_stage_id.as_deref());
    if let (Some(current_stage_index), Some(total_stages)) =
        (view.current_stage_index, view.total_stages)
    {
        lines.push(format!("stage_progress: {}/{}", current_stage_index + 1, total_stages));
    }
    push_optional_line(lines, "execution_path", view.execution_path.as_deref());
    if let Some(plan_revision) = view.plan_revision {
        lines.push(format!("plan_revision: {plan_revision}"));
    }
    if let Some(current_step_index) = view.current_step_index {
        lines.push(format!("current_step_index: {current_step_index}"));
    }
    push_optional_line(lines, "current_step_id", view.current_step_id.as_deref());
    push_optional_line(lines, "latest_validation_status", view.latest_validation_status.as_deref());
    push_optional_line(lines, "latest_trace_ref", view.latest_trace_ref.as_deref());
    if let Some(latest_changed_files) = &view.latest_changed_files
        && !latest_changed_files.is_empty()
    {
        lines.push(format!("latest_changed_files: {}", latest_changed_files.join(", ")));
    }
}

fn push_session_governance_detail_lines(lines: &mut Vec<String>, view: &SessionStatusView) {
    push_optional_line(lines, "latest_governance_stage", view.latest_governance_stage.as_deref());
    push_optional_line(
        lines,
        "latest_governance_runtime",
        view.latest_governance_runtime.as_deref(),
    );
    push_optional_line(lines, "latest_governance_mode", view.latest_governance_mode.as_deref());
    push_optional_line(
        lines,
        "latest_governance_run_ref",
        view.latest_governance_run_ref.as_deref(),
    );
    push_optional_line(lines, "latest_governance_state", view.latest_governance_state.as_deref());
    push_optional_line(
        lines,
        "latest_governance_runtime_state",
        view.latest_governance_runtime_state.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_rollout_profile",
        view.latest_governance_rollout_profile.as_deref(),
    );
    push_optional_line(lines, "latest_governance_reason", view.latest_governance_reason.as_deref());
    if let Some(latest_governance_contract_lines) = &view.latest_governance_contract_lines
        && !latest_governance_contract_lines.is_empty()
    {
        lines.push(format!(
            "latest_governance_contract_lines: {}",
            latest_governance_contract_lines.join(" | ")
        ));
    }
    push_optional_line(
        lines,
        "latest_governance_approval_provenance",
        view.latest_governance_approval_provenance.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_blocked_reason",
        view.latest_governance_blocked_reason.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_packet_ref",
        view.latest_governance_packet_ref.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_packet_source_stage",
        view.latest_governance_packet_source_stage.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_packet_binding_reason",
        view.latest_governance_packet_binding_reason.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_approval",
        view.latest_governance_approval.as_deref(),
    );
    push_optional_line(
        lines,
        "latest_governance_decision",
        view.latest_governance_decision.as_deref(),
    );
    push_optional_line(
        lines,
        "governance_lifecycle_runtime",
        view.governance_lifecycle_runtime.as_deref(),
    );
    push_optional_line(
        lines,
        "governance_lifecycle_mode_selection",
        view.governance_lifecycle_mode_selection.as_deref(),
    );
    push_optional_line(
        lines,
        "governance_lifecycle_selected_mode",
        view.governance_lifecycle_selected_mode.as_deref(),
    );
    if let Some(governance_lifecycle_selected_mode_sequence) =
        &view.governance_lifecycle_selected_mode_sequence
        && !governance_lifecycle_selected_mode_sequence.is_empty()
    {
        lines.push(format!(
            "governance_lifecycle_selected_mode_sequence: {}",
            governance_lifecycle_selected_mode_sequence.join(", ")
        ));
    }
    if let Some(latest_governance_candidates) = &view.latest_governance_candidates
        && !latest_governance_candidates.is_empty()
    {
        lines.push(format!(
            "latest_governance_candidates: {}",
            latest_governance_candidates.join(", ")
        ));
    }
}

fn push_session_footer_brief_lines(lines: &mut Vec<String>, view: &SessionStatusView) {
    lines.push(format!("latest_status: {}", session_status_text(view.latest_status)));
    push_optional_line(lines, "next_command", view.next_command.as_deref());
    lines.push(format!("explanation: {}", preview_session_brief_text(&view.explanation)));
}

fn session_input_brief_lines(view: &SessionStatusView) -> Vec<String> {
    let mut lines = Vec::new();

    if let Some(summary) =
        view.authored_input_summary.as_deref().filter(|summary| !summary.trim().is_empty())
    {
        lines.push(format!("authored_input_summary: {}", preview_session_brief_text(summary)));
    }

    if let Some(sources) = &view.authored_input_sources
        && !sources.is_empty()
    {
        lines.push(format!("authored_input_sources: {}", preview_session_brief_items(sources)));
    }

    if let Some(deduplicated_sources) = &view.authored_input_deduplicated_sources
        && !deduplicated_sources.is_empty()
    {
        lines.push(format!(
            "authored_input_deduplicated_sources: {}",
            preview_session_brief_items(deduplicated_sources)
        ));
    }

    lines
}

fn session_summary_brief_line(view: &SessionStatusView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(active_flow) = &view.active_flow {
        parts.push(format!("active_flow={active_flow}"));
    }
    if let Some(flow_state) = &view.flow_state {
        parts.push(format!("flow_state={}", preview_session_brief_text(flow_state)));
    }
    if let Some(goal_plan_state) = &view.goal_plan_state {
        match view.goal_plan_revision {
            Some(revision) => parts.push(format!("goal_plan_state={goal_plan_state} r{revision}")),
            None => parts.push(format!("goal_plan_state={goal_plan_state}")),
        }
    }
    if let Some(current_stage_id) = &view.current_stage_id {
        parts.push(format!("current_stage={current_stage_id}"));
    }
    if let Some(current_step_id) = &view.current_step_id {
        parts.push(format!("current_step_id={current_step_id}"));
    }
    if let Some(active_workflow) = &view.active_workflow {
        parts.push(format!("workflow={active_workflow}"));
    }
    if let Some(workflow_phase) = &view.workflow_phase {
        parts.push(format!("workflow_phase={workflow_phase}"));
    }
    if let Some(validation_status) = &view.latest_validation_status {
        parts.push(format!("latest_validation_status={validation_status}"));
    }
    if let Some(project_scale_current_stage) = &view.project_scale_current_stage {
        parts.push(format!("project_scale_current_stage={project_scale_current_stage}"));
    }
    if let Some(project_scale_path) = &view.project_scale_path {
        parts.push(format!("project_scale_path={project_scale_path}"));
    }

    (!parts.is_empty()).then(|| format!("summary: {}", parts.join("; ")))
}

fn session_artifacts_brief_line(view: &SessionStatusView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(goal_brief_ref) = &view.goal_brief_ref {
        parts.push(format!("goal_brief_ref={goal_brief_ref}"));
    }
    if let Some(session_plan_brief_ref) = &view.session_plan_brief_ref {
        parts.push(format!("session_plan_brief_ref={session_plan_brief_ref}"));
    }
    if let Some(run_brief_ref) = &view.run_brief_ref {
        parts.push(format!("run_brief_ref={run_brief_ref}"));
    }
    if let Some(latest_trace_ref) = &view.latest_trace_ref {
        parts.push(format!("latest_trace_ref={latest_trace_ref}"));
    }
    if let Some(plan_brief_ref) =
        view.latest_governance_stage.as_deref().and_then(planning_stage_brief_ref)
    {
        parts.push(format!("plan_brief_ref={plan_brief_ref}"));
    }
    if let Some(packet_ref) = &view.latest_governance_packet_ref {
        parts.push(format!("latest_governance_packet_ref={packet_ref}"));
    }
    if let Some(checkpoint_id) = &view.latest_checkpoint_id {
        match &view.latest_checkpoint_scope {
            Some(scope) => parts.push(format!("latest_checkpoint_id={checkpoint_id} ({scope})")),
            None => parts.push(format!("latest_checkpoint_id={checkpoint_id}")),
        }
    }
    if let Some(changed_files) = &view.latest_changed_files
        && !changed_files.is_empty()
    {
        parts.push(format!("latest_changed_files={}", preview_session_brief_items(changed_files)));
    }

    (!parts.is_empty()).then(|| format!("artifacts: {}", parts.join("; ")))
}

fn session_clarification_brief_lines(view: &SessionStatusView) -> Vec<String> {
    let mut lines = Vec::new();

    push_optional_preview_text_line(
        &mut lines,
        "clarification_headline",
        view.clarification_headline.as_deref(),
    );
    push_optional_preview_text_line(
        &mut lines,
        "clarification_prompt",
        view.clarification_prompt.as_deref(),
    );
    push_optional_preview_items_line(
        &mut lines,
        "clarification_missing_fields",
        view.clarification_missing_fields.as_deref(),
    );
    push_optional_preview_items_line(
        &mut lines,
        "clarification_questions",
        view.clarification_questions.as_deref(),
    );
    push_quality_brief_lines(
        &mut lines,
        "goal_quality_state",
        view.goal_quality_state.as_deref(),
        "goal_quality_findings",
        view.goal_quality_findings.as_deref(),
        Some(("goal_quality_assumptions", view.goal_quality_assumptions.as_deref())),
    );
    push_quality_brief_lines(
        &mut lines,
        "plan_quality_state",
        view.plan_quality_state.as_deref(),
        "plan_quality_findings",
        view.plan_quality_findings.as_deref(),
        Some(("plan_quality_assumptions", view.plan_quality_assumptions.as_deref())),
    );
    push_quality_brief_lines(
        &mut lines,
        "backlog_quality_state",
        view.backlog_quality_state.as_deref(),
        "backlog_quality_findings",
        view.backlog_quality_findings.as_deref(),
        None,
    );
    if let Some(task_count) = view.backlog_task_count {
        lines.push(format!("backlog_task_count: {task_count}"));
    }
    push_optional_preview_text_line(
        &mut lines,
        "backlog_mvp_scope",
        view.backlog_mvp_scope.as_deref(),
    );
    push_optional_preview_items_line(
        &mut lines,
        "backlog_unmapped_items",
        view.backlog_unmapped_items.as_deref(),
    );
    push_optional_line(
        &mut lines,
        "planning_analysis_state",
        view.planning_analysis_state.as_deref(),
    );
    if let Some(findings) = &view.planning_analysis_findings
        && !findings.is_empty()
    {
        lines.push(format!(
            "planning_analysis_findings: {}",
            preview_planning_analysis_findings(findings)
        ));
    }
    if let Some(coverage) = &view.planning_analysis_coverage {
        lines.push(format!(
            "planning_analysis_coverage: {}",
            planning_analysis_coverage_text(coverage)
        ));
    }

    lines
}

fn push_quality_brief_lines(
    lines: &mut Vec<String>,
    state_label: &str,
    state: Option<&str>,
    findings_label: &str,
    findings: Option<&[String]>,
    assumptions: Option<(&str, Option<&[String]>)>,
) {
    push_optional_line(lines, state_label, state);
    push_optional_preview_items_line(lines, findings_label, findings);
    if let Some((assumptions_label, assumptions)) = assumptions {
        push_optional_preview_items_line(lines, assumptions_label, assumptions);
    }
}

fn push_optional_line(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        lines.push(format!("{label}: {value}"));
    }
}

fn push_optional_preview_text_line(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("{label}: {}", preview_session_brief_text(value)));
    }
}

fn push_optional_preview_items_line(
    lines: &mut Vec<String>,
    label: &str,
    values: Option<&[String]>,
) {
    if let Some(values) = values
        && !values.is_empty()
    {
        lines.push(format!("{label}: {}", preview_session_brief_items(values)));
    }
}

fn session_review_brief_line(view: &SessionStatusView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(trigger) = &view.latest_review_trigger {
        parts.push(format!("latest_review_trigger={trigger}"));
    }
    if let Some(outcome) = &view.latest_review_outcome {
        parts.push(format!("latest_review_outcome={outcome}"));
    }
    if let Some(headline) = &view.latest_review_headline {
        parts.push(format!("latest_review_headline={}", preview_session_brief_text(headline)));
    }
    if let Some(vote) = &view.latest_review_vote {
        parts.push(format!("latest_review_vote={}", preview_session_brief_text(vote)));
    }

    (!parts.is_empty()).then(|| format!("review: {}", parts.join("; ")))
}

fn session_governance_brief_line(view: &SessionStatusView) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(stage) = &view.latest_governance_stage {
        parts.push(format!("latest_governance_stage={stage}"));
    }
    if let Some(runtime) = &view.latest_governance_runtime {
        parts.push(format!("latest_governance_runtime={runtime}"));
    }
    if let Some(state) = &view.latest_governance_state {
        parts.push(format!("latest_governance_state={state}"));
    }
    if let Some(reason) = &view.latest_governance_blocked_reason {
        parts.push(format!(
            "latest_governance_blocked_reason={}",
            preview_session_brief_text(reason)
        ));
    }
    if let Some(next_action) = &view.governance_next_action {
        parts.push(format!("governance_next_action={}", preview_session_brief_text(next_action)));
    }

    (!parts.is_empty()).then(|| format!("governance: {}", parts.join("; ")))
}

fn session_reasoning_brief_line(view: &SessionStatusView) -> Option<String> {
    let reasoning_profile = view.latest_reasoning_profile.as_ref()?;
    let mut parts = vec![
        format!("latest_reasoning_profile_id={}", reasoning_profile.profile_id),
        format!("latest_reasoning_profile_status={}", reasoning_profile.status.as_str()),
    ];

    if let Some(outcome) = &reasoning_profile.outcome {
        parts.push(format!(
            "latest_reasoning_contribution={}",
            preview_session_brief_text(&outcome.headline)
        ));
        if let Some(next_action) = &outcome.next_action {
            parts.push(format!(
                "latest_reasoning_next_action={}",
                preview_session_brief_text(next_action)
            ));
        }
    } else if let Some((_, reason)) = Some(session_execution_condition_parts(view)) {
        parts.push(format!("latest_reasoning_block={}", preview_session_brief_text(&reason)));
    }

    Some(format!("reasoning: {}", parts.join("; ")))
}

fn preview_session_brief_text(text: &str) -> String {
    let trimmed = text.trim();
    let char_count = trimmed.chars().count();
    if char_count <= SESSION_BRIEF_CHAR_LIMIT {
        return trimmed.to_string();
    }

    let preview = trimmed.chars().take(SESSION_BRIEF_CHAR_LIMIT - 3).collect::<String>();
    format!("{preview}...")
}

fn preview_session_brief_items(items: &[String]) -> String {
    let visible = items
        .iter()
        .take(SESSION_BRIEF_ITEM_LIMIT)
        .map(|item| preview_session_brief_text(item))
        .collect::<Vec<_>>();
    let remaining = items.len().saturating_sub(visible.len());
    if remaining == 0 {
        visible.join(", ")
    } else {
        format!("{} (+{remaining} more)", visible.join(", "))
    }
}

fn preview_planning_analysis_findings(
    findings: &[crate::domain::goal_plan::PlanningAnalysisFinding],
) -> String {
    let items = findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{}:{}",
                finding.severity.as_str(),
                finding.source.as_str(),
                preview_session_brief_text(&finding.message)
            )
        })
        .collect::<Vec<_>>();
    preview_session_brief_items(&items)
}

fn planning_analysis_coverage_text(
    coverage: &crate::domain::goal_plan::PlanningAnalysisCoverage,
) -> String {
    let mut parts = vec![format!(
        "success_criteria={}/{}",
        coverage.success_criteria_covered, coverage.success_criteria_total
    )];
    if let Some(backlog_task_count) = coverage.backlog_task_count {
        parts.push(format!("backlog_tasks={backlog_task_count}"));
    }
    if let Some(mapped_plan_task_count) = coverage.mapped_plan_task_count {
        parts.push(format!(
            "mapped_plan_tasks={mapped_plan_task_count}/{}",
            coverage.success_criteria_total
        ));
    }
    parts.join(", ")
}

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
    lines.extend(framework_adapter_status_lines(&view.workspace_ref));
    lines.extend(framework_adapter_stage_routing_lines(
        view.latest_framework_adapter_stage_routing.as_ref(),
    ));
    lines.extend(framework_adapter_hook_dispatch_lines(
        view.latest_framework_adapter_hook_dispatch.as_ref(),
    ));
    lines.extend(framework_adapter_stage_failure_lines(
        view.latest_framework_adapter_stage_failure.as_ref(),
    ));

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

    if let Some(goal_quality_state) = &view.goal_quality_state {
        lines.push(format!("goal_quality_state: {goal_quality_state}"));
    }

    if let Some(goal_quality_findings) = &view.goal_quality_findings
        && !goal_quality_findings.is_empty()
    {
        lines.push(format!("goal_quality_findings: {}", goal_quality_findings.join(", ")));
    }

    if let Some(goal_quality_assumptions) = &view.goal_quality_assumptions
        && !goal_quality_assumptions.is_empty()
    {
        lines.push(format!("goal_quality_assumptions: {}", goal_quality_assumptions.join(", ")));
    }

    if let Some(plan_quality_state) = &view.plan_quality_state {
        lines.push(format!("plan_quality_state: {plan_quality_state}"));
    }

    if let Some(plan_quality_findings) = &view.plan_quality_findings
        && !plan_quality_findings.is_empty()
    {
        lines.push(format!("plan_quality_findings: {}", plan_quality_findings.join(", ")));
    }

    if let Some(plan_quality_assumptions) = &view.plan_quality_assumptions
        && !plan_quality_assumptions.is_empty()
    {
        lines.push(format!("plan_quality_assumptions: {}", plan_quality_assumptions.join(", ")));
    }

    if let Some(backlog_quality_state) = &view.backlog_quality_state {
        lines.push(format!("backlog_quality_state: {backlog_quality_state}"));
    }

    if let Some(backlog_quality_findings) = &view.backlog_quality_findings
        && !backlog_quality_findings.is_empty()
    {
        lines.push(format!("backlog_quality_findings: {}", backlog_quality_findings.join(", ")));
    }

    if let Some(backlog_task_count) = view.backlog_task_count {
        lines.push(format!("backlog_task_count: {backlog_task_count}"));
    }

    if let Some(backlog_mvp_scope) = &view.backlog_mvp_scope {
        lines.push(format!("backlog_mvp_scope: {backlog_mvp_scope}"));
    }

    if let Some(backlog_unmapped_items) = &view.backlog_unmapped_items
        && !backlog_unmapped_items.is_empty()
    {
        lines.push(format!("backlog_unmapped_items: {}", backlog_unmapped_items.join(", ")));
    }

    if let Some(planning_analysis_state) = &view.planning_analysis_state {
        lines.push(format!("planning_analysis_state: {planning_analysis_state}"));
    }

    if let Some(planning_analysis_findings) = &view.planning_analysis_findings
        && !planning_analysis_findings.is_empty()
    {
        lines.push(format!(
            "planning_analysis_findings: {}",
            preview_planning_analysis_findings(planning_analysis_findings)
        ));
    }

    if let Some(planning_analysis_coverage) = &view.planning_analysis_coverage {
        lines.push(format!(
            "planning_analysis_coverage: {}",
            planning_analysis_coverage_text(planning_analysis_coverage)
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

    if let Some(clarification_questions) = &view.clarification_questions
        && !clarification_questions.is_empty()
    {
        lines.push(format!("clarification_questions: {}", clarification_questions.join(", ")));
    }

    push_governance_display_lines(
        &mut lines,
        view.requested_governance_runtime.as_deref(),
        view.requested_governance_risk.as_deref(),
        view.requested_governance_zone.as_deref(),
        view.requested_governance_owner.as_deref(),
    );

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
    if let Some(governance_lifecycle_selected_mode_sequence) =
        &view.governance_lifecycle_selected_mode_sequence
    {
        lines.push(format!(
            "governance_lifecycle_selected_mode_sequence: {}",
            governance_lifecycle_selected_mode_sequence.join(", ")
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
