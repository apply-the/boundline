# Tasks: Session and Assistant Fine-Tuning

**Input**: Design documents from `/specs/064-session-assistant-fine-tuning/`  
**Prerequisites**: spec.md (required), plan.md (required)

**Tests**: This slice requires formatting, lint, and representative behavioral coverage for touched session and prompt surfaces.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel
- **[Story]**: Related story (`US1`, `US2`, `US3`, `US4`)

## Phase 1: Session Reference Fine-Tuning

- [x] T001 [US1] Update session reference generation to `YYYYMMDD-NNN-slug` in `src/domain/session.rs`.
- [x] T002 [US1] Add deterministic date-prefix derivation helper and keep slug constraints in `src/domain/session.rs`.
- [x] T003 [US1] Align session creation flows to use same-date sequence counting in `src/cli/session.rs`.
- [x] T004 [US1] Align governance session initialization with the new session reference contract in `src/cli/govern.rs`.
- [x] T005 [US1] Update or add tests for new reference shape and sequence behavior in `src/domain/session.rs` and related test surfaces.

## Phase 2: Local Install Flow Fine-Tuning

- [x] T006 [US2] Create local install refresh script in `scripts/install-local.sh`.
- [x] T007 [US2] Ensure script performs release build and refreshes local Homebrew-bound binary path.

## Phase 3: Prompt Routing Fine-Tuning

- [x] T008 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-goal.prompt.md`.
- [x] T009 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-plan.prompt.md`.
- [x] T010 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-step.prompt.md`.
- [x] T011 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-run.prompt.md`.
- [x] T012 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-status.prompt.md`.
- [x] T013 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-inspect.prompt.md`.
- [x] T014 [US3] Update next-step routing to two-button pattern in `assistant/copilot/prompts/boundline-next.prompt.md`.

## Phase 4: Validation and Behavioral Alignment

- [x] T015 [US4] Resolve failing assertions for clarification semantics in `src/cli.rs` test surfaces.
- [x] T016 [US4] Run lint and test validation and confirm no regression in updated scope.

## Dependencies & Execution Order

- Session reference tasks (`T001-T005`) should complete before broad validation.
- Prompt routing tasks (`T008-T014`) can proceed independently once command safety boundaries are confirmed.
- Final validation (`T016`) closes the slice after all implementation tasks.

## Notes

- This task list documents already-implemented fine-tuning work for release traceability.
- Allowed follow-up command boundaries remain unchanged by prompt routing updates.
