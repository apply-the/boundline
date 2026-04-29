//! Flow-as-policy model: maps flow stages to allowed decision types (feature 013).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::decision::DecisionType;
use crate::domain::flow::{FlowDefinition, built_in_flow};

/// Condition required for transitioning from one stage to the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionCondition {
    /// All decisions in the current stage must be Verified.
    AllVerified,
    /// Operator must explicitly confirm stage advance.
    ExplicitAdvance,
}

/// Policy for a single flow stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagePolicy {
    pub stage_id: String,
    pub allowed_decisions: Vec<DecisionType>,
    pub transition_condition: TransitionCondition,
}

impl StagePolicy {
    pub fn allows(&self, decision_type: DecisionType) -> bool {
        self.allowed_decisions.contains(&decision_type)
    }
}

/// Maps an entire flow to stage-level decision policies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowPolicy {
    pub flow_name: String,
    pub stage_policies: Vec<StagePolicy>,
    pub current_stage_index: usize,
}

impl FlowPolicy {
    /// Build a FlowPolicy from a built-in flow definition.
    pub fn from_builtin(flow_name: &str) -> Result<Self, FlowPolicyError> {
        let flow = built_in_flow(flow_name)
            .ok_or_else(|| FlowPolicyError::UnknownFlow(flow_name.to_string()))?;
        Ok(Self::from_definition(flow))
    }

    /// Build a FlowPolicy from a FlowDefinition reference.
    pub fn from_definition(flow: &FlowDefinition) -> Self {
        let stage_policies = flow
            .stages
            .iter()
            .map(|stage| {
                let allowed = allowed_decisions_for_stage(flow.name, stage.id);
                StagePolicy {
                    stage_id: stage.id.to_string(),
                    allowed_decisions: allowed,
                    transition_condition: TransitionCondition::AllVerified,
                }
            })
            .collect();

        Self { flow_name: flow.name.to_string(), stage_policies, current_stage_index: 0 }
    }

    /// Get the current stage policy.
    pub fn current_stage(&self) -> Option<&StagePolicy> {
        self.stage_policies.get(self.current_stage_index)
    }

    /// Check if a decision type is allowed at the current stage.
    pub fn is_allowed(&self, decision_type: DecisionType) -> bool {
        self.current_stage().is_some_and(|stage| stage.allows(decision_type))
    }

    /// Advance to the next stage. Returns false if already at the final stage.
    pub fn advance_stage(&mut self) -> Result<bool, FlowPolicyError> {
        let next = self.current_stage_index + 1;
        if next >= self.stage_policies.len() {
            return Ok(false);
        }
        self.current_stage_index = next;
        Ok(true)
    }

    /// Whether the current stage is the final stage.
    pub fn is_final_stage(&self) -> bool {
        self.current_stage_index + 1 >= self.stage_policies.len()
    }

    pub fn validate(&self) -> Result<(), FlowPolicyError> {
        if self.flow_name.trim().is_empty() {
            return Err(FlowPolicyError::EmptyFlowName);
        }
        if self.stage_policies.is_empty() {
            return Err(FlowPolicyError::NoStages);
        }
        if self.current_stage_index >= self.stage_policies.len() {
            return Err(FlowPolicyError::InvalidStageIndex {
                index: self.current_stage_index,
                total: self.stage_policies.len(),
            });
        }
        for stage in &self.stage_policies {
            if stage.allowed_decisions.is_empty() {
                return Err(FlowPolicyError::NoAllowedDecisions {
                    stage_id: stage.stage_id.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Built-in decision type mapping for each flow stage.
fn allowed_decisions_for_stage(flow_name: &str, stage_id: &str) -> Vec<DecisionType> {
    match (flow_name, stage_id) {
        // bug-fix
        ("bug-fix", "investigate") => vec![DecisionType::Analyze],
        ("bug-fix", "implement") => vec![DecisionType::Code, DecisionType::Fix],
        ("bug-fix", "verify") => vec![DecisionType::Test, DecisionType::Replan],
        // change
        ("change", "understand-change") => vec![DecisionType::Analyze],
        ("change", "implement") => vec![DecisionType::Code, DecisionType::Fix],
        ("change", "verify") => vec![DecisionType::Test, DecisionType::Replan],
        // delivery
        ("delivery", "requirements") => vec![DecisionType::Analyze],
        ("delivery", "architecture") => vec![DecisionType::Analyze, DecisionType::Code],
        ("delivery", "backlog") => vec![DecisionType::Analyze, DecisionType::Replan],
        ("delivery", "implementation") => {
            vec![DecisionType::Code, DecisionType::Test, DecisionType::Fix, DecisionType::Replan]
        }
        // fallback: allow all
        _ => vec![
            DecisionType::Analyze,
            DecisionType::Code,
            DecisionType::Test,
            DecisionType::Fix,
            DecisionType::Replan,
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FlowPolicyError {
    #[error("unknown flow `{0}`")]
    UnknownFlow(String),
    #[error("flow name must not be empty")]
    EmptyFlowName,
    #[error("flow policy must have at least one stage")]
    NoStages,
    #[error("invalid stage index {index} for total {total}")]
    InvalidStageIndex { index: usize, total: usize },
    #[error("stage `{stage_id}` must have at least one allowed decision type")]
    NoAllowedDecisions { stage_id: String },
}
