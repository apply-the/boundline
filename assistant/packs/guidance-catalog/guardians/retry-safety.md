# Retry Safety Guardian

Check retry behavior for idempotency, bounded backoff, and visible stop conditions.

## Rules

### retry-without-idempotency
Retrying a non-idempotent operation can cause duplicate side effects (double charges, duplicate messages, repeated mutations). Ensure the operation is safe to repeat before adding retry logic.

Triggers: retry loops around write operations without idempotency keys, retry on POST requests without deduplication, retry around operations that create resources.

### infinite-retry
Retry loops must have bounded attempts. Infinite retry can mask permanent failures, exhaust resources, and delay error visibility.

Triggers: retry loops without max attempt limits, missing circuit breaker for repeated failures, retry without timeout budget.

### retry-without-backoff
Immediate retry can overwhelm a struggling dependency. Use exponential backoff with jitter to spread retry load.

Triggers: retry with fixed zero delay, retry without increasing intervals, retry without jitter in distributed systems.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages. Most relevant when code interacts with external services, databases, or message queues.
