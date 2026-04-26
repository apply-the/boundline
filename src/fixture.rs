use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use thiserror::Error;

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::tool::FnToolAdapter;
use crate::domain::execution::{
    ChangeEvidence, ChangeStatus, ExecutionAttemptDefinition, ExecutionCommand,
    ExecutionFailureMode, ExecutionProfileError, ValidationRecord, WorkspaceChange,
    WorkspaceExecutionProfile,
};
use crate::domain::flow::{
    FLOW_METADATA_KEY, SessionFlowState, attach_stage_metadata, built_in_flow,
};
use crate::domain::limits::RunLimits;
use crate::domain::plan::Plan;
use crate::domain::step::{
    ErrorInfo, Recoverability, Step, StepError, StepExecutionRequest, StepExecutionResult,
};
use crate::domain::task::TaskRunRequest;
use crate::orchestrator::planner::StaticPlanner;
use crate::registry::agent_registry::{AgentRegistry, RegistryError as AgentRegistryError};
use crate::registry::tool_registry::{RegistryError as ToolRegistryError, ToolRegistry};

const EXECUTION_RELATIVE_PATH: &str = ".synod/execution.json";
const FIXTURE_RELATIVE_PATH: &str = ".synod/fixture.json";

#[derive(Clone)]
pub struct FixtureRuntime {
    pub profile: WorkspaceExecutionProfile,
    pub planner: StaticPlanner,
    pub agents: AgentRegistry,
    pub tools: ToolRegistry,
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
    build_fixture_plan_for_flow(workspace, None)
}

pub fn build_fixture_plan_for_flow(
    workspace: &Path,
    active_flow: Option<&SessionFlowState>,
) -> Result<Plan, FixtureRuntimeError> {
    let profile = load_workspace_execution_profile(workspace)?;
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
    let planner = StaticPlanner::with_replans(
        build_vertical_slice_plan(&profile, active_flow, 0)?,
        build_replan_queue(&profile, active_flow)?,
    );
    let workspace_ref = workspace.to_path_buf();

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

    let mut tools = ToolRegistry::new();
    tools.register("tester", {
        let workspace_ref = workspace_ref.clone();
        let profile = profile.clone();
        FnToolAdapter::new(move |request| {
            verify_workspace_fixture(&workspace_ref, &profile, request)
        })
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

    let steps = match flow.name {
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
        ]);
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
    ])
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
        limits: fixture.limits,
        legacy_source: Some(FIXTURE_RELATIVE_PATH.to_string()),
    };
    profile.validate()?;
    Ok(profile)
}

fn analyze_workspace_fixture(
    workspace: &Path,
    profile: &WorkspaceExecutionProfile,
    _request: StepExecutionRequest,
) -> StepExecutionResult {
    match snapshot_workspace_targets(workspace, &read_targets_for_profile(profile)) {
        Ok(snapshots) => StepExecutionResult::success(json!({
            "execution_profile": profile.name,
            "analysis_targets": snapshots,
            "legacy_source": profile.legacy_source,
        })),
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

    let attempt_index =
        request.input.get("attempt_index").and_then(Value::as_u64).unwrap_or(0) as usize;

    let attempt = match execution_attempt(profile, attempt_index) {
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

    match apply_execution_attempt(workspace, attempt) {
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

            StepExecutionResult::success_with_patch(
                json!({
                    "execution_profile": profile.name,
                    "attempt_id": attempt.attempt_id,
                    "change_applied": true,
                    "changed_files": changed_files,
                    "already_applied_files": report.already_applied_files,
                    "change_evidence": report.change_evidence,
                }),
                state_patch,
            )
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
    let attempt_index =
        request.input.get("attempt_index").and_then(Value::as_u64).unwrap_or(0) as usize;
    let attempt = match execution_attempt(profile, attempt_index) {
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
            StepExecutionResult::success_with_patch(
                json!({
                    "execution_profile": profile.name,
                    "attempt_id": attempt.attempt_id,
                    "validation": record,
                    "goal_satisfied": true,
                }),
                state_patch,
            )
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
        run_fixture_command, verify_workspace_fixture,
    };
    use crate::domain::flow::built_in_flow;
    use crate::domain::limits::RunLimits;
    use crate::domain::step::{Recoverability, StepExecutionRequest, StepKind};
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
            limits: RunLimits::default(),
            legacy_source: None,
        }
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
