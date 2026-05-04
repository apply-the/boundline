use std::fs;
use std::path::Path;

#[test]
fn release_surface_closes_on_0_41_0_without_an_upcoming_041_entry() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let changelog = fs::read_to_string(repo_root.join("CHANGELOG.md")).unwrap();
    let roadmap = fs::read_to_string(repo_root.join("ROADMAP.md")).unwrap();

    assert!(cargo_toml.contains("version = \"0.41.0\""));
    assert!(changelog.contains("## [0.41.0] - 2026-05-04"));
    assert!(changelog.contains("041` - Checkpoint Rewind"));
    assert!(roadmap.contains("## Current Status: v0.41.0"));
    assert!(roadmap.contains("### Delivered in 0.41.0"));
    assert!(!roadmap.contains("041-checkpoint-rewind"));
}
