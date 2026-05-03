# Tasks: Goal Negotiation And Constraint Modeling

**Input**: Design documents from `/specs/026-goal-constraint-modeling/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes executable capture, plan gating, follow-up rendering, and route-authority cues. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each negotiation improvement can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.26.0` release boundary and prepare the negotiation test harnesses

- [ ] T001 Bump crate version to `0.26.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [ ] T002 Extend authored-input fixtures and helper data for negotiated goal scenarios in `/Users/rt/workspace/boundline/tests/support/workspace_fixture.rs`
- [ ] T003 Register negotiation test modules in `/Users/rt/workspace/boundline/tests/contract.rs`, `/Users/rt/workspace/boundline/tests/integration.rs`, and `/Users/rt/workspace/boundline/tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared negotiation models, persistence, and planning hooks needed by every story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create negotiated packet domain models and persistence projections in `/Users/rt/workspace/boundline/src/domain/negotiation.rs`, `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/task_context.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/lib.rs`
- [ ] T005 [P] Integrate shared negotiation derivation and planning hooks in `/Users/rt/workspace/boundline/src/domain/brief.rs`, `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`
- [ ] T006 [P] Add session command and projection helpers for negotiated state in `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs`
- [ ] T007 [P] Add foundational unit coverage for negotiation serialization, session projections, and trace/task-context helpers in `/Users/rt/workspace/boundline/tests/unit/negotiation_models.rs`, `/Users/rt/workspace/boundline/tests/unit/session_record.rs`, and `/Users/rt/workspace/boundline/tests/unit/task_context_state.rs`

**Checkpoint**: Shared negotiated-delivery primitives are available and the session-native path can persist, validate, and project negotiation state without changing the overall operator workflow.

---

## Phase 3: User Story 1 - Capture A Negotiated Delivery Packet (Priority: P1) 🎯 MVP

**Goal**: Let capture derive one explicit negotiated delivery packet and stop planning early when the request is not yet credible.

**Independent Test**: Start a session, capture a goal with or without authored inputs, and confirm that Boundline persists one negotiated packet or an explicit clarification/conflict state before planning begins.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add contract coverage for negotiated capture defaults and planning gate behavior in `/Users/rt/workspace/boundline/tests/contract/negotiated_capture_contract.rs`
- [ ] T009 [P] [US1] Add integration coverage for successful goal-only and authored-brief negotiated capture flows in `/Users/rt/workspace/boundline/tests/integration/negotiated_capture_flow.rs`
- [ ] T010 [P] [US1] Add integration coverage for ambiguous or conflicting negotiated capture outcomes in `/Users/rt/workspace/boundline/tests/integration/negotiated_capture_blocked.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement negotiated packet derivation during capture from goals, authored briefs, governance intent, and defaults in `/Users/rt/workspace/boundline/src/domain/negotiation.rs`, `/Users/rt/workspace/boundline/src/domain/brief.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`
- [ ] T012 [US1] Gate planning on a credible negotiated packet and preserve packet summaries in planned state in `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`, and `/Users/rt/workspace/boundline/src/cli/session.rs`
- [ ] T013 [US1] Record negotiation packet creation and blocking signals in session, task-context, and trace state in `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/task_context.rs`, and `/Users/rt/workspace/boundline/src/domain/trace.rs`

**Checkpoint**: Capture yields one inspectable negotiated packet and planning cannot proceed on materially ambiguous or conflicting boundaries.

---

## Phase 4: User Story 2 - Carry Constraints Through Planning And Follow-Up (Priority: P2)

**Goal**: Project the active acceptance boundary, binding constraints, and selected tradeoff story through planning and follow-up surfaces.

**Independent Test**: Capture and plan a negotiated session, then verify that `plan`, `run`, `status`, `next`, and `inspect` identify the active negotiation story in representative success and non-success paths.

### Tests for User Story 2

- [ ] T014 [P] [US2] Add contract coverage for negotiated follow-up summaries and blocking-constraint cues in `/Users/rt/workspace/boundline/tests/contract/constraint_follow_up_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs`
- [ ] T015 [P] [US2] Add integration coverage for negotiated `status`, `next`, and `inspect` behavior in `/Users/rt/workspace/boundline/tests/integration/negotiated_follow_up_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/cli_trace_inspection.rs`
- [ ] T016 [P] [US2] Add unit coverage for negotiated CLI rendering and inspect projections in `/Users/rt/workspace/boundline/tests/unit/cli_output.rs`, `/Users/rt/workspace/boundline/tests/unit/negotiation_models.rs`, and `/Users/rt/workspace/boundline/tests/unit/compatibility_continuity.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement negotiated rendering across `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/inspect.rs`, and `/Users/rt/workspace/boundline/src/cli/session.rs`
- [ ] T018 [US2] Keep compatibility and clustered follow-up authority explicit while projecting negotiated state in `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs`

**Checkpoint**: Operators can identify the active acceptance boundary, binding constraint, and route authority from existing follow-up surfaces.

---

## Phase 5: User Story 3 - Ship The Negotiation Story As One Release (Priority: P3)

**Goal**: Ship runtime behavior, docs, assistant guidance, and release metadata as one coherent `0.26.0` negotiation story.

**Independent Test**: Follow the updated docs and assistant guidance on a representative session, then confirm the observed runtime output matches the documented negotiation behavior.

### Tests for User Story 3

- [ ] T019 [P] [US3] Add assistant and documentation coverage for negotiated capture and follow-up guidance in `/Users/rt/workspace/boundline/tests/contract/assistant_session_continuity_contract.rs` and `/Users/rt/workspace/boundline/tests/unit/cli_output.rs`

### Implementation for User Story 3

- [ ] T020 [US3] Update the negotiated delivery operator story, impacted docs, and release notes in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/getting-started.md`, `/Users/rt/workspace/boundline/docs/configuration.md`, `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/ROADMAP.md`, and `/Users/rt/workspace/boundline/CHANGELOG.md`
- [ ] T021 [US3] Refresh generated agent context for the negotiation surface in `/Users/rt/workspace/boundline/AGENTS.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.26.0` story for negotiated capture, planning, and follow-up.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout

- [ ] T022 Run coverage-aware release validation for modified Rust files, refresh `/Users/rt/workspace/boundline/lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `/Users/rt/workspace/boundline/src/` and `/Users/rt/workspace/boundline/tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the negotiated packet behavior delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the negotiated packet and planning gate delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared negotiation state and planner hooks come before CLI wording cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Compatibility and cluster authority cues must remain explicit before story sign-off.

### Parallel Opportunities

- T005, T006, and T007 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T020 and T021 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for negotiated capture defaults and planning gate behavior in tests/contract/negotiated_capture_contract.rs"
Task: "Add integration coverage for successful goal-only and authored-brief negotiated capture flows in tests/integration/negotiated_capture_flow.rs"

# Launch packet derivation and planning integration work together after the foundational model exists:
Task: "Implement negotiated packet derivation during capture from goals, authored briefs, governance intent, and defaults in src/domain/negotiation.rs, src/domain/brief.rs, and src/orchestrator/session_runtime.rs"
Task: "Gate planning on a credible negotiated packet and preserve packet summaries in planned state in src/orchestrator/goal_planner.rs, src/domain/goal_plan.rs, and src/cli/session.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that capture yields one inspectable negotiated packet and blocks non-credible planning.

### Incremental Delivery

1. Reserve `0.26.0` and the negotiation fixtures.
2. Tighten shared negotiation state, session persistence, and planner hooks.
3. Enable negotiated capture and plan gating.
4. Project negotiated constraints and tradeoffs through follow-up surfaces.
5. Ship docs, assistant guidance, roadmap, contributor guidance, and changelog updates.
6. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.26.0` as the first task.
- T020 intentionally combines impacted docs and changelog updates as one release-guidance task.
- T022 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.