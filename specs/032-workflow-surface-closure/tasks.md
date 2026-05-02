# Tasks: Product Unification And Surface Closure

**Input**: Design documents from `/Users/rt/workspace/synod/specs/032-workflow-surface-closure/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required because this feature changes assistant
workflow surfaces, workflow-facing CLI summaries, compatibility-boundary
guidance, and the `0.32.0` release story. Modified or newly created Rust files
 must remain above 95% coverage.

**Organization**: Tasks are grouped by user story so workflow-first assistant
guidance, workflow routing inspectability, compatibility-boundary closure, and
release closeout can each be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Reserve the `0.32.0` release boundary and register the workflow
surface validation targets for this slice.

- [x] T001 Bump crate version to `0.32.0` in `/Users/rt/workspace/synod/Cargo.toml` and `/Users/rt/workspace/synod/Cargo.lock`
- [x] T002 Register workflow-surface validation coverage in `/Users/rt/workspace/synod/tests/contract/assistant_command_definition_contract.rs`, `/Users/rt/workspace/synod/tests/contract/assistant_session_continuity_contract.rs`, `/Users/rt/workspace/synod/tests/contract/workflow_command_surface_contract.rs`, and `/Users/rt/workspace/synod/tests/unit/workflow_session_projection.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Close the shared workflow product-surface gap that every story in
this slice depends on.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Add explicit workflow product-surface cues and workflow-discovery explanation updates in `/Users/rt/workspace/synod/src/cli/workflow.rs`
- [x] T004 [P] Add foundational unit or contract coverage for workflow route projection and primary-versus-subordinate path cues in `/Users/rt/workspace/synod/tests/unit/workflow_session_projection.rs` and `/Users/rt/workspace/synod/tests/contract/workflow_command_surface_contract.rs`

**Checkpoint**: Workflow output now states the same primary Synod product story
as direct native execution before assistant-pack work begins.

---

## Phase 3: User Story 1 - Start And Continue Workflows Through Unified Assistant Surfaces (Priority: P1) 🎯 MVP

**Goal**: Ship first-class workflow assistant guidance for the shipped assistant
families.

**Independent Test**: Validate that Claude, Codex, Copilot, and Gemini guidance
can discover and continue named workflows using bounded Synod commands rather
than raw undocumented fallback instructions.

### Tests for User Story 1

- [x] T005 [P] [US1] Extend assistant asset contract coverage for workflow discovery and continuation commands in `/Users/rt/workspace/synod/tests/contract/assistant_command_definition_contract.rs`
- [x] T006 [P] [US1] Extend assistant continuity coverage for workflow-first follow-through guidance in `/Users/rt/workspace/synod/tests/contract/assistant_session_continuity_contract.rs`

### Implementation for User Story 1

- [x] T007 [US1] Add workflow assistant guidance to `/Users/rt/workspace/synod/assistant/README.md` and `/Users/rt/workspace/synod/assistant/gemini/README.md`
- [x] T008 [P] [US1] Add Claude workflow assistant command files in `/Users/rt/workspace/synod/assistant/claude/commands/synod-workflow-list.md`, `/Users/rt/workspace/synod/assistant/claude/commands/synod-workflow-run.md`, `/Users/rt/workspace/synod/assistant/claude/commands/synod-workflow-status.md`, `/Users/rt/workspace/synod/assistant/claude/commands/synod-workflow-resume.md`, and `/Users/rt/workspace/synod/assistant/claude/commands/synod-workflow-inspect.md`
- [x] T009 [P] [US1] Add Codex workflow assistant command files in `/Users/rt/workspace/synod/assistant/codex/commands/synod-workflow-list.md`, `/Users/rt/workspace/synod/assistant/codex/commands/synod-workflow-run.md`, `/Users/rt/workspace/synod/assistant/codex/commands/synod-workflow-status.md`, `/Users/rt/workspace/synod/assistant/codex/commands/synod-workflow-resume.md`, and `/Users/rt/workspace/synod/assistant/codex/commands/synod-workflow-inspect.md`
- [x] T010 [P] [US1] Add Copilot workflow prompt files in `/Users/rt/workspace/synod/assistant/copilot/prompts/synod-workflow-list.prompt.md`, `/Users/rt/workspace/synod/assistant/copilot/prompts/synod-workflow-run.prompt.md`, `/Users/rt/workspace/synod/assistant/copilot/prompts/synod-workflow-status.prompt.md`, `/Users/rt/workspace/synod/assistant/copilot/prompts/synod-workflow-resume.prompt.md`, and `/Users/rt/workspace/synod/assistant/copilot/prompts/synod-workflow-inspect.prompt.md`

**Checkpoint**: A workflow can now be discovered and continued from any shipped
assistant surface without dropping to undocumented raw CLI usage.

---

## Phase 4: User Story 2 - Inspect Workflow Routing And Assistant Binding (Priority: P2)

**Goal**: Make workflow-facing output expose the same route, binding, and
bounded next-step cues as the rest of Synod.

**Independent Test**: Run representative workflows and verify that workflow
status or inspect surfaces route ownership, route-config projection, and
assistant-binding context explicitly.

### Tests for User Story 2

- [x] T011 [P] [US2] Extend workflow command-surface and discovery contracts for route projection cues in `/Users/rt/workspace/synod/tests/contract/workflow_command_surface_contract.rs` and `/Users/rt/workspace/synod/tests/contract/workflow_discovery_contract.rs`
- [x] T012 [P] [US2] Extend workflow integration coverage for route projection and assistant-binding visibility in `/Users/rt/workspace/synod/tests/integration/workflow_discovery.rs`, `/Users/rt/workspace/synod/tests/integration/workflow_layer_resume.rs`, and `/Users/rt/workspace/synod/tests/integration/workflow_follow_through.rs`

### Implementation for User Story 2

- [x] T013 [US2] Keep workflow reports explicitly aligned with session route projection and bounded next-step cues in `/Users/rt/workspace/synod/src/cli/workflow.rs` and `/Users/rt/workspace/synod/src/cli/output.rs`
- [x] T014 [US2] Extend workflow-facing wording for assistant-binding mismatch and route authority in `/Users/rt/workspace/synod/src/cli/workflow.rs` and `/Users/rt/workspace/synod/src/orchestrator/session_runtime.rs`

**Checkpoint**: Workflow-native output is inspectable enough that operators do
not need to reconstruct route or binding state from config files.

---

## Phase 5: User Story 3 - Keep One Primary Product Story Across Workflow And Compatibility Paths (Priority: P3)

**Goal**: Keep workflow and direct native execution primary while preserving
explicit compatibility ownership as a subordinate route.

**Independent Test**: Compare workflow-native and compatibility follow-up
surfaces and verify that the route boundary remains explicit in both runtime
output and assistant guidance.

### Tests for User Story 3

- [x] T015 [P] [US3] Extend compatibility-boundary coverage in `/Users/rt/workspace/synod/tests/integration/workflow_layer_compat.rs`, `/Users/rt/workspace/synod/tests/integration/workflow_follow_through_compat.rs`, and `/Users/rt/workspace/synod/tests/contract/assistant_session_continuity_contract.rs`

### Implementation for User Story 3

- [x] T016 [US3] Align compatibility-boundary and product-identity wording in `/Users/rt/workspace/synod/assistant/README.md`, `/Users/rt/workspace/synod/src/cli/workflow.rs`, and `/Users/rt/workspace/synod/src/cli/session.rs`

**Checkpoint**: Operators can tell from one glance whether they are on a
primary Synod path or on the subordinate compatibility route.

---

## Phase 6: User Story 4 - Ship Product Closure As 0.32.0 (Priority: P4)

**Goal**: Ship runtime behavior, assistant guidance, docs, roadmap, and release
artifacts as one coherent `0.32.0` product story.

**Independent Test**: Follow the updated workflow-first docs on a representative
workspace and confirm the release surfaces match the shipped runtime behavior.

### Implementation for User Story 4

- [x] T017 [US4] Update the `0.32.0` release story in `/Users/rt/workspace/synod/README.md`, `/Users/rt/workspace/synod/docs/getting-started.md`, `/Users/rt/workspace/synod/docs/configuration.md`, `/Users/rt/workspace/synod/CONTRIBUTING.md`, `/Users/rt/workspace/synod/ROADMAP.md`, and `/Users/rt/workspace/synod/CHANGELOG.md`
- [x] T018 [US4] Regenerate assistant or agent context for product closure in `/Users/rt/workspace/synod/AGENTS.md`

**Checkpoint**: Maintainers and assistants now describe one coherent `0.32.0`
story where Synod owns the product surface and Canon remains secondary.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finish release-quality validation and closeout.

- [x] T019 Run focused validation for `/Users/rt/workspace/synod/src/cli/workflow.rs` and any other modified or newly created Rust files, refresh `/Users/rt/workspace/synod/lcov.info`, verify modified-Rust coverage remains above 95%, resolve remaining `cargo clippy` issues, and run `cargo fmt --all`

---

## Dependencies & Execution Order

### Phase Dependencies

- Setup (Phase 1) starts immediately.
- Foundational (Phase 2) depends on Setup and blocks all user story work.
- User Story 1 depends on Foundational and is the MVP path.
- User Story 2 depends on Foundational and the settled workflow output shape from US1.
- User Story 3 depends on Foundational and should align with the route and assistant-boundary wording from US1 and US2.
- User Story 4 depends on the settled runtime and assistant story from US1-US3.
- Polish depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No user-story dependency after Foundational.
- **US2**: Depends on Foundational and the workflow-first assistant story from US1.
- **US3**: Depends on Foundational and should align with the settled workflow routing story from US2.
- **US4**: Depends on the final product wording from US1-US3.

### Within Each User Story

- Contract or integration coverage should exist before a story is considered complete.
- Shared runtime wording comes before release docs and assistant context closeout.
- Product-boundary wording must stay consistent across workflow, native, and compatibility surfaces.

### Parallel Opportunities

- T004 can run in parallel with coverage design inside T002 once T003 is defined.
- T008, T009, and T010 can run in parallel after shared workflow guidance is settled in T007.
- T011 and T012 can run in parallel.
- T017 and T018 can run in parallel once runtime and assistant behavior is stable.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate that workflow discovery and continuation now exist as first-class
   assistant surfaces.

### Incremental Delivery

1. Reserve `0.32.0` and the workflow surface validation targets.
2. Make workflow output explicitly state the primary Synod product story.
3. Add first-class workflow guidance for the shipped assistant families.
4. Extend workflow output and tests for route and binding inspectability.
5. Align compatibility-boundary wording and release docs.
6. Close with coverage, clippy, fmt, and refreshed release artifacts.

## Notes

- `[P]` tasks touch different files or independent validation surfaces and can be split safely.
- T001 intentionally reserves the version bump to `0.32.0` as the first task.
- T017 intentionally includes all impacted docs plus changelog updates as one release-guidance task.
- T019 intentionally reserves the final modified-Rust coverage check, `cargo clippy`, and `cargo fmt` closeout after runtime, tests, and docs are complete.