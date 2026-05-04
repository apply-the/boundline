use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::workspace_fixture::terminal_text;

fn snapshot_repo_paths(repo_root: &Path, targets: &[&str]) -> BTreeMap<String, Vec<u8>> {
    let mut snapshot = BTreeMap::new();
    for target in targets {
        collect_snapshot(repo_root, &repo_root.join(target), &mut snapshot);
    }
    snapshot
}

fn collect_snapshot(repo_root: &Path, path: &Path, snapshot: &mut BTreeMap<String, Vec<u8>>) {
    if path.is_file() {
        let relative = path.strip_prefix(repo_root).unwrap().to_string_lossy().into_owned();
        snapshot.insert(relative, fs::read(path).unwrap());
        return;
    }

    if path.is_dir() {
        let mut entries =
            fs::read_dir(path).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
        entries.sort();
        for entry in entries {
            collect_snapshot(repo_root, &entry, snapshot);
        }
    }
}

#[test]
fn sync_distribution_metadata_script_runs_cleanly_against_the_repo() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let before = snapshot_repo_paths(
        repo_root,
        &["distribution/homebrew/Formula/boundline.rb", "distribution/winget/manifests"],
    );

    let sync = Command::new("bash")
        .args(["scripts/sync-distribution-metadata.sh"])
        .current_dir(repo_root)
        .output()
        .unwrap();
    let sync_text = terminal_text(&sync);
    assert_eq!(sync.status.code(), Some(0), "{sync_text}");

    let after = snapshot_repo_paths(
        repo_root,
        &["distribution/homebrew/Formula/boundline.rb", "distribution/winget/manifests"],
    );
    assert_eq!(after, before, "{sync_text}");
}
