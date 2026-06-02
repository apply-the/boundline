use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use boundline::adapters::config_store::FileConfigStore;
use boundline::adapters::{
    FrameworkAdapterPreflightBlockReason, FrameworkAdapterPreflightResponse,
    FrameworkAdapterPreflightStatus,
};
use boundline::cli::adapter::{
    AdapterConfigInteractor, AddAdapterRequest, RemoveAdapterRequest, ShowAdapterRequest,
};
use boundline::cli::{Cli, CommandExitStatus, DeveloperCommandSession};
use boundline::fixture::{
    sample_framework_adapter_describe_response, sample_framework_adapter_preflight_ready_response,
    sample_framework_adapter_success_envelope,
};
use clap::Parser;
use uuid::Uuid;

use crate::framework_adapter::{SPECKIT_ADAPTER_ID, temp_framework_adapter_workspace};

static PATH_ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn acquire_path_env_lock() -> MutexGuard<'static, ()> {
    PATH_ENV_MUTEX.get_or_init(|| Mutex::new(())).lock().unwrap()
}

struct PathGuard {
    saved: Option<std::ffi::OsString>,
    _lock: MutexGuard<'static, ()>,
}

#[derive(Debug)]
struct ScriptedAdapterConfigInteractor {
    inputs: Vec<Result<String, String>>,
}

impl ScriptedAdapterConfigInteractor {
    fn new(inputs: Vec<Result<String, String>>) -> Self {
        Self { inputs }
    }
}

impl AdapterConfigInteractor for ScriptedAdapterConfigInteractor {
    fn input(&mut self, _prompt: &str, _initial: &str, _secret: bool) -> Result<String, String> {
        if self.inputs.is_empty() {
            return Err("missing scripted adapter input".to_string());
        }
        self.inputs.remove(0)
    }
}

impl PathGuard {
    fn set(path_value: &std::ffi::OsStr) -> Self {
        let lock = acquire_path_env_lock();
        let saved = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", path_value);
        }
        Self { saved, _lock: lock }
    }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.saved {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
    }
}

#[test]
fn adapter_add_known_profile_accepts_workspace_command_args_and_json() {
    let cli = Cli::try_parse_from([
        "boundline",
        "adapter",
        "add",
        "speckit",
        "--workspace",
        "/tmp/workspace",
        "--command",
        "boundline-adapter-speckit",
        "--arg",
        "--profile",
        "--set",
        "template_repo=../boundline-framework-template",
        "--non-interactive",
        "--json",
    ])
    .unwrap();

    let session = DeveloperCommandSession::from_command(cli.command.as_ref().unwrap());
    assert_eq!(
        session.workspace_ref,
        Some(PathBuf::from("/tmp/workspace").to_string_lossy().into_owned())
    );
    assert!(session.goal.is_none());
    assert!(!session.install_check);
}

#[test]
fn adapter_add_custom_profile_accepts_required_identifiers() {
    let cli = Cli::try_parse_from([
        "boundline",
        "adapter",
        "add",
        "custom",
        "--workspace",
        "/tmp/workspace",
        "--id",
        "custom-demo",
        "--command",
        "./bin/custom-adapter",
        "--set",
        "adapter_repo=../boundline-adapter-speckit",
    ])
    .unwrap();

    let session = DeveloperCommandSession::from_command(cli.command.as_ref().unwrap());
    assert_eq!(
        session.workspace_ref,
        Some(PathBuf::from("/tmp/workspace").to_string_lossy().into_owned())
    );
}

#[test]
fn adapter_show_accepts_workspace_and_json_output() {
    let cli = Cli::try_parse_from([
        "boundline",
        "adapter",
        "show",
        "--workspace",
        "/tmp/workspace",
        "--json",
    ])
    .unwrap();

    let session = DeveloperCommandSession::from_command(cli.command.as_ref().unwrap());
    assert_eq!(
        session.workspace_ref,
        Some(PathBuf::from("/tmp/workspace").to_string_lossy().into_owned())
    );
}

#[test]
fn adapter_remove_accepts_workspace_scope() {
    let cli =
        Cli::try_parse_from(["boundline", "adapter", "remove", "--workspace", "/tmp/workspace"])
            .unwrap();

    let session = DeveloperCommandSession::from_command(cli.command.as_ref().unwrap());
    assert_eq!(
        session.workspace_ref,
        Some(PathBuf::from("/tmp/workspace").to_string_lossy().into_owned())
    );
}

#[test]
fn adapter_show_reports_built_in_default_when_no_selection_exists() -> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-show-built-in-default");

    let report = boundline::cli::adapter::execute_show(ShowAdapterRequest {
        workspace: Some(workspace.path()),
    });

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains("status: built_in_default"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("execution_source: built_in"),
        "{}",
        report.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_show_keeps_built_in_default_when_speckit_is_only_discoverable_on_path()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-show-discovery-hint");
    let binary_dir = workspace.path().join("bin");
    fs::create_dir_all(&binary_dir)?;
    let binary_path = binary_dir.join("boundline-adapter-speckit");
    fs::write(&binary_path, "#!/bin/sh\nexit 0\n")?;
    let mut permissions = fs::metadata(&binary_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions)?;
    let _path_guard = PathGuard::set(binary_dir.as_os_str());

    let report = boundline::cli::adapter::execute_show(ShowAdapterRequest {
        workspace: Some(workspace.path()),
    });

    assert_eq!(report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        report.terminal_output.contains("status: built_in_default"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("execution_source: built_in"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("discovery_hint: speckit available on PATH"),
        "{}",
        report.terminal_output
    );
    assert!(
        report.terminal_output.contains("activation_required: boundline adapter add speckit"),
        "{}",
        report.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_add_known_profile_persists_selection_and_show_reports_capabilities()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-known-profile");
    let script_path = write_ready_protocol_script(workspace.path())?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::Succeeded);
    assert!(add_report.terminal_output.contains("status: ready"), "{}", add_report.terminal_output);
    assert!(
        add_report.terminal_output.contains("adapter_id: speckit"),
        "{}",
        add_report.terminal_output
    );
    assert!(
        add_report.terminal_output.contains("supported_transports: stdio/json/stdin->stdout"),
        "{}",
        add_report.terminal_output
    );

    let persisted = FileConfigStore::for_workspace(workspace.path()).local_adapter()?.unwrap();
    assert_eq!(persisted.selection.adapter_id, SPECKIT_ADAPTER_ID);
    assert_eq!(persisted.selection.command, "/bin/sh");

    let show_report = boundline::cli::adapter::execute_show(ShowAdapterRequest {
        workspace: Some(workspace.path()),
    });

    assert_eq!(show_report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        show_report.terminal_output.contains("adapter_id: speckit"),
        "{}",
        show_report.terminal_output
    );
    assert!(
        show_report.terminal_output.contains("declared_stage_overrides: plan, run"),
        "{}",
        show_report.terminal_output
    );
    assert!(
        show_report.terminal_output.contains("supported_transports: stdio/json/stdin->stdout"),
        "{}",
        show_report.terminal_output
    );
    assert!(
        show_report
            .terminal_output
            .contains("declared_hook_subscriptions: stage_completed, stage_failed"),
        "{}",
        show_report.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_add_reports_blocked_unavailable_binary_for_missing_known_profile_command()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-missing-known-binary");
    let missing_command = workspace.path().join("missing-boundline-adapter-speckit");
    let adapter_args: Vec<String> = Vec::new();
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some(missing_command.to_string_lossy().as_ref()),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(
        add_report.terminal_output.contains("status: blocked"),
        "{}",
        add_report.terminal_output
    );
    assert!(
        add_report.terminal_output.contains("reason: unavailable_binary"),
        "{}",
        add_report.terminal_output
    );
    assert!(
        add_report.terminal_output.contains("recovery: boundline adapter add speckit --workspace"),
        "{}",
        add_report.terminal_output
    );

    let persisted = FileConfigStore::for_workspace(workspace.path()).local_adapter()?;
    assert!(persisted.is_none());

    Ok(())
}

#[test]
fn adapter_add_rejects_second_registration_when_one_adapter_is_already_active()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-single-active");
    let script_path = write_ready_protocol_script(workspace.path())?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let first_add = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });
    assert_eq!(first_add.exit_status, CommandExitStatus::Succeeded);

    let second_add = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(second_add.exit_status, CommandExitStatus::NonSuccess);
    assert!(
        second_add.terminal_output.contains("status: blocked"),
        "{}",
        second_add.terminal_output
    );
    assert!(
        second_add.terminal_output.contains("reason: adapter_already_selected"),
        "{}",
        second_add.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_remove_clears_selection_and_restores_built_in_default() -> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-remove-selection");
    let script_path = write_ready_protocol_script(workspace.path())?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });
    assert_eq!(add_report.exit_status, CommandExitStatus::Succeeded);

    let remove_report = boundline::cli::adapter::execute_remove(RemoveAdapterRequest {
        workspace: Some(workspace.path()),
    });

    assert_eq!(remove_report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        remove_report.terminal_output.contains("status: removed"),
        "{}",
        remove_report.terminal_output
    );
    assert!(
        remove_report.terminal_output.contains("execution_source: built_in"),
        "{}",
        remove_report.terminal_output
    );

    let persisted = FileConfigStore::for_workspace(workspace.path()).local_adapter()?;
    assert!(persisted.is_none());

    let show_report = boundline::cli::adapter::execute_show(ShowAdapterRequest {
        workspace: Some(workspace.path()),
    });
    assert_eq!(show_report.exit_status, CommandExitStatus::Succeeded);
    assert!(
        show_report.terminal_output.contains("status: built_in_default"),
        "{}",
        show_report.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_add_blocks_when_supported_transports_do_not_include_v1_stdio_json()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-unsupported-transport");
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["supported_transports"] = serde_json::json!([
        {
            "transport": "stdio",
            "encoding": "json",
            "request_channel": "stdout",
            "response_channel": "stdout"
        }
    ]);
    let script_path = write_protocol_script_with_describe_value(workspace.path(), describe)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(
        add_report.terminal_output.contains("status: blocked"),
        "{}",
        add_report.terminal_output
    );
    assert!(
        add_report.terminal_output.contains("reason: unsupported_transport"),
        "{}",
        add_report.terminal_output
    );

    Ok(())
}

#[test]
fn adapter_add_collects_missing_required_values_interactively() -> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-guided-required-config");
    let script_path = write_required_field_protocol_script(workspace.path(), false)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(workspace.path()),
        id: Some("custom-guided"),
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: false,
        interactive_terminal_override: Some(true),
        interactor: Some(Box::new(ScriptedAdapterConfigInteractor::new(vec![Ok(
            "workspace-demo".to_string(),
        )]))),
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::Succeeded);
    let persisted = FileConfigStore::for_workspace(workspace.path()).local_adapter()?.unwrap();
    assert!(persisted.interactive_resolution);
    assert_eq!(persisted.value_count, 1);
    assert_eq!(persisted.values[0].field_key, "workspace_slug");
    assert_eq!(persisted.values[0].string_value.as_deref(), Some("workspace-demo"));

    Ok(())
}

#[test]
fn adapter_add_cancellation_leaves_persisted_state_unchanged() -> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-guided-cancel");
    let script_path = write_required_field_protocol_script(workspace.path(), false)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(workspace.path()),
        id: Some("custom-guided"),
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: false,
        interactive_terminal_override: Some(true),
        interactor: Some(Box::new(ScriptedAdapterConfigInteractor::new(vec![Err(
            "user cancelled".to_string(),
        )]))),
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(add_report.terminal_output.contains("status: cancelled"));
    assert!(add_report.terminal_output.contains("reason: setup_cancelled"));
    assert!(
        add_report.terminal_output.contains("recovery: boundline adapter add custom --workspace")
    );
    assert!(
        add_report.terminal_output.contains("inspect_or_edit: boundline config show --workspace")
    );
    assert!(FileConfigStore::for_workspace(workspace.path()).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn adapter_add_blocks_non_interactive_when_required_config_is_missing() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_framework_adapter_workspace("adapter-add-missing-required-config");
    let script_path = write_required_field_protocol_script(workspace.path(), false)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(workspace.path()),
        id: Some("custom-guided"),
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(add_report.terminal_output.contains("status: blocked"));
    assert!(add_report.terminal_output.contains("reason: missing_required_config"));
    assert!(add_report.terminal_output.contains("missing_fields: workspace_slug"));
    assert!(FileConfigStore::for_workspace(workspace.path()).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn adapter_add_blocks_when_protocol_line_is_incompatible() -> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-incompatible-protocol");
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["protocol_line"] = serde_json::Value::String("framework-adapter-v0".to_string());
    let script_path = write_protocol_script_with_describe_value(workspace.path(), describe)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(add_report.terminal_output.contains("reason: incompatible_protocol"));
    assert!(FileConfigStore::for_workspace(workspace.path()).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn adapter_add_blocks_when_known_profile_reports_unexpected_adapter_id()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-unexpected-adapter-id");
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["adapter_id"] = serde_json::Value::String("custom-demo".to_string());
    let script_path = write_protocol_script_with_describe_value(workspace.path(), describe)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });

    assert_eq!(add_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(add_report.terminal_output.contains("reason: unexpected_adapter_id"));
    assert!(FileConfigStore::for_workspace(workspace.path()).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn adapter_add_maps_blocked_preflight_reasons_for_unavailable_and_default_invalid_config()
-> Result<(), Box<dyn Error>> {
    let unavailable_workspace =
        temp_framework_adapter_workspace("adapter-add-unavailable-resource");
    let unavailable_preflight = FrameworkAdapterPreflightResponse {
        status: FrameworkAdapterPreflightStatus::Blocked,
        normalized_config_values: Vec::new(),
        warnings: Vec::new(),
        reason: Some(FrameworkAdapterPreflightBlockReason::UnavailableResource),
        missing_fields: Vec::new(),
        recovery: Some("restore the missing template repository".to_string()),
    };
    let unavailable_script = write_protocol_script_with_describe_and_preflight_value(
        unavailable_workspace.path(),
        serde_json::to_value(sample_framework_adapter_describe_response())?,
        unavailable_preflight,
    )?;
    let unavailable_args = vec![unavailable_script.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let unavailable_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(unavailable_workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &unavailable_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });
    assert_eq!(unavailable_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(unavailable_report.terminal_output.contains("reason: unavailable_resource"));

    let invalid_workspace = temp_framework_adapter_workspace("adapter-add-invalid-config-default");
    let invalid_preflight = FrameworkAdapterPreflightResponse {
        status: FrameworkAdapterPreflightStatus::Blocked,
        normalized_config_values: Vec::new(),
        warnings: Vec::new(),
        reason: None,
        missing_fields: Vec::new(),
        recovery: Some("repair adapter config".to_string()),
    };
    let invalid_script = write_protocol_script_with_describe_and_preflight_value(
        invalid_workspace.path(),
        serde_json::to_value(sample_framework_adapter_describe_response())?,
        invalid_preflight,
    )?;
    let invalid_args = vec![invalid_script.to_string_lossy().into_owned()];

    let invalid_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: SPECKIT_ADAPTER_ID,
        workspace: Some(invalid_workspace.path()),
        id: None,
        command: Some("/bin/sh"),
        arg: &invalid_args,
        set: &set_values,
        non_interactive: true,
        interactive_terminal_override: None,
        interactor: None,
    });
    assert_eq!(invalid_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(invalid_report.terminal_output.contains("reason: invalid_config"));

    Ok(())
}

#[test]
fn adapter_add_reports_unresolved_binary_detail_and_treats_eof_as_guided_cancel()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_framework_adapter_workspace("adapter-add-unresolved-binary");
    let set_values: Vec<String> = Vec::new();

    let missing_report = {
        let _path_guard = PathGuard::set(std::ffi::OsStr::new(""));
        boundline::cli::adapter::execute_add(AddAdapterRequest {
            profile: SPECKIT_ADAPTER_ID,
            workspace: Some(workspace.path()),
            id: None,
            command: None,
            arg: &[],
            set: &set_values,
            non_interactive: true,
            interactive_terminal_override: None,
            interactor: None,
        })
    };
    assert_eq!(missing_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(missing_report.terminal_output.contains("discovery_state: unresolved"));
    assert!(
        missing_report.terminal_output.contains("detail: default command was not found on PATH")
    );

    let guided_workspace = temp_framework_adapter_workspace("adapter-add-guided-eof-cancel");
    let script_path = write_required_field_protocol_script(guided_workspace.path(), false)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let guided_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(guided_workspace.path()),
        id: Some("custom-guided"),
        command: Some("/bin/sh"),
        arg: &adapter_args,
        set: &set_values,
        non_interactive: false,
        interactive_terminal_override: Some(true),
        interactor: Some(Box::new(ScriptedAdapterConfigInteractor::new(vec![Err(
            "EOF while reading response".to_string(),
        )]))),
    });
    assert_eq!(guided_report.exit_status, CommandExitStatus::NonSuccess);
    assert!(guided_report.terminal_output.contains("status: cancelled"));
    assert!(guided_report.terminal_output.contains("reason: setup_cancelled"));

    Ok(())
}

fn write_ready_protocol_script(workspace: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    write_protocol_script_with_describe_value(workspace, describe)
}

fn write_protocol_script_with_describe_value(
    workspace: &Path,
    describe: serde_json::Value,
) -> Result<PathBuf, Box<dyn Error>> {
    write_protocol_script_with_describe_and_preflight_value(
        workspace,
        describe,
        sample_framework_adapter_preflight_ready_response(),
    )
}

fn write_protocol_script_with_describe_and_preflight_value(
    workspace: &Path,
    describe: serde_json::Value,
    preflight: FrameworkAdapterPreflightResponse,
) -> Result<PathBuf, Box<dyn Error>> {
    let describe = serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?;
    let preflight = serde_json::to_string(&sample_framework_adapter_success_envelope(preflight))?;
    let script_body = format!(
        "#!/bin/sh\ncase \"$1\" in\n  describe)\n    while IFS= read -r line; do\n      printf '%s\\n' \"$line\"\n    done <<'BOUNDLINE_JSON'\n{describe}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n      :\n      stdin_line=''\n    done\n    while IFS= read -r line; do\n      printf '%s\\n' \"$line\"\n    done <<'BOUNDLINE_JSON'\n{preflight}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );

    fs::create_dir_all(workspace)?;
    let path = workspace.join(format!("adapter-management-{}.sh", Uuid::new_v4()));
    fs::write(&path, script_body)?;
    Ok(path)
}

fn write_required_field_protocol_script(
    workspace: &Path,
    secret: bool,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["required_config_fields"] = serde_json::json!([
        {
            "field_key": "workspace_slug",
            "display_label": "Workspace Slug",
            "value_kind": "string",
            "required": true,
            "secret": secret,
            "default_value_text": null,
            "prompt_text": "Workspace slug",
            "help_text": "used to bind the adapter workspace",
            "non_interactive_policy": "fail"
        }
    ]);

    let describe = serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?;
    let preflight =
        serde_json::to_string(&sample_framework_adapter_success_envelope(serde_json::json!({
            "status": "ready",
            "normalized_config_values": [],
            "warnings": [],
            "reason": null,
            "missing_fields": [],
            "recovery": null
        })))?;
    let script_body = format!(
        "#!/bin/sh\ncase \"$1\" in\n  describe)\n    while IFS= read -r line; do\n      printf '%s\\n' \"$line\"\n    done <<'BOUNDLINE_JSON'\n{describe}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n      :\n      stdin_line=''\n    done\n    while IFS= read -r line; do\n      printf '%s\\n' \"$line\"\n    done <<'BOUNDLINE_JSON'\n{preflight}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n"
    );

    fs::create_dir_all(workspace)?;
    let path = workspace.join(format!("adapter-required-field-{}.sh", Uuid::new_v4()));
    fs::write(&path, script_body)?;
    Ok(path)
}
