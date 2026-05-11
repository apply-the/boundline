use std::fs;
use std::path::Path;

use boundline::SUPPORTED_CANON_VERSION;

#[test]
fn distribution_metadata_keeps_versions_and_bundle_names_aligned() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let channel_metadata =
        fs::read_to_string(repo_root.join("distribution/channel-metadata.toml")).unwrap();
    let formula =
        fs::read_to_string(repo_root.join("distribution/homebrew/Formula/boundline.rb")).unwrap();
    let winget_version =
        fs::read_to_string(repo_root.join(
            "distribution/winget/manifests/a/ApplyThe/Boundline/0.49.0/ApplyThe.Boundline.yaml",
        ))
        .unwrap();
    let winget_installer = fs::read_to_string(repo_root.join(
        "distribution/winget/manifests/a/ApplyThe/Boundline/0.49.0/ApplyThe.Boundline.installer.yaml",
    ))
    .unwrap();
    let winget_locale = fs::read_to_string(repo_root.join(
        "distribution/winget/manifests/a/ApplyThe/Boundline/0.49.0/ApplyThe.Boundline.locale.en-US.yaml",
    ))
    .unwrap();
    let canon_version_line = format!("canon_version = \"{SUPPORTED_CANON_VERSION}\"");
    let formula_tag_line = format!("tag: \"{SUPPORTED_CANON_VERSION}\"");

    assert!(cargo_toml.contains("version = \"0.49.0\""));
    assert!(channel_metadata.contains("boundline_version = \"0.49.0\""));
    assert!(channel_metadata.contains(&canon_version_line));
    assert!(channel_metadata.contains("tap_repository = \"apply-the/homebrew-boundline\""));
    assert!(channel_metadata.contains("tap_name = \"apply-the/boundline\""));
    assert!(formula.contains("version \"0.49.0\""));
    assert!(formula.contains("using: :git, tag: \"0.49.0\""));
    assert!(formula.contains("resource \"canon-source\""));
    assert!(formula.contains(&formula_tag_line));
    assert!(formula.contains("boundline doctor --install"));
    assert!(winget_version.contains("PackageVersion: 0.49.0"));
    assert!(winget_installer.contains("boundline-bundle-0.49.0-windows-x86_64.zip"));
    assert!(winget_installer.contains("releases/download/0.49.0/"));
    assert!(winget_installer.contains("PortableCommandAlias: canon"));
    assert!(winget_locale.contains("boundline doctor --install"));
}
