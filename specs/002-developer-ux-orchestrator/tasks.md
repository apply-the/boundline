---

description: "Task list for implementing the developer UX for the orchestrator core"
---

# Tasks: Developer UX for Orchestrator Core

**Input**: Design documents from `/specs/002-developer-ux-orchestrator/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are included because this feature defines executable CLI behavior, bounded failure handling, deterministic recovery visibility, and trace inspection guarantees.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Every task includes exact file paths in the description

## Path Conventions

- Root crate files live in `Cargo.toml` and `src/`
- Validation files live in `tests/unit/`, `tests/integration/`, and `tests/contract/`
- Feature planning artifacts live in `specs/002-developer-ux-orchestrator/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the CLI package surface and create the new source and validation entry points described in the plan.

- [X] T001 Add the CLI dependency and package wiring in `Cargo.toml`
- [X] T002 [P] Create the CLI and demo module entrypoints in `src/bin/synod.rs`, `src/cli.rs`, and `src/demo.rs`
- [X] T003 [P] Extend the test harness registration for CLI coverage in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared command, deterministic fixture, trace-loading, and output primitives used by every story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Implement shared command parsing, command session types, and exit-code mapping in `src/cli.rs` and `src/cli/output.rs`
- [X] T005 [P] Implement deterministic demo and default-run profile builders in `src/demo/profile.rs` and `src/demo/endpoints.rs`
- [X] T006 [P] Extend the trace store with trace read and latest-trace lookup support in `src/adapters/trace_store.rs` and `src/domain/trace.rs`
- [X] T007 Implement shared workspace validation and actionable CLI error surfaces in `src/cli/diagnostics.rs` and `src/cli/output.rs`
- [X] T008 [P] Create the new CLI contract and integration test files in `tests/contract/cli_command_contract.rs`, `tests/contract/diagnostics_report_contract.rs`, `tests/contract/trace_summary_contract.rs`, `tests/integration/cli_demo_flow.rs`, `tests/integration/cli_custom_run.rs`, `tests/integration/cli_diagnostics.rs`, and `tests/integration/cli_trace_inspection.rs`
- [X] T009 [P] Create shared CLI output unit coverage scaffolding in `tests/unit/cli_output.rs` and `tests/unit.rs`

**Checkpoint**: Foundation ready. The command surface, deterministic local fixtures, trace-loading helpers, and shared output rules exist for all user stories.

---

## Phase 3: User Story 1 - Run a Guided Demo Task (Priority: P1) 🎯 MVP

**Goal**: Let a contributor verify local readiness and run a deterministic guided demo with visible step progression, visible recovery behavior, and an explicit terminal outcome.

**Independent Test**: From a fresh local checkout, run `cargo run --bin synod -- doctor --workspace "$PWD"` and `cargo run --bin synod -- demo --workspace "$PWD"`, then confirm readiness is reported, demo step progression is visible, at least one recovery event is surfaced, and the command ends with an explicit terminal status and trace location.

### Tests for User Story 1

- [X] T010 [P] [US1] Implement the developer command contract tests for `doctor` and `demo` in `tests/contract/cli_command_contract.rs`
- [X] T011 [P] [US1] Implement the diagnostics report contract tests in `tests/contract/diagnostics_report_contract.rs`
- [X] T012 [P] [US1] Implement the doctor and guided demo integration scenarios in `tests/integration/cli_diagnostics.rs` and `tests/integration/cli_demo_flow.rs`

### Implementation for User Story 1

- [X] T013 [P] [US1] Implement the `doctor` command checks and diagnostics report rendering in `src/bin/synod.rs`, `src/cli.rs`, and `src/cli/diagnostics.rs`
- [X] T014 [US1] Implement the deterministic guided demo command flow in `src/bin/synod.rs`, `src/cli.rs`, and `src/cli/run.rs`
- [X] T015 [US1] Wire the demo profile and built-in execution endpoints into the existing orchestrator in `src/demo/profile.rs`, `src/demo/endpoints.rs`, and `src/orchestrator/planner.rs`
- [X] T016 [US1] Surface readable step progression, recovery events, terminal outcomes, and trace locations in `src/cli/output.rs` and `src/cli/run.rs`
- [X] T017 [US1] Export any CLI-facing library helpers needed by the binary in `src/lib.rs`, `src/cli.rs`, and `src/demo.rs`

**Checkpoint**: User Story 1 is independently functional and delivers the first-run developer experience.

---

## Phase 4: User Story 2 - Run a Simple Custom Task (Priority: P2)

**Goal**: Allow a developer to submit a bounded local goal through the CLI, execute the default developer flow, and receive explicit progress, terminal status, and trace output.

**Independent Test**: Run `cargo run --bin synod -- run --goal "Summarize the current bounded developer flow" --workspace "$PWD"` and confirm the command validates the goal, executes the default bounded flow, reports progress, and leaves behind a trace that still explains non-success outcomes when the run fails.

### Tests for User Story 2

- [X] T018 [P] [US2] Extend the developer command contract coverage for `run` invocations and exit semantics in `tests/contract/cli_command_contract.rs`
- [X] T019 [P] [US2] Implement successful custom-run integration scenarios in `tests/integration/cli_custom_run.rs`
- [X] T020 [P] [US2] Implement non-success custom-run and trace-persistence scenarios in `tests/integration/cli_custom_run.rs`

### Implementation for User Story 2

- [X] T021 [P] [US2] Implement custom-run argument validation and request assembly in `src/bin/synod.rs` and `src/cli/run.rs`
- [X] T022 [US2] Implement the default developer flow for custom goals in `src/cli/run.rs`, `src/demo/profile.rs`, and `src/demo/endpoints.rs`
- [X] T023 [US2] Integrate custom-run progress, error, and trace reporting in `src/cli/output.rs` and `src/cli.rs`
- [X] T024 [US2] Make custom runs surface explicit success and non-success exits with trace locations in `src/bin/synod.rs`, `src/cli/run.rs`, and `tests/integration/cli_custom_run.rs`

**Checkpoint**: User Stories 1 and 2 both work, and the CLI can execute both a guided demo and a developer-supplied bounded objective.

---

## Phase 5: User Story 3 - Inspect a Recorded Run (Priority: P3)

**Goal**: Let a developer inspect a persisted trace through a readable command that reconstructs step order, recovery events, and the final terminal reason without reading raw trace data manually.

**Independent Test**: Generate a trace with `demo` or `run`, then execute `cargo run --bin synod -- inspect --trace <trace-path>` and confirm the output reconstructs the executed step order, recovery path, and final terminal reason from the stored trace alone.

### Tests for User Story 3

- [X] T025 [P] [US3] Implement the trace summary contract coverage in `tests/contract/trace_summary_contract.rs`
- [X] T026 [P] [US3] Implement the trace inspection integration scenarios in `tests/integration/cli_trace_inspection.rs`

### Implementation for User Story 3

- [X] T027 [P] [US3] Implement trace-summary transformation and validation in `src/cli/inspect.rs` and `src/domain/trace.rs`
- [X] T028 [US3] Implement the `inspect` command, trace selection, and read-error handling in `src/bin/synod.rs`, `src/cli.rs`, `src/cli/inspect.rs`, and `src/adapters/trace_store.rs`
- [X] T029 [US3] Surface step order, recovery events, and terminal reason in the inspection output in `src/cli/output.rs` and `tests/integration/cli_trace_inspection.rs`

**Checkpoint**: All user stories are independently functional, and persisted traces are readable through the command surface.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Tighten documentation, naming, validation coverage, and final command ergonomics across the full slice.

- [X] T030 [P] Update developer-facing usage and walkthroughs in `README.md` and `specs/002-developer-ux-orchestrator/quickstart.md`
- [X] T031 Clean up CLI naming, exit-code wiring, and actionable error diagnostics in `src/bin/synod.rs`, `src/cli.rs`, `src/cli/output.rs`, and `src/cli/diagnostics.rs`
- [X] T032 [P] Add final unit coverage for CLI output and dispatch helpers in `tests/unit/cli_output.rs` and `tests/unit.rs`
- [X] T033 Validate the documented command flows against `Cargo.toml`, `README.md`, and `specs/002-developer-ux-orchestrator/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies; start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user story work.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP developer experience.
- **Phase 4: User Story 2**: Depends on Phase 3 because it reuses the CLI entrypoint, shared output surfaces, and deterministic default flow wiring established for the demo.
- **Phase 5: User Story 3**: Depends on Phase 3 because it reuses the command entrypoint and consumes the persisted traces produced by the runnable command surface.
- **Phase 6: Polish**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: No story dependency after Foundational.
- **US2 (P2)**: Depends on US1 command dispatch and shared output plumbing.
- **US3 (P3)**: Depends on US1 command dispatch and persisted trace generation; it does not require US2 to be complete.

### Within Each User Story

- Validation tasks MUST fail before implementation changes are considered complete.
- Contracts and integration expectations come before command wiring.
- Shared command and output helpers come before story-specific polish.
- Story-specific pass criteria are met only after progress output, exit semantics, and trace behavior are validated.

### Parallel Opportunities

- Setup: T002 and T003 can proceed in parallel after T001.
- Foundational: T005, T006, T008, and T009 can proceed in parallel once T004 defines the command and output shape.
- US1: T010, T011, and T012 can proceed in parallel.
- US2: T018, T019, and T020 can proceed in parallel.
- US3: T025 and T026 can proceed in parallel.
- Polish: T030 and T032 can proceed in parallel after story completion.

---

## Parallel Example: User Story 1

```bash
# Launch US1 validation tasks together:
Task: "Implement the developer command contract tests in tests/contract/cli_command_contract.rs"
Task: "Implement the diagnostics report contract tests in tests/contract/diagnostics_report_contract.rs"
Task: "Implement the doctor and guided demo integration scenarios in tests/integration/cli_diagnostics.rs and tests/integration/cli_demo_flow.rs"

# Launch independent implementation work after shared foundations are ready:
Task: "Implement the doctor command checks and diagnostics report rendering in src/bin/synod.rs, src/cli.rs, and src/cli/diagnostics.rs"
Task: "Wire the demo profile and built-in execution endpoints into src/demo/profile.rs, src/demo/endpoints.rs, and src/orchestrator/planner.rs"
```

## Parallel Example: User Story 2

```bash
# Launch US2 validation tasks together:
Task: "Extend the developer command contract coverage for run invocations in tests/contract/cli_command_contract.rs"
Task: "Implement successful custom-run integration scenarios in tests/integration/cli_custom_run.rs"
Task: "Implement non-success custom-run and trace-persistence scenarios in tests/integration/cli_custom_run.rs"
```

## Parallel Example: User Story 3

```bash
# Launch US3 validation tasks together:
Task: "Implement the trace summary contract coverage in tests/contract/trace_summary_contract.rs"
Task: "Implement the trace inspection integration scenarios in tests/integration/cli_trace_inspection.rs"

# Launch independent implementation work after shared trace helpers are ready:
Task: "Implement trace-summary transformation and validation in src/cli/inspect.rs and src/domain/trace.rs"
Task: "Implement the inspect command and trace selection in src/bin/synod.rs, src/cli.rs, src/cli/inspect.rs, and src/adapters/trace_store.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate `tests/contract/cli_command_contract.rs`, `tests/contract/diagnostics_report_contract.rs`, `tests/integration/cli_diagnostics.rs`, and `tests/integration/cli_demo_flow.rs`.
5. Demo the first-run developer experience before expanding scope.

### Incremental Delivery

1. Finish Setup and Foundational to establish the CLI entrypoint, deterministic local fixtures, trace-read helpers, and shared output rules.
2. Ship US1 to prove onboarding and the guided demo path.
3. Ship US2 to prove bounded custom execution through the CLI.
4. Ship US3 to prove readable trace inspection over persisted runs.
5. Use Phase 6 to tighten docs, naming, diagnostics, and final validation without changing scope.

### Parallel Team Strategy

1. One engineer can own package wiring and shared command/output foundations.
2. A second engineer can implement deterministic demo/default flow builders while the first completes CLI parsing and trace helpers.
3. After US1 is stable, custom-run and trace-inspection work can overlap as long as shared files such as `src/bin/synod.rs`, `src/cli.rs`, and `src/cli/output.rs` are coordinated.

---

## Notes

- Total tasks: 33.
- User story task counts: US1 = 8, US2 = 7, US3 = 5.
- Suggested MVP scope: through Phase 3 (User Story 1) only.
- All tasks use the required checklist format: checkbox, task ID, optional `[P]`, required story label in story phases, and exact file paths.