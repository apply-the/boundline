# Tasks: Broaden Bounded Adaptive Repair

**Input**: Design documents from `/specs/023-broaden-bounded-adaptive-repair/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes adaptive candidate generation, credibility ranking, exhaustion handling, route-aware summary projection, and operator-facing docs. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each adaptive repair improvement can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and prepare adaptive fixtures and harnesses for the `0.23.0` slice

- [ ] T001 Bump crate version to `0.23.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Extend bounded adaptive workspace fixtures for broader mutation families and exhaustion scenarios in `tests/support/workspace_fixture.rs`
- [ ] T003 Register broaden-bounded-adaptive-repair test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared adaptive candidate, credibility, and exhaustion primitives needed by all broaden-bounded-adaptive-repair stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Extend adaptive execution profile and change-kind primitives for broader bounded mutation families in `src/domain/execution.rs`
- [ ] T005 [P] Extend adaptive candidate synthesis, credibility ranking, signature persistence, and explicit exhaustion plumbing for depleted, ambiguous, or non-credible evidence in `src/fixture.rs` and `src/orchestrator/terminal.rs`
- [ ] T006 [P] Add foundational unit coverage for execution-profile validation, candidate credibility, and summary rendering in `tests/unit/execution_profile.rs`, `tests/unit/adaptive_execution.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: The runtime can represent broader bounded adaptive candidates, rank them credibly, and stop explicitly when no bounded replan remains.

---

## Phase 3: User Story 1 - Repair More Credible Bounded Failures (Priority: P1) 🎯 MVP

**Goal**: Let adaptive compatibility execution try richer bounded mutation families and choose the most credible local repair candidate before exhausting.

**Independent Test**: Run an adaptive compatibility profile whose failure cannot be fixed by the current three mutation families, then confirm the broader bounded generator selects a new credible candidate and either succeeds or stops explicitly.

### Tests for User Story 1

- [ ] T007 [P] [US1] Add contract coverage for adaptive change-kind and run-trace behavior in `tests/contract/orchestrator_run.rs` and `tests/contract/trace_summary_contract.rs`
- [ ] T008 [P] [US1] Add integration coverage for broader bounded adaptive repair scenarios in `tests/integration/cli_adaptive_execution.rs`
- [ ] T009 [P] [US1] Add unit coverage for family-aware candidate ranking, rejection, and signature exclusion in `tests/unit/adaptive_execution.rs`

### Implementation for User Story 1

- [ ] T010 [US1] Implement new bounded mutation-family generators and manifest-backed change-kind handling in `src/fixture.rs` and `src/domain/execution.rs`
- [ ] T011 [US1] Implement family-aware credibility scoring, validation-guided rejection handling, and persisted candidate evidence in `src/fixture.rs` and `src/domain/execution.rs`

**Checkpoint**: Adaptive compatibility runs can repair a broader set of bounded failures without leaving the explicit compatibility path.

---

## Phase 4: User Story 2 - Explain Credibility And Exhaustion Clearly (Priority: P2)

**Goal**: Keep adaptive candidate credibility, rejected alternatives, and explicit exhaustion reasons visible across run, session, and trace summaries.

**Independent Test**: Run representative adaptive scenarios that succeed after replanning and that exhaust without a credible remaining candidate, then confirm `run`, `status`, `next`, and `inspect` expose the same credibility and exhaustion story.

### Tests for User Story 2

- [ ] T012 [P] [US2] Add contract coverage for adaptive credibility and exhaustion summaries in `tests/contract/runtime_routing_contract.rs`, `tests/contract/session_command_contract.rs`, and `tests/contract/trace_summary_contract.rs`
- [ ] T013 [P] [US2] Add integration coverage for adaptive credibility projection and exhaustion follow-up, including missing or ambiguous validation evidence, in `tests/integration/session_adaptive_flow.rs` and `tests/integration/cli_trace_inspection.rs`

### Implementation for User Story 2

- [ ] T014 [US2] Surface adaptive credibility, rejected-candidate, and exhaustion summaries in `src/cli/output.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`
- [ ] T015 [US2] Keep terminal conditions and compatibility follow-up guidance explicit for exhausted adaptive runs, including missing or ambiguous validation evidence, in `src/orchestrator/terminal.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Developers can tell why an adaptive candidate was selected, why others were rejected, and why bounded recovery stopped.

---

## Phase 5: User Story 3 - Ship One Complete Adaptive Operator Story (Priority: P3)

**Goal**: Ship the release, operator guidance, and assistant context for the broader bounded adaptive repair story without implying a new orchestration mode.

**Independent Test**: Follow the updated operator and assistant guidance for a representative adaptive compatibility run, then confirm the documented credibility and exhaustion behavior matches the CLI surfaces.

### Tests for User Story 3

- [ ] T016 [P] [US3] Add assistant and documentation coverage for adaptive operator guidance in `tests/contract/assistant_session_continuity_contract.rs` and `tests/unit/assistant_assets.rs`

### Implementation for User Story 3

- [ ] T017 [US3] Update the adaptive operator story, impacted docs, and release notes in `README.md`, `docs/adaptive-execution.md`, `docs/getting-started.md`, `docs/configuration.md`, `assistant/README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, and `CHANGELOG.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.23.0` story for bounded adaptive repair, credibility, and exhaustion.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Refresh generated context and finish release validation

- [ ] T018 [P] Refresh generated agent context for the broaden-bounded-adaptive-repair surface in `AGENTS.md`
- [ ] T019 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the runtime behavior delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled operator story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the broader candidate evidence delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Runtime behavior comes before CLI projection cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Explicit exhaustion and credibility coverage must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T017 and T018 can run in parallel once runtime and output behavior are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for adaptive change-kind and run-trace behavior in tests/contract/orchestrator_run.rs and tests/contract/trace_summary_contract.rs"
Task: "Add integration coverage for broader bounded adaptive repair scenarios in tests/integration/cli_adaptive_execution.rs"

# Launch independent generator and ranking work together after the primitives exist:
Task: "Implement new bounded mutation-family generators and manifest-backed change-kind handling in src/fixture.rs and src/domain/execution.rs"
Task: "Implement family-aware credibility scoring and persisted candidate evidence in src/fixture.rs and src/domain/execution.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that adaptive compatibility execution can now repair a broader bounded failure set and stop explicitly when no credible candidate remains.

### Incremental Delivery

1. Reserve `0.23.0` and the broader adaptive fixtures.
2. Tighten adaptive mutation, credibility, and exhaustion primitives.
3. Add explicit CLI and trace projection for credibility and exhaustion.
4. Ship docs, assistant guidance, roadmap, contributing guidance, and changelog updates.
5. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.23.0` as the very first task.
- T017 intentionally combines impacted docs and changelog updates as one release-guidance task.
- T019 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.