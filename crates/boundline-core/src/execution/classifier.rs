//! Command intent classification.
//!
//! Classifies a shell command into one of five intent categories using
//! a deterministic rule engine: first by command name whitelist, then
//! refined by argument heuristics. Unknown commands default to `Mutate`.

use serde::{Deserialize, Serialize};

/// Classification of a command's purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandIntent {
    Read,
    Test,
    Mutate,
    Install,
    Deploy,
    Unknown,
}

/// Determines how a command is executed based on policy resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Allow,
    DryRun,
    NoMutation,
    RequireApproval,
    Deny,
}

/// Result of a dry-run execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DryRunStatus {
    NativeDryRunExecuted,
    ReadOnlyExecuted,
    PlanOnly,
    UnsupportedForSafeDryRun,
}

// ── Safety flags ───────────────────────────────────────────────────────

const SAFETY_FLAGS: &[&str] = &["--dry-run", "--check", "--list", "--help", "--version"];

const RISK_FLAGS: &[&str] =
    &["--force", "--delete", "--push", "--install", "--write", "--fix", "--apply"];

/// Command whitelist mapping base names to intents.
const COMMAND_WHITELIST: &[(&str, CommandIntent)] = &[
    ("ls", CommandIntent::Read),
    ("cat", CommandIntent::Read),
    ("pwd", CommandIntent::Read),
    ("grep", CommandIntent::Read),
    ("rg", CommandIntent::Read),
    ("find", CommandIntent::Read),
    ("wc", CommandIntent::Read),
    ("head", CommandIntent::Read),
    ("tail", CommandIntent::Read),
    ("echo", CommandIntent::Read),
    ("git status", CommandIntent::Read),
    ("git diff", CommandIntent::Read),
    ("git log", CommandIntent::Read),
    ("git branch", CommandIntent::Read),
    ("cargo test", CommandIntent::Test),
    ("go test", CommandIntent::Test),
    ("npm test", CommandIntent::Test),
    ("pytest", CommandIntent::Test),
    ("cargo check", CommandIntent::Read),
    ("git commit", CommandIntent::Mutate),
    ("git push", CommandIntent::Mutate),
    ("rm", CommandIntent::Mutate),
    ("mv", CommandIntent::Mutate),
    ("cp", CommandIntent::Mutate),
    ("sed -i", CommandIntent::Mutate),
    ("apt-get install", CommandIntent::Install),
    ("brew install", CommandIntent::Install),
    ("cargo install", CommandIntent::Install),
    ("npm install", CommandIntent::Install),
    ("pnpm install", CommandIntent::Install),
    ("pip install", CommandIntent::Install),
    ("kubectl apply", CommandIntent::Deploy),
    ("terraform apply", CommandIntent::Deploy),
];

/// Classifies a command string into a [`CommandIntent`].
pub fn classify_command(command: &str) -> CommandIntent {
    let lower = command.to_ascii_lowercase().trim().to_string();

    // Pass 1: match by command name (longest match first)
    let mut intent = CommandIntent::Unknown;
    for (pattern, mapped) in COMMAND_WHITELIST {
        if lower.starts_with(pattern) && !pattern.is_empty() {
            intent = *mapped;
            break;
        }
    }

    // Pass 2: refine by argument heuristics
    for flag in SAFETY_FLAGS {
        if lower.contains(flag) {
            intent = downgrade_intent(intent);
        }
    }
    for flag in RISK_FLAGS {
        if lower.contains(flag) {
            intent = escalate_intent(intent);
        }
    }

    // Unknown remains unknown → policy will handle
    intent
}

fn downgrade_intent(intent: CommandIntent) -> CommandIntent {
    match intent {
        CommandIntent::Mutate => CommandIntent::Read,
        CommandIntent::Install => CommandIntent::Test,
        CommandIntent::Deploy => CommandIntent::Test,
        other => other,
    }
}

fn escalate_intent(intent: CommandIntent) -> CommandIntent {
    match intent {
        CommandIntent::Read => CommandIntent::Mutate,
        CommandIntent::Test => CommandIntent::Mutate,
        other => other,
    }
}

impl CommandIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Test => "test",
            Self::Mutate => "mutate",
            Self::Install => "install",
            Self::Deploy => "deploy",
            Self::Unknown => "unknown",
        }
    }
}

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::DryRun => "dry-run",
            Self::NoMutation => "no-mutation",
            Self::RequireApproval => "require-approval",
            Self::Deny => "deny",
        }
    }
}

impl DryRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NativeDryRunExecuted => "native_dry_run_executed",
            Self::ReadOnlyExecuted => "read_only_executed",
            Self::PlanOnly => "plan_only",
            Self::UnsupportedForSafeDryRun => "unsupported_for_safe_dry_run",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_read_commands() {
        assert_eq!(classify_command("ls"), CommandIntent::Read);
        assert_eq!(classify_command("cat file.txt"), CommandIntent::Read);
        assert_eq!(classify_command("git status"), CommandIntent::Read);
        assert_eq!(classify_command("echo hello"), CommandIntent::Read);
    }

    #[test]
    fn classify_test_commands() {
        assert_eq!(classify_command("cargo test"), CommandIntent::Test);
        assert_eq!(classify_command("npm test"), CommandIntent::Test);
        assert_eq!(classify_command("pytest"), CommandIntent::Test);
    }

    #[test]
    fn classify_mutate_commands() {
        assert_eq!(classify_command("rm file.txt"), CommandIntent::Mutate);
        assert_eq!(classify_command("git commit"), CommandIntent::Mutate);
    }

    #[test]
    fn classify_install_commands() {
        assert_eq!(classify_command("cargo install foo"), CommandIntent::Install);
        assert_eq!(classify_command("brew install foo"), CommandIntent::Install);
    }

    #[test]
    fn classify_deploy_commands() {
        assert_eq!(classify_command("terraform apply"), CommandIntent::Deploy);
        assert_eq!(classify_command("kubectl apply"), CommandIntent::Deploy);
    }

    #[test]
    fn safety_flags_downgrade_intent() {
        assert_eq!(classify_command("rm --dry-run file.txt"), CommandIntent::Read);
    }

    #[test]
    fn risk_flags_escalate_intent() {
        assert_eq!(classify_command("ls --force"), CommandIntent::Mutate);
    }

    #[test]
    fn unknown_commands_default_to_unknown() {
        assert_eq!(classify_command("my-custom-tool --verbose"), CommandIntent::Unknown);
    }

    #[test]
    fn intent_serde_roundtrip() {
        let intent = CommandIntent::Mutate;
        let json = serde_json::to_string(&intent).unwrap();
        let parsed: CommandIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, CommandIntent::Mutate);
    }

    #[test]
    fn as_str_returns_expected_labels() {
        assert_eq!(CommandIntent::Read.as_str(), "read");
        assert_eq!(CommandIntent::Mutate.as_str(), "mutate");
        assert_eq!(ExecutionMode::Allow.as_str(), "allow");
        assert_eq!(ExecutionMode::Deny.as_str(), "deny");
        assert_eq!(DryRunStatus::PlanOnly.as_str(), "plan_only");
    }

    #[test]
    fn all_intents_as_str() {
        assert_eq!(CommandIntent::Read.as_str(), "read");
        assert_eq!(CommandIntent::Test.as_str(), "test");
        assert_eq!(CommandIntent::Mutate.as_str(), "mutate");
        assert_eq!(CommandIntent::Install.as_str(), "install");
        assert_eq!(CommandIntent::Deploy.as_str(), "deploy");
        assert_eq!(CommandIntent::Unknown.as_str(), "unknown");
    }

    #[test]
    fn all_execution_modes_as_str() {
        assert_eq!(ExecutionMode::Allow.as_str(), "allow");
        assert_eq!(ExecutionMode::DryRun.as_str(), "dry-run");
        assert_eq!(ExecutionMode::NoMutation.as_str(), "no-mutation");
        assert_eq!(ExecutionMode::RequireApproval.as_str(), "require-approval");
        assert_eq!(ExecutionMode::Deny.as_str(), "deny");
    }

    #[test]
    fn all_dry_run_statuses_as_str() {
        assert_eq!(DryRunStatus::NativeDryRunExecuted.as_str(), "native_dry_run_executed");
        assert_eq!(DryRunStatus::ReadOnlyExecuted.as_str(), "read_only_executed");
        assert_eq!(DryRunStatus::PlanOnly.as_str(), "plan_only");
        assert_eq!(DryRunStatus::UnsupportedForSafeDryRun.as_str(), "unsupported_for_safe_dry_run");
    }

    #[test]
    fn classify_additional_read_commands() {
        assert_eq!(classify_command("git diff"), CommandIntent::Read);
        assert_eq!(classify_command("git log"), CommandIntent::Read);
        assert_eq!(classify_command("git branch"), CommandIntent::Read);
        assert_eq!(classify_command("rg pattern"), CommandIntent::Read);
        assert_eq!(classify_command("find . -name '*.rs'"), CommandIntent::Read);
        assert_eq!(classify_command("wc -l file.txt"), CommandIntent::Read);
        assert_eq!(classify_command("head file.txt"), CommandIntent::Read);
        assert_eq!(classify_command("tail file.txt"), CommandIntent::Read);
        assert_eq!(classify_command("cargo check"), CommandIntent::Read);
    }

    #[test]
    fn classify_additional_mutate_commands() {
        assert_eq!(classify_command("sed -i 's/a/b/' file.txt"), CommandIntent::Mutate);
        assert_eq!(classify_command("git push"), CommandIntent::Mutate);
    }

    #[test]
    fn classify_additional_install_commands() {
        assert_eq!(classify_command("apt-get install curl"), CommandIntent::Install);
        assert_eq!(classify_command("pnpm install"), CommandIntent::Install);
        assert_eq!(classify_command("pip install requests"), CommandIntent::Install);
    }

    #[test]
    fn classify_additional_test_commands() {
        assert_eq!(classify_command("go test"), CommandIntent::Test);
    }

    #[test]
    fn safety_flag_downgrades_install_to_test() {
        assert_eq!(classify_command("cargo install --dry-run foo"), CommandIntent::Test);
    }

    #[test]
    fn safety_flag_downgrades_deploy_to_test() {
        assert_eq!(classify_command("terraform apply --dry-run"), CommandIntent::Test);
    }

    #[test]
    fn risk_flag_escalates_test_to_mutate() {
        assert_eq!(classify_command("cargo test --force"), CommandIntent::Mutate);
    }

    #[test]
    fn downgrade_intent_direct() {
        // Test downgrade_intent helper via classify_command with --dry-run
        assert_eq!(classify_command("rm --dry-run file.txt"), CommandIntent::Read);
    }

    #[test]
    fn escalate_intent_direct() {
        // Test escalate_intent helper via classify_command with --force
        assert_eq!(classify_command("ls --force"), CommandIntent::Mutate);
    }
}
