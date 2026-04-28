use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{
    ConfigFile, ConfigShowScope, ConfigWriteScope, ModelRoute, RouteSlot, RoutingOverrides,
    RuntimeKind, ValueSource, resolve_effective_routing,
};

use super::init::runtime_available;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

pub fn execute_show(
    workspace: Option<&Path>,
    scope: Option<ConfigShowScope>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let scope = scope.unwrap_or(ConfigShowScope::Effective);

    let output = match scope {
        ConfigShowScope::Global => {
            let global = FileConfigStore::load_global()?.unwrap_or_default();
            render_scope("global", &global)
        }
        ConfigShowScope::Workspace => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let local = FileConfigStore::for_workspace(workspace).load_local()?.unwrap_or_default();
            render_scope("workspace", &local)
        }
        ConfigShowScope::Effective => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let store = FileConfigStore::for_workspace(workspace);
            let local = store.local_routing()?;
            let global = FileConfigStore::global_routing()?;
            let resolved = resolve_effective_routing(
                &RoutingOverrides::default(),
                local.as_ref(),
                global.as_ref(),
            );

            let mut lines = vec!["config: effective".to_string()];
            lines.push(format!(
                "planning: {} [{}]",
                route_text(&resolved.planning.route),
                source_text(resolved.planning.source)
            ));
            lines.push(format!(
                "implementation: {} [{}]",
                route_text(&resolved.implementation.route),
                source_text(resolved.implementation.source)
            ));
            lines.push(format!(
                "verification: {} [{}]",
                route_text(&resolved.verification.route),
                source_text(resolved.verification.source)
            ));
            lines.push(format!(
                "review: {} [{}]",
                route_text(&resolved.review.route),
                source_text(resolved.review.source)
            ));
            lines.push(format!(
                "adjudication: {} [{}]",
                route_text(&resolved.adjudication.route),
                source_text(resolved.adjudication.source)
            ));

            if resolved.reviewer_roles.is_empty() {
                lines.push("reviewer_roles: none".to_string());
            } else {
                lines.push("reviewer_roles:".to_string());
                for (role, route) in &resolved.reviewer_roles {
                    lines.push(format!(
                        "- {}: {} [{}]",
                        role,
                        route_text(&route.route),
                        source_text(route.source)
                    ));
                }
            }

            lines.join("\n")
        }
    };

    Ok(ConfigCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output: output })
}

pub fn execute_set(
    workspace: Option<&Path>,
    scope: ConfigWriteScope,
    slot: Option<RouteSlot>,
    reviewer: Option<&str>,
    adjudicator: bool,
    runtime: RuntimeKind,
    model: &str,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if !runtime_available(runtime) {
        return Err(ConfigCommandError::RuntimeUnavailable(runtime));
    }

    let target = mutation_target(slot, reviewer, adjudicator)?;
    let route = ModelRoute { runtime, model: model.to_string() };
    route.validate().map_err(|source| ConfigCommandError::InvalidRoute(source.to_string()))?;

    let (mut config, location) = load_config_for_scope(workspace, scope)?;

    match target {
        MutationTarget::Slot(slot) => config.routing.set_slot(slot, route),
        MutationTarget::Reviewer(role) => {
            config.routing.reviewer_roles.insert(role, route);
        }
        MutationTarget::Adjudicator => config.routing.adjudication = Some(route),
    }

    config
        .routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidRoute(source.to_string()))?;
    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated {location}"),
    })
}

pub fn execute_unset(
    workspace: Option<&Path>,
    scope: ConfigWriteScope,
    slot: Option<RouteSlot>,
    reviewer: Option<&str>,
    adjudicator: bool,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let target = mutation_target(slot, reviewer, adjudicator)?;
    let (mut config, location) = load_config_for_scope(workspace, scope)?;

    match target {
        MutationTarget::Slot(slot) => config.routing.unset_slot(slot),
        MutationTarget::Reviewer(role) => {
            config.routing.reviewer_roles.remove(&role);
        }
        MutationTarget::Adjudicator => config.routing.adjudication = None,
    }

    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: removed value from {location}"),
    })
}

fn render_scope(scope: &str, config: &ConfigFile) -> String {
    let mut lines = vec![format!("config: {scope}")];
    lines.push(format!(
        "planning: {}",
        config.routing.planning.as_ref().map(route_text).unwrap_or_else(|| "<unset>".to_string())
    ));
    lines.push(format!(
        "implementation: {}",
        config
            .routing
            .implementation
            .as_ref()
            .map(route_text)
            .unwrap_or_else(|| "<unset>".to_string())
    ));
    lines.push(format!(
        "verification: {}",
        config
            .routing
            .verification
            .as_ref()
            .map(route_text)
            .unwrap_or_else(|| "<unset>".to_string())
    ));
    lines.push(format!(
        "review: {}",
        config.routing.review.as_ref().map(route_text).unwrap_or_else(|| "<unset>".to_string())
    ));
    lines.push(format!(
        "adjudication: {}",
        config
            .routing
            .adjudication
            .as_ref()
            .map(route_text)
            .unwrap_or_else(|| "<unset>".to_string())
    ));

    if config.routing.reviewer_roles.is_empty() {
        lines.push("reviewer_roles: none".to_string());
    } else {
        lines.push("reviewer_roles:".to_string());
        for (role, route) in &config.routing.reviewer_roles {
            lines.push(format!("- {}: {}", role, route_text(route)));
        }
    }

    if config.routing.assistant_runtimes.is_empty() {
        lines.push("assistant_runtimes: none".to_string());
    } else {
        let runtimes = config
            .routing
            .assistant_runtimes
            .iter()
            .map(|runtime| runtime.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("assistant_runtimes: {runtimes}"));
    }

    lines.join("\n")
}

fn route_text(route: &ModelRoute) -> String {
    format!("{}:{}", route.runtime.as_str(), route.model)
}

fn source_text(source: ValueSource) -> &'static str {
    match source {
        ValueSource::Cli => "cli",
        ValueSource::Workspace => "workspace",
        ValueSource::Global => "global",
        ValueSource::BuiltIn => "built-in",
    }
}

enum MutationTarget {
    Slot(RouteSlot),
    Reviewer(String),
    Adjudicator,
}

fn mutation_target(
    slot: Option<RouteSlot>,
    reviewer: Option<&str>,
    adjudicator: bool,
) -> Result<MutationTarget, ConfigCommandError> {
    let count =
        usize::from(slot.is_some()) + usize::from(reviewer.is_some()) + usize::from(adjudicator);
    if count != 1 {
        return Err(ConfigCommandError::InvalidTargetSelection);
    }

    if let Some(slot) = slot {
        return Ok(MutationTarget::Slot(slot));
    }

    if let Some(role) = reviewer {
        if role.trim().is_empty() {
            return Err(ConfigCommandError::InvalidReviewerRole);
        }
        return Ok(MutationTarget::Reviewer(role.trim().to_string()));
    }

    Ok(MutationTarget::Adjudicator)
}

fn load_config_for_scope(
    workspace: Option<&Path>,
    scope: ConfigWriteScope,
) -> Result<(ConfigFile, String), ConfigCommandError> {
    match scope {
        ConfigWriteScope::Global => {
            let config = FileConfigStore::load_global()?.unwrap_or_default();
            Ok((config, "global config".to_string()))
        }
        ConfigWriteScope::Workspace => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let store = FileConfigStore::for_workspace(workspace);
            let config = store.load_local()?.unwrap_or_default();
            Ok((config, format!("workspace config at {}", store.local_config_path().display())))
        }
    }
}

fn save_config_for_scope(
    workspace: Option<&Path>,
    scope: ConfigWriteScope,
    config: &ConfigFile,
) -> Result<PathBuf, ConfigCommandError> {
    match scope {
        ConfigWriteScope::Global => Ok(FileConfigStore::save_global(config)?),
        ConfigWriteScope::Workspace => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let store = FileConfigStore::for_workspace(workspace);
            Ok(store.save_local(config)?)
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigCommandError {
    #[error("workspace is required for this command")]
    WorkspaceRequired,
    #[error("select exactly one target: --slot, --reviewer, or --adjudicator")]
    InvalidTargetSelection,
    #[error("reviewer role cannot be empty")]
    InvalidReviewerRole,
    #[error("runtime '{0}' is not currently available")]
    RuntimeUnavailable(RuntimeKind),
    #[error("invalid route: {0}")]
    InvalidRoute(String),
    #[error("config store error: {0}")]
    Store(#[from] ConfigStoreError),
}
