# Operations Readiness Guardian

Verify that code changes include the operational concerns needed for safe production deployment: health checks, graceful shutdown, configuration validation, and rollback capability.

## Rules

### missing-health-endpoint
Services must expose health check endpoints (liveness and readiness) for orchestration platforms to manage lifecycle. New services without health endpoints are not deployment-ready.

Triggers: new service without /health or /ready endpoints, services that report healthy before dependencies are available, readiness probes that do not check critical dependencies.

### missing-graceful-shutdown
Services must handle termination signals, drain in-flight work, and close resources cleanly. Abrupt termination drops requests and corrupts state.

Triggers: services without signal handlers, missing connection draining, background workers without shutdown coordination.

### deployment-without-rollback
Every deployment must have a tested rollback path. Code changes that cannot be safely reverted need explicit migration strategies.

Triggers: destructive schema migrations without expand/contract, irreversible data transformations deployed without feature flags, breaking API changes without versioning.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all service deployments. Cross-cutting; most relevant during deployment and operations review.
