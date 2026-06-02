use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use boundline::adapters::agent::{
    FrameworkAdapterHost, FrameworkAdapterHostError, SubprocessFrameworkAdapterHost,
};
use boundline::adapters::{
    FrameworkAdapterDescribeResponse, FrameworkAdapterEnvelopeError, FrameworkAdapterErrorEnvelope,
    FrameworkAdapterFailureClass, FrameworkAdapterResponseEnvelope,
    FrameworkAdapterResponseEnvelopeError, FrameworkAdapterStageExecutionStatus,
    FrameworkAdapterSuccessEnvelope,
};
use boundline::fixture::{
    pretty_fixture_json, round_trip_fixture, sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_failed_response,
    sample_framework_adapter_execute_stage_request,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_hook_emission_request,
    sample_framework_adapter_hook_emission_response,
    sample_framework_adapter_preflight_blocked_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_preflight_request,
    sample_framework_adapter_success_envelope,
};
use serde_json::Value;
use uuid::Uuid;

const EXPECTED_PROTOCOL_LINE: &str = "framework-adapter-v1";
const EXPECTED_ADAPTER_ID: &str = "speckit";
const EXPECTED_ADAPTER_VERSION: &str = "0.66.0";
const EXPECTED_STAGE_PLAN: &str = "plan";
const EXPECTED_STAGE_RUN: &str = "run";
const EXPECTED_HOOK_COMPLETED: &str = "stage_completed";
const EXPECTED_HOOK_FAILED: &str = "stage_failed";
const EXPECTED_PRELIGHT_READY: &str = "ready";
const EXPECTED_PRELIGHT_BLOCKED: &str = "blocked";
const EXPECTED_STAGE_FAILED: &str = "failed";
const EXPECTED_STAGE_SUCCEEDED: &str = "succeeded";
const EXPECTED_HOOK_DELIVERED: &str = "delivered";
const EXPECTED_FIELD_KEY: &str = "template_repo";
const EXPECTED_TRANSPORT: &str = "stdio";
const EXPECTED_ENCODING: &str = "json";
const EXPECTED_REQUEST_CHANNEL: &str = "stdin";
const EXPECTED_RESPONSE_CHANNEL: &str = "stdout";
const EXPECTED_FAILURE_CLASS: &str = "adapter_runtime";
const EXPECTED_SUCCESS_ENVELOPE_ERROR: &str = "success envelope must set success=true";
const EXPECTED_ERROR_ENVELOPE_ERROR: &str = "error envelope must set success=false";
const EXPECTED_NO_DIAGNOSTICS_DETAIL: &str =
    "subprocess returned a non-zero exit status without diagnostics";

#[test]
fn describe_fixture_contract_uses_expected_protocol_fields() -> Result<(), Box<dyn Error>> {
    let document = serde_json::to_value(sample_framework_adapter_describe_response())?;

    expect_string(&document, "/protocol_line", EXPECTED_PROTOCOL_LINE)?;
    expect_string(&document, "/adapter_id", EXPECTED_ADAPTER_ID)?;
    expect_string(&document, "/supported_transports/0/transport", EXPECTED_TRANSPORT)?;
    expect_string(&document, "/supported_transports/0/encoding", EXPECTED_ENCODING)?;
    expect_string(&document, "/supported_transports/0/request_channel", EXPECTED_REQUEST_CHANNEL)?;
    expect_string(
        &document,
        "/supported_transports/0/response_channel",
        EXPECTED_RESPONSE_CHANNEL,
    )?;
    expect_string(&document, "/declared_stage_overrides/0", EXPECTED_STAGE_PLAN)?;
    expect_string(&document, "/declared_stage_overrides/1", EXPECTED_STAGE_RUN)?;
    expect_string(&document, "/declared_hook_subscriptions/0", EXPECTED_HOOK_COMPLETED)?;
    expect_string(&document, "/declared_hook_subscriptions/1", EXPECTED_HOOK_FAILED)?;
    expect_string(&document, "/required_config_fields/0/field_key", EXPECTED_FIELD_KEY)?;

    Ok(())
}

#[test]
fn preflight_fixtures_round_trip_through_json() -> Result<(), Box<dyn Error>> {
    let request = sample_framework_adapter_preflight_request();
    let ready = sample_framework_adapter_preflight_ready_response();
    let blocked = sample_framework_adapter_preflight_blocked_response();

    let decoded_request = round_trip_fixture(&request)?;
    let decoded_ready = round_trip_fixture(&ready)?;
    let decoded_blocked = round_trip_fixture(&blocked)?;

    if decoded_request != request {
        return Err("preflight request fixture changed during round trip".into());
    }

    if decoded_ready != ready {
        return Err("preflight ready fixture changed during round trip".into());
    }

    if decoded_blocked != blocked {
        return Err("preflight blocked fixture changed during round trip".into());
    }

    let ready_document = serde_json::to_value(decoded_ready)?;
    let blocked_document = serde_json::to_value(decoded_blocked)?;
    expect_string(&ready_document, "/status", EXPECTED_PRELIGHT_READY)?;
    expect_string(&blocked_document, "/status", EXPECTED_PRELIGHT_BLOCKED)?;
    expect_string(&blocked_document, "/missing_fields/0", EXPECTED_FIELD_KEY)?;

    Ok(())
}

#[test]
fn stage_and_hook_fixtures_render_stable_operator_json() -> Result<(), Box<dyn Error>> {
    let stage_request = sample_framework_adapter_execute_stage_request();
    let stage_response = sample_framework_adapter_execute_stage_success_response();
    let hook_request = sample_framework_adapter_hook_emission_request();
    let hook_response = sample_framework_adapter_hook_emission_response();

    let rendered_stage_request = pretty_fixture_json(&stage_request)?;
    let rendered_stage_response = pretty_fixture_json(&stage_response)?;
    let rendered_hook_request = pretty_fixture_json(&hook_request)?;
    let rendered_hook_response = pretty_fixture_json(&hook_response)?;

    ensure_contains(&rendered_stage_request, EXPECTED_STAGE_PLAN)?;
    ensure_contains(&rendered_stage_response, EXPECTED_STAGE_SUCCEEDED)?;
    ensure_contains(&rendered_hook_request, EXPECTED_HOOK_COMPLETED)?;
    ensure_contains(&rendered_hook_response, EXPECTED_HOOK_DELIVERED)?;

    Ok(())
}

#[test]
fn failed_execute_stage_fixture_round_trips_failure_metadata() -> Result<(), Box<dyn Error>> {
    let failed = sample_framework_adapter_execute_stage_failed_response();
    let decoded_failed = round_trip_fixture(&failed)?;

    if decoded_failed != failed {
        return Err("failed execute-stage fixture changed during round trip".into());
    }

    let document = serde_json::to_value(decoded_failed)?;
    expect_string(&document, "/status", EXPECTED_STAGE_FAILED)?;
    expect_string(&document, "/failure_class", EXPECTED_FAILURE_CLASS)?;
    let next_action = document
        .pointer("/next_action")
        .and_then(Value::as_str)
        .ok_or("expected failed execute-stage fixture to expose next_action")?;
    if next_action.trim().is_empty() {
        return Err("failed execute-stage fixture next_action should not be empty".into());
    }

    Ok(())
}

#[test]
fn bootstrap_describe_response_uses_stdio_defaults_and_empty_claims() -> Result<(), Box<dyn Error>>
{
    let document = serde_json::to_value(FrameworkAdapterDescribeResponse::bootstrap(
        EXPECTED_ADAPTER_ID,
        EXPECTED_ADAPTER_VERSION,
    ))?;

    expect_string(&document, "/protocol_line", EXPECTED_PROTOCOL_LINE)?;
    expect_string(&document, "/adapter_id", EXPECTED_ADAPTER_ID)?;
    expect_string(&document, "/adapter_version", EXPECTED_ADAPTER_VERSION)?;
    expect_string(&document, "/supported_transports/0/transport", EXPECTED_TRANSPORT)?;
    expect_string(&document, "/supported_transports/0/encoding", EXPECTED_ENCODING)?;
    assert_eq!(
        document.pointer("/declared_stage_overrides").and_then(Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(
        document.pointer("/declared_hook_subscriptions").and_then(Value::as_array).map(Vec::len),
        Some(0)
    );
    assert_eq!(
        document.pointer("/required_config_fields").and_then(Value::as_array).map(Vec::len),
        Some(0)
    );

    Ok(())
}

#[test]
fn response_envelopes_reject_invalid_success_flags() -> Result<(), Box<dyn Error>> {
    let success_envelope =
        FrameworkAdapterResponseEnvelope::Success(FrameworkAdapterSuccessEnvelope {
            success: false,
            data: serde_json::json!({"ok": true}),
        });
    match success_envelope.into_result() {
        Err(FrameworkAdapterResponseEnvelopeError::InvalidEnvelope { detail }) => {
            assert_eq!(detail, EXPECTED_SUCCESS_ENVELOPE_ERROR);
        }
        other => return Err(format!("expected invalid success envelope, got {other:?}").into()),
    }

    let error_envelope =
        FrameworkAdapterResponseEnvelope::<Value>::Error(FrameworkAdapterErrorEnvelope {
            success: true,
            error: FrameworkAdapterEnvelopeError {
                code: "invalid_manifest".to_string(),
                message: "manifest rejected".to_string(),
                details: None,
            },
        });
    match error_envelope.into_result() {
        Err(FrameworkAdapterResponseEnvelopeError::InvalidEnvelope { detail }) => {
            assert_eq!(detail, EXPECTED_ERROR_ENVELOPE_ERROR);
        }
        other => return Err(format!("expected invalid error envelope, got {other:?}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_constructor_validates_command_text() -> Result<(), Box<dyn Error>> {
    let error =
        SubprocessFrameworkAdapterHost::new("   ").err().ok_or("blank command should fail")?;
    assert!(matches!(error, FrameworkAdapterHostError::EmptyCommand));

    let host = SubprocessFrameworkAdapterHost::new("/bin/sh")?;
    assert_eq!(host.command(), "/bin/sh");

    Ok(())
}

#[test]
fn subprocess_host_surfaces_default_transport_failure_detail_without_diagnostics()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-empty-diagnostics");
    let script_path = write_protocol_script(&workspace, "#!/bin/sh\nexit 9\n")?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.describe().unwrap_err() {
        FrameworkAdapterHostError::ProcessFailed { request_kind, detail, .. } => {
            assert_eq!(request_kind, "describe");
            assert_eq!(detail, EXPECTED_NO_DIAGNOSTICS_DETAIL);
        }
        other => return Err(format!("expected process failure, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_surfaces_invalid_envelope_when_success_flag_is_wrong()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-invalid-envelope");
    let script_path = write_describe_script(
        &workspace,
        serde_json::to_string(&FrameworkAdapterResponseEnvelope::Success(
            FrameworkAdapterSuccessEnvelope {
                success: false,
                data: sample_framework_adapter_describe_response(),
            },
        ))?,
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.describe().unwrap_err() {
        FrameworkAdapterHostError::InvalidEnvelope { request_kind, detail, .. } => {
            assert_eq!(request_kind, "describe");
            assert_eq!(detail, EXPECTED_SUCCESS_ENVELOPE_ERROR);
        }
        other => return Err(format!("expected invalid envelope error, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_preflight_surfaces_write_request_when_stdin_closes_early()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-write-request");
    let script_path = write_protocol_script(
        &workspace,
        "#!/bin/sh\ncase \"$1\" in\n  preflight)\n    exec 0<&-\n    sleep 0.1\n    exit 0\n    ;;\n  *)\n    exit 64\n    ;;\nesac\n",
    )?;
    let host = fixture_host(&script_path, &workspace)?;
    let mut request = sample_framework_adapter_preflight_request();
    request.workspace_ref = "workspace/".repeat(65_536);

    match host.preflight(&request).unwrap_err() {
        FrameworkAdapterHostError::WriteRequest { request_kind, .. } => {
            assert_eq!(request_kind, "preflight");
        }
        other => return Err(format!("expected write request failure, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_invokes_all_protocol_commands_against_fixture_binary()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host");
    let script_path = write_fixture_protocol_script(&workspace)?;
    let host = fixture_host(&script_path, &workspace)?;

    assert_eq!(host.describe()?, sample_framework_adapter_describe_response());
    assert_eq!(
        host.preflight(&sample_framework_adapter_preflight_request())?,
        sample_framework_adapter_preflight_ready_response()
    );
    assert_eq!(
        host.execute_stage(&sample_framework_adapter_execute_stage_request())?,
        sample_framework_adapter_execute_stage_success_response()
    );
    assert_eq!(
        host.emit_hook(&sample_framework_adapter_hook_emission_request())?,
        sample_framework_adapter_hook_emission_response()
    );

    Ok(())
}

#[test]
fn subprocess_host_surfaces_non_zero_exit_as_transport_failure() -> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-failure");
    let script_path = write_failing_protocol_script(&workspace, "adapter transport failed")?;
    let host = fixture_host(&script_path, &workspace)?;

    let error = host.describe().unwrap_err().to_string();
    ensure_contains(&error, "describe")?;
    ensure_contains(&error, "adapter transport failed")?;

    Ok(())
}

#[test]
fn subprocess_host_surfaces_protocol_error_envelope_even_with_stderr() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_protocol_workspace("framework-adapter-host-protocol-error");
    let script_path = write_protocol_error_script(
        &workspace,
        "adapter trace line",
        FrameworkAdapterResponseEnvelope::<Value>::Error(
            boundline::adapters::FrameworkAdapterErrorEnvelope {
                success: false,
                error: FrameworkAdapterEnvelopeError {
                    code: "invalid_manifest".to_string(),
                    message: "describe rejected by adapter".to_string(),
                    details: Some(serde_json::json!({"field": "supported_transports"})),
                },
            },
        ),
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.describe().unwrap_err() {
        FrameworkAdapterHostError::ProtocolError {
            request_kind, code, message, details, ..
        } => {
            assert_eq!(request_kind, "describe");
            assert_eq!(code, "invalid_manifest");
            assert_eq!(message, "describe rejected by adapter");
            assert_eq!(
                details.map(|value| *value),
                Some(serde_json::json!({"field": "supported_transports"}))
            );
        }
        other => return Err(format!("expected protocol error, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_execute_stage_preserves_in_band_failed_outcome_even_with_stderr()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-stage-failed");
    let mut response = sample_framework_adapter_execute_stage_success_response();
    response.status = FrameworkAdapterStageExecutionStatus::Failed;
    response.summary = "adapter reported a domain failure".to_string();
    response.failure_class = Some(FrameworkAdapterFailureClass::AdapterRuntime);
    response.next_action = Some("inspect adapter-owned artifacts".to_string());
    let script_path = write_execute_stage_script(
        &workspace,
        Some("adapter trace line"),
        Some(serde_json::to_string(&sample_framework_adapter_success_envelope(response.clone()))?),
        0,
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    assert_eq!(host.execute_stage(&sample_framework_adapter_execute_stage_request())?, response);

    Ok(())
}

#[test]
fn subprocess_host_execute_stage_surfaces_protocol_error_even_with_stderr()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-stage-protocol-error");
    let script_path = write_execute_stage_script(
        &workspace,
        Some("adapter trace line"),
        Some(serde_json::to_string(&FrameworkAdapterResponseEnvelope::<Value>::Error(
            boundline::adapters::FrameworkAdapterErrorEnvelope {
                success: false,
                error: FrameworkAdapterEnvelopeError {
                    code: "stage_contract_error".to_string(),
                    message: "execute-stage rejected by adapter".to_string(),
                    details: Some(serde_json::json!({"stage_key": "run"})),
                },
            },
        ))?),
        0,
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.execute_stage(&sample_framework_adapter_execute_stage_request()).unwrap_err() {
        FrameworkAdapterHostError::ProtocolError {
            request_kind, code, message, details, ..
        } => {
            assert_eq!(request_kind, "execute-stage");
            assert_eq!(code, "stage_contract_error");
            assert_eq!(message, "execute-stage rejected by adapter");
            assert_eq!(details.map(|value| *value), Some(serde_json::json!({"stage_key": "run"})));
        }
        other => return Err(format!("expected protocol error, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_execute_stage_surfaces_transport_failure_even_with_stderr()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-stage-transport-failure");
    let script_path =
        write_execute_stage_script(&workspace, Some("adapter transport failed"), None, 9)?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.execute_stage(&sample_framework_adapter_execute_stage_request()).unwrap_err() {
        FrameworkAdapterHostError::ProcessFailed { request_kind, detail, .. } => {
            assert_eq!(request_kind, "execute-stage");
            assert_eq!(detail, "adapter transport failed");
        }
        other => return Err(format!("expected transport failure, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_rejects_describe_manifest_with_unknown_stage_override()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-invalid-stage-manifest");
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["declared_stage_overrides"] = serde_json::json!(["plan", "ship"]);
    let script_path = write_describe_script(
        &workspace,
        serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?,
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.describe().unwrap_err() {
        FrameworkAdapterHostError::DeserializeResponse { request_kind, .. } => {
            assert_eq!(request_kind, "describe");
        }
        other => return Err(format!("expected malformed manifest rejection, got {other}").into()),
    }

    Ok(())
}

#[test]
fn subprocess_host_rejects_describe_manifest_with_unknown_hook_subscription()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_protocol_workspace("framework-adapter-host-invalid-hook-manifest");
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["declared_hook_subscriptions"] = serde_json::json!(["stage_completed", "notify"]);
    let script_path = write_describe_script(
        &workspace,
        serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?,
    )?;
    let host = fixture_host(&script_path, &workspace)?;

    match host.describe().unwrap_err() {
        FrameworkAdapterHostError::DeserializeResponse { request_kind, .. } => {
            assert_eq!(request_kind, "describe");
        }
        other => return Err(format!("expected malformed manifest rejection, got {other}").into()),
    }

    Ok(())
}

fn expect_string(document: &Value, pointer: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = document
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string at json pointer {pointer}"))?;

    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected} at {pointer}, got {actual}").into())
    }
}

fn ensure_contains(rendered: &str, expected_fragment: &str) -> Result<(), Box<dyn Error>> {
    if rendered.contains(expected_fragment) {
        Ok(())
    } else {
        Err(format!("expected rendered json to contain {expected_fragment}: {rendered}").into())
    }
}

fn fixture_host(
    script_path: &Path,
    workspace: &Path,
) -> Result<SubprocessFrameworkAdapterHost, Box<dyn Error>> {
    Ok(SubprocessFrameworkAdapterHost::new("/bin/sh")?
        .with_args(vec![script_path.to_string_lossy().into_owned()])
        .with_working_directory(workspace.to_path_buf()))
}

fn temp_protocol_workspace(prefix: &str) -> PathBuf {
    let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    let _ = fs::create_dir_all(&workspace);
    workspace
}

fn write_fixture_protocol_script(workspace: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let describe = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_describe_response(),
    ))?;
    let preflight = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_preflight_ready_response(),
    ))?;
    let execute_stage = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_execute_stage_success_response(),
    ))?;
    let emit_hook = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_hook_emission_response(),
    ))?;

    let script = format!(
        "#!/bin/sh\nconsume_stdin() {{\n  stdin_line=''\n  while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n    stdin_line=''\n  done\n}}\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\ncase \"$1\" in\n  describe)\n    print_json <<'BOUNDLINE_JSON'\n{describe}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    consume_stdin\n    print_json <<'BOUNDLINE_JSON'\n{preflight}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n    consume_stdin\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage}\nBOUNDLINE_JSON\n    ;;\n  emit-hook)\n    consume_stdin\n    print_json <<'BOUNDLINE_JSON'\n{emit_hook}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );

    write_protocol_script(workspace, script.as_str())
}

fn write_describe_script(
    workspace: &Path,
    describe_stdout: String,
) -> Result<PathBuf, Box<dyn Error>> {
    let script = format!(
        "#!/bin/sh\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\ncase \"$1\" in\n  describe)\n    print_json <<'BOUNDLINE_JSON'\n{describe_stdout}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );
    write_protocol_script(workspace, script.as_str())
}

fn write_failing_protocol_script(
    workspace: &Path,
    stderr_line: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let script = format!("#!/bin/sh\necho \"{stderr_line}\" >&2\nexit 9\n");
    write_protocol_script(workspace, script.as_str())
}

fn write_protocol_error_script(
    workspace: &Path,
    stderr_line: &str,
    envelope: FrameworkAdapterResponseEnvelope<Value>,
) -> Result<PathBuf, Box<dyn Error>> {
    let stdout_json = serde_json::to_string(&envelope)?;
    let script = format!(
        "#!/bin/sh\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\necho \"{stderr_line}\" >&2\nprint_json <<'BOUNDLINE_JSON'\n{stdout_json}\nBOUNDLINE_JSON\n"
    );
    write_protocol_script(workspace, script.as_str())
}

fn write_execute_stage_script(
    workspace: &Path,
    stderr_line: Option<&str>,
    stdout_json: Option<String>,
    exit_code: i32,
) -> Result<PathBuf, Box<dyn Error>> {
    let stderr_lines = stderr_line.map(|line| format!("echo \"{line}\" >&2\n")).unwrap_or_default();
    let stdout_block = stdout_json
        .map(|json| format!("print_json <<'BOUNDLINE_JSON'\n{json}\nBOUNDLINE_JSON\n"))
        .unwrap_or_default();
    let script = format!(
        "#!/bin/sh\nconsume_stdin() {{\n  stdin_line=''\n  while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n    stdin_line=''\n  done\n}}\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\ncase \"$1\" in\n  execute-stage)\n    consume_stdin\n    {stderr_lines}{stdout_block}    exit {exit_code}\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );
    write_protocol_script(workspace, script.as_str())
}

fn write_protocol_script(workspace: &Path, body: &str) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(workspace)?;
    let path = workspace.join(format!("adapter-fixture-{}.sh", Uuid::new_v4()));
    fs::write(&path, body)?;
    Ok(path)
}
