use std::fs;
use std::path::Path;

use boundline::SUPPORTED_CANON_VERSION;
use boundline::assistant_plugin_validation::workspace_version_from_toml;

#[test]
fn distribution_metadata_keeps_versions_and_bundle_names_aligned() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let version = workspace_version_from_toml(&cargo_toml).expect("workspace version must parse");
    let channel_metadata =
        fs::read_to_string(repo_root.join("distribution/channel-metadata.toml")).unwrap();
    let formula =
        fs::read_to_string(repo_root.join("distribution/homebrew/Formula/boundline.rb")).unwrap();
    let winget_manifest_root =
        format!("distribution/winget/manifests/a/ApplyThe/Boundline/{version}");
    let winget_version = fs::read_to_string(
        repo_root.join(format!("{winget_manifest_root}/ApplyThe.Boundline.yaml")),
    )
    .unwrap();
    let winget_installer = fs::read_to_string(
        repo_root.join(format!("{winget_manifest_root}/ApplyThe.Boundline.installer.yaml")),
    )
    .unwrap();
    let winget_locale = fs::read_to_string(
        repo_root.join(format!("{winget_manifest_root}/ApplyThe.Boundline.locale.en-US.yaml")),
    )
    .unwrap();
    let canon_version_line = format!("canon_version = \"{SUPPORTED_CANON_VERSION}\"");
    let formula_tag_line = format!("tag: \"{SUPPORTED_CANON_VERSION}\"");
    let boundline_version_line = format!("version = \"{version}\"");
    let channel_version_line = format!("boundline_version = \"{version}\"");
    let formula_version_line = format!("version \"{version}\"");
    let formula_url_line = format!("using: :git, tag: \"{version}\"");
    let winget_version_line = format!("PackageVersion: {version}");
    let winget_bundle_line = format!("boundline-bundle-{version}-windows-x86_64.zip");
    let winget_release_line = format!("releases/download/{version}/");

    assert!(cargo_toml.contains(&boundline_version_line));
    assert!(channel_metadata.contains(&channel_version_line));
    assert!(channel_metadata.contains(&canon_version_line));
    assert!(channel_metadata.contains("tap_repository = \"apply-the/homebrew-boundline\""));
    assert!(channel_metadata.contains("tap_name = \"apply-the/boundline\""));
    assert!(formula.contains(&formula_version_line));
    assert!(formula.contains(&formula_url_line));
    assert!(formula.contains("resource \"canon-source\""));
    assert!(formula.contains(&formula_tag_line));
    assert!(formula.contains("boundline doctor --install"));
    assert!(winget_version.contains(&winget_version_line));
    assert!(winget_installer.contains(&winget_bundle_line));
    assert!(winget_installer.contains(&winget_release_line));
    assert!(winget_installer.contains("PortableCommandAlias: canon"));
    assert!(winget_locale.contains("boundline doctor --install"));
}
