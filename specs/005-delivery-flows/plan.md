# Implementation Plan: Delivery Flows (SDLC Backbone)

**Branch**: `005-delivery-flows` | **Date**: 2026-04-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-delivery-flows/spec.md`

## Summary

Add deterministic delivery flows on top of the existing session model so a workspace session can bind one built-in flow, expose the current stage, advance through a fixed stage order, and keep retries or replans bounded within the active stage. The implementation stays inside the current Rust CLI crate, reuses the persisted session record and trace store under `<workspace>/.boundline/`, extends status and next-command guidance with flow awareness, and preserves non-flow session usage. Flow-aware planning will map `session.active_flow.flow_name` to built-in fixture-backed step generators in `src/fixture.rs` that emit flat, stage-tagged plans for the existing sequential orchestrator.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024 for the existing CLI and orchestrator backend  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`); no new runtime dependencies for this slice  
**Storage**: Workspace-local JSON session record at `<workspace>/.boundline/session.json` plus persisted execution traces under `<workspace>/.boundline/traces/`  
**Testing**: `cargo test` with new unit, integration, and contract coverage for flow definitions, session serialization, CLI flow selection, stage-aware status and next guidance, and bounded recovery inside a stage  
**Target Platform**: macOS and Linux developer workstations plus Linux CI for formatting, linting, and test validation  
**Project Type**: Single Rust package with a local CLI, orchestrator engine, file-backed state, and repository-managed assistant command assets  
**Execution Model**: Sequential session-backed execution where one flow can be bound to a session, one stage is active at a time, stage transitions occur only after the current stage reaches a successful terminal outcome, and retries or replans remain inside the current stage within existing bounded limits  
**Observability Surface**: Human-readable CLI status and next output, persisted session JSON, persisted execution traces, trace inspection summaries, and explicit flow and stage transition events  
**Performance Goals**: Flow selection, status, and next-command guidance remain interactive for local use; flow bookkeeping adds negligible overhead relative to current planning and execution; no material regression in existing non-flow commands  
**Constraints**: Reuse the existing orchestrator and session runtime, keep flow definitions deterministic and built in, preserve one-step-at-a-time execution, use the existing task retry and replan limits as the only execution bounds for stage recovery, avoid new agent types or background workers, keep non-flow session commands working unchanged, and avoid Canon dependencies or adaptive flow logic  
**Scale/Scope**: One active flow per workspace session, three built-in flows (`bug-fix`, `change`, `delivery`), stage sequences of two to four stages, and local single-developer workflows with dozens of stage transitions at most per task

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. Summary and Technical Context keep the feature focused on bounded engineering-task delivery by adding deterministic SDLC stage sequencing to the current session workflow.
- Delivery-first scope: PASS. The plan prioritizes stage-aware execution, session continuity, recovery behavior, and inspectability before any optional extensibility or polish.
- Bounded execution: PASS. Technical Context, research, and contracts preserve explicit start conditions, terminal states, max step limits, and max retry or replan limits from the existing orchestrator while adding visible stage transitions.
- Stateful execution: PASS. Data model and contracts extend the persisted session record with flow and stage state so later commands read and write shared execution context rather than inventing stateless shortcuts.
- Mutable planning: PASS. Research and data model keep the existing plan and replan model intact while constraining any replan to the active stage unless the user explicitly resets the flow.
- Sequential-first design: PASS. One stage and one step remain active at a time, with no concurrency, background workers, or hidden fan-out.
- Tool-agent symmetry: PASS. Stages remain sequences of visible plan steps, so reasoning, tool use, and evaluation continue to flow through the same explicit step model.
- Observability and explicit intelligence: PASS. Session JSON, status output, next guidance, and trace events expose flow selection, current stage, stage transitions, retries, replans, failures, and terminal outcomes.
- Non-goals and external separation: PASS. The plan avoids Canon coupling, councils, provider abstraction complexity, custom flow configuration, UI work, and deployment concerns.
- Minimal slice: PASS. The smallest independently valuable capability is a built-in deterministic flow bound to the existing session model with stage-aware execution and visibility.

## Project Structure

### Documentation (this feature)

```text
specs/005-delivery-flows/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── flow-command-contract.md
│   ├── flow-session-contract.md
│   └── flow-status-contract.md
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
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── flow.rs
│   ├── plan.rs
│   ├── step.rs
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   ├── engine.rs
│   └── session_runtime.rs
└── fixture.rs

tests/
├── contract/
│   ├── flow_command_contract.rs
│   ├── flow_session_contract.rs
│   └── flow_status_contract.rs
├── integration/
│   └── flow_cli_run.rs
└── unit/
    ├── flow_definition.rs
    └── session_flow_state.rs
```

**Structure Decision**: Keep all work inside the existing Rust crate. Add one small domain module for built-in flow definitions, extend the existing session model and CLI status surfaces, wire flow-aware behavior into the current session runtime and orchestrator logic, use `src/fixture.rs` as the flow-name-to-plan mapper for the built-in fixture-backed vertical slice, and mirror the established unit, integration, and contract test layout. No new top-level projects or runtime surfaces are introduced.

## Complexity Tracking

No constitution violations require justification at this stage.
