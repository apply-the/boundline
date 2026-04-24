# Implementation Plan: Delivery Orchestrator Core

**Branch**: `001-delivery-orchestrator-core` | **Date**: 2026-04-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-delivery-orchestrator-core/spec.md`

## Summary

Build the first Synod orchestration brain as a single Rust library crate that executes bounded tasks through a sequential loop, routes work to named agents and tools, preserves session-scoped state across steps, applies explicit retry and replanning policy, and emits inspectable execution traces without depending on Canon integration.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Rust standard library plus `serde`, `serde_json`, `thiserror`, `tracing`, and `uuid` for structured state, trace serialization, error handling, instrumentation, and stable identifiers  
**Storage**: In-memory task state for active runs; local append-only JSON trace files for inspection after completion or failure  
**Testing**: `cargo test` with unit, integration, and contract-style tests using deterministic fake agent/tool adapters  
**Target Platform**: Developer workstations on macOS and Linux, plus Linux CI targets already signaled by the Rust toolchain configuration  
**Project Type**: Single Rust library crate with internal adapters for agent execution, tool execution, and trace persistence  
**Performance Goals**: Support bounded runs of up to 100 sequential steps while keeping orchestrator decision overhead below 50 ms per loop cycle, excluding external agent/tool execution time  
**Constraints**: Sequential execution only, deterministic terminal precedence, bounded retry and replanning budgets, no Canon runtime dependency, and no hidden branching behavior  
**Scale/Scope**: One workspace-scoped task run per orchestrator instance in v1, with tens of steps, a small bounded recovery budget, and trace volume proportional to executed attempts

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature defines Synod as a delivery orchestrator for bounded engineering tasks, not as a generic agent platform.
- Delivery-first scope: PASS. The plan is centered on execution, orchestration, recovery, and traceability rather than optimization or polish.
- Bounded execution: PASS. The design enforces sequential execution, explicit terminal conditions, and configured step and recovery limits.
- Stateful execution: PASS. Shared task context is a core model and every step reads from or writes to that context.
- Mutable planning: PASS. Initial planning and bounded replanning are both first-class behaviors in the design.
- Sequential-first design: PASS. The implementation is intentionally limited to one active step at a time.
- Tool-agent symmetry: PASS. Separate registries keep agent and tool steps explicit while sharing one execution envelope.
- Observability and explicit intelligence: PASS. Trace contracts and recovery policy make decisions, retries, replanning events, and terminal outcomes inspectable.
- Non-goals and external separation: PASS. The feature excludes Canon dependency, councils, provider abstraction complexity, long-term memory, UI/UX, and deployment work.
- Minimal slice: PASS. This is the smallest useful orchestration core on which later delivery flows can build.

## Project Structure

### Documentation (this feature)

```text
specs/001-delivery-orchestrator-core/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── endpoint-execution-contract.md
│   ├── orchestrator-run-contract.md
│   └── trace-record-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
src/
├── lib.rs
├── domain/
│   ├── mod.rs
│   ├── limits.rs
│   ├── plan.rs
│   ├── step.rs
│   ├── task.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── mod.rs
│   ├── engine.rs
│   ├── planner.rs
│   ├── recovery.rs
│   └── terminal.rs
├── registry/
│   ├── mod.rs
│   ├── agent_registry.rs
│   └── tool_registry.rs
└── adapters/
    ├── mod.rs
    ├── agent.rs
    ├── tool.rs
    └── trace_store.rs

tests/
├── unit/
│   ├── recovery_policy.rs
│   ├── step_state.rs
│   └── terminal_precedence.rs
├── integration/
│   ├── retry_and_replan.rs
│   ├── sequential_task_run.rs
│   └── trace_capture.rs
└── contract/
    ├── endpoint_execution.rs
    ├── orchestrator_run.rs
    └── trace_record.rs
```

**Structure Decision**: Use a single root Rust library crate. The repository currently has Rust toolchain configuration but no source tree or crate manifest, so the feature will define the initial crate layout instead of splitting early into multiple packages. Domain models stay pure, orchestration logic stays centralized, registries isolate agent/tool lookup, and adapters contain process-bound integrations such as trace persistence.

## Complexity Tracking

No constitution violations require justification at this stage.

