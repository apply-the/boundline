# Tasks: Decision-Driven Orchestrator

**Input**: Design documents from `/specs/034-decision-driven-orchestrator/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes the native execution loop, decision recovery behavior, operator-facing CLI summaries, and persisted trace semantics.

**Organization**: Tasks are grouped by user story so each story remains independently testable while still delivering one complete macrofeature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the 034 feature pack and validation surfaces.

- [x] T001 Confirm 034 feature artifacts and update `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/plan.md`, `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/research.md`, `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/data-model.md`, `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/contracts/`, and `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references if new 034 test files require entries in `/Users/rt/workspace/boundline/tests/unit.rs`, `/Users/rt/workspace/boundline/tests/contract.rs`, or `/Users/rt/workspace/boundline/tests/integration.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the selector-driven decision primitives and shared projection fields needed by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `/Users/rt/workspace/boundline/src/domain/decision.rs` with explicit selector primitives, selector rationale, evidence-basis, verification-intent, and clarification-aware decision validation
- [x] T004 [P] Extend `/Users/rt/workspace/boundline/src/orchestrator/recovery.rs` and `/Users/rt/workspace/boundline/src/orchestrator/terminal.rs` with selector-aware retry, ask, replan, and stop-precedence primitives
- [x] T005 [P] Extend `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` with selector-driven projection fields shared by runtime and CLI surfaces
- [x] T006 [P] Extend `/Users/rt/workspace/boundline/tests/unit/decision_model.rs`, `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs`, and `/Users/rt/workspace/boundline/tests/unit/session_model.rs` with foundational coverage for selector primitives and projection invariants

**Checkpoint**: Selector primitives, recovery rules, and projection fields exist and can support all user stories.

---

## Phase 3: User Story 1 - Select The Next Bounded Action From Decision State (Priority: P1) 🎯 MVP

**Goal**: Make the native `DecisionLoop` select explicit `read/search/modify/test/ask/replan` actions from current evidence instead of replaying a mostly static task order.

**Independent Test**: Run a representative native goal-plan task and verify the loop chooses evidence-gathering, change, and verification selectors from decision state rather than from a fixed pre-shaped sequence.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for selector-driven decision payloads in `/Users/rt/workspace/boundline/tests/contract/decision_loop_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for selector ordering on the native path in `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for selector choice and evidence-first ordering in `/Users/rt/workspace/boundline/tests/unit/decision_loop.rs`

### Implementation for User Story 1

- [x] T010 [US1] Extend `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs` so observation state and decision rules choose explicit selectors from current evidence, decision history, and goal-plan targets
- [x] T011 [US1] Extend `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` native adapter registries and dispatch helpers so selector-driven actions execute through read/search/modify/test/ask/replan semantics on the native path
- [x] T012 [US1] Extend `/Users/rt/workspace/boundline/src/domain/decision.rs` and `/Users/rt/workspace/boundline/src/domain/trace.rs` so selector rationale, evidence basis, verification intent, and recovery linkage persist in decision and trace payloads

**Checkpoint**: The native loop now selects and executes one explicit bounded selector per iteration.

---

## Phase 4: User Story 2 - Inspect Decision-Driven Execution On Existing Surfaces (Priority: P2)

**Goal**: Surface selector-driven state, rationale, evidence, and verification or stop intent through the existing read-side surfaces without raw trace inspection.

**Independent Test**: After a native run, `run`, `status`, `next`, and `inspect` must explain the current selector, why it was chosen, and what will verify or stop it.

### Tests for User Story 2

- [x] T013 [P] [US2] Add contract coverage for selector projection on trace summaries in `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs`
- [x] T014 [P] [US2] Add integration coverage for selector-driven `status`, `next`, and `inspect` output in `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T015 [P] [US2] Add unit coverage for selector rendering and follow-through projection in `/Users/rt/workspace/boundline/tests/unit/cli_output.rs`, `/Users/rt/workspace/boundline/tests/unit/session_record.rs`, and `/Users/rt/workspace/boundline/tests/unit/workflow_session_projection.rs`

### Implementation for User Story 2

- [x] T016 [US2] Extend `/Users/rt/workspace/boundline/src/cli/inspect.rs` and `/Users/rt/workspace/boundline/src/domain/trace.rs` so authoritative traces summarize selector kind, selector rationale, evidence basis, verification intent, and recovery state
- [x] T017 [US2] Extend `/Users/rt/workspace/boundline/src/cli/output.rs` and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` to render selector-driven headlines, guidance, and stop reasons on run, status, next, and inspect output
- [x] T018 [US2] Extend `/Users/rt/workspace/boundline/src/cli/session.rs` and `/Users/rt/workspace/boundline/src/domain/session.rs` to surface selector-driven session state while preserving explicit compatibility authority and next-command continuity

**Checkpoint**: Operators can understand decision-driven state from normal CLI surfaces.

---

## Phase 5: User Story 3 - Stop, Ask, Or Replan Explicitly When No Credible Action Exists (Priority: P3)

**Goal**: Make ask, retry, replan, exhaustion, and terminal stop authoritative from decision state rather than generic lifecycle fallbacks.

**Independent Test**: Run bounded tasks that lack sufficient evidence or keep failing validation and verify that Boundline surfaces explicit ask/replan/terminal outcomes from decision state.

### Tests for User Story 3

- [x] T019 [P] [US3] Add contract coverage for selector-aware recovery and stop behavior in `/Users/rt/workspace/boundline/tests/contract/runtime_refoundation_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for explicit ask, replan, and exhaustion behavior in `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_failure.rs` and `/Users/rt/workspace/boundline/tests/integration/retry_and_replan.rs`
- [x] T021 [P] [US3] Add unit coverage for selector-aware recovery, clarification, and terminal precedence in `/Users/rt/workspace/boundline/tests/unit/decision_loop.rs` and `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs`

### Implementation for User Story 3

- [x] T022 [US3] Extend `/Users/rt/workspace/boundline/src/orchestrator/recovery.rs` and `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs` to derive retry, ask, replan, and terminal outcomes from decision state and verification evidence
- [x] T023 [US3] Extend `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` to persist clarification-style ask state and bounded next-command guidance
- [x] T024 [US3] Ensure `/Users/rt/workspace/boundline/src/cli/output.rs` and `/Users/rt/workspace/boundline/src/cli/inspect.rs` surface selector-driven ask, stop, and exhaustion reasons instead of generic lifecycle fallbacks

**Checkpoint**: Non-success decision-driven behavior is explicit, bounded, and inspectable.

---

## Phase 6: User Story 4 - Ship The Decision-Driven Runtime As 0.34.0 (Priority: P4)

**Goal**: Close the feature as a release-aligned macrofeature with updated product narrative, version metadata, roadmap closure, and validation evidence.

**Independent Test**: Runtime behavior, roadmap, docs, version metadata, and repository validation all align with the shipped `0.34.0` decision-driven model.

### Tests for User Story 4

- [x] T025 [P] [US4] Refresh focused coverage assertions for touched Rust files via `/Users/rt/workspace/boundline/lcov.info` and supporting validation commands

### Implementation for User Story 4

- [x] T026 [US4] Bump crate version to `0.34.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [x] T027 [US4] Update impacted docs and release narrative in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/getting-started.md`, `/Users/rt/workspace/boundline/docs/configuration.md`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/CHANGELOG.md`, and `/Users/rt/workspace/boundline/AGENTS.md`
- [x] T028 [US4] Update `/Users/rt/workspace/boundline/ROADMAP.md` to mark Spec 034 as delivered and remove it from the remaining future macrofeature line
- [x] T029 [US4] Update assistant guidance impacted by selector-driven execution in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-status.prompt.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-next.prompt.md`, `/Users/rt/workspace/boundline/assistant/gemini/README.md`, and `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/quickstart.md`

**Checkpoint**: Release artifacts describe `0.34.0` consistently.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close remaining quality gaps.

- [x] T030 [P] Run formatting with `cargo fmt --all`
- [x] T031 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run compile-oriented and broader Rust validation for the slice with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T033 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T034 Mark completed tasks in `/Users/rt/workspace/boundline/specs/034-decision-driven-orchestrator/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 selector state and trace payloads.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because ask/replan/stop behavior reuses the same selector-driven decision and projection vocabulary.
- **User Story 4 (Phase 6)**: Depends on all runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if no harness update is needed.
- T004, T005, and T006 can run in parallel after T003 defines the selector primitives.
- Within each user story, test tasks marked `[P]` can be developed in parallel before implementation tasks touching the same files.
- T025 can be prepared while release docs are being updated, but final coverage confirmation must wait for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that native execution chooses explicit selectors from decision state.
3. Use that as the base for projection and explicit recovery behavior.

### Incremental Delivery

1. Add selector primitives and selector-aware recovery rules.
2. Make the native loop choose and persist explicit selector-driven actions.
3. Project selector state through session and inspect surfaces.
4. Add explicit ask/replan/stop behavior from decision state.
5. Close the release with version, docs, roadmap, coverage, linting, and formatting.

## Notes

- This feature stays macro-level in value but bounded in implementation: it strengthens the existing native decision loop instead of introducing a second runtime.
- Compatibility maintenance for newer Canon releases remains outside this slice unless the wire contract breaks.
- The final summary must include a descriptive commit message for the completed feature.