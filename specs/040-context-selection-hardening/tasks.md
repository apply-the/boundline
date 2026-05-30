# Tasks: Context Selection Hardening

**Input**: Design documents from `/specs/040-context-selection-hardening/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes
planner selection behavior, context credibility, clustered scope boundaries,
CLI-visible provenance, documentation flow, and release surfaces.

**Organization**: Tasks are grouped by user story so each story remains
independently testable while still closing one full release-aligned feature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Finalize the 040 feature pack and confirm any new test entry points.

- [x] T001 Keep `/Users/rt/workspace/synod/specs/040-context-selection-hardening/spec.md`, `/Users/rt/workspace/synod/specs/040-context-selection-hardening/plan.md`, `/Users/rt/workspace/synod/specs/040-context-selection-hardening/research.md`, `/Users/rt/workspace/synod/specs/040-context-selection-hardening/data-model.md`, `/Users/rt/workspace/synod/specs/040-context-selection-hardening/contracts/`, and `/Users/rt/workspace/synod/specs/040-context-selection-hardening/quickstart.md` synchronized with the implementation
- [x] T002 [P] Add or update top-level test harness references in `/Users/rt/workspace/synod/tests/unit.rs`, `/Users/rt/workspace/synod/tests/contract.rs`, and `/Users/rt/workspace/synod/tests/integration.rs` if 040 introduces new test modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared context-evidence primitives and projections used by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `/Users/rt/workspace/synod/src/domain/goal_plan.rs` with stronger context-input validation and provenance helpers for evidence-backed context selection
- [x] T004 [P] Extend `/Users/rt/workspace/synod/src/domain/session.rs` and `/Users/rt/workspace/synod/src/domain/trace.rs` with any shared projection fields or invariants needed for richer context credibility and recovery cues
- [x] T005 [P] Extend `/Users/rt/workspace/synod/src/cli/output.rs` and `/Users/rt/workspace/synod/src/cli/inspect.rs` to support richer provenance projection and consistent stale or insufficient context wording
- [x] T006 [P] Extend `/Users/rt/workspace/synod/tests/unit/goal_plan_model.rs`, `/Users/rt/workspace/synod/tests/unit/session_model.rs`, and `/Users/rt/workspace/synod/tests/contract/trace_summary_contract.rs` with foundational coverage for context-pack validation and projection invariants

**Checkpoint**: Shared context-pack validation and output projection primitives exist for all user stories.

---

## Phase 3: User Story 1 - Select Bounded Context From Explicit Evidence (Priority: P1) 🎯 MVP

**Goal**: Make `goal -> plan` admit primary context files and artifacts from explicit evidence instead of heuristic keyword scoring.

**Independent Test**: Run representative native planning scenarios with failing tests, authored briefs, or reusable Canon artifacts and verify that the resulting context pack selects files or artifacts because of those evidence anchors.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for evidence-backed context selection in `/Users/rt/workspace/synod/tests/contract/goal_plan_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for evidence-selected `goal -> plan` behavior in `/Users/rt/workspace/synod/tests/integration/session_native_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for evidence candidate construction and bounded selection in `/Users/rt/workspace/synod/tests/unit/goal_planner.rs`

### Implementation for User Story 1

- [x] T010 [US1] Replace keyword-first workspace target admission in `/Users/rt/workspace/synod/src/orchestrator/goal_planner.rs` with explicit-evidence candidate selection for files and artifacts
- [x] T011 [US1] Extend `/Users/rt/workspace/synod/src/orchestrator/goal_planner.rs` and `/Users/rt/workspace/synod/src/orchestrator/flow_inference.rs` so context-selected targets and rationale drive later planning behavior consistently
- [x] T012 [US1] Extend `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs` so persisted goal-plan and trace payloads record the evidence-selected context pack on the native path

**Checkpoint**: Native planning builds an evidence-selected context pack or an explicit non-credible result.

---

## Phase 4: User Story 2 - Inspect Why Each Input Was Selected (Priority: P2)

**Goal**: Surface file-level or artifact-level provenance and context credibility across existing CLI and trace views.

**Independent Test**: After planning a goal, verify that `status`, `next`, `run`, and `inspect` show why inputs were selected without reading raw JSON.

### Tests for User Story 2

- [x] T013 [P] [US2] Add contract coverage for provenance projection in `/Users/rt/workspace/synod/tests/contract/trace_summary_contract.rs` and `/Users/rt/workspace/synod/tests/contract/session_command_contract.rs`
- [x] T014 [P] [US2] Add integration coverage for provenance on `status`, `next`, and `inspect` in `/Users/rt/workspace/synod/tests/integration/cli_trace_inspection.rs` and `/Users/rt/workspace/synod/tests/integration/session_native_flow.rs`
- [x] T015 [P] [US2] Add unit coverage for CLI rendering and session projection in `/Users/rt/workspace/synod/tests/unit/cli_output.rs` and `/Users/rt/workspace/synod/tests/unit/session_model.rs`

### Implementation for User Story 2

- [x] T016 [US2] Extend `/Users/rt/workspace/synod/src/cli/output.rs`, `/Users/rt/workspace/synod/src/cli/inspect.rs`, and `/Users/rt/workspace/synod/src/cli/session.rs` to render richer context provenance and credibility cues
- [x] T017 [US2] Extend `/Users/rt/workspace/synod/src/domain/session.rs`, `/Users/rt/workspace/synod/src/domain/trace.rs`, and `/Users/rt/workspace/synod/src/domain/goal_plan.rs` to keep one authoritative provenance vocabulary across session and trace projections
- [x] T018 [US2] Extend `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs` so the same authoritative context projection is emitted on run, replanning, and inspectable trace events

**Checkpoint**: Operators can explain why each admitted input shaped the current bounded work.

---

## Phase 5: User Story 3 - Stop Explicitly When Context Is Not Credible (Priority: P3)

**Goal**: Make insufficient, stale, contradictory, or scope-unsafe context selection stop planning explicitly instead of degrading silently.

**Independent Test**: Run vague-goal, stale-evidence, and cluster-boundary scenarios and verify that planning stops with explicit recovery cues.

### Tests for User Story 3

- [x] T019 [P] [US3] Add contract coverage for insufficient or stale context behavior in `/Users/rt/workspace/synod/tests/contract/goal_plan_contract.rs` and `/Users/rt/workspace/synod/tests/contract/session_command_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for insufficient-context and cluster-boundary scenarios in `/Users/rt/workspace/synod/tests/integration/session_native_flow.rs` and `/Users/rt/workspace/synod/tests/integration/cluster_delivery_flow.rs`
- [x] T021 [P] [US3] Add unit coverage for stale or scope-unsafe candidate rejection in `/Users/rt/workspace/synod/tests/unit/goal_planner.rs` and `/Users/rt/workspace/synod/tests/unit/session_model.rs`

### Implementation for User Story 3

- [x] T022 [US3] Extend `/Users/rt/workspace/synod/src/orchestrator/goal_planner.rs` to downgrade or reject context packs when evidence is stale, contradictory, or too broad
- [x] T023 [US3] Extend `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/synod/src/domain/session.rs` so non-credible context states persist one bounded recovery cue for status and next
- [x] T024 [US3] Extend `/Users/rt/workspace/synod/src/cli/output.rs`, `/Users/rt/workspace/synod/src/cli/inspect.rs`, and `/Users/rt/workspace/synod/src/domain/trace.rs` to surface non-credible context without falling back to generic wording

**Checkpoint**: Non-credible context becomes an explicit, inspectable stop path.

---

## Phase 6: User Story 4 - Ship A Coherent 0.40.0 Surface (Priority: P4)

**Goal**: Close the feature as a release-aligned slice with version metadata, clearer docs, changelog, roadmap cleanup, and assistant guidance aligned to the new context-selection story.

**Independent Test**: Follow the updated docs on a representative workspace and run the release validation suite to confirm the narrative and code ship together.

### Tests for User Story 4

- [x] T025 [P] [US4] Refresh focused coverage checks for touched Rust files through `/Users/rt/workspace/synod/lcov.info` and the supporting validation commands

### Implementation for User Story 4

- [x] T026 [US4] Bump the crate version to `0.40.0` in `/Users/rt/workspace/synod/Cargo.toml` and `/Users/rt/workspace/synod/Cargo.lock`
- [x] T027 [US4] Update impacted docs and the changelog in `/Users/rt/workspace/synod/README.md`, `/Users/rt/workspace/synod/docs/`, `/Users/rt/workspace/synod/CONTRIBUTING.md`, `/Users/rt/workspace/synod/CHANGELOG.md`, and `/Users/rt/workspace/synod/assistant/README.md`
- [x] T028 [US4] Update `/Users/rt/workspace/synod/ROADMAP.md` to remove the completed context-selection-hardening work from the future roadmap and record it as delivered
- [x] T029 [US4] Refresh assistant guidance impacted by the new context-selection story in `/Users/rt/workspace/synod/assistant/claude/commands/`, `/Users/rt/workspace/synod/assistant/codex/commands/`, `/Users/rt/workspace/synod/assistant/copilot/prompts/`, and `/Users/rt/workspace/synod/AGENTS.md`

**Checkpoint**: Release artifacts describe `0.40.0` consistently and the README first-run flow is less intimidating.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close the remaining quality gates.

- [x] T030 [P] Run formatting with `cargo fmt --all`
- [x] T031 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run compile-oriented and broader Rust validation for the slice with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T033 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T034 Mark completed tasks in `/Users/rt/workspace/synod/specs/040-context-selection-hardening/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because it projects the authoritative context selected there.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because non-credible context reuses the same provenance vocabulary and projections.
- **User Story 4 (Phase 6)**: Depends on runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if new test modules are added.
- T004, T005, and T006 can run in parallel after T003 defines the shared context invariants.
- Within each user story, tasks marked `[P]` can run in parallel before implementation tasks touching the same files.
- T025 can be prepared while release docs are being updated, but final coverage confirmation waits for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that planning selects context from explicit evidence.
3. Use that authoritative context story as the base for provenance projection and explicit stop handling.

### Incremental Delivery

1. Add shared context validation and projection primitives.
2. Make planning admit files and artifacts only from explicit evidence anchors.
3. Project the same provenance across CLI and trace surfaces.
4. Add explicit stale and insufficient stop handling.
5. Close the release with docs, roadmap, changelog, version bump, coverage, linting, and formatting.

## Notes

- This feature hardens the existing native planner; it does not introduce a new planning runtime or a new indexing subsystem.
- The final implementation summary must include a descriptive commit message for the completed feature.