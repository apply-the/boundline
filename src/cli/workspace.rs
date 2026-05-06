use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceResolutionError {
    #[error("failed to determine current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("ambiguous workspace: multiple .boundline/ directories found above {0}")]
    Ambiguous(PathBuf),
}

/// Resolve the workspace directory using the spec-required strategy:
///
/// 1. Use `--workspace <path>` when supplied.
/// 2. Search upward from CWD for an existing `.boundline/` directory; use its parent.
/// 3. Search upward for the nearest `.git/` (git root) directory; use its parent.
/// 4. Fall back to the current working directory.
pub fn resolve_workspace(workspace: Option<&Path>) -> Result<PathBuf, WorkspaceResolutionError> {
    // 1. Explicit workspace.
    if let Some(path) = workspace {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        return Ok(abs.canonicalize().unwrap_or(abs));
    }

    let cwd = std::env::current_dir()?;

    // 2. Upward search for `.boundline/` directory.
    let boundline_candidates = search_upward_all(&cwd, ".boundline");
    if boundline_candidates.len() > 1 {
        return Err(WorkspaceResolutionError::Ambiguous(cwd));
    }
    if let Some(found) = boundline_candidates.into_iter().next() {
        return Ok(found);
    }

    // 3. Upward search for `.git/` directory (git root).
    if let Some(found) = search_upward(&cwd, ".git") {
        return Ok(found);
    }

    // 4. Fall back to CWD.
    Ok(cwd)
}

/// Walk upward from `start` looking for a child directory named `target`.
/// Returns the parent directory that contains `target`, if found.
fn search_upward(start: &Path, target: &str) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(target);
        if candidate.is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn search_upward_all(start: &Path, target: &str) -> Vec<PathBuf> {
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
