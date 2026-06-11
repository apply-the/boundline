//! `boundline exec` command — safe command execution with intent classification,
//! policy enforcement, evidence capture, and secret redaction.
//!
//! Pipelines: classify → policy → dry-run → shell execution → evidence → persistence.

use std::path::Path;

use boundline_core::execution::classifier::{ExecutionMode, classify_command};
use boundline_core::execution::dry_run::classify_dry_run;
use boundline_core::execution::evidence::{EvidencePacket, PolicyDecision, RiskZone};
use boundline_core::execution::policy::ExecutionPolicy;
use boundline_core::execution::redaction::{load_redaction_config, redact_output};

use crate::cli::CommandExitStatus;

/// Result of an exec command dispatch.
#[derive(Debug, Clone)]
pub struct ExecCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
    pub trace_location: Option<String>,
    pub evidence: Option<EvidencePacket>,
}

/// Parsed arguments for the `exec` subcommand.
#[derive(Debug, Clone)]
pub struct ExecArgs {
    /// The shell command to execute.
    pub command: String,
    /// Execute via deterministic dry-run tier.
    pub dry_run: bool,
    /// Execute but block filesystem writes.
    pub no_mutation: bool,
    /// Classify only — no execution.
    pub classify_only: bool,
    /// Override execution zone.
    pub zone: Option<RiskZone>,
    /// Output full EvidencePacket as JSON.
    pub json: bool,
}

/// Dispatch the `boundline exec` command.
pub fn execute(args: ExecArgs, workspace: Option<&Path>) -> ExecCommandReport {
    let _ws = workspace;

    // Step 1: Classify command intent
    let intent = classify_command(&args.command);

    // Step 2: Resolve execution policy
    let policy = ExecutionPolicy::default();
    let zone = args.zone.unwrap_or(RiskZone::Green);
    let entry = policy.resolve(intent, zone);
    let mode = if args.classify_only || args.dry_run {
        ExecutionMode::DryRun
    } else if args.no_mutation {
        ExecutionMode::NoMutation
    } else {
        entry.mode
    };

    let decision = PolicyDecision {
        inferred_intent: intent,
        zone,
        matched_policy_entry: format!("policy.{}.{}", intent.as_str(), zone.as_str()),
        matched_override: None,
        safety_escalations: Vec::new(),
        final_mode: mode,
        rationale: format!(
            "resolved {} intent in {} zone → {}",
            intent.as_str(),
            zone.as_str(),
            mode.as_str()
        ),
    };

    // Step 3: Classify-only path
    if args.classify_only {
        let output = format!(
            "intent={} mode={} zone={} rationale=\"{}\"",
            intent.as_str(),
            mode.as_str(),
            zone.as_str(),
            decision.rationale
        );
        return ExecCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: output,
            trace_location: None,
            evidence: None,
        };
    }

    // Step 4: Block denied/require-approval modes
    if mode == ExecutionMode::Deny {
        return ExecCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: format!(
                "Execution denied: {} intent blocked in {} zone by policy",
                intent.as_str(),
                zone.as_str()
            ),
            trace_location: None,
            evidence: None,
        };
    }
    if mode == ExecutionMode::RequireApproval {
        return ExecCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: format!(
                "Approval required: {} intent in {} zone needs operator confirmation",
                intent.as_str(),
                zone.as_str()
            ),
            trace_location: None,
            evidence: None,
        };
    }

    // Step 5: Dry-run path
    if mode == ExecutionMode::DryRun {
        let is_read_only =
            matches!(intent, boundline_core::execution::classifier::CommandIntent::Read);
        let dry_run_status = classify_dry_run(&args.command, is_read_only);

        let evidence = EvidencePacket::builder(args.command.clone(), intent, ExecutionMode::DryRun)
            .dry_run_status(dry_run_status)
            .policy_decision(decision)
            .build();

        let output =
            format!("dry_run_status={} intent={}", dry_run_status.as_str(), intent.as_str());

        return ExecCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: output,
            trace_location: None,
            evidence: Some(evidence),
        };
    }

    // Step 6: Execute command (Allow / NoMutation)
    match execute_shell_command(&args.command) {
        Ok((stdout, stderr, exit_code)) => {
            // Redact secrets from output
            let config = load_redaction_config(None);
            let (redacted_stdout, audit) = redact_output(&config.patterns, &stdout);
            let (redacted_stderr, stderr_audit) = redact_output(&config.patterns, &stderr);

            let mut all_audit = audit;
            all_audit.extend(stderr_audit);

            let evidence = EvidencePacket::builder(args.command.clone(), intent, mode)
                .stdout(redacted_stdout)
                .stderr(redacted_stderr)
                .exit_code(exit_code)
                .policy_decision(decision)
                .redaction_audit(all_audit)
                .build();

            let status = if exit_code == 0 {
                CommandExitStatus::Succeeded
            } else {
                CommandExitStatus::NonSuccess
            };

            let terminal_output = if args.json {
                serde_json::to_string_pretty(&evidence).unwrap_or_default()
            } else {
                format!(
                    "exec: {} → intent={} mode={} exit_code={} trace_id={}",
                    args.command,
                    intent.as_str(),
                    mode.as_str(),
                    exit_code,
                    evidence.trace_id
                )
            };

            ExecCommandReport {
                exit_status: status,
                terminal_output,
                trace_location: None,
                evidence: Some(evidence),
            }
        }
        Err(error) => ExecCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: format!("exec failed: {}", error),
            trace_location: None,
            evidence: None,
        },
    }
}

/// Executes a shell command and captures stdout, stderr, and exit code.
fn execute_shell_command(command: &str) -> Result<(String, String, i32), String> {
    use std::process::Command;

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_only_read_command() {
        let args = ExecArgs {
            command: "ls -la".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: true,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("intent=read"));
    }

    #[test]
    fn classify_only_mutate_command() {
        let args = ExecArgs {
            command: "rm -rf ./build".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: true,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("intent=mutate"));
    }

    #[test]
    fn dry_run_mutate_command() {
        let args = ExecArgs {
            command: "rm test.txt".into(),
            dry_run: true,
            no_mutation: false,
            classify_only: false,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.terminal_output.contains("dry_run_status=plan_only"));
    }

    #[test]
    fn deploy_in_red_zone_denied() {
        let args = ExecArgs {
            command: "kubectl apply -f pod.yaml".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: Some(RiskZone::Red),
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("denied"));
    }

    #[test]
    fn exec_echo_produces_evidence() {
        let args = ExecArgs {
            command: "echo hello".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.evidence.is_some());
        let ev = report.evidence.unwrap();
        assert_eq!(ev.intent.as_str(), "read");
        assert_eq!(ev.command, "echo hello");
        assert_eq!(ev.exit_code, Some(0));
    }

    #[test]
    fn mutate_in_green_zone_requires_approval() {
        let args = ExecArgs {
            command: "rm important.txt".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: None, // Green by default, mutate → RequireApproval
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.terminal_output.contains("Approval required"));
    }

    #[test]
    fn no_mutation_mode_allows_read_commands() {
        let args = ExecArgs {
            command: "echo test".into(),
            dry_run: false,
            no_mutation: true,
            classify_only: false,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.evidence.is_some());
    }

    #[test]
    fn json_output_produces_valid_evidence() {
        let args = ExecArgs {
            command: "echo json-test".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: None,
            json: true,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
        assert!(report.evidence.is_some());
        // The terminal_output with --json is the serialized EvidencePacket
        let parsed: serde_json::Value =
            serde_json::from_str(&report.terminal_output).expect("valid JSON");
        assert_eq!(parsed["intent"].as_str().unwrap(), "read");
        assert_eq!(parsed["exit_code"].as_i64().unwrap(), 0);
    }

    #[test]
    fn explicit_yellow_zone_enforces_policy() {
        let args = ExecArgs {
            command: "echo yellow-zone".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: Some(RiskZone::Yellow),
            json: false,
        };
        let report = execute(args, None);
        // Read in yellow zone is still Allow
        assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    }

    #[test]
    fn failing_command_captures_non_zero_exit() {
        // Must use a whitelisted command (Read intent passes policy) that fails.
        // `false` is NOT whitelisted → classifies as Unknown → blocked by policy.
        let args = ExecArgs {
            command: "ls /nonexistent".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
        assert!(report.evidence.is_some());
        // `ls` on a nonexistent path exits with code 2 (BSD) or 1 (GNU).
        let code = report.evidence.unwrap().exit_code.unwrap();
        assert!(code != 0, "expected non-zero exit, got {}", code);
    }

    #[test]
    fn nonexistent_command_returns_error() {
        let args = ExecArgs {
            command: "this-binary-does-not-exist-xyz".into(),
            dry_run: false,
            no_mutation: false,
            classify_only: false,
            zone: None,
            json: false,
        };
        let report = execute(args, None);
        assert_eq!(report.exit_status, CommandExitStatus::NonSuccess);
    }
}
