use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::brief::AuthoredBriefBundle;
use crate::domain::trace::current_timestamp_millis;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationResolutionState {
    Credible,
    PendingClarification,
    Conflicting,
    Blocked,
}

impl NegotiationResolutionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::PendingClarification => "pending_clarification",
            Self::Conflicting => "conflicting",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationConstraintKind {
    Scope,
    Acceptance,
    Risk,
    Governance,
    ExecutionLimit,
    Routing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationConstraintSource {
    Goal,
    Brief,
    GovernanceIntent,
    WorkspaceSignal,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationConstraintState {
    Binding,
    Proposed,
    Conflicting,
    Satisfied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceBoundary {
    pub success_headline: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_outcomes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_outcomes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_evidence: Vec<String>,
    pub bounded_scope_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiationConstraint {
    pub constraint_id: String,
    pub kind: NegotiationConstraintKind,
    pub summary: String,
    pub source: NegotiationConstraintSource,
    pub state: NegotiationConstraintState,
    pub blocks_planning: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeoffDecision {
    pub prioritized_constraint_id: String,
    pub rejected_alternative_summary: String,
    pub rationale: String,
    pub surfaced_as_blocker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiatedDeliveryPacket {
    pub negotiation_id: String,
    pub session_id: String,
    pub workspace_ref: String,
    pub goal_summary: String,
    pub acceptance_boundary: AcceptanceBoundary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<NegotiationConstraint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tradeoff: Option<TradeoffDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_headline: Option<String>,
    pub resolution_state: NegotiationResolutionState,
    pub source_summary: String,
    pub created_at: u64,
}

impl NegotiatedDeliveryPacket {
    pub fn from_goal(session_id: &str, workspace_ref: &str, goal: &str) -> Self {
        let goal_summary = goal.trim().to_string();
        let acceptance_boundary = AcceptanceBoundary {
            success_headline: format!("deliver the bounded outcome: {goal_summary}"),
            required_outcomes: vec![goal_summary.clone()],
            excluded_outcomes: vec![
                "unbounded scope expansion outside the active session goal".to_string(),
            ],
            expected_evidence: vec![
                "planned or executed output aligned with the requested bounded outcome".to_string(),
            ],
            bounded_scope_summary:
                "bounded to the active session goal and existing session execution limits"
                    .to_string(),
        };

        let constraints = vec![
            NegotiationConstraint {
                constraint_id: Uuid::new_v4().to_string(),
                kind: NegotiationConstraintKind::Acceptance,
                summary: format!("preserve the requested outcome: {goal_summary}"),
                source: NegotiationConstraintSource::Goal,
                state: NegotiationConstraintState::Binding,
                blocks_planning: false,
            },
            NegotiationConstraint {
                constraint_id: Uuid::new_v4().to_string(),
                kind: NegotiationConstraintKind::ExecutionLimit,
                summary: "respect the current bounded session execution limits".to_string(),
                source: NegotiationConstraintSource::Default,
                state: NegotiationConstraintState::Binding,
                blocks_planning: false,
            },
        ];

        Self {
            negotiation_id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            workspace_ref: workspace_ref.to_string(),
            goal_summary,
            acceptance_boundary,
            constraints,
            tradeoff: None,
            clarification_headline: None,
            resolution_state: NegotiationResolutionState::Credible,
            source_summary: "direct goal defaults".to_string(),
            created_at: current_timestamp_millis(),
        }
    }

    pub fn from_authored_brief(
        session_id: &str,
        workspace_ref: &str,
        goal: &str,
        bundle: &AuthoredBriefBundle,
    ) -> Self {
        let mut packet = Self::from_goal(session_id, workspace_ref, goal);
        packet.source_summary = bundle.summary_text();

        if let Some(clarification_headline) = bundle.clarification_headline() {
            packet.clarification_headline = Some(clarification_headline.clone());
            packet.resolution_state = NegotiationResolutionState::PendingClarification;
            packet.constraints.push(NegotiationConstraint {
                constraint_id: Uuid::new_v4().to_string(),
                kind: NegotiationConstraintKind::Scope,
                summary: bundle.clarification_prompt().unwrap_or(clarification_headline),
                source: NegotiationConstraintSource::Brief,
                state: NegotiationConstraintState::Conflicting,
                blocks_planning: true,
            });
        }

        if let Some(governance_intent) = bundle.governance_intent.as_ref() {
            let mut governance_parts = Vec::new();
            if let Some(runtime) = governance_intent.runtime_preference {
                governance_parts.push(format!("runtime={runtime}"));
            }
            if let Some(risk) = governance_intent.risk.as_deref() {
                governance_parts.push(format!("risk={risk}"));
            }
            if let Some(zone) = governance_intent.zone.as_deref() {
                governance_parts.push(format!("zone={zone}"));
            }
            if let Some(owner) = governance_intent.owner.as_deref() {
                governance_parts.push(format!("owner={owner}"));
            }
            if !governance_parts.is_empty() {
                packet.constraints.push(NegotiationConstraint {
                    constraint_id: Uuid::new_v4().to_string(),
                    kind: NegotiationConstraintKind::Governance,
                    summary: format!(
                        "respect requested governance context: {}",
                        governance_parts.join(", ")
                    ),
                    source: NegotiationConstraintSource::GovernanceIntent,
                    state: NegotiationConstraintState::Binding,
                    blocks_planning: false,
                });
            }
        }

        packet
    }
}
