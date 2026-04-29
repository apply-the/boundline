# Tasks: Session-Native Surface Unification

**Input**: Design documents from `/specs/016-session-native-surface-unification/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes operator-facing execution semantics, route precedence, blocked and waiting guidance, optional mode projections, and compatibility labeling.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Release and test harness setup for the surface unification slice

- [X] T001 Bump crate version to `0.16.0` in `Cargo.toml` and `Cargo.lock`
- [X] T002 Create shared session-surface fixtures in `tests/support/session_surface_unification.rs`
- [X] T003 Register session-surface test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared summary-model and rendering primitives that every user story depends on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Extend the unified session summary primitives in `src/domain/session.rs` and `src/domain/trace.rs`
- [X] T005 [P] Add shared execution-condition and optional-mode projection helpers in `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/domain/governance.rs`
- [X] T006 [P] Add shared operator-surface rendering helpers in `src/cli/output.rs` and `src/cli/inspect.rs`
- [X] T007 Add foundational unit coverage for unified summary and rendering invariants in `tests/unit/session_surface_summary.rs`, `tests/unit/trace_summary.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: Synod can derive one coherent route, condition, and optional-mode summary before story-specific rendering behavior is widened.

---

## Phase 3: User Story 1 - One Coherent Session View (Priority: P1) 🎯 MVP

**Goal**: Let developers move from planning to execution to status to inspection without reinterpreting route, condition, decision, or next-step guidance.

**Independent Test**: Prepare a session-native task and verify that `run`, `status`, `next`, and `inspect` present the same route explanation, decision summary, and remediation guidance across active, blocked, and terminal states.

### Tests for User Story 1

- [X] T008 [P] [US1] Add contract coverage for unified route, condition, and latest-decision fields in `tests/contract/session_surface_contract.rs`
- [X] T009 [P] [US1] Add integration coverage for consistent native `run`, `status`, `next`, and `inspect` summaries in `tests/integration/session_surface_native.rs`
- [X] T010 [P] [US1] Add integration coverage for blocked, waiting, and non-success terminal guidance in `tests/integration/session_surface_conditions.rs`

### Implementation for User Story 1

- [X] T011 [US1] Implement normalized execution-condition projection in `src/domain/session.rs`, `src/cli/session.rs`, and `src/orchestrator/session_runtime.rs`
- [X] T012 [US1] Unify session-owned summary assembly for `status` and `next` in `src/cli/session.rs` and `src/cli/output.rs`
- [X] T013 [US1] Reuse unified route and condition semantics in trace summarization in `src/domain/trace.rs` and `src/cli/inspect.rs`
- [X] T014 [US1] Align `run` output with the session-owned summary model in `src/cli/run.rs` and `src/cli/output.rs`

**Checkpoint**: Native operator surfaces tell one coherent story for active, blocked, waiting, and terminal conditions.

---

## Phase 4: User Story 2 - Unified Optional Mode Summaries (Priority: P2)

**Goal**: Make bounded review, adaptive execution, and governance appear as extensions of the same session-native summary instead of as competing runtime stories.

**Independent Test**: Run representative review, adaptive, and governed scenarios and verify that each projects through the same session-owned summary model with consistent route explanation, current condition, and next-command guidance.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for review, adaptive, and governance projections in `tests/contract/session_mode_projection_contract.rs`
- [X] T016 [P] [US2] Add integration coverage for review and adaptive summary alignment in `tests/integration/session_surface_modes.rs`
- [X] T017 [P] [US2] Add integration coverage for governed waiting and blocked guidance in `tests/integration/session_surface_governance.rs`

### Implementation for User Story 2

- [X] T018 [US2] Attach review and adaptive projections to the unified session summary in `src/domain/session.rs`, `src/cli/session.rs`, and `src/cli/output.rs`
- [X] T019 [US2] Attach governance state and next-action projections to session and inspection summaries in `src/domain/session.rs`, `src/cli/inspect.rs`, and `src/cli/output.rs`
- [X] T020 [US2] Reuse optional-mode projections in trace summaries and run output in `src/domain/trace.rs`, `src/cli/run.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Optional bounded modes enrich the same operator-facing summary model rather than replacing it.

---

## Phase 5: User Story 3 - Explicit Compatibility Path (Priority: P3)

**Goal**: Preserve declarative compatibility behavior as a visibly distinct path while keeping ready session-native state authoritative unless compatibility is explicitly requested.

**Independent Test**: Compare a compatibility-only run with a workspace that also has a ready session-native plan, then verify that route choice, precedence, and explanations remain explicit across the operator-facing surfaces.

### Tests for User Story 3

- [X] T021 [P] [US3] Add contract coverage for explicit compatibility labeling and precedence in `tests/contract/session_compatibility_contract.rs`
- [X] T022 [P] [US3] Add integration coverage for compatibility-only operator surfaces in `tests/integration/session_surface_compat.rs`
- [X] T023 [P] [US3] Add integration coverage for native-preferred precedence when compatibility artifacts coexist with ready session state in `tests/integration/session_surface_routing.rs`

### Implementation for User Story 3

- [X] T024 [US3] Implement explicit compatibility route explanation and precedence projection in `src/orchestrator/session_runtime.rs`, `src/domain/session.rs`, and `src/cli/run.rs`
- [X] T025 [US3] Align compatibility rendering with the shared summary fields in `src/fixture.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [X] T026 [US3] Preserve blocked and missing-context remediation guidance across native and compatibility routes in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Compatibility stays available and explicit without obscuring the session-native story.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release hygiene, generated context, coverage, and product-story alignment

- [X] T027 [P] Refresh generated agent and contributor context in `AGENTS.md` and `CONTRIBUTING.md`
- [X] T028 [P] Run release validation and refresh `lcov.info` via `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and `cargo deny check licenses advisories bans sources`
- [X] T029 Increase coverage on staged files in `tests/unit/`, `tests/integration/`, and `lcov.info` and update `README.md`, `ROADMAP.md`, `docs/getting-started.md`, `docs/configuration.md`, `docs/adaptive-execution.md`, `docs/review-voting.md`, `docs/session-native-orchestrator-review.md`, `assistant/README.md`, `assistant/claude/commands/`, `assistant/codex/commands/`, `assistant/copilot/prompts/`, `.specify/templates/spec-template.md`, `.specify/templates/plan-template.md`, and `.specify/templates/tasks-template.md` for the `0.16.0` session-native surface unification release

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on User Story 1 because optional-mode projections should extend the coherent primary summary rather than define it.
- User Story 3 depends on Foundational and should reconcile with User Story 1 before final sign-off.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on US1 unified route and condition semantics.
- **US3**: Depends on Foundational and should align with US1 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Projection and state helpers come before rendering updates.
- Rendering and trace wiring must be finished before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T027 and T028 can run in parallel once the implementation is stable.

---

## Parallel Example: User Story 2

```bash
# Launch User Story 2 validation work together:
Task: "Add contract coverage for review, adaptive, and governance projections in tests/contract/session_mode_projection_contract.rs"
Task: "Add integration coverage for review and adaptive summary alignment in tests/integration/session_surface_modes.rs"

# Launch independent User Story 2 implementation work together after validations exist:
Task: "Attach review and adaptive projections to the unified session summary in src/domain/session.rs, src/cli/session.rs, and src/cli/output.rs"
Task: "Attach governance state and next-action projections to session and inspection summaries in src/domain/session.rs, src/cli/inspect.rs, and src/cli/output.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the coherent native operator story before widening scope.

### Incremental Delivery

1. Normalize the core session-native summary.
2. Layer optional review, adaptive, and governance projections onto it.
3. Reconcile explicit compatibility routing with the same summary model.
4. Finish with coverage growth and documentation rollout for `0.16.0`.

## Notes

- `[P]` tasks touch different files or surfaces and can be split safely.
- The first task intentionally reserves the release bump to `0.16.0`.
- The final task is intentionally reserved for staged-file coverage growth and README and ROADMAP inclusive documentation alignment for the release.