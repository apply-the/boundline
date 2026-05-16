# Implementation Plan: Advanced Context Intelligence

**Branch**: `058-advanced-context-intelligence` | **Date**: 2026-05-16 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/spec.md](/Users/rt/workspace/apply-the/boundline/specs/058-advanced-context-intelligence/spec.md)
**Input**: Feature specification from `/specs/058-advanced-context-intelligence/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend the existing runtime-intelligence substrate with one local-first,
bounded advanced-context layer that can expand structured context with
retrieved repository evidence, compatible Canon artifacts, and explainable
relationship and impact projections. The implementation stays sequential-first:
each plan, run, status, or inspect decision point may issue one retrieval query
at a time, apply a bounded number of refinement passes, and end in an explicit
selected, degraded, insufficient, or exhausted outcome. The first delivery
slice keeps structured runtime context authoritative, consumes Canon 051
artifact-indexing metadata only through a Boundline-owned consumer contract,
uses one workspace-local retrieval index, and makes remote semantic expansion
explicitly opt-in rather than required for correctness.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing workspace dependencies `clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, and Rust standard-library filesystem, path, collections, and process APIs, plus one embedded SQLite binding with FTS5 support for the workspace-local retrieval index; no external graph or vector service is required for the first slice  
**Storage**: Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/config.toml`, and Canon-promoted project-memory artifacts, plus a new workspace-local retrieval index under `.boundline/index/` for searchable document, evidence, and relationship state  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, contract, and integration tests for retrieval state, Canon-consumer compatibility, trace projection, and CLI inspect/status surfaces, `cargo test --no-run --all-targets`, focused member-crate tests such as `cargo test -p boundline-core ...`, `cargo test -p boundline-adapters ...`, `cargo test -p boundline-cli ...`, and `cargo nextest run --workspace --all-features` when feasible  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI and library workspace with persisted local runtime state  
**Execution Model**: Sequential session-native execution with one retrieval query active at a time, at most two refinement passes and one stale-refresh retry per decision point, and explicit terminal outcomes of selected, degraded, insufficient, or exhausted rather than hidden background expansion  
**Observability Surface**: `.boundline/session.json`, persisted traces under `.boundline/traces/`, the workspace-local retrieval index metadata, and the `plan`, `run`, `status`, `next`, and `inspect` CLI surfaces that must show retrieval mode, authority ordering, selected evidence, rejection rationale, relationship projections, impact findings, remote-disclosure state, and limit or degradation reasons  
**Performance Goals**: Operators should be able to identify why evidence or impact findings were surfaced from `status` or `inspect` in under 5 minutes, and local-first retrieval refresh must not materially change the bounded responsiveness of `plan`, `run`, or `inspect` on representative workspaces  
**Constraints**: Structured runtime context remains authoritative; Canon remains optional semantic enrichment only through the documented indexing contract; remote semantic expansion is opt-in and must never be default-on; no mandatory hosted retrieval infrastructure, no distributed search service, no hidden concurrency, no panic-prone runtime logic outside tests, and no magic literals or ad hoc stable JSON shapes in production Rust code  
**Scale/Scope**: One active workspace session and one retrieval index per workspace, one active retrieval query at a time, a bounded surfaced result set per command, up to two refinement passes and one stale-refresh retry per decision point, and a first slice focused on repository evidence, traces, review findings, verification evidence, and compatible Canon artifacts rather than enterprise-wide shared retrieval

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Primary workflow: State whether the main operator path is session-native (`start -> capture -> plan -> run -> status -> next -> inspect`) and identify any explicit compatibility path that remains available.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Catalog currency: Confirm current public provider docs were checked, the bundled model catalog was refreshed when needed, and the delta or no-change rationale is linked from the plan.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior beyond bounded governance/evidence boundaries or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

- **PASS** Delivery identity: The slice improves real bounded delivery by helping `plan`, `run`, `status`, and `inspect` find better evidence, expose impact earlier, and stop explicitly when expanded context is non-credible. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes context selection, impact analysis, failure handling, and inspectability ahead of provider breadth or retrieval polish. See Summary, Constraints, and Scale/Scope.
- **PASS** Primary workflow: The main operator path remains the session-native `start -> capture -> plan -> run -> status -> next -> inspect` workflow, with any compatibility-route use of advanced retrieval explicitly secondary. See Summary, Execution Model, and Constraints.
- **PASS** Bounded execution: Retrieval begins only from an active decision point and ends after one query, at most two refinement passes, and one stale-refresh retry, with explicit selected, degraded, insufficient, or exhausted outcomes. See Execution Model and Scale/Scope.
- **PASS** Stateful execution: Retrieval mode, selected evidence, impact findings, and degradation reasons are persisted through existing session and trace surfaces plus retrieval-index metadata so later commands can reuse or invalidate them explicitly. See Storage and Observability Surface.
- **PASS** Mutable planning: Retrieved evidence and impact findings may cause explicit replanning, step insertion, or replacement, but those changes remain visible in the same bounded session workflow. See Summary and Observability Surface.
- **PASS** Sequential-first design: The slice keeps one retrieval query active at a time and does not introduce background workers, asynchronous fan-out, or distributed search daemons. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Retrieval, selection, rejection, relationship projection, and impact inference remain explicit runtime state and trace events rather than hidden heuristics. See Summary and Observability Surface.
- **PASS** Observability and explicit intelligence: The plan requires operator-visible retrieval mode, authority ordering, selected evidence, rejected candidates, relationship projections, impact findings, and degradation reasons on normal runtime surfaces. See Observability Surface.
- **PASS** Catalog currency: The spec records a 2026-05-16 audit of OpenAI, Anthropic, and Google provider docs with a no-change result for `assistant/catalog/model-catalog.toml`; this plan carries that evidence forward unchanged. See Technical Context and Summary.
- **PASS** Non-goals and external separation: Canon remains a bounded producer input only; the plan does not depend on a new Canon feature, distributed retrieval service, UI work, councils, or deployment pipelines. See Constraints and Scale/Scope.
- **PASS** Minimal slice: The smallest independently valuable capability is one local-first hybrid retrieval and impact-projection path that strengthens existing context assembly without changing the authority model. See Summary.

## Project Structure

### Documentation (this feature)

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

### Source Code (repository root)
```text
src/
├── adapters/
│   ├── session_store.rs
│   └── trace_store.rs
├── cli/
│   ├── config.rs
│   ├── inspect.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── project_index.rs
│   ├── project_memory.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   └── session_runtime.rs
└── lib.rs

crates/
├── boundline-core/
│   └── src/
│       └── domain.rs
├── boundline-adapters/
│   └── src/
│       └── lib.rs
└── boundline-cli/
    └── src/
        └── lib.rs

tests/
├── contract/
├── integration/
└── unit/

assistant/
└── catalog/
    └── model-catalog.toml

docs/
└── configuration.md
```

**Structure Decision**: Keep the slice inside the existing runtime-intelligence,
session, trace, CLI, and Canon-consumer surfaces. The only new persistence
surface is a workspace-local retrieval index under `.boundline/index/`, which
extends the current local-first runtime model instead of introducing a new
top-level product subsystem. Any new shared domain modules added under
`src/domain/` must also be registered through `crates/boundline-core/src/domain.rs`
so the member crates continue to compile path-based imports.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
