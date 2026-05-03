---

description: "Task list for implementing assistant command packs"

---

# Tasks: Assistant Command Packs

**Input**: Design documents from `/specs/003-assistant-command-packs/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are included because this feature defines executable assistant workflows, chat-only fallback behavior, non-success handling, and trace-backed inspection guarantees.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`[US1]`, `[US2]`, `[US3]`)
- Every task includes exact file paths in the description

## Path Conventions

- Assistant asset files live in `assistant/`
- Existing CLI backend files live in `src/cli/`, `src/adapters/`, and `src/domain/`
- Validation files live in `tests/unit/`, `tests/integration/`, and `tests/contract/`
- Feature planning artifacts live in `specs/003-assistant-command-packs/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the assistant asset surface and wire new validation entrypoints into the existing Rust test harness.

- [x] T001 Create the assistant command-pack directory skeleton and shared documentation entrypoint in `assistant/README.md`, `assistant/claude/commands/`, `assistant/codex/commands/`, and `assistant/copilot/prompts/`
- [x] T002 [P] Register assistant asset validation modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`
- [x] T003 [P] Create assistant validation scaffolds in `tests/contract/assistant_command_pack_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, `tests/integration/assistant_shell_enabled_flow.rs`, `tests/integration/assistant_chat_fallback.rs`, and `tests/unit/assistant_assets.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define shared pack rules, backend mapping checks, and assistant-consumable CLI output guarantees used by every story.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T004 Implement shared installation, registration, and fallback conventions in `assistant/README.md`
- [x] T005 [P] Implement required command-pack coverage assertions in `tests/contract/assistant_command_pack_contract.rs`
- [x] T006 [P] Implement required section and backend-mapping assertions in `tests/contract/assistant_command_definition_contract.rs`
- [x] T007 [P] Implement asset filename, surface, and cross-pack consistency checks in `tests/unit/assistant_assets.rs`
- [x] T008 Implement shell-enabled and chat-only assistant flow scaffolding over `doctor`, `run`, and `inspect` in `tests/integration/assistant_shell_enabled_flow.rs` and `tests/integration/assistant_chat_fallback.rs`
- [x] T009 Align CLI output guarantees for assistant parsing in `src/cli/output.rs`, `src/cli/diagnostics.rs`, `src/cli/run.rs`, and `src/cli/inspect.rs`

**Checkpoint**: Foundation ready. The assistant asset structure, validation scaffolding, and CLI parsing guarantees are stable for all user stories.

---

## Phase 3: User Story 1 - Start a Workflow from Chat (Priority: P1) 🎯 MVP

**Goal**: Let a user start a Boundline workflow from chat, gather only missing context, and route cleanly into readiness checking or bounded execution without memorizing CLI syntax.

**Independent Test**: Invoke `/boundline-start` and `/boundline-plan` in a supported assistant surface with and without shell access, then confirm the assistant asks only for missing workspace or goal context, runs or recommends the correct CLI command, and explains the next action clearly.

### Tests for User Story 1

- [x] T010 [P] [US1] Extend command-pack coverage for `boundline-start` and `boundline-plan` across all assistants in `tests/contract/assistant_command_pack_contract.rs`
- [x] T011 [P] [US1] Add start/planning definition assertions in `tests/contract/assistant_command_definition_contract.rs`
- [x] T012 [P] [US1] Implement shell-enabled and chat-only start/planning scenarios in `tests/integration/assistant_shell_enabled_flow.rs` and `tests/integration/assistant_chat_fallback.rs`

### Implementation for User Story 1

- [x] T013 [P] [US1] Author Claude and Codex start/planning command files in `assistant/claude/commands/boundline-start.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/codex/commands/boundline-start.md`, and `assistant/codex/commands/boundline-plan.md`
- [x] T014 [P] [US1] Author Copilot start/planning prompt files in `assistant/copilot/prompts/boundline-start.prompt.md` and `assistant/copilot/prompts/boundline-plan.prompt.md`
- [x] T015 [US1] Document start/planning workflow, missing-input prompts, and run handoff in `assistant/README.md`

**Checkpoint**: User Story 1 is independently functional and delivers the MVP chat-first onboarding path.

---

## Phase 4: User Story 2 - Continue and Complete Work from Chat (Priority: P2)

**Goal**: Allow a user to continue an active workflow from chat, execute or guide the next bounded action, and summarize progress, failures, and next steps.

**Independent Test**: Invoke `/boundline-run`, `/boundline-step`, `/boundline-status`, and `/boundline-next` in a supported assistant surface, then confirm shell-enabled and chat-only flows both preserve context, surface terminal or recovery cues, and recommend the correct follow-up command.

### Tests for User Story 2

- [x] T016 [P] [US2] Extend command-pack coverage for `boundline-step`, `boundline-run`, `boundline-status`, and `boundline-next` across all assistants in `tests/contract/assistant_command_pack_contract.rs`
- [x] T017 [P] [US2] Add step/run/status/next definition assertions in `tests/contract/assistant_command_definition_contract.rs`
- [x] T018 [P] [US2] Implement shell-enabled run/status/next scenarios and non-success chat fallbacks in `tests/integration/assistant_shell_enabled_flow.rs` and `tests/integration/assistant_chat_fallback.rs`

### Implementation for User Story 2

- [x] T019 [P] [US2] Author Claude and Codex step/run/status/next command files in `assistant/claude/commands/boundline-step.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, and `assistant/claude/commands/boundline-next.md`, `assistant/codex/commands/boundline-step.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, and `assistant/codex/commands/boundline-next.md`
- [x] T020 [P] [US2] Author Copilot step/run/status/next prompt files in `assistant/copilot/prompts/boundline-step.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, and `assistant/copilot/prompts/boundline-next.prompt.md`
- [x] T021 [US2] Harden run and latest-trace summary cues for assistant routing in `src/cli/output.rs`, `src/cli/run.rs`, and `src/cli/inspect.rs`
- [x] T022 [US2] Extend workflow continuity and next-step guidance in `assistant/README.md`

**Checkpoint**: User Stories 1 and 2 both work, and users can continue bounded workflows from chat in shell-enabled and chat-only modes.

---

## Phase 5: User Story 3 - Inspect Prior Runs from Chat (Priority: P3)

**Goal**: Let a user inspect a completed or failed run from chat and understand outcome, recovery signals, and next action without reading raw trace files manually.

**Independent Test**: Generate a run, invoke `/boundline-inspect` with either a workspace or explicit trace reference, and confirm the assistant summarizes final status, recovery events, and trace-read failures correctly in both shell-enabled and chat-only modes.

### Tests for User Story 3

- [x] T023 [P] [US3] Extend command-pack coverage for `boundline-inspect` across all assistants in `tests/contract/assistant_command_pack_contract.rs`
- [x] T024 [P] [US3] Add inspect definition assertions and trace-read failure expectations in `tests/contract/assistant_command_definition_contract.rs`
- [x] T025 [P] [US3] Implement shell-enabled and chat-only inspection scenarios in `tests/integration/assistant_shell_enabled_flow.rs` and `tests/integration/assistant_chat_fallback.rs`

### Implementation for User Story 3

- [x] T026 [P] [US3] Author Claude and Codex inspection command files in `assistant/claude/commands/boundline-inspect.md` and `assistant/codex/commands/boundline-inspect.md`
- [x] T027 [P] [US3] Author Copilot inspection prompt file in `assistant/copilot/prompts/boundline-inspect.prompt.md`
- [x] T028 [US3] Refine explicit-trace and latest-trace inspection summaries for assistant interpretation in `src/cli/output.rs` and `src/cli/inspect.rs`
- [x] T029 [US3] Document inspection, trace selection, and trace-read recovery in `assistant/README.md`

**Checkpoint**: All user stories are independently functional, and inspection of prior runs works across all supported assistant surfaces.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Tighten documentation, formatting, and final validation across the complete assistant pack surface.

- [x] T030 [P] Update top-level discoverability and installation docs for assistant command packs in `README.md` and `assistant/README.md`
- [x] T031 [P] Add final cross-pack safety and formatting coverage in `tests/unit/assistant_assets.rs`
- [x] T032 Synchronize the implemented assistant flows with feature walkthroughs in `specs/003-assistant-command-packs/quickstart.md` and `assistant/README.md`
- [x] T033 Validate the documented assistant flows against `assistant/claude/commands/boundline-start.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-step.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-next.md`, `assistant/claude/commands/boundline-inspect.md`, `assistant/codex/commands/boundline-start.md`, `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-step.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-next.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-start.prompt.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-step.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, `assistant/copilot/prompts/boundline-next.prompt.md`, and `assistant/copilot/prompts/boundline-inspect.prompt.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies; start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user story work.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP onboarding path.
- **Phase 4: User Story 2**: Depends on Phase 2 and can proceed after the shared asset and CLI guarantees are stable; it does not require US1 implementation to be complete, but sequencing after US1 keeps the rollout aligned with story priority.
- **Phase 5: User Story 3**: Depends on Phase 2 and the inspection backend guarantees from Phase 2; it can be built independently of US2.
- **Phase 6: Polish**: Depends on all desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: No story dependency after Foundational.
- **US2 (P2)**: No strict story dependency after Foundational, but it should follow US1 for MVP sequencing and shared documentation coordination.
- **US3 (P3)**: No strict story dependency after Foundational; it reuses the same inspect backend and pack conventions.

### Within Each User Story

- Validation tasks MUST fail before implementation changes are considered complete.
- Contract coverage comes before authoring assistant asset files.
- Assistant asset files come before README guidance for the same story.
- CLI summary adjustments come before story sign-off when assistant interpretation depends on them.
- Each story is complete only when shell-enabled and chat-only flows both pass their independent test.

### Parallel Opportunities

- Setup: T002 and T003 can proceed in parallel after T001.
- Foundational: T005, T006, and T007 can proceed in parallel after T004; T008 and T009 can proceed once the shared conventions are fixed.
- US1: T010, T011, and T012 can proceed in parallel; T013 and T014 can proceed in parallel after the contract expectations are clear.
- US2: T016, T017, and T018 can proceed in parallel; T019 and T020 can proceed in parallel after the contract expectations are clear.
- US3: T023, T024, and T025 can proceed in parallel; T026 and T027 can proceed in parallel after the contract expectations are clear.
- Polish: T030 and T031 can proceed in parallel after all story work is complete.

---

## Parallel Example: User Story 1

```bash
# Launch US1 validation tasks together:
Task: "Extend command-pack coverage for boundline-start and boundline-plan in tests/contract/assistant_command_pack_contract.rs"
Task: "Add start/planning definition assertions in tests/contract/assistant_command_definition_contract.rs"
Task: "Implement shell-enabled and chat-only start/planning scenarios in tests/integration/assistant_shell_enabled_flow.rs and tests/integration/assistant_chat_fallback.rs"

# Launch independent asset authoring after contracts are stable:
Task: "Author Claude and Codex start/planning command files in assistant/claude/commands/ and assistant/codex/commands/"
Task: "Author Copilot start/planning prompt files in assistant/copilot/prompts/"
```

## Parallel Example: User Story 2

```bash
# Launch US2 validation tasks together:
Task: "Extend command-pack coverage for boundline-step, boundline-run, boundline-status, and boundline-next in tests/contract/assistant_command_pack_contract.rs"
Task: "Add step/run/status/next definition assertions in tests/contract/assistant_command_definition_contract.rs"
Task: "Implement shell-enabled run/status/next scenarios and non-success chat fallbacks in tests/integration/assistant_shell_enabled_flow.rs and tests/integration/assistant_chat_fallback.rs"

# Launch independent asset authoring after contracts are stable:
Task: "Author Claude and Codex step/run/status/next command files in assistant/claude/commands/ and assistant/codex/commands/"
Task: "Author Copilot step/run/status/next prompt files in assistant/copilot/prompts/"
```

## Parallel Example: User Story 3

```bash
# Launch US3 validation tasks together:
Task: "Extend command-pack coverage for boundline-inspect in tests/contract/assistant_command_pack_contract.rs"
Task: "Add inspect definition assertions and trace-read failure expectations in tests/contract/assistant_command_definition_contract.rs"
Task: "Implement shell-enabled and chat-only inspection scenarios in tests/integration/assistant_shell_enabled_flow.rs and tests/integration/assistant_chat_fallback.rs"

# Launch independent asset authoring after contracts are stable:
Task: "Author Claude and Codex inspection command files in assistant/claude/commands/ and assistant/codex/commands/"
Task: "Author Copilot inspection prompt file in assistant/copilot/prompts/"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate `tests/contract/assistant_command_pack_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, `tests/integration/assistant_shell_enabled_flow.rs`, and `tests/integration/assistant_chat_fallback.rs` for the start/planning slice.
5. Validate the onboarding path before expanding to workflow continuation and inspection.

### Incremental Delivery

1. Finish Setup and Foundational to establish the assistant asset surface, validation scaffolding, and CLI parsing guarantees.
2. Ship US1 to prove chat-first onboarding and bounded goal handoff.
3. Ship US2 to prove continued execution and next-step routing from chat.
4. Ship US3 to prove trace-backed inspection from chat.
5. Use Phase 6 to tighten documentation and final consistency without changing scope.

### Parallel Team Strategy

1. One engineer can own the shared README, test harness registration, and contract assertions.
2. A second engineer can own integration scaffolding and CLI output hardening in `src/cli/`.
3. After Foundational is stable, assistant asset authoring can split by environment: Claude/Codex on one side and Copilot on the other, with shared review for consistency.

---

## Notes

- Total tasks: 33.
- User story task counts: US1 = 6, US2 = 7, US3 = 7.
- Suggested MVP scope: through Phase 3 (User Story 1) only.
- All tasks use the required checklist format: checkbox, task ID, optional `[P]`, required story label in story phases, and exact file paths.