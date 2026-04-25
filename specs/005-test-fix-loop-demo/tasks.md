# Tasks: Test-Fix Loop Vertical Slice Demo

**Input**: Design documents from `/specs/005-test-fix-loop-demo/`  
**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/run-demo-cli.md

**Tests**: Required. The slice defines executable orchestration behavior with retry,
replan, and trace guarantees, so contract, integration, and unit tests are mandatory.

**Organization**: Tasks are grouped by user story (US1, US2, US3). US1 is the MVP; US2
and US3 layer the retry and replan acceptance evidence on top of US1.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no inter-task dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- All file paths are repository-relative.

## Path Conventions

Default Synod single-crate layout (`src/`, `tests/`). The only new source file is
`src/demo/workspace.rs`. Everything else extends existing files.

---

## Phase 1: Setup

- [X] T001 Confirm `Cargo.toml` requires no new dependencies for this slice; no edit
  needed if `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid` are already
  present (they are).
- [X] T002 Verify `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and `cargo test --all-targets` all pass on the current branch as the baseline before
  any change is made.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The single piece of new infrastructure all three user stories depend on
is the on-disk demo workspace helper. Everything else reuses existing surfaces.

- [X] T003 Create `src/demo/workspace.rs` exporting `DemoWorkspace`,
  `seed_demo_workspace(root: &Path)`, `reset_demo_workspace(root: &Path)`, and
  `DemoWorkspaceError`. Implement seeding to produce exactly one buggy source file
  (`<root>/src/buggy.rs` containing a `// TODO-BUG: ...` marker plus a function whose
  body returns the wrong value) and one failing test definition file
  (`<root>/tests/buggy_test.rs` describing the expected behavior). Reject `root`
  paths whose final segment is not `demo-workspace` or whose parent segment is not
  `.synod`.
- [X] T004 Wire the new module by adding `pub mod workspace;` to `src/demo.rs` so the
  helper is reachable from `cli::run` and from tests.
- [X] T005 [P] Add `tests/unit/demo_workspace.rs` covering: (a) `seed_demo_workspace`
  creates the two seeded files with the expected contents; (b) `reset_demo_workspace`
  on an existing workspace removes prior contents and re-seeds the buggy state;
  (c) both functions reject roots that don't end in `.synod/demo-workspace`.
- [X] T006 [P] Update `tests/unit.rs` to register the new `mod demo_workspace;`
  module alongside the existing unit-test modules.

**Checkpoint**: After Phase 2 the demo workspace can be created and reset
deterministically and is unit-tested in isolation. No CLI / orchestrator change yet.

---

## Phase 3: User Story 1 — Run the demo and see a failing test become passing (Priority: P1)

**Goal**: A developer can run `synod run-demo` from a clean checkout, watch the
orchestrator execute analyzer → coder → tester end-to-end, and observe the seeded
buggy source file end in its fixed state with terminal status `Succeeded` and a trace
file path printed.

**Independent Test**: Run `synod run-demo`; assert the process exits 0, the trace
file exists at the printed path, the trace's `terminal_status` equals `Succeeded`,
and `<root>/src/buggy.rs` no longer contains the `// TODO-BUG` marker.

### Tests for User Story 1

- [X] T007 [P] [US1] Add `tests/contract/run_demo_contract.rs` asserting the CLI
  surface contract from `contracts/run-demo-cli.md`: subcommand name `run-demo`,
  optional `--workspace` flag only, no `--goal`, no `--profile`, default workspace
  path equals `<cwd>/.synod/demo-workspace`. The test parses the clap definition
  (and/or invokes the binary with `--help`) without executing the orchestrator.
- [X] T008 [P] [US1] Register the new contract test module in `tests/contract.rs`.
- [X] T009 [P] [US1] Add `tests/integration/run_demo_flow.rs` containing the
  end-to-end happy-path test: invoke `cli::run::execute_run_demo(<temp>)` against a
  temporary `.synod/demo-workspace` root, assert the returned `RunCommandReport`
  has `exit_status == Succeeded`, the trace file exists at
  `report.trace_location`, and the seeded `src/buggy.rs` inside the temp root no
  longer contains the bug marker.
- [X] T010 [P] [US1] Register the new integration test module in
  `tests/integration.rs`.

### Implementation for User Story 1

- [X] T011 [US1] Extend `src/cli.rs`: add `RunDemo { workspace: Option<PathBuf> }` to
  `DeveloperCommand`, add `RunDemo` to `CommandName` (with `as_str()` returning
  `"run-demo"`), extend `DeveloperCommand::name()` and
  `DeveloperCommandSession::from_command` to cover the new variant.
- [X] T012 [US1] Add `DemoRunProfile::test_fix_loop(workspace: &DemoWorkspace) -> Self`
  to `src/demo/profile.rs` per the data model: same step outline as `guided_demo`,
  injecting `target_file`, `fixed_content`, `bug_marker`, `force_retry` (on `code`),
  and `force_replan` (on `verify`). Add a unit test in `src/demo/profile.rs`
  asserting the constructed profile validates and contains the expected step inputs.
- [X] T013 [US1] Extend the coder closure in `src/demo/endpoints.rs`: when the step
  input contains `target_file` and `fixed_content`, the **successful** branch writes
  `fixed_content` to `target_file` (truncate + write, propagate I/O errors as a
  `Recoverability::Terminal` failure with cause `coder_io`) and reports the path in
  `output["updated_file"]`. The existing `force_retry` and `force_replan` branches
  remain unchanged.
- [X] T014 [US1] Extend the tester closure in `src/demo/endpoints.rs`: when the step
  input contains `target_file` and `bug_marker`, read the file once per attempt and
  report success (`output["verified_file"] = path`) when the marker is absent. I/O
  errors return a `Recoverability::Terminal` failure with cause `tester_io`. The
  existing `force_terminal_failure` branch remains unchanged. (The new
  `force_replan` branch is added in Phase 5 / US3.)
- [X] T015 [US1] Add `pub fn execute_run_demo(workspace_root: &Path) -> Result<RunCommandReport, RunCommandError>`
  to `src/cli/run.rs`. Implementation: canonicalize the workspace root (default to
  `<cwd>/.synod/demo-workspace` when the caller passes `None` via a thin wrapper),
  call `reset_demo_workspace`, build `DemoRunProfile::test_fix_loop(&workspace)`,
  reuse `execute_profile("run-demo", profile, &root)`, and append a
  `final source file: <target_file>` line to the rendered terminal output before
  returning.
- [X] T016 [US1] Add a `RunCommandError::DemoWorkspace(#[from] DemoWorkspaceError)`
  variant in `src/cli/run.rs` and route it through the existing `bin/synod.rs` exit
  handling so workspace seed failures map to a non-zero exit.
- [X] T017 [US1] Route the new variant in `src/bin/synod.rs`: when
  `DeveloperCommand::RunDemo { workspace }` is received, resolve the workspace path
  (default to `<cwd>/.synod/demo-workspace`) and call `execute_run_demo`. Emit the
  resulting terminal output and exit code via the existing rendering pipeline.

**Checkpoint**: After Phase 3 the demo runs end-to-end on a real on-disk workspace,
the bug file is overwritten with the fix, and the trace shows the full plan.
US1 acceptance scenarios both pass.

---

## Phase 4: User Story 2 — Visible retry on the coder step (Priority: P2)

**Goal**: A developer running `synod run-demo` sees the first coder attempt fail
recoverably and the second attempt succeed, both for the original `code` step and
in the trace file.

**Independent Test**: Inspect the trace produced by US1's integration test for an
entry on step `code` with `recoverability == Retryable` followed by a successful
attempt on the same `code` step.

### Tests for User Story 2

- [X] T018 [P] [US2] Extend `tests/integration/run_demo_flow.rs` (or add
  `tests/integration/run_demo_retry.rs`) to load the trace JSON written by the
  US1 run and assert: there is at least one trace entry where
  `step_id == "code"` and `recoverability == "Retryable"`, immediately followed
  (in trace order) by another `step_id == "code"` entry that succeeded with
  `attempt_number == 2`.
- [X] T019 [P] [US2] Register any new integration test module in
  `tests/integration.rs`.

### Implementation for User Story 2

- [X] T020 [US2] (No new code beyond US1 needed — the retry behavior is already
  produced by `force_retry` set in `DemoRunProfile::test_fix_loop` in T012 and the
  existing coder retry logic.) Confirm by running the new test in T018 and ensure
  it passes against the US1 implementation. If the trace shape needs a small
  helper to be readable from tests, add it under `src/adapters/trace_store.rs` as a
  pub(crate) reader and use it from the test only — do NOT extend the public
  surface.

**Checkpoint**: After Phase 4, US2's acceptance scenario is verified by an
automated test against the real trace.

---

## Phase 5: User Story 3 — Visible replan triggered by the tester (Priority: P3)

**Goal**: A developer running `synod run-demo` sees the first tester attempt return
`ReplanRequired`, sees an analyzer + coder pair inserted into the plan, and sees the
final tester attempt succeed.

**Independent Test**: Inspect the trace produced by the run-demo integration test
for a `step_id == "verify"` entry with `recoverability == "ReplanRequired"`,
followed by inserted analyzer and coder step entries, followed by a successful
`verify` attempt.

### Tests for User Story 3

- [X] T021 [P] [US3] Extend `tests/integration/run_demo_flow.rs` (or add
  `tests/integration/run_demo_replan.rs`) to assert: the trace contains a
  `verify` entry with `recoverability == "ReplanRequired"`; the next steps in
  trace order are an analyzer step and a coder step inserted by replan; the final
  `verify` entry succeeded; the orchestrator response's `terminal_status ==
  "Succeeded"` and `replan_count == 1`.
- [X] T022 [P] [US3] Register the new integration test module in
  `tests/integration.rs` if a new file was added.

### Implementation for User Story 3

- [X] T023 [US3] Extend the tester closure in `src/demo/endpoints.rs` to honor a
  `force_replan` flag in the step input. Use the same `Arc<Mutex<HashMap<String, usize>>>`
  retry-counter pattern already in scope. On the **first** attempt of a step with
  `force_replan == true`, return
  `StepExecutionResult::failure(ErrorInfo::new("deterministic_replan", ...), Recoverability::ReplanRequired)`
  with attempt evidence; on subsequent attempts continue with the normal
  pass/fail logic from T014.
- [X] T024 [US3] Add a unit test inside `src/demo/endpoints.rs` (the existing
  `#[cfg(test)] mod tests`) asserting that, given `force_replan = true` and a
  fresh attempt counter, the first tester invocation returns
  `Recoverability::ReplanRequired` and the second returns success when the
  bug marker is absent.

**Checkpoint**: After Phase 5, US3's acceptance scenario is verified by an
automated test against the real trace, and `synod run-demo` produces the full
retry + replan + success arc on every invocation.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T025 [P] Add coverage for the spec's edge cases in
  `tests/integration/run_demo_edge_cases.rs`: (a) retry-limit exhaustion (force
  the coder to keep failing recoverably and assert `Failed` terminal status with
  `retry_limit_exhausted`); (b) replan-limit exhaustion (force the tester to keep
  returning `ReplanRequired` and assert `Failed` terminal status with
  `replan_limit_exhausted`). Use new ad-hoc profiles in the test module that
  reuse `build_demo_runtime`; do NOT add public profile constructors that aren't
  needed by `synod run-demo`.
- [X] T026 [P] Update `README.md` (and `ROADMAP.md` if appropriate) with a short
  "Try it: `synod run-demo`" section linking to
  `specs/005-test-fix-loop-demo/quickstart.md`.
- [X] T027 Run `cargo fmt --all`, then
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, then
  `cargo test --all-targets`. All must pass green before the slice is considered
  done.
- [X] T028 Manually run `cargo run --release -- run-demo` from a clean working
  tree, verify the printed step sequence matches the contract in
  `contracts/run-demo-cli.md`, verify `<cwd>/.synod/demo-workspace/src/buggy.rs`
  ends in the fixed state, and verify the printed trace file contains the
  retry, replan, and `Succeeded` markers.

---

## Dependency notes

- **T001–T002** are pure verification, no edits, MUST happen first.
- **T003** unblocks every later task (US1 / US2 / US3 all depend on the demo
  workspace helper). T004 / T005 / T006 follow T003.
- **Phase 3 (US1)** delivers the MVP. T011 → T012 → T013 → T014 → T015 → T016 →
  T017 sequence is mostly serial because all touch a small set of shared files
  (`src/cli.rs`, `src/cli/run.rs`, `src/demo/endpoints.rs`); T007–T010 (tests)
  can be written in parallel before the implementation tasks.
- **Phase 4 (US2)** depends only on US1 plus T012 (which already sets the
  `force_retry` flag on the `code` step).
- **Phase 5 (US3)** depends on US1 plus T012 (which already sets the
  `force_replan` flag on the `verify` step) and T023 (which makes the tester
  adapter honor that flag). Phase 5 implementation cannot land without T023.
- **Phase 6** depends on Phases 3–5 complete.

## Parallelization summary

Tasks marked `[P]` touch different files and can be authored concurrently.
Inside Phase 3, tests T007–T010 are `[P]` and can be written before any
implementation; implementation tasks T011–T017 are mostly serial.
