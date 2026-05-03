use std::path::Path;
use std::process::Command;

use crate::workspace_fixture::terminal_text;

#[test]
fn sync_distribution_metadata_script_runs_cleanly_against_the_repo() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let sync = Command::new("bash")
        .args(["scripts/sync-distribution-metadata.sh"])
        .current_dir(repo_root)
        .output()
        .unwrap();
    let sync_text = terminal_text(&sync);
    assert_eq!(sync.status.code(), Some(0), "{sync_text}");

    let diff = Command::new("git")
        .args([
            "diff",
            "--exit-code",
            "--",
            "distribution/homebrew/Formula/boundline.rb",
            "distribution/winget/manifests",
        ])
        .current_dir(repo_root)
        .output()
        .unwrap();
    let diff_text = terminal_text(&diff);
    assert_eq!(diff.status.code(), Some(0), "{diff_text}");
}
