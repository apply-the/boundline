use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;

use serde_json::{Map, json};
use uuid::Uuid;

use super::{
    SessionRuntime, cluster_task_status_text, cluster_workspace_is_blocked,
    effective_assistant_runtimes, is_governance_trace_event, project_scale_input_for_goal,
    project_scale_state_for_goal, session_status_for_task_status,
};
use crate::adapters::checkpoint_store::FileCheckpointStore;
use crate::adapters::config_store::FileConfigStore;
use crate::adapters::env_layer::{
    DEEPSEEK_API_KEY_ENV, DEEPSEEK_BASE_URL_ENV, GROQ_API_KEY_ENV, GROQ_BASE_URL_ENV,
    OPENAI_API_KEY_ENV, OPENAI_BASE_URL_ENV,
};
use crate::adapters::session_store::SessionStore;
use crate::adapters::trace_store::TraceStore;
use crate::domain::brief::{GovernanceIntent, normalize_inputs, normalize_inputs_with_governance};
use crate::domain::cluster::{ClusterSessionProjection, ClusteredExecutionKind};
use crate::domain::configuration::{
    CapabilityState, ConfigFile, EffortFallbackPolicy, EffortLevel, ModelRoute, RouteSlot,
    RoutingConfig, RuntimeCapabilityProfile, RuntimeKind, SlotEffortPolicy,
};
use crate::domain::decision::{Decision, DecisionType, EvidenceRef};
use crate::domain::domain_templates::{
    DomainFamily, DomainTemplateSettings, ExternalContextBinding, ExternalContextKind,
};
use crate::domain::execution::{
    ExecutionAttemptDefinition, ExecutionCommand, ExecutionFailureMode, WorkspaceChange,
    WorkspaceExecutionProfile,
};
use crate::domain::flow::{attach_stage_metadata, built_in_flow};
use crate::domain::flow_policy::FlowPolicy;
use crate::domain::goal_plan::{GoalPlan, InferredFlow, PlannedTask};
use crate::domain::governance::{
    ApprovalState, CanonMode, CanonModeSelectionPreference, CanonRuntimeConfig,
    GovernanceLifecycleState, GovernanceProfile, GovernanceRuntimeKind, GovernedSessionLifecycle,
    GovernedStageRecord, PacketReadiness, StageGovernancePolicy, SystemContextBinding,
};
use crate::domain::guidance::{
    CapabilityPhase, FindingConfidence, GuardianDisposition, GuardianFinding,
    GuidanceAuthoritySource, GuidanceGuardianProjection,
};
use crate::domain::limits::{RunLimits, TerminalCondition};
use crate::domain::plan::Plan;
use crate::domain::project_memory::{
    CompatibilityOutcome, LineageRef, ProjectMemoryContext, ProjectMemoryStatus,
    ProjectMemorySurface, PromotionStateView,
};
use crate::domain::reasoning::{
    IndependenceAssessment, IndependenceAssessmentResult, IndependenceFloor, ParticipantAssignment,
    ParticipantRoleDefinition, ReasoningActivationStatus, ReasoningAdjudicationMode,
    ReasoningBudget, ReasoningConfidenceLevel, ReasoningDegradationPolicy,
    ReasoningObservedDistinctness, ReasoningOutcomeKind, ReasoningParticipantRoleKind,
    ReasoningParticipantStatus, ReasoningProfileDefinition, ReasoningProfileFamily,
    ReasoningProfileId, ReasoningRoutePreference,
};
use crate::domain::review::{
    AdjudicationDefinition, ReviewProfile, ReviewScenario, ReviewTrigger, ReviewerDefinition,
    ReviewerDisposition, ReviewerFinding, VoteRuleDefinition,
};
use crate::domain::session::{
    ActiveSessionRecord, ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState,
    SessionCommand, SessionStatus,
};
use crate::domain::step::{
    ExecutionStatus, Recoverability, Step, StepExecutionResult, StepKind, StepStatus,
};
use crate::domain::task::{Task, TaskRunRequest, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::tool_result::ToolResult;
use crate::domain::trace::{ExecutionTrace, TraceEventType};
use crate::domain::workflow::ProjectScaleStageKind;
use crate::fixture::FixtureRuntime;
use crate::orchestrator::planner::StaticPlanner;
use crate::registry::agent_registry::AgentRegistry;
use crate::registry::tool_registry::ToolRegistry;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvRestore<'a> {
    saved: BTreeMap<&'static str, Option<std::ffi::OsString>>,
    _lock: MutexGuard<'a, ()>,
}

impl Drop for EnvRestore<'_> {
    fn drop(&mut self) {
        unsafe {
            for (key, value) in &self.saved {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

fn temp_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    workspace
}

fn request_headers_complete(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n").map(|index| index + 4)
}

fn request_content_length(buffer: &[u8]) -> Option<usize> {
    let headers_end = request_headers_complete(buffer)?;
    let headers = String::from_utf8_lossy(&buffer[..headers_end]);
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.trim().eq_ignore_ascii_case("content-length") {
            return None;
        }
        value.trim().parse::<usize>().ok()
    })
}

fn request_complete(buffer: &[u8]) -> bool {
    match (request_headers_complete(buffer), request_content_length(buffer)) {
        (Some(headers_end), Some(content_length)) => buffer.len() >= headers_end + content_length,
        (Some(_), None) => true,
        _ => false,
    }
}

fn with_env_test<T>(tracked_keys: &[&'static str], action: impl FnOnce() -> T) -> T {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let saved =
        tracked_keys.iter().map(|key| (*key, std::env::var_os(key))).collect::<BTreeMap<_, _>>();
    let restore = EnvRestore { saved, _lock: lock };

    unsafe {
        for key in tracked_keys {
            std::env::remove_var(key);
        }
    }

    let result = action();
    drop(restore);
    result
}

fn sample_project_memory_lineage(run_ref: &str, mode: &str) -> LineageRef {
    LineageRef {
        contract_version: "v1".to_string(),
        producer: "canon".to_string(),
        source_ref: format!("canon-run:{run_ref}"),
        source_artifacts: vec!["architecture-overview.md".to_string()],
        mode: Some(mode.to_string()),
        promotion_state: "auto".to_string(),
        approval_state: Some("Completed".to_string()),
        stage: Some(mode.to_string()),
        owner: Some("Owner <owner@example.com>".to_string()),
        risk: Some("bounded-impact".to_string()),
        zone: Some("yellow".to_string()),
        promoted_at: "2026-05-13T14:30:00Z".to_string(),
        content_digest: "sha256:abc123".to_string(),
        packet_readiness: Some("complete".to_string()),
        promotion_profile: Some("project-memory".to_string()),
    }
}

fn write_execution_profile_workspace(
    prefix: &str,
    attempts: Vec<ExecutionAttemptDefinition>,
) -> PathBuf {
    write_governed_execution_profile_workspace(prefix, attempts, Vec::new(), None)
}

fn write_governed_execution_profile_workspace(
    prefix: &str,
    attempts: Vec<ExecutionAttemptDefinition>,
    read_targets: Vec<String>,
    governance: Option<GovernanceProfile>,
) -> PathBuf {
    let workspace = temp_workspace(prefix);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "session-runtime-profile".to_string(),
            read_targets,
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts,
            adaptive: None,
            limits: RunLimits::default(),
            governance,
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();
    workspace
}

fn build_request(workspace_ref: &str) -> TaskRunRequest {
    TaskRunRequest {
        goal: "Drive a session runtime branch".to_string(),
        input: json!({"ticket": "SESSION-RUNTIME"}),
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace_ref.to_string(),
        limits: RunLimits::default(),
        initial_context: None,
    }
}

fn decision_task(workspace_ref: &str, input: serde_json::Value) -> Task {
    let plan = Plan::new(vec![Step::decision("decide", input).unwrap()]).unwrap();
    Task::new("task-runtime", &build_request(workspace_ref), plan).unwrap()
}

fn build_session(workspace: &Path, task: Task) -> ActiveSessionRecord {
    ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Drive a session runtime branch".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: Some(task),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    }
}

fn spawn_scripted_response_server(
    response_bodies: Vec<String>,
) -> Result<(String, mpsc::Receiver<String>, thread::JoinHandle<()>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let address = listener.local_addr().map_err(|error| error.to_string())?;
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        for response_body in response_bodies {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };

            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
            loop {
                match stream.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(read) => {
                        buffer.extend_from_slice(&chunk[..read]);
                        if request_complete(&buffer) {
                            break;
                        }
                    }
                    Err(_) => return,
                }
            }

            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let _ = sender.send(request_text);
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    Ok((format!("http://{address}"), receiver, handle))
}

fn openai_completion_response(payload: serde_json::Value) -> String {
    json!({
        "choices": [
            {
                "message": {
                    "content": payload.to_string()
                }
            }
        ]
    })
    .to_string()
}

#[cfg(unix)]
fn write_fake_canon_command(workspace: &Path) -> PathBuf {
    let packet_dir = workspace.join(".canon/planning-packet");
    fs::create_dir_all(&packet_dir).unwrap();
    fs::write(packet_dir.join("brief.md"), "planning packet\n").unwrap();
    let response = serde_json::json!({
        "status": "governed_ready",
        "approval_state": "not_needed",
        "run_ref": "canon-run-plan",
        "packet_ref": ".canon/planning-packet",
        "expected_document_refs": [".canon/planning-packet/brief.md"],
        "document_refs": [".canon/planning-packet/brief.md"],
        "packet_readiness": "reusable",
        "headline": "planning packet ready",
        "message": "planning governance completed"
    });
    let response_path = workspace.join("fake-canon-response.json");
    fs::write(&response_path, response.to_string()).unwrap();
    let command_path = workspace.join("fake-canon");
    fs::write(
        &command_path,
        format!("#!/bin/sh\ncat >/dev/null\ncat '{}'\n", response_path.to_string_lossy()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&command_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&command_path, permissions).unwrap();
    command_path
}

#[cfg(unix)]
fn write_fake_execution_canon_command(workspace: &Path) -> (PathBuf, PathBuf) {
    let requests_path = workspace.join("fake-canon-requests.ndjson");
    let implementation_packet_dir = workspace.join(".canon/execution/implementation");
    let verification_packet_dir = workspace.join(".canon/execution/verification");
    fs::create_dir_all(&implementation_packet_dir).unwrap();
    fs::create_dir_all(&verification_packet_dir).unwrap();
    fs::write(implementation_packet_dir.join("brief.md"), "implementation packet\n").unwrap();
    fs::write(verification_packet_dir.join("brief.md"), "verification packet\n").unwrap();

    let implementation_response_path = workspace.join("fake-canon-implementation-response.json");
    fs::write(
        &implementation_response_path,
        json!({
            "status": "governed_ready",
            "approval_state": "not_needed",
            "run_ref": "canon-run-implementation",
            "packet_ref": ".canon/execution/implementation",
            "expected_document_refs": [".canon/execution/implementation/brief.md"],
            "document_refs": [".canon/execution/implementation/brief.md"],
            "packet_readiness": "reusable",
            "headline": "implementation governance ready",
            "message": "implementation governance completed"
        })
        .to_string(),
    )
    .unwrap();

    let verification_response_path = workspace.join("fake-canon-verification-response.json");
    fs::write(
        &verification_response_path,
        json!({
            "status": "governed_ready",
            "approval_state": "not_needed",
            "run_ref": "canon-run-verification",
            "packet_ref": ".canon/execution/verification",
            "expected_document_refs": [".canon/execution/verification/brief.md"],
            "document_refs": [".canon/execution/verification/brief.md"],
            "packet_readiness": "reusable",
            "headline": "verification governance ready",
            "message": "verification governance completed"
        })
        .to_string(),
    )
    .unwrap();

    let command_path = workspace.join("fake-execution-canon");
    fs::write(
        &command_path,
        format!(
            "#!/bin/sh\nrequest=$(cat)\nprintf '%s\\n' \"$request\" >> '{}'\nif printf '%s' \"$request\" | grep -q '\"mode\":\"verification\"'; then\n  cat '{}'\nelse\n  cat '{}'\nfi\n",
            requests_path.to_string_lossy(),
            verification_response_path.to_string_lossy(),
            implementation_response_path.to_string_lossy(),
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&command_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&command_path, permissions).unwrap();
    (command_path, requests_path)
}

fn manual_runtime() -> FixtureRuntime {
    FixtureRuntime {
        profile: WorkspaceExecutionProfile {
            name: "manual-runtime".to_string(),
            read_targets: Vec::new(),
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
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
        },
        planner: std::sync::Arc::new(StaticPlanner::new(
            Plan::new(vec![Step::decision("placeholder", json!({})).unwrap()]).unwrap(),
        )),
        agents: AgentRegistry::new(),
        tools: ToolRegistry::new(),
    }
}

fn context() -> TaskContext {
    TaskContext::new("session-runtime", "/tmp/workspace", RunLimits::default(), Map::new())
}

fn save_local_routing(workspace: &Path, routing: RoutingConfig) {
    FileConfigStore::for_workspace(workspace)
        .save_local(&ConfigFile { version: 1, routing, canon: None })
        .unwrap();
}

fn independent_pair_review_profile() -> ReasoningProfileDefinition {
    ReasoningProfileDefinition {
        profile_id: ReasoningProfileId::IndependentPairReview,
        family: ReasoningProfileFamily::BlindReview,
        allowed_stages: vec![CanonMode::Discovery],
        limits: ReasoningBudget {
            max_participants: 2,
            max_branches: 1,
            max_debate_rounds: 0,
            max_reflexion_revisions: 0,
            max_calls: 2,
            max_tokens: 8_000,
            max_adjudication_steps: 1,
        },
        participant_roles: vec![
            ParticipantRoleDefinition {
                role_id: "reviewer_primary".to_string(),
                role_kind: ReasoningParticipantRoleKind::BlindReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: true,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
            ParticipantRoleDefinition {
                role_id: "reviewer_secondary".to_string(),
                role_kind: ReasoningParticipantRoleKind::BlindReviewer,
                preferred_slot: ReasoningRoutePreference::Review,
                independence_requirements: IndependenceFloor {
                    route_distinct: true,
                    provider_distinct: true,
                    context_distinct: false,
                    prompt_pattern_distinct: false,
                    minimum_participants: 2,
                },
                required: true,
            },
        ],
        adjudication_mode: ReasoningAdjudicationMode::GovernanceReview,
        degradation_policy: ReasoningDegradationPolicy {
            allow_degraded_independence: false,
            allow_reduced_participants: false,
            interruptible: true,
            blocked_next_action: Some(
                "configure distinct reviewer routes for reviewer_primary and reviewer_secondary"
                    .to_string(),
            ),
        },
    }
}

fn review_kind_role(
    role_id: &str,
    role_kind: ReasoningParticipantRoleKind,
    preferred_slot: ReasoningRoutePreference,
) -> ParticipantRoleDefinition {
    ParticipantRoleDefinition {
        role_id: role_id.to_string(),
        role_kind,
        preferred_slot,
        independence_requirements: IndependenceFloor {
            route_distinct: true,
            provider_distinct: true,
            context_distinct: false,
            prompt_pattern_distinct: false,
            minimum_participants: 2,
        },
        required: true,
    }
}

fn reasoning_profile_with_id(profile_id: ReasoningProfileId) -> ReasoningProfileDefinition {
    let mut profile = independent_pair_review_profile();
    profile.profile_id = profile_id;
    profile
}

fn sample_reasoning_participants() -> Vec<ParticipantAssignment> {
    vec![
        ParticipantAssignment {
            role_id: "reviewer_primary".to_string(),
            participant_id: "participant-1".to_string(),
            effective_route: "reviewer_roles.alpha:claude:sonnet-4".to_string(),
            provider_family: Some("claude".to_string()),
            context_basis: "reasoning_context:bug-fix:investigate".to_string(),
            prompting_pattern: "blind_reviewer".to_string(),
            status: ReasoningParticipantStatus::Pending,
            result_summary: None,
        },
        ParticipantAssignment {
            role_id: "reviewer_secondary".to_string(),
            participant_id: "participant-2".to_string(),
            effective_route: "reviewer_roles.beta:gemini:gemini-2.5-pro".to_string(),
            provider_family: Some("gemini".to_string()),
            context_basis: "reasoning_context:bug-fix:verify".to_string(),
            prompting_pattern: "heterogeneous_reviewer".to_string(),
            status: ReasoningParticipantStatus::Pending,
            result_summary: None,
        },
        ParticipantAssignment {
            role_id: "reviewer_shadow".to_string(),
            participant_id: "participant-3".to_string(),
            effective_route: "reviewer_roles.alpha:claude:sonnet-4".to_string(),
            provider_family: Some("claude".to_string()),
            context_basis: "reasoning_context:bug-fix:investigate".to_string(),
            prompting_pattern: "blind_reviewer".to_string(),
            status: ReasoningParticipantStatus::Pending,
            result_summary: Some("already summarized".to_string()),
        },
    ]
}

#[test]
fn reasoning_route_for_review_kinds_falls_back_to_configured_reviewer_roles_in_order() {
    let workspace = temp_workspace("boundline-runtime-reasoning-reviewer-role-fallback");
    let mut routing = RoutingConfig {
        review: Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4.1".to_string() }),
        ..RoutingConfig::default()
    };
    routing.reviewer_roles.insert(
        "alpha".to_string(),
        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
    );
    routing.reviewer_roles.insert(
        "beta".to_string(),
        ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() },
    );
    save_local_routing(&workspace, routing);

    let effective_routing = super::effective_routing_for_workspace(&workspace);
    let expected = [
        (
            review_kind_role(
                "blind",
                ReasoningParticipantRoleKind::BlindReviewer,
                ReasoningRoutePreference::Review,
            ),
            "reviewer_roles.alpha:claude:sonnet-4",
        ),
        (
            review_kind_role(
                "heterogeneous",
                ReasoningParticipantRoleKind::HeterogeneousReviewer,
                ReasoningRoutePreference::Review,
            ),
            "reviewer_roles.beta:gemini:gemini-2.5-pro",
        ),
        (
            review_kind_role(
                "critic",
                ReasoningParticipantRoleKind::Critic,
                ReasoningRoutePreference::Review,
            ),
            "reviewer_roles.alpha:claude:sonnet-4",
        ),
        (
            review_kind_role(
                "reviser",
                ReasoningParticipantRoleKind::Reviser,
                ReasoningRoutePreference::Review,
            ),
            "reviewer_roles.beta:gemini:gemini-2.5-pro",
        ),
    ];

    for (ordinal, (role, expected_route)) in expected.iter().enumerate() {
        let (effective_route, provider_family) =
            super::reasoning_route_for_role(role, &effective_routing, ordinal % 2);
        assert_eq!(effective_route, *expected_route);
        assert!(provider_family.is_some());
    }
}

#[test]
fn reasoning_route_for_arbiter_prefers_adjudication_slot() {
    let workspace = temp_workspace("boundline-runtime-reasoning-arbiter-route");
    let routing = RoutingConfig {
        review: Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() }),
        adjudication: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "o4-mini".to_string(),
        }),
        ..RoutingConfig::default()
    };
    save_local_routing(&workspace, routing);

    let effective_routing = super::effective_routing_for_workspace(&workspace);
    let role = review_kind_role(
        "arbiter",
        ReasoningParticipantRoleKind::Arbiter,
        ReasoningRoutePreference::Review,
    );

    let (effective_route, provider_family) =
        super::reasoning_route_for_role(&role, &effective_routing, 0);

    assert_eq!(effective_route, "adjudication:codex:o4-mini");
    assert_eq!(provider_family.as_deref(), Some("codex"));
}

#[test]
fn reasoning_independence_helpers_cover_gap_transitions_and_missing_dimensions() {
    let participants = sample_reasoning_participants();
    let observed = super::observed_reasoning_distinctness(&participants);
    assert_eq!(
        observed,
        ReasoningObservedDistinctness {
            distinct_routes: 2,
            distinct_providers: 2,
            distinct_contexts: 2,
            distinct_prompt_patterns: 2,
        }
    );
    assert_eq!(
        super::count_distinct_participant_values(&participants, |participant| {
            participant.provider_family.as_deref()
        }),
        2
    );

    let requested_floor = IndependenceFloor {
        route_distinct: true,
        provider_distinct: true,
        context_distinct: true,
        prompt_pattern_distinct: true,
        minimum_participants: 2,
    };
    let profile = independent_pair_review_profile();
    let passed_gaps = super::ReasoningIndependenceGaps::from_observed(
        &requested_floor,
        participants.len(),
        &observed,
    );
    assert_eq!(passed_gaps.result(false, false), IndependenceAssessmentResult::Passed);
    let passed_reason = super::reasoning_independence_reason(
        "bug-fix:investigate",
        &profile,
        participants.len(),
        requested_floor.minimum_participants,
        &observed,
        passed_gaps,
        IndependenceAssessmentResult::Passed,
    );
    assert!(passed_reason.contains("satisfies the requested independence"));

    let collapsed_observed = ReasoningObservedDistinctness {
        distinct_routes: 1,
        distinct_providers: 1,
        distinct_contexts: 1,
        distinct_prompt_patterns: 1,
    };
    let failed_gaps =
        super::ReasoningIndependenceGaps::from_observed(&requested_floor, 1, &collapsed_observed);
    assert_eq!(failed_gaps.result(false, false), IndependenceAssessmentResult::Failed);
    assert_eq!(failed_gaps.result(true, true), IndependenceAssessmentResult::Degraded);
    assert_eq!(
        failed_gaps.missing_dimensions(
            1,
            requested_floor.minimum_participants,
            &collapsed_observed,
        ),
        vec![
            "participants=1 < required=2".to_string(),
            "distinct_routes=1 < required=2".to_string(),
            "distinct_providers=1 < required=2".to_string(),
            "distinct_contexts=1 < required=2".to_string(),
            "distinct_prompt_patterns=1 < required=2".to_string(),
        ]
    );
    let failed_reason = super::reasoning_independence_reason(
        "bug-fix:investigate",
        &profile,
        1,
        requested_floor.minimum_participants,
        &collapsed_observed,
        failed_gaps,
        IndependenceAssessmentResult::Failed,
    );
    assert!(failed_reason.contains("participants=1 < required=2"));
    assert!(failed_reason.contains("distinct_prompt_patterns=1 < required=2"));
}

#[test]
fn reasoning_outcome_helpers_project_profile_specific_success_cases() {
    let participants = sample_reasoning_participants();
    let passed = IndependenceAssessment {
        requested_floor: IndependenceFloor {
            route_distinct: true,
            provider_distinct: true,
            context_distinct: true,
            prompt_pattern_distinct: true,
            minimum_participants: 2,
        },
        observed_distinctions: ReasoningObservedDistinctness {
            distinct_routes: 2,
            distinct_providers: 2,
            distinct_contexts: 2,
            distinct_prompt_patterns: 2,
        },
        result: IndependenceAssessmentResult::Passed,
        reason: "reasoning independence satisfied".to_string(),
    };

    let pair_outcome = super::reasoning_outcome_for_activation(
        "bug-fix:investigate",
        &independent_pair_review_profile(),
        &participants,
        &passed,
    )
    .expect("independent pair review should produce an adjudicated outcome");
    assert_eq!(pair_outcome.outcome_kind, ReasoningOutcomeKind::Adjudicated);

    let heterogeneous_outcome = super::reasoning_outcome_for_activation(
        "bug-fix:investigate",
        &reasoning_profile_with_id(ReasoningProfileId::HeterogeneousSecurityReview),
        &participants,
        &passed,
    )
    .expect("heterogeneous review should produce a converged outcome");
    assert_eq!(heterogeneous_outcome.outcome_kind, ReasoningOutcomeKind::Converged);

    let reflexion_outcome = super::reasoning_outcome_for_activation(
        "bug-fix:investigate",
        &reasoning_profile_with_id(ReasoningProfileId::BoundedReflexion),
        &participants,
        &passed,
    )
    .expect("bounded reflexion should produce a converged outcome");
    assert_eq!(reflexion_outcome.outcome_kind, ReasoningOutcomeKind::Converged);
    assert_eq!(reflexion_outcome.iterations.len(), 1);
    assert_eq!(
        reflexion_outcome.iterations[0].participants,
        participants
            .iter()
            .map(|participant| participant.participant_id.clone())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        reflexion_outcome.iterations[0].condition,
        crate::domain::reasoning::ReasoningIterationCondition::Completed
    );

    let active_outcome = super::reasoning_outcome_for_activation(
        "bug-fix:investigate",
        &reasoning_profile_with_id(ReasoningProfileId::BoundedSelfConsistency),
        &participants,
        &passed,
    );
    assert!(active_outcome.is_none());

    assert_eq!(
        super::reasoning_status_for_activation(&passed, Some(&pair_outcome)),
        ReasoningActivationStatus::Completed
    );
    assert_eq!(
        super::reasoning_status_for_activation(&passed, active_outcome.as_ref()),
        ReasoningActivationStatus::Active
    );

    let degraded = IndependenceAssessment {
        result: IndependenceAssessmentResult::Degraded,
        reason: "reasoning independence degraded".to_string(),
        ..passed.clone()
    };
    assert_eq!(
        super::reasoning_status_for_activation(&degraded, None),
        ReasoningActivationStatus::Degraded
    );

    let failed = IndependenceAssessment {
        result: IndependenceAssessmentResult::Failed,
        reason: "reasoning independence failed".to_string(),
        ..passed.clone()
    };
    assert_eq!(
        super::reasoning_status_for_activation(&failed, None),
        ReasoningActivationStatus::Blocked
    );

    let mut completed_participants = participants.clone();
    super::mark_reasoning_participants_completed(&mut completed_participants);
    assert!(
        completed_participants
            .iter()
            .all(|participant| participant.status == ReasoningParticipantStatus::Completed)
    );
    assert!(completed_participants[..2].iter().all(|participant| {
        participant
            .result_summary
            .as_deref()
            .is_some_and(|summary| summary.starts_with("completed via "))
    }));
    assert_eq!(completed_participants[2].result_summary.as_deref(), Some("already summarized"));
}

#[test]
fn guardian_phase_helpers_map_steps_and_decisions() {
    let workspace = temp_workspace("boundline-runtime-guardian-phases");
    let task = decision_task(workspace.to_string_lossy().as_ref(), json!({}));
    let mut session = build_session(&workspace, task);
    session.goal_plan = Some(
        GoalPlan::new(
            "Drive a session runtime branch",
            vec![
                PlannedTask {
                    task_id: "task-1".to_string(),
                    description: "Investigate the problem".to_string(),
                    target: "docs/brief.md".to_string(),
                    expected_outcome: None,
                    decision_type_hint: Some(DecisionType::Analyze),
                },
                PlannedTask {
                    task_id: "task-2".to_string(),
                    description: "Repair the implementation".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: None,
                    decision_type_hint: Some(DecisionType::Code),
                },
                PlannedTask {
                    task_id: "task-3".to_string(),
                    description: "Verify the bounded change".to_string(),
                    target: "tests/red_to_green.rs".to_string(),
                    expected_outcome: None,
                    decision_type_hint: Some(DecisionType::Test),
                },
                PlannedTask {
                    task_id: "task-4".to_string(),
                    description: "Replan the remaining work".to_string(),
                    target: "plan.md".to_string(),
                    expected_outcome: None,
                    decision_type_hint: Some(DecisionType::Replan),
                },
            ],
        )
        .unwrap(),
    );

    assert_eq!(SessionRuntime::guardian_phase_for_step(&session, 0), CapabilityPhase::Planning);
    assert_eq!(
        SessionRuntime::guardian_phase_for_step(&session, 1),
        CapabilityPhase::Implementation
    );
    assert_eq!(SessionRuntime::guardian_phase_for_step(&session, 2), CapabilityPhase::Verification);
    assert_eq!(SessionRuntime::guardian_phase_for_step(&session, 3), CapabilityPhase::Review);

    let decisions = vec![
        Decision::new(
            DecisionType::Analyze,
            "docs/brief.md",
            "collect bounded context",
            "bounded context collected",
            Vec::new(),
        ),
        Decision::new(
            DecisionType::Test,
            "tests/red_to_green.rs",
            "verify the change",
            "verification is recorded",
            Vec::new(),
        ),
        Decision::new(
            DecisionType::Replan,
            "plan.md",
            "tighten the next steps",
            "review phase is active",
            Vec::new(),
        ),
    ];

    assert_eq!(
        SessionRuntime::guardian_phase_for_decisions(&decisions),
        Some(CapabilityPhase::Review)
    );
    assert_eq!(SessionRuntime::guardian_phase_for_decisions(&[]), None);
}

#[test]
fn changed_files_for_guardian_prefers_state_then_evidence_then_targets() {
    let mut task = decision_task("/tmp/workspace", json!({}));
    task.context
        .state
        .insert("latest_changed_files".to_string(), json!(["src/lib.rs", "tests/red_to_green.rs"]));
    let step = Step::new(
        "step-state",
        StepKind::Decision,
        Some("src/from-step.rs".to_string()),
        json!({}),
    )
    .unwrap();
    let result = StepExecutionResult::success(json!({"ok": true}))
        .with_evidence(json!({"changed_files": ["src/from-evidence.rs"]}));

    assert_eq!(
        SessionRuntime::changed_files_for_guardian(&task, &result, &step, "workspace"),
        vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()]
    );

    let task = decision_task("/tmp/workspace", json!({}));
    let result = StepExecutionResult::success(json!({"ok": true}))
        .with_evidence(json!({"changed_files": ["src/from-evidence.rs"]}));
    assert_eq!(
        SessionRuntime::changed_files_for_guardian(&task, &result, &step, "workspace"),
        vec!["src/from-evidence.rs".to_string()]
    );

    let result = StepExecutionResult::success(json!({"ok": true}));
    assert_eq!(
        SessionRuntime::changed_files_for_guardian(&task, &result, &step, "workspace"),
        vec!["src/from-step.rs".to_string()]
    );

    let fallback_step = Step::decision("step-fallback", json!({})).unwrap();
    assert_eq!(
        SessionRuntime::changed_files_for_guardian(
            &task,
            &result,
            &fallback_step,
            "src/fallback.rs"
        ),
        vec!["src/fallback.rs".to_string()]
    );
}

#[test]
fn guardian_request_helpers_collect_targets_and_deduplicate_evidence() {
    let workspace = temp_workspace("boundline-runtime-guardian-requests");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "pub fn add() -> i32 { 2 }\n").unwrap();
    fs::write(workspace.join("tests/red_to_green.rs"), "#[test]\nfn add() {}\n").unwrap();

    let goal_plan = GoalPlan::new(
        "Drive a session runtime branch",
        vec![
            PlannedTask {
                task_id: "task-1".to_string(),
                description: "Repair arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: None,
                decision_type_hint: Some(DecisionType::Code),
            },
            PlannedTask {
                task_id: "task-2".to_string(),
                description: "Verify arithmetic".to_string(),
                target: "tests/red_to_green.rs".to_string(),
                expected_outcome: None,
                decision_type_hint: Some(DecisionType::Test),
            },
        ],
    )
    .unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    session.goal = Some("Drive a session runtime branch".to_string());
    session.goal_plan = Some(goal_plan.clone());

    let mut code = Decision::new(
        DecisionType::Code,
        "src/lib.rs",
        "repair arithmetic",
        "implementation is updated",
        vec![EvidenceRef::file("src/lib.rs")],
    );
    code.tool_result = Some(ToolResult::new("apply_patch", "apply_patch src/lib.rs", true, 10));

    let mut test = Decision::new(
        DecisionType::Test,
        "tests/red_to_green.rs",
        "verify the patch",
        "tests are green",
        vec![EvidenceRef::file("src/lib.rs"), EvidenceRef::tool_output("cargo test --quiet")],
    );
    test.tool_result = Some(ToolResult::new("cargo-test", "cargo test --quiet", true, 20));

    let request = runtime
        .native_guardian_request(&session, &goal_plan, &[code.clone(), test.clone()])
        .unwrap();
    assert_eq!(request.phase, CapabilityPhase::Verification);
    assert_eq!(
        request.changed_files,
        vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()]
    );
    assert_eq!(request.target_ref, "src/lib.rs");
    assert_eq!(
        request.evidence_refs.iter().filter(|reference| *reference == "src/lib.rs").count(),
        1
    );
    assert!(request.evidence_refs.iter().any(|reference| reference == "cargo test --quiet"));
    assert!(request.evidence_refs.iter().any(|reference| reference == "apply_patch src/lib.rs"));

    let planning_request = runtime
        .native_guardian_request(
            &session,
            &goal_plan,
            &[Decision::new(
                DecisionType::Analyze,
                "docs/brief.md",
                "collect context",
                "planning evidence is recorded",
                Vec::new(),
            )],
        )
        .unwrap();
    assert_eq!(planning_request.phase, CapabilityPhase::Planning);
    assert_eq!(
        planning_request.changed_files,
        vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()]
    );

    let step = Step::decision("guardian-step", json!({})).unwrap();
    let result = StepExecutionResult::success(json!({"ok": true}))
        .with_evidence(json!({"changed_files": ["src/lib.rs"]}));
    let task_ref = session.active_task.as_ref().unwrap();

    session.goal = None;
    let step_request = runtime.guardian_request_for_step(
        &session,
        task_ref,
        &step,
        CapabilityPhase::Implementation,
        &result,
    );
    assert_eq!(step_request.goal_text, task_ref.goal);
    assert_eq!(step_request.target_ref, "src/lib.rs");
    assert_eq!(step_request.changed_files, vec!["src/lib.rs".to_string()]);
    assert!(step_request.evidence_refs.iter().any(|reference| reference.contains("changed_files")));
}

#[test]
fn guardian_projection_merge_and_payload_helpers_preserve_planning_fields() {
    let finding = GuardianFinding {
        finding_id: "finding-1".to_string(),
        guardian_id: "verification_guardian".to_string(),
        rule_id: "verification".to_string(),
        disposition: GuardianDisposition::Warn,
        summary: "verification evidence is stale".to_string(),
        evidence_refs: vec!["tests/red_to_green.rs".to_string()],
        confidence: FindingConfidence::Medium,
        recommended_action: "rerun the bounded verification command".to_string(),
        authority_source: GuidanceAuthoritySource::WorkspaceOverride,
        source_ref: ".boundline/guardians/verification.toml".to_string(),
        phase: CapabilityPhase::Verification,
    };
    let mut projection = GuidanceGuardianProjection {
        capability_resolution_summary: Some("planning guidance selected".to_string()),
        loaded_guidance_sources: vec!["assistant/packs/shared/guidance/clean-code.md".to_string()],
        skipped_guidance_sources: vec![".canon/boundline/guidance (missing)".to_string()],
        ..GuidanceGuardianProjection::default()
    };
    let update = GuidanceGuardianProjection {
        capability_resolution_summary: Some("verification guidance selected".to_string()),
        loaded_packs: Vec::new(),
        skipped_packs: Vec::new(),
        catalog_validation_findings: Vec::new(),
        loaded_guidance_sources: vec![".boundline/guidance/local.md".to_string()],
        skipped_guidance_sources: vec![
            "assistant/packs/shared/guidance/clean-code.md (shadowed)".to_string(),
        ],
        loaded_guardian_sources: vec![".boundline/guardians/verification.toml".to_string()],
        skipped_guardian_sources: vec![
            "assistant/packs/shared/guardians/verification.toml (shadowed)".to_string(),
        ],
        guardian_timeline: vec!["verification_guardian: completed".to_string()],
        guardian_findings_summary: Some("1 guardian finding(s); blocking=false".to_string()),
        guardian_findings: vec![finding.clone()],
        guardian_degradations: vec!["verification route unavailable".to_string()],
        guardian_blocking_outcome: Some(
            "guardian findings recorded without a blocking outcome".to_string(),
        ),
    };

    SessionRuntime::merge_guardian_projection(&mut projection, &update);

    assert_eq!(
        projection.capability_resolution_summary.as_deref(),
        Some("planning guidance selected")
    );
    assert_eq!(
        projection.loaded_guidance_sources,
        vec!["assistant/packs/shared/guidance/clean-code.md".to_string()]
    );
    assert_eq!(projection.loaded_guardian_sources, update.loaded_guardian_sources);
    assert_eq!(projection.guardian_findings, vec![finding]);

    let mut payload = json!({"existing": "value"});
    SessionRuntime::append_guardian_projection_payload(&mut payload, &projection);
    assert_eq!(payload["existing"], "value");
    assert_eq!(payload["capability_resolution_summary"], "planning guidance selected");
    assert_eq!(payload["guardian_findings_summary"], "1 guardian finding(s); blocking=false");
    assert_eq!(payload["guardian_timeline"][0], "verification_guardian: completed");
    assert_eq!(
        payload["guardian_blocking_outcome"],
        "guardian findings recorded without a blocking outcome"
    );

    let mut scalar_payload = json!("skip");
    SessionRuntime::append_guardian_projection_payload(&mut scalar_payload, &projection);
    assert_eq!(scalar_payload, json!("skip"));
}

#[test]
fn native_delegation_for_goal_plan_covers_mismatch_handoff_and_escalation_paths() {
    let workspace = temp_workspace("boundline-runtime-native-delegation-paths");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let goal_plan = GoalPlan::new(
        "Drive a session runtime branch",
        vec![PlannedTask {
            task_id: "task-1".to_string(),
            description: "Repair arithmetic".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: Some(DecisionType::Code),
        }],
    )
    .unwrap()
    .with_flow(InferredFlow {
        flow_name: "bug-fix".to_string(),
        confidence_reason: "flow confirmed for native routing".to_string(),
        confirmed: true,
    });

    let mut mismatch = RoutingConfig::default();
    mismatch.set_slot(
        RouteSlot::Implementation,
        ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-4o".to_string() },
    );
    mismatch.assistant_runtimes = vec![RuntimeKind::Claude];
    mismatch.set_runtime_capability(
        RuntimeKind::Codex,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: None,
        },
    );
    save_local_routing(&workspace, mismatch);

    let (packet, continuity) = runtime.native_delegation_for_goal_plan(&goal_plan).unwrap();
    assert_eq!(packet.kind, crate::domain::session::DelegationPacketKind::Escalation);
    assert_eq!(packet.target_owner, "operator");
    assert_eq!(continuity.mode, DelegationContinuityMode::EscalationRequired);
    assert!(continuity.evidence_summary.contains("available assistant runtimes are: claude"));

    let mut handoff = RoutingConfig::default();
    handoff.set_slot(
        RouteSlot::Implementation,
        ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-4o".to_string() },
    );
    handoff.assistant_runtimes = vec![RuntimeKind::Codex, RuntimeKind::Claude];
    handoff.set_runtime_capability(
        RuntimeKind::Codex,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Unsupported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Unsupported,
            escalation_context: CapabilityState::Supported,
            notes: Some("implementation runtime cannot continue".to_string()),
        },
    );
    handoff.set_runtime_capability(
        RuntimeKind::Claude,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: None,
        },
    );
    handoff.set_slot_effort_policy(
        RouteSlot::Implementation,
        SlotEffortPolicy {
            level: EffortLevel::High,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: None,
        },
    );
    save_local_routing(&workspace, handoff);

    let (packet, continuity) = runtime.native_delegation_for_goal_plan(&goal_plan).unwrap();
    assert_eq!(packet.kind, crate::domain::session::DelegationPacketKind::Handoff);
    assert_eq!(packet.target_owner, "claude");
    assert_eq!(continuity.mode, DelegationContinuityMode::HandoffRequired);
    assert!(continuity.evidence_summary.contains("codex lacks continuation support"));

    let mut escalation = RoutingConfig::default();
    escalation.set_slot(
        RouteSlot::Implementation,
        ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-4o".to_string() },
    );
    escalation.assistant_runtimes = vec![RuntimeKind::Codex];
    escalation.set_runtime_capability(
        RuntimeKind::Codex,
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Unsupported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Unsupported,
            escalation_context: CapabilityState::Supported,
            notes: Some("operator escalation is still possible".to_string()),
        },
    );
    escalation.set_slot_effort_policy(
        RouteSlot::Implementation,
        SlotEffortPolicy {
            level: EffortLevel::High,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: None,
        },
    );
    save_local_routing(&workspace, escalation);

    let (packet, continuity) = runtime.native_delegation_for_goal_plan(&goal_plan).unwrap();
    assert_eq!(packet.kind, crate::domain::session::DelegationPacketKind::Escalation);
    assert_eq!(packet.target_owner, "operator");
    assert_eq!(continuity.mode, DelegationContinuityMode::EscalationRequired);
    assert_eq!(continuity.next_command, "boundline inspect");
}

#[test]
fn runtime_store_helpers_and_compatibility_planning_cover_private_accessors() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-store-helpers",
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
    );
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "left - right\n").unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);

    assert_eq!(runtime.workspace_ref(), workspace.as_path());
    assert!(runtime.latest_trace().unwrap().is_none());

    let session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    runtime.persist_session(&session).unwrap();
    assert!(runtime.session_store().load().unwrap().is_some());
    assert!(runtime.load_session().unwrap().is_some());

    let mut trace = ExecutionTrace::new("task-runtime-store", "session-runtime", "goal");
    trace.terminal_status = Some(TaskStatus::Succeeded);
    trace.terminal_reason =
        Some(TerminalReason::new(TerminalCondition::GoalSatisfied, "done", None));
    trace.ended_at = Some(trace.started_at + 1);
    runtime.trace_store().persist(&trace).unwrap();
    assert!(runtime.latest_trace().unwrap().is_some());

    let mut missing_goal =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    missing_goal.goal = None;
    assert!(matches!(
        runtime.plan_compatibility_task(&mut missing_goal),
        Err(super::SessionRuntimeError::MissingGoal)
    ));

    let mut planned =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    planned.goal = Some("Drive a session runtime branch".to_string());
    planned.active_flow = Some(built_in_flow("bug-fix").unwrap().initial_state());
    runtime.plan_compatibility_task(&mut planned).unwrap();
    let active_task_id = planned.active_task.as_ref().unwrap().id.clone();
    assert!(planned.goal_plan.is_none());
    assert_eq!(planned.latest_status, SessionStatus::Planned);

    runtime.ensure_flow_selected_compatibility_task(&mut planned).unwrap();
    assert_eq!(planned.active_task.as_ref().unwrap().id, active_task_id);

    runtime.clear_session().unwrap();
    assert!(runtime.load_session().unwrap().is_none());
}

#[test]
fn planning_context_sources_include_authored_documents_and_recent_change_signals() {
    let workspace = temp_workspace("boundline-runtime-planning-context");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(workspace.join("src/add.rs"), "pub fn add() -> i32 { 2 }\n").unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Focus on src/add.rs and tests/add.rs before broad scanning.\n",
    )
    .unwrap();

    let authored_brief = normalize_inputs(
        &workspace,
        Some("Fix the failing add test"),
        &[PathBuf::from("brief.md")],
    )
    .unwrap();
    let mut task = decision_task(workspace.to_string_lossy().as_ref(), json!({}));
    task.context.state.insert("latest_changed_files".to_string(), json!(["src/add.rs"]));
    task.context.state.insert("latest_validation_status".to_string(), json!("failed"));

    let mut session = build_session(&workspace, task);
    session.goal = Some("Fix the failing add test".to_string());
    session.authored_brief = Some(authored_brief);

    let runtime = SessionRuntime::for_workspace(&workspace);
    let sources = runtime.planning_context_sources(&session, "Fix the failing add test");

    assert!(
        sources.authored_input_documents.iter().any(|document| document.label.contains("brief.md")
            && document.content.contains("src/add.rs"))
    );
    assert_eq!(sources.latest_changed_files, vec!["src/add.rs".to_string()]);
    assert_eq!(sources.latest_validation_status.as_deref(), Some("failed"));
    assert!(sources.authored_input_sources.iter().any(|label| label.contains("brief.md")));

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn project_scale_helpers_classify_broad_goals_and_operational_entries() {
    let onboarding =
        project_scale_input_for_goal("Build a customer onboarding capability with audit logging")
            .expect("broad onboarding goal should be classified");
    assert!(!onboarding.existing_system_change);
    assert!(onboarding.problem_unclear);
    assert!(onboarding.product_scope_unclear);
    assert!(onboarding.capability_structure_unclear);
    assert!(onboarding.architecture_material);
    assert_eq!(onboarding.operational_entry, None);

    let existing = project_scale_input_for_goal("Modify the existing onboarding auth flow")
        .expect("existing system change should be classified");
    assert!(existing.existing_system_change);
    assert!(!existing.problem_unclear);
    assert!(!existing.product_scope_unclear);

    let supply_chain = project_scale_input_for_goal("Assess supply-chain risk before migration")
        .expect("supply-chain goal should be classified");
    assert_eq!(supply_chain.operational_entry, Some(ProjectScaleStageKind::SupplyChainAnalysis));

    let security = project_scale_input_for_goal("Run security review for the auth boundary")
        .expect("security goal should be classified");
    assert_eq!(security.operational_entry, Some(ProjectScaleStageKind::SecurityAssessment));

    let incident = project_scale_input_for_goal("Handle incident follow up for auth outage")
        .expect("incident goal should be classified");
    assert_eq!(incident.operational_entry, Some(ProjectScaleStageKind::Incident));

    let migration = project_scale_input_for_goal("Migrate onboarding state to the new schema")
        .expect("migration goal should be classified");
    assert_eq!(migration.operational_entry, Some(ProjectScaleStageKind::Migration));

    let system_assessment = project_scale_input_for_goal("Assess the system before broad refactor")
        .expect("system assessment goal should be classified");
    assert_eq!(system_assessment.operational_entry, Some(ProjectScaleStageKind::SystemAssessment));

    let platform_initiative =
        project_scale_input_for_goal("Drive a platform initiative for the billing project rollout")
            .expect("platform initiative should be classified as a broad goal");
    assert!(!platform_initiative.existing_system_change);
    assert!(platform_initiative.problem_unclear);
    assert!(platform_initiative.product_scope_unclear);
    assert!(platform_initiative.capability_structure_unclear);
    assert_eq!(platform_initiative.operational_entry, None);

    let long_goal = project_scale_input_for_goal(
        "Coordinate design notes across multiple teams before locking the delivery sequence",
    )
    .expect("long goals should be classified even without a named keyword");
    assert!(!long_goal.existing_system_change);
    assert!(long_goal.problem_unclear);
    assert!(long_goal.product_scope_unclear);
    assert!(!long_goal.capability_structure_unclear);
    assert_eq!(long_goal.operational_entry, None);

    let concrete_feature_goal = project_scale_input_for_goal(
        "Implement the first slice of a Rust user-management microservice with REST endpoints, gRPC methods, and OAuth2 authorization",
    )
    .expect("concrete feature goals should be classified as project-scale delivery work");
    assert!(!concrete_feature_goal.existing_system_change);
    assert!(!concrete_feature_goal.problem_unclear);
    assert!(concrete_feature_goal.product_scope_unclear);
    assert!(!concrete_feature_goal.capability_structure_unclear);
    assert!(concrete_feature_goal.architecture_material);
    assert_eq!(concrete_feature_goal.operational_entry, None);

    assert_eq!(project_scale_input_for_goal("Fix typo"), None);
}

#[test]
fn project_scale_state_uses_first_stage_and_work_unit_id() {
    let state = project_scale_state_for_goal(
        "Build a customer onboarding capability with audit logging",
        "confirm_project_scale_path",
    )
    .expect("broad goal should produce project-scale state");

    assert_eq!(state.active_stage_index, 0);
    assert_eq!(state.active_work_unit_id.as_deref(), Some("stage-001-discovery"));
    assert_eq!(state.next_action, "confirm_project_scale_path");
    assert_eq!(state.active_stage_text().as_deref(), Some("discovery"));
    assert!(state.path.stage_names().contains("pr-review"));
    assert!(state.checkpoint_refs.is_empty());
    assert!(state.trace_refs.is_empty());

    let security =
        project_scale_state_for_goal("Run security review for the auth boundary", "repair_context")
            .expect("security goal should produce project-scale state");
    assert_eq!(
        security.path.stages.first().map(|stage| stage.kind),
        Some(ProjectScaleStageKind::SecurityAssessment)
    );
    assert_eq!(security.next_action, "repair_context");

    let concrete_feature = project_scale_state_for_goal(
        "Implement the first slice of a Rust user-management microservice with REST endpoints, gRPC methods, and OAuth2 authorization",
        "confirm_project_scale_path",
    )
    .expect("concrete feature goal should produce project-scale state");
    assert_eq!(
        concrete_feature.path.stages.first().map(|stage| stage.kind),
        Some(ProjectScaleStageKind::Requirements)
    );
    assert_eq!(concrete_feature.active_work_unit_id.as_deref(), Some("stage-001-requirements"));
    assert_eq!(concrete_feature.active_stage_text().as_deref(), Some("requirements"));

    assert_eq!(project_scale_state_for_goal("Fix typo", "repair_context"), None);
}

#[test]
fn planning_context_sources_include_execution_profile_read_targets() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-execution-profile-targets",
        vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: "Replace subtraction with addition".to_string(),
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left - right".to_string(),
                replace: "left + right".to_string(),
            }],
            failure_mode: ExecutionFailureMode::Terminal,
        }],
        vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()],
        None,
    );

    let mut session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    session.goal = Some("Fix the failing add test".to_string());

    let runtime = SessionRuntime::for_workspace(&workspace);
    let sources = runtime.planning_context_sources(&session, "Fix the failing add test");

    assert_eq!(
        sources.execution_profile_read_targets,
        vec!["src/lib.rs".to_string(), "tests/red_to_green.rs".to_string()]
    );

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn planning_context_sources_fall_back_to_project_memory_surfaces() {
    let workspace = temp_workspace("boundline-runtime-project-memory");
    let project_dir = workspace.join("docs/project");
    let evidence_dir = workspace.join("docs/evidence/architecture/run-123");
    fs::create_dir_all(&project_dir).unwrap();
    fs::create_dir_all(&evidence_dir).unwrap();

    fs::write(
        project_dir.join("architecture-map.md"),
        "# Architecture Map\n\nStable Canon context.\n",
    )
    .unwrap();
    fs::write(evidence_dir.join("architecture-overview.md"), "overview\n").unwrap();
    fs::write(
        project_dir.join("architecture-map.packet-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "run_id": "run-123",
            "mode": "architecture",
            "risk": "bounded-impact",
            "zone": "yellow",
            "publish_timestamp": "2026-05-13T14:30:00Z",
            "descriptor": "architecture-map",
            "destination": "docs/project/architecture-map.md",
            "source_artifacts": ["architecture-overview.md"],
            "profile": "project-memory",
            "promotion_state": "auto-if-approved",
            "update_strategy": "managed-blocks",
            "lineage": {
                "contract_version": "v1",
                "producer": "canon",
                "source_ref": "canon-run:run-123",
                "mode": "architecture",
                "promotion_state": "auto-if-approved",
                "approval_state": "Completed",
                "packet_readiness": "complete",
                "promoted_at": "2026-05-13T14:30:00Z",
                "content_digest": "sha256:abc123",
                "promotion_profile": "project-memory",
                "source_artifacts": ["architecture-overview.md"]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    session.goal = Some("Plan the next bounded change".to_string());

    let runtime = SessionRuntime::for_workspace(&workspace);
    let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
    let memory = sources
        .compacted_canon_memory
        .expect("project memory should be compacted into planning sources");

    assert_eq!(memory.credibility, crate::domain::governance::MemoryCredibilityState::Credible);
    assert_eq!(memory.run_ref.as_deref(), Some("run-123"));
    assert!(memory.artifact_refs.contains(&"docs/project/architecture-map.md".to_string()));
    assert!(memory.artifact_refs.contains(&"docs/evidence/architecture/run-123".to_string()));

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn planning_context_sources_rejects_future_project_memory_contract_line() {
    let workspace = temp_workspace("boundline-runtime-project-memory-guidance");
    let project_dir = workspace.join("docs/project");
    fs::create_dir_all(&project_dir).unwrap();

    fs::write(
        project_dir.join("architecture-map.md"),
        "# Architecture Map\n\nStable Canon context.\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("architecture-map.packet-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "run_id": "run-123",
            "mode": "architecture",
            "risk": "bounded-impact",
            "zone": "yellow",
            "publish_timestamp": "2026-05-13T14:30:00Z",
            "descriptor": "architecture-map",
            "destination": "docs/project/architecture-map.md",
            "source_artifacts": ["architecture-overview.md"],
            "profile": "project-memory",
            "promotion_state": "auto-if-approved",
            "update_strategy": "managed-blocks",
            "lineage": {
                "contract_version": "v2",
                "producer": "canon",
                "source_ref": "canon-run:run-123",
                "mode": "architecture",
                "promotion_state": "auto-if-approved",
                "approval_state": "Completed",
                "packet_readiness": "complete",
                "promoted_at": "2026-05-13T14:30:00Z",
                "content_digest": "sha256:abc123",
                "promotion_profile": "project-memory",
                "source_artifacts": ["architecture-overview.md"]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    session.goal = Some("Plan the next bounded change".to_string());

    let runtime = SessionRuntime::for_workspace(&workspace);
    let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
    let memory = sources
        .compacted_canon_memory
        .expect("unsupported project memory should still surface repair guidance");

    assert_eq!(memory.credibility, crate::domain::governance::MemoryCredibilityState::Insufficient);
    assert_eq!(memory.reason_code.as_deref(), Some("project_memory_contract_incompatible"));
    assert_eq!(
        memory.recommended_next_action.as_ref().map(|action| action.action.as_str()),
        Some("update")
    );

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn planning_context_sources_block_on_incompatible_project_memory_contract() {
    let workspace = temp_workspace("boundline-runtime-project-memory-incompatible");
    let project_dir = workspace.join("docs/project");
    fs::create_dir_all(&project_dir).unwrap();

    fs::write(
        project_dir.join("architecture-map.md"),
        "# Architecture Map\n\nIncompatible Canon context.\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("architecture-map.packet-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "run_id": "run-999",
            "mode": "architecture",
            "risk": "bounded-impact",
            "zone": "yellow",
            "publish_timestamp": "2026-05-13T14:30:00Z",
            "descriptor": "architecture-map",
            "destination": "docs/project/architecture-map.md",
            "source_artifacts": ["architecture-overview.md"],
            "profile": "project-memory",
            "promotion_state": "auto-if-approved",
            "update_strategy": "managed-blocks",
            "lineage": {
                "contract_version": "v2",
                "producer": "canon",
                "source_ref": "canon-run:run-999",
                "mode": "architecture",
                "promotion_state": "auto-if-approved",
                "approval_state": "Completed",
                "packet_readiness": "complete",
                "promoted_at": "2026-05-13T14:30:00Z",
                "content_digest": "sha256:def456",
                "promotion_profile": "project-memory",
                "source_artifacts": ["architecture-overview.md"]
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let mut session =
        build_session(&workspace, decision_task(workspace.to_string_lossy().as_ref(), json!({})));
    session.goal = Some("Plan the next bounded change".to_string());

    let runtime = SessionRuntime::for_workspace(&workspace);
    let sources = runtime.planning_context_sources(&session, "Plan the next bounded change");
    let memory = sources
        .compacted_canon_memory
        .expect("incompatible project memory should still surface repair guidance");

    assert_eq!(memory.credibility, crate::domain::governance::MemoryCredibilityState::Insufficient);
    assert_eq!(memory.reason_code.as_deref(), Some("project_memory_contract_incompatible"));
    assert_eq!(
        memory.recommended_next_action.as_ref().map(|action| action.action.as_str()),
        Some("update")
    );

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn compacted_project_memory_maps_non_credible_states_to_actions() {
    let workspace = temp_workspace("boundline-runtime-project-memory-states");
    let cases = [
        (
            PromotionStateView::PendingOrIndex,
            "Canon project memory is pending",
            "project_memory_pending",
            "refresh",
        ),
        (
            PromotionStateView::EvidenceOnly,
            "Canon project memory is evidence-only",
            "project_memory_evidence_only",
            "promote",
        ),
        (
            PromotionStateView::Manual,
            "Canon project memory requires manual promotion",
            "project_memory_manual",
            "promote",
        ),
        (
            PromotionStateView::Unknown,
            "Canon project memory metadata is incomplete",
            "project_memory_unknown",
            "inspect",
        ),
    ];

    for (state, headline, reason_code, action) in cases {
        let context = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: PathBuf::from("docs/project/overview.md"),
                lineage: Some(sample_project_memory_lineage("run-123", "architecture")),
                promotion_view: state,
                category: "overview".to_string(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(state),
        };

        let memory = SessionRuntime::compacted_canon_memory_from_project_memory_context(
            &workspace, &context,
        )
        .expect("non-credible project memory should still compact");

        assert_eq!(memory.headline, headline);
        assert_eq!(memory.credibility, crate::domain::governance::MemoryCredibilityState::Stale);
        assert_eq!(memory.reason_code.as_deref(), Some(reason_code));
        assert_eq!(memory.run_ref.as_deref(), Some("run-123"));
        assert_eq!(memory.possible_actions[0].action, action);
        assert_eq!(
            memory.recommended_next_action.as_ref().map(|next| next.action.as_str()),
            Some(action)
        );
        assert_eq!(
            memory
                .evidence_summary
                .as_ref()
                .map(|summary| summary.artifact_provenance_links.clone()),
            Some(vec!["docs/project/overview.md".to_string()])
        );
    }

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn compacted_project_memory_maps_hard_stop_states_to_actions() {
    let workspace = temp_workspace("boundline-runtime-project-memory-hard-stops");
    let cases = [
        (
            {
                let mut lineage = sample_project_memory_lineage("run-awaiting", "architecture");
                lineage.promotion_state = "auto-if-approved".to_string();
                lineage.approval_state = Some("requested".to_string());
                lineage.packet_readiness = Some("pending".to_string());
                lineage
            },
            PromotionStateView::PendingOrIndex,
            "Canon project memory is waiting for required approval",
            "project_memory_missing_approval",
            "approve",
        ),
        (
            {
                let mut lineage = sample_project_memory_lineage("run-blocked", "architecture");
                lineage.promotion_state = "auto-if-approved".to_string();
                lineage.approval_state = Some("rejected".to_string());
                lineage.packet_readiness = Some("rejected".to_string());
                lineage
            },
            PromotionStateView::PendingOrIndex,
            "Canon project memory reports blocked governance",
            "project_memory_blocked",
            "unblock",
        ),
        (
            {
                let mut lineage =
                    sample_project_memory_lineage("run-missing-artifact", "architecture");
                lineage.source_artifacts = vec!["architecture-overview.md".to_string()];
                lineage
            },
            PromotionStateView::Stable,
            "Canon project memory is missing required source artifacts",
            "project_memory_missing_source_artifacts",
            "restore",
        ),
    ];

    for (lineage, state, headline, reason_code, action) in cases {
        let context = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: PathBuf::from("docs/project/overview.md"),
                lineage: Some(lineage),
                promotion_view: state,
                category: "overview".to_string(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(state),
        };

        let memory = SessionRuntime::compacted_canon_memory_from_project_memory_context(
            &workspace, &context,
        )
        .expect("hard-stop project memory should still compact");

        assert_eq!(memory.headline, headline);
        assert_eq!(
            memory.credibility,
            crate::domain::governance::MemoryCredibilityState::Insufficient
        );
        assert_eq!(memory.reason_code.as_deref(), Some(reason_code));
        assert_eq!(memory.possible_actions[0].action, action);
        assert_eq!(
            memory.recommended_next_action.as_ref().map(|next| next.action.as_str()),
            Some(action)
        );
    }

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn project_memory_artifact_refs_skip_missing_and_duplicate_evidence_roots() {
    let workspace = temp_workspace("boundline-runtime-project-memory-artifact-refs");
    fs::create_dir_all(workspace.join("docs/evidence/architecture/run-123")).unwrap();

    let existing_lineage = sample_project_memory_lineage("run-123", "architecture");
    let missing_lineage =
        LineageRef { source_ref: "canon-run:run-missing".to_string(), ..existing_lineage.clone() };

    let context = ProjectMemoryContext {
        status: ProjectMemoryStatus::Available,
        compatibility: Some(CompatibilityOutcome::Compatible),
        surfaces: vec![ProjectMemorySurface {
            path: PathBuf::from("docs/project/architecture-map.md"),
            lineage: None,
            promotion_view: PromotionStateView::Stable,
            category: "architecture-map".to_string(),
        }],
        evidence_refs: vec![existing_lineage.clone(), existing_lineage, missing_lineage],
        effective_promotion_state: Some(PromotionStateView::Stable),
    };

    let refs = SessionRuntime::project_memory_artifact_refs(&workspace, &context);

    assert_eq!(
        refs,
        vec![
            "docs/project/architecture-map.md".to_string(),
            "docs/evidence/architecture/run-123".to_string(),
        ]
    );

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn compacted_project_memory_carries_managed_block_attribution() {
    let workspace = temp_workspace("boundline-runtime-project-memory-managed-blocks");
    let evidence_dir = workspace.join("docs/evidence/architecture/run-123");
    fs::create_dir_all(&evidence_dir).unwrap();
    fs::write(
            evidence_dir.join("verification.md"),
            concat!(
                "<!-- project-memory:managed:start producer=\"canon\" source_ref=\"canon-run:run-123\" contract_version=\"v1\" -->\n",
                "Canon evidence\n",
                "<!-- project-memory:managed:end -->\n",
                "<!-- project-memory:managed:start producer=\"boundline\" source_ref=\"trace-9\" contract_version=\"v1\" -->\n",
                "Boundline evidence\n",
                "<!-- project-memory:managed:end -->\n"
            ),
        )
        .unwrap();

    let lineage = sample_project_memory_lineage("run-123", "architecture");
    let context = ProjectMemoryContext {
        status: ProjectMemoryStatus::Available,
        compatibility: Some(CompatibilityOutcome::Compatible),
        surfaces: vec![ProjectMemorySurface {
            path: PathBuf::from("docs/project/overview.md"),
            lineage: Some(lineage.clone()),
            promotion_view: PromotionStateView::Stable,
            category: "overview".to_string(),
        }],
        evidence_refs: vec![lineage],
        effective_promotion_state: Some(PromotionStateView::Stable),
    };

    let memory =
        SessionRuntime::compacted_canon_memory_from_project_memory_context(&workspace, &context)
            .expect("project memory with evidence attribution should compact");

    let carried_forward_items = memory
        .evidence_summary
        .as_ref()
        .map(|summary| summary.carried_forward_items.clone())
        .unwrap_or_default();
    assert_eq!(carried_forward_items.len(), 2);
    assert!(carried_forward_items.iter().any(|summary| summary.contains("producer=canon")));
    assert!(carried_forward_items.iter().any(|summary| summary.contains("producer=boundline")));

    fs::remove_dir_all(workspace).unwrap();
}

#[test]
fn execute_step_routes_agent_tool_and_decision_edge_cases() {
    let runtime = SessionRuntime::for_workspace(temp_workspace("boundline-runtime-routing"));
    let fixture_runtime = manual_runtime();
    let context = context();

    let mut missing_agent_target = Step::agent("agent", "coder", json!({})).unwrap();
    missing_agent_target.target_name = None;
    let missing_agent = runtime.execute_step(&fixture_runtime, &missing_agent_target, &context);
    assert_eq!(missing_agent.status, ExecutionStatus::Failed);
    assert_eq!(missing_agent.recoverability, Recoverability::Terminal);

    let unknown_agent = runtime.execute_step(
        &fixture_runtime,
        &Step::agent("agent", "unknown", json!({})).unwrap(),
        &context,
    );
    assert_eq!(unknown_agent.status, ExecutionStatus::Failed);

    let mut missing_tool_target = Step::tool("tool", "tester", json!({})).unwrap();
    missing_tool_target.target_name = None;
    let missing_tool = runtime.execute_step(&fixture_runtime, &missing_tool_target, &context);
    assert_eq!(missing_tool.status, ExecutionStatus::Failed);

    let unknown_tool = runtime.execute_step(
        &fixture_runtime,
        &Step::tool("tool", "unknown", json!({})).unwrap(),
        &context,
    );
    assert_eq!(unknown_tool.status, ExecutionStatus::Failed);

    let plain_decision = runtime.execute_decision(&Step::decision("plain", json!("ok")).unwrap());
    assert_eq!(plain_decision.status, ExecutionStatus::Succeeded);

    let retry_decision = runtime
        .execute_decision(&Step::decision("retry", json!({"retryable_failure": true})).unwrap());
    assert_eq!(retry_decision.recoverability, Recoverability::Retryable);

    let replan_decision = runtime
        .execute_decision(&Step::decision("replan", json!({"replan_required": true})).unwrap());
    assert_eq!(replan_decision.recoverability, Recoverability::ReplanRequired);

    let terminal_decision = runtime
        .execute_decision(&Step::decision("terminal", json!({"terminal_failure": true})).unwrap());
    assert_eq!(terminal_decision.recoverability, Recoverability::Terminal);

    let patched_decision = runtime.execute_decision(
        &Step::decision(
            "patched",
            json!({"output": {"ok": true}, "state_patch": {"goal_satisfied": true}}),
        )
        .unwrap(),
    );
    assert_eq!(patched_decision.status, ExecutionStatus::Succeeded);
    assert_eq!(patched_decision.state_patch.as_ref().unwrap()["goal_satisfied"], json!(true));

    assert_eq!(
        runtime.session_store().path(),
        runtime.workspace_ref().join(".boundline/session.json")
    );
    assert_eq!(runtime.trace_store().root(), runtime.workspace_ref().join(".boundline/traces"));
    assert_eq!(session_status_for_task_status(TaskStatus::Aborted), SessionStatus::Aborted);

    let mut workspace_routing = RoutingConfig {
        assistant_runtimes: vec![RuntimeKind::Copilot],
        ..RoutingConfig::default()
    };
    let mut cluster_routing =
        RoutingConfig { assistant_runtimes: vec![RuntimeKind::Codex], ..RoutingConfig::default() };
    let global_routing =
        RoutingConfig { assistant_runtimes: vec![RuntimeKind::Claude], ..RoutingConfig::default() };
    assert_eq!(
        effective_assistant_runtimes(
            Some(&workspace_routing),
            Some(&cluster_routing),
            Some(&global_routing)
        ),
        vec![RuntimeKind::Copilot]
    );
    workspace_routing.assistant_runtimes.clear();
    assert_eq!(
        effective_assistant_runtimes(
            Some(&workspace_routing),
            Some(&cluster_routing),
            Some(&global_routing)
        ),
        vec![RuntimeKind::Codex]
    );
    cluster_routing.assistant_runtimes.clear();
    assert_eq!(
        effective_assistant_runtimes(
            Some(&workspace_routing),
            Some(&cluster_routing),
            Some(&global_routing)
        ),
        vec![RuntimeKind::Claude]
    );

    let cluster_ready = write_execution_profile_workspace(
        "boundline-runtime-cluster-ready",
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
    );
    fs::create_dir_all(cluster_ready.join("src")).unwrap();
    fs::write(cluster_ready.join("src/lib.rs"), "left - right\n").unwrap();
    assert!(!cluster_workspace_is_blocked(cluster_ready.to_string_lossy().as_ref()));
    assert!(cluster_workspace_is_blocked(
        temp_workspace("boundline-runtime-cluster-blocked").to_string_lossy().as_ref()
    ));
    assert_eq!(cluster_task_status_text(TaskStatus::Exhausted), "exhausted");
    assert!(is_governance_trace_event(TraceEventType::GovernanceBlocked));
    assert!(!is_governance_trace_event(TraceEventType::TaskStarted));
}

#[test]
fn load_or_create_trace_and_flow_helpers_cover_private_flow_branches() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-flow-helpers",
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
    );
    let runtime = SessionRuntime::for_workspace(&workspace);

    let flow = built_in_flow("bug-fix").unwrap();
    let stage0 = Step::agent(
        "investigate",
        "analyzer",
        attach_stage_metadata(json!({"phase": "investigate"}), flow, 0).unwrap(),
    )
    .unwrap();
    let stage1 = Step::agent(
        "implement",
        "coder",
        attach_stage_metadata(json!({"phase": "implement"}), flow, 1).unwrap(),
    )
    .unwrap();
    let request = build_request(workspace.to_string_lossy().as_ref());
    let task =
        Task::new("task-flow", &request, Plan::new(vec![stage0.clone(), stage1.clone()]).unwrap())
            .unwrap();
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Drive a session runtime branch".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(flow.initial_state()),
        active_task: Some(task.clone()),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    let created = runtime.load_or_create_trace(&mut session, &task).unwrap();
    assert_eq!(created.events[0].event_type, TraceEventType::TaskStarted);
    assert_eq!(created.events[1].event_type, TraceEventType::FlowSelected);

    let reused = runtime.load_or_create_trace(&mut session, &task).unwrap();
    assert_eq!(reused.goal, created.goal);

    let transition = runtime.advance_session_flow(&mut session, &task, 0).unwrap().unwrap();
    assert_eq!(transition.0.stage_id, "investigate");
    assert_eq!(transition.1.stage_id, "implement");
    assert_eq!(session.active_flow.as_ref().unwrap().current_stage_id, "implement");

    let payload = runtime.flow_payload_for_step(&stage0).unwrap().unwrap();
    assert_eq!(payload["stage_id"], json!("investigate"));
    assert_eq!(
        runtime.flow_payload_for_step(&Step::decision("plain", json!({})).unwrap()).unwrap(),
        None
    );

    let mut trace = ExecutionTrace::new("task-flow", "session-runtime", "goal");
    runtime.record_stage_failure(
        &mut trace,
        &session,
        "implement",
        0,
        &TerminalReason::new(TerminalCondition::UnrecoverableError, "failed", None),
    );
    assert_eq!(trace.events[0].event_type, TraceEventType::StageFailed);
}

#[test]
fn session_lifecycle_helpers_cover_capture_selection_planning_and_cluster_projection() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-lifecycle-helpers",
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
    );
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "left - right\n").unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: vec![crate::domain::decision::Decision::new(
            crate::domain::decision::DecisionType::Analyze,
            "src/lib.rs",
            "inspect the file",
            "bounded context collected",
            Vec::new(),
        )],
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: Some(TerminalReason::new(
            TerminalCondition::TaskNotCredible,
            "stale",
            None,
        )),
        latest_trace_ref: Some("trace.json".to_string()),
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    assert!(matches!(
        runtime.capture_goal(&mut session, "   "),
        Err(super::SessionRuntimeError::MissingGoal)
    ));

    runtime.capture_goal(&mut session, "  Drive a session runtime branch  ").unwrap();
    assert_eq!(session.goal.as_deref(), Some("Drive a session runtime branch"));
    assert!(session.negotiation_packet.is_some());
    assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
    assert!(session.decisions.is_empty());
    assert!(session.latest_terminal_reason.is_none());
    assert!(session.latest_trace_ref.is_none());

    runtime.select_flow(&mut session, "bug-fix").unwrap();
    assert_eq!(session.active_flow.as_ref().unwrap().flow_name, "bug-fix");
    assert!(session.active_flow_policy.is_some());
    assert_eq!(session.latest_status, SessionStatus::GoalCaptured);
    assert!(!runtime.uses_native_goal_plan(&session).unwrap());
    assert!(matches!(
        runtime.confirm_goal_plan(&mut session),
        Err(super::SessionRuntimeError::MissingGoalPlan)
    ));

    assert!(matches!(
        runtime.plan_task(&mut session, Some("missing"), false),
        Err(super::SessionRuntimeError::UnknownFlow { .. })
    ));

    session.active_task =
        Some(decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})));
    let previous_task_id = session.active_task.as_ref().unwrap().id.clone();
    runtime.plan_task(&mut session, None, false).unwrap();
    assert_ne!(session.active_task.as_ref().unwrap().id, previous_task_id);
    assert!(matches!(
        runtime.select_flow(&mut session, "delivery"),
        Err(super::SessionRuntimeError::FlowReplacementRequiresReset { .. })
    ));

    session.active_task = None;
    session.goal_plan = Some(
        GoalPlan::new(
            "Drive a session runtime branch",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Repair arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap(),
    );
    assert!(matches!(
        runtime.select_flow(&mut session, "delivery"),
        Err(super::SessionRuntimeError::FlowReplacementRequiresReset { .. })
    ));

    runtime.confirm_goal_plan(&mut session).unwrap();
    assert!(!session.goal_plan.as_ref().unwrap().requires_confirmation());
    assert_eq!(session.latest_status, SessionStatus::Planned);
    assert!(runtime.uses_native_goal_plan(&session).unwrap());

    let mut gated_session = session.clone();
    gated_session.goal_plan = Some(
        GoalPlan::new(
            "Drive a session runtime branch",
            vec![PlannedTask {
                task_id: "planned-task-gated".to_string(),
                description: "Repair arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap(),
    );
    gated_session.governance_lifecycle = Some(GovernedSessionLifecycle {
        governance_runtime: GovernanceRuntimeKind::Canon,
        explicit_opt_out: false,
        mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
        selected_mode: Some(CanonMode::Requirements),
        selected_mode_sequence: vec![CanonMode::Requirements, CanonMode::Architecture],
        latest_reasoning_profile: None,
        current_stage_index: 0,
        stage_records: vec![GovernedStageRecord {
            stage_key: "plan:requirements".to_string(),
            runtime: GovernanceRuntimeKind::Canon,
            lifecycle_state: GovernanceLifecycleState::AwaitingApproval,
            required: true,
            autopilot_enabled: false,
            approval_state: ApprovalState::Requested,
            canon_run_ref: Some("canon-run-plan".to_string()),
            governance_attempt_id: "attempt-plan-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: Some(".canon/planning-packet".to_string()),
            decision_ref: None,
            stage_council: None,
            blocked_reason: Some("waiting for Canon approval".to_string()),
        }],
        accumulated_context: Vec::new(),
        terminal_reason: Some("awaiting approval: waiting for Canon approval".to_string()),
        planning_input_fingerprint: None,
    });
    assert!(matches!(
        runtime.confirm_goal_plan(&mut gated_session),
        Err(super::SessionRuntimeError::PlanningGovernanceUnresolved { .. })
    ));

    session.active_task =
        Some(decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})));
    let projection = ClusterSessionProjection {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: workspace.to_string_lossy().into_owned(),
        member_workspace_refs: vec![workspace.to_string_lossy().into_owned()],
        started_from_command: "boundline cluster status".to_string(),
        updated_at: 10,
    };
    runtime.prepare_cluster_run(&mut session, &projection).unwrap();
    assert_eq!(
        session
            .active_task
            .as_ref()
            .unwrap()
            .context
            .cluster_session_projection()
            .unwrap()
            .unwrap(),
        projection
    );
    assert_eq!(
        session.goal_plan.as_ref().unwrap().cluster_session_projection.as_ref().unwrap(),
        &projection
    );
}

#[test]
fn broad_goal_planning_persists_project_scale_state_when_context_is_insufficient() {
    let workspace = temp_workspace("boundline-runtime-project-scale-clarify");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime
        .capture_goal(&mut session, "Build a customer onboarding capability with audit logging")
        .unwrap();

    let error = runtime.plan_task(&mut session, None, false).unwrap_err();
    let rendered_error = error.to_string();
    let prompt = rendered_error
            .strip_prefix(
                "active session requires clarification before planning can continue: bounded context required before planning: ",
            )
            .expect("plan_task should return clarification-required details");
    assert!(!prompt.trim().is_empty());

    assert_eq!(session.latest_status, SessionStatus::Blocked);
    assert!(session.goal_plan.is_some());
    let project_scale = session.project_scale.expect("project scale state should be persisted");
    assert_eq!(project_scale.next_action, "repair_context");
    assert_eq!(project_scale.active_stage_text().as_deref(), Some("discovery"));
    assert!(project_scale.path.stage_names().contains("pr-review"));
}

#[test]
fn plan_task_blocks_when_plan_quality_detects_stale_context() {
    let workspace = temp_workspace("boundline-runtime-plan-quality-stale-context");
    fs::write(workspace.join("package.json"), r#"{"dependencies":{"react":"18.0.0"}}"#).unwrap();
    fs::create_dir_all(workspace.join("src/components")).unwrap();
    fs::create_dir_all(workspace.join("design")).unwrap();
    fs::write(workspace.join("design/reference.md"), "button guidance\n").unwrap();
    thread::sleep(Duration::from_millis(20));
    fs::write(
        workspace.join("src/components/App.tsx"),
        "export function App() { return <button>Save</button>; }\n",
    )
    .unwrap();

    save_local_routing(
        &workspace,
        RoutingConfig {
            domain_templates: BTreeMap::from([(
                DomainFamily::React,
                DomainTemplateSettings {
                    enabled: Some(true),
                    standards: Some("workspace react standards".to_string()),
                    external_context_bindings: vec![ExternalContextBinding {
                        kind: ExternalContextKind::DesignReference,
                        reference: "design/reference.md".to_string(),
                        required: true,
                        notes: None,
                    }],
                },
            )]),
            ..RoutingConfig::default()
        },
    );

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime-stale-context".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime
        .capture_goal(
            &mut session,
            "Refresh src/components/App.tsx against the latest design guidance",
        )
        .unwrap();

    let error = runtime.plan_task(&mut session, None, false).unwrap_err();
    let rendered_error = error.to_string();

    assert!(rendered_error.contains("required external context is stale"), "{rendered_error}");
    assert_eq!(session.latest_status, SessionStatus::Blocked);
    let goal_plan =
        session.goal_plan.as_ref().expect("blocked planning should persist the goal plan");
    assert_eq!(goal_plan.plan_quality_state().as_deref(), Some("blocked"));
    assert_eq!(goal_plan.plan_quality_findings().unwrap(), vec!["context_pack_stale".to_string()]);
}

#[test]
fn plan_goal_plan_populates_canon_mode_sequence_from_selected_flow() {
    let workspace = temp_workspace("boundline-runtime-plan-canon-sequence");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    session.authored_brief = Some(
        normalize_inputs_with_governance(
            &workspace,
            Some("Deliver a governed feature"),
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
        .unwrap(),
    );
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.expect("canon lifecycle should exist");
    assert_eq!(lifecycle.selected_mode, Some(CanonMode::Requirements));
    assert_eq!(
        lifecycle.selected_mode_sequence,
        vec![
            CanonMode::Requirements,
            CanonMode::SystemShaping,
            CanonMode::Architecture,
            CanonMode::Backlog,
            CanonMode::Implementation,
        ]
    );
    assert!(session.goal_plan.is_some());
}

#[test]
fn prepare_planning_governance_requests_materializes_stage_briefs_for_delivery_flow() {
    let workspace = temp_workspace("boundline-runtime-planning-governance-requests");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let goal = session.goal.clone().unwrap();
    let context_sources = runtime.planning_context_sources(&session, &goal);
    let goal_plan = session.goal_plan.as_ref().unwrap().clone();
    let requests = runtime
        .prepare_planning_governance_requests(&mut session, &goal_plan, &context_sources)
        .unwrap();

    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].request.stage_key, "plan:requirements");
    assert_eq!(requests[0].request.mode, Some(CanonMode::Requirements));
    assert_eq!(requests[0].request.risk.as_deref(), Some("bounded-impact"));
    assert_eq!(requests[0].request.zone.as_deref(), Some("yellow"));
    assert_eq!(requests[0].request.owner.as_deref(), Some("delivery-engineer"));
    assert_eq!(requests[1].request.stage_key, "plan:system-shaping");
    assert_eq!(requests[1].request.mode, Some(CanonMode::SystemShaping));
    assert_eq!(requests[3].request.stage_key, "plan:backlog");
    assert_eq!(requests[3].request.system_context, Some(SystemContextBinding::Existing));

    let stage_brief_ref = requests[0]
        .request
        .bounded_context
        .stage_brief_ref
        .as_deref()
        .expect("planning request should reference a stage brief");
    let stage_brief_path = workspace.join(stage_brief_ref);
    assert!(stage_brief_path.exists());

    let stage_brief = fs::read_to_string(stage_brief_path).unwrap();
    assert!(stage_brief.contains("stage_key: plan:requirements"));
    assert!(stage_brief.contains("canon_mode: requirements"));
    assert!(stage_brief.contains("goal: Deliver a governed feature"));
    assert!(stage_brief.contains("## Problem Domain"));
    assert!(stage_brief.contains("## Known Facts"));
    assert!(stage_brief.contains("## Unknowns"));
    assert!(stage_brief.contains("## Assumptions"));
    assert!(stage_brief.contains("## Validation Targets"));
    assert!(stage_brief.contains("## Confidence Levels"));
    assert!(stage_brief.contains("## Discovery Handoff"));
}

#[cfg(unix)]
#[test]
fn plan_task_executes_canon_planning_requests_and_persists_stage_records() {
    let workspace = temp_workspace("boundline-runtime-plan-canon-execution");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "session-runtime-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(
        lifecycle.stage_records.len(),
        4,
        "stage_records={:#?}\nselected_mode_sequence={:#?}\nterminal_reason={:#?}",
        lifecycle.stage_records,
        lifecycle.selected_mode_sequence,
        lifecycle.terminal_reason,
    );
    assert_eq!(lifecycle.stage_records[0].stage_key, "plan:requirements");
    assert_eq!(lifecycle.stage_records[1].stage_key, "plan:system-shaping");
    assert_eq!(lifecycle.stage_records[2].stage_key, "plan:architecture");
    assert_eq!(lifecycle.stage_records[3].stage_key, "plan:backlog");
    assert!(lifecycle.stage_records.iter().all(|record| {
        record.lifecycle_state == GovernanceLifecycleState::GovernedReady
            && record.packet_ref.as_deref() == Some(".canon/planning-packet")
            && record.canon_run_ref.as_deref() == Some("canon-run-plan")
    }));
    assert_eq!(lifecycle.accumulated_context.len(), 4);
    assert!(lifecycle.terminal_reason.is_none());
    assert_eq!(lifecycle.current_stage_index, 4);
    assert_eq!(
        session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.compacted_canon_memory.as_ref())
            .and_then(|memory| memory.stage_key.as_deref()),
        Some("plan:backlog")
    );
}

#[cfg(unix)]
#[test]
fn plan_task_skips_canon_for_completed_planning_stages() {
    let workspace = temp_workspace("boundline-runtime-plan-canon-completed-skip");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "session-runtime-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime-completed".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    // All 4 stages should be GovernedReady after the first plan_task call.
    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    assert!(
        lifecycle
            .stage_records
            .iter()
            .all(|record| record.lifecycle_state == GovernanceLifecycleState::GovernedReady)
    );

    // Simulate complete_planning_stage: mark first two stages as Completed.
    let lifecycle = session.governance_lifecycle.as_mut().unwrap();
    lifecycle.stage_records[0].lifecycle_state = GovernanceLifecycleState::Completed;
    lifecycle.stage_records[1].lifecycle_state = GovernanceLifecycleState::Completed;

    // Re-plan: the fix ensures Completed stages are skipped, not re-executed.
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    assert_eq!(
        lifecycle.stage_records[0].lifecycle_state,
        GovernanceLifecycleState::Completed,
        "first completed stage should remain Completed, not re-executed"
    );
    assert_eq!(
        lifecycle.stage_records[1].lifecycle_state,
        GovernanceLifecycleState::Completed,
        "second completed stage should remain Completed, not re-executed"
    );
    assert_eq!(lifecycle.stage_records[2].lifecycle_state, GovernanceLifecycleState::GovernedReady);
    assert_eq!(lifecycle.stage_records[3].lifecycle_state, GovernanceLifecycleState::GovernedReady);
}

#[cfg(unix)]
#[test]
fn plan_task_adopts_workspace_canon_governance_without_explicit_session_selection() {
    let workspace = temp_workspace("boundline-runtime-plan-canon-autoadopt");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "session-runtime-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();
    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            version: 1,
            routing: RoutingConfig::default(),
            canon: Some(crate::domain::configuration::CanonPreferences {
                mode_selection: CanonModeSelectionPreference::AutoConfirm,
                default_risk: Some("medium".to_string()),
                default_zone: Some("engineering".to_string()),
                default_owner: Some("platform".to_string()),
                default_system_context: Some("existing".to_string()),
            }),
        })
        .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: normalize_inputs_with_governance(
            &workspace,
            Some("Deliver a governed feature"),
            &[PathBuf::from("brief.md")],
            None,
        )
        .ok(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.governance_runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(lifecycle.mode_selection_preference, CanonModeSelectionPreference::AutoConfirm);
    assert_eq!(lifecycle.stage_records.len(), 4, "{:#?}", lifecycle.stage_records);
    assert!(lifecycle.stage_records.iter().all(|record| {
        record.lifecycle_state == GovernanceLifecycleState::GovernedReady
            && record.packet_ref.as_deref() == Some(".canon/planning-packet")
            && record.canon_run_ref.as_deref() == Some("canon-run-plan")
    }));
    assert_eq!(
        session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.compacted_canon_memory.as_ref())
            .and_then(|memory| memory.stage_key.as_deref()),
        Some("plan:backlog")
    );
}

#[cfg(unix)]
#[test]
fn plan_task_blocks_nested_workspace_before_canon_targets_parent_git_root() {
    let repo_root = temp_workspace("boundline-runtime-plan-canon-nested-root");
    fs::create_dir_all(repo_root.join(".git")).unwrap();
    let workspace = repo_root.join("tmp");
    fs::create_dir_all(workspace.join(".boundline")).unwrap();
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "session-runtime-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();
    FileConfigStore::for_workspace(&workspace)
        .save_local(&ConfigFile {
            version: 1,
            routing: RoutingConfig::default(),
            canon: Some(crate::domain::configuration::CanonPreferences {
                mode_selection: CanonModeSelectionPreference::AutoConfirm,
                default_risk: Some("medium".to_string()),
                default_zone: Some("engineering".to_string()),
                default_owner: Some("platform".to_string()),
                default_system_context: Some("existing".to_string()),
            }),
        })
        .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: normalize_inputs_with_governance(
            &workspace,
            Some("Deliver a governed feature"),
            &[PathBuf::from("brief.md")],
            None,
        )
        .ok(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.governance_runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(lifecycle.stage_records.len(), 1, "{:#?}", lifecycle.stage_records);
    assert_eq!(lifecycle.stage_records[0].lifecycle_state, GovernanceLifecycleState::Blocked);
    assert!(
        lifecycle.stage_records[0]
            .blocked_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("would target git root"))
    );
    assert!(
        lifecycle
            .terminal_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("would target git root"))
    );
}

#[test]
fn plan_task_blocks_when_canon_is_selected_but_not_initialized() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-plan-canon-uninitialized",
        vec![ExecutionAttemptDefinition {
            attempt_id: "plan-execution".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left + right".to_string(),
                replace: "left + right".to_string(),
            }],
        }],
        vec!["src/lib.rs".to_string()],
        None,
    );
    assert!(fs::create_dir_all(workspace.join("src")).is_ok());
    assert!(fs::create_dir_all(workspace.join("tests")).is_ok());
    assert!(
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
        )
        .is_ok()
    );
    assert!(
        fs::write(
            workspace.join("brief.md"),
            "Deliver the feature through requirements, architecture, backlog, and implementation for src/lib.rs.\n",
        )
        .is_ok()
    );
    assert!(
        FileConfigStore::for_workspace(&workspace)
            .save_local(&ConfigFile {
                version: 1,
                routing: RoutingConfig::default(),
                canon: Some(crate::domain::configuration::CanonPreferences {
                    mode_selection: CanonModeSelectionPreference::AutoConfirm,
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_owner: Some("platform".to_string()),
                    default_system_context: Some("existing".to_string()),
                }),
            })
            .is_ok()
    );

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: normalize_inputs_with_governance(
            &workspace,
            Some("Deliver a governed feature"),
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
        .ok(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    assert!(runtime.capture_goal(&mut session, "Deliver a governed feature").is_ok());
    assert!(runtime.select_flow(&mut session, "delivery").is_ok());
    assert!(runtime.plan_task(&mut session, None, false).is_ok());

    assert_eq!(
        session.governance_lifecycle.as_ref().map(|lifecycle| lifecycle.stage_records.len()),
        Some(1)
    );
    assert_eq!(
        session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.stage_records.first())
            .map(|record| record.stage_key.as_str()),
        Some("plan:requirements")
    );
    assert_eq!(
        session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.stage_records.first())
            .map(|record| record.lifecycle_state),
        Some(GovernanceLifecycleState::Blocked)
    );
    assert!(
        session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.stage_records.first())
            .and_then(|record| record.blocked_reason.as_deref())
            .is_some_and(|reason| reason.contains("missing governance.canon"))
    );
    assert!(
        session
            .governance_lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.terminal_reason.as_deref())
            .is_some_and(|reason| reason.contains("missing governance.canon"))
    );
    assert_eq!(
        session
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.compacted_canon_memory.as_ref())
            .and_then(|memory| memory.stage_key.as_deref()),
        Some("plan:requirements")
    );
}

#[test]
fn run_to_terminal_rejects_unresolved_planning_governance_for_confirmed_goal_plan() {
    let workspace = temp_workspace("boundline-runtime-plan-governance-run-gate");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut goal_plan = GoalPlan::new(
        "Ship the governed feature",
        vec![PlannedTask {
            task_id: "planned-task-plan-gate".to_string(),
            description: "Implement the governed change".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    goal_plan.confirm().unwrap();

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Ship the governed feature".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Canon,
            explicit_opt_out: false,
            mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
            selected_mode: Some(CanonMode::Requirements),
            selected_mode_sequence: vec![CanonMode::Requirements, CanonMode::Architecture],
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: vec![GovernedStageRecord {
                stage_key: "plan:requirements".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: GovernanceLifecycleState::AwaitingApproval,
                required: true,
                autopilot_enabled: false,
                approval_state: ApprovalState::Requested,
                canon_run_ref: Some("canon-run-plan".to_string()),
                governance_attempt_id: "attempt-plan-1".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: Some(".canon/planning-packet".to_string()),
                decision_ref: None,
                stage_council: None,
                blocked_reason: Some("waiting for Canon approval".to_string()),
            }],
            accumulated_context: Vec::new(),
            terminal_reason: Some("awaiting approval: waiting for Canon approval".to_string()),
            planning_input_fingerprint: None,
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    assert!(matches!(
        runtime.run_to_terminal(&mut session),
        Err(super::SessionRuntimeError::PlanningGovernanceUnresolved { .. })
    ));
}

#[test]
fn execute_next_step_covers_retry_replan_and_terminal_decision_recovery() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-decision-recovery",
        vec![
            ExecutionAttemptDefinition {
                attempt_id: "bad-fix".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Replan,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left - right".to_string(),
                    replace: "left / right".to_string(),
                }],
            },
            ExecutionAttemptDefinition {
                attempt_id: "good-fix".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left / right".to_string(),
                    replace: "left + right".to_string(),
                }],
            },
        ],
    );
    let runtime = SessionRuntime::for_workspace(&workspace);

    let mut retry_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"retryable_failure": true})),
    );
    runtime.execute_next_step(&mut retry_session).unwrap();
    assert_eq!(retry_session.active_task.as_ref().unwrap().retry_count, 1);
    assert_eq!(
        retry_session.active_task.as_ref().unwrap().plan.steps[0].status,
        StepStatus::Pending
    );

    let mut replan_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"replan_required": true})),
    );
    runtime.execute_next_step(&mut replan_session).unwrap();
    assert_eq!(replan_session.active_task.as_ref().unwrap().replan_count, 1);
    assert_eq!(replan_session.active_task.as_ref().unwrap().plan.revision, 1);

    let mut terminal_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"terminal_failure": true})),
    );
    runtime.execute_next_step(&mut terminal_session).unwrap();
    assert_eq!(terminal_session.latest_status, SessionStatus::Failed);
    assert!(terminal_session.latest_terminal_reason.is_some());

    let mut exhausted_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"terminal_failure": true})),
    );
    let max_steps = exhausted_session.active_task.as_ref().unwrap().limits.max_steps;
    exhausted_session.active_task.as_mut().unwrap().total_step_attempts = max_steps;
    let exhausted = runtime.run_to_terminal(&mut exhausted_session).unwrap();
    assert_eq!(exhausted.terminal_status, TaskStatus::Exhausted);
    assert_eq!(exhausted.terminal_reason.condition, TerminalCondition::StepLimitExceeded);

    let mut no_step_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})),
    );
    let no_step_task = no_step_session.active_task.as_mut().unwrap();
    no_step_task.plan.current_step_index = no_step_task.plan.steps.len();
    let no_step = runtime.run_to_terminal(&mut no_step_session).unwrap();
    assert_eq!(no_step.terminal_status, TaskStatus::Failed);
    assert_eq!(no_step.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);

    let mut terminal_response_session = build_session(
        &workspace,
        decision_task(workspace.to_string_lossy().as_ref(), json!({"ok": true})),
    );
    let terminal_task = terminal_response_session.active_task.as_ref().unwrap().clone();
    runtime.load_or_create_trace(&mut terminal_response_session, &terminal_task).unwrap();
    terminal_response_session.active_task.as_mut().unwrap().apply_terminal(
        TaskStatus::Succeeded,
        TerminalReason::new(TerminalCondition::GoalSatisfied, "already complete", None),
    );
    let terminal_response = runtime.run_to_terminal(&mut terminal_response_session).unwrap();
    assert_eq!(terminal_response.terminal_status, TaskStatus::Succeeded);
    assert_eq!(terminal_response.terminal_reason.message, "already complete");
}

#[test]
fn execute_next_step_creates_a_compatibility_task_for_flow_selected_goal_plans() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-compatibility-goal-plan",
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
    );
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "left - right\n").unwrap();
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut goal_plan = GoalPlan::new(
        "Drive a session runtime branch",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Repair arithmetic".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("tests pass".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    goal_plan.confirm().unwrap();

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Drive a session runtime branch".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.execute_next_step(&mut session).unwrap();

    assert!(session.active_task.is_some());
    assert_eq!(session.latest_status, SessionStatus::Running);
    assert!(session.goal_plan.is_some());
}

#[test]
fn run_to_terminal_uses_provider_analysis_and_change_routes_for_flow_selected_goal_plans()
-> Result<(), Box<dyn std::error::Error>> {
    with_env_test(&[OPENAI_BASE_URL_ENV, OPENAI_API_KEY_ENV], || {
        let (base_url, receiver, handle) = spawn_scripted_response_server(vec![
            openai_completion_response(json!({
                "headline": "Inspect arithmetic",
                "summary": "The requested branch still subtracts instead of adding.",
                "risks": []
            })),
            openai_completion_response(json!({
                "headline": "Repair arithmetic",
                "summary": "Switch subtraction to addition.",
                "changes": [
                    {
                        "path": "src/lib.rs",
                        "find": "left - right",
                        "replace": "left + right"
                    }
                ]
            })),
        ])
        .map_err(std::io::Error::other)?;

        unsafe {
            std::env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            std::env::set_var(OPENAI_API_KEY_ENV, "token");
        }

        let workspace = temp_workspace("boundline-runtime-native-provider-flow");
        fs::create_dir_all(workspace.join("src"))?;
        fs::write(
            workspace.join("src/lib.rs"),
            "fn compute(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        )?;

        let execution_profile = WorkspaceExecutionProfile {
            name: "session-runtime-provider-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand { program: "true".to_string(), args: Vec::new() },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "fix-add".to_string(),
                summary: "repair arithmetic".to_string(),
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
        fs::write(
            workspace.join(".boundline/execution.json"),
            serde_json::to_string_pretty(&execution_profile)?,
        )?;

        save_local_routing(
            &workspace,
            RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                implementation: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                ..RoutingConfig::default()
            },
        );

        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut goal_plan = GoalPlan::new(
            "Drive a session runtime branch",
            vec![
                PlannedTask {
                    task_id: "planned-task-1".to_string(),
                    description: "Inspect arithmetic inputs".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some(
                        "analysis identifies the arithmetic mismatch".to_string(),
                    ),
                    decision_type_hint: Some(DecisionType::Analyze),
                },
                PlannedTask {
                    task_id: "planned-task-2".to_string(),
                    description: "Repair arithmetic".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some(
                        "implementation switches subtraction to addition".to_string(),
                    ),
                    decision_type_hint: Some(DecisionType::Code),
                },
            ],
        )?;
        goal_plan.confirm()?;

        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Drive a session runtime branch".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(goal_plan),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: Some(FlowPolicy::from_builtin("bug-fix")?),
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            delight_feedback: None,
            latest_voting: None,
        };

        let response = runtime.run_to_terminal(&mut session)?;

        assert_eq!(response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(session.decisions.len(), 2);
        assert_eq!(session.decisions[0].decision_type, DecisionType::Analyze);
        assert_eq!(session.decisions[1].decision_type, DecisionType::Code);
        assert!(fs::read_to_string(workspace.join("src/lib.rs"))?.contains("left + right"));

        let analysis_request = receiver.recv_timeout(Duration::from_secs(2))?;
        let change_request = receiver.recv_timeout(Duration::from_secs(2))?;
        assert!(analysis_request.contains("Drive a session runtime branch"));
        assert!(analysis_request.contains("src/lib.rs"));
        assert!(analysis_request.contains("left - right"));
        assert!(change_request.contains("Drive a session runtime branch"));
        assert!(change_request.contains("src/lib.rs"));
        assert!(change_request.contains("left - right"));

        let _ = handle.join();
        Ok(())
    })
}

#[test]
fn run_to_terminal_executes_provider_review_for_native_goal_plans()
-> Result<(), Box<dyn std::error::Error>> {
    with_env_test(&[OPENAI_BASE_URL_ENV, OPENAI_API_KEY_ENV], || {
        let (base_url, receiver, handle) = spawn_scripted_response_server(vec![
            openai_completion_response(json!({
                "headline": "Inspect arithmetic",
                "summary": "The requested branch still subtracts instead of adding.",
                "risks": []
            })),
            openai_completion_response(json!({
                "headline": "Repair arithmetic",
                "summary": "Switch subtraction to addition.",
                "changes": [
                    {
                        "path": "src/lib.rs",
                        "find": "left - right",
                        "replace": "left + right"
                    }
                ]
            })),
            openai_completion_response(json!({
                "disposition": "approve",
                "summary": "Bounded change is acceptable.",
                "details": "The review confirmed the arithmetic fix.",
                "required_action": null,
                "evidence_refs": ["src/lib.rs"]
            })),
        ])
        .map_err(std::io::Error::other)?;

        unsafe {
            std::env::set_var(OPENAI_BASE_URL_ENV, &base_url);
            std::env::set_var(OPENAI_API_KEY_ENV, "token");
        }

        let workspace = temp_workspace("boundline-runtime-native-provider-review");
        fs::create_dir_all(workspace.join("src"))?;
        fs::write(
            workspace.join("src/lib.rs"),
            "fn compute(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
        )?;

        save_local_routing(
            &workspace,
            RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                implementation: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                ..RoutingConfig::default()
            },
        );

        let runtime = SessionRuntime::for_workspace(&workspace);
        let mut goal_plan = GoalPlan::new(
            "Ship the arithmetic fix",
            vec![
                PlannedTask {
                    task_id: "planned-task-1".to_string(),
                    description: "Inspect arithmetic inputs".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some(
                        "analysis identifies the arithmetic mismatch".to_string(),
                    ),
                    decision_type_hint: Some(DecisionType::Analyze),
                },
                PlannedTask {
                    task_id: "planned-task-2".to_string(),
                    description: "Repair arithmetic".to_string(),
                    target: "src/lib.rs".to_string(),
                    expected_outcome: Some(
                        "implementation switches subtraction to addition".to_string(),
                    ),
                    decision_type_hint: Some(DecisionType::Code),
                },
            ],
        )?;
        goal_plan.confirm()?;

        let mut session = ActiveSessionRecord {
            session_id: "session-runtime".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Ship the arithmetic fix".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(goal_plan),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 10,
            updated_at: 10,
            governance_lifecycle: None,
            project_scale: None,
            delight_feedback: None,
            latest_voting: None,
        };

        let response = runtime.run_to_terminal(&mut session)?;

        assert_eq!(response.terminal_status, TaskStatus::Succeeded);
        assert_eq!(session.latest_status, SessionStatus::Succeeded);
        assert_eq!(session.decisions.len(), 2);
        assert_eq!(
            response
                .final_context
                .state
                .get("latest_review_outcome")
                .and_then(|value| value.as_str()),
            Some("accepted")
        );
        assert_eq!(
            response
                .final_context
                .state
                .get("latest_validation_status")
                .and_then(|value| value.as_str()),
            Some("passed")
        );
        assert!(
            response
                .final_context
                .state
                .get("latest_changed_files")
                .and_then(|value| value.as_array())
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("src/lib.rs")))
        );

        let analysis_request = receiver.recv_timeout(Duration::from_secs(2))?;
        let change_request = receiver.recv_timeout(Duration::from_secs(2))?;
        let review_request = receiver.recv_timeout(Duration::from_secs(2))?;
        assert!(analysis_request.contains("Ship the arithmetic fix"));
        assert!(change_request.contains("Ship the arithmetic fix"));
        assert!(review_request.contains("provider-review"));
        assert!(review_request.contains("Provider Review"));
        assert!(review_request.contains("latest_changed_files"));
        assert!(review_request.contains("src/lib.rs"));

        let _ = handle.join();
        Ok(())
    })
}

#[test]
fn run_to_terminal_executes_provider_adjudication_for_flow_selected_goal_plans()
-> Result<(), Box<dyn std::error::Error>> {
    with_env_test(
        &[
            OPENAI_BASE_URL_ENV,
            OPENAI_API_KEY_ENV,
            DEEPSEEK_BASE_URL_ENV,
            DEEPSEEK_API_KEY_ENV,
            GROQ_BASE_URL_ENV,
            GROQ_API_KEY_ENV,
        ],
        || {
            let (base_url, receiver, handle) = spawn_scripted_response_server(vec![
                openai_completion_response(json!({
                    "headline": "Inspect arithmetic",
                    "summary": "The requested branch still subtracts instead of adding.",
                    "risks": []
                })),
                openai_completion_response(json!({
                    "headline": "Repair arithmetic",
                    "summary": "Switch subtraction to addition.",
                    "changes": [
                        {
                            "path": "src/lib.rs",
                            "find": "left - right",
                            "replace": "left + right"
                        }
                    ]
                })),
                openai_completion_response(json!({
                    "disposition": "approve",
                    "summary": "Arithmetic change looks bounded.",
                    "details": "The implementation matches the requested fix.",
                    "required_action": null,
                    "evidence_refs": ["src/lib.rs"]
                })),
                openai_completion_response(json!({
                    "disposition": "concern",
                    "summary": "Verification evidence should be double-checked.",
                    "details": "The change is small but review wants an explicit tie-break.",
                    "required_action": "confirm validation evidence",
                    "evidence_refs": ["src/lib.rs"]
                })),
                openai_completion_response(json!({
                    "disposition": "approve",
                    "summary": "Adjudication accepts the bounded change.",
                    "details": "The council disagreement is resolved in favor of the fix.",
                    "required_action": null,
                    "evidence_refs": ["src/lib.rs"]
                })),
            ])
            .map_err(std::io::Error::other)?;

            unsafe {
                std::env::set_var(OPENAI_BASE_URL_ENV, &base_url);
                std::env::set_var(OPENAI_API_KEY_ENV, "token");
                std::env::set_var(DEEPSEEK_BASE_URL_ENV, &base_url);
                std::env::set_var(DEEPSEEK_API_KEY_ENV, "token");
                std::env::set_var(GROQ_BASE_URL_ENV, &base_url);
                std::env::set_var(GROQ_API_KEY_ENV, "token");
            }

            let workspace = temp_workspace("boundline-runtime-provider-adjudication");
            fs::create_dir_all(workspace.join("src"))?;
            fs::write(
                workspace.join("src/lib.rs"),
                "fn compute(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
            )?;

            let execution_profile = WorkspaceExecutionProfile {
                name: "session-runtime-provider-adjudication".to_string(),
                read_targets: vec!["src/lib.rs".to_string()],
                validation_command: ExecutionCommand {
                    program: "true".to_string(),
                    args: Vec::new(),
                },
                attempts: vec![ExecutionAttemptDefinition {
                    attempt_id: "fix-add".to_string(),
                    summary: "repair arithmetic".to_string(),
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
                review: Some(ReviewProfile {
                    triggers: vec![ReviewTrigger::PrReady],
                    reviewers: vec![
                        ReviewerDefinition {
                            reviewer_id: "alpha".to_string(),
                            role: "Safety".to_string(),
                            source: None,
                            weight: 1,
                        },
                        ReviewerDefinition {
                            reviewer_id: "beta".to_string(),
                            role: "Maintainability".to_string(),
                            source: None,
                            weight: 1,
                        },
                    ],
                    vote_rule: VoteRuleDefinition::default(),
                    adjudication: AdjudicationDefinition {
                        enabled: true,
                        reviewer_id: Some("arbiter".to_string()),
                    },
                    scenarios: vec![ReviewScenario {
                        trigger: ReviewTrigger::PrReady,
                        findings: vec![
                            ReviewerFinding::new(
                                "alpha".to_string(),
                                ReviewerDisposition::Approve,
                                "placeholder approval".to_string(),
                            ),
                            ReviewerFinding::new(
                                "beta".to_string(),
                                ReviewerDisposition::Concern,
                                "placeholder concern".to_string(),
                            ),
                        ],
                        adjudication_finding: Some(ReviewerFinding::new(
                            "arbiter".to_string(),
                            ReviewerDisposition::Approve,
                            "placeholder adjudication".to_string(),
                        )),
                    }],
                }),
                legacy_source: None,
            };
            fs::write(
                workspace.join(".boundline/execution.json"),
                serde_json::to_string_pretty(&execution_profile)?,
            )?;

            let mut routing = RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                implementation: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                adjudication: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "groq/llama-3.1-8b-instant".to_string(),
                }),
                ..RoutingConfig::default()
            };
            routing.reviewer_roles.insert(
                "alpha".to_string(),
                ModelRoute { runtime: RuntimeKind::Codex, model: "openai/gpt-5.4".to_string() },
            );
            routing.reviewer_roles.insert(
                "beta".to_string(),
                ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "deepseek/deepseek-chat".to_string(),
                },
            );
            save_local_routing(&workspace, routing);

            let runtime = SessionRuntime::for_workspace(&workspace);
            let mut goal_plan = GoalPlan::new(
                "Ship the arithmetic fix",
                vec![
                    PlannedTask {
                        task_id: "planned-task-1".to_string(),
                        description: "Inspect arithmetic inputs".to_string(),
                        target: "src/lib.rs".to_string(),
                        expected_outcome: Some(
                            "analysis identifies the arithmetic mismatch".to_string(),
                        ),
                        decision_type_hint: Some(DecisionType::Analyze),
                    },
                    PlannedTask {
                        task_id: "planned-task-2".to_string(),
                        description: "Repair arithmetic".to_string(),
                        target: "src/lib.rs".to_string(),
                        expected_outcome: Some(
                            "implementation switches subtraction to addition".to_string(),
                        ),
                        decision_type_hint: Some(DecisionType::Code),
                    },
                ],
            )?;
            goal_plan.confirm()?;

            let mut session = ActiveSessionRecord {
                session_id: "session-runtime".to_string(),
                workspace_ref: workspace.to_string_lossy().into_owned(),
                goal: Some("Ship the arithmetic fix".to_string()),
                authored_brief: None,
                negotiation_packet: None,
                active_flow: None,
                active_task: None,
                goal_plan: Some(goal_plan),
                workflow_progress: None,
                decisions: Vec::new(),
                active_flow_policy: Some(FlowPolicy::from_builtin("bug-fix")?),
                latest_status: SessionStatus::Planned,
                latest_terminal_reason: None,
                latest_trace_ref: None,
                created_at: 10,
                updated_at: 10,
                governance_lifecycle: None,
                project_scale: None,
                delight_feedback: None,
                latest_voting: None,
            };

            let response = runtime.run_to_terminal(&mut session)?;

            assert_eq!(response.terminal_status, TaskStatus::Succeeded);
            assert_eq!(session.latest_status, SessionStatus::Succeeded);
            assert_eq!(
                response
                    .final_context
                    .state
                    .get("latest_review_outcome")
                    .and_then(|value| value.as_str()),
                Some("accepted")
            );
            let adjudication = response
                .final_context
                .state
                .get("latest_review_adjudication")
                .cloned()
                .and_then(|value| serde_json::from_value::<ReviewerFinding>(value).ok())
                .ok_or("expected persisted adjudication finding")?;
            assert_eq!(adjudication.reviewer_id, "arbiter");
            assert_eq!(adjudication.disposition, ReviewerDisposition::Approve);

            let analysis_request = receiver.recv_timeout(Duration::from_secs(2))?;
            let change_request = receiver.recv_timeout(Duration::from_secs(2))?;
            let alpha_review_request = receiver.recv_timeout(Duration::from_secs(2))?;
            let beta_review_request = receiver.recv_timeout(Duration::from_secs(2))?;
            let arbiter_review_request = receiver.recv_timeout(Duration::from_secs(2))?;

            assert!(analysis_request.contains("Ship the arithmetic fix"));
            assert!(change_request.contains("Ship the arithmetic fix"));
            assert!(alpha_review_request.contains("alpha"));
            assert!(beta_review_request.contains("beta"));
            assert!(arbiter_review_request.contains("arbiter"));

            let _ = handle.join();
            Ok(())
        },
    )
}

#[cfg(unix)]
#[test]
fn run_to_terminal_executes_post_implementation_canon_governance()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = temp_workspace("boundline-runtime-execution-canon");
    fs::create_dir_all(workspace.join("src"))?;
    fs::write(
        workspace.join("src/lib.rs"),
        "fn compute(left: i32, right: i32) -> i32 {\n    left - right\n}\n",
    )?;

    let (canon_command, requests_path) = write_fake_execution_canon_command(&workspace);
    let execution_profile = WorkspaceExecutionProfile {
        name: "session-runtime-canon-execution-profile".to_string(),
        read_targets: vec!["src/lib.rs".to_string()],
        validation_command: ExecutionCommand { program: "true".to_string(), args: Vec::new() },
        attempts: vec![ExecutionAttemptDefinition {
            attempt_id: "fix-add".to_string(),
            summary: "repair arithmetic".to_string(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "left - right".to_string(),
                replace: "left + right".to_string(),
            }],
        }],
        adaptive: None,
        limits: RunLimits::default(),
        governance: Some(GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Canon,
            canon: Some(CanonRuntimeConfig {
                command: canon_command.to_string_lossy().into_owned(),
                default_owner: Some("platform".to_string()),
                default_risk: Some("medium".to_string()),
                default_zone: Some("engineering".to_string()),
                default_system_context: Some(SystemContextBinding::Existing),
            }),
            stages: Vec::new(),
        }),
        review: None,
        legacy_source: None,
    };
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&execution_profile)?,
    )?;

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut goal_plan = GoalPlan::new(
        "Drive a governed session runtime branch",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Repair arithmetic".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("implementation switches subtraction to addition".to_string()),
            decision_type_hint: Some(DecisionType::Code),
        }],
    )?;
    goal_plan.confirm()?;

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Drive a governed session runtime branch".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: Some(FlowPolicy::from_builtin("bug-fix")?),
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    let response = runtime.run_to_terminal(&mut session)?;

    assert_eq!(response.terminal_status, TaskStatus::Succeeded);
    assert!(!session.decisions.is_empty());

    let requests = fs::read_to_string(&requests_path)?;
    assert!(requests.contains("\"stage_key\":\"run:implementation\""), "{requests}");
    assert!(requests.contains("\"stage_key\":\"run:verification\""), "{requests}");
    assert!(
        requests.contains(".boundline/governance/execution/implementation/brief.md"),
        "{requests}"
    );
    assert!(
        requests.contains(".boundline/governance/execution/verification/brief.md"),
        "{requests}"
    );

    assert!(workspace.join(".boundline/governance/execution/implementation/brief.md").exists());
    assert!(workspace.join(".boundline/governance/execution/verification/brief.md").exists());

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert!(lifecycle.stage_records.iter().any(|record| {
        record.stage_key == "run:implementation"
            && record.runtime == GovernanceRuntimeKind::Canon
            && record.lifecycle_state == GovernanceLifecycleState::GovernedReady
    }));
    assert!(lifecycle.stage_records.iter().any(|record| {
        record.stage_key == "run:verification"
            && record.runtime == GovernanceRuntimeKind::Canon
            && record.lifecycle_state == GovernanceLifecycleState::GovernedReady
    }));
    assert!(lifecycle.accumulated_context.iter().any(|document| {
        document.stage_key == "run:implementation"
            && document.canon_mode == CanonMode::Implementation
    }));
    assert!(lifecycle.accumulated_context.iter().any(|document| {
        document.stage_key == "run:verification" && document.canon_mode == CanonMode::Verification
    }));
    assert!(
        session.goal_plan.as_ref().and_then(|plan| plan.compacted_canon_memory.as_ref()).is_some()
    );

    Ok(())
}

#[test]
fn native_goal_plan_short_circuits_for_existing_delegation_continuity() {
    let workspace = temp_workspace("boundline-runtime-native-delegation");
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut goal_plan = GoalPlan::new(
        "Drive a delegated continuation boundary",
        vec![PlannedTask {
            task_id: "planned-task-1".to_string(),
            description: "Inspect the delegated boundary".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("status explains the continuity boundary".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    goal_plan.confirm().unwrap();
    goal_plan = goal_plan
        .with_delegation_state(
            Vec::new(),
            DelegationContinuityState {
                active_packet_id: None,
                mode: DelegationContinuityMode::InspectOnly,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: "delegated continuity requires operator inspection".to_string(),
                evidence_summary: "bounded continuation stopped at an inspect-only boundary"
                    .to_string(),
            },
        )
        .unwrap();

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Drive a delegated continuation boundary".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: Some(goal_plan),
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    let response = runtime.run_to_terminal(&mut session).unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Failed);
    assert_eq!(response.terminal_reason.condition, TerminalCondition::NoCredibleNextStep);
    assert_eq!(session.latest_status, SessionStatus::Planned);
    assert!(session.goal_plan.as_ref().unwrap().delegation_continuity().is_some());
    assert!(session.active_task.is_none());
}

#[test]
fn execute_next_step_falls_back_to_local_governance_when_canon_is_optional() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-governance-local-fallback",
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
        vec!["README.md".to_string()],
        Some(GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: Some(CanonRuntimeConfig {
                command: "canon-missing-for-test".to_string(),
                default_owner: Some("platform".to_string()),
                default_risk: Some("medium".to_string()),
                default_zone: Some("engineering".to_string()),
                default_system_context: Some(SystemContextBinding::Existing),
            }),
            stages: vec![StageGovernancePolicy {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                enabled: true,
                required: false,
                autopilot: false,
                require_adaptive_companion: false,
                runtime: Some(GovernanceRuntimeKind::Canon),
                canon_mode: Some(CanonMode::Discovery),
                reasoning_profile: None,
                system_context: Some(SystemContextBinding::Existing),
                risk: Some("medium".to_string()),
                zone: Some("engineering".to_string()),
                owner: Some("platform".to_string()),
            }],
        }),
    );
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
    runtime.select_flow(&mut session, "bug-fix").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();
    runtime.execute_next_step(&mut session).unwrap();

    let task = session.active_task.as_ref().unwrap();
    let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
    let governed_packet = task.context.latest_governance_packet().unwrap().unwrap();
    assert_eq!(governed_stage.stage_key, "bug-fix:investigate");
    assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Local);
    assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);
    assert_eq!(governed_packet.runtime, GovernanceRuntimeKind::Local);
    assert_eq!(governed_packet.readiness, PacketReadiness::Reusable);
    assert!(!governed_packet.document_refs.is_empty());

    let trace =
        runtime.trace_store().load(Path::new(session.latest_trace_ref.as_ref().unwrap())).unwrap();
    assert!(
        trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceSelected),
        "{:?}",
        trace.events
    );
    assert!(
        trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceCompleted),
        "{:?}",
        trace.events
    );
}

#[test]
fn execute_next_step_reassesses_reasoning_profile_after_routing_changes() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-reasoning-profile-gate",
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
        vec!["README.md".to_string()],
        Some(GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: None,
            stages: vec![StageGovernancePolicy {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                enabled: true,
                required: false,
                autopilot: false,
                require_adaptive_companion: false,
                runtime: Some(GovernanceRuntimeKind::Local),
                canon_mode: Some(CanonMode::Discovery),
                reasoning_profile: Some(independent_pair_review_profile()),
                system_context: None,
                risk: None,
                zone: None,
                owner: None,
            }],
        }),
    );
    let runtime = SessionRuntime::for_workspace(&workspace);
    let collapsed_routing = RoutingConfig {
        review: Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() }),
        ..RoutingConfig::default()
    };
    save_local_routing(&workspace, collapsed_routing.clone());

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
    runtime.select_flow(&mut session, "bug-fix").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();
    runtime.execute_next_step(&mut session).unwrap();

    let Some(blocked_profile) = session
        .governance_lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
    else {
        panic!("expected a blocked reasoning profile after the first run");
    };
    assert_eq!(blocked_profile.status, ReasoningActivationStatus::Blocked);
    assert_eq!(
        blocked_profile.independence.as_ref().map(|assessment| assessment.result),
        Some(IndependenceAssessmentResult::Failed)
    );
    assert_eq!(
        blocked_profile.confidence.as_ref().map(|confidence| confidence.confidence_level),
        Some(ReasoningConfidenceLevel::Low)
    );
    assert_eq!(session.latest_status, SessionStatus::Running);
    assert_eq!(
        session
            .active_task
            .as_ref()
            .and_then(|task| task.plan.steps.first())
            .map(|step| step.status),
        Some(StepStatus::Pending)
    );

    let mut recovered_routing = collapsed_routing;
    recovered_routing.reviewer_roles.insert(
        "reviewer_primary".to_string(),
        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
    );
    recovered_routing.reviewer_roles.insert(
        "reviewer_secondary".to_string(),
        ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() },
    );
    save_local_routing(&workspace, recovered_routing);

    let profile = independent_pair_review_profile();
    let effective_routing = super::effective_routing_for_workspace(&workspace);
    let participants = super::reasoning_participants_for_profile(
        "bug-fix:investigate",
        &profile,
        &effective_routing,
    );
    let assessment =
        super::assess_reasoning_independence("bug-fix:investigate", &profile, &participants);
    assert_eq!(assessment.result, IndependenceAssessmentResult::Passed);

    runtime.execute_next_step(&mut session).unwrap();

    let Some(active_profile) = session
        .governance_lifecycle
        .as_ref()
        .and_then(|lifecycle| lifecycle.latest_reasoning_profile.as_ref())
    else {
        panic!("expected a completed reasoning profile after reviewer routes diverged");
    };
    let step_status = session
        .active_task
        .as_ref()
        .and_then(|task| task.plan.steps.first())
        .map(|step| step.status);
    assert!(step_status.is_some_and(|status| status != StepStatus::Pending), "{active_profile:?}");
    assert_eq!(
        active_profile.status,
        ReasoningActivationStatus::Completed,
        "{step_status:?} {active_profile:?}"
    );
    assert_eq!(
        active_profile.independence.as_ref().map(|assessment| assessment.result),
        Some(IndependenceAssessmentResult::Passed)
    );
    assert_eq!(
        active_profile.outcome.as_ref().map(|outcome| outcome.outcome_kind),
        Some(ReasoningOutcomeKind::Adjudicated)
    );
    assert_eq!(
        active_profile.confidence.as_ref().map(|confidence| confidence.confidence_level),
        Some(ReasoningConfidenceLevel::Medium)
    );
    assert_eq!(active_profile.participants.len(), 2);
    assert!(
        active_profile
            .participants
            .iter()
            .all(|participant| participant.status == ReasoningParticipantStatus::Completed)
    );
    assert!(active_profile.participants.iter().all(|participant| {
        participant.result_summary.as_deref().is_some_and(|summary| !summary.trim().is_empty())
    }));
    assert!(
        active_profile
            .participants
            .iter()
            .any(|participant| participant.provider_family.as_deref() == Some("claude"))
    );
}

#[test]
fn canon_reasoning_posture_uses_current_release_window() {
    let posture = super::reasoning_posture_for_activation(
        &independent_pair_review_profile(),
        GovernanceRuntimeKind::Canon,
        "attempt-7",
    )
    .unwrap()
    .expect("canon runtime should project a reasoning posture");

    assert_eq!(
        posture.contract_line,
        crate::domain::reasoning::REASONING_POSTURE_V1_CONTRACT_LINE.to_string()
    );
    assert_eq!(
        posture.required_profile_id,
        Some(crate::domain::reasoning::ReasoningProfileId::IndependentPairReview)
    );
    assert!(posture.compatibility_window.admits_versions(
        super::CURRENT_BOUNDLINE_VERSION,
        crate::domain::distribution::SUPPORTED_CANON_VERSION,
    ));
    assert_eq!(posture.provenance_ref, "governance_attempt:attempt-7");
}

#[test]
fn native_persistence_projects_cluster_story_and_copies_changes() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-cluster-primary",
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
    );
    let ready_member = write_execution_profile_workspace(
        "boundline-runtime-cluster-ready-member",
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
    );
    let blocked_member = write_execution_profile_workspace(
        "boundline-runtime-cluster-blocked-member",
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
    );
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(ready_member.join("src")).unwrap();
    fs::create_dir_all(blocked_member.join("src")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "left + right\n").unwrap();
    fs::write(ready_member.join("src/lib.rs"), "left - right\n").unwrap();
    fs::write(blocked_member.join("src/lib.rs"), "unchanged\n").unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut goal_plan = GoalPlan::new(
        "Deliver cluster follow-through",
        vec![PlannedTask {
            task_id: "planned-task-cluster".to_string(),
            description: "Propagate the bounded change".to_string(),
            target: "src/lib.rs".to_string(),
            expected_outcome: Some("cluster state records the authoritative route".to_string()),
            decision_type_hint: None,
        }],
    )
    .unwrap();
    goal_plan.confirm().unwrap();
    goal_plan.cluster_session_projection = Some(ClusterSessionProjection {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: workspace.to_string_lossy().into_owned(),
        member_workspace_refs: vec![
            workspace.to_string_lossy().into_owned(),
            ready_member.to_string_lossy().into_owned(),
            blocked_member.to_string_lossy().into_owned(),
        ],
        started_from_command: "boundline cluster status".to_string(),
        updated_at: 10,
    });

    let mut fixture_runtime = manual_runtime();
    fixture_runtime.profile.attempts = vec![ExecutionAttemptDefinition {
        attempt_id: "fix-add".to_string(),
        summary: String::new(),
        failure_mode: ExecutionFailureMode::Terminal,
        changes: vec![WorkspaceChange {
            path: "src/lib.rs".to_string(),
            find: "left - right".to_string(),
            replace: "left + right".to_string(),
        }],
    }];
    runtime.propagate_cluster_delivery_changes(&goal_plan, &fixture_runtime).unwrap();
    assert_eq!(fs::read_to_string(ready_member.join("src/lib.rs")).unwrap(), "left + right\n");
    assert_eq!(fs::read_to_string(blocked_member.join("src/lib.rs")).unwrap(), "unchanged\n");

    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Deliver cluster follow-through".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: None,
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Planned,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };
    let trace = ExecutionTrace::new("task-cluster", "session-runtime", "cluster goal");
    let response = runtime
        .persist_native_result(
            &mut session,
            goal_plan,
            Vec::new(),
            trace,
            super::NativePersistenceInput {
                checkpoint_projection: None,
                terminal_reason: TerminalReason::new(
                    TerminalCondition::GoalSatisfied,
                    "cluster goal satisfied",
                    None,
                ),
                limits: RunLimits::default(),
                native_context: TaskContext::new(
                    "session-runtime",
                    workspace.to_string_lossy().into_owned(),
                    RunLimits::default(),
                    Map::new(),
                ),
                record_terminal_event: true,
                projected_task: None,
            },
        )
        .unwrap();

    assert_eq!(response.terminal_status, TaskStatus::Failed);
    assert_eq!(session.latest_status, SessionStatus::Failed);
    let cluster_story =
        session.goal_plan.as_ref().unwrap().cluster_delivery_story.as_ref().unwrap();
    assert_eq!(cluster_story.execution_condition.kind, ClusteredExecutionKind::Failed);
    assert!(cluster_story.execution_condition.summary.contains("blocked by workspace"));
}

#[test]
fn cluster_story_helper_covers_success_paused_failed_and_exhausted_states() {
    let primary = write_execution_profile_workspace(
        "boundline-runtime-cluster-story-primary",
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
    );
    let member = write_execution_profile_workspace(
        "boundline-runtime-cluster-story-member",
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
    );
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(member.join("src/lib.rs"), "left - right\n").unwrap();

    let runtime = SessionRuntime::for_workspace(&primary);
    let projection = ClusterSessionProjection {
        cluster_id: "cluster-1".to_string(),
        primary_workspace_ref: primary.to_string_lossy().into_owned(),
        member_workspace_refs: vec![
            primary.to_string_lossy().into_owned(),
            member.to_string_lossy().into_owned(),
        ],
        started_from_command: "boundline cluster status".to_string(),
        updated_at: 10,
    };

    let success = runtime.build_cluster_delivery_story(&projection, TaskStatus::Succeeded);
    assert_eq!(success.execution_condition.kind, ClusteredExecutionKind::Success);
    assert!(!success.execution_condition.recovery_allowed);
    assert_eq!(success.participating_workspaces[0].latest_status.as_deref(), Some("succeeded"));
    assert_eq!(
        success.participating_workspaces[1].participation_kind,
        crate::domain::cluster::WorkspaceParticipationKind::ReadOnly
    );

    let paused = runtime.build_cluster_delivery_story(&projection, TaskStatus::Running);
    assert_eq!(paused.execution_condition.kind, ClusteredExecutionKind::Paused);
    assert!(paused.execution_condition.recovery_allowed);

    let exhausted = runtime.build_cluster_delivery_story(&projection, TaskStatus::Exhausted);
    assert_eq!(exhausted.execution_condition.kind, ClusteredExecutionKind::Exhausted);

    let failed = runtime.build_cluster_delivery_story(&projection, TaskStatus::Aborted);
    assert_eq!(failed.execution_condition.kind, ClusteredExecutionKind::Failed);
    assert!(failed.execution_condition.recovery_allowed);
}

#[test]
fn refresh_governance_state_handles_refreshable_and_non_refreshable_records() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-governance-refresh",
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
        vec!["README.md".to_string()],
        Some(GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: None,
            stages: vec![StageGovernancePolicy {
                flow_name: "bug-fix".to_string(),
                stage_id: "investigate".to_string(),
                enabled: true,
                required: false,
                autopilot: false,
                require_adaptive_companion: false,
                runtime: Some(GovernanceRuntimeKind::Local),
                canon_mode: None,
                reasoning_profile: None,
                system_context: Some(SystemContextBinding::Existing),
                risk: None,
                zone: None,
                owner: None,
            }],
        }),
    );
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: Some("Refresh governed stage".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
        active_task: Some(
            Task::new(
                "task-govern-refresh",
                &build_request(workspace.to_string_lossy().as_ref()),
                Plan::new(vec![
                    Step::agent(
                        "investigate",
                        "analyzer",
                        attach_stage_metadata(
                            json!({"phase": "investigate"}),
                            built_in_flow("bug-fix").unwrap(),
                            0,
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
            )
            .unwrap(),
        ),
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Running,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };
    session
        .active_task
        .as_mut()
        .unwrap()
        .context
        .set_latest_governance_stage(&GovernedStageRecord {
            stage_key: "bug-fix:investigate".to_string(),
            runtime: GovernanceRuntimeKind::Local,
            lifecycle_state: GovernanceLifecycleState::AwaitingApproval,
            required: false,
            autopilot_enabled: false,
            approval_state: ApprovalState::Requested,
            canon_run_ref: None,
            governance_attempt_id: "attempt-1".to_string(),
            previous_governance_attempt_id: None,
            packet_ref: None,
            decision_ref: None,
            stage_council: None,
            blocked_reason: None,
        })
        .unwrap();

    assert!(runtime.refresh_governance_state(&mut session).unwrap());
    let refreshed =
        session.active_task.as_ref().unwrap().context.latest_governance_stage().unwrap().unwrap();
    assert_eq!(refreshed.lifecycle_state, GovernanceLifecycleState::GovernedReady);

    session
        .active_task
        .as_mut()
        .unwrap()
        .context
        .set_latest_governance_stage(&GovernedStageRecord {
            stage_key: "bug-fix:investigate".to_string(),
            runtime: GovernanceRuntimeKind::Local,
            lifecycle_state: GovernanceLifecycleState::Blocked,
            required: false,
            autopilot_enabled: false,
            approval_state: ApprovalState::NotNeeded,
            canon_run_ref: None,
            governance_attempt_id: "attempt-2".to_string(),
            previous_governance_attempt_id: Some("attempt-1".to_string()),
            packet_ref: None,
            decision_ref: None,
            stage_council: None,
            blocked_reason: Some("still blocked".to_string()),
        })
        .unwrap();
    assert!(!runtime.refresh_governance_state(&mut session).unwrap());
}

#[test]
fn execute_next_step_blocks_when_required_canon_governance_is_unavailable() {
    let workspace = write_governed_execution_profile_workspace(
        "boundline-runtime-governance-required-canon",
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
        vec!["README.md".to_string()],
        Some(GovernanceProfile {
            default_runtime: GovernanceRuntimeKind::Local,
            canon: Some(CanonRuntimeConfig {
                command: "canon-missing-for-test".to_string(),
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
                canon_mode: Some(CanonMode::Discovery),
                reasoning_profile: None,
                system_context: Some(SystemContextBinding::Existing),
                risk: Some("medium".to_string()),
                zone: Some("engineering".to_string()),
                owner: Some("platform".to_string()),
            }],
        }),
    );
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Drive governed bug fix").unwrap();
    runtime.select_flow(&mut session, "bug-fix").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();
    runtime.execute_next_step(&mut session).unwrap();

    let task = session.active_task.as_ref().unwrap();
    let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
    assert_eq!(session.latest_status, SessionStatus::Failed);
    assert_eq!(task.status, TaskStatus::Failed);
    assert_eq!(governed_stage.stage_key, "bug-fix:investigate");
    assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::Blocked);
    assert!(task.context.latest_governance_packet().unwrap().is_none());
    assert!(
        session
            .latest_terminal_reason
            .as_ref()
            .unwrap()
            .message
            .contains("governance blocked stage bug-fix:investigate")
    );
    assert_eq!(task.plan.current_step_index, 0);
    assert_eq!(task.plan.steps[0].status, StepStatus::Pending);

    let trace =
        runtime.trace_store().load(Path::new(session.latest_trace_ref.as_ref().unwrap())).unwrap();
    assert!(
        trace.events.iter().any(|event| event.event_type == TraceEventType::GovernanceBlocked),
        "{:?}",
        trace.events
    );
}

#[test]
fn required_canon_governance_reports_missing_configuration_and_mode() {
    let workspace_missing_config = temp_workspace("boundline-runtime-governance-required-config");
    let runtime_missing_config = SessionRuntime::for_workspace(&workspace_missing_config);
    let mut missing_config_session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace_missing_config.to_string_lossy().into_owned(),
        goal: Some("Drive governed bug fix".to_string()),
        authored_brief: None,
        negotiation_packet: None,
        active_flow: Some(built_in_flow("bug-fix").unwrap().initial_state()),
        active_task: None,
        goal_plan: None,
        workflow_progress: None,
        decisions: Vec::new(),
        active_flow_policy: None,
        latest_status: SessionStatus::Initialized,
        latest_terminal_reason: None,
        latest_trace_ref: None,
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };
    let policy = StageGovernancePolicy {
        flow_name: "bug-fix".to_string(),
        stage_id: "investigate".to_string(),
        enabled: true,
        required: true,
        autopilot: false,
        require_adaptive_companion: false,
        runtime: Some(GovernanceRuntimeKind::Canon),
        canon_mode: Some(CanonMode::Discovery),
        reasoning_profile: None,
        system_context: Some(SystemContextBinding::Existing),
        risk: None,
        zone: None,
        owner: None,
    };
    let governance = GovernanceProfile {
        default_runtime: GovernanceRuntimeKind::Local,
        canon: None,
        stages: vec![policy.clone()],
    };
    let mut fixture_runtime = manual_runtime();
    fixture_runtime.profile.read_targets = vec!["README.md".to_string()];
    let step = Step::agent(
        "investigate",
        "analyzer",
        attach_stage_metadata(
            json!({"phase": "investigate"}),
            built_in_flow("bug-fix").unwrap(),
            0,
        )
        .unwrap(),
    )
    .unwrap();
    let metadata = super::FlowStepMetadata::from_step(&step).unwrap().unwrap();
    let mut task = Task::new(
        "task-governance-config",
        &build_request(workspace_missing_config.to_string_lossy().as_ref()),
        Plan::new(vec![step.clone()]).unwrap(),
    )
    .unwrap();
    let mut trace = ExecutionTrace::new("task-governance-config", "session-runtime", "goal");

    let decision = runtime_missing_config
        .execute_governance_for_step(
            &mut missing_config_session,
            &mut task,
            &mut trace,
            &fixture_runtime,
            &step,
            &metadata,
            &governance,
            &policy,
            super::GovernanceRequestKind::Start,
        )
        .unwrap();
    match decision {
        super::GovernanceStepDecision::Terminal(response) => {
            assert!(response.terminal_reason.message.contains("requires Canon configuration"));
        }
        _ => panic!("expected terminal governance block"),
    }

    let command_workspace = temp_workspace("boundline-runtime-governance-required-mode-command");
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
    fs::write(
        &command_path,
        format!("#!/bin/sh\ncat >/dev/null\ncat '{}'\n", response_path.to_string_lossy()),
    )
    .unwrap();
    let mut permissions = fs::metadata(&command_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&command_path, permissions).unwrap();

    let workspace_missing_mode = write_governed_execution_profile_workspace(
        "boundline-runtime-governance-required-mode",
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
        vec!["README.md".to_string()],
        Some(GovernanceProfile {
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
                reasoning_profile: None,
                system_context: Some(SystemContextBinding::Existing),
                risk: None,
                zone: None,
                owner: None,
            }],
        }),
    );
    let document_path = workspace_missing_mode.join(document_ref);
    fs::create_dir_all(document_path.parent().unwrap()).unwrap();
    fs::write(&document_path, "# Discovery\n\nCredible governed evidence.\n").unwrap();
    let runtime_missing_mode = SessionRuntime::for_workspace(&workspace_missing_mode);
    let mut missing_mode_session = ActiveSessionRecord {
        session_id: "session-runtime".to_string(),
        workspace_ref: workspace_missing_mode.to_string_lossy().into_owned(),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: None,
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };
    runtime_missing_mode.capture_goal(&mut missing_mode_session, "Drive governed bug fix").unwrap();
    runtime_missing_mode.select_flow(&mut missing_mode_session, "bug-fix").unwrap();
    runtime_missing_mode.plan_task(&mut missing_mode_session, None, false).unwrap();
    runtime_missing_mode.execute_next_step(&mut missing_mode_session).unwrap();
    let task = missing_mode_session.active_task.as_ref().unwrap();
    let governed_stage = task.context.latest_governance_stage().unwrap().unwrap();
    let governed_packet = task.context.latest_governance_packet().unwrap().unwrap();
    assert_eq!(governed_stage.runtime, GovernanceRuntimeKind::Canon);
    assert_eq!(governed_stage.lifecycle_state, GovernanceLifecycleState::GovernedReady);
    assert_eq!(governed_packet.canon_mode, Some(CanonMode::Discovery));
}

#[test]
fn prepare_checkpoint_for_mutation_records_workspace_projection_on_task_context() {
    let workspace = write_execution_profile_workspace(
        "boundline-runtime-checkpoint-workspace",
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
    );
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::write(workspace.join("src/lib.rs"), "left - right").unwrap();

    let task = decision_task(&workspace.to_string_lossy(), json!({"decision": "checkpoint"}));
    let mut session = build_session(&workspace, task);
    let runtime = SessionRuntime::for_workspace(&workspace);

    let projection = runtime
        .prepare_checkpoint_for_mutation(&mut session, SessionCommand::Step)
        .unwrap()
        .unwrap();

    assert_eq!(projection.scope, "workspace");
    assert_eq!(projection.workspace_refs, vec![workspace.to_string_lossy().into_owned()]);
    assert_eq!(
        session
            .active_task
            .as_ref()
            .unwrap()
            .context
            .state
            .get("latest_checkpoint_id")
            .and_then(|value| value.as_str()),
        Some(projection.checkpoint_id.as_str())
    );

    fs::write(workspace.join("src/lib.rs"), "left + right").unwrap();
    runtime.refresh_checkpoint_projection(&session, &projection).unwrap();

    let manifest = FileCheckpointStore::for_session(&workspace, &session.session_id)
        .load(&projection.checkpoint_id)
        .unwrap()
        .unwrap();
    assert_ne!(
        manifest.captured_files[0].captured_fingerprint,
        manifest.captured_files[0].observed_after_capture_fingerprint
    );
}

#[test]
fn prepare_checkpoint_for_mutation_creates_grouped_cluster_checkpoints() {
    let primary = write_execution_profile_workspace(
        "boundline-runtime-checkpoint-primary",
        vec![ExecutionAttemptDefinition {
            attempt_id: "fix-primary".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/lib.rs".to_string(),
                find: "before".to_string(),
                replace: "after".to_string(),
            }],
        }],
    );
    let member = write_execution_profile_workspace(
        "boundline-runtime-checkpoint-member",
        vec![ExecutionAttemptDefinition {
            attempt_id: "fix-member".to_string(),
            summary: String::new(),
            failure_mode: ExecutionFailureMode::Terminal,
            changes: vec![WorkspaceChange {
                path: "src/member.rs".to_string(),
                find: "before".to_string(),
                replace: "after".to_string(),
            }],
        }],
    );
    fs::create_dir_all(primary.join("src")).unwrap();
    fs::create_dir_all(member.join("src")).unwrap();
    fs::write(primary.join("src/lib.rs"), "before").unwrap();
    fs::write(member.join("src/member.rs"), "before").unwrap();

    let mut task =
        decision_task(&primary.to_string_lossy(), json!({"decision": "cluster-checkpoint"}));
    task.context
        .set_cluster_session_projection(&ClusterSessionProjection {
            cluster_id: "cluster-a".to_string(),
            primary_workspace_ref: primary.to_string_lossy().into_owned(),
            member_workspace_refs: vec![
                primary.to_string_lossy().into_owned(),
                member.to_string_lossy().into_owned(),
            ],
            started_from_command: "run".to_string(),
            updated_at: 1,
        })
        .unwrap();
    let mut session = build_session(&primary, task);
    let runtime = SessionRuntime::for_workspace(&primary);

    let projection = runtime
        .prepare_checkpoint_for_mutation(&mut session, SessionCommand::Run)
        .unwrap()
        .unwrap();

    assert_eq!(projection.scope, "cluster");
    assert_eq!(projection.workspace_refs.len(), 2);

    fs::write(primary.join("src/lib.rs"), "after").unwrap();
    fs::write(member.join("src/member.rs"), "after").unwrap();
    runtime.refresh_checkpoint_projection(&session, &projection).unwrap();

    let primary_manifests = FileCheckpointStore::for_session(&primary, &session.session_id)
        .load_group(&projection.checkpoint_id)
        .unwrap();
    let member_manifests = FileCheckpointStore::for_session(&member, &session.session_id)
        .load_group(&projection.checkpoint_id)
        .unwrap();
    assert_eq!(primary_manifests.len(), 1);
    assert_eq!(member_manifests.len(), 1);
}

#[cfg(unix)]
#[test]
fn reset_planning_governance_clears_blocked_records_on_retry_with_same_fingerprint() {
    let workspace = temp_workspace("boundline-runtime-reset-planning-blocked");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements and architecture for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "reset-planning-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-reset-blocked".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    // All 4 stages should be GovernedReady.
    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    assert!(lifecycle.planning_input_fingerprint.is_some());

    // Simulate a blocked stage: mark first two as Blocked while keeping fingerprint unchanged.
    let lifecycle = session.governance_lifecycle.as_mut().unwrap();
    lifecycle.stage_records[0].lifecycle_state = GovernanceLifecycleState::Blocked;
    lifecycle.stage_records[0].blocked_reason = Some("Canon rejected packet".to_string());
    lifecycle.stage_records[1].lifecycle_state = GovernanceLifecycleState::Blocked;
    lifecycle.stage_records[1].blocked_reason = Some("Canon rejected packet".to_string());

    // Re-plan with same fingerprint: blocked records should be cleared and re-executed.
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    assert_eq!(
        lifecycle.stage_records[0].lifecycle_state,
        GovernanceLifecycleState::GovernedReady,
        "previously blocked stage should be re-executed and now GovernedReady"
    );
    assert_eq!(
        lifecycle.stage_records[1].lifecycle_state,
        GovernanceLifecycleState::GovernedReady,
        "previously blocked stage should be re-executed and now GovernedReady"
    );
    assert_eq!(lifecycle.stage_records[2].lifecycle_state, GovernanceLifecycleState::GovernedReady);
    assert_eq!(lifecycle.stage_records[3].lifecycle_state, GovernanceLifecycleState::GovernedReady);
}

#[cfg(unix)]
#[test]
fn reset_planning_governance_preserves_completed_records_when_others_are_blocked() {
    let workspace = temp_workspace("boundline-runtime-reset-planning-mixed");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements and architecture for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "reset-planning-mixed".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-reset-mixed".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    // Mark first two stages as Completed, third as Blocked.
    let lifecycle = session.governance_lifecycle.as_mut().unwrap();
    lifecycle.stage_records[0].lifecycle_state = GovernanceLifecycleState::Completed;
    lifecycle.stage_records[1].lifecycle_state = GovernanceLifecycleState::Completed;
    lifecycle.stage_records[2].lifecycle_state = GovernanceLifecycleState::Blocked;
    lifecycle.stage_records[2].blocked_reason = Some("Canon rejected packet".to_string());

    // Re-plan: completed records stay, blocked record is cleared and retried.
    runtime.plan_task(&mut session, None, false).unwrap();

    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    assert_eq!(
        lifecycle.stage_records[0].lifecycle_state,
        GovernanceLifecycleState::Completed,
        "completed stage should be preserved"
    );
    assert_eq!(
        lifecycle.stage_records[1].lifecycle_state,
        GovernanceLifecycleState::Completed,
        "completed stage should be preserved"
    );
    assert_eq!(
        lifecycle.stage_records[2].lifecycle_state,
        GovernanceLifecycleState::GovernedReady,
        "previously blocked stage should be re-executed and now GovernedReady"
    );
    assert_eq!(lifecycle.stage_records[3].lifecycle_state, GovernanceLifecycleState::GovernedReady);
}

#[cfg(unix)]
#[test]
fn prepare_planning_requests_uses_refresh_when_stage_has_existing_run_ref() {
    use crate::domain::governance::CanonCapabilitySnapshot;

    let workspace = temp_workspace("boundline-runtime-planning-refresh");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(workspace.join("tests")).unwrap();
    fs::write(
        workspace.join("src/lib.rs"),
        "pub fn add(left: i32, right: i32) -> i32 { left + right }\n",
    )
    .unwrap();
    fs::write(
        workspace.join("brief.md"),
        "Deliver the feature through requirements and architecture for src/lib.rs.\n",
    )
    .unwrap();

    let canon_command = write_fake_canon_command(&workspace);
    fs::write(
        workspace.join(".boundline/execution.json"),
        serde_json::to_string_pretty(&WorkspaceExecutionProfile {
            name: "planning-refresh-profile".to_string(),
            read_targets: vec!["src/lib.rs".to_string()],
            validation_command: ExecutionCommand {
                program: "cargo".to_string(),
                args: vec!["test".to_string(), "--quiet".to_string()],
            },
            attempts: vec![ExecutionAttemptDefinition {
                attempt_id: "plan-execution".to_string(),
                summary: String::new(),
                failure_mode: ExecutionFailureMode::Terminal,
                changes: vec![WorkspaceChange {
                    path: "src/lib.rs".to_string(),
                    find: "left + right".to_string(),
                    replace: "left + right".to_string(),
                }],
            }],
            adaptive: None,
            limits: RunLimits::default(),
            governance: Some(GovernanceProfile {
                default_runtime: GovernanceRuntimeKind::Canon,
                canon: Some(CanonRuntimeConfig {
                    command: canon_command.to_string_lossy().into_owned(),
                    default_owner: Some("platform".to_string()),
                    default_risk: Some("medium".to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_system_context: Some(SystemContextBinding::Existing),
                }),
                stages: Vec::new(),
            }),
            review: None,
            legacy_source: None,
        })
        .unwrap(),
    )
    .unwrap();

    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut session = ActiveSessionRecord {
        session_id: "session-planning-refresh".to_string(),
        workspace_ref: workspace.to_string_lossy().into_owned(),
        goal: None,
        authored_brief: Some(
            normalize_inputs_with_governance(
                &workspace,
                Some("Deliver a governed feature"),
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
            .unwrap(),
        ),
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
        created_at: 10,
        updated_at: 10,
        governance_lifecycle: Some(GovernedSessionLifecycle {
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
        }),
        project_scale: None,
        delight_feedback: None,
        latest_voting: None,
    };

    runtime.capture_goal(&mut session, "Deliver a governed feature").unwrap();
    runtime.select_flow(&mut session, "delivery").unwrap();
    runtime.plan_task(&mut session, None, false).unwrap();

    // First plan produces GovernedReady with canon_run_ref from the fake Canon.
    let lifecycle = session.governance_lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.stage_records.len(), 4);
    let first_run_ref = lifecycle.stage_records[0].canon_run_ref.clone();
    assert!(first_run_ref.is_some(), "fake Canon should have set canon_run_ref");

    // Mark first stage as Blocked (simulating a Canon rejection after a prior start).
    let lifecycle = session.governance_lifecycle.as_mut().unwrap();
    lifecycle.stage_records[0].lifecycle_state = GovernanceLifecycleState::Blocked;
    lifecycle.stage_records[0].blocked_reason = Some("Canon rejected packet".to_string());

    // Set up canon_capability_snapshot with "refresh" in operations via active_task context.
    let snapshot = CanonCapabilitySnapshot {
        canon_version: "0.45.0".to_string(),
        supported_schema_versions: vec!["2026-02-01".to_string()],
        operations: vec!["capabilities".to_string(), "start".to_string(), "refresh".to_string()],
        supported_modes: vec![
            CanonMode::Requirements,
            CanonMode::SystemShaping,
            CanonMode::Architecture,
            CanonMode::Backlog,
        ],
        status_values: Vec::new(),
        approval_state_values: Vec::new(),
        packet_readiness_values: Vec::new(),
        compatibility_notes: Vec::new(),
    };
    let mut task_context = TaskContext::new(
        "session-planning-refresh".to_string(),
        workspace.to_string_lossy().into_owned(),
        RunLimits::default(),
        Map::new(),
    );
    task_context.set_latest_canon_capability_snapshot(&snapshot).unwrap();
    session.active_task = Some(Task {
        id: "refresh-probe".to_string(),
        goal: "Deliver a governed feature".to_string(),
        input: json!({}),
        context: task_context,
        plan: Plan {
            revision: 0,
            steps: Vec::new(),
            current_step_index: 0,
            status: crate::domain::plan::PlanStatus::Active,
        },
        status: TaskStatus::Running,
        limits: RunLimits::default(),
        terminal_reason: None,
        retry_count: 0,
        replan_count: 0,
        total_step_attempts: 0,
    });

    let goal = session.goal.clone().unwrap();
    let context_sources = runtime.planning_context_sources(&session, &goal);
    assert!(
        context_sources.canon_capability_snapshot.is_some(),
        "capability snapshot must be available for refresh test"
    );

    let goal_plan = session.goal_plan.as_ref().unwrap().clone();
    let requests = runtime
        .prepare_planning_governance_requests(&mut session, &goal_plan, &context_sources)
        .unwrap();

    // The first stage should use Refresh (it has an existing canon_run_ref).
    assert_eq!(requests[0].request.stage_key, "plan:requirements");
    assert_eq!(
        requests[0].request.request_kind,
        super::GovernanceRequestKind::Refresh,
        "retrying a blocked stage with existing run_ref should use Refresh"
    );
    assert_eq!(
        requests[0].request.run_ref.as_ref(),
        first_run_ref.as_ref(),
        "refresh request should carry the previous canon_run_ref"
    );

    // Stages without prior canon_run_ref should use Start.
    // Stages 1-3 were GovernedReady (which means they already have run refs too from the first
    // plan_task), so they may also use Refresh. The second stage should show Refresh as well.
    assert_eq!(
        requests[1].request.request_kind,
        super::GovernanceRequestKind::Refresh,
        "stage with existing run_ref and refresh capability should use Refresh"
    );
}
