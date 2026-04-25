# Tasks: Delivery Flows (SDLC Backbone)

**Input**: Design documents from `/specs/005-delivery-flows/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds executable CLI behavior, persisted session flow state, bounded recovery rules, and new trace-visible stage transitions.

**Organization**: Tasks are grouped by user story so each flow slice can be implemented, validated, and reviewed with bounded delivery value.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the delivery-flow surface in the existing crate and test harnesses.

- [X] T001 Wire flow module exports and test harness entries in src/domain.rs, src/lib.rs, tests/unit.rs, tests/integration.rs, and tests/contract.rs
- [X] T002 [P] Add CLI command scaffolding for `flow` in src/cli.rs and src/bin/synod.rs
- [X] T003 [P] Create delivery-flow feature skeletons in src/domain/flow.rs, src/cli/session.rs, src/orchestrator/session_runtime.rs, tests/unit/flow_definition.rs, tests/integration/flow_cli_run.rs, and tests/contract/flow_command_contract.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared flow definitions, persisted flow state, rendering, and runtime invariants required by every user story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Add foundational unit coverage for flow definitions and session flow validation in tests/unit/flow_definition.rs and tests/unit/session_flow_state.rs
- [X] T005 Implement built-in flow definitions and validation in src/domain/flow.rs and src/domain.rs
- [X] T006 [P] Extend ActiveSessionRecord, SessionStatusView, and validation rules for optional flow state in src/domain/session.rs
- [X] T007 [P] Extend trace events and inspectable flow lifecycle payloads for flow selection, stage transitions, retries, replans, and failures in src/domain/trace.rs and tests/contract/flow_session_contract.rs
- [X] T008 Implement flow selection, flow-state persistence, and stage advancement helpers in src/orchestrator/session_runtime.rs
- [X] T009 [P] Generate flow-aware plans by mapping `session.active_flow.flow_name` to built-in fixture-backed stage generators in src/fixture.rs and src/domain/step.rs
- [X] T010 [P] Render optional flow and stage fields in shared session output helpers in src/cli/output.rs and tests/contract/flow_status_contract.rs

**Checkpoint**: Foundation ready - built-in flows can be selected, serialized, validated, and rendered consistently before story-specific execution behavior lands.

---

## Phase 3: User Story 1 - Run a standard bug-fix flow (Priority: P1) MVP

**Goal**: Let developers select the bug-fix flow and execute investigate -> implement -> verify with deterministic stage progression and bounded same-stage recovery.

**Independent Test**: Start a session with a bug-fix goal, select `bug-fix`, plan and run it, and verify that stage progression and failure handling remain within the active stage until terminal success or failure.

### Tests for User Story 1

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T011 [P] [US1] Add contract coverage for `synod flow bug-fix` selection and rejection cases in tests/contract/flow_command_contract.rs
- [X] T012 [P] [US1] Add integration coverage for the bug-fix happy path in tests/integration/flow_cli_run.rs
- [X] T013 [P] [US1] Add integration coverage for retry staying inside the active bug-fix stage in tests/integration/flow_cli_run.rs

### Implementation for User Story 1

- [X] T014 [P] [US1] Implement the `flow` command handler and dispatch for bug-fix selection in src/cli/session.rs, src/cli.rs, and src/bin/synod.rs
- [X] T015 [US1] Implement bug-fix flow planning and initial stage binding in src/orchestrator/session_runtime.rs and src/fixture.rs
- [X] T016 [US1] Enforce same-stage retry or replan behavior, configured execution bounds, and stage transition recording for bug-fix execution in src/orchestrator/engine.rs and src/domain/trace.rs
- [X] T017 [US1] Persist bug-fix flow state and stage progression in src/domain/session.rs and src/orchestrator/session_runtime.rs
- [X] T018 [US1] Surface bug-fix flow progress and next-command guidance in src/cli/session.rs and src/cli/output.rs

**Checkpoint**: User Story 1 is complete when a bug-fix task can run through all stages with explicit same-stage recovery and inspectable stage progression.

---

## Phase 4: User Story 2 - Run a standard change flow (Priority: P2)

**Goal**: Let developers run the lighter `change` flow and see stage-aware status and next guidance without breaking non-flow session usage.

**Independent Test**: Start a session for a change request, select `change`, execute part of the flow, and verify that status and next guidance show the correct stage while non-flow sessions still work without flow data.

### Tests for User Story 2

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T019 [P] [US2] Add contract coverage for flow-aware status and next output in tests/contract/flow_status_contract.rs
- [X] T020 [P] [US2] Add integration coverage for change-flow stage visibility in tests/integration/flow_cli_run.rs
- [X] T021 [P] [US2] Add integration coverage for no-flow regression and invalid flow replacement handling in tests/integration/flow_cli_run.rs

### Implementation for User Story 2

- [X] T022 [P] [US2] Add the `change` flow definition and stage mapping in src/domain/flow.rs and src/fixture.rs
- [X] T023 [US2] Implement flow-aware status-view mapping for active flow, current stage, stage progress, and current step fields in src/domain/session.rs and src/cli/output.rs
- [X] T024 [US2] Implement stage-aware next-command derivation and flow replacement guardrails in src/cli/session.rs and src/orchestrator/session_runtime.rs
- [X] T025 [US2] Preserve non-flow session behavior while rendering optional flow fields in src/cli/session.rs, src/cli/output.rs, and src/cli/inspect.rs

**Checkpoint**: User Story 2 is complete when the `change` flow exposes correct stage-aware guidance and non-flow sessions remain fully functional.

---

## Phase 5: User Story 3 - Run a full delivery flow (Priority: P3)

**Goal**: Let developers bind the broader `delivery` flow and progress through requirements -> architecture -> backlog -> implementation with explicit stage completion and final terminal state.

**Independent Test**: Start a session for a broader delivery request, select `delivery`, execute it to completion, and verify that all stages advance in order and remain inspectable after terminal completion.

### Tests for User Story 3

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T026 [P] [US3] Add contract coverage for persisted delivery-flow session state in tests/contract/flow_session_contract.rs
- [X] T027 [P] [US3] Add integration coverage for end-to-end delivery-flow stage progression in tests/integration/flow_cli_run.rs
- [ ] T028 [P] [US3] Add integration coverage for terminal completion and invalid stage-state recovery in tests/integration/flow_cli_run.rs

### Implementation for User Story 3

- [X] T029 [P] [US3] Add the `delivery` flow definition and four-stage plan mapping in src/domain/flow.rs and src/fixture.rs
- [X] T030 [US3] Finalize delivery-flow stage advancement, stage terminal-result rules, and terminal completion handling in src/orchestrator/engine.rs and src/orchestrator/session_runtime.rs
- [X] T031 [US3] Surface delivery-flow completion and inspectable final stage state in src/domain/session.rs, src/cli/session.rs, src/cli/output.rs, and src/cli/inspect.rs

**Checkpoint**: All three built-in flows are now independently functional and inspectable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Close the feature with documentation, versioning, and full validation.

- [X] T032 [P] Update user-facing delivery-flow documentation in README.md, assistant/README.md, ROADMAP.md, and specs/005-delivery-flows/quickstart.md
- [X] T033 [P] Bump the crate version for the new feature in Cargo.toml
- [X] T034 [P] Expand cross-cutting regression coverage in tests/unit/session_flow_state.rs, tests/integration/flow_cli_run.rs, and tests/contract/flow_status_contract.rs
- [X] T035 Run formatting, lint, and test validation for the delivery-flow slice with `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --all-targets`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP bug-fix flow.
- **Phase 4: User Story 2**: Depends on Phase 2 and reuses the shared flow-state and rendering primitives.
- **Phase 5: User Story 3**: Depends on Phase 2 and should land after the shared stage model is stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but integrates with shared flow-state and output behavior proven by US1.
- **US3 (P3)**: Starts after Foundational but should integrate after US1 and US2 stabilize stage progression and guidance semantics.

### Within Each User Story

- Contract and integration coverage should be written first and observed failing before implementation.
- Flow definitions and session-state changes should land before CLI adapters that render or mutate them.
- Runtime stage progression rules should be stable before status, next, or inspect messaging is finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T004, T006, T007, T009, and T010 can run in parallel once the module scaffolding is in place, while T005 and T008 should sequence the shared core model and runtime helpers.
- **US1**: T011, T012, and T013 can run in parallel; after tests exist, T014 can run in parallel with the first draft of T015.
- **US2**: T019, T020, and T021 can run in parallel; T022 can run in parallel with T023 once the shared model is stable.
- **US3**: T026, T027, and T028 can run in parallel; T029 can run in parallel with the first draft of T031 after foundational flow-state support exists.
- **Polish**: T032, T033, and T034 can run in parallel before the final validation task T035.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T011 Add contract coverage for `synod flow bug-fix` selection and rejection cases in tests/contract/flow_command_contract.rs"
Task: "T012 Add integration coverage for the bug-fix happy path in tests/integration/flow_cli_run.rs"
Task: "T013 Add integration coverage for retry staying inside the active bug-fix stage in tests/integration/flow_cli_run.rs"

# Split command and planning work after tests are in place:
Task: "T014 Implement the `flow` command handler and dispatch for bug-fix selection in src/cli/session.rs, src/cli.rs, and src/bin/synod.rs"
Task: "T015 Implement bug-fix flow planning and initial stage binding in src/orchestrator/session_runtime.rs and src/fixture.rs"
```

## Parallel Example: User Story 2

```bash
# Validate flow-aware operator guidance together:
Task: "T019 Add contract coverage for flow-aware status and next output in tests/contract/flow_status_contract.rs"
Task: "T020 Add integration coverage for change-flow stage visibility in tests/integration/flow_cli_run.rs"
Task: "T021 Add integration coverage for no-flow regression and invalid flow replacement handling in tests/integration/flow_cli_run.rs"

# Then split flow definition and output work:
Task: "T022 Add the `change` flow definition and stage mapping in src/domain/flow.rs and src/fixture.rs"
Task: "T023 Implement flow-aware status-view mapping and stage progress fields in src/domain/session.rs and src/cli/output.rs"
```

## Parallel Example: User Story 3

```bash
# Validate full delivery-flow persistence and progression together:
Task: "T026 Add contract coverage for persisted delivery-flow session state in tests/contract/flow_session_contract.rs"
Task: "T027 Add integration coverage for end-to-end delivery-flow stage progression in tests/integration/flow_cli_run.rs"
Task: "T028 Add integration coverage for terminal completion and invalid stage-state recovery in tests/integration/flow_cli_run.rs"

# Then split definition and terminal-state work:
Task: "T029 Add the `delivery` flow definition and four-stage plan mapping in src/domain/flow.rs and src/fixture.rs"
Task: "T031 Surface delivery-flow completion and inspectable final stage state in src/domain/session.rs, src/cli/session.rs, src/cli/output.rs, and src/cli/inspect.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the independent bug-fix flow selection, progression, and same-stage recovery journeys.
5. Validate the MVP bug-fix flow before broadening the catalog.

### Incremental Delivery

1. Deliver Setup + Foundational to establish built-in flow definitions, persisted flow state, runtime helpers, and shared rendering.
2. Deliver US1 to make deterministic bug-fix execution available.
3. Deliver US2 to expose flow-aware operator guidance and preserve non-flow compatibility.
4. Deliver US3 to add the broader delivery path and terminal-stage visibility.
5. Finish with documentation, versioning, regression coverage, and full validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared flow-state foundation so the first increment already delivers deterministic value without overextending the scope.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Documentation and version updates are explicitly tracked in T032 and T033 per the requested release hygiene.
- Stop at each story checkpoint to validate bounded behavior before expanding the flow catalog.