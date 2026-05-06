use std::fs;
use std::path::Path;
use std::sync::Mutex;

use boundline::cli::workspace::{WorkspaceResolutionError, resolve_workspace};
use uuid::Uuid;

static CWD_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("boundline-{prefix}-{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolve_workspace_covers_explicit_boundline_git_cwd_and_ambiguous_cases() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_dir = std::env::current_dir().unwrap();
    let root = temp_dir("workspace-resolution");

    let explicit = root.join("explicit");
    fs::create_dir_all(&explicit).unwrap();
    assert_eq!(resolve_workspace(Some(&explicit)).unwrap(), explicit.canonicalize().unwrap());

    let boundline_root = root.join("boundline-root");
    let nested = boundline_root.join("a/b/c");
    fs::create_dir_all(boundline_root.join(".boundline")).unwrap();
    fs::create_dir_all(&nested).unwrap();
    std::env::set_current_dir(&nested).unwrap();
    assert_eq!(resolve_workspace(None).unwrap(), boundline_root.canonicalize().unwrap());

    let git_root = root.join("git-root");
    let git_nested = git_root.join("packages/app");
    fs::create_dir_all(git_root.join(".git")).unwrap();
    fs::create_dir_all(&git_nested).unwrap();
    std::env::set_current_dir(&git_nested).unwrap();
    assert_eq!(resolve_workspace(None).unwrap(), git_root.canonicalize().unwrap());

    let plain_root = root.join("plain-root");
    fs::create_dir_all(&plain_root).unwrap();
    std::env::set_current_dir(&plain_root).unwrap();
    assert_eq!(resolve_workspace(None).unwrap(), plain_root.canonicalize().unwrap());

    let ambiguous_root = root.join("ambiguous-root");
    let ambiguous_nested = ambiguous_root.join("child/grandchild");
    fs::create_dir_all(ambiguous_root.join(".boundline")).unwrap();
    fs::create_dir_all(ambiguous_root.join("child/.boundline")).unwrap();
    fs::create_dir_all(&ambiguous_nested).unwrap();
    std::env::set_current_dir(&ambiguous_nested).unwrap();
    let err = resolve_workspace(None).unwrap_err();
    assert!(matches!(err, WorkspaceResolutionError::Ambiguous(_)));

    std::env::set_current_dir(previous_dir).unwrap();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn resolve_workspace_makes_relative_explicit_paths_absolute() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_dir = std::env::current_dir().unwrap();
    let root = temp_dir("workspace-relative-resolution");
    fs::create_dir_all(root.join("child")).unwrap();

    std::env::set_current_dir(&root).unwrap();
    assert_eq!(
        resolve_workspace(Some(Path::new("child"))).unwrap(),
        root.join("child").canonicalize().unwrap()
    );

    std::env::set_current_dir(previous_dir).unwrap();
    let _ = fs::remove_dir_all(root);
}
