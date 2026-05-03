use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::cli::CommandExitStatus;
use crate::domain::configuration::{
    CapabilityState, ConfigFile, ConfigShowScope, ConfigWriteScope, EffortFallbackPolicy,
    EffortLevel, ModelRoute, RouteSlot, RoutingOverrides, RuntimeCapabilityProfile, RuntimeKind,
    SlotEffortPolicy, ValueSource, resolve_effective_routing,
    resolve_effective_runtime_capabilities, resolve_effective_slot_effort_policies,
};
use crate::domain::routing_decision::RoutingDecisionProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigCommandReport {
    pub exit_status: CommandExitStatus,
    pub terminal_output: String,
}

#[derive(Debug, Clone, Copy)]
pub struct SetConfigRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub slot: Option<RouteSlot>,
    pub reviewer: Option<&'a str>,
    pub adjudicator: bool,
    pub runtime: RuntimeKind,
    pub model: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct SetCapabilityRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub runtime: RuntimeKind,
    pub continuation: CapabilityState,
    pub resume: CapabilityState,
    pub validation: CapabilityState,
    pub handoff_target: CapabilityState,
    pub escalation_context: CapabilityState,
    pub notes: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct SetEffortRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub slot: RouteSlot,
    pub level: EffortLevel,
    pub fallback: EffortFallbackPolicy,
    pub rationale: Option<&'a str>,
}

pub fn execute_show(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
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
        ConfigShowScope::Cluster => {
            let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
            let store = FileClusterStore::for_workspace(cluster);
            let config = store.load()?.ok_or_else(|| {
                ConfigCommandError::MissingClusterConfig(store.cluster_config_path())
            })?;
            let scope_view = ConfigFile { version: config.version, routing: config.routing };
            render_scope("cluster", &scope_view)
        }
        ConfigShowScope::Effective => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let store = FileConfigStore::for_workspace(workspace);
            let local = store.local_routing()?;
            let cluster_routing = if let Some(cluster) = cluster {
                let store = FileClusterStore::for_workspace(cluster);
                Some(
                    store
                        .load()?
                        .ok_or_else(|| {
                            ConfigCommandError::MissingClusterConfig(store.cluster_config_path())
                        })?
                        .routing,
                )
            } else {
                None
            };
            let global = FileConfigStore::global_routing()?;
            let resolved = resolve_effective_routing(
                &RoutingOverrides::default(),
                local.as_ref(),
                cluster_routing.as_ref(),
                global.as_ref(),
            );
            let effective_capabilities = resolve_effective_runtime_capabilities(
                local.as_ref(),
                cluster_routing.as_ref(),
                global.as_ref(),
            );
            let effective_effort = resolve_effective_slot_effort_policies(
                local.as_ref(),
                cluster_routing.as_ref(),
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

            lines.extend(
                RoutingDecisionProjection::from_effective_routing(&resolved).projection_lines(),
            );

            if effective_capabilities.is_empty() {
                lines.push("runtime_capabilities: none".to_string());
            } else {
                lines.push("runtime_capabilities:".to_string());
                for (runtime, profile) in &effective_capabilities {
                    lines.push(format!(
                        "- {}: {} [{}]",
                        runtime.as_str(),
                        profile_text(&profile.profile),
                        source_text(profile.source)
                    ));
                }
            }

            if effective_effort.is_empty() {
                lines.push("slot_effort_policies: none".to_string());
            } else {
                lines.push("slot_effort_policies:".to_string());
                for (slot, policy) in &effective_effort {
                    lines.push(format!(
                        "- {}: {} [{}]",
                        slot_label(*slot),
                        effort_policy_text(&policy.policy),
                        source_text(policy.source)
                    ));
                }
            }

            lines.join("\n")
        }
    };

    Ok(ConfigCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output: output })
}

pub fn execute_set(
    request: SetConfigRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let target = mutation_target(request.slot, request.reviewer, request.adjudicator)?;
    let route = ModelRoute { runtime: request.runtime, model: request.model.to_string() };
    route.validate().map_err(|source| ConfigCommandError::InvalidRoute(source.to_string()))?;

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;

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
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!("config: updated cluster config at {}", path.display()),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;

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
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated {location}"),
    })
}

pub fn execute_unset(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    slot: Option<RouteSlot>,
    reviewer: Option<&str>,
    adjudicator: bool,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let target = mutation_target(slot, reviewer, adjudicator)?;

    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;

        match target {
            MutationTarget::Slot(slot) => config.routing.unset_slot(slot),
            MutationTarget::Reviewer(role) => {
                config.routing.reviewer_roles.remove(&role);
            }
            MutationTarget::Adjudicator => config.routing.adjudication = None,
        }

        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: removed value from cluster config at {}",
                path.display()
            ),
        });
    }

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

pub fn execute_set_capability(
    request: SetCapabilityRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let profile = RuntimeCapabilityProfile {
        continuation: request.continuation,
        resume: request.resume,
        validation: request.validation,
        handoff_target: request.handoff_target,
        escalation_context: request.escalation_context,
        notes: request.notes.map(str::to_string),
    };
    profile.validate().map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.set_runtime_capability(request.runtime, profile);
        config
            .routing
            .validate()
            .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: updated runtime capability in cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;
    config.routing.set_runtime_capability(request.runtime, profile);
    config
        .routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated runtime capability in {location}"),
    })
}

pub fn execute_unset_capability(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    runtime: RuntimeKind,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.unset_runtime_capability(runtime);
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: removed runtime capability from cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(workspace, scope)?;
    config.routing.unset_runtime_capability(runtime);
    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: removed runtime capability from {location}"),
    })
}

pub fn execute_set_effort(
    request: SetEffortRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let policy = SlotEffortPolicy {
        level: request.level,
        fallback: request.fallback,
        rationale: request.rationale.map(str::to_string),
    };
    policy.validate().map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.set_slot_effort_policy(request.slot, policy);
        config
            .routing
            .validate()
            .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: updated slot effort policy in cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;
    config.routing.set_slot_effort_policy(request.slot, policy);
    config
        .routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated slot effort policy in {location}"),
    })
}

pub fn execute_unset_effort(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    slot: RouteSlot,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.unset_slot_effort_policy(slot);
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: removed slot effort policy from cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(workspace, scope)?;
    config.routing.unset_slot_effort_policy(slot);
    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: removed slot effort policy from {location}"),
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

    if config.routing.runtime_capabilities.is_empty() {
        lines.push("runtime_capabilities: none".to_string());
    } else {
        lines.push("runtime_capabilities:".to_string());
        for (runtime, profile) in &config.routing.runtime_capabilities {
            lines.push(format!("- {}: {}", runtime.as_str(), profile_text(profile)));
        }
    }

    if config.routing.slot_effort_policies.is_empty() {
        lines.push("slot_effort_policies: none".to_string());
    } else {
        lines.push("slot_effort_policies:".to_string());
        for (slot, policy) in &config.routing.slot_effort_policies {
            lines.push(format!("- {}: {}", slot_label(*slot), effort_policy_text(policy)));
        }
    }

    lines.join("\n")
}

fn route_text(route: &ModelRoute) -> String {
    format!("{}:{}", route.runtime.as_str(), route.model)
}

fn profile_text(profile: &RuntimeCapabilityProfile) -> String {
    profile.summary_text()
}

fn effort_policy_text(policy: &SlotEffortPolicy) -> String {
    policy.summary_text()
}

fn slot_label(slot: RouteSlot) -> &'static str {
    match slot {
        RouteSlot::Planning => "planning",
        RouteSlot::Implementation => "implementation",
        RouteSlot::Verification => "verification",
        RouteSlot::Review => "review",
    }
}

fn source_text(source: ValueSource) -> &'static str {
    match source {
        ValueSource::Cli => "cli",
        ValueSource::Workspace => "workspace",
        ValueSource::Cluster => "cluster",
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
        ConfigWriteScope::Cluster => Err(ConfigCommandError::ClusterRequired),
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
        ConfigWriteScope::Cluster => Err(ConfigCommandError::ClusterRequired),
    }
}

#[derive(Debug, Error)]
pub enum ConfigCommandError {
    #[error("workspace is required for this command")]
    WorkspaceRequired,
    #[error("cluster primary workspace is required for this command")]
    ClusterRequired,
    #[error("select exactly one target: --slot, --reviewer, or --adjudicator")]
    InvalidTargetSelection,
    #[error("reviewer role cannot be empty")]
    InvalidReviewerRole,
    #[error("invalid route: {0}")]
    InvalidRoute(String),
    #[error("invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("config store error: {0}")]
    Store(#[from] ConfigStoreError),
    #[error("cluster store error: {0}")]
    ClusterStore(#[from] ClusterStoreError),
    #[error("cluster config is missing at {0}")]
    MissingClusterConfig(PathBuf),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use uuid::Uuid;

    use super::*;
    use crate::adapters::cluster_store::FileClusterStore;
    use crate::adapters::config_store::FileConfigStore;
    use crate::domain::cluster::{
        ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, WorkspaceCluster,
    };
    use crate::domain::configuration::RoutingConfig;

    fn temp_workspace(prefix: &str) -> PathBuf {
        let workspace = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();
        workspace
    }

    fn build_cluster_config(workspace: &Path) -> ClusterConfigFile {
        let member_workspace = workspace.join("member");
        ClusterConfigFile {
            version: 1,
            cluster: WorkspaceCluster {
                cluster_id: "cluster-a".to_string(),
                primary_workspace_ref: workspace.to_string_lossy().into_owned(),
                members: vec![
                    ClusterMemberRegistration {
                        workspace_ref: workspace.to_string_lossy().into_owned(),
                        display_name: Some("primary".to_string()),
                        role: ClusterMemberRole::Primary,
                    },
                    ClusterMemberRegistration {
                        workspace_ref: member_workspace.to_string_lossy().into_owned(),
                        display_name: Some("member".to_string()),
                        role: ClusterMemberRole::Member,
                    },
                ],
                created_at: 1,
                updated_at: 1,
            },
            routing: RoutingConfig::default(),
        }
    }

    fn supported_profile(notes: Option<&str>) -> RuntimeCapabilityProfile {
        RuntimeCapabilityProfile {
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: notes.map(str::to_string),
        }
    }

    fn slot_policy(
        level: EffortLevel,
        fallback: EffortFallbackPolicy,
        rationale: Option<&str>,
    ) -> SlotEffortPolicy {
        SlotEffortPolicy { level, fallback, rationale: rationale.map(str::to_string) }
    }

    #[test]
    fn render_scope_formats_empty_and_populated_config() {
        let empty = render_scope("workspace", &ConfigFile::default());
        assert!(empty.contains("config: workspace"));
        assert!(empty.contains("planning: <unset>"));
        assert!(empty.contains("reviewer_roles: none"));
        assert!(empty.contains("assistant_runtimes: none"));
        assert!(empty.contains("runtime_capabilities: none"));
        assert!(empty.contains("slot_effort_policies: none"));

        assert_eq!(slot_label(RouteSlot::Planning), "planning");
        assert_eq!(slot_label(RouteSlot::Implementation), "implementation");
        assert_eq!(slot_label(RouteSlot::Verification), "verification");
        assert_eq!(slot_label(RouteSlot::Review), "review");
        assert_eq!(source_text(ValueSource::Cli), "cli");
        assert_eq!(source_text(ValueSource::Workspace), "workspace");
        assert_eq!(source_text(ValueSource::Cluster), "cluster");
        assert_eq!(source_text(ValueSource::Global), "global");
        assert_eq!(source_text(ValueSource::BuiltIn), "built-in");

        let mut config = ConfigFile::default();
        config.routing.planning =
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() });
        config.routing.implementation =
            Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() });
        config.routing.verification =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() });
        config.routing.review =
            Some(ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() });
        config.routing.adjudication =
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() });
        config.routing.reviewer_roles.insert(
            "security".to_string(),
            ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
        );
        config.routing.assistant_runtimes = vec![RuntimeKind::Codex, RuntimeKind::Copilot];
        config
            .routing
            .runtime_capabilities
            .insert(RuntimeKind::Claude, supported_profile(Some("supports delegated review")));
        config.routing.slot_effort_policies.insert(
            RouteSlot::Verification,
            slot_policy(
                EffortLevel::High,
                EffortFallbackPolicy::Preserve,
                Some("required for release"),
            ),
        );

        assert_eq!(route_text(config.routing.review.as_ref().unwrap()), "gemini:gemini-2.5-pro");
        assert!(
            profile_text(&supported_profile(Some("supports delegated review")))
                .contains("notes=supports delegated review")
        );
        assert!(
            effort_policy_text(&slot_policy(
                EffortLevel::High,
                EffortFallbackPolicy::Preserve,
                Some("required for release"),
            ))
            .contains("rationale=required for release")
        );

        let rendered = render_scope("cluster", &config);
        assert!(rendered.contains("config: cluster"));
        assert!(rendered.contains("planning: codex:gpt-5-codex"));
        assert!(rendered.contains("implementation: claude:sonnet-4"));
        assert!(rendered.contains("verification: copilot:gpt-5.4"));
        assert!(rendered.contains("review: gemini:gemini-2.5-pro"));
        assert!(rendered.contains("adjudication: codex:gpt-5-codex"));
        assert!(rendered.contains("- security: claude:sonnet-4"));
        assert!(rendered.contains("assistant_runtimes: codex, copilot"));
        assert!(rendered.contains("runtime_capabilities:"));
        assert!(rendered.contains("- claude: continuation=supported"));
        assert!(rendered.contains("slot_effort_policies:"));
        assert!(rendered.contains(
            "- verification: level=high, fallback=preserve, rationale=required for release"
        ));
    }

    #[test]
    fn mutation_and_scope_helpers_cover_validation_and_workspace_paths() {
        let workspace = temp_workspace("synod-cli-config-helpers");
        let config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "gpt-5-codex".to_string(),
                }),
                ..RoutingConfig::default()
            },
        };

        let saved_path =
            save_config_for_scope(Some(workspace.as_path()), ConfigWriteScope::Workspace, &config)
                .unwrap();
        assert!(saved_path.ends_with(Path::new(".synod/config.toml")));

        let (loaded, location) =
            load_config_for_scope(Some(workspace.as_path()), ConfigWriteScope::Workspace).unwrap();
        assert_eq!(loaded.routing.planning.unwrap().model, "gpt-5-codex");
        assert!(location.contains(".synod/config.toml"));

        assert!(matches!(
            mutation_target(None, None, false),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            mutation_target(Some(RouteSlot::Planning), Some("security"), false),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            mutation_target(None, Some("   "), false),
            Err(ConfigCommandError::InvalidReviewerRole)
        ));
        assert!(matches!(
            mutation_target(Some(RouteSlot::Review), None, false),
            Ok(MutationTarget::Slot(RouteSlot::Review))
        ));
        assert!(matches!(
            mutation_target(None, Some(" security "), false),
            Ok(MutationTarget::Reviewer(role)) if role == "security"
        ));
        assert!(matches!(mutation_target(None, None, true), Ok(MutationTarget::Adjudicator)));

        assert!(matches!(
            load_config_for_scope(None, ConfigWriteScope::Workspace),
            Err(ConfigCommandError::WorkspaceRequired)
        ));
        assert!(matches!(
            load_config_for_scope(Some(workspace.as_path()), ConfigWriteScope::Cluster),
            Err(ConfigCommandError::ClusterRequired)
        ));
        assert!(matches!(
            save_config_for_scope(None, ConfigWriteScope::Workspace, &ConfigFile::default()),
            Err(ConfigCommandError::WorkspaceRequired)
        ));
        assert!(matches!(
            save_config_for_scope(
                Some(workspace.as_path()),
                ConfigWriteScope::Cluster,
                &ConfigFile::default(),
            ),
            Err(ConfigCommandError::ClusterRequired)
        ));
    }

    #[test]
    fn execute_show_surfaces_workspace_cluster_effective_and_missing_scope_errors() {
        let workspace = temp_workspace("synod-cli-config-show");

        let local_config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "gpt-5-codex".to_string(),
                }),
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-5.4".to_string(),
                }),
                reviewer_roles: BTreeMap::from([(
                    "security".to_string(),
                    ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-5.4".to_string() },
                )]),
                assistant_runtimes: vec![RuntimeKind::Codex, RuntimeKind::Copilot],
                ..RoutingConfig::default()
            },
        };
        FileConfigStore::for_workspace(&workspace).save_local(&local_config).unwrap();

        let mut cluster_config = build_cluster_config(&workspace);
        cluster_config.routing.implementation =
            Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() });
        cluster_config.routing.runtime_capabilities.insert(
            RuntimeKind::Claude,
            RuntimeCapabilityProfile {
                continuation: CapabilityState::Unsupported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Unsupported,
                escalation_context: CapabilityState::Supported,
                notes: Some("requires escalation for implementation".to_string()),
            },
        );
        cluster_config.routing.slot_effort_policies.insert(
            RouteSlot::Verification,
            slot_policy(
                EffortLevel::High,
                EffortFallbackPolicy::Preserve,
                Some("cluster validation bar"),
            ),
        );
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();

        let workspace_view =
            execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Workspace))
                .unwrap();
        assert_eq!(workspace_view.exit_status, CommandExitStatus::Succeeded);
        assert!(workspace_view.terminal_output.contains("config: workspace"));
        assert!(workspace_view.terminal_output.contains("reviewer_roles:"));
        assert!(workspace_view.terminal_output.contains("assistant_runtimes: codex, copilot"));

        let cluster_view = execute_show(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            Some(ConfigShowScope::Cluster),
        )
        .unwrap();
        assert!(cluster_view.terminal_output.contains("config: cluster"));
        assert!(cluster_view.terminal_output.contains("implementation: claude:sonnet-4"));
        assert!(cluster_view.terminal_output.contains("runtime_capabilities:"));

        let effective_view = execute_show(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            Some(ConfigShowScope::Effective),
        )
        .unwrap();
        assert!(effective_view.terminal_output.contains("config: effective"));
        assert!(effective_view.terminal_output.contains("planning: codex:gpt-5-codex [workspace]"));
        assert!(
            effective_view.terminal_output.contains("implementation: claude:sonnet-4 [cluster]")
        );
        assert!(effective_view.terminal_output.contains("runtime_capabilities:"));
        assert!(effective_view.terminal_output.contains("slot_effort_policies:"));
        assert!(effective_view
            .terminal_output
            .contains("- verification: level=high, fallback=preserve, rationale=cluster validation bar [cluster]"));

        assert!(matches!(
            execute_show(None, None, Some(ConfigShowScope::Workspace)),
            Err(ConfigCommandError::WorkspaceRequired)
        ));
        assert!(matches!(
            execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Cluster)),
            Err(ConfigCommandError::ClusterRequired)
        ));

        let missing_cluster_workspace = temp_workspace("synod-cli-config-show-missing-cluster");
        assert!(matches!(
            execute_show(
                Some(missing_cluster_workspace.as_path()),
                Some(missing_cluster_workspace.as_path()),
                Some(ConfigShowScope::Cluster),
            ),
            Err(ConfigCommandError::MissingClusterConfig(_))
        ));
    }

    #[test]
    fn execute_set_and_unset_cover_workspace_and_cluster_targets() {
        let workspace = temp_workspace("synod-cli-config-set-unset");

        let slot_report = execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: Some(RouteSlot::Planning),
            reviewer: None,
            adjudicator: false,
            runtime: RuntimeKind::Codex,
            model: "gpt-5-codex",
        })
        .unwrap();
        assert!(slot_report.terminal_output.contains("workspace config"));

        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: None,
            reviewer: Some("security"),
            adjudicator: false,
            runtime: RuntimeKind::Claude,
            model: "sonnet-4",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: None,
            reviewer: None,
            adjudicator: true,
            runtime: RuntimeKind::Copilot,
            model: "gpt-5.4",
        })
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(local.routing.planning.unwrap().runtime, RuntimeKind::Codex);
        assert_eq!(
            local.routing.reviewer_roles.get("security").unwrap().runtime,
            RuntimeKind::Claude
        );
        assert_eq!(local.routing.adjudication.unwrap().runtime, RuntimeKind::Copilot);

        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            Some(RouteSlot::Planning),
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            None,
            Some("security"),
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            None,
            None,
            true,
        )
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(local.routing.planning.is_none());
        assert!(local.routing.reviewer_roles.is_empty());
        assert!(local.routing.adjudication.is_none());

        let cluster_config = build_cluster_config(&workspace);
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();

        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: Some(RouteSlot::Implementation),
            reviewer: None,
            adjudicator: false,
            runtime: RuntimeKind::Claude,
            model: "sonnet-4",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: None,
            reviewer: Some("safety"),
            adjudicator: false,
            runtime: RuntimeKind::Copilot,
            model: "gpt-5.4",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: None,
            reviewer: None,
            adjudicator: true,
            runtime: RuntimeKind::Codex,
            model: "gpt-5-codex",
        })
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert_eq!(cluster.routing.implementation.unwrap().runtime, RuntimeKind::Claude);
        assert_eq!(
            cluster.routing.reviewer_roles.get("safety").unwrap().runtime,
            RuntimeKind::Copilot
        );
        assert_eq!(cluster.routing.adjudication.unwrap().runtime, RuntimeKind::Codex);

        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            Some(RouteSlot::Implementation),
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            None,
            Some("safety"),
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            None,
            None,
            true,
        )
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert!(cluster.routing.implementation.is_none());
        assert!(cluster.routing.reviewer_roles.is_empty());
        assert!(cluster.routing.adjudication.is_none());

        assert!(matches!(
            execute_set(SetConfigRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                slot: None,
                reviewer: None,
                adjudicator: false,
                runtime: RuntimeKind::Codex,
                model: "gpt-5-codex",
            }),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            execute_set(SetConfigRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                slot: Some(RouteSlot::Planning),
                reviewer: None,
                adjudicator: false,
                runtime: RuntimeKind::Codex,
                model: "   ",
            }),
            Err(ConfigCommandError::InvalidRoute(_))
        ));
    }

    #[test]
    fn capability_and_effort_commands_cover_workspace_cluster_and_invalid_policy_paths() {
        let workspace = temp_workspace("synod-cli-config-policy");

        let capability_report = execute_set_capability(SetCapabilityRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            runtime: RuntimeKind::Claude,
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: Some("workspace capability"),
        })
        .unwrap();
        assert!(
            capability_report
                .terminal_output
                .contains("updated runtime capability in workspace config")
        );
        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(
            local.routing.runtime_capabilities.get(&RuntimeKind::Claude).unwrap().notes.as_deref(),
            Some("workspace capability")
        );

        execute_unset_capability(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            RuntimeKind::Claude,
        )
        .unwrap();
        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(!local.routing.runtime_capabilities.contains_key(&RuntimeKind::Claude));

        let effort_report = execute_set_effort(SetEffortRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: RouteSlot::Planning,
            level: EffortLevel::Max,
            fallback: EffortFallbackPolicy::Preserve,
            rationale: Some("workspace planning depth"),
        })
        .unwrap();
        assert!(
            effort_report
                .terminal_output
                .contains("updated slot effort policy in workspace config")
        );
        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(
            local.routing.slot_effort_policies.get(&RouteSlot::Planning).unwrap().level,
            EffortLevel::Max
        );

        execute_unset_effort(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            RouteSlot::Planning,
        )
        .unwrap();
        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(!local.routing.slot_effort_policies.contains_key(&RouteSlot::Planning));

        let cluster_config = build_cluster_config(&workspace);
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();

        execute_set_capability(SetCapabilityRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            runtime: RuntimeKind::Copilot,
            continuation: CapabilityState::Supported,
            resume: CapabilityState::Supported,
            validation: CapabilityState::Supported,
            handoff_target: CapabilityState::Supported,
            escalation_context: CapabilityState::Supported,
            notes: Some("cluster capability"),
        })
        .unwrap();
        execute_set_effort(SetEffortRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: RouteSlot::Verification,
            level: EffortLevel::High,
            fallback: EffortFallbackPolicy::AllowLower,
            rationale: Some("cluster verification depth"),
        })
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert!(cluster.routing.runtime_capabilities.contains_key(&RuntimeKind::Copilot));
        assert!(cluster.routing.slot_effort_policies.contains_key(&RouteSlot::Verification));

        execute_unset_capability(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            RuntimeKind::Copilot,
        )
        .unwrap();
        execute_unset_effort(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            RouteSlot::Verification,
        )
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert!(cluster.routing.runtime_capabilities.is_empty());
        assert!(cluster.routing.slot_effort_policies.is_empty());

        assert!(matches!(
            execute_set_capability(SetCapabilityRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                runtime: RuntimeKind::Gemini,
                continuation: CapabilityState::Unsupported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: Some("invalid handoff"),
            }),
            Err(ConfigCommandError::InvalidPolicy(_))
        ));
        assert!(matches!(
            execute_set_effort(SetEffortRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                slot: RouteSlot::Review,
                level: EffortLevel::Low,
                fallback: EffortFallbackPolicy::AllowLower,
                rationale: Some("   "),
            }),
            Err(ConfigCommandError::InvalidPolicy(_))
        ));
        assert!(
            execute_set_capability(SetCapabilityRequest {
                workspace: Some(workspace.as_path()),
                cluster: Some(workspace.as_path()),
                scope: ConfigWriteScope::Cluster,
                runtime: RuntimeKind::Gemini,
                continuation: CapabilityState::Supported,
                resume: CapabilityState::Supported,
                validation: CapabilityState::Supported,
                handoff_target: CapabilityState::Supported,
                escalation_context: CapabilityState::Supported,
                notes: Some("cluster capability"),
            })
            .is_ok()
        );

        let missing_cluster_workspace = temp_workspace("synod-cli-config-policy-missing-cluster");
        assert!(matches!(
            execute_unset_capability(
                Some(missing_cluster_workspace.as_path()),
                Some(missing_cluster_workspace.as_path()),
                ConfigWriteScope::Cluster,
                RuntimeKind::Codex,
            ),
            Err(ConfigCommandError::MissingClusterConfig(_))
        ));
        assert!(matches!(
            execute_unset_effort(
                Some(missing_cluster_workspace.as_path()),
                Some(missing_cluster_workspace.as_path()),
                ConfigWriteScope::Cluster,
                RouteSlot::Planning,
            ),
            Err(ConfigCommandError::MissingClusterConfig(_))
        ));
    }
}
