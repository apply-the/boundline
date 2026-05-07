use std::fs;
use std::path::Path;

#[test]
fn release_surface_closes_on_0_44_0_without_an_upcoming_044_entry() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let changelog = fs::read_to_string(repo_root.join("CHANGELOG.md")).unwrap();
    let roadmap = fs::read_to_string(repo_root.join("ROADMAP.md")).unwrap();
    let windows_release_workflow =
        fs::read_to_string(repo_root.join(".github/workflows/release-windows-distribution.yml"))
            .unwrap();

    assert!(cargo_toml.contains("version = \"0.44.0\""));
    assert!(changelog.contains("## [0.44.0] - 2026-05-07"));
    assert!(changelog.contains("044` - CLI Init UX"));
    assert!(roadmap.contains("## Current Status: v0.44.0"));
    assert!(roadmap.contains("### Delivered in 0.44.0"));
    assert!(windows_release_workflow.contains(
        "git clone --depth 1 --branch \"$canonVersion\" https://github.com/apply-the/canon canon-source"
    ));
    assert!(windows_release_workflow.contains(
        "cargo build --locked --release --package canon-cli --bin canon --target x86_64-pc-windows-msvc --manifest-path canon-source/Cargo.toml --target-dir canon-source/target"
    ));
    assert!(
        !windows_release_workflow
            .contains("Invoke-WebRequest -Uri $canonUrl -OutFile $canonArchive")
    );
    assert!(!roadmap.contains("044-cli-init-ux"));
}
