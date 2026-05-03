//! Goal-derived bounded task draft model (feature 013).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::cluster::{ClusterDeliveryStory, ClusterSessionProjection};
use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::governance::CompactedCanonMemory;
use crate::domain::session::{
    ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState, DelegationPacket,
    DelegationPacketState,
};
use crate::domain::trace::current_timestamp_millis;
use crate::domain::workflow::WorkflowProgressState;

/// Status of a goal-derived plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPlanStatus {
    Draft,
    Confirmed,
    Superseded,
}

/// Workspace signals collected during plan derivation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSignals {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub file_count: usize,
    pub has_config: bool,
    pub has_canon: bool,
    pub has_tests: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackCredibility {
    Credible,
    Insufficient,
    Stale,
}

impl ContextPackCredibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::Insufficient => "insufficient",
            Self::Stale => "stale",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextInputKind {
    WorkspaceFile,
    SymbolHint,
    AuthoredBrief,
    Negotiation,
    RecentTrace,
    DomainTemplate,
    DomainStandard,
    ExternalContextInput,
    CanonArtifact,
    CanonCapability,
    CanonMemory,
}

impl ContextInputKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceFile => "workspace_file",
            Self::SymbolHint => "symbol_hint",
            Self::AuthoredBrief => "authored_brief",
            Self::Negotiation => "negotiation",
            Self::RecentTrace => "recent_trace",
            Self::DomainTemplate => "domain_template",
            Self::DomainStandard => "domain_standard",
            Self::ExternalContextInput => "external_context_input",
            Self::CanonArtifact => "canon_artifact",
            Self::CanonCapability => "canon_capability",
            Self::CanonMemory => "canon_memory",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextInput {
    pub kind: ContextInputKind,
    pub reference: String,
    pub rationale: String,
    pub source: String,
    #[serde(default)]
    pub primary: bool,
}

impl ContextInput {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.reference.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputReference);
        }
        if self.rationale.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputRationale {
                reference: self.reference.clone(),
            });
        }
        if self.source.trim().is_empty() {
            return Err(GoalPlanError::MissingContextInputSource {
                reference: self.reference.clone(),
            });
        }
        Ok(())
    }

    pub fn provenance_line(&self) -> String {
        format!("{}: {} ({})", self.kind.as_str(), self.reference, self.rationale)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPack {
    pub pack_id: String,
    pub summary: String,
    pub credibility: ContextPackCredibility,
    #[serde(default)]
    pub inputs: Vec<ContextInput>,
    #[serde(default)]
    pub selected_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staleness_reason: Option<String>,
}

impl ContextPack {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.pack_id.trim().is_empty() {
            return Err(GoalPlanError::MissingContextPackId);
        }
        if self.summary.trim().is_empty() {
            return Err(GoalPlanError::MissingContextPackSummary);
        }
        for input in &self.inputs {
            input.validate()?;
        }
        if self.credibility == ContextPackCredibility::Credible
            && self.primary_inputs().is_empty()
            && self.selected_targets.is_empty()
        {
            return Err(GoalPlanError::MissingCredibleContextPrimaryInput);
        }
        if self.credibility == ContextPackCredibility::Stale
            && self.staleness_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(GoalPlanError::MissingContextStalenessReason);
        }
        Ok(())
    }

    pub fn primary_inputs(&self) -> Vec<&ContextInput> {
        self.inputs.iter().filter(|input| input.primary).collect()
    }

    pub fn primary_references(&self) -> Vec<String> {
        let primary = self
            .primary_inputs()
            .into_iter()
            .map(|input| input.reference.clone())
            .collect::<Vec<_>>();
        if primary.is_empty() { self.selected_targets.clone() } else { primary }
    }

    pub fn provenance_lines(&self) -> Vec<String> {
        self.inputs.iter().map(ContextInput::provenance_line).collect()
    }
}

/// An inferred flow proposal attached to a goal plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferredFlow {
    pub flow_name: String,
    pub confidence_reason: String,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPlanFlowMode {
    Proposed,
    Confirmed,
    Skipped,
    Absent,
}

impl GoalPlanFlowMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Confirmed => "confirmed",
            Self::Skipped => "skipped",
            Self::Absent => "absent",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalPlanFlowState {
    pub mode: GoalPlanFlowMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_reason: Option<String>,
}

impl GoalPlanFlowState {
    pub fn summary_text(&self) -> String {
        match (self.flow_name.as_deref(), self.confidence_reason.as_deref()) {
            (Some(flow_name), Some(confidence_reason)) => {
                format!("{} ({flow_name}) - {confidence_reason}", self.mode.as_str())
            }
            (Some(flow_name), None) => format!("{} ({flow_name})", self.mode.as_str()),
            _ => self.mode.as_str().to_string(),
        }
    }
}

/// A single planned task in a goal-derived plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedTask {
    pub task_id: String,
    pub description: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_type_hint: Option<DecisionType>,
}

impl PlannedTask {
    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.task_id.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskId);
        }
        if self.description.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskDescription { task_id: self.task_id.clone() });
        }
        if self.target.trim().is_empty() {
            return Err(GoalPlanError::MissingTaskTarget { task_id: self.task_id.clone() });
        }
        Ok(())
    }
}

/// A bounded task draft derived from goal, workspace, documents, and Canon artifacts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalPlan {
    pub plan_id: String,
    pub goal_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_goal_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_resolution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub negotiation_acceptance_boundary: Option<String>,
    pub tasks: Vec<PlannedTask>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_pack: Option<ContextPack>,
    #[serde(default)]
    pub source_evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub workspace_signals: WorkspaceSignals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_policy_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning_rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<InferredFlow>,
    #[serde(default)]
    pub flow_skipped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_progress: Option<WorkflowProgressState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compacted_canon_memory: Option<CompactedCanonMemory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_session_projection: Option<ClusterSessionProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster_delivery_story: Option<ClusterDeliveryStory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_packet_history: Vec<DelegationPacket>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_continuity: Option<DelegationContinuityState>,
    #[serde(default = "default_goal_plan_revision")]
    pub proposal_revision: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by_revision: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_at: Option<u64>,
    pub created_at: u64,
    pub status: GoalPlanStatus,
}

const fn default_goal_plan_revision() -> usize {
    1
}

impl GoalPlan {
    pub fn new(
        goal_text: impl Into<String>,
        tasks: Vec<PlannedTask>,
    ) -> Result<Self, GoalPlanError> {
        let plan = Self {
            plan_id: Uuid::new_v4().to_string(),
            goal_text: goal_text.into(),
            negotiation_goal_summary: None,
            negotiation_resolution: None,
            negotiation_acceptance_boundary: None,
            tasks,
            context_pack: None,
            source_evidence: Vec::new(),
            workspace_signals: WorkspaceSignals::default(),
            routing_policy_summary: None,
            planning_rationale: None,
            verification_strategy: None,
            flow: None,
            flow_skipped: false,
            workflow_progress: None,
            compacted_canon_memory: None,
            cluster_session_projection: None,
            cluster_delivery_story: None,
            delegation_packet_history: Vec::new(),
            delegation_continuity: None,
            proposal_revision: default_goal_plan_revision(),
            superseded_by_revision: None,
            superseded_reason: None,
            confirmed_at: None,
            created_at: current_timestamp_millis(),
            status: GoalPlanStatus::Draft,
        };
        plan.validate()?;
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), GoalPlanError> {
        if self.goal_text.trim().is_empty() {
            return Err(GoalPlanError::MissingGoalText);
        }
        if self.tasks.is_empty() {
            return Err(GoalPlanError::NoTasks);
        }
        for task in &self.tasks {
            task.validate()?;
        }
        if let Some(context_pack) = &self.context_pack {
            context_pack.validate()?;
        }
        if let Some(workflow_progress) = &self.workflow_progress {
            workflow_progress
                .validate()
                .map_err(|error| GoalPlanError::InvalidWorkflowProgress(error.to_string()))?;
        }
        for packet in &self.delegation_packet_history {
            packet.validate().map_err(GoalPlanError::InvalidDelegationPacket)?;
        }
        if let Some(continuity) = &self.delegation_continuity {
            continuity
                .validate(&self.delegation_packet_history)
                .map_err(GoalPlanError::InvalidDelegationContinuity)?;
        }
        if let Some(projection) = &self.cluster_session_projection {
            projection
                .validate()
                .map_err(|error| GoalPlanError::InvalidClusterProjection(error.to_string()))?;
        }
        if let Some(story) = &self.cluster_delivery_story {
            story
                .validate()
                .map_err(|error| GoalPlanError::InvalidClusterDeliveryStory(error.to_string()))?;
        }
        if self.proposal_revision == 0 {
            return Err(GoalPlanError::MissingProposalRevision);
        }
        Ok(())
    }

    pub fn requires_confirmation(&self) -> bool {
        self.status == GoalPlanStatus::Draft
    }

    pub fn proposal_state_text(&self) -> &'static str {
        match self.status {
            GoalPlanStatus::Draft => "proposed",
            GoalPlanStatus::Confirmed => "confirmed",
            GoalPlanStatus::Superseded => "superseded",
        }
    }

    pub fn confirm(&mut self) -> Result<(), GoalPlanError> {
        if self.status != GoalPlanStatus::Draft {
            return Err(GoalPlanError::InvalidTransition {
                from: self.status,
                to: GoalPlanStatus::Confirmed,
            });
        }
        self.status = GoalPlanStatus::Confirmed;
        if let Some(flow) = self.flow.as_mut() {
            flow.confirmed = true;
        }
        self.confirmed_at = Some(current_timestamp_millis());
        Ok(())
    }

    pub fn supersede(&mut self) -> Result<(), GoalPlanError> {
        if self.status != GoalPlanStatus::Confirmed {
            return Err(GoalPlanError::InvalidTransition {
                from: self.status,
                to: GoalPlanStatus::Superseded,
            });
        }
        self.status = GoalPlanStatus::Superseded;
        Ok(())
    }

    pub fn supersede_with(
        &mut self,
        superseded_by_revision: usize,
        reason: impl Into<String>,
    ) -> Result<(), GoalPlanError> {
        self.supersede()?;
        self.superseded_by_revision = Some(superseded_by_revision);
        self.superseded_reason = Some(reason.into());
        Ok(())
    }

    pub fn with_signals(mut self, signals: WorkspaceSignals) -> Self {
        self.workspace_signals = signals;
        self
    }

    pub fn with_planning_rationale(mut self, planning_rationale: impl Into<String>) -> Self {
        self.planning_rationale = Some(planning_rationale.into());
        self
    }

    pub fn with_routing_policy_summary(
        mut self,
        routing_policy_summary: impl Into<String>,
    ) -> Self {
        self.routing_policy_summary = Some(routing_policy_summary.into());
        self
    }

    pub fn with_verification_strategy(mut self, verification_strategy: impl Into<String>) -> Self {
        self.verification_strategy = Some(verification_strategy.into());
        self
    }

    pub fn next_revision(&self) -> usize {
        self.proposal_revision + 1
    }

    pub fn with_flow(mut self, flow: InferredFlow) -> Self {
        self.flow = Some(flow);
        self.flow_skipped = false;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<EvidenceRef>) -> Self {
        self.source_evidence = evidence;
        self
    }

    pub fn with_context_pack(mut self, context_pack: ContextPack) -> Self {
        self.context_pack = Some(context_pack);
        self
    }

    pub fn with_workflow_progress(mut self, workflow_progress: WorkflowProgressState) -> Self {
        self.workflow_progress = Some(workflow_progress);
        self
    }

    pub fn with_compacted_canon_memory(
        mut self,
        compacted_canon_memory: CompactedCanonMemory,
    ) -> Self {
        self.compacted_canon_memory = Some(compacted_canon_memory);
        self
    }

    pub fn with_delegation_state(
        mut self,
        packet_history: Vec<DelegationPacket>,
        continuity: DelegationContinuityState,
    ) -> Result<Self, GoalPlanError> {
        self.delegation_packet_history = packet_history;
        self.delegation_continuity = Some(continuity);
        self.validate()?;
        Ok(self)
    }

    pub fn with_negotiation_projection(
        mut self,
        goal_summary: impl Into<String>,
        resolution: impl Into<String>,
        acceptance_boundary: impl Into<String>,
    ) -> Self {
        self.negotiation_goal_summary = Some(goal_summary.into());
        self.negotiation_resolution = Some(resolution.into());
        self.negotiation_acceptance_boundary = Some(acceptance_boundary.into());
        self
    }

    pub fn mark_flow_skipped(&mut self) {
        self.flow = None;
        self.flow_skipped = true;
    }

    pub fn flow_state(&self) -> GoalPlanFlowState {
        match self.flow.as_ref() {
            Some(flow) => GoalPlanFlowState {
                mode: if flow.confirmed {
                    GoalPlanFlowMode::Confirmed
                } else {
                    GoalPlanFlowMode::Proposed
                },
                flow_name: Some(flow.flow_name.clone()),
                confidence_reason: Some(flow.confidence_reason.clone()),
            },
            None => GoalPlanFlowState {
                mode: if self.flow_skipped {
                    GoalPlanFlowMode::Skipped
                } else {
                    GoalPlanFlowMode::Absent
                },
                flow_name: None,
                confidence_reason: None,
            },
        }
    }

    pub fn workflow_name(&self) -> Option<String> {
        self.workflow_progress.as_ref().map(|workflow| workflow.workflow_name.clone())
    }

    pub fn workflow_phase_text(&self) -> Option<String> {
        self.workflow_progress.as_ref().and_then(WorkflowProgressState::current_phase_text)
    }

    pub fn workflow_next_action(&self) -> Option<String> {
        self.workflow_progress.as_ref().and_then(WorkflowProgressState::next_action_text)
    }

    pub fn delegation_continuity(&self) -> Option<&DelegationContinuityState> {
        self.delegation_continuity.as_ref()
    }

    pub fn delegation_packet_history(&self) -> &[DelegationPacket] {
        &self.delegation_packet_history
    }

    pub fn active_delegation_packet(&self) -> Option<&DelegationPacket> {
        let active_packet_id = self.delegation_continuity.as_ref()?.active_packet_id.as_deref()?;
        self.delegation_packet_history.iter().find(|packet| packet.packet_id == active_packet_id)
    }

    pub fn record_delegation_packet(
        &mut self,
        packet: DelegationPacket,
        continuity: DelegationContinuityState,
    ) -> Result<(), GoalPlanError> {
        packet.validate().map_err(GoalPlanError::InvalidDelegationPacket)?;

        let mut history = self.delegation_packet_history.clone();
        let next_packet_id = packet.packet_id.clone();
        if let Some(active_packet_id) = self
            .delegation_continuity
            .as_ref()
            .and_then(|state| state.active_packet_id.as_ref())
            .filter(|active_packet_id| *active_packet_id != &next_packet_id)
            && let Some(existing_packet) = history
                .iter_mut()
                .find(|existing_packet| existing_packet.packet_id == *active_packet_id)
            && matches!(
                existing_packet.state,
                DelegationPacketState::Active | DelegationPacketState::Stuck
            )
        {
            existing_packet.mark_superseded(next_packet_id.clone());
        }

        if let Some(existing_packet) =
            history.iter_mut().find(|existing_packet| existing_packet.packet_id == next_packet_id)
        {
            *existing_packet = packet;
        } else {
            history.push(packet);
        }

        continuity.validate(&history).map_err(GoalPlanError::InvalidDelegationContinuity)?;
        self.delegation_packet_history = history;
        self.delegation_continuity = Some(continuity);
        Ok(())
    }

    pub fn resolve_active_delegation(
        &mut self,
        headline: impl Into<String>,
        evidence_summary: impl Into<String>,
        next_command: impl Into<String>,
    ) -> Result<(), GoalPlanError> {
        let Some(active_packet_id) =
            self.delegation_continuity.as_ref().and_then(|state| state.active_packet_id.clone())
        else {
            return Ok(());
        };

        let packet = self
            .delegation_packet_history
            .iter_mut()
            .find(|packet| packet.packet_id == active_packet_id)
            .ok_or_else(|| {
                GoalPlanError::InvalidDelegationContinuity(format!(
                    "delegation active_packet_id `{active_packet_id}` is missing from history"
                ))
            })?;
        packet.mark_resolved();

        let continuity = DelegationContinuityState {
            active_packet_id: None,
            mode: DelegationContinuityMode::Resolved,
            authority_source: ContinuityAuthority::NativeSession,
            next_command: next_command.into(),
            headline: headline.into(),
            evidence_summary: evidence_summary.into(),
        };
        continuity
            .validate(&self.delegation_packet_history)
            .map_err(GoalPlanError::InvalidDelegationContinuity)?;
        self.delegation_continuity = Some(continuity);
        Ok(())
    }

    pub fn context_summary(&self) -> Option<String> {
        match (
            self.context_pack.as_ref().map(|pack| pack.summary.clone()),
            self.compacted_canon_memory.as_ref().map(CompactedCanonMemory::summary_text),
        ) {
            (Some(context_summary), Some(canon_summary)) => {
                Some(format!("{context_summary}; canon memory: {canon_summary}"))
            }
            (Some(context_summary), None) => Some(context_summary),
            (None, Some(canon_summary)) => Some(format!("canon memory: {canon_summary}")),
            (None, None) => None,
        }
    }

    pub fn context_credibility(&self) -> Option<String> {
        self.context_pack.as_ref().map(|pack| pack.credibility.as_str().to_string())
    }

    pub fn context_primary_inputs(&self) -> Vec<String> {
        let mut inputs =
            self.context_pack.as_ref().map(ContextPack::primary_references).unwrap_or_default();
        if inputs.is_empty()
            && let Some(memory) = self.compacted_canon_memory.as_ref()
        {
            inputs.extend(memory.artifact_refs.iter().take(2).cloned());
        }
        inputs
    }

    pub fn context_provenance_lines(&self) -> Vec<String> {
        let mut lines =
            self.context_pack.as_ref().map(ContextPack::provenance_lines).unwrap_or_default();
        if let Some(memory) = self.compacted_canon_memory.as_ref() {
            lines.push(format!(
                "canon_memory: {} [{}]",
                memory.headline,
                memory.credibility.as_str()
            ));
            if let Some(packet_ref) = memory.packet_ref.as_ref() {
                lines.push(format!("canon_memory_packet: {packet_ref}"));
            }
            if let Some(reason_code) = memory.reason_code.as_ref() {
                lines.push(format!("canon_memory_reason: {reason_code}"));
            }
            if let Some(mode_summary) = memory.mode_summary.as_ref() {
                lines.push(format!("canon_memory_mode: {}", mode_summary.summary_text()));
            }
            if let Some(evidence_summary) = memory.evidence_summary.as_ref() {
                for link in &evidence_summary.artifact_provenance_links {
                    lines.push(format!("canon_provenance: {link}"));
                }
            }
        }
        lines
    }

    pub fn canon_memory_staleness_reason(&self) -> Option<String> {
        self.compacted_canon_memory.as_ref().and_then(|memory| {
            (memory.credibility != crate::domain::governance::MemoryCredibilityState::Credible)
                .then(|| memory.reason_code.clone().unwrap_or_else(|| memory.headline.clone()))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GoalPlanError {
    #[error("goal text must not be empty")]
    MissingGoalText,
    #[error("goal plan must have at least one task")]
    NoTasks,
    #[error("task id must not be empty")]
    MissingTaskId,
    #[error("task `{task_id}` description must not be empty")]
    MissingTaskDescription { task_id: String },
    #[error("task `{task_id}` target must not be empty")]
    MissingTaskTarget { task_id: String },
    #[error("context pack id must not be empty")]
    MissingContextPackId,
    #[error("context pack summary must not be empty")]
    MissingContextPackSummary,
    #[error("credible context pack must have at least one primary input or selected target")]
    MissingCredibleContextPrimaryInput,
    #[error("stale context pack must explain why it is stale")]
    MissingContextStalenessReason,
    #[error("goal plan proposal revision must be at least 1")]
    MissingProposalRevision,
    #[error("context input reference must not be empty")]
    MissingContextInputReference,
    #[error("context input `{reference}` rationale must not be empty")]
    MissingContextInputRationale { reference: String },
    #[error("context input `{reference}` source must not be empty")]
    MissingContextInputSource { reference: String },
    #[error("goal plan workflow progress is invalid: {0}")]
    InvalidWorkflowProgress(String),
    #[error("invalid delegation packet: {0}")]
    InvalidDelegationPacket(String),
    #[error("invalid delegation continuity: {0}")]
    InvalidDelegationContinuity(String),
    #[error("invalid cluster projection: {0}")]
    InvalidClusterProjection(String),
    #[error("invalid cluster delivery story: {0}")]
    InvalidClusterDeliveryStory(String),
    #[error("invalid goal plan status transition from {from:?} to {to:?}")]
    InvalidTransition { from: GoalPlanStatus, to: GoalPlanStatus },
}

#[cfg(test)]
mod tests {
    use super::{
        ContextInput, ContextInputKind, ContextPack, ContextPackCredibility, GoalPlan, PlannedTask,
    };
    use crate::domain::governance::{
        CanonEvidenceInspectSummary, CanonModeSummary, CanonResultActionSummary,
        CompactedCanonMemory, MemoryCredibilityState,
    };
    use crate::domain::session::{
        ContinuityAuthority, DelegationContinuityMode, DelegationContinuityState, DelegationPacket,
        DelegationPacketKind, DelegationPacketState, StuckEvidenceMarker, StuckRecoveryAction,
    };

    fn build_plan() -> GoalPlan {
        GoalPlan::new(
            "Fix delegated continuity",
            vec![PlannedTask {
                task_id: "planned-task-1".to_string(),
                description: "Repair the blocked bounded flow".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status explains the blocked continuation".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
    }

    fn build_packet(packet_id: &str, kind: DelegationPacketKind) -> DelegationPacket {
        DelegationPacket {
            packet_id: packet_id.to_string(),
            kind,
            state: DelegationPacketState::Active,
            created_at: 100,
            resolved_at: None,
            source_route_owner: "native".to_string(),
            target_owner: match kind {
                DelegationPacketKind::Handoff => "codex".to_string(),
                DelegationPacketKind::Escalation => "operator".to_string(),
            },
            continuity_reason: "declared runtime cannot continue the bounded step".to_string(),
            recommended_next_action: "boundline status".to_string(),
            evidence_refs: vec!["routing:implementation=claude/sonnet-4".to_string()],
            capability_summary: Some(
                "claude lacks continuation support for implementation".to_string(),
            ),
            stuck_marker: None,
            superseded_by_packet_id: None,
        }
    }

    #[test]
    fn recording_delegation_packet_supersedes_previous_active_packet() {
        let mut plan = build_plan();
        let first_packet = build_packet("packet-1", DelegationPacketKind::Handoff);
        let second_packet = build_packet("packet-2", DelegationPacketKind::Escalation);

        plan.record_delegation_packet(
            first_packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-1".to_string()),
                mode: DelegationContinuityMode::HandoffRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline status".to_string(),
                headline: "handoff required: implementation route cannot continue".to_string(),
                evidence_summary: "routing policy requires a handoff".to_string(),
            },
        )
        .unwrap();

        plan.record_delegation_packet(
            second_packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-2".to_string()),
                mode: DelegationContinuityMode::EscalationRequired,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: "escalation required: no declared continuation path remains".to_string(),
                evidence_summary: "all declared routes are blocked by capability policy"
                    .to_string(),
            },
        )
        .unwrap();

        let first_packet = plan
            .delegation_packet_history()
            .iter()
            .find(|packet| packet.packet_id == "packet-1")
            .unwrap();
        assert_eq!(first_packet.state, DelegationPacketState::Superseded);
        assert_eq!(first_packet.superseded_by_packet_id.as_deref(), Some("packet-2"));
        assert_eq!(plan.active_delegation_packet().unwrap().packet_id, "packet-2");
        assert_eq!(
            plan.delegation_continuity().unwrap().mode,
            DelegationContinuityMode::EscalationRequired
        );
    }

    #[test]
    fn resolving_delegation_packet_preserves_history_and_clears_active_pointer() {
        let mut plan = build_plan();
        let mut packet = build_packet("packet-stuck", DelegationPacketKind::Handoff);
        packet.state = DelegationPacketState::Stuck;
        packet.stuck_marker = Some(StuckEvidenceMarker {
            repeated_attempts: 3,
            same_reason_count: 3,
            unchanged_workspace_signal: true,
            stale_route_policy: false,
            recommended_recovery: StuckRecoveryAction::Replan,
        });

        plan.record_delegation_packet(
            packet,
            DelegationContinuityState {
                active_packet_id: Some("packet-stuck".to_string()),
                mode: DelegationContinuityMode::Stuck,
                authority_source: ContinuityAuthority::NativeSession,
                next_command: "boundline inspect".to_string(),
                headline: "stuck delegated continuity requires recovery".to_string(),
                evidence_summary: "the same blocked continuity reason repeated three times"
                    .to_string(),
            },
        )
        .unwrap();

        plan.resolve_active_delegation(
            "delegated continuity resolved after config update",
            "operator updated the declared runtime policy",
            "boundline run",
        )
        .unwrap();

        let resolved_packet = plan
            .delegation_packet_history()
            .iter()
            .find(|packet| packet.packet_id == "packet-stuck")
            .unwrap();
        assert_eq!(resolved_packet.state, DelegationPacketState::Resolved);
        assert!(resolved_packet.resolved_at.is_some());

        let continuity = plan.delegation_continuity().unwrap();
        assert_eq!(continuity.mode, DelegationContinuityMode::Resolved);
        assert!(continuity.active_packet_id.is_none());
        assert_eq!(continuity.next_command, "boundline run");
    }

    #[test]
    fn context_and_flow_helpers_surface_negotiation_and_canon_memory_details() {
        let mut plan = GoalPlan::new(
            "Tighten bounded context",
            vec![PlannedTask {
                task_id: "planned-task-context".to_string(),
                description: "Confirm the governed packet context".to_string(),
                target: "src/lib.rs".to_string(),
                expected_outcome: Some("status reflects bounded context".to_string()),
                decision_type_hint: None,
            }],
        )
        .unwrap()
        .with_context_pack(ContextPack {
            pack_id: "cp-context".to_string(),
            summary: "bounded context from src/lib.rs".to_string(),
            credibility: ContextPackCredibility::Credible,
            inputs: vec![ContextInput {
                kind: ContextInputKind::WorkspaceFile,
                reference: "src/lib.rs".to_string(),
                rationale: "primary workspace slice".to_string(),
                source: "workspace_scan".to_string(),
                primary: true,
            }],
            selected_targets: vec!["src/lib.rs".to_string()],
            staleness_reason: None,
        })
        .with_negotiation_projection(
            "deliver the smallest safe fix",
            "confirmed",
            "tests stay green",
        )
        .with_compacted_canon_memory(CompactedCanonMemory {
            headline: "Governed packet needs refresh".to_string(),
            credibility: MemoryCredibilityState::Stale,
            stage_key: Some("bug-fix:verify".to_string()),
            run_ref: Some("canon-run-1".to_string()),
            packet_ref: Some(".canon/runs/canon-run-1".to_string()),
            reason_code: Some("refresh_required".to_string()),
            artifact_refs: vec![".canon/runs/canon-run-1/verification.md".to_string()],
            mode_summary: Some(CanonModeSummary {
                headline: "Discovery mode packet ready".to_string(),
                artifact_packet_summary: "packet can be resumed".to_string(),
                execution_posture: Some("awaiting operator review".to_string()),
                primary_artifact_title: "verification packet".to_string(),
                primary_artifact_path: ".canon/runs/canon-run-1/verification.md".to_string(),
                primary_artifact_action: CanonResultActionSummary {
                    label: "inspect".to_string(),
                    target: ".canon/runs/canon-run-1/verification.md".to_string(),
                },
                result_excerpt: "governed packet is reusable once refreshed".to_string(),
                action_chip_labels: vec!["inspect".to_string()],
            }),
            possible_actions: Vec::new(),
            recommended_next_action: None,
            evidence_summary: Some(CanonEvidenceInspectSummary {
                execution_posture: Some("paused".to_string()),
                carried_forward_items: Vec::new(),
                artifact_provenance_links: vec![
                    "canon:packet=.canon/runs/canon-run-1".to_string(),
                    "canon:artifact=.canon/runs/canon-run-1/verification.md".to_string(),
                ],
                closure_status: None,
                closure_findings: Vec::new(),
            }),
        });

        assert_eq!(
            plan.context_summary().as_deref(),
            Some(
                "bounded context from src/lib.rs; canon memory: Governed packet needs refresh [stale]"
            )
        );
        assert_eq!(plan.context_credibility().as_deref(), Some("credible"));
        assert_eq!(plan.context_primary_inputs(), vec!["src/lib.rs".to_string()]);
        assert!(
            plan.context_provenance_lines()
                .iter()
                .any(|line| line.contains("canon_memory_packet: .canon/runs/canon-run-1"))
        );
        assert!(plan
            .context_provenance_lines()
            .iter()
            .any(|line| line.contains("canon_memory_mode: Discovery mode packet ready; packet can be resumed; execution posture: awaiting operator review")));
        assert_eq!(plan.canon_memory_staleness_reason().as_deref(), Some("refresh_required"));
        assert_eq!(plan.negotiation_goal_summary.as_deref(), Some("deliver the smallest safe fix"));
        assert_eq!(plan.negotiation_resolution.as_deref(), Some("confirmed"));
        assert_eq!(plan.negotiation_acceptance_boundary.as_deref(), Some("tests stay green"));

        plan.mark_flow_skipped();
        let flow_state = plan.flow_state();
        assert_eq!(flow_state.mode, super::GoalPlanFlowMode::Skipped);
        assert!(flow_state.flow_name.is_none());
    }
}
