//! Deterministic dry-run tiering.
//!
//! Maps commands to native safe modes or emits dry-run plans without
//! executing unknown mutating commands. V1 uses curated mappings —
//! never executes an unknown command just to discover mutations.

use super::classifier::DryRunStatus;

/// Mapped native dry-run equivalents for known commands.
const NATIVE_DRY_RUN_MAPPINGS: &[(&str, &str)] = &[
    ("cargo test", "cargo test --no-run"),
    ("cargo build", "cargo check"),
    ("npm install", "npm install --dry-run"),
    ("pnpm install", "pnpm install --dry-run"),
    ("terraform apply", "terraform plan"),
    ("kubectl apply", "kubectl diff -f -"),
    ("git push", "git push --dry-run"),
];

/// Determines the [`DryRunStatus`] for a given command string.
///
/// Returns `NativeDryRunExecuted` if the command has a known safe
/// mapping, `PlanOnly` if it is mutating but has no safe mapping,
/// or `ReadOnlyExecuted` if it is a read-only command.
pub fn classify_dry_run(command: &str, is_read_only: bool) -> DryRunStatus {
    if is_read_only {
        return DryRunStatus::ReadOnlyExecuted;
    }

    let cmd_lower = command.to_ascii_lowercase();
    for (pattern, _replacement) in NATIVE_DRY_RUN_MAPPINGS {
        if cmd_lower.starts_with(pattern) {
            return DryRunStatus::NativeDryRunExecuted;
        }
    }

    DryRunStatus::PlanOnly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_command_returns_read_only_executed() {
        assert_eq!(classify_dry_run("ls -la", true), DryRunStatus::ReadOnlyExecuted);
    }

    #[test]
    fn known_native_dry_run_mapping() {
        assert_eq!(classify_dry_run("cargo build", false), DryRunStatus::NativeDryRunExecuted);
        assert_eq!(
            classify_dry_run("terraform apply -auto-approve", false),
            DryRunStatus::NativeDryRunExecuted
        );
        assert_eq!(
            classify_dry_run("git push origin main", false),
            DryRunStatus::NativeDryRunExecuted
        );
        assert_eq!(
            classify_dry_run("kubectl apply -f pod.yaml", false),
            DryRunStatus::NativeDryRunExecuted
        );
    }

    #[test]
    fn mutating_command_without_native_dry_run_returns_plan_only() {
        assert_eq!(classify_dry_run("rm -rf /tmp/test", false), DryRunStatus::PlanOnly);
    }

    #[test]
    fn unknown_command_returns_plan_only() {
        assert_eq!(classify_dry_run("some-unknown-tool --flag", false), DryRunStatus::PlanOnly);
    }

    #[test]
    fn dry_run_status_as_str() {
        assert_eq!(DryRunStatus::NativeDryRunExecuted.as_str(), "native_dry_run_executed");
        assert_eq!(DryRunStatus::ReadOnlyExecuted.as_str(), "read_only_executed");
        assert_eq!(DryRunStatus::PlanOnly.as_str(), "plan_only");
        assert_eq!(DryRunStatus::UnsupportedForSafeDryRun.as_str(), "unsupported_for_safe_dry_run");
    }
}
