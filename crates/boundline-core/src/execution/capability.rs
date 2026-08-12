//! One fail-closed capability service governs providers, tools, and adapters.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use thiserror::Error;

/// Git authority requested by an executor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitAuthority {
    /// Inspect repository state without mutation.
    Read,
    /// Modify the Git index.
    WriteIndex,
    /// Create a Git commit.
    Commit,
    /// Advance or replace a Git ref.
    UpdateRef,
    /// Publish into the authoritative checkout.
    Publish,
}

/// Typed capability request evaluated before executor side effects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityRequest {
    /// Read one worktree path.
    ReadPath(PathBuf),
    /// Write one worktree path.
    WritePath(PathBuf),
    /// Run an exact executable and argument vector.
    Command { executable: String, arguments: Vec<String> },
    /// Read a non-secret environment key.
    Environment(String),
    /// Inherit a secret.
    Secret(String),
    /// Contact a network host.
    Network(String),
    /// Leave work running after invocation completion.
    BackgroundProcess,
    /// Create a number of processes.
    ProcessCount(u32),
    /// Emit a number of output bytes.
    OutputBytes(u64),
    /// Consume an execution interval.
    TimeoutMillis(u64),
    /// Exercise Git authority.
    Git(GitAuthority),
    /// Access the protected Boundline state root.
    StateRoot(PathBuf),
}

/// Deterministic admission result with a stable denial category.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityAdmission {
    /// The exact request appears in the invocation grant.
    Admitted,
    /// The request exceeds the invocation grant.
    Denied { reason: CapabilityDenial },
}

/// Stable classes of capability denial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityDenial {
    /// Path escaped its allowlist.
    Path,
    /// Command or arguments were not admitted.
    Command,
    /// Environment key was not admitted.
    Environment,
    /// Secret inheritance was requested.
    Secret,
    /// Network destination was not admitted.
    Network,
    /// Background processing was requested.
    Process,
    /// Process or time budget was exceeded.
    Resource,
    /// Output budget was exceeded.
    Output,
    /// Git mutation authority was requested.
    GitAuthority,
    /// State-root access was requested.
    StateRoot,
}

/// Immutable capability policy attached to one invocation.
#[derive(Clone, Debug)]
pub struct CapabilityPolicy {
    worktree: PathBuf,
    state_root: PathBuf,
    read_roots: Vec<PathBuf>,
    write_roots: Vec<PathBuf>,
    commands: BTreeMap<String, Vec<String>>,
    environment: BTreeSet<String>,
    network_hosts: BTreeSet<String>,
    max_processes: u32,
    max_output_bytes: u64,
    timeout_millis: u64,
}

impl CapabilityPolicy {
    /// Starts a policy builder with no executor authority admitted.
    pub fn builder(
        worktree: impl AsRef<Path>,
        state_root: impl AsRef<Path>,
    ) -> CapabilityPolicyBuilder {
        CapabilityPolicyBuilder::new(worktree.as_ref(), state_root.as_ref())
    }

    /// Evaluates exactly one requested capability.
    pub fn admit(&self, request: &CapabilityRequest) -> CapabilityAdmission {
        let denial = match request {
            CapabilityRequest::ReadPath(path) if !self.path_admitted(path, &self.read_roots) => {
                Some(CapabilityDenial::Path)
            }
            CapabilityRequest::WritePath(path) if !self.path_admitted(path, &self.write_roots) => {
                Some(CapabilityDenial::Path)
            }
            CapabilityRequest::Command { executable, arguments }
                if self.commands.get(executable) != Some(arguments) =>
            {
                Some(CapabilityDenial::Command)
            }
            CapabilityRequest::Environment(name) if !self.environment.contains(name) => {
                Some(CapabilityDenial::Environment)
            }
            CapabilityRequest::Secret(_) => Some(CapabilityDenial::Secret),
            CapabilityRequest::Network(host) if !self.network_hosts.contains(host) => {
                Some(CapabilityDenial::Network)
            }
            CapabilityRequest::BackgroundProcess => Some(CapabilityDenial::Process),
            CapabilityRequest::ProcessCount(count) if *count > self.max_processes => {
                Some(CapabilityDenial::Resource)
            }
            CapabilityRequest::OutputBytes(bytes) if *bytes > self.max_output_bytes => {
                Some(CapabilityDenial::Output)
            }
            CapabilityRequest::TimeoutMillis(milliseconds)
                if *milliseconds > self.timeout_millis =>
            {
                Some(CapabilityDenial::Resource)
            }
            CapabilityRequest::Git(authority) if *authority != GitAuthority::Read => {
                Some(CapabilityDenial::GitAuthority)
            }
            CapabilityRequest::StateRoot(_) => Some(CapabilityDenial::StateRoot),
            _ => None,
        };
        denial
            .map_or(CapabilityAdmission::Admitted, |reason| CapabilityAdmission::Denied { reason })
    }

    fn path_admitted(&self, path: &Path, allowed: &[PathBuf]) -> bool {
        let Some(path) = normalize_requested_path(path) else {
            return false;
        };
        if path.starts_with(&self.state_root) || !path.starts_with(&self.worktree) {
            return false;
        }
        allowed.iter().any(|root| path.starts_with(root))
            && path.components().all(|component| !matches!(component, Component::ParentDir))
    }
}

fn normalize_requested_path(path: &Path) -> Option<PathBuf> {
    if path.components().any(|component| matches!(component, Component::ParentDir)) {
        return None;
    }
    if std::fs::symlink_metadata(path).is_ok() {
        return std::fs::canonicalize(path).ok();
    }
    let parent = std::fs::canonicalize(path.parent()?).ok()?;
    Some(parent.join(path.file_name()?))
}

/// Builder that makes every authority grant explicit.
#[derive(Clone, Debug)]
pub struct CapabilityPolicyBuilder {
    worktree: PathBuf,
    state_root: PathBuf,
    read_roots: Vec<PathBuf>,
    write_roots: Vec<PathBuf>,
    commands: BTreeMap<String, Vec<String>>,
    environment: BTreeSet<String>,
    network_hosts: BTreeSet<String>,
    max_processes: u32,
    max_output_bytes: u64,
    timeout_millis: u64,
}

impl CapabilityPolicyBuilder {
    fn new(worktree: &Path, state_root: &Path) -> Self {
        Self {
            worktree: worktree.to_path_buf(),
            state_root: state_root.to_path_buf(),
            read_roots: Vec::new(),
            write_roots: Vec::new(),
            commands: BTreeMap::new(),
            environment: BTreeSet::new(),
            network_hosts: BTreeSet::new(),
            max_processes: 0,
            max_output_bytes: 0,
            timeout_millis: 0,
        }
    }
    pub fn allow_read_path(mut self, path: PathBuf) -> Self {
        self.read_roots.push(path);
        self
    }
    pub fn allow_write_path(mut self, path: PathBuf) -> Self {
        self.write_roots.push(path);
        self
    }
    pub fn allow_command<I, S>(mut self, executable: impl Into<String>, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.commands.insert(executable.into(), arguments.into_iter().map(Into::into).collect());
        self
    }
    pub fn allow_environment(mut self, name: impl Into<String>) -> Self {
        self.environment.insert(name.into());
        self
    }
    pub fn allow_network_host(mut self, host: impl Into<String>) -> Self {
        self.network_hosts.insert(host.into());
        self
    }
    pub const fn limits(mut self, processes: u32, output_bytes: u64, timeout_millis: u64) -> Self {
        self.max_processes = processes;
        self.max_output_bytes = output_bytes;
        self.timeout_millis = timeout_millis;
        self
    }
    pub fn build(mut self) -> Result<CapabilityPolicy, CapabilityPolicyError> {
        self.worktree = std::fs::canonicalize(&self.worktree)?;
        self.state_root = std::fs::canonicalize(&self.state_root)?;
        self.read_roots = resolve_roots(&self.worktree, self.read_roots)?;
        self.write_roots = resolve_roots(&self.worktree, self.write_roots)?;
        Ok(CapabilityPolicy {
            worktree: self.worktree,
            state_root: self.state_root,
            read_roots: self.read_roots,
            write_roots: self.write_roots,
            commands: self.commands,
            environment: self.environment,
            network_hosts: self.network_hosts,
            max_processes: self.max_processes,
            max_output_bytes: self.max_output_bytes,
            timeout_millis: self.timeout_millis,
        })
    }
}

/// Capability policy construction errors.
#[derive(Debug, Error)]
pub enum CapabilityPolicyError {
    #[error("capability root I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("capability root escaped the worktree")]
    PathEscape,
}

fn resolve_roots(
    worktree: &Path,
    roots: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, CapabilityPolicyError> {
    roots
        .into_iter()
        .map(|root| {
            if root.is_absolute()
                || root.components().any(|component| matches!(component, Component::ParentDir))
            {
                return Err(CapabilityPolicyError::PathEscape);
            }
            Ok(worktree.join(root))
        })
        .collect()
}
