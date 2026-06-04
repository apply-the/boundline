//! Integration coverage for guided adapter configuration and persisted config flows.

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use boundline::FileConfigStore;
use boundline::cli::CommandExitStatus;
use boundline::cli::adapter::{AdapterConfigInteractor, AddAdapterRequest};
use boundline::cli::session::{execute_goal, execute_plan};
use boundline::fixture::{
    sample_framework_adapter_describe_response,
    sample_framework_adapter_execute_stage_success_response,
    sample_framework_adapter_success_envelope,
};
use serde_json::json;
use uuid::Uuid;

use crate::framework_adapter::optional_built_speckit_binary_dir;
use crate::workspace_fixture::{
    run_boundline_in_with_env, supported_canon_path, temp_fixture_workspace, terminal_text,
};

const BUG_FIX_FLOW: &str = "bug-fix";
const FIX_GOAL: &str = "fix the failing add test";
const TEMPLATE_REPO_FIELD_KEY: &str = "template_repo";
const ADAPTER_REPO_FIELD_KEY: &str = "adapter_repo";
const CUSTOM_REQUIRED_FIELD_KEY: &str = "workspace_slug";

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

#[test]
fn known_speckit_profile_activates_with_prefilled_defaults_and_runs_plan()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_specify_workspace_fixture("framework-adapter-config-speckit-defaults")?;
    let Some(path_env) = speckit_path_env()? else {
        return Ok(());
    };

    let add = run_boundline_in_with_env(
        &workspace,
        &["adapter", "add", "speckit", "--workspace", "."],
        &[("PATH", path_env.as_str())],
    );
    let add_text = terminal_text(&add);
    assert_eq!(add.status.code(), Some(0), "{add_text}");

    let adapter = FileConfigStore::for_workspace(&workspace)
        .local_adapter()?
        .ok_or("expected persisted Speckit adapter selection")?;
    assert_eq!(adapter.value_count, 2);
    assert!(has_path_value(
        &adapter.values,
        TEMPLATE_REPO_FIELD_KEY,
        "../boundline-framework-template"
    ));
    assert!(has_path_value(
        &adapter.values,
        ADAPTER_REPO_FIELD_KEY,
        "../boundline-adapter-speckit"
    ));

    assert_command_succeeds(run_boundline_in_with_env(
        &workspace,
        &["goal", "--goal", FIX_GOAL],
        &[("PATH", path_env.as_str())],
    ))?;

    let plan = run_boundline_in_with_env(
        &workspace,
        &["plan", "--flow", BUG_FIX_FLOW],
        &[("PATH", path_env.as_str())],
    );
    let plan_text = terminal_text(&plan);
    assert_eq!(plan.status.code(), Some(0), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage: plan"), "{plan_text}");
    assert!(plan_text.contains("framework_adapter_stage_status: succeeded"), "{plan_text}");
    assert!(
        plan_text.contains(
            "framework_adapter_produced_artifacts: specs/068-backlog-contract/spec.md, specs/068-backlog-contract/plan.md, specs/068-backlog-contract/tasks.md, .specify/workflows/speckit/planning.yml"
        ),
        "{plan_text}"
    );
    assert!(!workspace.join("speckit-plan-claimed.txt").exists(), "{plan_text}");

    Ok(())
}

#[test]
fn custom_adapter_guided_setup_persists_prompted_value_and_runs_plan() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_fixture_workspace("framework-adapter-config-custom-guided");
    let stage_marker = workspace.join("custom-plan-claimed.txt");
    let script_path = write_guided_plan_adapter_script(workspace.as_path(), &stage_marker)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(workspace.as_path()),
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
    assert_eq!(
        add_report.exit_status,
        CommandExitStatus::Succeeded,
        "{}",
        add_report.terminal_output
    );

    let adapter = FileConfigStore::for_workspace(workspace.as_path())
        .local_adapter()?
        .ok_or("expected persisted custom adapter selection")?;
    assert_eq!(adapter.value_count, 1);
    assert!(has_string_value(&adapter.values, CUSTOM_REQUIRED_FIELD_KEY, "workspace-demo"));

    execute_goal(Some(workspace.as_path()), Some(FIX_GOAL), &[], None, None, None, None)?;
    let plan = execute_plan(Some(workspace.as_path()), Some(BUG_FIX_FLOW), false)?;
    assert_eq!(plan.exit_status, CommandExitStatus::Succeeded, "{}", plan.terminal_output);
    assert!(stage_marker.is_file(), "{}", plan.terminal_output);

    Ok(())
}

#[test]
fn custom_adapter_guided_cancellation_keeps_persisted_state_unchanged() -> Result<(), Box<dyn Error>>
{
    let workspace = temp_fixture_workspace("framework-adapter-config-custom-cancelled");
    let stage_marker = workspace.join("custom-plan-claimed.txt");
    let script_path = write_guided_plan_adapter_script(workspace.as_path(), &stage_marker)?;
    let adapter_args = vec![script_path.to_string_lossy().into_owned()];
    let set_values: Vec<String> = Vec::new();

    let add_report = boundline::cli::adapter::execute_add(AddAdapterRequest {
        profile: "custom",
        workspace: Some(workspace.as_path()),
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
    assert!(FileConfigStore::for_workspace(workspace.as_path()).local_adapter()?.is_none());

    Ok(())
}

#[test]
fn custom_adapter_cli_add_blocks_non_interactive_when_required_value_is_missing()
-> Result<(), Box<dyn Error>> {
    let workspace = temp_fixture_workspace("framework-adapter-config-custom-non-interactive");
    let stage_marker = workspace.join("custom-plan-claimed.txt");
    let script_path = write_guided_plan_adapter_script(workspace.as_path(), &stage_marker)?;
    let script_path_arg = script_path.to_string_lossy().into_owned();

    let add = run_boundline_in_with_env(
        &workspace,
        &[
            "adapter",
            "add",
            "custom",
            "--workspace",
            ".",
            "--id",
            "custom-guided",
            "--command",
            "/bin/sh",
            "--arg",
            script_path_arg.as_str(),
            "--non-interactive",
        ],
        &[],
    );
    let add_text = terminal_text(&add);

    assert_ne!(add.status.code(), Some(0), "{add_text}");
    assert!(add_text.contains("status: blocked"), "{add_text}");
    assert!(add_text.contains("reason: missing_required_config"), "{add_text}");
    assert!(add_text.contains("recovery: boundline adapter add custom --workspace"), "{add_text}");
    assert!(FileConfigStore::for_workspace(&workspace).local_adapter()?.is_none());

    Ok(())
}

fn speckit_path_env() -> Result<Option<String>, Box<dyn Error>> {
    let Some(binary_dir) = optional_built_speckit_binary_dir()? else {
        return Ok(None);
    };
    Ok(Some(format!("{}:{}", binary_dir.display(), supported_canon_path())))
}

fn assert_command_succeeds(output: std::process::Output) -> Result<(), Box<dyn Error>> {
    let rendered = terminal_text(&output);
    if output.status.code() == Some(0) {
        Ok(())
    } else {
        Err(format!("command failed: {rendered}").into())
    }
}

fn write_guided_plan_adapter_script(
    workspace: &Path,
    stage_marker: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let mut describe = serde_json::to_value(sample_framework_adapter_describe_response())?;
    describe["adapter_id"] = json!("custom-guided");
    describe["declared_stage_overrides"] = json!(["plan"]);
    describe["declared_hook_subscriptions"] = json!([]);
    describe["required_config_fields"] = json!([
        {
            "field_key": CUSTOM_REQUIRED_FIELD_KEY,
            "display_label": "Workspace Slug",
            "value_kind": "string",
            "required": true,
            "secret": false,
            "default_value_text": null,
            "prompt_text": "Workspace slug",
            "help_text": "used to bind the adapter workspace",
            "non_interactive_policy": "fail"
        }
    ]);

    let describe = serde_json::to_string(&sample_framework_adapter_success_envelope(describe))?;
    let preflight = serde_json::to_string(&sample_framework_adapter_success_envelope(json!({
        "status": "ready",
        "normalized_config_values": [],
        "warnings": [],
        "reason": null,
        "missing_fields": [],
        "recovery": null
    })))?;
    let execute_stage = serde_json::to_string(&sample_framework_adapter_success_envelope(
        sample_framework_adapter_execute_stage_success_response(),
    ))?;

    let script = format!(
        "#!/bin/sh\nset -eu\nconsume_stdin() {{\n  stdin_line=''\n  while IFS= read -r stdin_line || [ -n \"$stdin_line\" ]; do\n    stdin_line=''\n  done\n}}\nprint_json() {{\n  while IFS= read -r line; do\n    printf '%s\\n' \"$line\"\n  done\n}}\ncase \"$1\" in\n  describe)\n    print_json <<'BOUNDLINE_JSON'\n{describe}\nBOUNDLINE_JSON\n    ;;\n  preflight)\n    consume_stdin\n    print_json <<'BOUNDLINE_JSON'\n{preflight}\nBOUNDLINE_JSON\n    ;;\n  execute-stage)\n    consume_stdin\n    : > \"{}\"\n    print_json <<'BOUNDLINE_JSON'\n{execute_stage}\nBOUNDLINE_JSON\n    ;;\n  *)\n    echo \"unsupported command: $1\" >&2\n    exit 64\n    ;;\nesac\n",
        stage_marker.display()
    );

    let script_path = workspace.join(format!("guided-plan-adapter-{}.sh", Uuid::new_v4()));
    fs::write(&script_path, script)?;
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    Ok(script_path)
}

fn temp_specify_workspace_fixture(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
    let workspace = temp_fixture_workspace(prefix);
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("rsync")
        .args([
            "-a",
            "--exclude",
            ".git",
            "--exclude",
            "target",
            "--exclude",
            ".boundline",
            &format!("{}/", repo_root.display()),
            &format!("{}/", workspace.display()),
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "failed to copy Spec Kit workspace fixture: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(workspace)
}

fn has_path_value(
    values: &[boundline::domain::configuration::AdapterConfigValueRecord],
    field_key: &str,
    expected: &str,
) -> bool {
    values
        .iter()
        .any(|value| value.field_key == field_key && value.path_value.as_deref() == Some(expected))
}

fn has_string_value(
    values: &[boundline::domain::configuration::AdapterConfigValueRecord],
    field_key: &str,
    expected: &str,
) -> bool {
    values.iter().any(|value| {
        value.field_key == field_key && value.string_value.as_deref() == Some(expected)
    })
}
