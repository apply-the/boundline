# Tasks: Human-Facing Brief Ingestion

**Input**: Design documents from `/specs/010-human-brief-ingestion/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new CLI entry behavior, bounded clarification stops, persisted input provenance, governed human-input flows, and user-facing inspection guarantees.

**Organization**: Tasks are grouped by user story so each slice can be implemented, tested, and shipped independently.

## Delivery Status (v0.10.0)

The v0.10.0 release now reflects the full delivered scope of this feature.
Boundline accepts direct text, repeated Markdown briefs, referenced workspace
Markdown, clarification-aware task drafting, and business-level governance
intent across `goal`, direct-input `run`, `status`, and `inspect`.
All tasks below are complete and describe the delivered implementation.


## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the test harness and workspace fixtures needed by every human-input story.

- [X] T001 Wire human-input test harness registrations in tests/unit.rs, tests/contract.rs, and tests/integration.rs
- [X] T002 [P] Add reusable Markdown brief workspace fixtures and helper builders in tests/support/workspace_fixture.rs
- [X] T003 [P] Scaffold feature-focused test files in tests/contract/human_input_cli_contract.rs, tests/contract/human_input_session_contract.rs, tests/contract/human_input_governance_contract.rs, tests/integration/human_input_capture_flow.rs, tests/integration/human_input_multi_source_flow.rs, tests/integration/human_input_governance_flow.rs, tests/unit/human_input_ingestion.rs, and tests/unit/human_input_governance.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared human-input models, normalization rules, and observability primitives that all stories need.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create shared external-input, brief-bundle, clarification, and derived-draft models in src/domain/session.rs, src/domain/task.rs, and src/domain/task_context.rs
- [X] T005 [P] Extend developer command parsing and validation for `--goal`, repeated `--brief`, and governance flags in src/cli.rs and src/cli/session.rs
- [X] T006 [P] Implement workspace-bounded Markdown resolution, precedence, deduplication, and clarification helpers in src/orchestrator/session_runtime.rs and src/fixture.rs
- [X] T007 [P] Extend session, trace, status, and inspect projection primitives for input provenance and clarification state in src/domain/trace.rs, src/domain/session.rs, src/cli/output.rs, and src/cli/inspect.rs
- [X] T008 Implement direct-run and assistant handoff for normalized human input in src/cli/run.rs, src/orchestrator/engine.rs, and src/orchestrator/session_runtime.rs

**Checkpoint**: Foundation ready - Boundline can parse human-facing input, normalize it deterministically, persist it with the active session, and expose enough state for later story-specific behavior.

---

## Phase 3: User Story 1 - Start Work From Human Input (Priority: P1) 🎯 MVP

**Goal**: Let a developer start a bounded task from plain text or one authored Markdown brief without authoring an internal manifest.

**Independent Test**: Start a new task from either plain text or one Markdown brief, then verify that Boundline records the request, derives a bounded task draft, and either plans the work or stops with one explicit clarification without asking for internal files.

### Tests for User Story 1

- [X] T009 [P] [US1] Add contract coverage for `goal` and direct-input `run` with direct text or one Markdown brief in tests/contract/human_input_cli_contract.rs and tests/contract/session_command_contract.rs
- [X] T010 [P] [US1] Add integration coverage for `start -> capture -> flow -> plan` with direct text and one brief in tests/integration/human_input_capture_flow.rs and tests/integration/session_cli_flow.rs
- [X] T011 [P] [US1] Add unit coverage for brief-bundle persistence and clarification blocking in tests/unit/human_input_ingestion.rs, tests/unit/session_record.rs, and tests/unit/task_context_state.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement direct-text and single-brief normalization plus persisted `AuthoredBriefBundle` state in src/orchestrator/session_runtime.rs, src/domain/session.rs, src/domain/task.rs, and src/domain/task_context.rs
- [X] T013 [US1] Connect planning-ready versus clarification-blocked task draft derivation for session commands and direct `run` in src/cli/session.rs, src/cli/run.rs, src/fixture.rs, and src/orchestrator/engine.rs
- [X] T014 [US1] Surface captured brief summaries and clarification headlines in src/domain/trace.rs, src/cli/output.rs, src/cli/inspect.rs, and src/cli/session.rs

**Checkpoint**: User Story 1 is complete when Boundline can start from direct text or one Markdown brief, persist the accepted brief, and stop credibly when clarification is required.

---

## Phase 4: User Story 2 - Reuse Multiple Authored Sources (Priority: P2)

**Goal**: Let a developer combine multiple Markdown briefs and text-referenced workspace documents into one bounded, inspectable authored bundle.

**Independent Test**: Start a task from multiple Markdown files and a text note that references existing repository documents, then verify that Boundline resolves the inputs into one bounded bundle with visible provenance and stops explicitly on missing or conflicting sources.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for repeated `--brief`, text-referenced Markdown, and inspectable source provenance in tests/contract/human_input_cli_contract.rs, tests/contract/human_input_session_contract.rs, and tests/contract/trace_summary_contract.rs
- [X] T016 [P] [US2] Add integration coverage for multi-brief capture, text-referenced docs, deduplication, and missing-source failure in tests/integration/human_input_multi_source_flow.rs and tests/integration/cli_trace_inspection.rs
- [X] T017 [P] [US2] Add unit coverage for workspace-bounded path resolution, precedence, deduplication, and derived-draft validation in tests/unit/human_input_ingestion.rs and tests/unit/cli_output.rs

### Implementation for User Story 2

- [X] T018 [US2] Implement multi-source resolution, canonical workspace path handling, and deterministic deduplication in src/orchestrator/session_runtime.rs, src/fixture.rs, and src/domain/task_context.rs
- [X] T019 [US2] Persist accepted source provenance, conflict metadata, and resumed-session continuity in src/domain/session.rs, src/adapters/session_store.rs, and src/domain/task.rs
- [X] T020 [US2] Extend `status`, `inspect`, and trace rendering for ordered source sets, deduplication outcomes, and source-specific failure reasons in src/domain/trace.rs, src/cli/output.rs, and src/cli/inspect.rs

**Checkpoint**: User Stories 1 and 2 are complete when Boundline can reuse multiple authored sources, preserve precedence visibly, and refuse silent merges or omissions.

---

## Phase 5: User Story 3 - Govern Human-Facing Runs Without Internal Wiring (Priority: P3)

**Goal**: Let a developer declare governance intent in business terms alongside the authored brief and carry that intent through the existing governed execution path.

**Independent Test**: Start a governed task from human-authored input plus business-level governance values, then verify that Boundline maps them into internal governance behavior and reports blocked or approval-gated states without asking for internal configuration.

### Tests for User Story 3

- [X] T021 [P] [US3] Add contract coverage for human governance intent flags and user-facing governed status projections in tests/contract/human_input_governance_contract.rs and tests/contract/governance_session_contract.rs
- [X] T022 [P] [US3] Add integration coverage for governed human-input runs, missing business values, and approval or blocking states in tests/integration/human_input_governance_flow.rs and tests/integration/session_governance_flow.rs
- [X] T023 [P] [US3] Add unit coverage for human governance intent normalization and mapping rules in tests/unit/human_input_governance.rs and tests/unit/governance_policy.rs

### Implementation for User Story 3

- [X] T024 [US3] Implement human governance intent parsing and normalization for `goal` and direct `run` in src/cli.rs, src/cli/session.rs, and src/domain/session.rs
- [X] T025 [US3] Map normalized governance intent into the existing governance runtime request and task-draft flow in src/orchestrator/governance.rs, src/adapters/governance_runtime.rs, src/domain/governance.rs, and src/fixture.rs
- [X] T026 [US3] Project governed human-input status, clarification, and next-step guidance through src/domain/session.rs, src/cli/output.rs, src/cli/session.rs, and src/cli/inspect.rs

**Checkpoint**: All user stories are complete when governed human-input runs can start from business values, stop explicitly for missing business context, and expose approval or blocked state through normal session surfaces.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, release metadata, coverage, assistant-facing assets, and repository-wide validation for the 0.10.0 slice.

- [X] T027 [P] Update human-input operator docs and walkthroughs in README.md, tech-docs/getting-started.md, tech-docs/adaptive-execution.md, assistant/README.md, and specs/010-human-brief-ingestion/quickstart.md
- [X] T028 [P] Update release notes and roadmap references for the human-input slice in CHANGELOG.md, README.md, and ROADMAP.md
- [X] T029 [P] Raise source coverage for human-input normalization, provenance, clarification, and governed-entry branches in tests/unit/coverage_additional.rs, tests/unit/human_input_ingestion.rs, tests/unit/human_input_governance.rs, tests/integration/human_input_capture_flow.rs, tests/integration/human_input_multi_source_flow.rs, and tests/integration/human_input_governance_flow.rs
- [X] T030 [P] Bump crate and lockfile version references to 0.10.0 in Cargo.toml and Cargo.lock
- [X] T031 Harden assistant-facing command packs and prompt assets for the new human-input CLI surface in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, and assistant/copilot/prompts/
- [X] T032 Run formatting, lint, test, and coverage validation against Cargo.toml and lcov.info with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the MVP human-input entry slice.
- **Phase 4: User Story 2**: Depends on Phase 2 and is safest once the normalized brief bundle from US1 is stable.
- **Phase 5: User Story 3**: Depends on Phase 2 and is safest once the normalized input state and observability surfaces from US1 and US2 are stable.
- **Phase 6: Polish**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other user stories.
- **US2 (P2)**: Starts after Foundational but depends on the normalized brief bundle and session projections established in US1.
- **US3 (P3)**: Starts after Foundational but depends on the normalized input model from US1 and the provenance plus inspection surfaces stabilized in US2.

### Within Each User Story

- Contract, integration, and unit coverage should be written first and observed failing before implementation.
- Command parsing and domain normalization should land before session persistence and inspect output that consume the new state.
- Clarification and bounded-stop behavior should be stable before release docs and assistant assets are updated.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel after T004; T008 should follow once the shared normalization model exists.
- **US1**: T009, T010, and T011 can run in parallel; T012 and T013 can overlap once the brief-bundle types are stable.
- **US2**: T015, T016, and T017 can run in parallel; T018 and T019 can overlap once the precedence and provenance rules are fixed.
- **US3**: T021, T022, and T023 can run in parallel; T024 and T025 can overlap once governance intent normalization is defined.
- **Polish**: T027, T028, T029, T030, and T031 can run in parallel before the final validation task T032.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T009 Add contract coverage for capture and direct-input run with direct text or one Markdown brief in tests/contract/human_input_cli_contract.rs and tests/contract/session_command_contract.rs"
Task: "T010 Add integration coverage for start -> capture -> flow -> plan with direct text and one brief in tests/integration/human_input_capture_flow.rs and tests/integration/session_cli_flow.rs"
Task: "T011 Add unit coverage for brief-bundle persistence and clarification blocking in tests/unit/human_input_ingestion.rs, tests/unit/session_record.rs, and tests/unit/task_context_state.rs"

# Then split normalization and task-draft integration work:
Task: "T012 Implement direct-text and single-brief normalization plus persisted AuthoredBriefBundle state in src/orchestrator/session_runtime.rs, src/domain/session.rs, src/domain/task.rs, and src/domain/task_context.rs"
Task: "T013 Connect planning-ready versus clarification-blocked task draft derivation for session commands and direct run in src/cli/session.rs, src/cli/run.rs, src/fixture.rs, and src/orchestrator/engine.rs"
```

## Parallel Example: User Story 2

```bash
# Validate multi-source behavior together:
Task: "T015 Add contract coverage for repeated brief flags, text-referenced Markdown, and inspectable source provenance in tests/contract/human_input_cli_contract.rs, tests/contract/human_input_session_contract.rs, and tests/contract/trace_summary_contract.rs"
Task: "T016 Add integration coverage for multi-brief capture, text-referenced docs, deduplication, and missing-source failure in tests/integration/human_input_multi_source_flow.rs and tests/integration/cli_trace_inspection.rs"
Task: "T017 Add unit coverage for workspace-bounded path resolution, precedence, deduplication, and derived-draft validation in tests/unit/human_input_ingestion.rs and tests/unit/cli_output.rs"

# Then split resolution and persistence work:
Task: "T018 Implement multi-source resolution, canonical workspace path handling, and deterministic deduplication in src/orchestrator/session_runtime.rs, src/fixture.rs, and src/domain/task_context.rs"
Task: "T019 Persist accepted source provenance, conflict metadata, and resumed-session continuity in src/domain/session.rs, src/adapters/session_store.rs, and src/domain/task.rs"
```

## Parallel Example: User Story 3

```bash
# Validate governance intent behavior together:
Task: "T021 Add contract coverage for human governance intent flags and user-facing governed status projections in tests/contract/human_input_governance_contract.rs and tests/contract/governance_session_contract.rs"
Task: "T022 Add integration coverage for governed human-input runs, missing business values, and approval or blocking states in tests/integration/human_input_governance_flow.rs and tests/integration/session_governance_flow.rs"
Task: "T023 Add unit coverage for human governance intent normalization and mapping rules in tests/unit/human_input_governance.rs and tests/unit/governance_policy.rs"

# Then split parsing and runtime mapping work:
Task: "T024 Implement human governance intent parsing and normalization for capture and direct run in src/cli.rs, src/cli/session.rs, and src/domain/session.rs"
Task: "T025 Map normalized governance intent into the existing governance runtime request and task-draft flow in src/orchestrator/governance.rs, src/adapters/governance_runtime.rs, src/domain/governance.rs, and src/fixture.rs"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Validate direct-text and single-brief starts, including one clarification stop.
5. Demo the human-facing entry path before expanding into multi-source provenance and governance intent.

### Incremental Delivery

1. Deliver Setup + Foundational to establish the human-input models, normalization rules, and observability primitives.
2. Deliver US1 so users can start work from text or one Markdown brief without a manifest.
3. Deliver US2 to reuse multiple authored sources with visible precedence and bounded failure handling.
4. Deliver US3 to carry governance intent through the existing governed runtime.
5. Finish with docs, coverage, assistant asset updates, and the version bump to 0.10.0.

### Suggested MVP Scope

- User Story 1 only.
- Keep User Stories 2 and 3 behind the shared foundation so the first increment already removes manifest authoring for normal human use.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for story tasks, and exact file paths.
- The release slice includes explicit tasks for coverage, version bump to 0.10.0, and documentation updates because those are part of shipping this feature credibly.
- Coverage validation uses the repository-standard `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` workflow.