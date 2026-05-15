# Observability

A production system must explain itself when it fails. Observability exists to answer: what happened, where, who was affected, how severe, what changed recently, and what should the operator inspect next.

## Core Principles

### Structured Logs

Logs should capture discrete events with structured fields: stable event names, operation names, correlation IDs, entity identifiers, error class or code, outcome, and duration where relevant.

Avoid: free-form-only logs, duplicate logs across every layer, raw exception dumps without context, logging sensitive payloads, logs that require reading source code to understand meaning.

### Metrics

Metrics should describe system behavior over time. Important categories: request count, error count, latency, saturation, queue depth, retry count, timeout count, external dependency failures, business-critical counters.

Avoid: high-cardinality labels, user IDs or raw request IDs as metric labels, metrics that cannot drive an operational decision, vanity counters that never appear in dashboards or alerts.

### Traces

Traces should connect distributed work across boundaries. Trace: inbound requests, outbound calls, database operations, message processing, background jobs, cross-service workflows, AI/model/tool invocations.

Trace spans should include: operation names, status, duration, important stable attributes, error context.

### Logging Levels

Use logging levels consistently across the codebase. The intent of each level must be stable and understood by on-call operators, not just developers.

| Level | When to use | Example |
|-------|-------------|----------|
| `ERROR` | An operation failed and requires attention; the system could not fulfill a request or complete a critical task | Payment processing failed, database connection lost, unrecoverable deserialization error |
| `WARN` | Something unexpected happened but the system recovered or degraded gracefully; an operator should investigate if it recurs | Retry succeeded after transient failure, deprecated API call received, cache miss falling back to database |
| `INFO` | Normal operational milestones worth recording for auditing, debugging, or capacity tracking | Request handled, job started/completed, configuration loaded, deployment started |
| `DEBUG` | Detailed diagnostic context useful during development or incident investigation; never enabled by default in production | SQL query parameters, serialized request/response bodies (sanitized), intermediate computation state |
| `TRACE` | Very fine-grained flow tracing; almost never enabled outside local development | Function entry/exit, loop iteration details, wire-level protocol bytes |

Rules:
- API request handlers should log at `INFO` on completion with status, duration, and correlation ID.
- Failed external calls should log at `WARN` on recoverable failure and `ERROR` on terminal failure.
- Never log at `ERROR` for expected business outcomes (e.g., validation rejection is `INFO` or `WARN`, not `ERROR`).
- Never use `DEBUG` or `TRACE` as a substitute for structured event fields at `INFO`.
- Background jobs and scheduled tasks should log lifecycle events (`started`, `completed`, `failed`) at `INFO`.

### Correlation IDs

Every request or workflow should preserve a correlation ID flowing through logs, traces, error reports, downstream requests, queue messages, and audit events.

If a system creates work asynchronously, it must preserve or intentionally derive correlation context.

### Error Observability

Errors should include: stable type or code, operation, causal chain, sanitized message, relevant entity identifier, and correlation ID.

Avoid: losing root cause, wrapping without context, swallowing errors, logging secrets, returning raw internals to users.

### Boundary Instrumentation

Instrument: API boundaries, message consumers, scheduled jobs, external provider calls, database calls, cache calls, model/tool calls, file or object-store operations, batch jobs.

Boundary instrumentation is usually more valuable than low-level noise.

## Patterns

### OpenTelemetry

OpenTelemetry is the preferred general standard where the stack supports it. Use it to standardize traces, metrics, context propagation, and semantic conventions. Do not add telemetry as random one-off logging.

### Log Or Return, Not Both

Logging and returning the same error at multiple layers creates duplicate noise. Low-level code returns typed/contextual errors; boundary code logs once with request/session context; observability layer attaches correlation metadata.

## Anti-Patterns

- Logging and returning the same error repeatedly
- No correlation ID on request/workflow boundaries
- Unstructured logs in production paths
- Missing metrics for critical dependency calls
- High-cardinality metric labels (user IDs, request bodies)
- Raw secrets or PII in logs
- Swallowed errors with no trace
- Tracing only the happy path
- Background jobs without lifecycle logs
- External calls without duration/error metrics
- AI-generated code that adds business behavior without logs

## Guardian Hooks

Guardians that apply to this guidance:
- `observability`: external-call-without-telemetry, missing-correlation-id, unstructured-log
- `clean_code`: no-hidden-side-effects (when side effects bypass telemetry)
- `retry_safety`: retry-without-backoff (when retries lack visibility)
