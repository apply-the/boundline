//! Session-native CLI command handlers and status/report projection helpers.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::domain::brief::{
    AuthoredBriefBundle, BriefIngestionError, GovernanceIntent, normalize_governance_intent,
    normalize_inputs_with_governance,
};
use serde_json::Value;
use thiserror::Error;

use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::cli::CommandExitStatus;
use crate::cli::inspect::summarize_trace;
use crate::cli::output;
use crate::cli::workspace as cli_workspace;
use crate::domain::cluster::{ClusterDeliveryStory, ClusterSessionProjection};
use crate::domain::decision::ActionSelector;
use crate::domain::execution::StageRoutingDecisionRecord;
use crate::domain::goal_plan::PlanQualityState;
use crate::domain::governance::{
    BacklogQualityAssessment, CanonModeSelectionPreference, GovernanceLifecycleState,
    GovernanceRuntimeKind, GovernedSessionLifecycle, backlog_quality_snapshot_for_lifecycle,
    governance_confidence_handoff, is_planning_stage_key, planning_canon_mode_for_stage_key,
};
use crate::domain::guidance::GuidanceGuardianProjection;
use crate::domain::negotiation::NegotiatedDeliveryPacket;
use crate::domain::reasoning::ReasoningAdmissionEffect;
use crate::domain::session::{
    ActiveSessionRecord, CompatibilityFollowUpMode, CompatibilityFollowUpView, ContinuityAuthority,
    FrameworkAdapterStageFailureDetails, SessionStatus, SessionStatusView, date_prefix_from_millis,
    decision_status_text, delegation_next_command, delegation_status_view, execution_path_text,
    generate_session_ref, routing_outcome, session_branch_ref, session_goal_brief_ref,
    session_plan_brief_ref, session_run_brief_ref, session_storage_root_ref,
    task_state_attempt_lineage_summary, task_state_canon_memory_context_credibility,
    task_state_canon_memory_context_summary, task_state_canon_memory_primary_inputs,
    task_state_canon_memory_provenance, task_state_canon_memory_staleness_reason,
    task_state_governance_approval_provenance, task_state_governance_approval_text,
    task_state_governance_blocked_reason, task_state_governance_candidate_actions,
    task_state_governance_canon_run_ref, task_state_governance_contract_lines,
    task_state_governance_decision_headline, task_state_governance_mode_text,
    task_state_governance_next_action, task_state_governance_packet_binding_reason,
    task_state_governance_packet_ref, task_state_governance_packet_source_stage,
    task_state_governance_reason, task_state_governance_rollout_profile_text,
    task_state_governance_runtime_state_text, task_state_governance_runtime_text,
    task_state_governance_stage_key, task_state_governance_state_text, task_state_string,
    task_state_strings, task_state_workspace_slice_summary,
};
use crate::domain::task::{ClarificationReasonKind, TaskStatus};
use crate::domain::trace::{TraceSummaryView, current_timestamp_millis};
use crate::fixture::FixtureRuntimeError;
use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};

/// Result returned by session-native CLI commands.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
    pub session_status: Option<SessionStatusView>,
    pub guidance_guardian: Option<GuidanceGuardianProjection>,
    pub trace_summary: Option<TraceSummaryView>,
}

const SESSION_BRIEF_ARTIFACT_NOTE: &str =
    "This runtime-owned artifact preserves Boundline's compact operator brief fields.";
const SESSION_SOURCE_OF_TRUTH: &str =
    ".boundline/active-session -> .boundline/sessions/<session_ref>/session.json";
const SESSION_HISTORY_RESUME_HINT: &str = "boundline session resume <session_id>";
const SESSION_HISTORY_CREATE_HINT: &str = "boundline goal --goal <goal>";
const GIT_DIRECTORY_ENTRY_NAME: &str = ".git";
const GIT_PROGRAM: &str = "git";
const GIT_SHOW_REF_SUBCOMMAND: &str = "show-ref";
const GIT_SWITCH_SUBCOMMAND: &str = "switch";
const GIT_CREATE_BRANCH_FLAG: &str = "-c";
const GIT_VERIFY_FLAG: &str = "--verify";
const GIT_QUIET_FLAG: &str = "--quiet";
const GIT_LOCAL_BRANCH_REF_PREFIX: &str = "refs/heads";
const GIT_INDEX_LOCK_MARKER: &str = ".git/index.lock";
const GIT_FILE_EXISTS_MARKER: &str = "File exists";
const GIT_BRANCH_ACTIVATION_RETRY_ATTEMPTS: u8 = 40;
const GIT_BRANCH_ACTIVATION_RETRY_DELAY_MILLIS: u64 = 50;
const DIRECT_RUN_BOUNDED_CONTEXT_HEADLINE: &str = "bounded context required before planning";
const DIRECT_RUN_BOUNDED_CONTEXT_REPAIR: &str =
    "provide a credible brief or concrete workspace target before retrying direct run";
const RUN_PLAN_QUALITY_NOT_READY_EXPLANATION: &str =
    "run returned without resuming because the current goal plan is not ready for execution";
const RUN_BACKLOG_QUALITY_NOT_READY_EXPLANATION: &str =
    "run returned without resuming because the governed backlog packet is not ready for execution";
const RUN_PLANNING_ANALYSIS_BLOCKED_EXPLANATION: &str =
    "run returned without resuming because planning analysis found a blocking execution gap";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionBriefArtifactKind {
    Goal,
    Plan,
    Run,
}

impl SessionBriefArtifactKind {
    const fn title(self) -> &'static str {
        match self {
            Self::Goal => "Goal Brief",
            Self::Plan => "Plan Brief",
            Self::Run => "Run Brief",
        }
    }

    fn brief_ref(self, session_id: &str) -> String {
        match self {
            Self::Goal => session_goal_brief_ref(session_id),
            Self::Plan => session_plan_brief_ref(session_id),
            Self::Run => session_run_brief_ref(session_id),
        }
    }
}

fn report_with_session_status(
    exit_status: CommandExitStatus,
    view: SessionStatusView,
) -> SessionCommandReport {
    report_with_session_guidance(exit_status, view, None)
}

fn report_with_session_guidance(
    exit_status: CommandExitStatus,
    view: SessionStatusView,
    guidance_guardian: Option<&GuidanceGuardianProjection>,
) -> SessionCommandReport {
    let trace_location = view.latest_trace_ref.clone();
    let mut terminal_output = output::render_session_status(&view);
    let guidance_lines =
        guidance_guardian.map(output::render_guidance_projection_lines).unwrap_or_default();
    if !guidance_lines.is_empty() {
        terminal_output.push('\n');
        terminal_output.push_str(&guidance_lines.join("\n"));
    }
    SessionCommandReport {
        exit_status,
        terminal_output,
        trace_location,
        session_status: Some(view),
        guidance_guardian: guidance_guardian.cloned(),
        trace_summary: None,
    }
}

fn report_with_trace_summary(
    exit_status: CommandExitStatus,
    terminal_output: String,
    trace_location: Option<String>,
    trace_summary: Option<TraceSummaryView>,
) -> SessionCommandReport {
    SessionCommandReport {
        exit_status,
        terminal_output,
        trace_location,
        session_status: None,
        guidance_guardian: None,
        trace_summary,
    }
}

fn report_with_text(
    exit_status: CommandExitStatus,
    terminal_output: String,
) -> SessionCommandReport {
    SessionCommandReport {
        exit_status,
        terminal_output,
        trace_location: None,
        session_status: None,
        guidance_guardian: None,
        trace_summary: None,
    }
}

fn cluster_delivery_story_for_record(record: &ActiveSessionRecord) -> Option<ClusterDeliveryStory> {
    record
        .goal_plan
        .as_ref()
        .and_then(|goal_plan| goal_plan.cluster_delivery_story.clone())
        .or_else(|| {
            record
                .active_task
                .as_ref()
                .and_then(|task| task.context.cluster_delivery_story().ok().flatten())
        })
}

fn summarize_session_goal(goal: Option<&str>) -> String {
    let Some(line) = goal.unwrap_or_default().lines().map(str::trim).find(|line| !line.is_empty())
    else {
        return "none".to_string();
    };

    const MAX_GOAL_SUMMARY_CHARS: usize = 96;
    let mut summary: String = line.chars().take(MAX_GOAL_SUMMARY_CHARS).collect();
    if line.chars().nth(MAX_GOAL_SUMMARY_CHARS).is_some() {
        summary.push_str("...");
    }

    summary
}

fn session_goal_hint<'a>(
    goal: Option<&'a str>,
    bundle: &'a AuthoredBriefBundle,
) -> Option<&'a str> {
    goal.map(str::trim).filter(|goal| !goal.is_empty()).or_else(|| {
        bundle
            .derived_task_draft
            .as_ref()
            .map(|draft| draft.bounded_goal.trim())
            .filter(|goal| !goal.is_empty())
    })
}

fn session_history_status_label(status: SessionStatus) -> &'static str {
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

fn render_session_history(
    workspace: &Path,
    active_session_id: Option<&str>,
    sessions: &[ActiveSessionRecord],
) -> String {
    let mut lines = vec![
        "session_history:".to_string(),
        format!("workspace: {}", workspace.display()),
        format!("source_of_truth: {SESSION_SOURCE_OF_TRUTH}"),
        format!("session_count: {}", sessions.len()),
        format!("active_session: {}", active_session_id.unwrap_or("none")),
    ];

    if sessions.is_empty() {
        lines.push("summary: no persisted sessions found for the workspace history".to_string());
        lines.push(format!("next_command: {SESSION_HISTORY_CREATE_HINT}"));
        return lines.join("\n");
    }

    for (index, record) in sessions.iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("session_{}:", index + 1));
        lines.push(format!("session_id: {}", record.session_id));
        lines.push(format!("active: {}", active_session_id == Some(record.session_id.as_str())));
        lines.push(format!("branch: {}", session_branch_ref(&record.session_id)));
        lines
            .push(format!("latest_status: {}", session_history_status_label(record.latest_status)));
        lines.push(format!("goal_summary: {}", summarize_session_goal(record.goal.as_deref())));
        lines.push(format!("updated_at: {}", record.updated_at));
        lines.push(format!(
            "latest_trace_ref: {}",
            record.latest_trace_ref.as_deref().unwrap_or("none")
        ));
    }

    lines.push(String::new());
    lines.push(format!("next_command: {SESSION_HISTORY_RESUME_HINT}"));
    lines.join("\n")
}

fn persist_session_status_brief_artifact(
    workspace: &Path,
    kind: SessionBriefArtifactKind,
    view: &SessionStatusView,
) -> Result<String, SessionCommandError> {
    persist_session_brief_artifact(
        workspace,
        &view.session_id,
        kind,
        &output::render_session_status_brief(view),
    )
}

fn persist_trace_summary_brief_artifact(
    workspace: &Path,
    session_id: &str,
    summary: &TraceSummaryView,
    next_command: &str,
) -> Result<String, SessionCommandError> {
    persist_session_brief_artifact(
        workspace,
        session_id,
        SessionBriefArtifactKind::Run,
        &output::render_trace_summary_brief(summary, None, next_command),
    )
}

fn persist_session_brief_artifact(
    workspace: &Path,
    session_id: &str,
    kind: SessionBriefArtifactKind,
    rendered_brief: &str,
) -> Result<String, SessionCommandError> {
    let artifact_ref = kind.brief_ref(session_id);
    let artifact_path = workspace.join(&artifact_ref);
    let Some(parent) = artifact_path.parent() else {
        return Err(SessionCommandError::InvalidRequest(format!(
            "session brief artifact path has no parent: {}",
            artifact_path.display()
        )));
    };

    fs::create_dir_all(parent).map_err(|source| SessionCommandError::BriefWrite {
        artifact_ref: artifact_ref.clone(),
        source,
    })?;
    fs::write(&artifact_path, render_session_brief_markdown(kind, rendered_brief)).map_err(
        |source| SessionCommandError::BriefWrite { artifact_ref: artifact_ref.clone(), source },
    )?;

    Ok(artifact_ref)
}

fn render_session_brief_markdown(kind: SessionBriefArtifactKind, rendered_brief: &str) -> String {
    let mut lines = vec![
        format!("# {}", kind.title()),
        String::new(),
        SESSION_BRIEF_ARTIFACT_NOTE.to_string(),
        String::new(),
    ];

    for line in rendered_brief.lines().map(str::trim).filter(|line| !line.is_empty()) {
        lines.push(format!("- {line}"));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn persisted_session_brief_ref(workspace: &Path, brief_ref: &str) -> Option<String> {
    workspace.join(brief_ref).is_file().then(|| brief_ref.to_string())
}

fn nearest_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    loop {
        if current.join(GIT_DIRECTORY_ENTRY_NAME).exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn git_command_failure_detail(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }

    "git command exited without output".to_string()
}

fn activate_session_branch(workspace: &Path, session_ref: &str) -> Result<(), SessionCommandError> {
    let Some(repo_root) = nearest_git_root(workspace) else {
        return Ok(());
    };
    let branch_ref = session_branch_ref(session_ref);
    let mut last_detail = String::new();
    for attempt_index in 0..GIT_BRANCH_ACTIVATION_RETRY_ATTEMPTS {
        let mut command = Command::new(GIT_PROGRAM);
        command.current_dir(&repo_root).arg(GIT_SWITCH_SUBCOMMAND);
        if !session_branch_exists(&repo_root, &branch_ref)? {
            command.arg(GIT_CREATE_BRANCH_FLAG);
        }
        let output = command.arg(&branch_ref).output().map_err(|source| {
            SessionCommandError::GitBranchCreate {
                branch_ref: branch_ref.clone(),
                repo_root: repo_root.clone(),
                source,
            }
        })?;

        if output.status.success() {
            return Ok(());
        }

        let detail = git_command_failure_detail(&output);
        last_detail = detail.clone();
        let has_retries_remaining = attempt_index + 1 < GIT_BRANCH_ACTIVATION_RETRY_ATTEMPTS;
        if has_retries_remaining && is_git_index_lock_contention(&detail) {
            thread::sleep(Duration::from_millis(GIT_BRANCH_ACTIVATION_RETRY_DELAY_MILLIS));
            continue;
        }

        return Err(SessionCommandError::GitBranchCreateFailed { branch_ref, repo_root, detail });
    }

    Err(SessionCommandError::GitBranchCreateFailed { branch_ref, repo_root, detail: last_detail })
}

fn is_git_index_lock_contention(detail: &str) -> bool {
    detail.contains(GIT_INDEX_LOCK_MARKER) && detail.contains(GIT_FILE_EXISTS_MARKER)
}

fn session_branch_exists(repo_root: &Path, branch_ref: &str) -> Result<bool, SessionCommandError> {
    let branch_head_ref = format!("{GIT_LOCAL_BRANCH_REF_PREFIX}/{branch_ref}");
    let output = Command::new(GIT_PROGRAM)
        .current_dir(repo_root)
        .arg(GIT_SHOW_REF_SUBCOMMAND)
        .arg(GIT_VERIFY_FLAG)
        .arg(GIT_QUIET_FLAG)
        .arg(&branch_head_ref)
        .output()
        .map_err(|source| SessionCommandError::GitBranchCreate {
            branch_ref: branch_ref.to_string(),
            repo_root: repo_root.to_path_buf(),
            source,
        })?;

    Ok(output.status.success())
}

fn apply_capture_governance_selection(
    record: &mut ActiveSessionRecord,
    governance: GovernanceRuntimeKind,
) {
    let mut lifecycle = record.governance_lifecycle.clone().unwrap_or(GovernedSessionLifecycle {
        governance_runtime: governance,
        explicit_opt_out: governance == GovernanceRuntimeKind::Local,
        mode_selection_preference: CanonModeSelectionPreference::default(),
        selected_mode: None,
        selected_mode_sequence: Vec::new(),
        latest_reasoning_profile: None,
        current_stage_index: 0,
        stage_records: Vec::new(),
        accumulated_context: Vec::new(),
        terminal_reason: None,
        planning_input_fingerprint: None,
    });

    lifecycle.governance_runtime = governance;
    lifecycle.explicit_opt_out = governance == GovernanceRuntimeKind::Local;
    lifecycle.selected_mode = None;
    lifecycle.selected_mode_sequence.clear();
    lifecycle.latest_reasoning_profile = None;
    lifecycle.current_stage_index = 0;
    lifecycle.stage_records.clear();
    lifecycle.accumulated_context.clear();
    lifecycle.terminal_reason = None;
    record.governance_lifecycle = Some(lifecycle);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedSessionTarget {
    owner_workspace: PathBuf,
    cluster_projection: Option<ClusterSessionProjection>,
}

/// Lists persisted sessions for the current or requested workspace.
pub fn execute_session_list(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_session_list_with_target(workspace, None)
}

/// Lists persisted sessions for an explicit workspace or cluster target.
pub fn execute_session_list_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "session list")?.owner_workspace;
    let store = FileSessionStore::for_workspace(&workspace);
    let active_session_id = store.load().map_err(map_store_error)?.map(|record| record.session_id);
    let sessions = store.list_sessions().map_err(map_store_error)?;

    Ok(report_with_text(
        CommandExitStatus::Succeeded,
        render_session_history(&workspace, active_session_id.as_deref(), &sessions),
    ))
}

/// Reactivates one persisted session for the current or requested workspace.
pub fn execute_session_resume(
    workspace: Option<&Path>,
    session_id: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_session_resume_with_target(workspace, None, session_id)
}

/// Reactivates one persisted session for an explicit workspace or cluster target.
pub fn execute_session_resume_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    session_id: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "session resume")?;
    let workspace = target.owner_workspace;
    let store = FileSessionStore::for_workspace(&workspace);
    let Some(record) = store.select_active_session(session_id).map_err(map_store_error)? else {
        return Err(SessionCommandError::UnknownSession {
            session_id: session_id.to_string(),
            workspace,
        });
    };

    let branch_ref = session_branch_ref(&record.session_id);
    let switched_branch = nearest_git_root(Path::new(&record.workspace_ref)).is_some();
    activate_session_branch(Path::new(&record.workspace_ref), &record.session_id)?;

    let explanation = if target.cluster_projection.is_some() {
        if switched_branch {
            format!(
                "reactivated the persisted clustered session and switched the repository to `{branch_ref}`"
            )
        } else {
            "reactivated the persisted clustered session for the primary workspace".to_string()
        }
    } else if switched_branch {
        format!(
            "reactivated the persisted workspace session and switched the repository to `{branch_ref}`"
        )
    } else {
        "reactivated the persisted workspace session".to_string()
    };
    let view = build_status_view(&record, suggested_next_command(&record), explanation);

    Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
}

/// Returns the number of session directories in `workspace/.boundline/sessions/`
/// whose names start with `date_prefix` (e.g. `"20260525"`).  Used to derive
/// the 1-based daily sequence number for new session references.
pub(crate) fn count_sessions_for_date(workspace: &Path, date_prefix: &str) -> u16 {
    let sessions_dir = workspace.join(session_storage_root_ref());
    let Ok(entries) = fs::read_dir(&sessions_dir) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(date_prefix))
        .count()
        .min(u16::MAX as usize) as u16
}

fn persist_initialized_session_with_goal_hint(
    workspace: &Path,
    goal_hint: Option<&str>,
    slug_override: Option<&str>,
) -> Result<ActiveSessionRecord, SessionCommandError> {
    let now = current_timestamp_millis();
    let date_prefix = date_prefix_from_millis(now);
    let daily_seq = count_sessions_for_date(workspace, &date_prefix) + 1;
    let record = ActiveSessionRecord {
        session_id: generate_session_ref(goal_hint, &date_prefix, daily_seq, slug_override),
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
    };

    activate_session_branch(workspace, &record.session_id)?;
    FileSessionStore::for_workspace(workspace).persist(&record)?;

    Ok(record)
}

/// Records a goal and optional authored briefs into the active session.
pub fn execute_goal(
    workspace: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_goal_with_target(workspace, None, goal, briefs, governance, risk, zone, owner, None)
}

/// Updates the active session goal and optional authored briefs in place.
#[allow(clippy::too_many_arguments)]
pub fn execute_goal_update(
    workspace: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_goal_update_with_target(workspace, None, goal, briefs, governance, risk, zone, owner)
}

fn can_update_goal_in_place(status: SessionStatus) -> bool {
    matches!(
        status,
        SessionStatus::Initialized
            | SessionStatus::GoalCaptured
            | SessionStatus::Planned
            | SessionStatus::Blocked
            | SessionStatus::Running
    )
}

fn load_goal_update_session(workspace: &Path) -> Result<ActiveSessionRecord, SessionCommandError> {
    let record = load_active_session(workspace)?;
    if can_update_goal_in_place(record.latest_status) {
        Ok(record)
    } else {
        Err(SessionCommandError::GoalUpdateRequiresNewSession { status: record.latest_status })
    }
}

/// Records a goal and optional authored briefs into an explicit workspace or cluster target.
#[allow(clippy::too_many_arguments)]
pub fn execute_goal_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    slug: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_goal_with_target_mode(
        workspace, cluster, goal, briefs, governance, risk, zone, owner, false, slug,
    )
}

/// Updates the active goal and authored briefs for an explicit workspace or cluster target.
#[allow(clippy::too_many_arguments)]
pub fn execute_goal_update_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_goal_with_target_mode(
        workspace, cluster, goal, briefs, governance, risk, zone, owner, true, None,
    )
}

/// Sets or updates the bounded goal using upsert semantics: updates the active
/// non-terminal session when one exists, creates a new session otherwise.
///
/// This mirrors the behavior that `orchestrate --goal` uses internally (try
/// update first, fall back to create on missing or terminal session).
#[allow(clippy::too_many_arguments)]
pub fn execute_goal_upsert_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    slug: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    match execute_goal_with_target_mode(
        workspace, cluster, goal, briefs, governance, risk, zone, owner, true, None,
    ) {
        Ok(report) => Ok(report),
        Err(
            SessionCommandError::MissingActiveSession
            | SessionCommandError::GoalUpdateRequiresNewSession { .. },
        ) => execute_goal_with_target_mode(
            workspace, cluster, goal, briefs, governance, risk, zone, owner, false, slug,
        ),
        Err(error) => Err(error),
    }
}

pub fn execute_goal_clarification_answer_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    answer: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "goal clarification")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;
    let bundle = record
        .authored_brief
        .as_ref()
        .ok_or_else(|| {
            SessionCommandError::InvalidRequest(
                "goal clarification answer requires authored brief input".to_string(),
            )
        })?
        .with_clarification_answer(answer);
    let effective_goal = bundle.render_goal_text();

    runtime.capture_goal(&mut record, &effective_goal).map_err(map_runtime_error)?;
    record.authored_brief = Some(bundle.clone());
    record.negotiation_packet = Some(NegotiatedDeliveryPacket::from_authored_brief(
        &record.session_id,
        &record.workspace_ref,
        &effective_goal,
        &bundle,
    ));
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let summary = if bundle.clarification.is_some() {
        "updated the active goal with the clarification answer, but clarification is still required before planning can continue"
            .to_string()
    } else {
        "applied the clarification answer to the active goal".to_string()
    };
    let explanation = if target.cluster_projection.is_some() {
        format!("{summary} for the current clustered delivery session")
    } else {
        summary
    };
    let preview_view =
        build_status_view(&record, suggested_next_command(&record), explanation.clone());
    persist_session_status_brief_artifact(
        &workspace,
        SessionBriefArtifactKind::Goal,
        &preview_view,
    )?;
    let view = build_status_view(&record, suggested_next_command(&record), explanation);

    Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
}

#[allow(clippy::too_many_arguments)]
fn execute_goal_with_target_mode(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    update_existing: bool,
    slug: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(
        workspace,
        cluster,
        if update_existing { "goal update" } else { "goal" },
    )?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);

    let governance_intent = normalize_governance_intent(governance, risk, zone, owner)
        .map_err(SessionCommandError::BriefIngestion)?;
    let bundle = normalize_inputs_with_governance(&workspace, goal, briefs, governance_intent)
        .map_err(SessionCommandError::BriefIngestion)?;
    let effective_goal = bundle.render_goal_text();

    let goal_hint = session_goal_hint(goal, &bundle);
    let mut record = if update_existing {
        load_goal_update_session(&workspace)?
    } else {
        persist_initialized_session_with_goal_hint(&workspace, goal_hint, slug)?
    };

    runtime.capture_goal(&mut record, &effective_goal).map_err(map_runtime_error)?;
    record.authored_brief = Some(bundle.clone());
    record.negotiation_packet = Some(NegotiatedDeliveryPacket::from_authored_brief(
        &record.session_id,
        &record.workspace_ref,
        &effective_goal,
        &bundle,
    ));
    if let Some(governance) = governance {
        apply_capture_governance_selection(&mut record, governance);
    }
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let summary = goal_capture_summary(&bundle, update_existing);

    let explanation = if target.cluster_projection.is_some() {
        format!("{summary} for the current clustered delivery session")
    } else {
        summary
    };
    let preview_view =
        build_status_view(&record, suggested_next_command(&record), explanation.clone());
    persist_session_status_brief_artifact(
        &workspace,
        SessionBriefArtifactKind::Goal,
        &preview_view,
    )?;
    let view = build_status_view(&record, suggested_next_command(&record), explanation);

    Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
}

fn goal_capture_summary(bundle: &AuthoredBriefBundle, update_existing: bool) -> String {
    let action = if update_existing { "updated" } else { "recorded" };
    if bundle.clarification.is_some() {
        return format!(
            "{action} the active goal, but clarification is required before planning can continue"
        );
    }

    let markdown_source_count = bundle.markdown_source_count();
    if markdown_source_count == 0 {
        return format!("{action} the active goal for the current workspace session");
    }

    format!(
        "{action} the active goal with {markdown_source_count} Markdown brief source(s) for the current workspace session"
    )
}

/// Selects a delivery flow for the active session.
pub fn execute_flow(
    workspace: Option<&Path>,
    name: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_flow_with_target(workspace, None, name)
}

/// Selects a delivery flow for an explicit workspace or cluster target.
pub fn execute_flow_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    name: &str,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "flow")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    runtime.select_flow(&mut record, name).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let view = build_status_view(
        &record,
        suggested_next_command(&record),
        if target.cluster_projection.is_some() {
            format!("selected the `{}` delivery flow for the active clustered session", name)
        } else {
            format!("selected the `{}` delivery flow for the active workspace session", name)
        },
    );

    Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
}

/// Builds a plan for the active session.
pub fn execute_plan(
    workspace: Option<&Path>,
    requested_flow: Option<&str>,
    no_flow: bool,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_plan_with_target_input(
        workspace,
        None,
        requested_flow,
        no_flow,
        false,
        None,
        false,
        false,
        None,
    )
}

/// Builds a plan for an explicit workspace or cluster target.
pub fn execute_plan_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    requested_flow: Option<&str>,
    no_flow: bool,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_plan_with_target_input(
        workspace,
        cluster,
        requested_flow,
        no_flow,
        false,
        None,
        false,
        false,
        None,
    )
}

/// Builds a plan for an explicit workspace or cluster target,
/// optionally refreshing the authored planning input from a Markdown file.
#[allow(clippy::too_many_arguments)]
pub fn execute_plan_with_target_input(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    requested_flow: Option<&str>,
    no_flow: bool,
    no_canon: bool,
    input: Option<&Path>,
    refine: bool,
    no_refine: bool,
    max_rounds: Option<u32>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "plan")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if let Some(input) = input {
        let governance_intent =
            record.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.clone());
        let direct_goal = record
            .authored_brief
            .as_ref()
            .and_then(|bundle| bundle.primary_goal_text.as_deref())
            .or_else(|| {
                record.authored_brief.is_none().then_some(record.goal.as_deref()).flatten()
            });
        let bundle = normalize_inputs_with_governance(
            &workspace,
            direct_goal,
            &[input.to_path_buf()],
            governance_intent,
        )
        .map_err(SessionCommandError::BriefIngestion)?;
        runtime.refresh_planning_input(&mut record, bundle).map_err(map_runtime_error)?;
    }

    if no_canon {
        apply_capture_governance_selection(&mut record, GovernanceRuntimeKind::Local);
        if let Some(bundle) = record.authored_brief.as_mut() {
            let mut intent = bundle.governance_intent.clone().unwrap_or(GovernanceIntent {
                requested: true,
                runtime_preference: Some(GovernanceRuntimeKind::Local),
                risk: None,
                zone: None,
                owner: None,
                explicit_mode: None,
                explicit_no_canon: true,
            });
            intent.requested = true;
            intent.runtime_preference = Some(GovernanceRuntimeKind::Local);
            intent.explicit_no_canon = true;
            bundle.governance_intent = Some(intent);
        }
    }

    let plan_result = runtime.plan_task(&mut record, requested_flow, no_flow);

    if let Err(error) = plan_result {
        if matches!(&error, SessionRuntimeError::ClarificationRequired { .. })
            || record.latest_terminal_reason.is_some()
            || record.latest_trace_ref.is_some()
            || record.latest_status.is_terminal()
        {
            runtime.persist_session(&record).map_err(map_runtime_error)?;
        }
        return Err(map_runtime_error(error));
    }
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    // ── Refinement Hook ────────────────────────────────────────────────
    // After plan_task succeeds, optionally run a bounded refinement loop.
    let _refinement_outcome =
        run_plan_refinement_if_enabled(&workspace, refine, no_refine, max_rounds);

    let planning_explanation = if target.cluster_projection.is_some() {
        format!("{} for the clustered delivery story", planning_summary(&record))
    } else {
        planning_summary(&record)
    };
    let preview_view =
        build_status_view(&record, suggested_next_command(&record), planning_explanation.clone());
    persist_session_status_brief_artifact(
        &workspace,
        SessionBriefArtifactKind::Plan,
        &preview_view,
    )?;
    let mut view =
        build_status_view(&record, suggested_next_command(&record), planning_explanation);
    if let Some(trace_ref) = record.latest_trace_ref.as_deref()
        && let Ok(trace) = runtime.trace_store().load(Path::new(trace_ref))
    {
        if let Ok(summary) = summarize_trace(Path::new(trace_ref), &trace) {
            view.latest_framework_adapter_stage_routing = summary.framework_adapter_stage_routing;
            view.latest_framework_adapter_hook_dispatch = summary.framework_adapter_hook_dispatch;
        } else {
            view.latest_framework_adapter_stage_routing =
                latest_framework_adapter_stage_routing_from_trace(&trace);
        }
    }

    Ok(report_with_session_guidance(
        exit_status_for_session(record.latest_status),
        view,
        record.goal_plan.as_ref().map(|goal_plan| &goal_plan.guidance_guardian),
    ))
}

/// Run a bounded plan refinement loop if refinement is enabled via CLI flags
/// or workspace config. Returns the outcome on success, or silently skips
/// refinement on config errors (refinement is best-effort after plan_task
/// succeeds).
fn run_plan_refinement_if_enabled(
    workspace: &Path,
    refine: bool,
    no_refine: bool,
    max_rounds: Option<u32>,
) -> Option<crate::domain::refinement::RefinementOutcome> {
    use crate::domain::refinement::{load_refinement_profile, resolve_effective_profile};
    use crate::orchestrator::refinement::{ResolvedRefinementRoles, execute_refinement_loop};
    use std::time::Duration;

    // Load profile from workspace config.
    let config_profile = load_refinement_profile(workspace, "plan_refinement").ok()??;

    // Resolve effective profile with CLI overrides.
    let effective =
        resolve_effective_profile(Some(config_profile), refine, no_refine, max_rounds, None)
            .ok()?;

    if !effective.enabled {
        return None;
    }

    // Resolve provider roles via a no-op lookup (full registry integration
    // deferred to a follow-up task). For initial implementation, any
    // non-empty provider ID is accepted.
    let roles = ResolvedRefinementRoles::resolve(&effective.roles, &|id: &str| {
        if id.is_empty() { Err("not found".to_string()) } else { Ok(()) }
    })
    .ok()?;

    let max_elapsed = Duration::from_secs(effective.max_elapsed_time_seconds);

    let outcome = execute_refinement_loop(&effective, &roles, max_elapsed, |_packet| {
        // In production, each packet is emitted as a trace event via
        // trace.record_event(TraceEventType::RefinementRoundCompleted, ...).
        // For initial implementation, packets are logged at trace level.
        tracing::debug!(
            profile = %effective.profile,
            round = %_packet.round,
            "refinement round completed"
        );
    })
    .ok()?;

    tracing::info!(
        profile = %effective.profile,
        outcome = ?outcome,
        "refinement loop finished"
    );

    Some(outcome)
}

/// Check if refinement is configured for this session's workspace and
/// return a refinement-aware next-step hint.
fn refinement_next_hint(_record: &ActiveSessionRecord) -> Option<String> {
    // In a full implementation, this would check the session's workspace
    // for .boundline/refinement-profiles.toml and return a hint like
    // "boundline plan --refine" or "boundline inspect" to see refinement
    // results. For initial delivery, refinement hints are deferred.
    None
}

/// Build a short human-readable summary of refinement state for status output.
fn refinement_status_summary(workspace: &Path) -> Option<String> {
    use crate::domain::refinement::load_refinement_profile;
    let profile = load_refinement_profile(workspace, "plan_refinement").ok()??;
    if !profile.enabled {
        return None;
    }
    Some(format!(
        "profile={} stage={} max_rounds={} max_time={}s",
        profile.profile, profile.stage, profile.max_rounds, profile.max_elapsed_time_seconds
    ))
}

/// Executes the next planned compatibility step for the active session.
pub fn execute_step(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_step_with_target(workspace, None)
}

/// Executes the next planned compatibility step for an explicit workspace or cluster target.
pub fn execute_step_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "step")?.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    if runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?
        && governance_refresh_requires_pause(&record)
    {
        runtime.persist_session(&record).map_err(map_runtime_error)?;
        let view = build_status_view(
            &record,
            suggested_next_command(&record),
            "refreshed governance approval state and returned without executing another step",
        );
        return Ok(report_with_session_status(exit_status_for_session(record.latest_status), view));
    }

    runtime.execute_next_step(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    let view = build_status_view(
        &record,
        suggested_next_command(&record),
        "executed the next planned step and persisted the updated session state",
    );

    Ok(report_with_session_status(exit_status_for_session(record.latest_status), view))
}

/// Runs the active session to terminal.
pub fn execute_run(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_run_with_target(workspace, None)
}

/// Runs the active session to terminal for an explicit workspace or cluster target.
pub fn execute_run_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let target = resolve_session_target(workspace, cluster, "run")?;
    let workspace = target.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    let mut record = load_active_session(&workspace)?;

    if record.goal.as_deref().map(str::trim).unwrap_or_default().is_empty() {
        return Err(SessionCommandError::MissingCapturedGoal);
    }

    if let Some(projection) = target.cluster_projection.as_ref() {
        runtime.prepare_cluster_run(&mut record, projection).map_err(map_runtime_error)?;
    }

    let uses_native_goal_plan =
        runtime.uses_native_goal_plan(&record).map_err(map_runtime_error)?;

    if !uses_native_goal_plan && record.active_task.is_none() {
        return Err(SessionCommandError::MissingPlannedTask);
    }

    if runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?
        && governance_refresh_requires_pause(&record)
    {
        runtime.persist_session(&record).map_err(map_runtime_error)?;
        let view = build_status_view(
            &record,
            suggested_next_command(&record),
            "refreshed governance approval state and returned without resuming the governed stage",
        );
        return Ok(report_with_session_status(exit_status_for_session(record.latest_status), view));
    }

    runtime.refresh_completion_verification_state(&mut record).map_err(map_runtime_error)?;

    if let Some(explanation) = run_pause_explanation(&record, &workspace) {
        record.latest_status = SessionStatus::Blocked;
        record.latest_terminal_reason = None;
        record.updated_at = current_timestamp_millis();
        runtime.persist_blocked_plan_quality_trace(&mut record).map_err(map_runtime_error)?;
        runtime.persist_session(&record).map_err(map_runtime_error)?;
        let view = build_status_view(&record, suggested_next_command(&record), explanation);
        return Ok(report_with_session_status(exit_status_for_session(record.latest_status), view));
    }

    let mut response = runtime.run_to_terminal(&mut record).map_err(map_runtime_error)?;
    runtime.persist_session(&record).map_err(map_runtime_error)?;

    if record.latest_status == SessionStatus::Blocked {
        let blocked_explanation = record
            .active_task
            .as_ref()
            .and_then(|task| task.context.completion_verification_projection().ok().flatten())
            .map(|projection| match projection.completion_verification_state {
                crate::domain::completion_verification::CompletionVerificationState::ProofRequired => {
                    "completion verification requires a fresh proof run before the task can close"
                        .to_string()
                }
                crate::domain::completion_verification::CompletionVerificationState::Blocked => {
                    "completion verification blocked closeout because the claimed outcome has no usable proof path"
                        .to_string()
                }
                crate::domain::completion_verification::CompletionVerificationState::Failed => {
                    "completion verification failed and left the task incomplete".to_string()
                }
                crate::domain::completion_verification::CompletionVerificationState::Ready => {
                    "completion verification remains ready".to_string()
                }
            })
            .unwrap_or_else(|| {
                "framework-adapter blocked the claimed run stage and left the session incomplete"
                    .to_string()
            });
        let mut view =
            build_status_view(&record, suggested_next_command(&record), blocked_explanation);
        if let Some(trace_ref) = record.latest_trace_ref.as_deref()
            && let Ok(trace) = runtime.trace_store().load(Path::new(trace_ref))
        {
            if let Ok(summary) = summarize_trace(Path::new(trace_ref), &trace) {
                view.latest_framework_adapter_stage_routing =
                    summary.framework_adapter_stage_routing;
                view.latest_framework_adapter_hook_dispatch =
                    summary.framework_adapter_hook_dispatch;
            } else {
                view.latest_framework_adapter_stage_routing =
                    latest_framework_adapter_stage_routing_from_trace(&trace);
            }
        }
        return Ok(report_with_session_status(exit_status_for_session(record.latest_status), view));
    }

    if response.final_context.cluster_delivery_story().ok().flatten().is_none()
        && let Some(cluster_story) = cluster_delivery_story_for_record(&record)
    {
        response.final_context.set_cluster_delivery_story(&cluster_story).map_err(|error| {
            SessionCommandError::InvalidRequest(format!(
                "failed to project clustered delivery story into run output: {error}"
            ))
        })?;
    }

    if response.terminal_status == TaskStatus::Failed && delegation_status_view(&record).is_some() {
        let view = build_status_view(
            &record,
            suggested_next_command(&record),
            "run stopped at an explicit delegated continuity boundary and persisted the packet in session-owned state",
        );
        return Ok(report_with_session_status(
            exit_status_for_task(response.terminal_status),
            view,
        ));
    }

    let trace = runtime.trace_store().load(Path::new(&response.trace_location)).ok();
    let trace_summary = trace
        .as_ref()
        .and_then(|trace| summarize_trace(Path::new(&response.trace_location), trace).ok());
    let next_command =
        suggested_next_command(&record).unwrap_or_else(|| "boundline inspect".to_string());
    let trace_location = Some(response.trace_location.clone());
    let run_brief_ref = if let Some(summary) = trace_summary.as_ref() {
        Some(persist_trace_summary_brief_artifact(
            &workspace,
            &record.session_id,
            summary,
            &next_command,
        )?)
    } else {
        Some(persist_session_brief_artifact(
            &workspace,
            &record.session_id,
            SessionBriefArtifactKind::Run,
            &output::render_run_trace("run", trace.as_ref(), &response, &next_command),
        )?)
    };
    let mut trace_summary = trace_summary;
    if let (Some(summary), Some(run_brief_ref)) = (trace_summary.as_mut(), run_brief_ref.as_ref()) {
        summary.run_brief_ref = Some(run_brief_ref.clone());
    }
    let routing_prefix = trace_summary
        .as_ref()
        .and_then(|summary| summary.routing_summary.clone())
        .unwrap_or_else(|| output::render_route_outcome(&routing_outcome(&record)));
    let mut terminal_output = format!(
        "{routing_prefix}\n{}",
        output::render_run_trace("run", trace.as_ref(), &response, &next_command),
    );
    if let Some(run_brief_ref) = run_brief_ref.as_ref() {
        terminal_output.push('\n');
        terminal_output.push_str(&format!("run_brief_ref: {run_brief_ref}"));
    }

    Ok(report_with_trace_summary(
        exit_status_for_task(response.terminal_status),
        terminal_output,
        trace_location,
        trace_summary,
    ))
}

/// Renders the current active session status for the workspace.
pub fn execute_status(
    workspace: Option<&Path>,
) -> Result<SessionCommandReport, SessionCommandError> {
    execute_status_with_target(workspace, None, None)
}

/// Renders the current active session status for an explicit workspace or cluster target.
pub fn execute_status_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    session_id: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "status")?.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    match load_selected_session(&workspace, session_id) {
        Ok(mut record) => {
            let refreshed =
                runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?;
            let completion_refreshed = runtime
                .refresh_completion_verification_state(&mut record)
                .map_err(map_runtime_error)?;
            if refreshed || completion_refreshed {
                persist_resolved_session(&runtime, &record, session_id)?;
            }
            let compatibility_follow_up = latest_workspace_compatibility_follow_up(
                &workspace,
                record.latest_trace_ref.as_deref(),
            )?;

            let mut view = build_status_view_with_follow_up(
                &record,
                suggested_next_command(&record),
                if compatibility_follow_up.is_some() {
                    if session_id.is_some() {
                        "current selected session state for the workspace; latest compatibility follow-up remains inspect-only"
                    } else {
                        "current active session state for the workspace; latest compatibility follow-up remains inspect-only"
                    }
                } else if refreshed {
                    if session_id.is_some() {
                        "refreshed governance approval state for the selected workspace session"
                    } else {
                        "refreshed governance approval state for the active workspace session"
                    }
                } else if completion_refreshed {
                    if session_id.is_some() {
                        "refreshed completion verification state for the selected workspace session"
                    } else {
                        "refreshed completion verification state for the active workspace session"
                    }
                } else if session_id.is_some() {
                    "current selected session state for the workspace"
                } else {
                    "current active session state for the workspace"
                },
                compatibility_follow_up,
            );
            if let Some(trace_ref) = record.latest_trace_ref.as_deref()
                && let Ok(trace) = runtime.trace_store().load(Path::new(trace_ref))
            {
                if let Ok(summary) = summarize_trace(Path::new(trace_ref), &trace) {
                    if view.cluster_delivery_story.is_none() {
                        view.cluster_delivery_story = summary.cluster_delivery_story;
                    }
                    view.latest_framework_adapter_stage_routing =
                        summary.framework_adapter_stage_routing;
                    view.latest_framework_adapter_hook_dispatch =
                        summary.framework_adapter_hook_dispatch;
                } else {
                    view.latest_framework_adapter_stage_routing =
                        latest_framework_adapter_stage_routing_from_trace(&trace);
                }
            }
            // Populate refinement summary for operator visibility.
            if let Some(refinement_info) = refinement_status_summary(&workspace) {
                view.refinement_summary = Some(refinement_info);
            }
            Ok(report_with_session_guidance(
                CommandExitStatus::Succeeded,
                view,
                record.goal_plan.as_ref().map(|goal_plan| &goal_plan.guidance_guardian),
            ))
        }
        Err(SessionCommandError::MissingActiveSession) => {
            let Some(compatibility_follow_up) =
                latest_workspace_compatibility_follow_up(&workspace, None)?
            else {
                return Ok(report_with_text(
                    CommandExitStatus::Succeeded,
                    render_missing_active_session_bootstrap(&workspace, "status"),
                ));
            };

            Ok(report_with_text(
                CommandExitStatus::Succeeded,
                output::render_compatibility_follow_up_status(
                    &workspace.to_string_lossy(),
                    ContinuityAuthority::CompatibilityTrace,
                    &compatibility_follow_up,
                    "no active session exists; latest compatibility trace is the authoritative follow-up state for the workspace",
                ),
            ))
        }
        Err(error) => Err(error),
    }
}

/// Returns the next recommended command for the active session.
pub fn execute_next(workspace: Option<&Path>) -> Result<SessionCommandReport, SessionCommandError> {
    execute_next_with_target(workspace, None, None)
}

/// Resolves the `continue` surface from the persisted active session.
pub fn execute_continue_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    session_id: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "continue")?.owner_workspace;
    let record = match load_selected_session(&workspace, session_id) {
        Ok(record) => record,
        Err(SessionCommandError::MissingActiveSession) => {
            return Ok(report_with_text(
                CommandExitStatus::Succeeded,
                render_missing_active_session_bootstrap(&workspace, "continue"),
            ));
        }
        Err(error) => return Err(error),
    };

    let next_command =
        suggested_next_command(&record).ok_or(SessionCommandError::NotImplemented {
            command_name: "continue",
            next_command: None,
        })?;
    let view = build_status_view(
        &record,
        Some(next_command.clone()),
        if session_id.is_some() {
            format!(
                "continue uses the selected session state resolved from `--session`; next recommended command is `{next_command}`"
            )
        } else {
            format!(
                "continue uses the active session state resolved through {SESSION_SOURCE_OF_TRUTH}; next recommended command is `{next_command}`"
            )
        },
    );
    Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
}

/// Returns the next recommended command for an explicit workspace or cluster target.
pub fn execute_next_with_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    session_id: Option<&str>,
) -> Result<SessionCommandReport, SessionCommandError> {
    let workspace = resolve_session_target(workspace, cluster, "next")?.owner_workspace;
    let runtime = SessionRuntime::for_workspace(&workspace);
    match load_selected_session(&workspace, session_id) {
        Ok(mut record) => {
            let refreshed =
                runtime.refresh_governance_state(&mut record).map_err(map_runtime_error)?;
            if refreshed {
                persist_resolved_session(&runtime, &record, session_id)?;
            }
            let next_command =
                suggested_next_command(&record).ok_or(SessionCommandError::NotImplemented {
                    command_name: "next",
                    next_command: None,
                })?;
            let compatibility_follow_up = latest_workspace_compatibility_follow_up(
                &workspace,
                record.latest_trace_ref.as_deref(),
            )?;

            let view = build_status_view_with_follow_up(
                &record,
                Some(next_command.clone()),
                if let Some(follow_up) = &compatibility_follow_up {
                    format!(
                        "{}; latest compatibility follow-up remains {} via `{}`",
                        next_command_summary(&next_command, refreshed),
                        follow_up.follow_up_mode.as_str(),
                        follow_up.next_command
                    )
                } else {
                    next_command_summary(&next_command, refreshed)
                },
                compatibility_follow_up,
            );
            Ok(report_with_session_status(CommandExitStatus::Succeeded, view))
        }
        Err(SessionCommandError::MissingActiveSession) => {
            let Some(compatibility_follow_up) =
                latest_workspace_compatibility_follow_up(&workspace, None)?
            else {
                return Err(SessionCommandError::MissingActiveSession);
            };

            Ok(report_with_text(
                CommandExitStatus::Succeeded,
                output::render_compatibility_follow_up_status(
                    &workspace.to_string_lossy(),
                    ContinuityAuthority::CompatibilityTrace,
                    &compatibility_follow_up,
                    format!(
                        "next recommended command for the latest compatibility follow-up is `{}`",
                        compatibility_follow_up.next_command
                    ),
                ),
            ))
        }
        Err(error) => Err(error),
    }
}

fn render_missing_active_session_bootstrap(workspace: &Path, command_name: &str) -> String {
    let initialized = workspace.join(".boundline").is_dir();
    let next_command = if initialized {
        format!("boundline session list --workspace {}", workspace.display())
    } else {
        format!("boundline init --workspace {}", workspace.display())
    };
    let alternate = if initialized {
        format!("boundline goal --workspace {} --goal <goal>", workspace.display())
    } else {
        format!("boundline doctor --workspace {}", workspace.display())
    };

    format!(
        concat!(
            "session_bootstrap:\n",
            "command: {}\n",
            "workspace: {}\n",
            "workspace_initialized: {}\n",
            "source_of_truth: {}\n",
            "summary: no active session available; persisted session history may still exist and chat history is not authoritative\n",
            "next_command: {}\n",
            "repair_command: {}\n"
        ),
        command_name,
        workspace.display(),
        initialized,
        SESSION_SOURCE_OF_TRUTH,
        next_command,
        alternate,
    )
}

/// Renders a user-facing session-command error.
pub fn render_error(command_name: &str, error: &SessionCommandError) -> String {
    let next_command = error.next_command();
    output::render_session_error(command_name, &error.message(), next_command.as_deref())
}

fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, SessionCommandError> {
    cli_workspace::resolve_workspace(workspace).map_err(|error| {
        SessionCommandError::WorkspaceResolution(std::io::Error::other(error.to_string()))
    })
}

fn resolve_session_target(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    command_name: &'static str,
) -> Result<ResolvedSessionTarget, SessionCommandError> {
    if let Some(cluster_workspace) = cluster {
        let owner_workspace = resolve_workspace(Some(cluster_workspace))?;
        let cluster_store = FileClusterStore::for_workspace(&owner_workspace);
        let Some(config) = cluster_store.load().map_err(SessionCommandError::ClusterStore)? else {
            return Err(SessionCommandError::MissingClusterConfig {
                workspace: owner_workspace,
                command_name,
            });
        };
        let projection = ClusterSessionProjection {
            cluster_id: config.cluster.cluster_id,
            primary_workspace_ref: config.cluster.primary_workspace_ref,
            member_workspace_refs: config
                .cluster
                .members
                .into_iter()
                .map(|member| member.workspace_ref)
                .collect(),
            started_from_command: command_name.to_string(),
            updated_at: current_timestamp_millis(),
        };

        return Ok(ResolvedSessionTarget { owner_workspace, cluster_projection: Some(projection) });
    }

    Ok(ResolvedSessionTarget {
        owner_workspace: resolve_workspace(workspace)?,
        cluster_projection: None,
    })
}

fn load_active_session(workspace: &Path) -> Result<ActiveSessionRecord, SessionCommandError> {
    load_selected_session(workspace, None)
}

fn load_selected_session(
    workspace: &Path,
    session_id: Option<&str>,
) -> Result<ActiveSessionRecord, SessionCommandError> {
    let workspace_ref = workspace.to_string_lossy().into_owned();
    let store = FileSessionStore::for_workspace(workspace);
    let record = match session_id {
        Some(session_id) => store
            .load_session(session_id)
            .map_err(|error| map_selected_store_error(session_id, error))?
            .ok_or_else(|| SessionCommandError::UnknownSession {
                session_id: session_id.to_string(),
                workspace: workspace.to_path_buf(),
            })?,
        None => store
            .load()
            .map_err(map_store_error)?
            .ok_or(SessionCommandError::MissingActiveSession)?,
    };

    if !workspace_refs_match(workspace, &record.workspace_ref) {
        return Err(SessionCommandError::WorkspaceMismatch {
            expected: workspace_ref,
            actual: record.workspace_ref,
        });
    }

    Ok(record)
}

fn workspace_refs_match(expected_workspace: &Path, actual_workspace_ref: &str) -> bool {
    if expected_workspace == Path::new(actual_workspace_ref) {
        return true;
    }

    match (fs::canonicalize(expected_workspace), fs::canonicalize(actual_workspace_ref)) {
        (Ok(expected), Ok(actual)) => expected == actual,
        _ => false,
    }
}

fn persist_resolved_session(
    runtime: &SessionRuntime,
    record: &ActiveSessionRecord,
    session_id: Option<&str>,
) -> Result<(), SessionCommandError> {
    if session_id.is_some() {
        runtime
            .session_store()
            .persist_without_select(record)
            .map_err(SessionCommandError::SessionStore)?;
        return Ok(());
    }

    runtime.persist_session(record).map_err(map_runtime_error)?;
    Ok(())
}

fn map_store_error(error: SessionStoreError) -> SessionCommandError {
    match error {
        SessionStoreError::InvalidRecord(message) => {
            SessionCommandError::InvalidActiveSession(message)
        }
        other => SessionCommandError::SessionStore(other),
    }
}

fn map_selected_store_error(session_id: &str, error: SessionStoreError) -> SessionCommandError {
    match error {
        SessionStoreError::InvalidRecord(message) => SessionCommandError::InvalidRequest(format!(
            "session `{session_id}` is invalid: {message}"
        )),
        other => SessionCommandError::SessionStore(other),
    }
}

fn map_runtime_error(error: SessionRuntimeError) -> SessionCommandError {
    match error {
        SessionRuntimeError::MissingGoal => SessionCommandError::MissingCapturedGoal,
        SessionRuntimeError::ClarificationRequired { headline, prompt } => {
            SessionCommandError::ClarificationRequired { headline, prompt }
        }
        SessionRuntimeError::MissingActiveTask => SessionCommandError::MissingPlannedTask,
        SessionRuntimeError::PlanningGovernanceUnresolved { stage_key, state, reason } => {
            SessionCommandError::PlanningGovernanceUnresolved { stage_key, state, reason }
        }
        SessionRuntimeError::MissingGoalPlan => SessionCommandError::MissingPlanProposal,
        SessionRuntimeError::UnknownFlow { requested, supported } => {
            SessionCommandError::UnknownFlow { requested, supported }
        }
        SessionRuntimeError::FlowReplacementRequiresReset { current, requested } => {
            SessionCommandError::FlowReplacementRequiresReset { current, requested }
        }
        SessionRuntimeError::InvalidFlowState(message) => {
            SessionCommandError::InvalidFlowState(message)
        }
        other => SessionCommandError::SessionRuntime(other),
    }
}

fn exit_status_for_session(status: SessionStatus) -> CommandExitStatus {
    match status {
        SessionStatus::Blocked
        | SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted
        | SessionStatus::Invalid => CommandExitStatus::NonSuccess,
        SessionStatus::Initialized
        | SessionStatus::GoalCaptured
        | SessionStatus::Planned
        | SessionStatus::Running
        | SessionStatus::Succeeded => CommandExitStatus::Succeeded,
    }
}

fn exit_status_for_task(status: TaskStatus) -> CommandExitStatus {
    match status {
        TaskStatus::Failed | TaskStatus::Exhausted | TaskStatus::Aborted => {
            CommandExitStatus::NonSuccess
        }
        TaskStatus::Planned | TaskStatus::Running | TaskStatus::Succeeded => {
            CommandExitStatus::Succeeded
        }
    }
}

pub(crate) fn build_status_view(
    record: &ActiveSessionRecord,
    next_command: Option<String>,
    explanation: impl Into<String>,
) -> SessionStatusView {
    build_status_view_with_follow_up(record, next_command, explanation, None)
}

pub(crate) fn build_status_view_with_follow_up(
    record: &ActiveSessionRecord,
    next_command: Option<String>,
    explanation: impl Into<String>,
    compatibility_follow_up: Option<CompatibilityFollowUpView>,
) -> SessionStatusView {
    let governance_intent =
        record.authored_brief.as_ref().and_then(|bundle| bundle.governance_intent.as_ref());
    let latest_decision = record.decisions.last();
    let latest_decision_selector = latest_decision.map(|decision| decision.selector_kind());
    let delegation = delegation_status_view(record);
    // Prefer the goal-plan retrieval story while it remains authoritative,
    // then fall back to the task-context snapshot after execution begins.
    let advanced_context = record
        .goal_plan
        .as_ref()
        .and_then(|goal_plan| goal_plan.context_pack.as_ref())
        .and_then(|context_pack| context_pack.advanced_context.clone())
        .or_else(|| {
            record
                .active_task
                .as_ref()
                .and_then(|task| task.context.latest_advanced_context().ok().flatten())
        });
    let task_context_summary =
        record.active_task.as_ref().and_then(task_state_canon_memory_context_summary);
    let task_context_credibility =
        record.active_task.as_ref().and_then(task_state_canon_memory_context_credibility);
    let task_context_primary_inputs =
        record.active_task.as_ref().and_then(task_state_canon_memory_primary_inputs);
    let task_context_provenance =
        record.active_task.as_ref().and_then(task_state_canon_memory_provenance);
    let task_context_staleness_reason =
        record.active_task.as_ref().and_then(task_state_canon_memory_staleness_reason);
    let governance_handoff = record.governance_lifecycle.as_ref().and_then(|lifecycle| {
        governance_confidence_handoff(lifecycle.latest_reasoning_profile.as_ref())
    });
    let completion_verification = record
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_verification_projection().ok().flatten());
    let workspace_path = Path::new(&record.workspace_ref);
    let goal_brief_ref =
        persisted_session_brief_ref(workspace_path, &session_goal_brief_ref(&record.session_id));
    let session_plan_brief_ref =
        persisted_session_brief_ref(workspace_path, &session_plan_brief_ref(&record.session_id));
    let run_brief_ref =
        persisted_session_brief_ref(workspace_path, &session_run_brief_ref(&record.session_id));
    let backlog_quality = backlog_quality_for_record(record, workspace_path);

    SessionStatusView {
        session_id: record.session_id.clone(),
        workspace_ref: record.workspace_ref.clone(),
        session_started_at: Some(record.created_at),
        goal: record.goal.clone(),
        advanced_context,
        negotiation_goal_summary: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.goal_summary.clone()),
        negotiation_resolution: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.resolution_state.as_str().to_string()),
        negotiation_acceptance_boundary: record
            .negotiation_packet
            .as_ref()
            .map(|packet| packet.acceptance_boundary.success_headline.clone()),
        cluster_delivery_story: cluster_delivery_story_for_record(record),
        authored_input_summary: record.authored_brief.as_ref().map(|bundle| bundle.summary_text()),
        authored_input_sources: record
            .authored_brief
            .as_ref()
            .map(|bundle| bundle.ordered_source_labels()),
        authored_input_deduplicated_sources: record.authored_brief.as_ref().and_then(|bundle| {
            let labels = bundle.deduplicated_source_labels();
            (!labels.is_empty()).then_some(labels)
        }),
        goal_quality_state: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::goal_quality_state),
        goal_quality_findings: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::goal_quality_findings),
        goal_quality_assumptions: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::goal_quality_assumptions),
        plan_quality_state: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.plan_quality_state()),
        plan_quality_findings: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.plan_quality_findings()),
        plan_quality_assumptions: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.plan_quality_assumptions()),
        backlog_quality_state: backlog_quality
            .as_ref()
            .map(|assessment| assessment.state.as_str().to_string()),
        backlog_quality_findings: backlog_quality.as_ref().and_then(|assessment| {
            (!assessment.findings.is_empty()).then_some(assessment.findings.clone())
        }),
        backlog_task_count: backlog_quality.as_ref().and_then(|assessment| assessment.task_count),
        backlog_mvp_scope: backlog_quality
            .as_ref()
            .and_then(|assessment| assessment.mvp_scope.clone()),
        backlog_unmapped_items: backlog_quality.as_ref().and_then(|assessment| {
            (!assessment.unmapped_items.is_empty()).then_some(assessment.unmapped_items.clone())
        }),
        planning_analysis_state: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.planning_analysis_state()),
        planning_analysis_findings: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.planning_analysis_findings()),
        planning_analysis_coverage: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.planning_analysis_coverage()),
        context_summary: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_summary())
            .or(task_context_summary),
        context_credibility: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_credibility())
            .or(task_context_credibility),
        context_primary_inputs: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let inputs = goal_plan.context_primary_inputs();
                (!inputs.is_empty()).then_some(inputs)
            })
            .or(task_context_primary_inputs),
        context_provenance: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| {
                let lines = goal_plan.context_provenance_lines();
                (!lines.is_empty()).then_some(lines)
            })
            .or(task_context_provenance),
        context_staleness_reason: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.context_pack.as_ref())
            .and_then(|pack| pack.staleness_reason.clone())
            .or_else(|| {
                record
                    .goal_plan
                    .as_ref()
                    .and_then(|goal_plan| goal_plan.canon_memory_staleness_reason())
            })
            .or(task_context_staleness_reason),
        clarification_headline: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_headline),
        clarification_prompt: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_prompt),
        clarification_missing_fields: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_missing_fields),
        clarification_questions: record
            .authored_brief
            .as_ref()
            .and_then(AuthoredBriefBundle::clarification_questions),
        requested_governance_runtime: governance_intent
            .and_then(|intent| intent.runtime_preference)
            .map(|runtime| requested_governance_runtime_text(runtime).to_string()),
        requested_governance_risk: governance_intent.and_then(|intent| intent.risk.clone()),
        requested_governance_zone: governance_intent.and_then(|intent| intent.zone.clone()),
        requested_governance_owner: governance_intent.and_then(|intent| intent.owner.clone()),
        active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
        flow_state: record
            .goal_plan
            .as_ref()
            .map(|goal_plan| goal_plan.flow_state().summary_text()),
        goal_plan_state: record
            .goal_plan
            .as_ref()
            .map(|goal_plan| goal_plan.proposal_state_text().to_string()),
        goal_plan_revision: record.goal_plan.as_ref().map(|goal_plan| goal_plan.proposal_revision),
        planning_rationale: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.planning_rationale.clone()),
        verification_strategy: record
            .goal_plan
            .as_ref()
            .and_then(|goal_plan| goal_plan.verification_strategy.clone()),
        active_workflow: record.active_workflow_name(),
        workflow_phase: record.active_workflow_phase_text(),
        workflow_next_action: record.active_workflow_next_action(),
        continuity_authority: delegation
            .as_ref()
            .map(|_| ContinuityAuthority::NativeSession)
            .or_else(|| {
                compatibility_follow_up.as_ref().map(|_| ContinuityAuthority::CompatibilityTrace)
            }),
        delegation,
        compatibility_follow_up,
        current_stage_id: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
        current_stage_index: record.active_flow.as_ref().map(|flow| flow.current_stage_index),
        total_stages: record.active_flow.as_ref().map(|flow| flow.total_stages),
        plan_revision: record.active_task.as_ref().map(|task| task.plan.revision),
        current_step_id: record
            .active_task
            .as_ref()
            .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
        current_step_index: record.active_task.as_ref().map(|task| task.plan.current_step_index),
        latest_status: record.latest_status,
        execution_path: execution_path_text(record),
        goal_brief_ref,
        session_plan_brief_ref,
        run_brief_ref,
        latest_trace_ref: record.latest_trace_ref.clone(),
        latest_framework_adapter_stage_routing: None,
        latest_framework_adapter_hook_dispatch: None,
        latest_framework_adapter_stage_failure: record
            .latest_terminal_reason
            .as_ref()
            .and_then(FrameworkAdapterStageFailureDetails::from_terminal_reason),
        latest_capability_provider_execution: None,
        latest_decision_status: latest_decision
            .map(|decision| decision_status_text(decision.status).to_string()),
        latest_decision_target: latest_decision.map(|decision| decision.target.clone()),
        latest_checkpoint_id: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_checkpoint_id")),
        latest_checkpoint_scope: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_checkpoint_scope")),
        latest_checkpoint_restore_command: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_checkpoint_restore_command")),
        latest_changed_files: record.active_task.as_ref().and_then(|task| {
            task.context.state.get("latest_changed_files").and_then(|value| {
                value.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
            })
        }),
        latest_workspace_slice: record
            .active_task
            .as_ref()
            .and_then(task_state_workspace_slice_summary),
        latest_selection_headline: record
            .active_task
            .as_ref()
            .and_then(|task| {
                task.context
                    .state
                    .get("latest_selection_headline")
                    .and_then(|value| value.as_str().map(str::to_string))
            })
            .or_else(|| {
                latest_decision.map(|decision| {
                    let evidence_suffix = decision_evidence_basis(decision)
                        .map(|basis| format!(" based on {basis}"))
                        .unwrap_or_default();
                    format!(
                        "selector {} -> {} (verify: {}){}",
                        decision.selector_kind().as_str(),
                        decision.target,
                        decision.expected_outcome,
                        evidence_suffix,
                    )
                })
            }),
        latest_candidate_family: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_candidate_family"))
            .or_else(|| latest_decision_selector.map(|selector| selector.as_str().to_string())),
        latest_selection_reason: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_selection_reason"))
            .or_else(|| latest_decision.map(|decision| decision.rationale.clone())),
        latest_rejected_candidates: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_strings(task, "latest_rejected_candidates")),
        latest_attempt_lineage: record
            .active_task
            .as_ref()
            .and_then(task_state_attempt_lineage_summary),
        latest_validation_status: record
            .active_task
            .as_ref()
            .and_then(|task| {
                task.context
                    .state
                    .get("latest_validation_status")
                    .and_then(|value| value.as_str().map(str::to_string))
            })
            .or_else(|| {
                latest_decision.and_then(|decision| {
                    match (decision.selector_kind(), decision.tool_result.as_ref()) {
                        (ActionSelector::Test, Some(tool_result)) => Some(if tool_result.success {
                            "passed".to_string()
                        } else {
                            "failed".to_string()
                        }),
                        _ => None,
                    }
                })
            }),
        latest_exhaustion_reason: record
            .active_task
            .as_ref()
            .and_then(|task| task_state_string(task, "latest_exhaustion_reason")),
        latest_review_trigger: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_trigger")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_vote: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_vote")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_outcome: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_outcome")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_council_profile: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_council_profile")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_independence_state: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_independence_state")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_stop_semantics: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_stop_semantics")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_selection_summary: record.active_task.as_ref().and_then(|task| {
            task.context
                .state
                .get("latest_review_selection_summary")
                .and_then(|value| value.as_str().map(str::to_string))
        }),
        latest_review_headline: record.active_task.as_ref().and_then(review_headline_from_task),
        latest_governance_stage: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_stage_key)
            .or_else(|| planning_governance_stage_key(record)),
        latest_governance_runtime: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_runtime_text)
            .or_else(|| planning_governance_runtime_text(record)),
        latest_governance_mode: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_mode_text)
            .or_else(|| planning_governance_mode_text(record)),
        latest_governance_run_ref: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_canon_run_ref)
            .or_else(|| planning_governance_run_ref(record)),
        latest_governance_state: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_state_text)
            .or_else(|| planning_governance_state_text(record)),
        latest_governance_runtime_state: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_runtime_state_text),
        latest_governance_rollout_profile: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_rollout_profile_text),
        latest_governance_reason: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_reason),
        latest_governance_contract_lines: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_contract_lines),
        latest_governance_approval_provenance: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_approval_provenance),
        latest_governance_blocked_reason: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_blocked_reason)
            .or_else(|| planning_governance_blocked_reason(record))
            .or_else(|| {
                governance_handoff.as_ref().and_then(|handoff| {
                    matches!(
                        handoff.admission_effect,
                        ReasoningAdmissionEffect::Gate | ReasoningAdmissionEffect::Escalate
                    )
                    .then_some(handoff.summary.clone())
                })
            }),
        latest_governance_packet_ref: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_ref)
            .or_else(|| planning_governance_packet_ref(record)),
        latest_governance_packet_source_stage: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_source_stage),
        latest_governance_packet_binding_reason: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_packet_binding_reason),
        latest_governance_approval: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_approval_text)
            .or_else(|| planning_governance_approval_text(record)),
        latest_governance_decision: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_decision_headline)
            .or_else(|| governance_handoff.as_ref().map(|handoff| handoff.summary.clone())),
        latest_governance_candidates: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_candidate_actions),
        latest_governance_confidence_level: governance_handoff
            .as_ref()
            .map(|handoff| handoff.confidence_level.as_str().to_string()),
        latest_governance_admission_effect: governance_handoff
            .as_ref()
            .map(|handoff| handoff.admission_effect.as_str().to_string()),
        latest_governance_confidence_summary: governance_handoff
            .as_ref()
            .map(|handoff| handoff.summary.clone()),
        governance_next_action: record
            .active_task
            .as_ref()
            .and_then(task_state_governance_next_action)
            .or_else(|| planning_governance_next_action(record))
            .or_else(|| {
                governance_handoff.as_ref().and_then(|handoff| handoff.next_action.clone())
            }),
        governance_lifecycle_runtime: record
            .governance_lifecycle
            .as_ref()
            .map(|lc| format!("{}", lc.governance_runtime)),
        governance_lifecycle_opt_out: record
            .governance_lifecycle
            .as_ref()
            .filter(|lc| lc.explicit_opt_out)
            .map(|lc| lc.explicit_opt_out),
        governance_lifecycle_mode_selection: record
            .governance_lifecycle
            .as_ref()
            .map(|lc| format!("{}", lc.mode_selection_preference)),
        governance_lifecycle_selected_mode: record
            .governance_lifecycle
            .as_ref()
            .and_then(|lc| lc.selected_mode.map(|mode| mode.as_str().to_string())),
        governance_lifecycle_selected_mode_sequence: record.governance_lifecycle.as_ref().and_then(
            |lc| {
                (!lc.selected_mode_sequence.is_empty()).then_some(
                    lc.selected_mode_sequence
                        .iter()
                        .map(|mode| mode.as_str().to_string())
                        .collect(),
                )
            },
        ),
        latest_reasoning_profile: record
            .governance_lifecycle
            .as_ref()
            .and_then(|lc| lc.latest_reasoning_profile.clone()),
        project_scale_path: record.project_scale.as_ref().map(|state| state.path.stage_names()),
        project_scale_current_stage: record
            .project_scale
            .as_ref()
            .and_then(crate::domain::session::ProjectScaleSessionState::active_stage_text),
        project_scale_next_action: record
            .project_scale
            .as_ref()
            .map(|state| state.next_action.clone()),
        project_scale_checkpoint_refs: record.project_scale.as_ref().and_then(|state| {
            (!state.checkpoint_refs.is_empty()).then_some(state.checkpoint_refs.clone())
        }),
        latest_voting_trigger: record.latest_voting.as_ref().map(|vote| vote.trigger.clone()),
        latest_voting_result: record.latest_voting.as_ref().map(|vote| vote.result.clone()),
        latest_voting_adjudication: record
            .latest_voting
            .as_ref()
            .and_then(|vote| vote.adjudication_result.clone()),
        latest_voting_reviewed_evidence: record
            .latest_voting
            .as_ref()
            .and_then(|vote| vote.reviewed_evidence_ref.clone()),
        latest_voting_blocking: record.latest_voting.as_ref().map(|vote| vote.blocking),
        latest_voting_next_action: record
            .latest_voting
            .as_ref()
            .map(|vote| vote.next_action.clone()),
        delight_feedback: record.delight_feedback.clone(),
        completion_verification_state: completion_verification
            .as_ref()
            .map(|projection| projection.completion_verification_state),
        completion_claim: completion_verification
            .as_ref()
            .and_then(|projection| projection.claim.clone()),
        completion_verification_findings: completion_verification.as_ref().and_then(|projection| {
            (!projection.completion_verification_findings.is_empty())
                .then_some(projection.completion_verification_findings.clone())
        }),
        completion_blocked_claims: completion_verification.as_ref().and_then(|projection| {
            (!projection.completion_blocked_claims.is_empty())
                .then_some(projection.completion_blocked_claims.clone())
        }),
        completion_evidence_refs: completion_verification.as_ref().and_then(|projection| {
            (!projection.completion_evidence_refs.is_empty())
                .then_some(projection.completion_evidence_refs.clone())
        }),
        refinement_summary: None,
        next_command,
        explanation: explanation.into(),
    }
}

fn backlog_quality_for_record(
    record: &ActiveSessionRecord,
    workspace_path: &Path,
) -> Option<BacklogQualityAssessment> {
    let lifecycle = record.governance_lifecycle.as_ref()?;
    backlog_quality_snapshot_for_lifecycle(lifecycle, workspace_path)
        .map(|snapshot| snapshot.assessment)
}

fn run_pause_explanation(
    record: &ActiveSessionRecord,
    workspace_path: &Path,
) -> Option<&'static str> {
    if record
        .goal_plan
        .as_ref()
        .map(|goal_plan| goal_plan.plan_quality_assessment().state)
        .is_some_and(|state| !matches!(state, PlanQualityState::Ready))
    {
        return Some(RUN_PLAN_QUALITY_NOT_READY_EXPLANATION);
    }

    if let Some(backlog_quality) = backlog_quality_for_record(record, workspace_path)
        && !matches!(backlog_quality.state, crate::domain::governance::BacklogQualityState::Ready)
    {
        return Some(RUN_BACKLOG_QUALITY_NOT_READY_EXPLANATION);
    }

    record.goal_plan.as_ref().and_then(|goal_plan| goal_plan.planning_analysis.as_ref()).and_then(
        |projection| {
            matches!(projection.state, crate::domain::goal_plan::PlanningAnalysisState::Blocked)
                .then_some(RUN_PLANNING_ANALYSIS_BLOCKED_EXPLANATION)
        },
    )
}

fn latest_workspace_compatibility_follow_up(
    workspace: &Path,
    session_trace_ref: Option<&str>,
) -> Result<Option<CompatibilityFollowUpView>, SessionCommandError> {
    let store = FileTraceStore::for_workspace(workspace);
    let Some(trace_path) = store.latest().map_err(|error| {
        SessionCommandError::SessionRuntime(SessionRuntimeError::TraceStore(error))
    })?
    else {
        return Ok(None);
    };

    if session_trace_ref.is_some_and(|trace_ref| Path::new(trace_ref) == trace_path.as_path()) {
        return Ok(None);
    }

    let trace = store.load(&trace_path).map_err(|error| {
        SessionCommandError::SessionRuntime(SessionRuntimeError::TraceStore(error))
    })?;
    let summary = summarize_trace(&trace_path, &trace)
        .map_err(|error| SessionCommandError::TraceSummary(error.to_string()))?;
    let Some(routing_summary) = summary.routing_summary.clone() else {
        return Ok(None);
    };

    if !routing_summary.starts_with("routing: compatibility") {
        return Ok(None);
    }

    Ok(Some(CompatibilityFollowUpView {
        follow_up_mode: CompatibilityFollowUpMode::InspectOnly,
        trace_ref: trace_path.to_string_lossy().into_owned(),
        routing_summary,
        execution_condition: output::trace_execution_condition_text(&summary),
        terminal_status: summary.terminal_status,
        terminal_reason: summary.terminal_reason.message.clone(),
        next_command: format!("boundline inspect --workspace {}", workspace.display()),
    }))
}

fn requested_governance_runtime_text(runtime: GovernanceRuntimeKind) -> &'static str {
    match runtime {
        GovernanceRuntimeKind::Local => "local",
        GovernanceRuntimeKind::Canon => "canon",
    }
}

fn review_headline_from_task(task: &crate::domain::task::Task) -> Option<String> {
    let latest_finding = task
        .context
        .state
        .get("latest_review_findings")
        .and_then(Value::as_array)
        .and_then(|findings| findings.last());
    if let Some(finding) = latest_finding {
        let reviewer_id = finding.get("reviewer_id").and_then(Value::as_str).unwrap_or("reviewer");
        let disposition = finding.get("disposition").and_then(Value::as_str).unwrap_or("unknown");
        let summary = finding.get("summary").and_then(Value::as_str).unwrap_or("review finding");
        return Some(format!("{reviewer_id} {disposition}: {summary}"));
    }

    let participants = task
        .context
        .state
        .get("latest_review_participants")
        .and_then(Value::as_array)
        .map(|participants| {
            participants
                .iter()
                .filter_map(|participant| {
                    let reviewer_id = participant.get("reviewer_id").and_then(Value::as_str)?;
                    let status =
                        participant.get("status").and_then(Value::as_str).unwrap_or("unknown");
                    Some(format!("{reviewer_id} {status}"))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if participants.is_empty() {
        None
    } else {
        Some(format!("participants: {}", participants.join(", ")))
    }
}

fn suggested_next_command(record: &ActiveSessionRecord) -> Option<String> {
    if record.authored_brief.as_ref().and_then(|bundle| bundle.clarification.as_ref()).is_some() {
        return Some(repair_capture_command(record));
    }

    let latest_checkpoint_restore_command = record
        .active_task
        .as_ref()
        .and_then(|task| task_state_string(task, "latest_checkpoint_restore_command"));

    if let Some(next_command) = delegation_next_command(record) {
        return Some(next_command);
    }

    if let Some(proof_command) = record
        .active_task
        .as_ref()
        .and_then(|task| task.context.completion_proof_selection().ok().flatten())
        .map(|selection| selection.command_line)
        && record.latest_status == SessionStatus::Blocked
    {
        return Some(proof_command);
    }

    if record.goal_plan.as_ref().and_then(|goal_plan| goal_plan.context_pack.as_ref()).is_some_and(
        |pack| pack.credibility != crate::domain::goal_plan::ContextPackCredibility::Credible,
    ) {
        return Some("boundline goal --goal <narrower goal>".to_string());
    }

    if let Some(governance_state) = record
        .active_task
        .as_ref()
        .and_then(task_state_governance_state_text)
        .or_else(|| planning_governance_state_text(record))
    {
        match governance_state.as_str() {
            "awaiting_approval" => return Some("boundline status".to_string()),
            "blocked" | "failed" => {
                if let Some(restore_command) = latest_checkpoint_restore_command.clone() {
                    return Some(restore_command);
                }
                return Some(repair_planning_command(record));
            }
            _ => {}
        }
    }

    // Refinement-aware suggestion: if refinement is configured for this
    // workspace and plan is complete, suggest the refinement-aware next step.
    if record.latest_status == SessionStatus::Planned
        && let Some(refinement_hint) = refinement_next_hint(record)
    {
        return Some(refinement_hint);
    }

    match record.latest_status {
        SessionStatus::Initialized => Some("boundline goal --goal <goal>".to_string()),
        SessionStatus::GoalCaptured => Some("boundline plan".to_string()),
        SessionStatus::Blocked => Some(repair_planning_command(record)),
        SessionStatus::Planned => {
            if record.goal_plan.is_some() && record.active_task.is_none() {
                return Some("boundline run".to_string());
            }

            Some("boundline step".to_string())
        }
        SessionStatus::Running => Some("boundline step".to_string()),
        SessionStatus::Succeeded
        | SessionStatus::Failed
        | SessionStatus::Exhausted
        | SessionStatus::Aborted => {
            latest_checkpoint_restore_command.or_else(|| Some("boundline inspect".to_string()))
        }
        SessionStatus::Invalid => Some("boundline goal --goal <goal>".to_string()),
    }
}

fn latest_framework_adapter_stage_routing_from_trace(
    trace: &crate::domain::trace::ExecutionTrace,
) -> Option<StageRoutingDecisionRecord> {
    trace.events.iter().rev().find_map(|event| {
        event
            .payload
            .get("framework_adapter_stage_routing")
            .cloned()
            .and_then(|value| serde_json::from_value::<StageRoutingDecisionRecord>(value).ok())
    })
}

fn clarification_reason(record: &ActiveSessionRecord) -> Option<ClarificationReasonKind> {
    record
        .authored_brief
        .as_ref()
        .and_then(|bundle| bundle.clarification.as_ref())
        .map(|clarification| clarification.reason_kind)
}

fn repair_capture_command(record: &ActiveSessionRecord) -> String {
    match clarification_reason(record) {
        Some(ClarificationReasonKind::MissingSource) => {
            "boundline goal --goal <goal> --brief <additional-brief>".to_string()
        }
        Some(
            ClarificationReasonKind::MissingContext
            | ClarificationReasonKind::SourceConflict
            | ClarificationReasonKind::UnsupportedSource
            | ClarificationReasonKind::UnboundedRequest,
        ) => "boundline goal --goal <narrower goal>".to_string(),
        None => first_authored_brief_path(record)
            .map(|path| format!("boundline goal --brief {path}"))
            .unwrap_or_else(|| "boundline goal --goal <narrower goal>".to_string()),
    }
}

fn repair_planning_command(record: &ActiveSessionRecord) -> String {
    if planning_requires_reviewer_route_repair(record) {
        return REVIEWER_ROUTE_REPAIR_COMMAND.to_string();
    }

    if planning_requires_input_repair(record) {
        return repair_capture_command(record);
    }

    "boundline inspect".to_string()
}

fn first_authored_brief_path(record: &ActiveSessionRecord) -> Option<String> {
    record.authored_brief.as_ref().and_then(|bundle| {
        bundle
            .sources
            .iter()
            .filter(|source| {
                !matches!(source.kind, crate::domain::brief::InputSourceKind::DirectText)
            })
            .filter_map(|source| source.workspace_path.clone())
            .next()
    })
}

fn governance_refresh_requires_pause(record: &ActiveSessionRecord) -> bool {
    record
        .active_task
        .as_ref()
        .and_then(task_state_governance_state_text)
        .or_else(|| planning_governance_state_text(record))
        .is_some_and(|state| matches!(state.as_str(), "awaiting_approval" | "blocked" | "failed"))
}

fn latest_planning_governance_record(
    record: &ActiveSessionRecord,
) -> Option<&crate::domain::governance::GovernedStageRecord> {
    record.governance_lifecycle.as_ref().and_then(|lifecycle| {
        lifecycle
            .stage_records
            .iter()
            .rev()
            .find(|stage_record| is_planning_stage_key(&stage_record.stage_key))
    })
}

const REVIEWER_ROUTE_REPAIR_COMMAND: &str = "boundline config set --scope workspace --reviewer reviewer_primary --runtime <runtime-a> --model <model-a> && boundline config set --scope workspace --reviewer reviewer_secondary --runtime <runtime-b> --model <model-b>";
const REVIEWER_ROUTE_REPAIR_KEYWORD: &str = "reviewer route";
const DISCOVERY_INPUT_REPAIR_PREFIX: &str = "repair discovery inputs";
const PLANNING_BRIEF_REPAIR_KEYWORD: &str = "planning brief";
const CLARIFICATION_REPAIR_KEYWORD: &str = "clarification";

fn planning_stage_council_next_action(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record).and_then(|stage_record| {
        stage_record.stage_council.as_ref().map(|outcome| outcome.next_action.clone())
    })
}

fn planning_governance_repair_hint(record: &ActiveSessionRecord) -> Option<String> {
    planning_stage_council_next_action(record)
        .or_else(|| planning_governance_blocked_reason(record))
}

fn planning_requires_reviewer_route_repair(record: &ActiveSessionRecord) -> bool {
    planning_governance_repair_hint(record)
        .is_some_and(|hint| hint.to_ascii_lowercase().contains(REVIEWER_ROUTE_REPAIR_KEYWORD))
}

fn planning_requires_input_repair(record: &ActiveSessionRecord) -> bool {
    planning_governance_repair_hint(record).is_some_and(|hint| {
        let lower = hint.to_ascii_lowercase();
        lower.contains(DISCOVERY_INPUT_REPAIR_PREFIX)
            || lower.contains(PLANNING_BRIEF_REPAIR_KEYWORD)
            || lower.contains(CLARIFICATION_REPAIR_KEYWORD)
    })
}

fn planning_governance_state_text(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record)
        .map(|stage_record| stage_record.lifecycle_state.as_str().to_string())
}

fn planning_governance_stage_key(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record).map(|stage_record| stage_record.stage_key.clone())
}

fn planning_governance_runtime_text(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record)
        .map(|stage_record| format!("{}", stage_record.runtime))
}

fn planning_governance_mode_text(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record).and_then(|stage_record| {
        planning_canon_mode_for_stage_key(&stage_record.stage_key)
            .map(|mode| mode.as_str().to_string())
    })
}

fn planning_governance_run_ref(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record)
        .and_then(|stage_record| stage_record.canon_run_ref.clone())
}

fn planning_governance_blocked_reason(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record)
        .and_then(|stage_record| stage_record.blocked_reason.clone())
        .or_else(|| {
            record
                .governance_lifecycle
                .as_ref()
                .and_then(|lifecycle| lifecycle.terminal_reason.clone())
        })
}

fn planning_governance_packet_ref(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record)
        .and_then(|stage_record| stage_record.packet_ref.clone())
}

fn planning_governance_approval_text(record: &ActiveSessionRecord) -> Option<String> {
    latest_planning_governance_record(record).map(|stage_record| {
        match stage_record.approval_state {
            crate::domain::governance::ApprovalState::NotNeeded => "not_needed".to_string(),
            crate::domain::governance::ApprovalState::Requested => "requested".to_string(),
            crate::domain::governance::ApprovalState::Granted => "granted".to_string(),
            crate::domain::governance::ApprovalState::Rejected => "rejected".to_string(),
            crate::domain::governance::ApprovalState::Expired => "expired".to_string(),
        }
    })
}

fn planning_governance_next_action(record: &ActiveSessionRecord) -> Option<String> {
    if let Some(next_action) = planning_stage_council_next_action(record) {
        return Some(next_action);
    }

    match planning_governance_state_text(record).as_deref() {
        Some("awaiting_approval") => {
            Some("wait for approval and rerun boundline plan".to_string())
        }
        Some("blocked") | Some("failed") => Some(
            planning_governance_repair_hint(record).unwrap_or_else(|| {
                "answer clarification or repair planning brief before rerunning boundline plan; use boundline plan --no-canon only for local-only planning"
                    .to_string()
            }),
        ),
        _ => None,
    }
}

fn next_command_summary(next_command: &str, refreshed: bool) -> String {
    if refreshed {
        format!(
            "refreshed governance approval state; next recommended command for the active session is `{next_command}`"
        )
    } else {
        format!("next recommended command for the active session is `{next_command}`")
    }
}

fn planning_summary(record: &ActiveSessionRecord) -> String {
    let Some(goal_plan) = record.goal_plan.as_ref() else {
        return "planned the active goal into a resumable task snapshot".to_string();
    };

    let task_count = goal_plan.tasks.len();
    if let Some(stage_record) = latest_planning_governance_record(record) {
        match stage_record.lifecycle_state {
            GovernanceLifecycleState::AwaitingApproval => {
                return format!(
                    "planned the active goal into {task_count} bounded goal-plan task(s); Canon planning stage `{}` is awaiting approval before planning can continue",
                    stage_record.stage_key
                );
            }
            GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => {
                let reason = stage_record
                    .blocked_reason
                    .as_deref()
                    .or(record
                        .governance_lifecycle
                        .as_ref()
                        .and_then(|lifecycle| lifecycle.terminal_reason.as_deref()))
                    .unwrap_or("planning governance is blocked");
                return format!(
                    "planned the active goal into {task_count} bounded goal-plan task(s); Canon planning stage `{}` is blocked: {reason}",
                    stage_record.stage_key
                );
            }
            _ => {}
        }
    }

    if goal_plan.requires_confirmation() {
        if let Some(flow) = goal_plan.flow.as_ref() {
            return format!(
                "planned the active goal into {task_count} bounded goal-plan task(s); proposed `{}` flow is ready for execution",
                flow.flow_name
            );
        }

        if goal_plan.flow_skipped {
            return format!(
                "planned the active goal into {task_count} bounded goal-plan task(s) with operator-skipped flow constraints; the proposed plan is ready for execution"
            );
        }

        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s); the proposed plan is ready for execution"
        );
    }

    if let Some(flow) = goal_plan.flow.as_ref()
        && flow.confirmed
    {
        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s) with confirmed `{}` flow",
            flow.flow_name
        );
    }

    if goal_plan.flow_skipped {
        return format!(
            "planned the active goal into {task_count} bounded goal-plan task(s) with operator-skipped flow constraints"
        );
    }

    format!(
        "planned the active goal into {task_count} bounded goal-plan task(s) without flow constraints"
    )
}

fn decision_evidence_basis(decision: &crate::domain::decision::Decision) -> Option<String> {
    let inputs = decision
        .evidence_inputs
        .iter()
        .map(|evidence| {
            let kind = match evidence.kind {
                crate::domain::decision::EvidenceKind::Trace => "trace",
                crate::domain::decision::EvidenceKind::File => "file",
                crate::domain::decision::EvidenceKind::Canon => "canon",
                crate::domain::decision::EvidenceKind::ToolOutput => "tool_output",
            };
            format!("{kind}:{}", evidence.reference)
        })
        .collect::<Vec<_>>();
    (!inputs.is_empty()).then_some(inputs.join(", "))
}

/// Errors surfaced by session-native CLI command handlers.
#[derive(Debug, Error)]
pub enum SessionCommandError {
    #[error("failed to resolve the current workspace: {0}")]
    WorkspaceResolution(#[from] std::io::Error),
    #[error("no active session found for the current workspace")]
    MissingActiveSession,
    #[error("active session is invalid: {0}")]
    InvalidActiveSession(String),
    #[error("active session belongs to a different workspace: expected {expected}, got {actual}")]
    WorkspaceMismatch { expected: String, actual: String },
    #[error("session `{session_id}` does not exist in {}", workspace.display())]
    UnknownSession { session_id: String, workspace: PathBuf },
    #[error("active session has no goal")]
    MissingCapturedGoal,
    #[error("active session has no planned task")]
    MissingPlannedTask,
    #[error("active session has no proposed goal plan")]
    MissingPlanProposal,
    #[error(
        "active session planning governance for `{stage_key}` is `{state}` and must be resolved before confirmation or execution can continue"
    )]
    PlanningGovernanceUnresolved {
        stage_key: String,
        state: GovernanceLifecycleState,
        reason: Option<String>,
    },
    #[error("unknown flow `{requested}`; supported flows: {supported}")]
    UnknownFlow { requested: String, supported: String },
    #[error(
        "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
    )]
    FlowReplacementRequiresReset { current: String, requested: String },
    #[error("active session flow state is invalid: {0}")]
    InvalidFlowState(String),
    #[error("invalid session command request: {0}")]
    InvalidRequest(String),
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session runtime operation failed: {0}")]
    SessionRuntime(#[from] SessionRuntimeError),
    #[error("failed to ingest authored brief: {0}")]
    BriefIngestion(#[from] BriefIngestionError),
    #[error("cluster store operation failed: {0}")]
    ClusterStore(#[from] ClusterStoreError),
    #[error("`{command_name}` requires a valid cluster config in {workspace}")]
    MissingClusterConfig { workspace: PathBuf, command_name: &'static str },
    #[error("failed to summarize the latest compatibility trace: {0}")]
    TraceSummary(String),
    #[error("failed to write session brief artifact {artifact_ref}: {source}")]
    BriefWrite { artifact_ref: String, source: std::io::Error },
    #[error("failed to activate session branch `{branch_ref}` in {}: {source}", repo_root.display())]
    GitBranchCreate { branch_ref: String, repo_root: PathBuf, source: std::io::Error },
    #[error("git refused to activate session branch `{branch_ref}` in {}: {detail}", repo_root.display())]
    GitBranchCreateFailed { branch_ref: String, repo_root: PathBuf, detail: String },
    #[error("{headline}: {prompt}")]
    ClarificationRequired { headline: String, prompt: String },
    #[error("active session cannot be updated in place")]
    GoalUpdateRequiresNewSession { status: SessionStatus },
    #[error("`{command_name}` session workflow is not implemented yet")]
    NotImplemented { command_name: &'static str, next_command: Option<&'static str> },
}

impl SessionCommandError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::MissingActiveSession => {
                "no active session found for the current workspace".to_string()
            }
            Self::InvalidActiveSession(message) => format!("active session is invalid: {message}"),
            Self::WorkspaceMismatch { expected, actual } => {
                format!(
                    "active session belongs to a different workspace: expected {expected}, got {actual}"
                )
            }
            Self::UnknownSession { session_id, workspace } => {
                format!("session `{session_id}` does not exist in {}", workspace.display())
            }
            Self::MissingCapturedGoal => "active session has no goal".to_string(),
            Self::MissingPlannedTask => "active session has no planned task".to_string(),
            Self::MissingPlanProposal => {
                "active session has no proposed goal plan; run `boundline plan` first".to_string()
            }
            Self::PlanningGovernanceUnresolved { stage_key, state, reason } => {
                let reason_suffix =
                    reason.as_deref().map(|reason| format!(": {reason}")).unwrap_or_default();
                format!(
                    "active session planning governance for `{stage_key}` is `{state}` and must be resolved before confirmation or execution can continue{reason_suffix}"
                )
            }
            Self::UnknownFlow { requested, supported } => {
                format!("unknown flow `{requested}`; supported flows: {supported}")
            }
            Self::FlowReplacementRequiresReset { current, requested } => {
                format!(
                    "cannot replace active flow `{current}` with `{requested}` while work is still present; start a new session to reset the flow"
                )
            }
            Self::InvalidFlowState(message) => {
                format!("active session flow state is invalid: {message}")
            }
            Self::InvalidRequest(message) => message.clone(),
            Self::NotImplemented { command_name, .. } => {
                format!("`{command_name}` session workflow is not implemented yet")
            }
            Self::WorkspaceResolution(error) => error.to_string(),
            Self::SessionStore(error) => error.to_string(),
            Self::SessionRuntime(error) => Self::session_runtime_message(error),
            Self::BriefIngestion(error) => format!("failed to ingest authored brief: {error}"),
            Self::ClusterStore(error) => error.to_string(),
            Self::MissingClusterConfig { workspace, command_name } => {
                format!(
                    "`{command_name}` requires a valid cluster config in {}",
                    workspace.display()
                )
            }
            Self::TraceSummary(message) => {
                format!("failed to summarize the latest compatibility trace: {message}")
            }
            Self::BriefWrite { artifact_ref, source } => {
                format!("failed to write session brief artifact {artifact_ref}: {source}")
            }
            Self::GitBranchCreate { branch_ref, repo_root, source } => {
                format!(
                    "failed to activate session branch `{branch_ref}` in {}: {source}",
                    repo_root.display()
                )
            }
            Self::GitBranchCreateFailed { branch_ref, repo_root, detail } => {
                format!(
                    "git refused to activate session branch `{branch_ref}` in {}: {detail}",
                    repo_root.display()
                )
            }
            Self::ClarificationRequired { headline, prompt } => format!("{headline}: {prompt}"),
            Self::GoalUpdateRequiresNewSession { status } => {
                let status_label = format!("{status:?}").to_lowercase();
                format!(
                    "active session status `{status_label}` cannot be updated in place; open a new session with `boundline goal --new --goal <goal>` instead"
                )
            }
        }
    }

    fn session_runtime_message(error: &SessionRuntimeError) -> String {
        match error {
            SessionRuntimeError::FixtureRuntime(
                FixtureRuntimeError::NoSynthesizeableGoalPlanTarget { goal, workspace },
            ) => format!(
                "{DIRECT_RUN_BOUNDED_CONTEXT_HEADLINE}: {DIRECT_RUN_BOUNDED_CONTEXT_REPAIR} for goal '{goal}' in workspace {}",
                workspace.display()
            ),
            _ => error.to_string(),
        }
    }

    fn next_command(&self) -> Option<String> {
        match self {
            Self::MissingActiveSession
            | Self::WorkspaceMismatch { .. }
            | Self::InvalidActiveSession(_) => Some("boundline goal --goal <goal>".to_string()),
            Self::UnknownSession { .. } => Some("boundline session list".to_string()),
            Self::MissingCapturedGoal => Some("boundline goal --goal <goal>".to_string()),
            Self::MissingPlannedTask => Some("boundline plan".to_string()),
            Self::MissingPlanProposal => Some("boundline plan".to_string()),
            Self::PlanningGovernanceUnresolved { state, .. } => match state {
                GovernanceLifecycleState::AwaitingApproval => Some("boundline status".to_string()),
                GovernanceLifecycleState::Blocked | GovernanceLifecycleState::Failed => {
                    Some("boundline plan".to_string())
                }
                _ => Some("boundline plan".to_string()),
            },
            Self::UnknownFlow { .. } => Some("boundline flow bug-fix".to_string()),
            Self::FlowReplacementRequiresReset { .. } => {
                Some("boundline goal --goal <goal>".to_string())
            }
            Self::InvalidFlowState(_) => Some("boundline goal --goal <goal>".to_string()),
            Self::InvalidRequest(_) => None,
            Self::NotImplemented { next_command, .. } => next_command.map(str::to_string),
            Self::ClarificationRequired { .. } => {
                Some("boundline goal --goal <narrower goal>".to_string())
            }
            Self::GoalUpdateRequiresNewSession { .. } => {
                Some("boundline goal --goal <goal>".to_string())
            }
            Self::WorkspaceResolution(_)
            | Self::SessionStore(_)
            | Self::SessionRuntime(_)
            | Self::ClusterStore(_)
            | Self::BriefWrite { .. }
            | Self::GitBranchCreate { .. }
            | Self::GitBranchCreateFailed { .. } => None,
            Self::TraceSummary(_) => None,
            Self::BriefIngestion(_) => Some("boundline goal --goal <goal>".to_string()),
            Self::MissingClusterConfig { .. } => Some("boundline cluster init --workspace <primary> --cluster-id <id> --member <workspace> --member <workspace>".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use serde_json::json;
    use uuid::Uuid;

    use super::{
        CommandExitStatus, GIT_PROGRAM, REVIEWER_ROUTE_REPAIR_COMMAND, SessionCommandError,
        build_status_view, build_status_view_with_follow_up, execute_continue_with_target,
        execute_flow, execute_goal, execute_goal_update, execute_goal_with_target, execute_next,
        execute_next_with_target, execute_plan, execute_plan_with_target_input, execute_run,
        execute_session_list, execute_session_resume, execute_status, execute_status_with_target,
        exit_status_for_session, exit_status_for_task, latest_workspace_compatibility_follow_up,
        load_active_session, map_runtime_error, map_store_error,
        persist_initialized_session_with_goal_hint, planning_governance_next_action,
        refinement_status_summary, render_error, requested_governance_runtime_text,
        resolve_workspace, review_headline_from_task, run_plan_refinement_if_enabled,
        suggested_next_command,
    };
    use crate::adapters::audit_store::{FileSessionAuditStore, FrameworkAdapterHookAuditStore};
    use crate::adapters::checkpoint_store::FileCheckpointStore;
    use crate::adapters::config_store::FileConfigStore;
    use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
    use crate::adapters::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
    use crate::domain::configuration::{
        CapabilityState, ConfigFile, EffortFallbackPolicy, EffortLevel, ModelRoute, RouteSlot,
        RoutingConfig, RuntimeCapabilityProfile, RuntimeKind, SlotEffortPolicy,
    };
    use crate::domain::context_intelligence::{
        AdvancedContextProjection, AuthorityRank, CandidateSelectionState, HybridOutcome,
        ImpactAnalysisFinding, ImpactFindingKind, ImpactFindingSeverity, ImpactFindingStatus,
        RelationshipCredibilityState, RelationshipKind, RelationshipProjection,
        RemoteTransmissionPolicyState, RetrievalCompatibilityState, RetrievalIndexState,
        RetrievalMatchOrigin, RetrievalMode, RetrievalSourceKind, RetrievalStalenessState,
        RetrievalState, RetrievedEvidenceCandidate, SemanticCapabilityState, SemanticPolicyState,
    };
    use crate::domain::decision::{Decision, DecisionType, EvidenceRef};
    use crate::domain::execution::StageRoutingDecisionRecord;
    use crate::domain::framework_adapter::{
        AdapterExecutionSource, AdapterFailureClass, AdapterHookKey, AdapterLifecycleStageKey,
        HookDispatchStatus, LifecycleStageExecutionStatus, StageClaimState,
        StageRoutingDecisionReason,
    };
    use crate::domain::goal_plan::{
        ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan,
        InferredFlow, PlannedTask,
    };
    use crate::domain::governance::{
        ApprovalState, CanonMode, CanonModeSelectionPreference, CompactedCanonMemory,
        GovernanceLifecycleState, GovernanceRuntimeKind, GovernedSessionLifecycle,
        GovernedStageRecord, MemoryCredibilityState,
    };
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{
        ActiveSessionRecord, FrameworkAdapterStageFailureDetails, LifecycleStageExecutionRecord,
        ProjectScaleSessionState, SessionStatus, VotingSessionState, active_session_pointer_ref,
        legacy_session_record_ref, session_branch_ref, session_checkpoints_root_ref,
        session_goal_brief_ref, session_plan_brief_ref, session_record_ref, session_run_brief_ref,
        session_traces_root_ref,
    };
    use crate::domain::task::{Task, TaskStatus, TerminalReason};
    use crate::domain::trace::{ExecutionTrace, HookEventDispatchRecord, TraceEventType};
    use crate::domain::workflow::{
        ProjectScalePath, ProjectScalePathKind, ProjectScaleStage, ProjectScaleStageKind,
    };
    use crate::fixture::{build_fixture_plan_for_goal, build_task_request};
    use crate::orchestrator::session_runtime::{SessionRuntime, SessionRuntimeError};
    use crate::test_support::{CurrentDirGuard, acquire_process_state_lock};

    const FIXTURE_CARGO_TOML: &str = r#"[package]
name = "session_cli_fixture"
version = "0.1.0"
edition = "2024"
"#;

    const RED_LIB_RS: &str = "pub fn add(left: i32, right: i32) -> i32 {\n    left - right\n}\n";

    const FIXTURE_TEST_RS: &str = r#"#[test]
fn red_to_green_addition() {
    assert_eq!(session_cli_fixture::add(2, 2), 4);
}
"#;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn write_execution_workspace(prefix: &str) -> PathBuf {
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
                "name": "session-execution",
                "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
                "validation_command": {
                    "program": "cargo",
                    "args": ["test", "--quiet"]
                },
                "attempts": [
                    {
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
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();
        workspace
    }

    fn sample_framework_adapter_stage_routing(run_id: &str) -> StageRoutingDecisionRecord {
        const ROUTING_RECORDED_AT: u64 = 101;

        StageRoutingDecisionRecord {
            run_id: run_id.to_string(),
            stage_key: AdapterLifecycleStageKey::Run,
            execution_source: AdapterExecutionSource::Adapter,
            decision_reason: StageRoutingDecisionReason::DeclaredOverride,
            claim_state: StageClaimState::Claimed,
            adapter_id: Some("speckit".to_string()),
            stage_status: Some(LifecycleStageExecutionStatus::Blocked),
            produced_artifacts: vec!["artifacts/run-brief.md".to_string()],
            details: None,
            recorded_at: ROUTING_RECORDED_AT,
        }
    }

    fn sample_framework_adapter_stage_failure(run_id: &str) -> FrameworkAdapterStageFailureDetails {
        const EXECUTION_STARTED_AT: u64 = 102;
        const EXECUTION_FINISHED_AT: u64 = 103;

        FrameworkAdapterStageFailureDetails {
            execution: LifecycleStageExecutionRecord {
                run_id: run_id.to_string(),
                stage_key: AdapterLifecycleStageKey::Run,
                execution_source: AdapterExecutionSource::Adapter,
                adapter_id: Some("speckit".to_string()),
                status: LifecycleStageExecutionStatus::Blocked,
                intervention_required: true,
                failure_class: Some(AdapterFailureClass::AdapterRuntime),
                produced_artifacts: vec!["artifacts/run-brief.md".to_string()],
                details: None,
                started_at: Some(EXECUTION_STARTED_AT),
                finished_at: Some(EXECUTION_FINISHED_AT),
            },
            claim_state: StageClaimState::Claimed,
            summary: "adapter blocked the run stage".to_string(),
            detail: Some("operator confirmation is required before retrying".to_string()),
            protocol_error_code: None,
        }
    }

    fn sample_framework_adapter_hook_dispatch(run_id: &str) -> HookEventDispatchRecord {
        const DISPATCH_RECORDED_AT: u64 = 104;

        HookEventDispatchRecord {
            run_id: run_id.to_string(),
            hook_key: AdapterHookKey::StageFailed,
            stage_key: AdapterLifecycleStageKey::Run,
            adapter_id: "speckit".to_string(),
            stage_claimed: true,
            payload_ref: "artifacts/hooks/stage-failed.json".to_string(),
            dispatch_status: HookDispatchStatus::Delivered,
            summary: "hook delivered to the selected adapter".to_string(),
            recorded_at: DISPATCH_RECORDED_AT,
        }
    }

    fn sample_goal_plan(goal: &str) -> GoalPlan {
        GoalPlan::new(
            goal,
            vec![PlannedTask {
                task_id: "planned-task-framework-adapter".to_string(),
                description: "Capture framework-adapter routing evidence".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status surfaces adapter routing details".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
    }

    /// Builds one stable advanced-context projection for status-view tests.
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
                subject_ref: "tests/red_to_green.rs".to_string(),
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

    fn write_context_brief(workspace: &Path) -> PathBuf {
        let brief = workspace.join("brief.md");
        fs::write(
            &brief,
            "Investigate src/lib.rs and tests/red_to_green.rs before broad scanning.\n",
        )
        .unwrap();
        brief
    }

    fn initialize_git_repo(workspace: &Path) {
        let init = Command::new(GIT_PROGRAM)
            .current_dir(workspace)
            .arg("init")
            .arg("--quiet")
            .output()
            .unwrap();
        assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    }

    fn current_git_branch(workspace: &Path) -> String {
        let output = Command::new(GIT_PROGRAM)
            .current_dir(workspace)
            .arg("symbolic-ref")
            .arg("--quiet")
            .arg("--short")
            .arg("HEAD")
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    fn write_review_execution_workspace(prefix: &str) -> PathBuf {
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
                "name": "session-review-execution",
                "read_targets": ["src/lib.rs", "tests/red_to_green.rs"],
                "validation_command": {
                    "program": "cargo",
                    "args": ["test", "--quiet"]
                },
                "attempts": [
                    {
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
                    }
                ],
                "review": {
                    "triggers": ["pr_ready"],
                    "reviewers": [
                        {
                            "reviewer_id": "safety",
                            "role": "Safety",
                            "source": "gpt",
                            "weight": 1
                        },
                        {
                            "reviewer_id": "maintainability",
                            "role": "Maintainability",
                            "source": "claude",
                            "weight": 1
                        }
                    ],
                    "vote_rule": {
                        "strategy": "majority"
                    },
                    "scenarios": [
                        {
                            "trigger": "pr_ready",
                            "findings": [
                                {
                                    "reviewer_id": "safety",
                                    "disposition": "approve",
                                    "summary": "No blockers"
                                },
                                {
                                    "reviewer_id": "maintainability",
                                    "disposition": "approve",
                                    "summary": "Ready to ship"
                                }
                            ]
                        }
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();
        let config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                reviewer_roles: std::collections::BTreeMap::from([
                    (
                        "safety".to_string(),
                        ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4.1".to_string() },
                    ),
                    (
                        "maintainability".to_string(),
                        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
                    ),
                ]),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();
        workspace
    }

    fn seed_fixture_planned_session(workspace: &Path, flow_name: &str) {
        let canonical_workspace = workspace.canonicalize().unwrap();
        let runtime = SessionRuntime::for_workspace(&canonical_workspace);
        let mut record = load_active_session(&canonical_workspace).unwrap();
        runtime.select_flow(&mut record, flow_name).unwrap();

        let request = build_task_request(
            &canonical_workspace,
            record.goal.clone().unwrap_or_default(),
            record.session_id.clone(),
            record.authored_brief.as_ref(),
            record.negotiation_packet.as_ref(),
        )
        .unwrap();
        let plan = build_fixture_plan_for_goal(
            &canonical_workspace,
            record.active_flow.as_ref(),
            record.goal.as_deref().unwrap_or_default(),
        )
        .unwrap();

        record.active_task = Some(Task::new("task-session-cli", &request, plan).unwrap());
        record.goal_plan = None;
        record.active_flow_policy = None;
        record.latest_status = SessionStatus::Planned;
        runtime.persist_session(&record).unwrap();
    }

    #[test]
    fn resolve_workspace_and_status_helpers_cover_remaining_branches() {
        let workspace = temp_workspace("boundline-cli-session-resolve");
        let child = workspace.join("child");
        fs::create_dir_all(&child).unwrap();

        let _current_dir_guard = CurrentDirGuard::change_to(&workspace);
        let resolved = resolve_workspace(Some(Path::new("child"))).unwrap();

        assert_eq!(resolved, child.canonicalize().unwrap());
        assert_eq!(exit_status_for_session(SessionStatus::Invalid), CommandExitStatus::NonSuccess);
        assert_eq!(exit_status_for_task(TaskStatus::Failed), CommandExitStatus::NonSuccess);
        assert_eq!(
            suggested_next_command(&crate::domain::session::ActiveSessionRecord {
                session_id: "session".to_string(),
                workspace_ref: "/tmp/workspace".to_string(),
                goal: None,
                authored_brief: None,
                negotiation_packet: None,
                active_flow: None,
                active_task: None,
                goal_plan: None,
                workflow_progress: None,
                decisions: Vec::new(),
                active_flow_policy: None,
                latest_status: SessionStatus::Invalid,
                latest_terminal_reason: None,
                latest_trace_ref: None,
                created_at: 1,
                updated_at: 1,
                governance_lifecycle: None,
                project_scale: None,
                latest_voting: None,
                delight_feedback: None,
            }),
            Some("boundline goal --goal <goal>".to_string())
        );
    }

    #[test]
    fn store_and_runtime_error_mapping_cover_translated_variants() {
        assert!(matches!(
            map_store_error(SessionStoreError::InvalidRecord("bad session".to_string())),
            SessionCommandError::InvalidActiveSession(message) if message == "bad session"
        ));
        assert!(matches!(
            map_store_error(SessionStoreError::Read(std::io::Error::other("read failed"))),
            SessionCommandError::SessionStore(_)
        ));

        assert!(matches!(
            map_runtime_error(SessionRuntimeError::MissingGoal),
            SessionCommandError::MissingCapturedGoal
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::MissingActiveTask),
            SessionCommandError::MissingPlannedTask
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::UnknownFlow {
                requested: "missing".to_string(),
                supported: "bug-fix".to_string(),
            }),
            SessionCommandError::UnknownFlow { .. }
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::FlowReplacementRequiresReset {
                current: "bug-fix".to_string(),
                requested: "delivery".to_string(),
            }),
            SessionCommandError::FlowReplacementRequiresReset { .. }
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::InvalidFlowState("bad flow".to_string())),
            SessionCommandError::InvalidFlowState(message) if message == "bad flow"
        ));
        assert!(matches!(
            map_runtime_error(SessionRuntimeError::TraceStore(TraceStoreError::Read(
                std::io::Error::other("trace read failed")
            ))),
            SessionCommandError::SessionRuntime(_)
        ));
    }

    #[test]
    fn session_command_error_helpers_cover_messages_and_next_commands() {
        let unknown_flow = SessionCommandError::UnknownFlow {
            requested: "missing".to_string(),
            supported: "bug-fix, change, delivery".to_string(),
        };
        let text = render_error("flow", &unknown_flow);
        assert!(text.contains("boundline flow bug-fix"), "{text}");

        let reset_required = SessionCommandError::FlowReplacementRequiresReset {
            current: "bug-fix".to_string(),
            requested: "delivery".to_string(),
        };
        let text = render_error("flow", &reset_required);
        assert!(text.contains("boundline goal --goal <goal>"), "{text}");

        let not_implemented = SessionCommandError::NotImplemented {
            command_name: "next",
            next_command: Some("boundline inspect"),
        };
        let text = render_error("next", &not_implemented);
        assert!(text.contains("boundline inspect"), "{text}");

        let runtime_error =
            SessionCommandError::SessionRuntime(SessionRuntimeError::MissingTraceReference);
        let text = render_error("run", &runtime_error);
        assert!(!text.contains("next_command:"), "{text}");
    }

    #[test]
    fn clustered_session_commands_resolve_the_primary_workspace_explicitly() {
        let primary = write_execution_workspace("boundline-cli-session-cluster-primary");
        let secondary = write_execution_workspace("boundline-cli-session-cluster-secondary");
        crate::cli::cluster::execute_init(
            &primary,
            "cluster-1",
            &[primary.clone(), secondary.clone()],
        )
        .unwrap();

        let report = execute_goal_with_target(
            None,
            Some(&primary),
            Some("Bootstrap clustered delivery"),
            &[],
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            report.terminal_output.contains("current clustered delivery session"),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn execute_run_status_and_next_cover_success_paths() {
        let _process_state_lock = acquire_process_state_lock();
        let workspace = write_execution_workspace("boundline-cli-session-success");
        let brief = write_context_brief(&workspace);

        let goal = execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(goal.exit_status, CommandExitStatus::Succeeded);
        let session_id = goal.session_status.as_ref().unwrap().session_id.clone();
        let goal_brief = workspace.join(session_goal_brief_ref(&session_id));
        assert!(goal_brief.is_file(), "{}", goal_brief.display());
        let goal_brief_text = fs::read_to_string(&goal_brief).unwrap();
        assert!(goal_brief_text.contains("# Goal Brief"), "{goal_brief_text}");
        assert!(goal_brief_text.contains("- goal: Fix the failing add test"), "{goal_brief_text}");

        assert_eq!(
            execute_plan(Some(&workspace), None, false).unwrap().exit_status,
            CommandExitStatus::Succeeded
        );

        let planned = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
        assert!(
            planned.terminal_output.contains("confirmed `bug-fix` flow"),
            "{}",
            planned.terminal_output
        );
        let plan_brief = workspace.join(session_plan_brief_ref(&session_id));
        assert!(plan_brief.is_file(), "{}", plan_brief.display());
        let plan_brief_text = fs::read_to_string(&plan_brief).unwrap();
        assert!(plan_brief_text.contains("# Plan Brief"), "{plan_brief_text}");
        assert!(plan_brief_text.contains("- latest_status: planned"), "{plan_brief_text}");

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(
            run.terminal_output.contains("terminal_status: succeeded"),
            "{}",
            run.terminal_output
        );
        assert!(run.terminal_output.contains("decision "), "{}", run.terminal_output);
        assert!(
            run.terminal_output
                .contains(&format!("run_brief_ref: {}", session_run_brief_ref(&session_id))),
            "{}",
            run.terminal_output
        );
        let run_brief = workspace.join(session_run_brief_ref(&session_id));
        assert!(run_brief.is_file(), "{}", run_brief.display());
        let run_brief_text = fs::read_to_string(&run_brief).unwrap();
        assert!(run_brief_text.contains("# Run Brief"), "{run_brief_text}");
        assert!(run_brief_text.contains("- latest_status: succeeded"), "{run_brief_text}");

        let status = execute_status(Some(&workspace)).unwrap();
        assert_eq!(status.exit_status, CommandExitStatus::Succeeded);
        assert!(
            status.terminal_output.contains("latest_status: succeeded"),
            "{}",
            status.terminal_output
        );

        let next = execute_next(Some(&workspace)).unwrap();
        assert_eq!(next.exit_status, CommandExitStatus::Succeeded);
        assert!(
            next.terminal_output.contains("next_command: boundline checkpoint restore"),
            "{}",
            next.terminal_output
        );

        let inspect =
            crate::cli::inspect::execute_inspect(None, Some(&workspace), None, false).unwrap();
        assert!(
            inspect
                .terminal_output
                .contains(&format!("goal_brief_ref: {}", session_goal_brief_ref(&session_id))),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains(&format!(
                "session_plan_brief_ref: {}",
                session_plan_brief_ref(&session_id)
            )),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect
                .terminal_output
                .contains(&format!("run_brief_ref: {}", session_run_brief_ref(&session_id))),
            "{}",
            inspect.terminal_output
        );
    }

    #[test]
    fn selected_session_status_next_and_continue_preserve_active_pointer() -> Result<(), String> {
        let workspace = write_execution_workspace("boundline-cli-session-selected-status");
        let canonical_workspace = workspace.canonicalize().map_err(|error| error.to_string())?;
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        let first_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;

        execute_goal(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        let second_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;

        let status =
            execute_status_with_target(Some(&workspace), None, Some(&first_record.session_id))
                .map_err(|error| error.to_string())?;
        let status_view =
            status.session_status.ok_or_else(|| "expected status session view".to_string())?;
        if status_view.session_id != first_record.session_id {
            return Err(format!(
                "expected selected status session {}, got {}",
                first_record.session_id, status_view.session_id
            ));
        }

        let next = execute_next_with_target(Some(&workspace), None, Some(&first_record.session_id))
            .map_err(|error| error.to_string())?;
        let next_view =
            next.session_status.ok_or_else(|| "expected next session view".to_string())?;
        if next_view.session_id != first_record.session_id {
            return Err(format!(
                "expected selected next session {}, got {}",
                first_record.session_id, next_view.session_id
            ));
        }

        let cont =
            execute_continue_with_target(Some(&workspace), None, Some(&first_record.session_id))
                .map_err(|error| error.to_string())?;
        let continue_view =
            cont.session_status.ok_or_else(|| "expected continue session view".to_string())?;
        if continue_view.session_id != first_record.session_id {
            return Err(format!(
                "expected selected continue session {}, got {}",
                first_record.session_id, continue_view.session_id
            ));
        }

        let active_after =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;
        if active_after.session_id != second_record.session_id {
            return Err(format!(
                "expected active session {} to remain selected, got {}",
                second_record.session_id, active_after.session_id
            ));
        }

        Ok(())
    }

    #[test]
    fn execute_status_refreshes_completion_verification_for_selected_sessions() -> Result<(), String>
    {
        let workspace = write_execution_workspace("boundline-cli-session-completion-refresh");

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            &[],
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        execute_plan(Some(&workspace), Some("bug-fix"), false)
            .map_err(|error| error.to_string())?;
        let run = execute_run(Some(&workspace)).map_err(|error| error.to_string())?;
        if run.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected successful run before staleness refresh, got {:?}",
                run.exit_status
            ));
        }

        let selected_record = load_active_session(&workspace).map_err(|error| error.to_string())?;
        fs::write(
            workspace.join("src/lib.rs"),
            "pub fn add(left: i32, right: i32) -> i32 {\n    left + right + 1\n}\n",
        )
        .map_err(|error| error.to_string())?;

        let status =
            execute_status_with_target(Some(&workspace), None, Some(&selected_record.session_id))
                .map_err(|error| error.to_string())?;
        let output = status.terminal_output;
        if !output.contains("explanation: refreshed completion verification state for the selected workspace session") {
            return Err(output);
        }
        if !output.contains("completion_verification_state: proof_required") {
            return Err(output);
        }
        if !output.contains("stale_proof") {
            return Err(output);
        }

        Ok(())
    }

    #[test]
    fn execute_goal_records_explicit_governance_selection() {
        let workspace = write_execution_workspace("boundline-cli-session-capture-governance");

        let report = execute_goal(
            Some(&workspace),
            Some("Plan a governed delivery flow"),
            &[],
            Some(GovernanceRuntimeKind::Canon),
            Some("medium"),
            Some("engineering"),
            Some("platform"),
        )
        .unwrap();

        let view = report.session_status.expect("capture should return session status");
        assert_eq!(view.governance_lifecycle_runtime.as_deref(), Some("canon"));
        assert_eq!(view.governance_lifecycle_mode_selection.as_deref(), Some("auto-confirm"));
        assert_eq!(view.governance_lifecycle_selected_mode, None);
    }

    #[test]
    fn execute_goal_auto_starts_missing_session() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-autostart");
        let brief = write_context_brief(&workspace);

        let report = execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);

        let view = report.session_status.expect("goal should return session status");
        assert_eq!(view.latest_status, SessionStatus::GoalCaptured);
        assert!(
            view.goal.as_deref().unwrap_or_default().contains("Fix the failing add test"),
            "{:?}",
            view.goal
        );
        assert_eq!(
            view.authored_input_summary.as_deref(),
            Some("direct_text + 1 markdown source(s)")
        );
    }

    #[test]
    fn execute_goal_update_reuses_active_session() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-updates-session");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let first_record = load_active_session(&canonical_workspace).unwrap();

        execute_goal_update(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let second_record = load_active_session(&canonical_workspace).unwrap();

        assert_eq!(first_record.session_id, second_record.session_id);
        assert_eq!(
            second_record.goal.as_deref(),
            Some(
                "Ship the follow-up cleanup\n\n## brief.md\nInvestigate src/lib.rs and tests/red_to_green.rs before broad scanning."
            )
        );
        assert!(canonical_workspace.join(session_record_ref(&second_record.session_id)).is_file());
    }

    #[test]
    fn execute_goal_creates_new_session_when_existing_session_is_goal_captured() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-new-after-goal");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let first_record = load_active_session(&canonical_workspace).unwrap();

        execute_goal(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let new_record = load_active_session(&canonical_workspace).unwrap();

        assert_ne!(first_record.session_id, new_record.session_id);
        assert_eq!(new_record.latest_status, SessionStatus::GoalCaptured);
        assert!(canonical_workspace.join(session_record_ref(&first_record.session_id)).is_file());
        assert!(canonical_workspace.join(session_record_ref(&new_record.session_id)).is_file());
    }

    #[test]
    fn execute_goal_update_reuses_planned_session() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-update-planned");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        execute_plan(Some(&workspace), None, false).unwrap();
        let planned_record = load_active_session(&canonical_workspace).unwrap();
        assert_eq!(planned_record.latest_status, SessionStatus::Planned);

        execute_goal_update(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let updated_record = load_active_session(&canonical_workspace).unwrap();

        assert_eq!(planned_record.session_id, updated_record.session_id);
        assert_eq!(updated_record.latest_status, SessionStatus::GoalCaptured);
        assert_eq!(
            updated_record.goal.as_deref(),
            Some(
                "Ship the follow-up cleanup\n\n## brief.md\nInvestigate src/lib.rs and tests/red_to_green.rs before broad scanning."
            )
        );
    }

    #[test]
    fn execute_goal_switches_to_session_branch_when_git_repo_exists() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-branch");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);
        initialize_git_repo(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let record = load_active_session(&canonical_workspace).unwrap();
        assert_eq!(
            current_git_branch(&canonical_workspace),
            session_branch_ref(&record.session_id)
        );
    }

    #[test]
    fn session_history_list_reports_active_and_historical_sessions() -> Result<(), String> {
        let workspace = write_execution_workspace("boundline-cli-session-history-list");
        let canonical_workspace = workspace.canonicalize().map_err(|error| error.to_string())?;
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        let first_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;

        execute_goal(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        let second_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;

        let report = execute_session_list(Some(&workspace)).map_err(|error| error.to_string())?;
        if report.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected session list to succeed, got {:?}: {}",
                report.exit_status, report.terminal_output
            ));
        }
        if !report.terminal_output.contains("session_history:") {
            return Err(format!("expected session history header, got {}", report.terminal_output));
        }
        if !report
            .terminal_output
            .contains(&format!("active_session: {}", second_record.session_id))
        {
            return Err(format!("expected active session in list, got {}", report.terminal_output));
        }
        if !report
            .terminal_output
            .contains(&format!("branch: {}", session_branch_ref(&second_record.session_id)))
        {
            return Err(format!("expected active branch in list, got {}", report.terminal_output));
        }
        let second_position = report
            .terminal_output
            .find(&second_record.session_id)
            .ok_or_else(|| format!("missing second session {}", second_record.session_id))?;
        let first_position = report
            .terminal_output
            .find(&first_record.session_id)
            .ok_or_else(|| format!("missing first session {}", first_record.session_id))?;
        if second_position >= first_position {
            return Err(format!("expected newest session first, got {}", report.terminal_output));
        }

        Ok(())
    }

    #[test]
    fn session_history_resume_reactivates_selected_session_and_switches_branch()
    -> Result<(), String> {
        let workspace = write_execution_workspace("boundline-cli-session-history-resume");
        let canonical_workspace = workspace.canonicalize().map_err(|error| error.to_string())?;
        let brief = write_context_brief(&workspace);
        initialize_git_repo(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;
        let first_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;

        execute_goal(
            Some(&workspace),
            Some("Ship the follow-up cleanup"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .map_err(|error| error.to_string())?;

        let report = execute_session_resume(Some(&workspace), &first_record.session_id)
            .map_err(|error| error.to_string())?;
        if report.exit_status != CommandExitStatus::Succeeded {
            return Err(format!(
                "expected session resume to succeed, got {:?}: {}",
                report.exit_status, report.terminal_output
            ));
        }
        let view = match report.session_status {
            Some(view) => view,
            None => {
                return Err(format!(
                    "expected session resume to return status view, got {}",
                    report.terminal_output
                ));
            }
        };
        if view.session_id != first_record.session_id {
            return Err(format!(
                "expected resumed session {}, got {}",
                first_record.session_id, view.session_id
            ));
        }

        let resumed_record =
            load_active_session(&canonical_workspace).map_err(|error| error.to_string())?;
        if resumed_record.session_id != first_record.session_id {
            return Err(format!(
                "expected active session {} after resume, got {}",
                first_record.session_id, resumed_record.session_id
            ));
        }
        if current_git_branch(&canonical_workspace) != session_branch_ref(&first_record.session_id)
        {
            return Err(format!(
                "expected git branch {}, got {}",
                session_branch_ref(&first_record.session_id),
                current_git_branch(&canonical_workspace)
            ));
        }

        Ok(())
    }

    #[test]
    fn session_history_resume_rejects_unknown_session_id() -> Result<(), String> {
        let workspace = write_execution_workspace("boundline-cli-session-history-missing");

        let error = execute_session_resume(Some(&workspace), "missing-session")
            .err()
            .ok_or_else(|| "expected session resume to fail for missing history".to_string())?;
        match error {
            SessionCommandError::UnknownSession { session_id, .. } => {
                if session_id != "missing-session" {
                    return Err(format!("expected missing-session error, got {session_id}"));
                }
            }
            other => {
                return Err(format!("expected unknown-session error, got {other:?}"));
            }
        }

        Ok(())
    }

    #[test]
    fn execute_run_persists_trace_and_checkpoint_under_active_session_root() {
        let workspace = write_execution_workspace("boundline-cli-session-run-session-root");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();
        execute_run(Some(&workspace)).unwrap();

        let record = load_active_session(&canonical_workspace).unwrap();
        let trace_ref = record.latest_trace_ref.clone().unwrap();
        assert!(trace_ref.contains(&session_traces_root_ref(&record.session_id)), "{trace_ref}");
        assert!(Path::new(&trace_ref).is_file(), "{trace_ref}");

        let checkpoint_store =
            FileCheckpointStore::for_session(&canonical_workspace, &record.session_id);
        let manifests = checkpoint_store.list().unwrap();
        assert!(!manifests.is_empty(), "expected at least one checkpoint manifest");
        let checkpoint_path = canonical_workspace
            .join(session_checkpoints_root_ref(&record.session_id))
            .join(format!("{}.json", manifests[0].checkpoint_id));
        assert!(checkpoint_path.is_file(), "{}", checkpoint_path.display());
    }

    #[test]
    fn execute_goal_autostart_persists_active_session_under_session_root() {
        let workspace = write_execution_workspace("boundline-cli-session-goal-session-root");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = write_context_brief(&workspace);

        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let record = load_active_session(&canonical_workspace).unwrap();
        let session_path = canonical_workspace.join(session_record_ref(&record.session_id));
        let pointer_path = canonical_workspace.join(active_session_pointer_ref());
        let legacy_path = canonical_workspace.join(legacy_session_record_ref());

        assert!(session_path.is_file(), "{}", session_path.display());
        assert!(legacy_path.is_file(), "{}", legacy_path.display());
        assert_eq!(fs::read_to_string(pointer_path).unwrap().trim(), record.session_id);

        let status = execute_status(Some(&canonical_workspace)).unwrap();
        let view = status.session_status.expect("status should expose session status");
        assert_eq!(view.session_id, record.session_id);
        assert_eq!(view.latest_status, SessionStatus::GoalCaptured);
    }

    #[test]
    fn execute_plan_with_target_input_preserves_existing_goal() {
        let workspace = write_execution_workspace("boundline-cli-session-plan-input");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let plan_input = workspace.join("plan-input.md");

        fs::write(
            &plan_input,
            "# Rust service brief\n\nImplement a rust microservice for user management.\n\n- Scope boundary: the first slice only supports create and read user flows.\n- API surface: create and read users over HTTP.\n- Persistence choice: SQLite is authoritative for the first slice.\n- Authentication boundary: OAuth2 bearer validation stops at the edge; service authorization begins in the application service layer.\n- Success criteria: operators can create and read users over HTTP in the first slice.\n- Validation target: cargo test.\n",
        )
        .unwrap();

        execute_goal(Some(&workspace), Some("placeholder goal"), &[], None, None, None, None)
            .unwrap();

        let report = execute_plan_with_target_input(
            Some(&workspace),
            None,
            None,
            false,
            false,
            Some(plan_input.as_path()),
            false,
            false,
            None,
        )
        .unwrap();

        let view = report.session_status.expect("plan should return session status");
        let record = load_active_session(&canonical_workspace).unwrap();
        assert_eq!(
            view.authored_input_sources,
            Some(vec![
                "direct_text: developer goal".to_string(),
                "attached_markdown: plan-input.md".to_string(),
            ])
        );
        assert_eq!(
            view.authored_input_summary.as_deref(),
            Some("direct_text + 1 markdown source(s)")
        );
        assert_eq!(view.goal.as_deref(), Some("placeholder goal"), "{}", report.terminal_output);
        assert_eq!(record.goal.as_deref(), Some("placeholder goal"));
        assert_eq!(
            record.authored_brief.as_ref().and_then(|bundle| bundle.primary_goal_text.as_deref()),
            Some("placeholder goal")
        );
    }

    #[test]
    fn execute_plan_with_target_input_requires_captured_goal() {
        let workspace = write_execution_workspace("boundline-cli-session-plan-input-missing-goal");
        let plan_input = workspace.join("plan-input.md");

        persist_initialized_session_with_goal_hint(&workspace, None, None).unwrap();

        fs::write(
            &plan_input,
            "# Rust service brief\n\nImplement a rust microservice for user management.\n",
        )
        .unwrap();

        let error = execute_plan_with_target_input(
            Some(&workspace),
            None,
            None,
            false,
            false,
            Some(plan_input.as_path()),
            false,
            false,
            None,
        )
        .unwrap_err();

        assert!(matches!(error, SessionCommandError::MissingCapturedGoal));
    }

    #[test]
    fn execute_goal_uses_brief_content_for_session_slug_when_direct_goal_is_absent() {
        let workspace = write_execution_workspace("boundline-cli-session-brief-slug");
        let canonical_workspace = workspace.canonicalize().unwrap();
        let brief = workspace.join("plan.md");

        fs::write(
            &brief,
            "# Rust service brief\n\nImplement a rust microservice for user management.\n",
        )
        .unwrap();

        execute_goal(Some(&workspace), None, std::slice::from_ref(&brief), None, None, None, None)
            .unwrap();

        let record = load_active_session(&canonical_workspace).unwrap();
        assert!(record.session_id.ends_with("rust-service-brief"), "{}", record.session_id);
        assert!(!record.session_id.ends_with("plan-md"), "{}", record.session_id);
    }

    #[test]
    fn execute_run_surfaces_delegation_packet_when_native_route_is_blocked() {
        let workspace = write_execution_workspace("boundline-cli-session-delegation");
        let brief = write_context_brief(&workspace);
        let mut config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                implementation: Some(ModelRoute {
                    runtime: RuntimeKind::Claude,
                    model: "sonnet-4".to_string(),
                }),
                assistant_runtimes: vec![RuntimeKind::Codex],
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
            capability_provider: None,
        };
        config.routing.slot_effort_policies.insert(
            RouteSlot::Implementation,
            SlotEffortPolicy {
                level: EffortLevel::High,
                fallback: EffortFallbackPolicy::Preserve,
                rationale: Some(
                    "keep implementation continuation on the highest-effort path".to_string(),
                ),
            },
        );
        config.routing.runtime_capabilities.insert(
            RuntimeKind::Claude,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Unsupported,
                resume: CapabilityState::Unsupported,
                validation: CapabilityState::Unsupported,
                handoff_target: CapabilityState::Unsupported,
                escalation_context: CapabilityState::Supported,
                notes: Some("requires a handoff for bounded continuation".to_string()),
            },
        );
        config.routing.runtime_capabilities.insert(
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
        FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();

        execute_goal(
            Some(&workspace),
            Some("fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let plan = execute_plan(Some(&workspace), Some("bug-fix"), false).unwrap();

        assert!(plan.terminal_output.contains("runtime_capabilities:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("slot_effort_policies:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("planning_rationale:"), "{}", plan.terminal_output);
        assert!(plan.terminal_output.contains("routing policy:"), "{}", plan.terminal_output);

        let run = execute_run(Some(&workspace)).unwrap();

        assert_eq!(run.exit_status, CommandExitStatus::NonSuccess);
        assert!(
            run.terminal_output.contains("delegation_mode: handoff_required"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("delegation_packet_kind: handoff"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("delegation_target_owner: codex"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("next_command: boundline status"),
            "{}",
            run.terminal_output
        );
    }

    #[test]
    fn execute_run_status_and_inspect_surface_review_evidence() {
        let workspace = write_review_execution_workspace("boundline-cli-session-review-success");
        let brief = write_context_brief(&workspace);

        assert_eq!(
            execute_goal(
                Some(&workspace),
                Some("Fix the failing add test and review it"),
                std::slice::from_ref(&brief),
                None,
                None,
                None,
                None,
            )
            .unwrap()
            .exit_status,
            CommandExitStatus::Succeeded
        );
        assert_eq!(
            execute_flow(Some(&workspace), "bug-fix").unwrap().exit_status,
            CommandExitStatus::Succeeded
        );

        seed_fixture_planned_session(&workspace, "bug-fix");

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
        assert!(
            run.terminal_output.contains("review_trigger: pr_ready"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("reviewer safety (Safety) approve: No blockers"),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains(
                "review_vote: strategy=Majority approvals=2 concerns=0 blocks=0 decision=Accepted"
            ),
            "{}",
            run.terminal_output
        );
        assert!(
            run.terminal_output.contains("review_outcome: accepted"),
            "{}",
            run.terminal_output
        );

        let status = execute_status(Some(&workspace)).unwrap();
        assert!(
            status.terminal_output.contains("latest_review_trigger: pr_ready"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("latest_review_outcome: accepted"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("latest_review_council_profile: yellow_pair"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("latest_review_independence_state: passed"),
            "{}",
            status.terminal_output
        );
        assert!(
            status.terminal_output.contains("latest_review_stop_semantics: council_required"),
            "{}",
            status.terminal_output
        );
        assert!(
            status
                .terminal_output
                .contains("latest_review_headline: maintainability approve: Ready to ship"),
            "{}",
            status.terminal_output
        );

        let inspect =
            crate::cli::inspect::execute_inspect(None, Some(&workspace), None, false).unwrap();
        assert!(
            inspect.terminal_output.contains("review_trigger: pr_ready"),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains(
                "review_vote: strategy=Majority approvals=2 concerns=0 blocks=0 decision=Accepted"
            ),
            "{}",
            inspect.terminal_output
        );
        assert!(
            inspect.terminal_output.contains("review_outcome: accepted"),
            "{}",
            inspect.terminal_output
        );
    }

    #[test]
    fn execute_run_succeeds_after_plan_without_explicit_confirmation() {
        let workspace = write_execution_workspace("boundline-cli-session-flow-confirmation");
        let brief = write_context_brief(&workspace);

        persist_initialized_session_with_goal_hint(&workspace, None, None).unwrap();
        execute_goal(
            Some(&workspace),
            Some("Fix the failing add test"),
            std::slice::from_ref(&brief),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        execute_plan(Some(&workspace), None, false).unwrap();

        let run = execute_run(Some(&workspace)).unwrap();
        assert_eq!(run.exit_status, CommandExitStatus::Succeeded);
    }

    #[test]
    fn execute_run_blocks_on_unresolved_planning_governance() {
        let workspace = write_execution_workspace("boundline-cli-session-plan-governance-gate");
        let canonical_workspace = fs::canonicalize(&workspace).unwrap();

        let draft_goal_plan = GoalPlan::new(
            "Prepare governed feature",
            vec![PlannedTask {
                task_id: "planned-task-plan-governance".to_string(),
                description: "Prepare governed plan".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("planning governance clears".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_planning_rationale("governed planning evidence is assembled for the next stage")
        .with_verification_strategy("rerun plan or status after Canon approval resolves");

        let draft_record = ActiveSessionRecord {
            session_id: "session-plan-governance-confirm".to_string(),
            workspace_ref: canonical_workspace.to_string_lossy().into_owned(),
            goal: Some("Prepare governed feature".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(draft_goal_plan),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Planned,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 2,
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
                    blocked_reason: Some("waiting for Canon approval".to_string()),
                    stage_council: None,
                }],
                accumulated_context: Vec::new(),
                terminal_reason: Some("awaiting approval: waiting for Canon approval".to_string()),
                planning_input_fingerprint: None,
            }),
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };
        FileSessionStore::for_workspace(&workspace).persist(&draft_record).unwrap();

        let run_error = execute_run(Some(&workspace)).unwrap_err();
        assert!(matches!(run_error, SessionCommandError::PlanningGovernanceUnresolved { .. }));
        let rendered_run = render_error("run", &run_error);
        assert!(rendered_run.contains("plan:requirements"), "{rendered_run}");
        assert!(rendered_run.contains("boundline status"), "{rendered_run}");
    }

    #[test]
    fn compatibility_follow_up_and_review_headline_helpers_cover_remaining_session_cli_branches() {
        let workspace = temp_workspace("boundline-cli-session-compat-follow-up");
        fs::create_dir_all(workspace.join(".boundline")).unwrap();

        let mut trace = ExecutionTrace::new("task-compat", "session-compat", "Compat trace");
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason = Some(TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "compatibility run failed",
            None,
        ));
        trace.ended_at = Some(trace.started_at + 1);
        let trace_path = FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();

        let follow_up =
            latest_workspace_compatibility_follow_up(&workspace, None).unwrap().unwrap();
        assert_eq!(follow_up.trace_ref, trace_path.to_string_lossy());
        assert!(follow_up.routing_summary.starts_with("routing: compatibility"));
        assert_eq!(
            follow_up.next_command,
            format!("boundline inspect --workspace {}", workspace.display())
        );
        assert!(
            latest_workspace_compatibility_follow_up(&workspace, Some(&follow_up.trace_ref))
                .unwrap()
                .is_none()
        );

        let execution_workspace =
            write_execution_workspace("boundline-cli-session-review-headline");
        let request = build_task_request(
            &execution_workspace,
            "Fix the failing add test".to_string(),
            "session-review".to_string(),
            None,
            None,
        )
        .unwrap();
        let plan =
            build_fixture_plan_for_goal(&execution_workspace, None, "Fix the failing add test")
                .unwrap();
        let mut task = Task::new("task-review", &request, plan).unwrap();
        task.context.state.insert(
            "latest_review_participants".to_string(),
            json!([
                {"reviewer_id": "safety", "status": "pending"},
                {"reviewer_id": "maintainability"}
            ]),
        );
        assert_eq!(
            review_headline_from_task(&task),
            Some("participants: safety pending, maintainability unknown".to_string())
        );

        assert_eq!(requested_governance_runtime_text(GovernanceRuntimeKind::Local), "local");
        assert_eq!(requested_governance_runtime_text(GovernanceRuntimeKind::Canon), "canon");
    }

    #[test]
    fn active_session_status_and_next_surface_compatibility_follow_up_without_replacing_session() {
        let workspace = temp_workspace("boundline-cli-session-active-compat-follow-up");

        persist_initialized_session_with_goal_hint(&workspace, None, None).unwrap();

        let mut trace = ExecutionTrace::new(
            "task-active-compat",
            "session-active-compat",
            "Compatibility trace for active session",
        );
        trace.terminal_status = Some(TaskStatus::Failed);
        trace.terminal_reason = Some(TerminalReason::new(
            TerminalCondition::UnrecoverableError,
            "compatibility run failed",
            None,
        ));
        trace.ended_at = Some(trace.started_at + 1);
        FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();

        let status = execute_status(Some(&workspace)).unwrap();
        assert!(
            status.terminal_output.contains("latest_status: initialized"),
            "{}",
            status.terminal_output
        );
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
        assert!(
            status.terminal_output.contains("latest compatibility follow-up remains inspect-only"),
            "{}",
            status.terminal_output
        );

        let next = execute_next(Some(&workspace)).unwrap();
        assert!(
            next.terminal_output.contains("continuity_authority: compatibility_trace"),
            "{}",
            next.terminal_output
        );
        assert!(
            next.terminal_output.contains("compatibility_follow_up: inspect_only"),
            "{}",
            next.terminal_output
        );
        assert!(
            next.terminal_output.contains("next_command: boundline goal --goal <goal>"),
            "{}",
            next.terminal_output
        );
        assert!(
            next.terminal_output.contains("latest compatibility follow-up remains inspect_only"),
            "{}",
            next.terminal_output
        );
    }

    #[test]
    fn execute_status_surfaces_framework_adapter_trace_summary_and_hook_dispatch() {
        let workspace = write_execution_workspace("boundline-cli-session-status-adapter-summary");
        let mut record = persist_initialized_session_with_goal_hint(
            &workspace,
            Some("Handle adapter block"),
            None,
        )
        .unwrap();
        let run_id = format!("run-{}", record.session_id);
        let routing = sample_framework_adapter_stage_routing(&run_id);
        let failure = sample_framework_adapter_stage_failure(&run_id);
        let hook_dispatch = sample_framework_adapter_hook_dispatch(&run_id);
        let terminal_reason = TerminalReason::new(
            TerminalCondition::NoCredibleNextStep,
            "adapter blocked run",
            Some(serde_json::to_value(&failure).unwrap()),
        );
        let mut trace = ExecutionTrace::new(
            "task-status-adapter-summary",
            &record.session_id,
            "Handle adapter block",
        );
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            0,
            json!({"framework_adapter_stage_routing": routing}),
        );
        trace.finalize(TaskStatus::Failed, terminal_reason.clone());
        let trace_ref = FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();

        record.goal = Some("Handle adapter block".to_string());
        record.goal_plan = Some(sample_goal_plan("Handle adapter block"));
        record.latest_status = SessionStatus::Blocked;
        record.latest_terminal_reason = Some(terminal_reason);
        record.latest_trace_ref = Some(trace_ref.to_string_lossy().into_owned());
        FileSessionStore::for_workspace(&workspace).persist(&record).unwrap();
        FileSessionAuditStore::for_session(&workspace, &record.session_id)
            .append_hook_dispatch(&hook_dispatch)
            .unwrap();

        let status = execute_status(Some(&workspace)).unwrap();
        let view = status.session_status.expect("status should expose session status");

        assert_eq!(
            view.latest_framework_adapter_stage_routing,
            Some(sample_framework_adapter_stage_routing(&run_id))
        );
        assert_eq!(
            view.latest_framework_adapter_hook_dispatch,
            Some(sample_framework_adapter_hook_dispatch(&run_id))
        );
        assert_eq!(
            view.latest_framework_adapter_stage_failure,
            Some(sample_framework_adapter_stage_failure(&run_id))
        );
    }

    #[test]
    fn execute_status_falls_back_to_framework_adapter_stage_routing_from_trace_payload() {
        let workspace = write_execution_workspace("boundline-cli-session-status-adapter-fallback");
        let mut record = persist_initialized_session_with_goal_hint(
            &workspace,
            Some("Recover adapter routing"),
            None,
        )
        .unwrap();
        let run_id = format!("run-{}", record.session_id);
        let routing = sample_framework_adapter_stage_routing(&run_id);
        let mut trace = ExecutionTrace::new(
            "task-status-adapter-fallback",
            &record.session_id,
            "Recover adapter routing",
        );
        trace.record_event(
            TraceEventType::StageRouted,
            None,
            0,
            json!({"framework_adapter_stage_routing": routing}),
        );
        let trace_ref = FileTraceStore::for_workspace(&workspace).persist(&trace).unwrap();
        let mut latest_trace = ExecutionTrace::new(
            "task-status-adapter-fallback-latest",
            &record.session_id,
            "Recover adapter routing",
        );
        latest_trace.record_event(
            TraceEventType::GoalPlanCreated,
            None,
            0,
            json!({"goal_plan_summary": "native plan persisted"}),
        );
        latest_trace.finalize(
            TaskStatus::Succeeded,
            TerminalReason::new(
                TerminalCondition::GoalSatisfied,
                "native planning completed",
                None,
            ),
        );
        latest_trace.started_at = trace.started_at.saturating_add(1);
        latest_trace.ended_at = Some(trace.started_at.saturating_add(2));
        FileTraceStore::for_workspace(&workspace).persist(&latest_trace).unwrap();

        record.goal = Some("Recover adapter routing".to_string());
        record.goal_plan = Some(sample_goal_plan("Recover adapter routing"));
        record.latest_status = SessionStatus::Blocked;
        record.latest_trace_ref = Some(trace_ref.to_string_lossy().into_owned());
        FileSessionStore::for_workspace(&workspace).persist(&record).unwrap();

        let status = execute_status(Some(&workspace)).unwrap();
        let view = status.session_status.expect("status should expose session status");

        assert_eq!(
            view.latest_framework_adapter_stage_routing,
            Some(sample_framework_adapter_stage_routing(&run_id))
        );
        assert!(view.latest_framework_adapter_hook_dispatch.is_none());
    }

    #[test]
    fn status_view_falls_back_to_test_decision_validation_and_evidence_basis() {
        let workspace = temp_workspace("boundline-cli-session-selector-fallback");
        let mut decision = Decision::new(
            DecisionType::Test,
            "test suite",
            "run bounded validation",
            "collect validation evidence",
            vec![
                EvidenceRef::trace("trace-1"),
                EvidenceRef::file("src/lib.rs"),
                EvidenceRef::canon(".canon/policy.json"),
                EvidenceRef::tool_output("decision-0"),
            ],
        );
        decision.mark_dispatched().unwrap();
        decision
            .mark_failed(crate::domain::tool_result::ToolResult::new(
                "tester",
                "tester test suite",
                false,
                1,
            ))
            .unwrap();

        let record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-selector-fallback".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: None,
            workflow_progress: None,
            decisions: vec![decision],
            active_flow_policy: None,
            latest_status: SessionStatus::Failed,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };

        let view = build_status_view_with_follow_up(
            &record,
            Some("boundline inspect".to_string()),
            "inspect the latest decision",
            None,
        );

        assert_eq!(view.latest_validation_status.as_deref(), Some("failed"));
        let headline = view.latest_selection_headline.as_deref().unwrap();
        assert!(headline.contains("trace:trace-1"), "{headline}");
        assert!(headline.contains("file:src/lib.rs"), "{headline}");
        assert!(headline.contains("canon:.canon/policy.json"), "{headline}");
        assert!(headline.contains("tool_output:decision-0"), "{headline}");
        assert_eq!(view.latest_candidate_family.as_deref(), Some("test"));
    }

    #[test]
    fn status_view_projects_task_level_canon_memory_when_goal_plan_is_absent() {
        let workspace = write_execution_workspace("boundline-cli-session-canon-memory");
        let request = build_task_request(
            &workspace,
            "Fix the failing add test",
            "session-canon-memory",
            None,
            None,
        )
        .unwrap();
        let plan =
            build_fixture_plan_for_goal(&workspace, None, "Fix the failing add test").unwrap();
        let mut task = Task::new("task-canon-memory", &request, plan).unwrap();
        task.context.set_latest_advanced_context(&sample_advanced_context()).unwrap();
        task.context
            .set_latest_compacted_canon_memory(&CompactedCanonMemory {
                headline: "Canon verification packet".to_string(),
                credibility: MemoryCredibilityState::Stale,
                stage_key: Some("change:verify".to_string()),
                run_ref: Some("run-9".to_string()),
                packet_ref: Some(".canon/runs/run-9".to_string()),
                reason_code: Some("refresh_required".to_string()),
                artifact_refs: vec![".canon/runs/run-9/verification.md".to_string()],
                mode_summary: None,
                possible_actions: Vec::new(),
                recommended_next_action: Some(
                    crate::domain::governance::CanonRecommendedActionSummary {
                        action: "refresh".to_string(),
                        rationale: "refresh the governed packet and reassess its credibility"
                            .to_string(),
                        target: Some("run-9".to_string()),
                    },
                ),
                evidence_summary: None,
                authority_provenance_lines: Vec::new(),
                adaptive_provenance_lines: Vec::new(),
                semantic_provenance_lines: Vec::new(),
            })
            .unwrap();
        let record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-canon-memory".to_string(),
            workspace_ref: workspace.to_string_lossy().into_owned(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: Some(task),
            goal_plan: None,
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::Running,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };

        let view = build_status_view_with_follow_up(
            &record,
            Some("boundline inspect".to_string()),
            "inspect the Canon packet",
            None,
        );

        assert_eq!(
            view.context_summary.as_deref(),
            Some("canon memory: Canon verification packet [stale]")
        );
        assert_eq!(view.context_credibility.as_deref(), Some("stale"));
        assert_eq!(
            view.context_primary_inputs.as_deref(),
            Some([".canon/runs/run-9/verification.md".to_string()].as_slice())
        );
        assert!(view.context_provenance.as_ref().is_some_and(|lines| {
            lines.contains(&"canon_memory_compatibility: warning".to_string())
        }));
        assert!(
            view.context_provenance.as_ref().is_some_and(|lines| {
                lines.contains(&"canon_memory_run_ref: run-9".to_string())
            })
        );
        assert_eq!(view.context_staleness_reason.as_deref(), Some("refresh_required"));
        assert_eq!(
            view.advanced_context.as_ref().map(AdvancedContextProjection::selected_evidence_count),
            Some(1)
        );
        assert_eq!(
            view.governance_next_action.as_deref(),
            Some("refresh: refresh the governed packet and reassess its credibility")
        );
    }

    #[test]
    fn suggested_next_command_and_error_helpers_cover_context_and_flow_follow_up() {
        let goal_plan_with_context = GoalPlan::new(
            "Fix the failing add test",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Fix arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_planning_rationale("workspace evidence isolates the failing arithmetic path")
        .with_verification_strategy("rerun the focused add regression after editing")
        .with_context_pack(ContextPack {
            pack_id: "cp-1".to_string(),
            summary: "bounded context from src/lib.rs".to_string(),
            credibility: ContextPackCredibility::Insufficient,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "closest source file".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            advanced_context: None,
            staleness_reason: None,
        });

        let base_record = crate::domain::session::ActiveSessionRecord {
            session_id: "session-next".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            goal: Some("Fix the failing add test".to_string()),
            authored_brief: None,
            negotiation_packet: None,
            active_flow: None,
            active_task: None,
            goal_plan: Some(goal_plan_with_context.clone()),
            workflow_progress: None,
            decisions: Vec::new(),
            active_flow_policy: None,
            latest_status: SessionStatus::GoalCaptured,
            latest_terminal_reason: None,
            latest_trace_ref: None,
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
        };
        assert_eq!(
            suggested_next_command(&base_record),
            Some("boundline goal --goal <narrower goal>".to_string())
        );

        let mut pending_flow_plan = GoalPlan::new(
            "Fix the failing add test",
            vec![PlannedTask {
                task_id: "planned-task-2".to_string(),
                description: "Fix arithmetic".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("tests pass".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_planning_rationale("the failing arithmetic task is narrowed to src/lib.rs")
        .with_verification_strategy("rerun the focused add regression after the change");
        pending_flow_plan.flow = Some(InferredFlow {
            flow_name: "bug-fix".to_string(),
            confidence_reason: "goal contains fix".to_string(),
            confirmed: false,
        });
        let mut pending_flow_record = base_record.clone();
        pending_flow_record.goal_plan = Some(pending_flow_plan);
        pending_flow_record.latest_status = SessionStatus::Planned;
        assert_eq!(suggested_next_command(&pending_flow_record), Some("boundline run".to_string()));

        let mut ready_run_record = pending_flow_record.clone();
        ready_run_record.goal_plan.as_mut().unwrap().confirm().unwrap();
        assert_eq!(suggested_next_command(&ready_run_record), Some("boundline run".to_string()));

        let mut blocked_planning_record = ready_run_record.clone();
        blocked_planning_record.latest_status = SessionStatus::Blocked;
        blocked_planning_record.latest_trace_ref = None;
        blocked_planning_record.authored_brief = Some(crate::domain::brief::AuthoredBriefBundle {
            bundle_id: "bundle-blocked".to_string(),
            primary_goal_text: Some("Fix the failing add test".to_string()),
            sources: vec![crate::domain::brief::InputSourceReference {
                source_id: "brief-1".to_string(),
                kind: crate::domain::brief::InputSourceKind::AttachedMarkdown,
                display_name: "brief.md".to_string(),
                workspace_path: Some("brief.md".to_string()),
                precedence: 0,
                content: "Need Canon review context".to_string(),
            }],
            deduplicated_sources: Vec::new(),
            governance_intent: None,
            resolution_state: crate::domain::brief::AuthoredBriefResolutionState::Ready,
            goal_quality: Default::default(),
            clarification: None,
            derived_task_draft: None,
            captured_at: 1,
        });
        blocked_planning_record.governance_lifecycle = Some(GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Canon,
            explicit_opt_out: false,
            mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
            selected_mode: Some(CanonMode::Discovery),
            selected_mode_sequence: vec![CanonMode::Discovery],
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: vec![GovernedStageRecord {
                stage_key: "plan:discovery".to_string(),
                runtime: GovernanceRuntimeKind::Canon,
                lifecycle_state: GovernanceLifecycleState::Blocked,
                required: true,
                autopilot_enabled: false,
                approval_state: ApprovalState::Rejected,
                canon_run_ref: Some("R-20260522-019e5141".to_string()),
                governance_attempt_id: "attempt-plan-discovery".to_string(),
                previous_governance_attempt_id: None,
                packet_ref: Some(".canon/artifacts/R-20260522-019e5141/discovery".to_string()),
                decision_ref: None,
                blocked_reason: Some(
                    "plan:discovery stage council blocked planning: configure distinct provider-backed reviewer routes before rerunning boundline plan".to_string(),
                ),
                stage_council: Some(crate::domain::stage_council::StageCouncilOutcome {
                    producer_output: crate::domain::stage_council::StageCouncilArtifact {
                        route_slot: "planning".to_string(),
                        evidence_ref: ".boundline/governance/planning/discovery/brief.md".to_string(),
                        summary: Some("planner produced the discovery artifact for council review".to_string()),
                    },
                    reviewer_findings: Vec::new(),
                    vote_resolution: crate::domain::stage_council::StageCouncilVoteResolution {
                        strategy: "bounded_majority".to_string(),
                        accepted_findings: Vec::new(),
                        rejected_findings: Vec::new(),
                        independent_review: false,
                    },
                    adjudication: None,
                    revised_output: crate::domain::stage_council::StageCouncilArtifact {
                        route_slot: "planning".to_string(),
                        evidence_ref: ".boundline/governance/planning/discovery/blocked.md".to_string(),
                        summary: Some("stage council blocked planning discovery".to_string()),
                    },
                    status: crate::domain::stage_council::StageCouncilStatus::Blocked,
                    next_action:
                        "configure distinct provider-backed reviewer routes before rerunning boundline plan"
                            .to_string(),
                }),
            }],
            accumulated_context: Vec::new(),
            terminal_reason: Some(
                "plan:discovery stage council blocked planning: configure distinct provider-backed reviewer routes before rerunning boundline plan"
                    .to_string(),
            ),
            planning_input_fingerprint: None,
        });
        assert_eq!(
            suggested_next_command(&blocked_planning_record),
            Some(REVIEWER_ROUTE_REPAIR_COMMAND.to_string())
        );
        assert_eq!(
            planning_governance_next_action(&blocked_planning_record),
            Some(
                "configure distinct provider-backed reviewer routes before rerunning boundline plan"
                    .to_string()
            )
        );

        let mut clarification_record = base_record.clone();
        clarification_record.authored_brief = Some(crate::domain::brief::AuthoredBriefBundle {
            bundle_id: "bundle-clarification".to_string(),
            primary_goal_text: Some("Fix the failing add test".to_string()),
            sources: vec![crate::domain::brief::InputSourceReference {
                source_id: "brief-2".to_string(),
                kind: crate::domain::brief::InputSourceKind::AttachedMarkdown,
                display_name: "brief.md".to_string(),
                workspace_path: Some("brief.md".to_string()),
                precedence: 0,
                content: "Missing the business context".to_string(),
            }],
            deduplicated_sources: Vec::new(),
            governance_intent: None,
            resolution_state:
                crate::domain::brief::AuthoredBriefResolutionState::ClarificationRequired,
            goal_quality: Default::default(),
            clarification: Some(crate::domain::task::ClarificationRecord {
                clarification_id: "clarification-1".to_string(),
                reason_kind: crate::domain::task::ClarificationReasonKind::MissingContext,
                prompt: "provide the missing business context".to_string(),
                missing_fields: vec!["api_operations".to_string()],
                questions: vec!["Which API operations are in scope first?".to_string()],
                blocking_sources: vec!["brief.md".to_string()],
                turn_index: 0,
                status: crate::domain::task::ClarificationStatus::Open,
            }),
            derived_task_draft: None,
            captured_at: 1,
        });
        assert_eq!(
            suggested_next_command(&clarification_record),
            Some("boundline goal --goal <narrower goal>".to_string())
        );

        let checkpoint_plan = crate::domain::plan::Plan::new(vec![
            crate::domain::step::Step::tool("verify", "cargo", json!({})).unwrap(),
        ])
        .unwrap();
        let checkpoint_request = crate::domain::task::TaskRunRequest {
            goal: "Fix the failing add test".to_string(),
            input: json!({}),
            session_id: "session-next".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            limits: crate::domain::limits::RunLimits::default(),
            initial_context: None,
        };
        let mut checkpoint_task =
            Task::new("task-checkpoint", &checkpoint_request, checkpoint_plan).unwrap();
        checkpoint_task.context.state.insert(
            "latest_checkpoint_restore_command".to_string(),
            json!("boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"),
        );
        let mut failed_checkpoint_record = base_record.clone();
        failed_checkpoint_record.goal_plan = None;
        failed_checkpoint_record.active_task = Some(checkpoint_task);
        failed_checkpoint_record.latest_status = SessionStatus::Failed;
        assert_eq!(
            suggested_next_command(&failed_checkpoint_record),
            Some(
                "boundline checkpoint restore checkpoint-123 --workspace /tmp/workspace"
                    .to_string(),
            )
        );

        let clarification_error = SessionCommandError::ClarificationRequired {
            headline: "bounded context required before planning".to_string(),
            prompt: "pick one bounded outcome".to_string(),
        };
        let clarification_text = render_error("plan", &clarification_error);
        assert!(
            clarification_text.contains("next_command: boundline goal --goal <narrower goal>")
                || clarification_text.contains("boundline goal --goal <narrower goal>"),
            "{clarification_text}"
        );

        let cluster_error = SessionCommandError::MissingClusterConfig {
            workspace: PathBuf::from("/tmp/workspace"),
            command_name: "status",
        };
        let cluster_text = render_error("status", &cluster_error);
        assert!(
            cluster_text.contains("boundline cluster init --workspace <primary>"),
            "{cluster_text}"
        );
    }

    #[test]
    fn status_and_continue_bootstrap_distinguish_workspace_initialization() {
        let uninitialized = temp_workspace("boundline-cli-session-uninitialized");
        let status_report = execute_status_with_target(Some(&uninitialized), None, None).unwrap();
        assert_eq!(status_report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            status_report.terminal_output.contains("command: status"),
            "{}",
            status_report.terminal_output
        );
        assert!(
            status_report.terminal_output.contains("workspace_initialized: false"),
            "{}",
            status_report.terminal_output
        );
        assert!(
            status_report.terminal_output.contains("next_command: boundline init --workspace "),
            "{}",
            status_report.terminal_output
        );

        let initialized = temp_workspace("boundline-cli-session-initialized");
        fs::create_dir_all(initialized.join(".boundline")).unwrap();
        let continue_report = execute_continue_with_target(Some(&initialized), None, None).unwrap();
        assert_eq!(continue_report.exit_status, CommandExitStatus::Succeeded);
        assert!(
            continue_report.terminal_output.contains("command: continue"),
            "{}",
            continue_report.terminal_output
        );
        assert!(
            continue_report.terminal_output.contains("workspace_initialized: true"),
            "{}",
            continue_report.terminal_output
        );
        assert!(
            continue_report.terminal_output.contains("chat history is not authoritative"),
            "{}",
            continue_report.terminal_output
        );
        assert!(
            continue_report
                .terminal_output
                .contains("next_command: boundline session list --workspace "),
            "{}",
            continue_report.terminal_output
        );
        assert!(
            continue_report.terminal_output.contains("repair_command: boundline goal --workspace "),
            "{}",
            continue_report.terminal_output
        );
    }

    #[test]
    fn continue_projects_governance_project_scale_and_voting_state_from_session_record() {
        let workspace = write_execution_workspace("boundline-cli-session-continue-projection");
        let canonical_workspace = fs::canonicalize(&workspace).unwrap();
        let goal_plan = GoalPlan::new(
            "Prepare review packet",
            vec![PlannedTask {
                task_id: "planned-task-review".to_string(),
                description: "Prepare governed review packet".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("review packet is ready".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_planning_rationale("the governed review packet is scoped to the active source target")
        .with_verification_strategy("rerun status after the governed review approval resolves");

        let record = ActiveSessionRecord {
            session_id: "session-continue-projection".to_string(),
            workspace_ref: canonical_workspace.to_string_lossy().into_owned(),
            goal: Some("Prepare review packet".to_string()),
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
            created_at: 1,
            updated_at: 2,
            governance_lifecycle: Some(GovernedSessionLifecycle {
                governance_runtime: GovernanceRuntimeKind::Canon,
                explicit_opt_out: false,
                mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
                selected_mode: Some(CanonMode::Review),
                selected_mode_sequence: vec![CanonMode::Review],
                latest_reasoning_profile: None,
                current_stage_index: 0,
                stage_records: vec![GovernedStageRecord {
                    stage_key: "govern:review".to_string(),
                    runtime: GovernanceRuntimeKind::Canon,
                    lifecycle_state: GovernanceLifecycleState::AwaitingApproval,
                    required: true,
                    autopilot_enabled: false,
                    approval_state: ApprovalState::Requested,
                    canon_run_ref: Some("run-42".to_string()),
                    governance_attempt_id: "attempt-1".to_string(),
                    previous_governance_attempt_id: None,
                    packet_ref: Some(".canon/runs/run-42".to_string()),
                    decision_ref: Some(".canon/runs/run-42/decision.md".to_string()),
                    blocked_reason: Some("waiting for review approval".to_string()),
                    stage_council: None,
                }],
                accumulated_context: Vec::new(),
                terminal_reason: None,
                planning_input_fingerprint: None,
            }),
            project_scale: Some(ProjectScaleSessionState {
                path: ProjectScalePath {
                    kind: ProjectScalePathKind::ExistingSystemChange,
                    goal: "Prepare review packet".to_string(),
                    stages: vec![
                        ProjectScaleStage {
                            kind: ProjectScaleStageKind::Requirements,
                            reason: "clarify the governed packet scope".to_string(),
                        },
                        ProjectScaleStage {
                            kind: ProjectScaleStageKind::Implementation,
                            reason: "prepare the packet inputs".to_string(),
                        },
                        ProjectScaleStage {
                            kind: ProjectScaleStageKind::Verification,
                            reason: "confirm the packet can be reviewed".to_string(),
                        },
                    ],
                    requires_confirmation: false,
                    next_action: "complete implementation changes".to_string(),
                    unbounded_autonomy: false,
                },
                active_stage_index: 1,
                active_work_unit_id: Some("impl-1".to_string()),
                checkpoint_refs: vec!["checkpoint-1".to_string()],
                trace_refs: vec!["trace-1".to_string()],
                next_action: "complete implementation changes".to_string(),
            }),
            latest_voting: Some(VotingSessionState {
                trigger: "pr_ready".to_string(),
                reviewed_evidence_ref: Some("govern:review".to_string()),
                result: "approved".to_string(),
                reviewer_findings: vec!["safety: approved".to_string()],
                adjudication_result: Some("accepted".to_string()),
                blocking: true,
                next_action: "collect approval".to_string(),
            }),
            delight_feedback: None,
        };

        FileSessionStore::for_workspace(&workspace).persist(&record).unwrap();

        let report = execute_continue_with_target(Some(&workspace), None, None).unwrap();
        let view = report.session_status.unwrap();

        assert_eq!(view.governance_lifecycle_runtime.as_deref(), Some("canon"));
        assert_eq!(view.governance_lifecycle_mode_selection.as_deref(), Some("auto-confirm"));
        assert_eq!(view.governance_lifecycle_selected_mode.as_deref(), Some("review"));
        assert_eq!(
            view.governance_lifecycle_selected_mode_sequence.as_ref(),
            Some(&vec!["review".to_string()])
        );
        assert_eq!(
            view.project_scale_path.as_deref(),
            Some("requirements -> implementation -> verification")
        );
        assert_eq!(view.project_scale_current_stage.as_deref(), Some("implementation"));
        assert_eq!(
            view.project_scale_next_action.as_deref(),
            Some("complete implementation changes")
        );
        assert_eq!(
            view.project_scale_checkpoint_refs.as_deref(),
            Some(["checkpoint-1".to_string()].as_slice())
        );
        assert_eq!(view.latest_voting_trigger.as_deref(), Some("pr_ready"));
        assert_eq!(view.latest_voting_result.as_deref(), Some("approved"));
        assert_eq!(view.latest_voting_adjudication.as_deref(), Some("accepted"));
        assert_eq!(view.latest_voting_reviewed_evidence.as_deref(), Some("govern:review"));
        assert_eq!(view.latest_voting_blocking, Some(true));
        assert_eq!(view.latest_voting_next_action.as_deref(), Some("collect approval"));
        assert_eq!(view.next_command.as_deref(), Some("boundline run"));
        let expected_explanation = format!(
            "continue uses the active session state resolved through {}",
            super::SESSION_SOURCE_OF_TRUTH
        );
        assert!(
            report.terminal_output.contains(&expected_explanation),
            "{}",
            report.terminal_output
        );
    }

    #[test]
    fn build_status_view_falls_back_to_planning_governance_lifecycle() {
        let workspace = write_execution_workspace("boundline-cli-session-plan-governance");
        let canonical_workspace = fs::canonicalize(&workspace).unwrap();
        let goal_plan = GoalPlan::new(
            "Prepare governed feature",
            vec![PlannedTask {
                task_id: "planned-task-plan-governance".to_string(),
                description: "Prepare governed plan".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("planning governance clears".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_planning_rationale("governed planning evidence is assembled for the next stage")
        .with_verification_strategy("rerun plan or status after Canon approval resolves");

        let record = ActiveSessionRecord {
            session_id: "session-plan-governance".to_string(),
            workspace_ref: canonical_workspace.to_string_lossy().into_owned(),
            goal: Some("Prepare governed feature".to_string()),
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
            created_at: 1,
            updated_at: 2,
            governance_lifecycle: Some(GovernedSessionLifecycle {
                governance_runtime: GovernanceRuntimeKind::Canon,
                explicit_opt_out: false,
                mode_selection_preference: CanonModeSelectionPreference::AutoConfirm,
                selected_mode: Some(crate::domain::governance::CanonMode::Requirements),
                selected_mode_sequence: vec![
                    crate::domain::governance::CanonMode::Requirements,
                    crate::domain::governance::CanonMode::Architecture,
                    crate::domain::governance::CanonMode::Backlog,
                ],
                latest_reasoning_profile: None,
                current_stage_index: 0,
                stage_records: vec![crate::domain::governance::GovernedStageRecord {
                    stage_key: "plan:requirements".to_string(),
                    runtime: GovernanceRuntimeKind::Canon,
                    lifecycle_state:
                        crate::domain::governance::GovernanceLifecycleState::AwaitingApproval,
                    required: true,
                    autopilot_enabled: false,
                    approval_state: crate::domain::governance::ApprovalState::Requested,
                    canon_run_ref: Some("canon-run-plan".to_string()),
                    governance_attempt_id: "attempt-plan-1".to_string(),
                    previous_governance_attempt_id: None,
                    packet_ref: Some(".canon/planning-packet".to_string()),
                    decision_ref: None,
                    blocked_reason: Some("waiting for Canon approval".to_string()),
                    stage_council: None,
                }],
                accumulated_context: Vec::new(),
                terminal_reason: Some("awaiting approval: waiting for Canon approval".to_string()),
                planning_input_fingerprint: None,
            }),
            project_scale: None,
            delight_feedback: None,
            latest_voting: None,
        };

        let view = build_status_view(
            &record,
            suggested_next_command(&record),
            "current active session state for the workspace",
        );

        assert_eq!(view.latest_governance_stage.as_deref(), Some("plan:requirements"));
        assert_eq!(view.latest_governance_runtime.as_deref(), Some("canon"));
        assert_eq!(view.latest_governance_mode.as_deref(), Some("requirements"));
        assert_eq!(view.latest_governance_run_ref.as_deref(), Some("canon-run-plan"));
        assert_eq!(view.latest_governance_state.as_deref(), Some("awaiting_approval"));
        assert_eq!(
            view.latest_governance_blocked_reason.as_deref(),
            Some("waiting for Canon approval")
        );
        assert_eq!(view.latest_governance_packet_ref.as_deref(), Some(".canon/planning-packet"));
        assert_eq!(view.latest_governance_approval.as_deref(), Some("requested"));
        assert_eq!(
            view.governance_next_action.as_deref(),
            Some("wait for approval and rerun boundline plan")
        );
        assert_eq!(view.next_command.as_deref(), Some("boundline status"));
    }

    #[test]
    fn goal_with_explicit_slug_embeds_slug_in_session_record() {
        let workspace = write_execution_workspace("boundline-cli-session-slug-override");

        let report = execute_goal_with_target(
            Some(&workspace),
            None,
            Some("Build a bounded user management microservice"),
            &[],
            None,
            None,
            None,
            None,
            Some("micro-rust-goal"),
        )
        .unwrap();

        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);

        let store = FileSessionStore::for_workspace(&workspace);
        let record = store.load().unwrap().unwrap();
        assert!(
            record.session_id.contains("micro-rust-goal"),
            "session_id should embed the slug override, got: {}",
            record.session_id
        );
    }

    #[test]
    fn run_plan_refinement_if_enabled_returns_some_when_config_active() {
        let workspace = tempfile::tempdir().unwrap();
        let boundline = workspace.path().join(".boundline");
        fs::create_dir_all(&boundline).unwrap();
        fs::write(
            boundline.join("refinement-profiles.toml"),
            "[profiles.plan_refinement]\nenabled = true\nmax_rounds = 2\nmax_elapsed_time_seconds = 60\n\n[profiles.plan_refinement.roles]\nplanner_provider_id = \"p\"\ncritic_provider_id = \"p\"\nfinalizer_provider_id = \"p\"\n",
        )
        .unwrap();
        let outcome = run_plan_refinement_if_enabled(workspace.path(), false, false, None);
        assert!(outcome.is_some());
    }

    #[test]
    fn run_plan_refinement_if_enabled_returns_none_when_disabled() {
        let workspace = tempfile::tempdir().unwrap();
        let boundline = workspace.path().join(".boundline");
        fs::create_dir_all(&boundline).unwrap();
        fs::write(
            boundline.join("refinement-profiles.toml"),
            "[profiles.plan_refinement]\nenabled = false\nmax_rounds = 2\nmax_elapsed_time_seconds = 60\n\n[profiles.plan_refinement.roles]\nplanner_provider_id = \"p\"\ncritic_provider_id = \"p\"\nfinalizer_provider_id = \"p\"\n",
        )
        .unwrap();
        let outcome = run_plan_refinement_if_enabled(workspace.path(), false, false, None);
        assert!(outcome.is_none());
    }

    #[test]
    fn refinement_status_summary_returns_some_when_enabled() {
        let workspace = tempfile::tempdir().unwrap();
        let boundline = workspace.path().join(".boundline");
        fs::create_dir_all(&boundline).unwrap();
        fs::write(
            boundline.join("refinement-profiles.toml"),
            "[profiles.plan_refinement]\nenabled = true\nmax_rounds = 5\nmax_elapsed_time_seconds = 120\n\n[profiles.plan_refinement.roles]\nplanner_provider_id = \"p\"\ncritic_provider_id = \"p\"\nfinalizer_provider_id = \"p\"\n",
        )
        .unwrap();
        let summary = refinement_status_summary(workspace.path());
        assert!(summary.is_some());
        let s = summary.unwrap();
        assert!(s.contains("plan_refinement"));
        assert!(s.contains("max_rounds=5"));
    }

    #[test]
    fn refinement_status_summary_returns_none_when_missing() {
        let workspace = tempfile::tempdir().unwrap();
        let summary = refinement_status_summary(workspace.path());
        assert!(summary.is_none());
    }

    #[test]
    fn refinement_next_hint_returns_none() {
        // Stub always returns None regardless of input.
    }
}
