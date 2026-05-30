use boundline::domain::configuration::{
    AdvancedContextConfig, ModelRoute, RoutingConfig, RoutingOverrides, RuntimeKind,
    SemanticAccelerationPolicy, SemanticAccelerationPolicyState, ValueSource,
    resolve_effective_advanced_context_config, resolve_effective_routing,
    resolve_effective_semantic_acceleration_config,
};
use boundline::domain::context_intelligence::{
    RemoteTransmissionPolicyState, RetrievalBudgets, RetrievalMode,
};

#[test]
fn effective_routing_prefers_cluster_over_global_when_workspace_is_absent() {
    let cluster = RoutingConfig {
        planning: Some(ModelRoute { runtime: RuntimeKind::Codex, model: "o4-mini".to_string() }),
        ..RoutingConfig::default()
    };
    let global = RoutingConfig {
        planning: Some(ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() }),
        ..RoutingConfig::default()
    };

    let resolved = resolve_effective_routing(
        &RoutingOverrides::default(),
        None,
        Some(&cluster),
        Some(&global),
    );
    assert_eq!(resolved.planning.source, ValueSource::Cluster);
    assert_eq!(resolved.planning.route.runtime, RuntimeKind::Codex);
}

#[test]
fn effective_routing_prefers_workspace_over_cluster() {
    let workspace = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "gpt-4o".to_string(),
        }),
        ..RoutingConfig::default()
    };
    let cluster = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "o4-mini".to_string(),
        }),
        ..RoutingConfig::default()
    };

    let resolved = resolve_effective_routing(
        &RoutingOverrides::default(),
        Some(&workspace),
        Some(&cluster),
        None,
    );
    assert_eq!(resolved.verification.source, ValueSource::Workspace);
    assert_eq!(resolved.verification.route.runtime, RuntimeKind::Copilot);
}

#[test]
fn effective_advanced_context_prefers_nearest_config_scope() {
    let workspace_policy = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Disabled,
        remote_policy: RemoteTransmissionPolicyState::Blocked,
        budgets: RetrievalBudgets { depth_limit: 3, ..RetrievalBudgets::default() },
    };
    let cluster_policy = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Local,
        remote_policy: RemoteTransmissionPolicyState::LocalOnly,
        budgets: RetrievalBudgets { depth_limit: 5, ..RetrievalBudgets::default() },
    };
    let global_policy = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Local,
        remote_policy: RemoteTransmissionPolicyState::Blocked,
        budgets: RetrievalBudgets { depth_limit: 7, ..RetrievalBudgets::default() },
    };

    let resolved = resolve_effective_advanced_context_config(
        Some(&RoutingConfig {
            advanced_context: Some(workspace_policy.clone()),
            ..RoutingConfig::default()
        }),
        Some(&RoutingConfig {
            advanced_context: Some(cluster_policy.clone()),
            ..RoutingConfig::default()
        }),
        Some(&RoutingConfig {
            advanced_context: Some(global_policy.clone()),
            ..RoutingConfig::default()
        }),
    );
    assert_eq!(resolved.source, ValueSource::Workspace);
    assert_eq!(resolved.policy, workspace_policy);

    let resolved = resolve_effective_advanced_context_config(
        None,
        Some(&RoutingConfig {
            advanced_context: Some(cluster_policy.clone()),
            ..RoutingConfig::default()
        }),
        Some(&RoutingConfig {
            advanced_context: Some(global_policy.clone()),
            ..RoutingConfig::default()
        }),
    );
    assert_eq!(resolved.source, ValueSource::Cluster);
    assert_eq!(resolved.policy, cluster_policy);

    let resolved = resolve_effective_advanced_context_config(
        None,
        None,
        Some(&RoutingConfig {
            advanced_context: Some(global_policy.clone()),
            ..RoutingConfig::default()
        }),
    );
    assert_eq!(resolved.source, ValueSource::Global);
    assert_eq!(resolved.policy, global_policy);

    let resolved = resolve_effective_advanced_context_config(None, None, None);
    assert_eq!(resolved.source, ValueSource::BuiltIn);
    assert_eq!(resolved.policy, AdvancedContextConfig::default());
}

#[test]
fn advanced_context_config_rejects_unsupported_remote_combinations() {
    let remote_mode = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Remote,
        remote_policy: RemoteTransmissionPolicyState::LocalOnly,
        budgets: RetrievalBudgets::default(),
    };
    assert_eq!(
        remote_mode.validate().unwrap_err().to_string(),
        "invalid advanced-context policy: remote retrieval mode is not supported in the local-only V1 engine"
    );

    let remote_allowed = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Local,
        remote_policy: RemoteTransmissionPolicyState::RemoteAllowed,
        budgets: RetrievalBudgets::default(),
    };
    assert_eq!(
        remote_allowed.validate().unwrap_err().to_string(),
        "invalid advanced-context policy: remote transmission is not supported in the local-only V1 engine"
    );

    let disabled_with_local_policy = AdvancedContextConfig {
        retrieval_mode: RetrievalMode::Disabled,
        remote_policy: RemoteTransmissionPolicyState::LocalOnly,
        budgets: RetrievalBudgets::default(),
    };
    assert_eq!(
        disabled_with_local_policy.validate().unwrap_err().to_string(),
        "invalid advanced-context policy: disabled retrieval requires blocked remote policy"
    );
}

#[test]
fn effective_semantic_acceleration_prefers_nearest_config_scope() {
    let workspace_policy = SemanticAccelerationPolicy {
        policy: SemanticAccelerationPolicyState::Local,
        ..SemanticAccelerationPolicy::default()
    };
    let cluster_policy = SemanticAccelerationPolicy {
        policy: SemanticAccelerationPolicyState::Disabled,
        ..SemanticAccelerationPolicy::default()
    };

    let resolved = resolve_effective_semantic_acceleration_config(
        Some(&RoutingConfig {
            semantic_acceleration: Some(workspace_policy.clone()),
            ..RoutingConfig::default()
        }),
        Some(&RoutingConfig {
            semantic_acceleration: Some(cluster_policy.clone()),
            ..RoutingConfig::default()
        }),
        None,
    );
    assert_eq!(resolved.source, ValueSource::Workspace);
    assert_eq!(resolved.policy, workspace_policy);

    let resolved = resolve_effective_semantic_acceleration_config(
        None,
        Some(&RoutingConfig {
            semantic_acceleration: Some(cluster_policy.clone()),
            ..RoutingConfig::default()
        }),
        None,
    );
    assert_eq!(resolved.source, ValueSource::Cluster);
    assert_eq!(resolved.policy, cluster_policy);

    let resolved = resolve_effective_semantic_acceleration_config(None, None, None);
    assert_eq!(resolved.source, ValueSource::BuiltIn);
    assert_eq!(resolved.policy, SemanticAccelerationPolicy::default());
}
