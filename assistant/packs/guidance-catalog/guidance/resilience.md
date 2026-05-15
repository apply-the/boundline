# Resilience

A system is resilient when it can tolerate expected failure modes without uncontrolled collapse. Resilience requires understanding timeout budget, retry budget, idempotency, dependency criticality, fallback behavior, blast radius, queue/backpressure behavior, and rollback path.

## Core Principles

### Timeouts

Every remote or potentially blocking operation should have a timeout: HTTP calls, database operations, cache calls, message publishing, object storage calls, AI/model/tool calls, external command execution.

Missing timeout means a dependency can consume the caller indefinitely. Timeouts should be explicit, observable, aligned with upstream budgets, and shorter than user-facing deadlines.

### Retries

Retries are dangerous without idempotency. Before adding retries, answer:
- Is the operation idempotent?
- Can it create duplicate side effects?
- Is there an idempotency key?
- Is there a retry budget with jitter and backoff?
- Will retries amplify an outage?

Avoid: infinite retries, synchronized retries, retries around non-idempotent writes, retrying validation errors, retrying authorization failures, retrying without metrics.

### Circuit Breakers

Circuit breakers protect callers and dependencies from repeated failures. Use when dependency failure can cascade, timeout/retry alone is insufficient, the system has a meaningful fallback, or external provider reliability is variable.

Avoid: circuit breakers without observability, circuit breakers around every call without reason, hiding critical failures behind silent fallbacks.

### Bulkheads

Bulkheads isolate resources so one failing dependency or tenant cannot consume everything. Useful for: high-fanout calls, tenant isolation, background workers, thread/connection pools, queue consumers, external providers.

### Backpressure

Systems must respond to overload intentionally: bounded queues, rate limits, concurrency limits, load shedding, graceful degradation, clear rejection semantics.

Avoid: unbounded queues, unbounded goroutines/tasks/promises, memory growth as implicit buffer, silent dropped work.

### Idempotency

Idempotency is required for safe retry of write operations. Use: idempotency keys, request deduplication, natural business keys, outbox pattern, exactly-once semantics only when truly supported.

Do not assume retries are safe because tests pass.

### Fallbacks

Good fallbacks: cached read-only data with staleness disclosure, reduced functionality, queued work for later processing, graceful user-facing degradation.

Bad fallbacks: silently accepting invalid state, bypassing authorization, defaulting to allow on security failure, hiding data loss.

## Anti-Patterns

- Retry without idempotency
- No timeout on external call
- No backoff or jitter on retry
- Infinite retry loops
- Fail-open for security decisions
- Unbounded queue or parallel fanout
- Fallback that hides data loss
- Circuit breaker without visibility
- Background work without supervision
- External call inside long transaction
- AI-generated code that adds retries without idempotency or swallows provider errors

## Guardian Hooks

Guardians that apply to this guidance:
- `retry_safety`: retry-without-idempotency, infinite-retry, retry-without-backoff
- `observability`: external-call-without-telemetry (when resilience mechanisms lack visibility)
- `architecture_boundary`: dependency-direction (when resilience logic leaks into domain)
