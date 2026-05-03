# Tasks: Bounded Delegated Execution

**Input**: Design documents from `/specs/037-bounded-delegation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes routing authority, session continuity, operator-facing follow-through summaries, and persisted trace semantics.

**Organization**: Tasks are grouped by user story so each slice remains independently testable while still delivering one complete macrofeature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Finalize the 037 feature pack and confirm test-harness entry points before runtime changes.

- [x] T001 Confirm and keep synchronized `/Users/rt/workspace/boundline/specs/037-bounded-delegation/plan.md`, `/Users/rt/workspace/boundline/specs/037-bounded-delegation/research.md`, `/Users/rt/workspace/boundline/specs/037-bounded-delegation/data-model.md`, `/Users/rt/workspace/boundline/specs/037-bounded-delegation/contracts/`, and `/Users/rt/workspace/boundline/specs/037-bounded-delegation/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references in `/Users/rt/workspace/boundline/tests/unit.rs`, `/Users/rt/workspace/boundline/tests/contract.rs`, and `/Users/rt/workspace/boundline/tests/integration.rs` if 037 introduces new test modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared routing-policy, delegation-packet, and continuity primitives used by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `/Users/rt/workspace/boundline/src/domain/configuration.rs` and `/Users/rt/workspace/boundline/src/domain/routing_decision.rs` with runtime capability profiles, slot effort policy, effective projection, and validation rules shared by planning and execution
- [x] T004 [P] Extend `/Users/rt/workspace/boundline/src/domain/task_context.rs`, `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/follow_through.rs`, and `/Users/rt/workspace/boundline/src/domain/trace.rs` with delegation packets, continuity state, stuck evidence markers, and packet lifecycle projection fields
- [x] T005 [P] Extend `/Users/rt/workspace/boundline/src/domain/goal_plan.rs` and `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs` so planning can persist capability-aware routing rationale and delegation-aware bounded stop summaries
- [x] T006 [P] Extend `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs`, `/Users/rt/workspace/boundline/tests/unit/session_model.rs`, and `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs` with foundational coverage for capability validation, effort-policy projection, packet invariants, and continuity state transitions

**Checkpoint**: Runtime policy, delegation state, and continuity projection primitives exist and can support all user stories.

---

## Phase 3: User Story 1 - Route Work Through Declared Runtime Capabilities (Priority: P1) 🎯 MVP

**Goal**: Make `config show`, `plan`, and `run` respect explicit runtime capability and effort policy before Boundline attempts a blocked route.

**Independent Test**: Configure capability and effort policy for routed slots, then verify that planning and direct native execution expose which rule changed the chosen bounded path.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for capability and effort projection in `/Users/rt/workspace/boundline/tests/contract/config_cli_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for capability-aware configuration and planning in `/Users/rt/workspace/boundline/tests/integration/config_workspace_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for effective routing, effort fallback, and capability-aware plan shaping in `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs` and `/Users/rt/workspace/boundline/tests/unit/session_model.rs`

### Implementation for User Story 1

- [x] T010 [US1] Extend `/Users/rt/workspace/boundline/src/cli.rs` and `/Users/rt/workspace/boundline/src/domain/configuration.rs` so operators can declare and unset runtime capability profiles and slot effort policy through the existing config surface
- [x] T011 [US1] Extend `/Users/rt/workspace/boundline/src/domain/routing_decision.rs`, `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` so route selection and plan rationale record capability and effort evidence explicitly
- [x] T012 [US1] Extend `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/run.rs` so `config show`, `plan`, and `run` surface the effective capability and effort policy that shaped the next bounded action

**Checkpoint**: Operators can see and influence route selection through declared capability and effort policy.

---

## Phase 4: User Story 2 - Persist Handoff And Escalation Packets (Priority: P2)

**Goal**: Turn blocked continuity into explicit handoff or escalation packets that persist in authoritative session state.

**Independent Test**: Run blocked native execution scenarios and verify that Boundline persists one active packet with decisive evidence, target owner, and next command instead of returning an opaque failure.

### Tests for User Story 2

- [x] T013 [P] [US2] Add contract coverage for packet creation and continuity authority in `/Users/rt/workspace/boundline/tests/contract/session_command_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/compatibility_continuity_contract.rs`
- [x] T014 [P] [US2] Add integration coverage for handoff and escalation packet creation in `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/session_compatibility_continuity.rs`
- [x] T015 [P] [US2] Add unit coverage for packet lifecycle, supersession, and continuity projection in `/Users/rt/workspace/boundline/tests/unit/session_model.rs` and `/Users/rt/workspace/boundline/tests/unit/decision_model.rs`

### Implementation for User Story 2

- [x] T016 [US2] Extend `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`, `/Users/rt/workspace/boundline/src/domain/task_context.rs`, and `/Users/rt/workspace/boundline/src/domain/session.rs` so blocked native runs persist active handoff or escalation packets and expose them as authoritative continuity state
- [x] T017 [US2] Extend `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`, `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` so unresolved packets block or redirect planning and follow-through explicitly
- [x] T018 [US2] Extend `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/inspect.rs`, and `/Users/rt/workspace/boundline/src/domain/trace.rs` so packet history, target owner, and next command project consistently on native and compatibility-aware surfaces

**Checkpoint**: Blocked continuity is represented by authoritative packets, not by ad hoc operator interpretation.

---

## Phase 5: User Story 3 - Detect Stuck Delegation And Preserve Recovery (Priority: P3)

**Goal**: Detect when delegated continuity is repeating without new evidence, then stop or redirect with an explicit stuck verdict and recovery recommendation.

**Independent Test**: Repeat the same blocked continuation path until the configured threshold is reached and verify that Boundline reports a stuck continuity state instead of repeating the same blocked action silently.

### Tests for User Story 3

- [x] T019 [P] [US3] Add contract coverage for stuck continuity and delegated follow-through summaries in `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/session_command_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for repeated-block detection and packet supersession in `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_flow.rs`, `/Users/rt/workspace/boundline/tests/integration/retry_and_replan.rs`, and `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T021 [P] [US3] Add unit coverage for stuck evidence aggregation and recovery guidance in `/Users/rt/workspace/boundline/tests/unit/decision_loop.rs`, `/Users/rt/workspace/boundline/tests/unit/decision_model.rs`, and `/Users/rt/workspace/boundline/tests/unit/session_model.rs`

### Implementation for User Story 3

- [x] T022 [US3] Extend `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs` and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` so repeated blocked attempts, unchanged evidence, or stale route declarations produce stuck markers and bounded recovery guidance
- [x] T023 [US3] Extend `/Users/rt/workspace/boundline/src/domain/follow_through.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/domain/session.rs` so active, stuck, resolved, superseded, and exhausted continuity states remain authoritative and inspectable
- [x] T024 [US3] Extend `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/inspect.rs`, and `/Users/rt/workspace/boundline/src/cli/session.rs` so `run`, `status`, `next`, and `inspect` render stuck reasoning, packet supersession, and recommended recovery or continuation commands consistently

**Checkpoint**: Delegated continuity stops or recovers explicitly when it stops making progress.

---

## Phase 6: User Story 4 - Ship Delegated Execution As 0.37.0 (Priority: P4)

**Goal**: Close the feature as a release-aligned macrofeature with updated version metadata, roadmap activation and closure, docs, assistant guidance, and validation evidence.

**Independent Test**: Follow the updated docs on a representative workspace, run the release validation suite, and verify that the version bump, roadmap, docs, changelog, formatting, linting, and coverage all align with delegated execution.

### Tests for User Story 4

- [x] T025 [P] [US4] Refresh focused coverage assertions for touched Rust files via `/Users/rt/workspace/boundline/lcov.info` and supporting validation commands

### Implementation for User Story 4

- [x] T026 [US4] Bump crate version to `0.37.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [x] T027 [US4] Update impacted docs and release narrative in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/CHANGELOG.md`, and `/Users/rt/workspace/boundline/AGENTS.md`
- [x] T028 [US4] Update `/Users/rt/workspace/boundline/ROADMAP.md` to activate and then mark Spec 037 as delivered on the next macrofeature line
- [x] T029 [US4] Update assistant guidance impacted by delegated execution in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/`, `/Users/rt/workspace/boundline/assistant/codex/commands/`, and `/Users/rt/workspace/boundline/assistant/copilot/prompts/`

**Checkpoint**: Release artifacts describe `0.37.0` and delegated execution consistently.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close remaining quality gaps.

- [x] T030 [P] Run formatting with `cargo fmt --all`
- [x] T031 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run compile-oriented and broader Rust validation for the slice with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T033 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T034 Mark completed tasks in `/Users/rt/workspace/boundline/specs/037-bounded-delegation/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because packet creation reuses the capability and effort-policy model already made authoritative.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because stuck detection and supersession rely on active packet and continuity state.
- **User Story 4 (Phase 6)**: Depends on all runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if new test modules are needed.
- T004, T005, and T006 can run in parallel after T003 defines the routing-policy primitives.
- Within each user story, test tasks marked `[P]` can be developed in parallel before the implementation tasks touching the same files.
- T025 can be prepared while release docs are being updated, but final coverage confirmation must wait for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that `config show`, `plan`, and `run` explain capability-aware route selection explicitly.
3. Use that route-policy model as the base for authoritative handoff and escalation packets.

### Incremental Delivery

1. Add runtime capability and effort-policy primitives.
2. Make planning and execution consume those declarations before blocked route attempts.
3. Persist handoff and escalation packets in session-owned continuity state.
4. Add evidence-based stuck detection, supersession, and delegated follow-through rendering.
5. Close the release with version, docs, roadmap, assistant guidance, coverage, linting, and formatting.

## Notes

- This feature stays macro-level in value but bounded in implementation: it strengthens the existing session-owned runtime instead of importing tmux, inbox, or background orchestration semantics.
- Explicit compatibility continuity remains outside the primary scope except where it must preserve authoritative route explanation using the same delegated vocabulary.
- The final summary must include a descriptive commit message for the completed feature.