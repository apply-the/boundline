# Implementation Plan: Session & Interaction Model Unification

**Branch**: `004-session-model-unification` | **Date**: 2026-04-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-session-model-unification/spec.md`

## Summary

Introduce a workspace-scoped session layer that sits between Boundline's CLI and the existing orchestrator runtime so goal capture, planning state, current execution position, latest outcome, and latest trace survive across invocations. The plan preserves the current Rust crate and bounded sequential orchestration semantics, adds a file-backed session record under `<workspace>/.boundline/session.json`, introduces session-backed CLI commands for `start`, `capture`, `plan`, `step`, `run`, `status`, and `next`, and updates assistant continuity rules so chat and CLI operate against the same explicit interaction state.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024 for the existing CLI and orchestrator backend  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.boundline/session.json` plus the existing file-backed traces under `<workspace>/.boundline/traces/`  
**Testing**: `cargo test` with new unit, integration, and contract tests for session persistence, CLI continuity, assistant continuity, recovery handling, and status routing  
**Target Platform**: macOS and Linux developer workstations plus Linux CI for formatting, linting, and test validation  
**Project Type**: Single Rust package with a local CLI, orchestrator engine, and repository-managed assistant command assets  
**Execution Model**: Sequential session-backed command flow where `start` establishes workspace state, `capture` and `plan` prepare bounded work, `step` advances one executable step, `run` continues to a terminal state, and `status`/`next` inspect the same persisted session without hidden background execution  
**Observability Surface**: Human-readable CLI output, persisted session JSON, persisted execution traces, trace inspection summaries, and assistant routing cues that must stay explicit and aligned with session state  
**Performance Goals**: Session resolution and status/next inspection remain interactive for local use, with no material overhead beyond the current CLI runtime and with session read/write work staying negligible relative to orchestration execution  
**Constraints**: No new runtime services, no background workers, no Canon dependency, no multi-session support, no long-term memory beyond the current workspace, preserve existing orchestrator terminal and recovery semantics, and keep session state human-readable and debuggable  
**Scale/Scope**: One active session per workspace, one bounded in-progress task per session, one active step at a time, and local single-developer workflows with dozens of session transitions at most per task

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. Summary, Technical Context, and research keep the slice focused on bounded engineering-task continuity across CLI and assistant interactions rather than introducing a generic agent or chat platform.
- Delivery-first scope: PASS. The plan prioritizes task continuity, execution resumption, planning state, and inspection ahead of optimization or interface polish.
- Bounded execution: PASS. Technical Context, research, and contracts preserve existing orchestrator start conditions, terminal conditions, retry limits, replan limits, and step limits while making session-dependent command failures explicit.
- Stateful execution: PASS. Data model and contracts introduce an explicit workspace-scoped session record that stores the active task snapshot, latest trace reference, and latest outcome required for later commands to continue.
- Mutable planning: PASS. Research and data model preserve initial planning plus explicit plan replacement by storing the active plan snapshot and resetting execution position when a new plan supersedes the previous one.
- Sequential-first design: PASS. The plan keeps one executable step active at a time, adds no background workers, and uses `step` and `run` as explicit user-invoked transitions over the same persisted task state.
- Tool-agent symmetry: PASS. The session layer stores task and plan state but does not hide whether the next action will execute an agent, tool, or decision step; those transitions remain visible through the existing task and trace models.
- Observability and explicit intelligence: PASS. Session JSON, status output, next-action output, existing traces, and inspect summaries make state transitions, failures, retries, replans, and recovery cues visible instead of heuristic or hidden.
- Non-goals and external separation: PASS. Technical Context and contracts avoid Canon coupling, distributed execution, provider abstraction complexity, UI surfaces, long-term memory, and deployment workflows.
- Minimal slice: PASS. The feature delivers one independently valuable capability: a single active workspace session that unifies existing CLI and assistant interactions over the current orchestrator semantics.

## Project Structure

### Documentation (this feature)

```text
specs/004-session-model-unification/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── assistant-session-continuity-contract.md
│   ├── session-command-contract.md
│   └── session-record-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
├── README.md
└── copilot/

src/
├── cli.rs
├── cli/
│   ├── diagnostics.rs
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── adapters/
│   ├── session_store.rs
│   └── trace_store.rs
├── domain/
│   ├── plan.rs
│   ├── session.rs
│   ├── step.rs
│   ├── task.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── engine.rs
│   ├── planner.rs
│   └── session_runtime.rs
└── fixture.rs

tests/
├── contract/
│   ├── assistant_session_continuity_contract.rs
│   ├── session_command_contract.rs
│   └── session_record_contract.rs
├── integration/
│   └── session_cli_flow.rs
└── unit/
    ├── session_record.rs
    └── session_store.rs
```

**Structure Decision**: Add one domain model for session state, one file-backed session adapter, one CLI module for session-facing commands, and one orchestration adapter that reuses the existing planner and step execution rules without creating a second runtime. This keeps all new complexity inside the existing Rust crate and uses the current fixture-backed execution slice plus the assistant asset surface rather than a separate state system.

## Complexity Tracking

No constitution violations require justification at this stage.