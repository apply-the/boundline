# Implementation Plan: Execution Engine (Code Delivery)

**Branch**: `006-execution-engine` | **Date**: 2026-04-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-execution-engine/spec.md`

## Summary

Replace the current fixture-only vertical slice with a workspace execution profile that still uses the existing orchestrator, session model, and trace store but can read workspace files, apply bounded multi-file change sets, record diff-style change evidence, run validation commands, and retry or replan within explicit delivery limits. The minimal slice keeps execution sequential, supports the new `.boundline/execution.json` manifest with backward compatibility for legacy `.boundline/fixture.json`, and makes `run`, session planning, status, and inspect surfaces reflect real delivery evidence instead of only fixture patch results.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library process and filesystem APIs; no new runtime dependencies for the initial execution-engine slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, and workspace execution manifests under `<workspace>/.boundline/execution.json` with legacy fallback to `<workspace>/.boundline/fixture.json`  
**Testing**: `cargo test --all-targets`, contract and integration coverage for execution manifests and delivery traces, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo fmt --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**Target Platform**: macOS and Linux developer workstations plus Linux CI via the existing GitHub Actions workflows  
**Project Type**: Single Rust CLI crate with file-backed session and trace persistence plus repository-managed assistant assets  
**Execution Model**: Sequential task execution where one delivery attempt reads workspace state, applies a bounded change set, runs validation, records change evidence, and either advances, retries, replans, or terminates within the current task limits  
**Observability Surface**: Persisted execution traces, session status and next-command output, inspect rendering, workspace diagnostics, and stable change evidence captured per attempt  
**Performance Goals**: Local delivery runs remain interactive for small workspace slices, change evidence generation adds negligible overhead relative to validation commands, and status or inspect rendering remains fast enough for command-line use  
**Constraints**: Reuse the existing orchestrator loop, preserve one-step-at-a-time execution, avoid background workers and multi-agent review, keep writes inside the workspace boundary, maintain legacy fixture compatibility during the transition, and raise every Rust source file to at least 90% line coverage in the updated `lcov.info` output  
**Scale/Scope**: One workspace-local execution profile per run, one active task per session, small multi-file change sets, one validation command per attempt, and bounded retry or replan loops driven by existing run limits

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature directly upgrades Boundline from orchestration-only behavior to working-code delivery in the active workspace.
- Delivery-first scope: PASS. The plan prioritizes file mutation, validation, terminal handling, and inspectable evidence before any optimization or later review features.
- Bounded execution: PASS. Delivery attempts, retries, replans, validation runs, and terminal outcomes remain controlled by the existing explicit run limits and terminal conditions.
- Stateful execution: PASS. Session state, task context, latest validation outcomes, change evidence, and traces all stay inside the existing shared session and trace surfaces.
- Mutable planning: PASS. The design keeps the current planner and replan hooks, but replacement plans and retries operate on real workspace change attempts rather than fixture-only patches.
- Sequential-first design: PASS. The feature keeps one active step at a time and introduces no concurrency, hidden fan-out, or background processing.
- Tool-agent symmetry: PASS. Analysis, change application, validation, and evidence collection remain explicit agent or tool steps executed through the existing registry model.
- Observability and explicit intelligence: PASS. Workspace reads, file writes, validation outcomes, retries, replans, diff evidence, and terminal outcomes become explicit trace data and CLI output.
- Non-goals and external separation: PASS. The plan does not depend on Canon, provider routing, councils, deployment pipelines, UI work, or long-term memory.
- Minimal slice: PASS. The smallest independently valuable increment is a workspace-backed execution profile that can drive one bounded real delivery loop with inspectable evidence.

## Project Structure

### Documentation (this feature)

```text
specs/006-execution-engine/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── execution-profile-contract.md
│   ├── run-command-contract.md
│   └── execution-trace-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── execution.rs
│   ├── session.rs
│   ├── step.rs
│   ├── task.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── engine.rs
│   ├── planner.rs
│   └── session_runtime.rs
└── adapters/
    ├── agent.rs
    ├── session_store.rs
    ├── tool.rs
    └── trace_store.rs

tests/
├── contract/
│   ├── execution_profile_contract.rs
│   ├── run_command_contract.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── cli_custom_run.rs
│   ├── session_cli_flow.rs
│   └── trace_capture.rs
├── support/
│   └── workspace_fixture.rs
└── unit/
    ├── execution_profile.rs
    ├── session_record.rs
    └── session_store.rs
```

**Structure Decision**: Keep the work inside the existing crate and extend the current execution surfaces instead of introducing a new runtime project. Add one new domain module for execution-profile and change-evidence state, evolve `src/fixture.rs` into the concrete workspace execution engine with legacy fixture compatibility, wire the new evidence into CLI and session runtime output, and expand the existing unit, contract, and integration test layout to enforce the 90%-per-file coverage gate.

## Complexity Tracking

No constitution violations require justification for this slice.
