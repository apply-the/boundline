# Tasks: Execution Engine (Code Delivery)

**Input**: Design documents from `/specs/006-execution-engine/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds real workspace mutation, validation-command execution, bounded recovery, trace-visible change evidence, and a repository-wide 90%-per-file coverage gate.

**Organization**: Tasks are grouped by user story so each execution-engine slice can be implemented, validated, and reviewed independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the execution-engine model and expand the test harnesses before delivery behavior changes.

- [x] T001 Wire execution-engine module exports and harness entries in src/domain.rs, src/lib.rs, tests/unit.rs, tests/integration.rs, and tests/contract.rs
- [x] T002 [P] Scaffold execution-profile test files in tests/unit/execution_profile.rs, tests/contract/execution_profile_contract.rs, and tests/contract/run_command_contract.rs
- [x] T003 [P] Extend workspace test helpers for execution manifests and multi-attempt workspaces in tests/support/workspace_fixture.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared execution-profile model, evidence projection, and runtime seams that all user stories rely on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 Create execution-profile domain types and validation rules in src/domain/execution.rs
- [x] T005 [P] Extend task-context and session-status evidence projection for changed files and validation outcome in src/domain/task_context.rs and src/domain/session.rs
- [x] T006 Replace fixture-only manifest loading with execution-profile loading plus legacy fixture conversion in src/fixture.rs and src/cli/diagnostics.rs
- [x] T007 Implement an execution-profile planner and runtime registration seam in src/orchestrator/planner.rs and src/fixture.rs
- [x] T008 [P] Extend trace summary and shared renderers to accept structured change evidence and validation summaries in src/domain/trace.rs, src/cli/output.rs, and src/cli/inspect.rs

**Checkpoint**: Foundation ready - execution profiles can be parsed, validated, projected into session state, and routed through the existing orchestrator loop.

---

## Phase 3: User Story 1 - Deliver a bounded code change (Priority: P1) MVP

**Goal**: Let `synod run` and the non-flow session path apply a real workspace change set, run validation, and stop in an explicit terminal state with inspectable evidence.

**Independent Test**: Run Synod against a temporary Rust workspace containing `.synod/execution.json`, confirm files are modified, validation passes, and the resulting trace exposes change evidence.

### Tests for User Story 1

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [x] T009 [P] [US1] Add unit coverage for execution-profile validation and legacy fixture conversion in tests/unit/execution_profile.rs
- [x] T010 [P] [US1] Add contract coverage for diagnostics and run-command success output in tests/contract/execution_profile_contract.rs and tests/contract/run_command_contract.rs
- [x] T011 [P] [US1] Add integration coverage for `synod run` applying execution-profile changes and persisting trace evidence in tests/integration/cli_custom_run.rs and tests/integration/fixture_vertical_slice.rs

### Implementation for User Story 1

- [x] T012 [P] [US1] Implement execution-profile parsing, legacy fallback, and task-request preparation in src/domain/execution.rs and src/fixture.rs
- [x] T013 [US1] Implement workspace analysis plus change-application evidence capture in src/fixture.rs
- [x] T014 [US1] Integrate validation-command execution and succeeded terminal projection in src/cli/run.rs, src/orchestrator/engine.rs, and src/fixture.rs
- [x] T015 [US1] Surface changed files and latest validation result in status and inspect output in src/domain/session.rs, src/cli/output.rs, and src/cli/inspect.rs

**Checkpoint**: User Story 1 is complete when Synod can deliver a real workspace change with passing validation and inspectable evidence.

---

## Phase 4: User Story 2 - Recover inside the validation loop (Priority: P2)

**Goal**: Keep failed validation inside the same bounded delivery run through explicit retry or replan behavior driven by later execution attempts.

**Independent Test**: Execute a profile with multiple attempts where the first validation fails and confirm that Synod records the failure, replans or retries within limits, and either succeeds or reaches an explicit exhausted or failed terminal state.

### Tests for User Story 2

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [x] T016 [P] [US2] Add unit coverage for attempt selection and failure-mode mapping in tests/unit/execution_profile.rs and tests/unit/planner_behaviors.rs
- [x] T017 [P] [US2] Add contract coverage for failed validation and legacy fallback diagnostics in tests/contract/run_command_contract.rs and tests/contract/diagnostics_report_contract.rs
- [x] T018 [P] [US2] Add integration coverage for bounded replan or retry across multiple attempts in tests/integration/retry_and_replan.rs and tests/integration/session_cli_flow.rs

### Implementation for User Story 2

- [x] T019 [P] [US2] Implement execution-profile replans from the attempt queue in src/orchestrator/planner.rs and src/fixture.rs
- [x] T020 [US2] Route the non-flow session plan and run path through execution profiles while preserving existing flow-backed sessions in src/orchestrator/session_runtime.rs and src/cli/session.rs
- [x] T021 [US2] Persist validation failure and recovery evidence into task context and traces in src/domain/task_context.rs, src/domain/trace.rs, src/orchestrator/engine.rs, and src/orchestrator/session_runtime.rs
- [x] T022 [US2] Harden workspace-boundary, missing-manifest, and command-launch errors in src/fixture.rs and src/cli/diagnostics.rs

**Checkpoint**: User Story 2 is complete when validation failures recover or terminate inside explicit limits with trace-visible evidence.

---

## Phase 5: User Story 3 - Inspect delivered output and evidence (Priority: P3)

**Goal**: Make changed files, validation outcomes, and recovery history easy to inspect after a delivery run.

**Independent Test**: After a successful and a failed delivery run, verify that `status` and `inspect` both expose changed files, validation outcomes, and the final terminal reason.

### Tests for User Story 3

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [x] T023 [P] [US3] Add contract coverage for trace-summary and session-status evidence projection in tests/contract/trace_summary_contract.rs and tests/contract/session_command_contract.rs
- [x] T024 [P] [US3] Add integration coverage for status and inspect after successful and failed delivery runs in tests/integration/session_cli_flow.rs and tests/integration/cli_trace_inspection.rs

### Implementation for User Story 3

- [x] T025 [P] [US3] Project latest changed files and validation outcome into session status views in src/domain/session.rs and src/cli/output.rs
- [x] T026 [US3] Extend trace summarization to expose change and validation headlines in src/cli/inspect.rs and src/domain/trace.rs
- [x] T027 [US3] Render post-run change evidence and recovery history in src/cli/output.rs and src/cli/run.rs

**Checkpoint**: All execution-engine user stories are now independently inspectable and usable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Close the feature with docs, versioning, coverage, and full validation.

- [x] T028 [P] Update execution-engine documentation in README.md, assistant/README.md, ROADMAP.md, AGENTS.md, and specs/006-execution-engine/quickstart.md
- [x] T029 [P] Bump crate version to 0.7.0 in Cargo.toml and Cargo.lock
- [x] T030 [P] Raise every Rust source file under src/ to at least 90% line coverage by expanding tests in tests/unit/, tests/contract/, and tests/integration/
- [x] T031 Run formatting, lint, test, and coverage validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP execution path.
- **Phase 4: User Story 2**: Depends on Phase 2 and builds on the same execution-profile runtime.
- **Phase 5: User Story 3**: Depends on Phase 2 and should land after execution evidence is stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but depends on the execution-profile runtime created for US1.
- **US3 (P3)**: Starts after Foundational and is safest once US1 and US2 have stabilized the evidence model.

### Within Each User Story

- Contract, unit, and integration coverage should be written first and observed failing before implementation.
- Domain and planner changes should land before CLI renderers or session projections that consume them.
- Change evidence and validation records should be stable before inspect and status messaging is finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005 and T008 can run in parallel once T004 exists; T006 and T007 should sequence the manifest and planner core.
- **US1**: T009, T010, and T011 can run in parallel; T012 can run in parallel with the first draft of T013.
- **US2**: T016, T017, and T018 can run in parallel; T019 can run in parallel with the first draft of T022 once the planner seam exists.
- **US3**: T023 and T024 can run in parallel; T025 can run in parallel with T026 once evidence projection fields exist.
- **Polish**: T028, T029, and T030 can run in parallel before the final validation task T031.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T009 Add unit coverage for execution-profile validation and legacy fixture conversion in tests/unit/execution_profile.rs"
Task: "T010 Add contract coverage for diagnostics and run-command success output in tests/contract/execution_profile_contract.rs and tests/contract/run_command_contract.rs"
Task: "T011 Add integration coverage for `synod run` applying execution-profile changes and persisting trace evidence in tests/integration/cli_custom_run.rs and tests/integration/fixture_vertical_slice.rs"

# Split parsing and evidence work after tests exist:
Task: "T012 Implement execution-profile parsing, legacy fallback, and task-request preparation in src/domain/execution.rs and src/fixture.rs"
Task: "T013 Implement workspace analysis plus change-application evidence capture in src/fixture.rs"
```

## Parallel Example: User Story 2

```bash
# Validate recovery behavior together:
Task: "T016 Add unit coverage for attempt selection and failure-mode mapping in tests/unit/execution_profile.rs and tests/unit/planner_behaviors.rs"
Task: "T017 Add contract coverage for failed validation and legacy fallback diagnostics in tests/contract/run_command_contract.rs and tests/contract/diagnostics_report_contract.rs"
Task: "T018 Add integration coverage for bounded replan or retry across multiple attempts in tests/integration/retry_and_replan.rs and tests/integration/session_cli_flow.rs"

# Then split planner and hardening work:
Task: "T019 Implement execution-profile replans from the attempt queue in src/orchestrator/planner.rs and src/fixture.rs"
Task: "T022 Harden workspace-boundary, missing-manifest, and command-launch errors in src/fixture.rs and src/cli/diagnostics.rs"
```

## Parallel Example: User Story 3

```bash
# Validate evidence projection together:
Task: "T023 Add contract coverage for trace-summary and session-status evidence projection in tests/contract/trace_summary_contract.rs and tests/contract/session_command_contract.rs"
Task: "T024 Add integration coverage for status and inspect after successful and failed delivery runs in tests/integration/session_cli_flow.rs and tests/integration/cli_trace_inspection.rs"

# Then split status and inspect work:
Task: "T025 Project latest changed files and validation outcome into session status views in src/domain/session.rs and src/cli/output.rs"
Task: "T026 Extend trace summarization to expose change and validation headlines in src/cli/inspect.rs and src/domain/trace.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the direct `run` path on a temporary workspace.
5. Confirm the trace captures change evidence before expanding recovery and inspection behavior.

### Incremental Delivery

1. Deliver Setup + Foundational to establish the execution-profile runtime and evidence model.
2. Deliver US1 to make real workspace delivery available.
3. Deliver US2 to add bounded retry and replan behavior around failed validation.
4. Deliver US3 to expose high-quality status and inspect evidence.
5. Finish with docs, versioning, per-file coverage, and full validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared execution-profile and evidence foundation so the first increment already delivers working code rather than another planning-only surface.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Coverage is a first-class deliverable in this slice because the user explicitly requires at least 90% line coverage for every Rust source file.
- Legacy fixture compatibility is part of the feature, not a post-release polish item.