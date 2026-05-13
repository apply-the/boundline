# Implementation Plan: Project Memory Delivery Integration

**Branch**: `050-project-memory-delivery-integration` | **Date**: 2026-05-13 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/050-project-memory-delivery-integration/spec.md`

## Summary

Add Boundline-owned consumer types and logic for reading Canon-promoted
project-memory and evidence surfaces, evaluating contract-version
compatibility, and surfacing Canon promotion state and lineage refs in
session-native delivery decisions. Boundline remains the delivery orchestrator
and does not redefine Canon promotion semantics. A new `ProjectMemoryContext`
snapshot feeds delivery-path, stage-planner, and assurance evaluation with
credible Canon output while treating pending or evidence-only outputs as
explicitly non-authoritative.

## Technical Context

**Language/Version**: Rust 1.95.0, Edition 2024
**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`,
`tracing`, `uuid`, `toml`
**Storage**: workspace-local `.boundline/session.json`,
`.boundline/traces/`, optional `.boundline/execution.json`; reads
Canon-promoted surfaces from `docs/project/`, `docs/evidence/`
**Testing**: `cargo test`, `cargo nextest run`, `cargo llvm-cov`
**Target Platform**: macOS/Linux developer workstations, Linux CI
**Project Type**: multi-crate Rust workspace (boundline-core, boundline-cli,
boundline-adapters)
**Execution Model**: sequential session-native loop; Canon output is read at
stage-planning time, never mutated by Boundline
**Observability Surface**: persisted execution trace records Canon refs,
promotion state, and compatibility outcome; session-native status/next/inspect
surfaces expose Canon context
**Performance Goals**: no measurable impact on CLI responsiveness
**Constraints**: Canon project-memory output is optional; Boundline MUST
continue delivery when it is absent. Boundline MUST NOT mutate Canon surfaces.
**Scale/Scope**: 6 promotion states, 3 update strategies, ~12 stable target
paths from the Canon contract

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **Delivery identity**: PASS. Consuming Canon-promoted project memory gives
  the stage planner credible context for selecting, skipping, or confirming
  stages, directly improving multi-step delivery decisions. (Spec §US1, §US2)
- **Delivery-first scope**: PASS. Work is prioritized as: (1) consumption
  types, (2) stage-planner integration, (3) assurance integration, (4)
  session-native surface integration, (5) compatibility checking, (6) docs and
  polish. (Plan §Implementation Phases)
- **Primary workflow**: PASS. The main operator path remains session-native
  (`start -> capture -> plan -> run -> status -> next -> inspect`). Canon refs
  appear inside `status`, `next`, and `inspect` when available. No
  compatibility path is needed. (Spec §US2)
- **Bounded execution**: PASS. Contract-version check is a single-step gate
  with two outcomes: compatible on the supported `0.1.x` line or unsupported
  (bounded stop). No loops or retries. (Spec §US3)
- **Stateful execution**: PASS. `ProjectMemoryContext` is written into session
  task context at stage-planning time and read by downstream status/next/inspect.
  (Plan §Data Model, Spec §FR-006)
- **Mutable planning**: PASS. Canon output may trigger replanning when stable
  project memory changes between stages; the stage planner already supports
  replanning. (Spec §US1 acceptance 3)
- **Sequential-first design**: PASS. Canon output is read once per
  stage-planning step; no parallel or background reads. (Plan §Implementation
  Phases)
- **Tool-agent symmetry**: PASS. Reading and evaluating Canon output is an
  explicit evaluation step in the stage-planner, not a hidden reasoning path.
  (Spec §FR-007)
- **Observability and explicit intelligence**: PASS. Trace records Canon refs,
  promotion state, compatibility outcome. Session-native surfaces expose these.
  No hidden heuristics. (Spec §FR-006, Plan §Technical Context)
- **Catalog currency**: PASS. No model catalog changes required; this slice
  consumes Canon file output, not provider APIs. No-change rationale: the
  bundled model catalog is unaffected.
- **Non-goals and external separation**: PASS. Boundline reads Canon output
  files but does not depend on Canon runtime behavior. Canon semantics are
  consumed, not redefined. No councils, voting, provider abstraction, or UI
  work. (Spec §Non-Goals, §Invariants)
- **Minimal slice**: PASS. The smallest independently valuable capability is
  reading stable Canon project-memory into the stage planner and surfacing its
  refs in session-native status. Everything else is additive. (Spec §US1)

## Project Structure

### Documentation (this feature)

```text
specs/050-project-memory-delivery-integration/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
  domain/
    project_memory.rs    # NEW: ProjectMemoryContext, PromotionStateView,
                         #      CompatibilityOutcome, LineageRef
  adapters.rs            # MODIFIED: add Canon output reader function
  orchestrator/
    session_runtime.rs   # MODIFIED: populate ProjectMemoryContext at planning
    planner.rs           # MODIFIED: consume ProjectMemoryContext
    governance.rs        # MODIFIED: consume Canon evidence refs
  cli.rs                 # MODIFIED: render Canon refs in status/inspect/next

tests/
  unit/                  # unit tests for project_memory domain types
  integration/           # integration tests with fixture Canon output
```

**Structure Decision**: Extends the existing flat `src/` layout with a new
domain module `project_memory.rs` and modifications to existing orchestrator
and CLI modules. No new crates or top-level directories.

## Implementation Phases

### Phase 1: Consumer domain types

1. Define `PromotionStateView` enum mirroring Canon's promotion states as a
   consumer-side read-only projection.
2. Define `LineageRef` struct with the fields Boundline needs from Canon
  lineage metadata (contract_version, source_run, mode, profile,
  promotion_state, approval_state, readiness, published_at,
  update_strategy, source_artifacts).
3. Define `CompatibilityOutcome` enum (`Compatible`, `Unsupported`).
4. Define `ProjectMemoryContext` struct aggregating available Canon refs,
   promotion states, and compatibility outcome.
5. Export new types from `boundline-core`.

### Phase 2: Canon output reader

1. Implement `read_project_memory()` in `src/domain/project_memory.rs` to scan
  Canon's named `docs/project/*.md` surfaces and supporting evidence roots
  under `docs/evidence/<mode>/<RUN_ID>/`.
2. Implement adjacent packet-metadata sidecar parsing
  (`<surface>.packet-metadata.json`) with legacy fallback only where needed.
3. Implement contract-version compatibility check against the supported `0.1.x`
  line.
4. Return `ProjectMemoryContext` or an explicit error/absence result.

### Phase 3: Stage planner and assurance integration

1. Call `read_project_memory()` at stage-planning time in
   `session_runtime.rs`.
2. Feed `ProjectMemoryContext` into stage-planner decisions (credible vs.
   non-authoritative context).
3. Feed Canon evidence refs into assurance evaluation where governed stages
   reference Canon output.
4. Write `ProjectMemoryContext` into session task context for downstream
   consumption.

### Phase 4: Session-native surface integration and polish

1. Render Canon refs, promotion state, and compatibility outcome in
   `status` and `inspect` views.
2. Update docs and changelog.
3. Run clippy, fmt, and coverage validation on modified files.

## Complexity Tracking

No constitution violations identified.
