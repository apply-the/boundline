use std::fs;
use std::path::Path;

use boundline::{
    CapabilityPhase, CatalogGuidanceIndex, CatalogGuidanceStrength, CatalogPillar,
    CatalogValidationSeverity, GuidanceAuthoritySource, discover_catalog_packs,
};
use uuid::Uuid;

#[test]
fn bundled_guidance_index_is_contract_valid_and_points_to_real_markdown() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_path = repo_root.join("assistant/packs/guidance-catalog/catalog/guidance-index.toml");
    let index_text = fs::read_to_string(&index_path).unwrap();
    let index: CatalogGuidanceIndex = toml::from_str(&index_text).unwrap();

    index.validate().unwrap();

    let clean_code = index.guidance.get("clean_code").unwrap();
    assert_eq!(clean_code.pillar, CatalogPillar::CleanCode);
    assert_eq!(clean_code.strength, CatalogGuidanceStrength::Recommended);

    for entry in index.guidance.values() {
        assert!(repo_root.join("assistant/packs/guidance-catalog").join(&entry.path).is_file());
    }
}

#[test]
fn guidance_index_rejects_legacy_strength_aliases() {
    let invalid = "[guidance.clean_code]\npath = \"guidance/clean-code.md\"\npillar = \"clean-code\"\nstrength = \"recommendation\"\napplies_to = [\"planning\"]\nroles = [\"planner\"]\n";

    assert!(toml::from_str::<CatalogGuidanceIndex>(invalid).is_err());
}

#[test]
fn guidance_index_rejects_invalid_lifecycle_labels() {
    let invalid = "[guidance.clean_code]\npath = \"guidance/clean-code.md\"\npillar = \"clean-code\"\nstrength = \"recommended\"\napplies_to = [\"deploy\"]\nroles = [\"planner\"]\n";

    assert!(toml::from_str::<CatalogGuidanceIndex>(invalid).is_err());
}

#[test]
fn guidance_index_rejects_duplicate_entry_ids() {
    let invalid = concat!(
        "[guidance.clean_code]\n",
        "path = \"guidance/clean-code.md\"\n",
        "pillar = \"clean-code\"\n",
        "strength = \"recommended\"\n",
        "applies_to = [\"planning\"]\n",
        "roles = [\"planner\"]\n\n",
        "[guidance.clean_code]\n",
        "path = \"guidance/clean-code-2.md\"\n",
        "pillar = \"clean-code\"\n",
        "strength = \"mandatory\"\n",
        "applies_to = [\"planning\"]\n",
        "roles = [\"planner\"]\n",
    );

    assert!(toml::from_str::<CatalogGuidanceIndex>(invalid).is_err());
}

#[test]
fn guidance_index_missing_referenced_markdown_emits_warning_during_discovery() {
    let temp_root = std::env::temp_dir()
        .join(format!("boundline-guidance-index-missing-markdown-{}", Uuid::new_v4()));
    let pack_dir = temp_root.join("assistant/packs/warning-pack");
    let catalog_dir = pack_dir.join("catalog");

    fs::create_dir_all(&catalog_dir).unwrap();
    fs::write(
        pack_dir.join("pack.toml"),
        "[pack]\nid = \"warning-pack\"\nversion = \"0.1.0\"\nkind = \"guidance-pack\"\ndescription = \"Warning Pack\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n",
    )
    .unwrap();
    fs::write(
        catalog_dir.join("catalog-manifest.toml"),
        "[catalog]\nid = \"warning-pack\"\nversion = \"0.1.0\"\nkind = \"guidance-catalog\"\nstatus = \"draft\"\ndescription = \"Warning Pack\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n\n[layout]\nguidance_dir = \"guidance\"\nguardians_dir = \"guardians\"\nschemas_dir = \"schemas\"\nexamples_dir = \"examples\"\n\n[pillars]\nincluded = [\"clean-code\"]\n",
    )
    .unwrap();
    fs::write(
        catalog_dir.join("guidance-index.toml"),
        "[guidance.clean_code]\npath = \"guidance/missing-clean-code.md\"\npillar = \"clean-code\"\nstrength = \"recommended\"\napplies_to = [\"planning\"]\nroles = [\"planner\"]\n",
    )
    .unwrap();
    fs::write(catalog_dir.join("guardian-index.toml"), "").unwrap();

    let discovery = discover_catalog_packs(
        &temp_root.join("assistant/packs"),
        &temp_root,
        GuidanceAuthoritySource::SharedPack,
        CapabilityPhase::Planning,
    );

    assert_eq!(discovery.loaded_packs.len(), 1);
    assert!(discovery.guidance.is_empty());
    assert!(discovery.validation_findings.iter().any(|finding| {
        finding.severity == CatalogValidationSeverity::Warning
            && finding.source_ref == "assistant/packs/warning-pack/guidance/missing-clean-code.md"
            && finding.message.contains("missing guidance markdown for catalog entry clean_code")
    }));

    let _ = fs::remove_dir_all(&temp_root);
}
