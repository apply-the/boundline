use super::cluster::render_cluster_story_lines;
use super::context::push_context_projection_lines;
use super::events::{governance_event_line, review_event_line, validation_line_from_event};
use super::routing::{
    render_route_config_projection, render_run_execution_condition,
    route_config_projection_for_run_trace, run_trace_route_owner,
};
use super::runtime::{
    adaptive_attempt_lineage_summary, adaptive_candidate_family_summary,
    adaptive_exhaustion_reason_summary, adaptive_rejected_candidates_summary,
    adaptive_selection_reason_summary, adaptive_workspace_slice_summary,
    append_reasoning_profile_lines, framework_adapter_stage_failure_lines,
};
use super::support::checkpoint_projection_from_state;
use std::path::Path;

use serde_json::Value;

use super::{
    ExecutionTrace, KEY_REASON, KEY_STAGE_ID, ProfileActivationRecord, TaskRunResponse,
    TraceEventType, UNKNOWN_STAGE_ID, task_status_text,
};
use crate::domain::session::FrameworkAdapterStageFailureDetails;

const KEY_PLAN_QUALITY_ASSUMPTIONS: &str = "plan_quality_assumptions";
const KEY_PLAN_QUALITY_FINDINGS: &str = "plan_quality_findings";
const KEY_PLAN_QUALITY_STATE: &str = "plan_quality_state";

fn value_as_string_list(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
    })
}

fn push_plan_quality_lines(lines: &mut Vec<String>, payload: &Value) {
    if let Some(plan_quality_state) = payload.get(KEY_PLAN_QUALITY_STATE).and_then(Value::as_str) {
        lines.push(format!("plan_quality_state: {plan_quality_state}"));
    }
    if let Some(findings) = payload
        .get(KEY_PLAN_QUALITY_FINDINGS)
        .and_then(value_as_string_list)
        .filter(|findings| !findings.is_empty())
    {
        lines.push(format!("plan_quality_findings: {}", findings.join(", ")));
    }
    if let Some(assumptions) = payload
        .get(KEY_PLAN_QUALITY_ASSUMPTIONS)
        .and_then(value_as_string_list)
        .filter(|assumptions| !assumptions.is_empty())
    {
        lines.push(format!("plan_quality_assumptions: {}", assumptions.join(", ")));
    }
}

pub fn render_run_trace(
    command_name: &str,
    trace: Option<&ExecutionTrace>,
    response: &TaskRunResponse,
    next_command: &str,
) -> String {
    let mut lines = vec![format!("{command_name}: {}", response.terminal_reason.message)];

    if let Some(trace) = trace {
        let mut context_summary: Option<String> = None;
        let mut context_credibility: Option<String> = None;
        let mut context_primary_inputs: Vec<String> = Vec::new();
        let mut context_provenance: Vec<String> = Vec::new();
        let mut context_staleness_reason: Option<String> = None;
        let mut governance_next_action: Option<String> = None;
        let mut reasoning_profile: Option<ProfileActivationRecord> = None;
        lines.insert(0, format!("goal: {}", trace.goal));
        lines.insert(1, format!("route_owner: {}", run_trace_route_owner(trace)));
        if let Some(route_config_projection) = render_route_config_projection(
            route_config_projection_for_run_trace(trace, Path::new(&response.trace_location)),
        ) {
            lines.insert(2, route_config_projection);
        }

        if let Some(input) = trace.events.iter().find_map(|event| {
            (event.event_type == TraceEventType::TaskStarted)
                .then(|| event.payload.get("input"))
                .flatten()
        }) {
            if let Some(authored_input_summary) =
                input.get("authored_input_summary").and_then(Value::as_str)
            {
                lines.push(format!("authored_input_summary: {authored_input_summary}"));
            }
            if let Some(clarification_headline) =
                input.get("clarification_headline").and_then(Value::as_str)
            {
                lines.push(format!("clarification_headline: {clarification_headline}"));
            }
            if let Some(clarification_prompt) =
                input.get("clarification_prompt").and_then(Value::as_str)
            {
                lines.push(format!("clarification_prompt: {clarification_prompt}"));
            }
            if let Some(negotiation_goal_summary) =
                input.get("negotiation_goal_summary").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
            }
            if let Some(negotiation_resolution) =
                input.get("negotiation_resolution").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
            }
            if let Some(negotiation_acceptance_boundary) =
                input.get("negotiation_acceptance_boundary").and_then(Value::as_str)
            {
                lines.push(format!(
                    "negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"
                ));
            }
            context_summary = input
                .get("context_summary")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_summary);
            context_credibility = input
                .get("context_credibility")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_credibility);
            if context_primary_inputs.is_empty() {
                context_primary_inputs = input
                    .get("context_primary_inputs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if context_provenance.is_empty() {
                context_provenance = input
                    .get("context_provenance")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            context_staleness_reason = input
                .get("context_staleness_reason")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_staleness_reason);
        }

        if let Some(goal_plan_created) =
            trace.events.iter().find(|event| event.event_type == TraceEventType::GoalPlanCreated)
        {
            push_plan_quality_lines(&mut lines, &goal_plan_created.payload);
            if let Some(negotiation_goal_summary) =
                goal_plan_created.payload.get("negotiation_goal_summary").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_goal_summary: {negotiation_goal_summary}"));
            }
            if let Some(negotiation_resolution) =
                goal_plan_created.payload.get("negotiation_resolution").and_then(Value::as_str)
            {
                lines.push(format!("negotiation_resolution: {negotiation_resolution}"));
            }
            if let Some(negotiation_acceptance_boundary) = goal_plan_created
                .payload
                .get("negotiation_acceptance_boundary")
                .and_then(Value::as_str)
            {
                lines.push(format!(
                    "negotiation_acceptance_boundary: {negotiation_acceptance_boundary}"
                ));
            }
            context_summary = goal_plan_created
                .payload
                .get("context_summary")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_summary);
            context_credibility = goal_plan_created
                .payload
                .get("context_credibility")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_credibility);
            if context_primary_inputs.is_empty() {
                context_primary_inputs = goal_plan_created
                    .payload
                    .get("context_primary_inputs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if context_provenance.is_empty() {
                context_provenance = goal_plan_created
                    .payload
                    .get("context_provenance")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            context_staleness_reason = goal_plan_created
                .payload
                .get("context_staleness_reason")
                .and_then(Value::as_str)
                .map(str::to_string)
                .or(context_staleness_reason);
        }

        for event in &trace.events {
            if !matches!(
                event.event_type,
                TraceEventType::GovernanceAwaitingApproval
                    | TraceEventType::GovernanceCompleted
                    | TraceEventType::GovernanceBlocked
                    | TraceEventType::GovernancePacketRejected
            ) {
                continue;
            }

            if context_summary.is_none() {
                context_summary = event
                    .payload
                    .get("canon_memory_summary")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_summary);
            }
            if context_credibility.is_none() {
                context_credibility = event
                    .payload
                    .get("canon_memory_credibility")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_credibility);
            }
            if context_primary_inputs.is_empty() {
                context_primary_inputs = event
                    .payload
                    .get("document_refs")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_string))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
            }
            if let Some(canon_memory_summary) =
                event.payload.get("canon_memory_summary").and_then(Value::as_str)
            {
                let line = format!("canon_memory: {canon_memory_summary}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_compatibility) =
                event.payload.get("canon_memory_compatibility").and_then(Value::as_str)
            {
                let line = format!("canon_memory_compatibility: {canon_memory_compatibility}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_run_ref) = event
                .payload
                .get("canon_memory_run_ref")
                .or_else(|| event.payload.get("run_ref"))
                .and_then(Value::as_str)
            {
                let line = format!("canon_memory_run_ref: {canon_memory_run_ref}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_packet_ref) = event
                .payload
                .get("canon_memory_packet_ref")
                .or_else(|| event.payload.get("packet_ref"))
                .and_then(Value::as_str)
            {
                let line = format!("canon_memory_packet: {canon_memory_packet_ref}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_memory_reason_code) =
                event.payload.get("canon_memory_reason_code").and_then(Value::as_str)
            {
                let line = format!("canon_memory_reason: {canon_memory_reason_code}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(canon_next_action) =
                event.payload.get("canon_next_action").and_then(Value::as_str)
            {
                let line = format!("canon_memory_next_action: {canon_next_action}");
                if !context_provenance.contains(&line) {
                    context_provenance.push(line);
                }
            }
            if let Some(authority_provenance_lines) =
                event.payload.get("authority_provenance_lines").and_then(Value::as_array)
            {
                for line in authority_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if let Some(adaptive_provenance_lines) =
                event.payload.get("adaptive_provenance_lines").and_then(Value::as_array)
            {
                for line in adaptive_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if let Some(semantic_provenance_lines) =
                event.payload.get("semantic_provenance_lines").and_then(Value::as_array)
            {
                for line in semantic_provenance_lines
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                {
                    if !context_provenance.contains(&line) {
                        context_provenance.push(line);
                    }
                }
            }
            if context_staleness_reason.is_none()
                && event
                    .payload
                    .get("canon_memory_credibility")
                    .and_then(Value::as_str)
                    .is_some_and(|credibility| credibility != "credible")
            {
                context_staleness_reason = event
                    .payload
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(context_staleness_reason);
            }
            if governance_next_action.is_none() {
                governance_next_action = event
                    .payload
                    .get("canon_next_action")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            if let Some(record) = event
                .payload
                .get("reasoning_profile_record")
                .cloned()
                .and_then(|value| serde_json::from_value(value).ok())
            {
                reasoning_profile = Some(record);
            }
        }

        push_context_projection_lines(
            &mut lines,
            context_summary.as_deref(),
            context_credibility.as_deref(),
            &context_primary_inputs,
            &context_provenance,
            context_staleness_reason.as_deref(),
        );

        if let Some(reasoning_profile) = &reasoning_profile {
            append_reasoning_profile_lines(&mut lines, "", reasoning_profile);
        }

        for event in &trace.events {
            if let Some(governance_next_action) = governance_next_action.as_ref() {
                lines.push(format!("governance_next_action: {governance_next_action}"));
            }
            match event.event_type {
                TraceEventType::TaskStarted
                | TraceEventType::TerminalRecorded
                | TraceEventType::ReviewerStarted => {}
                TraceEventType::FlowSelected => {
                    let flow_name = event
                        .payload
                        .get("flow_name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-flow");
                    let stage_id = event
                        .payload
                        .get("current_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("flow {flow_name} selected at {stage_id}"));
                }
                TraceEventType::CheckpointCreated => {
                    let checkpoint_id = event
                        .payload
                        .get("checkpoint_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-checkpoint");
                    let checkpoint_scope = event
                        .payload
                        .get("checkpoint_scope")
                        .and_then(Value::as_str)
                        .unwrap_or("workspace");
                    lines.push(format!("checkpoint {checkpoint_id} created ({checkpoint_scope})"));
                }
                TraceEventType::StageTransitioned => {
                    let from_stage = event
                        .payload
                        .get("from_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let to_stage = event
                        .payload
                        .get("to_stage_id")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    lines.push(format!("stage {from_stage} -> {to_stage}"));
                }
                TraceEventType::StageRouted => {
                    let stage_key = event
                        .payload
                        .get("framework_adapter_stage_routing")
                        .and_then(|value| value.get("stage_key"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown-stage");
                    let claim_state = event
                        .payload
                        .get("framework_adapter_stage_routing")
                        .and_then(|value| value.get("claim_state"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let execution_source = event
                        .payload
                        .get("framework_adapter_stage_routing")
                        .and_then(|value| value.get("execution_source"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let decision_reason = event
                        .payload
                        .get("framework_adapter_stage_routing")
                        .and_then(|value| value.get("decision_reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    lines.push(format!(
                        "framework-adapter routed {stage_key}: {execution_source} / {claim_state} / {decision_reason}"
                    ));
                }
                TraceEventType::StepStarted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let step_kind =
                        event.payload.get("step_kind").and_then(Value::as_str).unwrap_or("step");
                    lines.push(format!("step {step_id} ({step_kind}) started"));
                }
                TraceEventType::StepCompleted => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let status =
                        event.payload.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("step {step_id} {status}"));

                    if let Some(changed_files) = event
                        .payload
                        .get("output")
                        .and_then(|output| output.get("changed_files"))
                        .and_then(value_as_string_list)
                        && !changed_files.is_empty()
                    {
                        lines.push(format!("changed_files: {}", changed_files.join(", ")));
                    }

                    if let Some(validation_line) = validation_line_from_event(&event.payload) {
                        lines.push(validation_line);
                    }
                }
                TraceEventType::DecisionCreated => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let decision_type = event
                        .payload
                        .get("decision_type")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!(
                        "decision {decision_id} created: {decision_type} -> {target}"
                    ));
                }
                TraceEventType::DecisionDispatched => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    let target =
                        event.payload.get("target").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("decision {decision_id} dispatched: {target}"));
                }
                TraceEventType::DecisionVerified => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} verified"));
                }
                TraceEventType::DecisionFailed => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} failed"));
                }
                TraceEventType::DecisionRecovered => {
                    let decision_id = event.step_id.as_deref().unwrap_or("unknown-decision");
                    lines.push(format!("decision {decision_id} recovered"));
                }
                TraceEventType::ReasoningProfileActivated
                | TraceEventType::ReasoningParticipantStarted
                | TraceEventType::ReasoningParticipantCompleted
                | TraceEventType::ReasoningDisagreementRecorded
                | TraceEventType::ReasoningDebateRoundCompleted
                | TraceEventType::ReasoningReflexionRevisionCompleted
                | TraceEventType::ReasoningAdjudicationRecorded
                | TraceEventType::ReasoningConfidenceRecorded
                | TraceEventType::ReasoningProfileBlocked
                | TraceEventType::ReasoningProfileInterrupted
                | TraceEventType::ReasoningProfileEscalated => {}
                TraceEventType::GovernanceSelected
                | TraceEventType::GovernanceStarted
                | TraceEventType::GovernanceDecisionRecorded
                | TraceEventType::GovernanceAwaitingApproval
                | TraceEventType::GovernanceCompleted
                | TraceEventType::GovernanceBlocked
                | TraceEventType::GovernancePacketRejected => {
                    if let Some(line) = governance_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::RetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("retry for {step_id}: {reason}"));
                }
                TraceEventType::StageRetryScheduled => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("retry scheduled");
                    lines.push(format!("stage retry for {step_id}: {reason}"));
                }
                TraceEventType::Replanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("replan after {step_id}: {reason}"));
                }
                TraceEventType::StageReplanned => {
                    let step_id = event.step_id.as_deref().unwrap_or("unknown-step");
                    let reason = event
                        .payload
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("replan scheduled");
                    lines.push(format!("stage replan after {step_id}: {reason}"));
                }
                TraceEventType::StageFailed => {
                    let stage_id = event
                        .payload
                        .get(KEY_STAGE_ID)
                        .and_then(Value::as_str)
                        .unwrap_or(UNKNOWN_STAGE_ID);
                    let reason = event
                        .payload
                        .get(KEY_REASON)
                        .and_then(Value::as_str)
                        .unwrap_or("stage failed");
                    lines.push(format!("stage {stage_id} failed: {reason}"));
                }
                TraceEventType::ReviewStarted
                | TraceEventType::ReviewTriggerIgnored
                | TraceEventType::ReviewerCompleted
                | TraceEventType::ReviewCouncilAssembled
                | TraceEventType::ReviewStopSemanticsRecorded
                | TraceEventType::ReviewVoteResolved
                | TraceEventType::ReviewAdjudicated
                | TraceEventType::ReviewTerminalRecorded => {
                    if let Some(line) = review_event_line(event.event_type, &event.payload) {
                        lines.push(line);
                    }
                }
                TraceEventType::ProjectScalePathProposed
                | TraceEventType::ProjectScaleStageTransitioned
                | TraceEventType::VotingDecisionRecorded => {}
                TraceEventType::GoalPlanCreated => {
                    let goal =
                        event.payload.get("goal").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("goal plan created: {goal}"));
                }
                TraceEventType::FlowInferred => {
                    let flow =
                        event.payload.get("flow_name").and_then(Value::as_str).unwrap_or("unknown");
                    lines.push(format!("flow inferred: {flow}"));
                }
                TraceEventType::RefinementRoundCompleted => {
                    // Round packets are surfaced via inspection, not run-trace output.
                }
            }
        }

        lines.push(render_run_execution_condition(response));
    }

    if trace.is_none() {
        lines.push(render_run_execution_condition(response));
    }

    lines.extend(framework_adapter_stage_failure_lines(
        FrameworkAdapterStageFailureDetails::from_terminal_reason(&response.terminal_reason)
            .as_ref(),
    ));

    if let Some(workspace_slice) = adaptive_workspace_slice_summary(&response.final_context.state) {
        lines.push(format!("workspace_slice: {workspace_slice}"));
    }

    if let Some(attempt_lineage) = adaptive_attempt_lineage_summary(&response.final_context.state) {
        lines.push(format!("attempt_lineage: {attempt_lineage}"));
    }

    if let Some(candidate_family) = adaptive_candidate_family_summary(&response.final_context.state)
    {
        lines.push(format!("candidate_family: {candidate_family}"));
    }

    if let Some(selection_reason) = adaptive_selection_reason_summary(&response.final_context.state)
    {
        lines.push(format!("selection_reason: {selection_reason}"));
    }

    if let Some(rejected_candidates) =
        adaptive_rejected_candidates_summary(&response.final_context.state)
    {
        lines.push(format!("rejected_candidates: {rejected_candidates}"));
    }

    if let Some(exhaustion_reason) =
        adaptive_exhaustion_reason_summary(&response.final_context.state)
    {
        lines.push(format!("adaptive_exhaustion: {exhaustion_reason}"));
    }

    let (latest_checkpoint_id, latest_checkpoint_scope, latest_checkpoint_restore_command) =
        checkpoint_projection_from_state(&response.final_context.state);
    if let Some(latest_checkpoint_id) = latest_checkpoint_id {
        lines.push(format!("latest_checkpoint_id: {latest_checkpoint_id}"));
    }
    if let Some(latest_checkpoint_scope) = latest_checkpoint_scope {
        lines.push(format!("latest_checkpoint_scope: {latest_checkpoint_scope}"));
    }
    if let Some(latest_checkpoint_restore_command) = latest_checkpoint_restore_command {
        lines.push(format!(
            "latest_checkpoint_restore_command: {latest_checkpoint_restore_command}"
        ));
    }

    if let Ok(Some(cluster_story)) = response.final_context.cluster_delivery_story() {
        lines.extend(render_cluster_story_lines(&cluster_story));
    }

    lines.push(format!("terminal_status: {}", task_status_text(response.terminal_status)));
    lines.push(format!("terminal_reason: {}", response.terminal_reason.message));
    lines.push(format!("trace: {}", response.trace_location));
    lines.push(format!("next_command: {next_command}"));
    lines.join("\n")
}
