# Tasks: Multi-Agent Review & Voting

**Input**: Design documents from `/specs/007-multi-agent-review/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new executable review behavior, vote resolution, adjudication, duplicate-trigger handling, session projection, and trace-visible evidence on top of the execution engine.

**Organization**: Tasks are grouped by user story so each bounded review slice can be implemented, validated, and reviewed independently.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Register the review domain surface and extend the test harnesses before review behavior changes.

- [ ] T001 Wire review module exports and harness entries in src/domain.rs, src/lib.rs, tests/unit.rs, tests/contract.rs, and tests/integration.rs
- [ ] T002 [P] Scaffold review unit and contract test files in tests/unit/review_profile.rs, tests/unit/review_voting.rs, tests/contract/review_profile_contract.rs, tests/contract/review_run_contract.rs, tests/contract/review_trace_contract.rs, and tests/contract/review_adjudication_contract.rs
- [ ] T003 [P] Extend workspace fixture helpers for review councils, triggers, and duplicate-trigger scenarios in tests/support/workspace_fixture.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared review model, manifest loading, trace events, and state projection that every story relies on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create review domain types, validation rules, terminal outcomes, and vote-resolution primitives in src/domain/review.rs
- [X] T005 [P] Extend execution-profile loading and validation for review configuration in src/domain/execution.rs and src/fixture.rs
- [X] T006 [P] Extend task context and session projection for review trigger, vote, outcome, and participants in src/domain/task_context.rs and src/domain/session.rs
- [X] T007 [P] Add trace event and summary support for review lifecycle evidence in src/domain/trace.rs and src/cli/output.rs
- [X] T008 Implement shared duplicate-trigger and adjudication primitives in src/domain/review.rs, src/orchestrator/engine.rs, and src/orchestrator/session_runtime.rs after T005, T006, and T007 complete

**Checkpoint**: Foundation ready - review councils can be described, loaded, persisted, and traced inside the existing execution lifecycle.

---

## Phase 3: User Story 1 - Review A Delivery Result (Priority: P1) 🎯 MVP

**Goal**: Let Synod append one bounded review council to a reviewable delivery result and accept the output when the council vote resolves in favor of approval.

**Independent Test**: Run Synod against a temporary workspace with review configuration, confirm reviewer findings are captured, the vote resolves to acceptance, and run/status/inspect expose the accepted review outcome.

### Tests for User Story 1

- [X] T009 [P] [US1] Add unit coverage for review profile validation and accepted vote computation in tests/unit/review_profile.rs and tests/unit/review_voting.rs
- [ ] T010 [P] [US1] Add contract coverage for review profile parsing and accepted run output in tests/contract/review_profile_contract.rs and tests/contract/review_run_contract.rs
- [ ] T011 [P] [US1] Add integration coverage for accepted review runs and session-backed review flow in tests/integration/cli_review_run.rs and tests/integration/session_review_flow.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement manifest-backed review profile parsing, deterministic reviewer findings, and reviewer agent registration in src/fixture.rs and src/domain/execution.rs
- [X] T013 [US1] Append sequential reviewer agent steps and vote-resolution evaluation to reviewable runs in src/fixture.rs, src/orchestrator/planner.rs, and src/orchestrator/session_runtime.rs
- [X] T014 [US1] Persist reviewer findings, participation records, vote summaries, and accepted review state in src/orchestrator/engine.rs, src/orchestrator/session_runtime.rs, and src/domain/task_context.rs
- [X] T015 [US1] Render accepted review trigger, reviewers, vote summary, and outcome in src/cli/run.rs, src/cli/output.rs, and src/cli/session.rs

**Checkpoint**: User Story 1 is complete when a bounded review council can accept one delivery result with inspectable evidence.

---

## Phase 4: User Story 2 - Resolve Reviewer Disagreement (Priority: P2)

**Goal**: Keep conflicting reviewer input inside one bounded review lifecycle through explicit voting, optional adjudication, and terminal rejection, escalation, or failure.

**Independent Test**: Run review scenarios with blocking findings, ties, missing reviewer output, and duplicate triggers; verify that Synod rejects, adjudicates, escalates, or fails explicitly within configured limits.

### Tests for User Story 2

- [ ] T016 [P] [US2] Add unit coverage for majority voting, weighted voting, reject-on-blocking, and adjudication triggers in tests/unit/review_voting.rs and tests/unit/coverage_additional.rs
- [ ] T017 [P] [US2] Add contract coverage for review trace events, adjudication semantics, and non-success review run output in tests/contract/review_trace_contract.rs, tests/contract/review_adjudication_contract.rs, and tests/contract/review_run_contract.rs
- [ ] T018 [P] [US2] Add integration coverage for disagreement, adjudication, escalation, failure, and duplicate-trigger handling in tests/integration/cli_review_inspection.rs and tests/integration/session_review_flow.rs

### Implementation for User Story 2

- [ ] T019 [US2] Implement majority and weighted vote resolution, reject-on-blocking behavior, and `needs_adjudication` decisions in src/domain/review.rs and src/orchestrator/engine.rs
- [ ] T020 [US2] Implement bounded adjudication, explicit escalated versus failed review outcomes, and duplicate-trigger recording in src/orchestrator/engine.rs, src/orchestrator/session_runtime.rs, and src/domain/review.rs
- [ ] T021 [US2] Persist and render non-success review terminal reasons, failed reviewer participation, and adjudication evidence in src/domain/trace.rs, src/cli/inspect.rs, and src/cli/output.rs
- [ ] T022 [US2] Harden malformed reviewer output, unavailable reviewer handling, and diagnostics for invalid review configuration in src/fixture.rs and src/cli/diagnostics.rs

**Checkpoint**: User Story 2 is complete when disagreement and reviewer failures terminate explicitly with vote-visible evidence.

---

## Phase 5: User Story 3 - Inspect Review Evidence (Priority: P3)

**Goal**: Make the latest review trigger, findings, vote, and adjudication outcome easy to inspect from the existing CLI surfaces.

**Independent Test**: After accepted and non-success review runs, verify that `status`, `next`, and `inspect` all expose the latest review evidence without manual reconstruction.

### Tests for User Story 3

- [ ] T023 [P] [US3] Add contract coverage for session review projection and inspect review summaries in tests/contract/review_trace_contract.rs and tests/contract/session_command_contract.rs
- [ ] T024 [P] [US3] Add integration coverage for review-aware status, next, and inspect flows in tests/integration/cli_review_inspection.rs and tests/integration/session_review_flow.rs

### Implementation for User Story 3

- [ ] T025 [US3] Project latest review trigger, vote summary, outcome, and participants into session status and next guidance in src/domain/session.rs and src/cli/session.rs
- [ ] T026 [US3] Extend trace summarization and inspect rendering for reviewer findings, vote tallies, adjudication, and duplicate-trigger events in src/cli/inspect.rs and src/domain/trace.rs
- [ ] T027 [US3] Add review-aware CLI output and assistant-facing summaries in src/cli/output.rs, src/cli/run.rs, and assistant/README.md without making assistants runtime reviewers in the initial slice

**Checkpoint**: All review outcomes are now inspectable through the existing bounded CLI surfaces.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, versioning, coverage, and full validation for the review slice.

- [X] T028 [P] Add dedicated voting documentation in docs/review-voting.md covering majority voting, weighted voting, reject-on-blocking behavior, adjudication, malformed reviewer output, and accepted/rejected/escalated/failed outcomes with worked examples
- [ ] T029 [P] Update feature documentation in README.md, ROADMAP.md, AGENTS.md, assistant/README.md, and specs/007-multi-agent-review/quickstart.md
- [ ] T030 [P] Keep crate version at 0.7.0 in Cargo.toml and Cargo.lock
- [ ] T031 [P] Raise source coverage for new review paths in tests/unit/coverage_additional.rs, tests/unit/review_profile.rs, tests/unit/review_voting.rs, and review integration tests
- [ ] T032 Run formatting, lint, test, and coverage validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP bounded review path.
- **Phase 4: User Story 2**: Depends on Phase 2 and builds on the same council runtime created for US1.
- **Phase 5: User Story 3**: Depends on Phase 2 and is safest once review evidence is stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but depends on the review evidence model established in US1.
- **US3 (P3)**: Starts after Foundational and is safest once US1 and US2 have stabilized the review surfaces.

### Within Each User Story

- Contract, unit, and integration coverage should be written first and observed failing before implementation.
- Domain and manifest changes should land before CLI rendering or session projection work that consumes them.
- Vote resolution and adjudication rules should be stable before inspect and status messaging is finalized.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel once T004 exists; T008 should sequence the shared runtime rules.
- **US1**: T009, T010, and T011 can run in parallel; T012 and T013 can overlap once the domain model is stable.
- **US2**: T016, T017, and T018 can run in parallel; T019 and T022 can run in parallel after shared review primitives exist.
- **US3**: T023 and T024 can run in parallel; T025 and T026 can overlap once review evidence is persisted.
- **Polish**: T028, T029, T030, and T031 can run in parallel before the final validation task T032.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T009 Add unit coverage for review profile validation and accepted vote computation in tests/unit/review_profile.rs and tests/unit/review_voting.rs"
Task: "T010 Add contract coverage for review profile parsing and accepted run output in tests/contract/review_profile_contract.rs and tests/contract/review_run_contract.rs"
Task: "T011 Add integration coverage for accepted review runs and session-backed review flow in tests/integration/cli_review_run.rs and tests/integration/session_review_flow.rs"

# Split manifest loading and council execution after tests exist:
Task: "T012 Implement manifest-backed review profile parsing, deterministic reviewer findings, and reviewer agent registration in src/fixture.rs and src/domain/execution.rs"
Task: "T013 Append sequential reviewer agent steps and vote-resolution evaluation to reviewable runs in src/fixture.rs, src/orchestrator/planner.rs, and src/orchestrator/session_runtime.rs"
```

## Parallel Example: User Story 2

```bash
# Validate disagreement behavior together:
Task: "T016 Add unit coverage for majority voting, weighted voting, reject-on-blocking, and adjudication triggers in tests/unit/review_voting.rs and tests/unit/coverage_additional.rs"
Task: "T017 Add contract coverage for review trace events and non-success review run output in tests/contract/review_trace_contract.rs and tests/contract/review_run_contract.rs"
Task: "T018 Add integration coverage for disagreement, adjudication, escalation, failure, and duplicate-trigger handling in tests/integration/cli_review_inspection.rs and tests/integration/session_review_flow.rs"

# Then split core resolution and hardening work:
Task: "T019 Implement majority and weighted vote resolution, reject-on-blocking behavior, and `needs_adjudication` decisions in src/domain/review.rs and src/orchestrator/engine.rs"
Task: "T022 Harden malformed reviewer output, unavailable reviewer handling, and diagnostics for invalid review configuration in src/fixture.rs and src/cli/diagnostics.rs"
```

## Parallel Example: User Story 3

```bash
# Validate status and inspect behavior together:
Task: "T023 Add contract coverage for session review projection and inspect review summaries in tests/contract/review_trace_contract.rs and tests/contract/session_command_contract.rs"
Task: "T024 Add integration coverage for review-aware status, next, and inspect flows in tests/integration/cli_review_inspection.rs and tests/integration/session_review_flow.rs"

# Then split status and inspect implementation:
Task: "T025 Project latest review trigger, vote summary, outcome, and participants into session status and next guidance in src/domain/session.rs and src/cli/session.rs"
Task: "T026 Extend trace summarization and inspect rendering for reviewer findings, vote tallies, adjudication, and duplicate-trigger events in src/cli/inspect.rs and src/domain/trace.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate one accepted review run on a temporary workspace.
5. Confirm reviewer evidence is visible before expanding disagreement handling and docs.

### Incremental Delivery

1. Deliver Setup + Foundational to establish the review council runtime and evidence model.
2. Deliver US1 to make bounded review acceptance available.
3. Deliver US2 to add disagreement handling, adjudication, and non-success review outcomes.
4. Deliver US3 to expose high-quality status and inspect evidence.
5. Finish with voting docs, versioning, coverage, and full validation.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared review foundation so the first increment already delivers real multi-reviewer quality control instead of documentation-only scaffolding.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for user-story tasks, and exact file paths.
- Voting documentation is a first-class deliverable in this slice because the user explicitly requires a dedicated explanation of how review decisions are reached.
- The crate version remains 0.7.0 for this slice and should stay aligned across Cargo.toml, Cargo.lock, and user-facing docs.
