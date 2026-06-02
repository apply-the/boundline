# Tasks: Human-Friendly Init and Model Routing

**Input**: Design documents from `/specs/011-init-model-routing/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it adds new CLI setup behavior, persistent config precedence, bounded destructive-write handling, review-role routing, and user-facing documentation guarantees.

**Organization**: Tasks are grouped by user story so each slice can deliver bounded, inspectable value independently.

## Delivery Status (v0.11.0 delivered)

This feature delivers a human-friendly `boundline init` workflow, editable runtime
and model routing with global and workspace precedence, differentiated review
and adjudication routing, and synchronized documentation for the full operator
path. The target release is `0.11.0`.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare the test harness, fixture helpers, and feature scaffolding for init and config workflows.

- [X] T001 Wire init and config test harness registrations in tests/unit.rs, tests/contract.rs, and tests/integration.rs
- [X] T002 [P] Add reusable workspace, home-config, and assistant-asset fixture helpers in tests/support/workspace_fixture.rs
- [X] T003 [P] Scaffold feature test files in tests/contract/init_cli_contract.rs, tests/contract/config_cli_contract.rs, tests/contract/routing_resolution_contract.rs, tests/integration/init_bootstrap_flow.rs, tests/integration/config_precedence_flow.rs, tests/integration/review_routing_flow.rs, tests/unit/init_templates.rs, tests/unit/config_resolution.rs, tests/unit/config_store.rs, and tests/unit/runtime_capability.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build the shared config, routing, and persistence primitives needed by every user story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Create shared init, routing, and precedence models in src/domain/configuration.rs, src/domain/review.rs, and src/domain/execution.rs
- [X] T005 [P] Add workspace and global config persistence primitives in src/adapters/config_store.rs and src/lib.rs
- [X] T006 [P] Extend developer command parsing and dispatch for `init` and `config` in src/cli.rs, src/cli/init.rs, and src/cli/config.rs
- [X] T007 [P] Add init-preview, config-inspection, and runtime-capability rendering primitives in src/cli/output.rs and src/cli/diagnostics.rs
- [X] T008 Implement bounded template builders, preview generation, and assistant setup helpers in src/fixture.rs and assistant/

**Checkpoint**: Foundation ready - Boundline can parse init/config commands, persist global and workspace config, resolve precedence deterministically, and describe pending setup changes before mutating repository files.

---

## Phase 3: User Story 1 - Initialize a Workspace Without Hand-Written JSON (Priority: P1)

**Goal**: Let a developer bootstrap a bounded Boundline workspace with `boundline init` instead of manually authoring internal JSON.

**Independent Test**: Run `boundline init` in a fresh repository, choose a template, confirm the preview, then verify that the workspace is ready for `doctor`, `start`, `goal`, and `run` without manual file editing; rerun init on an existing workspace and verify it previews changes instead of silently overwriting files.

### Tests for User Story 1

- [X] T009 [P] [US1] Add contract coverage for `boundline init` preview, confirmation, and destructive-update rules in tests/contract/init_cli_contract.rs
- [X] T010 [P] [US1] Add integration coverage for fresh-workspace bootstrap and safe rerun behavior in tests/integration/init_bootstrap_flow.rs
- [X] T011 [P] [US1] Add unit coverage for template generation and runtime capability detection in tests/unit/init_templates.rs and tests/unit/runtime_capability.rs

### Implementation for User Story 1

- [X] T012 [US1] Implement `boundline init` preview and apply flow in src/cli/init.rs, src/cli.rs, and src/fixture.rs
- [X] T013 [US1] Generate bounded workspace files and safe overwrite behavior in src/domain/execution.rs, src/adapters/config_store.rs, and src/cli/diagnostics.rs
- [X] T014 [US1] Scaffold and refresh repository-local assistant setup selections during init in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, assistant/copilot/prompts/, and assistant/gemini/
- [X] T015 [US1] Surface init summaries, next-step guidance, and missing-runtime warnings in src/cli/output.rs and README.md

**Checkpoint**: User Story 1 is complete when Boundline can bootstrap a fresh workspace, preview destructive changes on rerun, and leave the repository ready for the bounded execution flow without manual JSON authoring.

---

## Phase 4: User Story 2 - Configure and Understand Effective Routing Defaults (Priority: P2)

**Goal**: Let developers store global defaults, override them per workspace, and inspect the effective resolved routing without hand-editing files.

**Independent Test**: Save one global route, override part of it in a workspace, inspect the effective config, then unset the workspace value and verify that the lower-precedence route becomes effective again.

### Tests for User Story 2

- [X] T016 [P] [US2] Add contract coverage for `boundline config show`, `set`, and `unset` in tests/contract/config_cli_contract.rs and tests/contract/routing_resolution_contract.rs
- [X] T017 [P] [US2] Add integration coverage for global-plus-workspace precedence resolution in tests/integration/config_precedence_flow.rs
- [X] T018 [P] [US2] Add unit coverage for config persistence, precedence, and invalid route validation in tests/unit/config_store.rs and tests/unit/config_resolution.rs

### Implementation for User Story 2

- [X] T019 [US2] Implement global and workspace config load/save/unset behavior in src/adapters/config_store.rs and src/domain/configuration.rs
- [X] T020 [US2] Implement `boundline config show`, `set`, and `unset` in src/cli/config.rs, src/cli.rs, and src/cli/output.rs
- [X] T021 [US2] Resolve effective routing precedence and source attribution in src/domain/configuration.rs, src/fixture.rs, and src/domain/trace.rs
- [X] T022 [US2] Integrate effective routing snapshots into diagnostics and runtime preparation in src/cli/diagnostics.rs, src/cli/run.rs, and src/cli/session.rs

**Checkpoint**: User Stories 1 and 2 are complete when developers can bootstrap a workspace, manage routing defaults at global and workspace scope, and understand exactly why a resolved value was chosen.

---

## Phase 5: User Story 3 - Route Different Models for Delivery and Review Roles (Priority: P3)

**Goal**: Let developers assign different runtimes and models to planning, implementation, verification, reviewer roles, and adjudication, especially across voting councils.

**Independent Test**: Configure distinct reviewer roles and an adjudicator, inspect the effective routing, and verify that the routed review configuration remains valid and visible without collapsing to one generic profile.

### Tests for User Story 3

- [X] T023 [P] [US3] Add contract coverage for reviewer-role and adjudicator routing in tests/contract/routing_resolution_contract.rs and tests/contract/governance_session_contract.rs
- [X] T024 [P] [US3] Add integration coverage for review-role routing and assistant setup reuse in tests/integration/review_routing_flow.rs and tests/integration/cli_trace_inspection.rs
- [X] T025 [P] [US3] Add unit coverage for review-role route validation and effective reviewer overrides in tests/unit/config_resolution.rs and tests/unit/governance_policy.rs

### Implementation for User Story 3

- [X] T026 [US3] Extend review-domain models for reviewer-role and adjudicator routing in src/domain/review.rs and src/domain/configuration.rs
- [X] T027 [US3] Map resolved routing into review-ready execution and inspection surfaces in src/domain/execution.rs, src/fixture.rs, src/cli/output.rs, and src/cli/inspect.rs
- [X] T028 [US3] Validate reviewer-role conflicts, missing adjudicator routing, and unavailable review runtimes in src/cli/config.rs, src/cli/init.rs, and src/cli/diagnostics.rs
- [X] T029 [US3] Keep assistant-facing command packs aligned with routed review and setup behavior in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, assistant/copilot/prompts/, and assistant/gemini/

**Checkpoint**: All user stories are complete when Boundline supports distinct delivery and review routing, including differentiated voting-council participants and adjudication, with clear effective-config output.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finalize docs, release metadata, coverage, and repository-wide validation for the `0.11.0` slice.

- [X] T030 [P] Update operator docs and walkthroughs in README.md, tech-docs/getting-started.md, tech-docs/review-voting.md, tech-docs/adaptive-execution.md, tech-docs/configuration.md, assistant/README.md, and specs/011-init-model-routing/quickstart.md
- [X] T031 [P] Update release notes, roadmap references, and version metadata for `0.11.0` in CHANGELOG.md, ROADMAP.md, Cargo.toml, and Cargo.lock
- [X] T032 [P] Raise coverage for init, config precedence, and review-role routing in tests/unit/coverage_additional.rs, tests/unit/config_resolution.rs, tests/unit/runtime_capability.rs, tests/integration/init_bootstrap_flow.rs, tests/integration/config_precedence_flow.rs, and tests/integration/review_routing_flow.rs
- [X] T033 Run formatting, lint, tests, and coverage validation with `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies, can start immediately.
- **Phase 2: Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3: User Story 1**: Depends on Phase 2 and delivers the init-first operator path.
- **Phase 4: User Story 2**: Depends on Phase 2 and is safest once init persistence and preview primitives are stable.
- **Phase 5: User Story 3**: Depends on Phase 2 and is safest once routing precedence and config persistence are stable.
- **Phase 6: Polish**: Depends on all user stories being complete.

### User Story Dependencies

- **US1 (P1)**: Starts after Foundational and has no dependency on other stories.
- **US2 (P2)**: Starts after Foundational but depends on the config persistence and inspection surfaces stabilized in US1.
- **US3 (P3)**: Starts after Foundational but depends on the routing model and precedence behavior stabilized in US2.

### Within Each User Story

- Contract, integration, and unit coverage should be written first and observed failing before implementation.
- Domain and persistence models should land before CLI mutation commands that consume them.
- Preview and diagnostics behavior should be stable before docs or assistant assets are updated.
- Each story should pass its independent test before moving to the next priority.

## Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001.
- **Foundational**: T005, T006, and T007 can run in parallel after T004; T008 should follow once the shared models exist.
- **US1**: T009, T010, and T011 can run in parallel; T013 and T014 can overlap once `boundline init` command wiring exists.
- **US2**: T016, T017, and T018 can run in parallel; T019 and T021 can overlap once config schema is stable.
- **US3**: T023, T024, and T025 can run in parallel; T026 and T029 can overlap once review-role routing semantics are fixed.
- **Polish**: T030, T031, and T032 can run in parallel before the final validation task T033.

## Parallel Example: User Story 1

```bash
# Build the User Story 1 validation surface together:
Task: "T009 Add contract coverage for boundline init preview, confirmation, and destructive-update rules in tests/contract/init_cli_contract.rs"
Task: "T010 Add integration coverage for fresh-workspace bootstrap and safe rerun behavior in tests/integration/init_bootstrap_flow.rs"
Task: "T011 Add unit coverage for template generation and runtime capability detection in tests/unit/init_templates.rs and tests/unit/runtime_capability.rs"

# Then split command and scaffolding work:
Task: "T012 Implement boundline init preview and apply flow in src/cli/init.rs, src/cli.rs, and src/fixture.rs"
Task: "T014 Scaffold and refresh repository-local assistant setup selections during init in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, assistant/copilot/prompts/, and assistant/gemini/"
```

## Parallel Example: User Story 2

```bash
# Validate config precedence together:
Task: "T016 Add contract coverage for boundline config show, set, and unset in tests/contract/config_cli_contract.rs and tests/contract/routing_resolution_contract.rs"
Task: "T017 Add integration coverage for global-plus-workspace precedence resolution in tests/integration/config_precedence_flow.rs"
Task: "T018 Add unit coverage for config persistence, precedence, and invalid route validation in tests/unit/config_store.rs and tests/unit/config_resolution.rs"

# Then split persistence and resolution work:
Task: "T019 Implement global and workspace config load/save/unset behavior in src/adapters/config_store.rs and src/domain/configuration.rs"
Task: "T021 Resolve effective routing precedence and source attribution in src/domain/configuration.rs, src/fixture.rs, and src/domain/trace.rs"
```

## Parallel Example: User Story 3

```bash
# Validate differentiated review routing together:
Task: "T023 Add contract coverage for reviewer-role and adjudicator routing in tests/contract/routing_resolution_contract.rs and tests/contract/governance_session_contract.rs"
Task: "T024 Add integration coverage for review-role routing and assistant setup reuse in tests/integration/review_routing_flow.rs and tests/integration/cli_trace_inspection.rs"
Task: "T025 Add unit coverage for review-role route validation and effective reviewer overrides in tests/unit/config_resolution.rs and tests/unit/governance_policy.rs"

# Then split review-domain and assistant-surface work:
Task: "T026 Extend review-domain models for reviewer-role and adjudicator routing in src/domain/review.rs and src/domain/configuration.rs"
Task: "T029 Keep assistant-facing command packs aligned with routed review and setup behavior in assistant/README.md, assistant/claude/commands/, assistant/codex/commands/, assistant/copilot/prompts/, and assistant/gemini/"
```

## Implementation Strategy

### Full Feature Delivery

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete User Story 1 so fresh-workspace init is usable.
4. Complete User Story 2 so global/workspace precedence and CLI mutation are usable.
5. Complete User Story 3 so review-role differentiation and assistant setup are feature-complete.
6. Finish with docs, release metadata, coverage, and repo-wide validation.

### Suggested Scope

- Deliver all three user stories in the same `0.11.0` slice.
- Do not stop at init-only behavior; the feature is not considered complete until routing precedence, CLI mutation, review-role differentiation, and docs all ship together.

## Notes

- All tasks follow the required checklist format: checkbox, task ID, optional `[P]`, required story label for story tasks, and exact file paths.
- The feature intentionally treats docs, assistant guidance, and release metadata as first-class shipping work because usability is the primary delivery value.