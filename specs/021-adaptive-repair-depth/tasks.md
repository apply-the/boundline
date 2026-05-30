# Tasks: Adaptive Repair Depth

**Input**: Design documents from `/specs/021-adaptive-repair-depth/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes adaptive replanning behavior, operator-facing selection evidence, route explanation, and trace summaries. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each adaptive repair slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and prepare adaptive fixtures and harness surfaces for the adaptive-repair-depth slice

- [ ] T001 Bump crate version to `0.21.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Create validation-guided adaptive fixture helpers and bounded multi-target workspaces in `tests/support/workspace_fixture.rs`
- [ ] T003 Register adaptive-repair-depth test modules in `tests/integration.rs`, `tests/contract.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared adaptive evidence and projection primitives needed by all adaptive-repair-depth stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Extend adaptive guidance, selection-evidence, and attempt-lineage primitives in `src/domain/execution.rs` and `src/fixture.rs`
- [ ] T005 [P] Extend session and inspect projection helpers for validation-guided adaptive evidence in `src/domain/session.rs`, `src/cli/session.rs`, `src/cli/output.rs`, and `src/cli/inspect.rs`
- [ ] T006 [P] Add foundational unit coverage for adaptive evidence summaries and output rendering in `tests/unit/adaptive_execution.rs`, `tests/unit/cli_output.rs`, and `tests/unit/coverage_additional.rs`

**Checkpoint**: The runtime and projection layers can represent validation-guided adaptive evidence before story-specific scenarios are completed.

---

## Phase 3: User Story 1 - Replan Adaptive Repairs From Validation Evidence (Priority: P1) 🎯 MVP

**Goal**: Let adaptive compatibility execution use failed validation evidence to choose a more credible bounded next repair candidate.

**Independent Test**: Run an adaptive compatibility profile whose first candidate fails with actionable validation output, then confirm that the next bounded attempt changes because of that evidence and either succeeds or stops explicitly.

### Tests for User Story 1

- [ ] T007 [P] [US1] Add contract coverage for validation-guided adaptive command output in `tests/contract/adaptive_run_contract.rs` and `tests/contract/adaptive_session_contract.rs`
- [ ] T008 [P] [US1] Add integration coverage for validation-guided adaptive replanning in `tests/integration/cli_adaptive_execution.rs`
- [ ] T009 [P] [US1] Add unit coverage for validation-guided candidate ranking and signature exclusion in `tests/unit/adaptive_execution.rs`

### Implementation for User Story 1

- [ ] T010 [US1] Implement validation-guided candidate extraction and bounded target reranking in `src/fixture.rs` and `src/domain/execution.rs`
- [ ] T011 [US1] Persist validation-guided selection reasons, updated workspace slices, and richer attempt-lineage transitions in `src/fixture.rs` and `src/domain/session.rs`

**Checkpoint**: Adaptive compatibility runs can replan from validation evidence instead of only exhausting the deterministic candidate order.

---

## Phase 4: User Story 2 - Inspect Adaptive Selection And Route Boundaries Clearly (Priority: P2)

**Goal**: Keep adaptive selection reasons, workspace-slice changes, and compatibility-route ownership explicit across CLI and trace surfaces.

**Independent Test**: Run a validation-guided adaptive scenario in a workspace that also exposes workflow, review, or governance surfaces, then confirm that `run`, `status`, `next`, and `inspect` preserve the adaptive evidence and explicit compatibility routing story.

### Tests for User Story 2

- [ ] T012 [P] [US2] Add contract coverage for adaptive selection evidence and route guidance in `tests/contract/adaptive_trace_contract.rs` and `tests/contract/adaptive_route_guidance_contract.rs`
- [ ] T013 [P] [US2] Add integration coverage for adaptive route-story projection in `tests/integration/adaptive_route_story.rs`

### Implementation for User Story 2

- [ ] T014 [US2] Surface validation-guided adaptive reasons and changed bounded slices in `src/cli/output.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`
- [ ] T015 [US2] Keep compatibility-route ownership explicit when workflows, review, or governance are present in `src/cli/output.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Developers can inspect adaptive path changes and route boundaries without inferring hidden ownership or heuristics.

---

## Phase 5: User Story 3 - Ship Bounded Adaptive Repair Guidance Cleanly (Priority: P3)

**Goal**: Let maintainers configure and release validation-guided adaptive repair without mistaking it for a new orchestration mode.

**Independent Test**: Follow the shipped guidance to configure a representative adaptive execution profile, validate the route story, and confirm unsupported expectations remain explicit.

### Tests for User Story 3

- [ ] T016 [P] [US3] Add contract coverage for adaptive profile guidance and assistant continuity in `tests/contract/adaptive_route_guidance_contract.rs` and `tests/contract/assistant_session_continuity_contract.rs`

### Implementation for User Story 3

- [ ] T017 [US3] Ship bounded adaptive repair guidance in `README.md`, `docs/adaptive-execution.md`, `docs/getting-started.md`, `docs/configuration.md`, and `assistant/README.md`
- [ ] T018 [US3] Update contributor and roadmap guidance for the adaptive-repair-depth slice in `CONTRIBUTING.md` and `ROADMAP.md`

**Checkpoint**: Maintainers have one coherent authored example for validation-guided adaptive repair and its route boundaries.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release alignment, changelog, agent context, and final validation closeout

- [ ] T019 [P] Refresh generated agent context for the adaptive-repair-depth surface in `AGENTS.md`
- [ ] T020 Update `CHANGELOG.md` and any touched adaptive-related assistant assets under `assistant/` to reflect the `0.21.0` adaptive-repair-depth release
- [ ] T021 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the runtime behavior delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the route story finalized by User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the validation-guided adaptive evidence delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Runtime behavior comes before CLI projection changes.
- Docs and guidance should follow the settled runtime and output story.
- Explicit exhaustion, route visibility, and evidence persistence coverage must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T017 and T018 can run in parallel once the runtime and output surfaces are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for validation-guided adaptive command output in tests/contract/adaptive_run_contract.rs and tests/contract/adaptive_session_contract.rs"
Task: "Add integration coverage for validation-guided adaptive replanning in tests/integration/cli_adaptive_execution.rs"

# Launch independent evidence and ranking work together after the primitives exist:
Task: "Implement validation-guided candidate extraction and bounded target reranking in src/fixture.rs and src/domain/execution.rs"
Task: "Persist validation-guided selection reasons and richer attempt-lineage transitions in src/fixture.rs and src/domain/session.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that adaptive compatibility execution replans from validation evidence and stops explicitly when no credible candidate remains.

### Incremental Delivery

1. Reserve `0.21.0` and the validation-guided adaptive fixtures.
2. Tighten adaptive evidence and candidate ranking primitives.
3. Add explicit CLI and trace projection for the validation-guided route story.
4. Ship guidance, roadmap, and changelog updates for the bounded adaptive repair slice.
5. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.21.0` as the very first task.
- T021 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.