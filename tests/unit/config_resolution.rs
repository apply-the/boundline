use std::collections::BTreeMap;

use boundline::domain::configuration::{
    ModelRoute, RoutingConfig, RoutingOverrides, RuntimeKind, ValueSource,
    resolve_effective_routing,
};

#[test]
fn effective_routing_prefers_workspace_over_global() {
    let workspace = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Copilot,
            model: "gpt-5.4".to_string(),
        }),
        ..RoutingConfig::default()
    };
    let global = RoutingConfig {
        verification: Some(ModelRoute {
            runtime: RuntimeKind::Claude,
            model: "sonnet-4".to_string(),
        }),
        ..RoutingConfig::default()
    };

    let resolved = resolve_effective_routing(
        &RoutingOverrides::default(),
        Some(&workspace),
        None,
        Some(&global),
    );
    assert_eq!(resolved.verification.source, ValueSource::Workspace);
    assert_eq!(resolved.verification.route.runtime, RuntimeKind::Copilot);
}

#[test]
fn reviewer_role_routes_can_be_resolved_from_cli_overrides() {
    let mut cli = RoutingOverrides::default();
    cli.reviewer_roles.insert(
        "security".to_string(),
        ModelRoute { runtime: RuntimeKind::Claude, model: "sonnet-4".to_string() },
    );

    let mut workspace_roles = BTreeMap::new();
    workspace_roles.insert(
        "security".to_string(),
        ModelRoute { runtime: RuntimeKind::Codex, model: "gpt-5-codex".to_string() },
    );
    let workspace = RoutingConfig { reviewer_roles: workspace_roles, ..RoutingConfig::default() };

    let resolved = resolve_effective_routing(&cli, Some(&workspace), None, None);
    let security = resolved.reviewer_roles.get("security").expect("security role should exist");
    assert_eq!(security.source, ValueSource::Cli);
    assert_eq!(security.route.runtime, RuntimeKind::Claude);
}
