use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Supported contract line for this Boundline integration slice.
const SUPPORTED_CONTRACT_MAJOR: u64 = 0;
const SUPPORTED_CONTRACT_MINOR: u64 = 1;
const PROFILE_METADATA_FILE_SUFFIX: &str = ".packet-metadata.json";
const LEGACY_LINEAGE_FILE_SUFFIX: &str = ".lineage.json";
const CANON_PROJECT_SURFACES: [&str; 11] = [
    "docs/project/overview.md",
    "docs/project/product-context.md",
    "docs/project/domain-language.md",
    "docs/project/domain-model.md",
    "docs/project/architecture-map.md",
    "docs/project/decision-index.md",
    "docs/project/delivery-map.md",
    "docs/project/operational-context.md",
    "docs/project/pending-decisions.md",
    "docs/project/open-risks.md",
    "docs/project/audit-log.md",
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
                match (lineage.approval_state.as_deref(), lineage.readiness.as_str()) {
                    (Some("Completed"), "complete") => Self::Stable,
                    (Some(_), _) => Self::PendingOrIndex,
                    (None, _) => Self::Unknown,
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
    pub source_run: String,
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    pub promotion_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_state: Option<String>,
    pub readiness: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_artifacts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CanonSurfaceMetadata {
    #[serde(default)]
    lineage: Option<LineageRef>,
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
            Some((SUPPORTED_CONTRACT_MAJOR, SUPPORTED_CONTRACT_MINOR)) => Self::Compatible,
            Some(_) => Self::Unsupported,
            None => Self::Unsupported,
        }
    }
}

fn parse_contract_version(contract_version: &str) -> Option<(u64, u64)> {
    let mut segments = contract_version.split('.');
    let major = segments.next()?.parse::<u64>().ok()?;
    let minor = segments.next().unwrap_or("0").parse::<u64>().ok()?;
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
}

/// Read Canon-promoted project-memory surfaces from a workspace.
///
/// Reads Canon's named `docs/project/*.md` surfaces and their adjacent
/// `<surface>.packet-metadata.json` sidecars. Legacy `*.lineage.json`
/// sidecars remain tolerated for compatibility with earlier fixtures.
pub fn read_project_memory(workspace_root: &std::path::Path) -> ProjectMemoryContext {
    let project_dir = workspace_root.join("docs/project");
    let evidence_dir = workspace_root.join("docs/evidence");

    if !project_dir.exists() && !evidence_dir.exists() {
        return ProjectMemoryContext::absent();
    }

    let mut surfaces = Vec::new();
    let mut evidence_refs = Vec::new();
    let mut best_state: Option<PromotionStateView> = None;
    let mut compatibility: Option<CompatibilityOutcome> = None;
    let mut discovered_paths = BTreeSet::new();

    for relative_path in CANON_PROJECT_SURFACES {
        let path = workspace_root.join(relative_path);
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
    let sidecar = packet_metadata_sidecar_path(artifact_path);
    let content = std::fs::read_to_string(&sidecar).ok()?;
    let metadata: CanonSurfaceMetadata = serde_json::from_str(&content).ok()?;
    metadata.lineage
}

fn packet_metadata_sidecar_path(artifact_path: &Path) -> PathBuf {
    let stem = artifact_path.file_stem().and_then(|value| value.to_str()).unwrap_or("packet");
    artifact_path.with_file_name(format!("{stem}{PROFILE_METADATA_FILE_SUFFIX}"))
}

fn legacy_lineage_sidecar_path(artifact_path: &Path) -> PathBuf {
    let stem = artifact_path.file_stem().and_then(|value| value.to_str()).unwrap_or("packet");
    artifact_path.with_file_name(format!("{stem}{LEGACY_LINEAGE_FILE_SUFFIX}"))
}

fn evidence_root_for_lineage(workspace_root: &Path, lineage: &LineageRef) -> PathBuf {
    workspace_root.join("docs/evidence").join(&lineage.mode).join(&lineage.source_run)
}

fn surface_category(surface_path: &Path) -> String {
    surface_path.file_stem().map(|value| value.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotion_state_view_display_and_helper_branches() {
        assert_eq!(PromotionStateView::Stable.to_string(), "stable");
        assert_eq!(PromotionStateView::PendingOrIndex.to_string(), "pending-or-index");
        assert_eq!(PromotionStateView::EvidenceOnly.to_string(), "evidence-only");
        assert_eq!(PromotionStateView::Manual.to_string(), "manual");
        assert_eq!(PromotionStateView::Unknown.to_string(), "unknown");

        assert_eq!(parse_contract_version("0"), Some((0, 0)));
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

        let lineage = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "run-1".into(),
            mode: "architecture".into(),
            profile: None,
            promotion_state: "pending-index".into(),
            approval_state: None,
            readiness: "partial".into(),
            published_at: None,
            update_strategy: None,
            source_artifacts: Vec::new(),
        };
        assert_eq!(
            evidence_root_for_lineage(Path::new("/tmp/workspace"), &lineage),
            PathBuf::from("/tmp/workspace/docs/evidence/architecture/run-1")
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
        let stable = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "run-stable".into(),
            mode: "architecture".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto-if-approved".into(),
            approval_state: Some("Completed".into()),
            readiness: "complete".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["architecture-overview.md".into()],
        };
        let pending = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "run-pending".into(),
            mode: "architecture".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto-if-approved".into(),
            approval_state: Some("AwaitingApproval".into()),
            readiness: "partial".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["architecture-overview.md".into()],
        };
        let incomplete = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "run-unknown".into(),
            mode: "architecture".into(),
            profile: None,
            promotion_state: "auto-if-approved".into(),
            approval_state: None,
            readiness: "complete".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["architecture-overview.md".into()],
        };

        assert_eq!(PromotionStateView::from_lineage(&stable), PromotionStateView::Stable);
        assert_eq!(PromotionStateView::from_lineage(&pending), PromotionStateView::PendingOrIndex);
        assert_eq!(PromotionStateView::from_lineage(&incomplete), PromotionStateView::Unknown);
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
    fn compatibility_outcome_check() {
        assert_eq!(CompatibilityOutcome::check("0.1.0"), CompatibilityOutcome::Compatible);
        assert_eq!(CompatibilityOutcome::check("0.1.7"), CompatibilityOutcome::Compatible);
        assert_eq!(CompatibilityOutcome::check("0.2.0"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("1.0.0"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("2.0.0"), CompatibilityOutcome::Unsupported);
        assert_eq!(CompatibilityOutcome::check("invalid"), CompatibilityOutcome::Unsupported);
    }

    #[test]
    fn lineage_ref_serde_round_trip() {
        let lineage = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "019738a4-test".into(),
            mode: "architecture".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto".into(),
            approval_state: Some("Completed".into()),
            readiness: "stable".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["architecture-overview.md".into()],
        };
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
        assert!(!ctx.has_credible_memory());
    }

    #[test]
    fn project_memory_context_with_stable_surface() {
        let ctx = ProjectMemoryContext {
            status: ProjectMemoryStatus::Available,
            compatibility: Some(CompatibilityOutcome::Compatible),
            surfaces: vec![ProjectMemorySurface {
                path: "docs/project/architecture-map.md".into(),
                lineage: Some(LineageRef {
                    contract_version: "0.1.0".into(),
                    source_run: "run-1".into(),
                    mode: "architecture".into(),
                    profile: Some("project-memory".into()),
                    promotion_state: "auto".into(),
                    approval_state: Some("Completed".into()),
                    readiness: "stable".into(),
                    published_at: Some("2026-05-13T14:30:00Z".into()),
                    update_strategy: Some("managed-blocks".into()),
                    source_artifacts: vec!["architecture-overview.md".into()],
                }),
                promotion_view: PromotionStateView::Stable,
                category: "architecture-map".into(),
            }],
            evidence_refs: Vec::new(),
            effective_promotion_state: Some(PromotionStateView::Stable),
        };
        assert!(ctx.has_credible_memory());
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
        let lineage = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "run-123".into(),
            mode: "architecture".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto-if-approved".into(),
            approval_state: Some("Completed".into()),
            readiness: "complete".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["architecture-overview.md".into()],
        };
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
    fn read_lineage_sidecar_prefers_packet_metadata_then_falls_back_to_legacy() {
        let temp = std::env::temp_dir().join("boundline-test-pm-sidecar-fallback");
        let _ = std::fs::remove_dir_all(&temp);
        let project_dir = temp.join("docs/project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let artifact = project_dir.join("decision-map.md");
        std::fs::write(&artifact, "# Decision Map\n").unwrap();

        let packet_lineage = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "packet-run".into(),
            mode: "architecture".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto".into(),
            approval_state: Some("Completed".into()),
            readiness: "complete".into(),
            published_at: None,
            update_strategy: None,
            source_artifacts: Vec::new(),
        };
        let legacy_lineage = LineageRef {
            contract_version: "0.1.0".into(),
            source_run: "legacy-run".into(),
            mode: "architecture".into(),
            profile: None,
            promotion_state: "pending-index".into(),
            approval_state: None,
            readiness: "partial".into(),
            published_at: None,
            update_strategy: None,
            source_artifacts: Vec::new(),
        };

        std::fs::write(
            packet_metadata_sidecar_path(&artifact),
            serde_json::to_string_pretty(&serde_json::json!({ "lineage": packet_lineage }))
                .unwrap(),
        )
        .unwrap();
        std::fs::write(
            legacy_lineage_sidecar_path(&artifact),
            serde_json::to_string_pretty(&legacy_lineage).unwrap(),
        )
        .unwrap();

        assert_eq!(
            read_lineage_sidecar(&artifact).as_ref().map(|lineage| lineage.source_run.as_str()),
            Some("packet-run")
        );

        std::fs::remove_file(packet_metadata_sidecar_path(&artifact)).unwrap();

        assert_eq!(
            read_lineage_sidecar(&artifact).as_ref().map(|lineage| lineage.source_run.as_str()),
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
            serde_json::to_string_pretty(&LineageRef {
                contract_version: "0.1.0".into(),
                source_run: "run-additive".into(),
                mode: "architecture".into(),
                profile: None,
                promotion_state: "pending-index".into(),
                approval_state: None,
                readiness: "partial".into(),
                published_at: None,
                update_strategy: None,
                source_artifacts: Vec::new(),
            })
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
            serde_json::to_string_pretty(&LineageRef {
                contract_version: "0.2.0".into(),
                source_run: "run-additive".into(),
                mode: "architecture".into(),
                profile: None,
                promotion_state: "auto".into(),
                approval_state: Some("Completed".into()),
                readiness: "complete".into(),
                published_at: None,
                update_strategy: None,
                source_artifacts: Vec::new(),
            })
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
                    "contract_version": "0.1.0",
                    "source_run": "run-123",
                    "mode": "architecture",
                    "promotion_state": "auto",
                    "approval_state": "Completed",
                    "readiness": "complete",
                    "published_at": "2026-05-13T14:30:00Z",
                    "update_strategy": "managed-blocks",
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
            ctx.surfaces[0].lineage.as_ref().and_then(|lineage| lineage.profile.as_deref()),
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
        let lineage = LineageRef {
            contract_version: "2.0.0".into(),
            source_run: "run-456".into(),
            mode: "requirements".into(),
            profile: Some("project-memory".into()),
            promotion_state: "auto".into(),
            approval_state: Some("Completed".into()),
            readiness: "stable".into(),
            published_at: Some("2026-05-13T14:30:00Z".into()),
            update_strategy: Some("managed-blocks".into()),
            source_artifacts: vec!["prd.md".into()],
        };
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
}
