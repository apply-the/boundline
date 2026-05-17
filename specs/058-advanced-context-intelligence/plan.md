# Implementation Plan: Advanced Context Intelligence

**Branch**: `058-advanced-context-intelligence` | **Date**: 2026-05-16 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/spec.md](/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/spec.md)
**Input**: Feature specification from `/specs/058-advanced-context-intelligence/spec.md`

## Summary

Deliver the S5 V1 advanced-context baseline as one local-first retrieval layer
that augments the existing runtime-intelligence substrate with workspace-local
SQLite + FTS5 indexing, structured fallback ordering, explainable selected
evidence, relationship and impact projection, Canon consumer compatibility, and
typed disabled or local retrieval policy. The implementation remains
sequential-first: one retrieval query per decision point, bounded budgets,
explicit selected or degraded terminal states, and no dependence on embeddings,
graph stores, or remote providers. Semantic acceleration, sqlite-vec, and graph
projection remain deferred to S5.v2 and later roadmap slices.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024
**Primary Dependencies**: existing workspace crates plus one embedded SQLite
binding with bundled FTS5 support; no vector, graph, or remote retrieval
dependency is required for V1
**Storage**: existing local `.boundline/session.json`, `.boundline/traces/`,
`.boundline/config.toml`, and Canon-promoted project-memory artifacts, plus one
local retrieval index at `.boundline/context-intelligence/retrieval-index.sqlite3`
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit tests for state and projection, focused CLI and planner tests, `cargo test --no-run --all-targets`, and broader integration or nextest runs during closeout
**Target Platform**: macOS and Linux developer workstations, plus Linux CI
**Project Type**: single Rust CLI and library workspace with persisted local runtime state
**Execution Model**: sequential session-native execution with one retrieval query at a time, bounded budgets, structured fallback, and explicit selected, degraded, insufficient, or unavailable outcomes
**Observability Surface**: `.boundline/session.json`, persisted traces, the local retrieval index metadata, and the `plan`, `status`, and `inspect` surfaces that must show retrieval mode, selected evidence, authority order, relationship findings, impact findings, and degraded or terminal reasons
**Constraints**: structured runtime context remains authoritative; Canon remains semantic producer only; remote retrieval is unsupported in V1; no hosted retrieval, no graph runtime, no hidden background indexing, no panic-prone runtime logic outside tests, and no magic literals in stable production contracts
**Scale/Scope**: one active workspace session, one local retrieval index per workspace, one retrieval query per decision point, bounded selected-evidence counts, and one first slice focused on local repository evidence, traces, tests, and compatible Canon artifacts

## Constitution Check

- **PASS** Delivery identity: The slice improves bounded delivery by surfacing better local evidence, relationship clues, and explicit impact gaps inside `plan`, `status`, and `inspect`.
- **PASS** Delivery-first scope: The plan prioritizes retrieval correctness, authority order, explicit degradation, and inspectability ahead of semantic acceleration or provider breadth.
- **PASS** Primary workflow: The main path remains the session-native workflow; advanced context is an augmentation to that path, not a separate subsystem.
- **PASS** Bounded execution: Retrieval starts from an active decision point, uses explicit budgets, and terminates in visible selected, degraded, insufficient, or unavailable states.
- **PASS** Stateful execution: Retrieval outputs are persisted through existing session, task-context, and trace surfaces plus the local SQLite index metadata.
- **PASS** Mutable planning: Selected evidence and impact findings can inform replanning, but those changes remain visible inside the same bounded workflow.
- **PASS** Sequential-first design: The slice keeps one retrieval query active at a time and does not introduce background workers or distributed search infrastructure.
- **PASS** Tool-agent symmetry: Retrieval, fallback, compatibility checks, and impact findings remain explicit runtime state and trace output rather than hidden heuristics.
- **PASS** Observability and explicit intelligence: Retrieval mode, selected evidence, authority ordering, relationships, impact findings, and degraded reasons are operator-visible.
- **PASS** Catalog currency: Provider catalogs were re-checked, but V1 remains local-first and does not change the routing catalog contract.
- **PASS** Non-goals and external separation: Canon remains a bounded producer input only, and semantic acceleration, sqlite-vec, graph projection, and remote providers stay deferred.
- **PASS** Minimal slice: The smallest independently valuable capability is one local SQLite + FTS5 retrieval path with explainable impact projection and explicit disabled or local policy.

## Project Structure

### Documentation

```text
specs/058-advanced-context-intelligence/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── advanced-context-intelligence-projection-contract.md
│   └── canon-semantic-retrieval-consumer-contract.md
└── tasks.md
```

### Source Code

```text
src/
├── cli/
│   ├── config.rs
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── context_intelligence.rs
│   ├── goal_plan.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── context_intelligence.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── unit/
│   ├── context_intelligence_projection.rs
│   └── context_intelligence_state.rs
├── contract/
└── integration/
```

**Structure Decision**: Keep the slice inside the existing runtime, planning,
session, trace, and CLI surfaces. The only new persistence surface is the
workspace-local SQLite retrieval index under
`.boundline/context-intelligence/retrieval-index.sqlite3`. There is no new
graph runtime, hosted retrieval service, or provider abstraction in V1.

## Complexity Tracking

No constitution violations are expected for this slice.