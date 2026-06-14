# Tasks: Plan Analysis Contract

**Input**: Design documents from `/specs/069-plan-analysis-contract/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/planning-analysis-runtime-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Add or refine focused regressions first,
confirm the relevant assertion fails before changing implementation, then close
the regression with the smallest coherent runtime change.

**Organization**: Tasks are grouped by user story so the planning-analysis
gate, additive runtime projections, and assistant-safe continuation surfaces
can be implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

## Phase 1: Setup

**Purpose**: Lock the release boundary, reconfirm the provider-catalog
no-change result and Canon compatibility assumptions, and fail fast if the
feature still depends on missing producer-owned data.

- [x] T001 Reconfirm the Canon `0.67.0` governed-evidence boundary and the public provider-catalog no-change result in `specs/069-plan-analysis-contract/research.md`; if execution readiness still depends on absent Canon-owned fields, stop implementation and record the producer gap before any runtime edits
- [x] T002 [P] Add or refine release-surface regressions for Boundline `0.70.0` and Canon `0.67.0` compatibility in `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, `tests/contract/distribution_release_surface_contract.rs`, and `tests/assistant_plugin_packages.rs`
- [x] T003 [P] Add or refine assistant planning-analysis contract regressions in `tests/contract/assistant_command_definition_contract.rs` and `tests/contract/assistant_session_continuity_contract.rs`

---

## Phase 2: Foundational

**Purpose**: Make the missing planning-analysis behavior fail in the smallest
high-signal places before user-story implementation work begins.

**Critical**: Complete this phase before user-story implementation.

- [x] T004 [P] Add focused failing planning-analysis domain regressions for uncovered success criteria, missing validation coverage, explicit plan/backlog or risk/constraint contradictions, producer contract gaps, Canon-optional routes, and finding deduplication in `tests/unit/goal_plan_model.rs`
- [x] T005 [P] Add focused failing runtime regressions for gate ordering, withheld execution handoff, and compatibility omission semantics in `tests/unit/session_cli_runtime.rs` and `tests/contract/planning_gate_pipeline_contract.rs`
- [x] T006 [P] Add focused failing status, inspect, and orchestration projection regressions for additive planning-analysis fields in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: The feature now fails for the required planning-analysis
contract while preserving backward-compatibility expectations.

---

## Phase 3: User Story 1 - Block Incoherent Execution Handoff (Priority: P1) 🎯 MVP

**Goal**: Prevent execution admission until the full planning picture is
coherent across goal, plan, validation, backlog, execution readiness, and
governed evidence.

**Independent Test**: In an isolated temporary workspace, evaluate one clean
planning session and one critically inconsistent planning session and confirm
that only the incoherent session loses execution handoff while the gate stays
read-only.

### Implementation

- [x] T007 [US1] Refactor and expand the planning-analysis domain model, stable finding codes, source refs, coverage summary fields, and helper constants in `src/domain/goal_plan.rs`
- [x] T008 [US1] Implement deterministic end-to-end coherence checks for success-criterion coverage, validation coverage, explicit typed-artifact contradictions, execution-input presence, producer contract gaps, and deduplicated blocked findings in `src/domain/goal_plan.rs` and `src/domain/governance.rs`
- [x] T009 [US1] Audit and complete planning-analysis admission ordering, Canon-optional fallback behavior, and withheld execution handoff in `src/orchestrator/session_runtime_planning_runtime.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`
- [x] T010 [US1] Run the focused US1 regression set in `tests/unit/goal_plan_model.rs`, `tests/unit/session_cli_runtime.rs`, `tests/contract/planning_gate_pipeline_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: Critically incoherent planning states cannot reach execution
handoff, and the runtime blocks honestly on producer contract gaps instead of
inventing Canon data.

---

## Phase 4: User Story 2 - Inspect Planning Coherence (Priority: P2)

**Goal**: Expose planning-analysis state, findings, source attribution, and
coverage metrics through every supported runtime projection.

**Independent Test**: Inspect clean, warning-only, blocked, and compatibility
session snapshots and confirm that status, inspect, and orchestration surfaces
show the same additive planning-analysis contract.

### Implementation

- [x] T011 [US2] Audit and complete additive planning-analysis compatibility defaults and session-view fields in `src/domain/session.rs`
- [x] T012 [US2] Audit and complete planning-analysis projection wiring for session, inspect, and machine-readable output in `src/cli/session.rs` and `src/cli/inspect/projections.rs`
- [x] T013 [US2] Audit and complete human-readable and JSON planning-analysis rendering, including risk/constraint coverage signals and source-attributed contradiction findings, in `src/cli/output_session_status.rs` and `src/cli/output_orchestrate.rs`
- [x] T014 [US2] Persist and trace planning-analysis state transitions, coverage metrics, and blocked repair routing with reproducible session context in `src/orchestrator/session_runtime.rs` and supporting helpers in `src/domain/goal_plan.rs`
- [x] T015 [US2] Run the focused US2 regression set in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: Every supported runtime projection exposes the same additive
planning-coherence decision without breaking older snapshots.

---

## Phase 5: User Story 3 - Preserve Assistant-Safe Continuation (Priority: P3)

**Goal**: Keep assistant-specific plan, run, status, and inspect assets thin
and symmetric over the same planning-analysis runtime contract.

**Independent Test**: Validate each supported assistant asset against a
blocked planning-analysis session and confirm that the host routes back to
planning repair instead of inventing direct execution continuation.

### Implementation

- [x] T016 [P] [US3] Align the Antigravity and Claude planning-analysis sections and next-step routing in `assistant/antigravity/commands/boundline-plan.md`, `assistant/antigravity/commands/boundline-run.md`, `assistant/antigravity/commands/boundline-status.md`, `assistant/antigravity/commands/boundline-inspect.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, and `assistant/claude/commands/boundline-inspect.md`
- [x] T017 [P] [US3] Align the Codex and Copilot planning-analysis sections and next-step routing in `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, and `assistant/copilot/prompts/boundline-inspect.prompt.md`
- [x] T018 [US3] Run assistant parity regressions in `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, and `tests/integration/assistant_chat_fallback.rs`

**Checkpoint**: All supported assistants remain projections over one runtime
contract and do not invent execution continuation while planning analysis is
blocked.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close versioning, docs, roadmap, quality gates, and coverage for
release `0.70.0`.

- [x] T019 [P] Refresh the provider-catalog audit result and explicit no-change rationale in `assistant/catalog/model-catalog.toml` and `specs/069-plan-analysis-contract/research.md`
- [x] T020 [P] Document the planning-analysis gate, blocked-versus-warning semantics, repair-path behavior, trace expectations, and Canon `0.67.0` compatibility posture in `README.md`, `docs/runtime/plan.md`, `docs/runtime/status.md`, `docs/runtime/inspect.md`, `docs/runtime/phase-requests.md`, `docs/runtime/trace.md`, and `docs/guide/common-workflows.md`
- [x] T021 [P] Update engineering guidance for planning-analysis persistence and execution gating in `tech-docs/architecture.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, and `docs/reference/configuration.md`
- [x] T022 [P] Bump release metadata to `0.70.0` and keep Canon `0.67.0` compatibility wording aligned in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `assistant/plugin-metadata.json`, `assistant/global/manifest.json`, `src/cli/init.rs`, and `tests/contract/canon_reasoning_posture_contract.rs`
- [x] T023 [P] Add the `0.70.0` WinGet release manifests under `distribution/winget/manifests/a/ApplyThe/Boundline/0.70.0/`
- [x] T024 [P] Record the delivered roadmap slice and release summary in `CHANGELOG.md`, `docs/roadmap/index.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/features/README.md`, and `roadmap/features/05-plan-analysis-contract.md`
- [x] T025 Run `cargo fmt` and verify formatting with `cargo fmt --check`
- [x] T026 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [x] T027 Run focused tests with `cargo test --test unit`, `cargo test --test contract`, and `cargo test --test integration host_session_runtime_flow::`
- [x] T028 Run release-surface regressions in `tests/assistant_plugin_packages.rs`, `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, and `tests/contract/distribution_release_surface_contract.rs`
- [x] T029 Run the full regression suite with `cargo test` and resolve any failures
- [x] T030 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [x] T031 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every touched Rust implementation file, and add tests until changed-file coverage is at least 95 percent
- [x] T032 Validate the isolated scenarios in `specs/069-plan-analysis-contract/quickstart.md` without running Boundline CLI commands against the repository root

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all story work.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP.
- **User Story 2 (Phase 4)**: Depends on the typed US1 runtime contract so
  projections expose the final decision shape.
- **User Story 3 (Phase 5)**: Depends on the US1 runtime contract but can run
  in parallel with US2 after US1 stabilizes.
- **Release and Quality Closure (Phase 6)**: Depends on all selected stories.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational and has no dependency on
  later stories.
- **User Story 2 (P2)**: Can start after US1 stabilizes because it projects
  the finalized domain contract.
- **User Story 3 (P3)**: Can start after US1 stabilizes because assistant
  assets must reflect the runtime gate semantics already implemented.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T004, T005, and T006 can run in parallel.
- T016 and T017 can run in parallel.
- T019 through T024 can run in parallel after runtime behavior stabilizes.

---

## Parallel Example: User Story 1

```bash
# Launch all focused failing regressions for the coherence gate together:
Task: "Add focused failing planning-analysis domain regressions in tests/unit/goal_plan_model.rs"
Task: "Add focused failing runtime regressions in tests/unit/session_cli_runtime.rs and tests/contract/planning_gate_pipeline_contract.rs"
Task: "Add focused failing projection regressions in tests/unit/cli_output.rs, tests/contract/host_command_output_contract.rs, and tests/integration/host_session_runtime_flow.rs"
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational regressions.
2. Complete US1 planning-analysis admission behavior.
3. Validate one blocked, one clean, and one producer-gap scenario in an
   isolated temporary workspace.
4. Proceed to additive projections and assistant surfaces.

### Incremental Delivery

1. Finish Setup + Foundational so the missing contract is failing in focused
   tests.
2. Add US1 and validate execution withholding.
3. Add US2 and validate operator-facing projections.
4. Add US3 and validate assistant-safe routing.
5. Close docs, release metadata, and coverage only after the runtime contract
   is stable.

### Quality Rule

Do not treat formatting, clippy, tests, docs, release metadata, or
changed-file coverage as deferred cleanup. The feature is complete only when
`cargo fmt --check`, strict clippy, the regression suite, and at least
95 percent changed-file coverage pass while the release and Canon compatibility
surfaces remain aligned.

---

## Notes

- `[P]` tasks target different files and can be executed in parallel.
- `[US1]`, `[US2]`, and `[US3]` labels preserve traceability from spec to
  implementation.
- Every user story remains independently testable once its phase completes.
- The runtime gate must stay read-only throughout implementation.
