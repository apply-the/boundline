//! Bounded observe→decide→act→verify→update execution loop (feature 013).

use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::trace_store::TraceStore;
use crate::domain::decision::{Decision, DecisionError, DecisionStatus, DecisionType, EvidenceRef};
use crate::domain::flow_policy::{FlowPolicy, FlowPolicyError};
use crate::domain::goal_plan::GoalPlan;
use crate::domain::limits::RunLimits;
use crate::domain::step::{
    ErrorInfo, ExecutionStatus, Recoverability, StepExecutionRequest, StepExecutionResult, StepKind,
};
use crate::domain::task_context::TaskContext;
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::{ExecutionTrace, TraceEventType};
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

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
            }),
        );

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

            let (target, rationale, expected_outcome, evidence) =
                if let Some(failed_decision) = previous_failed_decision.as_ref() {
                    (
                        failed_decision.target.clone(),
                        format!("Recover from failed decision {}", failed_decision.id),
                        format!("recover the failed action against {}", failed_decision.target),
                        vec![failed_decision.as_tool_output_evidence()],
                    )
                } else {
                    (
                        planned.target.clone(),
                        planned.description.clone(),
                        planned
                            .expected_outcome
                            .clone()
                            .unwrap_or_else(|| "task completed successfully".to_string()),
                        observation.accumulated_evidence.clone(),
                    )
                };

            let mut decision =
                Decision::new(decision_type, target, rationale, expected_outcome, evidence);

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
                    decisions.push(decision);
                    let terminal = LoopTerminal::NoActionableState(format!(
                        "recovery decision for {} failed",
                        planned.target
                    ));
                    trace.record_event(
                        TraceEventType::TerminalRecorded,
                        None,
                        0,
                        json!({
                            "terminal": "no_actionable_state",
                            "reason": format!("recovery decision for {} failed", planned.target),
                        }),
                    );
                    return Ok((terminal, decisions, trace));
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
        let (step_kind, adapter_name) = adapter_binding(decision.decision_type);
        let request = StepExecutionRequest {
            step_id: decision.id.clone(),
            step_kind,
            target_name: adapter_name.to_string(),
            input: json!({
                "target": decision.target,
                "rationale": decision.rationale,
                "expected_outcome": decision.expected_outcome,
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

fn decision_event_payload(decision: &Decision) -> Value {
    json!({
        "decision_type": decision.decision_type,
        "target": decision.target,
        "rationale": decision.rationale,
        "expected_outcome": decision.expected_outcome,
        "evidence_inputs": decision.evidence_inputs,
        "status": decision.status,
        "created_at": decision.created_at,
        "completed_at": decision.completed_at,
        "action_result": decision.tool_result,
    })
}

fn recovery_event_payload(decision: &Decision, recovery_decision: &Decision) -> Value {
    json!({
        "decision_type": decision.decision_type,
        "target": decision.target,
        "rationale": decision.rationale,
        "expected_outcome": decision.expected_outcome,
        "evidence_inputs": decision.evidence_inputs,
        "status": decision.status,
        "created_at": decision.created_at,
        "completed_at": decision.completed_at,
        "action_result": decision.tool_result,
        "recovery_decision_id": recovery_decision.id,
    })
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

fn adapter_binding(decision_type: DecisionType) -> (StepKind, &'static str) {
    match decision_type {
        DecisionType::Analyze => (StepKind::Agent, "analyzer"),
        DecisionType::Code | DecisionType::Fix => (StepKind::Agent, "coder"),
        DecisionType::Test => (StepKind::Tool, "tester"),
        DecisionType::Replan => (StepKind::Tool, "replanner"),
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
