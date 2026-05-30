---
description: "Task list for runtime intelligence substrate implementation"
---

# Tasks: Runtime Intelligence Substrate

**Input**: Design documents from `/specs/052-runtime-intelligence-substrate/`
**Prerequisites**: plan.md, spec.md

**Tests**: Validation tasks remain mandatory because this slice changes
planning behavior, failure handling, trace projection, and CLI-visible runtime
surfaces.

**Status Note**: This file now acts as the execution ledger for the current
branch. The original draft task breakdown has been collapsed into the concrete
work that actually landed, plus the remaining closeout tasks still required to
finish the package truthfully.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story or closeout group this task belongs to (`US1`, `US2`, `US3`, `Closeout`)
- Include exact file paths in descriptions

## Phase 1: Delivered Setup And Planning Artifacts

**Purpose**: Record the spec, release, and provider-audit work already landed

- [x] T001 Update the Boundline workspace to `0.52.0` in `Cargo.toml`, `Cargo.lock`, release metadata, and assistant package manifests touched by this slice
- [x] T002 Create the 052 planning artifacts in `specs/052-runtime-intelligence-substrate/research.md`, `specs/052-runtime-intelligence-substrate/data-model.md`, `specs/052-runtime-intelligence-substrate/quickstart.md`, `specs/052-runtime-intelligence-substrate/contracts/runtime-index-contract.md`, and `specs/052-runtime-intelligence-substrate/contracts/substrate-trace-contract.md`
- [x] T003 [P] Record the required provider-doc audit and no-change catalog result in `specs/052-runtime-intelligence-substrate/research.md` and keep it aligned with `assistant/catalog/model-catalog.toml`
- [x] T004 [P] Create and maintain the closeout support artifacts in `specs/052-runtime-intelligence-substrate/decision-log.md` and `specs/052-runtime-intelligence-substrate/validation-report.md`

---

## Phase 2: Delivered Runtime Substrate Behavior

**Purpose**: Capture the implementation work already landed in code and tests

- [x] T005 [US1] Extend the shared substrate model and provenance helpers in `src/domain/goal_plan.rs`
- [x] T006 [US1] Implement deterministic bounded-context assembly and credibility mapping in `src/orchestrator/goal_planner.rs`
- [x] T007 [US3] Project substrate context through `src/cli/inspect.rs` and `src/cli/output.rs`
- [x] T008 [US2] Keep Canon optional while aligning compatibility and governed-surface references in `src/domain/distribution.rs`, `src/adapters/governance_runtime.rs`, and related fixtures
- [x] T009 [P] [US1] Refresh the directly affected unit and integration coverage in `tests/unit/goal_plan_model.rs`, `tests/unit/cli_output.rs`, `tests/unit/decision_loop.rs`, `tests/unit/distribution_metadata.rs`, `tests/unit/governance_policy.rs`, `tests/unit/task_context_state.rs`, `tests/unit/canon_native_cli.rs`, and `tests/integration/canon_default_governance_flow.rs`

---

## Phase 3: Delivered Workspace Validation

**Purpose**: Preserve the executed validation evidence already collected for the branch

- [x] T010 [Closeout] Record focused provenance and compatibility checks in `specs/052-runtime-intelligence-substrate/validation-report.md`
- [x] T011 [Closeout] Run `cargo test --no-run --all-targets` in `repo root`
- [x] T012 [Closeout] Run `cargo fmt --all --check` in `repo root`
- [x] T013 [Closeout] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `repo root`
- [x] T014 [Closeout] Run `cargo nextest run --workspace --all-features` in `repo root`

---

## Phase 4: Completed Closeout Work

**Purpose**: Record the final package closure across docs, code comments, focused tests, and coverage

- [x] T015 [Closeout] Reconcile `spec.md`, `data-model.md`, `decision-log.md`, `contracts/runtime-index-contract.md`, `research.md`, and `validation-report.md` so they describe one consistent credibility model and closeout state
- [x] T016 [P] [Closeout] Align `ROADMAP.md` and `CHANGELOG.md` to the real `0.52.0` runtime-substrate narrative
- [x] T017 [P] [Closeout] Add the missing in-code documentation and any focused test supplements required for `src/domain/review.rs`, `src/fixture.rs`, `src/orchestrator/engine.rs`, and the other modified Rust sources still in scope
- [x] T018 [Closeout] Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` in `repo root` and confirm at least 95% coverage for every modified Rust source file
- [x] T019 [Closeout] Update `specs/052-runtime-intelligence-substrate/validation-report.md`, refresh `lcov.info`, and move `specs/052-runtime-intelligence-substrate/spec.md` out of Draft only after the closeout evidence is complete

---

## Dependencies & Execution Order

- All phases are complete, and this ledger now reflects a closed 052 package.
- The validation report is the authoritative record for the final command history and coverage evidence.

## Notes

- The first delivered task remained the version bump, as required.
- The modified-file coverage verification finished above the required 95% threshold.
- Canon enrichment remains optional throughout the implemented slice.