# Contract: Trace Summary View

## Purpose

Defines the readable inspection output emitted by `boundline inspect`.

## Summary Shape

| Field | Required | Description |
|-------|----------|-------------|
| `trace_ref` | Yes | Source trace location |
| `goal` | Yes | Goal recorded in the trace |
| `executed_steps` | Yes | Ordered summaries of each executed step |
| `recovery_events` | Yes | Ordered retry and replanning summaries derived from the trace |
| `terminal_status` | Yes | Final task status |
| `terminal_reason` | Yes | Human-readable explanation of why the task stopped |
| `duration` | No | Elapsed run duration when both start and end timestamps are present |

## Step Summary Shape

| Field | Required | Description |
|-------|----------|-------------|
| `step_id` | Yes | Executed step identifier |
| `step_kind` | Yes | `agent`, `tool`, or `decision` |
| `attempts` | Yes | Total attempts observed for the step |
| `final_status` | Yes | Final per-step result |
| `headline` | Yes | Short human-readable summary of what happened |

## Recovery Event Shape

| Field | Required | Description |
|-------|----------|-------------|
| `event_type` | Yes | `retry_scheduled` or `replanned` |
| `trigger` | Yes | Human-readable reason for the recovery action |
| `related_step_id` | No | Step associated with the recovery event |

## Behavioral Guarantees

- The summary is fully derived from the persisted trace and does not require live process state.
- The summary preserves execution order.
- Failed and exhausted runs surface the terminal reason prominently.
- Recovery summaries are explicit enough that a developer can reconstruct why execution continued or stopped.