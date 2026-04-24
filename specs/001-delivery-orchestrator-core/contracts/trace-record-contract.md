# Contract: Trace Record

## Purpose

Defines the persisted trace shape required for post-run inspection.

## Trace Metadata

| Field | Required | Description |
|-------|----------|-------------|
| `task_id` | Yes | Stable task identifier |
| `session_id` | Yes | Session identifier for the run |
| `goal` | Yes | Human-readable task goal |
| `started_at` | Yes | Task start timestamp |
| `ended_at` | No | Task end timestamp |
| `terminal_status` | No | Final task status when complete |
| `terminal_reason` | No | Structured reason for termination |

## Event Shape

Each trace event must include these fields:

| Field | Required | Description |
|-------|----------|-------------|
| `event_id` | Yes | Stable identifier for the event |
| `event_type` | Yes | Step lifecycle, retry, replanning, or terminal event |
| `step_id` | No | Present for step-related events |
| `plan_revision` | Yes | Active plan revision at the time of the event |
| `payload` | Yes | Structured event-specific data |
| `recorded_at` | Yes | Timestamp of the event |

## Persistence Guarantees

- Events are written in execution order.
- Retry and replanning events must reference the triggering evidence or failure.
- The final terminal event must be present before a trace is considered complete.
- A persisted trace must remain readable even if the task ends in failure or exhaustion.

## Inspection Guarantees

- An operator can reconstruct step order, attempts, recovery decisions, and final outcome from the persisted record alone.
- Trace consumers do not need access to live process memory to interpret the terminal result.