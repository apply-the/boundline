# Observability Guidance

## Purpose

This guidance defines observability expectations for AI-assisted delivery.

It applies to services, CLIs, workers, APIs, scheduled jobs, event consumers, infrastructure automation, and critical internal tools.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, operations policy, or Canon-governed standard.

## Core Thesis

A system is not production-ready merely because it works locally or passes tests.

A production system must explain itself when it fails.

Observability exists to answer:

- what happened?
- where did it happen?
- who or what was affected?
- how severe is it?
- what changed recently?
- what should the operator inspect next?

## Observability Pillars

### Logs

Logs should capture discrete events with structured fields.

Prefer:
- stable event names
- operation names
- correlation IDs
- entity identifiers
- error class or code
- outcome
- duration where relevant

Avoid:
- free-form-only logs
- duplicate logs across every layer
- raw exception dumps without context
- logging sensitive payloads
- logs that require reading source code to understand meaning

### Metrics

Metrics should describe system behavior over time.

Important categories:
- request count
- error count
- latency
- saturation
- queue depth
- retry count
- timeout count
- external dependency failures
- business-critical counters

Metrics should be stable and aggregatable.

Avoid:
- high-cardinality labels
- user IDs or raw request IDs as metric labels
- metrics that cannot drive an operational decision
- vanity counters that never appear in dashboards or alerts

### Traces

Traces should connect distributed work across boundaries.

Trace important:
- inbound requests
- outbound calls
- database operations
- message processing
- background jobs
- cross-service workflows
- AI/model/tool invocations where relevant

Trace spans should include:
- operation names
- status
- duration
- important stable attributes
- error context

### Correlation IDs

Every request or workflow should preserve a correlation ID.

Correlation ID should flow through:
- logs
- traces
- error reports
- downstream requests
- queue messages
- audit events

If a system creates work asynchronously, it must preserve or intentionally derive correlation context.

### Error Observability

Errors should include:
- stable type or code
- operation
- causal chain
- sanitized message
- relevant entity identifier
- correlation ID

Avoid:
- losing root cause
- wrapping without context
- swallowing errors
- logging secrets
- returning raw internals to users

## OpenTelemetry

OpenTelemetry is the preferred general standard where the stack supports it.

Use OpenTelemetry to standardize:
- traces
- metrics
- context propagation
- semantic conventions

Do not add telemetry as random one-off logging.

## Observability At Boundaries

Instrument:
- API boundaries
- message consumers
- scheduled jobs
- external provider calls
- database calls
- cache calls
- model/tool calls
- file or object-store operations
- batch jobs

Boundary instrumentation is usually more valuable than low-level noise.

## AI-Assisted Delivery Risks

AI-generated code often:
- adds business behavior without logs
- catches and hides errors
- creates retries without metrics
- adds external calls without timeout visibility
- loses correlation IDs
- logs verbose sensitive objects
- passes tests but remains unobservable

Guardians should challenge these failure modes.

## Anti-Patterns

- logging and returning the same error repeatedly
- no correlation ID
- unstructured logs in production paths
- missing metrics for critical dependency calls
- high-cardinality metric labels
- raw secrets in logs
- swallowed errors
- tracing only the happy path
- background jobs without lifecycle logs
- external calls without duration/error metrics

## Guardian Hooks

Recommended guardians:
- observability-guardian
- correlation-id-guardian
- logging-ownership-guardian
- metrics-coverage-guardian
- trace-boundary-guardian
- sensitive-log-guardian

## Structured Finding Example

```json
{
  "guardian": "observability",
  "rule": "external-call-without-telemetry",
  "disposition": "concern",
  "summary": "The payment provider call has no duration metric, error classification, or trace span.",
  "evidence_refs": ["src/payments/provider_client.ts"],
  "recommended_action": "Wrap the provider call with trace span, timeout/error metrics, and sanitized structured logging."
}
```

## Lifecycle Usage

Planning:
- identify observability requirements for new behavior

Architecture:
- define observability boundaries and operational signals

Implementation:
- add logs, metrics, traces, and correlation propagation

Testing:
- verify instrumentation where operationally important

Review:
- challenge missing telemetry and sensitive logs

Incident:
- use traces/logs/metrics to reconstruct failure path
