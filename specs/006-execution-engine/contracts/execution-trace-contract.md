# Contract: Execution Trace and Evidence

## Purpose

Define the minimum trace and inspection surface for the execution engine.

## Required trace events

Each delivery run MUST persist an execution trace that includes:

- `task_started`
- `step_started` for each analysis, change, and validation step that executes
- `step_completed` for each completed step
- `retry_scheduled` or `replanned` when recovery occurs
- `terminal_recorded`

Flow-aware sessions MAY also include the existing stage-specific events.

## Step payload requirements

### Change step

The completed payload for a successful change step MUST make the following information inspectable:

- attempt identifier
- changed file paths
- bounded change or diff evidence for each file

The completed payload for a failed change step MUST expose:

- the file path that failed
- the reason for failure
- whether the failure is terminal, retryable, or replan-required

### Validation step

The completed payload for a validation step MUST expose:

- rendered command
- exit code
- success or failure outcome
- captured stdout and stderr, bounded as needed for CLI rendering

## Inspect output requirements

`boundline inspect` MUST make the following information visible after a delivery run:

- the inspection target and trace path
- the executed steps and their final status
- any retry or replan events
- the final terminal status and reason
- enough change evidence to identify which files were modified
- the latest validation outcome

## Session status projection

When a session-backed delivery task has execution evidence, `boundline status` SHOULD surface:

- the latest changed files
- the latest validation outcome
- the latest trace reference

When no execution evidence exists yet, those fields MUST be omitted cleanly.