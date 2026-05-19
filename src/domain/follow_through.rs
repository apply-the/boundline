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

        if let Some(delegation) = &view.delegation {
            let packet_ref = delegation.packet_id.as_deref().unwrap_or("unknown-packet");
            let stop_reason = match delegation.target_owner.as_deref() {
                Some(target_owner) => {
                    format!("{}; target_owner={target_owner}", delegation.evidence_summary)
                }
                None => delegation.evidence_summary.clone(),
            };

            return Self {
                guidance: Some(match delegation.packet_state {
                    Some(packet_state) => {
                        format!("{} [{}]", delegation.headline, packet_state.as_str())
                    }
                    None => delegation.headline.clone(),
                }),
                evidence_source: Some(format!("session:delegation_packet:{packet_ref}")),
                next_action: view.next_command.clone(),
                stop_reason: Some(stop_reason),
            };
        }

        if view.context_credibility.as_deref().is_some_and(|credibility| credibility != "credible")
        {
            return Self {
                guidance: Some(view.context_summary.clone().unwrap_or_else(|| {
                    "bounded planning context is not credible enough to continue".to_string()
                })),
                evidence_source: Some("session:context_pack".to_string()),
                next_action: view
                    .next_command
                    .clone()
                    .or_else(|| Some("boundline capture --goal <narrower goal>".to_string())),
                stop_reason: view
                    .context_staleness_reason
                    .clone()
                    .or_else(|| view.context_credibility.clone()),
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
        if let Some(delegation) = &summary.delegation {
            let packet_ref = delegation.packet_id.as_deref().unwrap_or("unknown-packet");
            let stop_reason = match delegation.target_owner.as_deref() {
                Some(target_owner) => {
                    format!("{}; target_owner={target_owner}", delegation.evidence_summary)
                }
                None => delegation.evidence_summary.clone(),
            };

            return Self {
                guidance: Some(match delegation.packet_state {
                    Some(packet_state) => {
                        format!("{} [{}]", delegation.headline, packet_state.as_str())
                    }
                    None => delegation.headline.clone(),
                }),
                evidence_source: Some(format!("trace:delegation_packet:{packet_ref}")),
                next_action: next_command.map(str::to_string),
                stop_reason: Some(stop_reason),
            };
        }

        if summary
            .context_credibility
            .as_deref()
            .is_some_and(|credibility| credibility != "credible")
        {
            return Self {
                guidance: Some(summary.context_summary.clone().unwrap_or_else(|| {
                    "authoritative trace recorded a non-credible bounded context".to_string()
                })),
                evidence_source: Some("trace:context_pack".to_string()),
                next_action: next_command.map(str::to_string),
                stop_reason: summary
                    .context_staleness_reason
                    .clone()
                    .or_else(|| summary.context_credibility.clone()),
            };
        }

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
    use crate::domain::limits::TerminalCondition;
    use crate::domain::session::{
        CompatibilityFollowUpMode, CompatibilityFollowUpView, ContinuityAuthority,
        DelegationContinuityMode, DelegationPacketState, DelegationStatusView, SessionStatus,
        SessionStatusView,
    };
    use crate::domain::task::{TaskStatus, TerminalReason};
    use crate::domain::trace::TraceSummaryView;

    #[test]
    fn derives_recovery_guidance_from_session_view() {
        let projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            session_id: "session-1".to_string(),
            workspace_ref: "/tmp/workspace".to_string(),
            session_started_at: None,
            goal: Some("Fix the failing add test".to_string()),
            advanced_context: None,
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            cluster_delivery_story: None,
            authored_input_summary: None,
            authored_input_sources: None,
            authored_input_deduplicated_sources: None,
            context_summary: None,
            context_credibility: None,
            context_primary_inputs: None,
            context_provenance: None,
            context_staleness_reason: None,
            clarification_headline: None,
            clarification_prompt: None,
            clarification_missing_fields: None,
            requested_governance_runtime: None,
            requested_governance_risk: None,
            requested_governance_zone: None,
            requested_governance_owner: None,
            active_flow: None,
            flow_state: None,
            goal_plan_state: None,
            goal_plan_revision: None,
            planning_rationale: None,
            verification_strategy: None,
            active_workflow: None,
            workflow_phase: None,
            workflow_next_action: None,
            continuity_authority: None,
            delegation: None,
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
            latest_checkpoint_id: None,
            latest_checkpoint_scope: None,
            latest_checkpoint_restore_command: None,
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
            latest_review_council_profile: None,
            latest_review_independence_state: None,
            latest_review_stop_semantics: None,
            latest_review_selection_summary: None,
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
            latest_governance_confidence_level: None,
            latest_governance_admission_effect: None,
            latest_governance_confidence_summary: None,
            governance_next_action: None,
            governance_lifecycle_runtime: None,
            governance_lifecycle_opt_out: None,
            governance_lifecycle_mode_selection: None,
            governance_lifecycle_selected_mode: None,
            latest_reasoning_profile: None,
            project_scale_path: None,
            project_scale_current_stage: None,
            project_scale_next_action: None,
            project_scale_checkpoint_refs: None,
            latest_voting_trigger: None,
            latest_voting_result: None,
            latest_voting_adjudication: None,
            latest_voting_reviewed_evidence: None,
            latest_voting_blocking: None,
            latest_voting_next_action: None,
            delight_feedback: None,
            next_command: Some("boundline step".to_string()),
            explanation: "current active session state for the workspace".to_string(),
        });

        assert_eq!(
            projection.guidance,
            Some("selected src/lib.rs for the next bounded retry".to_string())
        );
        assert_eq!(projection.evidence_source, Some("session:recovery".to_string()));
        assert_eq!(projection.next_action, Some("boundline step".to_string()));
    }

    #[test]
    fn follow_through_projection_reports_empty_state_and_projection_lines() {
        let empty = FollowThroughProjection::default();
        assert!(empty.is_empty());
        assert!(empty.projection_lines().is_empty());

        let projection = FollowThroughProjection {
            guidance: Some("inspect the authoritative trace".to_string()),
            evidence_source: Some("trace:context_pack".to_string()),
            next_action: Some("boundline inspect".to_string()),
            stop_reason: Some("stale".to_string()),
        };

        assert!(!projection.is_empty());
        assert_eq!(
            projection.projection_lines(),
            vec![
                "follow_through_guidance: inspect the authoritative trace".to_string(),
                "follow_through_evidence_source: trace:context_pack".to_string(),
                "follow_through_next_action: boundline inspect".to_string(),
                "follow_through_stop_reason: stale".to_string(),
            ]
        );
    }

    #[test]
    fn follow_through_projection_prefers_compatibility_and_context_follow_up() {
        let compatibility_projection = FollowThroughProjection::from_session_view(
            &SessionStatusView {
                continuity_authority: Some(ContinuityAuthority::CompatibilityTrace),
                compatibility_follow_up: Some(CompatibilityFollowUpView {
                    follow_up_mode: CompatibilityFollowUpMode::InspectOnly,
                    trace_ref: "/tmp/workspace/.boundline/traces/compat.json".to_string(),
                    routing_summary: "routing: compatibility (execution_profile)".to_string(),
                    execution_condition:
                        "execution_condition: blocked - inspect the authoritative trace".to_string(),
                    terminal_status: TaskStatus::Failed,
                    terminal_reason: "compatibility run failed".to_string(),
                    next_command: "boundline inspect --workspace /tmp/workspace".to_string(),
                }),
                ..SessionStatusView::default()
            },
        );

        assert_eq!(
            compatibility_projection.guidance,
            Some(
                "compatibility follow-up remains inspect_only and should be inspected through the authoritative trace"
                    .to_string(),
            )
        );
        assert_eq!(
            compatibility_projection.evidence_source,
            Some("trace:compatibility_follow_up".to_string())
        );

        let context_projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            context_summary: None,
            context_credibility: Some("stale".to_string()),
            context_staleness_reason: Some("trace snapshot is stale".to_string()),
            next_command: None,
            ..SessionStatusView::default()
        });

        assert_eq!(
            context_projection.guidance,
            Some("bounded planning context is not credible enough to continue".to_string())
        );
        assert_eq!(context_projection.evidence_source, Some("session:context_pack".to_string()));
        assert_eq!(
            context_projection.next_action,
            Some("boundline capture --goal <narrower goal>".to_string())
        );
        assert_eq!(context_projection.stop_reason, Some("trace snapshot is stale".to_string()));
    }

    #[test]
    fn follow_through_projection_covers_session_trace_and_lifecycle_branches() {
        let governance_projection =
            FollowThroughProjection::from_session_view(&SessionStatusView {
                governance_next_action: Some(
                    "wait for approval and rerun boundline status".to_string(),
                ),
                next_command: Some("boundline status".to_string()),
                ..SessionStatusView::default()
            });
        assert_eq!(governance_projection.evidence_source, Some("session:governance".to_string()));

        let exhaustion_projection =
            FollowThroughProjection::from_session_view(&SessionStatusView {
                latest_exhaustion_reason: Some("retry limits exhausted".to_string()),
                next_command: Some("boundline inspect".to_string()),
                ..SessionStatusView::default()
            });
        assert_eq!(exhaustion_projection.stop_reason, Some("retry limits exhausted".to_string()));

        let selection_projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            latest_selection_reason: Some(
                "selected src/lib.rs based on failing test evidence".to_string(),
            ),
            next_command: Some("boundline step".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(
            selection_projection.guidance,
            Some(
                "recovery evidence currently favors this follow-up: selected src/lib.rs based on failing test evidence"
                    .to_string(),
            )
        );

        let decision_projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            latest_decision_status: Some("failed".to_string()),
            latest_decision_target: Some("src/lib.rs".to_string()),
            next_command: Some("boundline step".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(
            decision_projection.guidance,
            Some(
                "latest decision failed for src/lib.rs is guiding the bounded follow-up"
                    .to_string(),
            )
        );

        let lifecycle_projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            next_command: Some("boundline step".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(lifecycle_projection.evidence_source, Some("session:lifecycle".to_string()));
    }

    #[test]
    fn follow_through_projection_covers_trace_summary_branches() {
        let context_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView {
                context_summary: None,
                context_credibility: Some("insufficient".to_string()),
                context_staleness_reason: None,
                terminal_status: TaskStatus::Failed,
                terminal_reason: TerminalReason::new(
                    TerminalCondition::UnrecoverableError,
                    "trace failed",
                    None,
                ),
                ..TraceSummaryView::default()
            },
            Some("boundline capture --goal <narrower goal>"),
        );
        assert_eq!(
            context_projection.guidance,
            Some("authoritative trace recorded a non-credible bounded context".to_string())
        );
        assert_eq!(context_projection.stop_reason, Some("insufficient".to_string()));

        let governance_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView {
                governance_next_action: Some(
                    "resolve the governance blocker, then rerun boundline step".to_string(),
                ),
                ..TraceSummaryView::default()
            },
            Some("boundline step"),
        );
        assert_eq!(governance_projection.evidence_source, Some("trace:governance".to_string()));

        let decision_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView {
                decision_timeline: vec!["decision_status: decision-1 verified".to_string()],
                ..TraceSummaryView::default()
            },
            Some("boundline step"),
        );
        assert_eq!(
            decision_projection.guidance,
            Some("decision_status: decision-1 verified".to_string())
        );

        let failure_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView {
                failure_evidence: vec!["decision-1 src/lib.rs: test failed".to_string()],
                terminal_status: TaskStatus::Failed,
                terminal_reason: TerminalReason::new(
                    TerminalCondition::UnrecoverableError,
                    "trace failed",
                    None,
                ),
                ..TraceSummaryView::default()
            },
            Some("boundline inspect"),
        );
        assert_eq!(failure_projection.stop_reason, Some("trace failed".to_string()));

        let lifecycle_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView::default(),
            Some("boundline inspect"),
        );
        assert_eq!(lifecycle_projection.evidence_source, Some("trace:lifecycle".to_string()));
    }

    #[test]
    fn follow_through_projection_covers_delegation_branches() {
        let session_projection = FollowThroughProjection::from_session_view(&SessionStatusView {
            delegation: Some(DelegationStatusView {
                mode: DelegationContinuityMode::EscalationRequired,
                packet_id: Some("packet-123".to_string()),
                packet_kind: None,
                packet_state: Some(DelegationPacketState::Resolved),
                target_owner: Some("review-council".to_string()),
                headline: "escalation required".to_string(),
                evidence_summary: "governance packet requires escalation".to_string(),
            }),
            next_command: Some("boundline inspect --delegation packet-123".to_string()),
            ..SessionStatusView::default()
        });
        assert_eq!(session_projection.guidance, Some("escalation required [resolved]".to_string()));
        assert_eq!(
            session_projection.evidence_source,
            Some("session:delegation_packet:packet-123".to_string())
        );
        assert_eq!(
            session_projection.stop_reason,
            Some("governance packet requires escalation; target_owner=review-council".to_string())
        );

        let trace_projection = FollowThroughProjection::from_trace_summary(
            &TraceSummaryView {
                delegation: Some(DelegationStatusView {
                    mode: DelegationContinuityMode::HandoffRequired,
                    packet_id: Some("packet-456".to_string()),
                    packet_kind: None,
                    packet_state: Some(DelegationPacketState::Active),
                    target_owner: Some("delivery-owner".to_string()),
                    headline: "handoff required".to_string(),
                    evidence_summary: "packet is awaiting owner handoff".to_string(),
                }),
                ..TraceSummaryView::default()
            },
            Some("boundline step --resume packet-456"),
        );
        assert_eq!(trace_projection.guidance, Some("handoff required [active]".to_string()));
        assert_eq!(
            trace_projection.evidence_source,
            Some("trace:delegation_packet:packet-456".to_string())
        );
        assert_eq!(
            trace_projection.stop_reason,
            Some("packet is awaiting owner handoff; target_owner=delivery-owner".to_string())
        );
    }

    #[test]
    fn follow_through_projection_builds_from_compatibility_follow_up() {
        let projection =
            FollowThroughProjection::from_compatibility_follow_up(&CompatibilityFollowUpView {
                follow_up_mode: CompatibilityFollowUpMode::Resumable,
                trace_ref: "/tmp/workspace/.boundline/traces/compat.json".to_string(),
                routing_summary: "routing: compatibility (execution_profile)".to_string(),
                execution_condition: "execution_condition: waiting - inspect trace".to_string(),
                terminal_status: TaskStatus::Failed,
                terminal_reason: "compatibility trace failed".to_string(),
                next_command: "boundline inspect --workspace /tmp/workspace".to_string(),
            });

        assert_eq!(
            projection.guidance,
            Some(
                "compatibility follow-up remains resumable and should be inspected through the authoritative trace"
                    .to_string(),
            )
        );
        assert_eq!(projection.evidence_source, Some("trace:compatibility_follow_up".to_string()));
    }
}
