use std::path::{Path, PathBuf};

use crate::domain::dashboard::{
    DashboardActionKind, DashboardActionOutcome, DashboardActionRequest, DashboardActionResult,
    DashboardRefusalReason,
};

use super::dashboard_state::{DashboardStateAssembler, DashboardStateError};

const FALLBACK_STATUS: &str = "boundline status";

#[derive(Debug, Clone)]
pub struct DashboardActionDispatcher {
    assembler: DashboardStateAssembler,
    workspace: PathBuf,
}

impl DashboardActionDispatcher {
    pub fn for_workspace(workspace: impl AsRef<Path>) -> Self {
        let workspace = workspace.as_ref().to_path_buf();
        Self { assembler: DashboardStateAssembler::for_workspace(&workspace), workspace }
    }

    pub fn apply(
        &self,
        request: &DashboardActionRequest,
    ) -> Result<DashboardActionResult, DashboardStateError> {
        let snapshot = self.assembler.snapshot(true)?;
        if snapshot.degraded_state.is_some() {
            return Ok(refused_result(
                request,
                DashboardRefusalReason::DashboardDegraded,
                Some(format!("{FALLBACK_STATUS} --workspace {}", self.workspace.display())),
                "Dashboard is degraded. Refresh state through the normal command path.",
            ));
        }

        if request.action_kind == DashboardActionKind::InspectOnly {
            return Ok(DashboardActionResult {
                request_id: request.request_id.clone(),
                outcome: DashboardActionOutcome::Applied,
                state_transition: Some("focus_changed".to_string()),
                next_snapshot_ref: Some(snapshot.snapshot_id),
                next_command: snapshot.session.as_ref().map(|session| session.next_command.clone()),
                trace_refs: Vec::new(),
                refusal_reason: None,
                operator_message: "Focused the requested dashboard panel.".to_string(),
            });
        }

        if request.target_session_revision != snapshot.session_revision {
            return Ok(refused_result(
                request,
                DashboardRefusalReason::StaleSessionRevision,
                snapshot.session.as_ref().map(|session| session.next_command.clone()),
                "The session changed after this dashboard view was rendered. Refresh first.",
            ));
        }

        if matches!(request.action_kind, DashboardActionKind::Reject | DashboardActionKind::Replan)
            && request.operator_reason.as_deref().unwrap_or_default().trim().is_empty()
        {
            return Ok(refused_result(
                request,
                DashboardRefusalReason::MissingRequiredContext,
                snapshot.session.as_ref().map(|session| session.next_command.clone()),
                "This action requires a bounded operator reason.",
            ));
        }

        Ok(DashboardActionResult {
            request_id: request.request_id.clone(),
            outcome: DashboardActionOutcome::Applied,
            state_transition: Some(action_transition(request.action_kind).to_string()),
            next_snapshot_ref: Some(snapshot.snapshot_id),
            next_command: snapshot.session.as_ref().map(|session| session.next_command.clone()),
            trace_refs: snapshot
                .timeline
                .iter()
                .filter_map(|event| event.trace_ref.clone())
                .collect(),
            refusal_reason: None,
            operator_message: "Action accepted through the Boundline runtime boundary.".to_string(),
        })
    }
}

fn refused_result(
    request: &DashboardActionRequest,
    reason: DashboardRefusalReason,
    next_command: Option<String>,
    message: &str,
) -> DashboardActionResult {
    DashboardActionResult {
        request_id: request.request_id.clone(),
        outcome: DashboardActionOutcome::Refused,
        state_transition: None,
        next_snapshot_ref: None,
        next_command,
        trace_refs: Vec::new(),
        refusal_reason: Some(reason),
        operator_message: message.to_string(),
    }
}

fn action_transition(action_kind: DashboardActionKind) -> &'static str {
    match action_kind {
        DashboardActionKind::Confirm => "plan_confirmed",
        DashboardActionKind::Reject => "plan_rejected",
        DashboardActionKind::Replan => "replan_requested",
        DashboardActionKind::Recover => "recovery_requested",
        DashboardActionKind::Launch => "session_launch_requested",
        DashboardActionKind::Continue => "continue_requested",
        DashboardActionKind::InspectOnly => "focus_changed",
    }
}
