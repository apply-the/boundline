# Implementation Plan: Session-Native Surface Unification

**Branch**: `016-session-native-surface-unification` | **Date**: 2026-04-29 | **Spec**: [/Users/rt/workspace/boundline/specs/016-session-native-surface-unification/spec.md](/Users/rt/workspace/boundline/specs/016-session-native-surface-unification/spec.md)
**Input**: Feature specification from `/specs/016-session-native-surface-unification/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Unify Boundline's remaining operator-facing runtime surfaces so review, adaptive execution, governed stages, and explicit compatibility runs all project through the same session-native summary model established in 015-runtime-refoundation. The implementation will normalize route explanation, execution condition, latest decision status, optional mode projections, and next-command guidance across `run`, `status`, `next`, and `inspect`, while preserving `.boundline/execution.json` as an explicit compatibility surface and keeping Canon bounded to stage-boundary governance.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and updated repository docs and assistant assets  
**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo deny check licenses advisories bans sources`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with persisted workspace-local session and trace state  
**Execution Model**: Sequential session-owned planning and bounded execution with unified operator-surface projections for native, adaptive, review, governance, and compatibility scenarios  
**Observability Surface**: Persisted active session record, persisted execution traces, route-aware `run`, `status`, `next`, and `inspect` output, plus assistant-facing command-pack summaries  
**Performance Goals**: `status`, `next`, and `inspect` should render consistent route and condition summaries within one CLI round-trip for representative traces up to 500 events; route and condition resolution should remain negligible relative to execution time  
**Constraints**: Ship as crate version `0.16.0`; no new flow families; no provider abstraction or model gateway work; no distributed or multi-repository execution planning; no deeper Canon escalation beyond bounded stage-boundary governance and evidence projection; no orchestrator redesign; preserve explicit compatibility behavior; update Canon compatibility references to 0.24.0 where the release surface documents or validates the supported version  
**Scale/Scope**: One active session per workspace, bounded local engineering tasks, optional bounded review/adaptive/governance overlays, representative operator surfaces across native and compatibility routes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering delivery by making all operator surfaces describe the same session-native runtime story instead of fragmenting review, adaptive, governance, and compatibility into separate mental models. See Summary and Technical Context.
- **PASS** Delivery-first scope: The work is about execution explanation, session-state projection, route precedence, validation guidance, and inspectability before secondary ergonomics. See Summary and Constraints.
- **PASS** Primary workflow: `goal -> plan -> run -> status -> next -> inspect` remains the dominant operator path; direct compatibility runs remain available only as an explicit alternative. See Summary, Constraints, and quickstart scenarios.
- **PASS** Bounded execution: The slice preserves existing explicit start conditions, blocked and waiting conditions, terminal outcomes, and bounded execution semantics while improving how they are surfaced. See Technical Context, research decisions, and contracts.
- **PASS** Stateful execution: Unified summaries are derived from persisted session state, persisted decisions, task state, and persisted traces rather than from stateless rendering. See Summary, data-model, and contracts.
- **PASS** Mutable planning: Initial planning and later runtime changes remain explicit through route, condition, decision, and optional mode projections instead of hidden renderer-only logic. See research and data-model.
- **PASS** Sequential-first design: The feature does not introduce concurrency, background workers, or parallel execution. It only unifies how bounded sequential work is projected. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: The feature preserves explicit reasoning, mutation, validation, and inspection outcomes by surfacing them through shared route and condition projections rather than hiding them behind optional modes. See Summary, contracts, and quickstart.
- **PASS** Observability and explicit intelligence: Route explanation, execution condition, latest decision status, optional mode projections, blocked reasons, and next-command guidance become explicit across all operator surfaces. See Observability Surface, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains a bounded governance overlay, and the slice does not add provider abstraction, distributed execution, UI, long-term memory, or new review councils. See Constraints and spec scope boundaries.
- **PASS** Minimal slice: The smallest independently valuable capability is one coherent summary model shared by `run`, `status`, `next`, and `inspect` across native, optional, and compatibility scenarios. See Summary.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/016-session-native-surface-unification/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── operator-surface-contract.md
│   └── route-and-mode-projection-contract.md
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
src/
├── cli.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── goal_plan.rs
│   ├── governance.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   └── session_runtime.rs
└── fixture.rs

assistant/
├── README.md
├── claude/
├── codex/
└── copilot/

docs/
├── adaptive-execution.md
├── configuration.md
├── getting-started.md
├── review-voting.md
└── session-native-orchestrator-review.md

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the work inside the existing CLI, domain, orchestrator, fixture, tests, docs, and assistant asset surfaces. No new top-level runtime surface is needed; the value comes from normalizing projections across existing session-native and compatibility behavior.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
No constitution violations are expected for this slice.
