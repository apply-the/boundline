# Operations Readiness

Software is not done when it compiles. It is done when it can be deployed, monitored, debugged, and recovered in production.

## Deployment

Automate deployments completely. Use immutable artifacts. Support rollback. Verify deployment health before routing traffic.

## Health Checks

Expose liveness and readiness probes. Liveness indicates the process is alive. Readiness indicates the service can handle traffic (dependencies available, migrations complete).

## Configuration

Externalize configuration from code. Validate configuration at startup rather than failing at first use. Document required and optional configuration with defaults.

## Graceful Shutdown

Handle termination signals. Drain in-flight requests. Close database connections cleanly. Flush metrics and logs before exit.

## Runbooks

Document incident response procedures. Include diagnostic steps, common failure modes, and recovery actions. Keep runbooks next to the code they describe.

## Capacity Planning

Know the resource requirements of the service under expected and peak load. Set resource limits. Monitor utilization trends. Plan capacity before demand exceeds supply.

## Anti-Patterns

- Manual deployment steps
- Missing health check endpoints
- Configuration validated only at point of use
- Abrupt process termination dropping in-flight requests
- Missing runbooks for on-call incidents
- No monitoring for resource exhaustion
- Deployment without rollback capability

## Guardian Hooks

Guardians that apply to this guidance:
- `observability`: missing structured logs, missing health endpoints
- `resilience`: missing graceful shutdown, missing circuit breakers
- `operations_readiness`: deployment safety, rollback capability
