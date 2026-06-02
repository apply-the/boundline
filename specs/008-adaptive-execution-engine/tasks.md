# Tasks: Adaptive Execution Engine

**Input**: Design documents from `/specs/008-adaptive-execution-engine/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new executable planner behavior, adaptive workspace-slice selection, bounded replanning, explicit failure handling, session projection, and trace-visible evidence.

**Organization**: Tasks are grouped by user story so each bounded adaptive execution slice can be implemented, validated, and reviewed independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the adaptive execution surface and extend the test harnesses before behavior changes.

- [ ] T001 Wire adaptive execution module exports and harness entries in src/domain.rs, src/lib.rs, tests/unit.rs, tests/contract.rs, and tests/integration.rs
- [ ] T002 [P] Scaffold adaptive unit, contract, and integration test files in tests/unit/adaptive_execution.rs, tests/contract/adaptive_execution_profile_contract.rs, tests/contract/adaptive_run_contract.rs, tests/contract/adaptive_session_contract.rs, tests/contract/adaptive_trace_contract.rs, tests/integration/cli_adaptive_execution.rs, and tests/integration/session_adaptive_flow.rs
- [ ] T003 [P] Extend workspace fixture helpers for adaptive execution scenarios in tests/support/workspace_fixture.rs and shared test utilities

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared adaptive profile, planner, state projection, and trace primitives that every story relies on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create adaptive execution domain types, validation rules, and session projection primitives in src/domain/execution.rs, src/domain/task_context.rs, and src/domain/session.rs
- [X] T005 [P] Extend manifest loading and legacy compatibility for adaptive execution profiles in src/domain/execution.rs and src/fixture.rs
- [X] T006 [P] Add adaptive trace event and summary support in src/domain/trace.rs, src/cli/output.rs, and src/cli/inspect.rs
- [X] T007 [P] Extend planner and runtime interfaces for synthesized adaptive attempts in src/orchestrator/planner.rs, src/orchestrator/engine.rs, and src/orchestrator/session_runtime.rs
- [X] T008 Implement shared no-repeat candidate-signature and attempt-lineage primitives in src/domain/execution.rs, src/fixture.rs, and src/orchestrator/planner.rs

**Checkpoint**: Foundation ready - adaptive profiles can be described, loaded, persisted, and traced inside the existing execution lifecycle.

---

## Phase 3: User Story 1 - Select And Change The Relevant Workspace Slice (Priority: P1) 🎯 MVP

**Goal**: Let Boundline identify one bounded workspace slice from the current repository state, synthesize one adaptive change attempt, and execute it through the existing run flow.

**Independent Test**: Run Boundline against a temporary workspace with an adaptive execution profile, confirm a bounded workspace slice is selected, a synthesized change attempt is applied, validation runs, and `run`/`inspect` expose the adaptive evidence.

### Tests for User Story 1

- [X] T009 [P] [US1] Add unit coverage for adaptive profile validation, slice scoring, and candidate generation in tests/unit/adaptive_execution.rs and tests/unit/execution_profile.rs
- [ ] T010 [P] [US1] Add contract coverage for adaptive profile parsing and adaptive run output in tests/contract/adaptive_execution_profile_contract.rs and tests/contract/adaptive_run_contract.rs
- [X] T011 [P] [US1] Add integration coverage for a successful adaptive run and direct inspect flow in tests/integration/cli_adaptive_execution.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement adaptive profile parsing, bounded read-target scoring, and workspace-slice selection helpers in src/domain/execution.rs and src/fixture.rs
- [X] T013 [US1] Implement an adaptive planner that synthesizes the initial adaptive attempt in src/orchestrator/planner.rs and connect it from src/fixture.rs
- [X] T014 [US1] Teach the coder and verifier execution path to consume synthesized adaptive attempts in src/fixture.rs and src/orchestrator/engine.rs
- [X] T015 [US1] Render workspace-slice summaries and adaptive attempt metadata in src/cli/output.rs, src/cli/run.rs, and src/cli/inspect.rs

**Checkpoint**: User Story 1 is complete when a bounded adaptive run can solve one real workspace bug without pre-authored attempts and with inspectable evidence.

---

## Phase 4: User Story 2 - Adapt After Failed Validation (Priority: P2)

**Goal**: Keep failed validation inside one bounded adaptive delivery loop through new candidate synthesis, signature-based non-repeat behavior, and explicit exhausted or failed terminal outcomes.

**Independent Test**: Run adaptive scenarios where the first candidate fails validation, verify that Boundline records the failure, chooses a materially different next candidate or slice, and stops explicitly when no credible next path remains.

### Tests for User Story 2

- [ ] T016 [P] [US2] Add unit coverage for candidate-signature tracking, no-repeat enforcement, and attempt-lineage transitions in tests/unit/adaptive_execution.rs and tests/unit/planner_behaviors.rs
- [ ] T017 [P] [US2] Add contract coverage for non-success adaptive run output and adaptive trace semantics in tests/contract/adaptive_run_contract.rs and tests/contract/adaptive_trace_contract.rs
- [ ] T018 [P] [US2] Add integration coverage for failed-first-attempt replanning and exhausted adaptive execution in tests/integration/cli_adaptive_execution.rs and tests/integration/session_adaptive_flow.rs

### Implementation for User Story 2

- [X] T019 [US2] Implement validation-driven replanning and next-candidate synthesis in src/orchestrator/planner.rs and src/fixture.rs
- [X] T020 [US2] Persist adaptive candidate signatures, selection evidence, and attempt lineage in src/domain/task_context.rs, src/orchestrator/engine.rs, and src/orchestrator/session_runtime.rs
- [X] T021 [US2] Implement explicit no-credible-next-step and exhausted adaptive terminal handling in src/fixture.rs, src/orchestrator/engine.rs, and src/cli/output.rs
- [ ] T022 [US2] Harden invalid adaptive profile, unreadable target, and repeated-candidate diagnostics in src/domain/execution.rs, src/fixture.rs, and src/cli/diagnostics.rs

**Checkpoint**: User Story 2 is complete when failed adaptive attempts produce materially different replans or explicit terminal stop behavior with visible evidence.

---

## Phase 5: User Story 3 - Inspect Adaptive Decisions And Output (Priority: P3)

**Goal**: Make adaptive slice selection, attempt lineage, and terminal reasoning easy to inspect from the existing CLI surfaces.

**Independent Test**: After successful and non-success adaptive runs, verify that `status`, `next`, and `inspect` all expose the latest adaptive slice, lineage, validation evidence, and final terminal reasoning without manual reconstruction.

### Tests for User Story 3

- [ ] T023 [P] [US3] Add contract coverage for adaptive session projection and inspect summaries in tests/contract/adaptive_session_contract.rs and tests/contract/adaptive_trace_contract.rs
- [ ] T024 [P] [US3] Add integration coverage for adaptive-aware status, next, and inspect flows in tests/integration/session_adaptive_flow.rs and tests/integration/cli_adaptive_execution.rs

### Implementation for User Story 3

- [X] T025 [US3] Project latest workspace slice, selection headline, and attempt lineage into session status and next guidance in src/domain/session.rs and src/cli/session.rs
- [X] T026 [US3] Extend trace summarization and inspect rendering for adaptive slice-selection and lineage evidence in src/domain/trace.rs and src/cli/inspect.rs
- [ ] T027 [US3] Add adaptive-aware CLI output and assistant-facing summaries in src/cli/output.rs, src/cli/run.rs, and assistant/README.md

**Checkpoint**: All adaptive execution outcomes are inspectable through the existing bounded CLI surfaces.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, versioning, coverage, and full validation for the adaptive slice.

- [X] T028 [P] Add adaptive execution documentation in tech-docs/adaptive-execution.md covering manifest shape, workspace-slice selection, deterministic candidate generation, no-repeat behavior, and explicit terminal outcomes
- [ ] T029 [P] Update feature documentation in README.md, ROADMAP.md, AGENTS.md, assistant/README.md, and specs/008-adaptive-execution-engine/quickstart.md
- [X] T030 [P] Bump crate and lockfile version to 0.8.0 in Cargo.toml and Cargo.lock
- [ ] T031 [P] Raise source coverage for new adaptive paths in tests/unit/adaptive_execution.rs, adaptive contract tests, and adaptive integration tests
- [ ] T032 Run formatting, lint, test, and coverage validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP adaptive slice.
- **Phase 4: User Story 2**: Depends on Phase 2 and builds on the same adaptive planner and evidence model created for US1.
- **Phase 5: User Story 3**: Depends on Phase 2 and is safest once adaptive evidence is stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but depends on the adaptive evidence model established in US1.
- **US3 (P3)**: Starts after Foundational and is safest once US1 and US2 have stabilized the adaptive surfaces.

### Within Each User Story

- Contract, unit, and integration coverage should be written first and observed failing before implementation.
- Domain and planner changes should land before CLI rendering or session projection work that consumes them.
- Adaptive slice selection and no-repeat rules should be stable before inspect and status messaging is finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel once T004 exists; T008 should sequence the shared adaptive rules.
- **US1**: T009, T010, and T011 can run in parallel; T012 and T013 can overlap once the domain model is stable.
- **US2**: T016, T017, and T018 can run in parallel; T019 and T022 can run in parallel after shared adaptive primitives exist.
- **US3**: T023 and T024 can run in parallel; T025 and T026 can overlap once adaptive evidence is persisted.
- **Polish**: T028, T029, T030, and T031 can run in parallel before the final validation task T032.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T009 Add unit coverage for adaptive profile validation, slice scoring, and candidate generation in tests/unit/adaptive_execution.rs and tests/unit/execution_profile.rs"
Task: "T010 Add contract coverage for adaptive profile parsing and adaptive run output in tests/contract/adaptive_execution_profile_contract.rs and tests/contract/adaptive_run_contract.rs"
Task: "T011 Add integration coverage for a successful adaptive run and direct inspect flow in tests/integration/cli_adaptive_execution.rs"

# Split selection and planning work after tests exist:
Task: "T012 Implement adaptive profile parsing, bounded read-target scoring, and workspace-slice selection helpers in src/domain/execution.rs and src/fixture.rs"
Task: "T013 Implement an adaptive planner that synthesizes the initial adaptive attempt in src/orchestrator/planner.rs and connect it from src/fixture.rs"
```

## Parallel Example: User Story 2

```bash
# Validate non-success behavior together:
Task: "T016 Add unit coverage for candidate-signature tracking, no-repeat enforcement, and attempt-lineage transitions in tests/unit/adaptive_execution.rs and tests/unit/planner_behaviors.rs"
Task: "T017 Add contract coverage for non-success adaptive run output and adaptive trace semantics in tests/contract/adaptive_run_contract.rs and tests/contract/adaptive_trace_contract.rs"
Task: "T018 Add integration coverage for failed-first-attempt replanning and exhausted adaptive execution in tests/integration/cli_adaptive_execution.rs and tests/integration/session_adaptive_flow.rs"

# Then split core replanning and hardening work:
Task: "T019 Implement validation-driven replanning and next-candidate synthesis in src/orchestrator/planner.rs and src/fixture.rs"
Task: "T022 Harden invalid adaptive profile, unreadable target, and repeated-candidate diagnostics in src/domain/execution.rs, src/fixture.rs, and src/cli/diagnostics.rs"
```

## Parallel Example: User Story 3

```bash
# Validate status and inspect behavior together:
Task: "T023 Add contract coverage for adaptive session projection and inspect summaries in tests/contract/adaptive_session_contract.rs and tests/contract/adaptive_trace_contract.rs"
Task: "T024 Add integration coverage for adaptive-aware status, next, and inspect flows in tests/integration/session_adaptive_flow.rs and tests/integration/cli_adaptive_execution.rs"

# Then split status and inspect implementation:
Task: "T025 Project latest workspace slice, selection headline, and attempt lineage into session status and next guidance in src/domain/session.rs and src/cli/session.rs"
Task: "T026 Extend trace summarization and inspect rendering for adaptive slice-selection and lineage evidence in src/domain/trace.rs and src/cli/inspect.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate one successful adaptive run on a temporary workspace.
5. Confirm adaptive slice evidence is visible before expanding failed-validation replanning and docs.

### Incremental Delivery

1. Deliver Setup + Foundational to establish the adaptive planner and evidence model.
2. Deliver US1 to make bounded adaptive execution available.
3. Deliver US2 to add validation-driven replanning, no-repeat enforcement, and explicit non-success outcomes.
4. Deliver US3 to expose high-quality status and inspect evidence.
5. Finish with docs, version bump to 0.8.0, coverage, and full validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared adaptive foundation so the first increment already delivers a real adaptive execution path instead of documentation-only scaffolding.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Adaptive execution documentation is a first-class deliverable in this slice because the user explicitly requires visible explanation of how bounded slice selection and recovery work.
- The crate version moves to 0.8.0 for this slice and must stay aligned across Cargo.toml, Cargo.lock, docs, and user-facing examples.
