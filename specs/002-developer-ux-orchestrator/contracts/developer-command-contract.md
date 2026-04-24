# Contract: Developer Command Surface

## Purpose

Defines the developer-facing CLI surface used to run, inspect, and diagnose the orchestrator core.

## Commands

| Command | Purpose | Required Inputs |
|---------|---------|-----------------|
| `synod doctor` | Verify local readiness before attempting a run | `workspace_ref` |
| `synod demo` | Run the deterministic guided demo profile | `workspace_ref` |
| `synod run` | Execute a simple bounded developer-supplied objective | `goal`, `workspace_ref` |
| `synod inspect` | Render a readable summary from a persisted trace | `trace_ref` or a supported workspace-local default |

## Invocation Shape

| Field | Required | Description |
|-------|----------|-------------|
| `command_name` | Yes | One of `doctor`, `demo`, `run`, or `inspect` |
| `workspace_ref` | Yes | Local workspace used for readiness checks, run state, and trace persistence |
| `goal` | No | Required only for `run` |
| `trace_ref` | No | Required for `inspect` unless a supported local default is used |
| `limits_override` | No | Optional bounded execution overrides for `run` |

## Output Guarantees

- Every command emits human-readable terminal output.
- `demo` and `run` surface the active step, step category, recovery events when present, and the final terminal outcome.
- `inspect` renders step order, recovery events, and terminal reason from persisted trace data.
- `doctor` reports a ready or not-ready result plus actionable messages for each failing check.

## Exit Semantics

| Exit Code | Meaning |
|-----------|---------|
| `0` | Command completed successfully or environment is ready |
| `1` | Run or inspection completed in a non-success terminal state |
| `2` | Invocation or local readiness validation failed before execution started |
| `3` | Trace inspection could not read or interpret the requested trace |

## Behavioral Guarantees

- Commands are synchronous and return only after the requested action reaches an explicit end state.
- `demo` always exercises a deterministic bounded run profile.
- `run` always persists a trace when execution starts successfully.
- `inspect` does not mutate the source trace.
- `doctor` does not start task execution.