# Implementation Plan: Unify Route Summaries And Config Projection

**Branch**: `024-unify-route-summaries` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/boundline/specs/024-unify-route-summaries/spec.md](/Users/rt/workspace/boundline/specs/024-unify-route-summaries/spec.md)
**Input**: Feature specification from `/specs/024-unify-route-summaries/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Unify Boundline's operator-facing route summaries by projecting more native, workflow, review/governance, and explicit compatibility follow-up state through one shared summary model while keeping route ownership and continuity authority explicit. The slice stays bounded to read-side projection and release alignment: existing session, trace, workflow, and config state remain authoritative, config projection stays limited to material routing facts, and the release closes as `0.24.0` with updated docs, assistant guidance, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit, integration, and contract coverage for summary-model convergence and config projection, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session, trace, workflow, and config state  
**Execution Model**: Sequential explicit orchestration with one authoritative follow-up state at a time; this slice changes read-side projection and wording, not the underlying execution loop or retry model  
**Observability Surface**: Persisted session and trace state, workflow projection, routing/config summaries, `run`, `status`, `next`, `inspect`, workflow status/inspect outputs, review/governance follow-up cues, compatibility inspect guidance, and explicit route plus execution-condition output  
**Performance Goals**: Operators should identify route owner, continuity authority, execution condition, material config inputs, and recommended next action from one summary surface in under 2 minutes; maintainers should validate the unified `0.24.0` story from docs and CLI output in under 15 minutes  
**Constraints**: Session-native remains the primary operator path; compatibility remains explicit; no hidden promotion of compatibility into native ownership; no new orchestration engine, provider gateway, background service, or distributed execution surface; release must bump to `0.24.0` with updated docs, changelog, coverage refresh, clippy cleanup, and formatting  
**Scale/Scope**: One workspace at a time, one authoritative follow-up story at a time, existing route families only, config projection limited to fields that materially affect follow-up interpretation, and code changes bounded to current summary and routing surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Primary workflow: State whether the main operator path is session-native (`goal -> plan -> run -> status -> next -> inspect`) and identify any explicit compatibility path that remains available.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior beyond bounded governance/evidence boundaries or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making route follow-up summaries easier to understand across existing execution paths without changing orchestration ownership. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/024-unify-route-summaries/spec.md).
- **PASS** Delivery-first scope: The work prioritizes operator understanding of execution, follow-up authority, and routing/config interpretation ahead of polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: Session-native remains the main operator path, while workflow, review/governance, and explicit compatibility follow-up stay available and explicit within the same summary family. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The slice does not broaden execution budgets; it keeps existing start, stop, and authority rules and makes them more legible through aligned summaries. See Technical Context, data model, and quickstart.
- **PASS** Stateful execution: The design reuses existing persisted session, trace, workflow, and config state and projects them through a shared follow-up model. See Summary, Technical Context, data model, and contracts.
- **PASS** Mutable planning: The slice does not add a new planning engine; it aligns the read-side representation of planning outcomes, route ownership, and next-step guidance after the existing bounded planning/replanning behaviors. See Summary, research, and data model.
- **PASS** Sequential-first design: One authoritative follow-up state remains active at a time; the slice does not introduce concurrency or background work. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/024-unify-route-summaries/spec.md).
- **PASS** Tool-agent symmetry: Reasoning, planning, governance pauses, compatibility follow-up, and corrective guidance remain visible through explicit summary fields rather than hidden route-specific heuristics. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Route owner, authority, execution condition, config inputs, and next-step guidance remain inspectable across CLI and trace-derived surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not add Canon-owned orchestration, provider abstraction, long-term memory, UI, or distributed execution, and it reuses existing bounded route surfaces only. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/024-unify-route-summaries/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is a unified summary and config projection model across existing routes with explicit route ownership preserved. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/024-unify-route-summaries/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── route-summary-surface-contract.md
│   ├── config-projection-surface-contract.md
│   └── route-ownership-preservation-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── engine.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/

assistant/
└── README.md

docs/
├── adaptive-execution.md
├── configuration.md
└── getting-started.md

README.md
CONTRIBUTING.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
```

**Structure Decision**: Keep the slice inside the existing session, trace, CLI rendering, runtime projection, docs, and test surfaces. No new top-level runtime or persistence area is justified because the feature converges operator-facing summaries and config projection on existing bounded route state rather than introducing a new execution mode.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
