# Tasks: Session & Interaction Model Unification

**Input**: Design documents from `/specs/004-session-model-unification/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds executable CLI behavior, persisted session state, assistant continuity, and explicit failure/recovery handling.

**Organization**: Tasks are grouped by user story so each slice can be implemented, validated, and reviewed with bounded delivery value.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the new session feature surface in the existing crate and test harnesses.

- [X] T001 Wire session module exports and test harness entries in src/domain.rs, src/adapters.rs, src/orchestrator.rs, src/cli.rs, tests/unit.rs, tests/integration.rs, and tests/contract.rs
- [X] T002 [P] Add session command scaffolding in src/cli.rs and src/bin/synod.rs
- [X] T003 [P] Create session feature skeletons in src/domain/session.rs, src/adapters/session_store.rs, src/orchestrator/session_runtime.rs, and src/cli/session.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared persistence, runtime, and rendering primitives required by every user story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 [P] Add foundational unit coverage for session serialization and store invariants in tests/unit/session_record.rs and tests/unit/session_store.rs
- [X] T005 Implement ActiveSessionRecord, SessionTransition, SessionStatus, and SessionStatusView in src/domain/session.rs
- [X] T006 [P] Extend task snapshot persistence and workspace alignment in src/domain/task.rs, src/domain/task_context.rs, and src/domain/plan.rs
- [X] T007 [P] Implement SessionStore and file-backed session persistence in src/adapters/session_store.rs
- [X] T008 Extract session runtime scaffolding and orchestration hooks in src/orchestrator/session_runtime.rs and src/orchestrator.rs
- [X] T009 Implement shared session status and error rendering helpers in src/cli/output.rs
- [X] T010 [P] Add session-aware inspect trace resolution helpers in src/cli/inspect.rs

**Checkpoint**: Foundation ready - session state can be serialized, persisted, resumed, validated, and rendered consistently.

---

## Phase 3: User Story 1 - Start and Reuse Active Work Context (Priority: P1) MVP

**Goal**: Let developers establish one active workspace session and reuse it across follow-up commands without re-entering known context.

**Independent Test**: Start a session in a clean workspace, reuse it from a follow-up command without restating context, and verify that missing-session and invalid-session commands fail with explicit recovery guidance.

### Tests for User Story 1

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T011 [P] [US1] Add contract coverage for active session record creation and validation in tests/contract/session_record_contract.rs
- [X] T012 [P] [US1] Add integration coverage for start and session reuse flows in tests/integration/session_cli_flow.rs
- [X] T013 [P] [US1] Add integration coverage for missing-session recovery guidance in tests/integration/session_cli_flow.rs

### Implementation for User Story 1

- [X] T014 [P] [US1] Implement workspace session create, load, and replace operations in src/adapters/session_store.rs
- [X] T015 [US1] Implement session integrity validation and workspace-mismatch recovery routing in src/adapters/session_store.rs and src/cli/session.rs
- [X] T016 [US1] Implement start and session-resolution handlers in src/cli/session.rs
- [X] T017 [US1] Wire session-native command dispatch and workspace resolution in src/cli.rs and src/bin/synod.rs
- [X] T018 [US1] Surface session reuse and missing-session guidance in src/cli/output.rs

**Checkpoint**: User Story 1 is complete when Synod can create one active session per workspace and reuse it safely across invocations.

---

## Phase 4: User Story 2 - Plan and Execute Through Shared Session State (Priority: P2)

**Goal**: Let developers capture a goal, plan it, and execute stepwise or end-to-end while keeping the session, trace reference, and latest outcome synchronized.

**Independent Test**: Capture a goal inside an active session, plan it, execute one step and a full run, and verify the persisted session always reflects the latest plan position, trace reference, and terminal or recovery state.

### Tests for User Story 2

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T019 [P] [US2] Add contract coverage for session-backed planning and execution transitions in tests/contract/session_command_contract.rs
- [X] T020 [P] [US2] Add integration coverage for capture, plan, step, and run happy paths in tests/integration/session_cli_flow.rs
- [X] T021 [P] [US2] Add integration coverage for retry, replan, failure, and exhaustion persistence in tests/integration/session_cli_flow.rs

### Implementation for User Story 2

- [X] T022 [P] [US2] Persist TaskSnapshot counters, terminal reasons, and latest trace references in src/domain/session.rs and src/domain/task.rs
- [X] T023 [US2] Implement capture and plan session transitions in src/orchestrator/session_runtime.rs
- [X] T024 [US2] Implement step and run session transitions in src/orchestrator/session_runtime.rs
- [X] T025 [US2] Implement capture, plan, step, and run command handlers in src/cli/session.rs
- [X] T026 [US2] Reset execution position and replace stale plan state in src/orchestrator/session_runtime.rs and src/domain/plan.rs
- [X] T027 [US2] Persist latest trace and terminal outcomes after meaningful transitions in src/adapters/session_store.rs and src/orchestrator/session_runtime.rs
- [X] T028 [US2] Extend session-backed execution summaries in src/cli/output.rs
- [X] T029 [US2] Surface latest-trace and failure messaging for session-backed execution in src/cli/inspect.rs

**Checkpoint**: User Story 2 is complete when planning and execution can continue across invocations without reconstructing task state manually.

---

## Phase 5: User Story 3 - Inspect Session State and Route the Next Action (Priority: P3)

**Goal**: Let CLI and assistant surfaces inspect the same active session, report equivalent state, and recommend exactly one valid next action.

**Independent Test**: Partially execute a session, inspect status and next guidance from CLI-oriented and assistant-oriented flows, and verify they agree on current state, trace reference, and recovery guidance when the session is invalid.

### Tests for User Story 3

> **NOTE**: Write these tests first and confirm they fail before implementing the story.

- [X] T030 [P] [US3] Add contract coverage for assistant session continuity rules in tests/contract/assistant_session_continuity_contract.rs
- [X] T031 [P] [US3] Add integration coverage for status, next, and corrupted-session recovery in tests/integration/session_cli_flow.rs
- [X] T032 [P] [US3] Add integration coverage for CLI and assistant guidance equivalence in tests/integration/session_cli_flow.rs

### Implementation for User Story 3

- [X] T033 [P] [US3] Implement status-view mapping and next-command derivation in src/domain/session.rs
- [X] T034 [US3] Implement status and next command handlers in src/cli/session.rs
- [X] T035 [US3] Implement session-aware inspect fallback and invalid-session guidance in src/cli/inspect.rs
- [X] T036 [P] [US3] Update Codex assistant commands to reuse active session state in assistant/codex/commands/synod-start.md, assistant/codex/commands/synod-plan.md, assistant/codex/commands/synod-step.md, assistant/codex/commands/synod-run.md, assistant/codex/commands/synod-status.md, assistant/codex/commands/synod-next.md, and assistant/codex/commands/synod-inspect.md
- [X] T037 [P] [US3] Update Claude assistant commands to reuse active session state in assistant/claude/commands/synod-start.md, assistant/claude/commands/synod-plan.md, assistant/claude/commands/synod-step.md, assistant/claude/commands/synod-run.md, assistant/claude/commands/synod-status.md, assistant/claude/commands/synod-next.md, and assistant/claude/commands/synod-inspect.md
- [X] T038 [P] [US3] Update Copilot continuity prompts to reuse active session state in assistant/copilot/prompts/synod-start.prompt.md, assistant/copilot/prompts/synod-plan.prompt.md, assistant/copilot/prompts/synod-step.prompt.md, assistant/copilot/prompts/synod-run.prompt.md, assistant/copilot/prompts/synod-status.prompt.md, assistant/copilot/prompts/synod-next.prompt.md, and assistant/copilot/prompts/synod-inspect.prompt.md
- [X] T039 [US3] Align CLI-visible status and next messaging with assistant guidance in src/cli/output.rs and assistant/README.md

**Checkpoint**: User Story 3 is complete when CLI and assistant surfaces produce equivalent, explicit continuation guidance from the same active session.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Close the feature with extra coverage, documentation, roadmap alignment, and full validation.

- [X] T040 [P] Expand targeted edge-case coverage for session continuity in tests/unit/session_record.rs, tests/unit/session_store.rs, tests/unit/cli_output.rs, and tests/integration/session_cli_flow.rs
- [X] T041 [P] Update user-facing session workflow documentation in README.md, assistant/README.md, and specs/004-session-model-unification/quickstart.md
- [X] T042 [P] Update roadmap sequencing and follow-up notes for the session model in ROADMAP.md
- [X] T043 Run formatting, lint, and test validation from specs/004-session-model-unification/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP active-session capability.
- **Phase 4: User Story 2**: Depends on Phase 2 and builds on the active-session surface established in User Story 1.
- **Phase 5: User Story 3**: Depends on Phase 2 and should land after the core session state and execution transitions are stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no user-story dependency.
- **US2 (P2)**: Starts after Foundational, but reuses the active-session entry points delivered by US1.
- **US3 (P3)**: Starts after Foundational, but should integrate after US1 and US2 define the persisted state and next-action semantics.

### Within Each User Story

- Tests and contract coverage should be written first and observed failing before implementation.
- Domain state changes should land before CLI or assistant adapters that consume them.
- Session persistence and integrity validation should be stable before status, next-action, or trace-facing messaging is finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T006, T007, and T010 can run in parallel after T005, while T008 and T009 should follow once the shared session model is stable.
- **US1**: T011, T012, and T013 can run in parallel; after that, T014 can proceed in parallel with the first draft of T016.
- **US2**: T019, T020, and T021 can run in parallel; T022 can run in parallel with T023 once the story tests exist.
- **US3**: T030, T031, and T032 can run in parallel; T036, T037, and T038 can run in parallel after T034 and T035 define the final CLI and inspect semantics.
- **Polish**: T040, T041, and T042 can run in parallel before the final validation task T043.

## Parallel Example: User Story 1

```bash
# Launch the User Story 1 validation work together:
Task: "T011 Add contract coverage for active session record creation and validation in tests/contract/session_record_contract.rs"
Task: "T012 Add integration coverage for start and session reuse flows in tests/integration/session_cli_flow.rs"
Task: "T013 Add integration coverage for missing-session recovery guidance in tests/integration/session_cli_flow.rs"

# Once tests exist, split persistence and handler work across independent files:
Task: "T014 Implement workspace session create, load, and replace operations in src/adapters/session_store.rs"
Task: "T016 Implement start and session-resolution handlers in src/cli/session.rs"
```

## Parallel Example: User Story 2

```bash
# Build the planning and execution test surface together:
Task: "T019 Add contract coverage for session-backed planning and execution transitions in tests/contract/session_command_contract.rs"
Task: "T020 Add integration coverage for capture, plan, step, and run happy paths in tests/integration/session_cli_flow.rs"
Task: "T021 Add integration coverage for retry, replan, failure, and exhaustion persistence in tests/integration/session_cli_flow.rs"

# Split state-model and transition work after tests are in place:
Task: "T022 Persist TaskSnapshot counters, terminal reasons, and latest trace references in src/domain/session.rs and src/domain/task.rs"
Task: "T023 Implement capture and plan session transitions in src/orchestrator/session_runtime.rs"
```

## Parallel Example: User Story 3

```bash
# Validate session guidance and assistant continuity in parallel:
Task: "T030 Add contract coverage for assistant session continuity rules in tests/contract/assistant_session_continuity_contract.rs"
Task: "T031 Add integration coverage for status, next, and corrupted-session recovery in tests/integration/session_cli_flow.rs"
Task: "T032 Add integration coverage for CLI and assistant guidance equivalence in tests/integration/session_cli_flow.rs"

# Then split assistant asset updates by provider:
Task: "T036 Update Codex assistant commands to reuse active session state in assistant/codex/commands/..."
Task: "T037 Update Claude assistant commands to reuse active session state in assistant/claude/commands/..."
Task: "T038 Update Copilot continuity prompts to reuse active session state in assistant/copilot/prompts/..."
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate the independent US1 start, reuse, and invalid-session recovery flows.
5. Validate the MVP active-session workflow before adding planning or assistant continuity.

### Incremental Delivery

1. Deliver Setup + Foundational to establish session serialization, persistence, runtime hooks, and integrity checks.
2. Deliver US1 to make active session reuse available from the CLI.
3. Deliver US2 to bind capture, planning, and execution to the active session.
4. Deliver US3 to align status and next guidance across CLI and assistant surfaces.
5. Finish with Phase 6 coverage, docs, roadmap, and end-to-end validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep US2 and US3 behind the established session foundation so the first increment already reduces repeated context entry.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Coverage expansion, documentation updates, and ROADMAP.md alignment are explicitly tracked in T040, T041, and T042.
- Stop after each story checkpoint to validate bounded behavior before expanding scope.