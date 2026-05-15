# Resilience Guidance

## Purpose

This guidance defines resilience expectations for AI-assisted delivery.

It applies to services, APIs, workers, integrations, migrations, scheduled jobs, queues, and systems that depend on external resources.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, operations policy, architecture decision, or Canon-governed standard.

## Core Thesis

A system is resilient when it can tolerate expected failure modes without uncontrolled collapse.

Resilience is not "add retries".

Resilience requires understanding:

- timeout budget
- retry budget
- idempotency
- dependency criticality
- fallback behavior
- blast radius
- queue/backpressure behavior
- rollback path

## Timeouts

Every remote or potentially blocking operation should have a timeout.

Examples:
- HTTP calls
- database operations
- cache calls
- message publishing
- object storage calls
- AI/model/tool calls
- external command execution

Missing timeout means a dependency can consume the caller indefinitely.

Timeouts should be:
- explicit
- observable
- aligned with upstream budgets
- shorter than user-facing or workflow-level deadlines

## Retries

Retries are dangerous without idempotency.

Before adding retries, answer:
- is the operation idempotent?
- can it create duplicate side effects?
- is there an idempotency key?
- is there a retry budget?
- is there jitter?
- is there backoff?
- will retries amplify an outage?

Avoid:
- infinite retries
- synchronized retries
- retries around non-idempotent writes
- retrying validation errors
- retrying authorization failures
- retrying without metrics

## Circuit Breakers

Circuit breakers protect callers and dependencies from repeated failures.

Use when:
- dependency failure can cascade
- timeout/retry alone is insufficient
- the system has a meaningful fallback or failure mode
- external provider reliability is variable

Avoid:
- circuit breakers without observability
- circuit breakers around every call without reason
- hiding critical failures behind silent fallbacks

## Bulkheads

Bulkheads isolate resources so one failing dependency or tenant cannot consume everything.

Useful for:
- high-fanout calls
- tenant isolation
- background workers
- thread/connection pools
- queue consumers
- external providers

Guardians should flag unbounded fanout and shared resource pools for critical operations.

## Backpressure

Systems must respond to overload intentionally.

Approaches:
- bounded queues
- rate limits
- concurrency limits
- load shedding
- graceful degradation
- clear rejection semantics

Avoid:
- unbounded queues
- unbounded goroutines/tasks/promises
- memory growth as implicit buffer
- silent dropped work

## Idempotency

Idempotency is required for safe retry of write operations.

Use:
- idempotency keys
- request deduplication
- natural business keys
- outbox pattern
- exactly-once semantics only when truly supported

Do not assume retries are safe because tests pass.

## Fallbacks

Fallbacks must be explicit and safe.

Good fallbacks:
- cached read-only data with staleness disclosure
- reduced functionality
- queued work for later processing
- graceful user-facing degradation

Bad fallbacks:
- silently accepting invalid state
- bypassing authorization
- defaulting to allow on security failure
- hiding data loss

## Resilience And AI-Generated Code

AI-generated code often:
- adds retries without idempotency
- omits timeouts
- uses `Promise.all` or equivalent without limits
- swallows provider errors
- hides fallback behavior
- ignores cancellation
- creates background tasks without supervision

These are guardian targets.

## Anti-Patterns

- retry without idempotency
- no timeout on external call
- no backoff or jitter
- infinite retry
- fail-open for security
- unbounded queue
- unbounded parallel fanout
- fallback that hides data loss
- circuit breaker without visibility
- background work without supervision
- external call inside long transaction

## Guardian Hooks

Recommended guardians:
- timeout-policy-guardian
- retry-safety-guardian
- idempotency-guardian
- circuit-breaker-guardian
- bulkhead-guardian
- backpressure-guardian
- fallback-safety-guardian

## Structured Finding Example

```json
{
  "guardian": "retry-safety",
  "rule": "retry-without-idempotency",
  "disposition": "blocker",
  "summary": "The payment capture operation retries a non-idempotent write without an idempotency key.",
  "evidence_refs": ["src/payments/capture_service.go"],
  "recommended_action": "Introduce an idempotency key or remove automatic retry for this operation."
}
```

## Lifecycle Usage

Planning:
- identify dependency failure modes and retry/idempotency needs

Architecture:
- define resilience pattern placement and blast-radius controls

Implementation:
- add timeouts, retry budgets, cancellation, and isolation

Testing:
- test timeout, retry, cancellation, and failure behavior

Review:
- challenge unsafe retries, missing timeouts, and hidden fallbacks

Migration:
- ensure fallback and rollback behavior is resilient
