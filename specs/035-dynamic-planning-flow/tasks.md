# Tasks: Dynamic Planning And Flow Inference

**Input**: Design documents from `/specs/035-dynamic-planning-flow/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes native planning authority, proposal confirmation, bounded replanning, runtime routing, CLI summaries, and persisted trace semantics.

**Organization**: Tasks are grouped by user story so each slice remains independently testable while still delivering one complete macrofeature.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Finalize the 035 feature pack and test harness entry points before runtime changes.

- [x] T001 Confirm and keep synchronized `specs/035-dynamic-planning-flow/plan.md`, `specs/035-dynamic-planning-flow/research.md`, `specs/035-dynamic-planning-flow/data-model.md`, `specs/035-dynamic-planning-flow/contracts/`, and `specs/035-dynamic-planning-flow/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references in `tests/unit.rs`, `tests/contract.rs`, and `tests/integration.rs` if 035 introduces new test modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared proposal, confirmation, and revision primitives used by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `src/domain/goal_plan.rs` with explicit plan proposal state, verification strategy, proposal rationale, revision lineage, and validation rules for proposed, confirmed, and superseded plans
- [x] T004 [P] Extend `src/domain/session.rs` and `src/domain/trace.rs` with planning proposal projection fields, confirmation blockers, revision summaries, and inspectable evidence headlines shared by runtime and CLI surfaces
- [x] T005 [P] Extend `src/cli.rs` and `src/cli/session.rs` with bounded `plan --confirm` and `plan --replan` command semantics plus operator-visible error handling for missing or invalid proposals
- [x] T006 [P] Extend `tests/unit/goal_plan_model.rs`, `tests/unit/session_model.rs`, and `tests/unit/session_record.rs` with foundational coverage for proposal state transitions, confirmation blocking, and revision lineage invariants

**Checkpoint**: Proposal lifecycle, session projection, and CLI entry semantics exist and can support all user stories.

---

## Phase 3: User Story 1 - Infer A Bounded Plan From Workspace Evidence (Priority: P1) 🎯 MVP

**Goal**: Make `goal -> plan` infer a credible flow, target set, and verification strategy from workspace evidence instead of from keyword-first matching and a static task template.

**Independent Test**: Run representative native planning scenarios and verify the proposed flow and tasks are derived from files, symbols, tests, and acceptance cues rather than keyword matches alone.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for evidence-driven proposal payloads in `tests/contract/goal_plan_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for evidence-driven `goal -> plan` behavior in `tests/integration/session_native_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for evidence scoring, target selection, and verification-strategy inference in `tests/unit/flow_inference.rs` and `tests/unit/goal_planner.rs`

### Implementation for User Story 1

- [x] T010 [US1] Replace keyword-first flow inference in `src/orchestrator/flow_inference.rs` with evidence scoring derived from context inputs, selected targets, workspace signals, traces, and workflow guardrails
- [x] T011 [US1] Extend `src/orchestrator/goal_planner.rs` to build a planning evidence bundle, infer bounded targets, derive verification strategy, and shape planned tasks from that evidence instead of a static analyze/fix/test sequence
- [x] T012 [US1] Extend `src/orchestrator/session_runtime.rs` so `plan_task` persists an unconfirmed proposal or an insufficient-context stop with evidence-backed rationale on the native path

**Checkpoint**: Native planning now produces an evidence-driven proposal or an explicit bounded stop.

---

## Phase 4: User Story 2 - Confirm Or Adjust The Proposed Plan Explicitly (Priority: P2)

**Goal**: Surface the proposed plan across existing CLI surfaces and require explicit confirmation before execution continues.

**Independent Test**: After `plan`, verify that `status`, `next`, `run`, and `inspect` surface inferred flow, proposal state, evidence rationale, and any bounded operator action needed before execution proceeds.

### Tests for User Story 2

- [x] T013 [P] [US2] Add contract coverage for proposal confirmation and CLI-visible blocking semantics in `tests/contract/runtime_refoundation_contract.rs`
- [x] T014 [P] [US2] Add integration coverage for `plan --confirm`, `status`, `next`, and blocked `run` behavior in `tests/integration/session_native_flow.rs`
- [x] T015 [P] [US2] Add unit coverage for proposal rendering and execution-path projection in `tests/unit/runtime_routing.rs`, `tests/unit/session_model.rs`, and `tests/unit/flow_confirmation.rs`

### Implementation for User Story 2

- [x] T016 [US2] Extend `src/orchestrator/session_runtime.rs` and `src/domain/session.rs` so native routing distinguishes proposed, confirmed, insufficient-context, and compatibility-authoritative planning states
- [x] T017 [US2] Extend `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs` to render proposal summaries, evidence rationale, verification strategy, and next-command guidance on `plan`, `status`, `next`, `run`, and `inspect`
- [x] T018 [US2] Extend `src/domain/trace.rs` and `src/orchestrator/session_runtime.rs` so proposal confirmation and blocked-run reasons persist in authoritative trace summaries

**Checkpoint**: Operators can inspect and confirm the proposed plan from the normal session-native surfaces.

---

## Phase 5: User Story 3 - Replan Boundedly When New Evidence Changes The Best Path (Priority: P3)

**Goal**: Allow bounded replanning to revise targets, verification strategy, or flow choice when new evidence invalidates the prior proposal, while preserving rationale and acceptance-boundary visibility.

**Independent Test**: Run representative scenarios where initial analysis or validation invalidates the first proposal and verify that Boundline records a bounded new revision instead of silently mutating or continuing the old plan.

### Tests for User Story 3

- [x] T019 [P] [US3] Add contract coverage for bounded replan revision lineage in `tests/contract/goal_plan_contract.rs`
- [x] T020 [P] [US3] Add integration coverage for bounded replanning and explicit stop behavior in `tests/integration/retry_and_replan.rs` and `tests/integration/runtime_refoundation_failure.rs`
- [x] T021 [P] [US3] Add unit coverage for revision supersession, no-op replans, and guardrail conflicts in `tests/unit/goal_planner.rs`, `tests/unit/runtime_routing.rs`, and `tests/unit/session_record.rs`

### Implementation for User Story 3

- [x] T022 [US3] Extend `src/orchestrator/goal_planner.rs` and `src/orchestrator/flow_inference.rs` to compare fresh evidence against the active proposal and produce bounded replan revisions or explicit no-credible-plan stops
- [x] T023 [US3] Extend `src/orchestrator/session_runtime.rs` and `src/domain/goal_plan.rs` to supersede prior revisions, preserve acceptance-boundary continuity, and require reconfirmation before resumed execution
- [x] T024 [US3] Extend `src/cli/output.rs`, `src/cli/inspect.rs`, and `src/domain/trace.rs` to surface revision lineage, changed fields, explicit stop reasons, and workflow-guardrail conflicts

**Checkpoint**: Replanning is bounded, inspectable, and authoritative.

---

## Phase 6: User Story 4 - Ship The Dynamic Planner As 0.35.0 (Priority: P4)

**Goal**: Close the feature as a release-aligned macrofeature with updated version metadata, product narrative, roadmap closure, and repository validation evidence.

**Independent Test**: Follow the updated docs on a representative workspace, run the release validation suite, and verify that the version bump, roadmap, docs, changelog, formatting, linting, and coverage all align with the shipped dynamic planning model.

### Tests for User Story 4

- [x] T025 [P] [US4] Refresh focused coverage assertions for touched Rust files via `lcov.info` and supporting validation commands

### Implementation for User Story 4

- [x] T026 [US4] Bump crate version to `0.35.0` in `Cargo.toml` and `Cargo.lock`
- [x] T027 [US4] Update impacted docs and release narrative in `README.md`, `docs/`, `CONTRIBUTING.md`, `CHANGELOG.md`, and `AGENTS.md`
- [x] T028 [US4] Update `ROADMAP.md` to mark Spec 035 as delivered and remove it from the remaining future macrofeature line
- [x] T029 [US4] Update assistant guidance impacted by dynamic planning in `assistant/README.md`, `assistant/claude/commands/`, `assistant/codex/commands/`, and `assistant/copilot/prompts/`

**Checkpoint**: Release artifacts describe `0.35.0` consistently.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Validate the complete slice and close remaining quality gaps.

- [x] T030 [P] Run formatting with `cargo fmt --all`
- [x] T031 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T032 Run compile-oriented and broader Rust validation for the slice with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T033 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T034 Mark completed tasks in `specs/035-dynamic-planning-flow/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 proposal generation and foundational confirmation primitives.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because replanning reuses proposal state, evidence vocabulary, and confirmation routing.
- **User Story 4 (Phase 6)**: Depends on all runtime behavior being complete.
- **Polish (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if new test modules are needed.
- T004, T005, and T006 can run in parallel after T003 defines the proposal lifecycle primitives.
- Within each user story, test tasks marked `[P]` can be developed in parallel before the implementation tasks touching the same files.
- T025 can be prepared while release docs are being updated, but final coverage confirmation must wait for completed code.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that `plan` produces an evidence-driven proposal or an explicit bounded stop.
3. Use that proposal model as the base for confirmation and bounded replanning.

### Incremental Delivery

1. Add proposal lifecycle primitives and CLI semantics.
2. Make the planner infer flow, targets, and verification strategy from evidence.
3. Project proposal state through `status`, `next`, `run`, and `inspect`.
4. Add bounded replanning and revision lineage.
5. Close the release with version, docs, roadmap, assistant guidance, coverage, linting, and formatting.

## Notes

- This feature stays macro-level in value but bounded in implementation: it strengthens the existing native planning runtime instead of introducing a second planner.
- Explicit compatibility continuity remains outside the primary scope except where it must preserve authoritative route explanation.
- The final summary must include a descriptive commit message for the completed feature.