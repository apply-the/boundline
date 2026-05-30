# Tasks: Unify Route Summaries And Config Projection

**Input**: Design documents from `/specs/024-unify-route-summaries/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes cross-route runtime summaries, config projection, route ownership cues, and operator-facing docs. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each route-summary improvement can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.24.0` release boundary and prepare mixed-route fixtures and harnesses for the unified summary slice

- [ ] T001 Bump crate version to `0.24.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Extend mixed-route workspace fixtures and helper data for native, workflow, governance, and compatibility follow-up scenarios in `tests/support/workspace_fixture.rs`
- [ ] T003 Register unify-route-summaries test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared summary, ownership, and config-projection primitives needed by every story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Extend the shared follow-up summary and trace projection models for unified route summaries in `src/domain/session.rs` and `src/domain/trace.rs`
- [ ] T005 [P] Align runtime projection builders for route ownership, continuity authority, and material config inputs in `src/cli/session.rs`, `src/orchestrator/engine.rs`, and `src/orchestrator/session_runtime.rs`
- [ ] T006 [P] Add foundational unit coverage for summary-model convergence and projection helpers in `tests/unit/session_record.rs`, `tests/unit/workflow_session_projection.rs`, `tests/unit/compatibility_continuity.rs`, and `tests/unit/coverage_additional.rs`

**Checkpoint**: Unified summary primitives are available and route ownership plus config inputs can be projected consistently.

---

## Phase 3: User Story 1 - Read One Coherent Follow-Up Story (Priority: P1) 🎯 MVP

**Goal**: Align `status`, `next`, `inspect`, and workflow follow-up summaries around one bounded vocabulary while keeping route ownership explicit.

**Independent Test**: Run representative native, workflow, review/governance, and explicit compatibility scenarios, then confirm the follow-up commands expose the same summary family for owner, authority, execution condition, and next action.

### Tests for User Story 1

- [ ] T007 [P] [US1] Add contract coverage for aligned follow-up summary vocabulary in `tests/contract/trace_summary_contract.rs` and `tests/contract/route_summary_contract.rs`
- [ ] T008 [P] [US1] Add integration coverage for mixed-route summary alignment in `tests/integration/route_summary_projection.rs`
- [ ] T009 [P] [US1] Add unit coverage for aligned CLI summary rendering in `tests/unit/cli_output.rs` and `tests/unit/terminal_precedence.rs`

### Implementation for User Story 1

- [ ] T010 [US1] Implement unified route-summary rendering across `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs`
- [ ] T011 [US1] Preserve explicit route owner, continuity authority, and route evidence in `src/domain/session.rs`, `src/domain/trace.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Operators can read one coherent follow-up story across supported routes without losing ownership cues.

---

## Phase 4: User Story 2 - Surface Routing And Config Inputs Explicitly (Priority: P2)

**Goal**: Project only the routing and config inputs that materially explain why the current route owns follow-up.

**Independent Test**: Configure representative workspace and global defaults, run mixed-route scenarios, and verify that follow-up surfaces expose relevant overrides and defaults without leaking stale or irrelevant config.

### Tests for User Story 2

- [ ] T012 [P] [US2] Add contract coverage for config projection and ownership preservation in `tests/contract/route_summary_contract.rs` and `tests/contract/trace_summary_contract.rs`
- [ ] T013 [P] [US2] Add integration coverage for explicit route overrides and material config projection in `tests/integration/route_config_projection.rs` and `tests/integration/cli_adaptive_execution.rs`
- [ ] T014 [P] [US2] Add unit coverage for config filtering and summary projection in `tests/unit/cli_output.rs`, `tests/unit/session_record.rs`, and `tests/unit/coverage_additional.rs`

### Implementation for User Story 2

- [ ] T015 [US2] Implement material config projection and explicit route-override summaries in `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [ ] T016 [US2] Filter stale or irrelevant config while preserving compatibility and workflow ownership semantics in `src/domain/session.rs`, `src/orchestrator/engine.rs`, and `src/orchestrator/session_runtime.rs`

**Checkpoint**: Unified summaries expose the right routing and config facts without implying hidden ownership.

---

## Phase 5: User Story 3 - Ship The Unified Story As One Release (Priority: P3)

**Goal**: Ship the runtime, docs, assistant guidance, and release metadata as one coherent `0.24.0` story.

**Independent Test**: Follow the updated docs and assistant guidance on a mixed-route workspace, then confirm the observed CLI output matches the documented summary and config-projection behavior.

### Tests for User Story 3

- [ ] T017 [P] [US3] Add assistant and documentation coverage for mixed-route summary guidance in `tests/contract/assistant_session_continuity_contract.rs` and `tests/unit/cli_output.rs`

### Implementation for User Story 3

- [ ] T018 [US3] Update the unified route-summary operator story, impacted docs, and release notes in `README.md`, `docs/getting-started.md`, `docs/configuration.md`, `docs/adaptive-execution.md`, `assistant/README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, and `CHANGELOG.md`
- [ ] T019 [US3] Refresh generated agent context for the unified summary surface in `AGENTS.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.24.0` story for route summaries, ownership, and config projection.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish validation and release-quality checks

- [ ] T020 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the unified summary behavior delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the summary model delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared summary-model work comes before route-specific CLI wording cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Material config filtering must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T018 and T019 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for aligned follow-up summary vocabulary in tests/contract/trace_summary_contract.rs and tests/contract/route_summary_contract.rs"
Task: "Add integration coverage for mixed-route summary alignment in tests/integration/route_summary_projection.rs"

# Launch aligned rendering and ownership work together after the foundational model exists:
Task: "Implement unified route-summary rendering across src/cli/output.rs, src/cli/inspect.rs, and src/cli/session.rs"
Task: "Preserve explicit route owner, continuity authority, and route evidence in src/domain/session.rs, src/domain/trace.rs, and src/cli/inspect.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that operators can read one coherent route summary across supported follow-up surfaces.

### Incremental Delivery

1. Reserve `0.24.0` and the mixed-route fixtures.
2. Tighten the shared summary, ownership, and config-projection primitives.
3. Align CLI output and inspect surfaces across routes.
4. Add material config projection without hiding route ownership.
5. Ship docs, assistant guidance, roadmap, contributing guidance, and changelog updates.
6. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.24.0` as the first task.
- T018 intentionally combines impacted docs and changelog updates as one release-guidance task.
- T020 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.
