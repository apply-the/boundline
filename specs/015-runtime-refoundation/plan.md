# Implementation Plan: Runtime Refoundation

**Branch**: `015-runtime-refoundation` | **Date**: 2026-04-29 | **Spec**: [/Users/rt/workspace/synod/specs/015-runtime-refoundation/spec.md](/Users/rt/workspace/synod/specs/015-runtime-refoundation/spec.md)
**Input**: Feature specification from `/specs/015-runtime-refoundation/spec.md`

## Summary

Refound Synod's primary delivery path around session-native runtime control instead of fixture-shaped replay. Planning will derive a bounded task draft from captured goals, workspace evidence, authored inputs, and available Canon artifacts; execution will choose explicit next decisions from live state through a bounded observe-decide-act-verify-update loop; flow will become an operator-confirmed policy surface; compatibility execution remains explicit rather than implicit; and status or inspection output will explain route choice, decision history, recovery, and terminal outcome.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs  
**Storage**: Workspace-local `.synod/session.json`, `.synod/traces/`, optional `.synod/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout  
**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo deny check licenses advisories bans sources`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential session-owned planning and bounded one-decision-at-a-time execution with explicit recovery, replanning, and compatibility routing  
**Observability Surface**: Active session record, persisted execution traces, route-aware `status` and `next`, and decision-aware `inspect` output  
**Performance Goals**: Bounded task draft derivation and flow proposal stay under 5 seconds for workspaces with up to 1000 files; route resolution stays negligible relative to execution time; inspection remains fast enough for operator diagnosis inside one CLI round-trip  
**Constraints**: No silent flow auto-confirmation, no implicit fallback to compatibility when session-native state is sufficient, no Canon-owned per-action control flow, no parallel execution, no speculative new top-level runtime surfaces, preserve explicit compatibility behavior, ship as crate version `0.15.0`  
**Scale/Scope**: One active session per workspace, bounded local engineering tasks, existing built-in flows only, docs and assistant assets updated to make the refounded path the dominant product story

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering delivery by making session-native planning and live-state execution the primary path rather than a declarative profile replay. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/synod/specs/015-runtime-refoundation/spec.md).
- **PASS** Delivery-first scope: The work is about execution control, planning, recovery, routing, validation, and inspectability before documentation polish or secondary ergonomics. See Summary and Constraints.
- **PASS** Bounded execution: Start conditions, explicit route selection, max steps or retries, and terminal states remain first-class runtime behavior for both success and non-success paths. See Technical Context, research decisions, quickstart scenarios, and [spec.md](/Users/rt/workspace/synod/specs/015-runtime-refoundation/spec.md).
- **PASS** Stateful execution: The runtime persists bounded task drafts, decision history, flow constraint state, routing outcome, and terminal evidence in session and trace surfaces. See Summary, data-model, and contracts.
- **PASS** Mutable planning: The initial bounded task draft remains explicit and later recovery or replanning decisions mutate runtime intent through inspectable state transitions rather than hidden heuristics. See Summary, research, and data-model.
- **PASS** Sequential-first design: Execution remains one bounded decision at a time; no concurrency, background workers, or hidden fan-out are introduced. See Execution Model and Constraints.
- **PASS** Tool-agent symmetry: Reasoning, file mutation, validation, and route selection all remain explicit and observable through decision and evidence records rather than hiding action behind text-only planning. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Route choice, bounded task draft content, decision selection, failure evidence, recovery decisions, and terminal reasoning stay visible through session and trace output. See Observability Surface, quickstart, and contracts.
- **PASS** Non-goals and external separation: Canon remains a bounded planning or stage-boundary input; provider expansion, distributed execution, UI, long-term memory, and review councils stay out of scope. See Constraints and [spec.md](/Users/rt/workspace/synod/specs/015-runtime-refoundation/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is a single coherent refoundation where the primary session path plans and runs from live state without requiring init-first or fixture-first mental models. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/015-runtime-refoundation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── decision-runtime-contract.md
│   └── routing-governance-boundary-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── adapters/
│   ├── agent.rs
│   ├── governance_runtime.rs
│   ├── session_store.rs
│   ├── tool.rs
│   └── trace_store.rs
├── domain/
│   ├── decision.rs
│   ├── flow.rs
│   ├── flow_policy.rs
│   ├── goal_plan.rs
│   ├── session.rs
│   ├── tool_result.rs
│   └── trace.rs
├── fixture.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── flow_inference.rs
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── registry/
    ├── agent_registry.rs
    └── tool_registry.rs

assistant/
├── README.md
├── claude/
├── codex/
└── copilot/

docs/
README.md
ROADMAP.md

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the work inside the existing CLI, orchestrator, domain, adapter, registry, docs, and assistant-asset surfaces. No new top-level runtime or service is introduced. The refoundation should replace the dominant control model inside the current crate, while the final rollout also updates README, ROADMAP, docs, templates, and examples so the operator story matches runtime behavior.

## Complexity Tracking

No constitution violations are expected for this slice.
