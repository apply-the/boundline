use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use super::session_store::{FileSessionStore, SessionStore, SessionStoreError};
use super::trace_store::{FileTraceStore, TraceStore, TraceStoreError};
use crate::domain::dashboard::{
    ContextPackPanelItem, DashboardActionKind, DashboardActionOption, DashboardAuthority,
    DashboardBrandMark, DashboardColorProfile, DashboardDiagnosticItem, DashboardExpectedResult,
    DashboardPanels, DashboardSessionView, DashboardSnapshot, DegradedDashboardState,
    DegradedReason, DegradedSeverity, EvidencePanelItem, ExecutionCondition, GoalPlanPanel,
    GovernedReferencePanelItem, RuntimeEventProjection, UnavailablePanelFact,
};
use crate::domain::session::{ActiveSessionRecord, SessionStatus, execution_path_text};
use crate::domain::trace::{TraceEventType, current_timestamp_millis};

const WORDMARK: &str = "boundline";
const ROUTE_OWNER_RUNTIME: &str = "runtime";
const FALLBACK_STATUS: &str = "boundline status";
const FALLBACK_INSPECT: &str = "boundline inspect";

#[derive(Debug, Clone)]
pub struct DashboardStateAssembler {
    workspace: PathBuf,
}

impl DashboardStateAssembler {
    pub fn for_workspace(workspace: impl AsRef<Path>) -> Self {
        Self { workspace: workspace.as_ref().to_path_buf() }
    }

    pub fn snapshot(&self, no_color: bool) -> Result<DashboardSnapshot, DashboardStateError> {
        if !self.workspace.exists() || !self.workspace.is_dir() {
            return Ok(self.degraded_snapshot(
                no_color,
                DegradedReason::InvalidWorkspace,
                DegradedSeverity::Blocked,
                "Workspace is not readable.",
                vec![format!("boundline init --workspace {}", self.workspace.display())],
            ));
        }

        let store = FileSessionStore::for_workspace(&self.workspace);
        let record = match store.load() {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Ok(self.degraded_snapshot(
                    no_color,
                    DegradedReason::MissingActiveSession,
                    DegradedSeverity::Info,
                    "Start or capture a session first.",
                    vec![
                        format!("boundline start --workspace {}", self.workspace.display()),
                        format!("{FALLBACK_STATUS} --workspace {}", self.workspace.display()),
                    ],
                ));
            }
            Err(SessionStoreError::Deserialize(_)) | Err(SessionStoreError::InvalidRecord(_)) => {
                return Ok(self.degraded_snapshot(
                    no_color,
                    DegradedReason::InvalidSessionJson,
                    DegradedSeverity::Blocked,
                    "Repair or recreate the active session record.",
                    vec![
                        format!("{FALLBACK_STATUS} --workspace {}", self.workspace.display()),
                        format!("{FALLBACK_INSPECT} --workspace {}", self.workspace.display()),
                    ],
                ));
            }
            Err(error) => return Err(DashboardStateError::SessionStore(error)),
        };

        Ok(self.active_snapshot(no_color, &record))
    }

    fn active_snapshot(&self, no_color: bool, record: &ActiveSessionRecord) -> DashboardSnapshot {
        let session = self.session_view(record);
        let timeline = self.timeline(record).unwrap_or_default();
        let panels = panels_for_record(record);
        let actions = actions_for_session(&session, record.updated_at);
        DashboardSnapshot {
            snapshot_id: format!("snapshot-{}", Uuid::new_v4()),
            workspace_ref: self.workspace.to_string_lossy().into_owned(),
            captured_at: format!("unix-ms:{}", current_timestamp_millis()),
            authority: DashboardAuthority::SessionNative,
            session_revision: Some(record.updated_at),
            session: Some(session),
            timeline,
            panels,
            actions,
            degraded_state: None,
            branding: brand_mark(no_color),
        }
    }

    fn degraded_snapshot(
        &self,
        no_color: bool,
        reason: DegradedReason,
        severity: DegradedSeverity,
        recovery_hint: &str,
        available_commands: Vec<String>,
    ) -> DashboardSnapshot {
        DashboardSnapshot {
            snapshot_id: format!("snapshot-{}", Uuid::new_v4()),
            workspace_ref: self.workspace.to_string_lossy().into_owned(),
            captured_at: format!("unix-ms:{}", current_timestamp_millis()),
            authority: DashboardAuthority::Degraded,
            session_revision: None,
            session: None,
            timeline: Vec::new(),
            panels: DashboardPanels::empty(),
            actions: Vec::new(),
            degraded_state: Some(DegradedDashboardState {
                reason,
                severity,
                available_commands,
                unavailable_panels: vec!["goal_plan".to_string(), "evidence".to_string()],
                recovery_hint: Some(recovery_hint.to_string()),
            }),
            branding: brand_mark(no_color),
        }
    }

    fn session_view(&self, record: &ActiveSessionRecord) -> DashboardSessionView {
        let condition = execution_condition(record);
        let blocking_reason = blocking_reason(record, condition);
        DashboardSessionView {
            session_id: record.session_id.clone(),
            goal: record.goal.clone().unwrap_or_else(|| "No goal captured".to_string()),
            route_kind: execution_path_text(record).unwrap_or_else(|| "session_native".to_string()),
            route_owner: ROUTE_OWNER_RUNTIME.to_string(),
            active_flow: record.active_flow.as_ref().map(|flow| flow.flow_name.clone()),
            flow_state: record.goal_plan.as_ref().map(|plan| plan.flow_state().summary_text()),
            goal_plan_state: record
                .goal_plan
                .as_ref()
                .map(|plan| plan.proposal_state_text().to_string()),
            goal_plan_revision: record.goal_plan.as_ref().map(|plan| plan.proposal_revision),
            current_stage: record.active_flow.as_ref().map(|flow| flow.current_stage_id.clone()),
            current_step_id: record
                .active_task
                .as_ref()
                .and_then(|task| task.plan.current_step().map(|step| step.id.clone())),
            current_step_index: record
                .active_task
                .as_ref()
                .map(|task| task.plan.current_step_index),
            execution_condition: condition,
            latest_status: session_status_text(record.latest_status).to_string(),
            next_action_label: next_action_label(record, condition),
            next_command: next_command(record, condition, &self.workspace),
            blocking_reason,
            compatibility_context: compatibility_context(record),
        }
    }

    fn timeline(
        &self,
        record: &ActiveSessionRecord,
    ) -> Result<Vec<RuntimeEventProjection>, DashboardStateError> {
        let store = FileTraceStore::for_workspace(&self.workspace);
        let trace_path = match record.latest_trace_ref.as_deref() {
            Some(trace_ref) => PathBuf::from(trace_ref),
            None => match store.latest().map_err(DashboardStateError::TraceStore)? {
                Some(path) => path,
                None => return Ok(Vec::new()),
            },
        };
        if !trace_path.exists() {
            return Ok(vec![RuntimeEventProjection {
                event_id: "stale-trace-reference".to_string(),
                event_kind: "degraded".to_string(),
                occurred_at: format!("unix-ms:{}", current_timestamp_millis()),
                stage: None,
                step_id: None,
                status: "warning".to_string(),
                headline: "Latest trace reference is unavailable".to_string(),
                evidence_refs: Vec::new(),
                trace_ref: Some(trace_path.to_string_lossy().into_owned()),
                details: vec!["normal inspect fallback remains available".to_string()],
            }]);
        }

        let trace = store.load(&trace_path).map_err(DashboardStateError::TraceStore)?;
        Ok(trace
            .events
            .iter()
            .rev()
            .take(8)
            .map(|event| RuntimeEventProjection {
                event_id: event.event_id.clone(),
                event_kind: trace_event_kind(event.event_type).to_string(),
                occurred_at: format!("unix-ms:{}", event.recorded_at),
                stage: None,
                step_id: event.step_id.clone(),
                status: "recorded".to_string(),
                headline: trace_event_headline(event.event_type).to_string(),
                evidence_refs: vec![format!("trace:{}", trace.task_id)],
                trace_ref: Some(trace_path.to_string_lossy().into_owned()),
                details: Vec::new(),
            })
            .collect())
    }
}

pub fn brand_mark(no_color: bool) -> DashboardBrandMark {
    DashboardBrandMark {
        wordmark_lines: vec![WORDMARK.to_string()],
        color_profile: if no_color {
            DashboardColorProfile::Monochrome
        } else {
            DashboardColorProfile::Color
        },
        min_width: 20,
        fallback_label: WORDMARK.to_string(),
    }
}

fn panels_for_record(record: &ActiveSessionRecord) -> DashboardPanels {
    let mut panels = DashboardPanels::empty();
    panels.goal_plan = record.goal_plan.as_ref().map(|plan| GoalPlanPanel {
        revision: plan.proposal_revision,
        state: plan.proposal_state_text().to_string(),
        verification_strategy: plan.verification_strategy.clone(),
        targets: plan.tasks.iter().map(|task| task.target.clone()).collect(),
    });
    if let Some(plan) = &record.goal_plan
        && let Some(context_pack) = &plan.context_pack
    {
        panels.context_pack = context_pack
            .inputs
            .iter()
            .map(|input| ContextPackPanelItem {
                reason: input.rationale.clone(),
                source: input.source.clone(),
                budget: None,
                authority: input.kind.as_str().to_string(),
                evidence_ref: input.reference.clone(),
            })
            .collect();
        panels.evidence = context_pack
            .primary_references()
            .into_iter()
            .map(|reference| EvidencePanelItem {
                label: "selected context".to_string(),
                evidence_ref: reference,
                status: context_pack.credibility.as_str().to_string(),
            })
            .collect();
    }
    if record.goal_plan.is_none() {
        panels.context_degradation.push(UnavailablePanelFact {
            label: "goal_plan".to_string(),
            reason: "no goal plan is currently persisted".to_string(),
        });
    }
    if record
        .active_task
        .as_ref()
        .and_then(|task| {
            task.context.state.get("latest_governance_packet_ref").and_then(|value| value.as_str())
        })
        .is_some()
        && let Some(task) = &record.active_task
        && let Some(reference) =
            task.context.state.get("latest_governance_packet_ref").and_then(|value| value.as_str())
    {
        panels.governed_references.push(GovernedReferencePanelItem {
            reference: reference.to_string(),
            readiness: "available".to_string(),
            provenance: "boundline_task_context".to_string(),
            approval_cue: task
                .context
                .state
                .get("latest_governance_approval")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            read_only: true,
        });
    }
    panels.diagnostics.push(DashboardDiagnosticItem {
        category: "workspace".to_string(),
        status: "readable".to_string(),
        details: "dashboard snapshot assembled from Boundline-owned state".to_string(),
    });
    panels
}

fn actions_for_session(
    session: &DashboardSessionView,
    revision: u64,
) -> Vec<DashboardActionOption> {
    match session.execution_condition {
        ExecutionCondition::Ready => vec![action(
            DashboardActionKind::Continue,
            "Run",
            "Continue bounded execution",
            revision,
            DashboardExpectedResult::RunningOrTerminal,
        )],
        ExecutionCondition::Waiting => vec![action(
            DashboardActionKind::Confirm,
            "Confirm",
            "Confirm the proposed plan",
            revision,
            DashboardExpectedResult::PlannedOrConfirmed,
        )],
        ExecutionCondition::Failed | ExecutionCondition::Exhausted => vec![action(
            DashboardActionKind::Recover,
            "Recover",
            "Choose a bounded recovery path",
            revision,
            DashboardExpectedResult::RecoverySelected,
        )],
        _ => vec![DashboardActionOption {
            action_kind: DashboardActionKind::InspectOnly,
            label: "Inspect".to_string(),
            description: "Inspect current state without mutation".to_string(),
            requires_reason: false,
            requires_confirmation: false,
            target_session_revision: None,
            expected_result: DashboardExpectedResult::FocusChanged,
            disabled_reason: None,
        }],
    }
}

fn action(
    action_kind: DashboardActionKind,
    label: &str,
    description: &str,
    revision: u64,
    expected_result: DashboardExpectedResult,
) -> DashboardActionOption {
    DashboardActionOption {
        action_kind,
        label: label.to_string(),
        description: description.to_string(),
        requires_reason: matches!(action_kind, DashboardActionKind::Reject),
        requires_confirmation: action_kind.mutates_state(),
        target_session_revision: Some(revision),
        expected_result,
        disabled_reason: None,
    }
}

fn execution_condition(record: &ActiveSessionRecord) -> ExecutionCondition {
    if record.goal_plan.as_ref().is_some_and(|plan| plan.requires_confirmation()) {
        return ExecutionCondition::Waiting;
    }
    match record.latest_status {
        SessionStatus::Initialized | SessionStatus::GoalCaptured | SessionStatus::Planned => {
            ExecutionCondition::Ready
        }
        SessionStatus::Running => ExecutionCondition::Ready,
        SessionStatus::Succeeded => ExecutionCondition::Complete,
        SessionStatus::Failed => ExecutionCondition::Failed,
        SessionStatus::Exhausted => ExecutionCondition::Exhausted,
        SessionStatus::Aborted | SessionStatus::Invalid => ExecutionCondition::Invalid,
    }
}

fn blocking_reason(record: &ActiveSessionRecord, condition: ExecutionCondition) -> Option<String> {
    if record.goal_plan.as_ref().is_some_and(|plan| plan.requires_confirmation()) {
        return Some("Plan confirmation is required before execution.".to_string());
    }
    match condition {
        ExecutionCondition::Failed
        | ExecutionCondition::Exhausted
        | ExecutionCondition::Invalid => Some(record.latest_terminal_reason.as_ref().map_or_else(
            || "Session is not ready for normal progression.".to_string(),
            |reason| reason.message.clone(),
        )),
        ExecutionCondition::Blocked
        | ExecutionCondition::Waiting
        | ExecutionCondition::Degraded => {
            Some("Session is waiting on a bounded follow-up.".to_string())
        }
        ExecutionCondition::Ready | ExecutionCondition::Complete => None,
    }
}

fn next_action_label(record: &ActiveSessionRecord, condition: ExecutionCondition) -> String {
    if record.goal_plan.as_ref().is_some_and(|plan| plan.requires_confirmation()) {
        return "Confirm proposed plan".to_string();
    }
    match condition {
        ExecutionCondition::Ready => "Continue bounded execution".to_string(),
        ExecutionCondition::Complete => "Inspect completed session".to_string(),
        ExecutionCondition::Failed | ExecutionCondition::Exhausted => {
            "Recover or inspect".to_string()
        }
        _ => "Inspect current state".to_string(),
    }
}

fn next_command(
    record: &ActiveSessionRecord,
    condition: ExecutionCondition,
    workspace: &Path,
) -> String {
    let workspace_arg = workspace.display();
    if record.goal_plan.as_ref().is_some_and(|plan| plan.requires_confirmation()) {
        return format!("boundline plan --confirm --workspace {workspace_arg}");
    }
    match (record.latest_status, condition) {
        (SessionStatus::Initialized, _) => {
            format!("boundline capture --workspace {workspace_arg}")
        }
        (SessionStatus::GoalCaptured, _) => format!("boundline plan --workspace {workspace_arg}"),
        (_, ExecutionCondition::Ready) => format!("boundline run --workspace {workspace_arg}"),
        (_, ExecutionCondition::Complete) => {
            format!("{FALLBACK_INSPECT} --workspace {workspace_arg}")
        }
        (_, _) => format!("{FALLBACK_STATUS} --workspace {workspace_arg}"),
    }
}

fn compatibility_context(record: &ActiveSessionRecord) -> Option<String> {
    record.active_task.as_ref().map(|task| {
        format!("compatibility task {} remains tied to session {}", task.id, record.session_id)
    })
}

fn session_status_text(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Initialized => "initialized",
        SessionStatus::GoalCaptured => "goal_captured",
        SessionStatus::Planned => "planned",
        SessionStatus::Running => "running",
        SessionStatus::Succeeded => "succeeded",
        SessionStatus::Failed => "failed",
        SessionStatus::Exhausted => "exhausted",
        SessionStatus::Aborted => "aborted",
        SessionStatus::Invalid => "invalid",
    }
}

fn trace_event_kind(event_type: TraceEventType) -> &'static str {
    match event_type {
        TraceEventType::GoalPlanCreated => "plan",
        TraceEventType::DecisionCreated
        | TraceEventType::DecisionDispatched
        | TraceEventType::DecisionVerified
        | TraceEventType::DecisionFailed
        | TraceEventType::DecisionRecovered => "action",
        TraceEventType::CheckpointCreated => "checkpoint",
        TraceEventType::TerminalRecorded => "terminal",
        TraceEventType::RetryScheduled
        | TraceEventType::StageRetryScheduled
        | TraceEventType::Replanned
        | TraceEventType::StageReplanned => "replan",
        _ => "session",
    }
}

fn trace_event_headline(event_type: TraceEventType) -> &'static str {
    match event_type {
        TraceEventType::GoalPlanCreated => "Goal plan created",
        TraceEventType::DecisionVerified => "Decision verified",
        TraceEventType::DecisionFailed => "Decision failed",
        TraceEventType::DecisionRecovered => "Decision recovered",
        TraceEventType::CheckpointCreated => "Checkpoint created",
        TraceEventType::TerminalRecorded => "Terminal outcome recorded",
        _ => "Runtime event recorded",
    }
}

#[derive(Debug, Error)]
pub enum DashboardStateError {
    #[error("session store error: {0}")]
    SessionStore(SessionStoreError),
    #[error("trace store error: {0}")]
    TraceStore(TraceStoreError),
}
