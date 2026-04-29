# Implementation Plan: Native Loop Integration

**Branch**: `014-native-loop-integration` | **Date**: 2026-04-29 | **Spec**: [/Users/rt/workspace/synod/specs/014-native-loop-integration/spec.md](/Users/rt/workspace/synod/specs/014-native-loop-integration/spec.md)
**Input**: Feature specification from `/specs/014-native-loop-integration/spec.md`

## Summary

Move the real session CLI onto the native planning and decision-loop path by making `plan` persist `GoalPlan` plus inferred-flow confirmation state in the active session, making `run` prefer `DecisionLoop` whenever a goal plan exists, and replacing synthetic in-loop file and command behavior with adapter-backed execution whose decisions are persisted to session and trace state. The declarative fixture runtime remains available as an explicit compatibility path rather than the default session behavior.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, Rust standard library filesystem and process APIs  
**Storage**: Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts  
**Testing**: `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, contract/integration/unit harnesses under `tests/`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential session-owned planning followed by a bounded one-decision-at-a-time loop with explicit fixture fallback routing  
**Observability Surface**: Active session record, persisted execution trace, `synod status`, `synod run`, and `synod inspect` output  
**Performance Goals**: Session planning stays sub-5s on local workspaces up to 1000 files; routing overhead is negligible relative to existing CLI execution; inspect surfaces expose routing and decision state without extra commands  
**Constraints**: No silent flow auto-confirmation, no dependency on Canon for core control flow, no parallel execution, no hidden fixture fallback when a goal plan is present, preserve explicit declarative compatibility behavior  
**Scale/Scope**: One active session per workspace, bounded step counts from existing limits, native loop aimed at small-to-medium local engineering tasks on the CLI path

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering delivery by moving the primary session path from fixture-first execution to session-native planning and next-action control. See Summary and Technical Context.
- **PASS** Delivery-first scope: The work is strictly about planning, routing, execution, persistence, and traceability; no UI, provider, or governance expansion is introduced. See Summary and Constraints.
- **PASS** Bounded execution: Start condition is a persisted goal plan; terminal conditions remain success, failure, exhaustion, and no-credible-next-action, with existing run limits preserved. See Technical Context and research decisions.
- **PASS** Stateful execution: `GoalPlan`, flow confirmation state, persisted decisions, and trace references are all written into session-owned state instead of remaining implicit runtime locals. See Summary and data-model.
- **PASS** Mutable planning: Initial planning is explicit and bounded, and the existing `Replan` decision family remains the path for controlled plan mutation later in execution. See spec requirements and research decisions.
- **PASS** Sequential-first design: The decision loop remains one bounded action at a time and does not introduce concurrency, workers, or fan-out. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: The plan keeps the decision loop explicit while routing concrete actions through agent/tool adapters instead of embedding file and process calls in the loop body. See Summary and research decisions.
- **PASS** Observability and explicit intelligence: Route choice, inferred-flow confirmation outcome, decision creation, dispatch, verification, failure, and recovery all remain visible through session and trace surfaces. See Observability Surface and contracts.
- **PASS** Non-goals and external separation: The native path remains locally testable without Canon and does not expand into non-goal areas such as councils, provider routing, or deployment work. See Constraints and Scope Boundaries in the spec.
- **PASS** Minimal slice: The smallest valuable outcome is that the existing session CLI can plan and run through native state without silently reverting to fixture behavior. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/014-native-loop-integration/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── session-planning-contract.md
│   └── session-run-routing-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── output.rs
│   └── session.rs
├── adapters/
│   ├── agent.rs
│   ├── session_store.rs
│   ├── tool.rs
│   └── trace_store.rs
├── domain/
│   ├── decision.rs
│   ├── flow.rs
│   ├── flow_policy.rs
│   ├── goal_plan.rs
│   ├── session.rs
│   ├── step.rs
│   ├── task.rs
│   ├── task_context.rs
│   ├── tool_result.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── engine.rs
│   ├── flow_inference.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── registry/
    ├── agent_registry.rs
    └── tool_registry.rs

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the feature inside the existing CLI, orchestrator, domain, adapter, and registry layers. No new top-level runtime is introduced. The new work only wires the already-added native planning and decision primitives into the real session path and adds contracts/tests around routing, session persistence, and adapter-backed execution.

## Complexity Tracking

No constitution violations are expected for this slice.
