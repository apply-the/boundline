use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::flow::built_in_flow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceRuntimeKind {
    #[default]
    Local,
    Canon,
}

impl std::fmt::Display for GovernanceRuntimeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
            Self::Canon => f.write_str("canon"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum CanonModeSelectionPreference {
    Manual,
    #[default]
    AutoConfirm,
    Auto,
}

impl std::fmt::Display for CanonModeSelectionPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => f.write_str("manual"),
            Self::AutoConfirm => f.write_str("auto-confirm"),
            Self::Auto => f.write_str("auto"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemContextBinding {
    New,
    Existing,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
pub enum CanonMode {
    Requirements,
    Discovery,
    #[serde(alias = "system_shaping")]
    SystemShaping,
    Architecture,
    Backlog,
    Change,
    Implementation,
    Refactor,
    Review,
    Verification,
    Incident,
    #[serde(alias = "security_assessment")]
    SecurityAssessment,
    #[serde(alias = "system_assessment")]
    SystemAssessment,
    Migration,
    #[serde(alias = "supply_chain_analysis")]
    SupplyChainAnalysis,
    #[serde(alias = "pr_review")]
    PrReview,
}

impl CanonMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requirements => "requirements",
            Self::Discovery => "discovery",
            Self::SystemShaping => "system-shaping",
            Self::Architecture => "architecture",
            Self::Backlog => "backlog",
            Self::Change => "change",
            Self::Implementation => "implementation",
            Self::Refactor => "refactor",
            Self::Review => "review",
            Self::Verification => "verification",
            Self::Incident => "incident",
            Self::SecurityAssessment => "security-assessment",
            Self::SystemAssessment => "system-assessment",
            Self::Migration => "migration",
            Self::SupplyChainAnalysis => "supply-chain-analysis",
            Self::PrReview => "pr-review",
        }
    }

    pub const fn primary_document_name(self) -> &'static str {
        match self {
            Self::Requirements => "requirements.md",
            Self::Discovery => "discovery.md",
            Self::SystemShaping => "system-shaping.md",
            Self::Architecture => "architecture.md",
            Self::Backlog => "backlog.md",
            Self::Change => "change.md",
            Self::Implementation => "implementation.md",
            Self::Refactor => "refactor.md",
            Self::Review => "review.md",
            Self::Verification => "verification.md",
            Self::Incident => "incident.md",
            Self::SecurityAssessment => "security-assessment.md",
            Self::SystemAssessment => "system-assessment.md",
            Self::Migration => "migration.md",
            Self::SupplyChainAnalysis => "supply-chain-analysis.md",
            Self::PrReview => "pr-review.md",
        }
    }

    pub const fn requires_existing_context(self) -> bool {
        matches!(
            self,
            Self::Backlog
                | Self::Change
                | Self::Implementation
                | Self::Refactor
                | Self::Review
                | Self::Verification
                | Self::Incident
                | Self::SecurityAssessment
                | Self::SystemAssessment
                | Self::Migration
                | Self::SupplyChainAnalysis
                | Self::PrReview
        )
    }

    pub fn expected_document_refs(self, packet_ref: &str) -> Vec<String> {
        vec![format!("{}/{}", packet_ref.trim_end_matches('/'), self.primary_document_name())]
    }
}

impl std::fmt::Display for CanonMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for CanonMode {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().replace('_', "-").as_str() {
            "requirements" => Ok(Self::Requirements),
            "discovery" => Ok(Self::Discovery),
            "system-shaping" => Ok(Self::SystemShaping),
            "architecture" => Ok(Self::Architecture),
            "backlog" => Ok(Self::Backlog),
            "change" => Ok(Self::Change),
            "implementation" => Ok(Self::Implementation),
            "refactor" => Ok(Self::Refactor),
            "review" => Ok(Self::Review),
            "verification" => Ok(Self::Verification),
            "incident" => Ok(Self::Incident),
            "security-assessment" => Ok(Self::SecurityAssessment),
            "system-assessment" => Ok(Self::SystemAssessment),
            "migration" => Ok(Self::Migration),
            "supply-chain-analysis" => Ok(Self::SupplyChainAnalysis),
            "pr-review" => Ok(Self::PrReview),
            other => Err(format!("unknown Canon mode `{other}`")),
        }
    }
}

/// The current project-scale Canon mode set supported through Boundline.
pub const CANONICAL_MODES: [CanonMode; 16] = [
    CanonMode::Discovery,
    CanonMode::Requirements,
    CanonMode::SystemShaping,
    CanonMode::Architecture,
    CanonMode::Backlog,
    CanonMode::Change,
    CanonMode::Implementation,
    CanonMode::Refactor,
    CanonMode::Review,
    CanonMode::Verification,
    CanonMode::PrReview,
    CanonMode::Incident,
    CanonMode::SecurityAssessment,
    CanonMode::SystemAssessment,
    CanonMode::Migration,
    CanonMode::SupplyChainAnalysis,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernedStageCategory {
    Planning,
    ExecutionGuidance,
    Review,
    Verification,
    Assessment,
    Operational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedStageCatalogEntry {
    pub mode: CanonMode,
    pub consider_when: &'static str,
    pub required_system_context: &'static str,
    pub category: GovernedStageCategory,
    pub voting_may_be_required: bool,
    pub can_lead_to_implementation_or_refactor: bool,
    pub recommendation_only: bool,
}

pub fn governed_stage_catalog() -> &'static [GovernedStageCatalogEntry] {
    &GOVERNED_STAGE_CATALOG
}

pub fn validate_canon_capabilities_for_mode(
    snapshot: &CanonCapabilitySnapshot,
    mode: CanonMode,
) -> Result<(), String> {
    if !snapshot.supported_modes.contains(&mode) {
        return Err(format!(
            "Canon mode `{}` is unsupported by the installed capability snapshot",
            mode.as_str()
        ));
    }
    if !snapshot.operations.iter().any(|operation| operation == "capabilities") {
        return Err("Canon capability snapshot is missing `capabilities` operation".to_string());
    }
    Ok(())
}

static GOVERNED_STAGE_CATALOG: [GovernedStageCatalogEntry; 16] = [
    GovernedStageCatalogEntry {
        mode: CanonMode::Discovery,
        consider_when: "problem, user, or evidence is ambiguous",
        required_system_context: "goal, available briefs, known unknowns",
        category: GovernedStageCategory::Planning,
        voting_may_be_required: false,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Requirements,
        consider_when: "product scope or acceptance boundaries must be bounded",
        required_system_context: "goal, stakeholders or authored brief, constraints",
        category: GovernedStageCategory::Planning,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::SystemShaping,
        consider_when: "capability structure or domain boundaries are not fixed",
        required_system_context: "requirements, current system evidence, domain constraints",
        category: GovernedStageCategory::Planning,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Architecture,
        consider_when: "boundaries, invariants, C4, ADR, or structural decisions matter",
        required_system_context: "requirements, system-shaping evidence, current architecture",
        category: GovernedStageCategory::Planning,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Backlog,
        consider_when: "governed decomposition into delivery slices is needed",
        required_system_context: "requirements or architecture packet, constraints, priorities",
        category: GovernedStageCategory::Planning,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Change,
        consider_when: "existing-system modification boundary must be established",
        required_system_context: "current system evidence, target slice, validation strategy",
        category: GovernedStageCategory::ExecutionGuidance,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Implementation,
        consider_when: "a bounded behavior slice is ready to execute",
        required_system_context: "confirmed plan, target files, validation command",
        category: GovernedStageCategory::ExecutionGuidance,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Refactor,
        consider_when: "structural cleanup is needed without new behavior",
        required_system_context: "current behavior evidence, preservation tests, target slice",
        category: GovernedStageCategory::ExecutionGuidance,
        voting_may_be_required: false,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Review,
        consider_when: "work product or packet needs governed review",
        required_system_context: "evidence packet, changed files or artifacts, criteria",
        category: GovernedStageCategory::Review,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Verification,
        consider_when: "claims need governed validation evidence",
        required_system_context: "validation outputs, changed files, acceptance criteria",
        category: GovernedStageCategory::Verification,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::PrReview,
        consider_when: "a diff or worktree is ready for merge review",
        required_system_context: "base/head refs, diff summary, validation evidence",
        category: GovernedStageCategory::Review,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Incident,
        consider_when: "operational issue requires containment or follow-up reasoning",
        required_system_context: "incident brief, timeline, impact, current system state",
        category: GovernedStageCategory::Operational,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::SecurityAssessment,
        consider_when: "security risk or control coverage must be assessed",
        required_system_context: "threat context, assets, findings, current controls",
        category: GovernedStageCategory::Assessment,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::SystemAssessment,
        consider_when: "current-state understanding is weak or systemic risk exists",
        required_system_context: "system inventory, traces, architecture docs, known gaps",
        category: GovernedStageCategory::Assessment,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::Migration,
        consider_when: "cutover, fallback, compatibility, or data movement is material",
        required_system_context: "source/target state, rollback plan, validation strategy",
        category: GovernedStageCategory::Operational,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
    GovernedStageCatalogEntry {
        mode: CanonMode::SupplyChainAnalysis,
        consider_when: "dependency, provenance, license, or package risk is material",
        required_system_context: "dependency evidence, manifests, findings, policy",
        category: GovernedStageCategory::Assessment,
        voting_may_be_required: true,
        can_lead_to_implementation_or_refactor: true,
        recommendation_only: true,
    },
];

const DELIVERY_REQUIREMENTS_MODES: [CanonMode; 1] = [CanonMode::Requirements];
const DELIVERY_ARCHITECTURE_MODES: [CanonMode; 1] = [CanonMode::Architecture];
const DELIVERY_BACKLOG_MODES: [CanonMode; 1] = [CanonMode::Backlog];
const DELIVERY_IMPLEMENTATION_MODES: [CanonMode; 1] = [CanonMode::Implementation];
const CHANGE_UNDERSTAND_MODES: [CanonMode; 2] = [CanonMode::Change, CanonMode::Discovery];
const CHANGE_IMPLEMENT_MODES: [CanonMode; 2] = [CanonMode::Implementation, CanonMode::Refactor];
const CHANGE_VERIFY_MODES: [CanonMode; 4] = [
    CanonMode::SecurityAssessment,
    CanonMode::Verification,
    CanonMode::Review,
    CanonMode::PrReview,
];
const BUG_FIX_INVESTIGATE_MODES: [CanonMode; 3] =
    [CanonMode::Discovery, CanonMode::Change, CanonMode::Incident];
const BUG_FIX_IMPLEMENT_MODES: [CanonMode; 2] = [CanonMode::Implementation, CanonMode::Refactor];
const BUG_FIX_VERIFY_MODES: [CanonMode; 4] = [
    CanonMode::SecurityAssessment,
    CanonMode::Verification,
    CanonMode::Review,
    CanonMode::PrReview,
];
const NO_CANON_MODES: [CanonMode; 0] = [];

pub fn supported_canon_modes_for_stage(flow_name: &str, stage_id: &str) -> &'static [CanonMode] {
    match (flow_name, stage_id) {
        ("delivery", "requirements") => &DELIVERY_REQUIREMENTS_MODES,
        ("delivery", "architecture") => &DELIVERY_ARCHITECTURE_MODES,
        ("delivery", "backlog") => &DELIVERY_BACKLOG_MODES,
        ("delivery", "implementation") => &DELIVERY_IMPLEMENTATION_MODES,
        ("change", "understand-change") => &CHANGE_UNDERSTAND_MODES,
        ("change", "implement") => &CHANGE_IMPLEMENT_MODES,
        ("change", "verify") => &CHANGE_VERIFY_MODES,
        ("bug-fix", "investigate") => &BUG_FIX_INVESTIGATE_MODES,
        ("bug-fix", "implement") => &BUG_FIX_IMPLEMENT_MODES,
        ("bug-fix", "verify") => &BUG_FIX_VERIFY_MODES,
        _ => &NO_CANON_MODES,
    }
}

pub fn resolved_canon_mode(
    policy: &StageGovernancePolicy,
    default_runtime: GovernanceRuntimeKind,
) -> Option<CanonMode> {
    if policy.effective_runtime(default_runtime) != GovernanceRuntimeKind::Canon {
        return None;
    }

    policy.canon_mode.or_else(|| {
        let supported_modes = supported_canon_modes_for_stage(&policy.flow_name, &policy.stage_id);
        (supported_modes.len() == 1).then_some(supported_modes[0])
    })
}

pub fn candidate_canon_modes(
    policy: &StageGovernancePolicy,
    default_runtime: GovernanceRuntimeKind,
) -> Vec<CanonMode> {
    if policy.effective_runtime(default_runtime) != GovernanceRuntimeKind::Canon
        || policy.canon_mode.is_some()
    {
        return Vec::new();
    }

    supported_canon_modes_for_stage(&policy.flow_name, &policy.stage_id).to_vec()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonRuntimeConfig {
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_system_context: Option<SystemContextBinding>,
}

impl CanonRuntimeConfig {
    pub fn validate(&self) -> Result<(), GovernanceProfileError> {
        if self.command.trim().is_empty() {
            return Err(GovernanceProfileError::MissingCanonCommand);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageGovernancePolicy {
    pub flow_name: String,
    pub stage_id: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub autopilot: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<GovernanceRuntimeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_mode: Option<CanonMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_context: Option<SystemContextBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl StageGovernancePolicy {
    pub fn stage_key(&self) -> String {
        format!("{}:{}", self.flow_name, self.stage_id)
    }

    pub fn effective_runtime(
        &self,
        default_runtime: GovernanceRuntimeKind,
    ) -> GovernanceRuntimeKind {
        self.runtime.unwrap_or(default_runtime)
    }

    pub fn validate(
        &self,
        default_runtime: GovernanceRuntimeKind,
        canon: Option<&CanonRuntimeConfig>,
    ) -> Result<(), GovernanceProfileError> {
        if self.required && !self.enabled {
            return Err(GovernanceProfileError::RequiredPolicyNotEnabled(self.stage_key()));
        }

        if self.autopilot && !self.enabled {
            return Err(GovernanceProfileError::AutopilotPolicyNotEnabled(self.stage_key()));
        }

        let flow = built_in_flow(&self.flow_name)
            .ok_or_else(|| GovernanceProfileError::UnsupportedFlow(self.flow_name.clone()))?;
        if !flow.stages.iter().any(|stage| stage.id == self.stage_id) {
            return Err(GovernanceProfileError::UnsupportedStage {
                flow_name: self.flow_name.clone(),
                stage_id: self.stage_id.clone(),
            });
        }

        if self.effective_runtime(default_runtime) != GovernanceRuntimeKind::Canon {
            return Ok(());
        }

        let canon =
            canon.ok_or_else(|| GovernanceProfileError::MissingCanonConfig(self.stage_key()))?;
        let supported_modes = supported_canon_modes_for_stage(&self.flow_name, &self.stage_id);
        if supported_modes.is_empty() {
            return Err(GovernanceProfileError::UnsupportedCanonStage(self.stage_key()));
        }

        if let Some(mode) = self.canon_mode
            && !supported_modes.contains(&mode)
        {
            return Err(GovernanceProfileError::CanonModeNotAllowed {
                stage_key: self.stage_key(),
                mode,
            });
        }

        let system_context = self.system_context.or(canon.default_system_context);
        let risk = self.risk.as_deref().or(canon.default_risk.as_deref());
        let zone = self.zone.as_deref().or(canon.default_zone.as_deref());
        let owner = self.owner.as_deref().or(canon.default_owner.as_deref());

        if system_context.is_none() {
            return Err(GovernanceProfileError::MissingCanonField {
                stage_key: self.stage_key(),
                field: "system_context",
            });
        }
        if risk.is_none() {
            return Err(GovernanceProfileError::MissingCanonField {
                stage_key: self.stage_key(),
                field: "risk",
            });
        }
        if zone.is_none() {
            return Err(GovernanceProfileError::MissingCanonField {
                stage_key: self.stage_key(),
                field: "zone",
            });
        }
        if owner.is_none() {
            return Err(GovernanceProfileError::MissingCanonField {
                stage_key: self.stage_key(),
                field: "owner",
            });
        }

        let resolved_mode = self
            .canon_mode
            .or_else(|| if supported_modes.len() == 1 { Some(supported_modes[0]) } else { None });

        if let Some(mode) = resolved_mode
            && mode.requires_existing_context()
            && system_context != Some(SystemContextBinding::Existing)
        {
            return Err(GovernanceProfileError::InvalidSystemContextForMode {
                stage_key: self.stage_key(),
                mode,
                system_context: system_context.expect("system_context presence checked above"),
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceProfile {
    #[serde(default)]
    pub default_runtime: GovernanceRuntimeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon: Option<CanonRuntimeConfig>,
    #[serde(default)]
    pub stages: Vec<StageGovernancePolicy>,
}

impl GovernanceProfile {
    pub fn validate(&self) -> Result<(), GovernanceProfileError> {
        let mut seen_stage_keys = BTreeSet::new();
        let needs_canon = self.stages.iter().any(|policy| {
            policy.effective_runtime(self.default_runtime) == GovernanceRuntimeKind::Canon
        });

        if needs_canon {
            self.canon
                .as_ref()
                .ok_or(GovernanceProfileError::MissingCanonConfig(
                    "governance profile".to_string(),
                ))?
                .validate()?;
        }

        for policy in &self.stages {
            let stage_key = policy.stage_key();
            if !seen_stage_keys.insert(stage_key.clone()) {
                return Err(GovernanceProfileError::DuplicateStagePolicy(stage_key));
            }
            policy.validate(self.default_runtime, self.canon.as_ref())?;
        }

        Ok(())
    }

    pub fn stage_policy(&self, flow_name: &str, stage_id: &str) -> Option<&StageGovernancePolicy> {
        self.stages
            .iter()
            .find(|policy| policy.flow_name == flow_name && policy.stage_id == stage_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceLifecycleState {
    PendingSelection,
    Running,
    GovernedReady,
    AwaitingApproval,
    Blocked,
    Incomplete,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    NotNeeded,
    Requested,
    Granted,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketReadiness {
    Pending,
    Incomplete,
    Reusable,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonCapabilitySnapshot {
    pub canon_version: String,
    #[serde(default)]
    pub supported_schema_versions: Vec<String>,
    #[serde(default)]
    pub operations: Vec<String>,
    #[serde(default)]
    pub supported_modes: Vec<CanonMode>,
    #[serde(default)]
    pub status_values: Vec<String>,
    #[serde(default)]
    pub approval_state_values: Vec<String>,
    #[serde(default)]
    pub packet_readiness_values: Vec<String>,
    #[serde(default)]
    pub compatibility_notes: Vec<String>,
}

impl CanonCapabilitySnapshot {
    pub fn summary_text(&self) -> String {
        let version = self.canon_version.trim();
        if version.is_empty() {
            "Canon capabilities available".to_string()
        } else {
            format!("Canon {version} capabilities available")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonResultActionSummary {
    pub label: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonModeSummary {
    pub headline: String,
    pub artifact_packet_summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_posture: Option<String>,
    pub primary_artifact_title: String,
    pub primary_artifact_path: String,
    pub primary_artifact_action: CanonResultActionSummary,
    pub result_excerpt: String,
    #[serde(default)]
    pub action_chip_labels: Vec<String>,
}

impl CanonModeSummary {
    pub fn summary_text(&self) -> String {
        match self.execution_posture.as_deref() {
            Some(execution_posture) => format!(
                "{}; {}; execution posture: {execution_posture}",
                self.headline, self.artifact_packet_summary
            ),
            None => format!("{}; {}", self.headline, self.artifact_packet_summary),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonPossibleActionSummary {
    pub action: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonRecommendedActionSummary {
    pub action: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonEvidenceInspectSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_posture: Option<String>,
    #[serde(default)]
    pub carried_forward_items: Vec<String>,
    #[serde(default)]
    pub artifact_provenance_links: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closure_status: Option<String>,
    #[serde(default)]
    pub closure_findings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryCredibilityState {
    Credible,
    Stale,
    Contradicted,
    Insufficient,
}

impl MemoryCredibilityState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credible => "credible",
            Self::Stale => "stale",
            Self::Contradicted => "contradicted",
            Self::Insufficient => "insufficient",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactedCanonMemory {
    pub headline: String,
    pub credibility: MemoryCredibilityState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode_summary: Option<CanonModeSummary>,
    #[serde(default)]
    pub possible_actions: Vec<CanonPossibleActionSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_next_action: Option<CanonRecommendedActionSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_summary: Option<CanonEvidenceInspectSummary>,
}

impl CompactedCanonMemory {
    pub fn summary_text(&self) -> String {
        format!("{} [{}]", self.headline, self.credibility.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonContextSnapshot {
    pub summary: String,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_snapshot: Option<CanonCapabilitySnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compact_memory: Option<CompactedCanonMemory>,
}

pub fn classify_packet_readiness(
    workspace: &Path,
    expected_document_refs: &[String],
    document_refs: &[String],
    missing_sections: &[String],
    runtime_readiness: PacketReadiness,
) -> PacketReadiness {
    if matches!(runtime_readiness, PacketReadiness::Pending | PacketReadiness::Rejected) {
        return runtime_readiness;
    }

    if expected_document_refs.is_empty() {
        return PacketReadiness::Incomplete;
    }

    let available_documents = document_refs.iter().collect::<BTreeSet<_>>();
    let missing_expected = expected_document_refs
        .iter()
        .filter(|document_ref| !available_documents.contains(document_ref))
        .count();
    let missing_sections = derived_packet_missing_sections(
        workspace,
        expected_document_refs,
        document_refs,
        missing_sections,
    );
    let authored_body_failures = expected_document_refs
        .iter()
        .filter(|document_ref| {
            available_documents.contains(document_ref)
                && !document_has_authored_body(workspace, document_ref)
        })
        .count();

    if missing_expected == 0 && missing_sections.is_empty() && authored_body_failures == 0 {
        return PacketReadiness::Reusable;
    }

    if expected_document_refs.len() == authored_body_failures {
        return PacketReadiness::Rejected;
    }

    PacketReadiness::Incomplete
}

pub fn derived_packet_missing_sections(
    workspace: &Path,
    expected_document_refs: &[String],
    document_refs: &[String],
    missing_sections: &[String],
) -> Vec<String> {
    let available_documents = document_refs.iter().collect::<BTreeSet<_>>();
    let mut derived = missing_sections.iter().cloned().collect::<BTreeSet<_>>();

    let has_missing_authored_body = expected_document_refs.iter().any(|document_ref| {
        available_documents.contains(document_ref)
            && !document_has_authored_body(workspace, document_ref)
    });

    if has_missing_authored_body {
        derived.insert("substantive_body".to_string());
    }

    derived.into_iter().collect()
}

fn document_has_authored_body(workspace: &Path, document_ref: &str) -> bool {
    let path = resolve_document_path(workspace, document_ref);
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };

    contents.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed == "---"
            || trimmed.starts_with('#')
            || trimmed.starts_with("<!--")
            || trimmed.starts_with("-->")
            || trimmed.starts_with("```")
        {
            return false;
        }

        let lower = trimmed.to_ascii_lowercase();
        !matches!(
            lower.as_str(),
            "todo" | "tbd" | "n/a" | "[todo]" | "[tbd]" | "missing-authored-body"
        ) && !lower.contains("missing-authored-body")
            && !lower.contains("todo:")
            && !lower.contains("tbd:")
    })
}

fn resolve_document_path(workspace: &Path, document_ref: &str) -> PathBuf {
    let path = Path::new(document_ref);
    if path.is_absolute() { path.to_path_buf() } else { workspace.join(path) }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedStagePacket {
    pub packet_ref: String,
    pub runtime: GovernanceRuntimeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_mode: Option<CanonMode>,
    #[serde(default)]
    pub expected_document_refs: Vec<String>,
    #[serde(default)]
    pub document_refs: Vec<String>,
    pub readiness: PacketReadiness,
    #[serde(default)]
    pub missing_sections: Vec<String>,
    pub headline: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketReuseBinding {
    pub upstream_stage_key: String,
    pub downstream_stage_key: String,
    pub packet_ref: String,
    pub binding_reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutopilotAction {
    SelectMode,
    RetryStageWithNarrowedContext,
    EscalateVerification,
    EscalatePrReview,
    AwaitApproval,
    BlockStage,
}

pub const fn autopilot_action_text(action: AutopilotAction) -> &'static str {
    match action {
        AutopilotAction::SelectMode => "select_mode",
        AutopilotAction::RetryStageWithNarrowedContext => "retry_stage_with_narrowed_context",
        AutopilotAction::EscalateVerification => "escalate_verification",
        AutopilotAction::EscalatePrReview => "escalate_pr_review",
        AutopilotAction::AwaitApproval => "await_approval",
        AutopilotAction::BlockStage => "block_stage",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutopilotDecisionRecord {
    pub decision_id: String,
    pub stage_key: String,
    #[serde(default)]
    pub candidate_actions: Vec<AutopilotAction>,
    #[serde(default)]
    pub candidate_modes: Vec<CanonMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_action: Option<AutopilotAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_mode: Option<CanonMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_target_stage_key: Option<String>,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedStageRecord {
    pub stage_key: String,
    pub runtime: GovernanceRuntimeKind,
    pub lifecycle_state: GovernanceLifecycleState,
    pub required: bool,
    pub autopilot_enabled: bool,
    pub approval_state: ApprovalState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_run_ref: Option<String>,
    pub governance_attempt_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_governance_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedDocumentRef {
    pub stage_key: String,
    pub canon_mode: CanonMode,
    pub packet_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_path: Option<String>,
    pub readiness: PacketReadiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedSessionLifecycle {
    pub governance_runtime: GovernanceRuntimeKind,
    #[serde(default)]
    pub explicit_opt_out: bool,
    #[serde(default)]
    pub mode_selection_preference: CanonModeSelectionPreference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_mode: Option<CanonMode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_mode_sequence: Vec<CanonMode>,
    #[serde(default)]
    pub current_stage_index: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_records: Vec<GovernedStageRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accumulated_context: Vec<GovernedDocumentRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GovernanceProfileError {
    #[error("governance profile requires a Canon command when Canon runtime is selected")]
    MissingCanonCommand,
    #[error("governance stage policy '{0}' is duplicated")]
    DuplicateStagePolicy(String),
    #[error("governance flow '{0}' is not a supported built-in flow")]
    UnsupportedFlow(String),
    #[error("governance stage '{flow_name}:{stage_id}' is not a supported built-in stage")]
    UnsupportedStage { flow_name: String, stage_id: String },
    #[error("governance stage '{0}' is not supported by the first-slice Canon mapping")]
    UnsupportedCanonStage(String),
    #[error("governance stage '{0}' cannot be required unless it is enabled")]
    RequiredPolicyNotEnabled(String),
    #[error("governance stage '{0}' cannot enable autopilot unless it is enabled")]
    AutopilotPolicyNotEnabled(String),
    #[error("governance stage '{0}' requires Canon configuration")]
    MissingCanonConfig(String),
    #[error("governance stage '{stage_key}' is missing Canon field '{field}'")]
    MissingCanonField { stage_key: String, field: &'static str },
    #[error("governance stage '{stage_key}' cannot bind Canon mode '{mode:?}'")]
    CanonModeNotAllowed { stage_key: String, mode: CanonMode },
    #[error(
        "governance stage '{stage_key}' cannot bind system_context '{system_context:?}' for Canon mode '{mode:?}'"
    )]
    InvalidSystemContextForMode {
        stage_key: String,
        mode: CanonMode,
        system_context: SystemContextBinding,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_modes_and_catalog_cover_project_scale_stage_set() {
        assert_eq!("requirements".parse::<CanonMode>().unwrap(), CanonMode::Requirements);
        assert_eq!("pr-review".parse::<CanonMode>().unwrap(), CanonMode::PrReview);

        assert_eq!(CANONICAL_MODES.len(), 16);
        assert!(CANONICAL_MODES.contains(&CanonMode::Requirements));
        assert!(CANONICAL_MODES.contains(&CanonMode::PrReview));

        let catalog = governed_stage_catalog();
        assert_eq!(catalog.len(), 16);
        assert!(catalog.iter().any(|entry| {
            entry.mode == CanonMode::Review
                && entry.category == GovernedStageCategory::Review
                && entry.voting_may_be_required
        }));
        assert!(catalog.iter().any(|entry| {
            entry.mode == CanonMode::Verification
                && entry.category == GovernedStageCategory::Verification
        }));
        assert!(catalog.iter().any(|entry| {
            entry.mode == CanonMode::Incident
                && entry.category == GovernedStageCategory::Operational
        }));
        assert!(catalog.iter().any(|entry| {
            entry.mode == CanonMode::SecurityAssessment
                && entry.category == GovernedStageCategory::Assessment
        }));
        assert!(catalog.iter().any(|entry| {
            entry.mode == CanonMode::Refactor
                && !entry.voting_may_be_required
                && entry.recommendation_only
        }));
    }

    #[test]
    fn capability_snapshot_validation_and_summary_cover_success_and_failure_paths() {
        let snapshot = CanonCapabilitySnapshot {
            canon_version: "0.48.0".to_string(),
            supported_schema_versions: vec!["2026-02-01".to_string()],
            operations: vec!["capabilities".to_string(), "start".to_string()],
            supported_modes: vec![CanonMode::Change, CanonMode::PrReview],
            status_values: Vec::new(),
            approval_state_values: Vec::new(),
            packet_readiness_values: Vec::new(),
            compatibility_notes: Vec::new(),
        };

        assert_eq!(snapshot.summary_text(), "Canon 0.48.0 capabilities available");
        assert_eq!(
            CanonCapabilitySnapshot {
                canon_version: String::new(),
                supported_schema_versions: Vec::new(),
                operations: Vec::new(),
                supported_modes: Vec::new(),
                status_values: Vec::new(),
                approval_state_values: Vec::new(),
                packet_readiness_values: Vec::new(),
                compatibility_notes: Vec::new(),
            }
            .summary_text(),
            "Canon capabilities available"
        );

        assert!(validate_canon_capabilities_for_mode(&snapshot, CanonMode::PrReview).is_ok());

        let unsupported =
            validate_canon_capabilities_for_mode(&snapshot, CanonMode::Migration).unwrap_err();
        assert!(unsupported.contains("unsupported"), "{unsupported}");

        let missing_operation = validate_canon_capabilities_for_mode(
            &CanonCapabilitySnapshot { operations: vec!["start".to_string()], ..snapshot.clone() },
            CanonMode::Change,
        )
        .unwrap_err();
        assert!(missing_operation.contains("missing `capabilities` operation"));
    }
}
