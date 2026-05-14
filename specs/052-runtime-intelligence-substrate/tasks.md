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

- [x] T001 Update the Boundline workspace to `0.52.0` in `/Users/rt/workspace/apply-the/boundline/Cargo.toml`, `/Users/rt/workspace/apply-the/boundline/Cargo.lock`, release metadata, and assistant package manifests touched by this slice
- [x] T002 Create the 052 planning artifacts in `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/research.md`, `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/data-model.md`, `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/quickstart.md`, `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/contracts/runtime-index-contract.md`, and `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/contracts/substrate-trace-contract.md`
- [x] T003 [P] Record the required provider-doc audit and no-change catalog result in `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/research.md` and keep it aligned with `/Users/rt/workspace/apply-the/boundline/assistant/catalog/model-catalog.toml`
- [x] T004 [P] Create and maintain the closeout support artifacts in `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/decision-log.md` and `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/validation-report.md`

---

## Phase 2: Delivered Runtime Substrate Behavior

**Purpose**: Capture the implementation work already landed in code and tests

- [x] T005 [US1] Extend the shared substrate model and provenance helpers in `/Users/rt/workspace/apply-the/boundline/src/domain/goal_plan.rs`
- [x] T006 [US1] Implement deterministic bounded-context assembly and credibility mapping in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/goal_planner.rs`
- [x] T007 [US3] Project substrate context through `/Users/rt/workspace/apply-the/boundline/src/cli/inspect.rs` and `/Users/rt/workspace/apply-the/boundline/src/cli/output.rs`
- [x] T008 [US2] Keep Canon optional while aligning compatibility and governed-surface references in `/Users/rt/workspace/apply-the/boundline/src/domain/distribution.rs`, `/Users/rt/workspace/apply-the/boundline/src/adapters/governance_runtime.rs`, and related fixtures
- [x] T009 [P] [US1] Refresh the directly affected unit and integration coverage in `/Users/rt/workspace/apply-the/boundline/tests/unit/goal_plan_model.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/cli_output.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/decision_loop.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/distribution_metadata.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/governance_policy.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/task_context_state.rs`, `/Users/rt/workspace/apply-the/boundline/tests/unit/canon_native_cli.rs`, and `/Users/rt/workspace/apply-the/boundline/tests/integration/canon_default_governance_flow.rs`

---

## Phase 3: Delivered Workspace Validation

**Purpose**: Preserve the executed validation evidence already collected for the branch

- [x] T010 [Closeout] Record focused provenance and compatibility checks in `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/validation-report.md`
- [x] T011 [Closeout] Run `cargo test --no-run --all-targets` in `/Users/rt/workspace/apply-the/boundline`
- [x] T012 [Closeout] Run `cargo fmt --all --check` in `/Users/rt/workspace/apply-the/boundline`
- [x] T013 [Closeout] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in `/Users/rt/workspace/apply-the/boundline`
- [x] T014 [Closeout] Run `cargo nextest run --workspace --all-features` in `/Users/rt/workspace/apply-the/boundline`

---

## Phase 4: Completed Closeout Work

**Purpose**: Record the final package closure across docs, code comments, focused tests, and coverage

- [x] T015 [Closeout] Reconcile `spec.md`, `data-model.md`, `decision-log.md`, `contracts/runtime-index-contract.md`, `research.md`, and `validation-report.md` so they describe one consistent credibility model and closeout state
- [x] T016 [P] [Closeout] Align `/Users/rt/workspace/apply-the/boundline/ROADMAP.md` and `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md` to the real `0.52.0` runtime-substrate narrative
- [x] T017 [P] [Closeout] Add the missing in-code documentation and any focused test supplements required for `/Users/rt/workspace/apply-the/boundline/src/domain/review.rs`, `/Users/rt/workspace/apply-the/boundline/src/fixture.rs`, `/Users/rt/workspace/apply-the/boundline/src/orchestrator/engine.rs`, and the other modified Rust sources still in scope
- [x] T018 [Closeout] Run `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` in `/Users/rt/workspace/apply-the/boundline` and confirm at least 95% coverage for every modified Rust source file
- [x] T019 [Closeout] Update `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/validation-report.md`, refresh `/Users/rt/workspace/apply-the/boundline/lcov.info`, and move `/Users/rt/workspace/apply-the/boundline/specs/052-runtime-intelligence-substrate/spec.md` out of Draft only after the closeout evidence is complete

---

## Dependencies & Execution Order

- All phases are complete, and this ledger now reflects a closed 052 package.
- The validation report is the authoritative record for the final command history and coverage evidence.

## Notes

- The first delivered task remained the version bump, as required.
- The modified-file coverage verification finished above the required 95% threshold.
- Canon enrichment remains optional throughout the implemented slice.