# Tasks: Guided CLI UX And Clearer Messaging

**Input**: Design documents from `/specs/044-cli-init-ux/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-init-ux.md, quickstart.md

**Tests**: This slice changes operator-facing CLI behavior, validation failures, and diagnostics output. Add contract, unit, and integration coverage for prompt discoverability, recovery messages, assistant setup reporting, and rich/plain output parity.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align release metadata and test targets before changing behavior.

- [X] T001 Update release metadata to `0.44.0` in Cargo.toml, distribution/channel-metadata.toml, distribution/homebrew/Formula/boundline.rb, and CHANGELOG.md
- [X] T002 [P] Confirm the CLI UX validation surfaces and target files in tests/contract/init_cli_contract.rs, tests/integration/init_bootstrap_flow.rs, tests/integration/cli_diagnostics.rs, and tests/unit/cli_output.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish reusable CLI messaging and output primitives used across all stories.

**⚠️ CRITICAL**: No user story work should be considered complete until these tasks are done.

- [X] T003 Extend reusable init prompt and validation messaging helpers in src/cli/init.rs
- [X] T004 [P] Extend repository-local assistant scaffolding status helpers in src/cli/assistant_assets.rs and src/cli/init.rs
- [X] T005 [P] Add capability-aware output section rendering hooks for init and doctor in src/cli/output.rs and src/cli/diagnostics.rs

**Checkpoint**: Init and doctor have shared primitives for richer but automation-safe messaging.

---

## Phase 3: User Story 1 - Finish Init Without Leaving The Terminal (Priority: P1) 🎯 MVP

**Goal**: Make guided and help-based init self-explanatory for assistant selection, route syntax, and default behavior.

**Independent Test**: A first-run operator can complete `boundline init` or inspect `boundline init --help` without consulting external docs for route syntax or defaults.

### Tests for User Story 1

- [X] T006 [P] [US1] Add contract coverage for discoverable init help text in tests/contract/init_cli_contract.rs
- [X] T007 [P] [US1] Add integration coverage for seeded-route and assistant-setup summaries in tests/integration/init_bootstrap_flow.rs
- [X] T008 [P] [US1] Add unit coverage for guided prompt text and prompt-default behavior in src/cli/init.rs

### Implementation for User Story 1

- [X] T009 [US1] Implement discoverable guided prompt text, supported slot lists, and route examples in src/cli/init.rs
- [X] T010 [US1] Align clap init help text with guided prompt examples and blank/default behavior in src/cli.rs
- [X] T011 [US1] Extend post-init summaries to explain seeded routes, explicit overrides, assistant setup, and follow-up inspection in src/cli/init.rs

**Checkpoint**: Guided init and `init --help` are self-sufficient for first-run route selection.

---

## Phase 4: User Story 2 - Recover From Bad Input Quickly (Priority: P2)

**Goal**: Turn init validation failures into human-readable, actionable recovery guidance.

**Independent Test**: Malformed routes, unsupported assistant values, unavailable defaults, and overwrite conflicts fail with clear corrective guidance and no silent workspace mutation.

### Tests for User Story 2

- [X] T012 [P] [US2] Add integration coverage for malformed route input, unavailable assistant defaults, and overwrite-preview behavior in tests/integration/init_bootstrap_flow.rs
- [X] T013 [P] [US2] Add contract coverage for actionable init validation and recovery wording in tests/contract/init_cli_contract.rs
- [X] T014 [P] [US2] Add unit coverage for route parsing and recovery message classification in src/cli/init.rs

### Implementation for User Story 2

- [X] T015 [US2] Implement human-readable validation and recovery messages for malformed routes, unknown slots, and unsupported runtimes in src/cli/init.rs
- [X] T016 [US2] Clarify preview, overwrite, and non-interactive stop guidance in src/cli/init.rs and src/cli/diagnostics.rs
- [X] T017 [US2] Ensure assistant setup preview/apply participates in the same safe rerun contract in src/cli/init.rs and src/cli/assistant_assets.rs

**Checkpoint**: Init failures tell the operator what failed and what to do next.

---

## Phase 5: User Story 3 - Read The Outcome At A Glance (Priority: P3)

**Goal**: Make init and doctor output easier to scan while preserving plain-text automation semantics.

**Independent Test**: Interactive and plain-text diagnostics communicate the same meaning, with grouped sections for warnings, success, defaults, and next steps.

### Tests for User Story 3

- [X] T018 [P] [US3] Add unit coverage for grouped diagnostics rendering in tests/unit/cli_output.rs
- [X] T019 [P] [US3] Add integration coverage for doctor output structure in tests/integration/cli_diagnostics.rs
- [X] T020 [P] [US3] Add contract coverage for updated CLI UX documentation in tests/contract/distribution_docs_contract.rs and tests/contract/assistant_command_definition_contract.rs

### Implementation for User Story 3

- [X] T021 [US3] Implement semantically grouped init output with plain-text-safe rich formatting hooks in src/cli/output.rs and src/cli/init.rs
- [X] T022 [US3] Implement semantically grouped install/workspace diagnostic output in src/cli/output.rs and src/cli/diagnostics.rs
- [X] T023 [US3] Align operator and assistant guidance with the new init/doctor UX in README.md, tech-docs/getting-started.md, tech-docs/configuration.md, and assistant/README.md

**Checkpoint**: First-run init and doctor output are easier to scan without breaking scripting behavior.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup, formatting, linting, and feature-level validation.

- [X] T024 [P] Refresh release-aligned docs and assistant examples in tech-docs/getting-started.md, tech-docs/configuration.md, README.md, assistant/README.md, and specs/044-cli-init-ux/quickstart.md
- [X] T025 [P] Add extra regression coverage for touched init and diagnostics behavior in tests/integration/init_bootstrap_flow.rs, tests/integration/cli_diagnostics.rs, and tests/unit/cli_output.rs
- [X] T026 Resolve all lint and formatting issues across touched files with `cargo fmt` and targeted cleanup in src/cli/init.rs, src/cli/output.rs, src/cli/diagnostics.rs, src/cli/assistant_assets.rs, and tests/
- [X] T027 Run final validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused `cargo test` commands for touched init/diagnostics slices, and `cargo test --no-run --all-targets --all-features`

---

## Dependencies & Execution Order

### Phase Dependencies

- Phase 1 → no dependencies.
- Phase 2 → depends on Phase 1 and blocks all story work.
- Phase 3 → depends on Phase 2.
- Phase 4 → depends on Phase 3 surfaces being in place.
- Phase 5 → depends on Phases 3 and 4 because it groups and documents their output.
- Phase 6 → depends on all implementation phases completing.

### User Story Dependencies

- **US1 (P1)**: starts after foundational helpers land.
- **US2 (P2)**: depends on US1 prompt/help surfaces so recovery messages can reference the same examples and defaults.
- **US3 (P3)**: depends on US1 and US2 output content to group and present it consistently.

### Parallel Opportunities

- T002 can run in parallel with T001.
- T004 and T005 can run in parallel after T003 starts defining the shared messaging direction.
- Within each story, contract/integration/unit tests marked `[P]` can be authored in parallel.
- T024 and T025 can run in parallel during polish before T026/T027 final validation.

## Parallel Example: User Story 1

```bash
Task: "Add contract coverage for discoverable init help text in tests/contract/init_cli_contract.rs"
Task: "Add integration coverage for seeded-route and assistant-setup summaries in tests/integration/init_bootstrap_flow.rs"
Task: "Add unit coverage for guided prompt text and prompt-default behavior in src/cli/init.rs"
```

## Implementation Strategy

### MVP First (US1)

1. Complete Setup and Foundational phases.
2. Implement US1 and validate guided/help-based init.
3. Stop and verify that a first-run operator can understand route defaults from the CLI alone.

### Incremental Delivery

1. Add assisted recovery (US2) on top of the new prompt/help text.
2. Add grouped init/doctor output plus docs alignment (US3).
3. Finish with release bump, fmt, lint, and focused regression coverage.

## Notes

- Update task checkboxes to `[X]` as each item is completed during implementation.
- `[P]` means different files or independent validation work.
- Every task references a real repository path.
- The feature is not done until init/doctor UX, docs, version metadata, tests, fmt, and clippy all pass together.