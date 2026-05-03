# Tasks: Governed Delivery With Canon Inside The Loop

**Input**: Design documents from `/Users/rt/workspace/boundline/specs/031-canon-delivery-loop/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required because this feature changes success
semantics for the primary session-native route, Canon-governed stop behavior,
assistant-visible follow-through, and the `0.31.0` release story. Modified or
new Rust files must remain above 95% coverage.

**Organization**: Tasks are grouped by user story so the real governed-delivery
proof, explicit stop behavior, shared follow-through, and release closeout can
be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.31.0` release boundary and the focused governed
delivery validation surfaces.

- [x] T001 Bump crate version to `0.31.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [x] T002 Register governed-delivery validation surfaces in `/Users/rt/workspace/boundline/tests/integration/governance_autopilot_flow.rs`, `/Users/rt/workspace/boundline/tests/integration/canon_governance_flow.rs`, `/Users/rt/workspace/boundline/tests/contract/governance_session_contract.rs`, and `/Users/rt/workspace/boundline/tests/unit/coverage_additional.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the delivery-completion gate that every governed story in this
slice depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Add governed delivery-completion gate helpers in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`
- [ ] T004 [P] Project delivery-gate evidence through persisted session state and read-side views in `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs`
- [x] T005 [P] Add foundational unit coverage for governed delivery-completion gating in `/Users/rt/workspace/boundline/tests/unit/coverage_additional.rs`

**Checkpoint**: Native runs can only terminate successfully when the runtime can
prove governance allows completion, a material diff exists, and validation has
passed.

---

## Phase 3: User Story 1 - Deliver A Governed Code Change (Priority: P1) 🎯 MVP

**Goal**: Prove one real governed delivery path on the primary native route.

**Independent Test**: Run a representative governed bug-fix workspace through
the native route and confirm Canon-governed framing or verify evidence, a real
workspace diff, passed validation, and terminal success on the same session and
trace story.

### Tests for User Story 1

- [x] T006 [P] [US1] Add integration and contract coverage for governed native-delivery success in `/Users/rt/workspace/boundline/tests/integration/governance_autopilot_flow.rs` and `/Users/rt/workspace/boundline/tests/contract/governance_session_contract.rs`
- [x] T007 [P] [US1] Add end-to-end Canon lineage coverage for the same governed delivery proof in `/Users/rt/workspace/boundline/tests/integration/canon_governance_flow.rs`

### Implementation for User Story 1

- [ ] T008 [US1] Require governed change-framing continuity and verify-stage governed evidence on the native session path in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/orchestrator/governance.rs`
- [x] T009 [US1] Gate terminal success on material changed files plus passed validation evidence in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` and `/Users/rt/workspace/boundline/src/fixture.rs`
- [ ] T010 [US1] Keep governed completion evidence visible on `run`, `status`, `next`, and `inspect` in `/Users/rt/workspace/boundline/src/domain/follow_through.rs`, `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs`

**Checkpoint**: One governed native flow now proves real delivery instead of
claiming success just because the final step finished.

---

## Phase 4: User Story 2 - Stop Safely When Delivery Is Not Credible (Priority: P2)

**Goal**: Make blocked governance, approval gates, no-diff execution, and
missing validation evidence stop explicitly instead of ending in false success.

**Independent Test**: Exercise governed runs that hit approval pending,
rejected packet, no material diff, and missing validation evidence, then verify
that each run stops with an explicit reason and preserved session or trace
evidence.

### Tests for User Story 2

- [ ] T011 [P] [US2] Add integration coverage for explicit governed stop conditions in `/Users/rt/workspace/boundline/tests/integration/governance_autopilot_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/canon_governance_flow.rs`
- [ ] T012 [P] [US2] Add unit coverage for no-diff and no-validation stop conditions in `/Users/rt/workspace/boundline/tests/unit/coverage_additional.rs`

### Implementation for User Story 2

- [x] T013 [US2] Stop terminal completion when material diff or validation evidence is missing in `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs`
- [ ] T014 [US2] Keep resume and follow-through guidance aligned with approval-gated governed delivery in `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/workflow.rs`

**Checkpoint**: Governed delivery stops explicitly when its delivery claim is
not credible, and operators can see how to continue.

---

## Phase 5: User Story 3 - Keep One Follow-Through Model Across Governed And Non-Governed Runs (Priority: P3)

**Goal**: Keep one shared follow-through story across governed native,
non-governed native, and explicit compatibility runs.

**Independent Test**: Compare governed native, non-governed native, and
explicit compatibility traces and verify that route ownership, governance state,
changed-files evidence, and `next_command` remain aligned on the same read-side
surfaces.

### Tests for User Story 3

- [ ] T015 [P] [US3] Extend contract coverage for governed follow-through and compatibility separation in `/Users/rt/workspace/boundline/tests/contract/trace_summary_contract.rs` and `/Users/rt/workspace/boundline/tests/contract/assistant_session_continuity_contract.rs`

### Implementation for User Story 3

- [ ] T016 [US3] Surface governed delivery gate cues without hiding explicit compatibility ownership in `/Users/rt/workspace/boundline/src/domain/follow_through.rs`, `/Users/rt/workspace/boundline/src/domain/session.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs`

**Checkpoint**: Canon participation improves the same Boundline product story
instead of creating a parallel one.

---

## Phase 6: User Story 4 - Ship Governed Delivery As 0.31.0 (Priority: P4)

**Goal**: Ship runtime behavior, docs, assistant guidance, roadmap, and release
validation as one coherent governed-delivery story.

**Independent Test**: Follow the updated `0.31.0` operator and assistant
guidance on a representative workspace, then confirm version metadata, docs,
coverage, lint, and formatting align with shipped behavior.

### Implementation for User Story 4

- [x] T017 [US4] Update the `0.31.0` governed-delivery release story in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/getting-started.md`, `/Users/rt/workspace/boundline/docs/configuration.md`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/ROADMAP.md`, and `/Users/rt/workspace/boundline/CHANGELOG.md`
- [x] T018 [US4] Update assistant guidance and regenerate agent context for governed delivery in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/boundline-inspect.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-status.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-next.md`, `/Users/rt/workspace/boundline/assistant/codex/commands/boundline-inspect.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-status.prompt.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-next.prompt.md`, `/Users/rt/workspace/boundline/assistant/copilot/prompts/boundline-inspect.prompt.md`, and `/Users/rt/workspace/boundline/AGENTS.md`

**Checkpoint**: Maintainers and assistants now describe one coherent `0.31.0`
story where Canon improves real delivery inside Boundline.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout.

- [x] T019 Run focused coverage for modified or created Rust files, refresh `/Users/rt/workspace/boundline/lcov.info`, verify modified-Rust coverage remains above 95%, resolve remaining `cargo clippy` issues, run `cargo fmt --all`, and finish with clean validation for touched files under `/Users/rt/workspace/boundline/src/` and `/Users/rt/workspace/boundline/tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and should align with the runtime story
  delivered by US1.
- User Story 3 depends on Foundational and should align with the settled
  governed-delivery and stop-condition behavior from US1 and US2.
- User Story 4 depends on the runtime and follow-through behavior from US1-US3.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and the terminal-success behavior settled by US1.
- **US3**: Depends on Foundational and should align with US1 plus US2 before sign-off.
- **US4**: Depends on the settled product story from US1-US3.

### Within Each User Story

- Focused validation tasks should exist before implementation is considered complete.
- Runtime gating comes before read-side wording and release docs.
- Assistant guidance and docs follow the settled runtime behavior.

### Parallel Opportunities

- T004 and T005 can run in parallel after T003.
- Validation tasks within each user story marked `[P]` can run in parallel.
- T017 and T018 can run in parallel once runtime behavior is stable.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that one governed native flow now requires both real code mutation
   and passed validation before terminal success.

### Incremental Delivery

1. Reserve `0.31.0` and the new governed-delivery validation surfaces.
2. Add the foundational delivery-completion gate using existing task-context evidence.
3. Prove one governed native flow reaches real code diff, governance evidence, and validation-backed success.
4. Stop explicitly on approval pending, blocked governance, no diff, or missing validation evidence.
5. Align follow-through, docs, assistant guidance, and roadmap wording.
6. Close with coverage, clippy, fmt, and final release validation.

## Notes

- `[P]` tasks touch different files or independent validation surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.31.0` as the first task.
- T017 intentionally includes all impacted docs and changelog updates as one release-guidance task.
- T019 intentionally reserves the final modified-Rust coverage check, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.