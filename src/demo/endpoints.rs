use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::json;
use thiserror::Error;

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::tool::FnToolAdapter;
use crate::demo::profile::{DemoProfileError, DemoRunProfile};
use crate::domain::step::{ErrorInfo, Recoverability, StepExecutionRequest, StepExecutionResult};
use crate::orchestrator::planner::StaticPlanner;
use crate::registry::agent_registry::{AgentRegistry, RegistryError as AgentRegistryError};
use crate::registry::tool_registry::{RegistryError as ToolRegistryError, ToolRegistry};

pub struct DemoRuntime {
    pub profile: DemoRunProfile,
    pub planner: StaticPlanner,
    pub agents: AgentRegistry,
    pub tools: ToolRegistry,
}

pub fn build_demo_runtime(profile: DemoRunProfile) -> Result<DemoRuntime, DemoRuntimeError> {
    let planner = StaticPlanner::new(profile.to_plan()?);
    let retry_counter = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let guided_demo = profile.name == "guided_demo";

    let mut agents = AgentRegistry::new();
    agents.register(
		"analyzer",
		FnAgentAdapter::new(|request: StepExecutionRequest| {
			StepExecutionResult::success(json!({
				"analysis": format!("analyzed {}", request.input["goal"].as_str().unwrap_or("goal")),
				"active_step": request.step_id,
				"ready_for_code": true,
			}))
		}),
	)?;

    agents.register("coder", {
        let recovery_trigger_step = profile.recovery_trigger_step.clone();
        let retry_counter = retry_counter.clone();
        FnAgentAdapter::new(move |request: StepExecutionRequest| {
            let mut counter = retry_counter.lock().unwrap();
            let attempts = counter.entry(request.step_id.clone()).or_insert(0);
            *attempts += 1;

            if input_flag(&request, "force_replan") && *attempts == 1 {
                StepExecutionResult::failure(
					ErrorInfo::new(
						"deterministic_replan",
						"built-in flow intentionally invalidates the current path once before replanning",
					),
					Recoverability::ReplanRequired,
				)
				.with_evidence(json!({"attempts": *attempts, "step_id": request.step_id}))
            } else if request.step_id == recovery_trigger_step
                && *attempts == 1
                && (guided_demo || input_flag(&request, "force_retry"))
            {
                StepExecutionResult::failure(
                    ErrorInfo::new(
                        "deterministic_retry",
                        "built-in demo intentionally triggers one retry before patch application",
                    ),
                    Recoverability::Retryable,
                )
                .with_evidence(json!({"attempts": *attempts, "step_id": request.step_id}))
            } else {
                StepExecutionResult::success(json!({
                    "patch_applied": true,
                    "updated_files": ["src/lib.rs"],
                    "active_step": request.step_id,
                }))
            }
        })
    })?;

    let mut tools = ToolRegistry::new();
    tools.register(
        "tester",
        FnToolAdapter::new(|request: StepExecutionRequest| {
            if input_flag(&request, "force_terminal_failure") {
                StepExecutionResult::failure(
                    ErrorInfo::new(
                        "deterministic_terminal_failure",
                        "built-in default flow intentionally stops before success",
                    ),
                    Recoverability::Terminal,
                )
            } else {
                StepExecutionResult::success(json!({
                    "tests_passed": true,
                    "goal_satisfied": true,
                    "verified_after": request.task_snapshot.state.get("last_step_id").cloned(),
                }))
            }
        }),
    )?;

    Ok(DemoRuntime { profile, planner, agents, tools })
}

fn input_flag(request: &StepExecutionRequest, key: &str) -> bool {
    request.input.get(key).and_then(|value| value.as_bool()).unwrap_or(false)
}

#[derive(Debug, Error)]
pub enum DemoRuntimeError {
    #[error("demo profile is invalid: {0}")]
    InvalidProfile(#[from] DemoProfileError),
    #[error("failed to register built-in agent: {0}")]
    AgentRegistry(#[from] AgentRegistryError),
    #[error("failed to register built-in tool: {0}")]
    ToolRegistry(#[from] ToolRegistryError),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::build_demo_runtime;
    use crate::demo::profile::DemoRunProfile;
    use crate::domain::limits::RunLimits;
    use crate::domain::step::Recoverability;
    use crate::domain::task_context::TaskContext;

    fn build_context() -> TaskContext {
        TaskContext::new("session-demo", "/tmp/demo", RunLimits::default(), Default::default())
    }

    #[test]
    fn demo_runtime_injects_a_single_retry_on_the_configured_recovery_step() {
        let runtime = build_demo_runtime(DemoRunProfile::guided_demo()).unwrap();
        let coder = runtime.agents.get("coder").unwrap();

        let first = coder.execute(crate::domain::step::StepExecutionRequest {
            step_id: runtime.profile.recovery_trigger_step.clone(),
            step_kind: crate::domain::step::StepKind::Agent,
            target_name: "coder".to_string(),
            input: json!({"goal": runtime.profile.goal}),
            task_snapshot: build_context(),
            attempt_number: 1,
        });
        let second = coder.execute(crate::domain::step::StepExecutionRequest {
            step_id: runtime.profile.recovery_trigger_step.clone(),
            step_kind: crate::domain::step::StepKind::Agent,
            target_name: "coder".to_string(),
            input: json!({"goal": runtime.profile.goal}),
            task_snapshot: build_context(),
            attempt_number: 2,
        });

        assert_eq!(first.recoverability, Recoverability::Retryable);
        assert_eq!(second.output.unwrap()["patch_applied"], json!(true));
    }

    #[test]
    fn default_run_can_force_a_terminal_failure_during_verification() {
        let runtime = build_demo_runtime(DemoRunProfile::default_run(
            "Force a non-success failure for the default developer flow",
        ))
        .unwrap();
        let tester = runtime.tools.get("tester").unwrap();

        let result = tester.execute(crate::domain::step::StepExecutionRequest {
            step_id: "verify".to_string(),
            step_kind: crate::domain::step::StepKind::Tool,
            target_name: "tester".to_string(),
            input: json!({"force_terminal_failure": true}),
            task_snapshot: build_context(),
            attempt_number: 1,
        });

        assert_eq!(result.recoverability, Recoverability::Terminal);
    }
}
