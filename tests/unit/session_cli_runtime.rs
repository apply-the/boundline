use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use boundline::adapters::config_store::FileConfigStore;
use boundline::adapters::env_layer::{OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV};
use boundline::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use boundline::adapters::trace_store::TraceStore;
use boundline::cli::diagnostics::{diagnose_native_direct_run_workspace, diagnose_workspace};
use boundline::cli::inspect::{TraceSummaryError, summarize_trace};
use boundline::cli::run::{RunCommandError, execute_native_direct_run};
use boundline::cli::session::{
    SessionCommandError, execute_flow, execute_goal, execute_next, execute_plan, execute_run,
    execute_status, execute_step, render_error,
};
use boundline::cli::{
    Cli, CliValidationError, CommandExitStatus, CommandName, DeveloperCommand,
    DeveloperCommandSession,
};
use boundline::domain::brief::{
    GovernanceIntent, normalize_inputs, normalize_inputs_with_governance,
};
use boundline::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, ConfigFile, ModelRoute,
    PersistedAdapterConfiguration, RoutingConfig, RuntimeKind,
};
use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
    ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
    RelationshipCredibilityState, RelationshipKind, RelationshipProjection,
    RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
    RetrievalMatchOrigin, RetrievalMode, RetrievalSourceKind, RetrievalStalenessState,
    RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState,
    SemanticTraceEventKind, SemanticTraceRecord,
};
use boundline::domain::execution::{
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, ExecutionProfileError,
    WorkspaceChange, WorkspaceExecutionProfile,
};
use boundline::domain::flow::{
    FlowStepMetadata, FlowValidationError, SessionFlowState, attach_stage_metadata, built_in_flow,
    supported_flow_names_csv,
};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use boundline::domain::goal_plan::{
    ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, PlannedTask,
    PlanningAnalysisCoverage, PlanningAnalysisFinding, PlanningAnalysisProjection,
    PlanningAnalysisSeverity, PlanningAnalysisSource, PlanningAnalysisSourceRef,
    PlanningAnalysisState,
};
use boundline::domain::governance::{
    CanonModeSelectionPreference, CanonSemanticProvenanceBoundary, GovernanceLifecycleState,
    GovernanceRuntimeKind, GovernedSessionLifecycle,
};
use boundline::domain::limits::{RunLimits, TerminalCondition};
use boundline::domain::negotiation::NegotiationResolutionState;
use boundline::domain::plan::{Plan, PlanError, PlanStatus};
use boundline::domain::session::{
    ActiveSessionRecord, SessionCommand, SessionStatus, SessionStatusView, SessionTransition,
    SessionValidationError,
};
use boundline::domain::stage_council::StageCouncilStatus;
use boundline::domain::step::Step;
use boundline::domain::task::{
    Task, TaskPersistenceError, TaskRequestError, TaskRunRequest, TaskStatus, TerminalReason,
};
use boundline::domain::task_context::TaskContextError;
use boundline::domain::trace::{ExecutionTrace, TraceEventType};
use boundline::fixture::{
    build_fixture_plan_for_goal, build_task_request, sample_framework_adapter_describe_response,
    sample_framework_adapter_success_envelope,
};
use boundline::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};
use boundline::{
    CanonMode, CanonRuntimeConfig, GovernanceProfile, StageGovernancePolicy, SystemContextBinding,
};
use clap::Parser;
use serde_json::{Value, json};
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

fn write_describe_only_adapter_script(workspace: &Path) -> PathBuf {
    let describe_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        serde_json::to_value(sample_framework_adapter_describe_response()).unwrap(),
    ))
    .unwrap();
    let script_path = workspace.join(format!("status-adapter-{}.sh", Uuid::new_v4()));
    let script = format!(
        "#!/bin/sh\nset -eu\ncase \"$1\" in\n  describe)\n    cat <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );
    fs::write(&script_path, script).unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();
    script_path
}

// Serializes all tests that mutate or observe environment variables so that
// concurrent threads in the same test binary cannot interfere with each other.
static TEST_ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_test_env_lock() -> MutexGuard<'static, ()> {
    TEST_ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct EnvVarGuard {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
    // Held until the env vars are fully restored in Drop so no other thread
    // observes the temporary mutation.
    _lock: MutexGuard<'static, ()>,
}

impl EnvVarGuard {
    fn set(pairs: &[(&'static str, &'static str)]) -> Self {
        let _lock = acquire_test_env_lock();
        let saved = pairs.iter().map(|(key, _)| (*key, std::env::var_os(key))).collect::<Vec<_>>();
        unsafe {
            for (key, value) in pairs {
                std::env::set_var(key, value);
            }
        }
        Self { saved, _lock }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            for (key, value) in &self.saved {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
        // _lock is released here (after Drop::drop completes) because it is a
        // struct field, not a local, so it drops after the function body.
    }
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
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(build_task(workspace_ref)),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: Some(format!("{workspace_ref}/.boundline/traces/task.json")),
        created_at: 10,
        updated_at: 20,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    }
}

fn build_ready_goal_plan() -> Result<GoalPlan, Box<dyn std::error::Error>> {
    Ok(GoalPlan::new(
        "Deliver a bounded change",
        vec![PlannedTask {
            task_id: "T001".to_string(),
            description: "Update the bounded implementation".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("bounded change delivered".to_string()),
            decision_type_hint: None,
            depends_on: None,
        }],
    )?
    .with_planning_rationale("workspace evidence supports this bounded change")
    .with_verification_strategy("run the focused regression checks after editing"))
}

fn build_status_view(record: &ActiveSessionRecord) -> SessionStatusView {
    let active_task = record.active_task.as_ref();
    SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        goal: record.goal.clone(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        authored_input_summary: None,
        authored_input_sources: None,
        authored_input_deduplicated_sources: None,
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: None,
        clarification_questions: None,
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
        current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
        total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
        plan_revision: active_task.map(|task| task.plan.revision),
        current_step_id: active_task
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
        current_step_index: active_task.map(|task| task.plan.current_step_index),
        latest_status: record.latest_status,
        session_started_at: Some(record.created_at),
        execution_path: boundline::domain::session::execution_path_text(record),
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_decision_status: None,
        latest_decision_target: None,
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
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: active_task.and_then(|task| {
            task.context
                .state
                .get("latest_validation_status")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_exhaustion_reason: None,
        latest_review_trigger: None,
        latest_review_vote: None,
        latest_review_outcome: None,
        latest_review_headline: None,
        latest_governance_stage: None,
        latest_governance_runtime: None,
        latest_governance_mode: None,
        latest_governance_run_ref: None,
        latest_governance_state: None,
        latest_governance_runtime_state: None,
        latest_governance_rollout_profile: None,
        latest_governance_reason: None,
        latest_governance_contract_lines: None,
        latest_governance_approval_provenance: None,
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: None,
        latest_governance_packet_source_stage: None,
        latest_governance_packet_binding_reason: None,
        latest_governance_approval: None,
        latest_governance_decision: None,
        latest_governance_candidates: None,
        governance_next_action: None,
        next_command: Some("boundline step".to_string()),
        explanation: "session state is internally consistent".to_string(),
        ..Default::default()
    }
}

fn sample_advanced_context() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-session".to_string(),
        retrieval_mode: RetrievalMode::Local,
        retrieval_state: RetrievalState::Selected,
        retrieval_index_state: RetrievalIndexState::Ready,
        semantic_policy_state: SemanticPolicyState::Disabled,
        semantic_capability_state: SemanticCapabilityState::Unsupported,
        hybrid_outcome: HybridOutcome::BaselineOnly,
        budgets: Default::default(),
        remote_policy_state: RemoteTransmissionPolicyState::LocalOnly,
        used_remote: false,
        terminal_reason: None,
        selected_evidence: vec![RetrievedEvidenceCandidate {
            candidate_id: "candidate-1".to_string(),
            source_kind: RetrievalSourceKind::WorkspaceFile,
            source_ref: "src/lib.rs".to_string(),
            authority_rank: AuthorityRank::Structured,
            match_origin: RetrievalMatchOrigin::Fts,
            selection_state: CandidateSelectionState::Selected,
            selection_reason: "goal keyword matched the implementation surface".to_string(),
            provenance_summary: "workspace file selected through local retrieval".to_string(),
            compatibility_state: RetrievalCompatibilityState::Compatible,
            staleness_state: RetrievalStalenessState::Fresh,
            lexical_score: None,
            semantic_score: None,
            canon_semantic_contract_line: None,
            canon_semantic_provenance_ref: None,
        }],
        rejected_candidates: Vec::new(),
        semantic_trace_records: Vec::new(),
        relationships: vec![RelationshipProjection {
            relationship_id: "relationship-1".to_string(),
            subject_ref: "src/lib.rs".to_string(),
            relationship_kind: RelationshipKind::ExercisesTest,
            credibility_state: RelationshipCredibilityState::Credible,
            explanation: "the matching test file names the same target".to_string(),
            supporting_candidate_ids: vec!["candidate-1".to_string()],
        }],
        impact_findings: vec![ImpactAnalysisFinding {
            finding_id: "finding-1".to_string(),
            finding_kind: ImpactFindingKind::MissingTest,
            subject_ref: "tests/lib.rs".to_string(),
            status: ImpactFindingStatus::Open,
            severity: ImpactFindingSeverity::Medium,
            recommended_follow_up: "add or refresh the focused regression test".to_string(),
            supporting_relationship_ids: vec!["relationship-1".to_string()],
        }],
        context_pack_entries: Vec::new(),
        omission_findings: Vec::new(),
        repository_map_state: None,
        snapshot_cache_state: None,
        patch_safe_edit_attempts: Vec::new(),
    }
}

fn build_started_session(workspace: &Path) -> ActiveSessionRecord {
    let now = 10;
    ActiveSessionRecord {
        session_id: Uuid::new_v4().to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: now,
        updated_at: now,
        governance_lifecycle: None,
        project_scale: None,
        latest_voting: None,
        delight_feedback: None,
        active_execution_run_id: None,
    }
}

fn build_goal_captured_session(workspace: &Path) -> ActiveSessionRecord {
    let mut session = build_started_session(workspace);
    session.goal = Some("Fix the failing add test".to_string());
    session.latest_status = SessionStatus::GoalCaptured;
    session
}

fn save_local_routing(workspace: &Path, routing: RoutingConfig) {
    FileConfigStore::for_workspace(workspace)
        .save_local(&ConfigFile {
            version: 1,
            routing,
            canon: None,
            adapter: None,
            capability_provider: None,
        })
        .unwrap();
}

fn write_execution_workspace(prefix: &str, attempts: Vec<Value>) -> PathBuf {
    let workspace = temp_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();
    fs::write(
        workspace.join(".boundline/execution.json"),
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

fn write_governed_execution_workspace(
    prefix: &str,
    attempts: Vec<ExecutionAttemptDefinition>,
    governance: GovernanceProfile,
    limits: RunLimits,
) -> PathBuf {
    let workspace = temp_workspace(prefix);
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join("src/lib.rs"), RED_LIB_RS).unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), FIXTURE_TEST_RS).unwrap();

    let profile = WorkspaceExecutionProfile {
        name: "coverage-execution".to_string(),
        read_targets: vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
        validation_command: ExecutionCommand {
            program: "cargo".to_string(),
            args: vec!["test".to_string(), "--quiet".to_string()],
        },
        attempts,
        adaptive: None,
        limits,
        governance: Some(governance),
        review: None,
        legacy_source: None,
    };

    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&profile).unwrap(),
    )
    .unwrap();
    workspace
}

fn write_executable_script(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
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
    let workspace = PathBuf::from("/tmp/boundline-cli");
    let trace = PathBuf::from("/tmp/boundline-cli/trace.json");
    let commands = vec![
        DeveloperCommand::Doctor { workspace: Some(workspace.clone()), install: false },
        DeveloperCommand::Goal {
            workspace: Some(workspace.clone()),
            cluster: None,
            update: false,
            new_session: false,
            goal: Some("session goal".to_string()),
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            slug: None,
        },
        DeveloperCommand::Flow {
            name: "bug-fix".to_string(),
            workspace: Some(workspace.clone()),
            cluster: None,
        },
        DeveloperCommand::Plan {
            workspace: Some(workspace.clone()),
            cluster: None,
            input: None,
            flow: None,
            no_flow: false,
            no_canon: false,
            refine: false,
            no_refine: false,
            max_rounds: None,
        },
        DeveloperCommand::Step { workspace: Some(workspace.clone()), cluster: None },
        DeveloperCommand::Run {
            workspace: Some(workspace.clone()),
            cluster: None,
            goal: Some("ship it".to_string()),
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
            plan: None,
            accepted_plan: false,
            resume: None,
        },
        DeveloperCommand::Inspect {
            trace: Some(trace.clone()),
            workspace: Some(workspace.clone()),
            cluster: None,
            session: None,
            audit: false,
        },
        DeveloperCommand::Status {
            workspace: Some(workspace.clone()),
            cluster: None,
            session: None,
        },
        DeveloperCommand::Next { workspace: Some(workspace.clone()), cluster: None, session: None },
    ];

    for command in &commands {
        let session = DeveloperCommandSession::from_command(command);
        assert_eq!(session.command_name.as_str(), command.name().as_str());
    }

    assert_eq!(CommandName::Doctor.to_string(), "doctor");
    let cli = Cli::try_parse_from(["boundline", "inspect", "--workspace", "."]).unwrap();
    assert!(matches!(cli.command, Some(DeveloperCommand::Inspect { .. })));

    let invalid_doctor = DeveloperCommandSession {
        command_name: CommandName::Doctor,
        workspace_ref: Some(" ".to_string()),
        requires_workspace_ref: false,
        install_check: false,
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

    let invalid_goal = DeveloperCommandSession::from_command(&DeveloperCommand::Goal {
        workspace: Some(workspace.clone()),
        cluster: None,
        update: false,
        new_session: false,
        goal: Some("  ".to_string()),
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
        slug: None,
    });
    assert_eq!(
        invalid_goal.validate().unwrap_err(),
        CliValidationError::MissingGoal(CommandName::Goal)
    );

    let invalid_flow = DeveloperCommandSession::from_command(&DeveloperCommand::Flow {
        name: " ".to_string(),
        workspace: Some(workspace.clone()),
        cluster: None,
    });
    assert_eq!(invalid_flow.validate().unwrap_err(), CliValidationError::MissingFlowName);

    let direct_run_without_workspace =
        DeveloperCommandSession::from_command(&DeveloperCommand::Run {
            workspace: None,
            cluster: None,
            goal: Some("ship".to_string()),
            compatibility: false,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
            plan: None,
            accepted_plan: false,
            resume: None,
        });
    assert!(direct_run_without_workspace.validate().is_ok());

    let invalid_compatibility_run_workspace =
        DeveloperCommandSession::from_command(&DeveloperCommand::Run {
            workspace: None,
            cluster: None,
            goal: Some("ship".to_string()),
            compatibility: true,
            brief: Vec::new(),
            governance: None,
            risk: None,
            zone: None,
            owner: None,
            mode: None,
            no_canon: false,
            plan: None,
            accepted_plan: false,
            resume: None,
        });
    assert_eq!(
        invalid_compatibility_run_workspace.validate().unwrap_err(),
        CliValidationError::MissingWorkspaceRef(CommandName::Run)
    );

    let invalid_run_goal = DeveloperCommandSession::from_command(&DeveloperCommand::Run {
        workspace: Some(workspace.clone()),
        cluster: None,
        goal: Some(" ".to_string()),
        compatibility: false,
        brief: Vec::new(),
        governance: None,
        risk: None,
        zone: None,
        owner: None,
        mode: None,
        no_canon: false,
        plan: None,
        accepted_plan: false,
        resume: None,
    });
    assert_eq!(
        invalid_run_goal.validate().unwrap_err(),
        CliValidationError::MissingGoal(CommandName::Run)
    );

    let invalid_inspect = DeveloperCommandSession::from_command(&DeveloperCommand::Inspect {
        trace: None,
        workspace: None,
        cluster: None,
        session: None,
        audit: false,
    });
    assert_eq!(invalid_inspect.validate().unwrap_err(), CliValidationError::MissingTraceSelection);

    let mut completed = DeveloperCommandSession::from_command(&DeveloperCommand::Status {
        workspace: Some(workspace),
        cluster: None,
        session: None,
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
fn native_direct_run_diagnostics_do_not_require_execution_profile() {
    let workspace = temp_workspace("boundline-native-direct-run-diagnostics");
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();

    let native_report = diagnose_native_direct_run_workspace(&workspace);
    assert!(native_report.ready, "{native_report:?}");
    assert!(native_report.checks.iter().any(|check| check.name == "workspace_execution_profile"
        && check.message == "execution profile is optional for native direct run"));

    let compatibility_report = diagnose_workspace(&workspace);
    assert!(!compatibility_report.ready, "{compatibility_report:?}");
    assert!(
        compatibility_report
            .missing_prerequisites
            .contains(&"workspace_execution_profile".to_string())
    );
}

#[test]
fn native_direct_run_diagnostics_ignore_invalid_profile_when_optional() {
    let workspace = temp_workspace("boundline-native-direct-run-invalid-profile");
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::write(workspace.join("Cargo.toml"), FIXTURE_CARGO_TOML).unwrap();
    fs::write(workspace.join(".boundline/execution.json"), "{not valid json").unwrap();

    let native_report = diagnose_native_direct_run_workspace(&workspace);
    assert!(native_report.ready, "{native_report:?}");
    assert!(native_report.checks.iter().any(|check| {
        check.name == "workspace_execution_profile"
            && check
                .message
                .contains("execution profile is optional for native direct run; current profile state is ignored")
    }));

    let compatibility_report = diagnose_workspace(&workspace);
    assert!(!compatibility_report.ready, "{compatibility_report:?}");
    assert!(compatibility_report.checks.iter().any(|check| {
        check.name == "workspace_execution_profile"
            && check.message.contains("workspace execution profile is unavailable")
    }));
}

#[test]
fn native_direct_run_reuses_existing_initialized_session() {
    let workspace = write_execution_workspace(
        "boundline-native-direct-run-initialized",
        vec![success_attempt()],
    );
    FileSessionStore::for_workspace(&workspace)
        .persist(&build_started_session(&workspace))
        .unwrap();

    // Hold the env lock for the entire run so that a concurrent test using
    // EnvVarGuard cannot inject API-key env vars that cause route_is_available
    // to return true, which would register AI agents that fail at connection time.
    let report = {
        let _env_lock = acquire_test_env_lock();
        execute_native_direct_run(
            &workspace,
            Some("Fix the failing add test"),
            &[],
            None,
            None,
            None,
            None,
            None,
            false,
        )
        .unwrap()
    };

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(report.terminal_output.contains("routing: native (goal_plan)"));
    assert!(report.trace_location.is_some());
}

#[test]
fn execute_status_surfaces_blocked_goal_capture_routing_for_goal_captured_session() {
    let workspace = temp_workspace("boundline-status-goal-captured");
    let canonical_workspace = workspace.canonicalize().unwrap();
    FileSessionStore::for_workspace(&canonical_workspace)
        .persist(&build_goal_captured_session(&canonical_workspace))
        .unwrap();

    let report = execute_status(Some(&canonical_workspace)).unwrap();

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains(
            "routing: blocked (goal_capture) - goal captured but a goal plan is not ready yet"
        ),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains(
            "execution_condition: blocked - goal captured but a goal plan is not ready yet"
        ),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("latest_status: goal_captured"),
        "{}",
        report.terminal_output
    );
}

#[test]
fn execute_status_surfaces_adapter_config_projection_details() {
    let workspace = temp_workspace("boundline-status-adapter-config-projection");
    let canonical_workspace = workspace.canonicalize().unwrap();
    let adapter_script = write_describe_only_adapter_script(&canonical_workspace);
    FileConfigStore::for_workspace(&canonical_workspace)
        .save_local(&ConfigFile {
            adapter: Some(PersistedAdapterConfiguration {
                selection: AdapterSelectionRecord {
                    selection_mode: AdapterSelectionMode::Custom,
                    adapter_id: "custom-guided".to_string(),
                    display_name: "Custom Guided".to_string(),
                    command: "/bin/sh".to_string(),
                    args: vec![adapter_script.to_string_lossy().into_owned()],
                    registration_source: AdapterRegistrationSource::AdapterAdd,
                    discovery_state: AdapterDiscoveryState::ExplicitCommand,
                    compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                    updated_at: 42,
                },
                schema_fingerprint: format!(
                    "{}:{}:template_repo",
                    FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, "speckit"
                ),
                completeness_state: AdapterConfigCompletenessState::Complete,
                interactive_resolution: true,
                last_validated_at: Some(42),
                value_count: 1,
                values: vec![AdapterConfigValueRecord {
                    field_key: "template_repo".to_string(),
                    value_kind: AdapterValueKind::Path,
                    secret: false,
                    string_value: None,
                    path_value: Some("../boundline-framework-template".to_string()),
                    bool_value: None,
                    int_value: None,
                    value_source: AdapterValueSource::OperatorPrompt,
                    resolution_state: StoredAdapterConfigValueState::Present,
                }],
            }),
            ..ConfigFile::default()
        })
        .unwrap();
    FileSessionStore::for_workspace(&canonical_workspace)
        .persist(&build_goal_captured_session(&canonical_workspace))
        .unwrap();

    let report = execute_status(Some(&canonical_workspace)).unwrap();

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains("framework_adapter_config_state: complete"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("framework_adapter_interactive_resolution: true"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("framework_adapter_value_count: 1"),
        "{}",
        report.terminal_output
    );
}

#[test]
fn execute_status_revalidates_adapter_config_state_when_required_value_is_missing() {
    let workspace = temp_workspace("boundline-status-adapter-config-revalidation");
    let canonical_workspace = workspace.canonicalize().unwrap();
    let adapter_script = write_describe_only_adapter_script(&canonical_workspace);
    FileConfigStore::for_workspace(&canonical_workspace)
        .save_local(&ConfigFile {
            adapter: Some(PersistedAdapterConfiguration {
                selection: AdapterSelectionRecord {
                    selection_mode: AdapterSelectionMode::Custom,
                    adapter_id: "custom-guided".to_string(),
                    display_name: "Custom Guided".to_string(),
                    command: "/bin/sh".to_string(),
                    args: vec![adapter_script.to_string_lossy().into_owned()],
                    registration_source: AdapterRegistrationSource::AdapterAdd,
                    discovery_state: AdapterDiscoveryState::ExplicitCommand,
                    compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                    updated_at: 42,
                },
                schema_fingerprint: format!(
                    "{}:{}:template_repo",
                    FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, "speckit"
                ),
                completeness_state: AdapterConfigCompletenessState::Complete,
                interactive_resolution: false,
                last_validated_at: Some(42),
                value_count: 0,
                values: Vec::new(),
            }),
            ..ConfigFile::default()
        })
        .unwrap();
    FileSessionStore::for_workspace(&canonical_workspace)
        .persist(&build_goal_captured_session(&canonical_workspace))
        .unwrap();

    let report = execute_status(Some(&canonical_workspace)).unwrap();

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains("framework_adapter_config_state: missing_required"),
        "{}",
        report.terminal_output
    );
}

#[test]
fn native_direct_run_surfaces_clarification_without_planning() {
    let workspace = temp_workspace("boundline-native-direct-run-clarification");

    let report = execute_native_direct_run(
        &workspace,
        Some("Improve the platform docs and fix whatever tests are broken"),
        &[],
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .unwrap();

    assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
    assert!(report.terminal_output.contains(
        "clarification_headline: clarification required: narrow the request to one bounded outcome"
    ));
    assert!(report.trace_location.is_none());
}

#[test]
fn native_direct_run_rejects_meaningful_active_session_state() {
    let workspace = temp_workspace("boundline-native-direct-run-conflict");
    let session_store = FileSessionStore::for_workspace(&workspace);
    session_store.persist(&build_goal_captured_session(&workspace)).unwrap();

    let error = execute_native_direct_run(
        &workspace,
        Some("Ship the checkout change"),
        &[],
        None,
        None,
        None,
        None,
        None,
        false,
    )
    .unwrap_err();

    assert!(matches!(error, RunCommandError::ActiveSessionConflict));
}

#[test]
fn task_plan_and_session_store_validation_cover_error_branches() {
    let workspace = "/tmp/boundline-task-coverage";

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

    let workspace = temp_workspace("boundline-corrupt-session-store");
    let store = FileSessionStore::for_workspace(&workspace);
    assert!(store.path().ends_with(Path::new(".boundline/session.json")));
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
    let workspace = "/tmp/boundline-session-coverage";

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
fn session_status_view_tracks_latest_advanced_context_snapshot() {
    let workspace = "/tmp/session-status-advanced-context";
    let mut record = build_planned_record(workspace);
    let expected_advanced_context = sample_advanced_context();
    record
        .active_task
        .as_mut()
        .unwrap()
        .context
        .set_latest_advanced_context(&expected_advanced_context)
        .unwrap();

    let persisted_advanced_context =
        record.active_task.as_ref().unwrap().context.latest_advanced_context().unwrap().unwrap();
    assert_eq!(persisted_advanced_context.semantic_policy_state, SemanticPolicyState::Disabled);
    assert_eq!(
        persisted_advanced_context.semantic_capability_state,
        SemanticCapabilityState::Unsupported
    );
    assert_eq!(persisted_advanced_context.hybrid_outcome, HybridOutcome::BaselineOnly);

    let mut matching_status = build_status_view(&record);
    matching_status.advanced_context = Some(expected_advanced_context.clone());
    matching_status.validate(&record).unwrap();

    let mut wrong_status = build_status_view(&record);
    wrong_status.advanced_context = Some(AdvancedContextProjection {
        query_id: "query-session-mismatch".to_string(),
        ..expected_advanced_context
    });
    assert!(matches!(
        wrong_status.validate(&record).unwrap_err(),
        SessionValidationError::StatusViewAdvancedContextMismatch { .. }
    ));
}

#[test]
fn session_status_view_preserves_canon_semantic_skip_reason_snapshot() {
    let workspace = "/tmp/session-status-canon-semantic";
    let mut record = build_planned_record(workspace);
    let mut expected_advanced_context = sample_advanced_context();
    expected_advanced_context.semantic_policy_state = SemanticPolicyState::Local;
    expected_advanced_context.semantic_capability_state = SemanticCapabilityState::Ready;
    expected_advanced_context.hybrid_outcome = HybridOutcome::BaselineOnly;
    expected_advanced_context.rejected_candidates = vec![RetrievedEvidenceCandidate {
        candidate_id: "candidate-canon-skipped".to_string(),
        source_kind: RetrievalSourceKind::CanonArtifact,
        source_ref: ".canon/excluded-guidance.md".to_string(),
        authority_rank: AuthorityRank::Canon,
        match_origin: RetrievalMatchOrigin::StructuredFallback,
        selection_state: CandidateSelectionState::Rejected,
        selection_reason:
            "Canon semantic compatibility skipped the artifact: excluded by Canon semantic policy"
                .to_string(),
        provenance_summary: "excluded canon artifact surfaced through session state".to_string(),
        compatibility_state: RetrievalCompatibilityState::PolicyBlocked,
        staleness_state: RetrievalStalenessState::Fresh,
        lexical_score: None,
        semantic_score: None,
        canon_semantic_contract_line: Some("v1".to_string()),
        canon_semantic_provenance_ref: Some(
            ".canon/excluded-guidance.md#section:overview".to_string(),
        ),
    }];
    expected_advanced_context.semantic_trace_records = vec![SemanticTraceRecord {
        record_id: "trace-canon-skip".to_string(),
        event_kind: SemanticTraceEventKind::CanonArtifactSkipped,
        candidate_ref: Some(".canon/excluded-guidance.md".to_string()),
        match_origin: None,
        compatibility_state: Some(RetrievalCompatibilityState::PolicyBlocked),
        semantic_score: None,
        canon_artifact_class: Some("stable".to_string()),
        canon_semantic_contract_line: Some("v1".to_string()),
        canon_semantic_provenance_boundary: Some(CanonSemanticProvenanceBoundary::Section),
        canon_semantic_provenance_ref: Some(
            ".canon/excluded-guidance.md#section:overview".to_string(),
        ),
        reason: "excluded by Canon semantic policy".to_string(),
    }];

    record
        .active_task
        .as_mut()
        .unwrap()
        .context
        .set_latest_advanced_context(&expected_advanced_context)
        .unwrap();

    let mut matching_status = build_status_view(&record);
    matching_status.advanced_context = Some(expected_advanced_context.clone());
    matching_status.validate(&record).unwrap();

    let preserved = matching_status.advanced_context.expect("advanced context snapshot");
    assert_eq!(
        preserved.rejected_candidates[0].compatibility_state,
        RetrievalCompatibilityState::PolicyBlocked
    );
    assert!(
        preserved.rejected_candidates[0]
            .selection_reason
            .contains("excluded by Canon semantic policy")
    );
    assert_eq!(
        preserved.semantic_trace_records[0].event_kind,
        SemanticTraceEventKind::CanonArtifactSkipped
    );
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

    let workspace = temp_workspace("boundline-cli-session-errors");
    FileSessionStore::for_workspace(&workspace)
        .persist(&build_started_session(&workspace))
        .unwrap();
    assert!(matches!(
        execute_plan(Some(&workspace), None, false).unwrap_err(),
        SessionCommandError::MissingCapturedGoal
    ));

    execute_goal(Some(&workspace), Some("Fix the failing add test"), &[], None, None, None, None)
        .unwrap();
    assert!(matches!(
        execute_step(Some(&workspace)).unwrap_err(),
        SessionCommandError::MissingPlannedTask
    ));
    assert!(matches!(
        execute_flow(Some(&workspace), "missing-flow").unwrap_err(),
        SessionCommandError::UnknownFlow { .. }
    ));

    let mismatch_workspace = temp_workspace("boundline-cli-session-mismatch");
    let foreign_workspace = temp_workspace("boundline-cli-session-foreign");
    let foreign_record = build_planned_record(foreign_workspace.to_string_lossy().as_ref());
    FileSessionStore::for_workspace(&mismatch_workspace).persist(&foreign_record).unwrap();
    assert!(matches!(
        execute_status(Some(&mismatch_workspace)).unwrap_err(),
        SessionCommandError::WorkspaceMismatch { .. }
    ));

    let missing_session_workspace = temp_workspace("boundline-cli-next-missing");
    let error = execute_next(Some(&missing_session_workspace)).unwrap_err();
    assert!(matches!(error, SessionCommandError::MissingActiveSession));
    let rendered = render_error("next", &error);
    assert!(rendered.contains("boundline goal --goal <goal>"), "{rendered}");
}

#[test]
fn execute_run_blocks_when_plan_quality_requires_clarification()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-cli-run-plan-quality-block");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    record.goal_plan = Some(build_ready_goal_plan()?.with_verification_strategy(" "));
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);

    let Some(session_status) = report.session_status.as_ref() else {
        return Err(std::io::Error::other("run report missing session status").into());
    };

    assert_eq!(session_status.latest_status, SessionStatus::Blocked);
    assert_eq!(session_status.plan_quality_state.as_deref(), Some("clarification_required"));
    assert!(report.terminal_output.contains("current goal plan is not ready for execution"));

    let persisted = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or_else(|| std::io::Error::other("run block must persist the active session"))?;
    let trace_ref = persisted
        .latest_trace_ref
        .as_deref()
        .ok_or_else(|| std::io::Error::other("run block must persist a plan-quality trace"))?;
    let trace =
        SessionRuntime::for_workspace(&workspace).trace_store().load(Path::new(trace_ref))?;
    let goal_plan_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::GoalPlanCreated)
        .ok_or_else(|| std::io::Error::other("plan-quality trace missing goal plan event"))?;
    assert_eq!(goal_plan_event.payload["plan_quality_state"], "clarification_required");
    assert_eq!(goal_plan_event.payload["plan_quality_findings"], json!(["verification_strategy"]));

    Ok(())
}

#[test]
fn summarize_trace_surfaces_plan_quality_projection() -> Result<(), Box<dyn std::error::Error>> {
    let mut trace = ExecutionTrace::new("plan-quality-trace", "session-quality", "Deliver safely");
    trace.record_event(
        TraceEventType::GoalPlanCreated,
        None,
        0,
        json!({
            "goal": "Deliver safely",
            "task_count": 1,
            "goal_plan_state": "proposed",
            "goal_plan_revision": 0,
            "plan_quality_state": "clarification_required",
            "plan_quality_findings": ["verification_strategy"],
            "plan_quality_assumptions": ["no explicit route override is required for this plan"]
        }),
    );
    trace.finalize(
        TaskStatus::Failed,
        TerminalReason::new(
            TerminalCondition::NoCredibleNextStep,
            "plan quality requires clarification",
            None,
        ),
    );

    let summary = summarize_trace("/tmp/trace.json", &trace)?;
    assert_eq!(summary.plan_quality_state.as_deref(), Some("clarification_required"));
    assert_eq!(summary.plan_quality_findings, vec!["verification_strategy".to_string()]);
    assert_eq!(
        summary.plan_quality_assumptions,
        vec!["no explicit route override is required for this plan".to_string()]
    );

    Ok(())
}

#[test]
fn persist_blocked_plan_quality_trace_records_blocked_and_ignores_ready_plans()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-cli-plan-quality-trace");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = build_planned_record(workspace.to_string_lossy().as_ref());
    session.latest_trace_ref = None;
    session.goal_plan = None;

    runtime.persist_blocked_plan_quality_trace(&mut session)?;
    assert!(session.latest_trace_ref.is_none());

    let ready_goal_plan = build_ready_goal_plan()?;
    assert_eq!(ready_goal_plan.plan_quality_state().as_deref(), Some("ready"));
    session.goal_plan = Some(ready_goal_plan.clone());
    session.latest_trace_ref = Some("ready-trace".to_string());
    runtime.persist_blocked_plan_quality_trace(&mut session)?;
    assert_eq!(session.latest_trace_ref.as_deref(), Some("ready-trace"));

    let blocked_goal_plan = ready_goal_plan.with_context_pack(ContextPack {
        pack_id: "cp-blocked".to_string(),
        summary: "stale context".to_string(),
        credibility: ContextPackCredibility::Stale,
        inputs: vec![ContextInput {
            kind: ContextInputKind::RecentTrace,
            reference: ".boundline/traces/old.json".to_string(),
            rationale: "was the last authoritative trace".to_string(),
            source: "latest_trace".to_string(),
            primary: false,
        }],
        selected_targets: Vec::new(),
        advanced_context: None,
        staleness_reason: Some("refresh the context before continuing".to_string()),
    });
    assert_eq!(blocked_goal_plan.plan_quality_state().as_deref(), Some("blocked"));

    session.goal_plan = Some(blocked_goal_plan);
    session.latest_trace_ref = None;
    runtime.persist_blocked_plan_quality_trace(&mut session)?;

    let trace_ref = session
        .latest_trace_ref
        .as_deref()
        .ok_or_else(|| std::io::Error::other("blocked plan quality must persist a trace"))?;
    let trace = runtime.trace_store().load(Path::new(trace_ref))?;
    let goal_plan_event = trace
        .events
        .iter()
        .find(|event| event.event_type == TraceEventType::GoalPlanCreated)
        .ok_or_else(|| std::io::Error::other("plan-quality trace missing goal plan event"))?;
    assert_eq!(goal_plan_event.payload["plan_quality_state"], "blocked");
    assert_eq!(goal_plan_event.payload["plan_quality_findings"], json!(["context_pack_stale"]));
    assert_eq!(
        goal_plan_event.payload["plan_quality_assumptions"],
        json!(["no explicit route override is required for this plan"])
    );

    Ok(())
}

#[test]
fn execute_run_blocks_when_backlog_quality_is_not_ready() -> Result<(), Box<dyn std::error::Error>>
{
    let workspace = temp_workspace("boundline-cli-run-backlog-quality-block");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    record.goal_plan = Some(build_ready_goal_plan()?);
    record.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: None,
        selected_mode_sequence: vec![
            CanonMode::Discovery,
            CanonMode::Architecture,
            CanonMode::Backlog,
        ],
        latest_reasoning_profile: None,
        current_stage_index: 2,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    });
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);

    let Some(session_status) = report.session_status.as_ref() else {
        return Err(std::io::Error::other("run report missing session status").into());
    };

    assert_eq!(session_status.latest_status, SessionStatus::Blocked);
    assert_eq!(session_status.backlog_quality_state.as_deref(), Some("clarification_required"));
    assert_eq!(
        session_status.backlog_quality_findings.as_ref(),
        Some(&vec![
            "backlog_packet_pending".to_string(),
            "missing_backlog_document".to_string(),
            "missing_section:backlog_overview".to_string(),
            "missing_section:epic_tree".to_string(),
            "missing_section:capability_to_epic_map".to_string(),
            "missing_section:dependency_map".to_string(),
            "missing_section:delivery_slices".to_string(),
            "missing_section:sequencing_plan".to_string(),
            "missing_section:acceptance_anchors".to_string(),
            "missing_section:planning_risks".to_string(),
        ])
    );
    assert!(report.terminal_output.contains("governed backlog packet is not ready for execution"));

    Ok(())
}

#[test]
fn execute_run_blocks_when_planning_analysis_is_blocked() -> Result<(), Box<dyn std::error::Error>>
{
    let workspace = temp_workspace("boundline-cli-run-planning-analysis-block");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    let mut goal_plan = build_ready_goal_plan()?;
    goal_plan.planning_analysis = Some(PlanningAnalysisProjection {
        state: PlanningAnalysisState::Blocked,
        findings: vec![PlanningAnalysisFinding {
            severity: PlanningAnalysisSeverity::Critical,
            source: PlanningAnalysisSource::Goal,
            code: "success_criterion_uncovered".to_string(),
            message: "acceptance target is not covered by the active plan".to_string(),
            source_refs: vec![PlanningAnalysisSourceRef {
                artifact_kind: "goal_plan".to_string(),
                artifact_ref: "T001".to_string(),
                anchor: Some("acceptance target".to_string()),
            }],
        }],
        coverage: Some(PlanningAnalysisCoverage {
            success_criteria_total: 1,
            success_criteria_covered: 0,
            backlog_slice_total: Some(1),
            backlog_slice_covered: Some(0),
            validation_anchor_total: None,
            validation_anchor_covered: None,
            risk_total: None,
            risk_covered: None,
            constraint_total: None,
            constraint_covered: None,
            governed_evidence_ready: false,
        }),
    });
    record.goal_plan = Some(goal_plan);
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);

    let Some(session_status) = report.session_status.as_ref() else {
        return Err(std::io::Error::other("run report missing session status").into());
    };

    assert_eq!(session_status.latest_status, SessionStatus::Blocked);
    assert_eq!(session_status.planning_analysis_state.as_deref(), Some("blocked"));
    assert!(report.terminal_output.contains("planning analysis found a blocking execution gap"));

    Ok(())
}

#[test]
fn execute_run_prefers_backlog_quality_before_planning_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-cli-run-backlog-before-analysis");
    let mut record = build_planned_record(workspace.to_string_lossy().as_ref());
    let mut goal_plan = build_ready_goal_plan()?;
    goal_plan.planning_analysis = Some(PlanningAnalysisProjection {
        state: PlanningAnalysisState::Blocked,
        findings: vec![PlanningAnalysisFinding {
            severity: PlanningAnalysisSeverity::Critical,
            source: PlanningAnalysisSource::Goal,
            code: "success_criterion_uncovered".to_string(),
            message: "acceptance target is not covered by the active plan".to_string(),
            source_refs: vec![PlanningAnalysisSourceRef {
                artifact_kind: "goal_plan".to_string(),
                artifact_ref: "T001".to_string(),
                anchor: Some("acceptance target".to_string()),
            }],
        }],
        coverage: Some(PlanningAnalysisCoverage {
            success_criteria_total: 1,
            success_criteria_covered: 0,
            backlog_slice_total: Some(1),
            backlog_slice_covered: Some(0),
            validation_anchor_total: None,
            validation_anchor_covered: None,
            risk_total: None,
            risk_covered: None,
            constraint_total: None,
            constraint_covered: None,
            governed_evidence_ready: false,
        }),
    });
    record.goal_plan = Some(goal_plan);
    record.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: None,
        selected_mode_sequence: vec![
            CanonMode::Discovery,
            CanonMode::Architecture,
            CanonMode::Backlog,
        ],
        latest_reasoning_profile: None,
        current_stage_index: 2,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    });
    FileSessionStore::for_workspace(&workspace).persist(&record)?;

    let report = execute_run(Some(&workspace))?;
    assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
    assert!(report.terminal_output.contains("governed backlog packet is not ready for execution"));

    Ok(())
}

#[test]
fn session_runtime_public_methods_cover_goal_flow_and_trace_management() {
    let workspace = temp_workspace("boundline-session-runtime-public");
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
        write_execution_workspace("boundline-runtime-success", vec![success_attempt()]);
    let runtime = SessionRuntime::for_workspace(&success_workspace);
    let mut session = build_goal_captured_session(&success_workspace);
    runtime.plan_task(&mut session, Some("bug-fix"), false).unwrap();
    // Hold the env lock so a concurrent EnvVarGuard cannot inject API-key env
    // vars that make route_is_available return true inside
    // build_fixture_runtime_for_goal_plan, which would swap the fixture agent
    // for an AI agent that fails at the connection step.
    let response = {
        let _env_lock = acquire_test_env_lock();
        runtime.run_to_terminal(&mut session).unwrap()
    };
    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(session.latest_status, SessionStatus::Succeeded);
    assert!(session.latest_trace_ref.is_some());
    assert!(session.goal_plan.is_some());

    runtime.execute_next_step(&mut session).unwrap();
    assert_eq!(session.latest_status, SessionStatus::Succeeded);

    let replan_workspace = write_execution_workspace("boundline-runtime-replan", replan_attempts());
    let replan_runtime = SessionRuntime::for_workspace(&replan_workspace);
    let mut replan_session = build_goal_captured_session(&replan_workspace);
    replan_runtime.plan_task(&mut replan_session, Some("bug-fix"), false).unwrap();
    let replan_response = {
        let _env_lock = acquire_test_env_lock();
        replan_runtime.run_to_terminal(&mut replan_session).unwrap()
    };
    assert_eq!(replan_response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(replan_session.latest_status, SessionStatus::Succeeded);
}

#[test]
fn session_runtime_run_to_terminal_uses_default_canon_mode_for_required_investigate_stage() {
    let command_workspace = temp_workspace("boundline-runtime-governed-mode-command");
    let command_path = command_workspace.join("fake-canon");
    let response_path = command_workspace.join("canon-response.json");
    let document_ref = ".canon/runs/canon-run-investigate/discovery.md";
    fs::write(
        &response_path,
        json!({
            "status": "governed_ready",
            "approval_state": "not_needed",
            "message": "Canon completed the governed stage",
            "run_ref": "canon-run-investigate",
            "packet_ref": ".canon/runs/canon-run-investigate",
            "expected_document_refs": [document_ref],
            "document_refs": [document_ref],
            "packet_readiness": "reusable",
            "missing_sections": [],
            "authority_governance": {
                "contract_line": "authority-governance-v1",
                "authority_zone": "green",
                "change_class": "low-impact",
                "intended_persona": "delivery-engineer",
                "approval_state": "not_needed",
                "packet_readiness": "reusable",
                "risk": "low-impact"
            },
            "headline": "discovery packet ready",
            "reason_code": "packet_ready"
        })
        .to_string(),
    )
    .unwrap();
    write_executable_script(
        &command_path,
        &format!("#!/bin/sh\ncat >/dev/null\ncat '{}'\n", response_path.to_string_lossy()),
    );

    let workspace = write_governed_execution_workspace(
        "boundline-runtime-governed-default-mode",
        vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left - right".to_string(),
                replace: "left + right".to_string(),
            }],
        }],
        GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: Some(CanonRuntimeConfig {
                command: command_path.to_string_lossy().into_owned(),
                default_owner: Some("platform".to_string()),
                default_risk: Some("medium".to_string()),
                default_zone: Some("engineering".to_string()),
                default_system_context: Some(SystemContextBinding::Existing),
            }),
            stages: vec![StageGovernancePolicy {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                enabled: true,
                required: true,
                autopilot: false,
                require_adaptive_companion: false,
                runtime: Some(GovernanceRuntimeKind::Canon),
                canon_mode: None,
                system_context: Some(SystemContextBinding::Existing),
                risk: None,
                zone: None,
                owner: None,
                reasoning_profile: None,
            }],
        },
        RunLimits { max_steps: 1, ..RunLimits::default() },
    );
    let document_path = workspace.join(document_ref);
    fs::create_dir_all(document_path.parent().unwrap()).unwrap();
    fs::write(&document_path, "# Discovery\n\nCredible governed evidence.\n").unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = build_goal_captured_session(&workspace);
    runtime.select_flow(&mut session, "bug-fix").unwrap();
    let request = build_task_request(
        &workspace,
        session.goal.clone().unwrap(),
        session.session_id.clone(),
        None,
        None,
    )
    .unwrap();
    let plan = build_fixture_plan_for_goal(
        &workspace,
        session.active_flow.as_ref(),
        session.goal.as_deref().unwrap(),
    )
    .unwrap();
    session.active_task =
        Some(Task::new("task-runtime-governed-default-mode", &request, plan).unwrap());
    session.latest_status = SessionStatus::Planned;

    let response = runtime.run_to_terminal(&mut session).unwrap();
    let governed_stage = response.final_context.latest_governance_stage().unwrap().unwrap();
    let governed_packet = response.final_context.latest_governance_packet().unwrap().unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Exhausted);
    assert_eq!(governed_stage.stage_key, "bug-fix:investigate");
    assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);
    assert_eq!(governed_packet.canon_mode, Some(CanonMode::Discovery));
}

#[test]
fn session_runtime_surfaces_terminal_failures_for_broken_execution_profiles() {
    let workspace = write_execution_workspace("boundline-runtime-failure", vec![failing_attempt()]);
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = build_goal_captured_session(&workspace);
    runtime.select_flow(&mut session, "bug-fix").unwrap();
    let request = build_task_request(
        &workspace,
        session.goal.clone().unwrap(),
        session.session_id.clone(),
        None,
        None,
    )
    .unwrap();
    let plan = build_fixture_plan_for_goal(
        &workspace,
        session.active_flow.as_ref(),
        session.goal.as_deref().unwrap(),
    )
    .unwrap();
    session.active_task = Some(Task::new("task-runtime-failure", &request, plan).unwrap());
    session.latest_status = SessionStatus::Planned;
    let response = runtime.run_to_terminal(&mut session).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Failed);
    assert_eq!(session.latest_status, SessionStatus::Failed);
    assert!(session.latest_terminal_reason.is_some());
    assert_eq!(
        session.latest_terminal_reason.as_ref().unwrap().condition,
        TerminalCondition::UnrecoverableError
    );
}

#[test]
fn session_runtime_public_error_paths_cover_missing_goal_task_and_terminal_shortcuts() {
    let workspace = write_execution_workspace("boundline-runtime-errors", vec![success_attempt()]);
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
        Some(workspace.join(".boundline/traces/existing.json").to_string_lossy().into_owned());
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

#[test]
fn session_runtime_capture_goal_uses_authored_brief_packet_projection() {
    let workspace = temp_workspace("boundline-runtime-authored-brief-capture");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let authored_brief = normalize_inputs(
        &workspace,
        Some("Improve the platform docs and fix whatever tests are broken"),
        &[],
    )
    .unwrap();
    let expected_summary = authored_brief.summary_text();
    let expected_headline = authored_brief.clarification_headline();

    let mut session = build_started_session(&workspace);
    session.authored_brief = Some(authored_brief);

    runtime
        .capture_goal(&mut session, "Improve the platform docs and fix whatever tests are broken")
        .unwrap();

    let packet = session.negotiation_packet.expect("capture should persist a negotiation packet");
    assert_eq!(packet.source_summary, expected_summary);
    assert_eq!(packet.clarification_headline, expected_headline);
    assert_eq!(packet.resolution_state, NegotiationResolutionState::PendingClarification);
}

#[test]
fn session_runtime_blocks_planning_when_authored_brief_needs_clarification() {
    let workspace = temp_workspace("boundline-runtime-authored-brief-clarification");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let authored_brief = normalize_inputs(
        &workspace,
        Some("Improve the platform docs and fix whatever tests are broken"),
        &[],
    )
    .unwrap();
    let expected_headline = authored_brief
        .clarification_headline()
        .expect("broad authored brief should request clarification");
    let expected_prompt = authored_brief
        .clarification_prompt()
        .expect("broad authored brief should carry a clarification prompt");

    let mut session = build_goal_captured_session(&workspace);
    session.goal = Some(authored_brief.render_goal_text());
    session.authored_brief = Some(authored_brief);
    session.negotiation_packet = None;

    let error = runtime.plan_task(&mut session, None, false).unwrap_err();
    assert!(matches!(
        error,
        SessionRuntimeError::ClarificationRequired { headline, prompt }
            if headline == expected_headline && prompt == expected_prompt
    ));
    assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
    assert!(session.latest_trace_ref.is_none());
}

#[test]
fn session_runtime_blocks_discovery_stage_council_when_reviewer_routes_collapse() {
    let workspace = temp_workspace("boundline-runtime-discovery-stage-council-block");
    fs::write(
        workspace.join("brief.md"),
        concat!(
            "Goal: repair the onboarding regression before release.\n",
            "Scope boundary: first slice repairs account creation only.\n",
            "Intended outcome: restore account creation and keep audit logs intact.\n",
            "Domain model: onboarding requests create customer accounts and audit entries.\n",
            "API operations: POST /customers should accept a valid onboarding payload.\n",
            "Persistence choice: Postgres remains the source of truth.\n",
            "Auth boundary: authenticated support operators trigger the workflow.\n",
            "Role model semantics: support operators create accounts and auditors inspect history.\n",
            "Success criteria: support operators can create customer accounts while audit history remains intact.\n",
            "Validation target: cargo test onboarding_flow should pass.\n",
        ),
    )
    .unwrap();
    save_local_routing(
        &workspace,
        RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "openai/gpt-5.4".to_string(),
            }),
            ..RoutingConfig::default()
        },
    );

    let _env_guard = EnvVarGuard::set(&[
        (OPENAI_BASE_URL_ENV, "http://127.0.0.1:9"),
        (OPENAI_API_KEY_ENV, "token"),
    ]);

    let runtime = SessionRuntime::for_workspace(&workspace);
    let authored_brief = normalize_inputs_with_governance(
        &workspace,
        Some("Repair the bounded governed onboarding regression"),
        &[PathBuf::from("brief.md")],
        Some(GovernanceIntent {
            requested: true,
            runtime_preference: Some(GovernanceRuntimeKind::Canon),
            risk: Some("medium".to_string()),
            zone: Some("engineering".to_string()),
            owner: Some("platform".to_string()),
            explicit_mode: None,
            explicit_no_canon: false,
        }),
    )
    .unwrap();

    let mut session = build_goal_captured_session(&workspace);
    session.goal = Some(authored_brief.render_goal_text());
    session.authored_brief = Some(authored_brief);
    session.negotiation_packet = None;
    session.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: None,
        selected_mode_sequence: Vec::new(),
        latest_reasoning_profile: None,
        current_stage_index: 0,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    });

    runtime.select_flow(&mut session, "bug-fix").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session
        .governance_lifecycle
        .as_ref()
        .expect("discovery planning should keep governance lifecycle");
    let record =
        lifecycle.stage_records.first().expect("discovery planning should persist a stage record");
    assert_eq!(record.stage_key, "plan:discovery");
    assert_eq!(record.lifecycle_state, GovernanceLifecycleState::Blocked);
    assert!(
        record
            .blocked_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("stage council blocked planning"))
    );

    let council = record
        .stage_council
        .as_ref()
        .expect("blocked discovery planning should persist the stage council outcome");
    assert_eq!(council.status, StageCouncilStatus::Blocked);
    assert!(!council.vote_resolution.independent_review);
    assert!(council.reviewer_findings.is_empty());
    assert!(workspace.join(&council.producer_output.evidence_ref).exists());
    assert!(workspace.join(&council.revised_output.evidence_ref).exists());
    assert_eq!(session.latest_status, SessionStatus::Blocked);
    assert_eq!(
        session.latest_voting.as_ref().map(|state| state.trigger.as_str()),
        Some("stage_council:plan:discovery")
    );
}

#[test]
fn session_runtime_confirms_goal_plan_for_selected_flow_when_context_is_sufficient() {
    let workspace = write_execution_workspace(
        "boundline-runtime-flow-selected-compat",
        vec![success_attempt()],
    );
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = build_goal_captured_session(&workspace);
    session.goal = Some("Fix the failing add test".to_string());

    runtime.select_flow(&mut session, "bug-fix").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    assert_eq!(session.latest_status, SessionStatus::Planned);
    assert_eq!(session.active_flow.as_ref().map(|flow| flow.flow_name.as_str()), Some("bug-fix"));
    assert!(session.active_task.is_none());
    let goal_plan = session.goal_plan.as_ref().expect("goal plan should be persisted");
    assert!(goal_plan.planning_analysis_state().is_some());
}
