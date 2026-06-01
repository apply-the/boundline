use std::path::{Path, PathBuf};

use thiserror::Error;

const PWD_ENV_VAR: &str = "PWD";

#[derive(Debug, Error)]
pub enum WorkspaceResolutionError {
    #[error("failed to determine current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("ambiguous workspace: multiple .boundline/ directories found above {0}")]
    Ambiguous(PathBuf),
    #[error("workspace not found or not a directory: {0}")]
    NotFound(PathBuf),
}

/// Resolve the workspace directory using the spec-required strategy:
///
/// 1. Use `--workspace <path>` when supplied.
/// 2. Search upward from CWD for an existing `.boundline/` directory; use its parent.
/// 3. Search upward for the nearest `.git/` (git root) directory; use its parent.
/// 4. Fall back to the current working directory.
pub fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, WorkspaceResolutionError> {
    // Treat `.` the same as omitting the flag so commands default to the repo root.
    if let Some(path) = workspace
        && path != Path::new(".")
    {
        let abs =
            if path.is_absolute() { path.to_path_buf() } else { current_dir_or_pwd()?.join(path) };
        let resolved = abs.canonicalize().unwrap_or_else(|_| abs.clone());
        if !resolved.is_dir() {
            return Err(WorkspaceResolutionError::NotFound(resolved));
        }
        return Ok(resolved);
    }

    let cwd = current_dir_or_pwd()?;

    discover_workspace_root(&cwd)
}

fn current_dir_or_pwd() -> Result<PathBuf, WorkspaceResolutionError> {
    match std::env::current_dir() {
        Ok(current_dir) => Ok(current_dir),
        Err(source) => resolve_pwd_directory().ok_or(WorkspaceResolutionError::CurrentDir(source)),
    }
}

fn resolve_pwd_directory() -> Option<PathBuf> {
    let pwd = std::env::var_os(PWD_ENV_VAR).map(PathBuf::from)?;
    if pwd.is_absolute() && pwd.is_dir() { Some(pwd) } else { None }
}

pub fn discover_workspace_root(start: &Path) -> Result<PathBuf, WorkspaceResolutionError> {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    // 2. Upward search for `.boundline/` directory.
    let boundline_candidates = search_upward_all_dirs(&start, ".boundline");
    if boundline_candidates.len() > 1 {
        return Err(WorkspaceResolutionError::Ambiguous(start));
    }
    if let Some(found) = boundline_candidates.into_iter().next() {
        return Ok(found);
    }

    // 3. Upward search for the nearest `.git` entry (directory or worktree file).
    if let Some(found) = search_upward_entry(&start, ".git") {
        return Ok(found);
    }

    // 4. Fall back to CWD.
    Ok(start)
}

/// Walk upward from `start` looking for any child entry named `target`.
/// Returns the parent directory that contains `target`, if found.
fn search_upward_entry(start: &Path, target: &str) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(target);
        if candidate.exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn search_upward_all_dirs(start: &Path, target: &str) -> Vec<PathBuf> {
    let mut current = start.to_path_buf();
    let mut matches = Vec::new();
    loop {
        let candidate = current.join(target);
        if candidate.is_dir() {
            matches.push(current.clone());
        }
        if !current.pop() {
            return matches;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::Path;

    use tempfile::tempdir;

    use super::{PWD_ENV_VAR, current_dir_or_pwd, discover_workspace_root, resolve_workspace};

    struct CurrentDirGuard {
        original: std::path::PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &std::path::Path) -> Self {
            let original = current_dir_or_pwd().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original).unwrap();
        }
    }

    struct PwdEnvGuard {
        original: Option<OsString>,
    }

    impl PwdEnvGuard {
        fn set(path: Option<&Path>) -> Self {
            let original = std::env::var_os(PWD_ENV_VAR);
            unsafe {
                match path {
                    Some(path) => std::env::set_var(PWD_ENV_VAR, path),
                    None => std::env::remove_var(PWD_ENV_VAR),
                };
            }
            Self { original }
        }
    }

    impl Drop for PwdEnvGuard {
        fn drop(&mut self) {
            match self.original.as_ref() {
                Some(value) => unsafe {
                    std::env::set_var(PWD_ENV_VAR, value);
                },
                None => unsafe {
                    std::env::remove_var(PWD_ENV_VAR);
                },
            }
        }
    }

    #[test]
    fn discover_workspace_root_prefers_boundline_root_over_git_root() {
        let temp = tempdir().unwrap();
        let git_root = temp.path().join("repo");
        let workspace_root = git_root.join("nested");
        let child = workspace_root.join("src/subdir");
        fs::create_dir_all(workspace_root.join(".boundline")).unwrap();
        fs::create_dir_all(git_root.join(".git")).unwrap();
        fs::create_dir_all(&child).unwrap();

        let resolved = discover_workspace_root(&child).unwrap();

        assert_eq!(resolved, workspace_root.canonicalize().unwrap());
    }

    #[test]
    fn discover_workspace_root_accepts_git_worktree_marker_files() {
        let temp = tempdir().unwrap();
        let git_root = temp.path().join("repo");
        let child = git_root.join("crates/canon-cli");
        fs::create_dir_all(&child).unwrap();
        fs::write(git_root.join(".git"), "gitdir: /tmp/worktree\n").unwrap();

        let resolved = discover_workspace_root(&child).unwrap();

        assert_eq!(resolved, git_root.canonicalize().unwrap());
    }

    #[test]
    fn resolve_workspace_treats_dot_as_repo_discovery() {
        let temp = tempdir().unwrap();
        let git_root = temp.path().join("repo");
        let child = git_root.join("src/subdir");
        fs::create_dir_all(&child).unwrap();
        fs::create_dir_all(git_root.join(".git")).unwrap();
        let _current_dir_guard = CurrentDirGuard::change_to(&child);

        let resolved = resolve_workspace(Some(std::path::Path::new("."))).unwrap();

        assert_eq!(resolved, git_root.canonicalize().unwrap());
    }

    #[test]
    fn resolve_workspace_uses_pwd_when_current_directory_is_unavailable() {
        let temp = tempdir().unwrap();
        let git_root = temp.path().join("repo");
        let child = git_root.join("src/subdir");
        let broken_workspace = temp.path().join("broken-cwd");
        fs::create_dir_all(&child).unwrap();
        fs::create_dir_all(git_root.join(".git")).unwrap();
        fs::create_dir_all(&broken_workspace).unwrap();
        let _current_dir_guard = CurrentDirGuard::change_to(&broken_workspace);
        fs::remove_dir_all(&broken_workspace).unwrap();
        let _pwd_guard = PwdEnvGuard::set(Some(&child));

        let resolved = resolve_workspace(None).unwrap();

        assert_eq!(resolved, git_root.canonicalize().unwrap());
    }
}
