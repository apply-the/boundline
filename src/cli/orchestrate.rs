use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore};
use crate::cli::CommandExitStatus;
use crate::cli::assistant_assets::AssistantHost;
use crate::cli::session::{self, SessionCommandError};
use crate::domain::governance::{
    ApprovalState, CanonMode, CompactedCanonMemory, GovernanceLifecycleState,
    GovernanceRuntimeKind, MemoryCredibilityState, planning_canon_mode_for_stage_key,
    planning_canon_mode_sequence, planning_stage_brief_ref, planning_stage_key_for_mode,
};
use crate::domain::session::ActiveSessionRecord;
use crate::domain::session::{SessionStatus, SessionStatusView, task_state_compacted_canon_memory};
use crate::domain::trace::{TraceSummaryView, current_timestamp_millis};
use crate::orchestrator::session_runtime::{decompose_goal_text, planning_unknown_markers};

const EVENT_KIND_SESSION_OPENED: &str = "session_opened";
const EVENT_KIND_SESSION_UPDATED: &str = "session_updated";
const EVENT_KIND_SESSION_RESUMED: &str = "session_resumed";
const EVENT_KIND_PHASE_STARTED: &str = "phase_started";
const EVENT_KIND_PHASE_REQUEST: &str = "phase_request";
const EVENT_KIND_ARTIFACT_RECORDED: &str = "artifact_recorded";
const EVENT_KIND_GOVERNANCE_UPDATE: &str = "governance_update";
const EVENT_KIND_EXECUTION_UPDATE: &str = "execution_update";
const EVENT_KIND_TERMINAL: &str = "terminal";
const EVENT_KIND_PHASE_REQUEST_ANSWERED: &str = "phase_request_answered";
const PHASE_REQUEST_KIND_CLARIFICATION: &str = "clarification";
const PHASE_REQUEST_KIND_REVIEW: &str = "review";
const PHASE_REQUEST_EXPECTED_ANSWER_CONFIRMATION: &str = "confirmation";
const PHASE_REQUEST_EXPECTED_ANSWER_FREE_TEXT: &str = "free_text";
const PHASE_REQUEST_EXPECTED_ANSWER_SINGLE_CHOICE: &str = "single_choice";
const PHASE_REQUEST_ID_PREFIX: &str = "req";
const PHASE_REQUEST_QUESTION_FRAGMENT_MAX_CHARS: usize = 24;
const PHASE_KIND_GOAL: &str = "goal_capture";
const PHASE_KIND_PLANNING: &str = "planning";
const PHASE_KIND_EXECUTION: &str = "execution";
const STAGE_KEY_GOAL: &str = "goal";
const STAGE_KEY_PLAN: &str = "plan";
const STAGE_KEY_RUN: &str = "run";
const ROUTE_SLOT_PLANNING: &str = "planning";
const ROUTE_SLOT_IMPLEMENTATION: &str = "implementation";
const ROUTE_SLOT_REVIEW: &str = "review";
const ROUTE_SLOT_ADJUDICATION: &str = "adjudication";
const ROUTING_PROJECTION_SLOT_SEPARATOR: char = '=';
const ROUTING_PROJECTION_MODEL_SEPARATOR: char = '/';
const ROUTING_PROJECTION_SOURCE_SEPARATOR: &str = " [";
const PLANNING_STAGE_REQUEST_ID_REQUIRES_ACTIVE_SESSION: &str =
    "planning stage completion request_id requires an active session";
const PLANNING_STAGE_REQUEST_ID_REQUIRES_ACTIVE_PHASE_REQUEST: &str =
    "planning stage completion request_id requires an active phase_request";
const GOAL_CLARIFICATION_ANSWER_REQUIRES_REQUEST_ID: &str =
    "goal clarification answer requires a request_id";
const GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_SESSION: &str =
    "goal clarification answer requires an active session";
const GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_PHASE_REQUEST: &str =
    "goal clarification answer requires an active goal phase_request";
const GOAL_CLARIFICATION_ANSWER_REQUIRES_NON_EMPTY_TEXT: &str =
    "goal clarification answer must not be empty";

#[derive(Debug, Default)]
struct OrchestrateEventMetadata {
    actor_kind: Option<String>,
    actor_name: Option<String>,
    runtime_kind: Option<String>,
    provider: Option<String>,
    route_slot: Option<String>,
    model_name: Option<String>,
    decision_family: Option<String>,
    review_step: Option<String>,
    vote_summary: Option<String>,
    adjudication_summary: Option<String>,
    governance_mode: Option<String>,
}

#[derive(Debug)]
struct RoutingProjectionSelection {
    runtime_kind: String,
    model_name: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrateIntent {
    #[value(name = "plan", alias = "plan-only")]
    PlanOnly,
    #[value(name = "phase-request", alias = "continue-until-phase-request")]
    #[default]
    ContinueUntilPhaseRequest,
    #[value(name = "terminal", alias = "continue-until-terminal")]
    ContinueUntilTerminal,
}

impl OrchestrateIntent {
    pub const fn as_cli_arg(self) -> &'static str {
        match self {
            Self::PlanOnly => "plan",
            Self::ContinueUntilPhaseRequest => "phase-request",
            Self::ContinueUntilTerminal => "terminal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrateArtifactKind {
    GoalBrief,
    PlanBrief,
    PlanningStageBrief,
    RunBrief,
    Clarification,
    RequirementsDoc,
    PrdDoc,
    ArchitectureDoc,
    DomainModelDoc,
    BacklogDoc,
    CanonPacket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanningStagePhaseRequest {
    stage_key: String,
    stage_label: String,
    artifacts: Vec<OrchestrateArtifactRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrateArtifactRef {
    pub artifact_kind: OrchestrateArtifactKind,
    pub artifact_ref: String,
}

/// A selectable option shown to the user during a phase_request.
/// The host renders `label` as the short clickable choice and uses `value`
/// as the full text sent back as the clarification answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRequestOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestratePhaseRequestExpectedAnswer {
    #[serde(rename = "type")]
    pub answer_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<PhaseRequestOption>,
}

impl OrchestratePhaseRequestExpectedAnswer {
    #[allow(dead_code)]
    fn free_text() -> Self {
        Self {
            answer_type: PHASE_REQUEST_EXPECTED_ANSWER_FREE_TEXT.to_string(),
            options: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn single_choice(options: Vec<PhaseRequestOption>) -> Self {
        Self { answer_type: PHASE_REQUEST_EXPECTED_ANSWER_SINGLE_CHOICE.to_string(), options }
    }

    fn confirmation() -> Self {
        Self {
            answer_type: PHASE_REQUEST_EXPECTED_ANSWER_CONFIRMATION.to_string(),
            options: Vec::new(),
        }
    }

    /// Builds the appropriate expected-answer type for a clarification question,
    /// using predefined options when the question matches a well-known pattern.
    /// The answer type remains `free_text` so the host renders suggestions
    /// while still accepting a custom typed answer.
    fn for_clarification_question(question: &str) -> Self {
        let options = clarification_question_options(question);
        Self { answer_type: PHASE_REQUEST_EXPECTED_ANSWER_FREE_TEXT.to_string(), options }
    }

    /// Builds the expected-answer for a planning stage question when the Canon
    /// packet is incomplete. Always `free_text` so the user can provide a
    /// file/folder path or custom instruction alongside the suggested options.
    fn for_planning_stage_question(
        canon_memory: Option<&CompactedCanonMemory>,
        stage_key: &str,
    ) -> Self {
        let options = planning_stage_question_options(canon_memory, stage_key);
        Self { answer_type: PHASE_REQUEST_EXPECTED_ANSWER_FREE_TEXT.to_string(), options }
    }
}

/// Builds suggested answer options for a planning stage phase_request based on
/// the Canon memory state and the specific planning stage.
///
/// When the packet is incomplete, options guide the assistant or human toward
/// filling or providing reference material. For Discovery and SystemShaping
/// stages, options are tailored to the stage's purpose (scope confirmation
/// vs domain model approval).
fn planning_stage_question_options(
    canon_memory: Option<&CompactedCanonMemory>,
    stage_key: &str,
) -> Vec<PhaseRequestOption> {
    let is_incomplete =
        canon_memory.is_some_and(|memory| memory.credibility != MemoryCredibilityState::Credible);

    // Stage-specific options for Discovery and SystemShaping take precedence
    // over the generic incomplete/credible split when the stage is first
    // surfaced to the host (Canon may not have run yet, so there's no packet).
    match stage_key {
        "plan:discovery" => {
            if is_incomplete {
                return vec![
                    opt(
                        "fill discovery gaps",
                        "proceed: fill discovery gaps using available project context and goal text",
                    ),
                    opt(
                        "fill from best practices",
                        "proceed: fill discovery gaps using industry best practices and established conventions for the detected technology stack",
                    ),
                    opt(
                        "narrow scope",
                        "I want to narrow the scope: I will specify which entities and operations to include",
                    ),
                    opt(
                        "inspect packet",
                        "I want to inspect the discovery packet before proceeding",
                    ),
                ];
            }
            return vec![
                opt(
                    "scope confirmed",
                    "the discovery scope is confirmed and complete; proceed to requirements",
                ),
                opt(
                    "narrow scope",
                    "narrow the scope: I will specify which entities and operations to include",
                ),
                opt("revise discovery", "I want to revise the discovery output before proceeding"),
            ];
        }
        "plan:system-shaping" => {
            if is_incomplete {
                return vec![
                    opt(
                        "fill domain model",
                        "proceed: generate the domain model and system boundaries from available context",
                    ),
                    opt(
                        "fill from best practices",
                        "proceed: generate the domain model and system boundaries using industry best practices and established conventions for the detected technology stack",
                    ),
                    opt(
                        "provide domain docs",
                        "I will provide existing domain documentation as source material",
                    ),
                    opt(
                        "inspect packet",
                        "I want to inspect the system-shaping packet before proceeding",
                    ),
                ];
            }
            return vec![
                opt(
                    "domain model approved",
                    "the domain model and system boundaries are correct; proceed to architecture",
                ),
                opt(
                    "revise boundaries",
                    "I want to revise the system boundaries before proceeding",
                ),
            ];
        }
        _ => {}
    }

    // Generic options for Requirements, Architecture, Backlog
    if is_incomplete {
        vec![
            opt(
                "fill using context",
                "proceed: fill placeholder sections using available project context",
            ),
            opt(
                "fill from best practices",
                "proceed: fill placeholder sections using industry best practices and established conventions for the detected technology stack",
            ),
            opt(
                "provide reference path",
                "I will provide a reference file or folder path as source material",
            ),
            opt("inspect manually", "I want to inspect and edit the packet manually first"),
        ]
    } else {
        vec![
            opt("proceed", "proceed: the planning brief is ready"),
            opt("revise first", "I want to revise the planning brief before continuing"),
        ]
    }
}

/// Constructs a [`PhaseRequestOption`] from short label and full-sentence value.
fn opt(label: &str, value: &str) -> PhaseRequestOption {
    PhaseRequestOption { label: label.to_string(), value: value.to_string() }
}

/// Maps well-known clarification questions to their suggested answer options.
/// Returns an empty vec for questions without predefined alternatives.
/// Each option has a short `label` for display and a full-sentence `value`
/// recorded as the clarification answer.
fn clarification_question_options(question: &str) -> Vec<PhaseRequestOption> {
    if question.contains("persistence store") {
        return vec![
            opt("PostgreSQL", "PostgreSQL as the primary relational persistence store"),
            opt("SQLite", "SQLite as a lightweight embedded relational store"),
            opt("in-memory", "in-memory store with no external database dependency for this slice"),
            opt("Redis", "Redis as the primary key-value or caching store"),
            opt("DynamoDB", "DynamoDB as the managed NoSQL persistence store"),
            opt("MongoDB", "MongoDB as the document-oriented persistence store"),
        ];
    }
    if question.contains("OAuth2") || question.contains("authentication stop") {
        return vec![
            opt(
                "gateway + JWT claims",
                "gateway handles OAuth2; service uses validated JWT claims",
            ),
            opt("service validates directly", "service validates OAuth2 tokens directly"),
            opt(
                "external IdP + authz only",
                "external IdP issues tokens; service manages authorization only",
            ),
        ];
    }
    if question.contains("API operations") || question.contains("endpoints") {
        return vec![
            opt("REST CRUD", "REST CRUD endpoints over HTTP"),
            opt("gRPC", "gRPC service methods defined in protobuf"),
            opt("GraphQL", "GraphQL queries and mutations"),
        ];
    }
    if question.contains("validation command") || question.contains("acceptance evidence") {
        return vec![
            opt("cargo test", "cargo test (Rust unit and integration tests in the workspace)"),
            opt("npm test", "npm test (JavaScript/TypeScript test suite)"),
            opt("pytest", "pytest (Python test suite)"),
            opt("manual verification", "manual verification with a smoke-test script"),
        ];
    }
    if question.contains("role semantics") || question.contains("permissions") {
        return vec![
            opt("RBAC", "RBAC with a fixed role hierarchy"),
            opt("ABAC", "ABAC with attribute-based policies"),
            opt("admin/user split", "simple owner/admin/member model with no complex hierarchy"),
        ];
    }
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestratePhaseRequest {
    pub request_id: String,
    pub kind: String,
    pub phase: String,
    pub reason: String,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_answer: Option<OrchestratePhaseRequestExpectedAnswer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrateEventEnvelope {
    pub event_id: String,
    pub timestamp_ms: u64,
    pub event_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_slot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_step: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vote_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adjudication_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_key: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<OrchestrateArtifactRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_request: Option<OrchestratePhaseRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_resume_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assistant_next_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<SessionStatusView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_summary: Option<TraceSummaryView>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrchestrateCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
    pub session_status: Option<SessionStatusView>,
    pub trace_summary: Option<TraceSummaryView>,
    pub events: Vec<OrchestrateEventEnvelope>,
}

#[derive(Debug, Error)]
pub enum OrchestrateCommandError {
    #[error(transparent)]
    Session(#[from] SessionCommandError),
}

#[allow(clippy::too_many_arguments)]
pub fn execute_orchestrate(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    flow: Option<&str>,
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    intent: OrchestrateIntent,
    planning_stage_complete: Option<&str>,
    request_id: Option<&str>,
    answer: Option<&str>,
    assistant_host: Option<AssistantHost>,
    no_canon: bool,
    slug: Option<&str>,
) -> Result<OrchestrateCommandReport, OrchestrateCommandError> {
    let mut events = Vec::new();
    let mut event_counter = 0usize;
    let mut seen_artifacts = BTreeSet::new();
    let mut latest_session_status: Option<SessionStatusView> = None;
    let mut latest_trace_summary: Option<TraceSummaryView> = None;
    let mut latest_trace_location: Option<String> = None;
    let mut latest_terminal_output = String::new();
    let workspace_hint = workspace.or(cluster);
    let brief_only_planning_input = goal.is_none()
        && briefs.len() == 1
        && governance.is_none()
        && risk.is_none()
        && zone.is_none()
        && owner.is_none();
    let mut planning_input: Option<&Path> = None;

    if answer.is_some_and(|value| value.trim().is_empty()) {
        return Err(SessionCommandError::InvalidRequest(
            GOAL_CLARIFICATION_ANSWER_REQUIRES_NON_EMPTY_TEXT.to_string(),
        )
        .into());
    }

    if brief_only_planning_input {
        match session::execute_status_with_target(workspace, cluster, None) {
            Ok(status_report) if status_report.session_status.is_some() => {
                latest_terminal_output = status_report.terminal_output.clone();
                latest_trace_location = status_report.trace_location.clone();
                latest_session_status = status_report.session_status.clone();
                latest_trace_summary = status_report.trace_summary.clone();
                push_event(
                    &mut events,
                    &mut event_counter,
                    EVENT_KIND_SESSION_RESUMED,
                    latest_session_status.as_ref(),
                    None,
                    None,
                    "resuming the active session".to_string(),
                    None,
                    None,
                    None,
                    latest_session_status.clone(),
                    latest_trace_summary.clone(),
                );
                planning_input = briefs.first().map(|path| path.as_path());
            }
            Ok(_) | Err(SessionCommandError::MissingActiveSession) => {
                let goal_report = session::execute_goal_with_target(
                    workspace, cluster, None, briefs, governance, risk, zone, owner, slug,
                )?;
                latest_terminal_output = goal_report.terminal_output.clone();
                latest_trace_location = goal_report.trace_location.clone();
                latest_session_status = goal_report.session_status.clone();
                latest_trace_summary = goal_report.trace_summary.clone();
                push_event(
                    &mut events,
                    &mut event_counter,
                    EVENT_KIND_SESSION_OPENED,
                    latest_session_status.as_ref(),
                    Some(PHASE_KIND_GOAL.to_string()),
                    Some(STAGE_KEY_GOAL.to_string()),
                    "opened a new session and captured the goal from the brief".to_string(),
                    None,
                    None,
                    None,
                    latest_session_status.clone(),
                    latest_trace_summary.clone(),
                );
                push_artifact_events(
                    &mut events,
                    &mut event_counter,
                    &mut seen_artifacts,
                    latest_session_status.as_ref(),
                    latest_trace_summary.as_ref(),
                );
                if let Some(view) = latest_session_status.as_ref()
                    && clarification_requested(view)
                {
                    let question = clarification_question(view);
                    let resume_request_id = question.as_deref().map(|question| {
                        phase_request_id(
                            &view.session_id,
                            PHASE_KIND_GOAL,
                            STAGE_KEY_GOAL,
                            Some(question),
                        )
                    });
                    let resume = clarification_resume_command(
                        workspace_hint,
                        OrchestrateIntent::ContinueUntilPhaseRequest,
                        resume_request_id.as_deref(),
                    );
                    push_phase_request(
                        &mut events,
                        &mut event_counter,
                        view,
                        PhaseRequestPayload {
                            phase_kind: PHASE_KIND_GOAL,
                            stage_key: STAGE_KEY_GOAL,
                            artifact: clarification_artifact(view),
                            message: view.clarification_headline.clone().unwrap_or_else(|| {
                                "clarification is required before planning can continue"
                                    .to_string()
                            }),
                            phase_request_kind: question
                                .as_deref()
                                .map(|_| PHASE_REQUEST_KIND_CLARIFICATION),
                            question: question.clone(),
                            expected_answer: question
                                .as_deref()
                                .map(OrchestratePhaseRequestExpectedAnswer::for_clarification_question),
                            instruction: question
                                .as_ref()
                                .map(|question| {
                                    format!(
                                        "answer this question before planning continues: {question}"
                                    )
                                })
                                .or_else(|| {
                                    Some(
                                        "update the goal brief with the missing context, then resume orchestration"
                                            .to_string(),
                                    )
                                }),
                            resume_command: Some(resume),
                        },
                        assistant_host,
                    );
                    return Ok(build_report(
                        CommandExitStatus::Succeeded,
                        latest_terminal_output,
                        events,
                        latest_trace_location,
                        latest_session_status,
                        latest_trace_summary,
                    ));
                }
                planning_input = briefs.first().map(|path| path.as_path());
            }
            Err(error) => return Err(error.into()),
        }
    }

    if let Some(answer) = answer.map(str::trim) {
        let status_report =
            session::execute_status_with_target(workspace, cluster, None).map_err(|error| {
                match error {
                    SessionCommandError::MissingActiveSession => {
                        SessionCommandError::InvalidRequest(
                            GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_SESSION.to_string(),
                        )
                    }
                    other => other,
                }
            })?;
        latest_session_status = status_report.session_status.clone();

        let request_id = request_id.ok_or_else(|| {
            SessionCommandError::InvalidRequest(
                GOAL_CLARIFICATION_ANSWER_REQUIRES_REQUEST_ID.to_string(),
            )
        })?;
        let view = latest_session_status.as_ref().ok_or_else(|| {
            SessionCommandError::InvalidRequest(
                GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_SESSION.to_string(),
            )
        })?;
        let session_workspace = Path::new(&view.workspace_ref);
        load_active_session_record(session_workspace)?.ok_or_else(|| {
            SessionCommandError::InvalidRequest(
                GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_SESSION.to_string(),
            )
        })?;
        validate_goal_clarification_request_id(view, request_id)?;

        let answer_report = session::execute_goal_clarification_answer_with_target(
            Some(session_workspace),
            None,
            answer,
        )?;
        latest_terminal_output = answer_report.terminal_output.clone();
        latest_trace_location = answer_report.trace_location.clone();
        latest_session_status = answer_report.session_status.clone();
        latest_trace_summary = answer_report.trace_summary.clone();
        push_event(
            &mut events,
            &mut event_counter,
            EVENT_KIND_PHASE_REQUEST_ANSWERED,
            latest_session_status.as_ref(),
            Some(PHASE_KIND_GOAL.to_string()),
            Some(STAGE_KEY_GOAL.to_string()),
            format!("recorded clarification answer for request {request_id}"),
            None,
            None,
            None,
            latest_session_status.clone(),
            None,
        );
        push_event(
            &mut events,
            &mut event_counter,
            EVENT_KIND_SESSION_UPDATED,
            latest_session_status.as_ref(),
            Some(PHASE_KIND_GOAL.to_string()),
            Some(STAGE_KEY_GOAL.to_string()),
            "applied the clarification answer to the active goal".to_string(),
            None,
            None,
            None,
            latest_session_status.clone(),
            None,
        );
        push_artifact_events(
            &mut events,
            &mut event_counter,
            &mut seen_artifacts,
            latest_session_status.as_ref(),
            latest_trace_summary.as_ref(),
        );
        if let Some(view) = latest_session_status.as_ref()
            && clarification_requested(view)
        {
            let question = clarification_question(view);
            let resume_request_id = question.as_deref().map(|question| {
                phase_request_id(&view.session_id, PHASE_KIND_GOAL, STAGE_KEY_GOAL, Some(question))
            });
            let resume = clarification_resume_command(
                workspace_hint,
                OrchestrateIntent::ContinueUntilPhaseRequest,
                resume_request_id.as_deref(),
            );
            push_phase_request(
                &mut events,
                &mut event_counter,
                view,
                PhaseRequestPayload {
                    phase_kind: PHASE_KIND_GOAL,
                    stage_key: STAGE_KEY_GOAL,
                    artifact: clarification_artifact(view),
                    message: view.clarification_headline.clone().unwrap_or_else(|| {
                        "clarification is required before planning can continue".to_string()
                    }),
                    phase_request_kind: question
                        .as_deref()
                        .map(|_| PHASE_REQUEST_KIND_CLARIFICATION),
                    question: question.clone(),
                    expected_answer: question
                        .as_deref()
                        .map(OrchestratePhaseRequestExpectedAnswer::for_clarification_question),
                    instruction: question.as_ref().map(|question| {
                        format!("answer this question before planning continues: {question}")
                    }).or_else(|| {
                        Some(
                            "update the goal brief with the missing context, then resume orchestration"
                                .to_string(),
                        )
                    }),
                    resume_command: Some(resume),
                },
                assistant_host,
            );
            return Ok(build_report(
                CommandExitStatus::Succeeded,
                latest_terminal_output,
                events,
                latest_trace_location,
                latest_session_status,
                latest_trace_summary,
            ));
        }
    }

    let has_new_goal_input = goal.is_some()
        || (!briefs.is_empty() && planning_input.is_none())
        || governance.is_some()
        || risk.is_some()
        || zone.is_some()
        || owner.is_some();
    let mut completed_planning_stage: Option<String> = None;

    if has_new_goal_input {
        let (goal_report, session_event_kind, session_event_message) =
            match session::execute_goal_update_with_target(
                workspace, cluster, goal, briefs, governance, risk, zone, owner,
            ) {
                Ok(report) => (
                    report,
                    EVENT_KIND_SESSION_UPDATED,
                    "updated the active session and captured the requested goal".to_string(),
                ),
                Err(
                    SessionCommandError::MissingActiveSession
                    | SessionCommandError::GoalUpdateRequiresNewSession { .. },
                ) => (
                    session::execute_goal_with_target(
                        workspace, cluster, goal, briefs, governance, risk, zone, owner, slug,
                    )?,
                    EVENT_KIND_SESSION_OPENED,
                    "opened a new session and captured the requested goal".to_string(),
                ),
                Err(error) => return Err(error.into()),
            };
        latest_terminal_output = goal_report.terminal_output.clone();
        latest_trace_location = goal_report.trace_location.clone();
        latest_session_status = goal_report.session_status.clone();
        latest_trace_summary = goal_report.trace_summary.clone();
        push_event(
            &mut events,
            &mut event_counter,
            session_event_kind,
            latest_session_status.as_ref(),
            Some(PHASE_KIND_GOAL.to_string()),
            Some(STAGE_KEY_GOAL.to_string()),
            session_event_message,
            None,
            None,
            None,
            latest_session_status.clone(),
            None,
        );
        push_artifact_events(
            &mut events,
            &mut event_counter,
            &mut seen_artifacts,
            latest_session_status.as_ref(),
            latest_trace_summary.as_ref(),
        );
        if let Some(view) = latest_session_status.as_ref()
            && clarification_requested(view)
        {
            let question = clarification_question(view);
            let resume_request_id = question.as_deref().map(|question| {
                phase_request_id(&view.session_id, PHASE_KIND_GOAL, STAGE_KEY_GOAL, Some(question))
            });
            let resume = clarification_resume_command(
                workspace_hint,
                OrchestrateIntent::ContinueUntilPhaseRequest,
                resume_request_id.as_deref(),
            );
            push_phase_request(
                &mut events,
                &mut event_counter,
                view,
                PhaseRequestPayload {
                    phase_kind: PHASE_KIND_GOAL,
                    stage_key: STAGE_KEY_GOAL,
                    artifact: clarification_artifact(view),
                    message: view.clarification_headline.clone().unwrap_or_else(|| {
                        "clarification is required before planning can continue".to_string()
                    }),
                    phase_request_kind: question
                        .as_deref()
                        .map(|_| PHASE_REQUEST_KIND_CLARIFICATION),
                    question: question.clone(),
                    expected_answer: question
                        .as_deref()
                        .map(OrchestratePhaseRequestExpectedAnswer::for_clarification_question),
                    instruction: question.as_ref().map(|question| {
                        format!("answer this question before planning continues: {question}")
                    }).or_else(|| {
                        Some(
                            "update the goal brief with the missing context, then resume orchestration"
                                .to_string(),
                        )
                    }),
                    resume_command: Some(resume),
                },
                assistant_host,
            );
            return Ok(build_report(
                CommandExitStatus::Succeeded,
                latest_terminal_output,
                events,
                latest_trace_location,
                latest_session_status,
                latest_trace_summary,
            ));
        }
    } else if latest_session_status.is_none() {
        match session::execute_status_with_target(workspace, cluster, None) {
            Ok(status_report) => {
                latest_terminal_output = status_report.terminal_output.clone();
                latest_trace_location = status_report.trace_location.clone();
                latest_session_status = status_report.session_status.clone();
                latest_trace_summary = status_report.trace_summary.clone();
                push_event(
                    &mut events,
                    &mut event_counter,
                    EVENT_KIND_SESSION_RESUMED,
                    latest_session_status.as_ref(),
                    None,
                    None,
                    "resuming the active session".to_string(),
                    None,
                    None,
                    None,
                    latest_session_status.clone(),
                    latest_trace_summary.clone(),
                );
            }
            Err(SessionCommandError::MissingActiveSession) => {
                latest_terminal_output =
                    "no active session found; provide a goal or brief before orchestration can continue"
                        .to_string();
                latest_trace_location = None;
                latest_session_status = None;
                latest_trace_summary = None;
                let resume = format!(
                    "{} --goal \"<goal>\"",
                    resume_command(
                        workspace_hint,
                        OrchestrateIntent::ContinueUntilPhaseRequest,
                        None,
                    )
                );
                push_event(
                    &mut events,
                    &mut event_counter,
                    EVENT_KIND_PHASE_REQUEST,
                    None,
                    Some(PHASE_KIND_GOAL.to_string()),
                    Some(STAGE_KEY_GOAL.to_string()),
                    "provide a goal or brief before orchestration can continue".to_string(),
                    None,
                    Some(
                        "capture the requested goal with `boundline goal --goal <goal>` or pass --goal on resume"
                            .to_string(),
                    ),
                    Some(resume),
                    None,
                    None,
                );
                return Ok(build_report(
                    CommandExitStatus::Succeeded,
                    latest_terminal_output,
                    events,
                    latest_trace_location,
                    latest_session_status,
                    latest_trace_summary,
                ));
            }
            Err(error) => return Err(error.into()),
        }
    }

    let resolved_workspace =
        latest_session_status.as_ref().map(|view| PathBuf::from(view.workspace_ref.clone()));
    let mut active_record =
        resolved_workspace.as_deref().map(load_active_session_record).transpose()?.flatten();

    if let Some(stage_key) = planning_stage_complete {
        let Some(session_workspace) = resolved_workspace.as_deref() else {
            return Err(SessionCommandError::InvalidRequest(
                "planning stage completion requires an active session workspace".to_string(),
            )
            .into());
        };

        if let Some(request_id) = request_id {
            let Some(record) = active_record.as_ref() else {
                return Err(SessionCommandError::InvalidRequest(
                    PLANNING_STAGE_REQUEST_ID_REQUIRES_ACTIVE_SESSION.to_string(),
                )
                .into());
            };
            let Some(view) = latest_session_status.as_ref() else {
                return Err(SessionCommandError::InvalidRequest(
                    PLANNING_STAGE_REQUEST_ID_REQUIRES_ACTIVE_SESSION.to_string(),
                )
                .into());
            };
            validate_planning_stage_completion_request_id(record, view, stage_key, request_id)?;
        }

        let updated_record = complete_planning_stage(session_workspace, stage_key)?;
        completed_planning_stage = Some(stage_key.to_string());
        active_record = Some(updated_record);

        let status_report =
            session::execute_status_with_target(Some(session_workspace), None, None)?;
        latest_terminal_output = status_report.terminal_output.clone();
        latest_trace_location = status_report.trace_location.clone();
        latest_session_status = status_report.session_status.clone();
        latest_trace_summary = status_report.trace_summary.clone();
    }

    if !has_new_goal_input
        && matches!(intent, OrchestrateIntent::ContinueUntilPhaseRequest)
        && let (Some(record), Some(view)) = (active_record.as_ref(), latest_session_status.as_ref())
        && let Some(request) = next_pending_planning_phase_request(record, view)
    {
        if let Some(stage_key) = completed_planning_stage.as_deref() {
            push_planning_stage_completion_event(&mut events, &mut event_counter, view, stage_key);
        }
        push_planning_stage_artifact_events(
            &mut events,
            &mut event_counter,
            &mut seen_artifacts,
            Some(record),
            Some(view),
        );
        push_planning_stage_phase_request(
            &mut events,
            &mut event_counter,
            view,
            record,
            request,
            workspace_hint,
            assistant_host,
        );
        return Ok(build_report(
            CommandExitStatus::Succeeded,
            latest_terminal_output,
            events,
            latest_trace_location,
            latest_session_status,
            latest_trace_summary,
        ));
    }

    if matches!(intent, OrchestrateIntent::PlanOnly | OrchestrateIntent::ContinueUntilPhaseRequest)
        || should_plan(latest_session_status.as_ref())
    {
        push_event(
            &mut events,
            &mut event_counter,
            EVENT_KIND_PHASE_STARTED,
            latest_session_status.as_ref(),
            Some(PHASE_KIND_PLANNING.to_string()),
            Some(STAGE_KEY_PLAN.to_string()),
            "starting the planning phase".to_string(),
            None,
            None,
            None,
            latest_session_status.clone(),
            latest_trace_summary.clone(),
        );

        let plan_report = session::execute_plan_with_target_input(
            workspace,
            cluster,
            flow,
            false,
            no_canon,
            planning_input,
        )?;
        latest_terminal_output = plan_report.terminal_output.clone();
        latest_trace_location = plan_report.trace_location.clone();
        latest_session_status = plan_report.session_status.clone();
        latest_trace_summary = plan_report.trace_summary.clone();
        active_record =
            resolved_workspace.as_deref().map(load_active_session_record).transpose()?.flatten();
        push_artifact_events(
            &mut events,
            &mut event_counter,
            &mut seen_artifacts,
            latest_session_status.as_ref(),
            latest_trace_summary.as_ref(),
        );
        push_planning_stage_artifact_events(
            &mut events,
            &mut event_counter,
            &mut seen_artifacts,
            active_record.as_ref(),
            latest_session_status.as_ref(),
        );
        push_status_update_events(&mut events, &mut event_counter, latest_session_status.as_ref());

        if let Some(view) = latest_session_status.as_ref()
            && clarification_requested(view)
        {
            let question = clarification_question(view);
            let resume_request_id = question.as_deref().map(|question| {
                phase_request_id(
                    &view.session_id,
                    PHASE_KIND_PLANNING,
                    STAGE_KEY_PLAN,
                    Some(question),
                )
            });
            let resume = clarification_resume_command(
                workspace_hint,
                OrchestrateIntent::ContinueUntilPhaseRequest,
                resume_request_id.as_deref(),
            );
            push_phase_request(
                &mut events,
                &mut event_counter,
                view,
                PhaseRequestPayload {
                    phase_kind: PHASE_KIND_PLANNING,
                    stage_key: STAGE_KEY_PLAN,
                    artifact: clarification_artifact(view),
                    message: view.clarification_headline.clone().unwrap_or_else(|| {
                        "clarification is required before planning can continue".to_string()
                    }),
                    phase_request_kind: question
                        .as_deref()
                        .map(|_| PHASE_REQUEST_KIND_CLARIFICATION),
                    question: question.clone(),
                    expected_answer: question
                        .as_deref()
                        .map(OrchestratePhaseRequestExpectedAnswer::for_clarification_question),
                    instruction: question.as_ref().map(|question| {
                        format!("answer this question before planning continues: {question}")
                    }),
                    resume_command: Some(resume),
                },
                assistant_host,
            );
            return Ok(build_report(
                CommandExitStatus::Succeeded,
                latest_terminal_output,
                events,
                latest_trace_location,
                latest_session_status,
                latest_trace_summary,
            ));
        }

        if matches!(intent, OrchestrateIntent::PlanOnly) {
            push_terminal_event(
                &mut events,
                &mut event_counter,
                latest_session_status.as_ref(),
                latest_trace_summary.as_ref(),
                "planning completed and orchestration stopped at the requested plan boundary"
                    .to_string(),
            );
            return Ok(build_report(
                CommandExitStatus::Succeeded,
                latest_terminal_output,
                events,
                latest_trace_location,
                latest_session_status,
                latest_trace_summary,
            ));
        }

        if matches!(intent, OrchestrateIntent::ContinueUntilPhaseRequest)
            && let Some(view) = latest_session_status.as_ref()
            && let Some(record) = active_record.as_ref()
            && let Some(request) = next_pending_planning_phase_request(record, view)
        {
            if let Some(stage_key) = completed_planning_stage.as_deref() {
                push_planning_stage_completion_event(
                    &mut events,
                    &mut event_counter,
                    view,
                    stage_key,
                );
            }
            push_planning_stage_phase_request(
                &mut events,
                &mut event_counter,
                view,
                record,
                request,
                workspace_hint,
                assistant_host,
            );
            return Ok(build_report(
                CommandExitStatus::Succeeded,
                latest_terminal_output,
                events,
                latest_trace_location,
                latest_session_status,
                latest_trace_summary,
            ));
        }
    }

    if let (Some(stage_key), Some(view)) =
        (completed_planning_stage.as_deref(), latest_session_status.as_ref())
    {
        push_planning_stage_completion_event(&mut events, &mut event_counter, view, stage_key);
    }

    push_event(
        &mut events,
        &mut event_counter,
        EVENT_KIND_PHASE_STARTED,
        latest_session_status.as_ref(),
        Some(PHASE_KIND_EXECUTION.to_string()),
        Some(STAGE_KEY_RUN.to_string()),
        "starting the execution phase".to_string(),
        None,
        None,
        None,
        latest_session_status.clone(),
        latest_trace_summary.clone(),
    );

    let run_report = session::execute_run_with_target(workspace, cluster)?;
    latest_terminal_output = run_report.terminal_output.clone();
    latest_trace_location = run_report.trace_location.clone();
    latest_session_status = run_report.session_status.clone().or(latest_session_status);
    latest_trace_summary = run_report.trace_summary.clone();
    push_artifact_events(
        &mut events,
        &mut event_counter,
        &mut seen_artifacts,
        latest_session_status.as_ref(),
        latest_trace_summary.as_ref(),
    );
    push_status_update_events(&mut events, &mut event_counter, latest_session_status.as_ref());
    push_terminal_event(
        &mut events,
        &mut event_counter,
        latest_session_status.as_ref(),
        latest_trace_summary.as_ref(),
        "execution reached a terminal orchestration outcome".to_string(),
    );

    Ok(build_report(
        run_report.exit_status,
        latest_terminal_output,
        events,
        latest_trace_location,
        latest_session_status,
        latest_trace_summary,
    ))
}

fn build_report(
    exit_status: CommandExitStatus,
    terminal_output: String,
    events: Vec<OrchestrateEventEnvelope>,
    trace_location: Option<String>,
    session_status: Option<SessionStatusView>,
    trace_summary: Option<TraceSummaryView>,
) -> OrchestrateCommandReport {
    OrchestrateCommandReport {
        exit_status,
        terminal_output,
        trace_location,
        session_status,
        trace_summary,
        events,
    }
}

fn should_plan(view: Option<&SessionStatusView>) -> bool {
    let Some(view) = view else {
        return true;
    };
    view.goal_plan_state.is_none()
}

fn clarification_requested(view: &SessionStatusView) -> bool {
    view.clarification_prompt.is_some()
        || view.clarification_headline.is_some()
        || view.clarification_missing_fields.as_ref().is_some_and(|fields| !fields.is_empty())
}

fn clarification_question(view: &SessionStatusView) -> Option<String> {
    view.clarification_questions
        .as_ref()
        .and_then(|questions| {
            questions.iter().find_map(|question| {
                let trimmed = question.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
        })
        .or_else(|| {
            view.clarification_prompt.as_ref().and_then(|prompt| {
                let trimmed = prompt.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
        })
}

fn clarification_artifact(view: &SessionStatusView) -> Option<OrchestrateArtifactRef> {
    view.goal_brief_ref.clone().map(|artifact_ref| OrchestrateArtifactRef {
        artifact_kind: OrchestrateArtifactKind::Clarification,
        artifact_ref,
    })
}

/// Produces the stage-specific question text shown to the host for a planning
/// stage phase_request.
///
/// Each stage has a distinct question that reflects its purpose:
/// - **Discovery**: asks whether the discovered scope is complete
/// - **SystemShaping**: asks whether the domain model is correct
/// - **Requirements/Architecture/Backlog**: asks whether the brief is ready
fn planning_stage_question(stage_label: &str) -> String {
    match stage_label {
        "discovery" => {
            "Review the discovery output: is the identified scope complete and correct?".to_string()
        }
        "system-shaping" => {
            "Review the domain model and system boundaries: are they correct?".to_string()
        }
        "requirements" => {
            "Is the requirements planning brief ready to resume orchestration?".to_string()
        }
        "architecture" => {
            "Is the architecture planning brief ready to resume orchestration?".to_string()
        }
        _ => format!("Is the {stage_label} planning brief ready to resume orchestration?"),
    }
}

fn session_status_label(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Blocked => "blocked",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn resume_command(
    workspace: Option<&Path>,
    intent: OrchestrateIntent,
    request_id: Option<&str>,
) -> String {
    let mut parts = vec!["boundline orchestrate".to_string()];
    if let Some(workspace) = workspace {
        parts.push(format!("--workspace {}", workspace.display()));
    }
    parts.push(format!("--until {}", intent.as_cli_arg()));
    parts.push("--json-stream".to_string());
    if let Some(request_id) = request_id {
        parts.push(format!("--request-id {request_id}"));
    }
    parts.join(" ")
}

fn clarification_resume_command(
    workspace: Option<&Path>,
    intent: OrchestrateIntent,
    request_id: Option<&str>,
) -> String {
    format!("{} --answer \"<answer>\"", resume_command(workspace, intent, request_id))
}

fn planning_stage_resume_command(
    workspace: Option<&Path>,
    stage_key: &str,
    intent: OrchestrateIntent,
    request_id: Option<&str>,
) -> String {
    let mut parts = vec!["boundline orchestrate".to_string()];
    if let Some(workspace) = workspace {
        parts.push(format!("--workspace {}", workspace.display()));
    }
    parts.push(format!("--planning-stage-complete {}", stage_key));
    parts.push(format!("--until {}", intent.as_cli_arg()));
    parts.push("--json-stream".to_string());
    if let Some(request_id) = request_id {
        parts.push(format!("--request-id {request_id}"));
    }
    parts.join(" ")
}

fn planning_phase_requests(
    record: &ActiveSessionRecord,
    view: &SessionStatusView,
) -> Vec<PlanningStagePhaseRequest> {
    let Some(lifecycle) = record.governance_lifecycle.as_ref() else {
        return Vec::new();
    };

    let goal_text =
        record.goal_plan.as_ref().map(|plan| plan.goal_text.as_str()).unwrap_or_default();
    let active_flow = view.active_flow.as_deref();
    let workspace = Path::new(&view.workspace_ref);

    let mut seen_stage_keys = BTreeSet::new();
    planning_canon_mode_sequence(&lifecycle.selected_mode_sequence)
        .into_iter()
        .filter(|mode| should_emit_host_planning_stage(*mode, goal_text, active_flow, workspace))
        .filter_map(|mode| {
            let stage_key = planning_stage_key_for_mode(mode)?.to_string();
            if !seen_stage_keys.insert(stage_key.clone()) {
                return None;
            }
            Some(PlanningStagePhaseRequest {
                artifacts: planning_stage_artifacts(record, view, &stage_key),
                stage_label: mode.as_str().to_string(),
                stage_key,
            })
        })
        .collect()
}

/// Determines whether a Canon planning mode should be surfaced to the host
/// as a user-visible phase_request.
///
/// # Stage visibility rules
///
/// The previous implementation was a static filter that hard-coded
/// `Requirements | Architecture | Backlog`. The dynamic version considers:
///
/// | Mode | Visible when |
/// |------|-------------|
/// | **Discovery** | Greenfield delivery flow AND goal decomposition has significant gaps (no problem, no entities, no operations) |
/// | **Requirements** | Always visible for delivery flows; visible for change flows when goal is under-specified |
/// | **SystemShaping** | Greenfield delivery flow AND no existing domain model artifacts in workspace |
/// | **Architecture** | Always visible for delivery flows |
/// | **Backlog** | Always visible (terminal planning stage) |
///
/// # Safety contract
///
/// Hiding a stage means the orchestrator has HIGH CONFIDENCE that the goal text
/// or existing workspace state already covers what that stage would produce.
/// When in doubt, the stage is surfaced (safe default). A hidden stage still
/// executes in Canon's governance pipeline; it just doesn't require a host
/// phase_request confirmation before proceeding.
fn should_emit_host_planning_stage(
    mode: CanonMode,
    goal_text: &str,
    active_flow: Option<&str>,
    workspace: &Path,
) -> bool {
    match mode {
        // Backlog is always visible: it's the terminal planning stage that
        // produces the actionable task list.
        CanonMode::Backlog => true,

        // Requirements and Architecture are visible for delivery and change flows.
        CanonMode::Requirements | CanonMode::Architecture => {
            matches!(active_flow, Some("delivery") | Some("change") | None)
        }

        // Discovery is visible only when the goal decomposition reveals
        // significant gaps: the orchestrator cannot determine problem, entities,
        // or operations from the goal text alone, so the user needs to review
        // Canon's discovery output before proceeding.
        CanonMode::Discovery => {
            let is_delivery = matches!(active_flow, Some("delivery") | None);
            if !is_delivery {
                return false;
            }
            let decomposition = decompose_goal_text(goal_text);
            decomposition.problem.is_none()
                || (decomposition.entities.is_empty() && decomposition.operations.is_empty())
        }

        // SystemShaping is visible for greenfield delivery when no existing
        // domain model artifacts have been produced. Once Canon has a
        // Reusable system-shaping packet, this stage doesn't need host
        // confirmation.
        CanonMode::SystemShaping => {
            let is_delivery = matches!(active_flow, Some("delivery") | None);
            if !is_delivery {
                return false;
            }
            // Check if workspace already has system-shaping output
            let has_existing_shaping =
                workspace.join(".boundline/governance/planning/system-shaping").is_dir();
            !has_existing_shaping
        }

        // Other modes (e.g. Implementation, Verification) are never surfaced
        // as host-visible planning stages.
        _ => false,
    }
}

fn planning_stage_artifacts(
    record: &ActiveSessionRecord,
    view: &SessionStatusView,
    stage_key: &str,
) -> Vec<OrchestrateArtifactRef> {
    let mut artifacts = Vec::new();

    if let Some(memory) = planning_stage_canon_memory(record, stage_key) {
        if let Some(packet_ref) = memory.packet_ref.clone() {
            artifacts.push(OrchestrateArtifactRef {
                artifact_kind: OrchestrateArtifactKind::CanonPacket,
                artifact_ref: packet_ref,
            });
        }

        for artifact_ref in &memory.artifact_refs {
            if canon_memory_artifact_exists(view, artifact_ref) {
                artifacts.push(OrchestrateArtifactRef {
                    artifact_kind: canon_memory_artifact_kind(stage_key, artifact_ref),
                    artifact_ref: artifact_ref.clone(),
                });
            }
        }

        if !artifacts.is_empty() {
            return artifacts;
        }
    }

    if let Some(stage_brief_ref) = planning_stage_brief_ref(stage_key)
        && planning_stage_brief_exists(view, &stage_brief_ref)
    {
        artifacts.push(OrchestrateArtifactRef {
            artifact_kind: OrchestrateArtifactKind::PlanningStageBrief,
            artifact_ref: stage_brief_ref,
        });

        let canon_kind = match stage_key {
            "plan:requirements" => Some(OrchestrateArtifactKind::RequirementsDoc),
            "plan:architecture" => Some(OrchestrateArtifactKind::ArchitectureDoc),
            "plan:backlog" => Some(OrchestrateArtifactKind::BacklogDoc),
            "plan:system-shaping" => Some(OrchestrateArtifactKind::DomainModelDoc),
            _ => None,
        };

        if let Some(kind) = canon_kind
            && let Some(mode) = planning_canon_mode_for_stage_key(stage_key)
        {
            let packet_ref = format!(".boundline/governance/planning/{}", mode.as_str());
            let expected_refs = mode.expected_document_refs(&packet_ref);
            for artifact_ref in expected_refs {
                if planning_stage_brief_exists(view, &artifact_ref) {
                    artifacts.push(OrchestrateArtifactRef { artifact_kind: kind, artifact_ref });
                }
            }
        }
    }

    if artifacts.is_empty()
        && let Some(artifact_ref) = view.session_plan_brief_ref.clone()
    {
        artifacts.push(OrchestrateArtifactRef {
            artifact_kind: OrchestrateArtifactKind::PlanBrief,
            artifact_ref,
        });
    }

    artifacts
}

fn planning_stage_brief_exists(view: &SessionStatusView, artifact_ref: &str) -> bool {
    Path::new(&view.workspace_ref).join(artifact_ref).is_file()
}

fn canon_memory_artifact_exists(view: &SessionStatusView, artifact_ref: &str) -> bool {
    Path::new(&view.workspace_ref).join(artifact_ref).exists()
}

fn canon_memory_artifact_kind(stage_key: &str, artifact_ref: &str) -> OrchestrateArtifactKind {
    let file_name =
        Path::new(artifact_ref).file_name().and_then(|name| name.to_str()).unwrap_or_default();

    match (stage_key, file_name) {
        ("plan:requirements", "prd.md") => OrchestrateArtifactKind::PrdDoc,
        ("plan:requirements", _) => OrchestrateArtifactKind::RequirementsDoc,
        ("plan:architecture", "domain-model.md") => OrchestrateArtifactKind::DomainModelDoc,
        ("plan:architecture", _) => OrchestrateArtifactKind::ArchitectureDoc,
        ("plan:backlog", _) => OrchestrateArtifactKind::BacklogDoc,
        _ => OrchestrateArtifactKind::CanonPacket,
    }
}

fn planning_stage_canon_memory(
    record: &ActiveSessionRecord,
    stage_key: &str,
) -> Option<CompactedCanonMemory> {
    if let Some(memory) = record
        .active_task
        .as_ref()
        .and_then(task_state_compacted_canon_memory)
        .filter(|memory| memory.stage_key.as_deref() == Some(stage_key))
    {
        return Some(memory);
    }

    record
        .goal_plan
        .as_ref()
        .and_then(|goal_plan| goal_plan.compacted_canon_memory.clone())
        .filter(|memory| memory.stage_key.as_deref() == Some(stage_key))
}

fn planning_stage_cursor(record: &ActiveSessionRecord) -> usize {
    record.governance_lifecycle.as_ref().map(|lifecycle| lifecycle.current_stage_index).unwrap_or(0)
}

fn next_pending_planning_phase_request(
    record: &ActiveSessionRecord,
    view: &SessionStatusView,
) -> Option<PlanningStagePhaseRequest> {
    let requests = planning_phase_requests(record, view);
    requests.get(planning_stage_cursor(record)).cloned()
}

fn planning_stage_resume_intent(
    record: &ActiveSessionRecord,
    request_count: usize,
) -> OrchestrateIntent {
    if planning_stage_cursor(record) + 1 < request_count {
        OrchestrateIntent::ContinueUntilPhaseRequest
    } else {
        OrchestrateIntent::ContinueUntilTerminal
    }
}

fn validate_planning_stage_completion_request_id(
    record: &ActiveSessionRecord,
    view: &SessionStatusView,
    completed_stage_key: &str,
    request_id: &str,
) -> Result<(), OrchestrateCommandError> {
    let Some(request) = next_pending_planning_phase_request(record, view) else {
        return Err(SessionCommandError::InvalidRequest(
            PLANNING_STAGE_REQUEST_ID_REQUIRES_ACTIVE_PHASE_REQUEST.to_string(),
        )
        .into());
    };

    if request.stage_key != completed_stage_key {
        return Err(SessionCommandError::InvalidRequest(format!(
            "planning stage completion expected `{}` but received `{completed_stage_key}`",
            request.stage_key
        ))
        .into());
    }

    let question = planning_stage_question(&request.stage_label);
    let expected_request_id = phase_request_id(
        &view.session_id,
        PHASE_KIND_PLANNING,
        completed_stage_key,
        Some(question.as_str()),
    );

    if expected_request_id != request_id {
        return Err(SessionCommandError::InvalidRequest(format!(
            "planning stage completion expected request_id `{expected_request_id}` for `{completed_stage_key}` but received `{request_id}`"
        ))
        .into());
    }

    Ok(())
}

fn validate_goal_clarification_request_id(
    view: &SessionStatusView,
    request_id: &str,
) -> Result<(), OrchestrateCommandError> {
    let Some(question) = clarification_question(view) else {
        return Err(SessionCommandError::InvalidRequest(
            GOAL_CLARIFICATION_ANSWER_REQUIRES_ACTIVE_PHASE_REQUEST.to_string(),
        )
        .into());
    };

    let expected_request_id = phase_request_id(
        &view.session_id,
        PHASE_KIND_GOAL,
        STAGE_KEY_GOAL,
        Some(question.as_str()),
    );

    if expected_request_id != request_id {
        return Err(SessionCommandError::InvalidRequest(format!(
            "goal clarification answer expected request_id `{expected_request_id}` but received `{request_id}`"
        ))
        .into());
    }

    Ok(())
}

fn load_active_session_record(
    workspace: &Path,
) -> Result<Option<ActiveSessionRecord>, OrchestrateCommandError> {
    FileSessionStore::for_workspace(workspace)
        .load()
        .map_err(SessionCommandError::from)
        .map_err(OrchestrateCommandError::from)
}

fn complete_planning_stage(
    workspace: &Path,
    completed_stage_key: &str,
) -> Result<ActiveSessionRecord, OrchestrateCommandError> {
    let store = FileSessionStore::for_workspace(workspace);
    let mut record = store
        .load()
        .map_err(SessionCommandError::from)?
        .ok_or(SessionCommandError::MissingActiveSession)?;

    let Some(lifecycle) = record.governance_lifecycle.as_mut() else {
        return Err(SessionCommandError::InvalidRequest(
            "active session has no governed planning stage handoff to complete".to_string(),
        )
        .into());
    };

    let goal_text =
        record.goal_plan.as_ref().map(|plan| plan.goal_text.as_str()).unwrap_or_default();
    let active_flow = record
        .goal_plan
        .as_ref()
        .and_then(|plan| plan.flow.as_ref())
        .map(|flow| flow.flow_name.as_str());

    let stage_sequence = planning_canon_mode_sequence(&lifecycle.selected_mode_sequence)
        .into_iter()
        .filter(|mode| should_emit_host_planning_stage(*mode, goal_text, active_flow, workspace))
        .filter_map(planning_stage_key_for_mode)
        .collect::<Vec<_>>();

    let current_index = lifecycle.current_stage_index;
    let Some(expected_stage_key) = stage_sequence.get(current_index).copied() else {
        return Err(SessionCommandError::InvalidRequest(
            "active session has no pending planning stage handoff to complete".to_string(),
        )
        .into());
    };

    if expected_stage_key != completed_stage_key {
        return Err(SessionCommandError::InvalidRequest(format!(
            "planning stage completion expected `{expected_stage_key}` but received `{completed_stage_key}`"
        ))
        .into());
    }

    if let Some(stage_record) =
        lifecycle.stage_records.iter_mut().find(|record| record.stage_key == completed_stage_key)
    {
        stage_record.lifecycle_state = GovernanceLifecycleState::Completed;
        stage_record.approval_state = ApprovalState::NotNeeded;
        stage_record.blocked_reason = None;
    }

    lifecycle.current_stage_index = current_index + 1;
    lifecycle.terminal_reason = None;
    record.updated_at = current_timestamp_millis();
    store.persist(&record).map_err(SessionCommandError::from)?;
    Ok(record)
}

fn push_status_update_events(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    view: Option<&SessionStatusView>,
) {
    let Some(view) = view else {
        return;
    };

    push_event(
        events,
        event_counter,
        EVENT_KIND_EXECUTION_UPDATE,
        Some(view),
        None,
        None,
        format!("session status is now {}", session_status_label(view.latest_status)),
        None,
        None,
        None,
        Some(view.clone()),
        None,
    );

    if view.latest_governance_stage.is_some()
        || view.latest_governance_runtime.is_some()
        || view.latest_governance_mode.is_some()
    {
        let message = match (
            view.latest_governance_stage.as_deref(),
            view.latest_governance_runtime.as_deref(),
            view.latest_governance_mode.as_deref(),
        ) {
            (Some(stage), Some(runtime), Some(mode)) => {
                format!("governance is active for {stage} via {runtime} ({mode})")
            }
            (Some(stage), Some(runtime), None) => {
                format!("governance is active for {stage} via {runtime}")
            }
            _ => "governance state updated".to_string(),
        };
        push_event(
            events,
            event_counter,
            EVENT_KIND_GOVERNANCE_UPDATE,
            Some(view),
            None,
            view.latest_governance_stage.clone(),
            message,
            None,
            None,
            None,
            Some(view.clone()),
            None,
        );
    }
}

struct PhaseRequestPayload<'a> {
    phase_kind: &'a str,
    stage_key: &'a str,
    artifact: Option<OrchestrateArtifactRef>,
    message: String,
    phase_request_kind: Option<&'a str>,
    question: Option<String>,
    expected_answer: Option<OrchestratePhaseRequestExpectedAnswer>,
    instruction: Option<String>,
    resume_command: Option<String>,
}

fn phase_request_id(
    session_id: &str,
    phase_kind: &str,
    stage_key: &str,
    question: Option<&str>,
) -> String {
    let stage_key_fragment = stage_key.replace(':', "-");
    if let Some(question_fragment) = phase_request_question_fragment(question) {
        return format!(
            "{PHASE_REQUEST_ID_PREFIX}-{session_id}-{phase_kind}-{stage_key_fragment}-{question_fragment}"
        );
    }

    format!("{PHASE_REQUEST_ID_PREFIX}-{session_id}-{phase_kind}-{stage_key_fragment}")
}

fn phase_request_question_fragment(question: Option<&str>) -> Option<String> {
    let question = question?.trim();
    if question.is_empty() {
        return None;
    }

    let mut fragment = String::new();
    let mut last_was_separator = false;
    for character in question.chars() {
        if character.is_ascii_alphanumeric() {
            fragment.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !fragment.is_empty() && !last_was_separator {
            fragment.push('-');
            last_was_separator = true;
        }

        if fragment.len() >= PHASE_REQUEST_QUESTION_FRAGMENT_MAX_CHARS {
            break;
        }
    }

    while fragment.ends_with('-') {
        fragment.pop();
    }

    (!fragment.is_empty()).then_some(fragment)
}

fn push_phase_request(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    view: &SessionStatusView,
    payload: PhaseRequestPayload<'_>,
    assistant_host: Option<AssistantHost>,
) {
    let PhaseRequestPayload {
        phase_kind,
        stage_key,
        artifact,
        message,
        phase_request_kind,
        question,
        expected_answer,
        instruction,
        resume_command,
    } = payload;
    *event_counter += 1;
    let resume_command = resume_command.clone();
    let next_command = if resume_command.is_some() { None } else { view.next_command.clone() };
    let metadata = event_metadata(Some(view), None, Some(phase_kind));
    let resolved_request_id = phase_request_kind.and_then(|_| {
        question.as_deref().map(|question| {
            phase_request_id(&view.session_id, phase_kind, stage_key, Some(question))
        })
    });
    let phase_request =
        phase_request_kind.zip(question).map(|(kind, question)| OrchestratePhaseRequest {
            request_id: resolved_request_id.clone().unwrap_or_default(),
            kind: kind.to_string(),
            phase: phase_kind.to_string(),
            reason: message.clone(),
            question,
            expected_answer,
        });

    events.push(OrchestrateEventEnvelope {
        event_id: format!("orchestrate-event-{}", *event_counter),
        timestamp_ms: current_timestamp_millis(),
        event_kind: EVENT_KIND_PHASE_REQUEST.to_string(),
        actor_kind: metadata.actor_kind,
        actor_name: metadata.actor_name,
        runtime_kind: metadata.runtime_kind,
        provider: metadata.provider,
        route_slot: metadata.route_slot,
        model_name: metadata.model_name,
        decision_family: metadata.decision_family,
        review_step: metadata.review_step,
        vote_summary: metadata.vote_summary,
        adjudication_summary: metadata.adjudication_summary,
        governance_mode: metadata.governance_mode,
        session_ref: Some(view.session_id.clone()),
        phase_kind: Some(phase_kind.to_string()),
        stage_key: Some(stage_key.to_string()),
        message,
        artifact,
        phase_request,
        instruction,
        resume_command: resume_command.clone(),
        assistant_resume_command: assistant_host.and_then(|host| {
            assistant_phase_command(host, phase_kind, stage_key)
                .filter(|_| resume_command.is_some())
        }),
        next_command: next_command.clone(),
        assistant_next_command: assistant_host
            .and_then(|host| assistant_command_for_cli(host, next_command.as_deref())),
        session_status: Some(view.clone()),
        trace_summary: None,
    });
}

fn push_planning_stage_phase_request(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    view: &SessionStatusView,
    record: &ActiveSessionRecord,
    request: PlanningStagePhaseRequest,
    workspace_hint: Option<&Path>,
    assistant_host: Option<AssistantHost>,
) {
    let request_count = planning_phase_requests(record, view).len();
    let resume_intent = planning_stage_resume_intent(record, request_count);
    let stage_key = request.stage_key;
    let stage_label = request.stage_label;
    let question = planning_stage_question(&stage_label);
    let request_id = phase_request_id(
        &view.session_id,
        PHASE_KIND_PLANNING,
        &stage_key,
        Some(question.as_str()),
    );
    let resume = planning_stage_resume_command(
        workspace_hint,
        &stage_key,
        resume_intent,
        Some(request_id.as_str()),
    );
    let canon_memory = planning_stage_canon_memory(record, &stage_key);
    let is_incomplete = canon_memory
        .as_ref()
        .is_some_and(|memory| memory.credibility != MemoryCredibilityState::Credible);
    let message = canon_memory.as_ref().map_or_else(
        || format!("author or review the {stage_label} planning brief before execution continues"),
        |memory| {
            let packet_ref = memory.packet_ref.as_deref().unwrap_or("the governed packet");
            format!(
                "{stage_label} planning is blocked on Canon packet {packet_ref}: {}",
                memory.summary_text()
            )
        },
    );
    let instruction = if is_incomplete {
        let artifact_hint = canon_memory
            .as_ref()
            .and_then(|m| m.artifact_refs.first().or(m.packet_ref.as_ref()))
            .cloned()
            .unwrap_or_else(|| format!("the {stage_label} planning brief"));
        let markers = record.goal_plan.as_ref().map_or_else(Vec::new, |gp| {
            let has_authored =
                view.authored_input_sources.as_ref().is_some_and(|sources| !sources.is_empty());
            planning_unknown_markers(
                &gp.goal_text,
                gp.verification_strategy.as_deref(),
                has_authored,
            )
        });
        let mut text = String::new();
        if !markers.is_empty()
            && !markers.first().is_some_and(|m| m.starts_with("no explicit unknown"))
        {
            text.push_str(
                "The following context gaps were detected in your goal and produced placeholder sections in the packet:\n",
            );
            for (i, marker) in markers.iter().enumerate() {
                text.push_str(&format!("{}. {}\n", i + 1, marker));
            }
            text.push('\n');
        }
        text.push_str(&format!(
            "Author the placeholder sections in {artifact_hint} using the goal brief and project context. \
             If the user provides a file or folder path, use its content as primary source material. \
             Once the sections are filled, resume orchestration."
        ));
        Some(text)
    } else {
        canon_memory.as_ref().map_or_else(
            || Some(format!(
                "complete the {stage_label} planning brief using the bounded context, then resume orchestration"
            )),
            |memory| {
                let next_action = memory
                    .next_action_text()
                    .unwrap_or_else(|| "inspect the governed packet and confirm readiness".to_string());
                Some(format!(
                    "the {stage_label} stage is ready; Canon next action: {next_action}"
                ))
            },
        )
    };
    let (phase_request_kind, expected_answer) = if is_incomplete {
        (
            PHASE_REQUEST_KIND_CLARIFICATION,
            OrchestratePhaseRequestExpectedAnswer::for_planning_stage_question(
                canon_memory.as_ref(),
                &stage_key,
            ),
        )
    } else {
        (PHASE_REQUEST_KIND_REVIEW, OrchestratePhaseRequestExpectedAnswer::confirmation())
    };
    push_phase_request(
        events,
        event_counter,
        view,
        PhaseRequestPayload {
            phase_kind: PHASE_KIND_PLANNING,
            stage_key: &stage_key,
            artifact: request.artifacts.first().cloned(),
            message,
            phase_request_kind: Some(phase_request_kind),
            question: Some(question),
            expected_answer: Some(expected_answer),
            instruction,
            resume_command: Some(resume),
        },
        assistant_host,
    );
}

fn assistant_phase_command(
    _assistant_host: AssistantHost,
    phase_kind: &str,
    _stage_key: &str,
) -> Option<String> {
    match phase_kind {
        PHASE_KIND_GOAL => Some("/boundline-goal".to_string()),
        PHASE_KIND_PLANNING => Some("/boundline-plan".to_string()),
        PHASE_KIND_EXECUTION => Some("/boundline-run".to_string()),
        _ => None,
    }
}

fn assistant_command_for_cli(
    _assistant_host: AssistantHost,
    cli_command: Option<&str>,
) -> Option<String> {
    let cli_command = cli_command?.trim();
    if cli_command.starts_with("boundline goal") {
        Some("/boundline-goal".to_string())
    } else if cli_command.starts_with("boundline plan") {
        Some("/boundline-plan".to_string())
    } else if cli_command.starts_with("boundline run") {
        Some("/boundline-run".to_string())
    } else if cli_command.starts_with("boundline step") {
        Some("/boundline-step".to_string())
    } else if cli_command.starts_with("boundline next") {
        Some("/boundline-next".to_string())
    } else if cli_command.starts_with("boundline status") {
        Some("/boundline-status".to_string())
    } else if cli_command.starts_with("boundline inspect") {
        Some("/boundline-inspect".to_string())
    } else if cli_command.starts_with("boundline govern") {
        Some("/boundline-govern".to_string())
    } else if cli_command.starts_with("boundline init") {
        Some("/boundline-init".to_string())
    } else {
        None
    }
}

fn push_planning_stage_completion_event(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    view: &SessionStatusView,
    stage_key: &str,
) {
    push_event(
        events,
        event_counter,
        EVENT_KIND_EXECUTION_UPDATE,
        Some(view),
        Some(PHASE_KIND_PLANNING.to_string()),
        Some(stage_key.to_string()),
        format!("recorded host completion for planning stage {stage_key}"),
        None,
        None,
        None,
        Some(view.clone()),
        None,
    );
}

fn push_terminal_event(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    session_status: Option<&SessionStatusView>,
    trace_summary: Option<&TraceSummaryView>,
    message: String,
) {
    push_event(
        events,
        event_counter,
        EVENT_KIND_TERMINAL,
        session_status,
        Some(PHASE_KIND_EXECUTION.to_string()),
        Some(STAGE_KEY_RUN.to_string()),
        message,
        None,
        None,
        None,
        session_status.cloned(),
        trace_summary.cloned(),
    );
}

fn push_artifact_events(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    seen_artifacts: &mut BTreeSet<String>,
    session_status: Option<&SessionStatusView>,
    trace_summary: Option<&TraceSummaryView>,
) {
    if let Some(view) = session_status {
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            Some(view),
            OrchestrateArtifactKind::GoalBrief,
            view.goal_brief_ref.clone(),
            "recorded the session goal brief".to_string(),
        );
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            Some(view),
            OrchestrateArtifactKind::PlanBrief,
            view.session_plan_brief_ref.clone(),
            "recorded the session plan brief".to_string(),
        );
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            Some(view),
            OrchestrateArtifactKind::RunBrief,
            view.run_brief_ref.clone(),
            "recorded the session run brief".to_string(),
        );
    }

    if let Some(summary) = trace_summary {
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            session_status,
            OrchestrateArtifactKind::GoalBrief,
            summary.goal_brief_ref.clone(),
            "recorded the session goal brief".to_string(),
        );
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            session_status,
            OrchestrateArtifactKind::PlanBrief,
            summary.session_plan_brief_ref.clone(),
            "recorded the session plan brief".to_string(),
        );
        push_artifact_event(
            events,
            event_counter,
            seen_artifacts,
            session_status,
            OrchestrateArtifactKind::RunBrief,
            summary.run_brief_ref.clone(),
            "recorded the session run brief".to_string(),
        );
    }
}

fn push_planning_stage_artifact_events(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    seen_artifacts: &mut BTreeSet<String>,
    active_record: Option<&ActiveSessionRecord>,
    session_status: Option<&SessionStatusView>,
) {
    let (Some(record), Some(view)) = (active_record, session_status) else {
        return;
    };

    for request in planning_phase_requests(record, view) {
        for artifact in request.artifacts {
            push_artifact_event(
                events,
                event_counter,
                seen_artifacts,
                Some(view),
                artifact.artifact_kind,
                Some(artifact.artifact_ref),
                format!("recorded the {} planning brief", request.stage_label),
            );
        }
    }
}

fn push_artifact_event(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    seen_artifacts: &mut BTreeSet<String>,
    session_status: Option<&SessionStatusView>,
    artifact_kind: OrchestrateArtifactKind,
    artifact_ref: Option<String>,
    message: String,
) {
    let Some(artifact_ref) = artifact_ref else {
        return;
    };
    if !seen_artifacts.insert(artifact_ref.clone()) {
        return;
    }

    push_event(
        events,
        event_counter,
        EVENT_KIND_ARTIFACT_RECORDED,
        session_status,
        None,
        None,
        message,
        Some(OrchestrateArtifactRef { artifact_kind, artifact_ref }),
        None,
        None,
        session_status.cloned(),
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn push_event(
    events: &mut Vec<OrchestrateEventEnvelope>,
    event_counter: &mut usize,
    event_kind: &str,
    session_status: Option<&SessionStatusView>,
    phase_kind: Option<String>,
    stage_key: Option<String>,
    message: String,
    artifact: Option<OrchestrateArtifactRef>,
    instruction: Option<String>,
    resume_command: Option<String>,
    session_status_snapshot: Option<SessionStatusView>,
    trace_summary: Option<TraceSummaryView>,
) {
    *event_counter += 1;
    let next_command = session_status_snapshot.as_ref().and_then(|view| view.next_command.clone());
    let metadata = event_metadata(
        session_status_snapshot.as_ref(),
        trace_summary.as_ref(),
        phase_kind.as_deref(),
    );

    events.push(OrchestrateEventEnvelope {
        event_id: format!("orchestrate-event-{}", *event_counter),
        timestamp_ms: current_timestamp_millis(),
        event_kind: event_kind.to_string(),
        actor_kind: metadata.actor_kind,
        actor_name: metadata.actor_name,
        runtime_kind: metadata.runtime_kind,
        provider: metadata.provider,
        route_slot: metadata.route_slot,
        model_name: metadata.model_name,
        decision_family: metadata.decision_family,
        review_step: metadata.review_step,
        vote_summary: metadata.vote_summary,
        adjudication_summary: metadata.adjudication_summary,
        governance_mode: metadata.governance_mode,
        session_ref: session_status.map(|view| view.session_id.clone()),
        phase_kind,
        stage_key,
        message,
        artifact,
        phase_request: None,
        instruction,
        resume_command,
        assistant_resume_command: None,
        next_command,
        assistant_next_command: None,
        session_status: session_status_snapshot,
        trace_summary,
    });
}

fn event_metadata(
    session_status: Option<&SessionStatusView>,
    trace_summary: Option<&TraceSummaryView>,
    phase_kind: Option<&str>,
) -> OrchestrateEventMetadata {
    let route_slot = infer_event_route_slot(phase_kind, session_status).map(str::to_string);
    let routed_selection = route_slot.as_deref().and_then(|slot| {
        trace_summary.and_then(|summary| routing_projection_selection(summary, slot))
    });

    let runtime_kind = routed_selection
        .as_ref()
        .map(|selection| selection.runtime_kind.clone())
        .or_else(|| session_status.and_then(|view| view.latest_governance_runtime.clone()));
    let provider = route_slot
        .as_deref()
        .and_then(|slot| {
            trace_summary.and_then(|summary| routing_projection_provider(summary, slot))
        })
        .or_else(|| runtime_kind.clone());
    let model_name = routed_selection.map(|selection| selection.model_name);

    OrchestrateEventMetadata {
        actor_kind: None,
        actor_name: None,
        runtime_kind,
        provider,
        route_slot,
        model_name,
        decision_family: session_status.and_then(|view| view.latest_candidate_family.clone()),
        review_step: session_status.and_then(|view| {
            view.latest_review_headline
                .clone()
                .or_else(|| view.latest_review_selection_summary.clone())
                .or_else(|| view.latest_review_trigger.clone())
        }),
        vote_summary: session_status.and_then(|view| {
            view.latest_voting_result.clone().or_else(|| view.latest_review_vote.clone())
        }),
        adjudication_summary: session_status.and_then(|view| {
            view.latest_voting_adjudication.clone().or_else(|| view.latest_review_outcome.clone())
        }),
        governance_mode: session_status.and_then(|view| view.latest_governance_mode.clone()),
    }
}

fn infer_event_route_slot(
    phase_kind: Option<&str>,
    session_status: Option<&SessionStatusView>,
) -> Option<&'static str> {
    if phase_kind.is_some_and(|kind| kind == PHASE_KIND_GOAL || kind == PHASE_KIND_PLANNING) {
        return Some(ROUTE_SLOT_PLANNING);
    }

    if session_status.is_some_and(|view| view.latest_voting_adjudication.is_some()) {
        return Some(ROUTE_SLOT_ADJUDICATION);
    }

    if session_status.is_some_and(|view| {
        view.latest_review_trigger.is_some()
            || view.latest_review_vote.is_some()
            || view.latest_review_outcome.is_some()
            || view.latest_voting_result.is_some()
    }) {
        return Some(ROUTE_SLOT_REVIEW);
    }

    phase_kind.is_some_and(|kind| kind == PHASE_KIND_EXECUTION).then_some(ROUTE_SLOT_IMPLEMENTATION)
}

fn routing_projection_selection(
    trace_summary: &TraceSummaryView,
    route_slot: &str,
) -> Option<RoutingProjectionSelection> {
    trace_summary.routing_projection.effective_routing.iter().find_map(|entry| {
        parse_routing_projection_entry(entry).and_then(|(slot, runtime_kind, model_name)| {
            (slot == route_slot).then_some(RoutingProjectionSelection { runtime_kind, model_name })
        })
    })
}

fn routing_projection_provider(
    trace_summary: &TraceSummaryView,
    route_slot: &str,
) -> Option<String> {
    trace_summary.routing_projection.assistant_bindings.iter().find_map(|entry| {
        parse_slot_binding_entry(entry)
            .and_then(|(slot, provider)| (slot == route_slot).then_some(provider))
    })
}

fn parse_routing_projection_entry(entry: &str) -> Option<(String, String, String)> {
    let (slot, route) = entry.split_once(ROUTING_PROJECTION_SLOT_SEPARATOR)?;
    let route = route
        .split_once(ROUTING_PROJECTION_SOURCE_SEPARATOR)
        .map(|(path, _)| path)
        .unwrap_or(route);
    let (runtime_kind, model_name) = route.split_once(ROUTING_PROJECTION_MODEL_SEPARATOR)?;
    Some((slot.to_string(), runtime_kind.to_string(), model_name.to_string()))
}

fn parse_slot_binding_entry(entry: &str) -> Option<(String, String)> {
    let (slot, binding) = entry.split_once(ROUTING_PROJECTION_SLOT_SEPARATOR)?;
    Some((slot.to_string(), binding.to_string()))
}
