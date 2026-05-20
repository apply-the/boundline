use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardAuthority {
    SessionNative,
    CompatibilityTrace,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionCondition {
    Ready,
    Waiting,
    Blocked,
    Failed,
    Exhausted,
    Invalid,
    Degraded,
    Complete,
}

impl ExecutionCondition {
    pub const fn requires_blocking_reason(self) -> bool {
        matches!(
            self,
            Self::Waiting
                | Self::Blocked
                | Self::Failed
                | Self::Exhausted
                | Self::Invalid
                | Self::Degraded
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardColorProfile {
    Color,
    Monochrome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DashboardActionKind {
    Confirm,
    Reject,
    Replan,
    Recover,
    Launch,
    Continue,
    InspectOnly,
}

impl DashboardActionKind {
    pub const fn mutates_state(self) -> bool {
        !matches!(self, Self::InspectOnly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardExpectedResult {
    PlannedOrConfirmed,
    ReplanRequested,
    RecoverySelected,
    RunningOrTerminal,
    SessionLaunched,
    FocusChanged,
    Refused,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradedReason {
    InvalidWorkspace,
    MissingActiveSession,
    InvalidSessionJson,
    StaleTraceReference,
    TerminalUnsupported,
    DashboardUnavailable,
    RuntimeCommandUnavailable,
    StateReadFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradedSeverity {
    Info,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub snapshot_id: String,
    pub workspace_ref: String,
    pub captured_at: String,
    pub authority: DashboardAuthority,
    pub session_revision: Option<u64>,
    pub session: Option<DashboardSessionView>,
    #[serde(default)]
    pub timeline: Vec<RuntimeEventProjection>,
    pub panels: DashboardPanels,
    #[serde(default)]
    pub actions: Vec<DashboardActionOption>,
    pub degraded_state: Option<DegradedDashboardState>,
    pub branding: DashboardBrandMark,
}

impl DashboardSnapshot {
    pub fn validate(&self) -> Result<(), DashboardValidationError> {
        if self.snapshot_id.trim().is_empty() {
            return Err(DashboardValidationError::MissingSnapshotId);
        }
        if self.workspace_ref.trim().is_empty() {
            return Err(DashboardValidationError::MissingWorkspaceRef);
        }
        if self.session.is_none() && self.degraded_state.is_none() {
            return Err(DashboardValidationError::MissingSessionOrDegradedState);
        }
        if let Some(session) = &self.session {
            session.validate()?;
        }
        for action in &self.actions {
            action.validate()?;
        }
        self.branding.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSessionView {
    pub session_id: String,
    pub goal: String,
    pub route_kind: String,
    pub route_owner: String,
    pub active_flow: Option<String>,
    pub flow_state: Option<String>,
    pub goal_plan_state: Option<String>,
    pub goal_plan_revision: Option<usize>,
    pub current_stage: Option<String>,
    pub current_step_id: Option<String>,
    pub current_step_index: Option<usize>,
    pub execution_condition: ExecutionCondition,
    pub latest_status: String,
    pub next_action_label: String,
    pub next_command: String,
    pub blocking_reason: Option<String>,
    pub compatibility_context: Option<String>,
}

impl DashboardSessionView {
    fn validate(&self) -> Result<(), DashboardValidationError> {
        if self.session_id.trim().is_empty() {
            return Err(DashboardValidationError::MissingSessionId);
        }
        if self.route_kind.trim().is_empty() {
            return Err(DashboardValidationError::MissingRouteKind);
        }
        if self.next_command.trim().is_empty() {
            return Err(DashboardValidationError::MissingNextCommand);
        }
        if self.execution_condition.requires_blocking_reason()
            && self.blocking_reason.as_deref().unwrap_or_default().trim().is_empty()
        {
            return Err(DashboardValidationError::MissingBlockingReason);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEventProjection {
    pub event_id: String,
    pub event_kind: String,
    pub occurred_at: String,
    pub stage: Option<String>,
    pub step_id: Option<String>,
    pub status: String,
    pub headline: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub trace_ref: Option<String>,
    #[serde(default)]
    pub details: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardPanels {
    pub goal_plan: Option<GoalPlanPanel>,
    #[serde(default)]
    pub context_pack: Vec<ContextPackPanelItem>,
    #[serde(default)]
    pub evidence: Vec<EvidencePanelItem>,
    #[serde(default)]
    pub context_degradation: Vec<UnavailablePanelFact>,
    #[serde(default)]
    pub stop_rules: Vec<UnavailablePanelFact>,
    #[serde(default)]
    pub findings: Vec<FindingPanelItem>,
    #[serde(default)]
    pub checkpoints: Vec<CheckpointPanelItem>,
    #[serde(default)]
    pub governed_references: Vec<GovernedReferencePanelItem>,
    #[serde(default)]
    pub diagnostics: Vec<DashboardDiagnosticItem>,
}

impl DashboardPanels {
    pub fn empty() -> Self {
        Self {
            goal_plan: None,
            context_pack: Vec::new(),
            evidence: Vec::new(),
            context_degradation: Vec::new(),
            stop_rules: Vec::new(),
            findings: Vec::new(),
            checkpoints: Vec::new(),
            governed_references: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalPlanPanel {
    pub revision: usize,
    pub state: String,
    pub verification_strategy: Option<String>,
    #[serde(default)]
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPackPanelItem {
    pub reason: String,
    pub source: String,
    pub budget: Option<String>,
    pub authority: String,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePanelItem {
    pub label: String,
    pub evidence_ref: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnavailablePanelFact {
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingPanelItem {
    pub status: String,
    pub severity: String,
    pub summary: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointPanelItem {
    pub checkpoint_ref: String,
    pub scope: String,
    pub restore_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedReferencePanelItem {
    pub reference: String,
    pub readiness: String,
    pub provenance: String,
    pub approval_cue: Option<String>,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardDiagnosticItem {
    pub category: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardActionOption {
    pub action_kind: DashboardActionKind,
    pub label: String,
    pub description: String,
    pub requires_reason: bool,
    pub requires_confirmation: bool,
    pub target_session_revision: Option<u64>,
    pub expected_result: DashboardExpectedResult,
    pub disabled_reason: Option<String>,
}

impl DashboardActionOption {
    fn validate(&self) -> Result<(), DashboardValidationError> {
        if self.label.trim().is_empty() {
            return Err(DashboardValidationError::MissingActionLabel);
        }
        if self.action_kind.mutates_state() && self.target_session_revision.is_none() {
            return Err(DashboardValidationError::MissingActionRevision);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradedDashboardState {
    pub reason: DegradedReason,
    pub severity: DegradedSeverity,
    #[serde(default)]
    pub available_commands: Vec<String>,
    #[serde(default)]
    pub unavailable_panels: Vec<String>,
    pub recovery_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardBrandMark {
    #[serde(default)]
    pub wordmark_lines: Vec<String>,
    pub color_profile: DashboardColorProfile,
    pub min_width: usize,
    pub fallback_label: String,
}

impl DashboardBrandMark {
    fn validate(&self) -> Result<(), DashboardValidationError> {
        if self.wordmark_lines.is_empty() {
            return Err(DashboardValidationError::MissingWordmark);
        }
        if self.fallback_label.trim().is_empty() {
            return Err(DashboardValidationError::MissingWordmark);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardActionRequest {
    pub request_id: String,
    pub workspace_ref: String,
    pub action_kind: DashboardActionKind,
    pub target_session_id: Option<String>,
    pub target_session_revision: Option<u64>,
    pub operator_reason: Option<String>,
    pub requested_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardActionOutcome {
    Applied,
    Refused,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardRefusalReason {
    StaleSessionRevision,
    InvalidWorkspace,
    MissingActiveSession,
    MissingRequiredContext,
    BlockedByStopRule,
    ApprovalWaiting,
    UnsupportedAction,
    DashboardDegraded,
    RuntimeCommandUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardActionResult {
    pub request_id: String,
    pub outcome: DashboardActionOutcome,
    pub state_transition: Option<String>,
    pub next_snapshot_ref: Option<String>,
    pub next_command: Option<String>,
    #[serde(default)]
    pub trace_refs: Vec<String>,
    pub refusal_reason: Option<DashboardRefusalReason>,
    pub operator_message: String,
}

#[derive(Debug, Error)]
pub enum DashboardValidationError {
    #[error("dashboard snapshot requires snapshot_id")]
    MissingSnapshotId,
    #[error("dashboard snapshot requires workspace_ref")]
    MissingWorkspaceRef,
    #[error("dashboard snapshot requires either session or degraded_state")]
    MissingSessionOrDegradedState,
    #[error("dashboard session requires session_id")]
    MissingSessionId,
    #[error("dashboard session requires route_kind")]
    MissingRouteKind,
    #[error("dashboard session requires next_command")]
    MissingNextCommand,
    #[error("dashboard session requires blocking_reason for non-ready condition")]
    MissingBlockingReason,
    #[error("dashboard action requires label")]
    MissingActionLabel,
    #[error("mutating dashboard action requires target_session_revision")]
    MissingActionRevision,
    #[error("dashboard branding requires wordmark")]
    MissingWordmark,
}
