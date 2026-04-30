use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowGoalSource {
    Session,
}

impl WorkflowGoalSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
        }
    }
}

impl fmt::Display for WorkflowGoalSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowPhase {
    Capture,
    Clarify,
    Plan,
    Run,
    Review,
    Govern,
    Inspect,
}

impl WorkflowPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Clarify => "clarify",
            Self::Plan => "plan",
            Self::Run => "run",
            Self::Review => "review",
            Self::Govern => "govern",
            Self::Inspect => "inspect",
        }
    }
}

impl fmt::Display for WorkflowPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowConditionKind {
    MissingAuthoredInput,
    ReviewTriggered,
    GovernanceRequired,
}

impl WorkflowConditionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingAuthoredInput => "missing_authored_input",
            Self::ReviewTriggered => "review_triggered",
            Self::GovernanceRequired => "governance_required",
        }
    }
}

impl fmt::Display for WorkflowConditionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLifecycleState {
    Idle,
    Active,
    Paused,
    Blocked,
    Completed,
    Failed,
}

impl WorkflowLifecycleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for WorkflowLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowOutputPreferences {
    #[serde(default)]
    pub next_command: bool,
    #[serde(default)]
    pub routing_summary: bool,
    #[serde(default)]
    pub execution_condition: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConditionalWorkflowPhase {
    pub phase: WorkflowPhase,
    pub condition_kind: WorkflowConditionKind,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

const fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_name: String,
    pub goal_source: WorkflowGoalSource,
    pub entry_phase: WorkflowPhase,
    pub phases: Vec<WorkflowPhase>,
    #[serde(default)]
    pub allow_review: bool,
    #[serde(default)]
    pub allow_governance: bool,
    #[serde(default)]
    pub conditional_phases: Vec<ConditionalWorkflowPhase>,
    #[serde(default)]
    pub output_preferences: WorkflowOutputPreferences,
}

impl WorkflowDefinition {
    pub fn validate(&self) -> Result<(), WorkflowDefinitionError> {
        if self.workflow_name.trim().is_empty() {
            return Err(WorkflowDefinitionError::MissingWorkflowName);
        }
        if self.phases.is_empty() {
            return Err(WorkflowDefinitionError::MissingPhases {
                workflow_name: self.workflow_name.clone(),
            });
        }
        if self.phases.first().copied() != Some(self.entry_phase) {
            return Err(WorkflowDefinitionError::EntryPhaseMustBeFirst {
                workflow_name: self.workflow_name.clone(),
                entry_phase: self.entry_phase,
            });
        }

        let mut seen = BTreeSet::new();
        for phase in &self.phases {
            if !seen.insert(*phase) {
                return Err(WorkflowDefinitionError::DuplicatePhase {
                    workflow_name: self.workflow_name.clone(),
                    phase: *phase,
                });
            }
            if *phase == WorkflowPhase::Review && !self.allow_review {
                return Err(WorkflowDefinitionError::ReviewPhaseNotAllowed {
                    workflow_name: self.workflow_name.clone(),
                });
            }
            if *phase == WorkflowPhase::Govern && !self.allow_governance {
                return Err(WorkflowDefinitionError::GovernancePhaseNotAllowed {
                    workflow_name: self.workflow_name.clone(),
                });
            }
        }

        for conditional_phase in &self.conditional_phases {
            if !self.phases.contains(&conditional_phase.phase) {
                return Err(WorkflowDefinitionError::ConditionalPhaseMissing {
                    workflow_name: self.workflow_name.clone(),
                    phase: conditional_phase.phase,
                });
            }

            let expected_condition = match conditional_phase.phase {
                WorkflowPhase::Clarify => Some(WorkflowConditionKind::MissingAuthoredInput),
                WorkflowPhase::Review => Some(WorkflowConditionKind::ReviewTriggered),
                WorkflowPhase::Govern => Some(WorkflowConditionKind::GovernanceRequired),
                _ => None,
            };

            let Some(expected_condition) = expected_condition else {
                return Err(WorkflowDefinitionError::UnsupportedConditionalPhase {
                    workflow_name: self.workflow_name.clone(),
                    phase: conditional_phase.phase,
                });
            };

            if conditional_phase.condition_kind != expected_condition {
                return Err(WorkflowDefinitionError::UnexpectedConditionKind {
                    workflow_name: self.workflow_name.clone(),
                    phase: conditional_phase.phase,
                    condition_kind: conditional_phase.condition_kind,
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowRegistry {
    workflows: BTreeMap<String, WorkflowDefinition>,
}

impl WorkflowRegistry {
    pub fn from_toml_str(contents: &str) -> Result<Self, WorkflowDefinitionError> {
        let raw: WorkflowRegistryToml =
            toml::from_str(contents).map_err(WorkflowDefinitionError::ParseWorkflowDefinitions)?;

        if raw.workflow.is_empty() {
            return Err(WorkflowDefinitionError::MissingWorkflowDefinitions);
        }

        let mut workflows = BTreeMap::new();
        for (workflow_name, raw_definition) in raw.workflow {
            let definition = raw_definition.into_definition(workflow_name)?;
            workflows.insert(definition.workflow_name.clone(), definition);
        }

        Ok(Self { workflows })
    }

    pub fn load(path: &Path) -> Result<Self, WorkflowDefinitionError> {
        let contents =
            fs::read_to_string(path).map_err(WorkflowDefinitionError::ReadWorkflowDefinitions)?;
        Self::from_toml_str(&contents)
    }

    pub fn workflow(&self, workflow_name: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(workflow_name)
    }

    pub fn workflow_names(&self) -> Vec<&str> {
        self.workflows.keys().map(String::as_str).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowProgressState {
    pub workflow_name: String,
    pub lifecycle_state: WorkflowLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<WorkflowPhase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_phases: Vec<WorkflowPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing_summary: Option<String>,
}

impl WorkflowProgressState {
    pub fn validate(&self) -> Result<(), WorkflowDefinitionError> {
        if self.workflow_name.trim().is_empty() {
            return Err(WorkflowDefinitionError::MissingWorkflowName);
        }

        let mut seen = BTreeSet::new();
        for phase in &self.completed_phases {
            if !seen.insert(*phase) {
                return Err(WorkflowDefinitionError::DuplicateCompletedPhase {
                    workflow_name: self.workflow_name.clone(),
                    phase: *phase,
                });
            }
        }

        Ok(())
    }

    pub fn current_phase_text(&self) -> Option<String> {
        self.current_phase.map(|phase| phase.as_str().to_string())
    }

    pub fn next_action_text(&self) -> Option<String> {
        self.next_action.clone()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRegistryToml {
    #[serde(default)]
    workflow: BTreeMap<String, WorkflowDefinitionToml>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowDefinitionToml {
    goal_source: WorkflowGoalSource,
    #[serde(rename = "entry")]
    entry_phase: WorkflowPhase,
    phases: Vec<WorkflowPhase>,
    #[serde(default)]
    allow_review: bool,
    #[serde(default)]
    allow_governance: bool,
    #[serde(default)]
    when: WorkflowConditionToml,
    #[serde(default, rename = "output")]
    output_preferences: WorkflowOutputPreferences,
}

impl WorkflowDefinitionToml {
    fn into_definition(
        self,
        workflow_name: String,
    ) -> Result<WorkflowDefinition, WorkflowDefinitionError> {
        let mut conditional_phases = Vec::new();
        if let Some(condition_kind) = self.when.clarify {
            conditional_phases.push(ConditionalWorkflowPhase {
                phase: WorkflowPhase::Clarify,
                condition_kind,
                enabled: true,
            });
        }
        if let Some(condition_kind) = self.when.review {
            conditional_phases.push(ConditionalWorkflowPhase {
                phase: WorkflowPhase::Review,
                condition_kind,
                enabled: true,
            });
        }
        if let Some(condition_kind) = self.when.governance {
            conditional_phases.push(ConditionalWorkflowPhase {
                phase: WorkflowPhase::Govern,
                condition_kind,
                enabled: true,
            });
        }

        let definition = WorkflowDefinition {
            workflow_name,
            goal_source: self.goal_source,
            entry_phase: self.entry_phase,
            phases: self.phases,
            allow_review: self.allow_review,
            allow_governance: self.allow_governance,
            conditional_phases,
            output_preferences: self.output_preferences,
        };
        definition.validate()?;
        Ok(definition)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowConditionToml {
    #[serde(default)]
    clarify: Option<WorkflowConditionKind>,
    #[serde(default)]
    review: Option<WorkflowConditionKind>,
    #[serde(default)]
    governance: Option<WorkflowConditionKind>,
}

#[derive(Debug, Error)]
pub enum WorkflowDefinitionError {
    #[error("workflow name must not be empty")]
    MissingWorkflowName,
    #[error("workflow definitions file must contain at least one workflow")]
    MissingWorkflowDefinitions,
    #[error("workflow definitions could not be read: {0}")]
    ReadWorkflowDefinitions(std::io::Error),
    #[error("workflow definitions could not be parsed: {0}")]
    ParseWorkflowDefinitions(toml::de::Error),
    #[error("workflow `{workflow_name}` must declare at least one phase")]
    MissingPhases { workflow_name: String },
    #[error(
        "workflow `{workflow_name}` entry phase `{entry_phase}` must be the first declared phase"
    )]
    EntryPhaseMustBeFirst { workflow_name: String, entry_phase: WorkflowPhase },
    #[error("workflow `{workflow_name}` declares duplicate phase `{phase}`")]
    DuplicatePhase { workflow_name: String, phase: WorkflowPhase },
    #[error("workflow `{workflow_name}` includes `review` but allow_review is false")]
    ReviewPhaseNotAllowed { workflow_name: String },
    #[error("workflow `{workflow_name}` includes `govern` but allow_governance is false")]
    GovernancePhaseNotAllowed { workflow_name: String },
    #[error(
        "workflow `{workflow_name}` declares a conditional phase `{phase}` that is not present in phases"
    )]
    ConditionalPhaseMissing { workflow_name: String, phase: WorkflowPhase },
    #[error("workflow `{workflow_name}` does not support conditional phase `{phase}`")]
    UnsupportedConditionalPhase { workflow_name: String, phase: WorkflowPhase },
    #[error(
        "workflow `{workflow_name}` uses unexpected condition `{condition_kind}` for phase `{phase}`"
    )]
    UnexpectedConditionKind {
        workflow_name: String,
        phase: WorkflowPhase,
        condition_kind: WorkflowConditionKind,
    },
    #[error("workflow `{workflow_name}` repeats completed phase `{phase}` in persisted progress")]
    DuplicateCompletedPhase { workflow_name: String, phase: WorkflowPhase },
}
