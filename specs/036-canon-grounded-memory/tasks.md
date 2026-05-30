# Tasks: Canon-Grounded Reasoning And Structured Memory

**Input**: Design documents from `/specs/036-canon-grounded-memory/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes Canon-grounded planning authority, compact memory reuse across loops, runtime routing, CLI summaries, and persisted trace semantics.

**Organization**: Tasks are grouped by user story so each slice remains independently testable while still delivering one complete macrofeature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Finalize the 036 feature pack and confirm test-harness entry points before runtime changes.

- [x] T001 Confirm and keep synchronized `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/plan.md`, `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/research.md`, `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/data-model.md`, `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/contracts/`, and `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references in `/Users/rt/workspace/boundline/tests/unit.rs`, `/Users/rt/workspace/boundline/tests/contract.rs`, and `/Users/rt/workspace/boundline/tests/integration.rs` if 036 introduces new test modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared Canon snapshot, compact-memory, and credibility primitives used by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `/Users/rt/workspace/boundline/src/domain/task_context.rs`, `/Users/rt/workspace/boundline/src/domain/governance.rs`, and `/Users/rt/workspace/boundline/src/domain/decision.rs` with Canon capability snapshots, Canon context snapshots, compact Canon memory, and memory-credibility state primitives
- [x] T004 [P] Extend `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`, `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` with Canon-grounded projection fields shared by runtime and CLI surfaces
- [x] T005 [P] Extend `/Users/rt/workspace/boundline/src/adapters/governance_runtime.rs` and `/Users/rt/workspace/boundline/src/orchestrator/governance.rs` to normalize Canon 0.39.0 capabilities, governance summaries, packet lineage, and bounded artifact-summary inputs into Boundline-owned state
- [x] T006 [P] Extend `/Users/rt/workspace/boundline/tests/unit/decision_model.rs`, `/Users/rt/workspace/boundline/tests/unit/session_model.rs`, `/Users/rt/workspace/boundline/tests/unit/goal_plan_model.rs`, and `/Users/rt/workspace/boundline/tests/contract/decision_loop_contract.rs` with foundational coverage for Canon snapshot validation, compact-memory invariants, and credibility transitions

**Checkpoint**: Canon snapshot normalization, persisted compact-memory state, and read-side projections exist and can support all user stories.

---

## Phase 3: User Story 1 - Plan With Canon-Grounded Context (Priority: P1) 🎯 MVP

**Goal**: Make `goal -> plan` treat Canon packets, governed artifacts, artifact summaries, and capability signals as live bounded planning evidence instead of stage-end output only.

**Independent Test**: Run representative native planning scenarios on workspaces with reusable Canon-governed evidence and verify that the proposed plan, rationale, and verification strategy change because of that Canon-grounded input.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for Canon-grounded proposal payloads and planning fallbacks in `/Users/rt/workspace/boundline/tests/contract/goal_plan_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/runtime_refoundation_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for Canon-grounded `goal -> plan` behavior in `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for Canon snapshot normalization, planning influence, and bounded-stop shaping in `/Users/rt/workspace/boundline/tests/unit/goal_planner.rs` and `/Users/rt/workspace/boundline/tests/unit/session_model.rs`

### Implementation for User Story 1

- [x] T010 [US1] Extend `/Users/rt/workspace/boundline/src/orchestrator/governance.rs` and `/Users/rt/workspace/boundline/src/adapters/governance_runtime.rs` to derive Canon context snapshots from capabilities, governance summaries, packet reuse, and artifact summaries
- [x] T011 [US1] Extend `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs` and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` so Canon-grounded snapshots can shape target selection, verification strategy, planning rationale, and bounded planning stops
- [x] T012 [US1] Extend `/Users/rt/workspace/boundline/src/domain/goal_plan.rs` and `/Users/rt/workspace/boundline/src/domain/session.rs` so active proposals persist Canon-grounded headlines, lineage, and compact-memory credibility alongside the existing plan story

**Checkpoint**: Native planning now treats Canon-grounded evidence as live bounded reasoning input and projects that influence explicitly.

---

## Phase 4: User Story 2 - Carry Forward Compacted Canon Memory Across Loops (Priority: P2)

**Goal**: Let later loop iterations reuse a durable compact Canon-grounded memory when it remains credible, and require explicit refresh, replanning, or stop behavior when it does not.

**Independent Test**: Run representative native sessions that span multiple decisions and at least one replan or retry, then verify that later bounded actions can reuse compact Canon memory or stop explicitly when credibility is lost.

### Tests for User Story 2

- [x] T013 [P] [US2] Add contract coverage for compact-memory reuse and refresh semantics in `/Users/rt/workspace/boundline/tests/contract/decision_loop_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs`
- [x] T014 [P] [US2] Add integration coverage for long-running Canon-memory reuse and stale-memory handling in `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`
- [x] T015 [P] [US2] Add unit coverage for compact-memory lifecycle, credibility transitions, and Canon-influenced decision evidence in `/Users/rt/workspace/boundline/tests/unit/decision_loop.rs`, `/Users/rt/workspace/boundline/tests/unit/decision_model.rs`, and `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs`

### Implementation for User Story 2

- [x] T016 [US2] Extend `/Users/rt/workspace/boundline/src/domain/task_context.rs` and `/Users/rt/workspace/boundline/src/domain/decision.rs` to persist compact Canon memory, carry-forward evidence lines, and explicit credibility transitions across later bounded actions
- [x] T017 [US2] Extend `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs` so later decision selection can reuse credible compact Canon memory, attribute Canon evidence explicitly, and trigger refresh or replan when the memory is non-credible
- [x] T018 [US2] Extend `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/cli/run.rs` so direct native run and resumed native execution preserve and reuse compact Canon memory across proposal, execution, and replanning boundaries

**Checkpoint**: Long-running native sessions can reuse credible compact Canon memory and stop explicitly when that memory becomes stale, contradicted, or insufficient.

---

## Phase 5: User Story 3 - Inspect Canon Influence And Bounded Stops (Priority: P3)

**Goal**: Surface Canon-grounded context, compact-memory credibility, and any refresh or stop requirement through the normal Boundline read-side surfaces without hiding compatibility continuity.

**Independent Test**: Run representative planning, execution, and failure scenarios and verify that `run`, `status`, `next`, and `inspect` show Canon influence, compact-memory credibility, lineage, and explicit next actions on both native and explicit compatibility follow-up paths.

### Tests for User Story 3

- [x] T019 [P] [US3] Add contract coverage for CLI-visible Canon inspection and compatibility continuity in `/Users/rt/workspace/boundline/tests/contract/runtime_routing_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for Canon memory projection and explicit stop behavior on `run`, `status`, `next`, and `inspect` in `/Users/rt/workspace/boundline/tests/integration/session_native_flow.rs`, `/Users/rt/workspace/boundline/tests/integration/runtime_refoundation_flow.rs`, and `/Users/rt/workspace/boundline/tests/integration/flow_cli_run.rs`
- [x] T021 [P] [US3] Add unit coverage for Canon inspection rendering, follow-through guidance, and session execution-path projection in `/Users/rt/workspace/boundline/tests/unit/cli_output.rs`, `/Users/rt/workspace/boundline/tests/unit/session_model.rs`, and `/Users/rt/workspace/boundline/tests/unit/runtime_routing.rs`

### Implementation for User Story 3

- [x] T022 [US3] Extend `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/inspect.rs`, and `/Users/rt/workspace/boundline/src/domain/follow_through.rs` to render Canon context headlines, compact-memory credibility, lineage, and refresh or stop guidance
- [x] T023 [US3] Extend `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs` so authoritative session and trace summaries project Canon-grounded reasoning and compact-memory state consistently
- [x] T024 [US3] Extend `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/orchestrator/governance.rs` so explicit compatibility-governed follow-up remains trace-authoritative while reusing the same Canon-grounded reasoning vocabulary where trace evidence exists

**Checkpoint**: Operators can inspect Canon-grounded influence and bounded stops from the normal Boundline surfaces without losing route ownership clarity.

---

## Phase 6: User Story 4 - Ship Canon-Grounded Reasoning As 0.36.0 (Priority: P4)

**Goal**: Close the feature as a release-aligned macrofeature with updated version metadata, product narrative, roadmap closure, assistant guidance, and repository validation evidence.

**Independent Test**: Follow the updated docs on a representative workspace, run the release validation suite, and verify that the version bump, roadmap, docs, changelog, formatting, linting, and coverage all align with the shipped Canon-grounded reasoning model.

### Tests for User Story 4

- [x] T025 [P] [US4] Refresh focused coverage assertions for touched Rust files via `/Users/rt/workspace/boundline/lcov.info` and supporting validation commands

### Implementation for User Story 4

- [x] T026 [US4] Bump crate version to `0.36.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [x] T027 [US4] Update impacted docs and release narrative in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/CHANGELOG.md`, and `/Users/rt/workspace/boundline/AGENTS.md`
- [x] T028 [US4] Update `/Users/rt/workspace/boundline/ROADMAP.md` to mark Spec 036 as delivered and remove it from the remaining future macrofeature line
- [x] T029 [US4] Update assistant guidance impacted by Canon-grounded reasoning in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/`, `/Users/rt/workspace/boundline/assistant/codex/commands/`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/`, and `/Users/rt/workspace/boundline/assistant/gemini/README.md`

**Checkpoint**: Release artifacts describe `0.36.0` and Canon-grounded reasoning consistently.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close remaining quality gaps.

- [x] T030 [P] Run formatting with `cargo fmt --all`
- [x] T031 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run compile-oriented and broader Rust validation for the slice with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T033 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T034 Mark completed tasks in `/Users/rt/workspace/boundline/specs/036-canon-grounded-memory/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because compact Canon memory builds on the Canon-grounded planning snapshot and persisted projection primitives.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because inspection surfaces project the Canon-grounded planning and compact-memory state already made authoritative.
- **User Story 4 (Phase 6)**: Depends on all runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if new test modules are needed.
- T004, T005, and T006 can run in parallel after T003 defines the Canon snapshot and compact-memory primitives.
- Within each user story, test tasks marked `[P]` can be developed in parallel before the implementation tasks touching the same files.
- T025 can be prepared while release docs are being updated, but final coverage confirmation must wait for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that `plan` materially changes when relevant Canon-grounded evidence exists.
3. Use that Canon-grounded planning state as the base for compact-memory reuse and later inspection.

### Incremental Delivery

1. Add Canon snapshot and compact-memory primitives.
2. Make the planner consume Canon-grounded context directly.
3. Reuse compact Canon memory during later decisions and explicit refresh or stop handling.
4. Project Canon influence through `run`, `status`, `next`, and `inspect` while preserving compatibility continuity.
5. Close the release with version, docs, roadmap, assistant guidance, coverage, linting, and formatting.

## Notes

- This feature stays macro-level in value but bounded in implementation: it strengthens the existing native planning, governance, and decision-loop runtime instead of introducing a second memory or orchestration engine.
- Explicit compatibility continuity remains outside the primary scope except where it must preserve authoritative route explanation and Canon-grounded vocabulary.
- The final summary must include a descriptive commit message for the completed feature.