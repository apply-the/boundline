use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::governance::{
    ApprovalState, CanonMode, CanonSemanticArtifactDescriptorV1Envelope, PacketReadiness,
};
use crate::domain::project_index::{ProjectDocRoots, resolve_project_doc_roots};

/// Supported contract line for this Boundline integration slice.
const SUPPORTED_CONTRACT_MAJOR: u64 = 1;
const PROFILE_METADATA_FILE_SUFFIX: &str = ".packet-metadata.json";
const LEGACY_LINEAGE_FILE_SUFFIX: &str = ".lineage.json";
const CANON_PACKET_METADATA_FILE_NAME: &str = "packet-metadata.json";
const CANON_LINEAGE_CONTRACT_VERSION: &str = "v1";
const CANON_SOURCE_REF_PREFIX: &str = "canon-run:";
const EVIDENCE_ONLY_PROMOTION_STATE: &str = "evidence-only";
const EVIDENCE_PROMOTION_PROFILE: &str = "evidence-bundle";
const MANAGED_BLOCK_START_MARKER: &str = "<!-- project-memory:managed:start";
const CANON_PROJECT_SURFACES: [&str; 11] = [
    "overview.md",
    "product-context.md",
    "domain-language.md",
    "domain-model.md",
    "architecture-map.md",
    "decision-index.md",
    "delivery-map.md",
    "operational-context.md",
    "pending-decisions.md",
    "open-risks.md",
    "audit-log.md",
];

/// Consumer-side read-only projection of Canon's promotion state vocabulary.
///
/// Boundline MUST NOT redefine Canon promotion semantics; these variants
/// represent the consumer's interpretation for delivery decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromotionStateView {
    /// Canon output is accepted; Boundline may use as credible context.
    Stable,
    /// Canon output is pending or index-only; visible but non-authoritative.
    PendingOrIndex,
    /// Canon output is evidence-only; usable for assurance, not planning.
    EvidenceOnly,
    /// Canon output requires manual action; treat as absent for planning.
    Manual,
    /// Canon output has an unrecognized promotion state; non-authoritative.
    Unknown,
}

impl PromotionStateView {
    /// Map a raw Canon promotion state string to the consumer-side view.
    ///
    /// `auto-if-approved` requires approval metadata before it can be treated
    /// as stable project memory, so the raw-state-only mapping stays unknown.
    pub fn from_canon_state(raw: &str) -> Self {
        match raw {
            "auto" => Self::Stable,
            "auto-if-approved" => Self::Unknown,
            "pending-index" | "index-only" => Self::PendingOrIndex,
            "evidence-only" => Self::EvidenceOnly,
            "manual" => Self::Manual,
            _ => Self::Unknown,
        }
    }

    /// Map Canon lineage metadata to the consumer-side promotion view.
    pub fn from_lineage(lineage: &LineageRef) -> Self {
        match lineage.promotion_state.as_str() {
            "auto-if-approved" => {
                match (lineage.approval_state.as_deref(), lineage.packet_readiness.as_deref()) {
                    (Some(_), Some(_))
                        if lineage.approval_satisfied() && lineage.packet_ready() =>
                    {
                        Self::Stable
                    }
                    (Some(_), Some(_)) => Self::PendingOrIndex,
                    (Some(_), None) | (None, _) => Self::Unknown,
                }
            }
            other => Self::from_canon_state(other),
        }
    }

    /// Returns true when this view represents credible context for planning.
    pub fn is_credible(&self) -> bool {
        matches!(self, Self::Stable)
    }
}

impl std::fmt::Display for PromotionStateView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => f.write_str("stable"),
            Self::PendingOrIndex => f.write_str("pending-or-index"),
            Self::EvidenceOnly => f.write_str("evidence-only"),
            Self::Manual => f.write_str("manual"),
            Self::Unknown => f.write_str("unknown"),
        }
    }
}

/// Consumer-side lineage metadata preserved from Canon project-memory output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineageRef {
    pub contract_version: String,
    #[serde(default = "default_canon_producer")]
    pub producer: String,
    #[serde(alias = "source_run")]
    pub source_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_artifacts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    pub promotion_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, alias = "published_at", skip_serializing_if = "String::is_empty")]
    pub promoted_at: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "readiness")]
    pub packet_readiness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "profile")]
    pub promotion_profile: Option<String>,
}

impl LineageRef {
    /// Return the most stable human-readable run fragment for display.
    pub fn source_ref_leaf(&self) -> &str {
        self.source_ref.rsplit(':').next().unwrap_or(&self.source_ref)
    }

    /// Return the emitting Canon mode when present.
    pub fn mode_name(&self) -> &str {
        self.mode.as_deref().unwrap_or("unknown-mode")
    }

    /// Whether Canon requires an approval outcome before this surface can be reused.
    pub fn requires_approval(&self) -> bool {
        self.promotion_state.eq_ignore_ascii_case("auto-if-approved")
    }

    /// Whether the lineage records a satisfied approval outcome.
    pub fn approval_satisfied(&self) -> bool {
        matches!(
            normalized_metadata_value(self.approval_state.as_deref()).as_deref(),
            Some("completed" | "granted" | "not_needed" | "not-needed")
        )
    }

    /// Whether the lineage records a blocked approval outcome.
    pub fn approval_blocked(&self) -> bool {
        matches!(
            normalized_metadata_value(self.approval_state.as_deref()).as_deref(),
            Some("rejected" | "expired")
        )
    }

    /// Whether the lineage records a reusable packet.
    pub fn packet_ready(&self) -> bool {
        matches!(
            normalized_metadata_value(self.packet_readiness.as_deref()).as_deref(),
            Some("complete" | "ready" | "reusable")
        )
    }

    /// Whether the lineage records a rejected packet outcome.
    pub fn packet_blocked(&self) -> bool {
        matches!(
            normalized_metadata_value(self.packet_readiness.as_deref()).as_deref(),
            Some("rejected")
        )
    }
}

fn default_canon_producer() -> String {
    "canon".to_string()
}

fn normalized_metadata_value(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim).filter(|value| !value.is_empty()).map(|value| value.to_ascii_lowercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CanonSurfaceMetadata {
    #[serde(default)]
    lineage: Option<LineageRef>,
    #[serde(default)]
    publication_target_class: Option<String>,
    #[serde(default)]
    semantic_descriptor: Option<CanonSemanticArtifactDescriptorV1Envelope>,
    #[serde(default)]
    expertise_input: Option<ExpertiseInputRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpertiseInputRef {
    pub expertise_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domain_families: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernedExpertiseInputSurface {
    pub path: PathBuf,
    pub lineage: LineageRef,
    pub promotion_view: PromotionStateView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_target_class: Option<String>,
    pub expertise_input: ExpertiseInputRef,
}

/// Read-only consumer view of Canon semantic metadata carried beside one artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonSemanticArtifactSurface {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<LineageRef>,
    pub promotion_view: PromotionStateView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_target_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_descriptor: Option<CanonSemanticArtifactDescriptorV1Envelope>,
}

pub struct GovernedEvidencePromotionRequest<'a> {
    pub canon_mode: CanonMode,
    pub stage_key: &'a str,
    pub run_ref: &'a str,
    pub approval_state: ApprovalState,
    pub packet_readiness: PacketReadiness,
    pub packet_ref: &'a str,
    pub document_refs: &'a [String],
}

#[derive(Debug, Error)]
pub enum GovernedEvidencePromotionError {
    #[error("failed to read Canon packet metadata at {path}: {source}")]
    ReadPacketMetadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse Canon packet metadata at {path}: {source}")]
    ParsePacketMetadata {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create evidence root {path}: {source}")]
    CreateEvidenceRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to copy Canon artifact from {source_path} to {target_path}: {source}")]
    CopyArtifact {
        source_path: PathBuf,
        target_path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize evidence metadata for {path}: {source}")]
    SerializeEvidenceMetadata {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write evidence metadata at {path}: {source}")]
    WriteEvidenceMetadata {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct CanonPacketPromotionEnvelope {
    #[serde(default)]
    metadata: CanonPacketPromotionMetadata,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct CanonPacketPromotionMetadata {
    #[serde(default)]
    publication_target_class: Option<String>,
    #[serde(default)]
    semantic_descriptor: Option<CanonSemanticArtifactDescriptorV1Envelope>,
    #[serde(default)]
    expertise_input: Option<ExpertiseInputRef>,
}

/// Result of checking a Canon `contract_version` against the Boundline
/// supported version window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatibilityOutcome {
    /// Major version matches; all required fields present.
    Compatible,
    /// Major version mismatch; bounded stop with guidance.
    Unsupported,
}

impl CompatibilityOutcome {
    /// Check a Canon contract version string against the supported major.
    pub fn check(contract_version: &str) -> Self {
        match parse_contract_version(contract_version) {
            Some((SUPPORTED_CONTRACT_MAJOR, _)) => Self::Compatible,
            Some(_) => Self::Unsupported,
            None => Self::Unsupported,
        }
    }
}

fn parse_contract_version(contract_version: &str) -> Option<(u64, Option<u64>)> {
    let normalized = contract_version.trim();
    let numeric =
        normalized.strip_prefix('v').or_else(|| normalized.strip_prefix('V')).unwrap_or(normalized);

    let mut segments = numeric.split('.');
    let major = segments.next()?.parse::<u64>().ok()?;
    let minor = match segments.next() {
        Some(value) => Some(value.parse::<u64>().ok()?),
        None => None,
    };
    Some((major, minor))
}

fn merge_compatibility(
    current: Option<CompatibilityOutcome>,
    candidate: CompatibilityOutcome,
) -> Option<CompatibilityOutcome> {
    Some(match (current, candidate) {
        (Some(CompatibilityOutcome::Unsupported), _) | (_, CompatibilityOutcome::Unsupported) => {
            CompatibilityOutcome::Unsupported
        }
        _ => CompatibilityOutcome::Compatible,
    })
}

/// High-level status of Canon project-memory availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMemoryStatus {
    /// Canon project-memory surfaces are available.
    Available,
    /// No Canon project-memory surfaces found.
    Absent,
    /// Canon project-memory found but contract version is incompatible.
    Incompatible,
}

/// Consumer-side continuation stance for Canon project memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMemoryDecision {
    Proceed,
    Warning,
    HardStop,
}

impl ProjectMemoryDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::Warning => "warning",
            Self::HardStop => "hard-stop",
        }
    }
}

/// Primary condition that determines how Boundline should consume Canon
/// project memory for the current workspace state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectMemoryCondition {
    Stable,
    Pending,
    EvidenceOnly,
    ManualPromotion,
    BlockedGovernance,
    MissingRequiredApproval,
    MissingRequiredSourceArtifacts,
    IncompleteMetadata,
    UnsupportedContract,
}

impl ProjectMemoryCondition {
    pub const fn decision(self) -> ProjectMemoryDecision {
        match self {
            Self::Stable => ProjectMemoryDecision::Proceed,
            Self::BlockedGovernance
            | Self::MissingRequiredApproval
            | Self::MissingRequiredSourceArtifacts
            | Self::UnsupportedContract => ProjectMemoryDecision::HardStop,
            Self::Pending
            | Self::EvidenceOnly
            | Self::ManualPromotion
            | Self::IncompleteMetadata => ProjectMemoryDecision::Warning,
        }
    }

    pub const fn headline(self) -> &'static str {
        match self {
            Self::Stable => "Canon project memory available",
            Self::Pending => "Canon project memory is pending",
            Self::EvidenceOnly => "Canon project memory is evidence-only",
            Self::ManualPromotion => "Canon project memory requires manual promotion",
            Self::BlockedGovernance => "Canon project memory reports blocked governance",
            Self::MissingRequiredApproval => {
                "Canon project memory is waiting for required approval"
            }
            Self::MissingRequiredSourceArtifacts => {
                "Canon project memory is missing required source artifacts"
            }
            Self::IncompleteMetadata => "Canon project memory metadata is incomplete",
            Self::UnsupportedContract => "Canon project memory contract is unsupported",
        }
    }

    pub const fn reason_code(self) -> Option<&'static str> {
        match self {
            Self::Stable => None,
            Self::Pending => Some("project_memory_pending"),
            Self::EvidenceOnly => Some("project_memory_evidence_only"),
            Self::ManualPromotion => Some("project_memory_manual"),
            Self::BlockedGovernance => Some("project_memory_blocked"),
            Self::MissingRequiredApproval => Some("project_memory_missing_approval"),
            Self::MissingRequiredSourceArtifacts => Some("project_memory_missing_source_artifacts"),
            Self::IncompleteMetadata => Some("project_memory_unknown"),
            Self::UnsupportedContract => Some("project_memory_contract_incompatible"),
        }
    }
}

/// A single Canon-promoted document discovered by Boundline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMemorySurface {
    pub path: PathBuf,
    pub lineage: Option<LineageRef>,
    pub promotion_view: PromotionStateView,
    pub category: String,
}

/// Aggregated consumer-side snapshot of Canon project-memory state for a
/// given workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMemoryContext {
    pub status: ProjectMemoryStatus,
    pub compatibility: Option<CompatibilityOutcome>,
    pub surfaces: Vec<ProjectMemorySurface>,
    pub evidence_refs: Vec<LineageRef>,
    pub effective_promotion_state: Option<PromotionStateView>,
}

impl ProjectMemoryContext {
    /// Create an absent context when no Canon output is available.
    pub fn absent() -> Self {
        Self {
            status: ProjectMemoryStatus::Absent,
            compatibility: None,
            surfaces: Vec::new(),
            evidence_refs: Vec::new(),
            effective_promotion_state: None,
        }
    }

    /// Create an incompatible context with repair guidance.
    pub fn incompatible() -> Self {
        Self {
            status: ProjectMemoryStatus::Incompatible,
            compatibility: Some(CompatibilityOutcome::Unsupported),
            surfaces: Vec::new(),
            evidence_refs: Vec::new(),
            effective_promotion_state: None,
        }
    }

    /// Returns true when this context has credible stable project memory.
    pub fn has_credible_memory(&self) -> bool {
        self.effective_promotion_state.is_some_and(|s| s.is_credible())
    }

    /// Returns the primary project-memory condition when Canon output is
    /// available enough to classify.
    pub fn condition(&self) -> Option<ProjectMemoryCondition> {
        match self.status {
            ProjectMemoryStatus::Absent => None,
            ProjectMemoryStatus::Incompatible => Some(ProjectMemoryCondition::UnsupportedContract),
            ProjectMemoryStatus::Available => {
                if self.has_blocked_governance() {
                    return Some(ProjectMemoryCondition::BlockedGovernance);
                }
                if self.has_missing_required_approval() {
                    return Some(ProjectMemoryCondition::MissingRequiredApproval);
                }
                Some(match self.effective_promotion_state.unwrap_or(PromotionStateView::Unknown) {
                    PromotionStateView::Stable => ProjectMemoryCondition::Stable,
                    PromotionStateView::PendingOrIndex => ProjectMemoryCondition::Pending,
                    PromotionStateView::EvidenceOnly => ProjectMemoryCondition::EvidenceOnly,
                    PromotionStateView::Manual => ProjectMemoryCondition::ManualPromotion,
                    PromotionStateView::Unknown => ProjectMemoryCondition::IncompleteMetadata,
                })
            }
        }
    }

    /// Returns the project-memory condition after validating workspace-visible
    /// producer evidence such as required source artifacts.
    pub fn condition_for_workspace(&self, workspace_root: &Path) -> Option<ProjectMemoryCondition> {
        if self.has_missing_required_source_artifacts(workspace_root) {
            return Some(ProjectMemoryCondition::MissingRequiredSourceArtifacts);
        }

        self.condition()
    }

    pub fn decision(&self) -> Option<ProjectMemoryDecision> {
        self.condition().map(ProjectMemoryCondition::decision)
    }

    /// Returns the workspace-aware project-memory decision.
    pub fn decision_for_workspace(&self, workspace_root: &Path) -> Option<ProjectMemoryDecision> {
        self.condition_for_workspace(workspace_root).map(ProjectMemoryCondition::decision)
    }

    fn has_blocked_governance(&self) -> bool {
        self.surfaces
            .iter()
            .filter_map(|surface| surface.lineage.as_ref())
            .any(|lineage| lineage.approval_blocked() || lineage.packet_blocked())
    }

    fn has_missing_required_approval(&self) -> bool {
        self.surfaces.iter().filter_map(|surface| surface.lineage.as_ref()).any(|lineage| {
            lineage.requires_approval()
                && !lineage.approval_satisfied()
                && !lineage.approval_blocked()
        })
    }

    fn has_missing_required_source_artifacts(&self, workspace_root: &Path) -> bool {
        self.surfaces.iter().any(|surface| {
            let Some(lineage) = surface.lineage.as_ref() else {
                return false;
            };

            let artifacts_required = surface.promotion_view.is_credible()
                || (lineage.requires_approval() && lineage.approval_satisfied());
            if !artifacts_required {
                return false;
            }

            if lineage.requires_approval()
                && lineage.approval_satisfied()
                && !lineage.packet_ready()
            {
                return true;
            }

            if lineage.source_artifacts.is_empty() {
                return false;
            }

            let evidence_root = evidence_root_for_lineage(workspace_root, lineage);
            lineage.source_artifacts.iter().any(|artifact| !evidence_root.join(artifact).exists())
        })
    }
}

/// Read Canon-promoted project-memory surfaces from a workspace.
///
/// Reads Canon's named `docs/project/*.md` surfaces and their adjacent
/// `<surface>.packet-metadata.json` sidecars. Legacy `*.lineage.json`
/// sidecars remain tolerated for compatibility with earlier fixtures.
pub fn read_project_memory(workspace_root: &std::path::Path) -> ProjectMemoryContext {
    let doc_roots =
        resolve_project_doc_roots(workspace_root).unwrap_or_else(|_| ProjectDocRoots::default());
    let project_dir = doc_roots.project_memory_dir(workspace_root);
    let evidence_dir = doc_roots.evidence_dir(workspace_root);

    if !project_dir.exists() && !evidence_dir.exists() {
        return ProjectMemoryContext::absent();
    }

    let mut surfaces = Vec::new();
    let mut evidence_refs = Vec::new();
    let mut best_state: Option<PromotionStateView> = None;
    let mut compatibility: Option<CompatibilityOutcome> = None;
    let mut discovered_paths = BTreeSet::new();

    for relative_path in CANON_PROJECT_SURFACES {
        let path = project_dir.join(relative_path);
        if !path.exists() {
            continue;
        }

        discovered_paths.insert(path.clone());
        if collect_surface(
            workspace_root,
            &path,
            true,
            &mut surfaces,
            &mut evidence_refs,
            &mut best_state,
            &mut compatibility,
        )
        .is_err()
        {
            return ProjectMemoryContext::incompatible();
        }
    }

    // Tolerate additive top-level surfaces when Canon extends the project
    // surface set without breaking the current contract line.
    if let Ok(entries) = std::fs::read_dir(&project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file()
                || path.extension().is_none_or(|ext| ext != "md")
                || discovered_paths.contains(&path)
            {
                continue;
            }

            if collect_surface(
                workspace_root,
                &path,
                false,
                &mut surfaces,
                &mut evidence_refs,
                &mut best_state,
                &mut compatibility,
            )
            .is_err()
            {
                return ProjectMemoryContext::incompatible();
            }
        }
    }

    if evidence_refs.is_empty() && evidence_dir.exists() {
        for surface in &surfaces {
            if let Some(lineage) = &surface.lineage {
                let evidence_root = evidence_root_for_lineage(workspace_root, lineage);
                if evidence_root.exists() && !evidence_refs.contains(lineage) {
                    evidence_refs.push(lineage.clone());
                }
            }
        }
    }

    if surfaces.is_empty() && evidence_refs.is_empty() {
        return ProjectMemoryContext::absent();
    }

    ProjectMemoryContext {
        status: ProjectMemoryStatus::Available,
        compatibility,
        surfaces,
        evidence_refs,
        effective_promotion_state: best_state,
    }
}

/// Read Canon project-memory surfaces that carry governed expertise metadata.
pub fn read_governed_expertise_inputs(workspace_root: &Path) -> Vec<GovernedExpertiseInputSurface> {
    let doc_roots =
        resolve_project_doc_roots(workspace_root).unwrap_or_else(|_| ProjectDocRoots::default());
    let project_dir = doc_roots.project_memory_dir(workspace_root);
    if !project_dir.exists() {
        return Vec::new();
    }

    let mut discovered_paths = BTreeSet::new();
    let mut inputs = Vec::new();

    for relative_path in CANON_PROJECT_SURFACES {
        let path = project_dir.join(relative_path);
        if !path.exists() {
            continue;
        }
        discovered_paths.insert(path.clone());
        collect_governed_expertise_input(workspace_root, &path, &mut inputs);
    }

    if let Ok(entries) = std::fs::read_dir(&project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file()
                || path.extension().is_none_or(|extension| extension != "md")
                || discovered_paths.contains(&path)
            {
                continue;
            }
            collect_governed_expertise_input(workspace_root, &path, &mut inputs);
        }
    }

    inputs
}

fn collect_surface(
    workspace_root: &Path,
    surface_path: &Path,
    allow_without_lineage: bool,
    surfaces: &mut Vec<ProjectMemorySurface>,
    evidence_refs: &mut Vec<LineageRef>,
    best_state: &mut Option<PromotionStateView>,
    compatibility: &mut Option<CompatibilityOutcome>,
) -> Result<(), ()> {
    let lineage = read_lineage_sidecar(surface_path);
    if lineage.is_none() && !allow_without_lineage {
        return Ok(());
    }

    let promotion_view = lineage
        .as_ref()
        .map(PromotionStateView::from_lineage)
        .unwrap_or(PromotionStateView::Unknown);

    if let Some(ref lineage) = lineage {
        let compat = CompatibilityOutcome::check(&lineage.contract_version);
        if compat == CompatibilityOutcome::Unsupported {
            return Err(());
        }
        *compatibility = merge_compatibility(*compatibility, compat);

        let evidence_root = evidence_root_for_lineage(workspace_root, lineage);
        if evidence_root.exists() && !evidence_refs.contains(lineage) {
            evidence_refs.push(lineage.clone());
        }
    }

    if promotion_view.is_credible() {
        *best_state = Some(PromotionStateView::Stable);
    } else if best_state.is_none() {
        *best_state = Some(promotion_view);
    }

    surfaces.push(ProjectMemorySurface {
        path: surface_path.strip_prefix(workspace_root).unwrap_or(surface_path).to_path_buf(),
        lineage,
        promotion_view,
        category: surface_category(surface_path),
    });

    Ok(())
}

/// Read a lineage sidecar JSON file for a given artifact path.
fn read_lineage_sidecar(artifact_path: &std::path::Path) -> Option<LineageRef> {
    if let Some(lineage) = read_packet_metadata_sidecar(artifact_path) {
        return Some(lineage);
    }

    let sidecar = legacy_lineage_sidecar_path(artifact_path);
    let content = std::fs::read_to_string(&sidecar).ok()?;
    serde_json::from_str(&content).ok()
}

fn read_packet_metadata_sidecar(artifact_path: &Path) -> Option<LineageRef> {
    let metadata = read_packet_metadata_sidecar_metadata(artifact_path)?;
    metadata.lineage
}

fn read_packet_metadata_sidecar_metadata(artifact_path: &Path) -> Option<CanonSurfaceMetadata> {
    let sidecar = packet_metadata_sidecar_path(artifact_path);
    let content = std::fs::read_to_string(&sidecar).ok()?;
    serde_json::from_str(&content).ok()
}

/// Read Canon semantic sidecar metadata for one artifact when the packet metadata exists.
pub fn read_canon_semantic_artifact_surface(
    artifact_path: &Path,
) -> Option<CanonSemanticArtifactSurface> {
    let metadata = read_packet_metadata_sidecar_metadata(artifact_path)?;
    let promotion_view = metadata
        .lineage
        .as_ref()
        .map(PromotionStateView::from_lineage)
        .unwrap_or(PromotionStateView::Unknown);

    Some(CanonSemanticArtifactSurface {
        lineage: metadata.lineage,
        promotion_view,
        publication_target_class: normalized_metadata_value(
            metadata.publication_target_class.as_deref(),
        ),
        semantic_descriptor: metadata.semantic_descriptor,
    })
}

pub fn promote_governed_evidence_bundle(
    workspace_root: &Path,
    request: GovernedEvidencePromotionRequest<'_>,
) -> Result<Vec<PathBuf>, GovernedEvidencePromotionError> {
    let GovernedEvidencePromotionRequest {
        canon_mode,
        stage_key,
        run_ref,
        approval_state,
        packet_readiness,
        packet_ref,
        document_refs,
    } = request;
    let normalized_run_ref = run_ref.trim();
    if normalized_run_ref.is_empty() || document_refs.is_empty() {
        return Ok(Vec::new());
    }

    let source_artifacts = document_refs
        .iter()
        .filter_map(|document_ref| {
            Path::new(document_ref)
                .file_name()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if source_artifacts.is_empty() {
        return Ok(Vec::new());
    }

    let lineage = LineageRef {
        contract_version: CANON_LINEAGE_CONTRACT_VERSION.to_string(),
        producer: default_canon_producer(),
        source_ref: format!("{CANON_SOURCE_REF_PREFIX}{normalized_run_ref}"),
        source_artifacts,
        mode: Some(canon_mode.to_string()),
        promotion_state: EVIDENCE_ONLY_PROMOTION_STATE.to_string(),
        approval_state: Some(approval_state_text(approval_state).to_string()),
        stage: (!stage_key.trim().is_empty()).then(|| stage_key.to_string()),
        owner: None,
        risk: None,
        zone: None,
        promoted_at: String::new(),
        content_digest: String::new(),
        packet_readiness: Some(packet_readiness_text(packet_readiness).to_string()),
        promotion_profile: Some(EVIDENCE_PROMOTION_PROFILE.to_string()),
    };
    let evidence_root = evidence_root_for_lineage(workspace_root, &lineage);
    std::fs::create_dir_all(&evidence_root).map_err(|source| {
        GovernedEvidencePromotionError::CreateEvidenceRoot { path: evidence_root.clone(), source }
    })?;

    let packet_metadata = read_packet_promotion_metadata(workspace_root, packet_ref)?;
    let evidence_metadata = CanonSurfaceMetadata {
        lineage: Some(lineage),
        publication_target_class: packet_metadata
            .as_ref()
            .and_then(|metadata| metadata.publication_target_class.clone()),
        semantic_descriptor: packet_metadata
            .as_ref()
            .and_then(|metadata| metadata.semantic_descriptor.clone()),
        expertise_input: packet_metadata.and_then(|metadata| metadata.expertise_input),
    };

    let mut promoted_refs = Vec::new();
    for document_ref in document_refs {
        let source_path = workspace_root.join(document_ref);
        if !source_path.is_file() {
            continue;
        }

        let Some(file_name) = source_path.file_name() else {
            continue;
        };
        let target_path = evidence_root.join(file_name);
        std::fs::copy(&source_path, &target_path).map_err(|source| {
            GovernedEvidencePromotionError::CopyArtifact {
                source_path: source_path.clone(),
                target_path: target_path.clone(),
                source,
            }
        })?;

        let metadata_path = packet_metadata_sidecar_path(&target_path);
        let metadata_json = serde_json::to_string_pretty(&evidence_metadata).map_err(|source| {
            GovernedEvidencePromotionError::SerializeEvidenceMetadata {
                path: metadata_path.clone(),
                source,
            }
        })?;
        std::fs::write(&metadata_path, metadata_json).map_err(|source| {
            GovernedEvidencePromotionError::WriteEvidenceMetadata {
                path: metadata_path.clone(),
                source,
            }
        })?;

        promoted_refs.push(
            target_path.strip_prefix(workspace_root).unwrap_or(target_path.as_path()).to_path_buf(),
        );
    }

    Ok(promoted_refs)
}

fn read_packet_promotion_metadata(
    workspace_root: &Path,
    packet_ref: &str,
) -> Result<Option<CanonPacketPromotionMetadata>, GovernedEvidencePromotionError> {
    let normalized_packet_ref = packet_ref.trim();
    if normalized_packet_ref.is_empty() {
        return Ok(None);
    }

    let metadata_path =
        workspace_root.join(normalized_packet_ref).join(CANON_PACKET_METADATA_FILE_NAME);
    if !metadata_path.is_file() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&metadata_path).map_err(|source| {
        GovernedEvidencePromotionError::ReadPacketMetadata { path: metadata_path.clone(), source }
    })?;
    let envelope: CanonPacketPromotionEnvelope =
        serde_json::from_str(&content).map_err(|source| {
            GovernedEvidencePromotionError::ParsePacketMetadata {
                path: metadata_path.clone(),
                source,
            }
        })?;

    Ok(Some(envelope.metadata))
}

fn approval_state_text(state: ApprovalState) -> &'static str {
    match state {
        ApprovalState::NotNeeded => "not_needed",
        ApprovalState::Requested => "requested",
        ApprovalState::Granted => "granted",
        ApprovalState::Rejected => "rejected",
        ApprovalState::Expired => "expired",
    }
}

fn packet_readiness_text(readiness: PacketReadiness) -> &'static str {
    match readiness {
        PacketReadiness::Pending => "pending",
        PacketReadiness::Incomplete => "incomplete",
        PacketReadiness::Reusable => "reusable",
        PacketReadiness::Rejected => "rejected",
    }
}

fn packet_metadata_sidecar_path(artifact_path: &Path) -> PathBuf {
    let stem = artifact_path.file_stem().and_then(|value| value.to_str()).unwrap_or("packet");
    artifact_path.with_file_name(format!("{stem}{PROFILE_METADATA_FILE_SUFFIX}"))
}

fn legacy_lineage_sidecar_path(artifact_path: &Path) -> PathBuf {
    let stem = artifact_path.file_stem().and_then(|value| value.to_str()).unwrap_or("packet");
    artifact_path.with_file_name(format!("{stem}{LEGACY_LINEAGE_FILE_SUFFIX}"))
}

pub fn evidence_root_for_lineage(workspace_root: &Path, lineage: &LineageRef) -> PathBuf {
    resolve_project_doc_roots(workspace_root)
        .unwrap_or_else(|_| ProjectDocRoots::default())
        .evidence_dir(workspace_root)
        .join(lineage.mode_name())
        .join(lineage.source_ref_leaf())
}

/// Read producer-attributed managed-block summaries from an evidence root.
pub fn evidence_contribution_summaries(workspace_root: &Path, lineage: &LineageRef) -> Vec<String> {
    let evidence_root = evidence_root_for_lineage(workspace_root, lineage);
    if !evidence_root.exists() {
        return Vec::new();
    }

    let mut summaries = BTreeSet::new();
    for markdown_path in markdown_files_under(&evidence_root) {
        let Ok(content) = std::fs::read_to_string(&markdown_path) else {
            continue;
        };
        let target = markdown_path
            .strip_prefix(workspace_root)
            .unwrap_or(markdown_path.as_path())
            .display()
            .to_string();
        for summary in managed_block_summaries_for_content(&content, &target) {
            summaries.insert(summary);
        }
    }

    summaries.into_iter().collect()
}

fn markdown_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_files(root, &mut files);
    files
}

fn collect_markdown_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, files);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
}

fn managed_block_summaries_for_content(content: &str, target: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.contains(MANAGED_BLOCK_START_MARKER))
        .filter_map(|line| {
            let producer = managed_block_attribute(line, "producer")?;
            let source_ref = managed_block_attribute(line, "source_ref")?;
            let contract_version = managed_block_attribute(line, "contract_version");
            Some(match contract_version {
                Some(contract_version) => format!(
                    "producer={producer} source_ref={source_ref} contract_version={contract_version} target={target}"
                ),
                None => format!("producer={producer} source_ref={source_ref} target={target}"),
            })
        })
        .collect()
}

fn managed_block_attribute(line: &str, attribute: &str) -> Option<String> {
    let marker = format!("{attribute}=\"");
    let start = line.find(&marker)? + marker.len();
    let remainder = &line[start..];
    let end = remainder.find('"')?;
    let value = remainder[..end].trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn surface_category(surface_path: &Path) -> String {
    surface_path.file_stem().map(|value| value.to_string_lossy().into_owned()).unwrap_or_default()
}

fn collect_governed_expertise_input(
    workspace_root: &Path,
    surface_path: &Path,
    inputs: &mut Vec<GovernedExpertiseInputSurface>,
) {
    let Some(metadata) = read_packet_metadata_sidecar_metadata(surface_path) else {
        return;
    };
    let Some(lineage) = metadata.lineage else {
        return;
    };
    let Some(mut expertise_input) = metadata.expertise_input else {
        return;
    };

    let Some(expertise_kind) = normalized_metadata_value(Some(&expertise_input.expertise_kind))
    else {
        return;
    };
    let domain_families = expertise_input
        .domain_families
        .into_iter()
        .filter_map(|family| normalized_metadata_value(Some(&family)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if domain_families.is_empty() {
        return;
    }

    expertise_input.expertise_kind = expertise_kind;
    expertise_input.domain_families = domain_families;

    inputs.push(GovernedExpertiseInputSurface {
        path: surface_path.strip_prefix(workspace_root).unwrap_or(surface_path).to_path_buf(),
        promotion_view: PromotionStateView::from_lineage(&lineage),
        publication_target_class: normalized_metadata_value(
            metadata.publication_target_class.as_deref(),
        ),
        lineage,
        expertise_input,
    });
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        CompatibilityOutcome, LineageRef, ProjectMemoryCondition, ProjectMemoryContext,
        ProjectMemoryDecision, ProjectMemoryStatus, ProjectMemorySurface, PromotionStateView,
        evidence_contribution_summaries, evidence_root_for_lineage, legacy_lineage_sidecar_path,
        managed_block_attribute, merge_compatibility, packet_metadata_sidecar_path,
        parse_contract_version, read_lineage_sidecar, read_project_memory, surface_category,
    };

    fn sample_lineage_ref(
        source_ref: &str,
        mode: &str,
        promotion_state: &str,
        approval_state: Option<&str>,
        packet_readiness: Option<&str>,
    ) -> LineageRef {
        LineageRef {
            contract_version: "v1".into(),
            producer: "canon".into(),
            source_ref: source_ref.into(),
            source_artifacts: vec!["architecture-overview.md".into()],
            mode: Some(mode.into()),
            promotion_state: promotion_state.into(),
            approval_state: approval_state.map(str::to_string),
            stage: Some(mode.into()),
            owner: Some("Owner <owner@example.com>".into()),
            risk: Some("bounded-impact".into()),
            zone: Some("yellow".into()),
            promoted_at: "2026-05-13T14:30:00Z".into(),
            content_digest: "sha256:abc123".into(),
            packet_readiness: packet_readiness.map(str::to_string),
            promotion_profile: Some("project-memory".into()),
        }
    }

    #[test]
    fn promotion_state_view_display_and_helper_branches() {
        assert_eq!(PromotionStateView::Stable.to_string(), "stable");
        assert_eq!(PromotionStateView::PendingOrIndex.to_string(), "pending-or-index");
        assert_eq!(PromotionStateView::EvidenceOnly.to_string(), "evidence-only");
        assert_eq!(PromotionStateView::Manual.to_string(), "manual");
        assert_eq!(PromotionStateView::Unknown.to_string(), "unknown");

        assert_eq!(parse_contract_version("0"), Some((0, None)));
        assert_eq!(parse_contract_version("v1"), Some((1, None)));
        assert_eq!(parse_contract_version("v1.7"), Some((1, Some(7))));
        assert_eq!(parse_contract_version("0.invalid"), None);
        assert_eq!(
            merge_compatibility(
                Some(CompatibilityOutcome::Unsupported),
                CompatibilityOutcome::Compatible,
            ),
            Some(CompatibilityOutcome::Unsupported)
        );

        assert!(
            packet_metadata_sidecar_path(Path::new("/")).ends_with("packet.packet-metadata.json")
        );
        assert!(legacy_lineage_sidecar_path(Path::new("/")).ends_with("packet.lineage.json"));
        assert_eq!(surface_category(Path::new("/tmp/architecture-map.md")), "architecture-map");

        let lineage = sample_lineage_ref(
            "canon-run:run-1",
            "architecture",
            "pending-index",
            None,
            Some("partial"),
        );
        assert_eq!(
            evidence_root_for_lineage(Path::new("/tmp/workspace"), &lineage),
            PathBuf::from("/tmp/workspace/docs/evidence/architecture/run-1")
        );
        assert_eq!(
            managed_block_attribute(
                r#"<!-- project-memory:managed:start producer="canon" source_ref="run-1" contract_version="v1" -->"#,
                "producer"
            )
            .as_deref(),
            Some("canon")
        );
    }

    #[test]
    fn promotion_state_view_from_canon_state() {
        assert_eq!(PromotionStateView::from_canon_state("auto"), PromotionStateView::Stable);
        assert_eq!(
            PromotionStateView::from_canon_state("auto-if-approved"),
            PromotionStateView::Unknown
        );
        assert_eq!(
            PromotionStateView::from_canon_state("pending-index"),
            PromotionStateView::PendingOrIndex
        );
        assert_eq!(
            PromotionStateView::from_canon_state("index-only"),
            PromotionStateView::PendingOrIndex
        );
        assert_eq!(
            PromotionStateView::from_canon_state("evidence-only"),
            PromotionStateView::EvidenceOnly
        );
        assert_eq!(PromotionStateView::from_canon_state("manual"), PromotionStateView::Manual);
        assert_eq!(
            PromotionStateView::from_canon_state("unknown-state"),
            PromotionStateView::Unknown
        );
    }

    #[test]
    fn promotion_state_view_from_lineage_requires_approval_metadata() {
        let stable = sample_lineage_ref(
            "canon-run:run-stable",
            "architecture",
            "auto-if-approved",
            Some("Completed"),
            Some("complete"),
        );
        let pending = sample_lineage_ref(
            "canon-run:run-pending",
            "architecture",
            "auto-if-approved",
            Some("AwaitingApproval"),
            Some("partial"),
        );
        let incomplete = sample_lineage_ref(
            "canon-run:run-unknown",
            "architecture",
            "auto-if-approved",
            None,
            Some("complete"),
        );

        assert_eq!(PromotionStateView::from_lineage(&stable), PromotionStateView::Stable);
        assert_eq!(PromotionStateView::from_lineage(&pending), PromotionStateView::PendingOrIndex);
        assert_eq!(PromotionStateView::from_lineage(&incomplete), PromotionStateView::Unknown);

        let granted = sample_lineage_ref(
            "canon-run:run-granted",
            "architecture",
            "auto-if-approved",
            Some("granted"),
            Some("reusable"),
        );
        assert_eq!(PromotionStateView::from_lineage(&granted), PromotionStateView::Stable);
    }

    #[test]
    fn promotion_state_view_credibility() {
        assert!(PromotionStateView::Stable.is_credible());
        assert!(!PromotionStateView::PendingOrIndex.is_credible());
        assert!(!PromotionStateView::EvidenceOnly.is_credible());
        assert!(!PromotionStateView::Manual.is_credible());
        assert!(!PromotionStateView::Unknown.is_credible());
    }

    #[test]
    fn project_memory_condition_classifies_warning_and_hard_stop_states() {
        assert_eq!(ProjectMemoryDecision::Proceed.as_str(), "proceed");
        assert_eq!(ProjectMemoryDecision::Warning.as_str(), "warning");
        assert_eq!(ProjectMemoryDecision::HardStop.as_str(), "hard-stop");

        assert_eq!(ProjectMemoryCondition::Stable.decision(), ProjectMemoryDecision::Proceed);
        assert_eq!(ProjectMemoryCondition::Pending.decision(), ProjectMemoryDecision::Warning);
        assert_eq!(
            ProjectMemoryCondition::MissingRequiredApproval.decision(),
            ProjectMemoryDecision::HardStop
        );
        assert_eq!(
            ProjectMemoryCondition::UnsupportedContract.decision(),
            ProjectMemoryDecision::HardStop
        );
        assert_eq!(
            ProjectMemoryCondition::BlockedGovernance.reason_code(),
            Some("project_memory_blocked")
        );
        assert_eq!(
            ProjectMemoryCondition::MissingRequiredSourceArtifacts.reason_code(),
            Some("project_memory_missing_source_artifacts")
        );
        assert_eq!(
            ProjectMemoryCondition::UnsupportedContract.reason_code(),
            Some("project_memory_contract_incompatible")
        );
        assert_eq!(
            ProjectMemoryCondition::IncompleteMetadata.headline(),
            "Canon project memory metadata is incomplete"
        );
    }

    #[test]
    fn compatibility_outcome_check() {
        assert_eq!(CompatibilityOutcome::check("v1"), CompatibilityOutcome::Compatible);
        assert_eq!(CompatibilityOutcome::check("v1.7"), CompatibilityOutcome::Compatible);
        assert_eq!(CompatibilityOutcome::check("0.1.0"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("0.2.0"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("1.0.0"), CompatibilityOutcome::Compatible);
        assert_eq!(CompatibilityOutcome::check("v2"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("invalid"), CompatibilityOutcome::Unsupported);
    }

    #[test]
    fn lineage_ref_serde_round_trip() {
        let lineage = sample_lineage_ref(
            "canon-run:019738a4-test",
            "architecture",
            "auto",
            Some("Completed"),
            Some("complete"),
        );
        let json = serde_json::to_string(&lineage).unwrap();
        let back: LineageRef = serde_json::from_str(&json).unwrap();
        assert_eq!(lineage, back);
    }

    #[test]
    fn project_memory_context_absent() {
        let ctx = ProjectMemoryContext::absent();
        assert_eq!(ctx.status, ProjectMemoryStatus::Absent);
        assert!(!ctx.has_credible_memory());
    }

    #[test]
    fn project_memory_context_incompatible() {
        let ctx = ProjectMemoryContext::incompatible();
        assert_eq!(ctx.status, ProjectMemoryStatus::Incompatible);
        assert_eq!(ctx.compatibility, Some(CompatibilityOutcome::Unsupported));
        assert_eq!(ctx.condition(), Some(ProjectMemoryCondition::UnsupportedContract));
        assert_eq!(ctx.decision(), Some(ProjectMemoryDecision::HardStop));
        assert!(!ctx.has_credible_memory());
    }

    #[test]
    fn project_memory_context_with_stable_surface() {
        let ctx = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: "docs/project/architecture-map.md".into(),
                lineage: Some(sample_lineage_ref(
                    "canon-run:run-1",
                    "architecture",
                    "auto",
                    Some("Completed"),
                    Some("complete"),
                )),
                promotion_view: PromotionStateView::Stable,
                category: "architecture-map".into(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(PromotionStateView::Stable),
        };
        assert_eq!(ctx.condition(), Some(ProjectMemoryCondition::Stable));
        assert_eq!(ctx.decision(), Some(ProjectMemoryDecision::Proceed));
        assert!(ctx.has_credible_memory());
    }

    #[test]
    fn project_memory_context_classifies_missing_required_approval() {
        let ctx = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: "docs/project/architecture-map.md".into(),
                lineage: Some(sample_lineage_ref(
                    "canon-run:run-awaiting-approval",
                    "architecture",
                    "auto-if-approved",
                    Some("requested"),
                    Some("pending"),
                )),
                promotion_view: PromotionStateView::PendingOrIndex,
                category: "architecture-map".into(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(PromotionStateView::PendingOrIndex),
        };

        assert_eq!(ctx.condition(), Some(ProjectMemoryCondition::MissingRequiredApproval));
        assert_eq!(ctx.decision(), Some(ProjectMemoryDecision::HardStop));
    }

    #[test]
    fn project_memory_context_classifies_blocked_governance() {
        let ctx = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: "docs/project/architecture-map.md".into(),
                lineage: Some(sample_lineage_ref(
                    "canon-run:run-blocked",
                    "architecture",
                    "auto-if-approved",
                    Some("rejected"),
                    Some("rejected"),
                )),
                promotion_view: PromotionStateView::PendingOrIndex,
                category: "architecture-map".into(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(PromotionStateView::PendingOrIndex),
        };

        assert_eq!(ctx.condition(), Some(ProjectMemoryCondition::BlockedGovernance));
        assert_eq!(ctx.decision(), Some(ProjectMemoryDecision::HardStop));
    }

    #[test]
    fn project_memory_context_requires_ready_packet_after_approval() {
        let temp = std::env::temp_dir().join("boundline-test-pm-approved-packet-not-ready");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();

        let mut lineage = sample_lineage_ref(
            "canon-run:run-approved-not-ready",
            "architecture",
            "auto-if-approved",
            Some("Completed"),
            Some("partial"),
        );
        lineage.source_artifacts.clear();

        let ctx = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: "docs/project/architecture-map.md".into(),
                lineage: Some(lineage),
                promotion_view: PromotionStateView::Stable,
                category: "architecture-map".into(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(PromotionStateView::Stable),
        };

        assert_eq!(
            ctx.condition_for_workspace(&temp),
            Some(ProjectMemoryCondition::MissingRequiredSourceArtifacts)
        );
        assert_eq!(ctx.decision_for_workspace(&temp), Some(ProjectMemoryDecision::HardStop));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_absent_workspace() {
        let temp = std::env::temp_dir().join("boundline-test-pm-absent");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();
        let ctx = read_project_memory(&temp);
        assert_eq!(ctx.status, ProjectMemoryStatus::Absent);
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_with_surfaces() {
        let temp = std::env::temp_dir().join("boundline-test-pm-surfaces");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create a project memory surface
        std::fs::write(
            project_dir.join("architecture-map.md"),
            "# Architecture Map\n\nContent here.",
        )
        .unwrap();

        // Create a Canon packet-metadata sidecar.
        let lineage = sample_lineage_ref(
            "canon-run:run-123",
            "architecture",
            "auto-if-approved",
            Some("Completed"),
            Some("complete"),
        );
        std::fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-123",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "profile": "project-memory",
                "promotion_state": "auto",
                "update_strategy": "managed-blocks",
                "lineage": lineage,
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);
        assert_eq!(ctx.status, ProjectMemoryStatus::Available);
        assert_eq!(ctx.compatibility, Some(CompatibilityOutcome::Compatible));
        assert_eq!(ctx.surfaces.len(), 1);
        assert_eq!(ctx.effective_promotion_state, Some(PromotionStateView::Stable));
        assert!(ctx.has_credible_memory());
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_uses_project_index_doc_overrides() {
        let temp = std::env::temp_dir().join("boundline-test-pm-doc-overrides");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("knowledge/project");
        let evidence_dir = temp.join("knowledge/evidence/architecture/run-123");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&evidence_dir).unwrap();
        std::fs::write(
            temp.join("project.boundline.toml"),
            "[project]\nname = \"boundline\"\n\n[docs]\nproject_memory = \"knowledge/project\"\nevidence = \"knowledge/evidence\"\n",
        )
        .unwrap();
        std::fs::write(project_dir.join("architecture-map.md"), "# Architecture Map\n").unwrap();
        std::fs::write(evidence_dir.join("architecture-overview.md"), "overview\n").unwrap();
        std::fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "lineage": {
                    "contract_version": "v1",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "mode": "architecture",
                    "promotion_state": "auto",
                    "approval_state": "Completed",
                    "packet_readiness": "complete",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);

        assert_eq!(ctx.status, ProjectMemoryStatus::Available);
        assert_eq!(ctx.surfaces.len(), 1);
        assert_eq!(ctx.surfaces[0].path, PathBuf::from("knowledge/project/architecture-map.md"));
        assert_eq!(ctx.evidence_refs.len(), 1);
        assert_eq!(
            evidence_root_for_lineage(&temp, ctx.evidence_refs.first().unwrap()),
            temp.join("knowledge/evidence/architecture/run-123")
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_lineage_sidecar_prefers_packet_metadata_then_falls_back_to_legacy() {
        let temp = std::env::temp_dir().join("boundline-test-pm-sidecar-fallback");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("decision-map.md");
        std::fs::write(&artifact, "# Decision Map\n").unwrap();

        let packet_lineage = sample_lineage_ref(
            "canon-run:packet-run",
            "architecture",
            "auto",
            Some("Completed"),
            Some("complete"),
        );

        std::fs::write(
            packet_metadata_sidecar_path(&artifact),
            serde_json::to_string_pretty(&serde_json::json!({ "lineage": packet_lineage }))
                .unwrap(),
        )
        .unwrap();
        std::fs::write(
            legacy_lineage_sidecar_path(&artifact),
            serde_json::to_string_pretty(&serde_json::json!({
                "contract_version": "v1",
                "source_run": "legacy-run",
                "mode": "architecture",
                "promotion_state": "pending-index",
                "readiness": "partial"
            }))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            read_lineage_sidecar(&artifact).as_ref().map(|lineage| lineage.source_ref.as_str()),
            Some("canon-run:packet-run")
        );

        std::fs::remove_file(packet_metadata_sidecar_path(&artifact)).unwrap();

        assert_eq!(
            read_lineage_sidecar(&artifact).as_ref().map(|lineage| lineage.source_ref.as_str()),
            Some("legacy-run")
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_handles_additive_surface_with_legacy_lineage() {
        let temp = std::env::temp_dir().join("boundline-test-pm-additive-legacy");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        let evidence_dir = temp.join("docs/evidence/architecture/run-additive");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::create_dir_all(&evidence_dir).unwrap();

        let artifact = project_dir.join("custom-map.md");
        std::fs::write(&artifact, "# Custom Map\n").unwrap();
        std::fs::write(evidence_dir.join("summary.md"), "summary\n").unwrap();
        std::fs::write(
            legacy_lineage_sidecar_path(&artifact),
            serde_json::to_string_pretty(&serde_json::json!({
                "contract_version": "v1",
                "source_run": "run-additive",
                "mode": "architecture",
                "promotion_state": "pending-index",
                "readiness": "partial"
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);

        assert_eq!(ctx.status, ProjectMemoryStatus::Available);
        assert_eq!(ctx.compatibility, Some(CompatibilityOutcome::Compatible));
        assert_eq!(ctx.surfaces.len(), 1);
        assert_eq!(ctx.surfaces[0].category, "custom-map");
        assert_eq!(ctx.effective_promotion_state, Some(PromotionStateView::PendingOrIndex));
        assert_eq!(ctx.evidence_refs.len(), 1);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_skips_unlineaged_additive_surface_and_non_markdown_entries() {
        let temp = std::env::temp_dir().join("boundline-test-pm-additive-skip");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(project_dir.join("custom-map.md"), "# Custom Map\n").unwrap();
        std::fs::write(project_dir.join("notes.txt"), "ignore me\n").unwrap();

        let ctx = read_project_memory(&temp);

        assert_eq!(ctx.status, ProjectMemoryStatus::Absent);
        assert!(ctx.surfaces.is_empty());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_rejects_unsupported_additive_surface() {
        let temp = std::env::temp_dir().join("boundline-test-pm-additive-unsupported");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("custom-map.md");
        std::fs::write(&artifact, "# Custom Map\n").unwrap();
        std::fs::write(
            legacy_lineage_sidecar_path(&artifact),
            serde_json::to_string_pretty(&serde_json::json!({
                "contract_version": "v2",
                "source_run": "run-additive",
                "mode": "architecture",
                "promotion_state": "auto",
                "approval_state": "Completed",
                "readiness": "complete"
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);

        assert_eq!(ctx.status, ProjectMemoryStatus::Incompatible);

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_allows_missing_profile_metadata() {
        let temp = std::env::temp_dir().join("boundline-test-pm-missing-profile");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(
            project_dir.join("architecture-map.md"),
            "# Architecture Map\n\nContent here.",
        )
        .unwrap();

        std::fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-123",
                "mode": "architecture",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "architecture-map",
                "destination": "docs/project/architecture-map.md",
                "source_artifacts": ["architecture-overview.md"],
                "promotion_state": "auto",
                "update_strategy": "managed-blocks",
                "lineage": {
                    "contract_version": "v1",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "mode": "architecture",
                    "promotion_state": "auto",
                    "approval_state": "Completed",
                    "packet_readiness": "complete",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);
        assert_eq!(ctx.status, ProjectMemoryStatus::Available);
        assert_eq!(ctx.surfaces.len(), 1);
        assert_eq!(
            ctx.surfaces[0]
                .lineage
                .as_ref()
                .and_then(|lineage| lineage.promotion_profile.as_deref()),
            None
        );
        assert_eq!(ctx.effective_promotion_state, Some(PromotionStateView::Stable));
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_unsupported_version() {
        let temp = std::env::temp_dir().join("boundline-test-pm-unsupported");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(project_dir.join("overview.md"), "# Overview").unwrap();
        let mut lineage = sample_lineage_ref(
            "canon-run:run-456",
            "requirements",
            "auto",
            Some("Completed"),
            Some("complete"),
        );
        lineage.contract_version = "v2".into();
        lineage.source_artifacts = vec!["prd.md".into()];
        std::fs::write(
            project_dir.join("overview.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": "run-456",
                "mode": "requirements",
                "risk": "bounded-impact",
                "zone": "yellow",
                "publish_timestamp": "2026-05-13T14:30:00Z",
                "descriptor": "overview",
                "destination": "docs/project/overview.md",
                "source_artifacts": ["prd.md"],
                "profile": "project-memory",
                "promotion_state": "auto",
                "update_strategy": "managed-blocks",
                "lineage": lineage,
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);
        assert_eq!(ctx.status, ProjectMemoryStatus::Incompatible);
        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn read_project_memory_marks_missing_required_source_artifacts_as_hard_stop() {
        let temp = std::env::temp_dir().join("boundline-test-pm-missing-required-artifacts");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        std::fs::write(project_dir.join("architecture-map.md"), "# Architecture Map\n").unwrap();
        std::fs::write(
            project_dir.join("architecture-map.packet-metadata.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "lineage": {
                    "contract_version": "v1",
                    "producer": "canon",
                    "source_ref": "canon-run:run-123",
                    "mode": "architecture",
                    "promotion_state": "auto",
                    "approval_state": "Completed",
                    "packet_readiness": "reusable",
                    "promoted_at": "2026-05-13T14:30:00Z",
                    "content_digest": "sha256:abc123",
                    "source_artifacts": ["architecture-overview.md"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let ctx = read_project_memory(&temp);

        assert_eq!(
            ctx.condition_for_workspace(&temp),
            Some(ProjectMemoryCondition::MissingRequiredSourceArtifacts)
        );
        assert_eq!(ctx.decision_for_workspace(&temp), Some(ProjectMemoryDecision::HardStop));

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn evidence_contribution_summaries_preserve_managed_block_attribution() {
        let temp = std::env::temp_dir().join("boundline-test-pm-managed-block-attribution");
        let _ = std::fs::remove_dir_all(&temp);
        let evidence_dir = temp.join("docs/evidence/architecture/run-123");
        std::fs::create_dir_all(&evidence_dir).unwrap();
        std::fs::write(
            evidence_dir.join("verification.md"),
            concat!(
                "<!-- project-memory:managed:start producer=\"canon\" source_ref=\"canon-run:run-123\" contract_version=\"v1\" -->\n",
                "Canon-managed evidence\n",
                "<!-- project-memory:managed:end -->\n",
                "\n",
                "<!-- project-memory:managed:start producer=\"boundline\" source_ref=\"trace-7\" contract_version=\"v1\" -->\n",
                "Boundline-managed evidence\n",
                "<!-- project-memory:managed:end -->\n"
            ),
        )
        .unwrap();

        let summaries = evidence_contribution_summaries(
            &temp,
            &sample_lineage_ref(
                "canon-run:run-123",
                "architecture",
                "auto",
                Some("Completed"),
                Some("reusable"),
            ),
        );

        assert_eq!(summaries.len(), 2);
        assert!(summaries.iter().any(|summary| summary.contains("producer=canon")));
        assert!(summaries.iter().any(|summary| summary.contains("producer=boundline")));
        assert!(summaries.iter().all(|summary| {
            summary.contains("target=docs/evidence/architecture/run-123/verification.md")
        }));

        let _ = std::fs::remove_dir_all(&temp);
    }
}
