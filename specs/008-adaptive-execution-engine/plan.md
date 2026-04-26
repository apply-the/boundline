# Implementation Plan: Adaptive Execution Engine

**Branch**: `008-adaptive-execution-engine` | **Date**: 2026-04-26 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-adaptive-execution-engine/spec.md`

## Summary

Broaden Synod beyond fixed pre-authored delivery attempts by introducing a bounded adaptive execution mode that chooses one relevant workspace slice from the current repository state, synthesizes one deterministic candidate change from that slice, validates the result, and replans to the next credible candidate when validation fails. The minimal slice keeps the current orchestrator loop, session model, trace store, and bounded review integration, but extends the execution profile, planner, fixture runtime, and CLI evidence surfaces so adaptive decisions remain explicit, inspectable, and reproducible. The release target for this slice is `0.8.0`.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies for the initial adaptive slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, and workspace execution manifests under `<workspace>/.synod/execution.json` with legacy fallback to `<workspace>/.synod/fixture.json`  
**Testing**: `cargo test --all-targets`, focused contract and integration coverage for adaptive profile loading, planner behavior, adaptive traces, and CLI surfaces, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS and Linux developer workstations plus Linux CI via the existing GitHub Actions workflows  
**Project Type**: Single Rust CLI crate with file-backed session and trace persistence plus repository-managed assistant assets  
**Execution Model**: Sequential task execution with a dedicated adaptive planner that selects one bounded workspace slice from `read_targets`, generates one candidate attempt from the current repository state, validates it, and replans to the next untried candidate within the existing run limits  
**Observability Surface**: Persisted execution traces, session status and next-command output, inspect rendering, workspace-slice selection evidence, candidate-attempt lineage, validation records, and adaptive terminal reasons  
**Performance Goals**: Initial adaptive planning remains interactive for read target sets up to roughly 20 small files, candidate generation remains bounded enough to add negligible overhead relative to validation, and status/inspect rendering remains fast enough for normal CLI use  
**Constraints**: Reuse the existing orchestrator loop, preserve one-step-at-a-time execution, avoid Canon runtime coupling, avoid background workers and distributed execution, keep review compatibility intact, keep mutations inside the workspace boundary, keep adaptive behavior deterministic and inspectable, and ship the slice as version `0.8.0` with updated docs and validation  
**Scale/Scope**: One active adaptive path per task, one selected workspace slice per attempt, a small bounded candidate set per run, one validation command per attempt, and bounded retry/replan loops driven by the existing run limits

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature directly increases Synod's ability to deliver working code when no fixed attempt list exists by selecting and changing a bounded workspace slice. See Summary and Technical Context.
- Delivery-first scope: PASS. The plan prioritizes execution, validation, recovery, and inspectability ahead of optimization or polish. See Summary, Technical Context, and Project Structure.
- Bounded execution: PASS. Adaptive runs remain limited by explicit run limits, bounded slice selection, bounded candidate generation, and explicit succeeded/failed/exhausted terminal states. See Technical Context and research decisions.
- Stateful execution: PASS. Adaptive selection evidence, attempt lineage, validation records, and latest slice state are persisted in task context, session projections, and traces. See Technical Context and data model.
- Mutable planning: PASS. The design adds a planner that can synthesize the first attempt and replace remaining steps with a new bounded attempt when failure evidence indicates a credible next path. See Summary and research decisions.
- Sequential-first design: PASS. Only one adaptive step path is active at a time; there is no concurrency, background generation, or hidden fan-out. See Technical Context.
- Tool-agent symmetry: PASS. Analysis, code mutation, validation, and evaluation remain explicit planner, agent, or tool transitions rather than hidden internal loops. See Project Structure.
- Observability and explicit intelligence: PASS. Slice selection, candidate signatures, non-repeat safeguards, validation outputs, and terminal reasons are all surfaced through trace and CLI evidence. See Technical Context and contracts.
- Non-goals and external separation: PASS. The plan does not depend on Canon, distributed execution, provider-routing frameworks, long-term memory, UI/UX work, or expanded review beyond the existing bounded slice. See Technical Context and Scope Boundaries in the spec.
- Minimal slice: PASS. The smallest independently valuable increment is one adaptive planner plus one deterministic candidate generator that can solve a real workspace bug without pre-authored attempts and still terminate explicitly when it cannot. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/008-adaptive-execution-engine/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── adaptive-execution-profile-contract.md
│   ├── adaptive-run-contract.md
│   ├── adaptive-session-contract.md
│   └── adaptive-trace-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── execution.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── engine.rs
│   ├── planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
│   ├── adaptive_execution_profile_contract.rs
│   ├── adaptive_run_contract.rs
│   ├── adaptive_session_contract.rs
│   └── adaptive_trace_contract.rs
├── integration/
│   ├── cli_adaptive_execution.rs
│   └── session_adaptive_flow.rs
├── support/
│   └── workspace_fixture.rs
└── unit/
    ├── adaptive_execution.rs
    ├── execution_profile.rs
    └── planner_behaviors.rs
```

**Structure Decision**: Keep the feature inside the existing crate and extend the current execution surfaces instead of adding another runtime or project. Reuse `src/domain/execution.rs` for adaptive profile types and attempt metadata, extend `src/fixture.rs` with adaptive slice selection and deterministic candidate generation, evolve `src/orchestrator/planner.rs` to add an adaptive planner beside the existing static planner, and wire the new evidence into the existing CLI/session/trace modules. This keeps the slice delivery-first, sequential, and small enough to ship as one bounded increment.

## Complexity Tracking

No constitution violations require justification for this slice.
