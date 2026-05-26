# Tasks: Session, Assistant, and Audit Fine-Tuning

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

## Phase 5: Session Audit Attribution Refinement

- [x] T017 [US5] Extend `SessionAuditActor` with mixed-route attribution fields and populate review-council participant routes in `src/domain/audit.rs` and `src/orchestrator/session_runtime.rs`.
- [x] T018 [US5] Surface mixed-route audit attribution in human-readable inspect output in `src/cli/output_trace_summary.rs`.
- [x] T019 [US5] Add an explicit audit-first projection to orchestrate NDJSON event envelopes in `src/cli/orchestrate.rs`.
- [x] T020 [US5] Update Copilot command-pack guidance to prefer the explicit audit projection in `assistant/prompts/copilot-command-pack.md`, `assistant/copilot/prompts/boundline-goal.prompt.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, and `assistant/copilot/prompts/boundline-run.prompt.md`.

## Phase 6: Audit-Focused Inspect Surface

- [x] T021 [US6] Add `inspect --audit` plumbing and the audit-focused renderer in `src/cli.rs`, `src/cli/inspect.rs`, and `src/cli/output_trace_summary.rs`.
- [x] T022 [US6] Update inspect command-pack guidance to route audit-trail requests to `--audit` in `assistant/copilot/prompts/boundline-inspect.prompt.md` and `assistant/*/commands/boundline-inspect.md`.
- [x] T023 [US5] Add or refresh focused tests for audit attribution, audit-first orchestrate metadata, and inspect audit rendering in `src/cli/output.rs` and `src/cli/orchestrate.rs`.
- [x] T024 [US6] Add or refresh focused CLI dispatch coverage for `inspect --audit` in `src/cli.rs`.

## Dependencies & Execution Order

- Session reference tasks (`T001-T005`) should complete before broad validation.
- Prompt routing tasks (`T008-T014`) can proceed independently once command safety boundaries are confirmed.
- Session audit attribution tasks (`T017-T020`) should land before the dedicated inspect audit surface.
- Audit inspect tasks (`T021-T024`) should close after the audit projection contract is in place.
- Final validation (`T016`) remains required after all implementation tasks, including the audit refinement work.

## Notes

- This task list documents already-implemented fine-tuning work for release traceability.
- Allowed follow-up command boundaries remain unchanged by prompt routing updates.
- Audit surfaces remain projections over persisted lifecycle and trace data; they do not replace the trace store as the execution authority.
