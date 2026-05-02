# Implementation Plan: Decision-Driven Orchestrator

**Branch**: `034-decision-driven-orchestrator` | **Date**: 2026-05-02 | **Spec**: [/Users/rt/workspace/synod/specs/034-decision-driven-orchestrator/spec.md](/Users/rt/workspace/synod/specs/034-decision-driven-orchestrator/spec.md)
**Input**: Feature specification from `/specs/034-decision-driven-orchestrator/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Make the existing native `DecisionLoop` the clearly authoritative execution
model for bounded goal-plan work by introducing explicit action selectors
(`read`, `search`, `modify`, `test`, `ask`, `replan`) on top of the current
decision model, using decision state plus bounded evidence to choose the next
action each iteration, and projecting selector rationale, verification intent,
recovery state, and explicit stop conditions through `run`, `status`, `next`,
and `inspect`. Ship the slice as `0.34.0` with roadmap closure, docs,
assistant-surface updates, changelog, strict lint and format validation, and
per-file coverage above 95% for modified Rust files.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.synod/session.json`, `.synod/config.toml`, optional `.synod/workflows.toml`, persisted traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, and repository-managed docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit, integration, and contract tests, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native decision loop where one bounded observation produces one explicit next-action selector, one dispatched action, one verification outcome, and one bounded recovery or stop decision at a time  
**Observability Surface**: Persisted decision history in session state, decision-oriented trace events under `.synod/traces/`, CLI summaries on `run`, `status`, `next`, and `inspect`, plus release docs and assistant guidance that explain selector-driven execution  
**Performance Goals**: Operators should recover the active selector, rationale, and governing verification or stop condition from standard output in under 2 minutes; maintainers should complete release validation for the slice in under 20 minutes  
**Constraints**: No new top-level runtime, no distributed or parallel execution, no long-term memory system, no provider-abstraction refoundation, no Canon control-plane expansion, no hidden selector heuristics, and explicit compatibility follow-up must remain subordinate and clearly trace-authoritative  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing run limits with one active selector per loop iteration and a concise evidence set per decision

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly changes how Synod delivers bounded engineering work by making next-action selection an explicit runtime control surface. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes execution control, recovery, verification, observability, and release closure ahead of optimization or polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The primary operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up stays available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The design keeps one selector per loop iteration, uses existing run limits, and makes ask/replan/terminal conditions explicit instead of hidden fallbacks. See Technical Context, research, and contracts.
- **PASS** Stateful execution: Selector choice, evidence basis, verification intent, and recovery state remain persisted in decisions, session projections, and traces. See Summary, data model, and contracts.
- **PASS** Mutable planning: The slice keeps replanning explicit through selector-driven recovery instead of freezing execution into a static task order. See Summary, research, and data model.
- **PASS** Sequential-first design: One active selector and one dispatched action remain authoritative at a time; no concurrency or hidden fan-out is introduced. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Evidence gathering, code mutation, validation, clarification, and replanning stay visible as explicit selector-driven actions across agents and tools. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Selector decisions, evidence basis, recovery transitions, and terminal reasons are surfaced through traces and existing CLI summaries. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: Canon remains a bounded evidence or governance surface only; the slice does not introduce councils, memory systems, UI work, or distributed execution. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is making the native loop choose and project explicit next-action selectors with bounded ask/replan/stop behavior. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/034-decision-driven-orchestrator/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── decision-projection-contract.md
│   ├── decision-recovery-contract.md
│   └── selector-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── decision.rs
│   ├── follow_through.rs
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── recovery.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
│   ├── decision_loop_contract.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── flow_cli_run.rs
│   └── session_native_flow.rs
└── unit/
    ├── cli_output.rs
    ├── decision_loop.rs
    ├── decision_model.rs
    └── runtime_routing.rs

README.md
ROADMAP.md
CHANGELOG.md
assistant/
docs/
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing decision model,
native decision loop, session runtime, trace and session projections, CLI
renderers, and release surfaces. No new top-level directories or runtime
surfaces are needed because the feature strengthens the existing session-owned
native loop rather than introducing a second execution engine.

## Complexity Tracking

No constitution violations are expected for this slice.
