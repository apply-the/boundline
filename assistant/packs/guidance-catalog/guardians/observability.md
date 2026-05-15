# Observability Guardian

Look for missing telemetry on external calls, unstructured logs, and lost correlation identifiers.

## Rules

### external-call-without-telemetry
Every call to an external system (HTTP, database, queue, file system, external process) should emit timing, success/failure, and context information for production debugging.

Triggers: HTTP clients without request/response logging, database queries without timing spans, queue operations without trace propagation.

### missing-correlation-id
Requests flowing through the system must carry a correlation identifier from entry point through all downstream calls. Without this, distributed debugging is impossible.

Triggers: request handlers that do not propagate or generate correlation IDs, background job processors without trace context, event handlers without source tracing.

### unstructured-log
Logs must be structured (key-value or JSON) for machine parsing. Unstructured string interpolation makes log aggregation and alerting unreliable.

Triggers: `println!` or `console.log` with string formatting, log messages without severity levels, log messages without structured context fields.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all languages. Most relevant during implementation and review of code that interacts with external systems.
