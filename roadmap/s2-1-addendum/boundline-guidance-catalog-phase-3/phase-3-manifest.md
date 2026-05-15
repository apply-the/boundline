# Phase 3 Manifest — Framework Guidance

## Included Guidance

```text
guidance/framework-react.md
guidance/framework-node-services.md
guidance/framework-python-services.md
guidance/framework-jvm-services.md
guidance/framework-dotnet-services.md
guardians/framework-guardian-rule-seeds.md
```

## Pillars Covered

- UI architecture
- server state
- API boundaries
- validation
- framework leakage
- service boundaries
- dependency injection
- persistence boundaries
- error mapping
- integration testing
- operational readiness

## Suggested Pack Mapping

```toml
[guidance.framework_react]
path = "guidance/framework-react.md"
applies_to = ["planning", "implementation", "review", "testing", "refactor"]

[guidance.framework_node_services]
path = "guidance/framework-node-services.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing"]

[guidance.framework_python_services]
path = "guidance/framework-python-services.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing"]

[guidance.framework_jvm_services]
path = "guidance/framework-jvm-services.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing"]

[guidance.framework_dotnet_services]
path = "guidance/framework-dotnet-services.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing"]
```

## Suggested Guardians

```toml
[guardians.react_boundary]
rules = ["react-server-client-boundary", "react-server-state", "react-effect-misuse"]

[guardians.node_service_boundary]
rules = ["node-handler-business-logic", "node-validation-boundary", "node-framework-leakage"]

[guardians.python_service_boundary]
rules = ["python-route-business-logic", "python-schema-leakage", "python-framework-domain-leakage"]

[guardians.jvm_service_boundary]
rules = ["jvm-controller-business-logic", "jvm-entity-contract-leakage", "jvm-transaction-boundary"]

[guardians.dotnet_service_boundary]
rules = ["dotnet-controller-business-logic", "dotnet-api-error-shape", "dotnet-di-boundary"]
```

## Authority Strength

Default strength: recommendation.

Framework-specific choices may be mandatory only when declared by:

- workspace override
- Canon-governed standard
- organization pack
- repository architecture decision
