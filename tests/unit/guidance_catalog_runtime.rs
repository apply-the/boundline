use std::fs;
use std::path::Path;

use boundline::domain::goal_plan::WorkspaceSignals;
use boundline::{
    CapabilityPhase, CatalogGuardianDisposition, CatalogGuidanceStrength,
    CatalogValidationSeverity, ContextInput, ContextInputKind, ContextPack, ContextPackCredibility,
    GuidanceAuthoritySource, discover_catalog_packs, planning_runtime_evidence,
    resolve_capabilities_for_phase,
};
use uuid::Uuid;

#[test]
fn bundled_guidance_catalog_pack_loads_without_validation_findings() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = discover_catalog_packs(
        &repo_root.join("assistant/packs"),
        repo_root,
        GuidanceAuthoritySource::SharedPack,
        CapabilityPhase::Implementation,
    );

    assert!(discovery.loaded_packs.iter().any(|pack| {
        pack.contains("assistant/packs/guidance-catalog")
            && pack.contains("boundline-guidance-catalog")
    }));
    assert!(discovery.validation_findings.is_empty(), "{:#?}", discovery.validation_findings);
    assert!(discovery.guidance.iter().any(|capability| {
        capability.pack_id.as_deref() == Some("boundline-guidance-catalog")
            && capability.catalog_strength == Some(CatalogGuidanceStrength::Recommended)
            && capability.content_ref.starts_with("assistant/packs/guidance-catalog/guidance/")
    }));
    assert!(discovery.guardians.iter().any(|guardian| {
        guardian.guardian_id == "rust_zero_panic"
            && guardian.catalog_default_disposition == Some(CatalogGuardianDisposition::Warning)
            && guardian.command_ref.as_deref() == Some("builtin:no-panic-flow")
    }));
}

#[test]
fn canon_guidance_wins_over_catalog_pack_in_unit_surface() {
    let workspace =
        std::env::temp_dir().join(format!("boundline-canon-guidance-{}", Uuid::new_v4()));
    fs::create_dir_all(workspace.join(".canon/boundline/guidance")).unwrap();
    fs::write(
        workspace.join(".canon/boundline/guidance/clean-code.md"),
        "# Canon Clean Code\nPrefer the governed standard.\n",
    )
    .unwrap();

    let context_pack = ContextPack {
        pack_id: "context-pack".to_string(),
        summary: "canon precedence context".to_string(),
        credibility: ContextPackCredibility::Credible,
        inputs: vec![ContextInput {
            kind: ContextInputKind::WorkspaceFile,
            reference: "src/lib.rs".to_string(),
            rationale: "bounded target".to_string(),
            source: "workspace_scan".to_string(),
            primary: true,
        }],
        selected_targets: vec!["src/lib.rs".to_string()],
        advanced_context: None,
        staleness_reason: None,
    };
    let signals = WorkspaceSignals {
        language: Some("rust".to_string()),
        file_count: 1,
        has_config: false,
        has_canon: true,
        has_tests: true,
    };

    let resolution = resolve_capabilities_for_phase(
        &workspace,
        CapabilityPhase::Planning,
        &planning_runtime_evidence(
            "Apply the governed clean code standard",
            &context_pack,
            &signals,
        ),
    );

    assert!(resolution.guidance.iter().any(|capability| {
        capability.authority_source == GuidanceAuthoritySource::CanonGoverned
            && capability.source_ref == ".canon/boundline/guidance/clean-code.md"
    }));
    assert!(
        resolution
            .projection
            .loaded_guidance_sources
            .iter()
            .any(|source| { source == ".canon/boundline/guidance/clean-code.md" })
    );
    assert!(resolution.projection.skipped_guidance_sources.iter().any(|source| {
        source.contains("assistant/packs/guidance-catalog") && source.contains("shadowed")
    }));

    let _ = fs::remove_dir_all(&workspace);
}

#[test]
fn malformed_catalog_entry_emits_explicit_validation_finding() {
    let temp_root =
        std::env::temp_dir().join(format!("boundline-invalid-catalog-{}", Uuid::new_v4()));
    let packs_dir = temp_root.join("assistant/packs/guidance-catalog");
    let catalog_dir = packs_dir.join("catalog");

    fs::create_dir_all(&catalog_dir).unwrap();
    fs::write(
        packs_dir.join("pack.toml"),
        "[pack]\nid = \"catalog\"\nversion = \"0.1.0\"\nkind = \"guidance-pack\"\ndescription = \"Catalog\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n",
    )
    .unwrap();
    fs::write(
        catalog_dir.join("catalog-manifest.toml"),
        "[catalog]\nid = \"catalog\"\nversion = \"0.1.0\"\nkind = \"guidance-catalog\"\nstatus = \"draft\"\ndescription = \"Catalog\"\n\n[compatibility]\nboundline = \">=0.55\"\n\n[authority]\ndefault_source = \"shared-pack\"\ndefault_strength = \"recommended\"\ncanon_promotable = true\nworkspace_override_allowed = true\n\n[layout]\nguidance_dir = \"guidance\"\nguardians_dir = \"guardians\"\nschemas_dir = \"schemas\"\nexamples_dir = \"examples\"\n\n[pillars]\nincluded = [\"clean-code\"]\n",
    )
    .unwrap();
    fs::write(
        catalog_dir.join("guidance-index.toml"),
        "[guidance.clean_code]\npath = \"guidance/missing.md\"\npillar = \"clean-code\"\nstrength = \"recommended\"\napplies_to = [\"implementation\"]\nroles = [\"implementer\"]\n",
    )
    .unwrap();
    fs::write(
        catalog_dir.join("guardian-index.toml"),
        "[guardian.clean_code]\npillar = \"clean-code\"\nkind = \"llm\"\nrules = [\"intent-revealing-names\"]\napplies_to = [\"implementation\"]\ndefault_disposition = \"concern\"\n",
    )
    .unwrap();

    let discovery = discover_catalog_packs(
        &temp_root.join("assistant/packs"),
        &temp_root,
        GuidanceAuthoritySource::SharedPack,
        CapabilityPhase::Implementation,
    );

    assert_eq!(discovery.loaded_packs.len(), 1);
    assert_eq!(discovery.guidance.len(), 0);
    assert!(discovery.validation_findings.iter().any(|finding| {
        finding.severity == CatalogValidationSeverity::Warning
            && finding.message.contains("missing guidance markdown for catalog entry clean_code")
    }));

    let _ = fs::remove_dir_all(&temp_root);
}
