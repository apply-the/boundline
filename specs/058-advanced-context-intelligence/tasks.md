# Tasks: Advanced Context Intelligence

**Input**: Design documents from `/specs/058-advanced-context-intelligence/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`

**Tests**: Validation is mandatory because this slice changes bounded context
assembly, Canon-consumer compatibility, task and trace persistence, CLI-visible
runtime summaries, and retrieval policy behavior.

## Phase 1: Shared V1 Baseline

- [x] T001 Refresh the S5 research and no-change provider audit in `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/research.md`
- [x] T002 Create and export the shared advanced-context domain surface in `/Users/rt/workspace/apply-the/boundline/src/domain/context_intelligence.rs`, `/Users/rt/workspace/apply-the/boundline/src/domain.rs`, and `/Users/rt/workspace/apply-the/boundline/crates/boundline-core/src/domain.rs`
- [x] T003 Create focused unit scaffolding for advanced-context state and projection behavior in `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_state.rs` and `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_projection.rs`

## Phase 2: Foundational Retrieval Plumbing

- [x] T004 Extend bounded planning and persisted context surfaces with advanced-context projection state in `/Users/rt/workspace/apply-the/boundline/src/domain/goal_plan.rs`, `/Users/rt/workspace/apply-the/boundline/src/domain/session.rs`, `/Users/rt/workspace/apply-the/boundline/src/domain/task_context.rs`, and `/Users/rt/workspace/apply-the/boundline/src/domain/trace.rs`
- [x] T005 Wire advanced-context projection persistence through runtime orchestration in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/session_runtime.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli/session.rs`, and `/Users/rt/workspace/apply-the/boundline/src/cli/inspect.rs`
- [x] T006 Implement the workspace-local SQLite + FTS5 retrieval baseline in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/context_intelligence.rs` and add the required SQLite dependency in `/Users/rt/workspace/apply-the/boundline/Cargo.toml` and `/Users/rt/workspace/apply-the/boundline/crates/boundline-adapters/Cargo.toml`
- [x] T007 Project selected evidence, relationships, and impact findings through CLI output in `/Users/rt/workspace/apply-the/boundline/src/cli/output.rs`

## Phase 3: User Story 1 - Local Retrieval Without Losing Authority

### Validation

- [x] T008 Add unit coverage for selected local evidence and bounded fallback behavior in `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_projection.rs`
- [x] T009 Add contract coverage for Canon-consumer compatibility and projection shape in `/Users/rt/workspace/apply-the/boundline/tests/contract/context_intelligence_consumer_contract.rs` and `/Users/rt/workspace/apply-the/boundline/tests/contract/context_intelligence_projection_contract.rs`
- [x] T010 Add end-to-end integration coverage for `plan`, `status`, and `inspect` advanced-context flow in `/Users/rt/workspace/apply-the/boundline/tests/integration/context_intelligence_flow.rs`

### Implementation

- [x] T011 Implement local retrieval-query planning, authority ordering, and structured fallback selection in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/context_intelligence.rs` and `/Users/rt/workspace/apply-the/boundline/src/orchestrator/goal_planner.rs`
- [x] T012 Persist and surface selected evidence and retrieval state in `/Users/rt/workspace/apply-the/boundline/src/domain/session.rs`, `/Users/rt/workspace/apply-the/boundline/src/domain/task_context.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli/session.rs`, and `/Users/rt/workspace/apply-the/boundline/src/cli/inspect.rs`

## Phase 4: User Story 2 - Explainable Impact Projection

### Validation

- [x] T013 Add unit coverage for relationship and impact projection in `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_projection.rs` and `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_state.rs`
- [x] T014 Add contract coverage for relationship and impact projection in `/Users/rt/workspace/apply-the/boundline/tests/contract/context_intelligence_projection_contract.rs`
- [x] T015 Add integration coverage for missing-test and evidence-gap journeys in `/Users/rt/workspace/apply-the/boundline/tests/integration/context_intelligence_impact_flow.rs`

### Implementation

- [x] T016 Implement relationship and impact-finding projection in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/context_intelligence.rs` and `/Users/rt/workspace/apply-the/boundline/src/domain/context_intelligence.rs`
- [x] T017 Persist and render relationship and impact output in `/Users/rt/workspace/apply-the/boundline/src/domain/session.rs`, `/Users/rt/workspace/apply-the/boundline/src/domain/trace.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli/output.rs`, and `/Users/rt/workspace/apply-the/boundline/src/cli/inspect.rs`

## Phase 5: User Story 3 - Optional, Bounded, Local-First Policy

### Validation

- [x] T018 Add unit coverage for disabled or local policy behavior in `/Users/rt/workspace/apply-the/boundline/tests/unit/context_intelligence_projection.rs` and `/Users/rt/workspace/apply-the/boundline/src/domain/configuration.rs`
- [x] T019 Add contract coverage for disabled or degraded projection behavior in `/Users/rt/workspace/apply-the/boundline/tests/contract/context_intelligence_projection_contract.rs`
- [x] T020 Add integration coverage for disabled-policy command behavior in `/Users/rt/workspace/apply-the/boundline/tests/integration/context_intelligence_remote_policy.rs`

### Implementation

- [x] T021 Add typed advanced-context retrieval policy to configuration precedence in `/Users/rt/workspace/apply-the/boundline/src/domain/configuration.rs`, `/Users/rt/workspace/apply-the/boundline/src/cli/config.rs`, and `/Users/rt/workspace/apply-the/boundline/src/orchestrator/goal_planner.rs`
- [x] T022 Enforce S5 V1 local-only retrieval policy and explicit disabled behavior in `/Users/rt/workspace/apply-the/boundline/src/orchestrator/context_intelligence.rs`
- [x] T023 Surface advanced-context policy in config inspection output in `/Users/rt/workspace/apply-the/boundline/src/cli/config.rs`

## Final Phase: Documentation, Validation, And Release Closeout

- [x] T024 Realign the feature spec, implementation plan, and feature-local contract mirrors to the S5 V1 local SQLite + FTS5 baseline in `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/spec.md`, `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/plan.md`, `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/contracts/advanced-context-intelligence-projection-contract.md`, `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/contracts/canon-semantic-retrieval-consumer-contract.md`, and `/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/tasks.md`
- [x] T025 Update operator and contributor docs for the advanced-context policy and retrieval surfaces in `/Users/rt/workspace/apply-the/boundline/docs/configuration.md`, `/Users/rt/workspace/apply-the/boundline/README.md`, `/Users/rt/workspace/apply-the/boundline/CHANGELOG.md`, `/Users/rt/workspace/apply-the/boundline/AGENTS.md`, and `/Users/rt/workspace/apply-the/boundline/assistant/README.md`
- [x] T026 Run focused validation plus `cargo test --no-run --all-targets` in `/Users/rt/workspace/apply-the/boundline`
- [x] T027 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and coverage refresh for all modified Boundline files in `/Users/rt/workspace/apply-the/boundline`

## Notes

- S5 V1 is the local SQLite + FTS5 retrieval baseline.
- sqlite-vec, embeddings, graph projection, and remote providers are deferred to S5.v2 or later.
- Canon remains an optional semantic producer input only; this feature does not require a new Canon runtime role.