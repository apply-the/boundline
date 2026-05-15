use std::fs;
use std::path::Path;

use boundline::{CatalogAuthoritySource, CatalogGuidanceStrength, CatalogManifest, CatalogPillar};

#[test]
fn bundled_catalog_manifest_matches_the_055_contract() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_path =
        repo_root.join("assistant/packs/guidance-catalog/catalog/catalog-manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    let manifest: CatalogManifest = toml::from_str(&manifest_text).unwrap();

    manifest.validate().unwrap();

    assert_eq!(manifest.catalog.id, "boundline-guidance-catalog");
    assert_eq!(manifest.compatibility.boundline, ">=0.55");
    assert_eq!(manifest.authority.default_source, CatalogAuthoritySource::SharedPack);
    assert_eq!(manifest.authority.default_strength, CatalogGuidanceStrength::Recommended);
    assert!(manifest.pillars.included.contains(&CatalogPillar::Observability));
    assert!(manifest.pillars.included.contains(&CatalogPillar::SupplyChain));
}
