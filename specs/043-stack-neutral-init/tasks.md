# Tasks: Stack-Neutral Workspace Entry

**Input**: Design documents from `/specs/043-stack-neutral-init/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for this feature because it changes native entry readiness, init-time routing defaults, bounded file mutation, and CLI-visible output.

**Organization**: Tasks are grouped by user story so each slice can be implemented and validated independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare shared fixtures and module surfaces used by the feature.

- [x] T001 [P] Extend stack-neutral workspace fixtures in tests/support/workspace_fixture.rs for empty, non-Rust, Git, and mixed-stack repositories
- [x] T002 [P] Export the new hygiene policy surface in src/domain.rs and src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Create the shared policy surfaces required by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Create reusable assistant default-model helpers in src/domain/configuration.rs
- [x] T004 Create technology and tool hygiene policy plus merge primitives in src/domain/workspace_hygiene.rs
- [x] T005 [P] Add foundational unit coverage for assistant defaults and hygiene policy in src/domain/configuration.rs and src/domain/workspace_hygiene.rs

**Checkpoint**: Shared bootstrap defaults exist and are testable.

---

## Phase 3: User Story 1 - Start From A Stack-Neutral Workspace (Priority: P1) 🎯 MVP

**Goal**: Let empty and non-Rust repositories enter the native Boundline workflow without a Rust-specific prerequisite.

**Independent Test**: Verify `boundline doctor --workspace` and direct native `boundline run --goal` on empty and non-Rust workspaces without `Cargo.toml`.

### Tests for User Story 1

- [x] T006 [P] [US1] Add integration coverage for stack-neutral workspace readiness in tests/integration/stack_neutral_workspace_flow.rs
- [x] T007 [P] [US1] Add integration coverage for direct native run bootstrap without `Cargo.toml` in tests/integration/stack_neutral_workspace_flow.rs
- [x] T008 [P] [US1] Add CLI contract assertions for generic workspace diagnostics in tests/contract/stack_neutral_init_contract.rs

### Implementation for User Story 1

- [x] T009 [US1] Replace Rust-specific workspace readiness assumptions in src/cli/diagnostics.rs
- [x] T010 [US1] Align native direct-run dispatch with stack-neutral diagnostics in src/cli.rs and src/cli/run.rs
- [x] T011 [US1] Update diagnostics unit coverage in src/cli/diagnostics.rs for empty and non-Rust workspaces

**Checkpoint**: Empty and non-Rust repositories can reach the native planning path or stop explicitly for credibility reasons.

---

## Phase 4: User Story 2 - Choose Assistant Target With Credible Model Defaults (Priority: P2)

**Goal**: Seed deterministic route models during init once the operator selects Claude, Copilot, Codex, or Gemini.

**Independent Test**: Run `boundline init` with supported assistant targets and no explicit routes, then inspect the written config and init output.

### Tests for User Story 2

- [x] T012 [P] [US2] Add init integration coverage for assistant-target model auto-seeding in tests/integration/init_bootstrap_flow.rs
- [x] T013 [P] [US2] Add unit coverage for per-runtime default-model selection and multi-assistant fallback in src/domain/configuration.rs
- [x] T014 [P] [US2] Add CLI contract assertions for seeded route reporting in tests/contract/stack_neutral_init_contract.rs

### Implementation for User Story 2

- [x] T015 [US2] Expose a shared assistant default-model catalog from src/domain/configuration.rs
- [x] T016 [US2] Auto-seed planning, implementation, verification, and review routes in src/cli/init.rs when assistants are selected without explicit `--route` overrides
- [x] T017 [US2] Surface seeded defaults and override provenance in src/cli/init.rs and src/cli/config.rs

**Checkpoint**: Assistant selection during init produces deterministic, inspectable model routes without manual model lookup.

---

## Phase 5: User Story 3 - Seed Bounded Hygiene Defaults By Selected Technology (Priority: P3)

**Goal**: Carry selected domains and repository tool cues into merge-only ignore-file defaults.

**Independent Test**: Initialize representative repositories and verify only the relevant ignore files and patterns are added while custom lines survive.

### Tests for User Story 3

- [x] T018 [P] [US3] Add init integration coverage for hygiene file creation and merge behavior in tests/integration/init_bootstrap_flow.rs
- [x] T019 [P] [US3] Add unit coverage for technology and tool pattern selection in src/domain/workspace_hygiene.rs
- [x] T020 [P] [US3] Add contract assertions for hygiene-file behavior in tests/contract/stack_neutral_init_contract.rs

### Implementation for User Story 3

- [x] T021 [US3] Detect credible hygiene targets from selected domains and repository cues in src/domain/workspace_hygiene.rs
- [x] T022 [US3] Apply merge-only ignore-file updates during init in src/cli/init.rs
- [x] T023 [US3] Report created, updated, skipped, and preserved hygiene actions in src/cli/init.rs

**Checkpoint**: Init can seed relevant ignore defaults without overwriting existing local rules.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, release metadata, and focused validation for the shipped slice.

- [x] T024 [P] Update release-facing docs in README.md, tech-docs/getting-started.md, tech-docs/configuration.md, assistant/README.md, ROADMAP.md, and CHANGELOG.md
- [x] T025 [P] Bump version references in Cargo.toml, docs, and release metadata to 0.43.0
- [x] T026 [P] Refresh coverage-facing artifacts and command expectations in lcov.info and any touched contract fixtures if validation changes them
- [x] T027 Run focused validation from specs/043-stack-neutral-init/quickstart.md and the new targeted cargo test slices

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1**: No dependencies
- **Phase 2**: Depends on Phase 1 and blocks all user stories
- **Phase 3**: Depends on Phase 2
- **Phase 4**: Depends on Phase 2; can begin after US1 if shared files are stable
- **Phase 5**: Depends on Phase 2 and the shared hygiene policy from T004
- **Phase 6**: Depends on all implemented stories

### User Story Dependencies

- **US1 (P1)**: First MVP slice; no dependency on other stories
- **US2 (P2)**: Depends only on foundational assistant default helpers
- **US3 (P3)**: Depends on foundational hygiene policy helpers and integrates with init after US2 touches settle

### Within Each User Story

- Tests fail first before implementation changes
- Shared policy helpers land before CLI wiring
- CLI behavior changes land before docs and polish
- Story checkpoints must pass before moving to the next slice

### Parallel Opportunities

- T001 and T002 can run in parallel
- T005 can run in parallel with T003 and T004 once the policy surfaces exist
- Test tasks within each user story marked `[P]` can run in parallel
- T024 and T025 can run in parallel after code behavior is stable

## Parallel Example: User Story 2

```bash
# Launch User Story 2 validation work together:
Task: "Add init integration coverage for assistant-target model auto-seeding in tests/integration/init_bootstrap_flow.rs"
Task: "Add unit coverage for per-runtime default-model selection and multi-assistant fallback in src/domain/configuration.rs"
Task: "Add CLI contract assertions for seeded route reporting in tests/contract/stack_neutral_init_contract.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Setup and Foundational phases
2. Deliver User Story 1
3. Validate stack-neutral entry independently
4. Then add assistant defaults and hygiene defaults

### Incremental Delivery

1. Build shared policy surfaces
2. Ship stack-neutral entry
3. Add assistant-target model seeding
4. Add hygiene defaults
5. Finish docs, versioning, and focused validation

## Notes

- `[P]` tasks touch different files or independent tests
- Task IDs map directly to implementation progress and later checklist updates
- The final implementation must mark completed tasks as `[x]`
