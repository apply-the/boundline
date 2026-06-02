# Tasks: Assistant Plugin Packages

**Input**: Design documents from `/specs/048-assistant-plugin-packages/`  
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/assistant-plugin-package-contract.md`, `quickstart.md`, `coherence-review.md`

**Tests**: This feature requires focused Rust validation tests, plugin validation command coverage, release-version alignment checks, and final touched-Rust-file coverage evidence.

**Organization**: Tasks are grouped by setup, foundational validation, user stories, and final closeout so each story remains independently testable.

## Phase 1: Setup

**Purpose**: Align the feature release version before any package metadata depends on it.

- [x] T001 Upgrade Boundline version surfaces to `0.49.0` in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.49.0/`, `assistant/catalog/model-catalog.toml`, `CHANGELOG.md`, `ROADMAP.md`, `tests/contract/distribution_metadata_contract.rs`, and `tests/contract/distribution_release_surface_contract.rs`
- [x] T002 Record the no-entry-change catalog refresh result for this slice in `specs/048-assistant-plugin-packages/research.md` and keep `assistant/catalog/model-catalog.toml` metadata aligned with `0.49.0`

---

## Phase 2: Foundational Validation

**Purpose**: Add failing tests before implementation so manifest and command package behavior is driven by executable checks.

- [x] T003 [P] Create failing plugin package contract tests in `tests/assistant_plugin_packages.rs` for package files, required commands, versions, referenced paths, README/docs sections, and prohibited positioning
- [x] T004 [P] Create failing validation-helper tests in `tests/assistant_plugin_packages.rs` for invalid JSON, missing metadata, version drift, missing paths, missing commands, unsupported capability claims, and prohibited terms
- [x] T005 Run `cargo test --test assistant_plugin_packages` from `repo root` and confirm the new tests fail because package files and validation helpers are not implemented yet

**Checkpoint**: Foundation is ready when the focused tests fail for the intended missing behavior.

---

## Phase 3: User Story 1 - Install Boundline From A Chat Host (Priority: P1)

**Goal**: Provide host package folders and shared metadata so developers can discover Boundline from chat surfaces.

**Independent Test**: `cargo test --test assistant_plugin_packages package_folders_and_docs_are_present metadata_paths_and_versions_are_aligned`

- [x] T006 [US1] Implement validation helpers in `src/assistant_plugin_validation.rs` and expose them from `src/lib.rs`
- [x] T007 [P] [US1] Add shared plugin metadata, command definitions, starter prompts, Copilot prompt pack, and plugin SVG assets in `assistant/plugin-metadata.json`, `assistant/commands/session-workflow.json`, `assistant/prompts/starter-prompts.md`, `assistant/prompts/copilot-command-pack.md`, `assistant/assets/boundline-plugin-icon.svg`, and `assistant/assets/boundline-plugin-logo.svg`
- [x] T008 [P] [US1] Add Claude Code, Codex, Cursor, and Copilot prompt-pack package files in `.claude-plugin/manifest.json`, `.claude-plugin/commands.json`, `.codex-plugin/plugin.json`, `.cursor-plugin/manifest.json`, `.cursor-plugin/commands.json`, `.copilot-prompts/README.md`, and `.copilot-prompts/pack.json`
- [x] T009 [US1] Run `cargo test --test assistant_plugin_packages package_folders_and_docs_are_present metadata_paths_and_versions_are_aligned` from `repo root` and fix package metadata until US1 passes

---

## Phase 4: User Story 2 - Drive The Session-Native Loop From Chat (Priority: P2)

**Goal**: Ensure required namespaced commands call or guide the real Boundline runtime and surface explicit session state.

**Independent Test**: `cargo test --test assistant_plugin_packages manifests_expose_required_boundline_commands command_guidance_preserves_session_state`

- [x] T010 [P] [US2] Add `/boundline:recover` and `/boundline:govern` command assets for Claude, Codex, and Copilot in `assistant/claude/commands/boundline-recover.md`, `assistant/claude/commands/boundline-govern.md`, `assistant/codex/commands/boundline-recover.md`, `assistant/codex/commands/boundline-govern.md`, `assistant/copilot/prompts/boundline-recover.prompt.md`, and `assistant/copilot/prompts/boundline-govern.prompt.md`
- [x] T011 [US2] Update `assistant/README.md` so required chat commands preserve `.boundline/session.json`, CLI-reported `next_command`, non-success states, conditional Canon governance, and host fallback behavior
- [x] T012 [US2] Run `cargo test --test assistant_plugin_packages manifests_expose_required_boundline_commands command_guidance_preserves_session_state` from `repo root` and fix command bindings until US2 passes

---

## Phase 5: User Story 3 - Prevent Host Package Drift (Priority: P3)

**Goal**: Provide automated validation for manifests, shared metadata, paths, command coverage, version drift, and positioning.

**Independent Test**: `bash scripts/validate-assistant-plugins.sh`

- [x] T013 [US3] Add `scripts/validate-assistant-plugins.sh` to run the focused plugin package test target
- [x] T014 [US3] Harden `src/assistant_plugin_validation.rs` and `tests/assistant_plugin_packages.rs` so validation rejects missing fields, version drift, missing paths, missing commands, unsupported capability claims, invalid path value types, malformed arrays, and prohibited positioning
- [x] T015 [US3] Run `bash scripts/validate-assistant-plugins.sh` from `repo root` and fix validation until US3 passes

---

## Phase 6: Documentation And README

**Purpose**: Make host installation and chat-to-CLI state mapping discoverable.

- [x] T016 [P] Add host installation and limitation docs in `tech-docs/guides/assistant-plugin-packages.md`
- [x] T017 [P] Add README sections "Use Boundline from chat", "Use Boundline from CLI", and "How chat commands map to CLI/runtime state" in `README.md`
- [x] T018 Update `assistant/README.md`, `tech-docs/guides/assistant-plugin-packages.md`, and `README.md` so the Boundline/Cannon boundary stays explicit and Canon is only visible for conditional governance
- [x] T019 Run quickstart checks from `specs/048-assistant-plugin-packages/quickstart.md` and fix documentation gaps

---

## Phase 7: Cross-Cutting Validation

**Purpose**: Confirm release metadata, package metadata, docs, and tests remain aligned.

- [x] T020 Run `rg -n '0\.49\.0|/boundline:(start|capture|plan|run|status|inspect|recover|govern)|session.json|next_command' Cargo.toml CHANGELOG.md ROADMAP.md distribution assistant .claude-plugin .codex-plugin .cursor-plugin .copilot-prompts tech-docs/guides/assistant-plugin-packages.md README.md tests` from `repo root` and fix missing release or command references
- [x] T021 Run `cargo test --test assistant_plugin_packages` from `repo root` and fix focused package validation failures
- [x] T022 Run `cargo test --test contract distribution_metadata_contract` and `cargo test --test contract distribution_release_surface_contract` from `repo root` and fix release-surface regressions
- [x] T023 Update `specs/048-assistant-plugin-packages/validation-report.md` with validation commands as they are run and the current status

---

## Phase 8: Final Closeout

**Purpose**: Prove the implementation is formatted, lint-clean, tested, and covered.

- [x] T024 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` from `repo root`; ensure every Rust file created or modified by this slice has at least 95% line coverage, fix warnings/errors/test failures/coverage gaps, and record the evidence in `specs/048-assistant-plugin-packages/validation-report.md`

---

## Dependencies & Execution Order

- Phase 1 must run first because all package metadata uses the release version.
- Phase 2 must run before package implementation to preserve test-first validation.
- US1 package metadata is required before US2 command guidance can be fully validated.
- US3 validation depends on US1 and US2 assets.
- Documentation can run after package and command shapes stabilize.
- Final closeout must be the last task.

## Parallel Opportunities

- T003 and T004 can be authored together because they target different test behaviors in the same new test file before implementation.
- T007 and T008 can proceed in parallel after T006 defines validation expectations.
- T010 can proceed while T011 documentation guidance is drafted, provided final command wording is reconciled before T012.
- T016 and T017 can proceed in parallel after package metadata is stable.

## Implementation Strategy

1. Upgrade version surfaces first.
2. Create failing validation tests.
3. Add the smallest validation helpers and package files that satisfy install/discovery.
4. Add the missing recover/govern command guidance and state-handling docs.
5. Add validation script and harden negative cases.
6. Update docs and README.
7. Run focused checks, then full fmt/clippy/tests/coverage closeout.
