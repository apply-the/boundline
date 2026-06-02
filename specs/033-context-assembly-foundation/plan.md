# Implementation Plan: Context Assembly Foundation

**Branch**: `033-context-assembly-foundation` | **Date**: 2026-05-02 | **Spec**: [specs/033-context-assembly-foundation/spec.md](specs/033-context-assembly-foundation/spec.md)
**Input**: Feature specification from `/specs/033-context-assembly-foundation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Introduce a first-class bounded `ContextPack` into goal planning so Boundline stops
planning from ambient repository state and instead derives explicit planning
inputs from workspace signals, authored briefs, negotiated delivery state,
recent traces, and reusable Canon artifacts. The slice remains inside the
existing session-native and inspect surfaces, adds explicit credibility and
provenance reporting, and ships as `0.33.0` with roadmap, docs, changelog,
coverage, clippy, and formatting closeout.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, collections, and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/config.toml`, `.boundline/workflows.toml`, persisted traces under `<workspace>/.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and updated repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit and contract coverage for goal planning and output projection, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state and repository-managed assistant assets  
**Execution Model**: Sequential session-native planning and bounded execution where context assembly happens before goal-plan confirmation and later surfaces project the same context-pack state without introducing background workers or parallel branches  
**Observability Surface**: Persisted goal plans, session projections, trace `GoalPlanCreated` payloads, `plan`, `run`, `status`, `next`, and `inspect` summaries, plus release docs that describe context assembly as part of the primary Boundline path  
**Performance Goals**: Operators should identify context inputs and provenance from standard output in under 2 minutes; maintainers should validate the `0.33.0` release story in under 20 minutes  
**Constraints**: No new top-level runtime, no hidden retrieval loops, no unbounded repository indexing, no provider-routing refoundation, no GUI work, no distributed execution, and no Canon control-plane expansion beyond bounded artifact reuse  
**Scale/Scope**: One workspace or registered cluster at a time; bounded context packs should stay concise enough to explain one planned task sequence and its evidence without replaying the whole repository

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by ensuring planning can point to explicit context inputs instead of relying on ambient repository state. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes planning credibility, context provenance, inspectability, and bounded failure handling before release polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `goal -> plan -> run -> status -> next -> inspect`; explicit compatibility remains available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Context assembly runs once per planning step, yields an explicit credibility state, and blocks planning when no bounded context is credible; no infinite retrieval or hidden background work is introduced. See Technical Context, research, and data model.
- **PASS** Stateful execution: Context-pack state is attached to the goal plan and projected through session and trace surfaces rather than recomputed opaquely on every render. See Summary, data model, and contracts.
- **PASS** Mutable planning: Initial planning gains explicit context assembly while later replanning and follow-through continue to use mutable goal-plan and decision surfaces rather than a fixed scripted runner. See Summary, research, and data model.
- **PASS** Sequential-first design: One planning loop remains active at a time and the slice introduces no concurrency or hidden fan-out. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Evidence selection, failure signaling, and surfaced context rationale remain explicit in traces and CLI summaries rather than hidden inside adapters. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Context summaries, provenance cues, credibility state, failure evidence, and route authority are all surfaced through session or trace outputs. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains bounded evidence input only; the slice does not introduce long-term memory systems, deployment work, UI surfaces, or provider abstraction churn. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is one explicit context pack that makes planning inputs inspectable and blocks non-credible planning. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/033-context-assembly-foundation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── context-credibility-contract.md
│   ├── context-pack-contract.md
│   └── context-projection-contract.md
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
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── goal_plan.rs
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
├── integration/
└── unit/

tech-docs/
├── getting-started.md
└── configuration.md

README.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing goal-plan, session,
trace, CLI projection, docs, and tests surfaces. The expected code changes are
bounded updates to `goal_plan`, `goal_planner`, `session_runtime`, session or
trace projections, and CLI rendering. No new top-level runtime or storage
surface is justified because the feature adds explicit planning context rather
than a second orchestration model.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
