# Implementation Plan: Canon-Grounded Reasoning And Structured Memory

**Branch**: `036-canon-grounded-memory` | **Date**: 2026-05-03 | **Spec**: [specs/036-canon-grounded-memory/spec.md](specs/036-canon-grounded-memory/spec.md)
**Input**: Feature specification from `/specs/036-canon-grounded-memory/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend Boundline's primary session-native path so Canon packets, governed artifacts,
artifact summaries, and capability signals influence bounded planning and later
decision selection as first-class evidence instead of stage-end output only.
Persist one compact Canon-grounded reasoning memory inside existing
session-native state so long-running loops can reuse decisive Canon evidence
without replaying the whole workspace, while stopping explicitly when that
memory becomes stale or contradictory. Keep compatibility follow-up explicit and
subordinate, align the Canon adapter to the stable 0.39.0 surface, and ship the
slice as `0.36.0` with release closure and >95% coverage for modified Rust
files.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, task-context state embedded in session tasks, optional `.canon/` governed artifacts, and repository-managed docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native planning plus bounded observe -> decide -> act -> verify execution where Canon-grounded context snapshots and compact memory can influence planning, replanning, and later decision selection without introducing background execution  
**Observability Surface**: Persisted goal-plan/session/task-context state, decision-oriented traces under `.boundline/traces/`, CLI summaries on `plan`, `run`, `status`, `next`, and `inspect`, Canon-governance packet lineage, and release docs plus assistant guidance that explain Canon-grounded reasoning and compact-memory credibility  
**Performance Goals**: Operators should recover the decisive Canon-grounded evidence and current compact-memory credibility from normal CLI output in under 2 minutes; long-running loops should reuse compact Canon memory when credible instead of replaying the full workspace; maintainers should complete release validation for the slice in under 20 minutes  
**Constraints**: No new top-level runtime, no distributed or parallel execution, no generic long-term memory subsystem beyond bounded session/task scope, no provider-abstraction refoundation, no Canon-controlled takeover of Boundline routing, workflows remain guardrails rather than execution owners, and explicit compatibility follow-up remains subordinate and trace-authoritative  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing session/run limits with one authoritative Canon-grounded memory summary per active goal and explicit credibility state for later loop reuse

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making Canon-grounded evidence influence planning and later decisions on the native path. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes bounded planning, execution follow-through, memory credibility, and inspectable recovery ahead of release polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up remains available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The design keeps existing step and retry limits, requires explicit stop or refresh when Canon-grounded memory is non-credible, and introduces no hidden background activity. See Technical Context, research, and contracts.
- **PASS** Stateful execution: Canon-grounded context snapshots and compact memory remain persisted in existing goal-plan, session, task-context, and trace state rather than transient runtime flags. See Summary, data model, and contracts.
- **PASS** Mutable planning: Canon-grounded memory can influence initial planning, replanning, and later decision updates while keeping plan and decision changes traceable to explicit evidence. See Summary, research, and data model.
- **PASS** Sequential-first design: Planning, decision selection, refresh, and bounded stop handling remain one-step-at-a-time state transitions with no concurrency or implicit fan-out. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Canon-grounded reasoning remains visible through explicit context assembly, decision selection, refresh, and validation actions rather than hidden heuristics. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Canon influence, packet lineage, compact-memory credibility, and refresh or stop reasons are surfaced through traces and existing CLI summaries. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: The slice consumes Canon surfaces when present but remains independently testable and executable through explicit bounded fallbacks; it does not introduce councils, provider abstraction refoundation, UI work, or generic long-term memory. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is making Canon packets, summaries, and capability signals change bounded native planning and later decisions through one compact persisted reasoning memory. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/036-canon-grounded-memory/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── canon-grounded-planning-contract.md
│   ├── canon-memory-credibility-contract.md
│   └── canon-inspection-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── governance_runtime.rs
├── cli.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── decision.rs
│   ├── follow_through.rs
│   ├── goal_plan.rs
│   ├── governance.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── goal_planner.rs
│   ├── governance.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
│   ├── decision_loop_contract.rs
│   ├── goal_plan_contract.rs
│   ├── runtime_refoundation_contract.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── runtime_refoundation_flow.rs
│   └── session_native_flow.rs
└── unit/
    ├── decision_loop.rs
    ├── decision_model.rs
    ├── goal_planner.rs
    ├── runtime_routing.rs
    └── session_model.rs

README.md
ROADMAP.md
CHANGELOG.md
assistant/
docs/
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing governance adapter,
task-context state, goal planner, session runtime, decision loop, CLI read-side
surfaces, and release artifacts. No new top-level runtime or persistence system
is needed because 036 strengthens how the session-owned native path consumes and
reuses Canon-grounded evidence rather than introducing a second orchestration
surface.

## Complexity Tracking

No constitution violations are expected for this slice.
