# Implementation Plan: Multi-Workspace Orchestration

**Branch**: `012-multi-workspace` | **Date**: 2026-04-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/012-multi-workspace/spec.md`

**Note**: This plan keeps the first slice bounded to cluster bootstrap,
cluster-aware session projection, cluster status/inspection, and cluster-level
config precedence so the feature remains independently valuable and executable.

## Summary

Add a bounded multi-workspace slice that lets Boundline register a named cluster of
member repositories, project one shared cluster identity into session-aware
flows, aggregate status and inspection output across member workspaces, and
apply cluster-scoped routing defaults between workspace-local and user-global
configuration. The implementation keeps the current single-workspace execution
engine intact, stores cluster metadata under the primary workspace’s `.boundline/`
directory, reuses existing session and trace stores instead of introducing a
distributed runtime, and exposes the new behavior through explicit CLI surfaces
and output renderers.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) plus Rust standard library path and filesystem APIs; no new runtime dependencies for the first clustered slice  
**Storage**: Workspace-local `.boundline/session.json` and `.boundline/traces/` remain authoritative per repository, existing workspace `.boundline/config.toml` and user-global config remain in place, and a new primary-workspace `.boundline/cluster.toml` stores cluster membership and cluster-scoped defaults  
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused unit tests for cluster validation and precedence, focused integration tests for cluster CLI/session/status flows, and focused contract tests for cluster CLI/config surfaces  
**Target Platform**: macOS and Linux developer workstations, Linux CI, and assistant-driven repository sessions in VS Code  
**Project Type**: Single Rust CLI crate with file-backed state and explicit CLI output surfaces  
**Execution Model**: Sequential CLI commands with one cluster-aware operation active at a time, no background workers, no distributed execution, and cluster aggregation implemented as explicit reads over member workspace state  
**Observability Surface**: CLI cluster summaries, cluster-aware session/status output, cluster inspection output, persisted per-workspace traces, and explicit effective-config source attribution  
**Performance Goals**: Cluster init, status, and effective-config inspection should complete within one CLI round-trip for a small cluster of 2-5 workspaces and feel interactive, targeting roughly under 2 seconds before large trace loading  
**Constraints**: Preserve single-workspace behavior, keep the slice independent from Canon, avoid automatic cross-repository plan fan-out, keep precedence explicit as CLI > workspace > cluster > global > built-in, and defer distributed execution plus cluster-wide review-system expansion  
**Scale/Scope**: One primary workspace per cluster, 2-5 member workspaces in the first slice, one active clustered session projection at a time, and only bounded cluster bootstrap, inspection, and inherited defaults

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature directly improves bounded delivery across related repositories by turning multi-workspace work from manual coordination into explicit Boundline orchestration state. See Summary and Technical Context.
- Delivery-first scope: PASS. The plan prioritizes cluster bootstrap, session projection, status/inspection, and config resolution before documentation polish or future automation. See Summary and Project Structure.
- Bounded execution: PASS. Cluster commands have explicit start conditions, explicit invalid-member and malformed-config terminal states, and no hidden retries or background work. See Technical Context and research decisions.
- Stateful execution: PASS. The slice persists cluster metadata in `.boundline/cluster.toml` and reads or projects existing session and trace state from member workspaces instead of discarding context. See Technical Context and data model.
- Mutable planning: PASS. The feature does not replace the existing planner; it prepares bounded clustered context that later planning can consume while leaving future automatic cross-repository plan mutation deferred. See Summary and Scope Boundaries.
- Sequential-first design: PASS. Cluster init, status, inspect, and config mutation each run as one explicit command with sequential member aggregation. See Technical Context.
- Tool-agent symmetry: PASS. Cluster behavior is expressed through CLI commands, persisted files, and explicit output rather than hidden heuristics. See contracts and quickstart.
- Observability and explicit intelligence: PASS. Cluster status, cluster inspection, and effective-config source attribution make decisions and gaps visible. See Technical Context and contracts.
- Non-goals and external separation: PASS. The plan does not depend on Canon, does not introduce distributed systems or provider abstractions, and keeps future cross-repository execution automation explicitly out of scope. See Technical Context and Scope Boundaries.
- Minimal slice: PASS. The smallest independently valuable capability is cluster registration plus cluster-aware status/inspection and inherited defaults layered onto existing single-workspace behavior. See Summary.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/012-multi-workspace/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── cluster-cli-contract.md
│   └── cluster-config-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   ├── cluster_store.rs
│   ├── config_store.rs
│   ├── session_store.rs
│   └── trace_store.rs
├── cli/
│   ├── cluster.rs
│   ├── config.rs
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── cluster.rs
│   ├── configuration.rs
│   ├── session.rs
│   └── trace.rs
├── lib.rs
└── cli.rs

docs/
├── configuration.md
└── getting-started.md

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the entire slice inside the existing CLI crate and
state adapters. Add one dedicated cluster domain model and one cluster store so
clustered behavior stays explicit instead of being smeared across session and
config logic. Extend existing CLI modules for config, inspect, and session only
where cluster-aware projection is necessary, and add one focused `cluster`
module for the new command surface. No new top-level projects or runtime
surfaces are introduced.

## Complexity Tracking

No constitution violations require justification for this slice.