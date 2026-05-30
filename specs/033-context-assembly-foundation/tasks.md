# Tasks: Context Assembly Foundation

**Input**: Design documents from `/specs/033-context-assembly-foundation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes goal planning, explicit failure handling, trace payloads, and operator-facing CLI summaries.

**Organization**: Tasks are grouped by user story so each story remains independently testable while still delivering one complete macrofeature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. [US1], [US2])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the 033 feature pack and validation surfaces.

- [x] T001 Confirm 033 feature artifacts and update `specs/033-context-assembly-foundation/plan.md`, `specs/033-context-assembly-foundation/research.md`, `specs/033-context-assembly-foundation/data-model.md`, `specs/033-context-assembly-foundation/contracts/`, and `specs/033-context-assembly-foundation/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references if new 033 test files require entries in `tests/unit.rs`, `tests/contract.rs`, or `tests/integration.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the core context-pack model and planner plumbing needed by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `src/domain/goal_plan.rs` with context-pack, context-input, and credibility-state models plus serde validation
- [x] T004 [P] Extend `src/orchestrator/goal_planner.rs` with bounded context-assembly helpers for workspace files, authored inputs, recent traces, and Canon artifacts
- [x] T005 [P] Extend `tests/unit/goal_plan_model.rs` and `tests/unit/goal_planner.rs` with model and planner coverage for the new context-pack primitives

**Checkpoint**: The goal planner can create a context pack and the core model is validated.

---

## Phase 3: User Story 1 - Build A Bounded Context Pack Before Planning (Priority: P1) 🎯 MVP

**Goal**: Make plan creation derive one explicit bounded context pack before confirming a goal plan.

**Independent Test**: Run planning on a representative workspace and verify the goal plan contains explicit context inputs, provenance, and credibility state.

### Tests for User Story 1

- [x] T006 [P] [US1] Add contract coverage for context-pack creation in `tests/contract/goal_plan_contract.rs`
- [x] T007 [P] [US1] Add integration coverage for session-native planning with explicit context assembly in `tests/integration/session_native_flow.rs`
- [x] T008 [P] [US1] Add unit coverage for session-runtime planning inputs in `tests/unit/session_model.rs` or `tests/unit/runtime_routing.rs`

### Implementation for User Story 1

- [x] T009 [US1] Update `src/orchestrator/session_runtime.rs` to pass authored brief, negotiated delivery, and trace state into goal planning
- [x] T010 [US1] Update `src/orchestrator/goal_planner.rs` so planned task targets and evidence derive from the assembled context pack instead of coarse ambient defaults alone
- [x] T011 [US1] Update `src/orchestrator/decision_loop.rs` and `src/domain/trace.rs` so `GoalPlanCreated` persists context-pack state and provenance

**Checkpoint**: Planning creates a credible bounded context pack and traces it.

---

## Phase 4: User Story 2 - Inspect Context Narrowing On The Primary Boundline Path (Priority: P2)

**Goal**: Surface context-pack summaries and provenance through the main CLI path.

**Independent Test**: After planning or running, `status`, `next`, and `inspect` reveal context summary, credibility, and primary inputs without reading raw trace JSON.

### Tests for User Story 2

- [x] T012 [P] [US2] Add unit coverage for context-pack rendering in `tests/unit/cli_output.rs`
- [x] T013 [P] [US2] Add contract coverage for inspect projection in `tests/contract/trace_summary_contract.rs`
- [x] T014 [P] [US2] Add integration or session projection coverage in `tests/unit/workflow_session_projection.rs` and `tests/unit/session_record.rs`

### Implementation for User Story 2

- [x] T015 [US2] Extend `src/domain/session.rs` with context-pack projection fields used by session-native status surfaces
- [x] T016 [US2] Extend `src/cli/output.rs` to render context summary, credibility, primary inputs, and provenance on plan, run, status, next, and inspect outputs
- [x] T017 [US2] Extend `src/cli/inspect.rs` to recover context-pack data from goal-plan and trace payloads while preserving explicit compatibility authority

**Checkpoint**: Operators can inspect bounded context from normal CLI surfaces.

---

## Phase 5: User Story 3 - Stop Explicitly When Credible Context Cannot Be Built (Priority: P3)

**Goal**: Make context-credibility failure a first-class non-success path.

**Independent Test**: A workspace without credible bounded inputs stops planning explicitly and surfaces the bounded recovery action.

### Tests for User Story 3

- [x] T018 [P] [US3] Add unit coverage for insufficient or stale context packs in `tests/unit/goal_planner.rs`
- [x] T019 [P] [US3] Add contract coverage for failure projection in `tests/contract/runtime_refoundation_contract.rs` or `tests/contract/goal_plan_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for blocked planning or follow-through in `tests/integration/runtime_refoundation_failure.rs`

### Implementation for User Story 3

- [x] T021 [US3] Add explicit insufficient and stale credibility handling in `src/orchestrator/goal_planner.rs` and `src/orchestrator/session_runtime.rs`
- [x] T022 [US3] Extend `src/domain/follow_through.rs` and related session or trace projections to surface bounded recovery guidance when context is not credible
- [x] T023 [US3] Ensure `src/cli/output.rs` and `src/cli/inspect.rs` surface explicit context-credibility failures instead of generic planning errors

**Checkpoint**: Non-credible context blocks planning and follow-through explicitly.

---

## Phase 6: User Story 4 - Ship Context Assembly As 0.33.0 (Priority: P4)

**Goal**: Close the feature as a release-aligned macrofeature with updated product narrative and validation.

**Independent Test**: Runtime behavior, roadmap, docs, version metadata, and validation evidence all align with `0.33.0`.

### Tests for User Story 4

- [x] T024 [P] [US4] Refresh focused coverage assertions for touched Rust files via `lcov.info` and supporting validation commands

### Implementation for User Story 4

- [x] T025 [US4] Bump crate version to `0.33.0` in `Cargo.toml` and `Cargo.lock`
- [x] T026 [US4] Update impacted docs and release narrative in `README.md`, `docs/getting-started.md`, `docs/configuration.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, and `AGENTS.md`
- [x] T027 [US4] Update `ROADMAP.md` to mark Spec 033 as delivered and remove it from the remaining future macrofeature line
- [x] T028 [US4] Update assistant or quickstart guidance impacted by the new context-pack story in `assistant/README.md` and `specs/033-context-assembly-foundation/quickstart.md`

**Checkpoint**: Release artifacts describe `0.33.0` consistently.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close remaining quality gaps.

- [x] T029 [P] Run formatting with `cargo fmt --all`
- [x] T030 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T031 Run targeted and broader Rust validation for the slice with `cargo test --no-run --all-targets` and selected `cargo nextest run` coverage
- [x] T032 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T033 Mark completed tasks in `specs/033-context-assembly-foundation/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 context-pack data model and trace payloads.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because failure handling reuses the same context-pack projection vocabulary.
- **User Story 4 (Phase 6)**: Depends on all runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if no harness update is needed.
- T004 and T005 can run in parallel after T003 defines the core model.
- Within each user story, test tasks marked `[P]` can be developed in parallel before implementation tasks touching the same files.
- T024 can be prepared while release docs are being updated, but final coverage confirmation must wait for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that planning creates a credible context pack.
3. Use that as the base for output projection and failure handling.

### Incremental Delivery

1. Add context-pack models and builder.
2. Project the new state through session and inspect surfaces.
3. Add explicit failure handling for insufficient or stale context.
4. Close the release with version, docs, roadmap, coverage, clippy, and fmt.

## Notes

- This feature is intentionally macro-level, but the implementation still stays bounded to existing goal-plan, trace, session, and CLI surfaces.
- Compatibility drift with later Canon versions remains maintenance unless the wire contract breaks.
- The final summary must include a descriptive commit message for the completed feature.
