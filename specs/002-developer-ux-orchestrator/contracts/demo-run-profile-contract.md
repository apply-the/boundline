# Contract: Demo Run Profile

## Purpose

Defines the deterministic guided demo profile used by `synod demo`.

## Profile Shape

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Stable demo profile name |
| `goal` | Yes | Human-readable bounded objective shown to developers |
| `initial_input` | Yes | Structured request data used to start the orchestrator |
| `step_outline` | Yes | Ordered steps that model the intended analysis, change, and verification flow |
| `recovery_trigger_step` | Yes | Step identifier where the demo intentionally exposes a recoverable failure path |
| `limits` | Yes | Explicit bounded step, retry, and replanning limits |

## Behavioral Guarantees

- The demo profile is fully local and deterministic.
- The demo profile includes at least one visible recovery event before the run reaches its terminal outcome.
- The demo profile remains bounded by the same run-limit rules as the core orchestrator.
- The demo profile produces a persisted trace suitable for later inspection with `synod inspect`.

## Failure Contract

- If the demo profile cannot be loaded or validated, `synod demo` must fail before task execution starts.
- If the demo run reaches a non-success terminal state, the command still reports the final reason and leaves behind an inspectable trace.