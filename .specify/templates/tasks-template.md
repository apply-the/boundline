---

description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are expected for Synod features. Include them whenever the
feature defines executable behavior, failure handling, replanning, or trace guarantees.
Omit them only when the feature is truly documentation-only. When the feature changes
runtime routing, governance surfaces, or operator-facing summaries, include coverage for
route explanation, `execution_condition`, selected mode, approval state, packet provenance,
next-command guidance, and other CLI-visible surfaces, then refresh coverage artifacts such
as `lcov.info`.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded,
inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Default Synod layout**: `src/`, `tests/` at repository root
- Adjust paths only when `plan.md` explicitly defines a different structure
- Do not invent frontend/backend or deployment surfaces unless the Constitution Check justifies them

<!-- 
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.
  
  The /speckit.tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  - Entities from data-model.md
  - Endpoints from contracts/
  
  Tasks MUST be organized by user story so each story can be:
  - Implemented independently
  - Tested independently
  - Delivered as an MVP increment
  
  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create the project structure described in plan.md
- [ ] T002 Initialize the runtime or crate manifest with required dependencies
- [ ] T003 [P] Configure formatting, linting, and test commands

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

Examples of foundational tasks (adjust based on your project):

- [ ] T004 Create shared task, context, step, limit, or lifecycle models
- [ ] T005 [P] Define agent, tool, or execution contracts used across stories
- [ ] T006 [P] Setup trace or event recording infrastructure
- [ ] T007 Implement error handling, retry, and terminal-state primitives
- [ ] T008 Setup deterministic fixtures, fakes, or simulators for validation
- [ ] T009 Document explicit scope boundaries and deferred capabilities in code comments or docs where needed

**Checkpoint**: Foundation ready - user story work can begin with bounded execution,
failure handling, and observability primitives in place

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1

> **NOTE: Write these tests first and ensure they fail before implementation when the spec requires executable behavior.**

- [ ] T010 [P] [US1] Contract test for the primary execution interface in tests/contract/[name].rs
- [ ] T011 [P] [US1] Integration test for the primary bounded execution journey in tests/integration/[name].rs
- [ ] T012 [P] [US1] Integration test for one failure or termination path in tests/integration/[name]_failure.rs

### Implementation for User Story 1

- [ ] T013 [P] [US1] Create or extend the core domain model in src/domain/[file].rs
- [ ] T014 [US1] Implement the primary orchestration or service flow in src/[module]/[file].rs
- [ ] T015 [US1] Connect state updates and execution-limit enforcement
- [ ] T016 [US1] Emit trace events and visible error surfaces for this story
- [ ] T017 [US1] Add validation logic for story-specific success conditions

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2

- [ ] T018 [P] [US2] Contract test for the recovery or replanning interface in tests/contract/[name].rs
- [ ] T019 [P] [US2] Integration test for the recovery journey in tests/integration/[name].rs
- [ ] T020 [P] [US2] Integration test for exhausted or terminal recovery behavior in tests/integration/[name]_exhausted.rs

### Implementation for User Story 2

- [ ] T021 [P] [US2] Extend domain or policy models for retries, replanning, or failure decisions
- [ ] T022 [US2] Implement recovery behavior in src/[module]/[file].rs
- [ ] T023 [US2] Integrate recovery behavior with User Story 1 components
- [ ] T024 [US2] Record retry, replanning, and terminal events in the trace output

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3

- [ ] T025 [P] [US3] Contract test for the inspection or trace surface in tests/contract/[name].rs
- [ ] T026 [P] [US3] Integration test for the inspection journey in tests/integration/[name].rs

### Implementation for User Story 3

- [ ] T027 [P] [US3] Create or extend trace-view or inspection models in src/[module]/[file].rs
- [ ] T028 [US3] Implement the inspection surface in src/[module]/[file].rs
- [ ] T029 [US3] Validate that the recorded output explains step order, failures, and final outcome

**Checkpoint**: All user stories should now be independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] TXXX [P] Documentation updates in docs/
- [ ] TXXX [P] Refresh assistant command packs and contributor guidance when runtime story or CLI surfaces change
- [ ] TXXX Code cleanup and refactoring
- [ ] TXXX Performance optimization across all stories after delivery behavior is stable
- [ ] TXXX [P] Additional unit tests in tests/unit/
- [ ] TXXX [P] Additional integration coverage in tests/integration/ plus refreshed `lcov.info`
- [ ] TXXX Harden diagnostics, error messages, and trace readability
- [ ] TXXX Run quickstart.md validation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel if the plan allows it
  - Or sequentially in priority order (P1 -> P2 -> P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable

### Within Each User Story

- Validation tasks MUST fail before implementation when the spec requires executable behavior
- Models before orchestration logic
- Contracts before adapters or integration surfaces
- Core implementation before integration
- Trace and failure-handling coverage before story sign-off
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel when shared state and trace contracts are stable

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Contract test for the primary execution interface in tests/contract/[name].rs"
Task: "Integration test for the primary bounded execution journey in tests/integration/[name].rs"

# Launch all independent model work for User Story 1 together:
Task: "Create or extend the core domain model in src/domain/[file].rs"
Task: "Create supporting trace or state helpers in src/[module]/[file].rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify validation tasks fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, hidden-intelligence work, same-file conflicts, or cross-story dependencies that break independence
