# Implementation Plan: Plan Execution Orchestration

**Branch**: `080-plan-execution-orchestration` | **Date**: 2026-06-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/080-plan-execution-orchestration/spec.md`

## Summary

Add a runtime-owned execution control plane via `boundline run --plan <ref>`. Accepted plans with explicit per-task `depends_on` declarations are loaded as a dependency-ordered task registry, executed one task at a time with checkpointing after each terminal outcome, and resumed from the last checkpoint via `boundline run --resume <id>`. Execution state lives in `.boundline/execution/checkpoints/<run-id>.json` and projects additively through `SessionStatusView`. Hard dependency on spec 079 (completion-verification runtime).

## Technical Context

**Language/Version**: Rust 1.96.0, Edition 2024

**Primary Dependencies**: Existing workspace crates (`boundline-core`, `boundline-cli`, `boundline-adapters`) + `clap`, `serde`, `serde_json`, `toml`, `thiserror`, `tracing`, `uuid`. No new external dependencies.

**Storage**: `.boundline/execution/checkpoints/<run-id>.json` — canonical checkpoint file per run, atomically written (temp → flush → sync → rename). `ActiveSessionRecord` extended with `active_execution_run_id`. `SessionStatusView` extended with additive execution projection fields.

**Testing**: `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration`. Coverage target ≥95%.

**Target Platform**: Local Boundline CLI runtime on macOS, Linux, Windows.

**Project Type**: CLI + library (Rust workspace). Extends existing `boundline run` command surface.

**Performance Goals**: Topological sort <5ms for ≤50 tasks. Checkpoint write <50ms. Resume load <10ms.

**Constraints**: Sequential execution only. No parallel dispatch. No autonomous replanning. No implicit task creation. Hard dependency on spec 079.

**Scale/Scope**: Plans up to 50 tasks, dependency depth ≤20. One active execution run per session.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Delivery Identity | ✅ PASS | Directly improves multi-task delivery with checkpointing, resume, blocked-state handling |
| II. Delivery-First Scope | ✅ PASS | Core execution orchestration — fundamental delivery concern |
| III. No Abstract Agent Systems | ✅ PASS | Deterministic task dispatch, no agent reasoning |
| IV. Bounded Execution | ✅ PASS | Sequential, one task at a time, ≤50 tasks, explicit checkpoint limits |
| V. Stateful Execution | ✅ PASS | Checkpoints persist after every terminal task outcome |
| VI. Mutable Planning | ✅ PASS | Checkpoints capture plan fingerprint; resume validates identity |
| Language Rules: No panic | ✅ PASS | Explicit error propagation; typed serde models for checkpoint schema |
| Language Rules: No magic literals | ✅ PASS | Constants for schema version, projection field names, dependency graph rules |
| Sequential-first design | ✅ PASS | V1 forbids parallel execution; deterministic topological sort with plan-order tie-breaking |
| Tool-agent symmetry + observability | ✅ PASS | Execution state visible in `status`, `inspect`, `next`; blocked tasks expose stop reason and resume command |
| Separation from external systems | ✅ PASS | Canon owns progress/handoff packets; Boundline owns dispatch, checkpointing, resume |
| Catalog currency | ✅ PASS | No provider-catalog changes; research.md records no-change rationale |

## Project Structure

### Documentation

```text
specs/080-plan-execution-orchestration/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── execution-orchestration-projection.md
└── tasks.md             # Phase 2 output (speckit.tasks)
```

### Source Code

```text
src/
├── domain/
│   ├── session.rs                          # Extend with active_execution_run_id
│   ├── goal_plan.rs                        # Extend PlannedTask with depends_on
│   └── execution_orchestration.rs          # NEW: ExecutionRun, Checkpoint, DependencyGraph
├── orchestrator/
│   ├── session_runtime.rs                  # Extend with execution orchestration dispatch
│   ├── session_runtime_finalization.rs     # Integrate completion-verification gate
│   ├── session_runtime_surface.rs          # Project execution fields into SessionStatusView
│   └── execution_orchestrator.rs           # NEW: sequential runner, checkpoint, resume
├── cli/
│   ├── run.rs                              # Extend with --plan, --accepted-plan, --resume
│   └── output.rs                           # Extend status output with execution fields
tests/
├── unit/
│   └── execution_orchestrator.rs
├── contract/
│   └── execution_orchestration_contract.rs
└── integration/
    └── execution_orchestration_flow.rs
```

## Complexity Tracking

*No constitution violations — no entries needed.*
