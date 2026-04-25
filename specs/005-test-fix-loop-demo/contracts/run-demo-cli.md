# Contract: `synod run-demo` CLI

**Feature**: 005-test-fix-loop-demo  
**Date**: 2026-04-25

## Synopsis

```text
synod run-demo [--workspace <PATH>]
```

## Inputs

| Argument / Flag | Required | Type | Default | Description |
|-----------------|----------|------|---------|-------------|
| `--workspace`   | no       | path | `<cwd>/.synod/demo-workspace` | Root directory of the isolated demo workspace. The directory is created (or reset) before plan execution. |

The command takes no other flags. There is no `--goal`, no `--profile`, and no
`--trace` flag in this slice.

## Behavior

1. Resolve the workspace root (default if not supplied) and canonicalize it.
   Reject any path that does not end in `.synod/demo-workspace` (or whose
   canonical form would resolve outside the current working tree).
2. Reset the demo workspace via `reset_demo_workspace(root)`. After this step
   the workspace contains exactly one buggy source file and one failing test
   definition file.
3. Build the demo runtime via `build_demo_runtime(DemoRunProfile::test_fix_loop(&workspace))`.
4. Build a `TaskRunRequest` from the profile (workspace_ref = canonical root,
   session_id = `run-demo-<unix_millis>`).
5. Invoke `Orchestrator::run(request)` with the existing
   `FileTraceStore::for_workspace(root)` as the trace sink.
6. Render the trace via `output::render_run_trace("run-demo", trace, response, next)`.
7. Append a final line to the rendered output:
   `final source file: <absolute path of target_file>`.
8. Return a `RunCommandReport` with:
   - `exit_status = Succeeded` if `response.terminal_status == TaskStatus::Succeeded`;
     otherwise `NonSuccess`.
   - `terminal_output = <rendered text>`.
   - `trace_location = Some(<trace path>)`.

## Exit status

| Outcome | `exit_status` | Console exit code |
|---------|---------------|-------------------|
| Plan reaches `Succeeded` (US1) | `Succeeded` | 0 |
| Plan reaches `Failed` due to retry exhaustion | `NonSuccess` | non-zero |
| Plan reaches `Failed` due to replan exhaustion | `NonSuccess` | non-zero |
| Workspace cannot be reset (I/O error) | `InvalidInvocation` | non-zero |
| Trace store cannot write | `InvalidInvocation` | non-zero |

(`InvalidInvocation` and `NonSuccess` map to non-zero process exit codes via
the existing `bin/synod.rs` exit-code routing.)

## Console output (informative)

The rendered output MUST include, in order, lines that allow a developer to
verify all three user stories from the spec:

```text
[run-demo] step 1 of 3 — analyzer "analyze" — outcome=success attempt=1
[run-demo] step 2 of 3 — coder "code" — outcome=recoverable attempt=1
[run-demo] step 2 of 3 — coder "code" — outcome=success attempt=2
[run-demo] step 3 of 3 — tester "verify" — outcome=replan_required attempt=1
[run-demo] step 4 of 5 — analyzer "analyze#replan-1" — outcome=success attempt=1
[run-demo] step 5 of 5 — coder "code#replan-1" — outcome=success attempt=1
[run-demo] step 5 of 5 — tester "verify" — outcome=success attempt=2
[run-demo] terminal status: Succeeded
[run-demo] trace: <absolute trace path>
[run-demo] final source file: <absolute target_file path>
```

The exact line wording is determined by `output::render_run_trace`. The
**presence** of (a) a recoverable coder failure followed by a coder success,
(b) a `replan_required` event on the tester, and (c) a final `Succeeded`
status are part of this contract; the formatting strings are not.

## Trace contract

The trace file at the printed path MUST contain a JSON document that
includes, at minimum:

- the ordered list of step attempts (analyzer, coder×2, tester (replan),
  analyzer (inserted), coder (inserted), tester (success));
- the `Recoverability` value for each non-success attempt;
- the recovery action taken after each non-success attempt
  (`Retry` or `Replan` with the inserted step IDs);
- the final `terminal_status` (`Succeeded` for a healthy run).

This contract is satisfied automatically by reusing the existing
`FileTraceStore` and `Orchestrator` instrumentation; this slice does not
mutate the trace schema.

## Idempotence

Calling `synod run-demo` two consecutive times on the same machine MUST
produce the same terminal status and the same final file contents. This is
guaranteed by `reset_demo_workspace` running on every invocation.

## Failure modes covered

- **Retry exhaustion** — Reuses the existing `Orchestrator` exhaustion logic.
  The trace records the cause as `retry_limit_exhausted`.
- **Replan exhaustion** — Reuses the existing `Orchestrator` exhaustion logic.
  The trace records the cause as `replan_limit_exhausted`.
- **Workspace cannot be reset** — The handler returns a `RunCommandError`
  variant before invoking the orchestrator. No trace is written.
- **Trace cannot be written** — The handler returns a `RunCommandError`
  variant before completing. No half-trace is produced.
- **Tester I/O failure** — The tester adapter returns a `Terminal` failure;
  the orchestrator stops with `Failed`; the trace records `tester_io`.
