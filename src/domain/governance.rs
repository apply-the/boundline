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

    /// Map a Canon mode to its minimum authority-zone floor per S3 §8.1.
    pub const fn stage_authority_floor(self) -> CanonAuthorityZone {
        match self {
            Self::Discovery | Self::Requirements => CanonAuthorityZone::Green,
            Self::SystemShaping
            | Self::Architecture
            | Self::Backlog
            | Self::Change
            | Self::Implementation
            | Self::Refactor
            | Self::Review
            | Self::Verification
            | Self::PrReview => CanonAuthorityZone::Yellow,
            Self::Incident
            | Self::Migration
            | Self::SecurityAssessment
            | Self::SystemAssessment
            | Self::SupplyChainAnalysis => CanonAuthorityZone::Red,
        }
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
        {
            let Some(system_context) = system_context else {
                return Err(GovernanceProfileError::MissingCanonField {
                    stage_key: self.stage_key(),
                    field: "system_context",
                });
            };

            if system_context != SystemContextBinding::Existing {
                return Err(GovernanceProfileError::InvalidSystemContextForMode {
                    stage_key: self.stage_key(),
                    mode,
                    system_context,
                });
            }
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

pub const AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE: &str = "authority-governance-v1";
const AUTHORITY_PROVENANCE_UNAVAILABLE: &str = "unavailable";

/// Bounded council profile vocabulary defined by S3 §20.
///
/// The variants describe the minimum review-council shape required before a
/// governed stage may advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouncilProfile {
    /// No review council is required for this authority posture.
    None,
    /// One qualified reviewer is sufficient.
    LightSingle,
    /// Two distinct reviewers are required.
    YellowPair,
    /// Five reviewers plus the stronger red-zone posture are required.
    RedFive,
    /// Automation must stop and hand off to a human-controlled path.
    RestrictedManual,
}

impl CouncilProfile {
    /// Returns the stable serialized identifier used in session state and traces.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LightSingle => "light_single",
            Self::YellowPair => "yellow_pair",
            Self::RedFive => "red_five",
            Self::RestrictedManual => "restricted_manual",
        }
    }
}

impl std::fmt::Display for CouncilProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stop-semantics vocabulary defined by S3 §15.
///
/// These values describe what the operator is allowed to do next after the
/// authority matrix and any structural gates are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopSemantics {
    /// Delivery may continue without an extra governance step.
    Proceed,
    /// Delivery may continue, but the result should remain advisory.
    ProceedWithAdvisory,
    /// Delivery may continue, but the user should see an explicit warning.
    ProceedWithWarning,
    /// Delivery may continue in a degraded posture because stronger controls are unavailable.
    DegradedProceed,
    /// A review council must complete before the stage may proceed.
    CouncilRequired,
    /// Findings must be adjudicated before the stage may proceed.
    AdjudicationRequired,
    /// A human approval gate must be satisfied before the stage may proceed.
    HumanGateRequired,
    /// Automation must stop.
    HardStop,
}

impl StopSemantics {
    /// Returns the stable serialized identifier used in session state and traces.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::ProceedWithAdvisory => "proceed_with_advisory",
            Self::ProceedWithWarning => "proceed_with_warning",
            Self::DegradedProceed => "degraded_proceed",
            Self::CouncilRequired => "council_required",
            Self::AdjudicationRequired => "adjudication_required",
            Self::HumanGateRequired => "human_gate_required",
            Self::HardStop => "hard_stop",
        }
    }

    /// Reports whether the semantic requires delivery to stop immediately.
    pub const fn is_hard_stop(self) -> bool {
        matches!(self, Self::HardStop)
    }
}

impl std::fmt::Display for StopSemantics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanonAuthorityZone {
    /// Low-governance zone used for bounded, low-impact work.
    Green,
    /// Medium-governance zone used for bounded but review-relevant work.
    Yellow,
    /// High-governance zone used for system-wide or operationally sensitive work.
    Red,
    /// Explicit manual-only zone that prevents automated continuation.
    Restricted,
}

impl CanonAuthorityZone {
    /// Returns the stable serialized identifier used in Canon packets and projections.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
            Self::Restricted => "restricted",
        }
    }

    /// Numeric floor used by `effective_authority_floor` to compute the max.
    const fn floor_rank(self) -> u8 {
        match self {
            Self::Green => 0,
            Self::Yellow => 1,
            Self::Red => 2,
            Self::Restricted => 3,
        }
    }

    /// Returns the higher-governance zone of two candidates.
    const fn max(self, other: Self) -> Self {
        if other.floor_rank() > self.floor_rank() { other } else { self }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonChangeClass {
    /// Small, low-risk change bounded to a narrow delivery slice.
    LowImpact,
    /// Broader but still bounded change that needs extra review care.
    BoundedImpact,
    /// System-wide change that can affect multiple flows or contracts.
    SystemicImpact,
    /// Operationally critical change that should inherit the strongest floor.
    CriticalOperations,
}

impl CanonChangeClass {
    /// Returns the stable serialized identifier used in Canon packets and projections.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LowImpact => "low-impact",
            Self::BoundedImpact => "bounded-impact",
            Self::SystemicImpact => "systemic-impact",
            Self::CriticalOperations => "critical-operations",
        }
    }

    /// Map change class to its minimum authority-zone floor per S3 §8.
    pub const fn authority_floor(self) -> CanonAuthorityZone {
        match self {
            Self::LowImpact => CanonAuthorityZone::Green,
            Self::BoundedImpact => CanonAuthorityZone::Yellow,
            Self::SystemicImpact => CanonAuthorityZone::Yellow,
            Self::CriticalOperations => CanonAuthorityZone::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonIntendedPersona {
    /// Product-facing strategy author.
    ProductStrategist,
    /// Architecture and system-shaping author.
    SystemArchitect,
    /// Delivery and implementation author.
    DeliveryEngineer,
    /// Verification-focused reviewer or owner.
    VerificationLead,
    /// Operations or risk-governance owner.
    OperationsGovernor,
    /// Domain boundary and stewardship owner.
    DomainSteward,
}

impl CanonIntendedPersona {
    /// Returns the stable serialized identifier used in Canon packets and projections.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProductStrategist => "product-strategist",
            Self::SystemArchitect => "system-architect",
            Self::DeliveryEngineer => "delivery-engineer",
            Self::VerificationLead => "verification-lead",
            Self::OperationsGovernor => "operations-governor",
            Self::DomainSteward => "domain-steward",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonRiskClass {
    /// Low-risk work with limited blast radius.
    LowImpact,
    /// Bounded risk that still needs stronger review posture than green-only work.
    BoundedImpact,
    /// High-risk work with systemic consequences if it fails.
    SystemicImpact,
}

impl CanonRiskClass {
    /// Returns the stable serialized identifier used in Canon packets and projections.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LowImpact => "low-impact",
            Self::BoundedImpact => "bounded-impact",
            Self::SystemicImpact => "systemic-impact",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CanonStageRoleHintKind {
    ReviewerCapability,
    ReviewPosture,
    HumanGate,
}

impl CanonStageRoleHintKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReviewerCapability => "reviewer-capability",
            Self::ReviewPosture => "review-posture",
            Self::HumanGate => "human-gate",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonStageRoleHint {
    pub hint_kind: CanonStageRoleHintKind,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonAuthorityGovernanceV1Envelope {
    pub contract_line: String,
    pub authority_zone: CanonAuthorityZone,
    pub change_class: CanonChangeClass,
    pub intended_persona: CanonIntendedPersona,
    pub approval_state: ApprovalState,
    pub packet_readiness: PacketReadiness,
    pub risk: CanonRiskClass,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub persona_anti_behaviors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_artifact: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_order: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promotion_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stage_role_hints: Vec<CanonStageRoleHint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityControlResolution {
    /// Stable control-class label used by operator-facing projections.
    pub effective_control_class: String,
    /// Minimum council profile required for the evaluated posture.
    pub council_profile: CouncilProfile,
    /// Stop semantic that determines whether delivery may continue.
    pub stop_semantics: StopSemantics,
}

impl CanonAuthorityGovernanceV1Envelope {
    /// Reports whether the packet advertises the currently supported authority contract line.
    pub fn is_supported_contract_line(&self) -> bool {
        self.contract_line == AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE
    }

    /// Resolves the effective control class, council profile, and stop semantics.
    ///
    /// When `stage` is available the resolution incorporates the stage
    /// authority floor per S3 §8.1 and maps to the S3 §21 V1 matrix.
    /// When `stage` is `None` the resolution uses only the Canon authority
    /// zone and change class.
    pub fn control_resolution_for_stage(
        &self,
        stage: Option<CanonMode>,
    ) -> AuthorityControlResolution {
        // Structural gates override the matrix.
        if matches!(self.approval_state, ApprovalState::Requested)
            || matches!(self.authority_zone, CanonAuthorityZone::Restricted)
        {
            return AuthorityControlResolution {
                effective_control_class: "restricted_gate".to_string(),
                council_profile: CouncilProfile::RestrictedManual,
                stop_semantics: StopSemantics::HardStop,
            };
        }

        if matches!(self.packet_readiness, PacketReadiness::Incomplete | PacketReadiness::Rejected)
            || matches!(self.approval_state, ApprovalState::Rejected | ApprovalState::Expired)
        {
            return AuthorityControlResolution {
                effective_control_class: "blocked_contract".to_string(),
                council_profile: CouncilProfile::RestrictedManual,
                stop_semantics: StopSemantics::HardStop,
            };
        }

        // The first-slice roadmap matrix defines a few stage-specific outcomes
        // that are more specific than the generic floor calculation.
        if matches!(self.authority_zone, CanonAuthorityZone::Green)
            && matches!(self.change_class, CanonChangeClass::LowImpact)
        {
            if matches!(stage, Some(CanonMode::Discovery | CanonMode::Requirements)) {
                return AuthorityControlResolution {
                    effective_control_class: "bounded_delivery".to_string(),
                    council_profile: CouncilProfile::None,
                    stop_semantics: StopSemantics::Proceed,
                };
            }

            if matches!(stage, Some(CanonMode::Implementation | CanonMode::Refactor)) {
                return AuthorityControlResolution {
                    effective_control_class: "bounded_delivery".to_string(),
                    council_profile: CouncilProfile::LightSingle,
                    stop_semantics: StopSemantics::Proceed,
                };
            }
        }

        // Compute effective floor: max(authority_zone, change_class_floor, stage_floor).
        let change_floor = self.change_class.authority_floor();
        let stage_floor = stage.map_or(CanonAuthorityZone::Green, |m| m.stage_authority_floor());
        let effective_zone = self.authority_zone.max(change_floor).max(stage_floor);

        match effective_zone {
            CanonAuthorityZone::Restricted => AuthorityControlResolution {
                effective_control_class: "restricted_gate".to_string(),
                council_profile: CouncilProfile::RestrictedManual,
                stop_semantics: StopSemantics::HardStop,
            },
            CanonAuthorityZone::Red => AuthorityControlResolution {
                effective_control_class: "advisory_only".to_string(),
                council_profile: CouncilProfile::RedFive,
                stop_semantics: StopSemantics::HumanGateRequired,
            },
            CanonAuthorityZone::Yellow => {
                // Systemic or critical work inside the yellow band escalates to a
                // stronger council while still remaining below the red human gate.
                if matches!(
                    self.change_class,
                    CanonChangeClass::SystemicImpact | CanonChangeClass::CriticalOperations
                ) {
                    AuthorityControlResolution {
                        effective_control_class: "council_review".to_string(),
                        council_profile: CouncilProfile::RedFive,
                        stop_semantics: StopSemantics::AdjudicationRequired,
                    }
                } else {
                    AuthorityControlResolution {
                        effective_control_class: "council_review".to_string(),
                        council_profile: CouncilProfile::YellowPair,
                        stop_semantics: StopSemantics::CouncilRequired,
                    }
                }
            }
            CanonAuthorityZone::Green => {
                // Green zone: bounded-impact still needs a pair.
                if matches!(self.change_class, CanonChangeClass::BoundedImpact) {
                    AuthorityControlResolution {
                        effective_control_class: "bounded_delivery".to_string(),
                        council_profile: CouncilProfile::YellowPair,
                        stop_semantics: StopSemantics::CouncilRequired,
                    }
                } else if stage.is_some() {
                    // Stage present and green-floor: light review.
                    AuthorityControlResolution {
                        effective_control_class: "bounded_delivery".to_string(),
                        council_profile: CouncilProfile::LightSingle,
                        stop_semantics: StopSemantics::Proceed,
                    }
                } else {
                    // No stage context: no council.
                    AuthorityControlResolution {
                        effective_control_class: "bounded_delivery".to_string(),
                        council_profile: CouncilProfile::None,
                        stop_semantics: StopSemantics::Proceed,
                    }
                }
            }
        }
    }

    /// Resolves authority controls without a stage-specific floor.
    pub fn control_resolution(&self) -> AuthorityControlResolution {
        self.control_resolution_for_stage(None)
    }

    /// Returns whether the current authority posture requires an immediate hard stop.
    pub fn requires_hard_stop(&self) -> bool {
        self.control_resolution().stop_semantics.is_hard_stop()
    }

    /// Builds the operator-facing reason string for a hard stop, when one exists.
    pub fn hard_stop_reason(&self) -> Option<String> {
        self.requires_hard_stop().then(|| {
            let resolution = self.control_resolution();
            format!(
                "Canon authority {} with {} requires {}",
                self.authority_zone.as_str(),
                self.change_class.as_str(),
                resolution.stop_semantics.as_str()
            )
        })
    }

    /// Renders a compact projection of the authority packet for session and trace views.
    pub fn projection_lines(&self) -> Vec<String> {
        let resolution = self.control_resolution();
        let stage_role_hints = if self.stage_role_hints.is_empty() {
            AUTHORITY_PROVENANCE_UNAVAILABLE.to_string()
        } else {
            self.stage_role_hints
                .iter()
                .map(|hint| format!("{}:{}", hint.hint_kind.as_str(), hint.value))
                .collect::<Vec<_>>()
                .join(", ")
        };

        vec![
            format!("authority_contract_line: {}", self.contract_line),
            format!("authority_zone: {}", self.authority_zone.as_str()),
            format!("authority_change_class: {}", self.change_class.as_str()),
            format!("authority_intended_persona: {}", self.intended_persona.as_str()),
            format!("authority_risk: {}", self.risk.as_str()),
            format!("authority_control_class: {}", resolution.effective_control_class),
            format!("authority_council_profile: {}", resolution.council_profile.as_str()),
            format!("authority_stop_semantics: {}", resolution.stop_semantics.as_str()),
            format!(
                "authority_primary_artifact: {}",
                self.primary_artifact
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(AUTHORITY_PROVENANCE_UNAVAILABLE)
            ),
            format!(
                "authority_artifact_order: {}",
                if self.artifact_order.is_empty() {
                    AUTHORITY_PROVENANCE_UNAVAILABLE.to_string()
                } else {
                    self.artifact_order.join(", ")
                }
            ),
            format!(
                "authority_promotion_refs: {}",
                if self.promotion_refs.is_empty() {
                    AUTHORITY_PROVENANCE_UNAVAILABLE.to_string()
                } else {
                    self.promotion_refs.join(", ")
                }
            ),
            format!(
                "authority_persona_anti_behaviors: {}",
                if self.persona_anti_behaviors.is_empty() {
                    AUTHORITY_PROVENANCE_UNAVAILABLE.to_string()
                } else {
                    self.persona_anti_behaviors.join(", ")
                }
            ),
            format!("authority_stage_role_hints: {stage_role_hints}"),
        ]
    }
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

/// Compact Canon memory facts that Boundline can carry through planning,
/// status, and inspection surfaces without depending on the original packet.
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_provenance_lines: Vec<String>,
}

impl CompactedCanonMemory {
    /// Render the primary memory summary used by status and trace views.
    pub fn summary_text(&self) -> String {
        format!("{} [{}]", self.headline, self.credibility.as_str())
    }

    /// Map Canon-memory credibility onto the consumer-side compatibility state.
    pub const fn compatibility_state(&self) -> &'static str {
        match self.credibility {
            MemoryCredibilityState::Credible => "compatible",
            MemoryCredibilityState::Stale | MemoryCredibilityState::Contradicted => "warning",
            MemoryCredibilityState::Insufficient => "unsupported",
        }
    }

    /// Render the recommended Canon follow-up as a single CLI-safe string.
    pub fn next_action_text(&self) -> Option<String> {
        self.recommended_next_action
            .as_ref()
            .map(|action| format!("{}: {}", action.action, action.rationale))
    }

    /// Emit stable provenance lines for session-native status and inspect views.
    pub fn provenance_lines(&self) -> Vec<String> {
        let mut lines =
            vec![format!("canon_memory: {} [{}]", self.headline, self.credibility.as_str())];
        lines.push(format!("canon_memory_compatibility: {}", self.compatibility_state()));
        if let Some(run_ref) = self.run_ref.as_ref() {
            lines.push(format!("canon_memory_run_ref: {run_ref}"));
        }
        if let Some(packet_ref) = self.packet_ref.as_ref() {
            lines.push(format!("canon_memory_packet: {packet_ref}"));
        }
        if let Some(reason_code) = self.reason_code.as_ref() {
            lines.push(format!("canon_memory_reason: {reason_code}"));
        }
        lines.extend(self.authority_provenance_lines.clone());
        if let Some(next_action) = self.next_action_text() {
            lines.push(format!("canon_memory_next_action: {next_action}"));
        }
        if let Some(mode_summary) = self.mode_summary.as_ref() {
            lines.push(format!("canon_memory_mode: {}", mode_summary.summary_text()));
        }
        if let Some(evidence_summary) = self.evidence_summary.as_ref() {
            for item in &evidence_summary.carried_forward_items {
                lines.push(format!("canon_evidence_contribution: {item}"));
            }
            for link in &evidence_summary.artifact_provenance_links {
                lines.push(format!("canon_provenance: {link}"));
            }
        }
        lines
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_governance: Option<CanonAuthorityGovernanceV1Envelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketReuseBinding {
    pub upstream_stage_key: String,
    pub downstream_stage_key: String,
    pub packet_ref: String,
    pub binding_reason: PacketReuseBindingReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketReuseBindingReason {
    SameStageRerun,
    UpstreamStageContext,
}

impl PacketReuseBindingReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameStageRerun => "same_stage_rerun",
            Self::UpstreamStageContext => "upstream_stage_context",
        }
    }
}

impl std::fmt::Display for PacketReuseBindingReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
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
            canon_version: crate::domain::distribution::SUPPORTED_CANON_VERSION.to_string(),
            supported_schema_versions: vec!["2026-02-01".to_string()],
            operations: vec!["capabilities".to_string(), "start".to_string()],
            supported_modes: vec![CanonMode::Change, CanonMode::PrReview],
            status_values: Vec::new(),
            approval_state_values: Vec::new(),
            packet_readiness_values: Vec::new(),
            compatibility_notes: Vec::new(),
        };

        assert_eq!(
            snapshot.summary_text(),
            format!(
                "Canon {} capabilities available",
                crate::domain::distribution::SUPPORTED_CANON_VERSION
            )
        );
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

    #[test]
    fn packet_reuse_binding_reason_text_is_stable() {
        assert_eq!(PacketReuseBindingReason::SameStageRerun.as_str(), "same_stage_rerun");
        assert_eq!(
            PacketReuseBindingReason::UpstreamStageContext.as_str(),
            "upstream_stage_context"
        );
        assert_eq!(PacketReuseBindingReason::SameStageRerun.to_string(), "same_stage_rerun");
        assert_eq!(
            PacketReuseBindingReason::UpstreamStageContext.to_string(),
            "upstream_stage_context"
        );
    }

    #[test]
    fn authority_governance_projection_lines_include_resolution_and_unavailable_markers() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Yellow,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::SystemArchitect,
            approval_state: ApprovalState::Granted,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: vec!["unbounded implementation detail".to_string()],
            primary_artifact: Some("01-architecture-summary.md".to_string()),
            artifact_order: vec!["01-architecture-summary.md".to_string()],
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let lines = envelope.projection_lines();

        assert!(lines.contains(&"authority_control_class: council_review".to_string()));
        assert!(lines.contains(&"authority_council_profile: red_five".to_string()));
        assert!(lines.contains(&"authority_promotion_refs: unavailable".to_string()));
        assert!(lines.contains(&"authority_stage_role_hints: unavailable".to_string()));
    }

    #[test]
    fn authority_governance_requested_approval_resolves_to_restricted_gate() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Restricted,
            change_class: CanonChangeClass::BoundedImpact,
            intended_persona: CanonIntendedPersona::DeliveryEngineer,
            approval_state: ApprovalState::Requested,
            packet_readiness: PacketReadiness::Incomplete,
            risk: CanonRiskClass::BoundedImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution();

        assert_eq!(resolution.effective_control_class, "restricted_gate");
        assert_eq!(resolution.council_profile, CouncilProfile::RestrictedManual);
        assert_eq!(resolution.stop_semantics, StopSemantics::HardStop);
    }

    #[test]
    fn v1_matrix_green_low_impact_discovery_resolves_to_none_proceed() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Green,
            change_class: CanonChangeClass::LowImpact,
            intended_persona: CanonIntendedPersona::DeliveryEngineer,
            approval_state: ApprovalState::NotNeeded,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::LowImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::Discovery));
        assert_eq!(resolution.council_profile, CouncilProfile::None);
        assert_eq!(resolution.stop_semantics, StopSemantics::Proceed);
    }

    #[test]
    fn v1_matrix_green_low_impact_implementation_resolves_to_light_single_proceed() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Green,
            change_class: CanonChangeClass::LowImpact,
            intended_persona: CanonIntendedPersona::DeliveryEngineer,
            approval_state: ApprovalState::NotNeeded,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::LowImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::Implementation));
        assert_eq!(resolution.council_profile, CouncilProfile::LightSingle);
        assert_eq!(resolution.stop_semantics, StopSemantics::Proceed);
    }

    #[test]
    fn v1_matrix_yellow_systemic_architecture_resolves_to_red_five_adjudication() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Yellow,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::SystemArchitect,
            approval_state: ApprovalState::Granted,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::Architecture));
        assert_eq!(resolution.council_profile, CouncilProfile::RedFive);
        assert_eq!(resolution.stop_semantics, StopSemantics::AdjudicationRequired);
    }

    #[test]
    fn v1_matrix_red_zone_migration_resolves_to_red_five_human_gate() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Red,
            change_class: CanonChangeClass::SystemicImpact,
            intended_persona: CanonIntendedPersona::OperationsGovernor,
            approval_state: ApprovalState::Granted,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::Migration));
        assert_eq!(resolution.council_profile, CouncilProfile::RedFive);
        assert_eq!(resolution.stop_semantics, StopSemantics::HumanGateRequired);
    }

    #[test]
    fn v1_matrix_restricted_zone_resolves_to_restricted_manual_hard_stop() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Restricted,
            change_class: CanonChangeClass::CriticalOperations,
            intended_persona: CanonIntendedPersona::OperationsGovernor,
            approval_state: ApprovalState::Requested,
            packet_readiness: PacketReadiness::Incomplete,
            risk: CanonRiskClass::SystemicImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::Incident));
        assert_eq!(resolution.council_profile, CouncilProfile::RestrictedManual);
        assert_eq!(resolution.stop_semantics, StopSemantics::HardStop);
    }

    #[test]
    fn v1_matrix_stage_floor_escalates_green_zone_to_red() {
        let envelope = CanonAuthorityGovernanceV1Envelope {
            contract_line: AUTHORITY_GOVERNANCE_V1_CONTRACT_LINE.to_string(),
            authority_zone: CanonAuthorityZone::Green,
            change_class: CanonChangeClass::LowImpact,
            intended_persona: CanonIntendedPersona::DeliveryEngineer,
            approval_state: ApprovalState::NotNeeded,
            packet_readiness: PacketReadiness::Reusable,
            risk: CanonRiskClass::LowImpact,
            persona_anti_behaviors: Vec::new(),
            primary_artifact: None,
            artifact_order: Vec::new(),
            promotion_refs: Vec::new(),
            stage_role_hints: Vec::new(),
        };

        // SecurityAssessment has a red stage floor: green zone escalates.
        let resolution = envelope.control_resolution_for_stage(Some(CanonMode::SecurityAssessment));
        assert_eq!(resolution.council_profile, CouncilProfile::RedFive);
        assert_eq!(resolution.stop_semantics, StopSemantics::HumanGateRequired);
    }

    #[test]
    fn stage_authority_floor_maps_modes_to_expected_zones() {
        assert_eq!(CanonMode::Discovery.stage_authority_floor(), CanonAuthorityZone::Green);
        assert_eq!(CanonMode::Requirements.stage_authority_floor(), CanonAuthorityZone::Green);
        assert_eq!(CanonMode::Implementation.stage_authority_floor(), CanonAuthorityZone::Yellow);
        assert_eq!(CanonMode::Architecture.stage_authority_floor(), CanonAuthorityZone::Yellow);
        assert_eq!(CanonMode::PrReview.stage_authority_floor(), CanonAuthorityZone::Yellow);
        assert_eq!(CanonMode::Incident.stage_authority_floor(), CanonAuthorityZone::Red);
        assert_eq!(CanonMode::Migration.stage_authority_floor(), CanonAuthorityZone::Red);
        assert_eq!(CanonMode::SecurityAssessment.stage_authority_floor(), CanonAuthorityZone::Red);
    }

    #[test]
    fn change_class_authority_floor_maps_to_expected_zones() {
        assert_eq!(CanonChangeClass::LowImpact.authority_floor(), CanonAuthorityZone::Green);
        assert_eq!(CanonChangeClass::BoundedImpact.authority_floor(), CanonAuthorityZone::Yellow);
        assert_eq!(CanonChangeClass::SystemicImpact.authority_floor(), CanonAuthorityZone::Yellow);
        assert_eq!(CanonChangeClass::CriticalOperations.authority_floor(), CanonAuthorityZone::Red);
    }
}
