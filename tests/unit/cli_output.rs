use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use boundline::FileConfigStore;
use boundline::FileTraceStore;
use boundline::adapters::session_store::SessionStoreError;
use boundline::adapters::trace_store::TraceStore;
use boundline::cli::assistant_assets::{
    AssistantHost, AssistantInstallScope, assets_for_assistants, docs_assets_for_assistants_under,
};
use boundline::cli::diagnostics::{
    DiagnosticsCheck, DiagnosticsReport, DiagnosticsStatus, DiagnosticsSubject,
};
use boundline::cli::inspect::{
    InspectCommandError, TraceResolutionTarget, TraceSummaryError, execute_inspect, render_error,
    render_inspection_routing_summary, resolve_trace_path, summarize_trace,
};
use boundline::cli::output::{
    CommandExitCode, command_name, next_command_after_inspect, next_command_after_run,
    render_cluster_init, render_cluster_inspect, render_cluster_status, render_diagnostics,
    render_goal_plan_flow_state, render_host_command_json, render_inspect_failure,
    render_route_outcome, render_run_trace, render_session_status, render_session_status_brief,
    render_trace_summary, render_trace_summary_brief, validation_error_message,
};
use boundline::cli::session::{
    SessionCommandError, execute_next, execute_status, render_error as render_session_error,
};
use boundline::cli::{
    AssistantSubcommand, CheckpointSubcommand, ClusterSubcommand, ConfigSubcommand,
    WorkflowSubcommand,
};
use boundline::cli::{
    CliValidationError, CommandExitStatus, CommandName, DeveloperCommand, DeveloperCommandSession,
};
use boundline::domain::cluster::{
    ClusterDeliveryStory, ClusterInspectReport, ClusterMemberState, ClusterMemberStatusView,
    ClusterRouteOwner, ClusteredExecutionCondition, ClusteredExecutionKind,
    WorkspaceParticipationKind, WorkspaceParticipationRecord,
};
use boundline::domain::completion_verification::{
    ClaimInferenceConfidence, CompletionClaim, CompletionClaimKind, CompletionClaimSource,
    CompletionRequiredAction, CompletionVerificationFinding, CompletionVerificationFindingKind,
    CompletionVerificationFindingSeverity, CompletionVerificationProjection,
    CompletionVerificationScope, CompletionVerificationState,
};
use boundline::domain::configuration::{
    AdapterConfigValueRecord, AdapterSelectionRecord, AssistantHostKind, ConfigFile,
    InitConfigScope, ModelRoute, PersistedAdapterConfiguration, RoutingConfig, RuntimeKind,
};
use boundline::domain::context_intelligence::{
    AdvancedContextProjection, AuthorityRank, CandidateSelectionState, ContextFidelityTier,
    ContextInclusionMode, ContextOmissionFinding, ContextOmissionSeverity,
    ContextPackEntryProjection, DigestBackedArtifactRef, HybridOutcome, ImpactAnalysisFinding,
    ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus, PatchSafeEditAttempt,
    PatchSafeEditResultState, RelationshipCredibilityState, RelationshipKind,
    RelationshipProjection, RemoteTransmissionPolicyState, RepositoryMapState,
    RetrievalCompatibilityState, RetrievalIndexState, RetrievalMatchOrigin, RetrievalMode,
    RetrievalScore, RetrievalSourceKind, RetrievalStalenessState, RetrievalState,
    RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState, SnapshotCacheState,
};
use boundline::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
    AdapterSelectionMode, AdapterValueKind, AdapterValueSource, FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1,
    StoredAdapterConfigValueState,
};
use boundline::domain::goal_plan::{
    GoalPlanFlowMode, GoalPlanFlowState, PlanningAnalysisCoverage, PlanningAnalysisFinding,
    PlanningAnalysisSeverity, PlanningAnalysisSource, PlanningAnalysisSourceRef,
};
use boundline::domain::governance::GovernanceRuntimeKind;
use boundline::domain::limits::{RunLimits, TerminalCondition};
use boundline::domain::reasoning::{
    ProfileActivationRecord, ReasoningActivationStatus, ReasoningActivationTrigger,
    ReasoningBudget, ReasoningOutcome, ReasoningOutcomeKind, ReasoningProfileId,
};
use boundline::domain::routing_decision::RoutingDecisionProjection;
use boundline::domain::session::{
    CompatibilityFollowUpMode, CompatibilityFollowUpView, ContinuityAuthority,
    DelegationContinuityMode, DelegationStatusView, RoutingMode, RoutingOutcome, RoutingSource,
    SessionStatus, SessionStatusView,
};
use boundline::domain::step::{StepKind, StepStatus};
use boundline::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
use boundline::domain::task_context::TaskContext;
use boundline::domain::trace::{
    ExecutionTrace, TraceEvent, TraceEventType, TraceRecoveryEvent, TraceStepSummary,
    TraceSummaryView,
};
use boundline::fixture::{
    sample_framework_adapter_describe_response, sample_framework_adapter_success_envelope,
};
use serde_json::Map;
use serde_json::json;
use uuid::Uuid;

const OUTPUT_TEST_ADAPTER_COMMAND: &str = "/bin/sh";
const OUTPUT_TEST_ADAPTER_ID: &str = "speckit";
const OUTPUT_TEST_TRANSPORTS: &str = "stdio/json/stdin->stdout";
const OUTPUT_TEST_UNSUPPORTED_TRANSPORTS: &str = "stdio/json/stdout->stdout";
const OUTPUT_TEST_COMPATIBILITY_GATE: &str = "v1_json_over_stdin_stdout_only";

/// Builds one stable advanced-context projection for renderer and inspect tests.
fn sample_advanced_context() -> AdvancedContextProjection {
    AdvancedContextProjection {
        query_id: "query-render".to_string(),
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
            source_ref: "src/context_router.rs".to_string(),
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
            subject_ref: "src/context_router.rs".to_string(),
            relationship_kind: RelationshipKind::ExercisesTest,
            credibility_state: RelationshipCredibilityState::Credible,
            explanation: "the matching test file names the same target".to_string(),
            supporting_candidate_ids: vec!["candidate-1".to_string()],
        }],
        impact_findings: vec![ImpactAnalysisFinding {
            finding_id: "finding-1".to_string(),
            finding_kind: ImpactFindingKind::MissingTest,
            subject_ref: "tests/context_router.rs".to_string(),
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

fn configured_adapter_workspace(
    prefix: &str,
    describe_document: serde_json::Value,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace)?;
    let script_path = write_describe_only_adapter_script(&workspace, describe_document)?;
    FileConfigStore::for_workspace(&workspace).save_local(&ConfigFile {
        adapter: Some(PersistedAdapterConfiguration {
            selection: AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::KnownProfile,
                adapter_id: OUTPUT_TEST_ADAPTER_ID.to_string(),
                display_name: "Speckit".to_string(),
                command: OUTPUT_TEST_ADAPTER_COMMAND.to_string(),
                args: vec![script_path.to_string_lossy().into_owned()],
                registration_source: AdapterRegistrationSource::AdapterAdd,
                discovery_state: AdapterDiscoveryState::ExplicitCommand,
                compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                updated_at: 42,
            },
            schema_fingerprint: format!(
                "{}:{}:template_repo",
                FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, OUTPUT_TEST_ADAPTER_ID
            ),
            completeness_state: AdapterConfigCompletenessState::Complete,
            interactive_resolution: false,
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
                value_source: AdapterValueSource::KnownProfileDefault,
                resolution_state: StoredAdapterConfigValueState::Present,
            }],
        }),
        ..ConfigFile::default()
    })?;
    Ok(workspace)
}

fn write_describe_only_adapter_script(
    workspace: &std::path::Path,
    describe_document: serde_json::Value,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let describe_json =
        serde_json::to_string(&sample_framework_adapter_success_envelope(describe_document))?;
    let script_path = workspace.join("adapter-describe-only.sh");
    let script = format!(
        "#!/bin/sh\nset -eu\ncase \"$1\" in\n  describe)\n    cat <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );
    fs::write(&script_path, script)?;
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(script_path)
}

#[test]
fn exit_codes_match_the_command_contract() {
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::Succeeded).code(), 0);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::NonSuccess).code(), 1);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::InvalidInvocation).code(), 2);
    assert_eq!(CommandExitCode::for_status(CommandExitStatus::TraceReadFailure).code(), 3);
}

#[test]
fn command_names_render_from_subcommands() {
    let command = DeveloperCommand::Flow {
        name: "bug-fix".to_string(),
        workspace: Some(PathBuf::from("/tmp/workspace")),
        cluster: None,
    };
    assert_eq!(command_name(&command), "flow");
    assert_eq!(command.name(), CommandName::Flow);

    let command = DeveloperCommand::Run {
        workspace: Some(PathBuf::from("/tmp/workspace")),
        cluster: None,
        goal: None,
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
    };
    assert_eq!(command_name(&command), "run");
    assert_eq!(command.name(), CommandName::Run);
}

#[test]
fn render_host_command_json_surfaces_framework_adapter_built_in_default_projection() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-host-adapter-json-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let view = SessionStatusView {
        session_id: "session-host-adapter".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        latest_status: SessionStatus::Initialized,
        explanation: "host envelope should disclose built-in adapter execution".to_string(),
        ..SessionStatusView::default()
    };

    let rendered = render_host_command_json(
        "status",
        CommandExitStatus::Succeeded,
        "rendered",
        None,
        Some(&view),
        None,
    );

    assert!(rendered.contains("\"framework_adapter_status\": \"built_in_default\""), "{rendered}");
    assert!(
        rendered.contains("\"framework_adapter_execution_source\": \"built_in\""),
        "{rendered}"
    );
}

#[test]
fn render_host_command_json_surfaces_configured_adapter_supported_transports()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = configured_adapter_workspace(
        "boundline-host-adapter-configured",
        serde_json::to_value(sample_framework_adapter_describe_response())?,
    )?;

    let view = SessionStatusView {
        session_id: "session-host-adapter-configured".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        latest_status: SessionStatus::Initialized,
        explanation: "host envelope should disclose validated adapter transport support"
            .to_string(),
        ..SessionStatusView::default()
    };

    let rendered = render_host_command_json(
        "status",
        CommandExitStatus::Succeeded,
        "rendered",
        None,
        Some(&view),
        None,
    );

    assert!(rendered.contains("\"framework_adapter_status\": \"configured\""), "{rendered}");
    assert!(rendered.contains("\"framework_adapter_execution_source\": \"adapter\""), "{rendered}");
    assert!(rendered.contains("\"framework_adapter_config_state\": \"complete\""), "{rendered}");
    assert!(rendered.contains("\"framework_adapter_interactive_resolution\": false"), "{rendered}");
    assert!(rendered.contains("\"framework_adapter_value_count\": 1"), "{rendered}");
    assert!(
        rendered.contains(&format!(
            "\"framework_adapter_supported_transports\": \"{OUTPUT_TEST_TRANSPORTS}\""
        )),
        "{rendered}"
    );
    assert!(
        rendered.contains(&format!(
            "\"framework_adapter_compatibility_gate\": \"{OUTPUT_TEST_COMPATIBILITY_GATE}\""
        )),
        "{rendered}"
    );

    Ok(())
}

#[test]
fn run_session_requires_a_non_empty_goal() {
    let command = DeveloperCommand::Run {
        workspace: Some(PathBuf::from("/tmp/workspace")),
        cluster: None,
        goal: Some("   ".to_string()),
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
    };
    let session = DeveloperCommandSession::from_command(&command);

    assert_eq!(session.validate(), Err(CliValidationError::MissingGoal(CommandName::Run)));
}

#[test]
fn run_without_legacy_flags_is_valid_for_session_native_execution() {
    let command = DeveloperCommand::Run {
        workspace: None,
        cluster: None,
        goal: None,
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
    };
    let session = DeveloperCommandSession::from_command(&command);

    assert_eq!(session.validate(), Ok(()));
}

#[test]
fn inspect_session_requires_trace_or_workspace() {
    let session = DeveloperCommandSession {
        command_name: CommandName::Inspect,
        workspace_ref: None,
        requires_workspace_ref: false,
        install_check: false,
        goal: None,
        trace_ref: None,
        started_at: 0,
        completed_at: None,
        exit_status: None,
        trace_location: None,
    };

    assert_eq!(session.validate(), Err(CliValidationError::MissingTraceSelection));
    assert_eq!(
        validation_error_message(&CliValidationError::MissingTraceSelection),
        "inspect requires --trace or --workspace"
    );
}

#[test]
fn diagnostics_renderer_lists_check_names_and_actions() {
    let report = DiagnosticsReport {
        subject: DiagnosticsSubject::Workspace,
        workspace_ref: Some("/tmp/workspace".to_string()),
        installation_ref: None,
        checks: vec![
            DiagnosticsCheck {
                name: "workspace_exists".to_string(),
                status: DiagnosticsStatus::Passed,
                message: "workspace exists".to_string(),
            },
            DiagnosticsCheck {
                name: "trace_store".to_string(),
                status: DiagnosticsStatus::Failed,
                message: "fix the trace directory".to_string(),
            },
        ],
        ready: false,
        missing_prerequisites: vec!["trace_store".to_string()],
        suggested_actions: vec!["fix the trace directory".to_string()],
        boundline_version: None,
        supported_canon_version: None,
        companion_state: None,
        channel_candidates: Vec::new(),
    };

    let rendered = render_diagnostics(&report);

    assert!(rendered.contains("doctor: not ready"));
    assert!(rendered.contains("summary:"));
    assert!(rendered.contains("checks:"));
    assert!(rendered.contains("workspace_exists"));
    assert!(rendered.contains("trace_store"));
    assert!(rendered.contains("actions:"));
    assert!(rendered.contains("fix the trace directory"));
}

#[test]
fn next_command_helpers_match_assistant_routing_expectations() {
    assert_eq!(next_command_after_run(TaskStatus::Succeeded), "/boundline-status");
    assert_eq!(next_command_after_run(TaskStatus::Failed), "/boundline-next");
    assert_eq!(next_command_after_inspect(TaskStatus::Succeeded), "/boundline-next");
}

#[test]
fn inspect_failure_renderer_exposes_correction_cues() {
    let rendered = render_inspect_failure(
        "explicit-trace",
        Some("/tmp/missing-trace.json"),
        None,
        "failed to read the requested trace",
        "boundline inspect --trace <trace>",
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: explicit-trace"));
    assert!(rendered.contains("trace: /tmp/missing-trace.json"));
    assert!(rendered.contains("next_command: /boundline-inspect"));
    assert!(rendered.contains("corrected_command: boundline inspect --trace <trace>"));
}

#[test]
fn route_and_flow_render_helpers_expose_foundational_runtime_cues() {
    let route = RoutingOutcome {
        mode: RoutingMode::Blocked,
        source: RoutingSource::GoalPlan,
        reason: "plan confirmation is still pending".to_string(),
    };
    let flow_state = GoalPlanFlowState {
        mode: GoalPlanFlowMode::Proposed,
        flow_name: Some("bug-fix".to_string()),
        confidence_reason: Some("goal contains keyword 'fix'".to_string()),
    };

    assert_eq!(
        render_route_outcome(&route),
        "routing: blocked (goal_plan) - plan confirmation is still pending"
    );
    assert_eq!(
        render_goal_plan_flow_state(&flow_state),
        "flow_state: proposed (bug-fix) - goal contains keyword 'fix'"
    );

    let summary = render_inspection_routing_summary(&route, Some(&flow_state));
    assert_eq!(summary[0], "routing: blocked (goal_plan) - plan confirmation is still pending");
    assert_eq!(summary[1], "flow_state: proposed (bug-fix) - goal contains keyword 'fix'");
}

#[test]
fn inspect_invalid_session_errors_reuse_session_guidance() {
    let rendered = render_error(
        None,
        Some(std::path::Path::new("/tmp/workspace")),
        None,
        &InspectCommandError::InvalidSession(
            "active session is invalid: workspace_ref must not be empty".to_string(),
        ),
    );

    assert!(rendered.contains("inspect: session error"), "{rendered}");
    assert!(rendered.contains("reason: active session is invalid:"), "{rendered}");
    assert!(rendered.contains("next_command: boundline goal --goal <goal>"), "{rendered}");
}

#[test]
fn trace_summary_renderer_mentions_steps_recovery_and_terminal_reason() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/workspace/.boundline/traces/task.json".to_string(),
        goal: "Inspect a recorded run".to_string(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        routing_summary: None,
        routing_projection: RoutingDecisionProjection::default(),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: Vec::new(),
        failure_evidence: Vec::new(),
        adaptive_evidence: Vec::new(),
        executed_steps: vec![
            TraceStepSummary {
                step_id: "analyze".to_string(),
                step_kind: StepKind::Agent,
                attempts: 1,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 1 attempt(s)".to_string(),
            },
            TraceStepSummary {
                step_id: "code".to_string(),
                step_kind: StepKind::Agent,
                attempts: 2,
                final_status: StepStatus::Succeeded,
                headline: "succeeded after 2 attempt(s)".to_string(),
            },
        ],
        recovery_events: vec![TraceRecoveryEvent {
            event_type: TraceEventType::RetryScheduled,
            trigger: "retrying step code within remaining retry budget".to_string(),
            related_step_id: Some("code".to_string()),
        }],
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied after step verify",
            None,
        ),
        duration: Some(42),
        ..Default::default()
    };

    let rendered = render_trace_summary(
        &summary,
        "explicit-trace",
        next_command_after_inspect(summary.terminal_status),
    );

    assert!(rendered.contains("inspection_target: explicit-trace"));
    assert!(rendered.contains("trace: /tmp/workspace/.boundline/traces/task.json"));
    assert!(
        rendered.contains("execution_condition: terminal - goal satisfied after step verify"),
        "{rendered}"
    );
    assert!(rendered.contains("step analyze (agent) succeeded [1 attempt(s)]"));
    assert!(rendered.contains("step code (agent) succeeded [2 attempt(s)]"));
    assert!(rendered.contains("retry: retrying step code within remaining retry budget"));
    assert!(rendered.contains("terminal_reason: goal satisfied after step verify"));
    assert!(rendered.contains("next_command: /boundline-next"));
    assert!(rendered.contains("duration_ms: 42"));
}

#[test]
fn trace_summary_renderer_surfaces_why_and_risk_summaries() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/workspace/.boundline/traces/assistant-delight.json".to_string(),
        goal: "Explain the active delivery state".to_string(),
        goal_plan_summary: Some(
            "explain the bounded plan from authoritative runtime state".to_string(),
        ),
        failure_evidence: vec!["validation still has not run for the current attempt".to_string()],
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied after trace inspection",
            None,
        ),
        ..Default::default()
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(
        rendered.contains("why_summary: explain the bounded plan from authoritative runtime state"),
        "{rendered}"
    );
    assert!(
        rendered.contains("risk_summary: validation still has not run for the current attempt"),
        "{rendered}"
    );
}

#[test]
fn trace_summary_brief_surfaces_plan_quality_projection() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/workspace/.boundline/traces/task-plan-quality.json".to_string(),
        goal: "Inspect a plan-quality trace".to_string(),
        plan_quality_state: Some("clarification_required".to_string()),
        plan_quality_findings: vec![
            "planning_rationale".to_string(),
            "verification_strategy".to_string(),
        ],
        plan_quality_assumptions: vec![
            "no explicit route override is required for this plan".to_string(),
        ],
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(
            TerminalCondition::GoalSatisfied,
            "goal satisfied after trace inspection",
            None,
        ),
        ..Default::default()
    };

    let rendered = render_trace_summary_brief(&summary, Some("explicit-trace"), "/boundline-next");

    assert!(rendered.contains("plan_quality_state: clarification_required"), "{rendered}");
    assert!(
        rendered.contains("plan_quality_findings: planning_rationale, verification_strategy"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "plan_quality_assumptions: no explicit route override is required for this plan"
        ),
        "{rendered}"
    );
}

#[test]
fn session_status_renderer_surfaces_cognitive_lenses() {
    let view = SessionStatusView {
        session_id: "session-cognitive-lenses".to_string(),
        workspace_ref: "/tmp/session-cognitive-lenses".to_string(),
        goal: Some("Plan with bounded context".to_string()),
        advanced_context: Some(sample_advanced_context()),
        active_flow: Some("bug-fix".to_string()),
        flow_state: Some("implement".to_string()),
        planning_rationale: Some(
            "bounded runtime evidence points to the context router".to_string(),
        ),
        verification_strategy: Some(
            "run focused regression checks before applying the route".to_string(),
        ),
        latest_status: SessionStatus::Planned,
        latest_governance_runtime: Some("canon".to_string()),
        latest_governance_packet_ref: Some(".canon/runs/canon-run-security".to_string()),
        next_command: Some("boundline inspect".to_string()),
        explanation: "session is ready to execute the bounded plan".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("assumptions_summary: validation(1)"), "{rendered}");
    assert!(
        rendered.contains(
            "assumption_group: validation -> src/context_router.rs [explicit] source=workspace risk=low the matching test file names the same target"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("hidden_impact_summary: missing_tests(1)"), "{rendered}");
    assert!(
        rendered.contains(
            "hidden_impact_missing_tests: tests/context_router.rs [open/medium] add or refresh the focused regression test"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "challenge_strongest_objection: missing test coverage is still open for tests/context_router.rs"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "challenge_required_review: governance packet .canon/runs/canon-run-security remains authoritative"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("challenge_council_required: yes"), "{rendered}");
    assert!(
        rendered.contains(
            "explain_plan_summary: goal=Plan with bounded context; stages=bug-fix/implement"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "explain_plan_validation: run focused regression checks before applying the route"
        ),
        "{rendered}"
    );
}

#[test]
fn inspect_failure_renderer_includes_workspace_ref_when_provided() {
    let rendered = render_inspect_failure(
        "latest-workspace-trace",
        None,
        Some("/tmp/my-workspace"),
        "failed to read the requested trace",
        "boundline inspect --workspace <workspace>",
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: latest-workspace-trace"));
    assert!(rendered.contains("workspace_ref: /tmp/my-workspace"));
    assert!(rendered.contains("next_command: /boundline-inspect"));
    assert!(rendered.contains("corrected_command: boundline inspect --workspace <workspace>"));
}

#[test]
fn render_error_with_missing_trace_reference_uses_explicit_trace_correction() {
    let rendered = render_error(None, None, None, &InspectCommandError::MissingTraceReference);

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("terminal_reason: inspect requires --trace or --workspace"));
    assert!(rendered.contains("next_command: /boundline-inspect"));
    assert!(rendered.contains("corrected_command: boundline inspect --trace"));
}

#[test]
fn render_error_with_workspace_path_uses_workspace_correction_cues() {
    let rendered = render_error(
        None,
        Some(std::path::Path::new("/tmp/my-workspace")),
        None,
        &InspectCommandError::MissingLatestTrace,
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("inspection_target: latest-workspace-trace"));
    assert!(rendered.contains("terminal_reason: failed to read the requested trace"));
    assert!(rendered.contains("workspace_ref: /tmp/my-workspace"));
    assert!(rendered.contains("next_command: /boundline-inspect"));
    assert!(rendered.contains("corrected_command: boundline inspect --workspace <workspace>"));
}

#[test]
fn render_error_with_summary_failure_uses_summary_terminal_reason() {
    let rendered = render_error(
        Some(std::path::Path::new("/tmp/trace.json")),
        None,
        None,
        &InspectCommandError::Summary(TraceSummaryError::MissingTerminalStatus),
    );

    assert!(rendered.contains("inspect: trace read failure"));
    assert!(rendered.contains("terminal_reason: failed to summarize the requested trace"));
    assert!(rendered.contains("next_command: /boundline-inspect"));
}

fn minimal_trace(task_id: &str) -> ExecutionTrace {
    trace_with_terminal(
        task_id,
        "session-unit",
        "Unit test goal",
        TaskStatus::Succeeded,
        TerminalCondition::GoalSatisfied,
        "goal satisfied in unit test",
    )
}

fn trace_with_terminal(
    task_id: &str,
    session_id: &str,
    goal: &str,
    status: TaskStatus,
    condition: TerminalCondition,
    reason_msg: &str,
) -> ExecutionTrace {
    let mut trace = ExecutionTrace::new(task_id, session_id, goal);
    trace.terminal_status = Some(status);
    trace.terminal_reason = Some(TerminalReason::new(condition, reason_msg, None));
    trace
}

fn succeeded_trace(task_id: &str, goal: &str, reason_msg: &str) -> ExecutionTrace {
    trace_with_terminal(
        task_id,
        "session",
        goal,
        TaskStatus::Succeeded,
        TerminalCondition::GoalSatisfied,
        reason_msg,
    )
}

fn minimal_response(status: TaskStatus, reason_msg: &str) -> TaskRunResponse {
    TaskRunResponse {
        task_id: "task-unit".to_string(),
        terminal_status: status,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, reason_msg, None),
        final_context: TaskContext::new(
            "session-unit",
            "/tmp/workspace",
            RunLimits::default(),
            Map::new(),
        ),
        plan_revision: 1,
        trace_location: "/tmp/.boundline/traces/task-unit.json".to_string(),
    }
}

#[test]
fn render_run_trace_includes_next_command_and_trace_fields() {
    let response = minimal_response(TaskStatus::Succeeded, "goal satisfied");
    let rendered = render_run_trace("run", None, &response, "/boundline-status");

    assert!(rendered.contains("execution_condition: terminal - goal satisfied"), "{rendered}");
    assert!(rendered.contains("next_command: /boundline-status"), "{rendered}");
    assert!(rendered.contains("terminal_status: succeeded"), "{rendered}");
    assert!(rendered.contains("trace: /tmp/.boundline/traces/task-unit.json"), "{rendered}");
}

#[test]
fn render_run_trace_with_trace_events_includes_retry_and_replan_lines() {
    let mut trace = succeeded_trace("task-events", "Goal with events", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::RetryScheduled,
        step_id: Some("analyze".to_string()),
        plan_revision: 0,
        payload: json!({"reason": "transient error, retrying"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::Replanned,
        step_id: Some("analyze".to_string()),
        plan_revision: 1,
        payload: json!({"reason": "goal shifted, replanning"}),
        recorded_at: 1,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(rendered.contains("retry for analyze: transient error, retrying"), "{rendered}");
    assert!(rendered.contains("replan after analyze: goal shifted, replanning"), "{rendered}");
    assert!(rendered.contains("next_command: /boundline-status"), "{rendered}");
}

#[test]
fn render_run_trace_surfaces_completion_verification_projection() {
    let mut response = minimal_response(TaskStatus::Running, "proof needs rerun");
    let projection = CompletionVerificationProjection {
        completion_verification_state: CompletionVerificationState::ProofRequired,
        scope: CompletionVerificationScope::Task,
        claim: Some(CompletionClaim {
            claim_id: "claim-run-trace".to_string(),
            kind: CompletionClaimKind::BugFixed,
            scope: CompletionVerificationScope::Task,
            source: CompletionClaimSource::RuntimeInference,
            confidence: Some(ClaimInferenceConfidence::High),
            summary: "bug fix remains unproven".to_string(),
            supporting_signals: vec!["goal_text".to_string(), "changed_files".to_string()],
        }),
        completion_blocked_claims: vec![CompletionClaimKind::BugFixed],
        completion_evidence_refs: vec!["proof-1".to_string()],
        completion_verification_findings: vec![CompletionVerificationFinding {
            kind: CompletionVerificationFindingKind::StaleProof,
            severity: CompletionVerificationFindingSeverity::Blocking,
            message: "The previously passing proof is stale because workspace content changed after proof execution.".to_string(),
            proof_ref: Some("proof-20260612-abc123".to_string()),
            task_id: Some("task-unit".to_string()),
            changed_paths: vec!["src/lib.rs".to_string(), "Cargo.toml".to_string()],
            required_action: CompletionRequiredAction::RerunProof,
        }],
        child_summary: None,
    };
    response
        .final_context
        .set_completion_verification_projection(&projection)
        .expect("completion verification projection should attach");

    let rendered = render_run_trace("run", None, &response, "/boundline-status");

    assert!(rendered.contains("completion_verification_state: proof_required"), "{rendered}");
    assert!(rendered.contains("completion_claim_kind: bug_fixed"), "{rendered}");
    assert!(rendered.contains("completion_claim_source: runtime_inference"), "{rendered}");
    assert!(rendered.contains("completion_blocked_claims: bug_fixed"), "{rendered}");
    assert!(rendered.contains("completion_evidence_refs: proof-1"), "{rendered}");
    assert!(
        rendered.contains("completion_verification_changed_paths: src/lib.rs, Cargo.toml"),
        "{rendered}"
    );
    assert!(
        rendered.contains("completion_verification_required_action: rerun_proof"),
        "{rendered}"
    );
}

#[test]
fn render_run_trace_surfaces_goal_plan_negotiation_projection() {
    let mut trace = succeeded_trace("task-negotiation", "Goal with negotiation", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "plan_id": "plan-1",
            "goal": "Goal with negotiation",
            "task_count": 2,
            "negotiation_goal_summary": "Stabilize the failing add flow",
            "negotiation_resolution": "credible",
            "negotiation_acceptance_boundary": "deliver the bounded outcome: Stabilize the failing add flow"
        }),
        recorded_at: 0,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(
        rendered.contains("negotiation_goal_summary: Stabilize the failing add flow"),
        "{rendered}"
    );
    assert!(rendered.contains("negotiation_resolution: credible"), "{rendered}");
    assert!(
        rendered.contains(
            "negotiation_acceptance_boundary: deliver the bounded outcome: Stabilize the failing add flow"
        ),
        "{rendered}"
    );
}

#[test]
fn render_run_trace_surfaces_plan_quality_projection() {
    let mut trace = succeeded_trace("task-plan-quality", "Plan quality goal", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "plan_id": "plan-1",
            "goal": "Plan quality goal",
            "task_count": 1,
            "plan_quality_state": "clarification_required",
            "plan_quality_findings": ["planning_rationale", "verification_strategy"],
            "plan_quality_assumptions": ["no explicit route override is required for this plan"]
        }),
        recorded_at: 0,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(rendered.contains("plan_quality_state: clarification_required"), "{rendered}");
    assert!(
        rendered.contains("plan_quality_findings: planning_rationale, verification_strategy"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "plan_quality_assumptions: no explicit route override is required for this plan"
        ),
        "{rendered}"
    );
}

#[test]
fn render_run_trace_surfaces_task_started_negotiation_projection() {
    let mut trace = succeeded_trace("task-negotiation-compat", "Compat goal", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::TaskStarted,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "goal": "Compat goal",
            "input": {
                "negotiation_goal_summary": "Stabilize the failing add flow",
                "negotiation_resolution": "credible",
                "negotiation_acceptance_boundary": "deliver the bounded outcome: Stabilize the failing add flow"
            },
            "limits": RunLimits::default()
        }),
        recorded_at: 0,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(
        rendered.contains("negotiation_goal_summary: Stabilize the failing add flow"),
        "{rendered}"
    );
    assert!(rendered.contains("negotiation_resolution: credible"), "{rendered}");
    assert!(
        rendered.contains(
            "negotiation_acceptance_boundary: deliver the bounded outcome: Stabilize the failing add flow"
        ),
        "{rendered}"
    );
}

#[test]
fn render_run_trace_surfaces_context_projection() {
    let mut trace = succeeded_trace("task-context", "Context goal", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "plan_id": "plan-context",
            "goal": "Context goal",
            "task_count": 1,
            "context_summary": "bounded context from 2 primary input(s)",
            "context_credibility": "credible",
            "context_primary_inputs": ["src/context_router.rs", "src/lib.rs"],
            "context_provenance": [
                "workspace_file: src/context_router.rs (selected as a bounded workspace target for the current goal) [source=workspace_scan]",
                "recent_trace: .boundline/traces/last.json (reuses the latest persisted trace as bounded historical evidence) [source=latest_trace_ref]"
            ]
        }),
        recorded_at: 0,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(rendered.contains("context_summary: bounded context from 2 primary input(s)"));
    assert!(rendered.contains("context_credibility: credible"));
    assert!(rendered.contains("context_primary_inputs: src/context_router.rs, src/lib.rs"));
    assert!(rendered.contains("context_provenance: workspace_file: src/context_router.rs"));
}

#[test]
fn render_run_trace_surfaces_security_assessment_packet_provenance() {
    let mut trace = succeeded_trace("task-governance", "Governed goal", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GovernanceStarted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "bug-fix:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "canon_mode": "security-assessment",
            "run_ref": "canon-run-security",
            "packet_source_stage": "bug-fix:implement",
            "packet_binding_reason": "upstream_stage_context"
        }),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::GovernanceCompleted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "bug-fix:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "headline": "security assessment packet ready",
            "packet_ref": ".canon/runs/canon-run-security",
            "packet_source_stage": "bug-fix:implement",
            "packet_binding_reason": "upstream_stage_context"
        }),
        recorded_at: 1,
    });

    let response = minimal_response(TaskStatus::Succeeded, "done");
    let rendered = render_run_trace("run", Some(&trace), &response, "/boundline-status");

    assert!(
        rendered.contains(
            "governance_started: bug-fix:verify (security-assessment) [canon-run-security] from bug-fix:implement (upstream_stage_context)"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "governance_completed: security assessment packet ready [.canon/runs/canon-run-security] from bug-fix:implement (upstream_stage_context)"
        ),
        "{rendered}"
    );
}

#[test]
fn execute_inspect_explicit_trace_covers_inspection_target_and_next_command() {
    use std::fs;

    let dir = std::env::temp_dir().join(format!("boundline-unit-inspect-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();

    let trace = minimal_trace("task-explicit");
    let store = FileTraceStore::new(&dir);
    let trace_path = store.persist(&trace).unwrap();

    let report = execute_inspect(Some(&trace_path), None, None, false).unwrap();
    let output = &report.terminal_output;

    assert!(output.contains("inspection_target: explicit-trace"), "{output}");
    assert!(output.contains("next_command: /boundline-next"), "{output}");
}

#[test]
fn execute_inspect_workspace_covers_latest_workspace_trace_target() {
    use std::fs;

    let workspace = std::env::temp_dir().join(format!("boundline-unit-ws-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let trace = minimal_trace("task-workspace");
    let store = FileTraceStore::for_workspace(&workspace);
    store.persist(&trace).unwrap();

    let report = execute_inspect(None, Some(&workspace), None, false).unwrap();
    let output = &report.terminal_output;

    assert!(output.contains("inspection_target: latest-workspace-trace"), "{output}");
    assert!(output.contains("next_command: /boundline-next"), "{output}");
}

#[test]
fn execute_inspect_surfaces_goal_plan_negotiation_projection() {
    use std::fs;

    let dir =
        std::env::temp_dir().join(format!("boundline-unit-inspect-negotiation-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();

    let mut trace = minimal_trace("task-negotiation-inspect");
    trace.goal = "Goal with negotiation".to_string();
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "plan_id": "plan-1",
            "goal": "Goal with negotiation",
            "task_count": 2,
            "negotiation_goal_summary": "Stabilize the failing add flow",
            "negotiation_resolution": "credible",
            "negotiation_acceptance_boundary": "deliver the bounded outcome: Stabilize the failing add flow"
        }),
        recorded_at: 0,
    });

    let store = FileTraceStore::new(&dir);
    let trace_path = store.persist(&trace).unwrap();

    let report = execute_inspect(Some(&trace_path), None, None, false).unwrap();
    let output = &report.terminal_output;

    assert!(
        output.contains("negotiation_goal_summary: Stabilize the failing add flow"),
        "{output}"
    );
    assert!(output.contains("negotiation_resolution: credible"), "{output}");
    assert!(
        output.contains(
            "negotiation_acceptance_boundary: deliver the bounded outcome: Stabilize the failing add flow"
        ),
        "{output}"
    );
}

#[test]
fn execute_inspect_surfaces_context_projection() {
    use std::fs;

    let dir =
        std::env::temp_dir().join(format!("boundline-unit-inspect-context-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();

    let mut trace = minimal_trace("task-context-inspect");
    trace.goal = "Goal with context".to_string();
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "plan_id": "plan-context",
            "goal": "Goal with context",
            "task_count": 1,
            "context_summary": "bounded context from 1 primary input(s)",
            "context_credibility": "credible",
            "context_primary_inputs": ["src/context_router.rs"],
            "context_provenance": [
                "workspace_file: src/context_router.rs (selected as a bounded workspace target for the current goal) [source=workspace_scan]"
            ]
        }),
        recorded_at: 0,
    });

    let store = FileTraceStore::new(&dir);
    let trace_path = store.persist(&trace).unwrap();

    let report = execute_inspect(Some(&trace_path), None, None, false).unwrap();
    let output = &report.terminal_output;

    assert!(output.contains("context_summary: bounded context from 1 primary input(s)"));
    assert!(output.contains("context_credibility: credible"));
    assert!(output.contains("context_primary_inputs: src/context_router.rs"));
    assert!(output.contains("context_provenance: workspace_file: src/context_router.rs"));
}

#[test]
fn execute_inspect_surfaces_task_started_negotiation_projection() {
    use std::fs;

    let dir = std::env::temp_dir()
        .join(format!("boundline-unit-inspect-negotiation-compat-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();

    let mut trace = minimal_trace("task-negotiation-inspect-compat");
    trace.goal = "Compat goal".to_string();
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::TaskStarted,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "goal": "Compat goal",
            "input": {
                "negotiation_goal_summary": "Stabilize the failing add flow",
                "negotiation_resolution": "credible",
                "negotiation_acceptance_boundary": "deliver the bounded outcome: Stabilize the failing add flow"
            },
            "limits": RunLimits::default()
        }),
        recorded_at: 0,
    });

    let store = FileTraceStore::new(&dir);
    let trace_path = store.persist(&trace).unwrap();

    let report = execute_inspect(Some(&trace_path), None, None, false).unwrap();
    let output = &report.terminal_output;

    assert!(
        output.contains("negotiation_goal_summary: Stabilize the failing add flow"),
        "{output}"
    );
    assert!(output.contains("negotiation_resolution: credible"), "{output}");
    assert!(
        output.contains(
            "negotiation_acceptance_boundary: deliver the bounded outcome: Stabilize the failing add flow"
        ),
        "{output}"
    );
}

#[test]
fn summarize_trace_handles_tool_and_decision_step_kinds() {
    let mut trace = succeeded_trace("task-steps", "Steps test", "all steps done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("fetch".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "tool"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("fetch".to_string()),
        plan_revision: 0,
        payload: json!({"status": "succeeded"}),
        recorded_at: 1,
    });
    trace.events.push(TraceEvent {
        event_id: "e3".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("decide".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "decision"}),
        recorded_at: 2,
    });
    trace.events.push(TraceEvent {
        event_id: "e4".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("decide".to_string()),
        plan_revision: 0,
        payload: json!({"status": "failed"}),
        recorded_at: 3,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let fetch = summary.executed_steps.iter().find(|s| s.step_id == "fetch").unwrap();
    let decide = summary.executed_steps.iter().find(|s| s.step_id == "decide").unwrap();

    assert_eq!(fetch.step_kind, StepKind::Tool);
    assert_eq!(decide.step_kind, StepKind::Decision);
    assert_eq!(decide.final_status, StepStatus::Failed);
}

#[test]
fn summarize_trace_with_unknown_step_status_yields_running_final_status_and_completed_headline() {
    let mut trace = succeeded_trace("task-unk", "Unknown status test", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "agent"}),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::StepCompleted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"status": "unknown_status"}),
        recorded_at: 1,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let step = &summary.executed_steps[0];

    assert_eq!(step.final_status, StepStatus::Running);
    assert_eq!(step.headline, "completed");
}

#[test]
fn render_session_status_includes_goal_trace_and_next_command() {
    let view = SessionStatusView {
        session_id: "session-status".to_string(),
        workspace_ref: "/tmp/session-workspace".to_string(),
        goal: Some("Ship a bounded change".to_string()),
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
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: Some(2),
        current_step_id: Some("verify".to_string()),
        current_step_index: Some(1),
        latest_status: SessionStatus::Running,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/session-workspace/.boundline/traces/task.json".to_string()),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
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
        next_command: Some("boundline next".to_string()),
        explanation: "the active session can keep executing from the current step".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("session_id: session-status"), "{rendered}");
    assert!(rendered.contains("goal: Ship a bounded change"), "{rendered}");
    assert!(rendered.contains("latest_status: running"), "{rendered}");
    assert!(rendered.contains("execution_path: native_goal_plan"), "{rendered}");
    assert!(
        rendered.contains("latest_trace_ref: /tmp/session-workspace/.boundline/traces/task.json"),
        "{rendered}"
    );
    assert!(rendered.contains("next_command: boundline next"), "{rendered}");
}

#[test]
fn render_session_status_surfaces_framework_adapter_built_in_default() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-framework-adapter-status-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-status-adapter".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        latest_status: SessionStatus::GoalCaptured,
        explanation: "captured the goal and preserved built-in execution".to_string(),
        ..Default::default()
    });

    assert!(rendered.contains("framework_adapter_status: built_in_default"), "{rendered}");
    assert!(rendered.contains("framework_adapter_execution_source: built_in"), "{rendered}");
}

#[test]
fn render_session_status_surfaces_blocked_unsupported_transport_adapter()
-> Result<(), Box<dyn std::error::Error>> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["supported_transports"] = json!([
        {
            "transport": "stdio",
            "encoding": "json",
            "request_channel": "stdout",
            "response_channel": "stdout"
        }
    ]);
    let workspace =
        configured_adapter_workspace("boundline-framework-adapter-unsupported-status", describe)?;

    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-status-adapter-unsupported".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        latest_status: SessionStatus::GoalCaptured,
        explanation: "session status should disclose unsupported adapter transports".to_string(),
        ..Default::default()
    });

    assert!(rendered.contains("framework_adapter_status: blocked"), "{rendered}");
    assert!(rendered.contains("framework_adapter_execution_source: built_in"), "{rendered}");
    assert!(rendered.contains("framework_adapter_config_state: complete"), "{rendered}");
    assert!(rendered.contains("framework_adapter_interactive_resolution: false"), "{rendered}");
    assert!(rendered.contains("framework_adapter_value_count: 1"), "{rendered}");
    assert!(
        rendered.contains(&format!(
            "framework_adapter_supported_transports: {OUTPUT_TEST_UNSUPPORTED_TRANSPORTS}"
        )),
        "{rendered}"
    );
    assert!(
        rendered.contains(&format!(
            "framework_adapter_compatibility_gate: {OUTPUT_TEST_COMPATIBILITY_GATE}"
        )),
        "{rendered}"
    );
    assert!(
        rendered.contains("framework_adapter_blocked_reason: unsupported_transport"),
        "{rendered}"
    );

    Ok(())
}

#[test]
fn render_session_status_surfaces_security_assessment_projection() {
    let view = SessionStatusView {
        session_id: "session-governed".to_string(),
        workspace_ref: "/tmp/session-workspace".to_string(),
        goal: Some("Verify a governed change".to_string()),
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
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: Some("bug-fix".to_string()),
        flow_state: Some("confirmed".to_string()),
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: Some("verify".to_string()),
        current_stage_index: Some(3),
        total_stages: Some(4),
        plan_revision: Some(2),
        current_step_id: Some("verify".to_string()),
        current_step_index: Some(2),
        latest_status: SessionStatus::Running,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/session-workspace/.boundline/traces/task.json".to_string()),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
        latest_exhaustion_reason: None,
        latest_review_trigger: None,
        latest_review_vote: None,
        latest_review_outcome: None,
        latest_review_headline: None,
        latest_governance_stage: Some("bug-fix:verify".to_string()),
        latest_governance_runtime: Some("canon".to_string()),
        latest_governance_mode: Some("security-assessment".to_string()),
        latest_governance_run_ref: Some("canon-run-security".to_string()),
        latest_governance_state: Some("governed_ready".to_string()),
        latest_governance_blocked_reason: None,
        latest_governance_packet_ref: Some(".canon/runs/canon-run-security".to_string()),
        latest_governance_packet_source_stage: Some("bug-fix:implement".to_string()),
        latest_governance_packet_binding_reason: Some("upstream_stage_context".to_string()),
        latest_governance_approval: Some("not_needed".to_string()),
        latest_governance_decision: Some(
            "autopilot selected Canon mode SecurityAssessment for bug-fix:verify".to_string(),
        ),
        latest_governance_candidates: Some(vec![
            "select_mode".to_string(),
            "escalate_pr_review".to_string(),
        ]),
        governance_next_action: None,
        next_command: Some("boundline inspect".to_string()),
        explanation: "governance completed for the current verification stage".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("latest_governance_mode: security-assessment"), "{rendered}");
    assert!(
        rendered.contains("latest_governance_packet_ref: .canon/runs/canon-run-security"),
        "{rendered}"
    );
    assert!(
        rendered.contains("latest_governance_packet_source_stage: bug-fix:implement"),
        "{rendered}"
    );
    assert!(
        rendered.contains("latest_governance_packet_binding_reason: upstream_stage_context"),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_context_projection() {
    let view = SessionStatusView {
        session_id: "session-context-status".to_string(),
        workspace_ref: "/tmp/session-context".to_string(),
        goal: Some("Plan with bounded context".to_string()),
        latest_status: SessionStatus::Planned,
        advanced_context: Some(sample_advanced_context()),
        context_summary: Some("bounded context from 1 primary input(s)".to_string()),
        context_credibility: Some("credible".to_string()),
        context_primary_inputs: Some(vec!["src/context_router.rs".to_string()]),
        context_provenance: Some(vec![
            "workspace_file: src/context_router.rs (selected as a bounded workspace target for the current goal) [source=workspace_scan]"
                .to_string(),
        ]),
        next_command: Some("boundline run".to_string()),
        explanation: "session is ready to execute the bounded plan".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("context_summary: bounded context from 1 primary input(s)"));
    assert!(rendered.contains("context_credibility: credible"));
    assert!(rendered.contains("context_primary_inputs: src/context_router.rs"));
    assert!(rendered.contains("context_provenance: workspace_file: src/context_router.rs"));
    assert!(rendered.contains("retrieval_mode: local"));
    assert!(rendered.contains("retrieval_state: selected"));
    assert!(rendered.contains("semantic_policy_state: disabled"));
    assert!(rendered.contains("semantic_capability_state: unsupported"));
    assert!(rendered.contains("hybrid_outcome: baseline_only"));
    assert!(rendered.contains(
        "selected_evidence: src/context_router.rs [workspace_file] origin=fts goal keyword matched the implementation surface"
    ));
    assert!(rendered.contains(
        "impact_finding: tests/context_router.rs [missing_test] add or refresh the focused regression test"
    ));
}

#[test]
fn render_session_status_surfaces_context_substrate_projection_details() {
    let mut advanced_context = sample_advanced_context();
    advanced_context.repository_map_state = Some(RepositoryMapState::Missing);
    advanced_context.snapshot_cache_state = Some(SnapshotCacheState::Tracked);
    advanced_context.context_pack_entries = vec![ContextPackEntryProjection {
        source_ref: "logs/error.log".to_string(),
        source_kind: RetrievalSourceKind::Trace,
        authority_rank: AuthorityRank::Structured,
        fidelity_tier: ContextFidelityTier::Supporting,
        inclusion_mode: ContextInclusionMode::Digest,
        required_for_admission: false,
        reason: "large trace compacted to digest".to_string(),
        resolved_excerpt_anchor: Some("logs/error.log#digest-summary".to_string()),
        lifecycle_relevance: Some("recent_trace".to_string()),
        risk_relevance: Some("risk_signal".to_string()),
        ranking_rationale: Some("origin=fts, authority=structured".to_string()),
        digest_ref: Some(DigestBackedArtifactRef {
            digest: "fnv64:testdigest".to_string(),
            artifact_kind: "log".to_string(),
            summary: "validation failed | stack trace".to_string(),
            excerpt_anchor: Some("logs/error.log#digest-summary".to_string()),
            resolve_path: "logs/error.log".to_string(),
        }),
    }];
    advanced_context.omission_findings = vec![ContextOmissionFinding {
        severity: ContextOmissionSeverity::Blocking,
        reason_code: "critical_unavailable".to_string(),
        candidate_ref: "src/context_router.rs".to_string(),
        message: "critical context could not be admitted safely".to_string(),
        required_fidelity: Some(ContextFidelityTier::Critical),
        observed_mode: Some(ContextInclusionMode::Omitted),
    }];
    advanced_context.patch_safe_edit_attempts = vec![PatchSafeEditAttempt {
        target_ref: "src/context_router.rs".to_string(),
        anchor_refs: vec![
            "src/context_router.rs#start-anchor".to_string(),
            "src/context_router.rs#end-anchor".to_string(),
        ],
        pre_apply_digest: "fnv64:preapply".to_string(),
        post_apply_verification: vec!["cargo test --quiet".to_string()],
        result_state: PatchSafeEditResultState::ManualReviewRequired,
    }];

    let rendered = render_session_status(&SessionStatusView {
        advanced_context: Some(advanced_context),
        ..SessionStatusView::default()
    });

    assert!(rendered.contains("repository_map_state: missing"), "{rendered}");
    assert!(rendered.contains("snapshot_cache_state: tracked"), "{rendered}");
    assert!(rendered.contains("context_pack_entry_count: 1"), "{rendered}");
    assert!(rendered.contains("context_omission_finding_count: 1"), "{rendered}");
    assert!(rendered.contains("patch_safe_edit_attempt_count: 1"), "{rendered}");
    assert!(rendered.contains("context_entry: logs/error.log [trace]"), "{rendered}");
    assert!(rendered.contains("digest=fnv64:testdigest"), "{rendered}");
    assert!(rendered.contains("context_omission: src/context_router.rs [blocking]"), "{rendered}");
    assert!(rendered.contains("code=critical_unavailable"), "{rendered}");
    assert!(rendered.contains("required_fidelity=critical"), "{rendered}");
    assert!(rendered.contains("observed_mode=omitted"), "{rendered}");
    assert!(
        rendered.contains("patch_safe_edit: src/context_router.rs [manual_review_required]"),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_plan_quality_projection() {
    let rendered = render_session_status(&SessionStatusView {
        plan_quality_state: Some("clarification_required".to_string()),
        plan_quality_findings: Some(vec![
            "planning_rationale".to_string(),
            "verification_strategy".to_string(),
        ]),
        plan_quality_assumptions: Some(vec![
            "no explicit route override is required for this plan".to_string(),
        ]),
        ..SessionStatusView::default()
    });

    assert!(rendered.contains("plan_quality_state: clarification_required"), "{rendered}");
    assert!(
        rendered.contains("plan_quality_findings: planning_rationale, verification_strategy"),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "plan_quality_assumptions: no explicit route override is required for this plan"
        ),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_backlog_quality_projection() {
    let rendered = render_session_status(&SessionStatusView {
        backlog_quality_state: Some("clarification_required".to_string()),
        backlog_quality_findings: Some(vec![
            "missing_execution_handoff".to_string(),
            "missing_independent_verification_anchors".to_string(),
        ]),
        backlog_task_count: Some(2),
        backlog_mvp_scope: Some("SLICE-AUTH-001".to_string()),
        backlog_unmapped_items: Some(vec!["post-launch adoption metric".to_string()]),
        ..SessionStatusView::default()
    });

    assert!(rendered.contains("backlog_quality_state: clarification_required"), "{rendered}");
    assert!(
        rendered.contains(
            "backlog_quality_findings: missing_execution_handoff, missing_independent_verification_anchors"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("backlog_task_count: 2"), "{rendered}");
    assert!(rendered.contains("backlog_mvp_scope: SLICE-AUTH-001"), "{rendered}");
    assert!(rendered.contains("backlog_unmapped_items: post-launch adoption metric"), "{rendered}");
}

#[test]
fn render_session_status_surfaces_planning_analysis_projection() {
    let rendered = render_session_status(&SessionStatusView {
        planning_analysis_state: Some("blocked".to_string()),
        planning_analysis_findings: Some(vec![PlanningAnalysisFinding {
            severity: PlanningAnalysisSeverity::Critical,
            source: PlanningAnalysisSource::Validation,
            code: "validation_coverage_missing".to_string(),
            message: "selected slice is missing a matching acceptance anchor".to_string(),
            source_refs: vec![PlanningAnalysisSourceRef {
                artifact_kind: "backlog_document".to_string(),
                artifact_ref: "acceptance-anchors.md".to_string(),
                anchor: Some("slice_id=SLICE-SESSION-001".to_string()),
            }],
        }]),
        planning_analysis_coverage: Some(PlanningAnalysisCoverage {
            success_criteria_total: 2,
            success_criteria_covered: 2,
            backlog_slice_total: Some(2),
            backlog_slice_covered: Some(1),
            validation_anchor_total: Some(2),
            validation_anchor_covered: Some(1),
            risk_total: Some(1),
            risk_covered: Some(1),
            constraint_total: Some(1),
            constraint_covered: Some(1),
            governed_evidence_ready: false,
        }),
        ..SessionStatusView::default()
    });

    assert!(rendered.contains("planning_analysis_state: blocked"), "{rendered}");
    assert!(
        rendered.contains(
            "planning_analysis_findings: critical:validation:validation_coverage_missing:selected slice is missing a matching acceptance anchor"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "planning_analysis_coverage: success_criteria=2/2, backlog_slices=1/2, validation_anchors=1/2, risks=1/1, constraints=1/1, governed_evidence_ready=false"
        ),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_rejected_semantic_candidates() {
    let mut advanced_context = sample_advanced_context();
    advanced_context.semantic_policy_state = SemanticPolicyState::Local;
    advanced_context.semantic_capability_state = SemanticCapabilityState::Ready;
    advanced_context.hybrid_outcome = HybridOutcome::Expanded;
    advanced_context.rejected_candidates.push(RetrievedEvidenceCandidate {
        candidate_id: "candidate-rejected-1".to_string(),
        source_kind: RetrievalSourceKind::WorkspaceFile,
        source_ref: "src/semantic.rs".to_string(),
        authority_rank: AuthorityRank::Structured,
        match_origin: RetrievalMatchOrigin::SemanticExpand,
        selection_state: CandidateSelectionState::Rejected,
        selection_reason: "semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged".to_string(),
        provenance_summary: "workspace file evaluated through semantic expansion".to_string(),
        compatibility_state: RetrievalCompatibilityState::Compatible,
        staleness_state: RetrievalStalenessState::Fresh,
        lexical_score: None,
        semantic_score: RetrievalScore::from_raw(0.812),
        canon_semantic_contract_line: None,
        canon_semantic_provenance_ref: None,
    });

    let view = SessionStatusView {
        session_id: "session-context-status-rejected".to_string(),
        workspace_ref: "/tmp/session-context".to_string(),
        goal: Some("Plan with bounded context".to_string()),
        latest_status: SessionStatus::Planned,
        advanced_context: Some(advanced_context),
        explanation: "session is ready to execute the bounded plan".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("semantic_rejected_count: 1"));
    assert!(rendered.contains(
        "rejected_candidate: src/semantic.rs [workspace_file] origin=semantic_expand semantic_score=0.812 semantic similarity found the candidate but the bounded evidence limit kept the V1 set unchanged"
    ));
}

#[test]
fn render_session_status_surfaces_workflow_phase_and_pause_reason() {
    let view = SessionStatusView {
        session_id: "session-workflow-status".to_string(),
        workspace_ref: "/tmp/session-workflow".to_string(),
        goal: None,
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
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: None,
        active_workflow: Some("default".to_string()),
        workflow_phase: Some("capture".to_string()),
        workflow_next_action: Some(
            "boundline goal --workspace /tmp/session-workflow --goal <goal>".to_string(),
        ),
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: None,
        current_step_index: None,
        latest_status: SessionStatus::Initialized,
        execution_path: None,
        latest_trace_ref: None,
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
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
        next_command: None,
        explanation: "workflow is paused until a goal is captured".to_string(),
        ..Default::default()
    };

    let rendered = render_session_status(&view);

    assert!(rendered.contains("workflow: default"), "{rendered}");
    assert!(rendered.contains("workflow_phase: capture"), "{rendered}");
    assert!(
        rendered.contains(
            "execution_condition: waiting - workflow is waiting for a captured goal before it can continue"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "next_command: boundline goal --workspace /tmp/session-workflow --goal <goal>"
        ),
        "{rendered}"
    );
}

#[test]
fn resolve_trace_path_prefers_session_trace_ref_when_available() {
    use std::fs;

    let workspace =
        std::env::temp_dir().join(format!("boundline-unit-session-trace-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let explicit_session_trace =
        workspace.join(".boundline").join("traces").join("session-trace.json");
    fs::create_dir_all(explicit_session_trace.parent().unwrap()).unwrap();
    let store = FileTraceStore::new(explicit_session_trace.parent().unwrap());
    let trace = minimal_trace("task-session-trace");
    let persisted = store.persist(&trace).unwrap();

    let (target, path) =
        resolve_trace_path(None, Some(&workspace), Some(persisted.to_str().unwrap())).unwrap();

    assert_eq!(target, TraceResolutionTarget::SessionTraceRef);
    assert_eq!(path, persisted);
}

#[test]
fn execute_inspect_with_no_args_returns_missing_trace_reference_error() {
    let result = execute_inspect(None, None, None, false);
    assert!(matches!(result, Err(InspectCommandError::MissingTraceReference)), "{result:?}");
}

#[test]
fn execute_inspect_with_empty_workspace_returns_missing_latest_trace_error() {
    use std::fs;
    let workspace = std::env::temp_dir().join(format!("boundline-unit-empty-{}", Uuid::new_v4()));
    fs::create_dir_all(&workspace).unwrap();

    let result = execute_inspect(None, Some(&workspace), None, false);
    assert!(matches!(result, Err(InspectCommandError::MissingLatestTrace)), "{result:?}");
}

#[test]
fn summarize_trace_errors_on_unknown_step_kind() {
    use boundline::domain::trace::TraceEvent;
    use serde_json::json;

    let mut trace = succeeded_trace("task-badkind", "Bad kind test", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({"step_kind": "invalid_kind"}),
        recorded_at: 0,
    });

    let result = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace);
    assert!(matches!(result, Err(TraceSummaryError::UnknownStepKind(_))), "{result:?}");
}

#[test]
fn summarize_trace_errors_when_step_kind_payload_is_missing() {
    use boundline::domain::trace::TraceEvent;
    use serde_json::json;

    let mut trace = succeeded_trace("task-nokind", "Missing kind test", "done");
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::StepStarted,
        step_id: Some("step1".to_string()),
        plan_revision: 0,
        payload: json!({}),
        recorded_at: 0,
    });

    let result = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace);
    assert!(matches!(result, Err(TraceSummaryError::MissingStepKind(_))), "{result:?}");
}

#[test]
fn summarize_trace_uses_goal_plan_projection_and_decision_evidence_fallbacks() {
    use boundline::domain::trace::TraceEvent;

    let mut trace = ExecutionTrace::new("task-goal-plan", "session", "Decision summary test");
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::UnrecoverableError, "decision failed", None));
    trace.events.push(TraceEvent {
        event_id: "goal-plan".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "task_count": 2,
            "goal": "Decision summary test",
            "negotiation_goal_summary": "ship the bounded context slice",
            "negotiation_resolution": "credible",
            "negotiation_acceptance_boundary": "deliver the bounded outcome",
            "context_summary": "bounded context from src/lib.rs",
            "context_credibility": "stale",
            "context_primary_inputs": ["src/lib.rs"],
            "context_provenance": ["workspace_file: src/lib.rs (failing test target) [source=workspace_scan]"],
            "context_staleness_reason": "trace snapshot is stale",
            "advanced_context": sample_advanced_context()
        }),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "decision-created".to_string(),
        event_type: TraceEventType::DecisionCreated,
        step_id: Some("decision-1".to_string()),
        plan_revision: 0,
        payload: json!({
            "decision_type": "fix",
            "target": "src/lib.rs",
            "status": "created",
            "rationale": "failing tests point to arithmetic logic",
            "expected_outcome": "tests pass",
            "evidence_inputs": [{"kind": "workspace_file", "reference": "src/lib.rs"}]
        }),
        recorded_at: 1,
    });
    trace.events.push(TraceEvent {
        event_id: "decision-failed".to_string(),
        event_type: TraceEventType::DecisionFailed,
        step_id: Some("decision-1".to_string()),
        plan_revision: 0,
        payload: json!({
            "status": "failed",
            "target": "src/lib.rs",
            "action_result": {"stdout": "test failed"}
        }),
        recorded_at: 2,
    });
    trace.events.push(TraceEvent {
        event_id: "decision-recovered".to_string(),
        event_type: TraceEventType::DecisionRecovered,
        step_id: Some("decision-1".to_string()),
        plan_revision: 0,
        payload: json!({
            "status": "recovered",
            "recovery_decision_id": "decision-2"
        }),
        recorded_at: 3,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();

    assert_eq!(
        summary.goal_plan_summary.as_deref(),
        Some("2 bounded task(s) for Decision summary test")
    );
    assert_eq!(summary.negotiation_goal_summary.as_deref(), Some("ship the bounded context slice"));
    assert_eq!(summary.negotiation_resolution.as_deref(), Some("credible"));
    assert_eq!(summary.context_summary.as_deref(), Some("bounded context from src/lib.rs"));
    assert_eq!(summary.context_credibility.as_deref(), Some("stale"));
    assert_eq!(summary.context_primary_inputs, vec!["src/lib.rs".to_string()]);
    assert_eq!(
        summary.context_provenance,
        vec![
            "workspace_file: src/lib.rs (failing test target) [source=workspace_scan]".to_string()
        ]
    );
    assert_eq!(summary.context_staleness_reason.as_deref(), Some("trace snapshot is stale"));
    assert_eq!(
        summary.advanced_context.as_ref().map(AdvancedContextProjection::selected_evidence_count),
        Some(1)
    );
    assert!(
        summary
            .decision_timeline
            .iter()
            .any(|line| { line == "decision: decision-1 fix -> src/lib.rs [created]" })
    );
    assert!(
        summary
            .decision_timeline
            .iter()
            .any(|line| { line == "rationale: failing tests point to arithmetic logic" })
    );
    assert!(
        summary.decision_timeline.iter().any(|line| { line == "expected_outcome: tests pass" })
    );
    assert!(
        summary
            .decision_timeline
            .iter()
            .any(|line| { line == "evidence_inputs: workspace_file:src/lib.rs" })
    );
    assert!(
        summary
            .decision_timeline
            .iter()
            .any(|line| { line == "decision_status: decision-1 recovered via decision-2" })
    );
    assert_eq!(summary.failure_evidence, vec!["decision-1 src/lib.rs: test failed".to_string()]);

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");
    assert!(rendered.contains("retrieval_mode: local"), "{rendered}");
    assert!(rendered.contains("semantic_policy_state: disabled"), "{rendered}");
    assert!(
        rendered.contains(
            "selected_evidence: src/context_router.rs [workspace_file] origin=fts goal keyword matched the implementation surface"
        ),
        "{rendered}"
    );
}

#[test]
fn summarize_trace_reports_no_council_activity_without_review_evidence() {
    let mut trace = ExecutionTrace::new("task-council", "session", "Governed goal");
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason = Some(TerminalReason::new(
        TerminalCondition::NoCredibleNextStep,
        "blocked after governance",
        None,
    ));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GovernanceBlocked,
        step_id: Some("plan".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "plan:discovery",
            "reason": "stage council blocked planning"
        }),
        recorded_at: 0,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let council =
        summary.inspect_council.as_ref().expect("inspect summary should include a council closure");
    assert_eq!(council.headline, "no council activity was recorded");
    assert!(council.narrative_lines.is_empty());
    assert!(council.source_attribution.is_empty());

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");
    assert!(
        rendered.contains("inspect_council_headline: no council activity was recorded"),
        "{rendered}"
    );
}

#[test]
fn summarize_trace_extracts_advanced_context_from_goal_plan_payload() {
    use boundline::domain::trace::TraceEvent;

    let mut trace = succeeded_trace("task-advanced-context", "Inspect summary", "completed");
    trace.events.push(TraceEvent {
        event_id: "goal-plan".to_string(),
        event_type: TraceEventType::GoalPlanCreated,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "task_count": 1,
            "goal": "Inspect summary",
            "advanced_context": sample_advanced_context()
        }),
        recorded_at: 0,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();

    assert_eq!(
        summary.advanced_context.as_ref().map(AdvancedContextProjection::selected_evidence_count),
        Some(1)
    );
}

#[test]
fn compatibility_trace_without_active_session_surfaces_status_and_next_follow_up() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-unit-compat-status-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let mut trace = minimal_trace("task-compat-status");
    trace.terminal_status = Some(TaskStatus::Failed);
    trace.terminal_reason = Some(TerminalReason::new(
        TerminalCondition::UnrecoverableError,
        "compatibility run failed",
        None,
    ));
    FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();

    let status = execute_status(Some(&workspace)).unwrap();
    assert!(
        status.terminal_output.contains("continuity_authority: compatibility_trace"),
        "{}",
        status.terminal_output
    );
    assert!(
        status.terminal_output.contains("compatibility_follow_up: inspect_only"),
        "{}",
        status.terminal_output
    );

    let next = execute_next(Some(&workspace)).unwrap();
    assert!(
        next.terminal_output.contains("compatibility_follow_up: inspect_only"),
        "{}",
        next.terminal_output
    );
    assert!(
        next.terminal_output.contains("next_command: boundline inspect --workspace"),
        "{}",
        next.terminal_output
    );
}

#[test]
fn session_error_renderer_covers_trace_summary_and_cluster_config_guidance() {
    let trace_summary = render_session_error(
        "status",
        &SessionCommandError::TraceSummary("invalid compatibility trace".to_string()),
    );
    assert!(trace_summary.contains("status: session error"), "{trace_summary}");
    assert!(trace_summary.contains("reason: failed to summarize the latest compatibility trace: invalid compatibility trace"), "{trace_summary}");
    assert!(!trace_summary.contains("next_command:"), "{trace_summary}");

    let cluster_config = render_session_error(
        "run",
        &SessionCommandError::MissingClusterConfig {
            workspace: PathBuf::from("/tmp/cluster-owner"),
            command_name: "run",
        },
    );
    assert!(cluster_config.contains("run: session error"), "{cluster_config}");
    assert!(
        cluster_config
            .contains("reason: `run` requires a valid cluster config in /tmp/cluster-owner"),
        "{cluster_config}"
    );
    assert!(cluster_config.contains("next_command: boundline cluster init --workspace <primary> --cluster-id <id> --member <workspace> --member <workspace>"), "{cluster_config}");

    let session_store = render_session_error(
        "plan",
        &SessionCommandError::SessionStore(SessionStoreError::InvalidRecord(
            "workflow state mismatch".to_string(),
        )),
    );
    assert!(session_store.contains("plan: session error"), "{session_store}");
    assert!(
        session_store.contains("reason: invalid session record: workflow state mismatch"),
        "{session_store}"
    );
    assert!(!session_store.contains("next_command:"), "{session_store}");
}

#[test]
fn unimplemented_message_formats_the_command_name() {
    use boundline::cli::output::unimplemented_message;

    let msg = unimplemented_message(&DeveloperCommand::Doctor {
        workspace: Some(PathBuf::from("/tmp")),
        install: false,
    });
    assert_eq!(msg, "`doctor` is not implemented yet");
}

#[test]
fn command_names_render_for_all_four_subcommands() {
    assert_eq!(
        command_name(&DeveloperCommand::Doctor {
            workspace: Some(PathBuf::from("/tmp")),
            install: false,
        }),
        "doctor"
    );
    assert_eq!(
        command_name(&DeveloperCommand::Run {
            workspace: Some(PathBuf::from("/tmp")),
            cluster: None,
            goal: Some("x".to_string()),
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
        }),
        "run"
    );
    assert_eq!(
        command_name(&DeveloperCommand::Inspect {
            trace: None,
            workspace: None,
            cluster: None,
            session: None,
            audit: false,
        }),
        "inspect"
    );
}

#[test]
fn render_trace_summary_handles_all_terminal_status_variants() {
    let statuses = [
        (TaskStatus::Planned, "planned"),
        (TaskStatus::Running, "running"),
        (TaskStatus::Exhausted, "exhausted"),
        (TaskStatus::Aborted, "aborted"),
        (TaskStatus::Failed, "failed"),
    ];

    for (status, expected) in statuses {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/trace.json".to_string(),
            goal: "test".to_string(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            routing_summary: None,
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            executed_steps: vec![],
            recovery_events: vec![],
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: status,
            terminal_reason: TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "reason text",
                None,
            ),
            duration: None,
            ..Default::default()
        };
        let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");
        assert!(
            rendered.contains(&format!("terminal_status: {expected}")),
            "status {status:?}: {rendered}"
        );
    }
}

#[test]
fn render_trace_summary_surfaces_route_owner_and_config_projection() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/trace.json".to_string(),
        goal: "test".to_string(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        routing_summary: Some(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
                .to_string(),
        ),
        routing_projection: RoutingDecisionProjection::default(),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: Some("canon".to_string()),
        requested_governance_risk: Some("high".to_string()),
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: Vec::new(),
        failure_evidence: Vec::new(),
        adaptive_evidence: Vec::new(),
        executed_steps: Vec::new(),
        recovery_events: Vec::new(),
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
        ..Default::default()
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(rendered.contains("route_owner: compatibility"), "{rendered}");
    assert!(
        rendered.contains(
            "route_config_projection: requested_governance_runtime=canon | requested_governance_risk=high"
        ),
        "{rendered}"
    );
}

#[test]
fn render_session_status_projects_workspace_routing_defaults() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-route-config-status-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            planning: Some(ModelRoute {
                runtime: RuntimeKind::Codex,
                model: "o4-mini".to_string(),
            }),
            implementation: Some(ModelRoute {
                runtime: RuntimeKind::Copilot,
                model: "gpt-4o".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-config-projection".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Project route defaults".to_string()),
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
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: None,
        current_step_index: None,
        latest_status: SessionStatus::Initialized,
        execution_path: None,
        latest_trace_ref: None,
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
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
        next_command: Some("boundline goal --goal <goal>".to_string()),
        explanation: "session is waiting for a goal".to_string(),
        ..Default::default()
    });

    assert!(
        rendered.contains(
            "route_config_projection: workspace_routing: planning=codex/o4-mini, implementation=copilot/gpt-4o"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "effective_routing: planning=codex/o4-mini [workspace], implementation=copilot/gpt-4o [workspace], verification=copilot/gpt-4.1 [built-in], review=claude/sonnet-4 [built-in], adjudication=codex/o4-mini [built-in]"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "assistant_bindings: planning=codex, implementation=copilot, verification=copilot, review=claude, adjudication=codex"
        ),
        "{rendered}"
    );
}

#[test]
fn render_session_status_surfaces_follow_through_guidance() {
    let rendered = render_session_status(&SessionStatusView {
        session_id: "session-follow-through".to_string(),
        workspace_ref: "/tmp/workspace".to_string(),
        goal: Some("Fix the failing add test".to_string()),
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
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        active_flow: None,
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: None,
        current_step_id: Some("verify-fix-add".to_string()),
        current_step_index: Some(2),
        latest_status: SessionStatus::Running,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/workspace/.boundline/traces/trace.json".to_string()),
        latest_decision_status: Some("failed".to_string()),
        latest_decision_target: Some("verify-fix-add".to_string()),
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: Some(
            "selected src/lib.rs for the next bounded retry".to_string(),
        ),
        latest_candidate_family: Some("ordering_boundary_flip".to_string()),
        latest_selection_reason: Some("validation evidence still points to src/lib.rs".to_string()),
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: Some("failed".to_string()),
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
        explanation: "next recommended command for the active session is `boundline step`"
            .to_string(),
        ..Default::default()
    });

    assert!(
        rendered
            .contains("follow_through_guidance: selected src/lib.rs for the next bounded retry"),
        "{rendered}"
    );
    assert!(rendered.contains("follow_through_evidence_source: session:recovery"), "{rendered}");
    assert!(rendered.contains("follow_through_next_action: boundline step"), "{rendered}");
}

#[test]
fn render_trace_summary_projects_workspace_routing_defaults() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-route-config-trace-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let config = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-1".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

    let trace_ref = workspace.join(".boundline").join("traces").join("trace.json");
    let summary = TraceSummaryView {
        trace_ref: trace_ref.to_string_lossy().into_owned(),
        goal: "test".to_string(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        routing_summary: Some(
            "routing: native (goal_plan) - trace came from the session-native runtime".to_string(),
        ),
        routing_projection: RoutingDecisionProjection::default(),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: Vec::new(),
        failure_evidence: Vec::new(),
        adaptive_evidence: Vec::new(),
        executed_steps: Vec::new(),
        recovery_events: Vec::new(),
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
        ..Default::default()
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(
        rendered.contains("route_config_projection: workspace_routing: review=claude/reviewer-1"),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_prefers_persisted_routing_snapshot_over_current_workspace_config() {
    let workspace = std::env::temp_dir()
        .join(format!("boundline-route-config-trace-snapshot-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&workspace).unwrap();

    let current_config = ConfigFile {
        routing: RoutingConfig {
            review: Some(ModelRoute {
                runtime: RuntimeKind::Claude,
                model: "reviewer-now".to_string(),
            }),
            ..RoutingConfig::default()
        },
        ..ConfigFile::default()
    };
    FileConfigStore::for_workspace(&workspace).save_local(&current_config).unwrap();

    let trace_ref = workspace.join(".boundline").join("traces").join("trace.json");
    let mut trace = ExecutionTrace::new("task-1", "session-1", "test");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "event-1".to_string(),
        event_type: TraceEventType::TaskStarted,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "goal": "test",
            "input": {
                "routing_projection": {
                    "effective_routing": [
                        "planning=codex/o4-mini [workspace]",
                        "verification=copilot/gpt-4o [built-in]"
                    ],
                    "assistant_bindings": [
                        "planning=codex",
                        "verification=copilot"
                    ]
                }
            },
            "limits": RunLimits::default(),
        }),
        recorded_at: 0,
    });

    let summary = summarize_trace(&trace_ref, &trace).unwrap();
    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(
        rendered.contains(
            "route_config_projection: effective_routing: planning=codex/o4-mini [workspace], verification=copilot/gpt-4o [built-in] | assistant_bindings: planning=codex, verification=copilot"
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("reviewer-now"), "{rendered}");
}

#[test]
fn render_trace_summary_surfaces_follow_through_guidance() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/trace.json".to_string(),
        goal: "Fix the failing add test".to_string(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        routing_summary: Some(
            "routing: compatibility (execution_profile) - trace came from the explicit compatibility runtime"
                .to_string(),
        ),
        routing_projection: RoutingDecisionProjection::default(),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: vec![
            "decision verify-fix-add failed [1 attempt(s)] - validation failed after 1 attempt(s)"
                .to_string(),
        ],
        failure_evidence: vec!["validation_status: failed".to_string()],
        adaptive_evidence: Vec::new(),
        executed_steps: Vec::new(),
        recovery_events: Vec::new(),
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Failed,
        terminal_reason: TerminalReason::new(TerminalCondition::UnrecoverableError, "validation failed", None),
        duration: None,
        ..Default::default()
    };

    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(
        rendered.contains(
            "follow_through_guidance: decision verify-fix-add failed [1 attempt(s)] - validation failed after 1 attempt(s)"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("follow_through_evidence_source: trace:decision"), "{rendered}");
    assert!(rendered.contains("follow_through_next_action: /boundline-next"), "{rendered}");
}

#[test]
fn render_trace_summary_covers_replan_recovery_label_and_decision_step_kind() {
    let summary = TraceSummaryView {
        trace_ref: "/tmp/trace.json".to_string(),
        goal: "test".to_string(),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: None,
        routing_summary: None,
        routing_projection: RoutingDecisionProjection::default(),
        goal_plan_summary: None,
        authored_input_summary: None,
        authored_input_sources: Vec::new(),
        authored_input_deduplicated_sources: Vec::new(),
        clarification_headline: None,
        clarification_prompt: None,
        clarification_missing_fields: Vec::new(),
        requested_governance_runtime: None,
        requested_governance_risk: None,
        requested_governance_zone: None,
        requested_governance_owner: None,
        decision_timeline: Vec::new(),
        failure_evidence: Vec::new(),
        adaptive_evidence: Vec::new(),
        executed_steps: vec![TraceStepSummary {
            step_id: "decide".to_string(),
            step_kind: StepKind::Decision,
            attempts: 1,
            final_status: StepStatus::Succeeded,
            headline: "succeeded after 1 attempt(s)".to_string(),
        }],
        recovery_events: vec![TraceRecoveryEvent {
            event_type: TraceEventType::Replanned,
            trigger: "goal shifted".to_string(),
            related_step_id: None,
        }],
        governance_timeline: Vec::new(),
        governance_next_action: None,
        review_timeline: Vec::new(),
        terminal_status: TaskStatus::Succeeded,
        terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
        duration: None,
        ..Default::default()
    };

    let rendered = render_trace_summary(&summary, "latest-workspace-trace", "/boundline-next");

    assert!(rendered.contains("step decide (decision)"), "{rendered}");
    assert!(rendered.contains("replan: goal shifted"), "{rendered}");
}

#[test]
fn render_trace_summary_includes_security_assessment_packet_provenance() {
    let mut trace = ExecutionTrace::new("task-summary-governance", "session", "Governed goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::GovernanceStarted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "change:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "canon_mode": "security-assessment",
            "run_ref": "canon-run-security",
            "packet_source_stage": "change:implement",
            "packet_binding_reason": "same_stage_rerun"
        }),
        recorded_at: 0,
    });
    trace.events.push(TraceEvent {
        event_id: "e2".to_string(),
        event_type: TraceEventType::GovernanceCompleted,
        step_id: Some("verify".to_string()),
        plan_revision: 1,
        payload: json!({
            "stage_key": "change:verify",
            "runtime": GovernanceRuntimeKind::Canon,
            "headline": "security assessment packet ready",
            "packet_ref": ".canon/runs/canon-run-security",
            "packet_source_stage": "change:implement",
            "packet_binding_reason": "same_stage_rerun"
        }),
        recorded_at: 1,
    });

    let summary = summarize_trace(PathBuf::from("/tmp/trace.json"), &trace).unwrap();
    let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");

    assert!(
        rendered.contains(
            "governance_started: change:verify (security-assessment) [canon-run-security] from change:implement (same_stage_rerun)"
        ),
        "{rendered}"
    );
    assert!(
        rendered.contains(
            "governance_completed: security assessment packet ready [.canon/runs/canon-run-security] from change:implement (same_stage_rerun)"
        ),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_covers_pending_running_and_skipped_step_statuses() {
    let statuses = [
        (StepStatus::Pending, "pending"),
        (StepStatus::Running, "running"),
        (StepStatus::Skipped, "skipped"),
    ];

    for (status, expected) in statuses {
        let summary = TraceSummaryView {
            trace_ref: "/tmp/trace.json".to_string(),
            goal: "test".to_string(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            routing_summary: None,
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            executed_steps: vec![TraceStepSummary {
                step_id: "step1".to_string(),
                step_kind: StepKind::Agent,
                attempts: 1,
                final_status: status,
                headline: format!("{expected} after 1 attempt(s)"),
            }],
            recovery_events: vec![],
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None),
            duration: None,
            ..Default::default()
        };
        let rendered = render_trace_summary(&summary, "explicit-trace", "/boundline-next");
        assert!(
            rendered.contains(&format!("(agent) {expected} [1")),
            "status {status:?}: {rendered}"
        );
    }
}

#[test]
fn render_session_status_surfaces_cluster_delivery_story() {
    let rendered = render_session_status(&SessionStatusView {
        session_id: "cluster-session".to_string(),
        workspace_ref: "/tmp/primary".to_string(),
        goal: Some("Ship a clustered fix".to_string()),
        negotiation_goal_summary: None,
        negotiation_resolution: None,
        negotiation_acceptance_boundary: None,
        cluster_delivery_story: Some(sample_cluster_delivery_story()),
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
        active_flow: None,
        flow_state: None,
        active_workflow: None,
        workflow_phase: None,
        workflow_next_action: None,
        continuity_authority: None,
        compatibility_follow_up: None,
        current_stage_id: None,
        current_stage_index: None,
        total_stages: None,
        plan_revision: Some(1),
        current_step_id: Some("handoff".to_string()),
        current_step_index: Some(1),
        latest_status: SessionStatus::Failed,
        execution_path: Some("native_goal_plan".to_string()),
        latest_trace_ref: Some("/tmp/secondary/.boundline/traces/task.json".to_string()),
        latest_decision_status: None,
        latest_decision_target: None,
        latest_changed_files: None,
        latest_workspace_slice: None,
        latest_selection_headline: None,
        latest_candidate_family: None,
        latest_selection_reason: None,
        latest_rejected_candidates: None,
        latest_attempt_lineage: None,
        latest_validation_status: None,
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
        next_command: Some("boundline inspect --cluster /tmp/primary".to_string()),
        explanation: "secondary workspace could not continue the bounded handoff".to_string(),
        ..Default::default()
    });

    assert!(rendered.contains("cluster_id: cluster-1"), "{rendered}");
    assert!(rendered.contains("cluster_route_owner: native"), "{rendered}");
    assert!(rendered.contains("cluster_authoritative_workspace: /tmp/primary"), "{rendered}");
    assert!(
        rendered.contains(
            "cluster_execution_condition: failed - secondary workspace could not continue the bounded handoff"
        ),
        "{rendered}"
    );
    assert!(rendered.contains("cluster_blocking_workspace: /tmp/secondary"), "{rendered}");
    assert!(
        rendered.contains(
            "cluster_participating_workspaces: /tmp/primary [entry] | /tmp/secondary [blocked]"
        ),
        "{rendered}"
    );
}

#[test]
fn render_trace_summary_surfaces_cluster_delivery_story() {
    let rendered = render_trace_summary(
        &TraceSummaryView {
            trace_ref: "/tmp/secondary/.boundline/traces/task.json".to_string(),
            goal: "Ship a clustered fix".to_string(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: Some(sample_cluster_delivery_story()),
            routing_summary: Some(
                "routing: native (goal_plan) - trace came from the session-native runtime"
                    .to_string(),
            ),
            routing_projection: RoutingDecisionProjection::default(),
            goal_plan_summary: None,
            authored_input_summary: None,
            authored_input_sources: Vec::new(),
            authored_input_deduplicated_sources: Vec::new(),
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: Vec::new(),
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            decision_timeline: Vec::new(),
            failure_evidence: Vec::new(),
            adaptive_evidence: Vec::new(),
            executed_steps: Vec::new(),
            recovery_events: Vec::new(),
            governance_timeline: Vec::new(),
            governance_next_action: None,
            review_timeline: Vec::new(),
            terminal_status: TaskStatus::Failed,
            terminal_reason: TerminalReason::new(
                TerminalCondition::UnrecoverableError,
                "secondary workspace could not continue the bounded handoff",
                None,
            ),
            duration: None,
            ..Default::default()
        },
        "explicit-trace",
        "/boundline-next",
    );

    assert!(rendered.contains("cluster_id: cluster-1"), "{rendered}");
    assert!(rendered.contains("cluster_route_owner: native"), "{rendered}");
    assert!(rendered.contains("cluster_authoritative_workspace: /tmp/primary"), "{rendered}");
    assert!(rendered.contains("cluster_blocking_workspace: /tmp/secondary"), "{rendered}");
}

#[test]
fn cluster_rendering_covers_init_status_and_inspect_functions() {
    let rendered = render_cluster_init(
        "cluster-alpha",
        "/tmp/cluster.toml",
        &["/tmp/ws1".to_string(), "/tmp/ws2".to_string()],
    );
    assert!(rendered.contains("cluster: initialized"), "{rendered}");
    assert!(rendered.contains("cluster_id: cluster-alpha"), "{rendered}");
    assert!(rendered.contains("- /tmp/ws1"), "{rendered}");

    let report = ClusterInspectReport {
        cluster_id: "cluster-beta".to_string(),
        primary_workspace_ref: "/tmp/primary".to_string(),
        members: vec![
            ClusterMemberStatusView {
                workspace_ref: "/tmp/ws1".to_string(),
                state: ClusterMemberState::Healthy,
                latest_status: Some(SessionStatus::Succeeded),
                latest_trace_ref: Some("/tmp/ws1/.boundline/traces/t.json".to_string()),
                headline: "all steps completed".to_string(),
            },
            ClusterMemberStatusView {
                workspace_ref: "/tmp/ws2".to_string(),
                state: ClusterMemberState::MissingSession,
                latest_status: None,
                latest_trace_ref: None,
                headline: "no session found".to_string(),
            },
            ClusterMemberStatusView {
                workspace_ref: "/tmp/ws3".to_string(),
                state: ClusterMemberState::MissingTrace,
                latest_status: None,
                latest_trace_ref: None,
                headline: "trace missing".to_string(),
            },
            ClusterMemberStatusView {
                workspace_ref: "/tmp/ws4".to_string(),
                state: ClusterMemberState::Blocked,
                latest_status: None,
                latest_trace_ref: None,
                headline: "blocked by dependency".to_string(),
            },
            ClusterMemberStatusView {
                workspace_ref: "/tmp/ws5".to_string(),
                state: ClusterMemberState::Invalid,
                latest_status: None,
                latest_trace_ref: None,
                headline: "invalid config".to_string(),
            },
        ],
    };

    let status_rendered = render_cluster_status(&report);
    assert!(status_rendered.contains("cluster: status"), "{status_rendered}");
    assert!(status_rendered.contains("[healthy]"), "{status_rendered}");
    assert!(status_rendered.contains("status=succeeded"), "{status_rendered}");
    assert!(status_rendered.contains("[missing-session]"), "{status_rendered}");
    assert!(status_rendered.contains("[missing-trace]"), "{status_rendered}");
    assert!(status_rendered.contains("[blocked]"), "{status_rendered}");
    assert!(status_rendered.contains("[invalid]"), "{status_rendered}");

    let inspect_rendered = render_cluster_inspect(&report);
    assert!(inspect_rendered.contains("cluster: inspect"), "{inspect_rendered}");
    assert!(inspect_rendered.contains("trace=<missing>"), "{inspect_rendered}");
    assert!(
        inspect_rendered.contains("trace=/tmp/ws1/.boundline/traces/t.json"),
        "{inspect_rendered}"
    );
}

#[test]
fn cluster_story_lines_cover_all_route_owner_and_execution_kind_variants() {
    let base_story = |route_owner, kind| ClusterDeliveryStory {
        cluster_id: "c1".to_string(),
        primary_workspace_ref: "/tmp/p".to_string(),
        authoritative_workspace_ref: "/tmp/p".to_string(),
        route_owner,
        member_workspace_refs: Vec::new(),
        participating_workspaces: Vec::new(),
        started_from_command: "run".to_string(),
        execution_condition: ClusteredExecutionCondition {
            kind,
            active_workspace_ref: None,
            blocking_workspace_ref: None,
            summary: "ok".to_string(),
            recovery_allowed: false,
        },
        updated_at: 0,
    };

    for (owner, label) in [
        (ClusterRouteOwner::Workflow, "workflow"),
        (ClusterRouteOwner::Review, "review"),
        (ClusterRouteOwner::Governance, "governance"),
        (ClusterRouteOwner::Compatibility, "compatibility"),
    ] {
        let story = base_story(owner, ClusteredExecutionKind::Success);
        let lines = render_session_status(&SessionStatusView {
            cluster_delivery_story: Some(story),
            session_id: "s".to_string(),
            workspace_ref: "/tmp/p".to_string(),
            explanation: "ok".to_string(),
            ..Default::default()
        });
        assert!(lines.contains(&format!("cluster_route_owner: {label}")), "{lines}");
    }

    for (kind, label) in [
        (ClusteredExecutionKind::Success, "success"),
        (ClusteredExecutionKind::Paused, "paused"),
        (ClusteredExecutionKind::Blocked, "blocked"),
        (ClusteredExecutionKind::Exhausted, "exhausted"),
        (ClusteredExecutionKind::InspectOnly, "inspect_only"),
    ] {
        let story = base_story(ClusterRouteOwner::Native, kind);
        let lines = render_session_status(&SessionStatusView {
            cluster_delivery_story: Some(story),
            session_id: "s".to_string(),
            workspace_ref: "/tmp/p".to_string(),
            explanation: "ok".to_string(),
            ..Default::default()
        });
        assert!(lines.contains(&format!("cluster_execution_condition: {label} - ok")), "{lines}");
    }

    let story_with_participants = ClusterDeliveryStory {
        cluster_id: "c2".to_string(),
        primary_workspace_ref: "/tmp/p".to_string(),
        authoritative_workspace_ref: "/tmp/p".to_string(),
        route_owner: ClusterRouteOwner::Native,
        member_workspace_refs: Vec::new(),
        participating_workspaces: vec![
            WorkspaceParticipationRecord {
                workspace_ref: "/tmp/a".to_string(),
                participation_kind: WorkspaceParticipationKind::ReadOnly,
                order: 0,
                latest_trace_ref: None,
                latest_status: None,
                headline: "read only".to_string(),
                terminal_reason: None,
            },
            WorkspaceParticipationRecord {
                workspace_ref: "/tmp/b".to_string(),
                participation_kind: WorkspaceParticipationKind::Mutated,
                order: 1,
                latest_trace_ref: None,
                latest_status: None,
                headline: "mutated".to_string(),
                terminal_reason: None,
            },
            WorkspaceParticipationRecord {
                workspace_ref: "/tmp/c".to_string(),
                participation_kind: WorkspaceParticipationKind::Skipped,
                order: 2,
                latest_trace_ref: None,
                latest_status: None,
                headline: "skipped".to_string(),
                terminal_reason: None,
            },
        ],
        started_from_command: "run".to_string(),
        execution_condition: ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Success,
            active_workspace_ref: None,
            blocking_workspace_ref: None,
            summary: "all done".to_string(),
            recovery_allowed: false,
        },
        updated_at: 0,
    };
    let lines = render_session_status(&SessionStatusView {
        cluster_delivery_story: Some(story_with_participants),
        session_id: "s2".to_string(),
        workspace_ref: "/tmp/p".to_string(),
        explanation: "ok".to_string(),
        ..Default::default()
    });
    assert!(lines.contains("[read_only]"), "{lines}");
    assert!(lines.contains("[mutated]"), "{lines}");
    assert!(lines.contains("[skipped]"), "{lines}");
}

#[test]
fn command_names_render_for_all_remaining_subcommands() {
    let cases: &[(&DeveloperCommand, &str)] = &[
        (
            &DeveloperCommand::Checkpoint {
                command: CheckpointSubcommand::List {
                    workspace: None,
                    cluster: None,
                    session: None,
                },
            },
            "checkpoint",
        ),
        (
            &DeveloperCommand::Goal {
                workspace: None,
                cluster: None,
                update: false,
                new_session: false,
                goal: None,
                brief: Vec::new(),
                governance: None,
                risk: None,
                zone: None,
                owner: None,
                slug: None,
            },
            "goal",
        ),
        (
            &DeveloperCommand::Plan {
                workspace: None,
                cluster: None,
                input: None,
                flow: None,
                no_flow: false,
                no_canon: false,
                refine: false,
                no_refine: false,
                max_rounds: None,
            },
            "plan",
        ),
        (&DeveloperCommand::Step { workspace: None, cluster: None }, "step"),
        (
            &DeveloperCommand::Workflow { command: WorkflowSubcommand::List { workspace: None } },
            "workflow",
        ),
        (&DeveloperCommand::Status { workspace: None, cluster: None, session: None }, "status"),
        (&DeveloperCommand::Next { workspace: None, cluster: None, session: None }, "next"),
        (&DeveloperCommand::Continue { workspace: None, cluster: None, session: None }, "continue"),
        (
            &DeveloperCommand::Govern {
                workspace: None,
                mode: None,
                goal: None,
                brief: Vec::new(),
                base: None,
                head: None,
                risk: None,
                structural_impact: false,
                public_contract_change: false,
                validation_exhausted: false,
                pr_ready: false,
                preserved_behavior_evidence: false,
            },
            "govern",
        ),
        (
            &DeveloperCommand::Assistant {
                command: AssistantSubcommand::Install {
                    host: AssistantHost::Copilot,
                    scope: AssistantInstallScope::User,
                },
            },
            "assistant",
        ),
        (
            &DeveloperCommand::Init {
                scope: InitConfigScope::Workspace,
                workspace: std::path::PathBuf::from("."),
                non_interactive: false,
                template: None,
                ollama_profile: None,
                assistant: Vec::new(),
                adapter: None,
                route: Vec::new(),
                domain: Vec::new(),
                domain_standard: Vec::new(),
                context_binding: Vec::new(),
                required_context_binding: Vec::new(),
                canon_mode_selection: None,
                risk: None,
                zone: None,
                owner: None,
                ide: Vec::new(),
                auto_approve: None,
                semantic_index_hook_action: None,
                export_docs: false,
                refresh: false,
                diff: false,
                to: None,
                force: false,
            },
            "init",
        ),
        (
            &DeveloperCommand::Config {
                command: ConfigSubcommand::Show { workspace: None, cluster: None, scope: None },
            },
            "config",
        ),
        (
            &DeveloperCommand::Cluster {
                command: ClusterSubcommand::Status { workspace: std::path::PathBuf::from("/tmp") },
            },
            "cluster",
        ),
    ];
    for (command, expected) in cases {
        assert_eq!(command_name(command), *expected, "command_name mismatch for {expected}");
    }
}

#[test]
fn summarize_trace_extracts_delight_feedback_signal_from_events() {
    let mut trace = ExecutionTrace::new("task-delight", "session-delight", "Delight test goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.events.push(TraceEvent {
        event_id: "e1".to_string(),
        event_type: TraceEventType::TerminalRecorded,
        step_id: None,
        plan_revision: 0,
        payload: json!({
            "delight_feedback": {
                "total_explanations": 5,
                "attributed_explanations": 3,
                "accepted_next_actions": 2,
                "overridden_next_actions": 0,
                "captured_at": 99999
            }
        }),
        recorded_at: 0,
    });

    let summary = summarize_trace(std::path::PathBuf::from("/tmp/task.json"), &trace)
        .expect("summarize_trace failed");
    assert!(
        summary.delight_feedback.is_some(),
        "expected delight_feedback to be populated from trace event payload"
    );
    let signal = summary.delight_feedback.as_ref().unwrap();
    assert_eq!(signal.total_explanations, 5);
    assert_eq!(signal.attributed_explanations, 3);
}

#[test]
fn assistant_asset_catalog_exports_goal_template_for_scaffold_and_docs() {
    let scaffold_assets = assets_for_assistants(&[AssistantHostKind::Copilot]);
    let goal_template = scaffold_assets
        .iter()
        .find(|asset| asset.relative_path == "assistant/prompts/goal-template.md")
        .expect("assistant scaffold should include the goal template");
    assert!(goal_template.contents.contains("/boundline:goal"));

    let docs_assets = docs_assets_for_assistants_under(
        &[AssistantHostKind::Copilot],
        &PathBuf::from("docs/boundline"),
    );
    assert!(docs_assets.iter().any(|asset| {
        asset.relative_path == "docs/boundline/assistant/prompts/goal-template.md"
    }));
}

fn sample_cluster_delivery_story() -> ClusterDeliveryStory {
    ClusterDeliveryStory {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: "/tmp/primary".to_string(),
        authoritative_workspace_ref: "/tmp/primary".to_string(),
        route_owner: ClusterRouteOwner::Native,
        member_workspace_refs: vec!["/tmp/primary".to_string(), "/tmp/secondary".to_string()],
        participating_workspaces: vec![
            WorkspaceParticipationRecord {
                workspace_ref: "/tmp/primary".to_string(),
                participation_kind: WorkspaceParticipationKind::Entry,
                order: 0,
                latest_trace_ref: Some("/tmp/primary/.boundline/traces/task.json".to_string()),
                latest_status: Some("running".to_string()),
                headline: "primary workspace started the clustered run".to_string(),
                terminal_reason: None,
            },
            WorkspaceParticipationRecord {
                workspace_ref: "/tmp/secondary".to_string(),
                participation_kind: WorkspaceParticipationKind::Blocked,
                order: 1,
                latest_trace_ref: Some("/tmp/secondary/.boundline/traces/task.json".to_string()),
                latest_status: Some("failed".to_string()),
                headline: "secondary workspace could not continue the bounded handoff".to_string(),
                terminal_reason: Some(
                    "secondary workspace could not continue the bounded handoff".to_string(),
                ),
            },
        ],
        started_from_command: "run".to_string(),
        execution_condition: ClusteredExecutionCondition {
            kind: ClusteredExecutionKind::Failed,
            active_workspace_ref: Some("/tmp/secondary".to_string()),
            blocking_workspace_ref: Some("/tmp/secondary".to_string()),
            summary: "secondary workspace could not continue the bounded handoff".to_string(),
            recovery_allowed: false,
        },
        updated_at: 42,
    }
}

fn make_impact_finding(
    id: &str,
    kind: ImpactFindingKind,
    status: ImpactFindingStatus,
    severity: ImpactFindingSeverity,
) -> ImpactAnalysisFinding {
    ImpactAnalysisFinding {
        finding_id: id.to_string(),
        finding_kind: kind,
        subject_ref: format!("src/{id}.rs"),
        status,
        severity,
        recommended_follow_up: format!("address {id}"),
        supporting_relationship_ids: Vec::new(),
    }
}

fn make_delegation_view(mode: DelegationContinuityMode, headline: &str) -> DelegationStatusView {
    DelegationStatusView {
        mode,
        packet_id: None,
        packet_kind: None,
        packet_state: None,
        target_owner: None,
        headline: headline.to_string(),
        evidence_summary: "test evidence".to_string(),
    }
}

fn blocked_reasoning_profile(
    disagreement: Option<&str>,
    next_action: Option<&str>,
) -> ProfileActivationRecord {
    ProfileActivationRecord {
        activation_id: "test-activation".to_string(),
        stage_key: "test:stage".to_string(),
        profile_id: ReasoningProfileId::BoundedReflexion,
        trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
        activation_reason: "test block reason".to_string(),
        status: ReasoningActivationStatus::Blocked,
        participants: Vec::new(),
        budget: ReasoningBudget {
            max_participants: 1,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 1024,
            max_adjudication_steps: 0,
        },
        posture: None,
        independence: None,
        outcome: Some(ReasoningOutcome {
            outcome_kind: ReasoningOutcomeKind::Disagreed,
            headline: "test outcome".to_string(),
            disagreement_summary: disagreement.map(str::to_string),
            next_action: next_action.map(str::to_string),
            iterations: Vec::new(),
        }),
        confidence: None,
    }
}

// ── output_explanation.rs coverage ───────────────────────────────────────────

#[test]
fn explanation_trace_summary_staleness_risk_hits_stale_context_branch() {
    let summary = TraceSummaryView {
        context_staleness_reason: Some("workspace root changed".to_string()),
        failure_evidence: Vec::new(),
        reasoning_profile: None,
        ..TraceSummaryView::default()
    };
    let text = render_trace_summary(&summary, "run", "/workspace");
    assert!(
        text.contains("risk_summary: stale context reduces confidence: workspace root changed"),
        "{text}"
    );
}

#[test]
fn explanation_trace_summary_no_failure_risk_hits_no_explicit_failure_branch() {
    // governance_timeline makes canon_sources non-empty; default terminal_reason has empty message
    let summary = TraceSummaryView {
        governance_timeline: vec!["approval-granted".to_string()],
        ..TraceSummaryView::default()
    };
    let text = render_trace_summary(&summary, "run", "/workspace");
    assert!(text.contains("risk_summary: No explicit runtime failure"), "{text}");
}

#[test]
fn explanation_session_status_staleness_risk_branch() {
    let view = SessionStatusView {
        context_staleness_reason: Some("index out of date".to_string()),
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(
        text.contains("risk_summary: stale context reduces confidence: index out of date"),
        "{text}"
    );
}

#[test]
fn explanation_session_status_clarification_fallback_with_canon_present() {
    // canon_sources non-empty via latest_governance_packet_ref; clarification fields non-empty
    let view = SessionStatusView {
        clarification_missing_fields: Some(vec!["goal".to_string(), "scope".to_string()]),
        latest_governance_packet_ref: Some("packet-001".to_string()),
        context_staleness_reason: None,
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(
        text.contains("fallback_disclosure: Clarification is still required for: goal, scope"),
        "{text}"
    );
}

#[test]
fn explanation_trace_reasoning_disagreement_summary_risk_path() {
    let summary = TraceSummaryView {
        reasoning_profile: Some(blocked_reasoning_profile(
            Some("participants could not converge"),
            None,
        )),
        failure_evidence: Vec::new(),
        ..TraceSummaryView::default()
    };
    let text = render_trace_summary(&summary, "run", "/workspace");
    assert!(text.contains("participants could not converge"), "{text}");
}

#[test]
fn explanation_relationship_kinds_cover_affects_system_domain_suggests_reviewer_supports_risk() {
    let advanced = AdvancedContextProjection {
        selected_evidence: vec![
            RetrievedEvidenceCandidate {
                candidate_id: "c-canon".to_string(),
                source_kind: RetrievalSourceKind::CanonArtifact,
                source_ref: "canon/spec.md".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::Fts,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "canon source".to_string(),
                provenance_summary: "canon artifact selected".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            },
            RetrievedEvidenceCandidate {
                candidate_id: "c-review".to_string(),
                source_kind: RetrievalSourceKind::ReviewFinding,
                source_ref: "review/finding.md".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::Fts,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "review finding".to_string(),
                provenance_summary: "review finding selected".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            },
            RetrievedEvidenceCandidate {
                candidate_id: "c-verify".to_string(),
                source_kind: RetrievalSourceKind::VerificationEvidence,
                source_ref: "verify/evidence.md".to_string(),
                authority_rank: AuthorityRank::Structured,
                match_origin: RetrievalMatchOrigin::Fts,
                selection_state: CandidateSelectionState::Selected,
                selection_reason: "verification evidence".to_string(),
                provenance_summary: "verification evidence selected".to_string(),
                compatibility_state: RetrievalCompatibilityState::Compatible,
                staleness_state: RetrievalStalenessState::Fresh,
                lexical_score: None,
                semantic_score: None,
                canon_semantic_contract_line: None,
                canon_semantic_provenance_ref: None,
            },
        ],
        relationships: vec![
            RelationshipProjection {
                relationship_id: "r-affects-system".to_string(),
                subject_ref: "src/system_core.rs".to_string(),
                relationship_kind: RelationshipKind::AffectsSystem,
                credibility_state: RelationshipCredibilityState::Tentative,
                explanation: "affects core system".to_string(),
                supporting_candidate_ids: vec!["c-canon".to_string()],
            },
            RelationshipProjection {
                relationship_id: "r-affects-domain".to_string(),
                subject_ref: "src/domain_model.rs".to_string(),
                relationship_kind: RelationshipKind::AffectsDomain,
                credibility_state: RelationshipCredibilityState::Insufficient,
                explanation: "affects domain invariants".to_string(),
                supporting_candidate_ids: vec!["c-review".to_string()],
            },
            RelationshipProjection {
                relationship_id: "r-suggests-reviewer".to_string(),
                subject_ref: "reviewer@team.example".to_string(),
                relationship_kind: RelationshipKind::SuggestsReviewer,
                credibility_state: RelationshipCredibilityState::Credible,
                explanation: "suggests a reviewer".to_string(),
                supporting_candidate_ids: vec!["c-verify".to_string()],
            },
            RelationshipProjection {
                relationship_id: "r-supports-risk".to_string(),
                subject_ref: "src/risk_path.rs".to_string(),
                relationship_kind: RelationshipKind::SupportsRisk,
                credibility_state: RelationshipCredibilityState::Tentative,
                explanation: "supports identified risk".to_string(),
                supporting_candidate_ids: vec![],
            },
        ],
        impact_findings: Vec::new(),
        ..sample_advanced_context()
    };
    let summary =
        TraceSummaryView { advanced_context: Some(advanced), ..TraceSummaryView::default() };
    let text = render_trace_summary(&summary, "run", "/workspace");
    // AffectsSystem → architecture category
    assert!(text.contains("assumption_group: architecture"), "{text}");
    // AffectsDomain → domain category
    assert!(text.contains("assumption_group: domain"), "{text}");
    // SuggestsReviewer → governance category
    assert!(text.contains("assumption_group: governance"), "{text}");
    // SupportsRisk → implementation category
    assert!(text.contains("assumption_group: implementation"), "{text}");
    // CanonArtifact source → Canon
    assert!(text.contains("Canon"), "{text}");
    // Tentative credibility → medium risk
    assert!(text.contains("medium"), "{text}");
    // Insufficient credibility → high risk
    assert!(text.contains("high"), "{text}");
}

fn single_impact_trace(
    kind: ImpactFindingKind,
    status: ImpactFindingStatus,
    severity: ImpactFindingSeverity,
) -> String {
    let finding = make_impact_finding("target", kind, status, severity);
    let advanced = AdvancedContextProjection {
        impact_findings: vec![finding],
        relationships: Vec::new(),
        selected_evidence: Vec::new(),
        rejected_candidates: Vec::new(),
        ..sample_advanced_context()
    };
    let summary =
        TraceSummaryView { advanced_context: Some(advanced), ..TraceSummaryView::default() };
    render_trace_summary(&summary, "run", "/workspace")
}

#[test]
fn explanation_impact_finding_affected_system_covers_group_and_challenge_branches() {
    let text = single_impact_trace(
        ImpactFindingKind::AffectedSystem,
        ImpactFindingStatus::Acknowledged,
        ImpactFindingSeverity::Low,
    );
    assert!(text.contains("hidden_impact_affected_systems"), "{text}");
    assert!(text.contains("system impact extends beyond the current slice"), "{text}");
    assert!(text.contains("cross-system impact can escape"), "{text}");
    assert!(text.contains("acknowledged"), "{text}");
    assert!(text.contains("low"), "{text}");
}

#[test]
fn explanation_impact_finding_affected_domain_covers_group_and_challenge_branches() {
    let text = single_impact_trace(
        ImpactFindingKind::AffectedDomain,
        ImpactFindingStatus::Resolved,
        ImpactFindingSeverity::High,
    );
    assert!(text.contains("hidden_impact_affected_domains"), "{text}");
    assert!(text.contains("domain impact extends beyond the current slice"), "{text}");
    assert!(text.contains("domain invariants can drift"), "{text}");
    assert!(text.contains("resolved"), "{text}");
    assert!(text.contains("high"), "{text}");
}

#[test]
fn explanation_impact_finding_contract_exposure_covers_group_and_challenge_branches() {
    let text = single_impact_trace(
        ImpactFindingKind::ContractExposure,
        ImpactFindingStatus::Invalidated,
        ImpactFindingSeverity::Low,
    );
    assert!(text.contains("hidden_impact_contract_exposures"), "{text}");
    assert!(text.contains("contract exposure still needs review"), "{text}");
    assert!(text.contains("downstream consumers can break"), "{text}");
    assert!(text.contains("invalidated"), "{text}");
}

#[test]
fn explanation_impact_finding_reviewer_gap_covers_group_and_challenge_branches() {
    let text = single_impact_trace(
        ImpactFindingKind::ReviewerGap,
        ImpactFindingStatus::Open,
        ImpactFindingSeverity::High,
    );
    assert!(text.contains("hidden_impact_required_reviewers"), "{text}");
    assert!(text.contains("required reviewer coverage is still missing"), "{text}");
    assert!(text.contains("review can miss critical dissent"), "{text}");
}

#[test]
fn explanation_impact_finding_evidence_gap_covers_group_and_challenge_branches() {
    let text = single_impact_trace(
        ImpactFindingKind::EvidenceGap,
        ImpactFindingStatus::Open,
        ImpactFindingSeverity::Medium,
    );
    assert!(text.contains("hidden_impact_missing_evidence"), "{text}");
    assert!(text.contains("required evidence is still missing"), "{text}");
    assert!(text.contains("the plan can proceed without required evidence"), "{text}");
}

// ── output_routing.rs coverage ────────────────────────────────────────────────

#[test]
fn routing_goal_captured_status_maps_to_blocked() {
    let view = SessionStatusView {
        latest_status: SessionStatus::GoalCaptured,
        execution_path: None,
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(text.contains("execution_condition: blocked"), "{text}");
    assert!(text.contains("goal captured"), "{text}");
}

#[test]
fn routing_delegation_resolved_mode_maps_to_waiting() {
    let view = SessionStatusView {
        delegation: Some(make_delegation_view(
            DelegationContinuityMode::Resolved,
            "handoff resolved",
        )),
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(text.contains("execution_condition: waiting - handoff resolved"), "{text}");
}

#[test]
fn routing_delegation_exhausted_and_none_modes_in_session() {
    let exhausted = SessionStatusView {
        delegation: Some(make_delegation_view(
            DelegationContinuityMode::Exhausted,
            "delegation exhausted",
        )),
        ..SessionStatusView::default()
    };
    let exhausted_text = render_session_status(&exhausted);
    assert!(exhausted_text.contains("execution_condition: inspect_only"), "{exhausted_text}");

    let none_mode = SessionStatusView {
        delegation: Some(make_delegation_view(DelegationContinuityMode::None, "delegation none")),
        ..SessionStatusView::default()
    };
    let none_text = render_session_status(&none_mode);
    assert!(none_text.contains("execution_condition: waiting - delegation none"), "{none_text}");
}

#[test]
fn routing_trace_delegation_resolved_inspect_only_and_none_modes() {
    let resolved = TraceSummaryView {
        delegation: Some(make_delegation_view(
            DelegationContinuityMode::Resolved,
            "trace resolved",
        )),
        ..TraceSummaryView::default()
    };
    let r_text = render_trace_summary(&resolved, "run", "/workspace");
    assert!(r_text.contains("execution_condition: waiting - trace resolved"), "{r_text}");

    let inspect = TraceSummaryView {
        delegation: Some(make_delegation_view(
            DelegationContinuityMode::InspectOnly,
            "inspect only",
        )),
        ..TraceSummaryView::default()
    };
    let i_text = render_trace_summary(&inspect, "run", "/workspace");
    assert!(i_text.contains("execution_condition: inspect_only"), "{i_text}");

    let none_mode = TraceSummaryView {
        delegation: Some(make_delegation_view(DelegationContinuityMode::None, "delegation none")),
        ..TraceSummaryView::default()
    };
    let n_text = render_trace_summary(&none_mode, "run", "/workspace");
    assert!(n_text.contains("execution_condition: waiting - delegation none"), "{n_text}");
}

#[test]
fn routing_workflow_clarify_phase_with_pending_clarification_maps_to_waiting() {
    let view = SessionStatusView {
        workflow_phase: Some("clarify".to_string()),
        clarification_missing_fields: Some(vec!["goal".to_string()]),
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(text.contains("execution_condition: waiting"), "{text}");
    assert!(text.contains("clarification is still required"), "{text}");
}

#[test]
fn routing_workflow_review_phase_terminal_and_waiting_branches() {
    let terminal = SessionStatusView {
        workflow_phase: Some("review".to_string()),
        latest_status: SessionStatus::Failed,
        ..SessionStatusView::default()
    };
    let t_text = render_session_status(&terminal);
    assert!(t_text.contains("execution_condition: terminal"), "{t_text}");
    assert!(t_text.contains("non-success result"), "{t_text}");

    let waiting = SessionStatusView {
        workflow_phase: Some("review".to_string()),
        latest_status: SessionStatus::Running,
        latest_review_trigger: Some("review-trigger-001".to_string()),
        latest_review_outcome: None,
        ..SessionStatusView::default()
    };
    let w_text = render_session_status(&waiting);
    assert!(w_text.contains("execution_condition: waiting"), "{w_text}");
    assert!(w_text.contains("review outcome is still pending"), "{w_text}");
}

#[test]
fn routing_session_planned_with_current_step_id_uses_task_ready_message() {
    let view = SessionStatusView {
        latest_status: SessionStatus::Planned,
        current_step_id: Some("step-007".to_string()),
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(text.contains("a bounded task is ready for the next execution step"), "{text}");
}

#[test]
fn routing_session_failed_with_exhaustion_reason_uses_the_reason() {
    let view = SessionStatusView {
        latest_status: SessionStatus::Failed,
        latest_exhaustion_reason: Some("retry limit reached after 3 attempts".to_string()),
        ..SessionStatusView::default()
    };
    let text = render_session_status(&view);
    assert!(text.contains("retry limit reached after 3 attempts"), "{text}");
}

#[test]
fn routing_running_condition_failed_decision_and_review_trigger_branches() {
    let failed_decision = SessionStatusView {
        latest_status: SessionStatus::Running,
        latest_decision_status: Some("failed".to_string()),
        ..SessionStatusView::default()
    };
    let f_text = render_session_status(&failed_decision);
    assert!(f_text.contains("decision failed and recovery is in progress"), "{f_text}");

    let review_trigger = SessionStatusView {
        latest_status: SessionStatus::Running,
        latest_decision_status: None,
        latest_review_trigger: Some("context-drift".to_string()),
        ..SessionStatusView::default()
    };
    let r_text = render_session_status(&review_trigger);
    assert!(r_text.contains("review is in progress"), "{r_text}");
}

#[test]
fn routing_reasoning_block_with_disagreement_summary_and_no_next_action() {
    let profile = blocked_reasoning_profile(Some("team could not agree on the approach"), None);
    let summary =
        TraceSummaryView { reasoning_profile: Some(profile), ..TraceSummaryView::default() };
    let text = render_trace_summary(&summary, "run", "/workspace");
    assert!(text.contains("team could not agree on the approach"), "{text}");
    assert!(text.contains("execution_condition: blocked"), "{text}");
}

// ── output_host.rs command_name coverage ──────────────────────────────────────

#[test]
fn command_name_govern_and_config_variants_return_correct_names() {
    let govern = DeveloperCommand::Govern {
        workspace: None,
        mode: None,
        goal: None,
        brief: Vec::new(),
        base: None,
        head: None,
        risk: None,
        structural_impact: false,
        public_contract_change: false,
        validation_exhausted: false,
        pr_ready: false,
        preserved_behavior_evidence: false,
    };
    assert_eq!(command_name(&govern), "govern");

    let config = DeveloperCommand::Config {
        command: ConfigSubcommand::Show { workspace: None, cluster: None, scope: None },
    };
    assert_eq!(command_name(&config), "config");
}

// ── output_session_status.rs render_session_status_brief coverage ─────────────

#[test]
fn render_session_status_brief_covers_continuity_governance_and_review_fields() {
    let view = SessionStatusView {
        session_id: "brief-gov-test".to_string(),
        workspace_ref: "/tmp/ws".to_string(),
        latest_status: SessionStatus::Running,
        explanation: "test explanation".to_string(),
        continuity_authority: Some(ContinuityAuthority::NativeSession),
        execution_path: Some("native_goal_plan".to_string()),
        plan_revision: Some(3),
        current_step_index: Some(1),
        current_step_id: Some("step-abc".to_string()),
        latest_changed_files: Some(vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]),
        latest_review_trigger: Some("pre_merge".to_string()),
        latest_review_outcome: Some("approved".to_string()),
        latest_review_headline: Some("changes look good".to_string()),
        latest_review_vote: Some("approved".to_string()),
        latest_governance_run_ref: Some("gov-run-001".to_string()),
        latest_governance_state: Some("reviewing".to_string()),
        latest_governance_runtime_state: Some("active".to_string()),
        latest_governance_rollout_profile: Some("incremental".to_string()),
        latest_governance_reason: Some("blocked by review".to_string()),
        latest_governance_contract_lines: Some(vec!["contract-alpha".to_string()]),
        latest_governance_approval_provenance: Some("council approved".to_string()),
        latest_governance_blocked_reason: Some("pending review".to_string()),
        latest_governance_packet_ref: Some("packet-001".to_string()),
        latest_governance_packet_source_stage: Some("plan:discovery".to_string()),
        latest_governance_packet_binding_reason: Some("bound to discovery".to_string()),
        latest_governance_approval: Some("approved".to_string()),
        latest_governance_decision: Some("proceed".to_string()),
        governance_lifecycle_runtime: Some("synod-runtime".to_string()),
        governance_lifecycle_mode_selection: Some("auto".to_string()),
        governance_lifecycle_selected_mode: Some("review".to_string()),
        governance_lifecycle_selected_mode_sequence: Some(vec![
            "plan".to_string(),
            "review".to_string(),
        ]),
        latest_governance_candidates: Some(vec!["council-alpha".to_string()]),
        goal_plan_state: Some("active".to_string()),
        goal_plan_revision: Some(2),
        latest_checkpoint_id: Some("ckpt-001".to_string()),
        latest_checkpoint_scope: Some("workspace".to_string()),
        clarification_missing_fields: Some(vec!["goal".to_string()]),
        clarification_questions: Some(vec!["What is the scope?".to_string()]),
        authored_input_sources: Some(vec!["attached: spec.md".to_string()]),
        authored_input_deduplicated_sources: Some(vec!["spec.md".to_string()]),
        latest_validation_status: Some("passed".to_string()),
        latest_reasoning_profile: Some(ProfileActivationRecord {
            activation_id: "test-activation-brief".to_string(),
            stage_key: "test:stage".to_string(),
            profile_id: ReasoningProfileId::BoundedReflexion,
            trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
            activation_reason: "test reason".to_string(),
            status: ReasoningActivationStatus::Active,
            participants: Vec::new(),
            budget: ReasoningBudget {
                max_participants: 1,
                max_branches: 1,
                max_debate_rounds: 0,
                max_reflexion_revisions: 0,
                max_calls: 2,
                max_tokens: 1024,
                max_adjudication_steps: 0,
            },
            posture: None,
            independence: None,
            outcome: Some(ReasoningOutcome {
                outcome_kind: ReasoningOutcomeKind::Converged,
                headline: "reasoning complete".to_string(),
                next_action: Some("verify the contract".to_string()),
                iterations: vec![],
                disagreement_summary: None,
            }),
            confidence: None,
        }),
        ..SessionStatusView::default()
    };

    let text = render_session_status_brief(&view);

    assert!(text.contains("continuity_authority: native_session"), "{text}");
    assert!(text.contains("execution_path: native_goal_plan"), "{text}");
    assert!(text.contains("latest_changed_files: src/main.rs"), "{text}");
    assert!(text.contains("latest_governance_run_ref: gov-run-001"), "{text}");
    assert!(text.contains("latest_governance_candidates: council-alpha"), "{text}");
    assert!(text.contains("governance_lifecycle_selected_mode_sequence: plan, review"), "{text}");
    assert!(text.contains("review: latest_review_trigger=pre_merge"), "{text}");
    assert!(text.contains("summary: goal_plan_state=active r2"), "{text}");
    assert!(text.contains("latest_checkpoint_id=ckpt-001 (workspace)"), "{text}");
    assert!(text.contains("clarification_missing_fields: goal"), "{text}");
    assert!(text.contains("authored_input_sources: attached: spec.md"), "{text}");
    assert!(text.contains("latest_reasoning_next_action=verify the contract"), "{text}");
}

#[test]
fn render_session_status_brief_covers_compatibility_follow_up_and_cluster_delivery_story() {
    let view = SessionStatusView {
        session_id: "brief-compat-test".to_string(),
        workspace_ref: "/tmp/ws".to_string(),
        latest_status: SessionStatus::Running,
        explanation: "resumed from compatibility trace".to_string(),
        compatibility_follow_up: Some(CompatibilityFollowUpView {
            follow_up_mode: CompatibilityFollowUpMode::Resumable,
            trace_ref: "trace-abc-123".to_string(),
            routing_summary: "resumed from native trace".to_string(),
            execution_condition: "execution can resume".to_string(),
            terminal_status: TaskStatus::Succeeded,
            terminal_reason: "reached terminal state".to_string(),
            next_command: "boundline run".to_string(),
        }),
        cluster_delivery_story: Some(ClusterDeliveryStory {
            cluster_id: "cluster-001".to_string(),
            primary_workspace_ref: "/tmp/primary".to_string(),
            authoritative_workspace_ref: "/tmp/primary".to_string(),
            route_owner: ClusterRouteOwner::Native,
            member_workspace_refs: vec!["/tmp/member1".to_string()],
            participating_workspaces: vec![],
            started_from_command: "boundline run".to_string(),
            execution_condition: ClusteredExecutionCondition {
                kind: ClusteredExecutionKind::Success,
                active_workspace_ref: None,
                blocking_workspace_ref: None,
                summary: "all members succeeded".to_string(),
                recovery_allowed: false,
            },
            updated_at: 1_748_000_000,
        }),
        ..SessionStatusView::default()
    };

    let text = render_session_status_brief(&view);

    assert!(text.contains("compatibility_follow_up: resumable"), "{text}");
    assert!(text.contains("cluster_id: cluster-001"), "{text}");
}

#[test]
fn render_session_status_brief_covers_reasoning_blocking_condition_and_none_goal_plan_revision() {
    let view = SessionStatusView {
        session_id: "brief-blocking-test".to_string(),
        workspace_ref: "/tmp/ws".to_string(),
        latest_status: SessionStatus::Blocked,
        explanation: "blocked on reasoning profile".to_string(),
        goal_plan_state: Some("proposed".to_string()),
        // goal_plan_revision left as None → hits the None arm in session_summary_brief_line
        latest_reasoning_profile: Some(ProfileActivationRecord {
            activation_id: "test-activation-block".to_string(),
            stage_key: "test:stage".to_string(),
            profile_id: ReasoningProfileId::BoundedReflexion,
            trigger: ReasoningActivationTrigger::CanonRequiredChallenge,
            activation_reason: "blocked by canon challenge".to_string(),
            status: ReasoningActivationStatus::Blocked,
            participants: Vec::new(),
            budget: ReasoningBudget {
                max_participants: 1,
                max_branches: 1,
                max_debate_rounds: 0,
                max_reflexion_revisions: 0,
                max_calls: 2,
                max_tokens: 1024,
                max_adjudication_steps: 0,
            },
            posture: None,
            independence: None,
            outcome: None, // None → hits else-if blocking branch in session_reasoning_brief_line
            confidence: None,
        }),
        ..SessionStatusView::default()
    };

    let text = render_session_status_brief(&view);

    assert!(text.contains("summary: goal_plan_state=proposed"), "{text}");
    assert!(!text.contains("goal_plan_state=proposed r"), "{text}");
    assert!(text.contains("reasoning:"), "{text}");
    assert!(text.contains("latest_reasoning_profile_id=bounded_reflexion"), "{text}");
}
