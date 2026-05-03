# Tasks: Session-Native Workflow Layer

**Input**: Design documents from `/specs/018-workflow-layer/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes CLI routing, persisted session state, workflow validation, resume behavior, and operator-facing summaries. Coverage refresh is part of the final release closeout for this slice.

**Organization**: Tasks are grouped by user story so each workflow-layer slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and test harness for the workflow-layer slice

- [X] T001 Bump crate version to `0.18.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [X] T002 Create workflow-layer fixture helpers and workflow-definition samples in `/Users/rt/workspace/boundline/tests/support/workspace_fixture.rs`
- [X] T003 Register workflow-layer test modules in `/Users/rt/workspace/boundline/tests/contract.rs`, `/Users/rt/workspace/boundline/tests/integration.rs`, and `/Users/rt/workspace/boundline/tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared workflow-definition, session-projection, and validation primitives needed by every story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Create workflow definition, phase, and progress-state models plus parsing and validation primitives in `/Users/rt/workspace/boundline/src/domain/workflow.rs` and `/Users/rt/workspace/boundline/src/lib.rs`
- [X] T005 [P] Extend active session and goal-plan projection for workflow identity, active phase, and next action in `/Users/rt/workspace/boundline/src/domain/session.rs` and `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`
- [X] T006 [P] Add foundational unit coverage for workflow validation and session-projection invariants in `/Users/rt/workspace/boundline/tests/unit/workflow_definition.rs`, `/Users/rt/workspace/boundline/tests/unit/workflow_session_projection.rs`, and `/Users/rt/workspace/boundline/tests/unit.rs`

**Checkpoint**: Workflow definitions can be parsed, validated, and projected into session state before any story-specific CLI flow is added.

---

## Phase 3: User Story 1 - Run A Named Delivery Workflow (Priority: P1) 🎯 MVP

**Goal**: Let a developer launch one named workflow that executes through Boundline's existing session-native runtime instead of manually chaining phases.

**Independent Test**: Define one valid named workflow in a workspace, run `boundline workflow run <name>`, and confirm that Boundline validates the workflow, starts the correct session-native route, and blocks explicitly on invalid workflow state without hidden fallback.

### Tests for User Story 1

- [X] T007 [P] [US1] Add contract coverage for the workflow command surface in `/Users/rt/workspace/boundline/tests/contract/workflow_command_surface_contract.rs`
- [X] T008 [P] [US1] Add integration coverage for running a valid named workflow through the session-native route in `/Users/rt/workspace/boundline/tests/integration/workflow_layer_run.rs`
- [X] T009 [P] [US1] Add integration coverage for invalid workflow-definition blocking in `/Users/rt/workspace/boundline/tests/integration/workflow_layer_run.rs`

### Implementation for User Story 1

- [X] T010 [US1] Add the `workflow` command family and argument parsing in `/Users/rt/workspace/boundline/src/cli.rs` and `/Users/rt/workspace/boundline/src/cli/workflow.rs`
- [X] T011 [US1] Implement workflow-definition loading and validation from workspace-local workflow files in `/Users/rt/workspace/boundline/src/cli/workflow.rs` and `/Users/rt/workspace/boundline/src/domain/workflow.rs`
- [X] T012 [US1] Compile named workflow phases onto the existing session-native runtime in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`, `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/planner.rs`
- [X] T013 [US1] Persist active workflow state, phase progression, and blocked workflow reasons in `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, and `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`

**Checkpoint**: A named workflow can start real delivery work through the existing session-native route and stop explicitly on invalid or unmet conditions.

---

## Phase 4: User Story 2 - Resume And Inspect Workflow Progress (Priority: P2)

**Goal**: Make workflow progress resumable and inspectable through the same operator-facing status, next, and inspect surfaces used for session-native delivery work.

**Independent Test**: Start a named workflow, force it into a paused bounded condition, then verify that `workflow status`, `workflow resume`, and `workflow inspect` expose the workflow name, current phase, route, execution condition, and next command consistently.

### Tests for User Story 2

- [X] T014 [P] [US2] Add integration coverage for paused-workflow status and resume behavior in `/Users/rt/workspace/boundline/tests/integration/workflow_layer_resume.rs`
- [X] T015 [P] [US2] Add unit coverage for workflow-aware session and CLI rendering in `/Users/rt/workspace/boundline/tests/unit/cli_output.rs` and `/Users/rt/workspace/boundline/tests/unit/workflow_session_projection.rs`

### Implementation for User Story 2

- [X] T016 [US2] Implement `workflow status`, `workflow resume`, and `workflow inspect` handlers in `/Users/rt/workspace/boundline/src/cli/workflow.rs` and `/Users/rt/workspace/boundline/src/cli.rs`
- [X] T017 [US2] Extend workflow-aware operator summaries in `/Users/rt/workspace/boundline/src/cli/output.rs` and `/Users/rt/workspace/boundline/src/cli/inspect.rs`
- [X] T018 [US2] Align paused workflow resume behavior, trace-backed next-command guidance, and workflow execution conditions in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/domain/session.rs`

**Checkpoint**: Workflow progress is resumable and visible through one coherent operator story.

---

## Phase 5: User Story 3 - Keep Workflow Definitions Bounded And Session-Owned (Priority: P3)

**Goal**: Enforce the bounded workflow semantics of the first slice while preserving the current direct session-native and explicit compatibility paths.

**Independent Test**: Validate that unsupported control-flow behavior is rejected explicitly and that direct session-native commands plus explicit compatibility routing still behave as before when no named workflow is invoked.

### Tests for User Story 3

- [X] T019 [P] [US3] Add contract and unit coverage for unsupported workflow semantics and validation failures in `/Users/rt/workspace/boundline/tests/contract/workflow_definition_contract.rs` and `/Users/rt/workspace/boundline/tests/unit/workflow_definition.rs`
- [X] T020 [P] [US3] Add integration coverage that direct session-native commands and explicit compatibility routing remain available without workflow invocation in `/Users/rt/workspace/boundline/tests/integration/workflow_layer_compat.rs`

### Implementation for User Story 3

- [X] T021 [US3] Enforce bounded workflow semantics and reject unsupported loops, branching, fan-out, and Canon-owned progression in `/Users/rt/workspace/boundline/src/domain/workflow.rs` and `/Users/rt/workspace/boundline/src/cli/workflow.rs`
- [X] T022 [US3] Preserve direct session-native and explicit compatibility routing behavior when no named workflow is invoked in `/Users/rt/workspace/boundline/src/cli/run.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`

**Checkpoint**: The workflow layer stays bounded, sequential, and session-owned without taking over every execution path.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release alignment, docs, assistant assets, and final validation closeout

- [X] T023 [P] Refresh generated agent and contributor context for the new workflow surface in `/Users/rt/workspace/boundline/AGENTS.md` and `/Users/rt/workspace/boundline/CONTRIBUTING.md`
- [X] T024 [P] Update `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/getting-started.md`, `/Users/rt/workspace/boundline/docs/configuration.md`, `/Users/rt/workspace/boundline/assistant/README.md`, and workflow-related assistant command or prompt assets under `/Users/rt/workspace/boundline/assistant/`
- [X] T025 Update `/Users/rt/workspace/boundline/ROADMAP.md`, `/Users/rt/workspace/boundline/CHANGELOG.md`, and any touched `.specify` templates needed to reflect the `0.18.0` workflow-layer release
- [X] T026 Run coverage-aware release validation, refresh `/Users/rt/workspace/boundline/lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `/Users/rt/workspace/boundline/src/` and `/Users/rt/workspace/boundline/tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on User Story 1 because status, resume, and inspect must render real workflow state.
- User Story 3 depends on Foundational and should reconcile with User Story 1 before final sign-off.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on US1 because workflow resume and inspect must reflect real workflow progression.
- **US3**: Depends on Foundational and should align with US1 before final sign-off.

### Within Each User Story

- Contract, integration, and unit validations should exist before implementation is considered complete.
- Workflow-definition and projection models come before CLI widening.
- Runtime integration comes before workflow-aware summaries.
- Story-specific trace and blocked-state coverage must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T023 and T024 can run in parallel once the runtime and CLI surfaces are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for the workflow command surface in tests/contract/workflow_command_surface_contract.rs"
Task: "Add integration coverage for running a valid named workflow through the session-native route in tests/integration/workflow_layer_run.rs"

# Launch independent implementation work together after the domain model exists:
Task: "Add the workflow command family and argument parsing in src/cli.rs and src/cli/workflow.rs"
Task: "Implement workflow-definition loading and validation from workspace-local workflow files in src/cli/workflow.rs and src/domain/workflow.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that one named workflow starts bounded session-native delivery work and blocks explicitly on invalid definitions.

### Incremental Delivery

1. Add workflow-definition parsing and session projection primitives.
2. Add named workflow execution through the existing session-native runtime.
3. Add workflow-aware resume, status, and inspect behavior.
4. Tighten bounded workflow validation and preserve non-workflow routes.
5. Close with docs, roadmap, changelog, coverage, clippy cleanup, and fmt.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.18.0` as the very first task.
- T026 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after implementation and docs are complete.