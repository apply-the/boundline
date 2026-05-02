use serde::{Deserialize, Serialize};

use crate::domain::session::{CompatibilityFollowUpView, ContinuityAuthority, SessionStatusView};
use crate::domain::trace::TraceSummaryView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FollowThroughProjection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

impl FollowThroughProjection {
    pub fn is_empty(&self) -> bool {
        self.guidance.is_none()
            && self.evidence_source.is_none()
            && self.next_action.is_none()
            && self.stop_reason.is_none()
    }

    pub fn projection_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(guidance) = &self.guidance {
            lines.push(format!("follow_through_guidance: {guidance}"));
        }
        if let Some(evidence_source) = &self.evidence_source {
            lines.push(format!("follow_through_evidence_source: {evidence_source}"));
        }
        if let Some(next_action) = &self.next_action {
            lines.push(format!("follow_through_next_action: {next_action}"));
        }
        if let Some(stop_reason) = &self.stop_reason {
            lines.push(format!("follow_through_stop_reason: {stop_reason}"));
        }
        lines
    }

    pub fn from_session_view(view: &SessionStatusView) -> Self {
        if view.continuity_authority == Some(ContinuityAuthority::CompatibilityTrace)
            && let Some(follow_up) = &view.compatibility_follow_up
        {
            return Self {
                guidance: Some(format!(
                    "compatibility follow-up remains {} and should be inspected through the authoritative trace",
                    follow_up.follow_up_mode.as_str()
                )),
                evidence_source: Some("trace:compatibility_follow_up".to_string()),
                next_action: Some(follow_up.next_command.clone()),
                stop_reason: Some(follow_up.terminal_reason.clone()),
            };
        }

        if let Some(governance_next_action) = &view.governance_next_action {
            return Self {
                guidance: Some(format!(
                    "governance state is currently controlling the bounded follow-up: {governance_next_action}"
                )),
                evidence_source: Some("session:governance".to_string()),
                next_action: view.next_command.clone(),
                stop_reason: None,
            };
        }

        if let Some(latest_exhaustion_reason) = &view.latest_exhaustion_reason {
            return Self {
                guidance: Some(
                    "no further bounded action is currently credible until the exhausted path is inspected"
                        .to_string(),
                ),
                evidence_source: Some("session:exhaustion".to_string()),
                next_action: view.next_command.clone(),
                stop_reason: Some(latest_exhaustion_reason.clone()),
            };
        }

        if let Some(latest_selection_headline) = &view.latest_selection_headline {
            return Self {
                guidance: Some(latest_selection_headline.clone()),
                evidence_source: Some("session:recovery".to_string()),
                next_action: view.next_command.clone(),
                stop_reason: None,
            };
        }

        if let Some(latest_selection_reason) = &view.latest_selection_reason {
            return Self {
                guidance: Some(format!(
                    "recovery evidence currently favors this follow-up: {latest_selection_reason}"
                )),
                evidence_source: Some("session:recovery".to_string()),
                next_action: view.next_command.clone(),
                stop_reason: None,
            };
        }

        if let Some(latest_decision_status) = &view.latest_decision_status {
            let target = view.latest_decision_target.as_deref().unwrap_or("current task");
            return Self {
                guidance: Some(format!(
                    "latest decision {latest_decision_status} for {target} is guiding the bounded follow-up"
                )),
                evidence_source: Some("session:decision".to_string()),
                next_action: view.next_command.clone(),
                stop_reason: None,
            };
        }

        if let Some(next_command) = &view.next_command {
            return Self {
                guidance: Some(format!(
                    "persisted session state currently points to `{next_command}` as the next bounded action"
                )),
                evidence_source: Some("session:lifecycle".to_string()),
                next_action: Some(next_command.clone()),
                stop_reason: None,
            };
        }

        Self::default()
    }

    pub fn from_trace_summary(summary: &TraceSummaryView, next_command: Option<&str>) -> Self {
        if let Some(governance_next_action) = &summary.governance_next_action {
            return Self {
                guidance: Some(format!(
                    "trace evidence currently requires this governance follow-up: {governance_next_action}"
                )),
                evidence_source: Some("trace:governance".to_string()),
                next_action: next_command.map(str::to_string),
                stop_reason: None,
            };
        }

        if let Some(decision_line) = summary.decision_timeline.last() {
            return Self {
                guidance: Some(decision_line.clone()),
                evidence_source: Some("trace:decision".to_string()),
                next_action: next_command.map(str::to_string),
                stop_reason: None,
            };
        }

        if let Some(failure_line) = summary.failure_evidence.last() {
            return Self {
                guidance: Some(format!(
                    "authoritative trace evidence requires inspection before another bounded action: {failure_line}"
                )),
                evidence_source: Some("trace:failure".to_string()),
                next_action: next_command.map(str::to_string),
                stop_reason: Some(summary.terminal_reason.message.clone()),
            };
        }

        if let Some(next_command) = next_command {
            return Self {
                guidance: Some(format!(
                    "authoritative trace state currently points to `{next_command}` as the next bounded action"
                )),
                evidence_source: Some("trace:lifecycle".to_string()),
                next_action: Some(next_command.to_string()),
                stop_reason: None,
            };
        }

        Self::default()
    }

    pub fn from_compatibility_follow_up(follow_up: &CompatibilityFollowUpView) -> Self {
        Self {
            guidance: Some(format!(
                "compatibility follow-up remains {} and should be inspected through the authoritative trace",
                follow_up.follow_up_mode.as_str()
            )),
            evidence_source: Some("trace:compatibility_follow_up".to_string()),
            next_action: Some(follow_up.next_command.clone()),
            stop_reason: Some(follow_up.terminal_reason.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FollowThroughProjection;
    use crate::domain::session::{SessionStatus, SessionStatusView};

    #[test]
    fn derives_recovery_guidance_from_session_view() {
        let projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            session_id: "session-1".to_string(),
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
            current_step_id: None,
            current_step_index: None,
            latest_status: SessionStatus::Running,
            execution_path: None,
            latest_trace_ref: None,
            latest_decision_status: Some("failed".to_string()),
            latest_decision_target: Some("verify-fix-add".to_string()),
            latest_changed_files: None,
            latest_workspace_slice: None,
            latest_selection_headline: Some(
                "selected src/lib.rs for the next bounded retry".to_string(),
            ),
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
            latest_governance_blocked_reason: None,
            latest_governance_packet_ref: None,
            latest_governance_packet_source_stage: None,
            latest_governance_packet_binding_reason: None,
            latest_governance_approval: None,
            latest_governance_decision: None,
            latest_governance_candidates: None,
            governance_next_action: None,
            next_command: Some("synod step".to_string()),
            explanation: "current active session state for the workspace".to_string(),
        });

        assert_eq!(
            projection.guidance,
            Some("selected src/lib.rs for the next bounded retry".to_string())
        );
        assert_eq!(projection.evidence_source, Some("session:recovery".to_string()));
        assert_eq!(projection.next_action, Some("synod step".to_string()));
    }
}
