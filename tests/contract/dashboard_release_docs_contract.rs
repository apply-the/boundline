use std::fs;
use std::path::Path;

use crate::dashboard_fixture::{DashboardTestResult, require, require_contains};

#[test]
fn release_docs_align_dashboard_versions_and_hide_internal_roadmap_codes() -> DashboardTestResult {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = [
        "README.md",
        "CHANGELOG.md",
        "ROADMAP.md",
        "docs/getting-started.md",
        "docs/delivery-model.md",
        "docs/architecture.md",
        "docs/configuration.md",
        "docs/release-checklist.md",
    ];
    for file in files {
        let text = fs::read_to_string(repo_root.join(file))?;
        require(!text.contains("S8"), &format!("{file} must not mention internal roadmap code"))?;
    }

    require_contains(
        &fs::read_to_string(repo_root.join("README.md"))?,
        "boundline-dashboard",
        "README dashboard",
    )?;
    require_contains(
        &fs::read_to_string(repo_root.join("CHANGELOG.md"))?,
        "Boundline `0.64.0`, Canon `0.60.0`",
        "changelog versions",
    )?;
    require_contains(
        &fs::read_to_string(repo_root.join("distribution/channel-metadata.toml"))?,
        "canon_version = \"0.60.0\"",
        "canon metadata",
    )
}
