use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde_json::{Value, json};
use synod::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use synod::cli::inspect::{TraceSummaryError, summarize_trace};
use synod::cli::session::{
    SessionCommandError, execute_capture, execute_flow, execute_next, execute_plan, execute_start,
    execute_status, execute_step, render_error,
};
use synod::cli::{
    Cli, CliValidationError, CommandExitStatus, CommandName, DeveloperCommand,
    DeveloperCommandSession,
};
use synod::domain::execution::{
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, ExecutionProfileError,
    WorkspaceChange, WorkspaceExecutionProfile,
};
use synod::domain::flow::{
    FlowStepMetadata, FlowValidationError, SessionFlowState, attach_stage_metadata, built_in_flow,
    supported_flow_names_csv,
};
use synod::domain::limits::{RunLimits, TerminalCondition};
use synod::domain::plan::{Plan, PlanError, PlanStatus};
use synod::domain::session::{
    ActiveSessionRecord, SessionCommand, SessionStatus, SessionStatusView, SessionTransition,
    SessionValidationError,
};
use synod::domain::step::Step;
use synod::domain::task::{
    Task, TaskPersistenceError, TaskRequestError, TaskRunRequest, TaskStatus, TerminalReason,
};
use synod::domain::task_context::TaskContextError;
use synod::domain::trace::{ExecutionTrace, TraceEventType};
use synod::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};
use uuid::Uuid;

const FIXTURE_CARGO_TOML: &str = r#"[package]
name = "runtime_fixture"
version = "0.1.0"
edition = "2024"
"#;

const RED_LIB_RS: &str = "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n";

const FIXTURE_TEST_RS: &str = r#"#[test]
fn red_to_green_addition() {
    assert_eq!(runtime_fixture::add(2, 2), 4);
}
"#;

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();
    workspace
}

fn build_request(workspace_ref: &str) -> TaskRunRequest {
    TaskRunRequest {
        goal: "Deliver a bounded change".to_string(),
        input: json!({"ticket": "COV-1"}),
        session_id: "session-coverage".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    }
}

fn build_task(workspace_ref: &str) -> Task {
    let plan =
        Plan::new(vec![Step::decision("analyze", json!({"phase": "bootstrap"})).unwrap()]).unwrap();
    Task::new("task-coverage", &build_request(workspace_ref), plan).unwrap()
}

fn build_planned_record(workspace_ref: &str) -> ActiveSessionRecord {
    ActiveSessionRecord {
        session_id: "session-coverage".to_string(),
        workspace_ref: workspace_ref.to_string(),
        goal: Some("Deliver a bounded change".to_string()),
        authored_brief: None,
        active_flow: None,
        active_task: Some(build_task(workspace_ref)),
        goal_plan: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(format!("{workspace_ref}/.synod/traces/task.json")),
        created_at: 10,
        updated_at: 20,
    }
}

fn build_status_view(record: &ActiveSessionRecord) -> SessionStatusView {
    let active_task = record.active_task.as_ref();
    SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        authored_input_summary: None,
        authored_input_sources: None,
        authored_input_deduplicated_sources: None,
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: None,
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
        current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
        current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
        total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
        plan_revision: active_task.map(|task| task.plan.revision),
        current_step_id: active_task
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
        current_step_index: active_task.map(|task| task.plan.current_step_index),
        latest_status: record.latest_status,
        execution_path: synod::domain::session::execution_path_text(record),
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_changed_files: active_task.and_then(|task| {
            task.context.state.get("latest_changed_files").and_then(|value| {
                value.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
            })
        }),
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_attempt_lineage: None,
        latest_validation_status: active_task.and_then(|task| {
            task.context
                .state
                .get("latest_validation_status")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_trigger: None,
        latest_review_vote: None,
        latest_review_outcome: None,
        latest_review_headline: None,
        latest_governance_stage: None,
        latest_governance_runtime: None,
        latest_governance_mode: None,
        latest_governance_run_ref: None,
        latest_governance_state: None,
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: None,
        latest_governance_packet_source_stage: None,
        latest_governance_packet_binding_reason: None,
        latest_governance_approval: None,
        latest_governance_decision: None,
        latest_governance_candidates: None,
        governance_next_action: None,
        next_command: Some("synod step".to_string()),
        explanation: "session state is internally consistent".to_string(),
    }
}

fn build_started_session(workspace: &Path) -> ActiveSessionRecord {
    let now = 10;
    ActiveSessionRecord {
        session_id: Uuid::new_v4().to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: now,
        updated_at: now,
    }
}

fn build_goal_captured_session(workspace: &Path) -> ActiveSessionRecord {
    let mut session = build_started_session(workspace);
    session.goal = Some("Fix the failing add test".to_string());
    session.latest_status = SessionStatus::GoalCaptured;
    session
}

fn write_execution_workspace(prefix: &str, attempts: Vec<Value>) -> PathBuf {
    let workspace = temp_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".synod")).unwrap();
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".synod/execution.json"),
        serde_json::to_string_pretty(&json!({
            "name": "coverage-execution",
            "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
            "validation_command": {
                "program": "cargo",
                "args": ["test", "--quiet"]
            },
            "attempts": attempts,
        }))
        .unwrap(),
    )
    .unwrap();
    workspace
}

fn success_attempt() -> Value {
    json!({
        "attempt_id": "fix-add",
        "summary": "Replace subtraction with addition",
        "failure_mode": "terminal",
        "changes": [
            {
                "path": "src/lib.rs",
                "find": "left - right",
                "replace": "left + right"
            }
        ]
    })
}

fn failing_attempt() -> Value {
    json!({
        "attempt_id": "broken-fix",
        "summary": "Try a missing patch",
        "failure_mode": "terminal",
        "changes": [
            {
                "path": "src/lib.rs",
                "find": "left * right",
                "replace": "left + right"
            }
        ]
    })
}

fn replan_attempts() -> Vec<Value> {
    vec![
        json!({
            "attempt_id": "bad-fix",
            "summary": "Introduce a failing division change",
            "failure_mode": "replan",
            "changes": [
                {
                    "path": "src/lib.rs",
                    "find": "left - right",
                    "replace": "left / right"
                }
            ]
        }),
        json!({
            "attempt_id": "good-fix",
            "summary": "Replace division with addition",
            "failure_mode": "terminal",
            "changes": [
                {
                    "path": "src/lib.rs",
                    "find": "left / right",
                    "replace": "left + right"
                }
            ]
        }),
    ]
}

#[test]
fn developer_command_sessions_cover_variant_mapping_validation_and_completion() {
    let workspace = PathBuf::from("/tmp/synod-cli");
    let trace = PathBuf::from("/tmp/synod-cli/trace.json");
    let commands = vec![
        DeveloperCommand::Doctor { workspace: workspace.clone() },
        DeveloperCommand::Start { workspace: Some(workspace.clone()) },
        DeveloperCommand::Capture {
            workspace: Some(workspace.clone()),
            goal: Some("capture goal".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
        },
        DeveloperCommand::Flow { name: "bug-fix".to_string(), workspace: Some(workspace.clone()) },
        DeveloperCommand::Plan { workspace: Some(workspace.clone()), flow: None, no_flow: false },
        DeveloperCommand::Step { workspace: Some(workspace.clone()) },
        DeveloperCommand::Run {
            workspace: Some(workspace.clone()),
            goal: Some("ship it".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
        },
        DeveloperCommand::Inspect {
            trace: Some(trace.clone()),
            workspace: Some(workspace.clone()),
        },
        DeveloperCommand::Status { workspace: Some(workspace.clone()) },
        DeveloperCommand::Next { workspace: Some(workspace.clone()) },
    ];

    for command in &commands {
        let session = DeveloperCommandSession::from_command(command);
        assert_eq!(session.command_name.as_str(), command.name().as_str());
    }

    assert_eq!(CommandName::Doctor.to_string(), "doctor");
    let cli = Cli::try_parse_from(["synod", "inspect", "--workspace", "."]).unwrap();
    assert!(matches!(cli.command, DeveloperCommand::Inspect { .. }));

    let invalid_doctor = DeveloperCommandSession {
        command_name: CommandName::Doctor,
        workspace_ref: Some(" ".to_string()),
        goal: None,
        trace_ref: None,
        started_at: 1,
        completed_at: None,
        exit_status: None,
        trace_location: None,
    };
    assert_eq!(
        invalid_doctor.validate().unwrap_err(),
        CliValidationError::MissingWorkspaceRef(CommandName::Doctor)
    );

    let invalid_capture = DeveloperCommandSession::from_command(&DeveloperCommand::Capture {
        workspace: Some(workspace.clone()),
        goal: Some("  ".to_string()),
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    });
    assert_eq!(
        invalid_capture.validate().unwrap_err(),
        CliValidationError::MissingGoal(CommandName::Capture)
    );

    let invalid_flow = DeveloperCommandSession::from_command(&DeveloperCommand::Flow {
        name: " ".to_string(),
        workspace: Some(workspace.clone()),
    });
    assert_eq!(invalid_flow.validate().unwrap_err(), CliValidationError::MissingFlowName);

    let invalid_run_workspace = DeveloperCommandSession::from_command(&DeveloperCommand::Run {
        workspace: None,
        goal: Some("ship".to_string()),
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    });
    assert_eq!(
        invalid_run_workspace.validate().unwrap_err(),
        CliValidationError::MissingWorkspaceRef(CommandName::Run)
    );

    let invalid_run_goal = DeveloperCommandSession::from_command(&DeveloperCommand::Run {
        workspace: Some(workspace.clone()),
        goal: Some(" ".to_string()),
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
    });
    assert_eq!(
        invalid_run_goal.validate().unwrap_err(),
        CliValidationError::MissingGoal(CommandName::Run)
    );

    let invalid_inspect = DeveloperCommandSession::from_command(&DeveloperCommand::Inspect {
        trace: None,
        workspace: None,
    });
    assert_eq!(invalid_inspect.validate().unwrap_err(), CliValidationError::MissingTraceSelection);

    let mut completed = DeveloperCommandSession::from_command(&DeveloperCommand::Status {
        workspace: Some(workspace),
    });
    let exit_code = completed
        .complete(CommandExitStatus::NonSuccess, Some(trace.to_string_lossy().into_owned()));
    assert_eq!(exit_code.code(), 1);
    assert_eq!(completed.exit_status, Some(CommandExitStatus::NonSuccess));
    assert!(completed.completed_at.is_some());
}

#[test]
fn flow_and_execution_validation_cover_remaining_error_paths() {
    let flow = built_in_flow("delivery").unwrap();
    assert_eq!(supported_flow_names_csv(), "bug-fix, change, delivery");

    assert!(matches!(
        attach_stage_metadata(json!("not-an-object"), flow, 0).unwrap_err(),
        FlowValidationError::NonObjectStepInput { .. }
    ));
    assert!(matches!(
        attach_stage_metadata(json!({}), flow, 99).unwrap_err(),
        FlowValidationError::InvalidStageIndex { .. }
    ));
    assert_eq!(FlowStepMetadata::from_value(&Value::Null).unwrap(), None);
    assert_eq!(
        FlowStepMetadata::from_value(&json!({})).unwrap_err(),
        FlowValidationError::MissingMetadataField("flow_name")
    );

    let mut unknown_flow = SessionFlowState {
        flow_name: "missing".to_string(),
        current_stage_id: "unknown".to_string(),
        current_stage_index: 0,
        total_stages: 1,
    };
    assert!(matches!(unknown_flow.advance().unwrap_err(), FlowValidationError::UnknownFlow(_)));

    assert_eq!(
        ExecutionCommand { program: " ".to_string(), args: vec![] }.validate().unwrap_err(),
        ExecutionProfileError::MissingValidationProgram
    );
    assert_eq!(
        WorkspaceChange {
            path: " ".to_string(),
            find: "red".to_string(),
            replace: "green".to_string()
        }
        .validate()
        .unwrap_err(),
        ExecutionProfileError::MissingChangePath
    );
    assert_eq!(
        WorkspaceChange {
            path: "/tmp/outside.rs".to_string(),
            find: "red".to_string(),
            replace: "green".to_string(),
        }
        .validate()
        .unwrap_err(),
        ExecutionProfileError::InvalidWorkspacePath("/tmp/outside.rs".to_string())
    );
    assert_eq!(
        WorkspaceChange {
            path: "src/lib.rs".to_string(),
            find: "".to_string(),
            replace: "green".to_string()
        }
        .validate()
        .unwrap_err(),
        ExecutionProfileError::MissingFindPattern("src/lib.rs".to_string())
    );
    assert_eq!(
        ExecutionAttemptDefinition {
            attempt_id: " ".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Retry,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "red".to_string(),
                replace: "green".to_string(),
            }],
        }
        .validate()
        .unwrap_err(),
        ExecutionProfileError::MissingAttemptId
    );
    assert_eq!(
        ExecutionAttemptDefinition {
            attempt_id: "retry-1".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Retry,
            changes: vec![],
        }
        .validate()
        .unwrap_err(),
        ExecutionProfileError::MissingAttemptChanges("retry-1".to_string())
    );

    let mut profile = WorkspaceExecutionProfile {
        name: " ".to_string(),
        read_targets: vec!["src/lib.rs".to_string()],
        validation_command: ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string()],
        },
        attempts: vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
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
    };
    assert_eq!(profile.validate().unwrap_err(), ExecutionProfileError::MissingProfileName);

    profile.name = "profile".to_string();
    profile.attempts = Vec::new();
    assert_eq!(profile.validate().unwrap_err(), ExecutionProfileError::MissingAttempts);

    profile.attempts = vec![ExecutionAttemptDefinition {
        attempt_id: "fix-add".to_string(),
        summary: String::new(),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: vec![WorkspaceChange {
            path: "src/lib.rs".to_string(),
            find: "left - right".to_string(),
            replace: "left + right".to_string(),
        }],
    }];
    profile.limits = RunLimits { max_steps: 0, ..RunLimits::default() };
    assert!(matches!(profile.validate().unwrap_err(), ExecutionProfileError::InvalidRunLimits(_)));

    let stage_count_mismatch = SessionFlowState {
        flow_name: "bug-fix".to_string(),
        current_stage_id: "investigate".to_string(),
        current_stage_index: 0,
        total_stages: 99,
    };
    assert!(matches!(
        stage_count_mismatch.validate().unwrap_err(),
        FlowValidationError::StageCountMismatch { .. }
    ));

    let invalid_stage_index = SessionFlowState {
        flow_name: "bug-fix".to_string(),
        current_stage_id: "investigate".to_string(),
        current_stage_index: 99,
        total_stages: 3,
    };
    assert!(matches!(
        invalid_stage_index.validate().unwrap_err(),
        FlowValidationError::InvalidStageIndex { .. }
    ));
}

#[test]
fn task_plan_and_session_store_validation_cover_error_branches() {
    let workspace = "/tmp/synod-task-coverage";

    let mut unexpected_terminal = build_task(workspace);
    unexpected_terminal.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    assert_eq!(
        unexpected_terminal.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::UnexpectedTerminalReason(TaskStatus::Planned)
    );

    let mut missing_terminal_reason = build_task(workspace);
    missing_terminal_reason.status = TaskStatus::Succeeded;
    assert_eq!(
        missing_terminal_reason.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::MissingTerminalReason(TaskStatus::Succeeded)
    );

    let mut invalid_counters = build_task(workspace);
    invalid_counters.retry_count = 1;
    invalid_counters.total_step_attempts = 0;
    assert!(matches!(
        invalid_counters.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::InvalidAttemptCounters { .. }
    ));

    let mut missing_task_id = build_task(workspace);
    missing_task_id.id = " ".to_string();
    assert_eq!(
        missing_task_id.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::MissingTaskId
    );

    let mut missing_goal = build_task(workspace);
    missing_goal.goal = " ".to_string();
    assert_eq!(
        missing_goal.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::MissingGoal
    );

    let mut invalid_context = build_task(workspace);
    invalid_context.context.session_id = " ".to_string();
    assert!(matches!(
        invalid_context.validate_persisted_state().unwrap_err(),
        TaskPersistenceError::InvalidContext(_)
    ));

    let mut plan = Plan::new(vec![Step::decision("only", json!({})).unwrap()]).unwrap();
    assert_eq!(plan.replace_remaining_steps(Vec::new()).unwrap_err(), PlanError::NoExecutableSteps);

    let mut invalid_plan = Plan::new(vec![Step::decision("only", json!({})).unwrap()]).unwrap();
    invalid_plan.current_step_index = 2;
    assert!(matches!(
        invalid_plan.validate().unwrap_err(),
        PlanError::InvalidCurrentStepIndex { .. }
    ));

    let mut reset_plan = Plan::new(vec![Step::decision("only", json!({})).unwrap()]).unwrap();
    reset_plan.advance();
    assert_eq!(reset_plan.status, PlanStatus::Completed);
    reset_plan.reset_execution_position();
    assert_eq!(reset_plan.current_step_index, 0);
    assert_eq!(reset_plan.status, PlanStatus::Active);

    let workspace = temp_workspace("synod-corrupt-session-store");
    let store = FileSessionStore::for_workspace(&workspace);
    assert!(store.path().ends_with(Path::new(".synod/session.json")));
    fs::create_dir_all(store.path().parent().unwrap()).unwrap();
    fs::write(store.path(), b"{not json").unwrap();
    assert!(matches!(store.load().unwrap_err(), SessionStoreError::Deserialize(_)));
    store.clear().unwrap();
    store.clear().unwrap();

    assert_eq!(
        TaskRequestError::from(PlanError::NoExecutableSteps),
        TaskRequestError::InvalidPlan(
            "a plan must contain at least one executable step".to_string()
        )
    );
    assert_eq!(
        TaskRequestError::from(TaskContextError::MissingSessionId),
        TaskRequestError::InvalidContext("task context session_id must not be empty".to_string())
    );
}

#[test]
fn session_validation_transition_and_status_view_cover_mismatch_paths() {
    let workspace = "/tmp/synod-session-coverage";

    let mut record = build_planned_record(workspace);
    record.created_at = 20;
    record.updated_at = 10;
    assert!(matches!(
        record.validate().unwrap_err(),
        SessionValidationError::UpdatedBeforeCreated { .. }
    ));

    let mut record = build_planned_record(workspace);
    record.latest_trace_ref = Some("/tmp/outside/trace.json".to_string());
    assert!(matches!(
        record.validate().unwrap_err(),
        SessionValidationError::TraceOutsideWorkspace { .. }
    ));

    let mut record = build_planned_record(workspace);
    record.active_task = None;
    assert_eq!(
        record.validate().unwrap_err(),
        SessionValidationError::MissingActiveTask(SessionStatus::Planned)
    );

    let mut record = build_planned_record(workspace);
    record.active_task.as_mut().unwrap().goal = "Different goal".to_string();
    assert!(matches!(
        record.validate().unwrap_err(),
        SessionValidationError::TaskGoalMismatch { .. }
    ));

    let mut record = build_planned_record(workspace);
    record.active_task.as_mut().unwrap().status = TaskStatus::Running;
    assert!(matches!(
        record.validate().unwrap_err(),
        SessionValidationError::TaskStatusMismatch { .. }
    ));

    let record = build_planned_record(workspace);
    let missing_reason = SessionTransition {
        trigger_command: SessionCommand::Plan,
        from_status: Some(SessionStatus::GoalCaptured),
        to_status: SessionStatus::Planned,
        trace_ref: record.latest_trace_ref.clone(),
        reason: " ".to_string(),
    };
    assert_eq!(
        missing_reason.validate(&record).unwrap_err(),
        SessionValidationError::MissingTransitionReason
    );

    let wrong_status = SessionTransition {
        trigger_command: SessionCommand::Plan,
        from_status: Some(SessionStatus::GoalCaptured),
        to_status: SessionStatus::Running,
        trace_ref: record.latest_trace_ref.clone(),
        reason: "planned".to_string(),
    };
    assert!(matches!(
        wrong_status.validate(&record).unwrap_err(),
        SessionValidationError::TransitionStatusMismatch { .. }
    ));

    let wrong_trace = SessionTransition {
        trigger_command: SessionCommand::Plan,
        from_status: Some(SessionStatus::GoalCaptured),
        to_status: SessionStatus::Planned,
        trace_ref: None,
        reason: "planned".to_string(),
    };
    assert!(matches!(
        wrong_trace.validate(&record).unwrap_err(),
        SessionValidationError::TransitionTraceMismatch { .. }
    ));

    let mut record = build_planned_record(workspace);
    record
        .active_task
        .as_mut()
        .unwrap()
        .context
        .state
        .insert("latest_changed_files".to_string(), json!(["src/lib.rs"]));
    record
        .active_task
        .as_mut()
        .unwrap()
        .context
        .state
        .insert("latest_validation_status".to_string(), json!("passed"));

    let mut wrong_changed_files = build_status_view(&record);
    wrong_changed_files.latest_changed_files = Some(vec!["src/main.rs".to_string()]);
    assert!(matches!(
        wrong_changed_files.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewChangedFilesMismatch { .. }
    ));

    let mut wrong_validation = build_status_view(&record);
    wrong_validation.latest_validation_status = Some("failed".to_string());
    assert!(matches!(
        wrong_validation.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewValidationStatusMismatch { .. }
    ));

    let mut blank_explanation = build_status_view(&record);
    blank_explanation.explanation = " ".to_string();
    assert_eq!(
        blank_explanation.validate(&record).unwrap_err(),
        SessionValidationError::MissingStatusExplanation
    );

    let mut blank_next_command = build_status_view(&record);
    blank_next_command.next_command = Some(" ".to_string());
    assert_eq!(
        blank_next_command.validate(&record).unwrap_err(),
        SessionValidationError::MissingNextCommand
    );

    let mut wrong_session = build_status_view(&record);
    wrong_session.session_id = "other-session".to_string();
    assert!(matches!(
        wrong_session.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewSessionMismatch { .. }
    ));

    let mut wrong_workspace = build_status_view(&record);
    wrong_workspace.workspace_ref = "/tmp/other-workspace".to_string();
    assert!(matches!(
        wrong_workspace.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewWorkspaceMismatch { .. }
    ));

    let mut wrong_status = build_status_view(&record);
    wrong_status.latest_status = SessionStatus::Running;
    assert!(matches!(
        wrong_status.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewStatusMismatch { .. }
    ));

    let mut wrong_goal = build_status_view(&record);
    wrong_goal.goal = Some("different goal".to_string());
    assert!(matches!(
        wrong_goal.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewGoalMismatch { .. }
    ));

    let mut flow_record = build_planned_record(workspace);
    flow_record.active_flow = Some(built_in_flow("bug-fix").unwrap().initial_state());
    let mut wrong_flow = build_status_view(&flow_record);
    wrong_flow.active_flow = Some("delivery".to_string());
    assert!(matches!(
        wrong_flow.validate(&flow_record).unwrap_err(),
        SessionValidationError::StatusViewFlowMismatch { .. }
    ));

    let mut wrong_stage = build_status_view(&flow_record);
    wrong_stage.current_stage_id = Some("verify".to_string());
    assert!(matches!(
        wrong_stage.validate(&flow_record).unwrap_err(),
        SessionValidationError::StatusViewStageMismatch { .. }
    ));

    let mut wrong_step_id = build_status_view(&record);
    wrong_step_id.current_step_id = Some("different-step".to_string());
    assert!(matches!(
        wrong_step_id.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewStepIdMismatch { .. }
    ));

    let mut wrong_plan_revision = build_status_view(&record);
    wrong_plan_revision.plan_revision = Some(99);
    assert!(matches!(
        wrong_plan_revision.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewPlanRevisionMismatch { .. }
    ));
}

#[test]
fn inspect_summary_and_session_commands_cover_additional_error_paths() {
    let trace = ExecutionTrace::new("task-1", "session-1", "Summarize me");
    assert!(matches!(
        summarize_trace("/tmp/trace.json", &trace).unwrap_err(),
        TraceSummaryError::MissingTerminalStatus
    ));

    let mut trace = ExecutionTrace::new("task-1", "session-1", "Summarize me");
    trace.terminal_status = Some(TaskStatus::Failed);
    assert!(matches!(
        summarize_trace("/tmp/trace.json", &trace).unwrap_err(),
        TraceSummaryError::MissingTerminalReason
    ));

    let mut trace = ExecutionTrace::new("task-1", "session-1", "Summarize me");
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "broken", None));
    trace.ended_at = Some(trace.started_at + 1);
    trace.record_event(
        TraceEventType::StepCompleted,
        Some("verify".to_string()),
        0,
        json!({"status": "failed"}),
    );
    assert!(matches!(
        summarize_trace("/tmp/trace.json", &trace).unwrap_err(),
        TraceSummaryError::MissingStartedStep(step_id) if step_id == "verify"
    ));

    let workspace = temp_workspace("synod-cli-session-errors");
    execute_start(Some(&workspace)).unwrap();
    assert!(matches!(
        execute_plan(Some(&workspace), None, false).unwrap_err(),
        SessionCommandError::MissingCapturedGoal
    ));

    execute_capture(
        Some(&workspace),
        Some("Fix the failing add test"),
        &[],
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(matches!(
        execute_step(Some(&workspace)).unwrap_err(),
        SessionCommandError::MissingPlannedTask
    ));
    assert!(matches!(
        execute_flow(Some(&workspace), "missing-flow").unwrap_err(),
        SessionCommandError::UnknownFlow { .. }
    ));

    let mismatch_workspace = temp_workspace("synod-cli-session-mismatch");
    let foreign_workspace = temp_workspace("synod-cli-session-foreign");
    let foreign_record = build_planned_record(foreign_workspace.to_string_lossy().as_ref());
    FileSessionStore::for_workspace(&mismatch_workspace).persist(&foreign_record).unwrap();
    assert!(matches!(
        execute_status(Some(&mismatch_workspace)).unwrap_err(),
        SessionCommandError::WorkspaceMismatch { .. }
    ));

    let missing_session_workspace = temp_workspace("synod-cli-next-missing");
    let error = execute_next(Some(&missing_session_workspace)).unwrap_err();
    assert!(matches!(error, SessionCommandError::MissingActiveSession));
    let rendered = render_error("next", &error);
    assert!(rendered.contains("synod start"), "{rendered}");
}

#[test]
fn session_runtime_public_methods_cover_goal_flow_and_trace_management() {
    let workspace = temp_workspace("synod-session-runtime-public");
    let runtime = SessionRuntime::for_workspace(&workspace);

    assert_eq!(runtime.workspace_ref(), workspace.as_path());
    assert_eq!(runtime.latest_trace().unwrap(), None);
    assert_eq!(runtime.load_session().unwrap(), None);

    let mut session = build_started_session(&workspace);
    assert!(matches!(
        runtime.capture_goal(&mut session, "  ").unwrap_err(),
        SessionRuntimeError::MissingGoal
    ));
    runtime.capture_goal(&mut session, "  Fix the failing add test  ").unwrap();
    assert_eq!(session.goal.as_deref(), Some("Fix the failing add test"));
    assert_eq!(session.latest_status, SessionStatus::GoalCaptured);

    let mut initialized = build_started_session(&workspace);
    assert!(matches!(
        runtime.select_flow(&mut initialized, "missing").unwrap_err(),
        SessionRuntimeError::UnknownFlow { .. }
    ));
    runtime.select_flow(&mut initialized, "bug-fix").unwrap();
    assert_eq!(
        initialized.active_flow.as_ref().map(|flow| flow.flow_name.as_str()),
        Some("bug-fix")
    );

    let mut planned = build_planned_record(workspace.to_string_lossy().as_ref());
    assert!(matches!(
        runtime.select_flow(&mut planned, "delivery").unwrap_err(),
        SessionRuntimeError::FlowReplacementRequiresReset { .. }
    ));

    runtime.persist_session(&session).unwrap();
    assert!(runtime.load_session().unwrap().is_some());
    runtime.clear_session().unwrap();
    assert_eq!(runtime.load_session().unwrap(), None);
}

#[test]
fn session_runtime_runs_successful_terminal_and_replanned_execution_profiles() {
    let success_workspace =
        write_execution_workspace("synod-runtime-success", vec![success_attempt()]);
    let runtime = SessionRuntime::for_workspace(&success_workspace);
    let mut session = build_goal_captured_session(&success_workspace);
    runtime.plan_task(&mut session, Some("bug-fix"), false).unwrap();
    let response = runtime.run_to_terminal(&mut session).unwrap();
    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(session.latest_status, SessionStatus::Succeeded);
    assert!(session.latest_trace_ref.is_some());
    assert!(session.goal_plan.is_some());

    assert!(matches!(
        runtime.execute_next_step(&mut session).unwrap_err(),
        SessionRuntimeError::MissingActiveTask
    ));

    let replan_workspace = write_execution_workspace("synod-runtime-replan", replan_attempts());
    let replan_runtime = SessionRuntime::for_workspace(&replan_workspace);
    let mut replan_session = build_goal_captured_session(&replan_workspace);
    replan_runtime.plan_task(&mut replan_session, Some("bug-fix"), false).unwrap();
    let replan_response = replan_runtime.run_to_terminal(&mut replan_session).unwrap();
    assert_eq!(replan_response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(replan_session.latest_status, SessionStatus::Succeeded);
}

#[test]
fn session_runtime_surfaces_terminal_failures_for_broken_execution_profiles() {
    let workspace = write_execution_workspace("synod-runtime-failure", vec![failing_attempt()]);
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = build_goal_captured_session(&workspace);
    runtime.plan_task(&mut session, Some("bug-fix"), false).unwrap();
    let response = runtime.run_to_terminal(&mut session).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(session.latest_status, SessionStatus::Succeeded);
    assert!(session.latest_terminal_reason.is_some());
}

#[test]
fn session_runtime_public_error_paths_cover_missing_goal_task_and_terminal_shortcuts() {
    let workspace = write_execution_workspace("synod-runtime-errors", vec![success_attempt()]);
    let runtime = SessionRuntime::for_workspace(&workspace);

    let mut no_goal = build_started_session(&workspace);
    assert!(matches!(
        runtime.plan_task(&mut no_goal, None, false).unwrap_err(),
        SessionRuntimeError::MissingGoal
    ));
    assert!(matches!(
        runtime.execute_next_step(&mut no_goal).unwrap_err(),
        SessionRuntimeError::MissingGoal
    ));

    let mut invalid_flow = build_goal_captured_session(&workspace);
    invalid_flow.active_flow = Some(SessionFlowState {
        flow_name: "bug-fix".to_string(),
        current_stage_id: "verify".to_string(),
        current_stage_index: 0,
        total_stages: 3,
    });
    assert!(matches!(
        runtime.plan_task(&mut invalid_flow, None, false).unwrap_err(),
        SessionRuntimeError::InvalidFlowState(_)
    ));

    let mut missing_task = build_goal_captured_session(&workspace);
    assert!(matches!(
        runtime.execute_next_step(&mut missing_task).unwrap_err(),
        SessionRuntimeError::MissingActiveTask
    ));

    let mut missing_trace = build_goal_captured_session(&workspace);
    let mut terminal_task = build_task(workspace.to_string_lossy().as_ref());
    terminal_task.apply_terminal(
        TaskStatus::Succeeded,
        TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
    );
    missing_trace.active_task = Some(terminal_task);
    missing_trace.latest_status = SessionStatus::Succeeded;
    assert!(matches!(
        runtime.execute_next_step(&mut missing_trace).unwrap_err(),
        SessionRuntimeError::MissingTraceReference
    ));

    let mut missing_terminal_reason = build_goal_captured_session(&workspace);
    let mut broken_terminal_task = build_task(workspace.to_string_lossy().as_ref());
    broken_terminal_task.status = TaskStatus::Succeeded;
    missing_terminal_reason.active_task = Some(broken_terminal_task);
    missing_terminal_reason.latest_status = SessionStatus::Succeeded;
    missing_terminal_reason.latest_trace_ref =
        Some(workspace.join(".synod/traces/existing.json").to_string_lossy().into_owned());
    assert!(matches!(
        runtime.execute_next_step(&mut missing_terminal_reason).unwrap_err(),
        SessionRuntimeError::MissingTerminalReason
    ));

    let mut step_limited = build_goal_captured_session(&workspace);
    let mut step_limited_task = build_task(workspace.to_string_lossy().as_ref());
    step_limited_task.limits.max_steps = 0;
    step_limited.active_task = Some(step_limited_task);
    let response = runtime.run_to_terminal(&mut step_limited).unwrap();
    assert_eq!(response.terminal_status, TaskStatus::Exhausted);

    let mut no_next_step = build_goal_captured_session(&workspace);
    let mut no_next_step_task = build_task(workspace.to_string_lossy().as_ref());
    no_next_step_task.plan.current_step_index = no_next_step_task.plan.steps.len();
    no_next_step.active_task = Some(no_next_step_task);
    let response = runtime.run_to_terminal(&mut no_next_step).unwrap();
    assert_eq!(response.terminal_status, TaskStatus::Failed);
}
