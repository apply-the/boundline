//! Integration tests for `boundline exec` — full pipeline from CLI through
//! classification, policy, evidence capture, and redaction.

use std::process::Command;

fn boundline_binary() -> String {
    env!("CARGO_BIN_EXE_boundline").to_string()
}

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("boundline-exec-test-{}-{}", prefix, std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_exec(cmd: &str) -> std::process::Output {
    Command::new(boundline_binary())
        .arg("exec")
        .arg(cmd)
        .output()
        .expect("failed to execute boundline exec")
}

fn run_exec_with(cmd: &str, flags: &[&str]) -> std::process::Output {
    let mut c = Command::new(boundline_binary());
    c.arg("exec");
    for f in flags {
        c.arg(f);
    }
    c.arg(cmd).output().expect("failed to execute boundline exec")
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Extracts the EvidencePacket from a `--json` host envelope.
fn extract_evidence(output: &std::process::Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: serde_json::Value =
        serde_json::from_str(&stdout).expect("valid JSON host envelope");
    let rendered = envelope["rendered_output"].as_str().expect("rendered_output field");
    serde_json::from_str(rendered).expect("valid JSON evidence packet")
}

#[test]
fn t024_exec_echo_hello_produces_evidence() {
    let output = run_exec("echo hello");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout: {}\nstderr: {}", stdout, stderr_of(&output));
    assert!(stdout.contains("intent=read"), "{}", stdout);
    assert!(stdout.contains("mode=allow"), "{}", stdout);
    assert!(stdout.contains("exit_code=0"), "{}", stdout);
    assert!(stdout.contains("trace_id="), "{}", stdout);
}

#[test]
fn t061_dry_run_rm_does_not_delete() {
    let dir = TempDir::new("dry-run");
    let test_file = dir.path().join("test.txt");
    std::fs::write(&test_file, "keep me").expect("write test file");
    assert!(test_file.exists());

    let cmd = format!("rm {}", test_file.display());
    let output = run_exec_with(&cmd, &["--dry-run"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout: {}\nstderr: {}", stdout, stderr_of(&output));
    assert!(stdout.contains("dry_run_status=plan_only"), "{}", stdout);
    assert!(test_file.exists(), "file was deleted by dry-run!");
}

#[test]
fn t062_redacted_evidence_hides_secret() {
    let output = run_exec_with("echo secret: ghp_abc12345678901234567890123456", &["--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout: {}\nstderr: {}", stdout, stderr_of(&output));
    let evidence = extract_evidence(&output);
    // The evidence packet's stdout field must not contain the raw token.
    let captured_stdout = evidence["stdout"].as_str().unwrap();
    assert!(
        !captured_stdout.contains("ghp_abc"),
        "secret leaked in evidence stdout: {}",
        captured_stdout
    );
    // Redaction audit should be non-empty.
    let audit = &evidence["redaction_audit"];
    assert!(!audit.as_array().unwrap().is_empty(), "no redaction recorded");
}

#[test]
fn t063_json_evidence_packet_has_required_fields() {
    let output = run_exec_with("echo data", &["--json"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "stdout: {}\nstderr: {}", stdout, stderr_of(&output));
    let evidence = extract_evidence(&output);
    assert_eq!(evidence["intent"].as_str().unwrap(), "read");
    assert!(evidence["trace_id"].as_str().is_some());
    assert_eq!(evidence["exit_code"].as_i64().unwrap(), 0);
}
