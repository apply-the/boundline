# Tasks: Session And Compatibility Continuity

**Input**: Design documents from `/specs/022-session-compatibility-continuity/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required because this slice changes runtime routing, follow-up continuity, trace resolution, and operator-facing summary surfaces. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each continuity improvement can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and prepare continuity fixtures and harnesses for the `0.22.0` slice

- [X] T001 Bump crate version to `0.22.0` in `/Users/rt/workspace/synod/Cargo.toml` and `/Users/rt/workspace/synod/Cargo.lock`
- [X] T002 Reuse the existing compatibility continuity fixture helpers in `/Users/rt/workspace/synod/tests/support/runtime_refoundation.rs` and `/Users/rt/workspace/synod/tests/support/workspace_fixture.rs` for the new coverage
- [X] T003 Register continuity test modules in `/Users/rt/workspace/synod/tests/contract.rs`, `/Users/rt/workspace/synod/tests/integration.rs`, and `/Users/rt/workspace/synod/tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared continuity and summary primitives needed by all session/compatibility continuity stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Extend continuity authority and shared follow-up summary primitives in `/Users/rt/workspace/synod/src/domain/session.rs` and `/Users/rt/workspace/synod/src/cli/session.rs`
- [X] T005 [P] Extend trace-resolution and shared summary helpers for compatibility follow-up in `/Users/rt/workspace/synod/src/cli/inspect.rs` and `/Users/rt/workspace/synod/src/cli/output.rs`
- [X] T006 [P] Add foundational unit coverage for continuity authority and summary fallback behavior in `/Users/rt/workspace/synod/tests/unit/compatibility_continuity.rs` and `/Users/rt/workspace/synod/tests/unit/cli_output.rs`

**Checkpoint**: The runtime can represent authoritative follow-up state before story-specific command behavior is completed.

---

## Phase 3: User Story 1 - Continue After An Explicit Compatibility Run (Priority: P1) 🎯 MVP

**Goal**: Let operators run `status`, `next`, and `inspect` after an explicit compatibility run and get a clear, bounded answer about what state is authoritative next.

**Independent Test**: Run a compatibility execution profile in a workspace that also has an active native session, then verify that follow-up commands preserve native session state while surfacing explicit compatibility continuity and inspect-oriented next steps where needed.

### Tests for User Story 1

- [X] T007 [P] [US1] Add contract coverage for compatibility follow-up command behavior in `/Users/rt/workspace/synod/tests/contract/compatibility_continuity_contract.rs`
- [X] T008 [P] [US1] Add integration coverage for active native session plus newer compatibility trace in `/Users/rt/workspace/synod/tests/integration/session_compatibility_continuity.rs`
- [X] T009 [P] [US1] Add integration coverage for no-session compatibility follow-up and inspect-only continuity in `/Users/rt/workspace/synod/tests/integration/session_compatibility_continuity.rs`

### Implementation for User Story 1

- [X] T010 [US1] Implement continuity authority resolution from active session state and latest workspace trace in `/Users/rt/workspace/synod/src/cli/session.rs` and `/Users/rt/workspace/synod/src/domain/session.rs`
- [X] T011 [US1] Surface explicit inspect-oriented next-command behavior for compatibility follow-up in `/Users/rt/workspace/synod/src/cli/session.rs` and `/Users/rt/workspace/synod/src/cli/output.rs`
- [X] T012 [US1] Reuse existing compatibility run output and inspect fallback metadata in `/Users/rt/workspace/synod/src/cli/run.rs` and `/Users/rt/workspace/synod/src/cli/inspect.rs` for later continuity resolution

**Checkpoint**: Compatibility follow-up is now clear and bounded even when a native session also exists.

---

## Phase 4: User Story 2 - Reuse One Summary Vocabulary Across Routes (Priority: P2)

**Goal**: Align overlapping native and compatibility summaries without hiding which route actually ran.

**Independent Test**: Execute representative native and compatibility runs with adaptive, review, or governance evidence, then verify that follow-up surfaces reuse the same summary wording for overlapping concepts while keeping route attribution explicit.

### Tests for User Story 2

- [X] T013 [P] [US2] Add contract coverage for shared route summary wording in `/Users/rt/workspace/synod/tests/contract/runtime_routing_contract.rs` and `/Users/rt/workspace/synod/tests/contract/trace_summary_contract.rs`
- [X] T014 [P] [US2] Add integration coverage for aligned native and compatibility summaries in `/Users/rt/workspace/synod/tests/integration/runtime_refoundation_compat.rs` and `/Users/rt/workspace/synod/tests/integration/session_adaptive_flow.rs`

### Implementation for User Story 2

- [X] T015 [US2] Unify overlapping summary wording across `status`, `next`, and `inspect` in `/Users/rt/workspace/synod/src/cli/output.rs` and `/Users/rt/workspace/synod/src/cli/inspect.rs`
- [X] T016 [US2] Project compatibility continuity summaries alongside native session summaries without hiding route ownership in `/Users/rt/workspace/synod/src/cli/session.rs` and `/Users/rt/workspace/synod/src/domain/session.rs`

**Checkpoint**: Native and compatibility follow-up outputs now feel coherent while preserving explicit route differences.

---

## Phase 5: User Story 3 - Release One Coherent Operator Story (Priority: P3)

**Goal**: Ship docs, assistant guidance, and release notes that explain the new continuity model clearly.

**Independent Test**: Follow the updated docs and assistant guidance in a mixed-route workspace, then confirm that the documented follow-up behavior matches the CLI output.

### Tests for User Story 3

- [X] T017 [P] [US3] Add assistant and documentation continuity coverage in `/Users/rt/workspace/synod/tests/contract/assistant_session_continuity_contract.rs` and `/Users/rt/workspace/synod/tests/unit/assistant_assets.rs`

### Implementation for User Story 3

- [X] T018 [US3] Ship continuity guidance in `/Users/rt/workspace/synod/README.md`, `/Users/rt/workspace/synod/docs/getting-started.md`, `/Users/rt/workspace/synod/docs/configuration.md`, `/Users/rt/workspace/synod/docs/adaptive-execution.md`, and `/Users/rt/workspace/synod/assistant/README.md`
- [X] T019 [US3] Update contributor and release guidance for `0.22.0` in `/Users/rt/workspace/synod/CONTRIBUTING.md`, `/Users/rt/workspace/synod/ROADMAP.md`, and `/Users/rt/workspace/synod/CHANGELOG.md`

**Checkpoint**: Maintainers and assistants have one coherent continuity story for the release.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Refresh generated context and finish release validation

- [X] T020 [P] Refresh generated agent context in `/Users/rt/workspace/synod/AGENTS.md`
- [X] T021 Run coverage-aware release validation for modified Rust files, refresh `/Users/rt/workspace/synod/lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `/Users/rt/workspace/synod/src/` and `/Users/rt/workspace/synod/tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the continuity authority delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled follow-up story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the continuity authority and follow-up surfaces delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Continuity authority and fallback rules come before summary wording cleanup.
- Runtime behavior comes before docs and assistant guidance.
- Explicit no-session, inspect-only, and mixed-route follow-up behavior must be covered before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T018 and T019 can run in parallel once runtime and output behavior are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for compatibility follow-up command behavior in tests/contract/compatibility_continuity_contract.rs"
Task: "Add integration coverage for mixed-route and no-session compatibility follow-up in tests/integration/session_compatibility_continuity.rs"

# Launch independent continuity work together after the primitives exist:
Task: "Implement continuity authority resolution in src/cli/session.rs and src/domain/session.rs"
Task: "Keep compatibility run and inspect metadata sufficient for later continuity resolution in src/cli/run.rs and src/cli/inspect.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that mixed-route and no-session compatibility follow-up now stay explicit and bounded.

### Incremental Delivery

1. Reserve `0.22.0` and the continuity fixtures.
2. Tighten continuity authority and inspect fallback primitives.
3. Align shared route summaries once follow-up ownership is explicit.
4. Ship docs, assistant guidance, roadmap, and changelog updates.
5. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.22.0` as the very first task.
- T021 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.