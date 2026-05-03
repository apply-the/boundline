# Tasks: Native Loop Integration

**Input**: Design documents from `/specs/014-native-loop-integration/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for each story because this feature changes persisted session state, routing behavior, adapter execution, and CLI inspection.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Phase 1: Foundational

**Purpose**: Align session validation and routing primitives with native planning

- [x] T001 Update native-planned session validation in `src/domain/session.rs` so `SessionStatus::Planned` can be satisfied by persisted `goal_plan` without requiring `active_task`
- [x] T002 [P] Add focused unit coverage for native planned-session validation and status view behavior in `tests/unit/session_model.rs`
- [x] T003 [P] Introduce explicit native-vs-compat routing helpers in `src/orchestrator/session_runtime.rs` and cover them in `tests/unit/session_runtime.rs`

**Checkpoint**: Session state can represent a native planned run without silent fixture assumptions.

---

## Phase 2: User Story 1 - Session Planning Uses Goal Plan (Priority: P1) 🎯 MVP

**Goal**: `plan` persists `GoalPlan` and flow-confirmation outcome in session-owned state.

**Independent Test**: Drive `start -> capture -> plan` on the real CLI path and verify the session stores a goal plan plus confirmed/proposed/no-flow outcome without requiring an execution profile.

### Tests for User Story 1

- [x] T004 [P] [US1] Add CLI integration coverage for `start -> capture -> plan` goal-plan persistence in `tests/integration/session_native_flow.rs`
- [x] T005 [P] [US1] Add CLI integration coverage for inferred-flow confirmation and explicit no-flow behavior in `tests/integration/session_native_flow.rs`

### Implementation for User Story 1

- [x] T006 [US1] Extend `boundline plan` argument parsing in `src/cli.rs` and `src/cli/session.rs` to support lightweight flow confirmation inputs for the native path
- [x] T007 [US1] Wire `infer_flow` and goal-plan persistence into planning in `src/cli/session.rs` and `src/orchestrator/session_runtime.rs`
- [x] T008 [US1] Persist confirmed flow policy or explicit no-flow outcome in `src/orchestrator/session_runtime.rs` and `src/domain/session.rs`
- [x] T009 [US1] Update planning output surfaces in `src/cli/output.rs` and `src/cli/session.rs` so the operator can see confirmed, proposed, or absent flow state

**Checkpoint**: Planning produces resumable native session state centered on `goal_plan`.

---

## Phase 3: User Story 2 - Session Run Uses Decision Loop By Default (Priority: P2)

**Goal**: `run` prefers the native loop whenever a goal plan exists and uses fixture execution only as an explicit compatibility path.

**Independent Test**: Verify CLI `start -> capture -> plan -> run` selects native routing, while a workspace with only an explicit execution profile uses compatibility routing.

### Tests for User Story 2

- [x] T010 [P] [US2] Add integration coverage for native routing precedence in `tests/integration/session_native_flow.rs`
- [x] T011 [P] [US2] Add integration coverage for explicit compatibility routing in `tests/integration/fixture_compat_flow.rs`

### Implementation for User Story 2

- [x] T012 [US2] Route `boundline run` through native decision-loop selection in `src/cli/session.rs`
- [x] T013 [US2] Implement native/compat/blocked routing behavior and remediation messages in `src/orchestrator/session_runtime.rs`
- [x] T014 [US2] Keep fixture runtime assembly explicit in `src/orchestrator/session_runtime.rs` and `src/fixture.rs` without treating it as the implicit default for planned work
- [x] T015 [US2] Extend inspect or status-facing output in `src/cli/session.rs` and `src/cli/output.rs` so route choice is visible to the operator

**Checkpoint**: Native planning now changes real run behavior on the primary CLI path.

---

## Phase 4: User Story 3 - Real Adapter-Backed Decisions Are Persisted (Priority: P3)

**Goal**: Decision-loop execution uses real adapters, persists decisions, and remains inspectable end-to-end.

**Independent Test**: Execute the real CLI path on a bounded workspace that requires reads, validation, and recovery visibility; then verify persisted decisions and structured execution results in session state and traces.

### Tests for User Story 3

- [x] T016 [P] [US3] Add contract coverage for adapter-backed decision dispatch and persistence in `tests/contract/decision_loop_contract.rs`
- [x] T017 [P] [US3] Add end-to-end CLI coverage for persisted decisions and inspectable tool results in `tests/integration/session_native_flow.rs`
- [x] T018 [P] [US3] Add unit coverage for registry-backed dispatch mapping and persistence in `tests/unit/decision_loop.rs`

### Implementation for User Story 3

- [x] T019 [US3] Replace synthetic dispatch in `src/orchestrator/decision_loop.rs` with registry-backed adapter execution using `StepExecutionRequest` and `StepExecutionResult`
- [x] T020 [US3] Provide native adapter registration and concrete file or command execution primitives in `src/orchestrator/session_runtime.rs`, `src/adapters/tool.rs`, and related helpers
- [x] T021 [US3] Persist decision history and terminal evidence into session state and trace output in `src/orchestrator/decision_loop.rs` and `src/orchestrator/session_runtime.rs`
- [x] T022 [US3] Surface persisted decision summaries in `src/cli/session.rs` and `src/cli/output.rs`

**Checkpoint**: The session-native loop is no longer synthetic and is inspectable from the real CLI path.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and documentation alignment

- [x] T023 [P] Update operator-facing documentation in `README.md` and `docs/session-native-orchestrator-review.md` to reflect native routing and explicit compatibility behavior
- [x] T024 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, and `cargo nextest run --workspace --all-features`

---

## Dependencies & Execution Order

- T001-T003 must complete before user-story work.
- US1 depends on foundational tasks.
- US2 depends on US1 because routing uses persisted native planning state.
- US3 depends on US2 because persisted decisions are only meaningful on the real routed path.
- Polish runs after the desired stories are complete.

## Parallel Opportunities

- T002 and T003 can run in parallel after T001.
- T004 and T005 can run in parallel.
- T010 and T011 can run in parallel.
- T016, T017, and T018 can run in parallel.
- T023 can run in parallel with validation once implementation is stable.

## Implementation Strategy

1. Establish native session validity and explicit routing primitives.
2. Ship the planning slice so `goal_plan` and flow-confirmation state become durable.
3. Switch the primary `run` path to native routing.
4. Replace synthetic dispatch and finish with end-to-end validation.
