//! Typed guidance and guardian models persisted across planning, session views,
//! and trace inspection.

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::configuration::RouteSlot;

/// Lifecycle phases where guidance and guardians can participate in a bounded run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityPhase {
    Planning,
    Architecture,
    Implementation,
    Testing,
    Verification,
    Review,
}

impl CapabilityPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::Architecture => "architecture",
            Self::Implementation => "implementation",
            Self::Testing => "testing",
            Self::Verification => "verification",
            Self::Review => "review",
        }
    }
}

impl fmt::Display for CapabilityPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Source family used to resolve precedence when multiple capability definitions
/// compete for the same logical identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidanceAuthoritySource {
    WorkspaceOverride,
    CanonGoverned,
    SharedPack,
    BuiltIn,
}

impl GuidanceAuthoritySource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceOverride => "workspace_override",
            Self::CanonGoverned => "canon_governed",
            Self::SharedPack => "shared_pack",
            Self::BuiltIn => "built_in",
        }
    }

    pub const fn precedence_rank(self) -> u8 {
        match self {
            Self::WorkspaceOverride => 0,
            Self::CanonGoverned => 1,
            Self::SharedPack => 2,
            Self::BuiltIn => 3,
        }
    }
}

impl fmt::Display for GuidanceAuthoritySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Priority hint used after authority precedence when ranking guidance candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuidancePriority {
    Low,
    Medium,
    High,
}

impl GuidancePriority {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl fmt::Display for GuidancePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Execution style for a guardian capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianKind {
    Deterministic,
    Hybrid,
    Llm,
}

impl GuardianKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Hybrid => "hybrid",
            Self::Llm => "llm",
        }
    }

    pub const fn execution_rank(self) -> u8 {
        match self {
            Self::Deterministic => 0,
            Self::Hybrid => 1,
            Self::Llm => 2,
        }
    }
}

impl fmt::Display for GuardianKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Severity emitted by a guardian when it records a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianDisposition {
    Advise,
    Warn,
    Concern,
    Error,
    Block,
}

impl GuardianDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Advise => "advise",
            Self::Warn => "warn",
            Self::Concern => "concern",
            Self::Error => "error",
            Self::Block => "block",
        }
    }

    pub const fn is_blocking(self) -> bool {
        matches!(self, Self::Error | Self::Block)
    }
}

impl fmt::Display for GuardianDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Confidence carried by one guardian finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingConfidence {
    Low,
    Medium,
    High,
}

impl FindingConfidence {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

impl fmt::Display for FindingConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Persisted terminal state for one guardian execution attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardianExecutionState {
    Completed,
    Skipped,
    Degraded,
    Failed,
}

impl GuardianExecutionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Skipped => "skipped",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for GuardianExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Guidance content selected for a lifecycle phase after precedence and
/// runtime-evidence ranking are applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidanceCapability {
    pub capability_id: String,
    pub title: String,
    pub applies_to: Vec<CapabilityPhase>,
    pub roles: Vec<String>,
    pub content_ref: String,
    pub priority: GuidancePriority,
    pub authority_source: GuidanceAuthoritySource,
    pub source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
}

impl GuidanceCapability {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.capability_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingCapabilityId);
        }
        if self.title.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingCapabilityTitle {
                capability_id: self.capability_id.clone(),
            });
        }
        if self.applies_to.is_empty() {
            return Err(GuidanceCapabilityError::MissingCapabilityPhase {
                capability_id: self.capability_id.clone(),
            });
        }
        if self.content_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingGuidanceContentRef {
                capability_id: self.capability_id.clone(),
            });
        }
        if self.source_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingSourceRef {
                identifier: self.capability_id.clone(),
            });
        }
        Ok(())
    }
}

/// Guardian definition selected for execution in a lifecycle phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianCapability {
    pub guardian_id: String,
    pub title: String,
    pub kind: GuardianKind,
    pub applies_to: Vec<CapabilityPhase>,
    pub rules: Vec<String>,
    pub severity_floor: GuardianDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instruction_ref: Option<String>,
    pub authority_source: GuidanceAuthoritySource,
    pub source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_id: Option<String>,
}

impl GuardianCapability {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.guardian_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingGuardianId);
        }
        if self.title.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingGuardianTitle {
                guardian_id: self.guardian_id.clone(),
            });
        }
        if self.applies_to.is_empty() {
            return Err(GuidanceCapabilityError::MissingGuardianPhase {
                guardian_id: self.guardian_id.clone(),
            });
        }
        if self.rules.is_empty() || self.rules.iter().all(|rule| rule.trim().is_empty()) {
            return Err(GuidanceCapabilityError::MissingGuardianRules {
                guardian_id: self.guardian_id.clone(),
            });
        }
        if self.source_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingSourceRef {
                identifier: self.guardian_id.clone(),
            });
        }

        match self.kind {
            GuardianKind::Deterministic => {
                if self.command_ref.as_deref().map(str::trim).unwrap_or_default().is_empty() {
                    return Err(GuidanceCapabilityError::MissingGuardianCommand {
                        guardian_id: self.guardian_id.clone(),
                    });
                }
            }
            GuardianKind::Hybrid => {
                if self.command_ref.as_deref().map(str::trim).unwrap_or_default().is_empty() {
                    return Err(GuidanceCapabilityError::MissingGuardianCommand {
                        guardian_id: self.guardian_id.clone(),
                    });
                }
                if self.instruction_ref.as_deref().map(str::trim).unwrap_or_default().is_empty() {
                    return Err(GuidanceCapabilityError::MissingGuardianInstruction {
                        guardian_id: self.guardian_id.clone(),
                    });
                }
            }
            GuardianKind::Llm => {
                if self.instruction_ref.as_deref().map(str::trim).unwrap_or_default().is_empty() {
                    return Err(GuidanceCapabilityError::MissingGuardianInstruction {
                        guardian_id: self.guardian_id.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Source that contributed loaded capability entries to one persisted resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadedCapabilitySource {
    pub source_ref: String,
    pub authority_source: GuidanceAuthoritySource,
}

impl LoadedCapabilitySource {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.source_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingLoadedSourceRef);
        }
        Ok(())
    }
}

/// Source discovered during resolution but skipped with an explicit reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedCapabilitySource {
    pub source_ref: String,
    pub authority_source: GuidanceAuthoritySource,
    pub reason: String,
}

impl SkippedCapabilitySource {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.source_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingSkippedSourceRef);
        }
        if self.reason.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingSkippedSourceReason {
                source_ref: self.source_ref.clone(),
            });
        }
        Ok(())
    }
}

/// Persisted record of which capability sources won for one lifecycle phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityResolutionRecord {
    pub target_ref: String,
    pub phase: CapabilityPhase,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaded_guidance: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaded_guardians: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaded_sources: Vec<LoadedCapabilitySource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_sources: Vec<SkippedCapabilitySource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resolution_notes: Vec<String>,
    pub summary: String,
}

impl CapabilityResolutionRecord {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.target_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingResolutionTarget);
        }
        if self.summary.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingResolutionSummary);
        }
        for source in &self.loaded_sources {
            source.validate()?;
        }
        for source in &self.skipped_sources {
            source.validate()?;
        }
        Ok(())
    }
}

/// One guardian attempt, including the route slot used when semantic review ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianExecutionRecord {
    pub guardian_id: String,
    pub phase: CapabilityPhase,
    pub execution_state: GuardianExecutionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_slot: Option<RouteSlot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<String>,
}

impl GuardianExecutionRecord {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.guardian_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingExecutionGuardianId);
        }
        if matches!(self.execution_state, GuardianExecutionState::Degraded)
            && self.degradation_reason.as_deref().map(str::trim).unwrap_or_default().is_empty()
        {
            return Err(GuidanceCapabilityError::MissingDegradationReason {
                guardian_id: self.guardian_id.clone(),
            });
        }
        Ok(())
    }
}

/// Structured finding emitted by a guardian, or synthesized when guardian
/// execution itself fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardianFinding {
    pub finding_id: String,
    pub guardian_id: String,
    pub rule_id: String,
    pub disposition: GuardianDisposition,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<String>,
    pub confidence: FindingConfidence,
    pub recommended_action: String,
    pub authority_source: GuidanceAuthoritySource,
    pub source_ref: String,
    pub phase: CapabilityPhase,
}

impl GuardianFinding {
    pub fn validate(&self) -> Result<(), GuidanceCapabilityError> {
        if self.finding_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingFindingId);
        }
        if self.guardian_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingFindingGuardianId {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.rule_id.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingFindingRuleId {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.summary.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingFindingSummary {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.recommended_action.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingFindingAction {
                finding_id: self.finding_id.clone(),
            });
        }
        if self.source_ref.trim().is_empty() {
            return Err(GuidanceCapabilityError::MissingSourceRef {
                identifier: self.finding_id.clone(),
            });
        }
        Ok(())
    }
}

/// Flattened read-side projection reused by goal plans, session views, and traces.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuidanceGuardianProjection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_resolution_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaded_guidance_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_guidance_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaded_guardian_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped_guardian_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guardian_timeline: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardian_findings_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guardian_findings: Vec<GuardianFinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guardian_degradations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardian_blocking_outcome: Option<String>,
}

impl GuidanceGuardianProjection {
    /// Returns true when the projection carries no visible guidance or guardian story.
    pub fn is_empty(&self) -> bool {
        self.capability_resolution_summary.is_none()
            && self.loaded_guidance_sources.is_empty()
            && self.skipped_guidance_sources.is_empty()
            && self.loaded_guardian_sources.is_empty()
            && self.skipped_guardian_sources.is_empty()
            && self.guardian_timeline.is_empty()
            && self.guardian_findings_summary.is_none()
            && self.guardian_findings.is_empty()
            && self.guardian_degradations.is_empty()
            && self.guardian_blocking_outcome.is_none()
    }
}

/// Validation failures for guidance and guardian metadata that must remain
/// explicit in serialized state and operator-facing views.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GuidanceCapabilityError {
    #[error("guidance capability id cannot be empty")]
    MissingCapabilityId,
    #[error("guidance capability {capability_id} title cannot be empty")]
    MissingCapabilityTitle { capability_id: String },
    #[error("guidance capability {capability_id} must declare at least one phase")]
    MissingCapabilityPhase { capability_id: String },
    #[error("guidance capability {capability_id} content_ref cannot be empty")]
    MissingGuidanceContentRef { capability_id: String },
    #[error("guardian id cannot be empty")]
    MissingGuardianId,
    #[error("guardian {guardian_id} title cannot be empty")]
    MissingGuardianTitle { guardian_id: String },
    #[error("guardian {guardian_id} must declare at least one phase")]
    MissingGuardianPhase { guardian_id: String },
    #[error("guardian {guardian_id} must declare at least one rule")]
    MissingGuardianRules { guardian_id: String },
    #[error("guardian {guardian_id} requires a deterministic command reference")]
    MissingGuardianCommand { guardian_id: String },
    #[error("guardian {guardian_id} requires an instruction reference")]
    MissingGuardianInstruction { guardian_id: String },
    #[error("identifier {identifier} requires a non-empty source_ref")]
    MissingSourceRef { identifier: String },
    #[error("loaded capability source ref cannot be empty")]
    MissingLoadedSourceRef,
    #[error("skipped capability source ref cannot be empty")]
    MissingSkippedSourceRef,
    #[error("skipped capability source {source_ref} requires a reason")]
    MissingSkippedSourceReason { source_ref: String },
    #[error("capability resolution target cannot be empty")]
    MissingResolutionTarget,
    #[error("capability resolution summary cannot be empty")]
    MissingResolutionSummary,
    #[error("guardian execution record requires a guardian id")]
    MissingExecutionGuardianId,
    #[error("guardian execution record for {guardian_id} requires a degradation reason")]
    MissingDegradationReason { guardian_id: String },
    #[error("guardian finding id cannot be empty")]
    MissingFindingId,
    #[error("guardian finding {finding_id} requires a guardian id")]
    MissingFindingGuardianId { finding_id: String },
    #[error("guardian finding {finding_id} requires a rule id")]
    MissingFindingRuleId { finding_id: String },
    #[error("guardian finding {finding_id} summary cannot be empty")]
    MissingFindingSummary { finding_id: String },
    #[error("guardian finding {finding_id} recommended_action cannot be empty")]
    MissingFindingAction { finding_id: String },
}

#[cfg(test)]
mod tests {
    use crate::domain::configuration::RouteSlot;

    use super::{
        CapabilityPhase, CapabilityResolutionRecord, FindingConfidence, GuardianCapability,
        GuardianDisposition, GuardianExecutionRecord, GuardianExecutionState, GuardianFinding,
        GuardianKind, GuidanceAuthoritySource, GuidanceCapability, GuidanceCapabilityError,
        GuidanceGuardianProjection, GuidancePriority, LoadedCapabilitySource,
        SkippedCapabilitySource,
    };

    #[test]
    fn display_labels_match_serialized_guidance_names() {
        for (phase, label) in [
            (CapabilityPhase::Planning, "planning"),
            (CapabilityPhase::Architecture, "architecture"),
            (CapabilityPhase::Implementation, "implementation"),
            (CapabilityPhase::Testing, "testing"),
            (CapabilityPhase::Verification, "verification"),
            (CapabilityPhase::Review, "review"),
        ] {
            assert_eq!(phase.as_str(), label);
            assert_eq!(phase.to_string(), label);
        }

        for (source, label) in [
            (GuidanceAuthoritySource::WorkspaceOverride, "workspace_override"),
            (GuidanceAuthoritySource::CanonGoverned, "canon_governed"),
            (GuidanceAuthoritySource::SharedPack, "shared_pack"),
            (GuidanceAuthoritySource::BuiltIn, "built_in"),
        ] {
            assert_eq!(source.as_str(), label);
            assert_eq!(source.to_string(), label);
        }

        for (priority, label) in [
            (GuidancePriority::Low, "low"),
            (GuidancePriority::Medium, "medium"),
            (GuidancePriority::High, "high"),
        ] {
            assert_eq!(priority.as_str(), label);
            assert_eq!(priority.to_string(), label);
        }

        for (kind, label) in [
            (GuardianKind::Deterministic, "deterministic"),
            (GuardianKind::Hybrid, "hybrid"),
            (GuardianKind::Llm, "llm"),
        ] {
            assert_eq!(kind.as_str(), label);
            assert_eq!(kind.to_string(), label);
        }

        for (disposition, label, blocking) in [
            (GuardianDisposition::Advise, "advise", false),
            (GuardianDisposition::Warn, "warn", false),
            (GuardianDisposition::Concern, "concern", false),
            (GuardianDisposition::Error, "error", true),
            (GuardianDisposition::Block, "block", true),
        ] {
            assert_eq!(disposition.as_str(), label);
            assert_eq!(disposition.to_string(), label);
            assert_eq!(disposition.is_blocking(), blocking);
        }

        for (confidence, label) in [
            (FindingConfidence::Low, "low"),
            (FindingConfidence::Medium, "medium"),
            (FindingConfidence::High, "high"),
        ] {
            assert_eq!(confidence.as_str(), label);
            assert_eq!(confidence.to_string(), label);
        }

        for (state, label) in [
            (GuardianExecutionState::Completed, "completed"),
            (GuardianExecutionState::Skipped, "skipped"),
            (GuardianExecutionState::Degraded, "degraded"),
            (GuardianExecutionState::Failed, "failed"),
        ] {
            assert_eq!(state.as_str(), label);
            assert_eq!(state.to_string(), label);
        }
    }

    #[test]
    fn guidance_authority_precedence_keeps_workspace_first() {
        assert!(
            GuidanceAuthoritySource::WorkspaceOverride.precedence_rank()
                < GuidanceAuthoritySource::CanonGoverned.precedence_rank()
        );
        assert!(
            GuidanceAuthoritySource::CanonGoverned.precedence_rank()
                < GuidanceAuthoritySource::SharedPack.precedence_rank()
        );
        assert!(
            GuidanceAuthoritySource::SharedPack.precedence_rank()
                < GuidanceAuthoritySource::BuiltIn.precedence_rank()
        );
    }

    #[test]
    fn guardian_kind_execution_rank_orders_deterministic_before_semantic() {
        assert!(
            GuardianKind::Deterministic.execution_rank() < GuardianKind::Hybrid.execution_rank()
        );
        assert!(GuardianKind::Hybrid.execution_rank() < GuardianKind::Llm.execution_rank());
    }

    #[test]
    fn guidance_capability_validate_rejects_missing_content_ref() {
        let capability = GuidanceCapability {
            capability_id: "solid".to_string(),
            title: "SOLID".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: String::new(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guidance/solid.md".to_string(),
            pack_id: Some("engineering-foundations".to_string()),
        };

        assert!(capability.validate().is_err());
    }

    #[test]
    fn guidance_capability_validate_accepts_valid_shape_and_detects_missing_fields() {
        let capability = GuidanceCapability {
            capability_id: "solid".to_string(),
            title: "SOLID".to_string(),
            applies_to: vec![CapabilityPhase::Planning],
            roles: vec!["planner".to_string()],
            content_ref: "assistant/guidance/solid.md".to_string(),
            priority: GuidancePriority::High,
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guidance/solid.md".to_string(),
            pack_id: Some("engineering-foundations".to_string()),
        };

        assert!(capability.validate().is_ok());

        let mut missing_id = capability.clone();
        missing_id.capability_id.clear();
        assert_eq!(missing_id.validate(), Err(GuidanceCapabilityError::MissingCapabilityId));

        let mut missing_title = capability.clone();
        missing_title.title.clear();
        assert_eq!(
            missing_title.validate(),
            Err(GuidanceCapabilityError::MissingCapabilityTitle {
                capability_id: "solid".to_string(),
            })
        );

        let mut missing_phase = capability.clone();
        missing_phase.applies_to.clear();
        assert_eq!(
            missing_phase.validate(),
            Err(GuidanceCapabilityError::MissingCapabilityPhase {
                capability_id: "solid".to_string(),
            })
        );

        let mut missing_source = capability;
        missing_source.source_ref.clear();
        assert_eq!(
            missing_source.validate(),
            Err(GuidanceCapabilityError::MissingSourceRef { identifier: "solid".to_string() })
        );
    }

    #[test]
    fn guardian_capability_validate_enforces_kind_specific_fields() {
        let guardian = GuardianCapability {
            guardian_id: "solid_guardian".to_string(),
            title: "SOLID Guardian".to_string(),
            kind: GuardianKind::Hybrid,
            applies_to: vec![CapabilityPhase::Implementation],
            rules: vec!["srp".to_string()],
            severity_floor: GuardianDisposition::Concern,
            command_ref: Some("scripts/check-solid.sh".to_string()),
            instruction_ref: None,
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guardians/solid.toml".to_string(),
            pack_id: Some("engineering-foundations".to_string()),
        };

        assert!(guardian.validate().is_err());
    }

    #[test]
    fn guardian_capability_validate_covers_all_guardian_kinds() {
        let deterministic = GuardianCapability {
            guardian_id: "solid_guardian".to_string(),
            title: "SOLID Guardian".to_string(),
            kind: GuardianKind::Deterministic,
            applies_to: vec![CapabilityPhase::Implementation],
            rules: vec!["srp".to_string()],
            severity_floor: GuardianDisposition::Concern,
            command_ref: Some("scripts/check-solid.sh".to_string()),
            instruction_ref: None,
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guardians/solid.toml".to_string(),
            pack_id: Some("engineering-foundations".to_string()),
        };

        assert!(deterministic.validate().is_ok());

        let mut missing_rules = deterministic.clone();
        missing_rules.rules = vec![" ".to_string()];
        assert_eq!(
            missing_rules.validate(),
            Err(GuidanceCapabilityError::MissingGuardianRules {
                guardian_id: "solid_guardian".to_string(),
            })
        );

        let mut missing_command = deterministic.clone();
        missing_command.command_ref = None;
        assert_eq!(
            missing_command.validate(),
            Err(GuidanceCapabilityError::MissingGuardianCommand {
                guardian_id: "solid_guardian".to_string(),
            })
        );

        let mut hybrid = deterministic.clone();
        hybrid.kind = GuardianKind::Hybrid;
        hybrid.instruction_ref = Some("assistant/prompts/solid.md".to_string());
        assert!(hybrid.validate().is_ok());

        hybrid.instruction_ref = None;
        assert_eq!(
            hybrid.validate(),
            Err(GuidanceCapabilityError::MissingGuardianInstruction {
                guardian_id: "solid_guardian".to_string(),
            })
        );

        let mut llm = deterministic;
        llm.kind = GuardianKind::Llm;
        llm.command_ref = None;
        llm.instruction_ref = Some("assistant/prompts/solid.md".to_string());
        assert!(llm.validate().is_ok());

        llm.instruction_ref = None;
        assert_eq!(
            llm.validate(),
            Err(GuidanceCapabilityError::MissingGuardianInstruction {
                guardian_id: "solid_guardian".to_string(),
            })
        );
    }

    #[test]
    fn source_and_resolution_records_validate_nested_requirements() {
        let loaded = LoadedCapabilitySource {
            source_ref: "assistant/packs/rust/guidance/solid.md".to_string(),
            authority_source: GuidanceAuthoritySource::SharedPack,
        };
        let skipped = SkippedCapabilitySource {
            source_ref: "assistant/packs/shared/guidance/solid.md".to_string(),
            authority_source: GuidanceAuthoritySource::BuiltIn,
            reason: "shadowed by workspace override".to_string(),
        };
        let resolution = CapabilityResolutionRecord {
            target_ref: "src/lib.rs".to_string(),
            phase: CapabilityPhase::Planning,
            loaded_guidance: vec!["solid".to_string()],
            loaded_guardians: vec!["solid_guardian".to_string()],
            loaded_sources: vec![loaded.clone()],
            skipped_sources: vec![skipped.clone()],
            resolution_notes: vec!["workspace override won".to_string()],
            summary: "loaded workspace guidance and guardian sources".to_string(),
        };

        assert!(loaded.validate().is_ok());
        assert!(skipped.validate().is_ok());
        assert!(resolution.validate().is_ok());

        let mut missing_loaded = loaded;
        missing_loaded.source_ref.clear();
        assert_eq!(missing_loaded.validate(), Err(GuidanceCapabilityError::MissingLoadedSourceRef));

        let mut missing_skip_reason = skipped;
        missing_skip_reason.reason.clear();
        assert_eq!(
            missing_skip_reason.validate(),
            Err(GuidanceCapabilityError::MissingSkippedSourceReason {
                source_ref: "assistant/packs/shared/guidance/solid.md".to_string(),
            })
        );

        let mut missing_target = resolution.clone();
        missing_target.target_ref.clear();
        assert_eq!(
            missing_target.validate(),
            Err(GuidanceCapabilityError::MissingResolutionTarget)
        );

        let mut missing_summary = resolution;
        missing_summary.summary.clear();
        assert_eq!(
            missing_summary.validate(),
            Err(GuidanceCapabilityError::MissingResolutionSummary)
        );
    }

    #[test]
    fn degraded_execution_requires_reason() {
        let record = GuardianExecutionRecord {
            guardian_id: "solid_guardian".to_string(),
            phase: CapabilityPhase::Implementation,
            execution_state: GuardianExecutionState::Degraded,
            route_slot: None,
            evidence_refs: Vec::new(),
            finding_ids: Vec::new(),
            degradation_reason: None,
        };

        assert!(record.validate().is_err());
    }

    #[test]
    fn execution_records_findings_and_projection_cover_success_paths() {
        let completed = GuardianExecutionRecord {
            guardian_id: "solid_guardian".to_string(),
            phase: CapabilityPhase::Verification,
            execution_state: GuardianExecutionState::Completed,
            route_slot: Some(RouteSlot::Verification),
            evidence_refs: vec!["src/lib.rs".to_string(), "cargo test --quiet".to_string()],
            finding_ids: vec!["finding-1".to_string()],
            degradation_reason: None,
        };
        let degraded = GuardianExecutionRecord {
            execution_state: GuardianExecutionState::Degraded,
            degradation_reason: Some("verification route does not support validation".to_string()),
            ..completed.clone()
        };
        let finding = GuardianFinding {
            finding_id: "finding-1".to_string(),
            guardian_id: "solid_guardian".to_string(),
            rule_id: "verification".to_string(),
            disposition: GuardianDisposition::Warn,
            summary: "tests are stale".to_string(),
            evidence_refs: vec!["tests/add.rs".to_string()],
            confidence: FindingConfidence::Medium,
            recommended_action: "refresh the verification path".to_string(),
            authority_source: GuidanceAuthoritySource::SharedPack,
            source_ref: "assistant/guardians/verification.toml".to_string(),
            phase: CapabilityPhase::Verification,
        };
        let mut projection = GuidanceGuardianProjection::default();

        assert!(completed.validate().is_ok());
        assert!(degraded.validate().is_ok());
        assert!(finding.validate().is_ok());
        assert!(projection.is_empty());

        projection.capability_resolution_summary = Some("loaded verification guidance".to_string());
        projection.guardian_findings.push(finding.clone());
        projection.guardian_timeline.push("verification_guardian completed".to_string());
        assert!(!projection.is_empty());

        let mut missing_execution_guardian = degraded;
        missing_execution_guardian.guardian_id.clear();
        assert_eq!(
            missing_execution_guardian.validate(),
            Err(GuidanceCapabilityError::MissingExecutionGuardianId)
        );

        let mut missing_finding_action = finding;
        missing_finding_action.recommended_action.clear();
        assert_eq!(
            missing_finding_action.validate(),
            Err(GuidanceCapabilityError::MissingFindingAction {
                finding_id: "finding-1".to_string(),
            })
        );
    }

    #[test]
    fn finding_validate_accepts_explicit_failure_without_evidence() {
        let finding = GuardianFinding {
            finding_id: "finding-1".to_string(),
            guardian_id: "solid_guardian".to_string(),
            rule_id: "guardian_failure".to_string(),
            disposition: GuardianDisposition::Error,
            summary: "guardian execution failed".to_string(),
            evidence_refs: Vec::new(),
            confidence: FindingConfidence::High,
            recommended_action: "inspect the guardian failure output".to_string(),
            authority_source: GuidanceAuthoritySource::BuiltIn,
            source_ref: "assistant/guardians/solid.toml".to_string(),
            phase: CapabilityPhase::Implementation,
        };

        assert!(finding.validate().is_ok());
    }
}
