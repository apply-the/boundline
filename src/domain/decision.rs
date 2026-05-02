//! Decision object model for the session-native orchestrator (feature 013).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::tool_result::ToolResult;
use crate::domain::trace::current_timestamp_millis;

// -- Types implemented in Phase 2 (T009–T011) --

/// The type of bounded action a decision represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    Analyze,
    Code,
    Test,
    Fix,
    Replan,
}

/// The explicit next-action selector chosen for a decision iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionSelector {
    #[default]
    Read,
    Search,
    Modify,
    Test,
    Ask,
    Replan,
}

impl DecisionType {
    pub const fn default_selector(self) -> ActionSelector {
        match self {
            Self::Analyze => ActionSelector::Read,
            Self::Code | Self::Fix => ActionSelector::Modify,
            Self::Test => ActionSelector::Test,
            Self::Replan => ActionSelector::Replan,
        }
    }
}

impl ActionSelector {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Search => "search",
            Self::Modify => "modify",
            Self::Test => "test",
            Self::Ask => "ask",
            Self::Replan => "replan",
        }
    }
}

impl fmt::Display for ActionSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ActionSelector {
    type Err = ActionSelectorParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "read" => Ok(Self::Read),
            "search" => Ok(Self::Search),
            "modify" => Ok(Self::Modify),
            "test" => Ok(Self::Test),
            "ask" => Ok(Self::Ask),
            "replan" => Ok(Self::Replan),
            _ => Err(ActionSelectorParseError(value.to_string())),
        }
    }
}

/// Lifecycle status of a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Pending,
    Dispatched,
    Verified,
    Failed,
    Recovered,
}

impl DecisionStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Verified | Self::Failed | Self::Recovered)
    }
}

/// Kind of evidence referenced by a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Trace,
    File,
    Canon,
    ToolOutput,
}

/// A reference to a piece of evidence used as input to a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: EvidenceKind,
    pub reference: String,
}

impl EvidenceRef {
    pub fn trace(reference: impl Into<String>) -> Self {
        Self { kind: EvidenceKind::Trace, reference: reference.into() }
    }

    pub fn file(reference: impl Into<String>) -> Self {
        Self { kind: EvidenceKind::File, reference: reference.into() }
    }

    pub fn canon(reference: impl Into<String>) -> Self {
        Self { kind: EvidenceKind::Canon, reference: reference.into() }
    }

    pub fn tool_output(reference: impl Into<String>) -> Self {
        Self { kind: EvidenceKind::ToolOutput, reference: reference.into() }
    }
}

/// The atomic unit of the execution loop.
///
/// Each iteration of observe→decide→act→verify→update produces exactly one
/// Decision object that is persisted in session state and traces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub decision_type: DecisionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<ActionSelector>,
    pub target: String,
    pub rationale: String,
    pub expected_outcome: String,
    #[serde(default)]
    pub evidence_inputs: Vec<EvidenceRef>,
    pub status: DecisionStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<ToolResult>,
    pub created_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
}

impl Decision {
    /// Create a new pending decision.
    pub fn new(
        decision_type: DecisionType,
        target: impl Into<String>,
        rationale: impl Into<String>,
        expected_outcome: impl Into<String>,
        evidence_inputs: Vec<EvidenceRef>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            decision_type,
            selector: Some(decision_type.default_selector()),
            target: target.into(),
            rationale: rationale.into(),
            expected_outcome: expected_outcome.into(),
            evidence_inputs,
            status: DecisionStatus::Pending,
            tool_result: None,
            created_at: current_timestamp_millis(),
            completed_at: None,
        }
    }

    pub fn validate(&self) -> Result<(), DecisionError> {
        if self.id.trim().is_empty() {
            return Err(DecisionError::MissingId);
        }
        if self.target.trim().is_empty() {
            return Err(DecisionError::MissingTarget);
        }
        if self.rationale.trim().is_empty() {
            return Err(DecisionError::MissingRationale);
        }
        if self.expected_outcome.trim().is_empty() {
            return Err(DecisionError::MissingExpectedOutcome);
        }
        Ok(())
    }

    pub fn selector_kind(&self) -> ActionSelector {
        self.selector.unwrap_or_else(|| self.decision_type.default_selector())
    }

    pub fn with_selector(mut self, selector: ActionSelector) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Transition from Pending to Dispatched.
    pub fn mark_dispatched(&mut self) -> Result<(), DecisionError> {
        if self.status != DecisionStatus::Pending {
            return Err(DecisionError::InvalidTransition {
                from: self.status,
                to: DecisionStatus::Dispatched,
            });
        }
        self.status = DecisionStatus::Dispatched;
        Ok(())
    }

    /// Transition from Dispatched to Verified with a tool result.
    pub fn mark_verified(&mut self, tool_result: ToolResult) -> Result<(), DecisionError> {
        if self.status != DecisionStatus::Dispatched {
            return Err(DecisionError::InvalidTransition {
                from: self.status,
                to: DecisionStatus::Verified,
            });
        }
        self.tool_result = Some(tool_result);
        self.status = DecisionStatus::Verified;
        self.completed_at = Some(current_timestamp_millis());
        Ok(())
    }

    /// Transition from Dispatched to Failed with a tool result.
    pub fn mark_failed(&mut self, tool_result: ToolResult) -> Result<(), DecisionError> {
        if self.status != DecisionStatus::Dispatched {
            return Err(DecisionError::InvalidTransition {
                from: self.status,
                to: DecisionStatus::Failed,
            });
        }
        self.tool_result = Some(tool_result);
        self.status = DecisionStatus::Failed;
        self.completed_at = Some(current_timestamp_millis());
        Ok(())
    }

    /// Transition from Failed to Recovered (a recovery decision was created).
    pub fn mark_recovered(&mut self) -> Result<(), DecisionError> {
        if self.status != DecisionStatus::Failed {
            return Err(DecisionError::InvalidTransition {
                from: self.status,
                to: DecisionStatus::Recovered,
            });
        }
        self.status = DecisionStatus::Recovered;
        self.completed_at = Some(current_timestamp_millis());
        Ok(())
    }

    /// Create an EvidenceRef pointing to this decision's tool output.
    pub fn as_tool_output_evidence(&self) -> EvidenceRef {
        EvidenceRef::tool_output(&self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecisionError {
    #[error("decision id must not be empty")]
    MissingId,
    #[error("decision target must not be empty")]
    MissingTarget,
    #[error("decision rationale must not be empty")]
    MissingRationale,
    #[error("decision expected_outcome must not be empty")]
    MissingExpectedOutcome,
    #[error("invalid decision status transition from {from:?} to {to:?}")]
    InvalidTransition { from: DecisionStatus, to: DecisionStatus },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown action selector `{0}`")]
pub struct ActionSelectorParseError(String);
