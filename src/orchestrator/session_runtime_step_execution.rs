use serde_json::{Value, json};

use crate::domain::step::{
    ErrorInfo, Recoverability, Step, StepExecutionRequest, StepExecutionResult, StepKind,
};
use crate::domain::task_context::TaskContext;
use crate::fixture::FixtureRuntime;

use super::SessionRuntime;

impl SessionRuntime {
    pub(super) fn execute_step(
        &self,
        runtime: &FixtureRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        match step.kind {
            StepKind::Agent => self.execute_agent(runtime, step, context),
            StepKind::Tool => self.execute_tool(runtime, step, context),
            StepKind::Decision => self.execute_decision(step),
        }
    }

    fn execute_agent(
        &self,
        runtime: &FixtureRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "agent step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(agent) = runtime.agents.get(&target_name) else {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "unknown_agent",
                    format!("agent '{}' is not registered", target_name),
                ),
                Recoverability::Terminal,
            );
        };

        agent.execute(StepExecutionRequest {
            step_id: step.id.clone(),
            step_kind: step.kind,
            target_name,
            input: step.input.clone(),
            task_snapshot: context.clone(),
            attempt_number: step.attempt_count,
        })
    }

    fn execute_tool(
        &self,
        runtime: &FixtureRuntime,
        step: &Step,
        context: &TaskContext,
    ) -> StepExecutionResult {
        let Some(target_name) = step.target_name.clone() else {
            return StepExecutionResult::failure(
                ErrorInfo::new("missing_target", "tool step is missing a target name"),
                Recoverability::Terminal,
            );
        };

        let Some(tool) = runtime.tools.get(&target_name) else {
            return StepExecutionResult::failure(
                ErrorInfo::new("unknown_tool", format!("tool '{}' is not registered", target_name)),
                Recoverability::Terminal,
            );
        };

        tool.execute(StepExecutionRequest {
            step_id: step.id.clone(),
            step_kind: step.kind,
            target_name,
            input: step.input.clone(),
            task_snapshot: context.clone(),
            attempt_number: step.attempt_count,
        })
    }

    pub(super) fn execute_decision(&self, step: &Step) -> StepExecutionResult {
        let Some(object) = step.input.as_object() else {
            return StepExecutionResult::success(step.input.clone());
        };

        if object.get("retryable_failure").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_retry", "decision step requested a retry"),
                Recoverability::Retryable,
            );
        }

        if object.get("replan_required").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_replan", "decision step requested a replan"),
                Recoverability::ReplanRequired,
            );
        }

        if object.get("terminal_failure").and_then(Value::as_bool).unwrap_or(false) {
            return StepExecutionResult::failure(
                ErrorInfo::new("decision_terminal", "decision step requested terminal failure"),
                Recoverability::Terminal,
            );
        }

        let output = object.get("output").cloned().unwrap_or_else(|| step.input.clone());
        let state_patch = object.get("state_patch").and_then(Value::as_object).cloned();

        match state_patch {
            Some(patch) => StepExecutionResult::success_with_patch(output, patch),
            None => StepExecutionResult::success(output),
        }
    }

    pub(super) fn normalize_result(
        &self,
        result: StepExecutionResult,
        step: &Step,
    ) -> StepExecutionResult {
        match result.validate() {
            Ok(()) => result,
            Err(error) => StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_endpoint_result",
                    format!("step {} produced an invalid result: {}", step.id, error),
                )
                .with_details(json!({"step_id": step.id})),
                Recoverability::Terminal,
            ),
        }
    }
}
