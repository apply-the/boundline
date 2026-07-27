//! Internal-handler integration coverage for the non-public exec capability.

use boundline::cli::{
    CommandExitStatus,
    exec::{ExecArgs, execute},
};

use crate::workspace_fixture::temp_empty_workspace;

fn exec_args(command: impl Into<String>) -> ExecArgs {
    ExecArgs {
        command: command.into(),
        dry_run: false,
        no_mutation: false,
        classify_only: false,
        zone: None,
        json: false,
    }
}

fn ensure(condition: bool, message: impl Into<String>) -> Result<(), String> {
    if condition { Ok(()) } else { Err(message.into()) }
}

#[test]
fn t024_exec_echo_hello_produces_evidence() -> Result<(), String> {
    let report = execute(exec_args("echo hello"), None);
    ensure(report.exit_status == CommandExitStatus::Succeeded, &report.terminal_output)?;
    ensure(report.terminal_output.contains("intent=read"), &report.terminal_output)?;
    ensure(report.terminal_output.contains("mode=allow"), &report.terminal_output)?;
    ensure(report.evidence.is_some(), "internal exec omitted evidence")
}

#[test]
fn t061_dry_run_rm_does_not_delete() -> Result<(), String> {
    let workspace = temp_empty_workspace("boundline-exec-internal-dry-run");
    let target = workspace.join("test.txt");
    std::fs::write(&target, "keep me").map_err(|error| error.to_string())?;
    let mut args = exec_args(format!("rm {}", target.display()));
    args.dry_run = true;
    let report = execute(args, Some(&workspace));
    ensure(report.exit_status == CommandExitStatus::Succeeded, &report.terminal_output)?;
    ensure(report.terminal_output.contains("dry_run_status=plan_only"), &report.terminal_output)?;
    ensure(target.exists(), "internal dry-run deleted its target")
}

#[test]
fn t062_redacted_evidence_hides_secret() -> Result<(), String> {
    let report = execute(exec_args("echo secret: ghp_abc12345678901234567890123456"), None);
    let evidence = report.evidence.ok_or_else(|| "internal exec omitted evidence".to_string())?;
    ensure(!evidence.stdout.contains("ghp_abc"), "secret leaked in evidence")?;
    ensure(!evidence.redaction_audit.is_empty(), "redaction audit was empty")
}

#[test]
fn t063_json_evidence_packet_has_required_fields() -> Result<(), String> {
    let mut args = exec_args("echo data");
    args.json = true;
    let report = execute(args, None);
    let evidence = report.evidence.ok_or_else(|| "internal exec omitted evidence".to_string())?;
    ensure(report.exit_status == CommandExitStatus::Succeeded, &report.terminal_output)?;
    ensure(!evidence.trace_id.is_empty(), "evidence trace ID was empty")?;
    ensure(evidence.exit_code == Some(0), "evidence exit code was not zero")
}
