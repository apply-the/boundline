# Tasks: Large Codebase Context Substrate

**Input**: Design documents from `/specs/070-large-codebase-context-substrate/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`,
`contracts/large-codebase-context-runtime-contract.md`, `quickstart.md`

**Tests**: Test tasks are required. Add or refine focused regressions first,
confirm the relevant assertion fails before changing implementation, then close
the regression with the smallest coherent runtime change.

**Organization**: Tasks are grouped by user story so the large-codebase context
substrate, inspectable runtime projection, and derived-cache boundary can be
implemented and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it targets different files and has no
  dependency on incomplete work
- **[Story]**: Maps a task to a user story for traceability
- Every task includes repository-relative file paths

## Phase 1: Setup

**Purpose**: Lock the release boundary, reconfirm the provider-catalog
no-change result and Canon compatibility posture, and establish failing
release-surface and assistant regressions before runtime edits.

- [x] T001 Reconfirm the 2026-06-05 provider-catalog no-change audit, the local-only substrate boundary, and the Canon `0.67.0` compatibility assumption in `specs/070-large-codebase-context-substrate/research.md`; if the slice still depends on provider-owned retrieval or Canon-owned schema changes, stop implementation and record the blocking gap before any runtime edits
- [x] T002 [P] Add or refine release-surface and compatibility regressions for Boundline `0.71.0` and Canon `0.67.0` in `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, `tests/contract/distribution_release_surface_contract.rs`, and `tests/assistant_plugin_packages.rs`
- [x] T003 [P] Add or refine assistant context-substrate contract regressions in `tests/contract/assistant_command_definition_contract.rs` and `tests/contract/assistant_session_continuity_contract.rs`

---

## Phase 2: Foundational

**Purpose**: Make the missing substrate behavior fail in the smallest
high-signal places before user-story implementation work begins.

**Critical**: Complete this phase before user-story implementation.

- [x] T004 [P] Add focused failing domain regressions for fidelity-tier classification, inclusion-mode persistence, archive exclusion, omission finding validation, and patch-safe edit attempt state validation in `tests/unit/context_intelligence_model.rs` and `tests/unit/goal_plan_model.rs`
- [x] T005 [P] Add focused failing runtime regressions for critical-context blocking, unsafe oversized full-read refusal, search-before-read ordering, and patch-safe anchor-drift rejection in `tests/unit/session_cli_runtime.rs`, `tests/contract/planning_gate_pipeline_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`
- [x] T006 [P] Add focused failing projection regressions for repository-map state, digest-backed compaction, omission explanations, archived-context inspect-only visibility, ranking rationale, and stale-cache rendering in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, and `tests/integration/context_intelligence_semantic_inspect.rs`

**Checkpoint**: The feature now fails for the required large-codebase context
contract while preserving backward-compatibility expectations.

---

## Phase 3: User Story 1 - Protect Critical Context In Large Repositories (Priority: P1) 🎯 MVP

**Goal**: Prevent planning or execution from continuing when large-repository
context is unsafe, silently degraded, or missing at required fidelity.

**Independent Test**: In an isolated temporary workspace, evaluate one safe
planning context, one oversized-artifact request, and one critical-context
omission scenario and confirm that only the safe context can proceed.

### Implementation

- [x] T007 [US1] Add typed fidelity-tier, inclusion-mode, omission-finding, and critical-context admission models plus helper constants in `src/domain/context_intelligence.rs` and `src/domain/goal_plan.rs`
- [x] T008 [US1] Implement critical-context classification, archive exclusion, and required-fidelity validation in `src/orchestrator/goal_planner.rs` and `src/domain/goal_plan.rs`
- [x] T009 [US1] Implement search-before-read discovery, hybrid local ranking, oversized full-read refusal, and repository-map-assisted narrowing in `src/orchestrator/context_intelligence.rs` and `src/domain/project_index.rs`
- [x] T010 [US1] Wire blocked-versus-credible context admission ordering and runtime stop semantics in `src/orchestrator/session_runtime_planning_context.rs`, `src/orchestrator/session_runtime.rs`, and `src/cli/session.rs`
- [x] T011 [US1] Add patch-safe large-file edit guards with anchored scope, drift detection, and post-apply verification state in `src/domain/context_intelligence.rs` and `src/orchestrator/context_intelligence.rs`
- [x] T012 [US1] Run the focused US1 regression set in `tests/unit/context_intelligence_model.rs`, `tests/unit/goal_plan_model.rs`, `tests/unit/session_cli_runtime.rs`, `tests/contract/planning_gate_pipeline_contract.rs`, and `tests/integration/host_session_runtime_flow.rs`, including patch-safe anchor-drift and oversized-read refusal scenarios

**Checkpoint**: Unsafe large reads and missing critical context can no longer
silently reach planning or execution.

---

## Phase 4: User Story 2 - Explain Context Inclusion And Omission (Priority: P2)

**Goal**: Expose what Boundline selected, compacted, downgraded, or omitted,
with enough runtime-visible detail for operators to understand the pack without
opening raw artifacts.

**Independent Test**: Build a mixed context pack in an isolated fixture and
confirm that `status`, `inspect`, and orchestration surfaces show the same
included and omitted entries, tiers, modes, reasons, and digest-backed source
resolution data.

### Implementation

- [x] T013 [US2] Add additive session-facing context-pack projection fields, omission findings, and repository-map state to `src/domain/session.rs` and `src/domain/goal_plan.rs`
- [x] T014 [US2] Implement digest-backed artifact references, bounded summaries, and source-resolution metadata in `src/domain/context_intelligence.rs` and `src/orchestrator/context_intelligence.rs`
- [x] T015 [US2] Wire context-pack projection, machine-readable inspect output, and explicit archive-or-inspect-only retrieval visibility in `src/cli/inspect/projections.rs`, `src/cli/output_context.rs`, and `src/cli/output_runtime.rs`
- [x] T016 [US2] Implement human-readable rendering for selected entries, omitted entries, fidelity tiers, inclusion modes, ranking rationale, and compaction explanations in `src/cli/output_session_status.rs` and `src/cli/output_orchestrate.rs`
- [x] T017 [US2] Persist and trace context-substrate state transitions, repository-map freshness, and compaction decisions with reproducible session context in `src/orchestrator/session_runtime.rs` and `src/domain/context_intelligence.rs`
- [x] T018 [US2] Run the focused US2 regression set in `tests/unit/cli_output.rs`, `tests/contract/host_command_output_contract.rs`, `tests/integration/context_intelligence_semantic_inspect.rs`, and `tests/integration/context_intelligence_semantic_flow.rs`, including ranking-rationale and archive-inspect-only scenarios

**Checkpoint**: Operators can inspect why context was included, compacted, or
omitted from the standard runtime surfaces.

---

## Phase 5: User Story 3 - Keep Derived Cache Separate From Memory (Priority: P3)

**Goal**: Reuse local derived snapshot-cache state only when it is fresh,
invalidate it before unsafe reuse, and keep the cache explicitly separate from
reviewed memory or authoritative planning truth.

**Independent Test**: In an isolated fixture, create a reusable snapshot,
trigger freshness events and tracked-cache faults, and confirm that stale or
tracked cache state produces diagnostics and cannot silently influence a new
planning context.

### Implementation

- [x] T019 [US3] Add typed snapshot-cache entry, freshness-event, and authority-boundary models in `src/domain/context_intelligence.rs`, `src/domain/session.rs`, and `src/domain/project_memory.rs`
- [x] T020 [US3] Implement snapshot-cache freshness detection for branch, merge, rebase, config, schema, adapter, and Canon packet changes in `src/orchestrator/context_intelligence.rs` and `src/orchestrator/session_runtime_planning_context.rs`
- [x] T021 [US3] Implement tracked-cache and stale-cache diagnostics plus repair guidance in `src/cli/diagnostics.rs`, `src/cli/output_session_status.rs`, and `src/orchestrator/context_intelligence.rs`
- [x] T022 [US3] Enforce the cache-is-not-memory boundary in runtime projection, project-memory integration, and reuse policy in `src/domain/project_memory.rs`, `src/orchestrator/goal_planner.rs`, and `src/orchestrator/session_runtime.rs`
- [x] T023 [US3] Run the focused US3 regression set in `tests/unit/context_intelligence_model.rs`, `tests/contract/host_command_output_contract.rs`, `tests/integration/context_intelligence_semantic_flow.rs`, and `tests/integration/host_session_runtime_flow.rs`

**Checkpoint**: Fresh snapshot-cache reuse is explicit, stale cache is blocked
or downgraded before reuse, and the cache cannot masquerade as memory.

---

## Phase 6: Release, Documentation, and Quality Closure

**Purpose**: Close versioning, docs, roadmap, assistant assets, and quality
gates for release `0.71.0`.

- [x] T024 [P] Refresh the provider-catalog audit result and explicit no-change rationale in `assistant/catalog/model-catalog.toml` and `specs/070-large-codebase-context-substrate/research.md`
- [x] T025 [P] Document the large-codebase context substrate, critical-context blocking, digest-backed compaction, patch-safe editing, and stale-cache behavior in `README.md`, `docs/runtime/plan.md`, `docs/runtime/status.md`, `docs/runtime/inspect.md`, `docs/runtime/run.md`, `docs/architecture/context-intelligence.md`, and `docs/architecture/runtime-model.md`
- [x] T026 [P] Update operator and architecture guidance for repository-map navigation, derived-cache boundaries, and local retrieval expectations in `tech-docs/architecture.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `docs/reference/cli.md`, `docs/reference/file-layout.md`, and `docs/guide/core-concepts.md`
- [x] T027 [P] Align assistant plan, run, status, and inspect assets with the final context-substrate runtime contract in `assistant/antigravity/commands/boundline-plan.md`, `assistant/antigravity/commands/boundline-run.md`, `assistant/antigravity/commands/boundline-status.md`, `assistant/antigravity/commands/boundline-inspect.md`, `assistant/claude/commands/boundline-plan.md`, `assistant/claude/commands/boundline-run.md`, `assistant/claude/commands/boundline-status.md`, `assistant/claude/commands/boundline-inspect.md`, `assistant/codex/commands/boundline-plan.md`, `assistant/codex/commands/boundline-run.md`, `assistant/codex/commands/boundline-status.md`, `assistant/codex/commands/boundline-inspect.md`, `assistant/copilot/prompts/boundline-plan.prompt.md`, `assistant/copilot/prompts/boundline-run.prompt.md`, `assistant/copilot/prompts/boundline-status.prompt.md`, and `assistant/copilot/prompts/boundline-inspect.prompt.md`
- [x] T028 [P] Bump release metadata to `0.71.0` and keep Canon `0.67.0` compatibility wording aligned in `Cargo.toml`, `Cargo.lock`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `assistant/plugin-metadata.json`, `assistant/global/manifest.json`, `src/cli/init.rs`, and `tests/contract/canon_reasoning_posture_contract.rs`
- [x] T029 [P] Add the `0.71.0` WinGet release manifests under `distribution/winget/manifests/a/ApplyThe/Boundline/0.71.0/`
- [x] T030 [P] Record the delivered roadmap slice and release summary in `CHANGELOG.md`, `docs/roadmap/index.md`, `roadmap/Next - forward-roadmap.md`, `roadmap/features/README.md`, and `roadmap/joint-roadmap-graph.md`
- [x] T031 Run `cargo fmt` and verify formatting with `cargo fmt --check`
- [x] T032 Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix every reported issue
- [x] T033 Run focused tests with `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration context_intelligence_semantic_flow::`, `cargo test --test integration context_intelligence_semantic_inspect::`, and `cargo test --test integration host_session_runtime_flow::`
- [x] T034 Run assistant and release-surface regressions in `tests/assistant_plugin_packages.rs`, `tests/contract/assistant_command_definition_contract.rs`, `tests/contract/assistant_session_continuity_contract.rs`, `tests/contract/canon_reasoning_posture_contract.rs`, `tests/contract/distribution_metadata_contract.rs`, and `tests/contract/distribution_release_surface_contract.rs`
- [x] T035 Run the full regression suite with `cargo test` and resolve any failures
- [x] T036 Generate `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`
- [x] T037 Build an explicit repository-relative implementation-file list, run `scripts/common/coverage/intersect_patch_coverage.py` against every touched Rust implementation file, and add tests until changed-file coverage is at least 95 percent
- [x] T038 Validate the isolated scenarios in `specs/070-large-codebase-context-substrate/quickstart.md`, including the under-30-second operator explanation check and the maintained-fixture context-selection timing check, without running Boundline CLI commands against the repository root

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup and blocks all story work.
- **User Story 1 (Phase 3)**: Depends on Foundational and is the MVP.
- **User Story 2 (Phase 4)**: Depends on the typed US1 substrate contract so
  projections expose the final decision shape.
- **User Story 3 (Phase 5)**: Depends on the typed substrate and repository-map
  projection from US1 and may proceed after US1 stabilizes.
- **Release and Quality Closure (Phase 6)**: Depends on all selected stories.

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational and has no dependency on
  later stories.
- **User Story 2 (P2)**: Can start after US1 stabilizes because it projects the
  final selection and omission contract.
- **User Story 3 (P3)**: Can start after US1 stabilizes because freshness and
  cache boundaries depend on the substrate data model already being in place.

### Parallel Opportunities

- T002 and T003 can run in parallel.
- T004, T005, and T006 can run in parallel.
- T013 and T014 can run in parallel after US1 core models stabilize.
- T025 through T030 can run in parallel after runtime behavior stabilizes.

---

## Parallel Example: User Story 1

```bash
# Launch the core failing substrate regressions together:
Task: "Add focused failing domain regressions in tests/unit/context_intelligence_model.rs and tests/unit/goal_plan_model.rs"
Task: "Add focused failing runtime regressions in tests/unit/session_cli_runtime.rs, tests/contract/planning_gate_pipeline_contract.rs, and tests/integration/host_session_runtime_flow.rs"
Task: "Add focused failing projection regressions in tests/unit/cli_output.rs, tests/contract/host_command_output_contract.rs, and tests/integration/context_intelligence_semantic_inspect.rs"
```

## Parallel Example: User Story 2

```bash
# Launch projection and compaction work in parallel after the substrate contract lands:
Task: "Implement digest-backed artifact references in src/domain/context_intelligence.rs and src/orchestrator/context_intelligence.rs"
Task: "Wire context-pack projection in src/cli/inspect/projections.rs, src/cli/output_context.rs, and src/cli/output_runtime.rs"
Task: "Implement human-readable rendering in src/cli/output_session_status.rs and src/cli/output_orchestrate.rs"
```

## Parallel Example: User Story 3

```bash
# Launch freshness and diagnostics work in parallel once the cache model exists:
Task: "Implement snapshot-cache freshness detection in src/orchestrator/context_intelligence.rs and src/orchestrator/session_runtime_planning_context.rs"
Task: "Implement tracked-cache and stale-cache diagnostics in src/cli/diagnostics.rs and src/cli/output_session_status.rs"
```

---

## Implementation Strategy

### MVP First

1. Complete Setup and Foundational regressions.
2. Complete US1 context-safety behavior.
3. Validate one safe context, one unsafe full-read path, and one blocked
   critical-context omission in isolated fixtures.
4. Proceed to inspectability and cache-boundary work only after the safety
   boundary is stable.

### Incremental Delivery

1. Finish Setup + Foundational so the missing contract is failing in focused
   tests.
2. Add US1 and validate context admission and patch-safe behavior.
3. Add US2 and validate operator-facing inclusion, omission, and compaction
   projections.
4. Add US3 and validate freshness, derived-cache reuse, and diagnostics.
5. Close docs, assistant assets, release metadata, and coverage only after the
   runtime contract is stable.

### Quality Rule

Do not treat formatting, clippy, tests, docs, assistant assets, release
metadata, or changed-file coverage as deferred cleanup. The feature is complete
only when `cargo fmt --check`, strict clippy, the regression suite, and at
least 95 percent changed-file coverage pass while the release and Canon
compatibility surfaces remain aligned.

---

## Notes

- `[P]` tasks target different files and can be executed in parallel.
- `[US1]`, `[US2]`, and `[US3]` labels preserve traceability from spec to
  implementation.
- Every user story remains independently testable once its phase completes.
- The persistent snapshot cache must remain explicitly separate from memory in
  implementation, projection, and documentation.
