//! Goal-derived bounded task draft model (feature 013).

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::decision::{DecisionType, EvidenceRef};
use crate::domain::trace::current_timestamp_millis;

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

/// An inferred flow proposal attached to a goal plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferredFlow {
    pub flow_name: String,
    pub confidence_reason: String,
    #[serde(default)]
    pub confirmed: bool,
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
    pub tasks: Vec<PlannedTask>,
    #[serde(default)]
    pub source_evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub workspace_signals: WorkspaceSignals,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow: Option<InferredFlow>,
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
            tasks,
            source_evidence: Vec::new(),
            workspace_signals: WorkspaceSignals::default(),
            flow: None,
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
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<EvidenceRef>) -> Self {
        self.source_evidence = evidence;
        self
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
    #[error("invalid goal plan status transition from {from:?} to {to:?}")]
    InvalidTransition { from: GoalPlanStatus, to: GoalPlanStatus },
}
