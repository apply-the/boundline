# Tasks: Distribution & Bundling

**Input**: Design documents from `/specs/039-distribution-bundling/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes CLI
diagnostics, release metadata, install and repair behavior, and operator-facing
documentation surfaces.

**Organization**: Tasks are grouped by user story so each slice can deliver
bounded, inspectable value independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the repository-owned distribution surfaces and test entrypoints.

- [ ] T001 Create the distribution metadata scaffold in `distribution/canon-bundle.toml`, `distribution/homebrew/Formula/boundline.rb`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/`
- [ ] T002 [P] Create the release packaging workflow skeleton in `.github/workflows/release-distribution.yml`
- [ ] T003 [P] Register distribution-focused test modules in `tests/contract.rs`, `tests/integration.rs`, and `tests/unit.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared policy, diagnostics abstractions, fixtures, and metadata sync surface used by all stories.

**⚠️ CRITICAL**: No user story work should begin until this phase is complete.

- [ ] T004 Create shared distribution policy and companion-state types in `src/domain/distribution.rs`, `src/domain.rs`, and `src/lib.rs`
- [ ] T005 [P] Add reusable distribution fixtures in `tests/support/workspace_fixture.rs` and `tests/support/distribution_fixture.rs`
- [ ] T006 Generalize diagnostics report subjects and rendering hooks in `src/cli/diagnostics.rs` and `src/cli/output.rs`
- [ ] T007 [P] Implement version and checksum sync plumbing in `scripts/sync-distribution-metadata.sh`
- [ ] T008 Define stable bundle names and package metadata placeholders in `distribution/canon-bundle.toml`, `distribution/homebrew/Formula/boundline.rb`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.yaml`

**Checkpoint**: Shared distribution rules, diagnostics plumbing, and metadata layout are ready for story work.

---

## Phase 3: User Story 1 - Install Boundline Without Toolchain Friction (Priority: P1) 🎯 MVP

**Goal**: Let supported macOS and Windows users install Boundline from an official channel, verify the paired Canon state, and reach the first session-native path without a source checkout.

**Independent Test**: Install from the official channel metadata or fixture, run `boundline doctor --install`, and verify the CLI plus Canon pairing state before entering `doctor -> start -> capture -> plan -> run`.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add install diagnostics contract coverage in `tests/contract/distribution_cli_contract.rs`
- [ ] T010 [P] [US1] Add fresh-install integration coverage in `tests/integration/distribution_doctor_flow.rs`
- [ ] T011 [P] [US1] Add blocked-companion integration coverage in `tests/integration/distribution_doctor_blocked_flow.rs`

### Implementation for User Story 1

- [ ] T012 [US1] Add `doctor --install` CLI parsing and dispatch in `src/cli.rs`
- [ ] T013 [US1] Implement install diagnostics checks and Canon pairing evaluation in `src/cli/diagnostics.rs`
- [ ] T014 [US1] Render install diagnostics output and bounded actions in `src/cli/output.rs`
- [ ] T015 [US1] Publish bundled install metadata in `distribution/homebrew/Formula/boundline.rb`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.installer.yaml`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.locale.en-US.yaml`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.yaml`
- [ ] T016 [US1] Rewrite the first-run quick path in `README.md` and `docs/getting-started.md`

**Checkpoint**: Fresh installs are diagnosable, official channel metadata exists, and the quick path reaches the first session-native flow.

---

## Phase 4: User Story 2 - Keep Boundline And Canon Aligned Through Updates (Priority: P2)

**Goal**: Make updates explicit, bounded, and repairable when Boundline and Canon drift or only partially upgrade.

**Independent Test**: Start from an older or drifted install, run the update flow, then verify `boundline doctor --install` ends in `ready`, `blocked`, or `repair_needed` with a clear next action.

### Tests for User Story 2

- [ ] T017 [P] [US2] Add release metadata contract coverage in `tests/contract/distribution_metadata_contract.rs`
- [ ] T018 [P] [US2] Add update-and-repair integration coverage in `tests/integration/distribution_update_flow.rs`
- [ ] T019 [P] [US2] Add distribution policy unit coverage in `tests/unit/distribution_metadata.rs` and `tests/unit/distribution_diagnostics.rs`

### Implementation for User Story 2

- [ ] T020 [US2] Extend update policy and repair guidance in `src/domain/distribution.rs`
- [ ] T021 [US2] Extend install diagnostics for drift, blocked channels, and repair-needed states in `src/cli/diagnostics.rs` and `src/cli/output.rs`
- [ ] T022 [US2] Implement release bundle assembly and checksum publication in `.github/workflows/release-distribution.yml`
- [ ] T023 [US2] Wire version and checksum sync into package metadata in `scripts/sync-distribution-metadata.sh`, `distribution/homebrew/Formula/boundline.rb`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.installer.yaml`
- [ ] T024 [US2] Document update and repair flows in `README.md`, `docs/getting-started.md`, and `docs/architecture.md`

**Checkpoint**: Installed users have one bounded update and repair story backed by release assets and explicit diagnostics.

---

## Phase 5: User Story 3 - Learn The Product In Two Read Levels (Priority: P3)

**Goal**: Split onboarding into a brutal quick path plus an advanced architecture layer that keeps Boundline and Canon clearly separated.

**Independent Test**: A reader can find install plus first-run steps immediately from README, then read the advanced docs to explain the Boundline versus Canon boundary without reading the entire repository front to back.

### Tests for User Story 3

- [ ] T025 [P] [US3] Add docs-boundary contract coverage in `tests/contract/distribution_docs_contract.rs`
- [ ] T026 [P] [US3] Add documentation-layer integration coverage in `tests/integration/distribution_docs_flow.rs`

### Implementation for User Story 3

- [ ] T027 [US3] Split the onboarding and architecture narrative across `README.md`, `docs/getting-started.md`, and `docs/architecture.md`
- [ ] T028 [US3] Align Boundline versus Canon messaging in `assistant/README.md`, `assistant/codex/commands/boundline-start.md`, `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-next.md`, and `assistant/codex/commands/boundline-inspect.md`
- [ ] T029 [US3] Align Copilot and Claude guidance with the new docs split in `assistant/copilot/prompts/boundline-start.prompt.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, `assistant/copilot/prompts/boundline-next.prompt.md`, `assistant/copilot/prompts/boundline-inspect.prompt.md`, `assistant/claude/commands/boundline-start.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-next.md`, and `assistant/claude/commands/boundline-inspect.md`

**Checkpoint**: Quick path and advanced architecture are clearly separated across product and assistant docs.

---

## Phase 6: User Story 4 - Publish One Coherent Release Surface (Priority: P4)

**Goal**: Close the release with aligned versioning, channel metadata, roadmap state, changelog, and compatibility guidance.

**Independent Test**: Prepare one release candidate, refresh the package metadata and docs, then confirm version, Canon expectation, and channel readiness match everywhere.

### Tests for User Story 4

- [ ] T030 [P] [US4] Add release-surface integration coverage in `tests/integration/release_metadata_flow.rs`
- [ ] T031 [P] [US4] Add release alignment contract coverage in `tests/contract/distribution_release_surface_contract.rs`

### Implementation for User Story 4

- [ ] T032 [US4] Bump the release version in `Cargo.toml` and `Cargo.lock`
- [ ] T033 [US4] Update the public release narrative in `CHANGELOG.md`, `ROADMAP.md`, `README.md`, and `AGENTS.md`
- [ ] T034 [US4] Make channel readiness and source fallback explicit in `distribution/homebrew/Formula/boundline.rb`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/ApplyThe.Boundline.yaml`, `.github/workflows/release-distribution.yml`, and `docs/architecture.md`

**Checkpoint**: The release surface tells one coherent `0.39.0` story across metadata, docs, roadmap, and compatibility guidance.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finish validation, coverage, and final release-quality checks.

- [ ] T035 [P] Refresh distribution coverage support in `tests/unit/distribution_diagnostics.rs`, `tests/integration/distribution_update_flow.rs`, and `lcov.info`
- [ ] T036 Validate the operator scenarios captured in `specs/039-distribution-bundling/quickstart.md`
- [ ] T037 Validate formatting, lint, compile-only, integration, and coverage for `Cargo.toml`, `src/cli.rs`, `src/cli/diagnostics.rs`, `src/cli/output.rs`, `src/domain/distribution.rs`, and `lcov.info` with `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup** has no dependencies and can start immediately.
- **Phase 2: Foundational** depends on Phase 1 and blocks all user stories.
- **Phase 3: US1** depends on Phase 2 and delivers the MVP install surface.
- **Phase 4: US2** depends on Phase 2 and the shared install diagnostics model from US1.
- **Phase 5: US3** depends on Phase 2 and can proceed once the quick path terminology is stable.
- **Phase 6: US4** depends on Phases 3 through 5 because it closes the final release story.
- **Phase 7: Polish** depends on all user stories being complete.

### User Story Dependencies

- **US1 (P1)**: First deliverable and MVP.
- **US2 (P2)**: Builds on the install diagnostics and metadata policy introduced by US1.
- **US3 (P3)**: Builds on the final install and update wording from US1 and US2.
- **US4 (P4)**: Closes versioning and release narrative after all user-visible behavior is stable.

### Within Each User Story

- Tests should be written first and shown failing before implementation where the behavior is executable.
- Shared models and contracts should precede CLI wiring and docs updates.
- Diagnostics and metadata changes should land before story sign-off.
- Each story should be independently validated before moving to the next priority.

### Parallel Opportunities

- `T002` and `T003` can run in parallel after `T001`.
- `T005`, `T007`, and `T008` can run in parallel once `T004` is underway.
- Within US1, `T009`, `T010`, and `T011` can run in parallel.
- Within US2, `T017`, `T018`, and `T019` can run in parallel.
- Within US3, `T025` and `T026` can run in parallel.
- Within US4, `T030` and `T031` can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Launch all US1 validation work together:
Task: "Add install diagnostics contract coverage in tests/contract/distribution_cli_contract.rs"
Task: "Add fresh-install integration coverage in tests/integration/distribution_doctor_flow.rs"
Task: "Add blocked-companion integration coverage in tests/integration/distribution_doctor_blocked_flow.rs"

# Launch metadata and docs work after CLI wiring stabilizes:
Task: "Publish bundled install metadata in distribution/homebrew/Formula/boundline.rb and distribution/winget/manifests/a/ApplyThe/Boundline/0.39.0/..."
Task: "Rewrite the first-run quick path in README.md and docs/getting-started.md"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate `boundline doctor --install` plus the first session-native path.

### Incremental Delivery

1. Land the install surface and official channel metadata.
2. Add update and repair behavior.
3. Split and align the documentation and assistant guidance.
4. Close the version bump, roadmap, changelog, and release automation.
5. Finish with repository-wide validation and coverage refresh.

### Parallel Team Strategy

1. One contributor can own diagnostics and CLI wiring in `src/cli.rs`, `src/cli/diagnostics.rs`, and `src/cli/output.rs`.
2. One contributor can own release metadata and workflow files in `distribution/`, `scripts/`, and `.github/workflows/`.
3. One contributor can own README, docs, and assistant guidance once install wording stabilizes.

---

## Notes

- `[P]` tasks touch different files or can proceed once shared contracts are stable.
- Story labels map each task back to one independently testable user story.
- The final implementation must leave modified Rust files above 95% line coverage.