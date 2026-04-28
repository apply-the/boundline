use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use thiserror::Error;

use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{InitTemplate, RuntimeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_init(
    workspace: &Path,
    template: Option<InitTemplate>,
    assistants: &[RuntimeKind],
    force: bool,
) -> Result<InitCommandReport, InitCommandError> {
    fs::create_dir_all(workspace).map_err(|source| InitCommandError::CreateWorkspace {
        path: workspace.to_path_buf(),
        source,
    })?;

    let template = template.unwrap_or(InitTemplate::BugFix);
    let store = FileConfigStore::for_workspace(workspace);
    let execution_path = workspace.join(".synod/execution.json");
    let local_config_path = store.local_config_path();

    let mut planned = Vec::new();
    let execution_exists = execution_path.is_file();
    let config_exists = local_config_path.is_file();

    planned.push(if execution_exists {
        format!("- update {}", execution_path.display())
    } else {
        format!("- create {}", execution_path.display())
    });
    planned.push(if config_exists {
        format!("- update {}", local_config_path.display())
    } else {
        format!("- create {}", local_config_path.display())
    });

    if (execution_exists || config_exists) && !force {
        let mut lines = vec![
            "init: preview only - existing Synod files would be updated".to_string(),
            "use --force to apply updates to existing files".to_string(),
            format!("template: {}", template_label(template)),
        ];
        lines.extend(planned);
        return Ok(InitCommandReport {
            exit_status: CommandExitStatus::NonSuccess,
            terminal_output: lines.join("\n"),
        });
    }

    if let Some(parent) = execution_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|source| InitCommandError::WriteFile { path: parent.to_path_buf(), source })?;
    }

    let execution = execution_template(template);
    fs::write(
        &execution_path,
        serde_json::to_string_pretty(&execution).expect("execution template should serialize"),
    )
    .map_err(|source| InitCommandError::WriteFile { path: execution_path.clone(), source })?;

    let mut local = store.load_local()?.unwrap_or_default();
    local.routing.assistant_runtimes = assistants.to_vec();
    store.save_local(&local)?;

    let capabilities = assistants
        .iter()
        .map(|runtime| format!("- {}: {}", runtime.as_str(), runtime_capability_line(*runtime)))
        .collect::<Vec<_>>();

    let mut lines = vec![
        "init: workspace initialized".to_string(),
        format!("template: {}", template_label(template)),
        format!("execution_profile: {}", execution_path.display()),
        format!("workspace_config: {}", local_config_path.display()),
    ];

    if !capabilities.is_empty() {
        lines.push("runtime_capabilities:".to_string());
        lines.extend(capabilities);
    }

    lines.push("next: synod doctor --workspace <workspace>".to_string());

    Ok(InitCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: lines.join("\n"),
    })
}

fn execution_template(template: InitTemplate) -> serde_json::Value {
    let (name, attempt_id, summary) = match template {
        InitTemplate::BugFix => ("init-bug-fix", "apply-bug-fix", "Apply a bounded bug fix"),
        InitTemplate::Change => ("init-change", "apply-change", "Apply a bounded change"),
        InitTemplate::Delivery => {
            ("init-delivery", "apply-delivery", "Apply a bounded delivery update")
        }
    };

    json!({
        "name": name,
        "read_targets": ["src/", "tests/"],
        "validation_command": {
            "program": "cargo",
            "args": ["test", "--quiet"]
        },
        "attempts": [
            {
                "attempt_id": attempt_id,
                "summary": summary,
                "failure_mode": "replan",
                "changes": [
                    {
                        "path": "README.md",
                        "find": "TODO(init)",
                        "replace": "TODO(init)"
                    }
                ]
            }
        ]
    })
}

fn template_label(template: InitTemplate) -> &'static str {
    match template {
        InitTemplate::BugFix => "bug-fix",
        InitTemplate::Change => "change",
        InitTemplate::Delivery => "delivery",
    }
}

fn runtime_capability_line(runtime: RuntimeKind) -> &'static str {
    if runtime_available(runtime) { "available" } else { "missing from PATH or extension surface" }
}

pub fn runtime_available(runtime: RuntimeKind) -> bool {
    match runtime {
        RuntimeKind::Copilot => true,
        RuntimeKind::Claude => command_in_path("claude"),
        RuntimeKind::Codex => command_in_path("codex"),
        RuntimeKind::Gemini => command_in_path("gemini"),
    }
}

fn command_in_path(command: &str) -> bool {
    let path_var = match std::env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };

    for entry in std::env::split_paths(&path_var) {
        let candidate = entry.join(command);
        if candidate.is_file() {
            return true;
        }
    }

    false
}

#[derive(Debug, Error)]
pub enum InitCommandError {
    #[error("failed to create workspace directory {path}: {source}")]
    CreateWorkspace { path: PathBuf, source: std::io::Error },
    #[error("failed to write file {path}: {source}")]
    WriteFile { path: PathBuf, source: std::io::Error },
    #[error("failed to persist config: {0}")]
    ConfigStore(#[from] ConfigStoreError),
}
