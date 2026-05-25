# Implementation Plan: Expand Multi-Workspace Delivery

**Branch**: `025-multi-workspace-delivery` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/boundline/specs/025-multi-workspace-delivery/spec.md](/Users/rt/workspace/boundline/specs/025-multi-workspace-delivery/spec.md)
**Input**: Feature specification from `/specs/025-multi-workspace-delivery/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend Boundline's current cluster slice from registration, inspection, and shared
defaults into one bounded multi-workspace delivery story. The slice keeps the
session-native operator path primary by adding cluster-aware planning,
execution, and follow-up under one authoritative orchestration owner, makes
workspace participation explicit in traces and summaries, and closes as
`0.25.0` with version bump, impacted docs, changelog, coverage refresh for
modified Rust files, clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/cluster.toml`, `.boundline/session.json`, `.boundline/traces/`, `.boundline/execution.json`, `.boundline/config.toml`, optional `.boundline/workflows.toml`, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit, integration, and contract coverage for clustered planning, execution, and follow-up, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed cluster, session, trace, and config state  
**Execution Model**: Sequential explicit orchestration with one authoritative cluster delivery owner and one active workspace step at a time; this slice broadens the session-native path to traverse bounded cluster members without introducing parallel fan-out  
**Observability Surface**: Persisted cluster config, session state, task context, execution traces, cluster status/inspect surfaces, and cluster-aware `run`, `status`, `next`, and `inspect` output including route owner, authoritative workspace context, and workspace participation cues  
**Performance Goals**: Operators should complete a representative clustered delivery run from one entry point in under 10 minutes, identify authoritative route/workspace and next action from one summary surface in under 2 minutes, and maintainers should validate the `0.25.0` story from docs plus runtime output in under 20 minutes  
**Constraints**: Session-native remains the primary operator path; compatibility remains explicit; one authoritative owner must remain visible even when multiple repositories participate; no hidden distributed workers, background daemons, or Canon-owned orchestration; release must bump to `0.25.0` with updated docs, changelog, coverage refresh, clippy cleanup, and formatting  
**Scale/Scope**: Two-workspace and three-workspace clusters, one bounded delivery story at a time, one active workspace step at a time, file-backed local state only, and code changes bounded to existing cluster, session, trace, CLI, and docs surfaces

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

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by letting one session-native delivery story plan and mutate across clustered repositories without manual handoff between separate runs. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/025-multi-workspace-delivery/spec.md).
- **PASS** Delivery-first scope: The work is centered on orchestration, execution, follow-up authority, and observability across repositories; polish remains a release-closeout tail, not the slice core. See Summary and Technical Context.
- **PASS** Primary workflow: Session-native remains the preferred operator path, now deepened with cluster-aware entry and follow-up, while explicit compatibility behavior stays explicit rather than becoming the default cluster runtime. See Summary, Technical Context, research, and quickstart.
- **PASS** Bounded execution: Clustered runs stay inside existing step and retry limits, stop explicitly on blocked or non-credible member transitions, and do not introduce hidden fan-out or autonomous background work. See Technical Context, research, data model, and quickstart.
- **PASS** Stateful execution: The design reuses persisted cluster config, session state, task context, and traces to record authoritative ownership and workspace participation across the delivery story. See Summary, Technical Context, and data model.
- **PASS** Mutable planning: The slice reuses current plan/replan behavior while widening the delivery context to choose and record bounded member-workspace participation inside one story. See Summary, research, and data model.
- **PASS** Sequential-first design: Only one authoritative owner and one active workspace step remain live at a time; the slice explicitly rejects parallel fan-out, distributed workers, and hidden background loops. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/025-multi-workspace-delivery/spec.md).
- **PASS** Tool-agent symmetry: Clustered execution still surfaces explicit planning, action, validation, and follow-up transitions through existing session-native orchestration and trace surfaces rather than inventing a second opaque control path. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Route owner, authoritative workspace context, workspace participation, execution condition, and next-step guidance remain inspectable across cluster-aware command output and traces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not introduce Canon-owned orchestration, provider control planes, distributed execution, long-term memory, or UI work; it reuses bounded local cluster state only. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/025-multi-workspace-delivery/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one bounded cluster delivery story with explicit workspace participation and follow-up authority on top of the existing cluster bootstrap/status/config base. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/025-multi-workspace-delivery/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cluster-delivery-surface-contract.md
│   ├── cluster-follow-up-authority-contract.md
│   └── workspace-participation-surface-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── cluster_store.rs
├── cli/
│   ├── cluster.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── cluster.rs
│   ├── session.rs
│   ├── task.rs
│   ├── task_context.rs
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

**Structure Decision**: Keep the slice inside the existing cluster, session,
task-context, orchestrator, CLI rendering, test, and documentation surfaces.
No new top-level runtime or persistence area is justified because the feature
extends the current local cluster model into bounded execution and follow-up
instead of introducing a second orchestration engine.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
