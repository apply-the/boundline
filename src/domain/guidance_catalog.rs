//! Typed guidance-catalog pack models shared by runtime discovery, validation,
//! and operator-facing projections.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::guidance::{CapabilityPhase, GuardianDisposition, GuidancePriority};

const GUIDANCE_PACK_KIND: &str = "guidance-pack";
const GUIDANCE_CATALOG_KIND: &str = "guidance-catalog";
const MARKDOWN_EXTENSION: &str = "md";

/// Canonical pillar taxonomy owned by feature 055.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogPillar {
    CleanCode,
    Architecture,
    Testing,
    Language,
    Framework,
    Security,
    DomainLanguage,
    DomainModeling,
    ApiContracts,
    Migration,
    Observability,
    Resilience,
    OperationsReadiness,
    SupplyChain,
    DataAi,
    OptionalEcosystem,
}

impl CatalogPillar {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CleanCode => "clean-code",
            Self::Architecture => "architecture",
            Self::Testing => "testing",
            Self::Language => "language",
            Self::Framework => "framework",
            Self::Security => "security",
            Self::DomainLanguage => "domain-language",
            Self::DomainModeling => "domain-modeling",
            Self::ApiContracts => "api-contracts",
            Self::Migration => "migration",
            Self::Observability => "observability",
            Self::Resilience => "resilience",
            Self::OperationsReadiness => "operations-readiness",
            Self::SupplyChain => "supply-chain",
            Self::DataAi => "data-ai",
            Self::OptionalEcosystem => "optional-ecosystem",
        }
    }
}

impl fmt::Display for CatalogPillar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Canonical guidance strength vocabulary for catalog entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogGuidanceStrength {
    Mandatory,
    Recommended,
    LegacyWarning,
    TargetExcellence,
    AntiPattern,
    Deprecated,
}

impl CatalogGuidanceStrength {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mandatory => "mandatory",
            Self::Recommended => "recommended",
            Self::LegacyWarning => "legacy-warning",
            Self::TargetExcellence => "target-excellence",
            Self::AntiPattern => "anti-pattern",
            Self::Deprecated => "deprecated",
        }
    }

    pub const fn to_runtime_priority(self) -> GuidancePriority {
        match self {
            Self::Mandatory | Self::LegacyWarning | Self::AntiPattern => GuidancePriority::High,
            Self::Recommended => GuidancePriority::Medium,
            Self::TargetExcellence | Self::Deprecated => GuidancePriority::Low,
        }
    }
}

impl fmt::Display for CatalogGuidanceStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Canonical guardian disposition vocabulary for catalog entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogGuardianDisposition {
    Info,
    Observation,
    Concern,
    Warning,
    Risk,
    Blocker,
    Error,
}

impl CatalogGuardianDisposition {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Observation => "observation",
            Self::Concern => "concern",
            Self::Warning => "warning",
            Self::Risk => "risk",
            Self::Blocker => "blocker",
            Self::Error => "error",
        }
    }

    pub const fn to_runtime_disposition(self) -> GuardianDisposition {
        match self {
            Self::Info | Self::Observation => GuardianDisposition::Advise,
            Self::Concern => GuardianDisposition::Concern,
            Self::Warning | Self::Risk => GuardianDisposition::Warn,
            Self::Blocker => GuardianDisposition::Block,
            Self::Error => GuardianDisposition::Error,
        }
    }
}

impl fmt::Display for CatalogGuardianDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Catalog-side authority metadata surfaced through guidance resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogAuthoritySource {
    RuntimeEvidence,
    WorkspaceOverride,
    CanonGoverned,
    SharedPack,
    BoundlineBuiltIn,
}

impl CatalogAuthoritySource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeEvidence => "runtime-evidence",
            Self::WorkspaceOverride => "workspace-override",
            Self::CanonGoverned => "canon-governed",
            Self::SharedPack => "shared-pack",
            Self::BoundlineBuiltIn => "boundline-built-in",
        }
    }
}

impl fmt::Display for CatalogAuthoritySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Catalog lifecycle labels, including labels that collapse into existing 054
/// runtime phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogLifecycleLabel {
    Planning,
    SystemShaping,
    Architecture,
    Backlog,
    Implementation,
    Testing,
    Verification,
    Review,
    Refactor,
    Migration,
    Incident,
    SupplyChainAnalysis,
}

impl CatalogLifecycleLabel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planning => "planning",
            Self::SystemShaping => "system-shaping",
            Self::Architecture => "architecture",
            Self::Backlog => "backlog",
            Self::Implementation => "implementation",
            Self::Testing => "testing",
            Self::Verification => "verification",
            Self::Review => "review",
            Self::Refactor => "refactor",
            Self::Migration => "migration",
            Self::Incident => "incident",
            Self::SupplyChainAnalysis => "supply-chain-analysis",
        }
    }

    pub const fn matches_runtime_phase(self, phase: CapabilityPhase) -> bool {
        match phase {
            CapabilityPhase::Planning => {
                matches!(self, Self::Planning | Self::Backlog)
            }
            CapabilityPhase::Architecture => {
                matches!(self, Self::SystemShaping | Self::Architecture | Self::Migration)
            }
            CapabilityPhase::Implementation => {
                matches!(self, Self::Implementation | Self::Refactor | Self::Migration)
            }
            CapabilityPhase::Testing => matches!(self, Self::Testing),
            CapabilityPhase::Verification => {
                matches!(self, Self::Verification | Self::Incident | Self::SupplyChainAnalysis)
            }
            CapabilityPhase::Review => {
                matches!(
                    self,
                    Self::Review | Self::Refactor | Self::Migration | Self::SupplyChainAnalysis
                )
            }
        }
    }
}

impl fmt::Display for CatalogLifecycleLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Severity for explicit catalog validation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogValidationSeverity {
    Warning,
    Error,
}

impl CatalogValidationSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for CatalogValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One explicit validation finding associated with a catalog pack or entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogValidationFinding {
    pub severity: CatalogValidationSeverity,
    pub source_ref: String,
    pub message: String,
}

impl CatalogValidationFinding {
    pub fn display_line(&self) -> String {
        format!("{}: {} ({})", self.severity.as_str(), self.source_ref, self.message)
    }
}

/// Top-level `pack.toml` shape for a catalog pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogPackManifest {
    pub pack: CatalogPackIdentity,
    pub compatibility: CatalogCompatibility,
    pub authority: CatalogAuthorityDefaults,
}

impl CatalogPackManifest {
    pub fn validate(&self) -> Result<(), GuidanceCatalogError> {
        self.pack.validate_pack_identity()?;
        self.compatibility.validate()?;
        self.authority.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogPackIdentity {
    pub id: String,
    pub version: String,
    pub kind: String,
    pub description: String,
}

impl CatalogPackIdentity {
    fn validate_pack_identity(&self) -> Result<(), GuidanceCatalogError> {
        validate_non_empty(&self.id, GuidanceCatalogError::MissingPackId)?;
        validate_non_empty(&self.version, GuidanceCatalogError::MissingPackVersion)?;
        validate_non_empty(&self.description, GuidanceCatalogError::MissingPackDescription)?;
        if self.kind != GUIDANCE_PACK_KIND {
            return Err(GuidanceCatalogError::InvalidPackKind { actual: self.kind.clone() });
        }
        Ok(())
    }
}

/// Shared compatibility shape reused by `pack.toml` and `catalog-manifest.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogCompatibility {
    pub boundline: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_contract: Option<String>,
}

impl CatalogCompatibility {
    fn validate(&self) -> Result<(), GuidanceCatalogError> {
        validate_non_empty(&self.boundline, GuidanceCatalogError::MissingCompatibilityBoundline)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogAuthorityDefaults {
    pub default_source: CatalogAuthoritySource,
    pub default_strength: CatalogGuidanceStrength,
    pub canon_promotable: bool,
    pub workspace_override_allowed: bool,
}

impl CatalogAuthorityDefaults {
    fn validate(&self) -> Result<(), GuidanceCatalogError> {
        let _ = self.default_source;
        let _ = self.default_strength;
        Ok(())
    }
}

/// Top-level `catalog/catalog-manifest.toml` shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogManifest {
    pub catalog: CatalogIdentity,
    pub compatibility: CatalogCompatibility,
    pub authority: CatalogAuthorityDefaults,
    pub layout: CatalogLayout,
    pub pillars: CatalogPillarSet,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<CatalogRuntimeRequirements>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<CatalogTraceSettings>,
}

impl CatalogManifest {
    pub fn validate(&self) -> Result<(), GuidanceCatalogError> {
        self.catalog.validate_catalog_identity()?;
        self.compatibility.validate()?;
        self.authority.validate()?;
        self.layout.validate()?;
        self.pillars.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogIdentity {
    pub id: String,
    pub version: String,
    pub kind: String,
    pub status: String,
    pub description: String,
}

impl CatalogIdentity {
    fn validate_catalog_identity(&self) -> Result<(), GuidanceCatalogError> {
        validate_non_empty(&self.id, GuidanceCatalogError::MissingCatalogId)?;
        validate_non_empty(&self.version, GuidanceCatalogError::MissingCatalogVersion)?;
        validate_non_empty(&self.description, GuidanceCatalogError::MissingCatalogDescription)?;
        if self.kind != GUIDANCE_CATALOG_KIND {
            return Err(GuidanceCatalogError::InvalidCatalogKind { actual: self.kind.clone() });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogLayout {
    pub guidance_dir: String,
    pub guardians_dir: String,
    pub schemas_dir: String,
    pub examples_dir: String,
}

impl CatalogLayout {
    fn validate(&self) -> Result<(), GuidanceCatalogError> {
        validate_relative_directory(&self.guidance_dir, "layout.guidance_dir")?;
        validate_relative_directory(&self.guardians_dir, "layout.guardians_dir")?;
        validate_relative_directory(&self.schemas_dir, "layout.schemas_dir")?;
        validate_relative_directory(&self.examples_dir, "layout.examples_dir")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogPillarSet {
    pub included: Vec<CatalogPillar>,
}

impl CatalogPillarSet {
    fn validate(&self) -> Result<(), GuidanceCatalogError> {
        if self.included.is_empty() {
            return Err(GuidanceCatalogError::MissingCatalogPillars);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogRuntimeRequirements {
    pub requires_s2_1: bool,
    pub requires_s3: bool,
    pub requires_s4: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogTraceSettings {
    pub record_resolution: bool,
    pub record_authority_source: bool,
    pub record_guidance_strength: bool,
    pub record_guardian_findings: bool,
}

/// Parsed guidance index file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogGuidanceIndex {
    #[serde(default)]
    pub guidance: BTreeMap<String, CatalogGuidanceEntry>,
}

impl CatalogGuidanceIndex {
    pub fn validate(&self) -> Result<(), GuidanceCatalogError> {
        for (entry_id, entry) in &self.guidance {
            entry.validate(entry_id)?;
        }
        Ok(())
    }
}

/// One guidance index entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogGuidanceEntry {
    pub path: String,
    pub pillar: CatalogPillar,
    pub strength: CatalogGuidanceStrength,
    pub applies_to: Vec<CatalogLifecycleLabel>,
    pub roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_source: Option<CatalogAuthoritySource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_artifact_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
}

impl CatalogGuidanceEntry {
    fn validate(&self, entry_id: &str) -> Result<(), GuidanceCatalogError> {
        validate_non_empty(
            &self.path,
            GuidanceCatalogError::MissingGuidancePath { entry_id: entry_id.to_string() },
        )?;
        validate_markdown_path(&self.path, entry_id)?;
        if self.applies_to.is_empty() {
            return Err(GuidanceCatalogError::MissingGuidanceAppliesTo {
                entry_id: entry_id.to_string(),
            });
        }
        if self.roles.is_empty() || self.roles.iter().all(|role| role.trim().is_empty()) {
            return Err(GuidanceCatalogError::MissingGuidanceRoles {
                entry_id: entry_id.to_string(),
            });
        }
        Ok(())
    }
}

/// Parsed guardian index file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogGuardianIndex {
    #[serde(default)]
    pub guardian: BTreeMap<String, CatalogGuardianRuleSeed>,
}

impl CatalogGuardianIndex {
    pub fn validate(&self) -> Result<(), GuidanceCatalogError> {
        for (entry_id, entry) in &self.guardian {
            entry.validate(entry_id)?;
        }
        Ok(())
    }
}

/// One guardian rule seed entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogGuardianRuleSeed {
    pub pillar: CatalogPillar,
    pub kind: crate::domain::guidance::GuardianKind,
    pub rules: Vec<String>,
    pub applies_to: Vec<CatalogLifecycleLabel>,
    pub default_disposition: CatalogGuardianDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_guidance: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_findings: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_source: Option<CatalogAuthoritySource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl CatalogGuardianRuleSeed {
    fn validate(&self, entry_id: &str) -> Result<(), GuidanceCatalogError> {
        if self.rules.is_empty() || self.rules.iter().all(|rule| rule.trim().is_empty()) {
            return Err(GuidanceCatalogError::MissingGuardianRules {
                entry_id: entry_id.to_string(),
            });
        }
        if self.applies_to.is_empty() {
            return Err(GuidanceCatalogError::MissingGuardianAppliesTo {
                entry_id: entry_id.to_string(),
            });
        }
        Ok(())
    }
}

/// Validation failures for catalog packs and index entries.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GuidanceCatalogError {
    #[error("catalog pack id cannot be empty")]
    MissingPackId,
    #[error("catalog pack version cannot be empty")]
    MissingPackVersion,
    #[error("catalog pack description cannot be empty")]
    MissingPackDescription,
    #[error("catalog pack kind must be {GUIDANCE_PACK_KIND}, got {actual}")]
    InvalidPackKind { actual: String },
    #[error("catalog compatibility.boundline cannot be empty")]
    MissingCompatibilityBoundline,
    #[error("catalog id cannot be empty")]
    MissingCatalogId,
    #[error("catalog version cannot be empty")]
    MissingCatalogVersion,
    #[error("catalog description cannot be empty")]
    MissingCatalogDescription,
    #[error("catalog kind must be {GUIDANCE_CATALOG_KIND}, got {actual}")]
    InvalidCatalogKind { actual: String },
    #[error("catalog must include at least one canonical pillar")]
    MissingCatalogPillars,
    #[error("{field} must be a relative path under the pack root, got {path}")]
    InvalidRelativePath { field: String, path: String },
    #[error("guidance entry {entry_id} path cannot be empty")]
    MissingGuidancePath { entry_id: String },
    #[error(
        "guidance entry {entry_id} path must reference a markdown file inside the pack root, got {path}"
    )]
    InvalidGuidancePath { entry_id: String, path: String },
    #[error("guidance entry {entry_id} must declare at least one lifecycle label")]
    MissingGuidanceAppliesTo { entry_id: String },
    #[error("guidance entry {entry_id} must declare at least one role")]
    MissingGuidanceRoles { entry_id: String },
    #[error("guardian entry {entry_id} must declare at least one rule")]
    MissingGuardianRules { entry_id: String },
    #[error("guardian entry {entry_id} must declare at least one lifecycle label")]
    MissingGuardianAppliesTo { entry_id: String },
}

fn validate_non_empty(
    value: &str,
    error: GuidanceCatalogError,
) -> Result<(), GuidanceCatalogError> {
    if value.trim().is_empty() {
        return Err(error);
    }
    Ok(())
}

fn validate_relative_directory(path: &str, field: &str) -> Result<(), GuidanceCatalogError> {
    if !is_relative_subpath(path) {
        return Err(GuidanceCatalogError::InvalidRelativePath {
            field: field.to_string(),
            path: path.to_string(),
        });
    }
    Ok(())
}

fn validate_markdown_path(path: &str, entry_id: &str) -> Result<(), GuidanceCatalogError> {
    if !is_relative_subpath(path)
        || Path::new(path).extension().and_then(|value| value.to_str()) != Some(MARKDOWN_EXTENSION)
    {
        return Err(GuidanceCatalogError::InvalidGuidancePath {
            entry_id: entry_id.to_string(),
            path: path.to_string(),
        });
    }
    Ok(())
}

fn is_relative_subpath(path: &str) -> bool {
    let candidate = Path::new(path);
    !path.trim().is_empty()
        && candidate.is_relative()
        && candidate
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use super::{
        CatalogAuthoritySource, CatalogCompatibility, CatalogGuardianDisposition,
        CatalogGuardianIndex, CatalogGuardianRuleSeed, CatalogGuidanceEntry, CatalogGuidanceIndex,
        CatalogGuidanceStrength, CatalogIdentity, CatalogLayout, CatalogLifecycleLabel,
        CatalogManifest, CatalogPackIdentity, CatalogPackManifest, CatalogPillar, CatalogPillarSet,
        CatalogValidationFinding, CatalogValidationSeverity, GuidanceCatalogError,
    };
    use crate::domain::guidance::{CapabilityPhase, GuardianDisposition, GuardianKind};

    #[test]
    fn catalog_guidance_strength_maps_to_runtime_priority() {
        assert_eq!(CatalogGuidanceStrength::Mandatory.to_runtime_priority().as_str(), "high");
        assert_eq!(CatalogGuidanceStrength::Recommended.to_runtime_priority().as_str(), "medium");
        assert_eq!(CatalogGuidanceStrength::TargetExcellence.to_runtime_priority().as_str(), "low");
    }

    #[test]
    fn catalog_lifecycle_labels_collapse_into_runtime_phases() {
        assert!(
            CatalogLifecycleLabel::Backlog
                .matches_runtime_phase(crate::domain::guidance::CapabilityPhase::Planning)
        );
        assert!(
            CatalogLifecycleLabel::SystemShaping
                .matches_runtime_phase(crate::domain::guidance::CapabilityPhase::Architecture)
        );
        assert!(
            CatalogLifecycleLabel::SupplyChainAnalysis
                .matches_runtime_phase(crate::domain::guidance::CapabilityPhase::Verification)
        );
        assert!(
            !CatalogLifecycleLabel::Incident
                .matches_runtime_phase(crate::domain::guidance::CapabilityPhase::Testing)
        );
    }

    #[test]
    fn catalog_manifest_rejects_absolute_layout_paths() {
        let manifest = CatalogManifest {
            catalog: CatalogIdentity {
                id: "catalog".to_string(),
                version: "0.1.0".to_string(),
                kind: "guidance-catalog".to_string(),
                status: "draft".to_string(),
                description: "catalog".to_string(),
            },
            compatibility: CatalogCompatibility {
                boundline: ">=0.55".to_string(),
                canon_contract: None,
            },
            authority: super::CatalogAuthorityDefaults {
                default_source: super::CatalogAuthoritySource::SharedPack,
                default_strength: CatalogGuidanceStrength::Recommended,
                canon_promotable: true,
                workspace_override_allowed: true,
            },
            layout: CatalogLayout {
                guidance_dir: "/guidance".to_string(),
                guardians_dir: "guardians".to_string(),
                schemas_dir: "schemas".to_string(),
                examples_dir: "examples".to_string(),
            },
            pillars: CatalogPillarSet { included: vec![CatalogPillar::CleanCode] },
            runtime: None,
            trace: None,
        };

        assert_eq!(
            manifest.validate(),
            Err(GuidanceCatalogError::InvalidRelativePath {
                field: "layout.guidance_dir".to_string(),
                path: "/guidance".to_string(),
            })
        );
    }

    #[test]
    fn guidance_index_rejects_non_markdown_entry_paths() {
        let index = CatalogGuidanceIndex {
            guidance: [(
                "guidance.clean_code".to_string(),
                CatalogGuidanceEntry {
                    path: "guidance/clean-code.txt".to_string(),
                    pillar: CatalogPillar::CleanCode,
                    strength: CatalogGuidanceStrength::Recommended,
                    applies_to: vec![CatalogLifecycleLabel::Implementation],
                    roles: vec!["implementer".to_string()],
                    language: None,
                    framework: None,
                    authority_source: None,
                    canon_artifact_kind: None,
                    owner: None,
                    version: None,
                    deprecated: false,
                    replaced_by: None,
                },
            )]
            .into_iter()
            .collect(),
        };

        assert_eq!(
            index.validate(),
            Err(GuidanceCatalogError::InvalidGuidancePath {
                entry_id: "guidance.clean_code".to_string(),
                path: "guidance/clean-code.txt".to_string(),
            })
        );
    }

    #[test]
    fn pack_manifest_requires_guidance_pack_kind() {
        let pack = CatalogPackManifest {
            pack: CatalogPackIdentity {
                id: "catalog".to_string(),
                version: "0.1.0".to_string(),
                kind: "wrong-kind".to_string(),
                description: "catalog".to_string(),
            },
            compatibility: CatalogCompatibility {
                boundline: ">=0.55".to_string(),
                canon_contract: None,
            },
            authority: super::CatalogAuthorityDefaults {
                default_source: super::CatalogAuthoritySource::SharedPack,
                default_strength: CatalogGuidanceStrength::Recommended,
                canon_promotable: true,
                workspace_override_allowed: true,
            },
        };

        assert_eq!(
            pack.validate(),
            Err(GuidanceCatalogError::InvalidPackKind { actual: "wrong-kind".to_string() })
        );
    }

    #[test]
    fn catalog_enums_cover_canonical_labels_and_runtime_mappings() {
        let pillars = [
            CatalogPillar::CleanCode,
            CatalogPillar::Architecture,
            CatalogPillar::Testing,
            CatalogPillar::Language,
            CatalogPillar::Framework,
            CatalogPillar::Security,
            CatalogPillar::DomainLanguage,
            CatalogPillar::DomainModeling,
            CatalogPillar::ApiContracts,
            CatalogPillar::Migration,
            CatalogPillar::Observability,
            CatalogPillar::Resilience,
            CatalogPillar::OperationsReadiness,
            CatalogPillar::SupplyChain,
            CatalogPillar::DataAi,
            CatalogPillar::OptionalEcosystem,
        ];
        assert!(pillars.iter().all(|pillar| !pillar.to_string().is_empty()));

        let strengths = [
            (CatalogGuidanceStrength::Mandatory, "mandatory", "high"),
            (CatalogGuidanceStrength::Recommended, "recommended", "medium"),
            (CatalogGuidanceStrength::LegacyWarning, "legacy-warning", "high"),
            (CatalogGuidanceStrength::TargetExcellence, "target-excellence", "low"),
            (CatalogGuidanceStrength::AntiPattern, "anti-pattern", "high"),
            (CatalogGuidanceStrength::Deprecated, "deprecated", "low"),
        ];
        for (strength, label, priority) in strengths {
            assert_eq!(strength.as_str(), label);
            assert_eq!(strength.to_string(), label);
            assert_eq!(strength.to_runtime_priority().as_str(), priority);
        }

        let dispositions = [
            (CatalogGuardianDisposition::Info, "info", GuardianDisposition::Advise),
            (CatalogGuardianDisposition::Observation, "observation", GuardianDisposition::Advise),
            (CatalogGuardianDisposition::Concern, "concern", GuardianDisposition::Concern),
            (CatalogGuardianDisposition::Warning, "warning", GuardianDisposition::Warn),
            (CatalogGuardianDisposition::Risk, "risk", GuardianDisposition::Warn),
            (CatalogGuardianDisposition::Blocker, "blocker", GuardianDisposition::Block),
            (CatalogGuardianDisposition::Error, "error", GuardianDisposition::Error),
        ];
        for (disposition, label, runtime) in dispositions {
            assert_eq!(disposition.as_str(), label);
            assert_eq!(disposition.to_string(), label);
            assert_eq!(disposition.to_runtime_disposition(), runtime);
        }

        let authorities = [
            CatalogAuthoritySource::RuntimeEvidence,
            CatalogAuthoritySource::WorkspaceOverride,
            CatalogAuthoritySource::CanonGoverned,
            CatalogAuthoritySource::SharedPack,
            CatalogAuthoritySource::BoundlineBuiltIn,
        ];
        assert_eq!(
            authorities.iter().map(|authority| authority.to_string()).collect::<Vec<_>>(),
            vec![
                "runtime-evidence".to_string(),
                "workspace-override".to_string(),
                "canon-governed".to_string(),
                "shared-pack".to_string(),
                "boundline-built-in".to_string(),
            ]
        );

        let severities = [CatalogValidationSeverity::Warning, CatalogValidationSeverity::Error];
        assert_eq!(
            severities.iter().map(|severity| severity.to_string()).collect::<Vec<_>>(),
            vec!["warning".to_string(), "error".to_string()]
        );

        let labels = [
            (CatalogLifecycleLabel::Planning, CapabilityPhase::Planning, true),
            (CatalogLifecycleLabel::Backlog, CapabilityPhase::Planning, true),
            (CatalogLifecycleLabel::SystemShaping, CapabilityPhase::Architecture, true),
            (CatalogLifecycleLabel::Architecture, CapabilityPhase::Architecture, true),
            (CatalogLifecycleLabel::Implementation, CapabilityPhase::Implementation, true),
            (CatalogLifecycleLabel::Refactor, CapabilityPhase::Implementation, true),
            (CatalogLifecycleLabel::Migration, CapabilityPhase::Review, true),
            (CatalogLifecycleLabel::Testing, CapabilityPhase::Testing, true),
            (CatalogLifecycleLabel::Verification, CapabilityPhase::Verification, true),
            (CatalogLifecycleLabel::Incident, CapabilityPhase::Verification, true),
            (CatalogLifecycleLabel::SupplyChainAnalysis, CapabilityPhase::Review, true),
            (CatalogLifecycleLabel::Review, CapabilityPhase::Testing, false),
        ];
        for (label, phase, expected) in labels {
            assert_eq!(
                label.matches_runtime_phase(phase),
                expected,
                "{label} vs {}",
                phase.as_str()
            );
        }
    }

    #[test]
    fn catalog_validation_finding_formats_display_line() {
        let finding = CatalogValidationFinding {
            severity: CatalogValidationSeverity::Warning,
            source_ref: "assistant/packs/guidance-catalog/catalog/guidance-index.toml".to_string(),
            message: "legacy alias normalized".to_string(),
        };

        assert_eq!(
            finding.display_line(),
            "warning: assistant/packs/guidance-catalog/catalog/guidance-index.toml (legacy alias normalized)"
        );
    }

    #[test]
    fn catalog_models_reject_missing_required_fields() {
        let pack = CatalogPackManifest {
            pack: CatalogPackIdentity {
                id: "".to_string(),
                version: "0.1.0".to_string(),
                kind: "guidance-pack".to_string(),
                description: "catalog".to_string(),
            },
            compatibility: CatalogCompatibility { boundline: "".to_string(), canon_contract: None },
            authority: super::CatalogAuthorityDefaults {
                default_source: CatalogAuthoritySource::SharedPack,
                default_strength: CatalogGuidanceStrength::Recommended,
                canon_promotable: true,
                workspace_override_allowed: true,
            },
        };
        assert_eq!(pack.validate(), Err(GuidanceCatalogError::MissingPackId));

        let manifest = CatalogManifest {
            catalog: CatalogIdentity {
                id: "catalog".to_string(),
                version: "0.1.0".to_string(),
                kind: "guidance-catalog".to_string(),
                status: "draft".to_string(),
                description: "catalog".to_string(),
            },
            compatibility: CatalogCompatibility {
                boundline: ">=0.55".to_string(),
                canon_contract: None,
            },
            authority: super::CatalogAuthorityDefaults {
                default_source: CatalogAuthoritySource::SharedPack,
                default_strength: CatalogGuidanceStrength::Recommended,
                canon_promotable: true,
                workspace_override_allowed: true,
            },
            layout: CatalogLayout {
                guidance_dir: "guidance".to_string(),
                guardians_dir: "../guardians".to_string(),
                schemas_dir: "schemas".to_string(),
                examples_dir: "examples".to_string(),
            },
            pillars: CatalogPillarSet { included: Vec::new() },
            runtime: None,
            trace: None,
        };
        assert_eq!(
            manifest.validate(),
            Err(GuidanceCatalogError::InvalidRelativePath {
                field: "layout.guardians_dir".to_string(),
                path: "../guardians".to_string(),
            })
        );

        let missing_pillars = CatalogManifest {
            layout: CatalogLayout {
                guidance_dir: "guidance".to_string(),
                guardians_dir: "guardians".to_string(),
                schemas_dir: "schemas".to_string(),
                examples_dir: "examples".to_string(),
            },
            ..manifest
        };
        assert_eq!(missing_pillars.validate(), Err(GuidanceCatalogError::MissingCatalogPillars));
    }

    #[test]
    fn guidance_and_guardian_entries_require_roles_rules_and_lifecycle_labels() {
        let guidance = CatalogGuidanceIndex {
            guidance: [(
                "clean_code".to_string(),
                CatalogGuidanceEntry {
                    path: "guidance/clean-code.md".to_string(),
                    pillar: CatalogPillar::CleanCode,
                    strength: CatalogGuidanceStrength::Recommended,
                    applies_to: Vec::new(),
                    roles: vec![" ".to_string()],
                    language: None,
                    framework: None,
                    authority_source: None,
                    canon_artifact_kind: None,
                    owner: None,
                    version: None,
                    deprecated: false,
                    replaced_by: None,
                },
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            guidance.validate(),
            Err(GuidanceCatalogError::MissingGuidanceAppliesTo {
                entry_id: "clean_code".to_string(),
            })
        );

        let guidance = CatalogGuidanceIndex {
            guidance: [(
                "clean_code".to_string(),
                CatalogGuidanceEntry {
                    path: "guidance/clean-code.md".to_string(),
                    pillar: CatalogPillar::CleanCode,
                    strength: CatalogGuidanceStrength::Recommended,
                    applies_to: vec![CatalogLifecycleLabel::Implementation],
                    roles: vec![" ".to_string()],
                    language: None,
                    framework: None,
                    authority_source: None,
                    canon_artifact_kind: None,
                    owner: None,
                    version: None,
                    deprecated: false,
                    replaced_by: None,
                },
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            guidance.validate(),
            Err(GuidanceCatalogError::MissingGuidanceRoles { entry_id: "clean_code".to_string() })
        );

        let guardians = CatalogGuardianIndex {
            guardian: [(
                "rust_zero_panic".to_string(),
                CatalogGuardianRuleSeed {
                    pillar: CatalogPillar::Language,
                    kind: GuardianKind::Deterministic,
                    rules: Vec::new(),
                    applies_to: vec![CatalogLifecycleLabel::Implementation],
                    default_disposition: CatalogGuardianDisposition::Warning,
                    language: None,
                    framework: None,
                    requires_guidance: None,
                    requires_tools: None,
                    timeout_seconds: None,
                    max_findings: None,
                    authority_source: None,
                    owner: None,
                    version: None,
                },
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            guardians.validate(),
            Err(GuidanceCatalogError::MissingGuardianRules {
                entry_id: "rust_zero_panic".to_string(),
            })
        );

        let guardians = CatalogGuardianIndex {
            guardian: [(
                "rust_zero_panic".to_string(),
                CatalogGuardianRuleSeed {
                    pillar: CatalogPillar::Language,
                    kind: GuardianKind::Deterministic,
                    rules: vec!["no-panic".to_string()],
                    applies_to: Vec::new(),
                    default_disposition: CatalogGuardianDisposition::Warning,
                    language: None,
                    framework: None,
                    requires_guidance: None,
                    requires_tools: None,
                    timeout_seconds: None,
                    max_findings: None,
                    authority_source: None,
                    owner: None,
                    version: None,
                },
            )]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            guardians.validate(),
            Err(GuidanceCatalogError::MissingGuardianAppliesTo {
                entry_id: "rust_zero_panic".to_string(),
            })
        );
    }
}
