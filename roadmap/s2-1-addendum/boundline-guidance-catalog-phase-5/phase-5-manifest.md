# Phase 5 Manifest — Operations, Resilience, And Supply Chain

## Included Guidance

```text
guidance/observability.md
guidance/resilience.md
guidance/operations-readiness.md
guidance/supply-chain.md
guardians/operations-supply-chain-guardian-rule-seeds.md
```

## Pillars Covered

- structured logging
- metrics
- tracing
- correlation IDs
- alertability
- SLO/SLI thinking
- timeouts
- retries
- idempotency
- circuit breakers
- bulkheads
- rollback readiness
- runbooks
- incident readiness
- dependency posture
- license risk
- vulnerability management
- package manager lockfiles
- build script risk

## Suggested Pack Mapping

```toml
[guidance.observability]
path = "guidance/observability.md"
applies_to = ["planning", "architecture", "implementation", "review", "verification", "incident"]

[guidance.resilience]
path = "guidance/resilience.md"
applies_to = ["planning", "architecture", "implementation", "review", "testing", "migration"]

[guidance.operations_readiness]
path = "guidance/operations-readiness.md"
applies_to = ["planning", "architecture", "implementation", "review", "verification", "incident", "migration"]

[guidance.supply_chain]
path = "guidance/supply-chain.md"
applies_to = ["planning", "implementation", "review", "verification", "supply-chain-analysis"]
```

## Suggested Guardians

```toml
[guardians.observability]
rules = ["missing-correlation-id", "unstructured-log", "missing-critical-metric", "trace-boundary-missing"]

[guardians.resilience]
rules = ["retry-without-idempotency", "timeout-missing", "circuit-breaker-missing", "bulkhead-missing"]

[guardians.operations_readiness]
rules = ["runbook-missing", "rollback-path-missing", "alertability-missing", "incident-evidence-missing"]

[guardians.supply_chain]
rules = ["lockfile-missing", "dependency-unpinned", "license-unknown", "install-script-risk", "vulnerability-untriaged"]
```

## Authority Strength

Default strength: recommendation.

Rules may become warning, blocker, or mandatory only when promoted by:

- workspace override
- Canon-governed standard
- organization pack
- repository operational policy
- S3/S4 governance posture
