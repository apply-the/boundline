use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::tool::FnToolAdapter;
use crate::domain::execution::{
    AdaptiveChangeKind, AttemptLineage, AttemptTransitionKind, ChangeEvidence, ChangeStatus,
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, ExecutionProfileError,
    PathScore, SelectionEvidence, ValidationRecord, WorkspaceChange, WorkspaceExecutionProfile,
    WorkspaceSliceSelection,
};
use crate::domain::flow::{
    FLOW_METADATA_KEY, FlowStepMetadata, SessionFlowState, attach_stage_metadata, built_in_flow,
};
use crate::domain::limits::RunLimits;
use crate::domain::plan::Plan;
use crate::domain::review::{
    ReviewOutcome, ReviewProfile, ReviewTrigger, ReviewerDisposition, ReviewerFinding,
    ReviewerParticipation, ReviewerParticipationStatus, VoteDecision, VoteResolution,
};
use crate::domain::step::{
    ErrorInfo, Recoverability, Step, StepError, StepExecutionRequest, StepExecutionResult,
};
use crate::domain::task::TaskRunRequest;
use crate::orchestrator::governance::{bounded_reused_packets, select_packet_reuse_binding};
use crate::orchestrator::planner::{CallbackPlanner, Planner, PlanningError, StaticPlanner};
use crate::registry::agent_registry::{AgentRegistry, RegistryError as AgentRegistryError};
use crate::registry::tool_registry::{RegistryError as ToolRegistryError, ToolRegistry};

const EXECUTION_RELATIVE_PATH: &str = ".synod/execution.json";
const FIXTURE_RELATIVE_PATH: &str = ".synod/fixture.json";

#[derive(Clone)]
pub struct FixtureRuntime {
    pub profile: WorkspaceExecutionProfile,
    pub planner: Arc<dyn Planner>,
    pub agents: AgentRegistry,
    pub tools: ToolRegistry,
}

#[derive(Debug, Clone)]
struct AdaptiveAttemptPlan {
    attempt: ExecutionAttemptDefinition,
    workspace_slice: WorkspaceSliceSelection,
    selection_evidence: SelectionEvidence,
    candidate_signature: String,
    attempt_lineage: AttemptLineage,
}

#[derive(Debug, Clone)]
struct WorkspaceTargetSource {
    path: String,
    contents: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceFixture {
    pub name: String,
    #[serde(default = "default_test_command")]
    pub test_command: FixtureCommand,
    #[serde(default = "default_run_limits")]
    pub limits: RunLimits,
    #[serde(default)]
    pub file_patches: Vec<FilePatch>,
}

impl WorkspaceFixture {
    pub fn validate(&self) -> Result<(), FixtureValidationError> {
        if self.name.trim().is_empty() {
            return Err(FixtureValidationError::MissingName);
        }

        if self.test_command.program.trim().is_empty() {
            return Err(FixtureValidationError::MissingTestProgram);
        }

        if self.file_patches.is_empty() {
            return Err(FixtureValidationError::MissingFilePatches);
        }

        self.limits
            .validate()
            .map_err(|error| FixtureValidationError::InvalidRunLimits(error.to_string()))?;

        for patch in &self.file_patches {
            patch.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureCommand {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: String,
    pub find: String,
    pub replace: String,
}

impl FilePatch {
    fn validate(&self) -> Result<(), FixtureValidationError> {
        if self.path.trim().is_empty() {
            return Err(FixtureValidationError::MissingPatchPath);
        }

        if Path::new(&self.path).is_absolute() {
            return Err(FixtureValidationError::AbsolutePatchPath(self.path.clone()));
        }

        if self.find.is_empty() {
            return Err(FixtureValidationError::MissingFindPattern(self.path.clone()));
        }

        Ok(())
    }
}

pub fn fixture_manifest_path(workspace: &Path) -> PathBuf {
    workspace.join(FIXTURE_RELATIVE_PATH)
}

pub fn execution_manifest_path(workspace: &Path) -> PathBuf {
    workspace.join(EXECUTION_RELATIVE_PATH)
}

pub fn load_workspace_execution_profile(
    workspace: &Path,
) -> Result<WorkspaceExecutionProfile, FixtureRuntimeError> {
    let execution_path = execution_manifest_path(workspace);
    if execution_path.is_file() {
        let contents = fs::read_to_string(&execution_path).map_err(|source| {
            FixtureRuntimeError::ExecutionProfileRead { path: execution_path.clone(), source }
        })?;
        let profile =
            serde_json::from_str::<WorkspaceExecutionProfile>(&contents).map_err(|source| {
                FixtureRuntimeError::ExecutionProfileParse { path: execution_path.clone(), source }
            })?;
        profile.validate()?;
        return Ok(profile);
    }

    match load_workspace_fixture(workspace) {
        Ok(fixture) => legacy_fixture_to_execution_profile(fixture),
        Err(FixtureRuntimeError::MissingFixture(_)) => {
            Err(FixtureRuntimeError::MissingExecutionProfile {
                preferred: execution_path,
                legacy: fixture_manifest_path(workspace),
            })
        }
        Err(error) => Err(error),
    }
}

pub fn load_workspace_fixture(workspace: &Path) -> Result<WorkspaceFixture, FixtureRuntimeError> {
    let path = fixture_manifest_path(workspace);
    if !path.is_file() {
        return Err(FixtureRuntimeError::MissingFixture(path));
    }

    let contents = fs::read_to_string(&path)
        .map_err(|source| FixtureRuntimeError::FixtureRead { path: path.clone(), source })?;
    let fixture = serde_json::from_str::<WorkspaceFixture>(&contents)
        .map_err(|source| FixtureRuntimeError::FixtureParse { path: path.clone(), source })?;
    fixture.validate()?;
    Ok(fixture)
}

pub fn build_fixture_plan(workspace: &Path) -> Result<Plan, FixtureRuntimeError> {
    build_fixture_plan_for_goal(workspace, None, "")
}

pub fn build_fixture_plan_for_flow(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
) -> Result<Plan, FixtureRuntimeError> {
    build_fixture_plan_for_goal(workspace, active_flow, "")
}

pub fn build_fixture_plan_for_goal(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
    goal: &str,
) -> Result<Plan, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;

    if profile.adaptive.is_some() {
        return build_adaptive_initial_plan(workspace, &profile, active_flow, goal);
    }

    build_vertical_slice_plan(&profile, active_flow, 0)
}

pub fn build_task_request(
    workspace: &Path,
    goal: impl Into<String>,
    session_id: impl Into<String>,
) -> Result<TaskRunRequest, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;

    Ok(TaskRunRequest {
        goal: goal.into(),
        input: json!({
            "execution_profile": profile.name,
            "flow": "workspace_execution",
        }),
        session_id: session_id.into(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: profile.limits,
        initial_context: None,
    })
}

pub fn build_fixture_runtime(workspace: &Path) -> Result<FixtureRuntime, FixtureRuntimeError> {
    build_fixture_runtime_for_flow(workspace, None)
}

pub fn build_fixture_runtime_for_flow(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
) -> Result<FixtureRuntime, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;
    let workspace_ref = workspace.to_path_buf();
    let planner: Arc<dyn Planner> = if profile.adaptive.is_some() {
        Arc::new(CallbackPlanner::new(
            {
                let workspace_ref = workspace_ref.clone();
                let profile = profile.clone();
                let active_flow = active_flow.cloned();
                move |request, _context| {
                    build_adaptive_initial_plan(
                        &workspace_ref,
                        &profile,
                        active_flow.as_ref(),
                        &request.goal,
                    )
                    .map_err(|error| PlanningError::InvalidPlan(error.to_string()))
                }
            },
            {
                let workspace_ref = workspace_ref.clone();
                let profile = profile.clone();
                let active_flow = active_flow.cloned();
                move |task, failed_step, failure| {
                    build_adaptive_replan_steps(
                        &workspace_ref,
                        &profile,
                        active_flow.as_ref(),
                        task,
                        failed_step,
                        failure,
                    )
                }
            },
        ))
    } else {
        Arc::new(StaticPlanner::with_replans(
            build_vertical_slice_plan(&profile, active_flow, 0)?,
            build_replan_queue(&profile, active_flow)?,
        ))
    };

    let mut agents = AgentRegistry::new();
    agents.register("analyzer", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        FnAgentAdapter::new(move |request| {
            analyze_workspace_fixture(&workspace_ref, &profile, request)
        })
    })?;
    agents.register("coder", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        FnAgentAdapter::new(move |request| {
            apply_workspace_fixture(&workspace_ref, &profile, request)
        })
    })?;
    agents.register("reviewer", {
        let profile = profile.clone();
        FnAgentAdapter::new(move |request| review_workspace_fixture(&profile, request))
    })?;

    let mut tools = ToolRegistry::new();
    tools.register("tester", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        FnToolAdapter::new(move |request| {
            verify_workspace_fixture(&workspace_ref, &profile, request)
        })
    })?;
    tools.register("review-voter", {
        let profile = profile.clone();
        FnToolAdapter::new(move |request| resolve_review_vote(&profile, request))
    })?;
    tools.register("review-finalizer", {
        let profile = profile.clone();
        FnToolAdapter::new(move |request| finalize_workspace_review(&profile, request))
    })?;

    Ok(FixtureRuntime { profile, planner, agents, tools })
}

fn build_vertical_slice_plan(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Plan, FixtureRuntimeError> {
    let Some(active_flow) = active_flow else {
        let mut steps = vec![Step::agent("analyze", "analyzer", analysis_step_input(profile))?];
        steps.extend(build_attempt_steps(profile, None, attempt_index)?);
        return Ok(Plan::new(steps)?);
    };

    let flow = built_in_flow(&active_flow.flow_name)
        .expect("validated flow name should resolve for fixture planning");

    let mut steps = match flow.name {
        "bug-fix" => vec![
            Step::agent(
                "investigate",
                "analyzer",
                attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(
                    code_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": "implement",
                            "force_retry_once": profile.limits.max_retries > 0,
                        }),
                    )?,
                    flow,
                    1,
                )?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(
                    verify_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": "verify",
                        }),
                    )?,
                    flow,
                    2,
                )?,
            )?,
        ],
        "change" => vec![
            Step::agent(
                "understand-change",
                "analyzer",
                attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(
                    code_step_input(profile, attempt_index, json!({"phase": "implement"}))?,
                    flow,
                    1,
                )?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(
                    verify_step_input(profile, attempt_index, json!({"phase": "verify"}))?,
                    flow,
                    2,
                )?,
            )?,
        ],
        "delivery" => vec![
            Step::agent(
                "requirements",
                "analyzer",
                attach_stage_metadata(analysis_step_input(profile), flow, 0)?,
            )?,
            Step::decision(
                "architecture",
                attach_stage_metadata(
                    json!({
                        "phase": "architecture",
                        "output": {"architecture_ready": true},
                    }),
                    flow,
                    1,
                )?,
            )?,
            Step::decision(
                "backlog",
                attach_stage_metadata(
                    json!({
                        "phase": "backlog",
                        "output": {"backlog_ready": true},
                    }),
                    flow,
                    2,
                )?,
            )?,
            Step::agent(
                "implementation-code",
                "coder",
                attach_stage_metadata(
                    code_step_input(profile, attempt_index, json!({"phase": "implementation"}))?,
                    flow,
                    3,
                )?,
            )?,
            Step::tool(
                "implementation-verify",
                "tester",
                attach_stage_metadata(
                    verify_step_input(profile, attempt_index, json!({"phase": "implementation"}))?,
                    flow,
                    3,
                )?,
            )?,
        ],
        _ => unreachable!("unsupported built-in flow should have been rejected earlier"),
    };

    steps.extend(build_review_steps(profile, Some(active_flow), attempt_index)?);

    Ok(Plan::new(steps)?)
}

fn build_replan_queue(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
) -> Result<Vec<Vec<Step>>, FixtureRuntimeError> {
    let mut replans = Vec::new();
    for attempt_index in 1..profile.attempts.len() {
        replans.push(build_attempt_steps(profile, active_flow, attempt_index)?);
    }
    Ok(replans)
}

fn build_adaptive_initial_plan(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    goal: &str,
) -> Result<Plan, FixtureRuntimeError> {
    let Some(candidate) = build_adaptive_candidates(
        workspace,
        profile,
        goal,
        &BTreeSet::new(),
        None,
        "selected the initial adaptive candidate".to_string(),
    )?
    .into_iter()
    .next() else {
        return Err(FixtureRuntimeError::NoAdaptiveCandidate { profile: profile.name.clone() });
    };

    let Some(active_flow) = active_flow else {
        let mut steps = vec![Step::agent(
            "analyze",
            "analyzer",
            adaptive_analysis_step_input(profile, &candidate),
        )?];
        steps.extend(build_adaptive_attempt_steps(profile, None, &candidate)?);
        return Ok(Plan::new(steps)?);
    };

    let flow = built_in_flow(&active_flow.flow_name)
        .expect("validated flow name should resolve for adaptive fixture planning");

    let mut steps = match flow.name {
        "bug-fix" => vec![
            Step::agent(
                "investigate",
                "analyzer",
                attach_stage_metadata(adaptive_analysis_step_input(profile, &candidate), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(
                    adaptive_code_step_input(
                        profile,
                        &candidate,
                        json!({
                            "phase": "implement",
                            "force_retry_once": profile.limits.max_retries > 0,
                        }),
                    ),
                    flow,
                    1,
                )?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(
                    adaptive_verify_step_input(profile, &candidate, json!({"phase": "verify"})),
                    flow,
                    2,
                )?,
            )?,
        ],
        "change" => vec![
            Step::agent(
                "understand-change",
                "analyzer",
                attach_stage_metadata(adaptive_analysis_step_input(profile, &candidate), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(
                    adaptive_code_step_input(profile, &candidate, json!({"phase": "implement"})),
                    flow,
                    1,
                )?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(
                    adaptive_verify_step_input(profile, &candidate, json!({"phase": "verify"})),
                    flow,
                    2,
                )?,
            )?,
        ],
        "delivery" => vec![
            Step::agent(
                "requirements",
                "analyzer",
                attach_stage_metadata(adaptive_analysis_step_input(profile, &candidate), flow, 0)?,
            )?,
            Step::decision(
                "architecture",
                attach_stage_metadata(
                    json!({
                        "phase": "architecture",
                        "output": {"architecture_ready": true},
                    }),
                    flow,
                    1,
                )?,
            )?,
            Step::decision(
                "backlog",
                attach_stage_metadata(
                    json!({
                        "phase": "backlog",
                        "output": {"backlog_ready": true},
                    }),
                    flow,
                    2,
                )?,
            )?,
            Step::agent(
                "implementation-code",
                "coder",
                attach_stage_metadata(
                    adaptive_code_step_input(
                        profile,
                        &candidate,
                        json!({"phase": "implementation"}),
                    ),
                    flow,
                    3,
                )?,
            )?,
            Step::tool(
                "implementation-verify",
                "tester",
                attach_stage_metadata(
                    adaptive_verify_step_input(
                        profile,
                        &candidate,
                        json!({"phase": "implementation"}),
                    ),
                    flow,
                    3,
                )?,
            )?,
        ],
        _ => unreachable!("unsupported built-in flow should have been rejected earlier"),
    };

    steps.extend(build_review_steps_for_attempt(
        profile,
        Some(active_flow),
        candidate.attempt.attempt_id.as_str(),
    )?);

    Ok(Plan::new(steps)?)
}

fn build_adaptive_replan_steps(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    task: &crate::domain::task::Task,
    failed_step: &Step,
    failure: &StepExecutionResult,
) -> Result<Vec<Step>, PlanningError> {
    let used_signatures = adaptive_candidate_signatures_from_state(&task.context.state);
    let previous_attempt_id = latest_attempt_id_from_state(&task.context.state)
        .or_else(|| failed_step.input.get("attempt_id").and_then(Value::as_str))
        .map(str::to_string);
    let reason = failure.error.as_ref().map(|error| error.message.clone()).unwrap_or_else(|| {
        "adaptive validation failed and a new candidate is required".to_string()
    });

    let Some(candidate) = build_adaptive_candidates(
        workspace,
        profile,
        &task.goal,
        &used_signatures,
        previous_attempt_id.as_deref(),
        reason,
    )
    .map_err(|error| PlanningError::Internal(error.to_string()))?
    .into_iter()
    .next() else {
        return Err(PlanningError::ReplanUnavailable(
            "adaptive planner could not synthesize a new candidate".to_string(),
        ));
    };

    build_adaptive_attempt_steps(profile, active_flow, &candidate)
        .map_err(|error| PlanningError::InvalidPlan(error.to_string()))
}

fn build_adaptive_candidates(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    goal: &str,
    used_signatures: &BTreeSet<String>,
    previous_attempt_id: Option<&str>,
    lineage_reason: String,
) -> Result<Vec<AdaptiveAttemptPlan>, FixtureRuntimeError> {
    let adaptive = profile
        .adaptive
        .as_ref()
        .expect("adaptive candidate synthesis requires an adaptive profile");
    let goal_hint = adaptive_goal_hint(goal, profile);
    let goal_terms = tokenize_terms(&goal_hint);
    let validation_terms = tokenize_terms(&profile.validation_command.rendered());
    let sources = load_workspace_target_sources(workspace, &profile.read_targets)?;
    let mut path_scores = sources
        .iter()
        .map(|source| score_workspace_target(source, adaptive, &goal_terms, &validation_terms))
        .collect::<Vec<_>>();
    path_scores.sort_by(|left, right| {
        right.score.cmp(&left.score).then_with(|| left.path.cmp(&right.path))
    });

    let selected_targets = path_scores
        .iter()
        .take(adaptive.max_selected_targets)
        .map(|score| score.path.clone())
        .collect::<Vec<_>>();

    let mut raw_candidates = Vec::new();
    let mut seen_signatures = BTreeSet::new();
    for selected_target in &selected_targets {
        let Some(source) = sources.iter().find(|source| &source.path == selected_target) else {
            continue;
        };

        for change in adaptive_changes_for_target(&source.path, &source.contents, adaptive) {
            let signature = workspace_change_signature(&change);
            if !seen_signatures.insert(signature.clone()) {
                continue;
            }

            raw_candidates.push((change, signature));
            if raw_candidates.len() >= adaptive.max_generated_attempts {
                break;
            }
        }

        if raw_candidates.len() >= adaptive.max_generated_attempts {
            break;
        }
    }

    let available_candidates = raw_candidates
        .into_iter()
        .filter(|(_, signature)| !used_signatures.contains(signature))
        .collect::<Vec<_>>();

    let available_count = available_candidates.len();

    Ok(available_candidates
        .into_iter()
        .enumerate()
        .map(|(index, (change, signature))| {
            let attempt_id = format!("adaptive-attempt-{}", used_signatures.len() + index + 1);
            let workspace_slice = WorkspaceSliceSelection {
                selection_id: format!("adaptive-slice-{attempt_id}"),
                selected_targets: selected_targets.clone(),
                scored_candidates: path_scores.clone(),
                headline: format!("selected {} for adaptive delivery", change.path),
            };
            let selection_evidence = SelectionEvidence {
                goal_terms: goal_terms.clone(),
                validation_terms: validation_terms.clone(),
                path_scores: path_scores.clone(),
                reason: format!(
                    "selected {} from {} scored read target(s)",
                    change.path,
                    selected_targets.len()
                ),
            };
            let attempt = ExecutionAttemptDefinition {
                attempt_id: attempt_id.clone(),
                summary: format!(
                    "Adaptively update {} by replacing '{}' with '{}'",
                    change.path,
                    excerpt(&change.find),
                    excerpt(&change.replace)
                ),
                failure_mode: if index + 1 < available_count {
                    ExecutionFailureMode::Replan
                } else {
                    ExecutionFailureMode::Terminal
                },
                changes: vec![change],
            };
            let attempt_lineage = AttemptLineage {
                previous_attempt_id: previous_attempt_id.map(str::to_string),
                current_attempt_id: attempt_id,
                transition_kind: if previous_attempt_id.is_some() {
                    AttemptTransitionKind::Replaced
                } else {
                    AttemptTransitionKind::Initial
                },
                reason: lineage_reason.clone(),
            };

            AdaptiveAttemptPlan {
                attempt,
                workspace_slice,
                selection_evidence,
                candidate_signature: signature,
                attempt_lineage,
            }
        })
        .collect())
}

fn build_adaptive_attempt_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    candidate: &AdaptiveAttemptPlan,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    if let Some(active_flow) = active_flow {
        let code_id = format!(
            "{}-replan-{}-code",
            active_flow.current_stage_id, candidate.attempt.attempt_id
        );
        let verify_id = format!(
            "{}-replan-{}-verify",
            active_flow.current_stage_id, candidate.attempt.attempt_id
        );

        return Ok(vec![
            Step::agent(
                code_id,
                "coder",
                attach_current_stage_metadata(
                    adaptive_code_step_input(
                        profile,
                        candidate,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    ),
                    active_flow,
                ),
            )?,
            Step::tool(
                verify_id,
                "tester",
                attach_current_stage_metadata(
                    adaptive_verify_step_input(
                        profile,
                        candidate,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    ),
                    active_flow,
                ),
            )?,
        ]
        .into_iter()
        .chain(build_review_steps_for_attempt(
            profile,
            Some(active_flow),
            candidate.attempt.attempt_id.as_str(),
        )?)
        .collect());
    }

    Ok(vec![
        Step::agent(
            format!("code-{}", candidate.attempt.attempt_id),
            "coder",
            adaptive_code_step_input(profile, candidate, json!({"phase": "code"})),
        )?,
        Step::tool(
            format!("verify-{}", candidate.attempt.attempt_id),
            "tester",
            adaptive_verify_step_input(profile, candidate, json!({"phase": "verify"})),
        )?,
    ]
    .into_iter()
    .chain(build_review_steps_for_attempt(profile, None, candidate.attempt.attempt_id.as_str())?)
    .collect())
}

fn load_workspace_target_sources(
    workspace: &Path,
    targets: &[String],
) -> Result<Vec<WorkspaceTargetSource>, FixtureRuntimeError> {
    targets
        .iter()
        .map(|target| {
            let path = workspace.join(target);
            let contents = fs::read_to_string(&path)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            Ok(WorkspaceTargetSource { path: target.clone(), contents })
        })
        .collect()
}

fn score_workspace_target(
    source: &WorkspaceTargetSource,
    adaptive: &crate::domain::execution::AdaptiveExecutionProfile,
    goal_terms: &[String],
    validation_terms: &[String],
) -> PathScore {
    let mut score = 0_i64;
    let mut reasons = Vec::new();
    let lower_path = source.path.to_ascii_lowercase();
    let lower_contents = source.contents.to_ascii_lowercase();

    for preference in &adaptive.path_preferences {
        if lower_path.starts_with(&preference.to_ascii_lowercase()) {
            score += 50;
            reasons.push(format!("matched path preference {}", preference));
        }
    }

    if lower_path.starts_with("src/") {
        score += 15;
        reasons.push("prioritized source file".to_string());
    }

    if lower_path.starts_with("tests/") {
        score += 5;
        reasons.push("test target remains available for evidence".to_string());
    }

    for term in goal_terms {
        if lower_path.contains(term) {
            score += 20;
            reasons.push(format!("goal term '{}' matched path", term));
        } else if lower_contents.contains(term) {
            score += 5;
            reasons.push(format!("goal term '{}' matched contents", term));
        }
    }

    for term in validation_terms {
        if lower_path.contains(term) || lower_contents.contains(term) {
            score += 3;
        }
    }

    let candidate_count =
        adaptive_changes_for_target(&source.path, &source.contents, adaptive).len();
    if candidate_count > 0 {
        score += 10;
        reasons.push(format!("supports {candidate_count} adaptive candidate(s)"));
    }

    PathScore { path: source.path.clone(), score, reasons }
}

fn adaptive_changes_for_target(
    path: &str,
    contents: &str,
    adaptive: &crate::domain::execution::AdaptiveExecutionProfile,
) -> Vec<WorkspaceChange> {
    let mut changes = Vec::new();
    for kind in adaptive.effective_change_kinds() {
        changes.extend(match kind {
            AdaptiveChangeKind::ArithmeticSwap => arithmetic_swap_candidates(path, contents),
            AdaptiveChangeKind::ComparisonFlip => comparison_flip_candidates(path, contents),
            AdaptiveChangeKind::BooleanFlip => boolean_flip_candidates(path, contents),
        });
    }
    changes
}

fn arithmetic_swap_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    let patterns = [
        (" - ", [" + ", " / ", " * "]),
        (" * ", [" - ", " + ", " / "]),
        (" / ", [" * ", " + ", " - "]),
        (" + ", [" - ", " * ", " / "]),
    ];

    for (find, replacements) in patterns {
        if contents.contains(find) {
            return replacements
                .into_iter()
                .map(|replace| WorkspaceChange {
                    path: path.to_string(),
                    find: find.to_string(),
                    replace: replace.to_string(),
                })
                .collect();
        }
    }

    Vec::new()
}

fn comparison_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    if contents.contains(" != ") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: " != ".to_string(),
            replace: " == ".to_string(),
        }];
    }

    if contents.contains(" == ") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: " == ".to_string(),
            replace: " != ".to_string(),
        }];
    }

    Vec::new()
}

fn boolean_flip_candidates(path: &str, contents: &str) -> Vec<WorkspaceChange> {
    if contents.contains("false") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "false".to_string(),
            replace: "true".to_string(),
        }];
    }

    if contents.contains("true") {
        return vec![WorkspaceChange {
            path: path.to_string(),
            find: "true".to_string(),
            replace: "false".to_string(),
        }];
    }

    Vec::new()
}

fn tokenize_terms(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .filter_map(|segment| {
            let term = segment.trim().to_ascii_lowercase();
            if term.len() >= 3 { Some(term) } else { None }
        })
        .collect()
}

fn adaptive_goal_hint(goal: &str, profile: &WorkspaceExecutionProfile) -> String {
    let trimmed = goal.trim();
    if trimmed.is_empty() { profile.name.clone() } else { trimmed.to_string() }
}

fn workspace_change_signature(change: &WorkspaceChange) -> String {
    format!("{}::{}=>{}", change.path, change.find, change.replace)
}

fn adaptive_candidate_signatures_from_state(state: &Map<String, Value>) -> BTreeSet<String> {
    state
        .get("adaptive_candidate_signatures")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn latest_attempt_id_from_state(state: &Map<String, Value>) -> Option<&str> {
    state.get("latest_attempt_id").and_then(Value::as_str)
}

fn adaptive_analysis_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
) -> Value {
    json!({
        "phase": "analyze",
        "execution_profile": profile.name,
        "read_targets": read_targets_for_profile(profile),
        "legacy_source": profile.legacy_source,
        "workspace_slice": candidate.workspace_slice,
        "selection_headline": &candidate.workspace_slice.headline,
        "selection_evidence": candidate.selection_evidence,
    })
}

fn adaptive_code_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
    extra: Value,
) -> Value {
    let mut input = extra.as_object().cloned().unwrap_or_default();
    input.insert("execution_profile".to_string(), json!(profile.name));
    input.insert("attempt_id".to_string(), json!(&candidate.attempt.attempt_id));
    input.insert("failure_mode".to_string(), json!(candidate.attempt.failure_mode));
    input.insert(
        "adaptive_attempt".to_string(),
        serde_json::to_value(&candidate.attempt).unwrap_or(Value::Null),
    );
    input.insert(
        "workspace_slice".to_string(),
        serde_json::to_value(&candidate.workspace_slice).unwrap_or(Value::Null),
    );
    input.insert(
        "selection_evidence".to_string(),
        serde_json::to_value(&candidate.selection_evidence).unwrap_or(Value::Null),
    );
    input.insert("selection_headline".to_string(), json!(&candidate.workspace_slice.headline));
    input.insert("candidate_signature".to_string(), json!(&candidate.candidate_signature));
    input.insert(
        "attempt_lineage".to_string(),
        serde_json::to_value(&candidate.attempt_lineage).unwrap_or(Value::Null),
    );
    Value::Object(input)
}

fn adaptive_verify_step_input(
    profile: &WorkspaceExecutionProfile,
    candidate: &AdaptiveAttemptPlan,
    extra: Value,
) -> Value {
    adaptive_code_step_input(profile, candidate, extra)
}

fn insert_adaptive_state_from_input(
    state_patch: &mut Map<String, Value>,
    input: &Value,
    existing_state: &Map<String, Value>,
) {
    if let Some(workspace_slice) = input.get("workspace_slice") {
        state_patch.insert("latest_workspace_slice".to_string(), workspace_slice.clone());
    }

    if let Some(selection_headline) = input.get("selection_headline") {
        state_patch.insert("latest_selection_headline".to_string(), selection_headline.clone());
    }

    if let Some(selection_evidence) = input.get("selection_evidence") {
        state_patch.insert("latest_selection_evidence".to_string(), selection_evidence.clone());
    }

    if let Some(attempt_lineage) = input.get("attempt_lineage") {
        state_patch.insert("latest_attempt_lineage".to_string(), attempt_lineage.clone());
    }

    if let Some(candidate_signature) = input.get("candidate_signature").and_then(Value::as_str) {
        let mut signatures = adaptive_candidate_signatures_from_state(existing_state);
        signatures.insert(candidate_signature.to_string());
        state_patch.insert("latest_candidate_signature".to_string(), json!(candidate_signature));
        state_patch.insert(
            "adaptive_candidate_signatures".to_string(),
            json!(signatures.into_iter().collect::<Vec<_>>()),
        );
    }
}

fn build_attempt_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;

    if let Some(active_flow) = active_flow {
        let code_id =
            format!("{}-replan-{}-code", active_flow.current_stage_id, attempt.attempt_id);
        let verify_id =
            format!("{}-replan-{}-verify", active_flow.current_stage_id, attempt.attempt_id);

        return Ok(vec![
            Step::agent(
                code_id,
                "coder",
                attach_current_stage_metadata(
                    code_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    )?,
                    active_flow,
                ),
            )?,
            Step::tool(
                verify_id,
                "tester",
                attach_current_stage_metadata(
                    verify_step_input(
                        profile,
                        attempt_index,
                        json!({
                            "phase": active_flow.current_stage_id,
                        }),
                    )?,
                    active_flow,
                ),
            )?,
        ]
        .into_iter()
        .chain(build_review_steps(profile, Some(active_flow), attempt_index)?)
        .collect());
    }

    Ok(vec![
        Step::agent(
            format!("code-{}", attempt.attempt_id),
            "coder",
            code_step_input(profile, attempt_index, json!({"phase": "code"}))?,
        )?,
        Step::tool(
            format!("verify-{}", attempt.attempt_id),
            "tester",
            verify_step_input(profile, attempt_index, json!({"phase": "verify"}))?,
        )?,
    ]
    .into_iter()
    .chain(build_review_steps(profile, None, attempt_index)?)
    .collect())
}

fn analysis_step_input(profile: &WorkspaceExecutionProfile) -> Value {
    json!({
        "phase": "analyze",
        "execution_profile": profile.name,
        "read_targets": read_targets_for_profile(profile),
        "legacy_source": profile.legacy_source,
    })
}

fn code_step_input(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
    extra: Value,
) -> Result<Value, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;
    let mut input = extra.as_object().cloned().unwrap_or_default();
    input.insert("execution_profile".to_string(), json!(profile.name));
    input.insert("attempt_index".to_string(), json!(attempt_index));
    input.insert("attempt_id".to_string(), json!(attempt.attempt_id));
    input.insert("failure_mode".to_string(), json!(attempt.failure_mode));
    Ok(Value::Object(input))
}

fn verify_step_input(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
    extra: Value,
) -> Result<Value, FixtureRuntimeError> {
    code_step_input(profile, attempt_index, extra)
}

fn build_review_steps(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_index: usize,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let attempt = execution_attempt(profile, attempt_index)?;
    build_review_steps_for_attempt(profile, active_flow, &attempt.attempt_id)
}

fn build_review_steps_for_attempt(
    profile: &WorkspaceExecutionProfile,
    active_flow: Option<&SessionFlowState>,
    attempt_id: &str,
) -> Result<Vec<Step>, FixtureRuntimeError> {
    let Some(review) = profile.review.as_ref() else {
        return Ok(Vec::new());
    };

    let prefix = active_flow
        .map(|flow| format!("{}-review-{}", flow.current_stage_id, attempt_id))
        .unwrap_or_else(|| format!("review-{}", attempt_id));

    let mut steps = Vec::new();
    for reviewer in &review.reviewers {
        steps.push(review_agent_step(
            format!("{}-{}", prefix, reviewer.reviewer_id),
            review_step_input_for_attempt(profile, attempt_id, reviewer.reviewer_id.clone(), false),
            active_flow,
        )?);
    }

    steps.push(review_tool_step(
        format!("{}-vote", prefix),
        review_vote_step_input_for_attempt(profile, attempt_id),
        "review-voter",
        active_flow,
    )?);

    if review.adjudication.enabled {
        let adjudicator_id = review
            .adjudication
            .reviewer_id
            .as_ref()
            .expect("validated review profile must define an adjudicator when enabled")
            .clone();
        steps.push(review_agent_step(
            format!("{}-adjudicate", prefix),
            review_step_input_for_attempt(profile, attempt_id, adjudicator_id, true),
            active_flow,
        )?);
    }

    steps.push(review_tool_step(
        format!("{}-finalize", prefix),
        review_finalize_step_input_for_attempt(profile, attempt_id),
        "review-finalizer",
        active_flow,
    )?);

    Ok(steps)
}

fn review_agent_step(
    id: String,
    input: Value,
    active_flow: Option<&SessionFlowState>,
) -> Result<Step, StepError> {
    match active_flow {
        Some(active_flow) => {
            Step::agent(id, "reviewer", attach_current_stage_metadata(input, active_flow))
        }
        None => Step::agent(id, "reviewer", input),
    }
}

fn review_tool_step(
    id: String,
    input: Value,
    target_name: &str,
    active_flow: Option<&SessionFlowState>,
) -> Result<Step, StepError> {
    match active_flow {
        Some(active_flow) => {
            Step::tool(id, target_name, attach_current_stage_metadata(input, active_flow))
        }
        None => Step::tool(id, target_name, input),
    }
}

fn review_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
    reviewer_id: String,
    adjudication: bool,
) -> Value {
    json!({
        "phase": "review",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
        "reviewer_id": reviewer_id,
        "adjudication": adjudication,
        "default_review_trigger": profile.review.as_ref().and_then(default_success_review_trigger),
    })
}

fn review_vote_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
) -> Value {
    json!({
        "phase": "review-vote",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
    })
}

fn review_finalize_step_input_for_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_id: &str,
) -> Value {
    json!({
        "phase": "review-finalize",
        "execution_profile": profile.name,
        "attempt_id": attempt_id,
    })
}

fn default_success_review_trigger(review: &ReviewProfile) -> Option<ReviewTrigger> {
    review
        .triggers
        .iter()
        .copied()
        .find(|trigger| !matches!(trigger, ReviewTrigger::ValidationFailed))
}

fn attach_current_stage_metadata(input: Value, active_flow: &SessionFlowState) -> Value {
    let mut input_object = input.as_object().cloned().unwrap_or_default();
    input_object.insert(
        FLOW_METADATA_KEY.to_string(),
        json!({
            "flow_name": active_flow.flow_name,
            "stage_id": active_flow.current_stage_id,
            "stage_index": active_flow.current_stage_index,
            "total_stages": active_flow.total_stages,
        }),
    );
    Value::Object(input_object)
}

fn read_targets_for_profile(profile: &WorkspaceExecutionProfile) -> Vec<String> {
    let mut targets = BTreeSet::new();
    for target in &profile.read_targets {
        targets.insert(target.clone());
    }

    if let Some(attempt) = profile.attempts.first() {
        for change in &attempt.changes {
            targets.insert(change.path.clone());
        }
    }

    targets.into_iter().collect()
}

fn execution_attempt(
    profile: &WorkspaceExecutionProfile,
    attempt_index: usize,
) -> Result<&ExecutionAttemptDefinition, FixtureRuntimeError> {
    profile.attempts.get(attempt_index).ok_or_else(|| FixtureRuntimeError::InvalidAttemptIndex {
        profile: profile.name.clone(),
        attempt_index,
    })
}

fn execution_attempt_from_request(
    profile: &WorkspaceExecutionProfile,
    request: &StepExecutionRequest,
) -> Result<ExecutionAttemptDefinition, FixtureRuntimeError> {
    if let Some(attempt) = request.input.get("adaptive_attempt") {
        return serde_json::from_value(attempt.clone()).map_err(|error| {
            FixtureRuntimeError::InvalidAdaptiveAttemptMetadata {
                profile: profile.name.clone(),
                message: error.to_string(),
            }
        });
    }

    let attempt_index =
        request.input.get("attempt_index").and_then(Value::as_u64).unwrap_or(0) as usize;
    execution_attempt(profile, attempt_index).cloned()
}

fn legacy_fixture_to_execution_profile(
    fixture: WorkspaceFixture,
) -> Result<WorkspaceExecutionProfile, FixtureRuntimeError> {
    let read_targets = fixture
        .file_patches
        .iter()
        .map(|patch| patch.path.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let attempts = vec![ExecutionAttemptDefinition {
        attempt_id: "legacy-attempt-1".to_string(),
        summary: format!("Converted legacy fixture {}", fixture.name),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: fixture
            .file_patches
            .into_iter()
            .map(|patch| WorkspaceChange {
                path: patch.path,
                find: patch.find,
                replace: patch.replace,
            })
            .collect(),
    }];

    let profile = WorkspaceExecutionProfile {
        name: fixture.name,
        read_targets,
        validation_command: ExecutionCommand {
            program: fixture.test_command.program,
            args: fixture.test_command.args,
        },
        attempts,
        adaptive: None,
        limits: fixture.limits,
        governance: None,
        review: None,
        legacy_source: Some(FIXTURE_RELATIVE_PATH.to_string()),
    };
    profile.validate()?;
    Ok(profile)
}

fn analyze_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    match snapshot_workspace_targets(workspace, &read_targets_for_profile(profile)) {
        Ok(snapshots) => {
            let mut state_patch = Map::new();
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "analysis_targets": snapshots,
                "legacy_source": profile.legacy_source,
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            if state_patch.is_empty() {
                StepExecutionResult::success(rendered_output)
            } else {
                rendered_output["workspace_slice"] =
                    request.input.get("workspace_slice").cloned().unwrap_or(Value::Null);
                rendered_output["selection_evidence"] =
                    request.input.get("selection_evidence").cloned().unwrap_or(Value::Null);
                StepExecutionResult::success_with_patch(rendered_output, state_patch)
            }
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_analysis_failed",
                format!("failed to snapshot the workspace before delivery: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn apply_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    if request.input.get("force_retry_once").and_then(Value::as_bool).unwrap_or(false)
        && request.attempt_number == 1
    {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_retry_once",
                format!(
                    "workspace execution profile '{}' intentionally requests one retry before applying changes",
                    profile.name
                ),
            ),
            Recoverability::Retryable,
        );
    }

    let attempt = match execution_attempt_from_request(profile, &request) {
        Ok(attempt) => attempt,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_execution_attempt",
                    format!("invalid execution attempt metadata: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    match apply_execution_attempt(workspace, &attempt) {
        Ok(report) => {
            let changed_files = if report.updated_files.is_empty() {
                report.already_applied_files.clone()
            } else {
                report.updated_files.clone()
            };
            let mut state_patch = Map::new();
            state_patch.insert("latest_attempt_id".to_string(), json!(attempt.attempt_id));
            state_patch.insert("latest_changed_files".to_string(), json!(changed_files));
            state_patch.insert(
                "latest_change_evidence".to_string(),
                serde_json::to_value(&report.change_evidence).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "change_applied": true,
                "changed_files": changed_files,
                "already_applied_files": report.already_applied_files,
                "change_evidence": report.change_evidence,
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }

            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_change_failed",
                format!("failed to apply the workspace change set: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn verify_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let attempt = match execution_attempt_from_request(profile, &request) {
        Ok(attempt) => attempt,
        Err(error) => {
            return StepExecutionResult::failure(
                ErrorInfo::new(
                    "invalid_execution_attempt",
                    format!("invalid execution attempt metadata: {error}"),
                ),
                Recoverability::Terminal,
            );
        }
    };

    match run_execution_command(workspace, &profile.validation_command) {
        Ok(output) if output.succeeded() => {
            let record = output.to_validation_record();
            let mut state_patch = Map::new();
            state_patch.insert("latest_validation_status".to_string(), json!("passed"));
            state_patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(&record).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            if let Some(trigger) = profile.review.as_ref().and_then(default_success_review_trigger)
            {
                state_patch.insert("next_review_trigger".to_string(), json!(trigger));
            } else {
                state_patch.insert("goal_satisfied".to_string(), json!(true));
            }
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "validation": record,
                "review_trigger": profile.review.as_ref().and_then(default_success_review_trigger),
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }
            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Ok(output)
            if attempt.failure_mode == ExecutionFailureMode::Terminal
                && profile.review.as_ref().is_some_and(|review| {
                    review.triggers.contains(&ReviewTrigger::ValidationFailed)
                }) =>
        {
            let record = output.to_validation_record();
            let mut state_patch = Map::new();
            state_patch.insert("latest_validation_status".to_string(), json!("failed"));
            state_patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(&record).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut state_patch,
                &request.input,
                &request.task_snapshot.state,
            );
            state_patch
                .insert("next_review_trigger".to_string(), json!(ReviewTrigger::ValidationFailed));
            let governance_context = governance_context_from_request(&request);
            let mut rendered_output = json!({
                "execution_profile": profile.name,
                "attempt_id": attempt.attempt_id,
                "validation": record,
                "review_trigger": ReviewTrigger::ValidationFailed,
            });
            if let Some(governance_context) = governance_context {
                rendered_output["governance_context"] = governance_context;
            }
            StepExecutionResult::success_with_patch(rendered_output, state_patch)
        }
        Ok(output) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_validation_failed",
                format!(
                    "workspace execution profile '{}' still fails validation after attempt {}",
                    profile.name, attempt.attempt_id
                ),
            )
            .with_details(output.details()),
            attempt.failure_mode.recoverability(),
        )
        .with_state_patch({
            let mut patch = Map::new();
            patch.insert("latest_validation_status".to_string(), json!("failed"));
            patch.insert(
                "latest_validation_record".to_string(),
                serde_json::to_value(output.to_validation_record()).unwrap_or(Value::Null),
            );
            insert_adaptive_state_from_input(
                &mut patch,
                &request.input,
                &request.task_snapshot.state,
            );
            patch
        }),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "execution_verify_failed",
                format!("failed to execute the validation command: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn review_workspace_fixture(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success(json!({"review_skipped": true}));
    };

    let Some(reviewer_id) = request.input.get("reviewer_id").and_then(Value::as_str) else {
        return StepExecutionResult::failure(
            ErrorInfo::new("missing_reviewer_id", "review step is missing reviewer_id metadata"),
            Recoverability::Terminal,
        );
    };
    let adjudication = request.input.get("adjudication").and_then(Value::as_bool).unwrap_or(false);
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success(json!({
            "review_skipped": true,
            "reviewer_id": reviewer_id,
        }));
    };

    let (reviewer_role, reviewer_source) = match review.reviewer_by_id(reviewer_id) {
        Some(reviewer) => (reviewer.role.clone(), reviewer.source.clone()),
        None if adjudication => ("Adjudicator".to_string(), None),
        None => {
            return review_terminal_failure(
                "unknown_reviewer",
                format!("reviewer '{reviewer_id}' is not configured in the review council"),
                Some(trigger),
                reviewer_id,
            );
        }
    };

    let Some(scenario) = review.scenario_for(trigger) else {
        return review_terminal_failure(
            "missing_review_scenario",
            format!("review trigger '{trigger:?}' does not define a review scenario"),
            Some(trigger),
            reviewer_id,
        );
    };

    let finding = if adjudication {
        scenario.adjudication_finding.as_ref()
    } else {
        scenario.findings.iter().find(|finding| finding.reviewer_id == reviewer_id)
    };
    let Some(finding) = finding else {
        return review_terminal_failure(
            "missing_review_finding",
            format!("reviewer '{reviewer_id}' did not produce a configured finding"),
            Some(trigger),
            reviewer_id,
        );
    };

    let mut findings = review_findings_from_state(&request);
    findings.push(finding.clone());
    let mut reviewers = review_reviewer_ids_from_state(&request);
    if !reviewers.contains(&reviewer_id.to_string()) {
        reviewers.push(reviewer_id.to_string());
    }
    let mut participants = review_participants_from_state(&request);
    participants.push(ReviewerParticipation {
        reviewer_id: reviewer_id.to_string(),
        status: ReviewerParticipationStatus::Completed,
        reason: None,
    });

    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert(
        "latest_review_findings".to_string(),
        serde_json::to_value(&findings).unwrap_or(Value::Null),
    );
    state_patch.insert("latest_reviewers".to_string(), json!(reviewers));
    state_patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(&participants).unwrap_or(Value::Null),
    );
    if adjudication {
        state_patch.insert(
            "latest_review_adjudication".to_string(),
            serde_json::to_value(finding).unwrap_or(Value::Null),
        );
    }

    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "reviewer_id": reviewer_id,
            "reviewer_role": reviewer_role,
            "reviewer_source": reviewer_source,
            "finding": finding,
            "adjudication": adjudication,
        }),
        state_patch,
    )
}

fn governance_context_from_request(request: &StepExecutionRequest) -> Option<Value> {
    let metadata = FlowStepMetadata::from_value(request.input.get(FLOW_METADATA_KEY)?).ok()??;
    let reused_packets = bounded_reused_packets(&request.task_snapshot, &metadata).ok()?;
    if reused_packets.is_empty() {
        return None;
    }
    let reuse_binding =
        select_packet_reuse_binding(&request.task_snapshot, &metadata).ok().flatten();

    Some(json!({
        "reused_packets": reused_packets,
        "reuse_binding": reuse_binding,
    }))
}

fn resolve_review_vote(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };

    let findings = review_findings_from_state(&request);
    let resolution = match review.vote_rule.resolve(&review.reviewers, &findings) {
        Ok(resolution) => resolution,
        Err(error) => {
            return review_terminal_failure(
                "invalid_review_vote",
                format!("review vote could not be resolved: {error}"),
                Some(trigger),
                "review-voter",
            );
        }
    };

    if resolution
        .participants
        .iter()
        .any(|participant| participant.status != ReviewerParticipationStatus::Completed)
    {
        return review_terminal_failure(
            "incomplete_review_participation",
            "one or more configured reviewers did not complete the review",
            Some(trigger),
            "review-voter",
        );
    }

    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(&resolution.participants).unwrap_or(Value::Null),
    );
    state_patch.insert(
        "latest_review_vote_resolution".to_string(),
        serde_json::to_value(&resolution).unwrap_or(Value::Null),
    );
    state_patch.insert("latest_review_vote".to_string(), json!(render_vote_summary(&resolution)));
    state_patch.insert("latest_review_vote_decision".to_string(), json!(resolution.decision));

    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "vote": resolution,
        }),
        state_patch,
    )
}

fn finalize_workspace_review(
    profile: &WorkspaceExecutionProfile,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    let Some(review) = profile.review.as_ref() else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };
    let Some(trigger) = active_review_trigger(review, &request) else {
        return StepExecutionResult::success_with_patch(
            json!({"review_skipped": true}),
            Map::from_iter([(String::from("goal_satisfied"), json!(true))]),
        );
    };

    let vote_decision = request
        .task_snapshot
        .state
        .get("latest_review_vote_decision")
        .cloned()
        .and_then(|value| serde_json::from_value::<VoteDecision>(value).ok());

    match vote_decision {
        Some(VoteDecision::Accepted) => review_terminal_success(trigger, ReviewOutcome::Accepted),
        Some(VoteDecision::Rejected) => review_terminal_rejection(trigger),
        Some(VoteDecision::NeedsAdjudication) if review.adjudication.enabled => {
            let adjudication = request
                .task_snapshot
                .state
                .get("latest_review_adjudication")
                .cloned()
                .and_then(|value| serde_json::from_value::<ReviewerFinding>(value).ok());
            match adjudication.map(|finding| finding.disposition) {
                Some(ReviewerDisposition::Approve) => {
                    review_terminal_success(trigger, ReviewOutcome::Accepted)
                }
                Some(ReviewerDisposition::Block) => review_terminal_rejection(trigger),
                Some(ReviewerDisposition::Concern) => review_terminal_escalation(trigger),
                None => review_terminal_failure(
                    "missing_adjudication",
                    "review required adjudication but no adjudication finding was recorded",
                    Some(trigger),
                    "review-finalizer",
                ),
            }
        }
        Some(VoteDecision::NeedsAdjudication) => review_terminal_escalation(trigger),
        None => review_terminal_failure(
            "missing_review_vote",
            "review finalizer could not find a resolved vote decision",
            Some(trigger),
            "review-finalizer",
        ),
    }
}

fn review_terminal_success(trigger: ReviewTrigger, outcome: ReviewOutcome) -> StepExecutionResult {
    let mut state_patch = Map::new();
    state_patch.insert("latest_review_trigger".to_string(), json!(trigger));
    state_patch.insert("latest_review_outcome".to_string(), json!(outcome));
    state_patch.insert("goal_satisfied".to_string(), json!(true));
    StepExecutionResult::success_with_patch(
        json!({
            "review_trigger": trigger,
            "review_outcome": outcome,
        }),
        state_patch,
    )
}

fn review_terminal_rejection(trigger: ReviewTrigger) -> StepExecutionResult {
    let mut patch = Map::new();
    patch.insert("latest_review_trigger".to_string(), json!(trigger));
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Rejected));
    StepExecutionResult::failure(
        ErrorInfo::new(
            "review_rejected",
            format!("review trigger '{trigger:?}' rejected the delivery result"),
        ),
        Recoverability::Terminal,
    )
    .with_state_patch(patch)
}

fn review_terminal_escalation(trigger: ReviewTrigger) -> StepExecutionResult {
    let mut patch = Map::new();
    patch.insert("latest_review_trigger".to_string(), json!(trigger));
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Escalated));
    StepExecutionResult::failure(
        ErrorInfo::new(
            "review_escalated",
            format!("review trigger '{trigger:?}' ended in escalation"),
        ),
        Recoverability::Terminal,
    )
    .with_state_patch(patch)
}

fn review_terminal_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    trigger: Option<ReviewTrigger>,
    reviewer_id: &str,
) -> StepExecutionResult {
    let message = message.into();
    let mut patch = Map::new();
    if let Some(trigger) = trigger {
        patch.insert("latest_review_trigger".to_string(), json!(trigger));
    }
    patch.insert("latest_review_outcome".to_string(), json!(ReviewOutcome::Failed));
    patch.insert(
        "latest_review_participants".to_string(),
        serde_json::to_value(vec![ReviewerParticipation {
            reviewer_id: reviewer_id.to_string(),
            status: ReviewerParticipationStatus::Failed,
            reason: Some(message.clone()),
        }])
        .unwrap_or(Value::Null),
    );
    StepExecutionResult::failure(ErrorInfo::new(code, message), Recoverability::Terminal)
        .with_state_patch(patch)
}

fn active_review_trigger(
    review: &ReviewProfile,
    request: &StepExecutionRequest,
) -> Option<ReviewTrigger> {
    request
        .task_snapshot
        .state
        .get("next_review_trigger")
        .cloned()
        .or_else(|| request.input.get("default_review_trigger").cloned())
        .and_then(|value| serde_json::from_value::<ReviewTrigger>(value).ok())
        .filter(|trigger| review.triggers.contains(trigger))
}

fn review_findings_from_state(request: &StepExecutionRequest) -> Vec<ReviewerFinding> {
    request
        .task_snapshot
        .state
        .get("latest_review_findings")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerFinding>>(value).ok())
        .unwrap_or_default()
}

fn review_reviewer_ids_from_state(request: &StepExecutionRequest) -> Vec<String> {
    request
        .task_snapshot
        .state
        .get("latest_reviewers")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn review_participants_from_state(request: &StepExecutionRequest) -> Vec<ReviewerParticipation> {
    request
        .task_snapshot
        .state
        .get("latest_review_participants")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<ReviewerParticipation>>(value).ok())
        .unwrap_or_default()
}

fn render_vote_summary(resolution: &VoteResolution) -> String {
    format!(
        "strategy={:?} approvals={} concerns={} blocks={} decision={:?}",
        resolution.strategy,
        resolution.approvals,
        resolution.concerns,
        resolution.blocks,
        resolution.decision
    )
}

fn snapshot_workspace_targets(
    workspace: &Path,
    targets: &[String],
) -> Result<Vec<Value>, FixtureRuntimeError> {
    targets
        .iter()
        .map(|target| {
            let path = workspace.join(target);
            let contents = fs::read_to_string(&path)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            Ok(json!({
                "path": target,
                "preview": excerpt(&contents),
            }))
        })
        .collect()
}

fn apply_execution_attempt(
    workspace: &Path,
    attempt: &ExecutionAttemptDefinition,
) -> Result<ExecutionAttemptReport, FixtureRuntimeError> {
    let mut updated_files = Vec::new();
    let mut already_applied_files = Vec::new();
    let mut change_evidence = Vec::new();

    for change in &attempt.changes {
        let path = workspace.join(&change.path);
        let contents = fs::read_to_string(&path)
            .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;

        if contents.contains(&change.find) {
            let updated = contents.replacen(&change.find, &change.replace, 1);
            fs::write(&path, updated)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            updated_files.push(change.path.clone());
            change_evidence.push(ChangeEvidence {
                path: change.path.clone(),
                change_status: ChangeStatus::Updated,
                before_excerpt: excerpt(&change.find),
                after_excerpt: excerpt(&change.replace),
                diff_preview: diff_preview(&change.find, &change.replace),
            });
            continue;
        }

        if contents.contains(&change.replace) {
            already_applied_files.push(change.path.clone());
            change_evidence.push(ChangeEvidence {
                path: change.path.clone(),
                change_status: ChangeStatus::AlreadyApplied,
                before_excerpt: excerpt(&change.find),
                after_excerpt: excerpt(&change.replace),
                diff_preview: diff_preview(&change.find, &change.replace),
            });
            continue;
        }

        return Err(FixtureRuntimeError::PatchTargetMissing { path, needle: change.find.clone() });
    }

    Ok(ExecutionAttemptReport { updated_files, already_applied_files, change_evidence })
}

#[cfg(test)]
fn apply_fixture_patches(
    workspace: &Path,
    fixture: &WorkspaceFixture,
) -> Result<PatchReport, FixtureRuntimeError> {
    let attempt = ExecutionAttemptDefinition {
        attempt_id: "legacy-attempt-1".to_string(),
        summary: format!("Legacy patch application for {}", fixture.name),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: fixture
            .file_patches
            .iter()
            .map(|patch| WorkspaceChange {
                path: patch.path.clone(),
                find: patch.find.clone(),
                replace: patch.replace.clone(),
            })
            .collect(),
    };
    let report = apply_execution_attempt(workspace, &attempt)?;

    Ok(PatchReport {
        updated_files: report.updated_files,
        already_applied_files: report.already_applied_files,
    })
}

#[cfg(test)]
#[allow(dead_code)]
fn run_fixture_command(
    workspace: &Path,
    command: &FixtureCommand,
) -> Result<FixtureCommandOutput, FixtureRuntimeError> {
    run_execution_command(
        workspace,
        &ExecutionCommand { program: command.program.clone(), args: command.args.clone() },
    )
}

fn run_execution_command(
    workspace: &Path,
    command: &ExecutionCommand,
) -> Result<FixtureCommandOutput, FixtureRuntimeError> {
    let rendered_command = render_command(command);
    let output = Command::new(&command.program)
        .args(&command.args)
        .current_dir(workspace)
        .output()
        .map_err(|source| FixtureRuntimeError::CommandLaunch {
            command: rendered_command.clone(),
            source,
        })?;

    Ok(FixtureCommandOutput {
        rendered_command,
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn render_command(command: &ExecutionCommand) -> String {
    if command.args.is_empty() {
        command.program.clone()
    } else {
        format!("{} {}", command.program, command.args.join(" "))
    }
}

fn default_test_command() -> FixtureCommand {
    FixtureCommand {
        program: "cargo".to_string(),
        args: vec!["test".to_string(), "--quiet".to_string()],
    }
}

fn default_run_limits() -> RunLimits {
    RunLimits { max_steps: 3, max_retries: 0, max_replans: 0, ..RunLimits::default() }
}

#[cfg(test)]
struct PatchReport {
    updated_files: Vec<String>,
    already_applied_files: Vec<String>,
}

struct ExecutionAttemptReport {
    updated_files: Vec<String>,
    already_applied_files: Vec<String>,
    change_evidence: Vec<ChangeEvidence>,
}

struct FixtureCommandOutput {
    rendered_command: String,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl FixtureCommandOutput {
    fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    fn rendered_command(&self) -> &str {
        &self.rendered_command
    }

    fn details(&self) -> Value {
        json!({
            "command": self.rendered_command,
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
        })
    }

    fn to_validation_record(&self) -> ValidationRecord {
        ValidationRecord {
            command: self.rendered_command.clone(),
            exit_code: self.exit_code.unwrap_or(-1),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            succeeded: self.succeeded(),
        }
    }
}

fn excerpt(text: &str) -> String {
    const MAX_LEN: usize = 96;
    if text.len() <= MAX_LEN {
        return text.to_string();
    }

    format!("{}...", &text[..MAX_LEN])
}

fn diff_preview(before: &str, after: &str) -> String {
    format!("- {}\n+ {}", excerpt(before), excerpt(after))
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FixtureValidationError {
    #[error("workspace fixture requires a stable name")]
    MissingName,
    #[error("workspace fixture requires a test command program")]
    MissingTestProgram,
    #[error("workspace fixture requires at least one file patch")]
    MissingFilePatches,
    #[error("workspace fixture run limits are invalid: {0}")]
    InvalidRunLimits(String),
    #[error("workspace fixture file patch requires a path")]
    MissingPatchPath,
    #[error("workspace fixture file patch path must be relative: {0}")]
    AbsolutePatchPath(String),
    #[error("workspace fixture file patch requires a search pattern for {0}")]
    MissingFindPattern(String),
}

#[derive(Debug, Error)]
pub enum FixtureRuntimeError {
    #[error(
        "workspace execution profile is missing; looked for {preferred} and legacy fallback {legacy}"
    )]
    MissingExecutionProfile { preferred: PathBuf, legacy: PathBuf },
    #[error("workspace fixture is missing at {0}")]
    MissingFixture(PathBuf),
    #[error("failed to read workspace execution profile from {path}: {source}")]
    ExecutionProfileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("workspace execution profile is invalid at {path}: {source}")]
    ExecutionProfileParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("workspace execution profile is invalid: {0}")]
    ExecutionProfileValidation(#[from] ExecutionProfileError),
    #[error("failed to read workspace fixture from {path}: {source}")]
    FixtureRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("workspace fixture is invalid at {path}: {source}")]
    FixtureParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("workspace fixture is invalid: {0}")]
    FixtureValidation(#[from] FixtureValidationError),
    #[error("workspace fixture flow metadata is invalid: {0}")]
    FlowValidation(#[from] crate::domain::flow::FlowValidationError),
    #[error("workspace vertical slice contains an invalid step: {0}")]
    InvalidStep(#[from] StepError),
    #[error("workspace vertical slice contains an invalid plan: {0}")]
    InvalidPlan(#[from] crate::domain::plan::PlanError),
    #[error("failed to register fixture agent: {0}")]
    AgentRegistry(#[from] AgentRegistryError),
    #[error("failed to register fixture tool: {0}")]
    ToolRegistry(#[from] ToolRegistryError),
    #[error("failed to read or write {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("fixture patch search pattern was not found in {path}: {needle}")]
    PatchTargetMissing { path: PathBuf, needle: String },
    #[error("execution profile '{profile}' does not define attempt index {attempt_index}")]
    InvalidAttemptIndex { profile: String, attempt_index: usize },
    #[error("execution profile '{profile}' does not define a credible adaptive candidate")]
    NoAdaptiveCandidate { profile: String },
    #[error("execution profile '{profile}' returned invalid adaptive attempt metadata: {message}")]
    InvalidAdaptiveAttemptMetadata { profile: String, message: String },
    #[error("failed to execute fixture command `{command}`: {source}")]
    CommandLaunch {
        command: String,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::{Map, json};
    use uuid::Uuid;

    use super::{
        ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, FilePatch,
        FixtureRuntimeError, WorkspaceChange, WorkspaceExecutionProfile, WorkspaceFixture,
        analyze_workspace_fixture, apply_fixture_patches, apply_workspace_fixture,
        build_fixture_plan, build_fixture_runtime, build_task_request, build_vertical_slice_plan,
        execution_manifest_path, load_workspace_execution_profile, load_workspace_fixture,
        resolve_review_vote, run_fixture_command, verify_workspace_fixture,
    };
    use crate::domain::flow::{attach_stage_metadata, built_in_flow};
    use crate::domain::governance::{
        ApprovalState, CanonMode, GovernanceLifecycleState, GovernanceRuntimeKind,
        GovernedStagePacket, GovernedStageRecord, PacketReadiness,
    };
    use crate::domain::limits::RunLimits;
    use crate::domain::review::{
        ReviewProfile, ReviewScenario, ReviewTrigger, ReviewerDefinition, ReviewerDisposition,
        ReviewerFinding, VoteDecision, VoteRuleDefinition,
    };
    use crate::domain::step::{ExecutionStatus, Recoverability, StepExecutionRequest, StepKind};
    use crate::domain::task_context::TaskContext;

    fn temp_workspace() -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("synod-fixture-unit-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        workspace
    }

    fn write_execution_workspace(prefix: &str, source: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join("src")).unwrap();
        fs::create_dir_all(workspace.join("tests")).unwrap();
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            "[package]\nname = \"fixture_unit\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        fs::write(workspace.join("src/lib.rs"), source).unwrap();
        fs::write(
            workspace.join("tests/red_to_green.rs"),
            "#[test]\nfn red_to_green_addition() {\n    assert_eq!(fixture_unit::add(2, 2), 4);\n}\n",
        )
        .unwrap();
        workspace
    }

    fn sample_profile(validation_command: ExecutionCommand) -> WorkspaceExecutionProfile {
        WorkspaceExecutionProfile {
            name: "fixture-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
            validation_command,
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: "Replace subtraction with addition".to_string(),
                failure_mode: ExecutionFailureMode::Retry,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: None,
            review: None,
            legacy_source: None,
        }
    }

    fn sample_review_profile(validation_command: ExecutionCommand) -> WorkspaceExecutionProfile {
        let mut profile = sample_profile(validation_command);
        profile.review = Some(ReviewProfile {
            triggers: vec![ReviewTrigger::PrReady, ReviewTrigger::ValidationFailed],
            reviewers: vec![
                ReviewerDefinition {
                    reviewer_id: "safety".to_string(),
                    role: "Safety".to_string(),
                    source: Some("gpt".to_string()),
                    weight: 2,
                },
                ReviewerDefinition {
                    reviewer_id: "maintainability".to_string(),
                    role: "Maintainability".to_string(),
                    source: Some("claude".to_string()),
                    weight: 1,
                },
            ],
            vote_rule: VoteRuleDefinition::default(),
            adjudication: Default::default(),
            scenarios: vec![
                ReviewScenario {
                    trigger: ReviewTrigger::PrReady,
                    findings: vec![
                        ReviewerFinding {
                            reviewer_id: "safety".to_string(),
                            disposition: ReviewerDisposition::Approve,
                            summary: "No blocking issues".to_string(),
                            details: None,
                        },
                        ReviewerFinding {
                            reviewer_id: "maintainability".to_string(),
                            disposition: ReviewerDisposition::Approve,
                            summary: "Looks ready".to_string(),
                            details: None,
                        },
                    ],
                    adjudication_finding: None,
                },
                ReviewScenario {
                    trigger: ReviewTrigger::ValidationFailed,
                    findings: vec![
                        ReviewerFinding {
                            reviewer_id: "safety".to_string(),
                            disposition: ReviewerDisposition::Block,
                            summary: "Validation still fails".to_string(),
                            details: None,
                        },
                        ReviewerFinding {
                            reviewer_id: "maintainability".to_string(),
                            disposition: ReviewerDisposition::Concern,
                            summary: "Retry after a fix".to_string(),
                            details: None,
                        },
                    ],
                    adjudication_finding: None,
                },
            ],
        });
        profile
    }

    fn request(input: serde_json::Value, attempt_number: usize) -> StepExecutionRequest {
        StepExecutionRequest {
            step_id: "code".to_string(),
            step_kind: StepKind::Agent,
            target_name: "coder".to_string(),
            input,
            task_snapshot: TaskContext::new(
                "session-1",
                "/tmp/workspace",
                RunLimits::default(),
                Map::new(),
            ),
            attempt_number,
        }
    }

    fn request_with_state(
        input: serde_json::Value,
        attempt_number: usize,
        state: Map<String, serde_json::Value>,
    ) -> StepExecutionRequest {
        StepExecutionRequest {
            step_id: "review".to_string(),
            step_kind: StepKind::Tool,
            target_name: "review-voter".to_string(),
            input,
            task_snapshot: TaskContext::new(
                "session-1",
                "/tmp/workspace",
                RunLimits::default(),
                state,
            ),
            attempt_number,
        }
    }

    #[test]
    fn loader_rejects_a_missing_manifest() {
        let workspace = temp_workspace();
        let error = load_workspace_fixture(&workspace).unwrap_err();

        assert!(matches!(error, FixtureRuntimeError::MissingFixture(_)));
    }

    #[test]
    fn patch_application_is_idempotent_after_the_replacement_is_present() {
        let workspace = temp_workspace();
        let source_path = workspace.join("src.txt");
        fs::write(&source_path, "red").unwrap();
        let fixture = WorkspaceFixture {
            name: "bug-fix".to_string(),
            test_command: super::default_test_command(),
            limits: super::default_run_limits(),
            file_patches: vec![FilePatch {
                path: "src.txt".to_string(),
                find: "red".to_string(),
                replace: "green".to_string(),
            }],
        };

        let first = apply_fixture_patches(&workspace, &fixture).unwrap();
        let second = apply_fixture_patches(&workspace, &fixture).unwrap();

        assert_eq!(first.updated_files, vec!["src.txt".to_string()]);
        assert_eq!(second.already_applied_files, vec!["src.txt".to_string()]);
        assert_eq!(fs::read_to_string(source_path).unwrap(), "green");
    }

    #[test]
    fn workspace_fixture_validation_rejects_missing_fields_and_invalid_patches() {
        assert!(matches!(
            WorkspaceFixture {
                name: " ".to_string(),
                test_command: super::default_test_command(),
                limits: super::default_run_limits(),
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingName
        ));
        assert!(matches!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::FixtureCommand { program: " ".to_string(), args: vec![] },
                limits: super::default_run_limits(),
                file_patches: vec![FilePatch {
                    path: "src/lib.rs".to_string(),
                    find: "red".to_string(),
                    replace: "green".to_string(),
                }],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingTestProgram
        ));
        assert!(matches!(
            WorkspaceFixture {
                name: "fixture".to_string(),
                test_command: super::default_test_command(),
                limits: super::default_run_limits(),
                file_patches: vec![],
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingFilePatches
        ));
        assert!(matches!(
            FilePatch {
                path: " ".to_string(),
                find: "red".to_string(),
                replace: "green".to_string()
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingPatchPath
        ));
        assert!(matches!(
            FilePatch {
                path: "/tmp/outside.rs".to_string(),
                find: "red".to_string(),
                replace: "green".to_string(),
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::AbsolutePatchPath(_)
        ));
        assert!(matches!(
            FilePatch {
                path: "src/lib.rs".to_string(),
                find: "".to_string(),
                replace: "green".to_string()
            }
            .validate()
            .unwrap_err(),
            super::FixtureValidationError::MissingFindPattern(_)
        ));
    }

    #[test]
    fn load_workspace_execution_profile_reports_parse_and_missing_errors() {
        let workspace = temp_workspace();
        fs::write(execution_manifest_path(&workspace), b"{not json").unwrap();
        assert!(matches!(
            load_workspace_execution_profile(&workspace).unwrap_err(),
            FixtureRuntimeError::ExecutionProfileParse { .. }
        ));

        fs::remove_file(execution_manifest_path(&workspace)).unwrap();
        assert!(matches!(
            load_workspace_execution_profile(&workspace).unwrap_err(),
            FixtureRuntimeError::MissingExecutionProfile { .. }
        ));
    }

    #[test]
    fn build_vertical_slice_plan_covers_non_flow_and_all_built_in_flows() {
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let direct = build_vertical_slice_plan(&profile, None, 0).unwrap();
        assert_eq!(direct.steps[0].id, "analyze");
        assert_eq!(direct.steps[1].id, "code-fix-add");
        assert_eq!(direct.steps[2].id, "verify-fix-add");

        let bug_fix = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("bug-fix").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            bug_fix.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["investigate", "implement", "verify"]
        );

        let change = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("change").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            change.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["understand-change", "implement", "verify"]
        );

        let delivery = build_vertical_slice_plan(
            &profile,
            Some(&built_in_flow("delivery").unwrap().initial_state()),
            0,
        )
        .unwrap();
        assert_eq!(
            delivery.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "requirements",
                "architecture",
                "backlog",
                "implementation-code",
                "implementation-verify"
            ]
        );
    }

    #[test]
    fn build_vertical_slice_plan_appends_review_steps_when_review_is_configured() {
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let direct = build_vertical_slice_plan(&profile, None, 0).unwrap();

        assert_eq!(
            direct.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec![
                "analyze",
                "code-fix-add",
                "verify-fix-add",
                "review-fix-add-safety",
                "review-fix-add-maintainability",
                "review-fix-add-vote",
                "review-fix-add-finalize",
            ]
        );
    }

    #[test]
    fn review_validation_failure_is_routed_into_review_state() {
        let workspace = write_execution_workspace(
            "synod-fixture-review-validation-failure",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let mut profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        profile.attempts[0].failure_mode = ExecutionFailureMode::Terminal;

        let result =
            verify_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 1));

        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            result.state_patch.as_ref().unwrap()["next_review_trigger"],
            json!(ReviewTrigger::ValidationFailed)
        );
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("failed")
        );
    }

    #[test]
    fn review_vote_resolution_succeeds_for_pr_ready_findings() {
        let profile = sample_review_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let mut state = Map::new();
        state.insert("next_review_trigger".to_string(), json!(ReviewTrigger::PrReady));
        state.insert(
            "latest_review_findings".to_string(),
            serde_json::to_value(profile.review.as_ref().unwrap().scenarios[0].findings.clone())
                .unwrap(),
        );

        let result = resolve_review_vote(&profile, request_with_state(json!({}), 1, state));

        assert_eq!(result.status, ExecutionStatus::Succeeded);
        assert_eq!(
            result.state_patch.as_ref().unwrap()["latest_review_vote_decision"],
            json!(VoteDecision::Accepted)
        );
    }

    #[test]
    fn apply_workspace_fixture_covers_retry_invalid_attempt_success_and_already_applied() {
        let workspace = write_execution_workspace(
            "synod-fixture-apply",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let retry = apply_workspace_fixture(
            &workspace,
            &profile,
            request(json!({"force_retry_once": true, "attempt_index": 0}), 1),
        );
        assert_eq!(retry.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(retry.recoverability, Recoverability::Retryable);

        let invalid_attempt =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 9}), 2));
        assert_eq!(invalid_attempt.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(invalid_attempt.recoverability, Recoverability::Terminal);

        let applied =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 2));
        assert_eq!(applied.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(applied.output.as_ref().unwrap()["changed_files"], json!(["src/lib.rs"]));
        assert_eq!(applied.state_patch.as_ref().unwrap()["latest_attempt_id"], json!("fix-add"));

        let already_applied =
            apply_workspace_fixture(&workspace, &profile, request(json!({"attempt_index": 0}), 3));
        assert_eq!(already_applied.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            already_applied.output.as_ref().unwrap()["already_applied_files"],
            json!(["src/lib.rs"])
        );
    }

    #[test]
    fn verify_workspace_fixture_covers_success_failure_and_command_errors() {
        let success_workspace = write_execution_workspace(
            "synod-fixture-verify-success",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let retry_profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });

        let success = verify_workspace_fixture(
            &success_workspace,
            &retry_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(success.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            success.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("passed")
        );

        let failure_workspace = write_execution_workspace(
            "synod-fixture-verify-failure",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        );
        let failure = verify_workspace_fixture(
            &failure_workspace,
            &retry_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(failure.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(failure.recoverability, Recoverability::Retryable);
        assert_eq!(
            failure.state_patch.as_ref().unwrap()["latest_validation_status"],
            json!("failed")
        );

        let error_profile = sample_profile(ExecutionCommand {
            program: "definitely-not-a-real-command".to_string(),
            args: vec![],
        });
        let command_error = verify_workspace_fixture(
            &failure_workspace,
            &error_profile,
            request(json!({"attempt_index": 0}), 1),
        );
        assert_eq!(command_error.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(command_error.recoverability, Recoverability::Terminal);
    }

    #[test]
    fn fixture_runtime_helpers_cover_analysis_builders_and_verify_invalid_attempts() {
        let workspace = write_execution_workspace(
            "synod-fixture-runtime-helpers",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        fs::write(
            execution_manifest_path(&workspace),
            serde_json::to_string_pretty(&profile).unwrap(),
        )
        .unwrap();

        let plan = build_fixture_plan(&workspace).unwrap();
        assert_eq!(
            plan.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(),
            vec!["analyze", "code-fix-add", "verify-fix-add"]
        );

        let task_request =
            build_task_request(&workspace, "Fix the workspace", "session-1").unwrap();
        assert_eq!(task_request.input["execution_profile"], json!("fixture-profile"));
        assert_eq!(task_request.input["flow"], json!("workspace_execution"));

        let runtime = build_fixture_runtime(&workspace).unwrap();
        assert!(runtime.agents.get("analyzer").is_some());
        assert!(runtime.agents.get("coder").is_some());
        assert!(runtime.tools.get("tester").is_some());

        let analysis = analyze_workspace_fixture(&workspace, &profile, request(json!({}), 1));
        assert_eq!(analysis.status, crate::domain::step::ExecutionStatus::Succeeded);
        assert_eq!(
            analysis.output.as_ref().unwrap()["analysis_targets"].as_array().unwrap().len(),
            2
        );

        let invalid_attempt = verify_workspace_fixture(
            &workspace,
            &profile,
            request(json!({"attempt_index": 99}), 1),
        );
        assert_eq!(invalid_attempt.status, crate::domain::step::ExecutionStatus::Failed);
        assert_eq!(invalid_attempt.recoverability, Recoverability::Terminal);

        let command_output = run_fixture_command(
            &workspace,
            &super::FixtureCommand { program: "true".to_string(), args: vec![] },
        )
        .unwrap();
        assert!(command_output.succeeded());
        assert_eq!(command_output.rendered_command(), "true");
        assert_eq!(command_output.details()["command"], json!("true"));
    }

    #[test]
    fn fixture_analysis_surfaces_bounded_governance_context_for_flow_steps() {
        let workspace = write_execution_workspace(
            "synod-fixture-governance-context",
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right\n}\n",
        );
        let profile = sample_profile(ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        });
        let mut state = Map::new();
        state.insert(
            "latest_governance_stage".to_string(),
            serde_json::to_value(GovernedStageRecord {
                stage_key: "bug-fix:investigate".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: GovernanceLifecycleState::GovernedReady,
                required: false,
                autopilot_enabled: false,
                approval_state: ApprovalState::NotNeeded,
                canon_run_ref: Some("canon-run-3".to_string()),
                governance_attempt_id: "attempt-3".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: Some(".canon/runs/canon-run-3".to_string()),
                decision_ref: None,
                blocked_reason: None,
            })
            .unwrap(),
        );
        state.insert(
            "latest_governance_packet".to_string(),
            serde_json::to_value(GovernedStagePacket {
                packet_ref: ".canon/runs/canon-run-3".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                canon_mode: Some(CanonMode::Discovery),
                expected_document_refs: vec![".canon/runs/canon-run-3/discovery.md".to_string()],
                document_refs: vec![".canon/runs/canon-run-3/discovery.md".to_string()],
                readiness: PacketReadiness::Reusable,
                missing_sections: Vec::new(),
                headline: "investigation packet ready".to_string(),
            })
            .unwrap(),
        );
        let input = attach_stage_metadata(
            json!({
                "phase": "implement",
            }),
            built_in_flow("bug-fix").unwrap(),
            1,
        )
        .unwrap();

        let result =
            analyze_workspace_fixture(&workspace, &profile, request_with_state(input, 1, state));
        let governance_context = &result.output.as_ref().unwrap()["governance_context"];

        assert_eq!(
            governance_context["reused_packets"][0]["stage_key"],
            json!("bug-fix:investigate")
        );
        assert_eq!(
            governance_context["reused_packets"][0]["packet_ref"],
            json!(".canon/runs/canon-run-3")
        );
        assert_eq!(
            governance_context["reuse_binding"]["binding_reason"],
            json!("upstream_stage_context")
        );
    }

    #[test]
    fn execution_profile_loader_prefers_the_new_manifest_when_present() {
        let workspace = temp_workspace();
        fs::write(
            workspace.join(".synod/execution.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "preferred-profile",
                "read_targets": ["src/lib.rs"],
                "validation_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "attempts": [
                    {
                        "attempt_id": "fix-add",
                        "summary": "Apply the code fix",
                        "failure_mode": "terminal",
                        "changes": [
                            {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                        ]
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            workspace.join(".synod/fixture.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "legacy-profile",
                "test_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "file_patches": [
                    {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let profile = load_workspace_execution_profile(&workspace).unwrap();

        assert_eq!(profile.name, "preferred-profile");
        assert_eq!(profile.legacy_source, None);
        assert_eq!(profile.attempts.len(), 1);
    }

    #[test]
    fn execution_profile_loader_falls_back_to_the_legacy_fixture_manifest() {
        let workspace = temp_workspace();
        fs::write(
            workspace.join(".synod/fixture.json"),
            serde_json::to_vec_pretty(&json!({
                "name": "legacy-profile",
                "test_command": {"program": "cargo", "args": ["test", "--quiet"]},
                "file_patches": [
                    {"path": "src/lib.rs", "find": "left - right", "replace": "left + right"}
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let profile = load_workspace_execution_profile(&workspace).unwrap();

        assert_eq!(profile.name, "legacy-profile");
        assert_eq!(profile.legacy_source.as_deref(), Some(".synod/fixture.json"));
        assert_eq!(profile.read_targets, vec!["src/lib.rs".to_string()]);
        assert_eq!(profile.attempts.len(), 1);
        assert_eq!(profile.attempts[0].attempt_id, "legacy-attempt-1");
    }
}
