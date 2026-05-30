# Implementation Plan: Advanced Context Intelligence Semantic Acceleration

**Branch**: `059-semantic-acceleration` | **Date**: 2026-05-17 | **Spec**: [specs/059-semantic-acceleration/spec.md](specs/059-semantic-acceleration/spec.md)
**Input**: Feature specification from `/specs/059-semantic-acceleration/spec.md`

## Summary

Deliver S5.v2 as the smallest additive layer on top of the shipped S5 V1
advanced-context baseline: one optional local semantic accelerator that uses
the same workspace-local SQLite retrieval index, adds local embeddings plus
`sqlite-vec` similarity search, and surfaces explainable hybrid expansion or
reranking without changing retrieval authority order. The slice keeps the
session-native workflow primary, introduces a dedicated semantic-acceleration
policy surface that resolves alongside the V1 `AdvancedContextConfig` baseline
and defaults to disabled, preserves explicit fallback to the V1 FTS and
structured path when semantic capability is unavailable, and consumes Canon
artifacts only through the documented semantic producer contract.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: existing workspace runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`), existing `rusqlite` bundled SQLite support, and one optional `sqlite-vec` integration path for local vector tables; no remote embedding-provider dependency in the first slice  
**Storage**: existing `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and `.boundline/context-intelligence/retrieval-index.sqlite3`, extended with a dedicated `semantic_acceleration` config section resolved alongside the V1 `advanced_context` baseline plus semantic-index metadata and vector-backed chunk tables on the same workspace-local SQLite store  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused unit tests for semantic policy and hybrid ranking state, contract tests for Canon semantic compatibility and operator-facing projection shape, targeted integration tests for `plan`, `status`, and `inspect`, plus `cargo test --no-run --all-targets` for compile coverage  
**Target Platform**: macOS and Linux developer workstations, plus Linux CI  
**Project Type**: single Rust CLI and library workspace with persisted local runtime state  
**Execution Model**: sequential session-native execution with one hybrid retrieval query at a time, bounded expansion and reranking budgets, explicit disabled or local semantic states, and visible selected, degraded, insufficient, exhausted, or unavailable outcomes  
**Observability Surface**: `.boundline/session.json`, persisted traces, `boundline config show --scope effective`, and the `plan`, `status`, `next`, and `inspect` surfaces that must show the V1 advanced-context baseline, the separate semantic-acceleration policy and capability state, fallback reason, hybrid match origin, Canon semantic compatibility, and selected or rejected evidence rationale  
**Performance Goals**: improve evidence recall inside the normal interactive planning loop without introducing a second daemon or full-workspace semantic rebuild on every command; semantic refresh remains bounded to eligible changed artifacts and one decision-point query at a time  
**Constraints**: S5 V1 retrieval remains correct with semantic acceleration disabled; structured runtime context stays authoritative; semantic acceleration is local-only and defaults to disabled; the feature must not silently repurpose V1 `AdvancedContextConfig.retrieval_mode`; `sqlite-vec` is an optional capability and must degrade explicitly when unavailable; no remote embeddings, external vector stores, hidden fallback, panic-prone runtime logic, or magic literals in stable contracts  
**Scale/Scope**: one active workspace session, one retrieval index file per workspace, one dedicated semantic-acceleration policy resolved separately from the V1 `advanced_context` retrieval policy, one hybrid query per decision point, and semantic indexing limited to explainable workspace files, docs, traces, review findings, verification evidence, and compatible Canon artifacts

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by recovering semantically relevant local evidence when keyword overlap is weak while keeping the existing planning and inspection loop intact. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes retrieval correctness, fallback behavior, Canon compatibility, and operator-visible reasoning ahead of embedding polish or larger retrieval infrastructure. See Summary, Technical Context, and `research.md` Decisions 1 through 5.
- **PASS** Primary workflow: The primary operator path remains session-native (`goal -> plan -> run -> status -> next -> inspect`), while any compatibility path remains explicit and secondary in the same runtime surfaces. See Summary, Technical Context, and `quickstart.md`.
- **PASS** Bounded execution: Semantic acceleration starts only from a bounded advanced-context query, uses existing retrieval budgets plus explicit semantic expansion or rerank limits, and terminates in visible selected, degraded, insufficient, exhausted, or unavailable states. See Technical Context and `data-model.md`.
- **PASS** Stateful execution: The slice reads existing session, goal-plan, config, and retrieval-index state and writes back the semantic policy, hybrid match explanations, Canon compatibility outcomes, and terminal reasons through the same persisted session and trace surfaces. See Technical Context and `data-model.md`.
- **PASS** Mutable planning: The feature enriches the current plan and replanning flow by surfacing better evidence and explicit semantic findings, but it does not create a separate planning subsystem. See Summary, Technical Context, and `quickstart.md`.
- **PASS** Sequential-first design: The design keeps one hybrid retrieval query active at a time and does not introduce background workers, distributed indexing, or separate retrieval services. See Summary and Technical Context.
- **PASS** Tool-agent symmetry: Retrieval remains an explicit runtime action with visible candidate collection, semantic expansion or reranking, evaluation, and fallback outcomes rather than hidden heuristics. See `research.md` Decision 5 and `contracts/advanced-context-semantic-acceleration-projection-contract.md`.
- **PASS** Observability and explicit intelligence: Semantic capability state, match origin, selection and rejection reasons, Canon skip reasons, and V2-to-V1 fallback must be visible on compact and detailed runtime surfaces. See Technical Context and both contracts.
- **PASS** Catalog currency: Public provider docs were rechecked during spec creation and the no-change rationale is recorded in the feature spec; this plan keeps that evidence as the planning baseline and introduces no new hosted-provider dependency. See the Catalog Research & Currency section in the feature spec.
- **PASS** Non-goals and external separation: The plan does not depend on Canon runtime control, remote providers, review councils, long-term memory, new UI layers, or deployment infrastructure; Canon remains a bounded producer contract only. See Summary, Technical Context, and `research.md` Decision 6.
- **PASS** Minimal slice: The smallest independently valuable capability is local semantic expansion or reranking over the existing SQLite retrieval baseline with typed policy, explicit fallback, and explainable operator output. See Summary and `research.md` Decision 1.

## Project Structure

### Documentation

```text
specs/059-semantic-acceleration/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── advanced-context-semantic-acceleration-projection-contract.md
│   └── canon-semantic-acceleration-consumer-contract.md
└── tasks.md
```

### Source Code

```text
Cargo.toml
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
├── contract/
└── integration/
```

**Structure Decision**: Keep S5.v2 inside the existing advanced-context runtime,
configuration, session, trace, and CLI surfaces. Extend the current workspace-
local retrieval index at `.boundline/context-intelligence/retrieval-index.sqlite3`
rather than introducing a second data store or top-level service. If `sqlite-vec`
needs a small integration helper, keep it inside the existing orchestrator or
adapter boundary instead of creating a new project surface.

## Complexity Tracking

No constitution violations are expected for this slice.
