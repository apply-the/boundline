# Tasks: Chat-First Host-Integrated Runtime

**Input**: Design documents from `/specs/045-chat-first-runtime/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/host-command-json.md, quickstart.md

**Tests**: This slice changes operator-facing runtime contracts for assistant hosts. Add contract, integration, and unit coverage for structured output, continuity recovery, and assistant command-pack guidance.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare focused test anchors and feature registration before changing runtime behavior.

- [ ] T001 [P] Register the new host-runtime feature tests in tests/contract.rs and tests/integration.rs
- [ ] T002 [P] Extend tests/support/workspace_fixture.rs with JSON decoding helpers for host command output

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the reusable structured-output primitives shared by all host-facing commands.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T003 Create the reusable host command envelope and JSON rendering helpers in src/cli/output.rs
- [ ] T004 [P] Extend session, run, and inspect report structs to carry structured payloads in src/cli/session.rs, src/cli/run.rs, and src/cli/inspect.rs
- [ ] T005 [P] Add foundational unit coverage for host output helpers in src/cli/output.rs

**Checkpoint**: Structured host output can be rendered without changing command semantics.

---

## Phase 3: User Story 1 - Continue Delivery From Host Chat (Priority: P1) 🎯 MVP

**Goal**: Let host chats invoke the existing session-native lifecycle and inspection commands and consume a stable structured response.

**Independent Test**: Run `start`, `capture`, `plan`, `status`, `next`, `run`, and `inspect` with structured output enabled and verify that the envelope contains the expected session or trace payload plus the original rendered text.

### Tests for User Story 1

- [ ] T006 [P] [US1] Add contract coverage for the structured host command surface in tests/contract/host_command_output_contract.rs
- [ ] T007 [P] [US1] Add integration coverage for structured session-native command output in tests/integration/host_session_runtime_flow.rs
- [ ] T008 [P] [US1] Add integration coverage for structured run/inspect output and one non-success path in tests/integration/host_trace_runtime_flow.rs

### Implementation for User Story 1

- [ ] T009 [US1] Add structured-output flags and dispatch handling for `start`, `capture`, `flow`, `plan`, `step`, `run`, `status`, `next`, and `inspect` in src/cli.rs
- [ ] T010 [US1] Attach `SessionStatusView` payloads to the lifecycle commands in src/cli/session.rs
- [ ] T011 [US1] Attach `TraceSummaryView` payloads to run and inspect commands in src/cli/run.rs and src/cli/inspect.rs

**Checkpoint**: Host chats can drive the existing lifecycle through structured command responses without relying on ad hoc text parsing.

---

## Phase 4: User Story 2 - Resume From Persisted Workspace State (Priority: P2)

**Goal**: Preserve continuity, blocked states, and compatibility follow-up data in the structured host contract.

**Independent Test**: Lose the visible chat context, then use `status`, `next`, and `inspect` with structured output to recover the active workspace state or an explicit resume failure reason.

### Tests for User Story 2

- [ ] T012 [P] [US2] Add contract assertions for continuity authority, compatibility follow-up, and resume failure guidance in tests/contract/host_command_output_contract.rs
- [ ] T013 [P] [US2] Add integration coverage for structured resume and compatibility continuity in tests/integration/host_session_runtime_flow.rs
- [ ] T014 [P] [US2] Add integration coverage for invalid session or trace recovery in tests/integration/host_trace_runtime_flow.rs

### Implementation for User Story 2

- [ ] T015 [US2] Ensure structured session output preserves continuity authority, compatibility follow-up, and next-command guidance in src/cli/session.rs and src/cli/output.rs
- [ ] T016 [US2] Ensure structured run/inspect output preserves trace refs, terminal reasons, and recovery guidance in src/cli/run.rs and src/cli/inspect.rs

**Checkpoint**: Host chats can recover persisted delivery state and explicit failure reasons from structured output alone.

---

## Phase 5: User Story 3 - Keep Bootstrap And Automation Explicit (Priority: P3)

**Goal**: Align assistant command packs and repo guidance with the new structured shell-enabled path while preserving chat-only fallback and setup boundaries.

**Independent Test**: Read the assistant command assets for start/plan/step/run/status/next/inspect and verify that shell-enabled paths prefer structured output while chat-only fallback still uses copyable plain-text commands and pasted output.

### Tests for User Story 3

- [ ] T017 [P] [US3] Add contract assertions for structured shell-enabled guidance in tests/contract/assistant_command_definition_contract.rs and tests/contract/assistant_session_continuity_contract.rs
- [ ] T018 [P] [US3] Add integration coverage for assistant shell-enabled and chat-fallback host flows in tests/integration/assistant_shell_enabled_flow.rs and tests/integration/assistant_chat_fallback.rs

### Implementation for User Story 3

- [ ] T019 [US3] Update assistant shell-enabled guidance for start/plan/step/run/status/next/inspect in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, and assistant/copilot/prompts/
- [ ] T020 [US3] Keep bootstrap and chat-only fallback guidance explicit in assistant/README.md and assistant/gemini/README.md

**Checkpoint**: The repository-managed assistant packs describe the structured host path without hiding chat-only fallback or setup boundaries.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize validation and clean up touched runtime surfaces.

- [ ] T021 [P] Refresh feature docs in specs/045-chat-first-runtime/quickstart.md and any touched runtime guidance files
- [ ] T022 [P] Add extra unit or integration coverage for touched host-output edge cases in src/cli/output.rs, tests/integration/host_session_runtime_flow.rs, and tests/integration/host_trace_runtime_flow.rs
- [ ] T023 Resolve formatting and cleanup in src/cli.rs, src/cli/output.rs, src/cli/session.rs, src/cli/run.rs, src/cli/inspect.rs, assistant/, and tests/
- [ ] T024 Run focused validation from specs/045-chat-first-runtime/quickstart.md plus `cargo test --no-run --all-targets --all-features`

---

## Dependencies & Execution Order

### Phase Dependencies

- Phase 1 has no dependencies.
- Phase 2 depends on Phase 1 and blocks all story work.
- Phase 3 depends on Phase 2.
- Phase 4 depends on Phase 3 because it preserves the same host contract through resume and failure paths.
- Phase 5 depends on Phases 3 and 4 because assistant guidance must match the implemented host contract.
- Phase 6 depends on all implementation phases.

### User Story Dependencies

- **US1 (P1)**: First MVP slice; no dependency on other stories once foundational helpers land.
- **US2 (P2)**: Depends on the structured host contract from US1.
- **US3 (P3)**: Depends on the command contract being stable enough to document in assistant packs.

### Parallel Opportunities

- T001 and T002 can run in parallel.
- T004 and T005 can run in parallel after T003 defines the shared envelope shape.
- Tests within each story marked `[P]` can run in parallel.
- T017 and T018 can run in parallel once the command contract is stable.

## Parallel Example: User Story 1

```bash
Task: "Add contract coverage for the structured host command surface in tests/contract/host_command_output_contract.rs"
Task: "Add integration coverage for structured session-native command output in tests/integration/host_session_runtime_flow.rs"
Task: "Add integration coverage for structured run/inspect output and one non-success path in tests/integration/host_trace_runtime_flow.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Setup and Foundational phases.
2. Deliver the structured command contract for the existing lifecycle and inspection commands.
3. Validate that a host can run the workflow through machine-readable responses without losing the text surface.

### Incremental Delivery

1. Ship the structured command envelope and JSON-capable dispatch.
2. Preserve continuity and recovery semantics in the same contract.
3. Update assistant guidance to prefer the structured shell-enabled path.

## Notes

- Mark tasks as `[X]` when completed during implementation.
- `[P]` means different files or independent validation work.
- The feature is not complete until the structured host contract, continuity behavior, assistant guidance, and focused validations all agree.