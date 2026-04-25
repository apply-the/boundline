# Research: Test-Fix Loop Vertical Slice Demo

**Feature**: 005-test-fix-loop-demo  
**Date**: 2026-04-25

This slice has no `NEEDS CLARIFICATION` markers in the spec. Research therefore
focuses on confirming that every behavior the slice needs already exists in the
codebase, and on the smallest set of decisions required to wire them together.

---

## Decision 1 — Reuse the existing `Orchestrator` + `StaticPlanner` for execution

- **Decision**: The new `synod run-demo` command MUST call `Orchestrator::run`
  with `StaticPlanner` and the existing `AgentRegistry` / `ToolRegistry`
  populated by `build_demo_runtime(...)`.
- **Rationale**: All the required behaviors (sequential step execution,
  retry on `Recoverability::Retryable`, replan on `Recoverability::ReplanRequired`,
  bounded `RunLimits`, terminal-state reporting, trace persistence) are already
  implemented and covered by tests in `src/orchestrator/engine.rs`.
- **Alternatives considered**: Writing a new lightweight loop. **Rejected** —
  it would duplicate the existing engine, violate the "use what already exists"
  rule, and break trace and recovery semantics.

## Decision 2 — Add one new `DemoRunProfile` constructor (`test_fix_loop`)

- **Decision**: Introduce `DemoRunProfile::test_fix_loop()` that produces the
  same three-step outline as `guided_demo` (`analyze → code → verify`) but with
  two flags toggled: `force_retry = true` on the `code` step and
  `force_replan = true` on the `verify` step. The default `RunLimits`
  (`max_retries = 1`, `max_replans = 1`, `max_steps = 6`) are used unchanged.
- **Rationale**: The existing profile constructors (`guided_demo`,
  `default_run`) already expose the toggle pattern. Adding one more constructor
  is a one-function change and reuses every other piece of the demo runtime.
- **Alternatives considered**: Reusing `guided_demo` and adding a CLI flag to
  enable replan. **Rejected** — the spec says `synod run-demo` MUST NOT require
  flags; an explicit profile is clearer and keeps `guided_demo` semantics
  unchanged for the existing `synod demo` subcommand.

## Decision 3 — Extend the existing `tester` adapter to support `force_replan`

- **Decision**: Extend the tester adapter in `src/demo/endpoints.rs` so that a
  `force_replan` flag in the step input causes the **first** attempt to return
  `StepExecutionResult::failure(..., Recoverability::ReplanRequired)`, while
  subsequent attempts of the same step succeed. Track attempts using the same
  `Arc<Mutex<HashMap<String, usize>>>` retry-counter pattern already used by
  the coder adapter.
- **Rationale**: The tester adapter currently supports only
  `force_terminal_failure`. We need replan-trigger behavior on the verify step
  to satisfy User Story 3 / FR-005 deterministically.
- **Alternatives considered**: Triggering replan from the coder adapter.
  **Rejected** — the spec is explicit that the *tester* must trigger the
  replan (because that is the realistic shape of "fix didn't work, replan").

## Decision 4 — Make the coder adapter actually modify a real file

- **Decision**: Extend the existing coder adapter so that, when its step input
  contains a `target_file` string, the adapter writes the content from a
  `fixed_content` field into that file once it succeeds. The coder still
  returns `Recoverability::Retryable` on the first attempt when `force_retry`
  is set; only the second (successful) attempt mutates the file. The seeded
  `target_file` and `fixed_content` are supplied by the new `test_fix_loop`
  profile so that the coder adapter remains deterministic.
- **Rationale**: The user input requires a *real* code modification. The
  smallest possible mutation is "overwrite the buggy file with the fixed
  contents". No diffing, patching, or AI logic is needed for this slice.
- **Alternatives considered**: Spawning an external editor or calling a real
  LLM. **Rejected** — explicitly forbidden by the input.

## Decision 5 — Make the tester adapter actually read the file and report pass/fail

- **Decision**: Extend the tester adapter so that, when its step input contains
  a `target_file` string and a `bug_marker` string, the adapter reads the file
  and reports failure (terminal or replan, depending on flags and attempt) when
  the marker is still present, success otherwise. This is the deterministic
  in-process "test runner" used by the demo.
- **Rationale**: This satisfies FR-006 (the "fix" must result in a passing
  test) without spawning `cargo test` or any external runner. Determinism is
  preserved because the file's pre/post state is fully controlled by the
  profile and the coder.
- **Alternatives considered**: Spawning `cargo test` against a sub-crate.
  **Rejected** — increases runtime, adds an external dependency on the host
  toolchain, and is forbidden by the input ("can be fake or real" — fake is
  simpler and the input prefers deterministic behavior).

## Decision 6 — Seed the demo workspace via a small helper module

- **Decision**: Add `src/demo/workspace.rs` exposing two functions:
  `seed_demo_workspace(root: &Path) -> Result<DemoWorkspace, DemoWorkspaceError>`
  (creates the directory, writes the seeded buggy source file, writes the
  failing test definition file, returns the resolved paths) and
  `reset_demo_workspace(root: &Path) -> Result<DemoWorkspace, DemoWorkspaceError>`
  (removes and re-seeds). The CLI calls `reset_demo_workspace` on every
  `synod run-demo` invocation so that re-runs are idempotent.
- **Rationale**: Keeps file-system concerns isolated and unit-testable.
  Reset-on-every-run prevents the "already-fixed workspace falsely reports
  success" edge case in the spec.
- **Alternatives considered**: Hardcoding inline file writes inside
  `execute_run_demo`. **Rejected** — harder to test, and mixes orchestration
  wiring with file-system effects.

## Decision 7 — Default workspace path under `<repo>/.synod/demo-workspace/`

- **Decision**: When `synod run-demo` is invoked without explicit arguments,
  the demo workspace MUST live at `<cwd>/.synod/demo-workspace/`. Traces will
  go under `<cwd>/.synod/demo-workspace/.synod/traces/` via the existing
  `FileTraceStore::for_workspace`.
- **Rationale**: Matches the convention already established by the existing
  session model and trace store. Keeps the demo entirely scoped under
  `.synod/`, which is the recognized "synod-owned" prefix.
- **Alternatives considered**: A temporary directory under `/tmp`. **Rejected**
  — developers can't easily inspect the post-run file state, which is required
  by US1's acceptance scenario.

## Decision 8 — Console output reuses `output::render_run_trace`

- **Decision**: The `synod run-demo` CLI handler MUST reuse
  `output::render_run_trace` (the same renderer used by `synod demo` and
  `synod run`) and MUST append a single line at the end with the resolved
  path of the (now-fixed) target file.
- **Rationale**: Consistent UX across `demo`, `run`, and `run-demo`. The extra
  "final source file" line is needed by US1's acceptance scenario.
- **Alternatives considered**: A bespoke renderer. **Rejected** — duplicates
  existing logic and violates "use what already exists".

---

## Summary of new code surface

| Area | Change | Size |
|------|--------|------|
| `src/cli.rs` | Add `RunDemo` to `DeveloperCommand` + `CommandName::RunDemo` | small |
| `src/cli/run.rs` | Add `execute_run_demo(workspace_root)` | small |
| `src/bin/synod.rs` | Route the new variant to `execute_run_demo` | tiny |
| `src/demo/profile.rs` | Add `DemoRunProfile::test_fix_loop(workspace_root)` | small |
| `src/demo/endpoints.rs` | In-place extend `coder` (target_file/fixed_content) and `tester` (target_file/bug_marker + force_replan) | small |
| `src/demo/workspace.rs` | NEW helper: `seed_demo_workspace`, `reset_demo_workspace` | small |
| `src/demo.rs` | Add `pub mod workspace;` | trivial |
| `tests/integration/run_demo_flow.rs` | NEW end-to-end integration test | small |
| `tests/contract/run_demo_contract.rs` | NEW CLI contract test | small |
| `tests/unit/demo_workspace.rs` | NEW unit tests for seed/reset | small |

No new crates. No new top-level directories. No new abstractions.
