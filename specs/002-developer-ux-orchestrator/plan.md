# Implementation Plan: Developer UX for Orchestrator Core

**Branch**: `002-developer-ux-orchestrator` | **Date**: 2026-04-24 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-developer-ux-orchestrator/spec.md`

## Summary

Add a minimal developer-facing CLI to the existing Rust package so contributors can run a deterministic fixture-backed vertical slice, submit a simple local task, inspect recorded traces, and verify local readiness without writing integration code against the library directly. The plan keeps the existing orchestrator core as the execution engine, adds one local binary plus thin CLI and fixture modules, and preserves sequential bounded execution, persisted traces, and explicit failure handling.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus `clap` 4.x for a stable subcommand-based CLI surface  
**Storage**: In-memory task state during execution and local file-backed traces under `<workspace>/.synod/traces/` through the existing trace store  
**Testing**: `cargo test` with unit, integration, and contract tests, plus CLI-focused integration coverage over deterministic fixture-backed runs and custom goals  
**Target Platform**: macOS and Linux developer workstations, plus Linux CI validation for formatting, linting, and tests  
**Project Type**: Single Rust package with a reusable library crate and one local developer CLI binary  
**Execution Model**: Synchronous CLI invocations layered over the existing sequential task loop with bounded retries, bounded replanning, and deterministic local workspace fixtures  
**Observability Surface**: Human-readable CLI progress, explicit terminal exit codes, local readiness diagnostics, persisted JSON traces, and readable trace inspection summaries  
**Performance Goals**: Contributors can reach a first fixture-backed run from a documented checkout in under 5 minutes, while diagnostics and trace inspection add only interactive local overhead, targeting sub-2-second completion for typical single-run summaries excluding orchestrated step execution time  
**Constraints**: Reuse the current orchestrator core and trace store; keep the experience text-only and non-interactive after invocation; no Canon dependency, remote providers, background services, concurrency, or advanced configuration surface in this slice  
**Scale/Scope**: One developer-triggered local run or trace inspection at a time, tens of steps per bounded run, one deterministic fixture-backed red-to-green slice, and trace output sized for local debugging rather than fleet-scale operations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. Summary and Technical Context keep the feature focused on making bounded orchestrator execution usable for developer delivery work rather than introducing a generic chat or agent platform.
- Delivery-first scope: PASS. Summary, Technical Context, and Project Structure prioritize runnable execution, diagnostics, inspectability, and validation before any optimization or polish.
- Bounded execution: PASS. Technical Context, data-model, and contracts preserve explicit start conditions, terminal outcomes, and the existing step, retry, and replanning limits for every run.
- Stateful execution: PASS. Summary and data-model build directly on the existing task context, trace persistence, and step history so runs remain stateful and inspectable.
- Mutable planning: PASS. Technical Context, research, and data-model keep initial plan creation plus visible retry and replanning behavior in scope through the current orchestrator engine.
- Sequential-first design: PASS. Technical Context and Project Structure explicitly retain the existing one-step-at-a-time orchestrator and reject background workers or concurrent execution.
- Tool-agent symmetry: PASS. Summary, contracts, and data-model reuse existing agent and tool registries and require CLI progress plus trace summaries to surface both step categories explicitly.
- Observability and explicit intelligence: PASS. Technical Context, contracts, and quickstart require readable progress, diagnostics, persisted traces, recovery events, and explicit terminal reasons with no hidden fallback behavior.
- Non-goals and external separation: PASS. Technical Context and Project Structure avoid Canon runtime integration, provider complexity, distributed systems, memory expansion, interactive UI, and deployment work.
- Minimal slice: PASS. Summary and Project Structure add a single local CLI binary plus thin support modules over the existing library, which is the smallest independently valuable developer-UX slice.

## Project Structure

### Documentation (this feature)

```text
specs/002-developer-ux-orchestrator/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── developer-command-contract.md
│   ├── diagnostics-report-contract.md
│   ├── workspace-fixture-contract.md
│   └── trace-summary-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
src/
├── lib.rs
├── bin/
│   └── synod.rs
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   └── run.rs
├── fixture.rs
├── adapters/
│   ├── agent.rs
│   ├── tool.rs
│   └── trace_store.rs
├── domain/
│   ├── limits.rs
│   ├── plan.rs
│   ├── step.rs
│   ├── task.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── engine.rs
│   ├── planner.rs
│   ├── recovery.rs
│   └── terminal.rs
└── registry/
    ├── agent_registry.rs
    └── tool_registry.rs

tests/
├── contract/
│   ├── cli_command_contract.rs
│   ├── diagnostics_report_contract.rs
│   ├── endpoint_execution.rs
│   ├── orchestrator_run.rs
│   ├── trace_record.rs
│   └── trace_summary_contract.rs
├── integration/
│   ├── cli_custom_run.rs
│   ├── cli_diagnostics.rs
│   ├── cli_trace_inspection.rs
│   ├── fixture_vertical_slice.rs
│   ├── retry_and_replan.rs
│   ├── sequential_task_run.rs
│   └── trace_capture.rs
└── unit/
    ├── cli_output.rs
    ├── planner_behaviors.rs
    ├── recovery_policy.rs
    ├── step_state.rs
    ├── task_and_plan.rs
    ├── task_context_state.rs
    └── terminal_precedence.rs
```

**Structure Decision**: Keep a single Rust package and add one local binary plus thin `cli` and `fixture` modules inside the existing source tree. This preserves the current library-first architecture, avoids a second crate or workspace layer, and keeps new complexity limited to the developer command surface, deterministic workspace-fixture execution, and trace-summary formatting required by the spec.

## Complexity Tracking

No constitution violations require justification at this stage.
