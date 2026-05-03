use std::fs;
use std::path::Path;

#[test]
fn release_surface_closes_on_0_39_0_without_an_upcoming_039_entry() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let changelog = fs::read_to_string(repo_root.join("CHANGELOG.md")).unwrap();
    let roadmap = fs::read_to_string(repo_root.join("ROADMAP.md")).unwrap();

    assert!(cargo_toml.contains("version = \"0.39.0\""));
    assert!(changelog.contains("## [0.39.0] - 2026-05-03"));
    assert!(changelog.contains("039` - Distribution & Bundling"));
    assert!(roadmap.contains("## Current Status: v0.39.0"));
    assert!(roadmap.contains("### Delivered in 0.39.0"));
    assert!(!roadmap.contains("039 Distribution & Bundling (Upcoming)"));
}
