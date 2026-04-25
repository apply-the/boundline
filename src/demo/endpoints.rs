use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
    let plan = profile.to_plan()?;
    let replan_steps = profile.to_replan_steps()?;
    let planner = if replan_steps.is_empty() {
        StaticPlanner::new(plan)
    } else {
        StaticPlanner::with_replans(plan, replan_steps)
    };
    let retry_counter = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let tester_counter = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
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
                // If the step input names a real file plus its replacement
                // contents, perform the actual write before reporting success.
                if let (Some(target_file), Some(fixed_content)) =
                    (input_str(&request, "target_file"), input_str(&request, "fixed_content"))
                {
                    if let Err(error) = write_target_file(Path::new(target_file), fixed_content) {
                        return StepExecutionResult::failure(
                            ErrorInfo::new("coder_io", error.to_string()),
                            Recoverability::Terminal,
                        )
                        .with_evidence(json!({"step_id": request.step_id}));
                    }
                    return StepExecutionResult::success(json!({
                        "patch_applied": true,
                        "updated_file": target_file,
                        "active_step": request.step_id,
                    }));
                }
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
        FnToolAdapter::new(move |request: StepExecutionRequest| {
            let mut counter = tester_counter.lock().unwrap();
            let attempts = counter.entry(request.step_id.clone()).or_insert(0);
            *attempts += 1;

            if input_flag(&request, "force_terminal_failure") {
                return StepExecutionResult::failure(
                    ErrorInfo::new(
                        "deterministic_terminal_failure",
                        "built-in default flow intentionally stops before success",
                    ),
                    Recoverability::Terminal,
                );
            }

            if input_flag(&request, "force_replan") && *attempts == 1 {
                return StepExecutionResult::failure(
                    ErrorInfo::new(
                        "deterministic_replan",
                        "built-in test-fix loop intentionally requests a replan before convergence",
                    ),
                    Recoverability::ReplanRequired,
                )
                .with_evidence(json!({"attempts": *attempts, "step_id": request.step_id}));
            }

            // Real file-based verification when wired up.
            if let (Some(target_file), Some(bug_marker)) =
                (input_str(&request, "target_file"), input_str(&request, "bug_marker"))
            {
                match fs::read_to_string(target_file) {
                    Ok(body) => {
                        if body.contains(bug_marker) {
                            return StepExecutionResult::failure(
                                ErrorInfo::new(
                                    "tester_bug_present",
                                    format!("bug marker still present in {target_file}"),
                                ),
                                Recoverability::ReplanRequired,
                            )
                            .with_evidence(json!({
                                "attempts": *attempts,
                                "step_id": request.step_id,
                                "target_file": target_file,
                            }));
                        }
                        StepExecutionResult::success(json!({
                            "tests_passed": true,
                            "goal_satisfied": true,
                            "verified_file": target_file,
                        }))
                    }
                    Err(error) => StepExecutionResult::failure(
                        ErrorInfo::new("tester_io", error.to_string()),
                        Recoverability::Terminal,
                    )
                    .with_evidence(json!({"target_file": target_file})),
                }
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

fn input_str<'a>(request: &'a StepExecutionRequest, key: &str) -> Option<&'a str> {
    request.input.get(key).and_then(|value| value.as_str())
}

fn write_target_file(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
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

    #[test]
    fn tester_force_replan_triggers_replan_only_on_first_attempt() {
        use std::env;
        use std::fs;
        // Build a runtime with a profile that has `force_replan` on verify.
        let mut profile = DemoRunProfile::default_run(
            "Force a non-success failure for the default developer flow",
        );
        // Strip the terminal-failure flag, set replan instead.
        for step in profile.step_outline.iter_mut() {
            if step.step_id == "verify"
                && let Some(obj) = step.input.as_object_mut()
            {
                obj.remove("force_terminal_failure");
                obj.insert("force_replan".to_string(), json!(true));
            }
        }
        // Also: provide a target_file the tester can read on attempt 2.
        let mut tmp = env::temp_dir();
        tmp.push(format!(
            "synod-tester-{}-{}.txt",
            std::process::id(),
            crate::domain::trace::current_timestamp_millis()
        ));
        fs::write(&tmp, "no marker here\n").unwrap();
        for step in profile.step_outline.iter_mut() {
            if step.step_id == "verify"
                && let Some(obj) = step.input.as_object_mut()
            {
                obj.insert("target_file".to_string(), json!(tmp.to_string_lossy()));
                obj.insert("bug_marker".to_string(), json!("// TODO-BUG"));
            }
        }

        let runtime = build_demo_runtime(profile).unwrap();
        let tester = runtime.tools.get("tester").unwrap();

        let first = tester.execute(crate::domain::step::StepExecutionRequest {
            step_id: "verify".to_string(),
            step_kind: crate::domain::step::StepKind::Tool,
            target_name: "tester".to_string(),
            input: runtime
                .profile
                .step_outline
                .iter()
                .find(|s| s.step_id == "verify")
                .unwrap()
                .input
                .clone(),
            task_snapshot: build_context(),
            attempt_number: 1,
        });
        let second = tester.execute(crate::domain::step::StepExecutionRequest {
            step_id: "verify".to_string(),
            step_kind: crate::domain::step::StepKind::Tool,
            target_name: "tester".to_string(),
            input: runtime
                .profile
                .step_outline
                .iter()
                .find(|s| s.step_id == "verify")
                .unwrap()
                .input
                .clone(),
            task_snapshot: build_context(),
            attempt_number: 2,
        });

        assert_eq!(first.recoverability, Recoverability::ReplanRequired);
        assert_eq!(second.output.as_ref().unwrap()["tests_passed"], json!(true));
        let _ = fs::remove_file(tmp);
    }
}
