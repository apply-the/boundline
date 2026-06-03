# Tasks: Plan Quality Contract

**Input**: Design documents from `/specs/067-plan-quality-contract/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/plan-quality-runtime-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Add or refine focused tests first, confirm
that the relevant assertion fails before changing implementation, then close
the regression with the smallest coherent implementation change.

**Organization**: Tasks are grouped by user story so the execution-admission
gate, observability projections, and assistant recovery surfaces can be
validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

## Phase 1: Setup

**Purpose**: Confirm the current scaffolding and establish failing regression
coverage before implementation edits.

- [X] T001 Record the current planning-quality scaffolding audit and the public model-catalog refresh result in `specs/067-plan-quality-contract/research.md`
- [X] T002 [P] Add or refine backward-compatible session deserialization regressions for additive plan-quality fields in `tests/unit/session_record.rs`
- [X] T003 [P] Add or refine assistant planning-asset section regressions in `tests/contract/assistant_command_definition_contract.rs`

---

## Phase 2: Foundational

**Purpose**: Lock the typed runtime contract before changing story-specific
surfaces.

**Critical**: Complete this phase before user-story implementation.

- [X] T004 [P] Add focused failing domain regressions for missing verification strategy findings, visible accepted assumptions, and blocked context readiness in `tests/unit/goal_plan_model.rs`
- [X] T005 [P] Add focused failing runtime regressions for plan-quality gate ordering, one-question `phase_request` routing, and withheld execution handoff in `tests/unit/session_cli_runtime.rs` and `tests/contract/planning_gate_pipeline_contract.rs`
- [X] T006 [P] Add focused failing JSON projection regressions for additive plan-quality state, findings, and assumptions in `tests/unit/cli_output.rs` and `tests/contract/host_command_output_contract.rs`

**Checkpoint**: The contract fails for the missing behavior and remains
backward-compatible for older session snapshots.

---

## Phase 3: User Story 1 - Block Unsafe Execution Handoff (Priority: P1)

**Goal**: Prevent execution admission until the active plan exposes an
actionable verification strategy, while retaining visible accepted defaults.

**Independent Test**: In an isolated temporary workspace, submit a plan without
an adequate verification strategy and confirm that execution is withheld with
exactly one focused `phase_request`; answer the request and confirm that the
same session resumes.

### Implementation

- [X] T007 [US1] Audit and complete typed `PlanQualityState`, finding, assumption, and assessment behavior in `src/domain/goal_plan.rs`
- [X] T008 [US1] Audit and complete plan-quality admission ahead of backlog quality, planning analysis, governance, and execution routing in `src/orchestrator/session_runtime_planning_runtime.rs` and `src/cli/session.rs`
- [X] T009 [US1] Persist blocked and recovered planning-quality transitions with structured trace context in `src/orchestrator/session_runtime.rs` and `src/orchestrator/session_runtime_native_goal_plan.rs`
- [X] T010 [US1] Run the focused US1 regression set in `tests/unit/goal_plan_model.rs`, `tests/unit/session_cli_runtime.rs`, and `tests/contract/planning_gate_pipeline_contract.rs`

**Checkpoint**: Unsafe handoff is blocked and recoverable through one active
question.

---

## Phase 4: User Story 2 - Inspect Plan Readiness (Priority: P2)

**Goal**: Expose readiness state, concise findings, and accepted assumptions
through persisted session, status, orchestration, inspect, and trace surfaces.

**Independent Test**: Inspect one ready and one blocked temporary session and
confirm that all operator-facing JSON projections expose the same additive
plan-quality contract.

### Implementation

- [X] T011 [US2] Extend additive persisted session models and compatibility defaults in `src/domain/session.rs`
- [X] T012 [US2] Audit and complete status and orchestration JSON projections in `src/cli/output_session_status.rs` and `src/cli/output_orchestrate.rs`
- [X] T013 [US2] Audit and complete inspect and trace projections in `src/cli/inspect/projections.rs` and `src/cli/output_run_trace.rs`
- [X] T014 [US2] Run the focused US2 regression set in `tests/unit/session_record.rs`, `tests/unit/cli_output.rs`, and `tests/contract/host_command_output_contract.rs`

**Checkpoint**: Every supported runtime projection exposes the same additive
readiness decision without breaking older snapshots.

---

## Phase 5: User Story 3 - Resume Planning Through Assistant Surfaces (Priority: P3)

**Goal**: Keep assistant-specific planning commands thin and symmetric over
the CLI/runtime contract.

**Independent Test**: Validate each supported planning asset and confirm that
it renders the standardized summary sections and forwards the same single
`phase_request` recovery flow.

### Implementation

- [X] T015 [P] [US3] Align the Antigravity and Claude planning assets with the standardized summary and recovery contract in `assistant/antigravity/commands/boundline-plan.md` and `assistant/claude/commands/boundline-plan.md`
- [X] T016 [P] [US3] Align the Codex and Copilot planning assets with the standardized summary and recovery contract in `assistant/codex/commands/boundline-plan.md` and `assistant/copilot/prompts/boundline-plan.prompt.md`
- [X] T017 [US3] Run assistant parity regressions in `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_host_parity_contract.rs`, and `tests/unit/assistant_assets.rs`

**Checkpoint**: All supported assistants remain projections over one runtime
contract.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close the release surface and verify repository quality gates.

- [X] T018 [P] Remove the duplicate `opus-4.8` provider entry found during the public-doc refresh, preserve the current relevant provider families, and refresh the catalog metadata date in `assistant/catalog/model-catalog.toml`
- [X] T019 [P] Document the plan-quality gate, one-question recovery flow, additive projections, and current release line in `README.md`, `docs/runtime/plan.md`, `docs/runtime/phase-requests.md`, `docs/runtime/status.md`, `docs/runtime/inspect.md`, `docs/runtime/trace.md`, `docs/guide/common-workflows.md`, `docs/guide/introduction.md`, and `docs/architecture/runtime-model.md`
- [X] T020 [P] Update operator and architecture guidance for the planning-readiness contract in `tech-docs/architecture.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, and `tech-docs/host-orchestration-contract.md`
- [X] T021 [P] Bump release metadata to `0.67.0` in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `assistant/plugin-metadata.json`, and `assistant/global/manifest.json`
- [X] T022 [P] Add the `0.67.0` WinGet release manifests under `distribution/winget/manifests/a/ApplyThe/Boundline/0.67.0/`
- [X] T023 [P] Record the release summary and delivered roadmap slice in `CHANGELOG.md`, `docs/roadmap/index.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/features/README.md`, and `roadmap/features/03-plan-quality-contract.md`
- [X] T024 Run `cargo fmt` and verify formatting with `cargo fmt --check`
- [X] T025 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [X] T026 Run focused tests with `cargo test --test unit`, `cargo test --test contract`, and `cargo test --test integration human_input_capture_flow::`
- [X] T027 Run the full regression suite with `cargo test` and resolve any failures
- [X] T028 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [X] T029 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every changed or created implementation file, and add tests until patch coverage is at least 95 percent
- [X] T030 Validate the isolated scenarios in `specs/067-plan-quality-contract/quickstart.md` without running Boundline CLI commands against the repository root

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
- T015 and T016 can run in parallel.
- T018 through T023 can run in parallel after runtime behavior stabilizes.

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational regressions.
2. Complete US1 admission behavior.
3. Validate one blocked and recovered isolated session.
4. Proceed to additive projections and assistant surfaces.

### Quality Rule

Do not treat formatting, clippy, tests, or patch coverage as deferred release
work. The feature is complete only when `cargo fmt --check`, strict clippy,
the regression suite, and at least 95 percent changed-file patch coverage pass.
