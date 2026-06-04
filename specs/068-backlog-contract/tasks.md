# Tasks: Backlog Contract

**Input**: Design documents from `/specs/068-backlog-contract/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/backlog-quality-runtime-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Add or refine focused regressions first,
confirm the relevant assertion fails before changing implementation, then close
the regression with the smallest coherent implementation change.

**Organization**: Tasks are grouped by user story so the backlog-readiness
gate, additive runtime projections, and assistant-safe continuation surfaces
can be validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

## Phase 1: Setup

**Purpose**: Lock the Canon `0.67.0` compatibility audit, fail fast if the
producer contract is insufficient for this slice, and establish failing
release-surface and assistant regressions before runtime edits.

- [X] T001 Reconfirm the Canon `0.67.0` backlog packet audit and the public provider-catalog no-change result in `specs/068-backlog-contract/research.md`; if required producer fields are absent, stop Boundline implementation and record the Canon follow-up before any runtime edits
- [X] T002 [P] Add or refine Canon compatibility and release-surface regressions in `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, `tests/contract/distribution_release_surface_contract.rs`, and `tests/assistant_plugin_packages.rs`
- [X] T003 [P] Add or refine assistant backlog-gate contract regressions in `tests/contract/assistant_command_definition_contract.rs` and `tests/contract/assistant_session_continuity_contract.rs`

---

## Phase 2: Foundational

**Purpose**: Fail the current backlog-quality behavior in the smallest
high-signal places before story-specific implementation work.

**Critical**: Complete this phase before user-story implementation.

- [X] T004 [P] Add focused failing backlog-quality domain regressions for risk-only packets, missing stable identifiers, recoverable missing dependency order, and missing MVP scope in `tests/unit/governance_policy.rs`
- [X] T005 [P] Add focused failing runtime regressions for backlog-gate ordering, withheld execution handoff, and one-question clarification behavior in `tests/unit/session_cli_runtime.rs` and `tests/contract/planning_gate_pipeline_contract.rs`
- [X] T006 [P] Add focused failing session-status and orchestrate projection regressions for additive backlog-quality fields in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: The feature now fails for the required backlog-contract
behavior while preserving backward compatibility expectations.

---

## Phase 3: User Story 1 - Block Unsafe Backlog Handoff (Priority: P1)

**Goal**: Prevent execution admission until the Canon backlog packet is
credible enough for downstream work.

**Independent Test**: In an isolated temporary workspace, evaluate blocked,
clarification-required, and ready Canon-style backlog packets and confirm that
execution is withheld until backlog quality is ready.

### Implementation

- [X] T007 [US1] Audit and complete Canon `0.67.0` packet evidence parsing, stable identifier validation, finding classification, and blocked-versus-clarification semantics in `src/domain/governance.rs`
- [X] T008 [US1] Audit and complete backlog-gate admission ordering, recovery routing, and withheld execution handoff in `src/orchestrator/session_runtime_planning_runtime.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`
- [X] T009 [US1] Audit and complete downstream planning-analysis handoff assumptions that consume backlog task count, MVP scope, and unmapped items in `src/domain/goal_plan.rs`
- [X] T010 [US1] Run the focused US1 regression set in `tests/unit/governance_policy.rs`, `tests/unit/session_cli_runtime.rs`, `tests/contract/planning_gate_pipeline_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: Unsafe or closure-limited backlog packets cannot reach execution
handoff, while a credible packet may continue to later gates.

---

## Phase 4: User Story 2 - Inspect Backlog Readiness (Priority: P2)

**Goal**: Expose backlog quality state, findings, task count, MVP scope, and
unmapped items through persisted and rendered runtime surfaces.

**Independent Test**: Inspect ready, clarification-required, and blocked
temporary sessions and confirm that status, orchestrate output, and traces
surface the same additive backlog-quality contract.

### Implementation

- [X] T011 [US2] Audit and complete additive backlog session-view fields and compatibility defaults in `src/domain/session.rs`
- [X] T012 [US2] Audit and complete backlog-quality session projection wiring in `src/cli/session.rs`
- [X] T013 [US2] Audit and complete human-readable and JSON backlog-quality rendering in `src/cli/output_session_status.rs` and `src/cli/output_orchestrate.rs`
- [X] T014 [US2] Persist and trace backlog-quality state transitions with reproducible session context in `src/orchestrator/session_runtime.rs` and supporting helpers in `src/domain/governance.rs`
- [X] T015 [US2] Run the focused US2 regression set in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: Every supported runtime projection exposes the same additive
backlog-readiness decision without breaking older snapshots.

---

## Phase 5: User Story 3 - Preserve Backlog Gates Through Assistant Surfaces (Priority: P3)

**Goal**: Keep assistant-specific plan and run commands thin and symmetric over
the CLI/runtime backlog contract.

**Independent Test**: Validate each supported plan and run asset and confirm
that it preserves backlog-quality fields and planning-stage continuation when
backlog quality is not ready.

### Implementation

- [X] T016 [P] [US3] Align the Antigravity and Claude plan/run assets with the backlog-contract runtime semantics in `assistant/antigravity/commands/boundline-plan.md`, `assistant/antigravity/commands/boundline-run.md`, `assistant/claude/commands/boundline-plan.md`, and `assistant/claude/commands/boundline-run.md`
- [X] T017 [P] [US3] Align the Codex and Copilot plan/run assets with the backlog-contract runtime semantics in `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, and `assistant/copilot/prompts/boundline-run.prompt.md`
- [X] T018 [US3] Run assistant parity regressions in `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, and `tests/integration/assistant_chat_fallback.rs`

**Checkpoint**: All supported assistants remain projections over one runtime
contract and do not invent execution continuation while backlog quality is not
ready.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close versioning, Canon compatibility wording, docs, and quality
gates for release `0.69.0`.

- [X] T019 [P] Refresh the provider-catalog audit result and explicit no-change rationale in `assistant/catalog/model-catalog.toml` and `specs/068-backlog-contract/research.md`
- [X] T020 [P] Document the backlog gate, Canon `0.67.0` compatibility posture, and additive runtime projections in `README.md`, `docs/runtime/plan.md`, `docs/runtime/status.md`, `docs/runtime/phase-requests.md`, `docs/runtime/inspect.md`, `docs/runtime/trace.md`, `docs/guide/common-workflows.md`, `docs/guide/getting-started.md`, and `docs/architecture/runtime-model.md`
- [X] T021 [P] Update operator and architecture guidance for the backlog contract in `tech-docs/architecture.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `docs/reference/configuration.md`, and `docs/governance/guardians.md`
- [X] T022 [P] Bump release metadata to `0.69.0` and adopt Canon `0.67.0` compatibility wording in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `assistant/plugin-metadata.json`, `assistant/global/manifest.json`, `tests/contract/canon_reasoning_posture_contract.rs`, and `src/cli/init.rs`
- [X] T023 [P] Add the `0.69.0` WinGet release manifests under `distribution/winget/manifests/a/ApplyThe/Boundline/0.69.0/`
- [X] T024 [P] Record the release summary and delivered roadmap slice in `CHANGELOG.md`, `docs/roadmap/index.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/features/README.md`, and `roadmap/features/04-backlog-contract.md`
- [X] T025 Run `cargo fmt` and verify formatting with `cargo fmt --check`
- [X] T026 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [X] T027 Run focused tests with `cargo test --test unit`, `cargo test --test contract`, and `cargo test --test integration host_session_runtime_flow::`
- [X] T028 Run release-surface regressions in `tests/assistant_plugin_packages.rs`, `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, and `tests/contract/distribution_release_surface_contract.rs`
- [X] T029 Run the full regression suite with `cargo test` and resolve any failures
- [X] T030 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [X] T031 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every touched Rust implementation file, and add tests until changed-file coverage is at least 95 percent
- [X] T032 Validate the isolated scenarios in `specs/068-backlog-contract/quickstart.md` without running Boundline CLI commands against the repository root

---

## Dependencies and Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all story work.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP.
- **User Story 2 (Phase 4)**: Depends on the typed US1 gate so projections
  expose the final decision shape.
- **User Story 3 (Phase 5)**: Depends on the US1 runtime contract but can run
  in parallel with US2 after US1 stabilizes.
- **Release and Quality Closure (Phase 6)**: Depends on all selected stories.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T004, T005, and T006 can run in parallel.
- T016 and T017 can run in parallel.
- T019 through T024 can run in parallel after runtime behavior stabilizes.

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational regressions.
2. Complete US1 backlog-admission behavior.
3. Validate one blocked, one clarification-required, and one ready isolated
   backlog packet.
4. Proceed to additive projections and assistant surfaces.

### Quality Rule

Do not treat formatting, clippy, tests, or changed-file coverage as deferred
release work. The feature is complete only when `cargo fmt --check`, strict
clippy, the regression suite, and at least 95 percent changed-file coverage
pass while the release and Canon compatibility surfaces remain aligned.
