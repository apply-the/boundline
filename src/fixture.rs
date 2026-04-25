use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

use crate::adapters::agent::FnAgentAdapter;
use crate::adapters::tool::FnToolAdapter;
use crate::domain::flow::{SessionFlowState, attach_stage_metadata, built_in_flow};
use crate::domain::limits::RunLimits;
use crate::domain::plan::Plan;
use crate::domain::step::{
    ErrorInfo, Recoverability, Step, StepError, StepExecutionRequest, StepExecutionResult,
};
use crate::domain::task::TaskRunRequest;
use crate::orchestrator::planner::StaticPlanner;
use crate::registry::agent_registry::{AgentRegistry, RegistryError as AgentRegistryError};
use crate::registry::tool_registry::{RegistryError as ToolRegistryError, ToolRegistry};

const FIXTURE_RELATIVE_PATH: &str = ".synod/fixture.json";

#[derive(Clone)]
pub struct FixtureRuntime {
    pub fixture: WorkspaceFixture,
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
    let fixture = load_workspace_fixture(workspace)?;
    build_vertical_slice_plan(&fixture, active_flow)
}

pub fn build_task_request(
    workspace: &Path,
    goal: impl Into<String>,
    session_id: impl Into<String>,
) -> Result<TaskRunRequest, FixtureRuntimeError> {
    let fixture = load_workspace_fixture(workspace)?;

    Ok(TaskRunRequest {
        goal: goal.into(),
        input: json!({
            "fixture": fixture.name,
            "flow": "bug_fix_vertical_slice",
        }),
        session_id: session_id.into(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        limits: fixture.limits,
        initial_context: None,
    })
}

pub fn build_fixture_runtime(workspace: &Path) -> Result<FixtureRuntime, FixtureRuntimeError> {
    let fixture = load_workspace_fixture(workspace)?;
    let planner = StaticPlanner::new(build_vertical_slice_plan(&fixture, None)?);
    let workspace_ref = workspace.to_path_buf();

    let mut agents = AgentRegistry::new();
    agents.register("analyzer", {
        let workspace_ref = workspace_ref.clone();
        let fixture = fixture.clone();
        FnAgentAdapter::new(move |request| {
            analyze_workspace_fixture(&workspace_ref, &fixture, request)
        })
    })?;
    agents.register("coder", {
        let workspace_ref = workspace_ref.clone();
        let fixture = fixture.clone();
        FnAgentAdapter::new(move |request| {
            apply_workspace_fixture(&workspace_ref, &fixture, request)
        })
    })?;

    let mut tools = ToolRegistry::new();
    tools.register("tester", {
        let workspace_ref = workspace_ref.clone();
        let fixture = fixture.clone();
        FnToolAdapter::new(move |request| {
            verify_workspace_fixture(&workspace_ref, &fixture, request)
        })
    })?;

    Ok(FixtureRuntime { fixture, planner, agents, tools })
}

fn build_vertical_slice_plan(
    _fixture: &WorkspaceFixture,
    active_flow: Option<&SessionFlowState>,
) -> Result<Plan, FixtureRuntimeError> {
    let Some(active_flow) = active_flow else {
        let steps = vec![
            Step::agent("analyze", "analyzer", json!({"phase": "analyze"}))?,
            Step::agent("code", "coder", json!({"phase": "code"}))?,
            Step::tool("verify", "tester", json!({"phase": "verify"}))?,
        ];
        return Ok(Plan::new(steps)?);
    };

    let flow = built_in_flow(&active_flow.flow_name)
        .expect("validated flow name should resolve for fixture planning");

    let steps = match flow.name {
        "bug-fix" => vec![
            Step::agent(
                "investigate",
                "analyzer",
                attach_stage_metadata(json!({"phase": "investigate"}), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(
                    json!({
                        "phase": "implement",
                        "force_retry_once": _fixture.limits.max_retries > 0,
                    }),
                    flow,
                    1,
                )?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(json!({"phase": "verify"}), flow, 2)?,
            )?,
        ],
        "change" => vec![
            Step::agent(
                "understand-change",
                "analyzer",
                attach_stage_metadata(json!({"phase": "understand-change"}), flow, 0)?,
            )?,
            Step::agent(
                "implement",
                "coder",
                attach_stage_metadata(json!({"phase": "implement"}), flow, 1)?,
            )?,
            Step::tool(
                "verify",
                "tester",
                attach_stage_metadata(json!({"phase": "verify"}), flow, 2)?,
            )?,
        ],
        "delivery" => vec![
            Step::agent(
                "requirements",
                "analyzer",
                attach_stage_metadata(json!({"phase": "requirements"}), flow, 0)?,
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
                attach_stage_metadata(json!({"phase": "implementation"}), flow, 3)?,
            )?,
            Step::tool(
                "implementation-verify",
                "tester",
                attach_stage_metadata(json!({"phase": "implementation"}), flow, 3)?,
            )?,
        ],
        _ => unreachable!("unsupported built-in flow should have been rejected earlier"),
    };

    Ok(Plan::new(steps)?)
}

fn analyze_workspace_fixture(
    workspace: &Path,
    fixture: &WorkspaceFixture,
    _request: StepExecutionRequest,
) -> StepExecutionResult {
    match run_fixture_command(workspace, &fixture.test_command) {
        Ok(output) if output.succeeded() => StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_not_red",
                format!(
                    "workspace fixture '{}' is already passing before implementation",
                    fixture.name
                ),
            )
            .with_details(output.details()),
            Recoverability::Terminal,
        ),
        Ok(output) => StepExecutionResult::success(json!({
            "fixture": fixture.name,
            "red_phase_confirmed": true,
            "test_command": output.rendered_command(),
            "initial_test_exit_code": output.exit_code,
            "stdout": output.stdout,
            "stderr": output.stderr,
        })),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_analysis_failed",
                format!("failed to execute the fixture test command: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn apply_workspace_fixture(
    workspace: &Path,
    fixture: &WorkspaceFixture,
    request: StepExecutionRequest,
) -> StepExecutionResult {
    if request.input.get("force_retry_once").and_then(Value::as_bool).unwrap_or(false)
        && request.attempt_number == 1
    {
        return StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_retry_once",
                format!(
                    "workspace fixture '{}' intentionally requests one retry before applying patches",
                    fixture.name
                ),
            ),
            Recoverability::Retryable,
        );
    }

    match apply_fixture_patches(workspace, fixture) {
        Ok(report) => StepExecutionResult::success(json!({
            "fixture": fixture.name,
            "patch_applied": true,
            "updated_files": report.updated_files,
            "already_applied_files": report.already_applied_files,
        })),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_patch_failed",
                format!("failed to apply the fixture patch set: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn verify_workspace_fixture(
    workspace: &Path,
    fixture: &WorkspaceFixture,
    _request: StepExecutionRequest,
) -> StepExecutionResult {
    match run_fixture_command(workspace, &fixture.test_command) {
        Ok(output) if output.succeeded() => StepExecutionResult::success(json!({
            "fixture": fixture.name,
            "tests_passed": true,
            "goal_satisfied": true,
            "test_command": output.rendered_command(),
            "final_test_exit_code": output.exit_code,
            "stdout": output.stdout,
            "stderr": output.stderr,
        })),
        Ok(output) => StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_tests_still_failing",
                format!(
                    "workspace fixture '{}' still fails verification after the patch",
                    fixture.name
                ),
            )
            .with_details(output.details()),
            Recoverability::Terminal,
        ),
        Err(error) => StepExecutionResult::failure(
            ErrorInfo::new(
                "fixture_verify_failed",
                format!("failed to execute the verification command: {error}"),
            ),
            Recoverability::Terminal,
        ),
    }
}

fn apply_fixture_patches(
    workspace: &Path,
    fixture: &WorkspaceFixture,
) -> Result<PatchReport, FixtureRuntimeError> {
    let mut updated_files = Vec::new();
    let mut already_applied_files = Vec::new();

    for patch in &fixture.file_patches {
        let path = workspace.join(&patch.path);
        let contents = fs::read_to_string(&path)
            .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;

        if contents.contains(&patch.find) {
            let updated = contents.replacen(&patch.find, &patch.replace, 1);
            fs::write(&path, updated)
                .map_err(|source| FixtureRuntimeError::Io { path: path.clone(), source })?;
            updated_files.push(patch.path.clone());
            continue;
        }

        if contents.contains(&patch.replace) {
            already_applied_files.push(patch.path.clone());
            continue;
        }

        return Err(FixtureRuntimeError::PatchTargetMissing { path, needle: patch.find.clone() });
    }

    Ok(PatchReport { updated_files, already_applied_files })
}

fn run_fixture_command(
    workspace: &Path,
    command: &FixtureCommand,
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

fn render_command(command: &FixtureCommand) -> String {
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

struct PatchReport {
    updated_files: Vec<String>,
    already_applied_files: Vec<String>,
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
    #[error("workspace fixture is missing at {0}")]
    MissingFixture(PathBuf),
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

    use uuid::Uuid;

    use super::{
        FilePatch, FixtureRuntimeError, WorkspaceFixture, apply_fixture_patches,
        load_workspace_fixture,
    };

    fn temp_workspace() -> std::path::PathBuf {
        let workspace = std::env::temp_dir().join(format!("synod-fixture-unit-{}", Uuid::new_v4()));
        fs::create_dir_all(workspace.join(".synod")).unwrap();
        workspace
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
}
