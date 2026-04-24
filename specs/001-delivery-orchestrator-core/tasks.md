---

description: "Task list for implementing the delivery orchestrator core"
---

# Tasks: Delivery Orchestrator Core

**Input**: Design documents from `/specs/001-delivery-orchestrator-core/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are included because the feature requires executable
behavior, bounded failure handling, replanning, and persisted trace guarantees.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Every task includes exact file paths in the description

## Path Conventions

- Root crate files live in `Cargo.toml` and `src/`
- Validation files live in `tests/unit/`, `tests/integration/`, and `tests/contract/`
- Feature planning artifacts live in `specs/001-delivery-orchestrator-core/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Bootstrap the Rust crate and create the module and test layout from the plan.

- [X] T001 Create the Rust crate manifest and dependency set in Cargo.toml
- [X] T002 [P] Create the crate entry point and module declarations in src/lib.rs, src/domain/mod.rs, src/orchestrator/mod.rs, src/registry/mod.rs, and src/adapters/mod.rs
- [X] T003 [P] Create test scaffolding files in tests/contract/orchestrator_run.rs, tests/contract/endpoint_execution.rs, tests/contract/trace_record.rs, tests/integration/sequential_task_run.rs, tests/integration/retry_and_replan.rs, tests/integration/trace_capture.rs, tests/unit/step_state.rs, tests/unit/recovery_policy.rs, and tests/unit/terminal_precedence.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared state, registry, trace, and recovery primitives required by every story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Implement shared task and context models in src/domain/task.rs and src/domain/task_context.rs
- [X] T005 [P] Implement step, plan, and runtime limit models in src/domain/step.rs, src/domain/plan.rs, and src/domain/limits.rs
- [X] T006 [P] Implement agent/tool adapter traits and registries in src/adapters/agent.rs, src/adapters/tool.rs, src/registry/agent_registry.rs, and src/registry/tool_registry.rs
- [X] T007 [P] Implement trace entities and file-backed trace persistence in src/domain/trace.rs and src/adapters/trace_store.rs
- [X] T008 Implement recovery policy and terminal precedence primitives in src/orchestrator/recovery.rs and src/orchestrator/terminal.rs
- [X] T009 Implement planner scaffolding and shared fake endpoint fixtures in src/orchestrator/planner.rs, tests/contract/endpoint_execution.rs, and tests/integration/sequential_task_run.rs

**Checkpoint**: Foundation ready. Shared state, endpoint contracts, recovery policy,
and trace persistence exist for all user stories.

---

## Phase 3: User Story 1 - Execute Bounded Delivery Work (Priority: P1) 🎯 MVP

**Goal**: Deliver a sequential orchestrator run that executes ordered steps, preserves
shared context, and exits in an explicit terminal state.

**Independent Test**: Register fake analyzer, coder, and tester endpoints; run a
three-step task through the orchestrator; confirm the task reaches a success or failure
terminal state without manual intervention and with context preserved across steps.

### Tests for User Story 1

- [X] T010 [P] [US1] Implement task-run contract coverage for request validation and terminal responses in tests/contract/orchestrator_run.rs
- [X] T011 [P] [US1] Implement unit coverage for step lifecycle and terminal ordering in tests/unit/step_state.rs and tests/unit/terminal_precedence.rs
- [X] T012 [P] [US1] Implement three-step bounded execution scenarios in tests/integration/sequential_task_run.rs

### Implementation for User Story 1

- [X] T013 [US1] Implement the task-run entrypoint and sequential engine loop in src/orchestrator/engine.rs
- [X] T014 [US1] Implement context merge, step-attempt recording, and last-result updates in src/domain/task.rs, src/domain/task_context.rs, and src/domain/step.rs
- [X] T015 [US1] Implement registry-backed agent and tool dispatch for active steps in src/orchestrator/engine.rs, src/registry/agent_registry.rs, and src/registry/tool_registry.rs
- [X] T016 [US1] Export the bounded-run API and planner integration in src/lib.rs, src/orchestrator/mod.rs, and src/orchestrator/planner.rs
- [X] T017 [US1] Make the US1 contract and integration scenarios pass in tests/contract/orchestrator_run.rs and tests/integration/sequential_task_run.rs

**Checkpoint**: User Story 1 is independently functional and demonstrates the MVP orchestrator loop.

---

## Phase 4: User Story 2 - Recover From Failed Steps (Priority: P2)

**Goal**: Recoverable failures trigger bounded retries or replanning, while exhausted
or unrecoverable failures terminate explicitly and preserve prior context.

**Independent Test**: Run one task with a retryable failure and one task with a
replanning trigger; confirm retries and replans stay within configured budgets, retain
history, and end in success or exhausted failure with no silent loops.

### Tests for User Story 2

- [X] T018 [P] [US2] Implement endpoint recoverability contract coverage in tests/contract/endpoint_execution.rs
- [X] T019 [P] [US2] Implement retry-budget and recoverability unit coverage in tests/unit/recovery_policy.rs
- [X] T020 [P] [US2] Implement retry, replanning, and exhaustion scenarios in tests/integration/retry_and_replan.rs

### Implementation for User Story 2

- [X] T021 [US2] Extend recovery budget accounting and retry decisions in src/orchestrator/recovery.rs and src/domain/limits.rs
- [X] T022 [US2] Implement plan revision, step replacement, and exhaustion termination handling in src/orchestrator/planner.rs, src/domain/plan.rs, and src/orchestrator/terminal.rs
- [X] T023 [US2] Propagate failure metadata, recoverability, and attempt history through src/orchestrator/engine.rs, src/domain/step.rs, and src/domain/task_context.rs
- [X] T024 [US2] Make the US2 recovery contract and integration scenarios pass in tests/contract/endpoint_execution.rs and tests/integration/retry_and_replan.rs

**Checkpoint**: User Stories 1 and 2 both work, and recovery behavior is bounded and inspectable.

---

## Phase 5: User Story 3 - Inspect Execution History (Priority: P3)

**Goal**: Persist and expose execution traces that let developers reconstruct step
order, retries, replans, and final outcome without relying on live process state.

**Independent Test**: Run one successful task and one failed task, then inspect the
persisted trace output and confirm it reconstructs step order, retry or replanning
events, and the terminal reason from the stored record alone.

### Tests for User Story 3

- [X] T025 [P] [US3] Implement trace record contract coverage in tests/contract/trace_record.rs
- [X] T026 [P] [US3] Implement persisted trace inspection scenarios in tests/integration/trace_capture.rs

### Implementation for User Story 3

- [X] T027 [US3] Implement trace metadata, event serialization, and terminal record fields in src/domain/trace.rs and src/adapters/trace_store.rs
- [X] T028 [US3] Implement step, retry, replanning, and terminal trace emission in src/orchestrator/engine.rs, src/orchestrator/recovery.rs, and src/orchestrator/terminal.rs
- [X] T029 [US3] Expose trace location and inspection-friendly outputs in src/lib.rs and src/adapters/trace_store.rs
- [X] T030 [US3] Make the US3 contract and integration scenarios pass in tests/contract/trace_record.rs and tests/integration/trace_capture.rs

**Checkpoint**: All user stories are functional, and trace inspection explains both successful and failed runs.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Tighten developer workflow, public API clarity, and full-slice validation across stories.

- [X] T031 [P] Update developer-facing usage and validation steps in README.md and specs/001-delivery-orchestrator-core/quickstart.md
- [X] T032 Clean up public API naming and trace or error diagnostics in src/lib.rs, src/orchestrator/engine.rs, src/orchestrator/recovery.rs, and src/orchestrator/terminal.rs
- [X] T033 [P] Add final smoke coverage for end-to-end success, recovery, and trace inspection in tests/integration/sequential_task_run.rs, tests/integration/retry_and_replan.rs, and tests/integration/trace_capture.rs
- [X] T034 Validate quickstart commands against Cargo.toml, README.md, and specs/001-delivery-orchestrator-core/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies; start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user story work.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP execution loop.
- **Phase 4: User Story 2**: Depends on Phase 3 because recovery extends the primary engine and planner flow.
- **Phase 5: User Story 3**: Depends on Phases 3 and 4 because trace inspection must reflect both baseline execution and recovery behavior.
- **Phase 6: Polish**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: No story dependency after Foundational.
- **US2 (P2)**: Depends on US1 engine and planner behavior.
- **US3 (P3)**: Depends on US1 trace basics and US2 recovery-event coverage.

### Within Each User Story

- Validation tasks MUST fail before implementation changes are considered complete.
- Domain and contract updates come before engine or adapter wiring.
- Engine or recovery logic comes before final integration verification.
- Story-specific pass tasks mark the point where the slice is independently testable.

### Parallel Opportunities

- Setup: T002 and T003 can proceed in parallel after T001.
- Foundational: T005, T006, and T007 can proceed in parallel after T004 begins establishing shared types.
- US1: T010, T011, and T012 can proceed in parallel.
- US2: T018, T019, and T020 can proceed in parallel.
- US3: T025 and T026 can proceed in parallel.
- Polish: T031 and T033 can proceed in parallel after story completion.

---

## Parallel Example: User Story 1

```bash
# Launch US1 validation tasks together:
Task: "Implement task-run contract coverage in tests/contract/orchestrator_run.rs"
Task: "Implement unit coverage in tests/unit/step_state.rs and tests/unit/terminal_precedence.rs"
Task: "Implement three-step bounded execution scenarios in tests/integration/sequential_task_run.rs"

# Launch independent foundational work together before engine wiring:
Task: "Implement step, plan, and runtime limit models in src/domain/step.rs, src/domain/plan.rs, and src/domain/limits.rs"
Task: "Implement agent/tool adapter traits and registries in src/adapters/agent.rs, src/adapters/tool.rs, src/registry/agent_registry.rs, and src/registry/tool_registry.rs"
Task: "Implement trace entities and file-backed trace persistence in src/domain/trace.rs and src/adapters/trace_store.rs"
```

## Parallel Example: User Story 2

```bash
# Launch US2 validation tasks together:
Task: "Implement endpoint recoverability contract coverage in tests/contract/endpoint_execution.rs"
Task: "Implement retry-budget unit coverage in tests/unit/recovery_policy.rs"
Task: "Implement retry, replanning, and exhaustion scenarios in tests/integration/retry_and_replan.rs"
```

## Parallel Example: User Story 3

```bash
# Launch US3 validation tasks together:
Task: "Implement trace record contract coverage in tests/contract/trace_record.rs"
Task: "Implement persisted trace inspection scenarios in tests/integration/trace_capture.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate `tests/contract/orchestrator_run.rs` and `tests/integration/sequential_task_run.rs`.
5. Demo the bounded sequential orchestrator before expanding scope.

### Incremental Delivery

1. Finish Setup and Foundational to create the crate, domain model, registries, and trace base.
2. Ship US1 to prove bounded execution.
3. Ship US2 to prove bounded recovery and replanning.
4. Ship US3 to prove persisted inspection.
5. Use Phase 6 to tighten docs, naming, and smoke coverage without changing scope.

### Parallel Team Strategy

1. One engineer can own Phase 1 and phase wiring.
2. A second engineer can take T005 and T007 while the first completes T006 and T008.
3. After US1 is stable, recovery work and trace-inspection work can overlap only where file ownership does not collide.

---

## Notes

- Total tasks: 34.
- User story task counts: US1 = 8, US2 = 7, US3 = 6.
- Suggested MVP scope: through Phase 3 (User Story 1) only.
- All tasks use the required checklist format: checkbox, task ID, optional `[P]`, required story label in story phases, and exact file paths.