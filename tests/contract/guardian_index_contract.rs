use std::fs;
use std::path::Path;

use boundline::{CatalogGuardianDisposition, CatalogGuardianIndex, CatalogPillar, GuardianKind};

#[test]
fn bundled_guardian_index_is_contract_valid_and_preserves_canonical_vocabulary() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let index_path = repo_root.join("assistant/packs/guidance-catalog/catalog/guardian-index.toml");
    let index_text = fs::read_to_string(&index_path).unwrap();
    let index: CatalogGuardianIndex = toml::from_str(&index_text).unwrap();

    index.validate().unwrap();

    let rust_zero_panic = index.guardian.get("rust_zero_panic").unwrap();
    assert_eq!(rust_zero_panic.pillar, CatalogPillar::Language);
    assert_eq!(rust_zero_panic.kind, GuardianKind::Deterministic);
    assert_eq!(rust_zero_panic.default_disposition, CatalogGuardianDisposition::Warning);

    let clean_code = index.guardian.get("clean_code").unwrap();
    assert_eq!(clean_code.kind, GuardianKind::Llm);
    assert_eq!(clean_code.default_disposition, CatalogGuardianDisposition::Concern);
}

#[test]
fn guardian_index_rejects_unsupported_guardian_kind_values() {
    let invalid = "[guardian.clean_code]\npillar = \"clean-code\"\nkind = \"semantic\"\nrules = [\"intent-revealing-names\"]\napplies_to = [\"review\"]\ndefault_disposition = \"concern\"\n";

    assert!(toml::from_str::<CatalogGuardianIndex>(invalid).is_err());
}

#[test]
fn guardian_index_rejects_invalid_lifecycle_labels() {
    let invalid = "[guardian.clean_code]\npillar = \"clean-code\"\nkind = \"llm\"\nrules = [\"intent-revealing-names\"]\napplies_to = [\"deploy\"]\ndefault_disposition = \"concern\"\n";

    assert!(toml::from_str::<CatalogGuardianIndex>(invalid).is_err());
}

#[test]
fn guardian_index_rejects_duplicate_guardian_ids() {
    let invalid = concat!(
        "[guardian.clean_code]\n",
        "pillar = \"clean-code\"\n",
        "kind = \"llm\"\n",
        "rules = [\"intent-revealing-names\"]\n",
        "applies_to = [\"review\"]\n",
        "default_disposition = \"concern\"\n\n",
        "[guardian.clean_code]\n",
        "pillar = \"clean-code\"\n",
        "kind = \"hybrid\"\n",
        "rules = [\"intent-revealing-names\"]\n",
        "applies_to = [\"review\"]\n",
        "default_disposition = \"warning\"\n",
    );

    assert!(toml::from_str::<CatalogGuardianIndex>(invalid).is_err());
}
