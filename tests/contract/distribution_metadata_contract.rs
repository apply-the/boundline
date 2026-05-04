use std::fs;
use std::path::Path;

#[test]
fn distribution_metadata_keeps_versions_and_bundle_names_aligned() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let bundle_policy =
        fs::read_to_string(repo_root.join("distribution/canon-bundle.toml")).unwrap();
    let formula =
        fs::read_to_string(repo_root.join("distribution/homebrew/Formula/boundline.rb")).unwrap();
    let winget_version =
        fs::read_to_string(repo_root.join(
            "distribution/winget/manifests/a/ApplyThe/Boundline/0.40.0/ApplyThe.Boundline.yaml",
        ))
        .unwrap();
    let winget_installer = fs::read_to_string(repo_root.join(
        "distribution/winget/manifests/a/ApplyThe/Boundline/0.40.0/ApplyThe.Boundline.installer.yaml",
    ))
    .unwrap();
    let winget_locale = fs::read_to_string(repo_root.join(
        "distribution/winget/manifests/a/ApplyThe/Boundline/0.40.0/ApplyThe.Boundline.locale.en-US.yaml",
    ))
    .unwrap();

    assert!(cargo_toml.contains("version = \"0.40.0\""));
    assert!(bundle_policy.contains("boundline_version = \"0.40.0\""));
    assert!(bundle_policy.contains("canon_version = \"0.39.0\""));
    assert!(formula.contains("version \"0.40.0\""));
    assert!(formula.contains("boundline-bundle-0.40.0-macos-arm64.tar.gz"));
    assert!(formula.contains("boundline doctor --install"));
    assert!(winget_version.contains("PackageVersion: 0.40.0"));
    assert!(winget_installer.contains("boundline-bundle-0.40.0-windows-x86_64.zip"));
    assert!(winget_installer.contains("PortableCommandAlias: canon"));
    assert!(winget_locale.contains("boundline doctor --install"));
}
