//! Goal-derived bounded task draft model (feature 013).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::decision::{DecisionType, EvidenceRef};
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
    CanonArtifact,
}

impl ContextInputKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceFile => "workspace_file",
            Self::SymbolHint => "symbol_hint",
            Self::AuthoredBrief => "authored_brief",
            Self::Negotiation => "negotiation",
            Self::RecentTrace => "recent_trace",
            Self::CanonArtifact => "canon_artifact",
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
    pub flow: Option<InferredFlow>,
    #[serde(default)]
    pub flow_skipped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_progress: Option<WorkflowProgressState>,
    pub created_at: u64,
    pub status: GoalPlanStatus,
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
            flow: None,
            flow_skipped: false,
            workflow_progress: None,
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
        Ok(())
    }

    pub fn confirm(&mut self) -> Result<(), GoalPlanError> {
        if self.status != GoalPlanStatus::Draft {
            return Err(GoalPlanError::InvalidTransition {
                from: self.status,
                to: GoalPlanStatus::Confirmed,
            });
        }
        self.status = GoalPlanStatus::Confirmed;
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

    pub fn with_signals(mut self, signals: WorkspaceSignals) -> Self {
        self.workspace_signals = signals;
        self
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

    pub fn context_summary(&self) -> Option<String> {
        self.context_pack.as_ref().map(|pack| pack.summary.clone())
    }

    pub fn context_credibility(&self) -> Option<String> {
        self.context_pack.as_ref().map(|pack| pack.credibility.as_str().to_string())
    }

    pub fn context_primary_inputs(&self) -> Vec<String> {
        self.context_pack.as_ref().map(ContextPack::primary_references).unwrap_or_default()
    }

    pub fn context_provenance_lines(&self) -> Vec<String> {
        self.context_pack.as_ref().map(ContextPack::provenance_lines).unwrap_or_default()
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
    #[error("context input reference must not be empty")]
    MissingContextInputReference,
    #[error("context input `{reference}` rationale must not be empty")]
    MissingContextInputRationale { reference: String },
    #[error("context input `{reference}` source must not be empty")]
    MissingContextInputSource { reference: String },
    #[error("goal plan workflow progress is invalid: {0}")]
    InvalidWorkflowProgress(String),
    #[error("invalid goal plan status transition from {from:?} to {to:?}")]
    InvalidTransition { from: GoalPlanStatus, to: GoalPlanStatus },
}
