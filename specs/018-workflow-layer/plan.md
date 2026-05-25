# Implementation Plan: Session-Native Workflow Layer

**Branch**: `018-workflow-layer` | **Date**: 2026-04-30 | **Spec**: [/Users/rt/workspace/boundline/specs/018-workflow-layer/spec.md](/Users/rt/workspace/boundline/specs/018-workflow-layer/spec.md)
**Input**: Feature specification from `/specs/018-workflow-layer/spec.md`

## Summary

Add one thin named-workflow layer above Boundline's existing session-native runtime so operators can launch, resume, and inspect a bounded delivery workflow without manually chaining every phase. The implementation will validate one local workflow-definition surface, compile supported workflow phases onto the existing session-native path, persist workflow progress inside the session record, surface workflow identity and next action through the current operator summaries, and keep the direct session-native and explicit compatibility routes intact. The first slice stays sequential, rejects generic workflow-engine semantics, ships as crate version `0.18.0`, and closes with full validation plus docs, roadmap, and changelog alignment.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for the first slice  
**Storage**: Workspace-local `.boundline/workflows.toml`, `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout  
**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo deny check licenses advisories bans sources`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential session-owned workflow validation and one-phase-at-a-time progression over the existing session-native runtime with explicit stop, resume, and compatibility routing behavior  
**Observability Surface**: Active session record, persisted execution traces, workflow-aware `run`, `status`, `next`, and `inspect`, plus assistant-friendly summaries built on the same session projection model  
**Performance Goals**: Workflow validation and selection stay under 1 second for typical local workspaces; workflow start or resume stays within one normal CLI round-trip before underlying execution begins; inspection remains fast enough for operator diagnosis in under 2 minutes  
**Constraints**: No generic loops, switches, fan-out, fan-in, hidden background progression, or second workflow runtime; no Canon-owned orchestration; no silent fallback from invalid workflows into different routes; preserve existing direct session-native commands and explicit compatibility path; version bump to `0.18.0` occurs first; close the slice with coverage-aware validation, clippy hygiene, fmt, docs, roadmap, and changelog updates  
**Scale/Scope**: One active named workflow per workspace session, bounded local engineering tasks, initial workflow phases limited to existing delivery phases, docs and assistant assets updated only where the new command family changes the operator story

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering task delivery by turning repeatable phase sequences into a session-owned workflow entrypoint rather than adding generic automation. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/018-workflow-layer/spec.md).
- **PASS** Delivery-first scope: The work is about execution ergonomics, orchestration, workflow validation, session state, and inspectability before optimization or polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native, with workflow phases compiling onto `goal -> plan -> run -> status -> next -> inspect`; the explicit compatibility route through `.boundline/execution.json` remains available but opt-in. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Workflow start conditions, stop conditions, invalid-definition blocking, and existing runtime step or retry limits remain explicit; workflows pause or terminate at the first unmet bounded condition. See Technical Context, data model, quickstart, and contracts.
- **PASS** Stateful execution: Workflow identity, phase progress, and blocked or resume state live inside the shared session record and traces rather than in a separate stateless runner. See Summary, data-model, and contracts.
- **PASS** Mutable planning: Workflows compile onto existing mutable session planning and execution, so planning, replanning, and phase satisfaction remain evidence-driven rather than scripted replay. See Summary, research, and data-model.
- **PASS** Sequential-first design: The first slice keeps one active phase at a time and rejects hidden concurrency or fan-out. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/018-workflow-layer/spec.md).
- **PASS** Tool-agent symmetry: Workflow progression remains explicit because each phase still resolves through the same planner, runtime, tools, and traces already used by direct session-native execution. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Workflow identity, active phase, routing, execution condition, failures, and next action all remain visible through session and trace surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not depend on Canon beyond bounded governance or evidence boundaries and does not reintroduce deferred provider abstraction, UI, long-term memory, distributed execution, or generalized councils. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/018-workflow-layer/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one local named workflow surface that starts, resumes, and inspects existing session-native delivery work without adding a general workflow DSL. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/018-workflow-layer/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── workflow-command-surface-contract.md
│   └── workflow-definition-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   ├── session.rs
│   └── workflow.rs
├── domain/
│   ├── goal_plan.rs
│   ├── session.rs
│   └── workflow.rs
├── orchestrator/
│   ├── goal_planner.rs
│   ├── planner.rs
│   └── session_runtime.rs
├── lib.rs
├── README.md
└── ROADMAP.md

assistant/
├── README.md
├── claude/
├── codex/
└── copilot/

docs/
├── configuration.md
└── getting-started.md

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the feature inside the existing CLI, domain, orchestrator, docs, assistant-asset, and test surfaces. The only new top-level code modules expected for this slice are workflow-focused modules under the current crate, such as `src/domain/workflow.rs` and `src/cli/workflow.rs`, because the feature needs first-class workflow definitions and CLI entrypoints without introducing a separate runtime or project.

## Complexity Tracking

No constitution violations are expected for this slice.
