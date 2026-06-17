use std::path::{Path, PathBuf};

use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::adapters::config_store::FileConfigStore;
use crate::adapters::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use crate::adapters::trace_store::{FileTraceStore, TraceStore};
use crate::cli::CommandExitStatus;
use crate::cli::inspect::summarize_trace;
use crate::cli::output;
use crate::cli::session::{self, SessionCommandError};
use crate::domain::brief::{
    BriefIngestionError, normalize_governance_intent, normalize_inputs_with_governance,
};
use crate::domain::distribution::evaluate_canon_install;
use crate::domain::governance::{
    CanonAuthorityZone, CanonIntendedPersona, CanonMode, CanonModeSelectionPreference,
    CanonRiskClass, GovernanceRuntimeKind, GovernedSessionLifecycle,
};
use crate::domain::limits::TerminalCondition;
use crate::domain::session::{
    ActiveSessionRecord, RoutingMode, RoutingOutcome, RoutingSource, SessionStatus,
    SessionStatusView,
};
use crate::domain::task::{TaskRunResponse, TaskStatus, TerminalReason};
use crate::domain::task_context::TaskContext;
use crate::domain::trace::{ExecutionTrace, TraceEventType, TraceSummaryView};
use crate::fixture::{
    FixtureRuntimeError, build_fixture_runtime, build_task_request,
    load_workspace_execution_profile,
};
use crate::orchestrator::engine::{Orchestrator, OrchestratorError};

const DIRECT_RUN_BOUNDED_CONTEXT_HEADLINE: &str = "bounded context required before planning";
const DIRECT_RUN_BOUNDED_CONTEXT_REPAIR: &str =
    "provide a credible brief or concrete workspace target before retrying direct run";

#[derive(Debug, Clone, PartialEq)]
pub struct RunCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
    pub session_status: Option<SessionStatusView>,
    pub trace_summary: Option<TraceSummaryView>,
}

#[allow(clippy::too_many_arguments)]
pub fn execute_native_direct_run(
    workspace: &Path,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
    mode: Option<CanonMode>,
    no_canon: bool,
) -> Result<RunCommandReport, RunCommandError> {
    ensure_native_direct_run_can_bootstrap(workspace)?;

    // Resolve effective governance runtime and apply config defaults.
    let resolution = resolve_canon_default_governance(workspace, governance, mode, no_canon)?;

    let effective_risk = risk.or(resolution.default_risk.as_deref());
    let effective_zone = zone.or(resolution.default_zone.as_deref());
    let effective_owner = owner.or(resolution.default_owner.as_deref());

    session::execute_goal(
        Some(workspace),
        goal,
        briefs,
        resolution.governance,
        effective_risk,
        effective_zone,
        effective_owner,
    )
    .map_err(RunCommandError::SessionCommand)?;

    if resolution.governance == Some(GovernanceRuntimeKind::Canon)
        && load_workspace_execution_profile(workspace)
            .is_ok_and(|profile| profile.governance.is_some())
    {
        session::execute_flow(Some(workspace), native_direct_run_flow_for_mode(mode))
            .map_err(RunCommandError::SessionCommand)?;
    }

    // T032: Create GovernedSessionLifecycle when Canon governance is selected.
    if resolution.governance == Some(GovernanceRuntimeKind::Canon) {
        let mode_selection = resolution.mode_selection_preference.unwrap_or_default();
        let lifecycle = GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Canon,
            explicit_opt_out: false,
            mode_selection_preference: mode_selection,
            selected_mode: mode,
            selected_mode_sequence: mode.into_iter().collect(),
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: Vec::new(),
            accumulated_context: Vec::new(),
            terminal_reason: None,
            planning_input_fingerprint: None,
        };
        let session_store = FileSessionStore::for_workspace(workspace);
        if let Ok(Some(mut record)) = session_store.load() {
            record.governance_lifecycle = Some(lifecycle);
            let _ = session_store.persist(&record);
        }
    } else if no_canon {
        // Record explicit opt-out in session lifecycle.
        let lifecycle = GovernedSessionLifecycle {
            governance_runtime: GovernanceRuntimeKind::Local,
            explicit_opt_out: true,
            mode_selection_preference: CanonModeSelectionPreference::default(),
            selected_mode: None,
            selected_mode_sequence: Vec::new(),
            latest_reasoning_profile: None,
            current_stage_index: 0,
            stage_records: Vec::new(),
            accumulated_context: Vec::new(),
            terminal_reason: None,
            planning_input_fingerprint: None,
        };
        let session_store = FileSessionStore::for_workspace(workspace);
        if let Ok(Some(mut record)) = session_store.load() {
            record.governance_lifecycle = Some(lifecycle);
            let _ = session_store.persist(&record);
        }
    }

    let session_store = FileSessionStore::for_workspace(workspace);
    let record = session_store
        .load()
        .map_err(RunCommandError::SessionStore)?
        .ok_or(SessionCommandError::MissingActiveSession)
        .map_err(RunCommandError::SessionCommand)?;

    if native_direct_run_requires_clarification(&record) {
        let report =
            session::execute_status(Some(workspace)).map_err(RunCommandError::SessionCommand)?;
        return Ok(RunCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: report.terminal_output,
            trace_location: record.latest_trace_ref.clone(),
            session_status: report.session_status,
            trace_summary: report.trace_summary,
        });
    }

    session::execute_plan(Some(workspace), None, false).map_err(RunCommandError::SessionCommand)?;
    let report = session::execute_run(Some(workspace)).map_err(RunCommandError::SessionCommand)?;
    let trace_location = session_store
        .load()
        .map_err(RunCommandError::SessionStore)?
        .and_then(|record| record.latest_trace_ref);

    Ok(RunCommandReport {
        exit_status: report.exit_status,
        terminal_output: report.terminal_output,
        trace_location,
        session_status: report.session_status,
        trace_summary: report.trace_summary,
    })
}

fn native_direct_run_flow_for_mode(mode: Option<CanonMode>) -> &'static str {
    match mode {
        Some(
            CanonMode::Requirements
            | CanonMode::Architecture
            | CanonMode::Backlog
            | CanonMode::SystemShaping,
        ) => "delivery",
        Some(CanonMode::Change | CanonMode::Migration | CanonMode::SupplyChainAnalysis) => "change",
        _ => "bug-fix",
    }
}

pub fn execute_custom_run(
    workspace: &Path,
    goal: Option<&str>,
    briefs: &[PathBuf],
    governance: Option<GovernanceRuntimeKind>,
    risk: Option<&str>,
    zone: Option<&str>,
    owner: Option<&str>,
) -> Result<RunCommandReport, RunCommandError> {
    let governance_intent = normalize_governance_intent(governance, risk, zone, owner)?;
    let bundle = normalize_inputs_with_governance(workspace, goal, briefs, governance_intent)?;
    let goal = bundle.render_goal_text();
    let store = FileTraceStore::for_workspace(workspace);
    let trace_reader = store.clone();
    let request = build_task_request(
        workspace,
        goal,
        format!("run-{}", crate::domain::trace::current_timestamp_millis()),
        Some(&bundle),
        None,
    )?;

    if let Some(clarification) = bundle.clarification.as_ref() {
        let terminal_reason = TerminalReason::new(
            TerminalCondition::TaskNotCredible,
            clarification.prompt.clone(),
            Some(json!({
                "clarification_required": true,
                "clarification_headline": clarification.headline(),
                "clarification_missing_fields": clarification.missing_fields,
            })),
        );
        let mut trace = ExecutionTrace::new(
            Uuid::new_v4().to_string(),
            request.session_id.clone(),
            request.goal.clone(),
        );
        trace.record_event(
            TraceEventType::TaskStarted,
            None,
            0,
            json!({
                "goal": request.goal,
                "input": request.input,
                "limits": request.limits,
            }),
        );
        trace.record_event(
            TraceEventType::TerminalRecorded,
            None,
            0,
            json!({
                "terminal_status": TaskStatus::Failed,
                "terminal_reason": terminal_reason,
            }),
        );
        trace.finalize(TaskStatus::Failed, terminal_reason.clone());
        let trace_path = store.persist(&trace).map_err(RunCommandError::TraceStore)?;
        let trace_location = trace_path.to_string_lossy().into_owned();
        trace.set_trace_location(trace_location.clone());
        store.persist(&trace).map_err(RunCommandError::TraceStore)?;
        let loaded_trace = trace_reader.load(Path::new(&trace_location)).ok();
        let response = TaskRunResponse {
            task_id: trace.task_id.clone(),
            terminal_status: TaskStatus::Failed,
            terminal_reason,
            final_context: TaskContext::new(
                request.session_id,
                request.workspace_ref,
                request.limits,
                request.initial_context.unwrap_or_default(),
            ),
            plan_revision: 0,
            trace_location: trace_location.clone(),
        };
        let terminal_output = compatibility_terminal_output(output::render_run_trace(
            "run",
            loaded_trace.as_ref(),
            &response,
            "/boundline-inspect",
        ));
        let trace_summary = loaded_trace
            .as_ref()
            .and_then(|trace| summarize_trace(Path::new(&trace_location), trace).ok());
        return Ok(RunCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output,
            trace_location: Some(trace_location),
            session_status: None,
            trace_summary,
        });
    }

    let runtime = build_fixture_runtime(workspace)?;
    let orchestrator = Orchestrator::new(runtime.planner, runtime.agents, runtime.tools, store)
        .with_governance(runtime.profile.read_targets.clone(), runtime.profile.governance.clone());
    let response = orchestrator.run(request)?;
    let trace = trace_reader.load(Path::new(&response.trace_location)).ok();
    let exit_status = if response.terminal_status == TaskStatus::Succeeded {
        CommandExitStatus::Succeeded
    } else {
        CommandExitStatus::NonSuccess
    };
    let terminal_output = compatibility_terminal_output(output::render_run_trace(
        "run",
        trace.as_ref(),
        &response,
        output::next_command_after_run(response.terminal_status),
    ));
    let trace_summary = trace
        .as_ref()
        .and_then(|trace| summarize_trace(Path::new(&response.trace_location), trace).ok());

    Ok(RunCommandReport {
        exit_status,
        terminal_output,
        trace_location: Some(response.trace_location),
        session_status: None,
        trace_summary,
    })
}

#[derive(Debug, Error)]
pub enum RunCommandError {
    #[error("failed to ingest authored brief: {0}")]
    BriefIngestion(#[from] BriefIngestionError),
    #[error(
        "active session already contains meaningful work; continue it or reset the workspace session before using direct native run"
    )]
    ActiveSessionConflict,
    #[error("session store operation failed: {0}")]
    SessionStore(#[from] SessionStoreError),
    #[error("session command failed: {0}")]
    SessionCommand(#[from] SessionCommandError),
    #[error("failed to prepare the fixture-backed vertical slice: {0}")]
    FixtureRuntime(#[from] FixtureRuntimeError),
    #[error("failed to persist the clarification trace: {0}")]
    TraceStore(#[from] crate::adapters::trace_store::TraceStoreError),
    #[error("failed to run the orchestrator vertical slice: {0}")]
    Orchestrator(#[from] OrchestratorError),
    #[error(
        "Canon governance surface is not ready: {repair_actions}\n\nRun `boundline doctor --install` to resolve."
    )]
    CanonSurfaceNotReady { repair_actions: String },
}

impl RunCommandError {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::SessionCommand(error) => format!("session command failed: {}", error.message()),
            Self::FixtureRuntime(FixtureRuntimeError::NoSynthesizeableGoalPlanTarget {
                goal,
                workspace,
            }) => format!(
                "{DIRECT_RUN_BOUNDED_CONTEXT_HEADLINE}: {DIRECT_RUN_BOUNDED_CONTEXT_REPAIR} for goal '{goal}' in workspace {}",
                workspace.display()
            ),
            _ => self.to_string(),
        }
    }
}

/// Result of Canon-default governance resolution.
struct CanonGovernanceResolution {
    governance: Option<GovernanceRuntimeKind>,
    default_risk: Option<String>,
    default_zone: Option<String>,
    default_owner: Option<String>,
    mode_selection_preference: Option<CanonModeSelectionPreference>,
}

/// Resolve the effective governance runtime based on workspace config and CLI flags.
///
/// Priority:
/// 1. `--no-canon` → Local
/// 2. Explicit `--governance` flag → use as-is
/// 3. Workspace `[canon]` config present → Canon (after surface verification)
/// 4. Explicit `--mode` without workspace `[canon]` config → Canon (after surface verification)
/// 5. No `[canon]` config and no mode → Local (backward compatibility)
fn resolve_canon_default_governance(
    workspace: &Path,
    governance: Option<GovernanceRuntimeKind>,
    mode: Option<CanonMode>,
    no_canon: bool,
) -> Result<CanonGovernanceResolution, RunCommandError> {
    // --no-canon always forces Local.
    if no_canon {
        return Ok(CanonGovernanceResolution {
            governance: Some(GovernanceRuntimeKind::Local),
            default_risk: None,
            default_zone: None,
            default_owner: None,
            mode_selection_preference: None,
        });
    }

    let config_store = FileConfigStore::for_workspace(workspace);
    let config = config_store.load_local().ok().flatten();
    let canon_prefs = config.and_then(|c| c.canon);

    // Explicit --governance flag takes priority.
    if let Some(governance) = governance {
        if governance == GovernanceRuntimeKind::Canon {
            verify_canon_surface_ready()?;
        }
        return Ok(CanonGovernanceResolution {
            governance: Some(governance),
            default_risk: canon_prefs.as_ref().and_then(|prefs| prefs.default_risk.clone()).map(
                |risk| {
                    CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
                },
            ),
            default_owner: canon_prefs.as_ref().and_then(|prefs| prefs.default_owner.clone()).map(
                |owner| {
                    CanonIntendedPersona::canonicalize_label(&owner)
                        .map(str::to_string)
                        .unwrap_or(owner)
                },
            ),
            default_zone: canon_prefs.as_ref().and_then(|prefs| prefs.default_zone.clone()).map(
                |zone| {
                    CanonAuthorityZone::canonicalize_label(&zone)
                        .map(str::to_string)
                        .unwrap_or(zone)
                },
            ),
            mode_selection_preference: canon_prefs.as_ref().map(|prefs| prefs.mode_selection),
        });
    }

    match canon_prefs {
        Some(canon_prefs) => {
            // Workspace has [canon] config — Canon is the default runtime.
            verify_canon_surface_ready()?;
            Ok(CanonGovernanceResolution {
                governance: Some(GovernanceRuntimeKind::Canon),
                default_risk: canon_prefs.default_risk.map(|risk| {
                    CanonRiskClass::canonicalize_label(&risk).map(str::to_string).unwrap_or(risk)
                }),
                default_zone: canon_prefs.default_zone.map(|zone| {
                    CanonAuthorityZone::canonicalize_label(&zone)
                        .map(str::to_string)
                        .unwrap_or(zone)
                }),
                default_owner: canon_prefs.default_owner.map(|owner| {
                    CanonIntendedPersona::canonicalize_label(&owner)
                        .map(str::to_string)
                        .unwrap_or(owner)
                }),
                mode_selection_preference: Some(canon_prefs.mode_selection),
            })
        }
        None => {
            if mode.is_some() {
                verify_canon_surface_ready()?;
                return Ok(CanonGovernanceResolution {
                    governance: Some(GovernanceRuntimeKind::Canon),
                    default_risk: Some(CanonRiskClass::BoundedImpact.as_str().to_string()),
                    default_zone: Some("engineering".to_string()),
                    default_owner: Some("platform".to_string()),
                    mode_selection_preference: Some(CanonModeSelectionPreference::default()),
                });
            }
            // No [canon] config — backward-compatible local governance.
            Ok(CanonGovernanceResolution {
                governance: None,
                default_risk: None,
                default_zone: None,
                default_owner: None,
                mode_selection_preference: None,
            })
        }
    }
}

fn verify_canon_surface_ready() -> Result<(), RunCommandError> {
    let current_exe = std::env::current_exe().map_err(|error| RunCommandError::CanonSurfaceNotReady {
        repair_actions: format!(
            "Boundline could not determine the current executable before checking Canon: {error}"
        ),
    })?;
    let status = evaluate_canon_install(&current_exe);
    let ready = status.surface_verification.as_ref().is_some_and(|surface| surface.ready);
    if ready {
        return Ok(());
    }

    let repair_actions = status
        .surface_verification
        .as_ref()
        .map(|surface| surface.repair_actions.clone())
        .filter(|actions| !actions.is_empty())
        .unwrap_or(status.suggested_actions);
    let repair_actions =
        if repair_actions.is_empty() { status.message } else { repair_actions.join("; ") };
    Err(RunCommandError::CanonSurfaceNotReady { repair_actions })
}

fn compatibility_terminal_output(body: String) -> String {
    let routing = output::render_route_outcome(&RoutingOutcome {
        mode: RoutingMode::Compatibility,
        source: RoutingSource::ExecutionProfile,
        reason: "compatibility mode was chosen deliberately from the declarative execution path"
            .to_string(),
    });

    format!("{routing}\nexecution_path: fixture_compatibility\n{body}")
}

fn ensure_native_direct_run_can_bootstrap(workspace: &Path) -> Result<(), RunCommandError> {
    let Some(record) =
        FileSessionStore::for_workspace(workspace).load().map_err(RunCommandError::SessionStore)?
    else {
        return Ok(());
    };

    if active_session_has_meaningful_state(&record) {
        return Err(RunCommandError::ActiveSessionConflict);
    }

    Ok(())
}

fn active_session_has_meaningful_state(record: &ActiveSessionRecord) -> bool {
    record.goal.as_deref().map(str::trim).is_some_and(|goal| !goal.is_empty())
        || record.authored_brief.is_some()
        || record.negotiation_packet.is_some()
        || record.active_flow.is_some()
        || record.active_task.is_some()
        || record.goal_plan.is_some()
        || !record.decisions.is_empty()
        || record.latest_trace_ref.is_some()
        || !matches!(record.latest_status, SessionStatus::Initialized)
}

fn native_direct_run_requires_clarification(record: &ActiveSessionRecord) -> bool {
    record.authored_brief.as_ref().and_then(|bundle| bundle.clarification.as_ref()).is_some()
        || record.negotiation_packet.as_ref().is_some_and(|packet| {
            packet.resolution_state
                != crate::domain::negotiation::NegotiationResolutionState::Credible
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use super::{
        RunCommandError, active_session_has_meaningful_state, compatibility_terminal_output,
        native_direct_run_flow_for_mode, native_direct_run_requires_clarification,
        resolve_canon_default_governance,
    };
    use crate::domain::brief::{AuthoredBriefBundle, AuthoredBriefResolutionState};
    use crate::domain::governance::{CanonMode, GovernanceRuntimeKind};
    use crate::domain::negotiation::{NegotiatedDeliveryPacket, NegotiationResolutionState};
    use crate::domain::session::{ActiveSessionRecord, SessionStatus};
    use crate::domain::task::{ClarificationReasonKind, ClarificationRecord, ClarificationStatus};
    use crate::fixture::FixtureRuntimeError;

    fn temp_workspace(prefix: &str) -> Result<PathBuf, String> {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace)
            .map_err(|error| format!("failed to create temp workspace: {error}"))?;
        Ok(workspace)
    }

    fn empty_session_record(workspace: &Path) -> ActiveSessionRecord {
        ActiveSessionRecord {
            session_id: "session-run-tests".to_string(),
            workspace_ref: workspace.display().to_string(),
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
            created_at: 1,
            updated_at: 1,
            governance_lifecycle: None,
            project_scale: None,
            latest_voting: None,
            delight_feedback: None,
            active_execution_run_id: None,
        }
    }

    #[test]
    fn run_helpers_cover_flow_selection_and_session_state() {
        for mode in [
            CanonMode::Requirements,
            CanonMode::Architecture,
            CanonMode::Backlog,
            CanonMode::SystemShaping,
        ] {
            assert_eq!(native_direct_run_flow_for_mode(Some(mode)), "delivery");
        }
        for mode in [CanonMode::Change, CanonMode::Migration, CanonMode::SupplyChainAnalysis] {
            assert_eq!(native_direct_run_flow_for_mode(Some(mode)), "change");
        }
        for mode in [None, Some(CanonMode::Verification)] {
            assert_eq!(native_direct_run_flow_for_mode(mode), "bug-fix");
        }

        let workspace = temp_workspace("boundline-run-helper").unwrap();
        let empty_record = empty_session_record(&workspace);
        assert!(!active_session_has_meaningful_state(&empty_record));

        let mut goal_record = empty_record.clone();
        goal_record.goal = Some("investigate regression".to_string());
        assert!(active_session_has_meaningful_state(&goal_record));

        let mut status_record = empty_record.clone();
        status_record.latest_status = SessionStatus::Planned;
        assert!(active_session_has_meaningful_state(&status_record));

        assert!(!native_direct_run_requires_clarification(&empty_record));

        let mut clarification_record = empty_record.clone();
        clarification_record.authored_brief = Some(AuthoredBriefBundle {
            bundle_id: "bundle-1".to_string(),
            primary_goal_text: Some("investigate regression".to_string()),
            sources: Vec::new(),
            deduplicated_sources: Vec::new(),
            governance_intent: None,
            resolution_state: AuthoredBriefResolutionState::ClarificationRequired,
            goal_quality: Default::default(),
            clarification: Some(ClarificationRecord {
                clarification_id: "clarification-1".to_string(),
                reason_kind: ClarificationReasonKind::MissingContext,
                prompt: "describe the expected outcome".to_string(),
                missing_fields: vec!["acceptance".to_string()],
                questions: Vec::new(),
                blocking_sources: Vec::new(),
                turn_index: 0,
                status: ClarificationStatus::Open,
            }),
            derived_task_draft: None,
            captured_at: 1,
        });
        assert!(native_direct_run_requires_clarification(&clarification_record));

        let mut negotiation_record = empty_record.clone();
        let mut packet = NegotiatedDeliveryPacket::from_goal(
            &negotiation_record.session_id,
            &negotiation_record.workspace_ref,
            "investigate regression",
        );
        packet.resolution_state = NegotiationResolutionState::PendingClarification;
        negotiation_record.negotiation_packet = Some(packet);
        assert!(native_direct_run_requires_clarification(&negotiation_record));

        let terminal_output = compatibility_terminal_output("body".to_string());
        assert!(terminal_output.contains("execution_path: fixture_compatibility"));
        assert!(terminal_output.ends_with("body"));

        fs::remove_dir_all(&workspace).unwrap();
    }

    #[test]
    fn resolve_canon_default_governance_prefers_local_fallbacks() {
        let workspace = temp_workspace("boundline-run-governance").unwrap();

        let no_canon_resolution =
            resolve_canon_default_governance(&workspace, None, None, true).unwrap();
        assert_eq!(no_canon_resolution.governance, Some(GovernanceRuntimeKind::Local));

        let explicit_local = resolve_canon_default_governance(
            &workspace,
            Some(GovernanceRuntimeKind::Local),
            None,
            false,
        )
        .unwrap();
        assert_eq!(explicit_local.governance, Some(GovernanceRuntimeKind::Local));
        assert!(explicit_local.default_risk.is_none());
        assert!(explicit_local.default_zone.is_none());
        assert!(explicit_local.default_owner.is_none());

        let implicit_local =
            resolve_canon_default_governance(&workspace, None, None, false).unwrap();
        assert!(implicit_local.governance.is_none());
        assert!(implicit_local.default_risk.is_none());
        assert!(implicit_local.default_zone.is_none());
        assert!(implicit_local.default_owner.is_none());
        assert!(implicit_local.mode_selection_preference.is_none());

        fs::remove_dir_all(&workspace).unwrap();
    }

    #[test]
    fn run_command_error_message_covers_fixture_runtime_and_wildcard_branches() {
        let no_synthesize_error =
            RunCommandError::FixtureRuntime(FixtureRuntimeError::NoSynthesizeableGoalPlanTarget {
                goal: "implement bounded checkout".to_string(),
                workspace: PathBuf::from("/tmp/workspace"),
            });
        let msg = no_synthesize_error.message();
        assert!(msg.contains("bounded context required"), "{msg}");
        assert!(msg.contains("implement bounded checkout"), "{msg}");

        let canon_error =
            RunCommandError::CanonSurfaceNotReady { repair_actions: "run doctor".to_string() };
        let msg = canon_error.message();
        assert!(msg.contains("Canon governance"), "{msg}");
    }
}
