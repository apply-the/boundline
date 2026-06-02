use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::adapters::cluster_store::{ClusterStoreError, FileClusterStore};
use crate::adapters::config_store::{ConfigStoreError, FileConfigStore};
use crate::cli::CommandExitStatus;
use crate::cli::adapter::discovery_state_label;
use crate::domain::configuration::{
    AdapterConfigValueRecord, CanonPreferences, CapabilityState, ConfigFile, ConfigShowScope,
    ConfigWriteScope, EffortFallbackPolicy, EffortLevel, ModelRoute, PersistedAdapterConfiguration,
    RouteSlot, RoutingOverrides, RuntimeCapabilityProfile, RuntimeKind, SemanticAccelerationPolicy,
    SemanticAccelerationPolicyState, SlotEffortPolicy, ValueSource,
    resolve_effective_advanced_context_config, resolve_effective_domain_templates,
    resolve_effective_routing, resolve_effective_runtime_capabilities,
    resolve_effective_semantic_acceleration_config, resolve_effective_slot_effort_policies,
};
use crate::domain::domain_templates::{DomainFamily, ExternalContextBinding, ExternalContextKind};
use crate::domain::framework_adapter::{
    AdapterConfigCompletenessState, AdapterSelectionMode, AdapterValueKind, AdapterValueSource,
};
use crate::domain::governance::CanonModeSelectionPreference;
use crate::domain::routing_decision::RoutingDecisionProjection;

const REDACTED_ADAPTER_VALUE: &str = "<redacted>";

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
    pub chat: bool,
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

#[derive(Debug, Clone, Copy)]
pub struct SetSemanticAccelerationRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub policy: SemanticAccelerationPolicyState,
}

#[derive(Debug, Clone, Copy)]
pub struct SetDomainRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub family: DomainFamily,
    pub enable: bool,
    pub disable: bool,
    pub standards: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct BindContextRequest<'a> {
    pub workspace: Option<&'a Path>,
    pub cluster: Option<&'a Path>,
    pub scope: ConfigWriteScope,
    pub family: DomainFamily,
    pub kind: ExternalContextKind,
    pub reference: &'a str,
    pub required: bool,
    pub notes: Option<&'a str>,
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
            let scope_view = ConfigFile {
                version: config.version,
                routing: config.routing,
                canon: None,
                adapter: None,
            };
            render_scope("cluster", &scope_view)
        }
        ConfigShowScope::Effective => {
            let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
            let store = FileConfigStore::for_workspace(workspace);
            let local = store.local_routing()?;
            let local_adapter = store.local_adapter()?;
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
            let effective_advanced_context = resolve_effective_advanced_context_config(
                local.as_ref(),
                cluster_routing.as_ref(),
                global.as_ref(),
            );
            let effective_semantic_acceleration = resolve_effective_semantic_acceleration_config(
                local.as_ref(),
                cluster_routing.as_ref(),
                global.as_ref(),
            );
            let effective_domain_templates = resolve_effective_domain_templates(
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
                "chat: {}",
                resolved.chat.as_ref().map_or_else(
                    || "<unset>".to_string(),
                    |route| format!("{} [{}]", route_text(&route.route), source_text(route.source))
                )
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

            lines.push(format!(
                "advanced_context: {} [{}]",
                advanced_context_policy_text(&effective_advanced_context.policy),
                source_text(effective_advanced_context.source)
            ));
            lines.push(format!(
                "semantic_acceleration: {} [{}]",
                semantic_acceleration_policy_text(&effective_semantic_acceleration.policy),
                source_text(effective_semantic_acceleration.source)
            ));

            push_effective_domain_template_lines(&mut lines, &effective_domain_templates);
            push_effective_adapter_lines(&mut lines, local_adapter.as_ref());

            lines.join("\n")
        }
    };

    Ok(ConfigCommandReport { exit_status: CommandExitStatus::Succeeded, terminal_output: output })
}

pub fn execute_set(
    request: SetConfigRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let target =
        mutation_target(request.slot, request.chat, request.reviewer, request.adjudicator)?;
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
            MutationTarget::Chat => config.routing.chat = Some(route),
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
        MutationTarget::Chat => config.routing.chat = Some(route),
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

pub fn execute_set_canon(
    workspace: Option<&Path>,
    mode_selection: CanonModeSelectionPreference,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let workspace = workspace.ok_or(ConfigCommandError::WorkspaceRequired)?;
    let store = FileConfigStore::for_workspace(workspace);
    let mut config = store.load_local()?.unwrap_or_default();
    let mut canon = config.canon.unwrap_or(CanonPreferences {
        mode_selection,
        default_risk: None,
        default_zone: None,
        default_owner: None,
        default_system_context: None,
    });
    canon.mode_selection = mode_selection;
    config.canon = Some(canon);
    let path = store.save_local(&config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!(
            "config: updated Canon preferences in workspace config at {}\nmode_selection: {}",
            path.display(),
            mode_selection
        ),
    })
}

pub fn execute_set_semantic_acceleration(
    request: SetSemanticAccelerationRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let policy = SemanticAccelerationPolicy {
        policy: request.policy,
        ..SemanticAccelerationPolicy::default()
    };
    policy.validate().map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.set_semantic_acceleration_policy(policy);
        config
            .routing
            .validate()
            .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: updated semantic acceleration policy in cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;
    config.routing.set_semantic_acceleration_policy(policy);
    config
        .routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidPolicy(source.to_string()))?;
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated semantic acceleration policy in {location}"),
    })
}

pub fn execute_unset(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    slot: Option<RouteSlot>,
    chat: bool,
    reviewer: Option<&str>,
    adjudicator: bool,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let target = mutation_target(slot, chat, reviewer, adjudicator)?;

    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;

        match target {
            MutationTarget::Slot(slot) => config.routing.unset_slot(slot),
            MutationTarget::Chat => config.routing.chat = None,
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
        MutationTarget::Chat => config.routing.chat = None,
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

pub fn execute_set_domain(
    request: SetDomainRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if request.enable && request.disable {
        return Err(ConfigCommandError::InvalidDomainMutation(
            "select only one of --enable or --disable".to_string(),
        ));
    }
    if !request.enable && !request.disable && request.standards.is_none() {
        return Err(ConfigCommandError::InvalidDomainMutation(
            "set-domain requires --enable, --disable, or --standards".to_string(),
        ));
    }

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        apply_set_domain(&mut config.routing, request)?;
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: updated domain template in cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;
    apply_set_domain(&mut config.routing, request)?;
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated domain template in {location}"),
    })
}

pub fn execute_unset_domain(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    family: DomainFamily,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        config.routing.unset_domain_template_settings(family);
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: removed domain template from cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(workspace, scope)?;
    config.routing.unset_domain_template_settings(family);
    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: removed domain template from {location}"),
    })
}

pub fn execute_bind_context(
    request: BindContextRequest<'_>,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    let binding = ExternalContextBinding {
        kind: request.kind,
        reference: request.reference.to_string(),
        required: request.required,
        notes: request.notes.map(str::to_string),
    };
    binding
        .validate()
        .map_err(|source| ConfigCommandError::InvalidDomainMutation(source.to_string()))?;

    if request.scope == ConfigWriteScope::Cluster {
        let cluster = request.cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        apply_bind_context(&mut config.routing, request.family, binding)?;
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: updated external context binding in cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(request.workspace, request.scope)?;
    apply_bind_context(&mut config.routing, request.family, binding)?;
    save_config_for_scope(request.workspace, request.scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: updated external context binding in {location}"),
    })
}

pub fn execute_unbind_context(
    workspace: Option<&Path>,
    cluster: Option<&Path>,
    scope: ConfigWriteScope,
    family: DomainFamily,
    kind: ExternalContextKind,
    reference: &str,
) -> Result<ConfigCommandReport, ConfigCommandError> {
    if reference.trim().is_empty() {
        return Err(ConfigCommandError::InvalidDomainMutation(
            "reference cannot be empty".to_string(),
        ));
    }

    if scope == ConfigWriteScope::Cluster {
        let cluster = cluster.ok_or(ConfigCommandError::ClusterRequired)?;
        let store = FileClusterStore::for_workspace(cluster);
        let mut config = store
            .load()?
            .ok_or_else(|| ConfigCommandError::MissingClusterConfig(store.cluster_config_path()))?;
        apply_unbind_context(&mut config.routing, family, kind, reference);
        let path = store.save(&config)?;

        return Ok(ConfigCommandReport {
            exit_status: CommandExitStatus::Succeeded,
            terminal_output: format!(
                "config: removed external context binding from cluster config at {}",
                path.display()
            ),
        });
    }

    let (mut config, location) = load_config_for_scope(workspace, scope)?;
    apply_unbind_context(&mut config.routing, family, kind, reference);
    save_config_for_scope(workspace, scope, &config)?;

    Ok(ConfigCommandReport {
        exit_status: CommandExitStatus::Succeeded,
        terminal_output: format!("config: removed external context binding from {location}"),
    })
}

fn apply_set_domain(
    routing: &mut crate::domain::configuration::RoutingConfig,
    request: SetDomainRequest<'_>,
) -> Result<(), ConfigCommandError> {
    let settings = routing.domain_templates.entry(request.family).or_default();
    if request.enable {
        settings.enabled = Some(true);
    }
    if request.disable {
        settings.enabled = Some(false);
    }
    if let Some(standards) = request.standards {
        settings.standards = Some(standards.to_string());
    }
    routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidDomainMutation(source.to_string()))
}

fn apply_bind_context(
    routing: &mut crate::domain::configuration::RoutingConfig,
    family: DomainFamily,
    binding: ExternalContextBinding,
) -> Result<(), ConfigCommandError> {
    let settings = routing.domain_templates.entry(family).or_default();
    settings.enabled.get_or_insert(true);

    if let Some(existing) = settings
        .external_context_bindings
        .iter_mut()
        .find(|existing| existing.kind == binding.kind && existing.reference == binding.reference)
    {
        *existing = binding;
    } else {
        settings.external_context_bindings.push(binding);
    }

    routing
        .validate()
        .map_err(|source| ConfigCommandError::InvalidDomainMutation(source.to_string()))
}

fn apply_unbind_context(
    routing: &mut crate::domain::configuration::RoutingConfig,
    family: DomainFamily,
    kind: ExternalContextKind,
    reference: &str,
) {
    let should_remove = if let Some(settings) = routing.domain_templates.get_mut(&family) {
        settings
            .external_context_bindings
            .retain(|binding| !(binding.kind == kind && binding.reference == reference));
        settings.enabled.is_none()
            && settings.standards.is_none()
            && settings.external_context_bindings.is_empty()
    } else {
        false
    };

    if should_remove {
        routing.domain_templates.remove(&family);
    }
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
        "chat: {}",
        config.routing.chat.as_ref().map(route_text).unwrap_or_else(|| "<unset>".to_string())
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

    if let Some(policy) = config.routing.advanced_context.as_ref() {
        lines.push(format!("advanced_context: {}", advanced_context_policy_text(policy)));
    } else {
        lines.push("advanced_context: none".to_string());
    }

    if let Some(policy) = config.routing.semantic_acceleration.as_ref() {
        lines.push(format!("semantic_acceleration: {}", semantic_acceleration_policy_text(policy)));
    } else {
        lines.push("semantic_acceleration: none".to_string());
    }

    if config.routing.domain_templates.is_empty() {
        lines.push("domain_templates: none".to_string());
    } else {
        lines.push("domain_templates:".to_string());
        for (family, settings) in &config.routing.domain_templates {
            lines.push(format!(
                "- {}: enabled={}",
                family.as_str(),
                domain_enabled_text(settings.enabled)
            ));
            if let Some(standards) = settings.standards.as_deref().map(str::trim)
                && !standards.is_empty()
            {
                lines.push(format!("  standards: {standards}"));
            }
            if settings.external_context_bindings.is_empty() {
                lines.push("  external_context_bindings: none".to_string());
            } else {
                lines.push("  external_context_bindings:".to_string());
                for binding in &settings.external_context_bindings {
                    lines.push(format!("  - {}", binding_text(binding, None)));
                }
            }
        }
    }

    if let Some(canon) = config.canon.as_ref() {
        lines.push("canon:".to_string());
        lines.push(format!("  mode_selection: {}", canon.mode_selection));
        if let Some(risk) = canon.default_risk.as_deref() {
            lines.push(format!("  default_risk: {risk}"));
        }
        if let Some(zone) = canon.default_zone.as_deref() {
            lines.push(format!("  default_zone: {zone}"));
        }
        if let Some(owner) = canon.default_owner.as_deref() {
            lines.push(format!("  default_owner: {owner}"));
        }
    } else {
        lines.push("canon: none".to_string());
    }

    push_configured_adapter_lines(&mut lines, config.adapter.as_ref());

    lines.join("\n")
}

fn push_configured_adapter_lines(
    lines: &mut Vec<String>,
    adapter: Option<&PersistedAdapterConfiguration>,
) {
    let Some(adapter) = adapter else {
        lines.push("framework_adapter: none".to_string());
        return;
    };

    lines.push("framework_adapter:".to_string());
    lines.push("  status: configured".to_string());
    lines.push(format!("  adapter_id: {}", adapter.selection.adapter_id));
    lines.push(format!(
        "  selection_mode: {}",
        adapter_selection_mode_text(adapter.selection.selection_mode)
    ));
    lines.push(format!("  command: {}", adapter.selection.command));
    lines.push(format!(
        "  discovery_state: {}",
        discovery_state_label(adapter.selection.discovery_state)
    ));
    lines
        .push(format!("  config_state: {}", adapter_completeness_text(adapter.completeness_state)));
    lines.push(format!("  interactive_resolution: {}", adapter.interactive_resolution));
    lines.push(format!("  value_count: {}", adapter.value_count));
    push_workspace_adapter_value_lines(lines, adapter);
}

fn push_effective_adapter_lines(
    lines: &mut Vec<String>,
    adapter: Option<&PersistedAdapterConfiguration>,
) {
    let Some(adapter) = adapter else {
        lines.push("framework_adapter_status: built_in_default [built-in]".to_string());
        return;
    };

    lines.push("framework_adapter_status: configured [workspace]".to_string());
    lines.push(format!("framework_adapter_id: {}", adapter.selection.adapter_id));
    lines.push(format!("framework_adapter_command: {}", adapter.selection.command));
    lines.push(format!(
        "framework_adapter_discovery_state: {}",
        discovery_state_label(adapter.selection.discovery_state)
    ));
    lines.push(format!(
        "framework_adapter_config_state: {}",
        adapter_completeness_text(adapter.completeness_state)
    ));
    lines.push(format!(
        "framework_adapter_interactive_resolution: {}",
        adapter.interactive_resolution
    ));
    lines.push(format!("framework_adapter_value_count: {}", adapter.value_count));
    push_effective_adapter_value_lines(lines, adapter);
}

fn push_workspace_adapter_value_lines(
    lines: &mut Vec<String>,
    adapter: &PersistedAdapterConfiguration,
) {
    if adapter.values.is_empty() {
        return;
    }

    lines.push("  config_values:".to_string());
    for value in &adapter.values {
        lines.push(format!(
            "  - {}: {} [{}]",
            value.field_key,
            render_adapter_value(value),
            adapter_value_source_text(value.value_source)
        ));
    }
}

fn push_effective_adapter_value_lines(
    lines: &mut Vec<String>,
    adapter: &PersistedAdapterConfiguration,
) {
    if adapter.values.is_empty() {
        return;
    }

    lines.push("framework_adapter_config_values:".to_string());
    for value in &adapter.values {
        lines.push(format!(
            "- {}: {} [{}]",
            value.field_key,
            render_adapter_value(value),
            adapter_value_source_text(value.value_source)
        ));
    }
}

fn render_adapter_value(value: &AdapterConfigValueRecord) -> String {
    if value.secret {
        return REDACTED_ADAPTER_VALUE.to_string();
    }

    match value.value_kind {
        AdapterValueKind::String | AdapterValueKind::Enum => {
            value.string_value.clone().unwrap_or_else(|| "<unset>".to_string())
        }
        AdapterValueKind::Path => value.path_value.clone().unwrap_or_else(|| "<unset>".to_string()),
        AdapterValueKind::Boolean => {
            value.bool_value.map(|value| value.to_string()).unwrap_or_else(|| "<unset>".to_string())
        }
        AdapterValueKind::Integer => {
            value.int_value.map(|value| value.to_string()).unwrap_or_else(|| "<unset>".to_string())
        }
    }
}

fn adapter_value_source_text(source: AdapterValueSource) -> &'static str {
    match source {
        AdapterValueSource::CliFlag => "cli_flag",
        AdapterValueSource::KnownProfileDefault => "known_profile_default",
        AdapterValueSource::OperatorPrompt => "operator_prompt",
        AdapterValueSource::MigratedConfig => "migrated_config",
    }
}

fn adapter_selection_mode_text(selection_mode: AdapterSelectionMode) -> &'static str {
    match selection_mode {
        AdapterSelectionMode::None => "none",
        AdapterSelectionMode::KnownProfile => "known_profile",
        AdapterSelectionMode::Custom => "custom",
    }
}

fn adapter_completeness_text(completeness_state: AdapterConfigCompletenessState) -> &'static str {
    match completeness_state {
        AdapterConfigCompletenessState::Complete => "complete",
        AdapterConfigCompletenessState::MissingRequired => "missing_required",
        AdapterConfigCompletenessState::Invalid => "invalid",
    }
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

fn advanced_context_policy_text(
    policy: &crate::domain::configuration::AdvancedContextConfig,
) -> String {
    policy.summary_text()
}

fn semantic_acceleration_policy_text(policy: &SemanticAccelerationPolicy) -> String {
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

fn push_effective_domain_template_lines(
    lines: &mut Vec<String>,
    effective_domain_templates: &std::collections::BTreeMap<
        DomainFamily,
        crate::domain::configuration::ResolvedDomainTemplate,
    >,
) {
    if effective_domain_templates.is_empty() {
        lines.push("domain_templates: none".to_string());
        return;
    }

    lines.push("domain_templates:".to_string());
    for (family, template) in effective_domain_templates {
        lines.push(format!(
            "- {}: enabled={} [{}]",
            family.as_str(),
            template.enabled,
            source_text(template.enablement_source)
        ));
        let layer_sources = template
            .standards_layers
            .iter()
            .map(|layer| source_text(layer.source))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("  standards_layers: {layer_sources}"));
        if template.external_context_bindings.is_empty() {
            lines.push("  external_context_bindings: none".to_string());
        } else {
            lines.push("  external_context_bindings:".to_string());
            for binding in &template.external_context_bindings {
                lines.push(format!("  - {}", binding_text(&binding.binding, Some(binding.source))));
            }
        }
    }
}

fn domain_enabled_text(enabled: Option<bool>) -> &'static str {
    match enabled {
        Some(true) => "true",
        Some(false) => "false",
        None => "inherit",
    }
}

fn binding_text(binding: &ExternalContextBinding, source: Option<ValueSource>) -> String {
    let requirement = if binding.required { "required" } else { "optional" };
    let mut text = format!("{} {} ({requirement})", binding.kind.as_str(), binding.reference);
    if let Some(notes) = binding.notes.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        text.push_str(&format!(", notes={notes}"));
    }
    if let Some(source) = source {
        text.push_str(&format!(" [{}]", source_text(source)));
    }
    text
}

enum MutationTarget {
    Slot(RouteSlot),
    Chat,
    Reviewer(String),
    Adjudicator,
}

fn mutation_target(
    slot: Option<RouteSlot>,
    chat: bool,
    reviewer: Option<&str>,
    adjudicator: bool,
) -> Result<MutationTarget, ConfigCommandError> {
    let count = usize::from(slot.is_some())
        + usize::from(chat)
        + usize::from(reviewer.is_some())
        + usize::from(adjudicator);
    if count != 1 {
        return Err(ConfigCommandError::InvalidTargetSelection);
    }

    if let Some(slot) = slot {
        return Ok(MutationTarget::Slot(slot));
    }

    if chat {
        return Ok(MutationTarget::Chat);
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
    #[error("invalid domain mutation: {0}")]
    InvalidDomainMutation(String),
    #[error("workspace resolution error: {0}")]
    WorkspaceResolution(String),
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

    use super::{
        BindContextRequest, CapabilityState, ConfigCommandError, ConfigFile, ConfigShowScope,
        ConfigWriteScope, EffortFallbackPolicy, EffortLevel, ExternalContextBinding, ModelRoute,
        MutationTarget, RouteSlot, RuntimeCapabilityProfile, RuntimeKind,
        SemanticAccelerationPolicyState, SetCapabilityRequest, SetConfigRequest, SetDomainRequest,
        SetEffortRequest, SetSemanticAccelerationRequest, SlotEffortPolicy, ValueSource,
        binding_text, domain_enabled_text, effort_policy_text, execute_bind_context, execute_set,
        execute_set_capability, execute_set_domain, execute_set_effort,
        execute_set_semantic_acceleration, execute_show, execute_unbind_context, execute_unset,
        execute_unset_capability, execute_unset_domain, execute_unset_effort,
        load_config_for_scope, mutation_target, profile_text, push_effective_domain_template_lines,
        render_scope, route_text, save_config_for_scope, slot_label, source_text,
    };
    use crate::adapters::cluster_store::FileClusterStore;
    use crate::adapters::config_store::FileConfigStore;
    use crate::cli::CommandExitStatus;
    use crate::domain::cluster::{
        ClusterConfigFile, ClusterMemberRegistration, ClusterMemberRole, WorkspaceCluster,
    };
    use crate::domain::configuration::{
        AdapterConfigValueRecord, AdapterSelectionRecord, PersistedAdapterConfiguration,
        RoutingConfig,
    };
    use crate::domain::domain_templates::{DomainFamily, ExternalContextKind};
    use crate::domain::framework_adapter::{
        AdapterConfigCompletenessState, AdapterDiscoveryState, AdapterRegistrationSource,
        AdapterSelectionMode, AdapterValueKind, AdapterValueSource,
        FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1, StoredAdapterConfigValueState,
    };

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

    fn sample_persisted_adapter() -> PersistedAdapterConfiguration {
        PersistedAdapterConfiguration {
            selection: AdapterSelectionRecord {
                selection_mode: AdapterSelectionMode::KnownProfile,
                adapter_id: "speckit".to_string(),
                display_name: "Speckit".to_string(),
                command: "boundline-adapter-speckit".to_string(),
                args: Vec::new(),
                registration_source: AdapterRegistrationSource::AdapterAdd,
                discovery_state: AdapterDiscoveryState::DiscoveredOnPath,
                compatibility_line: FRAMEWORK_ADAPTER_PROTOCOL_LINE_V1.to_string(),
                updated_at: 1,
            },
            schema_fingerprint: "schema-v1".to_string(),
            completeness_state: AdapterConfigCompletenessState::Complete,
            interactive_resolution: false,
            last_validated_at: Some(1),
            value_count: 0,
            values: Vec::new(),
        }
    }

    fn sample_adapter_value(
        field_key: &str,
        value_kind: AdapterValueKind,
        value_source: AdapterValueSource,
    ) -> AdapterConfigValueRecord {
        AdapterConfigValueRecord {
            field_key: field_key.to_string(),
            value_kind,
            secret: false,
            string_value: None,
            path_value: None,
            bool_value: None,
            int_value: None,
            value_source,
            resolution_state: StoredAdapterConfigValueState::Present,
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
        assert!(empty.contains("chat: <unset>"));
        assert!(empty.contains("reviewer_roles: none"));
        assert!(empty.contains("assistant_runtimes: none"));
        assert!(empty.contains("runtime_capabilities: none"));
        assert!(empty.contains("slot_effort_policies: none"));
        assert!(empty.contains("advanced_context: none"));
        assert!(empty.contains("semantic_acceleration: none"));
        assert!(empty.contains("framework_adapter: none"));

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
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() });
        config.routing.implementation =
            Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() });
        config.routing.verification =
            Some(ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() });
        config.routing.review =
            Some(ModelRoute { runtime: RuntimeKind::Gemini, model: "gemini-2.5-pro".to_string() });
        config.routing.chat =
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "openai/gpt-5.4".to_string() });
        config.routing.adjudication =
            Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() });
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
        config.routing.advanced_context =
            Some(crate::domain::configuration::AdvancedContextConfig {
                retrieval_mode: crate::domain::context_intelligence::RetrievalMode::Local,
                remote_policy:
                    crate::domain::context_intelligence::RemoteTransmissionPolicyState::LocalOnly,
                budgets: crate::domain::context_intelligence::RetrievalBudgets::default(),
            });
        config.routing.semantic_acceleration =
            Some(crate::domain::configuration::SemanticAccelerationPolicy {
                policy: crate::domain::configuration::SemanticAccelerationPolicyState::Local,
                ..crate::domain::configuration::SemanticAccelerationPolicy::default()
            });
        config.adapter = Some(sample_persisted_adapter());

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
        assert!(rendered.contains("planning: codex:o4-mini"));
        assert!(rendered.contains("implementation: claude:sonnet-4"));
        assert!(rendered.contains("verification: copilot:gpt-4o"));
        assert!(rendered.contains("review: gemini:gemini-2.5-pro"));
        assert!(rendered.contains("chat: codex:openai/gpt-5.4"));
        assert!(rendered.contains("adjudication: codex:o4-mini"));
        assert!(rendered.contains("- security: claude:sonnet-4"));
        assert!(rendered.contains("assistant_runtimes: codex, copilot"));
        assert!(rendered.contains("runtime_capabilities:"));
        assert!(rendered.contains("- claude: continuation=supported"));
        assert!(rendered.contains("slot_effort_policies:"));
        assert!(rendered.contains(
            "- verification: level=high, fallback=preserve, rationale=required for release"
        ));
        assert!(rendered.contains("advanced_context: mode=local, remote_policy=local_only"));
        assert!(rendered.contains("semantic_acceleration: policy=local"));
        assert!(rendered.contains("framework_adapter:"));
        assert!(rendered.contains("  adapter_id: speckit"));
        assert!(rendered.contains("  command: boundline-adapter-speckit"));
        assert!(rendered.contains("  discovery_state: discovered_on_path"));
    }

    #[test]
    fn mutation_and_scope_helpers_cover_validation_and_workspace_paths() {
        let workspace = temp_workspace("boundline-cli-config-helpers");
        let config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "o4-mini".to_string(),
                }),
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: None,
        };

        let saved_path =
            save_config_for_scope(Some(workspace.as_path()), ConfigWriteScope::Workspace, &config)
                .unwrap();
        assert!(saved_path.ends_with(Path::new(".boundline/config.toml")));

        let (loaded, location) =
            load_config_for_scope(Some(workspace.as_path()), ConfigWriteScope::Workspace).unwrap();
        assert_eq!(loaded.routing.planning.unwrap().model, "o4-mini");
        assert!(location.contains(".boundline/config.toml"));

        assert!(matches!(
            mutation_target(None, false, None, false),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            mutation_target(Some(RouteSlot::Planning), false, Some("security"), false),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            mutation_target(None, false, Some("   "), false),
            Err(ConfigCommandError::InvalidReviewerRole)
        ));
        assert!(matches!(
            mutation_target(Some(RouteSlot::Review), false, None, false),
            Ok(MutationTarget::Slot(RouteSlot::Review))
        ));
        assert!(matches!(
            mutation_target(None, false, Some(" security "), false),
            Ok(MutationTarget::Reviewer(role)) if role == "security"
        ));
        assert!(matches!(mutation_target(None, true, None, false), Ok(MutationTarget::Chat)));
        assert!(matches!(
            mutation_target(None, false, None, true),
            Ok(MutationTarget::Adjudicator)
        ));

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
        let workspace = temp_workspace("boundline-cli-config-show");

        let local_config = ConfigFile {
            version: 1,
            routing: RoutingConfig {
                planning: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "o4-mini".to_string(),
                }),
                chat: Some(ModelRoute {
                    runtime: RuntimeKind::Codex,
                    model: "openai/gpt-5.4".to_string(),
                }),
                review: Some(ModelRoute {
                    runtime: RuntimeKind::Copilot,
                    model: "gpt-4o".to_string(),
                }),
                reviewer_roles: BTreeMap::from([(
                    "security".to_string(),
                    ModelRoute { runtime: RuntimeKind::Copilot, model: "gpt-4o".to_string() },
                )]),
                assistant_runtimes: vec![RuntimeKind::Codex, RuntimeKind::Copilot],
                ..RoutingConfig::default()
            },
            canon: None,
            adapter: Some(sample_persisted_adapter()),
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
        assert!(workspace_view.terminal_output.contains("chat: codex:openai/gpt-5.4"));
        assert!(workspace_view.terminal_output.contains("reviewer_roles:"));
        assert!(workspace_view.terminal_output.contains("assistant_runtimes: codex, copilot"));
        assert!(workspace_view.terminal_output.contains("framework_adapter:"));
        assert!(workspace_view.terminal_output.contains("  adapter_id: speckit"));

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
        assert!(effective_view.terminal_output.contains("planning: codex:o4-mini [workspace]"));
        assert!(effective_view.terminal_output.contains("chat: codex:openai/gpt-5.4 [workspace]"));
        assert!(
            effective_view.terminal_output.contains("implementation: claude:sonnet-4 [cluster]")
        );
        assert!(effective_view.terminal_output.contains("runtime_capabilities:"));
        assert!(effective_view.terminal_output.contains("slot_effort_policies:"));
        assert!(effective_view.terminal_output.contains(
            "semantic_acceleration: policy=disabled, index_hook_action=disabled [built-in]"
        ));
        assert!(
            effective_view
                .terminal_output
                .contains("framework_adapter_status: configured [workspace]")
        );
        assert!(effective_view.terminal_output.contains("framework_adapter_id: speckit"));
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

        let missing_cluster_workspace = temp_workspace("boundline-cli-config-show-missing-cluster");
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
    fn adapter_value_rendering_covers_redaction_and_built_in_effective_defaults() {
        let workspace = temp_workspace("boundline-cli-config-adapter-values");
        let mut adapter = sample_persisted_adapter();
        let mut plain_value = sample_adapter_value(
            "workspace_root",
            AdapterValueKind::Path,
            AdapterValueSource::CliFlag,
        );
        plain_value.path_value = Some("./fixtures/workspace".to_string());

        let mut secret_value = sample_adapter_value(
            "api_token",
            AdapterValueKind::String,
            AdapterValueSource::OperatorPrompt,
        );
        secret_value.secret = true;
        secret_value.string_value = Some("super-secret".to_string());

        let mut bool_value = sample_adapter_value(
            "strict_mode",
            AdapterValueKind::Boolean,
            AdapterValueSource::KnownProfileDefault,
        );
        bool_value.bool_value = Some(true);

        let mut int_value = sample_adapter_value(
            "max_attempts",
            AdapterValueKind::Integer,
            AdapterValueSource::MigratedConfig,
        );
        int_value.int_value = Some(7);

        adapter.value_count = 4;
        adapter.values = vec![plain_value, secret_value, bool_value, int_value];

        let config = ConfigFile { adapter: Some(adapter), ..ConfigFile::default() };

        let rendered = render_scope("workspace", &config);
        assert!(rendered.contains("framework_adapter:"));
        assert!(rendered.contains("  config_values:"));
        assert!(rendered.contains("  - workspace_root: ./fixtures/workspace [cli_flag]"));
        assert!(rendered.contains("  - api_token: <redacted> [operator_prompt]"));
        assert!(rendered.contains("  - strict_mode: true [known_profile_default]"));
        assert!(rendered.contains("  - max_attempts: 7 [migrated_config]"));

        FileConfigStore::for_workspace(&workspace).save_local(&config).unwrap();
        let effective =
            execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Effective))
                .unwrap();
        assert!(
            effective.terminal_output.contains("framework_adapter_status: configured [workspace]")
        );
        assert!(effective.terminal_output.contains("framework_adapter_config_values:"));
        assert!(effective.terminal_output.contains("- api_token: <redacted> [operator_prompt]"));

        let built_in_workspace = temp_workspace("boundline-cli-config-adapter-built-in");
        let built_in_effective = execute_show(
            Some(built_in_workspace.as_path()),
            None,
            Some(ConfigShowScope::Effective),
        )
        .unwrap();
        assert!(
            built_in_effective
                .terminal_output
                .contains("framework_adapter_status: built_in_default [built-in]")
        );
    }

    #[test]
    fn execute_set_and_unset_cover_workspace_and_cluster_targets() {
        let workspace = temp_workspace("boundline-cli-config-set-unset");

        let slot_report = execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: Some(RouteSlot::Planning),
            chat: false,
            reviewer: None,
            adjudicator: false,
            runtime: RuntimeKind::Codex,
            model: "o4-mini",
        })
        .unwrap();
        assert!(slot_report.terminal_output.contains("workspace config"));

        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: None,
            chat: true,
            reviewer: None,
            adjudicator: false,
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-5.4",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            slot: None,
            chat: false,
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
            chat: false,
            reviewer: None,
            adjudicator: true,
            runtime: RuntimeKind::Copilot,
            model: "gpt-4o",
        })
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(local.routing.planning.unwrap().runtime, RuntimeKind::Codex);
        assert_eq!(local.routing.chat.unwrap().model, "openai/gpt-5.4");
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
            false,
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            None,
            true,
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            None,
            false,
            Some("security"),
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            None,
            false,
            None,
            true,
        )
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(local.routing.planning.is_none());
        assert!(local.routing.chat.is_none());
        assert!(local.routing.reviewer_roles.is_empty());
        assert!(local.routing.adjudication.is_none());

        let cluster_config = build_cluster_config(&workspace);
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();

        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: Some(RouteSlot::Implementation),
            chat: false,
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
            chat: true,
            reviewer: None,
            adjudicator: false,
            runtime: RuntimeKind::Codex,
            model: "openai/gpt-5.4",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: None,
            chat: false,
            reviewer: Some("safety"),
            adjudicator: false,
            runtime: RuntimeKind::Copilot,
            model: "gpt-4o",
        })
        .unwrap();
        execute_set(SetConfigRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            slot: None,
            chat: false,
            reviewer: None,
            adjudicator: true,
            runtime: RuntimeKind::Codex,
            model: "o4-mini",
        })
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert_eq!(cluster.routing.implementation.unwrap().runtime, RuntimeKind::Claude);
        assert_eq!(cluster.routing.chat.unwrap().model, "openai/gpt-5.4");
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
            false,
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            None,
            true,
            None,
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            None,
            false,
            Some("safety"),
            false,
        )
        .unwrap();
        execute_unset(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            None,
            false,
            None,
            true,
        )
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert!(cluster.routing.implementation.is_none());
        assert!(cluster.routing.chat.is_none());
        assert!(cluster.routing.reviewer_roles.is_empty());
        assert!(cluster.routing.adjudication.is_none());

        assert!(matches!(
            execute_set(SetConfigRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                slot: None,
                chat: false,
                reviewer: None,
                adjudicator: false,
                runtime: RuntimeKind::Codex,
                model: "o4-mini",
            }),
            Err(ConfigCommandError::InvalidTargetSelection)
        ));
        assert!(matches!(
            execute_set(SetConfigRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                slot: Some(RouteSlot::Planning),
                chat: false,
                reviewer: None,
                adjudicator: false,
                runtime: RuntimeKind::Codex,
                model: "   ",
            }),
            Err(ConfigCommandError::InvalidRoute(_))
        ));
    }

    #[test]
    fn execute_set_semantic_acceleration_updates_workspace_and_cluster_targets() {
        let workspace = temp_workspace("boundline-cli-config-semantic-acceleration");

        let workspace_report = execute_set_semantic_acceleration(SetSemanticAccelerationRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            policy: SemanticAccelerationPolicyState::Local,
        })
        .unwrap();
        assert!(
            workspace_report
                .terminal_output
                .contains("semantic acceleration policy in workspace config")
        );

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert_eq!(
            local.routing.semantic_acceleration.unwrap().policy,
            SemanticAccelerationPolicyState::Local
        );

        let cluster_config = build_cluster_config(&workspace);
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();
        let cluster_report = execute_set_semantic_acceleration(SetSemanticAccelerationRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            policy: SemanticAccelerationPolicyState::Disabled,
        })
        .unwrap();
        assert!(
            cluster_report
                .terminal_output
                .contains("semantic acceleration policy in cluster config")
        );

        let effective_view = execute_show(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            Some(ConfigShowScope::Effective),
        )
        .unwrap();
        assert!(effective_view.terminal_output.contains(
            "semantic_acceleration: policy=local, index_hook_action=disabled [workspace]"
        ));
    }

    #[test]
    fn capability_and_effort_commands_cover_workspace_cluster_and_invalid_policy_paths() {
        let workspace = temp_workspace("boundline-cli-config-policy");

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

        let missing_cluster_workspace =
            temp_workspace("boundline-cli-config-policy-missing-cluster");
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

    #[test]
    fn domain_template_commands_cover_workspace_mutations_and_effective_rendering() {
        let workspace = temp_workspace("boundline-cli-config-domain");

        let report = execute_set_domain(SetDomainRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            family: DomainFamily::React,
            enable: true,
            disable: false,
            standards: Some("workspace react rules"),
        })
        .unwrap();
        assert!(report.terminal_output.contains("updated domain template"));

        execute_bind_context(BindContextRequest {
            workspace: Some(workspace.as_path()),
            cluster: None,
            scope: ConfigWriteScope::Workspace,
            family: DomainFamily::React,
            kind: ExternalContextKind::DesignSystem,
            reference: "mcp:design-system",
            required: true,
            notes: Some("shared system"),
        })
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        let settings = local.routing.domain_templates.get(&DomainFamily::React).unwrap();
        assert_eq!(settings.enabled, Some(true));
        assert_eq!(settings.standards.as_deref(), Some("workspace react rules"));
        assert_eq!(settings.external_context_bindings.len(), 1);

        let effective =
            execute_show(Some(workspace.as_path()), None, Some(ConfigShowScope::Effective))
                .unwrap();
        assert!(effective.terminal_output.contains("domain_templates:"));
        assert!(effective.terminal_output.contains("- react: enabled=true [workspace]"));
        assert!(effective.terminal_output.contains("standards_layers: built-in, workspace"));
        assert!(effective.terminal_output.contains(
            "design_system mcp:design-system (required), notes=shared system [workspace]"
        ));

        execute_unbind_context(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            DomainFamily::React,
            ExternalContextKind::DesignSystem,
            "mcp:design-system",
        )
        .unwrap();
        execute_unset_domain(
            Some(workspace.as_path()),
            None,
            ConfigWriteScope::Workspace,
            DomainFamily::React,
        )
        .unwrap();

        let local = FileConfigStore::for_workspace(&workspace).load_local().unwrap().unwrap();
        assert!(!local.routing.domain_templates.contains_key(&DomainFamily::React));

        assert!(matches!(
            execute_set_domain(SetDomainRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                family: DomainFamily::React,
                enable: true,
                disable: true,
                standards: None,
            }),
            Err(ConfigCommandError::InvalidDomainMutation(_))
        ));
    }

    #[test]
    fn domain_template_commands_cover_cluster_mutations_and_helper_rendering() {
        let workspace = temp_workspace("boundline-cli-config-domain-cluster");
        let cluster_config = build_cluster_config(&workspace);
        FileClusterStore::for_workspace(&workspace).save(&cluster_config).unwrap();

        let report = execute_set_domain(SetDomainRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            family: DomainFamily::React,
            enable: false,
            disable: true,
            standards: Some("cluster react rules"),
        })
        .unwrap();
        assert!(report.terminal_output.contains("cluster config"));

        execute_bind_context(BindContextRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            family: DomainFamily::React,
            kind: ExternalContextKind::DesignTokens,
            reference: "design/tokens.json",
            required: false,
            notes: Some("first"),
        })
        .unwrap();
        execute_bind_context(BindContextRequest {
            workspace: Some(workspace.as_path()),
            cluster: Some(workspace.as_path()),
            scope: ConfigWriteScope::Cluster,
            family: DomainFamily::React,
            kind: ExternalContextKind::DesignTokens,
            reference: "design/tokens.json",
            required: false,
            notes: Some("updated"),
        })
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        let settings = cluster.routing.domain_templates.get(&DomainFamily::React).unwrap();
        assert_eq!(settings.enabled, Some(false));
        assert_eq!(settings.standards.as_deref(), Some("cluster react rules"));
        assert_eq!(settings.external_context_bindings.len(), 1);
        assert_eq!(settings.external_context_bindings[0].notes.as_deref(), Some("updated"));

        let rendered = render_scope(
            "cluster",
            &ConfigFile {
                version: cluster.version,
                routing: cluster.routing.clone(),
                canon: None,
                adapter: None,
            },
        );
        assert!(rendered.contains("- react: enabled=false"));
        assert!(rendered.contains("standards: cluster react rules"));
        assert!(rendered.contains("design_tokens design/tokens.json (optional), notes=updated"));

        let helper_rendered = render_scope(
            "workspace",
            &ConfigFile {
                version: 1,
                routing: RoutingConfig {
                    domain_templates: BTreeMap::from([(
                        DomainFamily::Vue,
                        crate::domain::domain_templates::DomainTemplateSettings {
                            enabled: None,
                            standards: Some("shared vue rules".to_string()),
                            external_context_bindings: Vec::new(),
                        },
                    )]),
                    ..RoutingConfig::default()
                },
                canon: None,
                adapter: None,
            },
        );
        assert!(helper_rendered.contains("- vue: enabled=inherit"));
        assert!(helper_rendered.contains("standards: shared vue rules"));
        assert!(helper_rendered.contains("external_context_bindings: none"));

        let mut effective_lines = Vec::new();
        push_effective_domain_template_lines(&mut effective_lines, &BTreeMap::new());
        assert_eq!(effective_lines, vec!["domain_templates: none".to_string()]);
        assert_eq!(domain_enabled_text(None), "inherit");
        assert_eq!(domain_enabled_text(Some(false)), "false");

        let binding = ExternalContextBinding {
            kind: ExternalContextKind::DesignTokens,
            reference: "design/tokens.json".to_string(),
            required: false,
            notes: None,
        };
        assert_eq!(binding_text(&binding, None), "design_tokens design/tokens.json (optional)");
        assert_eq!(
            binding_text(&binding, Some(ValueSource::Cluster)),
            "design_tokens design/tokens.json (optional) [cluster]"
        );

        assert!(matches!(
            execute_unbind_context(
                Some(workspace.as_path()),
                Some(workspace.as_path()),
                ConfigWriteScope::Cluster,
                DomainFamily::React,
                ExternalContextKind::DesignTokens,
                "   ",
            ),
            Err(ConfigCommandError::InvalidDomainMutation(_))
        ));

        execute_unbind_context(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            DomainFamily::React,
            ExternalContextKind::DesignTokens,
            "design/tokens.json",
        )
        .unwrap();
        execute_unset_domain(
            Some(workspace.as_path()),
            Some(workspace.as_path()),
            ConfigWriteScope::Cluster,
            DomainFamily::React,
        )
        .unwrap();

        let cluster = FileClusterStore::for_workspace(&workspace).load().unwrap().unwrap();
        assert!(!cluster.routing.domain_templates.contains_key(&DomainFamily::React));

        assert!(matches!(
            execute_set_domain(SetDomainRequest {
                workspace: Some(workspace.as_path()),
                cluster: None,
                scope: ConfigWriteScope::Workspace,
                family: DomainFamily::React,
                enable: false,
                disable: false,
                standards: Some("   "),
            }),
            Err(ConfigCommandError::InvalidDomainMutation(_))
        ));
    }
}
