# Data Model: Test-Fix Loop Vertical Slice Demo

**Feature**: 005-test-fix-loop-demo  
**Date**: 2026-04-25

This slice introduces no new domain types. It composes existing types and adds
a small file-system helper struct. All listed types live in the `synod` crate.

## Reused existing types (no changes)

- `domain::task::TaskRunRequest` — unchanged. Used as the input to
  `Orchestrator::run`. The `workspace_ref` field carries the absolute path of
  the demo workspace.
- `domain::task::TaskStatus` — unchanged. Terminal values consumed by
  `synod run-demo`: `Succeeded` or `Failed`.
- `domain::limits::RunLimits` — unchanged. Demo uses the existing default
  (`max_steps = 6`, `max_retries = 1`, `max_replans = 1`).
- `domain::plan::Plan` and `domain::step::{Step, StepKind, StepExecutionResult,
  Recoverability}` — unchanged.
- `orchestrator::engine::Orchestrator` — unchanged.
- `orchestrator::planner::StaticPlanner` — unchanged.
- `registry::agent_registry::AgentRegistry` and
  `registry::tool_registry::ToolRegistry` — unchanged.
- `adapters::trace_store::FileTraceStore` — unchanged.
- `demo::profile::DemoRunProfile` — extended only by adding the
  `test_fix_loop(...)` constructor (see below).
- `demo::endpoints::build_demo_runtime` — unchanged signature; the embedded
  coder/tester closures are extended in-place.
- `cli::DeveloperCommand` — extended only by adding the `RunDemo` variant
  (see below).

## Added type: `DemoWorkspace` (in `src/demo/workspace.rs`)

Owns the seeded demo workspace on disk. It is a small value type.

| Field | Type | Description |
|-------|------|-------------|
| `root` | `PathBuf` | Absolute path of the demo workspace directory (e.g. `<cwd>/.synod/demo-workspace`). |
| `target_file` | `PathBuf` | Absolute path of the seeded buggy source file inside `root`. |
| `test_file` | `PathBuf` | Absolute path of the seeded failing test definition file inside `root`. |
| `bug_marker` | `&'static str` | Sentinel substring whose presence in `target_file` means the bug is still there. |
| `fixed_content` | `&'static str` | Static contents that the coder writes into `target_file` to apply the fix. |

### Lifecycle

- `seed_demo_workspace(root)` — Creates `root` if missing, writes the seeded
  buggy `target_file` and the seeded failing `test_file`. Returns the
  `DemoWorkspace`. Errors if any I/O step fails.
- `reset_demo_workspace(root)` — Removes `root` if it exists, then calls
  `seed_demo_workspace(root)`. Used by the CLI on every `synod run-demo` run.
- After a successful run, `target_file` contains `fixed_content`. The
  `DemoWorkspace` value is consumed by the CLI handler and is not persisted.

### Validation rules

- `root` MUST be an absolute path. Relative inputs MUST be canonicalized by
  the caller (the CLI handler does this).
- `root` MUST NOT be the repository root or the user's home directory; the
  helper rejects any `root` that resolves to those paths to prevent accidental
  destruction. (Only paths whose final segment is `demo-workspace` and whose
  parent is `.synod` are accepted.)

### State transitions

```text
(no workspace) --seed--> SEEDED(buggy)
SEEDED(buggy) --reset--> SEEDED(buggy)        # idempotent re-seed
SEEDED(buggy) --coder writes fixed_content--> SEEDED(fixed)
SEEDED(fixed) --reset--> SEEDED(buggy)
```

The orchestrator never observes or mutates `DemoWorkspace` directly; it only
sees `target_file`, `bug_marker`, and `fixed_content` through the step input
JSON injected by the new `test_fix_loop` profile.

## Added enum variant: `DeveloperCommand::RunDemo`

Single new clap subcommand:

```text
RunDemo {
    /// Optional: override the default <cwd>/.synod/demo-workspace path.
    workspace: Option<PathBuf>,
}
```

The variant carries no goal, no flags, and no other arguments. Reaching the
existing `output::render_run_trace` requires only the workspace path and the
orchestrator response, both of which are produced inside the handler.

## Added enum variant: `CommandName::RunDemo`

Mirrors `DeveloperCommand::RunDemo` and renders to the string literal
`"run-demo"` in `CommandName::as_str()`. Used by
`DeveloperCommandSession::from_command` to record the command in trace
metadata.

## Added profile constructor: `DemoRunProfile::test_fix_loop(workspace: &DemoWorkspace) -> Self`

Builds a profile equivalent to `guided_demo` with the following deltas:

- `name = "test_fix_loop"`.
- `goal = "Fix the seeded failing test in the demo workspace"`.
- `step_outline[code].input` includes:
  - `force_retry: true`
  - `target_file: <workspace.target_file>`
  - `fixed_content: <workspace.fixed_content>`
- `step_outline[verify].input` includes:
  - `force_replan: true`
  - `target_file: <workspace.target_file>`
  - `bug_marker: <workspace.bug_marker>`
- `recovery_trigger_step = "code"` (unchanged from `guided_demo`).
- `limits = RunLimits::default()` with `max_retries = 1`, `max_replans = 1`,
  `max_steps = 6` — unchanged.

## Extended adapter behaviors

**Coder (agent in `build_demo_runtime`)**:

- Existing `force_retry` / `force_replan` behavior is preserved.
- New: when the step input contains `target_file` (string) and
  `fixed_content` (string), the **successful** execution path writes
  `fixed_content` to `target_file` (truncate + write) before returning success.
- New: success output gains `"updated_file": <path>` in addition to the
  existing fields, so the trace records the path that was written.

**Tester (tool in `build_demo_runtime`)**:

- Existing `force_terminal_failure` behavior is preserved.
- New: a `force_replan` flag in the step input causes the **first** attempt
  to return `Recoverability::ReplanRequired` and increments a per-step retry
  counter (same `Arc<Mutex<HashMap<String, usize>>>` already in scope).
- New: when the step input contains `target_file` (string) and `bug_marker`
  (string), the adapter reads the file and:
  - returns the configured failure (replan or terminal, depending on flags)
    when the marker is still present and it's the first attempt;
  - returns success (with `"verified_file": <path>` in the output) when the
    marker is absent.
- If reading `target_file` fails (I/O error), the adapter returns a
  `Recoverability::Terminal` failure with cause `tester_io`. This satisfies
  the spec's "test runner unavailable" edge case.

No other domain or orchestrator types are modified.
