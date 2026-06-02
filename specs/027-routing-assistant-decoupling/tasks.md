# Tasks: Inspectable Routing And Assistant Decoupling

**Input**: Design documents from `/specs/027-routing-assistant-decoupling/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes executable routing resolution, assistant/backend binding, follow-up rendering, and assistant command-pack expectations. Coverage refresh for modified or created Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so routing visibility, assistant decoupling, and release closeout can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.27.0` release boundary and prepare routing-aware fixtures plus test registration

- [ ] T001 Bump crate version to `0.27.0` in `Cargo.toml` and `Cargo.lock`
- [ ] T002 Extend routing, config, and trace fixtures for inspectable routing scenarios in `tests/support/workspace_fixture.rs`
- [ ] T003 Register routing and assistant-binding test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared routing-decision and assistant-binding primitives required by every story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create shared routing-decision and assistant-binding models plus persistence projections in `src/domain/routing_decision.rs`, `src/domain/session.rs`, `src/domain/trace.rs`, and `src/lib.rs`
- [ ] T005 [P] Integrate effective routing resolution and assistant-binding helpers in `src/domain/configuration.rs`, `src/orchestrator/session_runtime.rs`, and `src/orchestrator/engine.rs`
- [ ] T006 [P] Add shared CLI routing-projection and rendering helpers in `src/cli/output.rs`, `src/cli/inspect.rs`, `src/cli/session.rs`, and `src/cli/run.rs`
- [ ] T007 [P] Add foundational unit coverage for routing-decision serialization, assistant-binding mapping, and CLI rendering helpers in `tests/unit/routing_decision_models.rs`, `tests/unit/assistant_binding.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: Shared routing-decision primitives exist, assistant binding can be derived from effective routing, and session or trace surfaces can render the new projection without changing the overall operator workflow.

---

## Phase 3: User Story 1 - See The Active Routing Decision (Priority: P1) 🎯 MVP

**Goal**: Let operators inspect which provider and model own the active bounded slot and where that decision came from before or after execution.

**Independent Test**: Configure representative routing, run a session-native or explicit compatibility flow, and confirm that `run`, `status`, `next`, and `inspect` expose the active route, authority source, and route owner on success and non-success paths.

### Tests for User Story 1

- [ ] T008 [P] [US1] Add contract coverage for routing-decision visibility across execution and follow-up surfaces in `tests/contract/routing_decision_surface_contract.rs`
- [ ] T009 [P] [US1] Add integration coverage for session-native and inspect-only routing follow-up flows in `tests/integration/routing_follow_up_flow.rs` and `tests/integration/compatibility_routing_follow_up.rs`

### Implementation for User Story 1

- [ ] T010 [US1] Persist and validate routing decisions plus authority source in `src/domain/session.rs`, `src/domain/trace.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`
- [ ] T011 [US1] Render routing headlines, authority-source cues, and route-owner summaries in `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/run.rs`
- [ ] T012 [US1] Preserve routing visibility for compatibility and clustered follow-up in `src/domain/session.rs`, `src/domain/trace.rs`, `src/cli/inspect.rs`, and `src/cli/output.rs`

**Checkpoint**: Operators can identify the active slot route, its authority source, and ownership from the same follow-up surfaces they already use.

---

## Phase 4: User Story 2 - Rebind Assistant Packs Without A Second Runtime (Priority: P2)

**Goal**: Let assistant/backend binding follow effective slot routing while keeping Boundline's CLI workflow and orchestration ownership unchanged.

**Independent Test**: Change the configured route for one or more slots and verify that the selected assistant-backed behavior follows the configured binding, while unsupported bindings fail explicitly and compatibility or cluster ownership stays clear.

### Tests for User Story 2

- [ ] T013 [P] [US2] Add contract coverage for assistant-binding assets, Gemini fallback guidance, and unsupported-runtime handling in `tests/contract/assistant_binding_surface_contract.rs` and `tests/contract/assistant_command_pack_contract.rs`
- [ ] T014 [P] [US2] Add unit and integration coverage for routing-aware backend selection in `tests/unit/assistant_binding.rs` and `tests/integration/routing_backend_selection.rs`

### Implementation for User Story 2

- [ ] T015 [US2] Replace hard-wired native adapter registration with routing-aware assistant or backend binding in `src/orchestrator/session_runtime.rs`, `src/adapters/agent.rs`, and `src/registry/agent_registry.rs`
- [ ] T016 [US2] Surface assistant-binding decisions and explicit unsupported-binding behavior in `src/cli/config.rs`, `src/cli/output.rs`, `src/domain/routing_decision.rs`, and `src/orchestrator/session_runtime.rs`

**Checkpoint**: Changing slot routing changes assistant/backend binding predictably, remains inspectable, and does not create a second runtime or a silent fallback.

---

## Phase 5: User Story 3 - Ship Routing Transparency As One Release (Priority: P3)

**Goal**: Ship runtime behavior, assistant guidance, docs, version metadata, and release notes as one coherent `0.27.0` routing-transparency story.

**Independent Test**: Follow the updated docs and assistant guidance on a representative routed workspace, then confirm the observed runtime output matches the documented routing-decision and assistant-binding behavior.

### Tests for User Story 3

- [ ] T017 [P] [US3] Add assistant-guidance and continuity coverage for routed follow-up surfaces in `tests/contract/assistant_session_continuity_contract.rs` and `tests/unit/cli_output.rs`

### Implementation for User Story 3

- [ ] T018 [US3] Update the routed operator story, impacted docs, assistant guidance, and release notes in `README.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `assistant/README.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-next.md`, `assistant/claude/commands/boundline-inspect.md`, `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-next.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, `assistant/copilot/prompts/boundline-next.prompt.md`, `assistant/copilot/prompts/boundline-inspect.prompt.md`, `assistant/gemini/README.md`, `CONTRIBUTING.md`, `ROADMAP.md`, and `CHANGELOG.md`
- [ ] T019 [US3] Refresh generated agent context for the routed assistant surface in `AGENTS.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.27.0` story for routing transparency, assistant binding, and release behavior.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout

- [ ] T020 Run coverage-aware release validation for modified Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the routing-decision behavior delivered by User Story 1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from User Story 1 and User Story 2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the routing-decision projection delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared routing-decision state and backend binding helpers come before CLI wording cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Compatibility and cluster authority cues must remain explicit before story sign-off.

### Parallel Opportunities

- T005, T006, and T007 can run in parallel after T004.
- Test tasks within each user story marked `[P]` can run in parallel.
- T018 and T019 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for routing-decision visibility across execution and follow-up surfaces in tests/contract/routing_decision_surface_contract.rs"
Task: "Add integration coverage for session-native and inspect-only routing follow-up flows in tests/integration/routing_follow_up_flow.rs and tests/integration/compatibility_routing_follow_up.rs"

# Launch route persistence and CLI projection work together after the foundational model exists:
Task: "Persist and validate routing decisions plus authority source in src/domain/session.rs, src/domain/trace.rs, src/orchestrator/session_runtime.rs, and src/cli/session.rs"
Task: "Render routing headlines, authority-source cues, and route-owner summaries in src/cli/output.rs, src/cli/inspect.rs, and src/cli/run.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that route selection stays visible on representative success and non-success follow-up paths.

### Incremental Delivery

1. Reserve `0.27.0` and extend routing-aware fixtures plus test registration.
2. Tighten shared routing-decision state, session or trace persistence, and assistant-binding helpers.
3. Project active routing decisions through session-native and compatibility follow-up surfaces.
4. Replace hard-wired backend selection with routing-aware assistant binding.
5. Ship docs, assistant guidance, roadmap, contributor guidance, changelog, and refreshed agent context.
6. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.27.0` as the first task.
- T018 intentionally combines impacted docs and changelog updates as one release-guidance task.
- T020 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.