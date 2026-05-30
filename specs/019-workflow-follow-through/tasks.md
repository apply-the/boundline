# Tasks: Workflow Follow-Through

**Input**: Design documents from `/specs/019-workflow-follow-through/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes workflow CLI routing, persisted session state, review/govern progression, workflow discovery, and operator-facing summaries. Coverage refresh is part of the final release closeout for this slice.

**Organization**: Tasks are grouped by user story so each workflow follow-through slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and test harness for the workflow follow-through slice

- [x] T001 Bump crate version to `0.19.0` in `Cargo.toml` and `Cargo.lock`
- [x] T002 Create workflow follow-through fixture helpers and discovery-oriented workflow samples in `tests/support/workspace_fixture.rs`
- [x] T003 Register workflow follow-through test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared workflow follow-through, discovery, and validation primitives needed by every story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Extend workflow definition, discovery metadata, and progress-state primitives for executable review/govern follow-through in `src/domain/workflow.rs` and `src/lib.rs`
- [x] T005 [P] Extend active session, goal-plan projection, and CLI summary inputs for workflow discovery and review/govern follow-through in `src/domain/session.rs`, `src/domain/goal_plan.rs`, and `src/cli/output.rs`
- [x] T006 [P] Add foundational unit coverage for follow-through validation, discovery metadata, and session-projection invariants in `tests/unit/workflow_definition.rs`, `tests/unit/workflow_session_projection.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: Workflow definitions can describe bounded discovery metadata and session-owned review/govern follow-through before story-specific CLI flow is added.

---

## Phase 3: User Story 1 - Continue Through Review And Govern (Priority: P1) 🎯 MVP

**Goal**: Let a named workflow execute bounded review and govern phases through the existing session-native runtime instead of stopping at those phases as declaration-only blockers.

**Independent Test**: Define one valid named workflow that includes `review` and `govern`, run it through a representative bounded engineering task, and confirm that Boundline either completes those phases or stops in an explicit paused, blocked, or failed state without manual session edits.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for the workflow follow-through command surface in `tests/contract/workflow_follow_through_command_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for executing review and govern through a named workflow in `tests/integration/workflow_follow_through.rs`
- [x] T009 [P] [US1] Add integration coverage for blocked or non-success review/govern follow-through in `tests/integration/workflow_follow_through_blocked.rs`

### Implementation for User Story 1

- [x] T010 [US1] Implement executable review and govern workflow phases in `src/cli/workflow.rs` and `src/orchestrator/session_runtime.rs`
- [x] T011 [US1] Align workflow-aware session, status, next, and inspect summaries for review/govern follow-through in `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs`
- [x] T012 [US1] Persist explicit paused, blocked, failed, and completed follow-through outcomes in `src/domain/session.rs` and `src/domain/goal_plan.rs`
- [x] T013 [US1] Preserve route ownership and inspectable evidence for review/govern workflow outcomes in `src/cli/workflow.rs` and `src/domain/workflow.rs`

**Checkpoint**: A named workflow can carry review and govern to an explicit bounded outcome through the primary session-native route.

---

## Phase 4: User Story 2 - Discover And Invoke Named Workflows Reliably (Priority: P2)

**Goal**: Make named workflows easier to discover and invoke from operator-driven and assistant-driven flows without inventing a separate runtime story.

**Independent Test**: Prepare a workspace with multiple named workflows, request the available workflow options, choose one from the surfaced guidance, and confirm that subsequent workflow status and resume behavior stay consistent.

### Tests for User Story 2

- [x] T014 [P] [US2] Add contract coverage for the workflow discovery surface in `tests/contract/workflow_discovery_contract.rs`
- [x] T015 [P] [US2] Add integration coverage for listing and invoking named workflows from discovery guidance in `tests/integration/workflow_discovery.rs`
- [x] T016 [P] [US2] Add unit coverage for workflow discovery summaries and invocation guidance in `tests/unit/workflow_definition.rs` and `tests/unit/cli_output.rs`

### Implementation for User Story 2

- [x] T017 [US2] Add the workflow discovery command surface and argument parsing in `src/cli.rs` and `src/cli/workflow.rs`
- [x] T018 [US2] Implement workflow discovery metadata loading, fallback summaries, and validation in `src/domain/workflow.rs` and `src/cli/workflow.rs`
- [x] T019 [US2] Surface assistant-friendly workflow invocation guidance and active-workflow continuation cues in `src/cli/output.rs` and `src/cli/session.rs`

**Checkpoint**: Operators and assistants can discover valid workflows and invoke them through one coherent workflow-aware CLI story.

---

## Phase 5: User Story 3 - Author Workflow Registries With Clear Boundaries (Priority: P3)

**Goal**: Let maintainers author workflow registries that include review and govern while keeping boundaries, examples, and non-goals explicit.

**Independent Test**: Follow the shipped guidance to author or update a representative workflow registry that includes review and govern, validate that the example remains bounded, and confirm that direct session-native and explicit compatibility paths remain available when no named workflow is invoked.

### Tests for User Story 3

- [x] T020 [P] [US3] Add contract coverage for workflow registry guidance and supported authored examples in `tests/contract/workflow_registry_guidance_contract.rs`
- [x] T021 [P] [US3] Add integration coverage that direct session-native and explicit compatibility routes remain available with discovery-enabled workflow registries in `tests/integration/workflow_follow_through_compat.rs`

### Implementation for User Story 3

- [x] T022 [US3] Preserve direct session-native and explicit compatibility behavior when workflow discovery metadata is present in `src/cli/run.rs`, `src/cli/session.rs`, and `src/cli/workflow.rs`
- [x] T023 [US3] Ship bounded registry authoring examples and workflow command relationship guidance in `README.md`, `docs/getting-started.md`, `docs/configuration.md`, and `assistant/README.md`

**Checkpoint**: Registry authors have clear supported examples, and the workflow layer remains bounded and session-owned without taking over every execution path.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release alignment, contributor context, roadmap, changelog, and final validation closeout

- [x] T024 [P] Refresh generated agent and contributor context for the workflow follow-through surface in `AGENTS.md` and `CONTRIBUTING.md`
- [x] T025 Update `ROADMAP.md`, `CHANGELOG.md`, and any touched workflow-related assistant assets under `assistant/` to reflect the `0.19.0` workflow follow-through release
- [x] T026 Run coverage-aware release validation, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with User Story 1 because workflow discovery must describe the real workflow surface.
- User Story 3 depends on Foundational and should reconcile with User Story 1 and User Story 2 before final sign-off.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the active workflow command surface delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract, integration, and unit validations should exist before implementation is considered complete.
- Domain and projection models come before CLI widening.
- Runtime integration comes before workflow-aware summaries and docs.
- Story-specific blocked-state, non-success, and route-preservation coverage must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T024 and T025 can run in parallel once the runtime and documentation surfaces are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for the workflow follow-through command surface in tests/contract/workflow_follow_through_command_contract.rs"
Task: "Add integration coverage for executing review and govern through a named workflow in tests/integration/workflow_follow_through.rs"

# Launch independent persistence and summary work together after runtime hooks exist:
Task: "Align workflow-aware session, status, next, and inspect summaries in src/cli/output.rs, src/cli/inspect.rs, and src/cli/session.rs"
Task: "Persist explicit paused, blocked, failed, and completed follow-through outcomes in src/domain/session.rs and src/domain/goal_plan.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that one named workflow can continue through review and govern or stop in an explicit bounded non-success state.

### Incremental Delivery

1. Add workflow follow-through primitives and projections.
2. Add executable review and govern behavior through the existing session-native runtime.
3. Add workflow discovery and invocation guidance.
4. Add authored registry guidance while preserving direct session-native and compatibility routes.
5. Close with `0.19.0` release alignment, docs, roadmap, changelog, coverage, clippy cleanup, and fmt.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.19.0` as the very first task.
- T026 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after implementation and docs are complete.