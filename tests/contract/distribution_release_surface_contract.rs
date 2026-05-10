use std::fs;
use std::path::Path;

#[test]
fn release_surface_closes_on_0_47_0_without_an_upcoming_047_entry() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(repo_root.join("Cargo.toml")).unwrap();
    let changelog = fs::read_to_string(repo_root.join("CHANGELOG.md")).unwrap();
    let roadmap = fs::read_to_string(repo_root.join("ROADMAP.md")).unwrap();
    let windows_release_workflow =
        fs::read_to_string(repo_root.join(".github/workflows/release-windows-distribution.yml"))
            .unwrap();
    let homebrew_tap_workflow =
        fs::read_to_string(repo_root.join(".github/workflows/sync-homebrew-tap.yml")).unwrap();

    assert!(cargo_toml.contains("version = \"0.47.0\""));
    assert!(changelog.contains("## [0.47.0] - 2026-05-09"));
    assert!(changelog.contains("Guided Init TUI and Runtime Catalog"));
    assert!(roadmap.contains("## Current Status: v0.47.0"));
    assert!(roadmap.contains("### Delivered in 0.47.0"));
    assert!(windows_release_workflow.contains(
        "git clone --depth 1 --branch \"$canonVersion\" https://github.com/apply-the/canon canon-source"
    ));
    assert!(windows_release_workflow.contains(
        "cargo build --locked --release --package canon-cli --bin canon --target x86_64-pc-windows-msvc --manifest-path canon-source/Cargo.toml --target-dir canon-source/target"
    ));
    assert!(homebrew_tap_workflow.contains("workflow_dispatch:"));
    assert!(homebrew_tap_workflow.contains("branches:\n      - main"));
    assert!(homebrew_tap_workflow.contains("tags:\n      - \"*.*.*\""));
    assert!(homebrew_tap_workflow.contains("paths:\n      - \"Cargo.toml\""));
    assert!(
        homebrew_tap_workflow.contains("distribution/channel-metadata.toml"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("distribution/homebrew/Formula/boundline.rb"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("scripts/release/sync-homebrew-tap.sh"),
        "{homebrew_tap_workflow}"
    );
    assert!(homebrew_tap_workflow.contains("id: sync_formula"), "{homebrew_tap_workflow}");
    assert!(
        homebrew_tap_workflow.contains("if: steps.sync_formula.outputs.status != 'noop'"),
        "{homebrew_tap_workflow}"
    );
    assert!(homebrew_tap_workflow.contains("ref: main"), "{homebrew_tap_workflow}");
    assert!(
        homebrew_tap_workflow.contains("git config user.name \"github-actions[bot]\""),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("git add Formula/boundline.rb"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("git commit -m \"boundline ${BOUNDLINE_VERSION}\""),
        "{homebrew_tap_workflow}"
    );
    assert!(homebrew_tap_workflow.contains("git push origin HEAD:main"), "{homebrew_tap_workflow}");
    assert!(
        homebrew_tap_workflow
            .contains("Expected ${FORMULA_PATH} to change before pushing Boundline"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow
            .contains("Add it as a repository or organization secret available to this repo"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("contents:write access to ${TAP_REPOSITORY}"),
        "{homebrew_tap_workflow}"
    );
    assert!(
        homebrew_tap_workflow.contains("permission to push to its main branch"),
        "{homebrew_tap_workflow}"
    );
    assert!(!homebrew_tap_workflow.contains("has_tap_token"));
    assert!(!homebrew_tap_workflow.contains("pull-request-number"));
    assert!(!homebrew_tap_workflow.contains("create-pull-request@v7"));
    assert!(
        !windows_release_workflow
            .contains("Invoke-WebRequest -Uri $canonUrl -OutFile $canonArchive")
    );
    assert!(!roadmap.contains("044-cli-init-ux"));
}
