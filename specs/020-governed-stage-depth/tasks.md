# Tasks: Governed Stage Depth

**Input**: Design documents from `/specs/020-governed-stage-depth/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes governed-stage runtime behavior, session and workflow projection, packet lineage visibility, and operator-facing guidance. Coverage refresh for modified Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so each governed-stage slice can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the release boundary and prepare fixture and harness surfaces for the governed-stage-depth slice

- [ ] T001 Bump crate version to `0.20.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Create governed-investigate fixture helpers and Canon stub workspaces in `tests/support/workspace_fixture.rs`
- [ ] T003 Register governed-stage-depth test modules in `tests/integration.rs`, `tests/contract.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared runtime and projection primitives needed by all governed-stage-depth stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Extend governed-stage runtime handling for the `bug-fix:investigate` to `verify` path in `src/orchestrator/session_runtime.rs` and `src/orchestrator/governance.rs`
- [ ] T005 [P] Extend session and workflow governance projection helpers for earlier-stage lineage and refresh visibility in `src/domain/session.rs`, `src/cli/session.rs`, `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/workflow.rs`
- [ ] T006 [P] Add foundational unit coverage for governed-stage mode mapping, output rendering, and packet-lineage helpers in `tests/unit/governance_runtime.rs`, `tests/unit/cli_output.rs`, and `tests/unit/workflow_session_projection.rs`

**Checkpoint**: The runtime and projection layers can represent a governed investigate stage ahead of the existing governed verify story before story-specific scenarios are completed.

---

## Phase 3: User Story 1 - Govern Investigate Before Verify On One Session Route (Priority: P1) 🎯 MVP

**Goal**: Let a bug-fix session govern `investigate` before later governed `verify`, preserving packet lineage and explicit non-success outcomes on the primary session-native route.

**Independent Test**: Run a bug-fix session with governance configured for `investigate`, confirm that investigate governance executes or stops explicitly, and then verify that later governed verify work can reuse bounded lineage when credible.

### Tests for User Story 1

- [ ] T007 [P] [US1] Add contract coverage for the governed-stage command surface in `tests/contract/governed_stage_command_surface_contract.rs`
- [ ] T008 [P] [US1] Add integration coverage for a successful governed `bug-fix:investigate` path that later reaches governed verify in `tests/integration/governed_stage_depth.rs`
- [ ] T009 [P] [US1] Add integration coverage for approval-pending and blocked investigate outcomes in `tests/integration/governed_stage_depth_blocked.rs`

### Implementation for User Story 1

- [ ] T010 [US1] Implement or tighten governed `bug-fix:investigate` execution, halt, and continue behavior in `src/orchestrator/session_runtime.rs` and `src/domain/governance.rs`
- [ ] T011 [US1] Persist investigate-to-verify packet reuse lineage and blocked reasons in `src/orchestrator/governance.rs` and `src/domain/session.rs`
- [ ] T012 [US1] Surface governed investigate stage identity, selected mode, and next-action guidance in `src/cli/output.rs`, `src/cli/session.rs`, and `src/cli/inspect.rs`

**Checkpoint**: A bug-fix session can govern investigate on the direct session-native route, stop explicitly when needed, and carry bounded lineage toward later verify work.

---

## Phase 4: User Story 2 - Refresh Governance State And Guidance Across Transitions (Priority: P2)

**Goal**: Keep approval refresh, packet readiness, and packet provenance explicit across later commands and workflow-aware projection surfaces.

**Independent Test**: Pause a governed bug-fix session after investigate, refresh it through later commands, and confirm that direct session and workflow surfaces report the same refreshed governance condition and lineage.

### Tests for User Story 2

- [ ] T013 [P] [US2] Add contract coverage for governed refresh and lineage visibility in `tests/contract/governed_stage_refresh_contract.rs`
- [ ] T014 [P] [US2] Add integration coverage for investigate-to-verify packet reuse and approval refresh in `tests/integration/governed_stage_depth_refresh.rs`
- [ ] T015 [P] [US2] Add integration coverage for workflow-aware governed-stage projection in `tests/integration/governed_stage_depth_workflow.rs`

### Implementation for User Story 2

- [ ] T016 [US2] Extend governance refresh behavior and explicit waiting or blocked guidance on later commands in `src/orchestrator/session_runtime.rs`, `src/cli/session.rs`, and `src/cli/output.rs`
- [ ] T017 [US2] Extend workflow-aware projection of refreshed governance state and packet provenance in `src/cli/workflow.rs`, `src/cli/output.rs`, and `src/domain/session.rs`
- [ ] T018 [US2] Surface refreshed governance timeline details for inspect output in `src/cli/inspect.rs` and `src/orchestrator/session_runtime.rs`

**Checkpoint**: Later commands and named workflows refresh governance state before progression and keep packet lineage visible and actionable.

---

## Phase 5: User Story 3 - Author And Ship Bounded Governed Depth Clearly (Priority: P3)

**Goal**: Let maintainers configure the deeper governed bug-fix slice and understand the routing boundaries without widening Boundline into a generic governance engine.

**Independent Test**: Follow the shipped guidance to configure governed `bug-fix:investigate`, validate unsupported expectations remain explicit, and confirm docs describe the same direct and workflow-aware route story.

### Tests for User Story 3

- [ ] T019 [P] [US3] Add contract coverage for governance profile guidance in `tests/contract/governance_profile_guidance_contract.rs`
- [ ] T020 [P] [US3] Add integration coverage that direct session-native and workflow-aware routes remain explicit with deeper governed bug-fix config in `tests/integration/governed_stage_depth_route_story.rs`

### Implementation for User Story 3

- [ ] T021 [US3] Ship bounded governance profile examples and route-boundary guidance in `README.md`, `tech-docs/getting-started.md`, `tech-docs/configuration.md`, and `assistant/README.md`
- [ ] T022 [US3] Update contributor and roadmap guidance for the governed-stage-depth slice in `CONTRIBUTING.md` and `ROADMAP.md`

**Checkpoint**: Maintainers have one coherent authored example for the deeper governed bug-fix slice, and route ownership stays explicit.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Release alignment, changelog, agent context, and final validation closeout

- [ ] T023 [P] Refresh generated agent context for the governed-stage-depth surface in `AGENTS.md`
- [ ] T024 Update `CHANGELOG.md` and any touched governance-related assistant assets under `assistant/` to reflect the `0.20.0` governed-stage-depth release
- [ ] T025 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

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
- **US2**: Depends on Foundational and should align with the active governed-stage runtime and packet-lineage surfaces delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Runtime behavior comes before CLI and workflow projection changes.
- Docs and guidance should follow the settled runtime and projection story.
- Blocked, waiting, and reuse-lineage coverage must be complete before story sign-off.

### Parallel Opportunities

- T005 and T006 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T021 and T022 can run in parallel once the runtime and output surfaces are stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for the governed-stage command surface in tests/contract/governed_stage_command_surface_contract.rs"
Task: "Add integration coverage for a successful governed bug-fix investigate path that later reaches governed verify in tests/integration/governed_stage_depth.rs"

# Launch independent persistence and output work together after runtime hooks exist:
Task: "Persist investigate-to-verify packet reuse lineage and blocked reasons in src/orchestrator/governance.rs and src/domain/session.rs"
Task: "Surface governed investigate stage identity and next-action guidance in src/cli/output.rs, src/cli/session.rs, and src/cli/inspect.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that a bug-fix session can govern investigate before later verify work and stop explicitly on non-success conditions.

### Incremental Delivery

1. Reserve `0.20.0` and the governed-stage-depth fixtures.
2. Tighten runtime and projection primitives for governed investigate plus later verify lineage.
3. Add explicit approval refresh and workflow-aware projection across later commands.
4. Ship guidance, roadmap, and changelog updates for the bounded deeper governed slice.
5. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.20.0` as the very first task.
- T025 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.