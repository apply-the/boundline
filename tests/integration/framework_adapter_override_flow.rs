//! Integration coverage for selective framework-adapter stage overrides and hook delivery.

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use boundline::adapters::{
    FrameworkAdapterEnvelopeError, FrameworkAdapterErrorEnvelope, FrameworkAdapterFailureClass,
    FrameworkAdapterResponseEnvelope, FrameworkAdapterStageExecutionStatus,
};
use boundline::domain::session::SessionStatus;
use boundline::fixture::{
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_hook_emission_response,
    sample_framework_adapter_preflight_ready_response, sample_framework_adapter_success_envelope,
};
use boundline::{FileSessionStore, SessionStore};
use serde_json::json;

use crate::framework_adapter::SPECKIT_BINARY_NAME;
use crate::workspace_fixture::{
    run_boundline_in_with_env, supported_canon_path, target_test_dir, temp_fixture_workspace,
    terminal_text,
};

const BUG_FIX_FLOW: &str = "bug-fix";
const FIX_GOAL: &str = "fix the failing add test";
const NATIVE_ROUTING_SUMMARY: &str = "routing: native (goal_plan)";

#[test]
fn declared_run_stage_emits_stage_completed_hook_after_claimed_success()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-override-hook-success");
    let adapter_dir = target_test_dir("framework-adapter-override-hook-success-bin");
    let run_marker = workspace.join("adapter-run-marker.txt");
    let hook_request_path = workspace.join("adapter-stage-completed-hook.json");
    write_hook_observing_run_adapter(&adapter_dir, &run_marker, &hook_request_path)?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let add_text = terminal_text(&add);
    assert_eq!(add.status.code(), Some(0), "{add_text}");

    let goal = run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(!run_marker.exists(), "{plan_text}");

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_marker.is_file(), "{run_text}");
    assert!(!hook_request_path.exists(), "{run_text}");
    assert!(!run_text.contains(NATIVE_ROUTING_SUMMARY), "{run_text}");
    assert!(run_text.contains("framework_adapter_execution_source: adapter"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage: run"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_claim: completed"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_status: succeeded"), "{run_text}");
    assert!(run_text.contains("completion_verification_state: failed"), "{run_text}");
    assert!(
        run_text.contains("completion_verification_required_action: rerun_proof"),
        "{run_text}"
    );

    let status = run_boundline_in_with_env(&workspace, &["status"], &[("PATH", path_env.as_str())]);
    let status_text = terminal_text(&status);
    assert_eq!(status.status.code(), Some(0), "{status_text}");
    assert!(status_text.contains("latest_status: blocked"), "{status_text}");
    assert!(status_text.contains("framework_adapter_execution_source: adapter"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage: run"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_claim: completed"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_status: succeeded"), "{status_text}");
    assert!(
        status_text.contains("framework_adapter_routing_reason: declared_override"),
        "{status_text}"
    );
    assert!(status_text.contains("completion_verification_state: failed"), "{status_text}");
    assert!(!status_text.contains("framework_adapter_hook: stage_completed"), "{status_text}");

    let inspect = run_boundline_in_with_env(
        &workspace,
        &["inspect", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let inspect_text = terminal_text(&inspect);
    assert_eq!(inspect.status.code(), Some(1), "{inspect_text}");
    assert!(inspect_text.contains("terminal_status: running"), "{inspect_text}");
    assert!(inspect_text.contains("latest_status: running"), "{inspect_text}");
    assert!(inspect_text.contains("next_command: /boundline-next"), "{inspect_text}");

    Ok(())
}

#[test]
fn claimed_run_stage_blocked_is_reported_as_blocked_without_native_routing()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-override-run-blocked");
    let adapter_dir = target_test_dir("framework-adapter-override-run-blocked-bin");
    let run_marker = workspace.join("adapter-run-marker-blocked.txt");
    let hook_request_path = workspace.join("adapter-stage-blocked-hook.json");
    write_run_adapter_with_behavior(
        &adapter_dir,
        &run_marker,
        &hook_request_path,
        ExecuteStageBehavior::Blocked,
    )?;
    let path_env = adapter_path(&adapter_dir);

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

    let goal = run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

    let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
    let run_text = terminal_text(&run);
    assert_eq!(run.status.code(), Some(1), "{run_text}");
    assert!(run_marker.is_file(), "{run_text}");
    assert!(!run_text.contains(NATIVE_ROUTING_SUMMARY), "{run_text}");
    assert!(run_text.contains("latest_status: blocked"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage: run"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_claim: claimed"), "{run_text}");
    assert!(run_text.contains("framework_adapter_stage_status: blocked"), "{run_text}");
    assert!(!hook_request_path.exists(), "{run_text}");

    let status = run_boundline_in_with_env(&workspace, &["status"], &[("PATH", path_env.as_str())]);
    let status_text = terminal_text(&status);
    assert!(status_text.contains("latest_status: blocked"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage: run"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_claim: claimed"), "{status_text}");
    assert!(status_text.contains("framework_adapter_stage_status: blocked"), "{status_text}");

    let session = FileSessionStore::for_workspace(&workspace)
        .load()?
        .ok_or("expected an active session after blocked run stage")?;
    assert_eq!(session.latest_status, SessionStatus::Blocked);
    assert!(session.latest_terminal_reason.is_some());

    Ok(())
}

#[test]
fn claimed_run_stage_failures_are_classified_for_run_status_and_inspect()
-> Result<(), Box<dyn Error>> {
    for behavior in [
        ExecuteStageBehavior::InBandFailed,
        ExecuteStageBehavior::ProtocolError,
        ExecuteStageBehavior::TransportFailure,
    ] {
        let workspace =
            temp_fixture_workspace(&format!("framework-adapter-override-{}", behavior.label()));
        let adapter_dir =
            target_test_dir(&format!("framework-adapter-override-{}-bin", behavior.label()));
        let run_marker = workspace.join(format!("adapter-run-marker-{}.txt", behavior.label()));
        let hook_request_path =
            workspace.join(format!("adapter-stage-failed-hook-{}.json", behavior.label()));
        write_run_adapter_with_behavior(&adapter_dir, &run_marker, &hook_request_path, behavior)?;
        let path_env = adapter_path(&adapter_dir);

        let add = run_boundline_in_with_env(
            &workspace,
            &["adapter", "add", "speckit", "--workspace", "."],
            &[("PATH", path_env.as_str())],
        );
        assert_eq!(add.status.code(), Some(0), "{}", terminal_text(&add));

        let goal = run_boundline_in_with_env(
            &workspace,
            &["goal", "--goal", FIX_GOAL],
            &[("PATH", path_env.as_str())],
        );
        assert_eq!(goal.status.code(), Some(0), "{}", terminal_text(&goal));

        let plan = run_boundline_in_with_env(
            &workspace,
            &["plan", "--flow", BUG_FIX_FLOW],
            &[("PATH", path_env.as_str())],
        );
        assert_eq!(plan.status.code(), Some(0), "{}", terminal_text(&plan));

        let run = run_boundline_in_with_env(&workspace, &["run"], &[("PATH", path_env.as_str())]);
        let run_text = terminal_text(&run);
        assert_eq!(run.status.code(), Some(1), "{} => {run_text}", behavior.label());
        assert!(run_marker.is_file(), "{} => {run_text}", behavior.label());
        assert!(run_text.contains("terminal_status: failed"), "{} => {run_text}", behavior.label());
        assert!(
            run_text.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{} => {run_text}",
            behavior.label()
        );
        assert!(
            run_text.contains("framework_adapter_stage_claim: failed_after_claim"),
            "{} => {run_text}",
            behavior.label()
        );

        let status =
            run_boundline_in_with_env(&workspace, &["status"], &[("PATH", path_env.as_str())]);
        let status_text = terminal_text(&status);
        assert!(
            status_text.contains("latest_status: failed"),
            "{} => {status_text}",
            behavior.label()
        );
        assert!(
            status_text.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{} => {status_text}",
            behavior.label()
        );

        let inspect = run_boundline_in_with_env(
            &workspace,
            &["inspect", "--workspace", "."],
            &[("PATH", path_env.as_str())],
        );
        let inspect_text = terminal_text(&inspect);
        assert_eq!(inspect.status.code(), Some(1), "{} => {inspect_text}", behavior.label());
        assert!(
            inspect_text.contains("terminal_status: failed"),
            "{} => {inspect_text}",
            behavior.label()
        );
        assert!(
            inspect_text.contains(&format!(
                "framework_adapter_failure_class: {}",
                behavior.expected_failure_class()
            )),
            "{} => {inspect_text}",
            behavior.label()
        );
        assert!(inspect_text.contains("audit: count="), "{} => {inspect_text}", behavior.label());

        let hook_request = fs::read_to_string(&hook_request_path)?;
        assert!(
            hook_request.contains("\"hook_key\":\"stage_failed\""),
            "{} => {hook_request}",
            behavior.label()
        );
        assert!(
            hook_request.contains("\"stage_key\":\"run\""),
            "{} => {hook_request}",
            behavior.label()
        );
        assert!(
            hook_request.contains("\"stage_claimed\":true"),
            "{} => {hook_request}",
            behavior.label()
        );
    }

    Ok(())
}

fn adapter_path(adapter_dir: &Path) -> String {
    format!("{}:{}", adapter_dir.display(), supported_canon_path())
}

#[derive(Clone, Copy)]
enum ExecuteStageBehavior {
    Succeeded,
    Blocked,
    InBandFailed,
    ProtocolError,
    TransportFailure,
}

impl ExecuteStageBehavior {
    fn label(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Blocked => "blocked",
            Self::InBandFailed => "in-band-failed",
            Self::ProtocolError => "protocol-error",
            Self::TransportFailure => "transport-failure",
        }
    }

    fn expected_failure_class(self) -> &'static str {
        match self {
            Self::Succeeded | Self::Blocked | Self::InBandFailed => "adapter_runtime",
            Self::ProtocolError => "protocol_error",
            Self::TransportFailure => "transport_failure",
        }
    }

    fn declared_hook_subscription(self) -> &'static str {
        match self {
            Self::Succeeded => "stage_completed",
            Self::Blocked | Self::InBandFailed | Self::ProtocolError | Self::TransportFailure => {
                "stage_failed"
            }
        }
    }
}

fn write_hook_observing_run_adapter(
    adapter_dir: &Path,
    run_marker: &Path,
    hook_request_path: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    write_run_adapter_with_behavior(
        adapter_dir,
        run_marker,
        hook_request_path,
        ExecuteStageBehavior::Succeeded,
    )
}

fn write_run_adapter_with_behavior(
    adapter_dir: &Path,
    run_marker: &Path,
    hook_request_path: &Path,
    behavior: ExecuteStageBehavior,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["declared_stage_overrides"] = json!(["run"]);
    describe["declared_hook_subscriptions"] = json!([behavior.declared_hook_subscription()]);
    let describe_json =
        serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?;
    let preflight_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_preflight_ready_response(),
    ))?;
    let execute_stage_script = execute_stage_script(run_marker, behavior)?;
    let emit_hook_json = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_hook_emission_response(),
    ))?;

    fs::create_dir_all(adapter_dir)?;
    let binary_path = adapter_dir.join(SPECKIT_BINARY_NAME);
    let script = format!(
        "#!/bin/sh\nset -eu\nconsume_stdin() {{\n  stdin_line=''\n  while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n    stdin_line=''\n  done\n}}\nconsume_stdin_to_file() {{\n  target_path=$1\n  : > \"$target_path\"\n  stdin_line=''\n  while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n    printf '%s\\n' \"$stdin_line\" >> \"$target_path\"\n    stdin_line=''\n  done\n}}\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\ncase \"$1\" in\n  describe)\n    print_json <<'BOUNDLINE_JSON'\n{describe_json}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    consume_stdin\n    print_json <<'BOUNDLINE_JSON'\n{preflight_json}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n    consume_stdin\n{execute_stage_script}\n    ;;\n  emit-hook)\n    consume_stdin_to_file \"{}\"\n    print_json <<'BOUNDLINE_JSON'\n{emit_hook_json}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n",
        hook_request_path.to_string_lossy(),
    );
    fs::write(&binary_path, script)?;
    let mut permissions = fs::metadata(&binary_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions)?;
    Ok(binary_path)
}

fn execute_stage_script(
    run_marker: &Path,
    behavior: ExecuteStageBehavior,
) -> Result<String, Box<dyn Error>> {
    let run_marker_ref = run_marker.to_string_lossy();

    match behavior {
        ExecuteStageBehavior::Succeeded => {
            let execute_stage_json =
                serde_json::to_string(&sample_framework_adapter_success_envelope(
                    sample_framework_adapter_execute_stage_success_response(),
                ))?;
            Ok(format!(
                "    : > \"{run_marker_ref}\"\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON"
            ))
        }
        ExecuteStageBehavior::Blocked => {
            let mut response = sample_framework_adapter_execute_stage_success_response();
            response.status = FrameworkAdapterStageExecutionStatus::Blocked;
            response.summary =
                "framework-adapter blocked the claimed stage pending operator action".to_string();
            let execute_stage_json =
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))?;
            Ok(format!(
                "    : > \"{run_marker_ref}\"\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON"
            ))
        }
        ExecuteStageBehavior::InBandFailed => {
            let mut response = sample_framework_adapter_execute_stage_success_response();
            response.status = FrameworkAdapterStageExecutionStatus::Failed;
            response.failure_class = Some(FrameworkAdapterFailureClass::AdapterRuntime);
            response.summary = "adapter reported a claimed run-stage failure".to_string();
            let execute_stage_json =
                serde_json::to_string(&sample_framework_adapter_success_envelope(response))?;
            Ok(format!(
                "    : > \"{run_marker_ref}\"\n    printf '%s\\n' 'adapter stderr: in-band failed outcome' >&2\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON"
            ))
        }
        ExecuteStageBehavior::ProtocolError => {
            let execute_stage_json = serde_json::to_string(&FrameworkAdapterResponseEnvelope::<
                serde_json::Value,
            >::Error(
                FrameworkAdapterErrorEnvelope {
                    success: false,
                    error: FrameworkAdapterEnvelopeError {
                        code: "invalid_request".to_string(),
                        message: "execute-stage payload was rejected".to_string(),
                        details: None,
                    },
                },
            ))?;
            Ok(format!(
                "    : > \"{run_marker_ref}\"\n    printf '%s\\n' 'adapter stderr: protocol error envelope' >&2\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage_json}\nBOUNDLINE_JSON"
            ))
        }
        ExecuteStageBehavior::TransportFailure => Ok(format!(
            "    : > \"{run_marker_ref}\"\n    printf '%s\\n' 'adapter stderr: transport failure' >&2\n    exit 23"
        )),
    }
}
