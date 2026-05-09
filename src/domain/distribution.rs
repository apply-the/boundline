use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::domain::governance::{CANONICAL_MODES, CanonCapabilitySnapshot, CanonMode};

pub const SUPPORTED_CANON_VERSION: &str = "0.42.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionChannel {
    Homebrew,
    Winget,
    Source,
}

impl DistributionChannel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Homebrew => "homebrew",
            Self::Winget => "winget",
            Self::Source => "source",
        }
    }
}

impl fmt::Display for DistributionChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionState {
    Ready,
    AlreadySatisfied,
    Blocked,
    RepairNeeded,
}

impl CompanionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::AlreadySatisfied => "already_satisfied",
            Self::Blocked => "blocked",
            Self::RepairNeeded => "repair_needed",
        }
    }
}

impl fmt::Display for CompanionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonSurfaceVerification {
    pub canon_path: PathBuf,
    pub version_compatible: bool,
    pub operations_verified: bool,
    pub missing_operations: Vec<String>,
    pub modes_verified: bool,
    pub missing_modes: Vec<CanonMode>,
    pub unsupported_modes: Vec<String>,
    pub capability_snapshot: Option<CanonCapabilitySnapshot>,
    pub ready: bool,
    pub repair_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonInstallStatus {
    pub state: CompanionState,
    pub version: Option<String>,
    pub location: Option<PathBuf>,
    pub bundled_with_boundline: bool,
    pub message: String,
    pub suggested_actions: Vec<String>,
    pub surface_verification: Option<CanonSurfaceVerification>,
}

pub fn supported_distribution_channels() -> Vec<DistributionChannel> {
    let mut channels = Vec::new();
    if let Some(channel) = official_distribution_channel() {
        channels.push(channel);
    }
    channels.push(DistributionChannel::Source);
    channels
}

pub fn evaluate_canon_install(executable_path: &Path) -> CanonInstallStatus {
    canon_install_status_from_discovery(discover_canon_binary(executable_path))
}

/// Verify the Canon governance surface from a capability snapshot.
///
/// Checks that the Canon binary reports the required governance operations
/// (`start`, `refresh`) and all 15 canonical modes. Returns a
/// `CanonSurfaceVerification` describing readiness, gaps, and repair actions.
pub fn verify_canon_surface(
    canon_path: &Path,
    snapshot: &CanonCapabilitySnapshot,
) -> CanonSurfaceVerification {
    let required_operations = ["start", "refresh"];
    let missing_operations: Vec<String> = required_operations
        .iter()
        .filter(|op| !snapshot.operations.iter().any(|s| s == **op))
        .map(|op| op.to_string())
        .collect();
    let operations_verified = missing_operations.is_empty();

    let missing_modes: Vec<CanonMode> = CANONICAL_MODES
        .iter()
        .copied()
        .filter(|mode| !snapshot.supported_modes.contains(mode))
        .collect();
    let modes_verified = missing_modes.is_empty();

    let unsupported_modes: Vec<String> = snapshot
        .supported_modes
        .iter()
        .filter(|mode| !CANONICAL_MODES.contains(mode))
        .map(|mode| format!("{:?}", mode))
        .collect();

    let version_compatible = snapshot.canon_version == SUPPORTED_CANON_VERSION;

    let ready = version_compatible && operations_verified && modes_verified;

    let mut repair_actions = Vec::new();
    if !version_compatible {
        repair_actions.push(format!(
            "Canon version {} does not match supported version {SUPPORTED_CANON_VERSION}; upgrade or reinstall Boundline",
            snapshot.canon_version
        ));
    }
    if !operations_verified {
        repair_actions.push(format!(
            "Canon binary is missing governance operations: {}; upgrade Canon to {SUPPORTED_CANON_VERSION}",
            missing_operations.join(", ")
        ));
    }
    if !modes_verified {
        let mode_names: Vec<&str> =
            missing_modes.iter().map(|m| m.primary_document_name()).collect();
        repair_actions.push(format!(
            "Canon binary is missing canonical modes: {}; upgrade Canon to {SUPPORTED_CANON_VERSION}",
            mode_names.join(", ")
        ));
    }

    CanonSurfaceVerification {
        canon_path: canon_path.to_path_buf(),
        version_compatible,
        operations_verified,
        missing_operations,
        modes_verified,
        missing_modes,
        unsupported_modes,
        capability_snapshot: Some(snapshot.clone()),
        ready,
        repair_actions,
    }
}

#[cfg(test)]
fn evaluate_canon_install_with_path_dirs(
    executable_path: &Path,
    path_dirs: &[PathBuf],
) -> CanonInstallStatus {
    canon_install_status_from_discovery(discover_canon_binary_in_paths(executable_path, path_dirs))
}

fn canon_install_status_from_discovery(discovered: Option<(PathBuf, bool)>) -> CanonInstallStatus {
    let Some((canon_path, bundled_with_boundline)) = discovered else {
        return CanonInstallStatus {
            state: CompanionState::RepairNeeded,
            version: None,
            location: None,
            bundled_with_boundline: false,
            message: format!(
                "Canon {SUPPORTED_CANON_VERSION} was not found beside Boundline or on PATH"
            ),
            suggested_actions: repair_actions(),
            surface_verification: None,
        };
    };

    match read_canon_version(&canon_path) {
        Some(version) if version == SUPPORTED_CANON_VERSION => {
            let surface_verification = query_canon_capabilities(&canon_path)
                .map(|snapshot| verify_canon_surface(&canon_path, &snapshot));
            let surface_ready = surface_verification.as_ref().is_some_and(|surface| surface.ready);
            let mut suggested_actions = Vec::new();
            if let Some(surface) = surface_verification.as_ref() {
                suggested_actions.extend(surface.repair_actions.clone());
            } else {
                suggested_actions.push(format!(
                    "Canon at {} did not report governance capabilities; upgrade Canon to {SUPPORTED_CANON_VERSION}",
                    canon_path.display()
                ));
            }
            let state = if surface_ready {
                if bundled_with_boundline {
                    CompanionState::Ready
                } else {
                    CompanionState::AlreadySatisfied
                }
            } else {
                CompanionState::RepairNeeded
            };
            CanonInstallStatus {
                state,
                version: Some(version.clone()),
                location: Some(canon_path.clone()),
                bundled_with_boundline,
                message: if surface_ready {
                    if bundled_with_boundline {
                        format!(
                            "Bundled Canon {version} is available at {} with verified governance surface",
                            canon_path.display()
                        )
                    } else {
                        format!(
                            "Canon {version} is already available on PATH at {} with verified governance surface",
                            canon_path.display()
                        )
                    }
                } else {
                    format!(
                        "Canon {version} at {} does not expose the required governance surface",
                        canon_path.display()
                    )
                },
                suggested_actions: if surface_ready { Vec::new() } else { suggested_actions },
                surface_verification,
            }
        }
        Some(version) => CanonInstallStatus {
            state: CompanionState::RepairNeeded,
            version: Some(version.clone()),
            location: Some(canon_path.clone()),
            bundled_with_boundline,
            message: format!(
                "Canon {version} at {} is outside the supported Boundline window; expected {SUPPORTED_CANON_VERSION}",
                canon_path.display()
            ),
            suggested_actions: repair_actions(),
            surface_verification: None,
        },
        None => CanonInstallStatus {
            state: CompanionState::Blocked,
            version: None,
            location: Some(canon_path.clone()),
            bundled_with_boundline,
            message: format!(
                "Canon was found at {} but its version could not be determined",
                canon_path.display()
            ),
            suggested_actions: vec![format!(
                "run `{} --version` manually and reinstall or upgrade Boundline if the reported Canon version is not {SUPPORTED_CANON_VERSION}",
                canon_path.display()
            )],
            surface_verification: None,
        },
    }
}

const fn official_distribution_channel() -> Option<DistributionChannel> {
    if cfg!(all(target_os = "macos", any(target_arch = "aarch64", target_arch = "x86_64"))) {
        Some(DistributionChannel::Homebrew)
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some(DistributionChannel::Winget)
    } else {
        None
    }
}

fn discover_canon_binary(executable_path: &Path) -> Option<(PathBuf, bool)> {
    let path_dirs = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();

    discover_canon_binary_in_paths(executable_path, &path_dirs)
}

fn discover_canon_binary_in_paths(
    executable_path: &Path,
    path_dirs: &[PathBuf],
) -> Option<(PathBuf, bool)> {
    if let Some(path) = bundled_canon_path(executable_path) {
        return Some((path, true));
    }

    let binary_name = canon_binary_name();
    path_dirs
        .iter()
        .map(|directory| directory.join(binary_name))
        .find(|candidate| candidate.is_file())
        .map(|candidate| (candidate, false))
}

fn bundled_canon_path(executable_path: &Path) -> Option<PathBuf> {
    let binary_name = canon_binary_name();
    executable_path
        .parent()
        .map(|directory| directory.join(binary_name))
        .filter(|candidate| candidate.is_file())
}

fn canon_binary_name() -> &'static str {
    if cfg!(target_os = "windows") { "canon.exe" } else { "canon" }
}

fn read_canon_version(command_path: &Path) -> Option<String> {
    let output = Command::new(command_path).arg("--version").output().ok()?;
    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    extract_semver_token(&combined)
}

fn query_canon_capabilities(command_path: &Path) -> Option<CanonCapabilitySnapshot> {
    let output = Command::new(command_path)
        .arg("governance")
        .arg("capabilities")
        .arg("--json")
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

fn extract_semver_token(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .map(|token| token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.'))
        .find(|token| {
            token.chars().next().is_some_and(|ch| ch.is_ascii_digit())
                && token.contains('.')
                && token.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        })
        .map(ToOwned::to_owned)
}

fn repair_actions() -> Vec<String> {
    if let Some(channel) = official_distribution_channel() {
        vec![format!(
            "reinstall or upgrade Boundline via {channel} so the bundled Canon companion returns to {SUPPORTED_CANON_VERSION}"
        )]
    } else {
        vec![format!(
            "install Canon {SUPPORTED_CANON_VERSION} on PATH or continue with the documented source fallback"
        )]
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use super::{
        CompanionState, DistributionChannel, SUPPORTED_CANON_VERSION,
        evaluate_canon_install_with_path_dirs, extract_semver_token,
        supported_distribution_channels,
    };

    fn temp_dir(prefix: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[cfg(unix)]
    fn write_fake_canon(directory: &Path, version_output: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let command_path = directory.join("canon");
        let capabilities = serde_json::json!({
            "canon_version": SUPPORTED_CANON_VERSION,
            "supported_schema_versions": ["2026-02-01"],
            "operations": ["start", "refresh", "capabilities"],
            "supported_modes": [
                "requirements",
                "discovery",
                "system-shaping",
                "architecture",
                "backlog",
                "change",
                "implementation",
                "refactor",
                "review",
                "verification",
                "incident",
                "security-assessment",
                "system-assessment",
                "migration",
                "supply-chain-analysis"
            ],
            "status_values": ["governed_ready", "awaiting_approval", "blocked"],
            "approval_state_values": ["not_needed", "requested", "granted"],
            "packet_readiness_values": ["reusable", "pending", "incomplete"],
            "compatibility_notes": ["stable-json"]
        });
        fs::write(
            &command_path,
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo '{version_output}'\n  exit 0\nfi\nif [ \"$1\" = \"governance\" ] && [ \"$2\" = \"capabilities\" ]; then\n  printf '%s' '{}'\n  exit 0\nfi\nexit 1\n",
                capabilities
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&command_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&command_path, permissions).unwrap();
        command_path
    }

    #[test]
    fn extract_semver_token_finds_a_canon_version() {
        assert_eq!(
            extract_semver_token("canon version 0.42.0 (stable)"),
            Some(SUPPORTED_CANON_VERSION.to_string())
        );
    }

    #[test]
    fn distribution_channel_list_always_keeps_source_fallback() {
        let channels = supported_distribution_channels();

        assert!(channels.contains(&DistributionChannel::Source));
    }

    #[test]
    fn companion_state_string_values_are_stable() {
        assert_eq!(CompanionState::RepairNeeded.to_string(), "repair_needed");
        assert_eq!(CompanionState::AlreadySatisfied.to_string(), "already_satisfied");
    }

    #[cfg(unix)]
    #[test]
    fn evaluate_canon_install_prefers_bundled_binary_then_path_fallback() {
        let install_root = temp_dir("boundline-distribution-bundled");
        let path_root = temp_dir("boundline-distribution-path");
        let executable = install_root.join("boundline");
        fs::write(&executable, "").unwrap();
        let bundled =
            write_fake_canon(&install_root, &format!("canon version {SUPPORTED_CANON_VERSION}"));
        let path_binary =
            write_fake_canon(&path_root, &format!("canon version {SUPPORTED_CANON_VERSION}"));

        let bundled_status =
            evaluate_canon_install_with_path_dirs(&executable, std::slice::from_ref(&path_root));
        assert_eq!(bundled_status.state, CompanionState::Ready);
        assert_eq!(bundled_status.location.as_deref(), Some(bundled.as_path()));
        assert!(bundled_status.bundled_with_boundline);

        fs::remove_file(&bundled).unwrap();
        let path_status = evaluate_canon_install_with_path_dirs(&executable, &[path_root]);
        assert_eq!(path_status.state, CompanionState::AlreadySatisfied);
        assert_eq!(path_status.location.as_deref(), Some(path_binary.as_path()));
        assert!(!path_status.bundled_with_boundline);
    }

    #[cfg(unix)]
    #[test]
    fn evaluate_canon_install_reports_version_mismatch_and_unreadable_version() {
        use std::os::unix::fs::PermissionsExt;

        let executable_root = temp_dir("boundline-distribution-executable");
        let mismatch_root = temp_dir("boundline-distribution-mismatch");
        let blocked_root = temp_dir("boundline-distribution-blocked");
        let executable = executable_root.join("boundline");
        fs::write(&executable, "").unwrap();

        let mismatch_binary = write_fake_canon(&mismatch_root, "canon version 0.38.0");
        let mismatch_status = evaluate_canon_install_with_path_dirs(&executable, &[mismatch_root]);
        assert_eq!(mismatch_status.state, CompanionState::RepairNeeded);
        assert_eq!(mismatch_status.version.as_deref(), Some("0.38.0"));
        assert_eq!(mismatch_status.location.as_deref(), Some(mismatch_binary.as_path()));
        assert!(!mismatch_status.suggested_actions.is_empty());

        let blocked_binary = blocked_root.join("canon");
        fs::write(&blocked_binary, "#!/bin/sh\nexit 0\n").unwrap();
        let mut permissions = fs::metadata(&blocked_binary).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&blocked_binary, permissions).unwrap();

        let blocked_status = evaluate_canon_install_with_path_dirs(&executable, &[blocked_root]);
        assert_eq!(blocked_status.state, CompanionState::Blocked);
        assert_eq!(blocked_status.location.as_deref(), Some(blocked_binary.as_path()));
        assert!(blocked_status.message.contains("could not be determined"));
        assert!(blocked_status.suggested_actions[0].contains("--version"));
    }
}
