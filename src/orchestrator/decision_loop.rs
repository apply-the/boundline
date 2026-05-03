//! Bounded observe→decide→act→verify→update execution loop (feature 013).

use std::path::Path;

use serde::Serialize;
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::trace_store::TraceStore;
use crate::domain::decision::{
    ActionSelector, Decision, DecisionError, DecisionStatus, DecisionType, EvidenceRef,
};
use crate::domain::flow_policy::{FlowPolicy, FlowPolicyError};
use crate::domain::goal_plan::{GoalPlan, PlannedTask};
use crate::domain::governance::{CompactedCanonMemory, MemoryCredibilityState};
use crate::domain::limits::RunLimits;
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, StepExecutionRequest, StepExecutionResult, StepKind,
};
use crate::domain::task_context::TaskContext;
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::{ExecutionTrace, TraceEventType};
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

#[derive(Debug, Serialize)]
struct DecisionActionInput<'a> {
    selector: ActionSelector,
    target: &'a str,
    rationale: &'a str,
    expected_outcome: &'a str,
}

#[derive(Debug, Serialize)]
struct DecisionTracePayload<'a> {
    decision_type: DecisionType,
    selector: ActionSelector,
    target: &'a str,
    rationale: &'a str,
    expected_outcome: &'a str,
    evidence_inputs: &'a [EvidenceRef],
    status: DecisionStatus,
    created_at: u64,
    completed_at: Option<u64>,
    action_result: Option<&'a ToolResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery_decision_id: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct TerminalTracePayload<'a> {
    terminal: &'static str,
    reason: &'a str,
    selector: ActionSelector,
}

/// Terminal state produced by the decision loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopTerminal {
    Success,
    Failure(String),
    Exhausted { steps_taken: usize, max_steps: usize },
    NoActionableState(String),
}

/// Observation collected at the start of each loop iteration.
#[derive(Debug, Clone)]
pub struct Observation {
    pub workspace_files: Vec<String>,
    pub last_decision: Option<Decision>,
    pub accumulated_evidence: Vec<EvidenceRef>,
    pub remaining_tasks: Vec<String>,
}

/// The decision loop runner.
#[allow(dead_code)]
pub struct DecisionLoop<S> {
    agents: AgentRegistry,
    tools: ToolRegistry,
    trace_store: S,
    max_steps: usize,
}

impl<S> DecisionLoop<S>
where
    S: TraceStore,
{
    pub fn new(
        agents: AgentRegistry,
        tools: ToolRegistry,
        trace_store: S,
        max_steps: usize,
    ) -> Self {
        Self { agents, tools, trace_store, max_steps }
    }

    /// Run the bounded decision loop on a confirmed goal plan.
    pub fn run(
        &self,
        plan: &GoalPlan,
        flow_policy: Option<&FlowPolicy>,
        workspace_ref: &str,
        session_id: &str,
    ) -> Result<(LoopTerminal, Vec<Decision>, ExecutionTrace), DecisionLoopError> {
        let mut trace = ExecutionTrace::new(
            session_id.to_string(),
            session_id.to_string(),
            plan.goal_text.clone(),
        );
        trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            json!({
                "plan_id": plan.plan_id,
                "goal": plan.goal_text,
                "task_count": plan.tasks.len(),
                "goal_plan_state": plan.proposal_state_text(),
                "goal_plan_revision": plan.proposal_revision,
                "flow_state": plan.flow_state().summary_text(),
                "planning_rationale": plan.planning_rationale,
                "verification_strategy": plan.verification_strategy,
                "negotiation_goal_summary": plan.negotiation_goal_summary,
                "negotiation_resolution": plan.negotiation_resolution,
                "negotiation_acceptance_boundary": plan.negotiation_acceptance_boundary,
                "context_summary": plan.context_summary(),
                "context_credibility": plan.context_credibility(),
                "context_primary_inputs": plan.context_primary_inputs(),
                "context_provenance": plan.context_provenance_lines(),
                "context_staleness_reason": plan
                    .context_pack
                    .as_ref()
                    .and_then(|pack| pack.staleness_reason.clone()),
                "canon_memory_summary": plan
                    .compacted_canon_memory
                    .as_ref()
                    .map(CompactedCanonMemory::summary_text),
                "canon_memory_credibility": plan.compacted_canon_memory.as_ref().map(|memory| {
                    memory.credibility.as_str().to_string()
                }),
                "canon_memory_reason_code": plan
                    .compacted_canon_memory
                    .as_ref()
                    .and_then(|memory| memory.reason_code.clone()),
                "canon_memory_artifact_refs": plan
                    .compacted_canon_memory
                    .as_ref()
                    .map(|memory| memory.artifact_refs.clone())
                    .unwrap_or_default(),
                "canon_next_action": plan
                    .compacted_canon_memory
                    .as_ref()
                    .and_then(|memory| memory.recommended_next_action.as_ref())
                    .map(|action| format!("{}: {}", action.action, action.rationale)),
            }),
        );

        if let Some(memory) = plan.compacted_canon_memory.as_ref()
            && memory.credibility != MemoryCredibilityState::Credible
        {
            let reason = canon_memory_terminal_reason(memory);
            trace.record_event(
                TraceEventType::TerminalRecorded,
                None,
                0,
                no_actionable_state_payload(ActionSelector::Replan, &reason),
            );
            return Ok((LoopTerminal::NoActionableState(reason), Vec::new(), trace));
        }

        let mut decisions: Vec<Decision> = Vec::new();
        let mut completed_task_indices: Vec<usize> = Vec::new();
        let mut step_count: usize = 0;
        let mut flow_policy_owned = flow_policy.cloned();

        loop {
            // -- Check step limit --
            if step_count >= self.max_steps {
                let terminal =
                    LoopTerminal::Exhausted { steps_taken: step_count, max_steps: self.max_steps };
                trace.record_event(
                    TraceEventType::TerminalRecorded,
                    None,
                    0,
                    json!({
                        "terminal": "exhausted",
                        "steps_taken": step_count,
                        "max_steps": self.max_steps,
                    }),
                );
                return Ok((terminal, decisions, trace));
            }

            // -- OBSERVE --
            let remaining: Vec<(usize, &str)> = plan
                .tasks
                .iter()
                .enumerate()
                .filter(|(i, _)| !completed_task_indices.contains(i))
                .map(|(i, t)| (i, t.target.as_str()))
                .collect();

            if remaining.is_empty() {
                let terminal = LoopTerminal::Success;
                trace.record_event(
                    TraceEventType::TerminalRecorded,
                    None,
                    0,
                    json!({ "terminal": "success", "steps_taken": step_count }),
                );
                return Ok((terminal, decisions, trace));
            }

            let observation = Observation {
                workspace_files: remaining.iter().map(|(_, t)| t.to_string()).collect(),
                last_decision: decisions.last().cloned(),
                accumulated_evidence: plan
                    .source_evidence
                    .iter()
                    .cloned()
                    .chain(
                        plan.compacted_canon_memory
                            .as_ref()
                            .into_iter()
                            .flat_map(canon_memory_evidence_refs),
                    )
                    .chain(
                        decisions
                            .iter()
                            .filter(|d| d.tool_result.is_some())
                            .map(|d| d.as_tool_output_evidence()),
                    )
                    .collect(),
                remaining_tasks: remaining.iter().map(|(_, t)| t.to_string()).collect(),
            };

            // -- DECIDE --
            let (task_index, _next_task) = remaining[0];
            let planned = &plan.tasks[task_index];
            let previous_failed_decision = decisions
                .last()
                .filter(|decision| decision.status == DecisionStatus::Failed)
                .cloned();
            let decision_type = if let Some(failed_decision) = previous_failed_decision.as_ref() {
                recovery_decision_type(flow_policy_owned.as_ref(), failed_decision.decision_type)
            } else {
                let hinted_type = planned.decision_type_hint.unwrap_or(DecisionType::Code);
                if let Some(ref fp) = flow_policy_owned {
                    if fp.is_allowed(hinted_type) {
                        hinted_type
                    } else {
                        let stage = fp.current_stage().ok_or(DecisionLoopError::NoActiveStage)?;
                        *stage.allowed_decisions.first().ok_or(DecisionLoopError::NoActiveStage)?
                    }
                } else {
                    hinted_type
                }
            };
            let selector = select_action_selector(
                Path::new(workspace_ref),
                &observation,
                planned.target.as_str(),
                previous_failed_decision.as_ref(),
                decision_type,
                plan.compacted_canon_memory.as_ref(),
            );

            let (target, rationale, expected_outcome, evidence) = decision_details(
                planned,
                &observation,
                selector,
                previous_failed_decision.as_ref(),
            );

            let mut decision =
                Decision::new(decision_type, target, rationale, expected_outcome, evidence)
                    .with_selector(selector);

            trace.record_event(
                TraceEventType::DecisionCreated,
                Some(decision.id.clone()),
                0,
                decision_event_payload(&decision),
            );

            // -- ACT --
            decision.mark_dispatched().map_err(DecisionLoopError::Decision)?;
            trace.record_event(
                TraceEventType::DecisionDispatched,
                Some(decision.id.clone()),
                0,
                decision_event_payload(&decision),
            );

            // Simulate tool execution: in the real implementation this dispatches
            // through the tool/agent adapter. For now, produce a synthetic result.
            let tool_result = self.dispatch_action(&decision, workspace_ref, session_id);

            // -- VERIFY --
            if tool_result.success {
                decision.mark_verified(tool_result).map_err(DecisionLoopError::Decision)?;
                trace.record_event(
                    TraceEventType::DecisionVerified,
                    Some(decision.id.clone()),
                    0,
                    decision_event_payload(&decision),
                );
                if previous_failed_decision.is_some() {
                    if let Some(previous_decision) = decisions.last_mut() {
                        previous_decision.mark_recovered().map_err(DecisionLoopError::Decision)?;
                        trace.record_event(
                            TraceEventType::DecisionRecovered,
                            Some(previous_decision.id.clone()),
                            0,
                            recovery_event_payload(previous_decision, &decision),
                        );
                    }

                    let planned_decision_type =
                        planned.decision_type_hint.unwrap_or(decision.decision_type);
                    if let Some((selector_label, reason)) = successful_recovery_terminal_reason(
                        planned,
                        planned_decision_type,
                        &decision,
                    ) {
                        decisions.push(decision);
                        let terminal = LoopTerminal::NoActionableState(reason.clone());
                        trace.record_event(
                            TraceEventType::TerminalRecorded,
                            None,
                            0,
                            no_actionable_state_payload(selector_label, &reason),
                        );
                        return Ok((terminal, decisions, trace));
                    }

                    if recovery_completes_task(planned_decision_type, decision.selector_kind()) {
                        completed_task_indices.push(task_index);

                        if let Some(ref mut fp) = flow_policy_owned {
                            let _ = fp.advance_stage();
                        }
                    }
                } else {
                    completed_task_indices.push(task_index);

                    if let Some(ref mut fp) = flow_policy_owned {
                        let _ = fp.advance_stage();
                    }
                }
            } else {
                decision.mark_failed(tool_result).map_err(DecisionLoopError::Decision)?;
                trace.record_event(
                    TraceEventType::DecisionFailed,
                    Some(decision.id.clone()),
                    0,
                    decision_event_payload(&decision),
                );

                if previous_failed_decision.is_some() {
                    if let Some((selector_label, reason)) =
                        failed_recovery_terminal_reason(planned, &decision)
                    {
                        decisions.push(decision);
                        let terminal = LoopTerminal::NoActionableState(reason.clone());
                        trace.record_event(
                            TraceEventType::TerminalRecorded,
                            None,
                            0,
                            no_actionable_state_payload(selector_label, &reason),
                        );
                        return Ok((terminal, decisions, trace));
                    }

                    decisions.push(decision);
                    step_count += 1;
                    continue;
                }
            }

            // -- UPDATE --
            decisions.push(decision);
            step_count += 1;
        }
    }

    /// Dispatch a decision action through the appropriate adapter.
    fn dispatch_action(
        &self,
        decision: &Decision,
        workspace_ref: &str,
        session_id: &str,
    ) -> ToolResult {
        let selector = decision.selector_kind();
        let (step_kind, adapter_name) = adapter_binding(selector);
        let request = StepExecutionRequest {
            step_id: decision.id.clone(),
            step_kind,
            target_name: adapter_name.to_string(),
            input: json!(DecisionActionInput {
                selector,
                target: &decision.target,
                rationale: &decision.rationale,
                expected_outcome: &decision.expected_outcome,
            }),
            task_snapshot: TaskContext::new(
                session_id.to_string(),
                workspace_ref.to_string(),
                RunLimits::default(),
                Map::new(),
            ),
            attempt_number: 1,
        };
        let step_result = match step_kind {
            StepKind::Agent => self
                .agents
                .get(adapter_name)
                .map(|adapter| adapter.execute(request.clone()))
                .unwrap_or_else(|| missing_adapter_result(adapter_name)),
            StepKind::Tool => self
                .tools
                .get(adapter_name)
                .map(|adapter| adapter.execute(request.clone()))
                .unwrap_or_else(|| missing_adapter_result(adapter_name)),
            StepKind::Decision => missing_adapter_result(adapter_name),
        };

        tool_result_from_step_execution(adapter_name, decision, &step_result)
    }
}

fn decision_details(
    planned: &PlannedTask,
    observation: &Observation,
    selector: ActionSelector,
    previous_failed_decision: Option<&Decision>,
) -> (String, String, String, Vec<EvidenceRef>) {
    if let Some(failed_decision) = previous_failed_decision {
        let target = failed_decision.target.clone();
        let evidence = vec![failed_decision.as_tool_output_evidence()];
        return match selector {
            ActionSelector::Search => (
                target.clone(),
                format!(
                    "Search for a credible bounded target after failed decision {}",
                    failed_decision.id
                ),
                format!("identify credible workspace evidence for {}", failed_decision.target),
                evidence,
            ),
            ActionSelector::Ask => (
                target.clone(),
                format!("Clarification is required after failed decision {}", failed_decision.id),
                format!(
                    "clarify the bounded target or acceptance boundary for {}",
                    failed_decision.target
                ),
                evidence,
            ),
            ActionSelector::Replan => (
                target.clone(),
                format!("Replan after failed decision {}", failed_decision.id),
                format!("record a bounded recovery path for {}", failed_decision.target),
                evidence,
            ),
            ActionSelector::Modify => (
                target.clone(),
                format!("Apply a new bounded change after {} failed", failed_decision.id),
                format!("produce a credible change against {}", failed_decision.target),
                evidence,
            ),
            ActionSelector::Test => (
                target.clone(),
                format!("Retest {} after a recovery action", failed_decision.target),
                format!("collect fresh validation evidence for {}", failed_decision.target),
                evidence,
            ),
            ActionSelector::Read => (
                target.clone(),
                format!(
                    "Read {} again after {} failed",
                    failed_decision.target, failed_decision.id
                ),
                format!("recover the failed action against {}", failed_decision.target),
                evidence,
            ),
        };
    }

    let target = planned.target.clone();
    let evidence = observation.accumulated_evidence.clone();
    match selector {
        ActionSelector::Read => (
            target,
            planned.description.clone(),
            planned
                .expected_outcome
                .clone()
                .unwrap_or_else(|| "file contents collected".to_string()),
            evidence,
        ),
        ActionSelector::Search => (
            target,
            format!("Search the workspace for bounded evidence related to {}", planned.target),
            format!("identify credible workspace evidence for {}", planned.target),
            evidence,
        ),
        ActionSelector::Modify => (
            target,
            planned.description.clone(),
            planned
                .expected_outcome
                .clone()
                .unwrap_or_else(|| "material workspace diff produced".to_string()),
            evidence,
        ),
        ActionSelector::Test => (
            target,
            planned.description.clone(),
            planned
                .expected_outcome
                .clone()
                .unwrap_or_else(|| "validation evidence collected".to_string()),
            evidence,
        ),
        ActionSelector::Ask => (
            target,
            format!("Clarification is required before continuing {}", planned.target),
            format!("clarify the next bounded action for {}", planned.target),
            evidence,
        ),
        ActionSelector::Replan => (
            target,
            planned.description.clone(),
            planned
                .expected_outcome
                .clone()
                .unwrap_or_else(|| "bounded recovery path recorded".to_string()),
            evidence,
        ),
    }
}

fn successful_recovery_terminal_reason(
    planned: &PlannedTask,
    planned_decision_type: DecisionType,
    decision: &Decision,
) -> Option<(ActionSelector, String)> {
    match decision.selector_kind() {
        ActionSelector::Ask => Some((ActionSelector::Ask, decision.expected_outcome.clone())),
        ActionSelector::Replan
            if matches!(planned_decision_type, DecisionType::Code | DecisionType::Fix) =>
        {
            Some((
                ActionSelector::Replan,
                format!("recovery decision for {} failed", planned.target),
            ))
        }
        _ => None,
    }
}

fn recovery_completes_task(planned_decision_type: DecisionType, selector: ActionSelector) -> bool {
    (planned_decision_type == DecisionType::Analyze
        && matches!(selector, ActionSelector::Read | ActionSelector::Search))
        || (matches!(planned_decision_type, DecisionType::Code | DecisionType::Fix)
            && selector == ActionSelector::Modify)
}

fn failed_recovery_terminal_reason(
    planned: &PlannedTask,
    decision: &Decision,
) -> Option<(ActionSelector, String)> {
    (decision.selector_kind() == ActionSelector::Ask).then(|| {
        (ActionSelector::Ask, format!("clarification request for {} failed", planned.target))
    })
}

fn select_action_selector(
    workspace_ref: &Path,
    observation: &Observation,
    target: &str,
    failed_decision: Option<&Decision>,
    decision_type: DecisionType,
    compacted_canon_memory: Option<&CompactedCanonMemory>,
) -> ActionSelector {
    if let Some(failed_decision) = failed_decision {
        return match failed_decision.selector_kind() {
            ActionSelector::Read => ActionSelector::Search,
            ActionSelector::Search => ActionSelector::Ask,
            ActionSelector::Modify => ActionSelector::Replan,
            ActionSelector::Test => ActionSelector::Modify,
            ActionSelector::Replan => ActionSelector::Ask,
            ActionSelector::Ask => ActionSelector::Ask,
        };
    }

    if decision_type == DecisionType::Analyze
        && compacted_canon_memory.is_some_and(|memory| {
            memory.credibility == MemoryCredibilityState::Credible
                && !memory.artifact_refs.is_empty()
        })
    {
        return ActionSelector::Search;
    }

    if decision_type == DecisionType::Analyze {
        let path = workspace_ref.join(target);
        if path.is_dir()
            || !path.exists()
            || target == "test suite"
            || observation.remaining_tasks.is_empty()
        {
            return ActionSelector::Search;
        }
    }

    decision_type.default_selector()
}

fn canon_memory_terminal_reason(memory: &CompactedCanonMemory) -> String {
    memory.reason_code.clone().unwrap_or_else(|| {
        format!("Canon-grounded memory is {}: {}", memory.credibility.as_str(), memory.headline)
    })
}

fn canon_memory_evidence_refs(memory: &CompactedCanonMemory) -> Vec<EvidenceRef> {
    let mut evidence = Vec::new();
    evidence.push(EvidenceRef::canon(format!("memory: {}", memory.summary_text())));
    if let Some(packet_ref) = memory.packet_ref.as_ref() {
        evidence.push(EvidenceRef::canon(packet_ref.clone()));
    }
    if let Some(run_ref) = memory.run_ref.as_ref() {
        evidence.push(EvidenceRef::canon(run_ref.clone()));
    }
    for artifact_ref in &memory.artifact_refs {
        evidence.push(EvidenceRef::canon(artifact_ref.clone()));
    }
    if let Some(evidence_summary) = memory.evidence_summary.as_ref() {
        for link in &evidence_summary.artifact_provenance_links {
            evidence.push(EvidenceRef::canon(link.clone()));
        }
    }
    evidence
}

fn decision_event_payload(decision: &Decision) -> Value {
    json!(DecisionTracePayload {
        decision_type: decision.decision_type,
        selector: decision.selector_kind(),
        target: &decision.target,
        rationale: &decision.rationale,
        expected_outcome: &decision.expected_outcome,
        evidence_inputs: &decision.evidence_inputs,
        status: decision.status,
        created_at: decision.created_at,
        completed_at: decision.completed_at,
        action_result: decision.tool_result.as_ref(),
        recovery_decision_id: None,
    })
}

fn recovery_event_payload(decision: &Decision, recovery_decision: &Decision) -> Value {
    json!(DecisionTracePayload {
        decision_type: decision.decision_type,
        selector: decision.selector_kind(),
        target: &decision.target,
        rationale: &decision.rationale,
        expected_outcome: &decision.expected_outcome,
        evidence_inputs: &decision.evidence_inputs,
        status: decision.status,
        created_at: decision.created_at,
        completed_at: decision.completed_at,
        action_result: decision.tool_result.as_ref(),
        recovery_decision_id: Some(&recovery_decision.id),
    })
}

fn no_actionable_state_payload(selector: ActionSelector, reason: &str) -> Value {
    json!(TerminalTracePayload { terminal: "no_actionable_state", reason, selector })
}

fn recovery_decision_type(
    flow_policy: Option<&FlowPolicy>,
    failed_type: DecisionType,
) -> DecisionType {
    let candidates: &[DecisionType] = match failed_type {
        DecisionType::Analyze => &[DecisionType::Replan, DecisionType::Analyze],
        DecisionType::Code | DecisionType::Fix => &[DecisionType::Fix, DecisionType::Replan],
        DecisionType::Test => &[DecisionType::Fix, DecisionType::Replan],
        DecisionType::Replan => &[DecisionType::Analyze, DecisionType::Fix],
    };

    if let Some(flow_policy) = flow_policy {
        for candidate in candidates {
            if flow_policy.is_allowed(*candidate) {
                return *candidate;
            }
        }

        if let Some(stage) = flow_policy.current_stage()
            && let Some(fallback) = stage.allowed_decisions.first()
        {
            return *fallback;
        }
    }

    candidates[0]
}

fn adapter_binding(selector: ActionSelector) -> (StepKind, &'static str) {
    match selector {
        ActionSelector::Read | ActionSelector::Search => (StepKind::Agent, "analyzer"),
        ActionSelector::Modify => (StepKind::Agent, "coder"),
        ActionSelector::Test => (StepKind::Tool, "tester"),
        ActionSelector::Ask => (StepKind::Tool, "asker"),
        ActionSelector::Replan => (StepKind::Tool, "replanner"),
    }
}

fn missing_adapter_result(adapter_name: &str) -> StepExecutionResult {
    StepExecutionResult::failure(
        ErrorInfo::new(
            "adapter_missing",
            format!("no adapter named `{adapter_name}` is registered for this decision"),
        ),
        Recoverability::Terminal,
    )
}

fn tool_result_from_step_execution(
    adapter_name: &str,
    decision: &Decision,
    result: &StepExecutionResult,
) -> ToolResult {
    let mut tool_result = ToolResult::new(
        adapter_name,
        format!("{adapter_name} {}", decision.target),
        matches!(result.status, ExecutionStatus::Succeeded),
        1,
    );

    if let Some(output) = result.output.as_ref() {
        if let Some(stdout) = output.get("stdout").and_then(Value::as_str) {
            tool_result = tool_result.with_stdout(stdout.to_string());
        } else {
            tool_result = tool_result.with_stdout(output.to_string());
        }

        if let Some(stderr) = output.get("stderr").and_then(Value::as_str) {
            tool_result = tool_result.with_stderr(stderr.to_string());
        }

        if let Some(diff) = output.get("diff").and_then(Value::as_str) {
            tool_result = tool_result.with_diff(diff.to_string());
        }

        if let Some(exit_code) = output.get("exit_code").and_then(Value::as_i64) {
            tool_result = tool_result.with_exit_code(exit_code as i32);
        }
    }

    if let Some(error) = result.error.as_ref()
        && tool_result.stderr.is_empty()
    {
        tool_result = tool_result.with_stderr(error.message.clone());
    }

    if let Some(evidence) = result.evidence.as_ref()
        && tool_result.stdout.is_empty()
    {
        tool_result = tool_result.with_stdout(evidence.to_string());
    }

    if matches!(result.status, ExecutionStatus::Failed) && tool_result.exit_code.is_none() {
        tool_result = tool_result.with_exit_code(-1);
    }

    tool_result
}

#[derive(Debug, Error)]
pub enum DecisionLoopError {
    #[error("decision error: {0}")]
    Decision(#[from] DecisionError),
    #[error("flow policy error: {0}")]
    FlowPolicy(#[from] FlowPolicyError),
    #[error("no active stage in flow policy")]
    NoActiveStage,
    #[error("trace store error: {0}")]
    TraceStore(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        Observation, adapter_binding, decision_details, decision_event_payload,
        failed_recovery_terminal_reason, recovery_completes_task, recovery_decision_type,
        recovery_event_payload, select_action_selector, successful_recovery_terminal_reason,
        tool_result_from_step_execution,
    };
    use crate::adapters::trace_store::FileTraceStore;
    use crate::domain::decision::{ActionSelector, Decision, DecisionType, EvidenceRef};
    use crate::domain::flow_policy::{FlowPolicy, StagePolicy, TransitionCondition};
    use crate::domain::goal_plan::{GoalPlan, PlannedTask};
    use crate::domain::governance::{
        CanonRecommendedActionSummary, CompactedCanonMemory, MemoryCredibilityState,
    };
    use crate::domain::step::{
        ErrorInfo, ExecutionStatus, Recoverability, StepExecutionResult, StepKind,
    };
    use crate::domain::tool_result::ToolResult;
    use crate::registry::agent_registry::AgentRegistry;
    use crate::registry::tool_registry::ToolRegistry;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn planned_task(
        target: &str,
        decision_type_hint: DecisionType,
        expected_outcome: Option<&str>,
    ) -> PlannedTask {
        PlannedTask {
            task_id: "task-1".to_string(),
            description: format!("Process {target}"),
            target: target.to_string(),
            expected_outcome: expected_outcome.map(str::to_string),
            decision_type_hint: Some(decision_type_hint),
        }
    }

    fn failed_decision(selector: ActionSelector, target: &str) -> Decision {
        let mut decision = Decision::new(
            DecisionType::Analyze,
            target,
            "observe the target",
            "collect evidence",
            vec![EvidenceRef::file(target)],
        )
        .with_selector(selector);
        decision.id = format!("failed-{selector:?}").to_lowercase();
        decision.mark_dispatched().unwrap();
        decision
            .mark_failed(ToolResult::new("adapter", format!("adapter {target}"), false, 1))
            .unwrap();
        decision
    }

    fn observation_with_evidence() -> Observation {
        Observation {
            workspace_files: vec!["src/lib.rs".to_string()],
            last_decision: None,
            accumulated_evidence: vec![
                EvidenceRef::trace("trace-1"),
                EvidenceRef::file("src/lib.rs"),
            ],
            remaining_tasks: vec!["src/lib.rs".to_string()],
        }
    }

    #[test]
    fn decision_details_cover_selector_specific_rationale_and_defaults() {
        let planned = planned_task("src/lib.rs", DecisionType::Analyze, None);
        let observation = observation_with_evidence();

        let recovery_cases = [
            (
                ActionSelector::Search,
                "Search for a credible bounded target after failed decision",
                "identify credible workspace evidence for src/lib.rs",
            ),
            (
                ActionSelector::Ask,
                "Clarification is required after failed decision",
                "clarify the bounded target or acceptance boundary for src/lib.rs",
            ),
            (
                ActionSelector::Replan,
                "Replan after failed decision",
                "record a bounded recovery path for src/lib.rs",
            ),
            (
                ActionSelector::Modify,
                "Apply a new bounded change after",
                "produce a credible change against src/lib.rs",
            ),
            (
                ActionSelector::Test,
                "Retest src/lib.rs after a recovery action",
                "collect fresh validation evidence for src/lib.rs",
            ),
            (
                ActionSelector::Read,
                "Read src/lib.rs again after",
                "recover the failed action against src/lib.rs",
            ),
        ];

        for (selector, rationale_fragment, expected_outcome) in recovery_cases {
            let previous_failed = failed_decision(selector, "src/lib.rs");
            let (target, rationale, outcome, evidence) =
                decision_details(&planned, &observation, selector, Some(&previous_failed));
            assert_eq!(target, "src/lib.rs");
            assert!(rationale.contains(rationale_fragment), "{rationale}");
            assert_eq!(outcome, expected_outcome);
            assert_eq!(evidence, vec![previous_failed.as_tool_output_evidence()]);
        }

        let planned_cases = [
            (ActionSelector::Read, "Process src/lib.rs", "file contents collected"),
            (
                ActionSelector::Search,
                "Search the workspace for bounded evidence related to src/lib.rs",
                "identify credible workspace evidence for src/lib.rs",
            ),
            (ActionSelector::Modify, "Process src/lib.rs", "material workspace diff produced"),
            (ActionSelector::Test, "Process src/lib.rs", "validation evidence collected"),
            (
                ActionSelector::Ask,
                "Clarification is required before continuing src/lib.rs",
                "clarify the next bounded action for src/lib.rs",
            ),
            (ActionSelector::Replan, "Process src/lib.rs", "bounded recovery path recorded"),
        ];

        for (selector, rationale, outcome) in planned_cases {
            let (target, actual_rationale, actual_outcome, evidence) =
                decision_details(&planned, &observation, selector, None);
            assert_eq!(target, "src/lib.rs");
            assert_eq!(actual_rationale, rationale);
            assert_eq!(actual_outcome, outcome);
            assert_eq!(evidence, observation.accumulated_evidence);
        }
    }

    #[test]
    fn recovery_helpers_cover_terminal_and_completion_cases() {
        let planned_fix = planned_task("src/lib.rs", DecisionType::Fix, Some("apply fix"));
        let ask_decision = Decision::new(
            DecisionType::Replan,
            "src/lib.rs",
            "need clarification",
            "clarify the next bounded action",
            Vec::new(),
        )
        .with_selector(ActionSelector::Ask);
        let replan_decision = Decision::new(
            DecisionType::Fix,
            "src/lib.rs",
            "record recovery",
            "record a bounded recovery path",
            Vec::new(),
        )
        .with_selector(ActionSelector::Replan);

        assert_eq!(
            successful_recovery_terminal_reason(&planned_fix, DecisionType::Analyze, &ask_decision),
            Some((ActionSelector::Ask, "clarify the next bounded action".to_string()))
        );
        assert_eq!(
            successful_recovery_terminal_reason(&planned_fix, DecisionType::Fix, &replan_decision),
            Some((ActionSelector::Replan, "recovery decision for src/lib.rs failed".to_string(),))
        );
        assert_eq!(
            successful_recovery_terminal_reason(
                &planned_fix,
                DecisionType::Analyze,
                &replan_decision
            ),
            None
        );

        assert!(recovery_completes_task(DecisionType::Analyze, ActionSelector::Read));
        assert!(recovery_completes_task(DecisionType::Analyze, ActionSelector::Search));
        assert!(recovery_completes_task(DecisionType::Fix, ActionSelector::Modify));
        assert!(!recovery_completes_task(DecisionType::Test, ActionSelector::Modify));
        assert!(!recovery_completes_task(DecisionType::Fix, ActionSelector::Replan));

        assert_eq!(
            failed_recovery_terminal_reason(&planned_fix, &ask_decision),
            Some((ActionSelector::Ask, "clarification request for src/lib.rs failed".to_string(),))
        );
        assert_eq!(failed_recovery_terminal_reason(&planned_fix, &replan_decision), None);
    }

    #[test]
    fn selector_payload_and_tool_result_helpers_cover_remaining_branches() {
        let workspace = temp_workspace("decision-loop-selector-helpers");
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::write(workspace.join("src/lib.rs"), "pub fn add() {}\n").unwrap();
        fs::create_dir_all(workspace.join("src/dir")).unwrap();

        let mut empty_remaining = observation_with_evidence();
        empty_remaining.remaining_tasks.clear();
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/dir",
                None,
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Search
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/missing.rs",
                None,
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Search
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "test suite",
                None,
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Search
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &empty_remaining,
                "src/lib.rs",
                None,
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Search
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Read, "src/lib.rs")),
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Search
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Search, "src/lib.rs")),
                DecisionType::Analyze,
                None,
            ),
            ActionSelector::Ask
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Modify, "src/lib.rs")),
                DecisionType::Fix,
                None,
            ),
            ActionSelector::Replan
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Test, "src/lib.rs")),
                DecisionType::Test,
                None,
            ),
            ActionSelector::Modify
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Replan, "src/lib.rs")),
                DecisionType::Replan,
                None,
            ),
            ActionSelector::Ask
        );
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                Some(&failed_decision(ActionSelector::Ask, "src/lib.rs")),
                DecisionType::Replan,
                None,
            ),
            ActionSelector::Ask
        );

        let canon_memory = CompactedCanonMemory {
            headline: "Canon verification packet is credible".to_string(),
            credibility: MemoryCredibilityState::Credible,
            stage_key: Some("change:verify".to_string()),
            run_ref: Some("run-1".to_string()),
            packet_ref: Some(".canon/runs/run-1".to_string()),
            reason_code: None,
            artifact_refs: vec![".canon/runs/run-1/verification.md".to_string()],
            mode_summary: None,
            possible_actions: Vec::new(),
            recommended_next_action: None,
            evidence_summary: None,
        };
        assert_eq!(
            select_action_selector(
                &workspace,
                &observation_with_evidence(),
                "src/lib.rs",
                None,
                DecisionType::Analyze,
                Some(&canon_memory),
            ),
            ActionSelector::Search
        );

        let mut decision = Decision::new(
            DecisionType::Fix,
            "src/lib.rs",
            "apply a fix",
            "issue resolved",
            vec![EvidenceRef::trace("trace-1")],
        )
        .with_selector(ActionSelector::Modify);
        decision.mark_dispatched().unwrap();
        decision
            .mark_verified(
                ToolResult::new("coder", "coder src/lib.rs", true, 1)
                    .with_diff("updated".to_string()),
            )
            .unwrap();
        let payload = decision_event_payload(&decision);
        assert_eq!(payload.get("selector").and_then(|value| value.as_str()), Some("modify"));
        assert!(payload.get("completed_at").and_then(|value| value.as_u64()).is_some());
        assert!(payload.get("action_result").is_some());

        let recovery_payload = recovery_event_payload(
            &decision,
            &Decision::new(
                DecisionType::Replan,
                "src/lib.rs",
                "replan",
                "bounded recovery path",
                Vec::new(),
            )
            .with_selector(ActionSelector::Replan),
        );
        assert_eq!(
            recovery_payload.get("recovery_decision_id").and_then(|value| value.as_str()),
            Some(recovery_payload["recovery_decision_id"].as_str().unwrap())
        );

        let flow_policy = FlowPolicy {
            flow_name: "custom".to_string(),
            stage_policies: vec![StagePolicy {
                stage_id: "only".to_string(),
                allowed_decisions: vec![DecisionType::Analyze],
                transition_condition: TransitionCondition::AllVerified,
            }],
            current_stage_index: 0,
        };
        assert_eq!(recovery_decision_type(None, DecisionType::Test), DecisionType::Fix);
        assert_eq!(
            recovery_decision_type(Some(&flow_policy), DecisionType::Test),
            DecisionType::Analyze
        );
        assert_eq!(
            recovery_decision_type(
                Some(&FlowPolicy::from_builtin("bug-fix").unwrap()),
                DecisionType::Analyze
            ),
            DecisionType::Analyze
        );

        assert_eq!(adapter_binding(ActionSelector::Read), (StepKind::Agent, "analyzer"));
        assert_eq!(adapter_binding(ActionSelector::Search), (StepKind::Agent, "analyzer"));
        assert_eq!(adapter_binding(ActionSelector::Modify), (StepKind::Agent, "coder"));
        assert_eq!(adapter_binding(ActionSelector::Test), (StepKind::Tool, "tester"));
        assert_eq!(adapter_binding(ActionSelector::Ask), (StepKind::Tool, "asker"));
        assert_eq!(adapter_binding(ActionSelector::Replan), (StepKind::Tool, "replanner"));

        let tool_result = tool_result_from_step_execution(
            "tester",
            &Decision::new(DecisionType::Test, "test suite", "run tests", "tests pass", Vec::new()),
            &StepExecutionResult {
                status: ExecutionStatus::Failed,
                output: None,
                error: Some(ErrorInfo::new("validation_failed", "tests failed")),
                recoverability: Recoverability::ReplanRequired,
                evidence: Some(json!({"kind": "validation", "status": "failed"})),
                state_patch: None,
            },
        );
        assert_eq!(tool_result.stderr, "tests failed");
        assert!(tool_result.stdout.contains("validation"));
        assert_eq!(tool_result.exit_code, Some(-1));
    }

    #[test]
    fn decision_loop_terminalizes_when_canon_memory_is_not_credible() {
        let workspace = temp_workspace("decision-loop-canon-stop");
        let plan = GoalPlan::new(
            "verify governed change",
            vec![planned_task("src/lib.rs", DecisionType::Analyze, Some("collect evidence"))],
        )
        .unwrap()
        .with_compacted_canon_memory(CompactedCanonMemory {
            headline: "Canon packet is stale and must be refreshed".to_string(),
            credibility: MemoryCredibilityState::Stale,
            stage_key: Some("change:verify".to_string()),
            run_ref: Some("run-2".to_string()),
            packet_ref: Some(".canon/runs/run-2".to_string()),
            reason_code: Some("refresh_required".to_string()),
            artifact_refs: vec![".canon/runs/run-2/verification.md".to_string()],
            mode_summary: None,
            possible_actions: Vec::new(),
            recommended_next_action: Some(CanonRecommendedActionSummary {
                action: "refresh".to_string(),
                rationale: "Refresh the governed packet before continuing".to_string(),
                target: Some(".canon/runs/run-2".to_string()),
            }),
            evidence_summary: None,
        });
        let loop_runner = crate::orchestrator::decision_loop::DecisionLoop::new(
            AgentRegistry::new(),
            ToolRegistry::new(),
            FileTraceStore::for_workspace(&workspace),
            4,
        );

        let (terminal, decisions, trace) = loop_runner
            .run(&plan, None, workspace.to_string_lossy().as_ref(), "session-canon-stop")
            .unwrap();

        assert_eq!(
            terminal,
            crate::orchestrator::decision_loop::LoopTerminal::NoActionableState(
                "refresh_required".to_string()
            )
        );
        assert!(decisions.is_empty());
        assert!(trace.events.iter().any(|event| {
            event.event_type == crate::domain::trace::TraceEventType::TerminalRecorded
                && event.payload.get("reason").and_then(|value| value.as_str())
                    == Some("refresh_required")
        }));

        fs::remove_dir_all(workspace).unwrap();
    }
}
