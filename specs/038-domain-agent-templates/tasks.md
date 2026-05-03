# Tasks: Domain Agent Templates

**Input**: Design documents from `/specs/038-domain-agent-templates/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Validation tasks are required for this feature because it changes configuration scope resolution, bounded planning gates, operator-facing summaries, and persisted trace semantics.

**Organization**: Tasks are grouped by user story so each slice remains independently testable while still delivering one complete macrofeature.

**Implementation note**: The delivered slice reuses existing module, contract, and integration suites plus existing read-side projections where that kept the feature smaller and more inspectable. Tasks are marked complete when the scoped behavior shipped and was validated, even when the exact landing file differed from the original plan.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this belongs to (e.g. [US1], [US2], [US3])
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Finalize the 038 feature pack and ensure test-harness entry points are ready before runtime changes.

- [x] T001 Confirm and keep synchronized `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/plan.md`, `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/research.md`, `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/data-model.md`, `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/contracts/`, and `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/quickstart.md`
- [x] T002 [P] Add or update top-level test harness references in `/Users/rt/workspace/boundline/tests/unit.rs`, `/Users/rt/workspace/boundline/tests/contract.rs`, and `/Users/rt/workspace/boundline/tests/integration.rs` for new 038 test modules

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the shared domain-template model, scoped resolution, and context-pack primitives used by all stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T003 Extend `/Users/rt/workspace/boundline/src/domain/configuration.rs` and create `/Users/rt/workspace/boundline/src/domain/domain_templates.rs` with the domain-family catalog, scoped template settings, external binding model, effective resolution helpers, and validation rules
- [x] T004 [P] Extend `/Users/rt/workspace/boundline/src/cli.rs`, `/Users/rt/workspace/boundline/src/cli/config.rs`, and `/Users/rt/workspace/boundline/src/cli/init.rs` with the request/command primitives needed to seed, mutate, and render domain-template settings across workspace, cluster, and global scopes
- [x] T005 [P] Extend `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`, `/Users/rt/workspace/boundline/src/domain/task_context.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs` so context packs can carry applied domain context, supporting-input status, and blocked-domain reasons
- [x] T006 [P] Add foundational coverage for domain-template validation and scoped resolution in `/Users/rt/workspace/boundline/tests/unit/domain_templates.rs` and `/Users/rt/workspace/boundline/tests/contract/domain_template_configuration_contract.rs`

**Checkpoint**: The domain-template catalog, scoped settings, and context-pack primitives exist and can support all user stories.

---

## Phase 3: User Story 1 - Apply The Right Domain Expert Per Task (Priority: P1) 🎯 MVP

**Goal**: Make `init`, `config show`, and `plan` choose and surface the right active domain family for the bounded task.

**Independent Test**: Initialize a mixed-stack repository with relevant domain families, plan a Rust-targeted task and a React-targeted task, and verify that the applied domain context changes to match the bounded target before execution.

### Tests for User Story 1

- [x] T007 [P] [US1] Add contract coverage for domain-template config rendering in `/Users/rt/workspace/boundline/tests/contract/domain_template_configuration_contract.rs`
- [x] T008 [P] [US1] Add integration coverage for workspace init and domain-family selection in `/Users/rt/workspace/boundline/tests/integration/domain_template_init_flow.rs` and `/Users/rt/workspace/boundline/tests/integration/domain_template_context_flow.rs`
- [x] T009 [P] [US1] Add unit coverage for file-target-to-domain selection in `/Users/rt/workspace/boundline/tests/unit/domain_templates.rs` and `/Users/rt/workspace/boundline/tests/unit/goal_planner_domain_context.rs`

### Implementation for User Story 1

- [x] T010 [US1] Extend `/Users/rt/workspace/boundline/src/cli/init.rs` and `/Users/rt/workspace/boundline/src/cli.rs` so `boundline init` can detect or accept active domain families and seed workspace-scoped domain settings
- [x] T011 [US1] Extend `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs` and `/Users/rt/workspace/boundline/src/domain/goal_plan.rs` so planning selects a credible applied domain context from task targets and blocks explicitly when no matching active family exists
- [x] T012 [US1] Extend `/Users/rt/workspace/boundline/src/cli/config.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs` so `config show`, `plan`, and `inspect` expose the active domain family, selected target, and winning guidance source

**Checkpoint**: Operators can initialize active domain families and see the correct domain context selected for the bounded task.

---

## Phase 4: User Story 2 - Apply Layered Template Inheritance (Priority: P2)

**Goal**: Layer built-in templates, shared standards, and workspace overrides with explicit precedence and post-init customization.

**Independent Test**: Configure one shared standards layer and one workspace override for the same family, then verify that effective config and later planning retain the built-in template while letting the workspace layer win on conflicts.

### Tests for User Story 2

- [x] T013 [P] [US2] Add integration coverage for shared-vs-workspace standards precedence in `/Users/rt/workspace/boundline/tests/integration/domain_template_standards_inheritance.rs`
- [x] T014 [P] [US2] Add unit coverage for precedence and source attribution in `/Users/rt/workspace/boundline/tests/unit/domain_templates.rs` and `/Users/rt/workspace/boundline/tests/unit/configuration_domain_resolution.rs`

### Implementation for User Story 2

- [x] T015 [US2] Extend `/Users/rt/workspace/boundline/src/domain/configuration.rs` and `/Users/rt/workspace/boundline/src/cli/config.rs` so operators can enable or disable domain families and update scoped standards after initialization
- [x] T016 [US2] Extend `/Users/rt/workspace/boundline/src/domain/domain_templates.rs`, `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, and `/Users/rt/workspace/boundline/src/cli/output.rs` so effective domain guidance resolves in explicit built-in → shared → workspace precedence order and remains inspectable
- [x] T017 [US2] Extend `/Users/rt/workspace/boundline/src/cli/session.rs` and `/Users/rt/workspace/boundline/src/cli/inspect.rs` so `status` and `inspect` keep the layered-domain story visible after later replans or task-target changes

**Checkpoint**: Shared defaults and local overrides coexist cleanly and remain visible on the normal operator path.

---

## Phase 5: User Story 3 - Keep Domain Coverage Bounded And Inspectable (Priority: P3)

**Goal**: Make unsupported or ambiguous domain selection stop explicitly and keep mixed-stack domain changes visible through the standard read-side surfaces.

**Independent Test**: Run a task that matches no active family or conflicting families and verify that planning stops explicitly, then run a mixed-stack replan and verify that the surfaced domain context changes.

### Tests for User Story 3

- [x] T018 [P] [US3] Add integration coverage for blocked-domain planning and mixed-stack replanning in `/Users/rt/workspace/boundline/tests/integration/domain_template_context_flow.rs`
- [x] T019 [P] [US3] Add contract coverage for domain-context projection on inspection surfaces in `/Users/rt/workspace/boundline/tests/contract/domain_template_context_contract.rs`

### Implementation for User Story 3

- [x] T020 [US3] Extend `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, `/Users/rt/workspace/boundline/src/orchestrator/decision_loop.rs`, and `/Users/rt/workspace/boundline/src/orchestrator/session_runtime.rs` so domain context can be recomputed from bounded targets and recorded when the current target changes
- [x] T021 [US3] Extend `/Users/rt/workspace/boundline/src/domain/task_context.rs`, `/Users/rt/workspace/boundline/src/domain/trace.rs`, `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/inspect.rs` so applied-domain context, blocked-domain reasons, and context credibility stay authoritative and inspectable

**Checkpoint**: Domain mismatches stop explicitly, and mixed-stack domain changes remain visible to operators.

---

## Phase 6: User Story 4 - Reuse Governed And External Context Inputs (Priority: P4)

**Goal**: Let Canon-governed artifacts and external context bindings augment the active domain context without taking ownership of template selection.

**Independent Test**: Bind optional and required external inputs for a domain family, run planning with and without those inputs available, and verify that Boundline surfaces used, skipped, unavailable, or stale status and blocks when a required input is missing.

### Tests for User Story 4

- [x] T022 [P] [US4] Add contract coverage for governed artifacts and external binding status in `/Users/rt/workspace/boundline/tests/contract/external_context_binding_contract.rs`
- [x] T023 [P] [US4] Add integration coverage for Canon-governed and external input reuse in `/Users/rt/workspace/boundline/tests/integration/domain_template_external_inputs.rs`
- [x] T024 [P] [US4] Add unit coverage for binding availability and required-input blocking in `/Users/rt/workspace/boundline/tests/unit/domain_templates.rs` and `/Users/rt/workspace/boundline/tests/unit/goal_planner_domain_context.rs`

### Implementation for User Story 4

- [x] T025 [US4] Extend `/Users/rt/workspace/boundline/src/domain/domain_templates.rs`, `/Users/rt/workspace/boundline/src/domain/configuration.rs`, and `/Users/rt/workspace/boundline/src/cli/config.rs` so operators can bind and unbind optional or required external context inputs per family and scope
- [x] T026 [US4] Extend `/Users/rt/workspace/boundline/src/orchestrator/goal_planner.rs`, `/Users/rt/workspace/boundline/src/domain/goal_plan.rs`, and `/Users/rt/workspace/boundline/src/domain/task_context.rs` so Canon-governed artifacts and external inputs augment the applied domain context with explicit status
- [x] T027 [US4] Extend `/Users/rt/workspace/boundline/src/cli/output.rs`, `/Users/rt/workspace/boundline/src/cli/session.rs`, and `/Users/rt/workspace/boundline/src/cli/inspect.rs` so `plan`, `run`, `status`, `next`, and `inspect` surface supporting-input status without implying those systems own template selection

**Checkpoint**: Supporting governed and external inputs enrich the bounded domain context while remaining subordinate and inspectable.

---

## Phase 7: Release & Cross-Cutting Concerns

**Purpose**: Close the feature as a release-aligned macrofeature with validation evidence.

- [x] T028 Bump crate version to `0.38.0` in `/Users/rt/workspace/boundline/Cargo.toml` and `/Users/rt/workspace/boundline/Cargo.lock`
- [x] T029 [P] Update impacted docs and release narrative in `/Users/rt/workspace/boundline/README.md`, `/Users/rt/workspace/boundline/docs/`, `/Users/rt/workspace/boundline/CONTRIBUTING.md`, `/Users/rt/workspace/boundline/CHANGELOG.md`, and `/Users/rt/workspace/boundline/AGENTS.md`
- [x] T030 [P] Update assistant guidance impacted by domain templates in `/Users/rt/workspace/boundline/assistant/README.md`, `/Users/rt/workspace/boundline/assistant/claude/commands/`, `/Users/rt/workspace/boundline/assistant/codex/commands/`, and `/Users/rt/workspace/boundline/assistant/copilot/prompts/`
- [x] T031 Update `/Users/rt/workspace/boundline/ROADMAP.md` to mark Spec 038 as delivered and remove it from the upcoming macrofeature line
- [x] T032 [P] Run formatting with `cargo fmt --all`
- [x] T033 [P] Run lint validation with `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] T034 Run compile-oriented and broader Rust validation with `cargo test --no-run --all-targets` and `cargo nextest run --workspace --all-features`
- [x] T035 Refresh line coverage with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` and confirm modified or new Rust files stay above 95%
- [x] T036 Mark completed tasks in `/Users/rt/workspace/boundline/specs/038-domain-agent-templates/tasks.md` and capture the final descriptive commit message in the implementation summary

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all stories.
- **User Story 1 (Phase 3)**: Depends on Foundational completion.
- **User Story 2 (Phase 4)**: Depends on User Story 1 because layered standards build on the scoped domain model and config surface.
- **User Story 3 (Phase 5)**: Depends on User Stories 1 and 2 because blocked-domain behavior and mixed-stack projection rely on the applied-domain context model.
- **User Story 4 (Phase 6)**: Depends on User Stories 1 through 3 because governed and external inputs augment the same applied-domain context.
- **Release (Phase 7)**: Depends on all prior phases.

### Parallel Opportunities

- T002 can run in parallel with T001 if new harness entries are needed.
- T004, T005, and T006 can run in parallel after T003 defines the shared domain model.
- Within each user story, tasks marked `[P]` can be developed in parallel before implementation tasks touching the same files.
- T029 and T030 can run in parallel once the CLI and read-side behavior are stable.

## Implementation Strategy

### MVP First

1. Finish Setup and Foundational work.
2. Finish User Story 1 and validate that `init`, `config show`, and `plan` surface a credible applied domain context.
3. Use that applied-domain model as the base for layered standards, blocked-domain behavior, and supporting inputs.

### Incremental Delivery

1. Add the shared domain-template catalog and scoped settings.
2. Make initialization and effective config projection domain-aware.
3. Make planning and inspection carry one applied-domain context and explicit blocked-domain behavior.
4. Add layered standards precedence and mixed-stack projection.
5. Add governed artifacts and external input bindings.
6. Close the release with version, docs, roadmap, assistant guidance, coverage, linting, and formatting.

## Notes

- This feature stays macro-level in value but bounded in implementation: it extends the existing config, planner, and inspection model instead of introducing a template marketplace or a new runtime.
- External context bindings may reference MCP-backed or repository-adjacent sources, but this slice treats them as bounded supporting inputs and status surfaces rather than an open-ended provider execution framework.
- The final summary must include a descriptive commit message for the completed feature.