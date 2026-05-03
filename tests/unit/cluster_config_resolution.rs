use boundline::domain::configuration::{
    ModelRoute, RoutingConfig, RoutingOverrides, RuntimeKind, ValueSource,
    resolve_effective_routing,
};

#[test]
fn effective_routing_prefers_cluster_over_global_when_workspace_is_absent() {
    let cluster = RoutingConfig {
        planning: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "gpt-5-codex".to_string(),
        }),
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
            model: "gpt-5.4".to_string(),
        }),
        ..RoutingConfig::default()
    };
    let cluster = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Codex,
            model: "gpt-5-codex".to_string(),
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
