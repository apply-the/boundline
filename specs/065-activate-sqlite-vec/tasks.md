# Tasks: Activate SQLite Vec And DB Merge Strategy

**Input**: Design documents from `specs/065-activate-sqlite-vec/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`

**Tests**: Validation is mandatory because this slice changes advanced-context runtime behavior, derived-index lifecycle semantics, Git hygiene behavior, and CLI-visible operator output. Include focused unit, contract, integration, compile, and quickstart validation tasks for each user story. Every implementation pass must also carry forward the provider-catalog delta audit against `assistant/catalog/model-catalog.toml`.

**Organization**: Tasks are grouped by user story so each slice can deliver independently testable value.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (`US1`, `US2`, `US3`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Carry forward the required catalog audit and prepare the SQLite-vec lifecycle surface.

- [X] T001 Refresh the provider-doc audit and apply the recorded Anthropic catalog delta in `assistant/catalog/model-catalog.toml` and `specs/065-activate-sqlite-vec/research.md`
- [X] T002 Add trusted SQLite extension and `sqlite-vec` dependency scaffolding in `Cargo.toml` and `crates/boundline-adapters/Cargo.toml`
- [X] T003 [P] Create index lifecycle contract and integration test scaffolding in `tests/contract/context_intelligence_index_lifecycle_contract.rs`, `tests/contract/context_intelligence_index_doctor_contract.rs`, `tests/integration/context_intelligence_index_refresh.rs`, `tests/integration/context_intelligence_index_rebuild.rs`, and `tests/integration/context_intelligence_index_doctor.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core manifest, lifecycle, and hygiene primitives that MUST exist before user-story work can complete.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T004 Extend typed derived-index lifecycle, manifest, chunk, vector, and health-state models in `src/domain/context_intelligence.rs`
- [X] T005 Implement shared manifest persistence, compatibility detection, and vector-capability helpers in `src/orchestrator/context_intelligence.rs`
- [X] T006 [P] Extend derived-index configuration, ignore management, and init preview/report primitives in `src/domain/configuration.rs`, `src/domain/workspace_hygiene.rs`, `src/cli/init.rs`, `src/cli/init/preview.rs`, and `src/cli/init/report.rs`
- [X] T007 [P] Add foundational unit coverage for lifecycle states and projection invariants in `tests/unit/context_intelligence_state.rs` and `tests/unit/context_intelligence_projection.rs`

**Checkpoint**: Foundation ready. One retrieval database, one manifest contract, and one hygiene path are stable enough for story work.

---

## Phase 3: User Story 1 - Surface Semantic Evidence Reliably (Priority: P1) 🎯 MVP

**Goal**: Activate SQL-side vector retrieval through `sqlite-vec` without weakening V1 authority order, source-level collapse, or explicit fallback behavior.

**Independent Test**: In a temporary workspace with `sqlite-vec` available, refresh the derived index and run a planning or inspection flow where the best evidence has weak lexical overlap; verify the runtime either returns semantically relevant evidence through the vector path or reports an explicit fallback reason without silent behavior changes.

### Tests for User Story 1

- [X] T008 [P] [US1] Extend semantic projection contract coverage for real vector-engine selection and fallback fields in `tests/contract/context_intelligence_semantic_projection_contract.rs` and `tests/contract/context_intelligence_semantic_inspect_contract.rs`
- [X] T009 [P] [US1] Extend integration coverage for ready vector-query execution and source-level collapse in `tests/integration/context_intelligence_semantic_flow.rs` and `tests/integration/context_intelligence_semantic_inspect.rs`
- [X] T010 [P] [US1] Extend integration coverage for unsupported, unavailable, and corrupt vector fallback paths in `tests/integration/context_intelligence_semantic_fallback.rs`

### Implementation for User Story 1

- [X] T011 [US1] Implement trusted `sqlite-vec` extension loading, `vec0` table creation, and dual-write refresh plumbing in `src/orchestrator/context_intelligence.rs`
- [X] T012 [US1] Replace Rust-side semantic full scans with SQL-side nearest-neighbor selection while preserving authority order and source-level collapse invariants in `src/orchestrator/context_intelligence.rs` and `src/domain/context_intelligence.rs`
- [X] T013 [US1] Persist and render `semantic_engine`, vector candidate counts, capability state, and fallback reasons in `src/domain/context_intelligence.rs`, `src/cli/output_context.rs`, and `src/cli/inspect.rs`
- [X] T014 [US1] Record vector-query, fallback, and degradation trace details in `src/domain/trace.rs`, `src/orchestrator/context_intelligence.rs`, and `src/cli/output.rs`

**Checkpoint**: User Story 1 should now deliver real vector retrieval as an additive capability while keeping V1 fallback explicit and inspectable.

---

## Phase 4: User Story 2 - Maintain The Derived Index Safely (Priority: P1)

**Goal**: Keep the unified retrieval database fresh, bounded, and recoverable through explicit lifecycle commands, manifest metadata, optional stale-marking automation, and a clear policy that treats checkout, merge, and rewrite as freshness events instead of mergeable DB state.

**Independent Test**: In a temporary workspace, build the index, modify only a subset of sources, simulate a branch-switch or merge freshness event, run `boundline index refresh --workspace <workspace>`, and verify that only changed sources update, disappeared sources are removed, fetch remains a no-op, commit hooks remain disabled unless explicitly configured, rebuild-required conditions are explicit, and `status` reports the manifest-backed stale or compatible lifecycle state accurately.

### Tests for User Story 2

- [X] T015 [P] [US2] Add contract coverage for `boundline index status`, `refresh`, `rebuild`, and `clean` JSON responses in `tests/contract/context_intelligence_index_lifecycle_contract.rs`
- [X] T016 [P] [US2] Add integration coverage for incremental refresh, branch-switch stale transitions, disappeared-source removal, and rebuild-required transitions in `tests/integration/context_intelligence_index_refresh.rs` and `tests/integration/context_intelligence_index_rebuild.rs`
- [X] T017 [P] [US2] Add unit coverage for manifest stale detection, stable chunk IDs, and rebuild triggers in `tests/unit/context_intelligence_state.rs`

### Implementation for User Story 2

- [X] T018 [US2] Implement manifest read/write, schema fingerprints, stale-reason tracking, and refresh outcome tracking in `src/orchestrator/context_intelligence.rs` and `src/domain/context_intelligence.rs`
- [X] T019 [US2] Convert semantic refresh from table-wide deletion to chunk-level upsert/delete with synchronized vector rows in `src/orchestrator/context_intelligence.rs`
- [X] T020 [US2] Add `boundline index status|refresh|rebuild|clean` command parsing and handlers plus explicit stale-reason output in `src/cli.rs`, `src/cli/index.rs`, `src/cli/output.rs`, and `src/cli/output_host.rs`
- [X] T021 [P] [US2] Extend derived-index ignore rules and tracked-file hygiene reporting for the derived DB, manifest sidecars, and SQLite WAL or SHM files in `src/domain/workspace_hygiene.rs`, `src/cli/init.rs`, `src/cli/init/preview.rs`, and `src/cli/init/report.rs`
- [X] T022 [US2] Implement optional stale-marking hook installation and hook-action configuration for checkout, merge, pull-with-merge, and post-rewrite while keeping fetch as a no-op, commit hooks disabled by default, and rebuilds manual in `src/domain/configuration.rs`, `src/cli/init.rs`, and `src/cli/diagnostics.rs`

**Checkpoint**: User Story 2 should make index refresh, rebuild, cleanup, and stale-marking behavior explicit and bounded for operators.

---

## Phase 5: User Story 3 - Diagnose Stale, Corrupt, Or Tracked Index State (Priority: P2)

**Goal**: Give operators clear diagnosis and recovery guidance when the derived index is stale, corrupt, incompatible, or accidentally tracked by Git.

**Independent Test**: In a temporary workspace, intentionally track the retrieval database or corrupt the manifest/vector schema, run `boundline index doctor --workspace <workspace>` plus `status` and `inspect`, and verify that the CLI reports the failure mode, degraded runtime state, and concrete recovery steps without silent fallback.

### Tests for User Story 3

- [X] T023 [P] [US3] Add contract coverage for `boundline index doctor` and degraded `status` or `inspect` fields in `tests/contract/context_intelligence_index_doctor_contract.rs` and `tests/contract/context_intelligence_semantic_inspect_contract.rs`
- [X] T024 [P] [US3] Add integration coverage for tracked database files, corrupt manifest state, and empty-vector degradation in `tests/integration/context_intelligence_index_doctor.rs` and `tests/integration/context_intelligence_semantic_inspect.rs`
- [X] T025 [P] [US3] Add unit coverage for doctor check results and operator recovery guidance in `tests/unit/context_intelligence_projection.rs` and `tests/unit/context_intelligence_state.rs`

### Implementation for User Story 3

- [X] T026 [US3] Implement `boundline index doctor` checks for Git tracking, managed ignore blocks, derived DB or manifest sidecars, SQLite WAL or SHM files, manifest consistency, and vector schema validity in `src/cli/index.rs` and `src/orchestrator/context_intelligence.rs`
- [X] T027 [US3] Render stale, incompatible, degraded, and corrupt recovery guidance across `status` and `inspect` in `src/cli/output_context.rs`, `src/cli/inspect.rs`, and `src/cli/session.rs`
- [X] T028 [US3] Extend probe and diagnostics surfaces for derived-index health, hook status, and semantic degradation evidence in `src/cli/probe.rs`, `src/cli/diagnostics.rs`, and `src/cli/output.rs`

**Checkpoint**: User Story 3 should make stale, corrupt, and tracked-index conditions obvious before they become silent retrieval failures.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Finish documentation, operator walkthroughs, and validation across all stories.

- [x] T029 Bump the Boundline workspace and distribution version from `0.64.0` to `0.65.0` in `Cargo.toml`, `Cargo.lock`, `assistant/plugin-metadata.json`, `distribution/channel-metadata.toml`, `distribution/homebrew/Formula/boundline.rb`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.65.0/ApplyThe.Boundline.yaml`, `distribution/winget/manifests/a/ApplyThe/Boundline/0.65.0/ApplyThe.Boundline.installer.yaml`, and `distribution/winget/manifests/a/ApplyThe/Boundline/0.65.0/ApplyThe.Boundline.locale.en-US.yaml`
- [x] T030 [P] Update release-facing changelog, roadmap, docs, and wiki pages for the sqlite-vec lifecycle slice in `CHANGELOG.md`, `ROADMAP.md`, `README.md`, `tech-docs/configuration.md`, `tech-docs/getting-started.md`, `tech-docs/architecture.md`, `tech-docs/project-memory-and-evidence-structure.md`, `tech-docs/release-checklist.md`, `../boundline.wiki/Home.md`, `../boundline.wiki/Getting-Started.md`, `../boundline.wiki/Quick-Start.md`, `../boundline.wiki/Troubleshooting.md`, `../boundline.wiki/Architecture-And-Decisions.md`, `../boundline.wiki/Configuration-Reference.md`, `../boundline.wiki/Project-Memory-Structure.md`, and `../boundline.wiki/Traces-And-Inspectability.md`
- [x] T031 Run the temporary-workspace quickstart walkthrough and capture any follow-up notes in `specs/065-activate-sqlite-vec/quickstart.md` and `specs/065-activate-sqlite-vec/checklists/requirements.md`
- [x] T032 Refresh `lcov.info` with `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, use `scripts/common/coverage/intersect_patch_coverage.py` to confirm every modified or created Rust file stays at or above 95% coverage, run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix all findings, then run `cargo fmt --all` plus `cargo test --no-run --all-targets` and record the validation results in `specs/065-activate-sqlite-vec/checklists/requirements.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: Depends on Setup completion and blocks all user stories.
- **User Stories (Phases 3-5)**: Depend on Foundational completion.
- **Polish (Phase 6)**: Depends on the desired user stories being complete.

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Foundational and is the MVP.
- **User Story 2 (P1)**: Starts after Foundational and can progress in parallel with US1 once the shared manifest and capability helpers are stable.
- **User Story 3 (P2)**: Starts after Foundational and should land after the shared `boundline index` surface from US2 plus the degraded-runtime fields from US1 are available.

### Within Each User Story

- Validation tasks MUST fail before implementation when the behavior is executable.
- Shared typed state before orchestration changes.
- Orchestration changes before CLI rendering and operator guidance.
- Index safety and hygiene behavior before optional hook automation.
- Story completion requires explicit degraded or fallback behavior wherever the spec calls for it.

### Parallel Opportunities

- Setup tasks marked `[P]` can run in parallel.
- Foundational tasks marked `[P]` can run in parallel.
- Validation tasks within a user story marked `[P]` can run in parallel.
- After Foundational completion, US1 retrieval work and US2 lifecycle CLI work can proceed in parallel if ownership of `src/orchestrator/context_intelligence.rs` is coordinated.

---

## Parallel Example: User Story 2

```bash
# Launch lifecycle validation coverage together:
Task: "Add contract coverage for boundline index status, refresh, rebuild, and clean JSON responses in tests/contract/context_intelligence_index_lifecycle_contract.rs"
Task: "Add integration coverage for incremental refresh, disappeared-source removal, and rebuild-required transitions in tests/integration/context_intelligence_index_refresh.rs and tests/integration/context_intelligence_index_rebuild.rs"
Task: "Add unit coverage for manifest stale detection, stable chunk IDs, and rebuild triggers in tests/unit/context_intelligence_state.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational.
3. Complete Phase 3: User Story 1.
4. Stop and validate real vector retrieval plus explicit fallback before proceeding.

### Incremental Delivery

1. Deliver US1 for SQL-side semantic retrieval over the V1 baseline.
2. Add US2 for manifest-backed refresh, rebuild, clean, and stale-marking flows.
3. Add US3 for diagnosis and recovery guidance around stale, corrupt, or tracked index state.
4. Finish with documentation, quickstart, and validation closeout.

### Parallel Team Strategy

1. One contributor completes the shared manifest, capability, and typed-state foundation.
2. A second contributor prepares lifecycle contract and integration coverage.
3. Once the foundation stabilizes, story work can split between US1 retrieval semantics and US2 lifecycle or hygiene surfaces before converging on US3 diagnostics.

---

## Notes

- `[P]` tasks touch different files or independent new files and can be worked in parallel safely.
- `[US#]` labels preserve traceability back to the feature spec.
- Run all CLI and quickstart validation in temporary workspaces, never against the repository root.
- Keep the single retrieval database plus manifest design intact; do not reintroduce a second semantic store in this slice.