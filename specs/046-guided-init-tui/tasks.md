# Tasks: Guided Init TUI and Runtime Catalog

**Input**: Design documents from `/specs/046-guided-init-tui/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Validation tasks are required for this slice because it changes operator-facing CLI behavior, error handling, TTY fallback, progress rendering, and persisted bootstrap outputs. Refresh `lcov.info` and keep every modified or created Rust file at or above 95% line coverage before sign-off.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish release alignment and the shared dependency surface for the init redesign.

- [ ] T001 Bump Boundline version to `0.47.0` in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, and `CHANGELOG.md`
- [ ] T002 Add the guided-init CLI dependencies and bundled catalog asset scaffold in `Cargo.toml`, `crates/boundline-cli/Cargo.toml`, and `assistant/catalog/model-catalog.toml`
- [ ] T003 [P] Extend shared test support for init command fixtures and output capture in `tests/support/workspace_fixture.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Put the shared catalog, prompt-state, and validation primitives in place before story work starts.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T004 Define bundled catalog loading, metadata parsing, and default-route seeding primitives in `src/cli/init.rs` and `assistant/catalog/model-catalog.toml`
- [ ] T005 Define guided init state, step transitions, TTY detection, and non-interactive validation helpers in `src/cli/init.rs`
- [ ] T006 Define bounded progress-feedback helpers for interactive spinner mode and stable non-interactive text mode in `src/cli/init.rs`

**Checkpoint**: Foundation ready. Guided input, catalog state, and progress plumbing are explicit and testable.

---

## Phase 3: User Story 1 - Bootstrap a Workspace Without Memorizing Syntax (Priority: P1) 🎯 MVP

**Goal**: Replace fragile freeform guided prompts with a navigable init wizard plus top-level version support.

**Independent Test**: Run `boundline --version` and complete guided `boundline init` in an empty workspace using navigation and selection prompts, then confirm the summary and verify written outputs.

### Tests for User Story 1

- [ ] T007 [P] [US1] Add contract coverage for `boundline --version`, `boundline -V`, and guided-init entry behavior in `tests/contract/init_cli_contract.rs`
- [ ] T008 [P] [US1] Extend guided bootstrap integration coverage for defaults, one-slot route editing, and final summary confirmation in `tests/integration/init_bootstrap_flow.rs`
- [ ] T009 [P] [US1] Add unit coverage for guided step validation, visible defaults, and route review transitions in `src/cli/init.rs`
- [ ] T010 [P] [US1] Add cancellation write-safety coverage proving no files are written when the operator cancels before confirmation in `tests/contract/init_cli_contract.rs` and `tests/integration/init_bootstrap_flow.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Add top-level version support and explicit `--non-interactive` init flag handling in `src/cli.rs`
- [ ] T012 [US1] Replace manual `read_line` guided prompts with dialoguer-backed Canon-mode, assistant multi-select, and route-review interactions in `src/cli/init.rs`
- [ ] T013 [US1] Render the new guided summary, cancel path, and preview/write output for init in `src/cli/init.rs`

**Checkpoint**: User Story 1 is independently functional and proves the new guided bootstrap path.

---

## Phase 4: User Story 2 - Choose Routes From an Honest Catalog (Priority: P2)

**Goal**: Expose a bundled runtime/model catalog, slot-by-slot route edits, custom-model warnings, and explicit assistant-pack scaffolding status.

**Independent Test**: Select assistant surfaces, inspect the route table, accept one bundled model, provide one custom model id, and verify the summary reports catalog source, warnings, and assistant-pack actions.

### Tests for User Story 2

- [ ] T014 [P] [US2] Add contract coverage for bundled catalog metadata, custom-model warnings, unset-slot visibility, and assistant-pack scaffolding status in `tests/contract/init_cli_contract.rs`
- [ ] T015 [P] [US2] Extend integration coverage for bundled defaults, custom route fallback, and assistant-pack refresh behavior in `tests/integration/init_bootstrap_flow.rs`
- [ ] T016 [P] [US2] Add unit coverage for catalog parsing, route seeding, custom-route validation, and assistant asset planning in `src/cli/init.rs` and `src/cli/assistant_assets.rs`

### Implementation for User Story 2

- [ ] T017 [US2] Implement bundled catalog parsing, route-table derivation, and per-slot model selection in `src/cli/init.rs` and `assistant/catalog/model-catalog.toml`
- [ ] T018 [US2] Extend assistant-pack scaffolding reporting and explicit unset/custom slot summaries in `src/cli/init.rs` and `src/cli/assistant_assets.rs`
- [ ] T019 [US2] Align repository docs for the bundled catalog and guided route editing surface in `README.md`, `docs/getting-started.md`, and `assistant/README.md`

**Checkpoint**: User Stories 1 and 2 work together and route choices are explicit, inspectable, and reviewable.

---

## Phase 5: User Story 3 - Automate and Observe Long Init Work (Priority: P3)

**Goal**: Preserve automation parity, add safe progress feedback for long init work, and make no-TTY behavior explicit.

**Independent Test**: Run non-interactive `boundline init` with explicit flags in a scriptable workspace, trigger long-running bootstrap work, and verify progress feedback behaves correctly in both terminal and redirected-output contexts.

### Tests for User Story 3

- [ ] T020 [P] [US3] Add contract coverage for non-interactive validation, no-TTY guidance, progress-output semantics, and the bounded progress threshold in `tests/contract/init_cli_contract.rs`
- [ ] T021 [P] [US3] Extend integration coverage for non-interactive parity, redirected-output safety, and failure/cancel progress cleanup in `tests/integration/init_bootstrap_flow.rs`
- [ ] T022 [P] [US3] Add unit coverage for progress activity state transitions, bounded threshold activation, and TTY fallback logic in `src/cli/init.rs`

### Implementation for User Story 3

- [ ] T023 [US3] Implement non-interactive init parity and explicit no-TTY guidance in `src/cli.rs` and `src/cli/init.rs`
- [ ] T024 [US3] Implement bounded spinner rendering for interactive long-running steps and stable text progress for non-interactive output in `src/cli/init.rs`
- [ ] T025 [US3] Update quickstart validation for version, guided bootstrap, non-interactive automation, and progress behavior in `specs/046-guided-init-tui/quickstart.md`

**Checkpoint**: All three user stories are independently functional, including automation and progress feedback.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Close release, coverage, and validation requirements across the full slice.

- [ ] T026 [P] Refresh init command help and release-aligned docs in `README.md`, `docs/configuration.md`, and `assistant/README.md`
- [ ] T027 [P] Refresh coverage artifacts and verify every modified or created Rust file reaches at least 95% line coverage in `lcov.info`
- [ ] T028 Run the full validation sweep from `specs/046-guided-init-tui/quickstart.md`, including `cargo test --no-run --all-targets --all-features`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo fmt --check`
- [ ] T029 Mark completed tasks and capture the released feature summary in `specs/046-guided-init-tui/tasks.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. Start immediately.
- **Foundational (Phase 2)**: Depends on Setup. Blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational. MVP entry point.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because the catalog and route-edit surfaces extend the guided wizard built there.
- **User Story 3 (Phase 5)**: Depends on User Story 1 and Foundational, then layers automation and progress behavior onto the same init flow.
- **Polish (Phase 6)**: Depends on all desired stories being complete.

### User Story Dependencies

- **US1**: No dependency on other user stories once Foundational is done.
- **US2**: Builds on US1 guided wizard and Foundational catalog primitives.
- **US3**: Builds on US1 guided and non-interactive flow, but does not depend on US2 documentation updates.

### Within Each User Story

- Write tests first and ensure they fail for new operator-visible behavior.
- Implement CLI and init logic before doc updates for that story.
- Keep failure handling, cancellation, and preview semantics explicit before story sign-off.
- Refresh coverage for touched Rust files before moving to the next story.

### Parallel Opportunities

- `T003` can run in parallel with `T002` after `T001` starts.
- Within each user story, the listed test tasks can run in parallel because they target different files or scopes.
- `T018`, `T024`, and `T025` can run after their corresponding implementation tasks land.
- Coverage and final validation stay sequential after all implementation is done.

---

## Parallel Example: User Story 1

```bash
# Launch the story-1 validations together:
Task: "Add contract coverage for boundline --version and guided-init entry behavior in tests/contract/init_cli_contract.rs"
Task: "Extend guided bootstrap integration coverage in tests/integration/init_bootstrap_flow.rs"
Task: "Add unit coverage for guided step validation in src/cli/init.rs"
```

---

## Implementation Notes

- Catalog schema authority: use the TOML example and rules in `specs/046-guided-init-tui/data-model.md` when implementing `assistant/catalog/model-catalog.toml`.
- No-TTY guidance text: use `Terminal interaction is unavailable. Rerun with --non-interactive and explicit flags.` as the baseline operator-facing message.
- Assistant-pack reporting format: render per surface as `surface: <created> created, <updated> updated, <unchanged> unchanged`.
- Progress validation: treat the interactive spinner threshold as a bounded, testable activation point so contract and unit coverage can verify SC-005 without relying on fragile wall-clock timing.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate `boundline --version` plus guided init independently.

### Incremental Delivery

1. Ship version bump and shared init primitives.
2. Deliver guided wizard MVP in US1.
3. Layer honest catalog and assistant-pack status in US2.
4. Layer non-interactive parity and progress feedback in US3.
5. Finish with coverage, clippy, fmt, and documentation alignment.

## Notes

- Keep the version bump as the first task exactly as requested.
- Avoid introducing a full-screen TUI or remote model-discovery dependency in this slice.
- Do not sign off until modified or created Rust files are at or above 95% line coverage and the repo is warning-free under clippy.
