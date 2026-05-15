//! Workflow definitions, project-scale paths, and persisted workflow progress.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// High-level project-scale path selected for a delivery effort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectScalePathKind {
    IdeaToCode,
    ExistingSystemChange,
    OperationalOrRisk,
}

/// Named project-scale stages used by delivery-path modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectScaleStageKind {
    Discovery,
    Requirements,
    DomainLanguage,
    DomainModel,
    SystemShaping,
    Architecture,
    Backlog,
    Change,
    Implementation,
    Refactor,
    Review,
    Verification,
    PrReview,
    Incident,
    SecurityAssessment,
    SystemAssessment,
    Migration,
    SupplyChainAnalysis,
}

impl fmt::Display for ProjectScaleStageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ProjectScaleStageKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Requirements => "requirements",
            Self::DomainLanguage => "domain-language",
            Self::DomainModel => "domain-model",
            Self::SystemShaping => "system-shaping",
            Self::Architecture => "architecture",
            Self::Backlog => "backlog",
            Self::Change => "change",
            Self::Implementation => "implementation",
            Self::Refactor => "refactor",
            Self::Review => "review",
            Self::Verification => "verification",
            Self::PrReview => "pr-review",
            Self::Incident => "incident",
            Self::SecurityAssessment => "security-assessment",
            Self::SystemAssessment => "system-assessment",
            Self::Migration => "migration",
            Self::SupplyChainAnalysis => "supply-chain-analysis",
        }
    }
}

/// Inputs used to propose a bounded project-scale path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScaleInput {
    pub goal: String,
    #[serde(default)]
    pub problem_unclear: bool,
    #[serde(default)]
    pub product_scope_unclear: bool,
    #[serde(default)]
    pub capability_structure_unclear: bool,
    #[serde(default)]
    pub architecture_material: bool,
    #[serde(default)]
    pub existing_system_change: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operational_entry: Option<ProjectScaleStageKind>,
}

/// One stage in a project-scale path with its supporting rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScaleStage {
    pub kind: ProjectScaleStageKind,
    pub reason: String,
}

/// Proposed project-scale path for a bounded delivery effort.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScalePath {
    pub kind: ProjectScalePathKind,
    pub goal: String,
    pub stages: Vec<ProjectScaleStage>,
    pub requires_confirmation: bool,
    pub next_action: String,
    pub unbounded_autonomy: bool,
}

/// Request to move from one project-scale stage to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScaleBoundaryRequest {
    pub active_stage: ProjectScaleStageKind,
    pub requested_stage: ProjectScaleStageKind,
    pub confirmed: bool,
}

/// Decision produced when evaluating a project-scale boundary transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScaleBoundaryDecision {
    pub blocked: bool,
    pub reason: String,
    pub next_action: String,
}

impl ProjectScalePath {
    /// Returns the ordered stage names for operator-facing summaries.
    pub fn stage_names(&self) -> String {
        self.stages.iter().map(|stage| stage.kind.as_str()).collect::<Vec<_>>().join(" -> ")
    }
}

/// Evaluates whether a requested project-scale transition can proceed immediately.
pub fn evaluate_project_scale_boundary(
    request: ProjectScaleBoundaryRequest,
) -> ProjectScaleBoundaryDecision {
    if request.active_stage == request.requested_stage {
        return ProjectScaleBoundaryDecision {
            blocked: false,
            reason: "requested action remains inside the active project-scale stage".to_string(),
            next_action: "continue_project_scale_stage".to_string(),
        };
    }

    if request.confirmed {
        return ProjectScaleBoundaryDecision {
            blocked: false,
            reason: format!(
                "confirmed transition from {} to {}",
                request.active_stage.as_str(),
                request.requested_stage.as_str()
            ),
            next_action: "continue_project_scale_stage".to_string(),
        };
    }

    ProjectScaleBoundaryDecision {
        blocked: true,
        reason: format!(
            "requested {} action exceeds current stage boundary {}; confirm before changing material delivery stage",
            request.requested_stage.as_str(),
            request.active_stage.as_str()
        ),
        next_action: "confirm_stage_transition".to_string(),
    }
}

/// Proposes a project-scale path from bounded input signals.
pub fn propose_project_scale_path(input: ProjectScaleInput) -> ProjectScalePath {
    let mut stages = Vec::new();

    let kind = if input.operational_entry.is_some() {
        ProjectScalePathKind::OperationalOrRisk
    } else if input.existing_system_change {
        ProjectScalePathKind::ExistingSystemChange
    } else {
        ProjectScalePathKind::IdeaToCode
    };

    if let Some(entry) = input.operational_entry {
        stages.push(ProjectScaleStage {
            kind: entry,
            reason: "operational or assessment entry stage requested".to_string(),
        });
    }

    match kind {
        ProjectScalePathKind::IdeaToCode => {
            if input.problem_unclear {
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::Discovery,
                    "problem framing is incomplete",
                );
            }
            if input.product_scope_unclear {
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::Requirements,
                    "product scope must be bounded",
                );
            }
            if input.capability_structure_unclear {
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::DomainLanguage,
                    "domain language is not fixed",
                );
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::DomainModel,
                    "domain model is not fixed",
                );
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::SystemShaping,
                    "capability structure is not fixed",
                );
            }
            if input.architecture_material {
                push_stage(
                    &mut stages,
                    ProjectScaleStageKind::Architecture,
                    "architecture and ownership boundaries are material",
                );
            }
            push_stage(&mut stages, ProjectScaleStageKind::Backlog, "delivery slices are required");
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Implementation,
                "first bounded implementation slice",
            );
            push_stage(&mut stages, ProjectScaleStageKind::Verification, "validate slice evidence");
            push_stage(&mut stages, ProjectScaleStageKind::PrReview, "review merge-ready diff");
        }
        ProjectScalePathKind::ExistingSystemChange => {
            push_stage(
                &mut stages,
                ProjectScaleStageKind::SystemAssessment,
                "current-state coverage may be weak",
            );
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Change,
                "establish existing-system change boundary",
            );
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Implementation,
                "bounded existing-system change slice",
            );
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Verification,
                "validate change evidence",
            );
            push_stage(&mut stages, ProjectScaleStageKind::PrReview, "review merge-ready diff");
        }
        ProjectScalePathKind::OperationalOrRisk => {
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Change,
                "route assessment output into a bounded delivery change",
            );
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Implementation,
                "bounded follow-up implementation slice",
            );
            push_stage(
                &mut stages,
                ProjectScaleStageKind::Verification,
                "validate follow-up evidence",
            );
            push_stage(&mut stages, ProjectScaleStageKind::PrReview, "review merge-ready diff");
        }
    }

    ProjectScalePath {
        kind,
        goal: input.goal,
        stages,
        requires_confirmation: true,
        next_action: "confirm_project_scale_path".to_string(),
        unbounded_autonomy: false,
    }
}

fn push_stage(stages: &mut Vec<ProjectScaleStage>, kind: ProjectScaleStageKind, reason: &str) {
    if stages.iter().any(|stage| stage.kind == kind) {
        return;
    }
    stages.push(ProjectScaleStage { kind, reason: reason.to_string() });
}

/// Source of the goal that drives a workflow definition.
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

/// Ordered workflow phases supported by the workflow layer.
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

/// Conditions that can gate optional workflow phases.
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

/// Persisted lifecycle state of a workflow run.
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

/// Output preferences attached to a workflow definition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowOutputPreferences {
    #[serde(default)]
    pub next_command: bool,
    #[serde(default)]
    pub routing_summary: bool,
    #[serde(default)]
    pub execution_condition: bool,
}

/// Conditional workflow phase that is enabled only when its condition is satisfied.
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

/// Persisted workflow definition loaded from workflow configuration.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_when: Option<String>,
}

impl WorkflowDefinition {
    /// Validates the workflow definition and its conditional phase wiring.
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

    /// Returns a short summary suitable for discovery listings.
    pub fn discovery_summary(&self) -> String {
        self.summary.clone().unwrap_or_else(|| {
            format!(
                "bounded workflow covering {}",
                self.phases.iter().map(|phase| phase.as_str()).collect::<Vec<_>>().join(" -> ")
            )
        })
    }

    /// Returns the ordered phase chain as text.
    pub fn phase_chain_text(&self) -> String {
        self.phases.iter().map(|phase| phase.as_str()).collect::<Vec<_>>().join(" -> ")
    }
}

/// Availability state shown when discovering workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowAvailabilityState {
    Ready,
    Invalid,
    Unsupported,
}

/// Discovery entry presented to CLI callers for one workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDiscoveryEntry {
    pub workflow_name: String,
    pub summary: String,
    pub phases: Vec<WorkflowPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_when: Option<String>,
    pub invocation_command: String,
    pub availability_state: WorkflowAvailabilityState,
}

/// Named delivery-path definition for project-scale orchestration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryPathDefinition {
    pub delivery_path_name: String,
    pub description: String,
    pub stages: Vec<ProjectScaleStageKind>,
    pub adaptive: bool,
}

impl DeliveryPathDefinition {
    /// Validates the delivery-path definition.
    pub fn validate(&self) -> Result<(), WorkflowDefinitionError> {
        if self.stages.is_empty() {
            return Err(WorkflowDefinitionError::MissingDeliveryPathStages {
                delivery_path_name: self.delivery_path_name.clone(),
            });
        }

        let mut seen = BTreeSet::new();
        for stage in &self.stages {
            if !seen.insert(*stage) {
                return Err(WorkflowDefinitionError::DuplicateDeliveryPathStage {
                    delivery_path_name: self.delivery_path_name.clone(),
                    stage: *stage,
                });
            }
        }

        Ok(())
    }

    /// Returns the ordered stage names for operator-facing summaries.
    pub fn stage_names(&self) -> String {
        self.stages.iter().map(|stage| stage.as_str()).collect::<Vec<_>>().join(" -> ")
    }
}

/// In-memory registry of workflow and delivery-path definitions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowRegistry {
    workflows: BTreeMap<String, WorkflowDefinition>,
    delivery_paths: BTreeMap<String, DeliveryPathDefinition>,
}

impl WorkflowRegistry {
    /// Parses a workflow registry from TOML contents.
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

        let mut delivery_paths = BTreeMap::new();
        for (delivery_path_name, raw_definition) in raw.delivery_paths {
            let definition = raw_definition.into_definition(delivery_path_name)?;
            delivery_paths.insert(definition.delivery_path_name.clone(), definition);
        }

        Ok(Self { workflows, delivery_paths })
    }

    /// Loads a workflow registry from disk.
    pub fn load(path: &Path) -> Result<Self, WorkflowDefinitionError> {
        let contents =
            fs::read_to_string(path).map_err(WorkflowDefinitionError::ReadWorkflowDefinitions)?;
        Self::from_toml_str(&contents)
    }

    /// Returns one workflow definition by name.
    pub fn workflow(&self, workflow_name: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(workflow_name)
    }

    /// Returns all workflow names in registry order.
    pub fn workflow_names(&self) -> Vec<&str> {
        self.workflows.keys().map(String::as_str).collect()
    }

    /// Returns one delivery-path definition by name.
    pub fn delivery_path(&self, delivery_path_name: &str) -> Option<&DeliveryPathDefinition> {
        self.delivery_paths.get(delivery_path_name)
    }

    /// Returns all delivery-path names in registry order.
    pub fn delivery_path_names(&self) -> Vec<&str> {
        self.delivery_paths.keys().map(String::as_str).collect()
    }

    /// Builds discovery entries for workflows in the target workspace.
    pub fn discovery_entries(&self, workspace: &Path) -> Vec<WorkflowDiscoveryEntry> {
        self.workflows
            .values()
            .map(|workflow| WorkflowDiscoveryEntry {
                workflow_name: workflow.workflow_name.clone(),
                summary: workflow.discovery_summary(),
                phases: workflow.phases.clone(),
                recommended_when: workflow.recommended_when.clone(),
                invocation_command: format!(
                    "boundline workflow run {} --workspace {}",
                    workflow.workflow_name,
                    workspace.display()
                ),
                availability_state: WorkflowAvailabilityState::Ready,
            })
            .collect()
    }
}

/// Persisted workflow progress state captured in session-native execution.
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
    /// Validates persisted workflow progress.
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

    /// Returns the current phase as text, when the workflow is inside a phase.
    pub fn current_phase_text(&self) -> Option<String> {
        self.current_phase.map(|phase| phase.as_str().to_string())
    }

    /// Returns the recommended next action, when one is present.
    pub fn next_action_text(&self) -> Option<String> {
        self.next_action.clone()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRegistryToml {
    #[serde(default)]
    workflow: BTreeMap<String, WorkflowDefinitionToml>,
    #[serde(default)]
    delivery_paths: BTreeMap<String, DeliveryPathDefinitionToml>,
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
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    recommended_when: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeliveryPathDefinitionToml {
    description: String,
    stages: Vec<ProjectScaleStageKind>,
    #[serde(default)]
    adaptive: bool,
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
            summary: self.summary,
            recommended_when: self.recommended_when,
        };
        definition.validate()?;
        Ok(definition)
    }
}

impl DeliveryPathDefinitionToml {
    fn into_definition(
        self,
        delivery_path_name: String,
    ) -> Result<DeliveryPathDefinition, WorkflowDefinitionError> {
        let definition = DeliveryPathDefinition {
            delivery_path_name,
            description: self.description,
            stages: self.stages,
            adaptive: self.adaptive,
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

/// Validation and loading errors for workflow definitions and persisted workflow state.
#[derive(Debug, Error)]
pub enum WorkflowDefinitionError {
    #[error("workflow name must not be empty")]
    MissingWorkflowName,
    #[error("workflow definitions file must contain at least one workflow")]
    MissingWorkflowDefinitions,
    #[error("workflow `{workflow_name}` is not defined")]
    MissingNamedWorkflow { workflow_name: String },
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
    #[error("delivery path `{delivery_path_name}` must declare at least one stage")]
    MissingDeliveryPathStages { delivery_path_name: String },
    #[error("delivery path `{delivery_path_name}` declares duplicate stage `{stage}`")]
    DuplicateDeliveryPathStage { delivery_path_name: String, stage: ProjectScaleStageKind },
}
