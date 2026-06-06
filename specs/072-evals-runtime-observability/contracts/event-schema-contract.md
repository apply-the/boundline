# Event Schema Contract: Evals And Runtime Observability

**Feature**: 072-evals-runtime-observability
**Date**: 2026-06-05
**Contract type**: Runtime output contract (JSONL export)

## Purpose

This contract defines the shape and compatibility rules for structured runtime events emitted by Boundline and consumed by external dashboards, CI pipelines, and analysis tools via JSONL export.

## JSONL Stream Format

Each line in the export is a self-contained JSON object terminated by `\n`. The stream has no header, footer, or envelope — it is a sequence of independent event objects.

```jsonl
{"event_id":"...","event_type":"planning.analysis.completed","schema_version":"1.0","timestamp":"...","payload":{...}}
{"event_id":"...","event_type":"guardian.finding.emitted","schema_version":"1.0","timestamp":"...","payload":{...}}
```

## Common Event Envelope

Every event MUST include these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_id` | string (UUID) | yes | Unique event identifier |
| `event_type` | string | yes | Discriminates the event kind (dot-separated, e.g., `"trace.compacted"`) |
| `schema_version` | string | yes | Per-event-type version in `MAJOR.MINOR` format (e.g., `"1.0"`) |
| `timestamp` | string (ISO 8601) | yes | When the event was emitted |
| `session_id` | string (UUID) | yes | Owning session identifier |
| `trace_ref` | string or null | no | Link to the source trace item, if applicable |
| `payload` | object | yes | Type-specific data (see per-type schemas below) |

## Schema Versioning Rules

1. **Additive fields** (new optional fields added to `payload`) are permitted within the same major `schema_version`.
2. **Field removal**, **meaning change**, or **incompatible type change** in `payload` requires a new major `schema_version` for that event type.
3. **Envelope field changes** (adding/removing fields from the common envelope) are NOT permitted within a major version — the envelope is part of the per-event-type contract.
4. Consumers MUST tolerate unknown fields in `payload` (forward-compatible parsing).
5. Consumers MUST reject events with an unsupported major `schema_version` for a given `event_type`.

## Event Types (Initial Set)

### `planning.analysis.completed` — v1.0

Emitted when the planning analysis gate completes.

```json
{
  "event_type": "planning.analysis.completed",
  "schema_version": "1.0",
  "payload": {
    "state": "blocked" | "clean" | "warning",
    "finding_count": 3,
    "blocked_finding_count": 1,
    "coverage_summary": {
      "outcomes_covered": 5,
      "outcomes_total": 5,
      "backlog_slices_covered": 12,
      "backlog_slices_total": 12
    }
  }
}
```

### `guardian.finding.emitted` — v1.0

Emitted when a guardian produces a finding.

```json
{
  "event_type": "guardian.finding.emitted",
  "schema_version": "1.0",
  "payload": {
    "guardian_id": "rust-zero-panic",
    "finding_id": "uuid",
    "severity": "blocker" | "warning" | "info",
    "source_file": "src/domain/foo.rs",
    "line": 42,
    "message": "unwrap detected outside main.rs"
  }
}
```

### `provider.call.completed` — v1.0

Emitted when a provider call finishes.

```json
{
  "event_type": "provider.call.completed",
  "schema_version": "1.0",
  "payload": {
    "provider_id": "speckit-adapter",
    "call_id": "uuid",
    "capability": "execute",
    "status": "success" | "failure" | "timeout",
    "latency_ms": 1234,
    "finding_count": 2
  }
}
```

### `trace.compacted` — v1.0

Emitted after a trace compaction run.

```json
{
  "event_type": "trace.compacted",
  "schema_version": "1.0",
  "payload": {
    "policy": "trace-compaction-v1",
    "source_trace": "trace://abc",
    "actions": [
      {
        "item_ref": "assistant-transcript-1",
        "from": "raw",
        "to": "summary",
        "lossy": true,
        "tiebreak": false
      }
    ],
    "preserved_refs": ["decision-12", "finding-22"],
    "metrics": {
      "compaction_count": 3,
      "class_distribution": {"lossless": 15, "structured": 42, "summary": 8, "index_only": 3, "discardable": 2},
      "trace_size_before_bytes": 1048576,
      "trace_size_after_bytes": 524288,
      "lossy_count": 8,
      "preserved_decision_count": 5,
      "preserved_rejection_count": 2
    }
  }
}
```

### `phase.requested` — v1.0

Emitted when the runtime transitions to a new phase.

```json
{
  "event_type": "phase.requested",
  "schema_version": "1.0",
  "payload": {
    "phase": "plan" | "run" | "review" | "verify",
    "trigger": "operator" | "auto" | "retry",
    "previous_phase": "plan"
  }
}
```

### `route.decision.made` — v1.0

Emitted when the orchestrator selects a route.

```json
{
  "event_type": "route.decision.made",
  "schema_version": "1.0",
  "payload": {
    "route_id": "uuid",
    "model_family": "claude-4",
    "reason": "expertise_match",
    "fallback_available": true
  }
}
```

### `context.selection.recorded` — v1.0

Emitted when context assembly completes.

```json
{
  "event_type": "context.selection.recorded",
  "schema_version": "1.0",
  "payload": {
    "total_items": 45,
    "total_bytes": 65536,
    "omitted_items": 3,
    "omission_reasons": {"oversized": 1, "stale": 2},
    "fidelity_tier": "full" | "excerpt" | "digest"
  }
}
```

## Sensitive Data Prevention

- The `payload` of every event type defines a field allowlist.
- Fields not in the allowlist MUST NOT appear in the JSONL export.
- Known sensitive field names (`token`, `secret`, `password`, `key`, `credential`, `authorization`) are never included in any allowlist.
- Consumers SHOULD NOT rely on the absence of sensitive fields alone; they SHOULD treat all exported data as potentially containing operational metadata.

## Eval Output Contract

The eval suite produces a JSON summary to stdout (for CI) or a human-readable table (for local use). The JSON shape is:

```json
{
  "suite_status": "pass" | "fail",
  "total_count": 7,
  "pass_count": 6,
  "fail_count": 1,
  "duration_ms": 2345,
  "results": [
    {
      "eval_id": "planning-quality-01",
      "eval_name": "Planning Quality - Blocked Execution",
      "status": "pass",
      "failure_reason": null,
      "source_refs": ["fixtures/blocked-session.json"],
      "expected_outcome": "execution handoff withheld",
      "actual_outcome": "execution handoff withheld",
      "duration_ms": 312
    }
  ]
}
```

CI integration: exit code 0 when `suite_status` is `"pass"`, exit code 1 when `"fail"`.
