//! Discovery and validation helpers for directory-based guidance catalog packs.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::guidance::{
    CapabilityPhase, GuardianCapability, GuardianKind, GuidanceAuthoritySource, GuidanceCapability,
    SkippedCapabilitySource,
};
use crate::domain::guidance_catalog::{
    CatalogGuardianDisposition, CatalogGuardianIndex, CatalogGuidanceIndex, CatalogLifecycleLabel,
    CatalogManifest, CatalogPackManifest, CatalogValidationFinding, CatalogValidationSeverity,
};

const PACK_MANIFEST_FILE: &str = "pack.toml";
const CATALOG_MANIFEST_FILE: &str = "catalog/catalog-manifest.toml";
const GUIDANCE_INDEX_FILE: &str = "catalog/guidance-index.toml";
const GUARDIAN_INDEX_FILE: &str = "catalog/guardian-index.toml";

/// Loaded capabilities plus explicit pack-level outcomes from 055-style catalog
/// directories.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogPackDiscovery {
    pub guidance: Vec<GuidanceCapability>,
    pub guardians: Vec<GuardianCapability>,
    pub loaded_packs: Vec<String>,
    pub skipped_packs: Vec<String>,
    pub validation_findings: Vec<CatalogValidationFinding>,
    pub skipped_sources: Vec<SkippedCapabilitySource>,
    pub resolution_notes: Vec<String>,
}

impl CatalogPackDiscovery {
    fn merge(&mut self, other: Self) {
        self.guidance.extend(other.guidance);
        self.guardians.extend(other.guardians);
        self.loaded_packs.extend(other.loaded_packs);
        self.skipped_packs.extend(other.skipped_packs);
        self.validation_findings.extend(other.validation_findings);
        self.skipped_sources.extend(other.skipped_sources);
        self.resolution_notes.extend(other.resolution_notes);
    }
}

/// Discover every directory-based catalog pack under `packs_dir` for the given
/// runtime authority family and lifecycle phase.
pub fn discover_catalog_packs(
    packs_dir: &Path,
    display_root: &Path,
    authority_source: GuidanceAuthoritySource,
    phase: CapabilityPhase,
) -> CatalogPackDiscovery {
    let mut discovery = CatalogPackDiscovery::default();

    let Ok(entries) = fs::read_dir(packs_dir) else {
        return discovery;
    };

    let mut pack_dirs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    pack_dirs.sort();

    for pack_dir in pack_dirs {
        discovery.merge(discover_catalog_pack(&pack_dir, display_root, authority_source, phase));
    }

    discovery
}

fn discover_catalog_pack(
    pack_dir: &Path,
    display_root: &Path,
    authority_source: GuidanceAuthoritySource,
    phase: CapabilityPhase,
) -> CatalogPackDiscovery {
    let pack_ref = display_relative_path(display_root, pack_dir);
    let pack_manifest = match load_and_validate::<CatalogPackManifest>(
        pack_dir,
        PACK_MANIFEST_FILE,
        display_root,
        "catalog pack manifest",
        &pack_ref,
        authority_source,
    ) {
        Ok(manifest) => manifest,
        Err(discovery) => return *discovery,
    };

    let catalog_manifest = match load_and_validate::<CatalogManifest>(
        pack_dir,
        CATALOG_MANIFEST_FILE,
        display_root,
        "catalog manifest",
        &pack_ref,
        authority_source,
    ) {
        Ok(manifest) => manifest,
        Err(discovery) => return *discovery,
    };

    let guidance_index = match load_and_validate::<CatalogGuidanceIndex>(
        pack_dir,
        GUIDANCE_INDEX_FILE,
        display_root,
        "guidance index",
        &pack_ref,
        authority_source,
    ) {
        Ok(index) => index,
        Err(discovery) => return *discovery,
    };

    let guardian_index = match load_and_validate::<CatalogGuardianIndex>(
        pack_dir,
        GUARDIAN_INDEX_FILE,
        display_root,
        "guardian index",
        &pack_ref,
        authority_source,
    ) {
        Ok(index) => index,
        Err(discovery) => return *discovery,
    };

    let pack_id = pack_manifest.pack.id.clone();
    let catalog_id = catalog_manifest.catalog.id.clone();
    let default_catalog_authority = catalog_manifest.authority.default_source;
    let mut discovery = CatalogPackDiscovery {
        loaded_packs: vec![format!("{pack_ref} (pack={pack_id}, catalog={catalog_id})")],
        resolution_notes: vec![format!(
            "catalog pack {pack_id} loaded from {pack_ref} for {}",
            phase.as_str()
        )],
        ..CatalogPackDiscovery::default()
    };

    for (capability_id, entry) in guidance_index.guidance {
        if !matches_phase(&entry.applies_to, phase) {
            continue;
        }

        let content_path = pack_dir.join(&entry.path);
        let content_ref = display_relative_path(display_root, &content_path);
        if !content_path.is_file() {
            discovery.validation_findings.push(CatalogValidationFinding {
                severity: CatalogValidationSeverity::Warning,
                source_ref: content_ref.clone(),
                message: format!("missing guidance markdown for catalog entry {capability_id}"),
            });
            discovery.skipped_sources.push(SkippedCapabilitySource {
                source_ref: content_ref,
                authority_source,
                reason: format!("missing guidance markdown for catalog entry {capability_id}"),
            });
            continue;
        }

        let catalog_authority = entry.authority_source.unwrap_or(default_catalog_authority);
        discovery.guidance.push(GuidanceCapability {
            capability_id,
            title: title_from_identifier(&entry.path),
            applies_to: runtime_phases(&entry.applies_to),
            roles: entry.roles,
            content_ref,
            priority: entry.strength.to_runtime_priority(),
            authority_source,
            source_ref: pack_ref.clone(),
            pack_id: Some(pack_id.clone()),
            catalog_pillar: Some(entry.pillar),
            catalog_strength: Some(entry.strength),
            catalog_authority_source: Some(catalog_authority),
        });
    }

    for (guardian_id, entry) in guardian_index.guardian {
        if !matches_phase(&entry.applies_to, phase) {
            continue;
        }

        let catalog_authority = entry.authority_source.unwrap_or(default_catalog_authority);
        discovery.guardians.push(GuardianCapability {
            guardian_id: guardian_id.clone(),
            title: title_from_identifier(&guardian_id),
            kind: entry.kind,
            applies_to: runtime_phases(&entry.applies_to),
            rules: entry.rules,
            severity_floor: entry.default_disposition.to_runtime_disposition(),
            command_ref: derived_guardian_command(
                &guardian_id,
                entry.kind,
                &entry.default_disposition,
            ),
            instruction_ref: derived_guardian_instruction_ref(
                pack_dir,
                display_root,
                &catalog_manifest,
                &guardian_id,
                entry.kind,
            ),
            authority_source,
            source_ref: pack_ref.clone(),
            pack_id: Some(pack_id.clone()),
            catalog_pillar: Some(entry.pillar),
            catalog_default_disposition: Some(entry.default_disposition),
            catalog_authority_source: Some(catalog_authority),
        });
    }

    discovery
}

fn load_and_validate<T>(
    pack_dir: &Path,
    relative_path: &str,
    display_root: &Path,
    label: &str,
    pack_ref: &str,
    authority_source: GuidanceAuthoritySource,
) -> Result<T, Box<CatalogPackDiscovery>>
where
    T: serde::de::DeserializeOwned + CatalogValidatable,
{
    let file_path = pack_dir.join(relative_path);
    let source_ref = display_relative_path(display_root, &file_path);
    let contents = match fs::read_to_string(&file_path) {
        Ok(contents) => contents,
        Err(error) => {
            return Err(Box::new(skipped_pack(
                pack_ref,
                authority_source,
                source_ref,
                format!("failed to read {label}: {error}"),
            )));
        }
    };

    let parsed = match toml::from_str::<T>(&contents) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(Box::new(skipped_pack(
                pack_ref,
                authority_source,
                source_ref,
                format!("failed to parse {label}: {error}"),
            )));
        }
    };

    if let Err(error) = parsed.validate_catalog() {
        return Err(Box::new(skipped_pack(
            pack_ref,
            authority_source,
            source_ref,
            error.to_string(),
        )));
    }

    Ok(parsed)
}

fn skipped_pack(
    pack_ref: &str,
    authority_source: GuidanceAuthoritySource,
    source_ref: String,
    reason: String,
) -> CatalogPackDiscovery {
    CatalogPackDiscovery {
        skipped_packs: vec![format!("{pack_ref} ({reason})")],
        validation_findings: vec![CatalogValidationFinding {
            severity: CatalogValidationSeverity::Error,
            source_ref: source_ref.clone(),
            message: reason.clone(),
        }],
        skipped_sources: vec![SkippedCapabilitySource { source_ref, authority_source, reason }],
        ..CatalogPackDiscovery::default()
    }
}

fn matches_phase(labels: &[CatalogLifecycleLabel], phase: CapabilityPhase) -> bool {
    labels.iter().copied().any(|label| label.matches_runtime_phase(phase))
}

fn runtime_phases(labels: &[CatalogLifecycleLabel]) -> Vec<CapabilityPhase> {
    let mut phases = BTreeSet::new();

    for label in labels {
        if label.matches_runtime_phase(CapabilityPhase::Planning) {
            phases.insert(CapabilityPhase::Planning);
        }
        if label.matches_runtime_phase(CapabilityPhase::Architecture) {
            phases.insert(CapabilityPhase::Architecture);
        }
        if label.matches_runtime_phase(CapabilityPhase::Implementation) {
            phases.insert(CapabilityPhase::Implementation);
        }
        if label.matches_runtime_phase(CapabilityPhase::Testing) {
            phases.insert(CapabilityPhase::Testing);
        }
        if label.matches_runtime_phase(CapabilityPhase::Verification) {
            phases.insert(CapabilityPhase::Verification);
        }
        if label.matches_runtime_phase(CapabilityPhase::Review) {
            phases.insert(CapabilityPhase::Review);
        }
    }

    phases.into_iter().collect()
}

fn derived_guardian_command(
    guardian_id: &str,
    kind: GuardianKind,
    disposition: &CatalogGuardianDisposition,
) -> Option<String> {
    match kind {
        GuardianKind::Deterministic => {
            if guardian_id.contains("zero_panic") {
                Some("builtin:no-panic-flow".to_string())
            } else {
                Some(match disposition {
                    CatalogGuardianDisposition::Error | CatalogGuardianDisposition::Blocker => {
                        "builtin:validation-evidence".to_string()
                    }
                    _ => "builtin:no-unwrap-expect".to_string(),
                })
            }
        }
        GuardianKind::Hybrid => Some("builtin:catalog-hybrid-review".to_string()),
        GuardianKind::Llm => None,
    }
}

fn derived_guardian_instruction_ref(
    pack_dir: &Path,
    display_root: &Path,
    manifest: &CatalogManifest,
    guardian_id: &str,
    kind: GuardianKind,
) -> Option<String> {
    if matches!(kind, GuardianKind::Deterministic) {
        return None;
    }

    let file_name = guardian_id.replace('_', "-") + ".md";
    let candidate = pack_dir.join(&manifest.layout.guardians_dir).join(file_name);
    Some(display_relative_path(display_root, &candidate))
}

fn display_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn title_from_identifier(identifier: &str) -> String {
    let identifier_path = PathBuf::from(identifier);
    let stem = identifier_path.file_stem().and_then(|value| value.to_str()).unwrap_or(identifier);

    stem.split(['-', '_', '.'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

trait CatalogValidatable {
    fn validate_catalog(&self)
    -> Result<(), crate::domain::guidance_catalog::GuidanceCatalogError>;
}

impl CatalogValidatable for CatalogPackManifest {
    fn validate_catalog(
        &self,
    ) -> Result<(), crate::domain::guidance_catalog::GuidanceCatalogError> {
        self.validate()
    }
}

impl CatalogValidatable for CatalogManifest {
    fn validate_catalog(
        &self,
    ) -> Result<(), crate::domain::guidance_catalog::GuidanceCatalogError> {
        self.validate()
    }
}

impl CatalogValidatable for CatalogGuidanceIndex {
    fn validate_catalog(
        &self,
    ) -> Result<(), crate::domain::guidance_catalog::GuidanceCatalogError> {
        self.validate()
    }
}

impl CatalogValidatable for CatalogGuardianIndex {
    fn validate_catalog(
        &self,
    ) -> Result<(), crate::domain::guidance_catalog::GuidanceCatalogError> {
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use crate::domain::guidance::{CapabilityPhase, GuidanceAuthoritySource};

    use super::{
        derived_guardian_command, derived_guardian_instruction_ref, discover_catalog_packs,
        display_relative_path, runtime_phases, title_from_identifier,
    };

    fn write_valid_catalog_pack(root: &std::path::Path) {
        let catalog_dir = root.join("catalog");
        let guidance_dir = root.join("guidance");
        let guardians_dir = root.join("guardians");

        fs::create_dir_all(&catalog_dir).unwrap();
        fs::create_dir_all(&guidance_dir).unwrap();
        fs::create_dir_all(&guardians_dir).unwrap();

        fs::write(
            root.join("pack.toml"),
            "[pack]\nid = \"catalog\"\nversion = \"0.1.0\"\nkind = \"guidance-pack\"\ndescription = \"Catalog\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n",
        )
        .unwrap();
        fs::write(
            catalog_dir.join("catalog-manifest.toml"),
            "[catalog]\nid = \"catalog\"\nversion = \"0.1.0\"\nkind = \"guidance-catalog\"\nstatus = \"draft\"\ndescription = \"Catalog\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n\n[layout]\nguidance_dir = \"guidance\"\nguardians_dir = \"guardians\"\nschemas_dir = \"schemas\"\nexamples_dir = \"examples\"\n\n[pillars]\nincluded = [\"clean-code\", \"language\"]\n",
        )
        .unwrap();
        fs::write(
            catalog_dir.join("guidance-index.toml"),
            "[guidance.clean_code]\npath = \"guidance/clean-code.md\"\npillar = \"clean-code\"\nstrength = \"recommended\"\napplies_to = [\"implementation\", \"review\"]\nroles = [\"implementer\"]\n\n[guidance.backlog_priority]\npath = \"guidance/backlog-priority.md\"\npillar = \"architecture\"\nstrength = \"mandatory\"\napplies_to = [\"backlog\"]\nroles = [\"planner\"]\n",
        )
        .unwrap();
        fs::write(
            catalog_dir.join("guardian-index.toml"),
            "[guardian.rust_zero_panic]\npillar = \"language\"\nkind = \"deterministic\"\nrules = [\"no-panic-in-production-path\"]\napplies_to = [\"implementation\"]\ndefault_disposition = \"warning\"\n\n[guardian.catalog_review]\npillar = \"clean-code\"\nkind = \"hybrid\"\nrules = [\"review\"]\napplies_to = [\"review\"]\ndefault_disposition = \"concern\"\n",
        )
        .unwrap();
        fs::write(guidance_dir.join("clean-code.md"), "# Clean Code\n").unwrap();
        fs::write(guidance_dir.join("backlog-priority.md"), "# Backlog Priority\n").unwrap();
        fs::write(guardians_dir.join("catalog-review.md"), "# Catalog Review\n").unwrap();
    }

    #[test]
    fn discover_catalog_packs_loads_directory_pack_for_matching_phase() {
        let temp_root =
            std::env::temp_dir().join(format!("boundline-guidance-catalog-{}", Uuid::new_v4()));
        let packs_dir = temp_root.join("assistant/packs/guidance-catalog");
        write_valid_catalog_pack(&packs_dir);

        let discovery = discover_catalog_packs(
            &temp_root.join("assistant/packs"),
            &temp_root,
            GuidanceAuthoritySource::SharedPack,
            CapabilityPhase::Implementation,
        );

        assert_eq!(discovery.guidance.len(), 1);
        assert_eq!(discovery.guardians.len(), 1);
        assert_eq!(discovery.loaded_packs.len(), 1);
        assert!(discovery.skipped_packs.is_empty());
        assert!(discovery.validation_findings.is_empty());

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn discover_catalog_packs_reports_parse_failures_and_missing_manifests() {
        let temp_root = std::env::temp_dir()
            .join(format!("boundline-guidance-catalog-invalid-{}", Uuid::new_v4()));
        let packs_dir = temp_root.join("assistant/packs/guidance-catalog");
        fs::create_dir_all(packs_dir.join("catalog")).unwrap();
        fs::write(packs_dir.join("pack.toml"), "[pack]\nid = \"catalog\"\n").unwrap();

        let discovery = discover_catalog_packs(
            &temp_root.join("assistant/packs"),
            &temp_root,
            GuidanceAuthoritySource::SharedPack,
            CapabilityPhase::Implementation,
        );

        assert!(discovery.guidance.is_empty());
        assert!(discovery.guardians.is_empty());
        assert_eq!(discovery.skipped_packs.len(), 1);
        assert!(
            discovery.validation_findings[0]
                .message
                .contains("failed to parse catalog pack manifest")
        );

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn catalog_runtime_helpers_cover_phase_mapping_titles_and_guardian_derivation() {
        let phases = runtime_phases(&[
            crate::domain::guidance_catalog::CatalogLifecycleLabel::Backlog,
            crate::domain::guidance_catalog::CatalogLifecycleLabel::Migration,
            crate::domain::guidance_catalog::CatalogLifecycleLabel::Review,
        ]);
        assert_eq!(
            phases,
            vec![
                CapabilityPhase::Planning,
                CapabilityPhase::Architecture,
                CapabilityPhase::Implementation,
                CapabilityPhase::Review,
            ]
        );

        assert_eq!(title_from_identifier("guidance/clean-code.md"), "Clean Code");
        assert_eq!(title_from_identifier("rust_zero_panic"), "Rust Zero Panic");
        assert_eq!(
            display_relative_path(
                std::path::Path::new("/tmp/root"),
                std::path::Path::new("/tmp/root/catalog/file.toml")
            ),
            "catalog/file.toml"
        );

        assert_eq!(
            derived_guardian_command(
                "rust_zero_panic",
                crate::domain::guidance::GuardianKind::Deterministic,
                &crate::domain::guidance_catalog::CatalogGuardianDisposition::Warning,
            )
            .as_deref(),
            Some("builtin:no-panic-flow")
        );
        assert_eq!(
            derived_guardian_command(
                "validation_evidence",
                crate::domain::guidance::GuardianKind::Deterministic,
                &crate::domain::guidance_catalog::CatalogGuardianDisposition::Error,
            )
            .as_deref(),
            Some("builtin:validation-evidence")
        );
        assert_eq!(
            derived_guardian_command(
                "clean_code_review",
                crate::domain::guidance::GuardianKind::Hybrid,
                &crate::domain::guidance_catalog::CatalogGuardianDisposition::Concern,
            )
            .as_deref(),
            Some("builtin:catalog-hybrid-review")
        );
        assert_eq!(
            derived_guardian_command(
                "clean_code_review",
                crate::domain::guidance::GuardianKind::Llm,
                &crate::domain::guidance_catalog::CatalogGuardianDisposition::Concern,
            ),
            None
        );

        let manifest: crate::domain::guidance_catalog::CatalogManifest = toml::from_str(
            "[catalog]\nid = \"catalog\"\nversion = \"0.1.0\"\nkind = \"guidance-catalog\"\nstatus = \"draft\"\ndescription = \"Catalog\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n\n[layout]\nguidance_dir = \"guidance\"\nguardians_dir = \"guardians\"\nschemas_dir = \"schemas\"\nexamples_dir = \"examples\"\n\n[pillars]\nincluded = [\"clean-code\"]\n"
        ).unwrap();

        assert_eq!(
            derived_guardian_instruction_ref(
                std::path::Path::new("/tmp/root/assistant/packs/guidance-catalog"),
                std::path::Path::new("/tmp/root"),
                &manifest,
                "catalog_review",
                crate::domain::guidance::GuardianKind::Hybrid,
            )
            .as_deref(),
            Some("assistant/packs/guidance-catalog/guardians/catalog-review.md")
        );
        assert_eq!(
            derived_guardian_instruction_ref(
                std::path::Path::new("/tmp/root/assistant/packs/guidance-catalog"),
                std::path::Path::new("/tmp/root"),
                &manifest,
                "rust_zero_panic",
                crate::domain::guidance::GuardianKind::Deterministic,
            ),
            None
        );
    }

    #[test]
    fn discover_catalog_packs_filters_entries_by_phase() {
        let temp_root = std::env::temp_dir()
            .join(format!("boundline-guidance-catalog-phase-{}", Uuid::new_v4()));
        let packs_dir = temp_root.join("assistant/packs/guidance-catalog");
        write_valid_catalog_pack(&packs_dir);

        let discovery = discover_catalog_packs(
            &temp_root.join("assistant/packs"),
            &temp_root,
            GuidanceAuthoritySource::SharedPack,
            CapabilityPhase::Planning,
        );

        assert_eq!(discovery.guidance.len(), 1);
        assert_eq!(discovery.guidance[0].capability_id, "backlog_priority");
        assert!(discovery.guardians.is_empty());

        let _ = fs::remove_dir_all(&temp_root);
    }
}
