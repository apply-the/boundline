# Tasks: Expand Multi-Workspace Delivery

**Input**: Design documents from `/specs/025-multi-workspace-delivery/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes executable clustered delivery behavior, follow-up authority, workspace participation cues, and operator-facing summaries. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each clustered delivery capability can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.25.0` release boundary and prepare the clustered-delivery test harnesses

- [ ] T001 Bump crate version to `0.25.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Extend clustered workspace fixtures and helper data for multi-repository delivery scenarios in `tests/support/workspace_fixture.rs`
- [ ] T003 Register multi-workspace delivery test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared clustered-delivery state and command primitives needed by every story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Extend clustered delivery, authority, and workspace-participation models in `src/domain/cluster.rs`, `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/domain/trace.rs`
- [ ] T005 [P] Add cluster-aware command inputs and shared resolution helpers for session-native commands in `src/cli.rs`, `src/cli/session.rs`, and `src/cli/run.rs`
- [ ] T006 [P] Align orchestration and persistence helpers for one authoritative clustered delivery story in `src/orchestrator/session_runtime.rs`, `src/orchestrator/engine.rs`, and `src/adapters/cluster_store.rs`
- [ ] T007 [P] Add foundational unit coverage for clustered delivery state, task-context serialization, and command parsing in `tests/unit/cluster_models.rs`, `tests/unit/task_context_state.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: Shared clustered-delivery primitives are available and the session-native path can resolve a cluster entry point without introducing parallel ownership.

---

## Phase 3: User Story 1 - Deliver One Bounded Change Across Repositories (Priority: P1) 🎯 MVP

**Goal**: Let one bounded delivery story plan and execute across cluster member workspaces while preserving one authoritative orchestration owner.

**Independent Test**: Register a valid cluster, run one bounded multi-workspace delivery task, and confirm that Boundline records one authoritative delivery story that can read or mutate more than one member workspace without splitting into unrelated runs.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add command and runtime contract coverage for clustered delivery entry and failure boundaries in `tests/contract/cluster_cli_contract.rs` and `tests/contract/cluster_delivery_contract.rs`
- [ ] T009 [P] [US1] Add integration coverage for a successful bounded clustered delivery journey in `tests/integration/cluster_delivery_flow.rs`
- [ ] T010 [P] [US1] Add integration coverage for blocked or non-credible member handoff in `tests/integration/cluster_delivery_blocked.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement cluster-aware `start`, `goal`, `plan`, and `run` command flow in `src/cli.rs`, `src/cli/session.rs`, and `src/cli/run.rs`
- [ ] T012 [US1] Implement bounded workspace-selection, handoff, and participation recording in `src/orchestrator/session_runtime.rs`, `src/orchestrator/engine.rs`, and `src/domain/task.rs`
- [ ] T013 [US1] Persist clustered delivery authority and workspace participation into session and trace state in `src/domain/session.rs`, `src/domain/task_context.rs`, and `src/domain/trace.rs`

**Checkpoint**: One clustered delivery story can execute bounded work across multiple repositories and stop explicitly on success or a member-specific non-success path.

---

## Phase 4: User Story 2 - Follow Clustered Work Without Losing Authority (Priority: P2)

**Goal**: Project authoritative route, authoritative workspace context, and workspace participation through cluster-aware follow-up and inspection surfaces.

**Independent Test**: Run representative successful, blocked, and inspect-only clustered scenarios, then verify that follow-up surfaces expose cluster-aware authority, execution condition, and next action without ambiguity.

### Tests for User Story 2

- [ ] T014 [P] [US2] Add contract coverage for clustered follow-up authority and participation summaries in `tests/contract/runtime_routing_contract.rs`, `tests/contract/trace_summary_contract.rs`, and `tests/contract/cluster_delivery_contract.rs`
- [ ] T015 [P] [US2] Add integration coverage for clustered `status`, `next`, and `inspect` authority handling in `tests/integration/cluster_follow_up_flow.rs` and `tests/integration/cli_trace_inspection.rs`
- [ ] T016 [P] [US2] Add unit coverage for clustered summary rendering and inspect-only authority in `tests/unit/cli_output.rs`, `tests/unit/cluster_projection.rs`, and `tests/unit/compatibility_continuity.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement clustered follow-up rendering across `src/cli/output.rs`, `src/cli/inspect.rs`, `src/cli/cluster.rs`, and `src/cli/session.rs`
- [ ] T018 [US2] Extend session and trace projection builders with authoritative workspace and workspace-participation cues in `src/domain/session.rs`, `src/domain/trace.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Clustered follow-up surfaces clearly identify the authoritative route, authoritative workspace, participating repositories, and next action.

---

## Phase 5: User Story 3 - Ship The Clustered Story As One Release (Priority: P3)

**Goal**: Ship runtime behavior, docs, assistant guidance, and release metadata as one coherent `0.25.0` clustered delivery story.

**Independent Test**: Follow the updated docs and assistant guidance on a representative cluster, then confirm the observed runtime output matches the documented operator story.

### Tests for User Story 3

- [ ] T019 [P] [US3] Add assistant and documentation coverage for clustered delivery guidance in `tests/contract/assistant_session_continuity_contract.rs` and `tests/unit/cli_output.rs`

### Implementation for User Story 3

- [ ] T020 [US3] Update the clustered delivery operator story, impacted docs, and release notes in `README.md`, `tech-docs/getting-started.md`, `tech-docs/configuration.md`, `assistant/README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, and `CHANGELOG.md`
- [ ] T021 [US3] Refresh generated agent context for clustered delivery surfaces in `AGENTS.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.25.0` clustered delivery story.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout

- [ ] T022 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the clustered delivery state introduced by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the clustered delivery story delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared clustered state and command parsing come before route-specific rendering cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Workspace participation and authoritative-workspace cues must be complete before story sign-off.

### Parallel Opportunities

- T005, T006, and T007 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T020 and T021 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add command and runtime contract coverage for clustered delivery entry and failure boundaries in tests/contract/cluster_cli_contract.rs and tests/contract/cluster_delivery_contract.rs"
Task: "Add integration coverage for a successful bounded clustered delivery journey in tests/integration/cluster_delivery_flow.rs"

# Launch clustered execution work together after the foundational model exists:
Task: "Implement cluster-aware start, capture, plan, and run command flow in src/cli.rs, src/cli/session.rs, and src/cli/run.rs"
Task: "Implement bounded workspace-selection, handoff, and participation recording in src/orchestrator/session_runtime.rs, src/orchestrator/engine.rs, and src/domain/task.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that one clustered delivery story can execute bounded work across multiple repositories under one authoritative owner.

### Incremental Delivery

1. Reserve `0.25.0` and clustered fixture support.
2. Tighten shared clustered state, command parsing, and persistence primitives.
3. Enable clustered planning and mutation across member repositories.
4. Project authoritative workspace and participation through follow-up surfaces.
5. Ship docs, assistant guidance, roadmap, contributor guidance, and changelog updates.
6. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.25.0` as the first task.
- T020 intentionally combines impacted docs and changelog updates as one release-guidance task.
- T022 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.