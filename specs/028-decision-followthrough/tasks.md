# Tasks: Decision Continuity And Guided Follow-Through

**Input**: Design documents from `/specs/028-decision-followthrough/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes executable `status`, `next`, and `inspect` behavior, continuity authority, trace or session evidence precedence, and assistant-facing guidance. Coverage refresh for modified or created Rust files is part of release closeout.

**Organization**: Tasks are grouped by user story so guided next-action projection, continuity preservation, and release closeout can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.28.0` release boundary and register the new test surfaces

- [X] T001 Bump crate version to `0.28.0` in `Cargo.toml` and `Cargo.lock`
- [X] T002 Reuse the existing test harness surfaces and extend focused follow-through coverage in `tests/unit/cli_output.rs`, `tests/unit/compatibility_continuity.rs`, `tests/contract/trace_summary_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, and `tests/contract/assistant_session_continuity_contract.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared decision-continuity primitives and evidence-precedence helpers required by every story in this slice

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T003 Create shared follow-through and continuity-evidence models in `src/domain/follow_through.rs`, `src/domain.rs`, and `src/lib.rs`
- [X] T004 [P] Integrate continuity-evidence precedence helpers for session-native versus compatibility authority in `src/domain/follow_through.rs` and `src/cli/output.rs`
- [X] T005 [P] Add shared rendering helpers for guided follow-through in `src/cli/output.rs`
- [X] T006 [P] Add foundational unit coverage for follow-through projection and compatibility continuity in `src/domain/follow_through.rs`, `tests/unit/compatibility_continuity.rs`, and `tests/unit/cli_output.rs`

**Checkpoint**: Shared follow-through primitives exist, one winning evidence source can be projected, and CLI surfaces can render the guidance without changing the operator workflow.

---

## Phase 3: User Story 1 - Explain The Next Bounded Action (Priority: P1) 🎯 MVP

**Goal**: Let operators see what Boundline should do next and why that next step is credible on `status`, `next`, and `inspect`.

**Independent Test**: Run representative retry, replan, blocked, and inspect-only flows, then confirm that `status`, `next`, and `inspect` each project one explicit next bounded action or stop condition together with its supporting evidence.

### Tests for User Story 1

- [X] T007 [P] [US1] Add focused contract coverage for guided next-action projection on runtime follow-up surfaces in `tests/contract/trace_summary_contract.rs`
- [X] T008 [P] [US1] Add focused unit coverage for retry-style and inspect-style follow-through rendering in `tests/unit/cli_output.rs`

### Implementation for User Story 1

- [X] T009 [US1] Project guided next-action and evidence headlines into session status rendering in `src/domain/follow_through.rs` and `src/cli/output.rs`
- [X] T010 [US1] Extend trace summaries and inspect rendering with the same guided follow-through story in `src/domain/follow_through.rs` and `src/cli/output.rs`
- [X] T011 [US1] Replace generic follow-up wording with bounded guidance or explicit stop conditions in `src/domain/follow_through.rs` and `tests/unit/cli_output.rs`

**Checkpoint**: Operators can identify one credible next bounded action or explicit stop condition from the same follow-up surfaces they already use.

---

## Phase 4: User Story 2 - Preserve Decision Continuity Across Reload And Follow-Up (Priority: P2)

**Goal**: Keep the guided follow-through story coherent after session reloads and explicit compatibility follow-up by reusing persisted session and trace evidence with visible precedence.

**Independent Test**: Persist a non-terminal or inspect-only follow-up state, reload from the saved session or workspace trace, and confirm that `status`, `next`, and `inspect` preserve the correct continuity authority and winning evidence source.

### Tests for User Story 2

- [X] T012 [P] [US2] Add coverage for continuity-evidence precedence and compatibility authority boundaries in `tests/contract/trace_summary_contract.rs` and `tests/unit/compatibility_continuity.rs`
- [X] T013 [P] [US2] Add focused compatibility follow-up continuity coverage in `tests/unit/compatibility_continuity.rs` and `tests/unit/cli_output.rs`

### Implementation for User Story 2

- [X] T014 [US2] Reuse existing persisted session continuity fields through the shared follow-through projection in `src/domain/follow_through.rs` and `src/cli/output.rs`
- [X] T015 [US2] Reuse authoritative trace evidence for compatibility follow-up without losing route or continuity ownership in `src/domain/follow_through.rs` and `src/cli/output.rs`
- [X] T016 [US2] Align compatibility follow-up presentation with explicit evidence precedence and authority boundaries in `src/domain/follow_through.rs`, `src/cli/output.rs`, and `tests/unit/compatibility_continuity.rs`

**Checkpoint**: Session reloads and explicit compatibility follow-up preserve one coherent guidance story with visible continuity authority.

---

## Phase 5: User Story 3 - Ship Guided Follow-Through As One Release (Priority: P3)

**Goal**: Ship runtime behavior, assistant guidance, docs, version metadata, and release notes as one coherent `0.28.0` follow-through story.

**Independent Test**: Follow the updated docs and assistant guidance on a representative workspace, then confirm the observed runtime output matches the documented continuity and next-action behavior.

### Tests for User Story 3

- [X] T017 [P] [US3] Extend assistant-guidance and continuity contract coverage for the new follow-through vocabulary in `tests/contract/assistant_session_continuity_contract.rs`, `tests/contract/assistant_command_definition_contract.rs`, and `tests/contract/assistant_command_pack_contract.rs`

### Implementation for User Story 3

- [X] T018 [US3] Update the guided follow-through operator story and release notes in `README.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `CONTRIBUTING.md`, `ROADMAP.md`, and `CHANGELOG.md`
- [X] T019 [US3] Update assistant guidance and generated agent context for the follow-through surface in `assistant/README.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-next.md`, `assistant/claude/commands/boundline-inspect.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-next.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, `assistant/copilot/prompts/boundline-next.prompt.md`, `assistant/copilot/prompts/boundline-inspect.prompt.md`, and `AGENTS.md`

**Checkpoint**: Maintainers and assistants have one coherent `0.28.0` story for guided follow-through, continuity authority, and release behavior.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout

- [X] T020 Run focused coverage for modified or created Rust files, refresh `lcov.info`, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `src/` and `tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the follow-through projection delivered by US1.
- User Story 3 depends on Foundational and should reconcile with the settled runtime story from US1 and US2.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and should align with the guided follow-through behavior delivered by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before final sign-off.

### Within Each User Story

- Contract and integration validations should exist before implementation is considered complete.
- Shared continuity state and evidence-precedence helpers come before wording cleanup.
- CLI projection should settle before docs and assistant guidance are finalized.
- Compatibility and native continuity boundaries must remain explicit before story sign-off.

### Parallel Opportunities

- T004, T005, and T006 can run in parallel after T003.
- Test tasks within each user story marked `[P]` can run in parallel.
- T018 and T019 can run in parallel once runtime behavior is stable.

---

## Parallel Example: User Story 1

```bash
# Launch User Story 1 validation work together:
Task: "Add contract coverage for guided next-action projection on runtime follow-up surfaces in tests/contract/follow_through_surface_contract.rs"
Task: "Add integration coverage for retry, replanning, and no-credible-next-action follow-up in tests/integration/follow_through_status_flow.rs and tests/integration/follow_through_failure_flow.rs"

# Launch session and trace projection work together after the foundational model exists:
Task: "Project guided next-action and evidence headlines into session status views in src/domain/session.rs, src/cli/session.rs, and src/cli/output.rs"
Task: "Extend trace summaries and inspect rendering with the same guided follow-through story in src/domain/trace.rs, src/cli/inspect.rs, and src/cli/output.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that guided next-action projection works on representative success and non-success follow-up paths.

### Incremental Delivery

1. Reserve `0.28.0` and register the new follow-through test surfaces.
2. Tighten shared follow-through state, continuity-evidence precedence, and rendering helpers.
3. Project guided next actions through session-native and inspect surfaces.
4. Preserve decision continuity across reload and explicit compatibility follow-up.
5. Ship docs, assistant guidance, roadmap, contributor guidance, changelog, and refreshed agent context.
6. Close with coverage refresh, clippy cleanup, formatting, and final validation.

## Notes

- `[P]` tasks touch different files or independent surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.28.0` as the first task.
- T018 intentionally includes impacted docs and changelog updates as one release-guidance task.
- T020 intentionally reserves the final coverage, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.