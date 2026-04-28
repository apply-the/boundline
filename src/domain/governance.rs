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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemContextBinding {
    New,
    Existing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonMode {
    Requirements,
    Architecture,
    Backlog,
    Change,
    Discovery,
    Implementation,
    Verification,
    PrReview,
}

impl CanonMode {
    pub const fn primary_document_name(self) -> &'static str {
        match self {
            Self::Requirements => "requirements.md",
            Self::Architecture => "architecture.md",
            Self::Backlog => "backlog.md",
            Self::Change => "change.md",
            Self::Discovery => "discovery.md",
            Self::Implementation => "implementation.md",
            Self::Verification => "verification.md",
            Self::PrReview => "pr-review.md",
        }
    }

    pub const fn requires_existing_context(self) -> bool {
        matches!(
            self,
            Self::Backlog
                | Self::Change
                | Self::Implementation
                | Self::Verification
                | Self::PrReview
        )
    }

    pub fn expected_document_refs(self, packet_ref: &str) -> Vec<String> {
        vec![format!("{}/{}", packet_ref.trim_end_matches('/'), self.primary_document_name())]
    }
}

const DELIVERY_REQUIREMENTS_MODES: [CanonMode; 1] = [CanonMode::Requirements];
const DELIVERY_ARCHITECTURE_MODES: [CanonMode; 1] = [CanonMode::Architecture];
const DELIVERY_BACKLOG_MODES: [CanonMode; 1] = [CanonMode::Backlog];
const DELIVERY_IMPLEMENTATION_MODES: [CanonMode; 1] = [CanonMode::Implementation];
const CHANGE_UNDERSTAND_MODES: [CanonMode; 1] = [CanonMode::Change];
const CHANGE_IMPLEMENT_MODES: [CanonMode; 1] = [CanonMode::Implementation];
const CHANGE_VERIFY_MODES: [CanonMode; 2] = [CanonMode::Verification, CanonMode::PrReview];
const BUG_FIX_INVESTIGATE_MODES: [CanonMode; 2] = [CanonMode::Discovery, CanonMode::Change];
const BUG_FIX_IMPLEMENT_MODES: [CanonMode; 1] = [CanonMode::Implementation];
const BUG_FIX_VERIFY_MODES: [CanonMode; 2] = [CanonMode::Verification, CanonMode::PrReview];
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
