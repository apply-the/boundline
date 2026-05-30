# Implementation Plan: Goal Negotiation And Constraint Modeling

**Branch**: `026-goal-constraint-modeling` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/boundline/specs/026-goal-constraint-modeling/spec.md](/Users/rt/workspace/boundline/specs/026-goal-constraint-modeling/spec.md)
**Input**: Feature specification from `/specs/026-goal-constraint-modeling/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add one explicit negotiated delivery packet to the existing session-native
`goal -> plan -> run -> status -> next -> inspect` story. The slice derives
acceptance boundaries, scope limits, binding constraints, and tradeoff summaries
from the recorded goal plus authored inputs, blocks planning when negotiation is
not credible, carries the negotiated story into follow-up and inspection
surfaces, keeps explicit compatibility behavior visibly separate, and closes as
`0.26.0` with version bump, impacted docs, changelog, coverage refresh for
modified Rust files, clippy cleanup, and formatting.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, task-context state embedded in persisted session tasks, optional cluster projection in primary-workspace session state, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted unit, integration, and contract coverage for negotiated capture, planning gate, and follow-up rendering, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, and `cargo nextest run --workspace --all-features`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native orchestration with bounded retries and replans; negotiation runs during capture, planning stays explicit, and compatibility follow-up remains a separate named route rather than a second control loop  
**Observability Surface**: Persisted session state, task context, execution traces, CLI `goal`, `plan`, `run`, `status`, `next`, and `inspect` summaries, plus assistant/operator documentation that explains acceptance boundaries, binding constraints, and tradeoff decisions  
**Performance Goals**: Operators should identify the negotiated outcome and binding constraints from capture or follow-up output in under 2 minutes; representative ambiguous requests must stop before plan confirmation 100% of the time; maintainers should validate the `0.26.0` story from docs plus runtime output in under 20 minutes  
**Constraints**: Session-native remains the primary path; no new standalone negotiation runtime or background loop; no Canon-owned planning control flow; no hidden compatibility authority; goal-only sessions must remain supported through explicit defaults; cluster sessions, when present, keep packet authority in the primary workspace  
**Scale/Scope**: One bounded session at a time, representative goal-only and authored-brief flows, explicit success plus non-success negotiation states, and focused code changes in existing session, plan, trace, CLI, test, and documentation surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making the goal-to-plan boundary explicit and challengeable before code mutation begins. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/026-goal-constraint-modeling/spec.md).
- **PASS** Delivery-first scope: The plan centers on capture, planning, follow-up authority, and validation surfaces first; release polish is a closeout phase, not the feature core. See Summary and Technical Context.
- **PASS** Primary workflow: Session-native remains the main operator path, while explicit compatibility behavior stays visible and separate instead of becoming an implicit negotiation authority. See Summary, Technical Context, research, and quickstart.
- **PASS** Bounded execution: Negotiation happens once during capture, planning stays blocked on unresolved ambiguity, and the slice does not introduce open-ended loops or hidden background reasoning. See Technical Context, research, data model, and quickstart.
- **PASS** Stateful execution: Negotiation output is persisted in session state, projected into task context and traces when execution proceeds, and reused by follow-up surfaces. See Summary, Technical Context, research, and data model.
- **PASS** Mutable planning: The plan preserves the current plan/replan behavior while making the acceptance boundary and active constraints inspectable before and after any replanning step. See Summary, research, and data model.
- **PASS** Sequential-first design: One session, one active step, and one negotiation authority remain live at a time; the slice explicitly rejects background negotiation loops or distributed control flow. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/026-goal-constraint-modeling/spec.md).
- **PASS** Tool-agent symmetry: Reasoning stays explicit through negotiated summaries, constraints, and tradeoff traces, while action remains in the existing capture/plan/run flow rather than being hidden behind a new opaque subsystem. See Summary, research, contracts, and quickstart.
- **PASS** Observability and explicit intelligence: Negotiation packet creation, blocking constraints, selected tradeoffs, and route ownership remain visible through session state, traces, and CLI summaries. See Technical Context, research, quickstart, and contracts.
- **PASS** Non-goals and external separation: The slice does not require Canon to negotiate plans, does not introduce councils or voting, does not expand provider abstraction, and does not add UI, deployment, or long-term memory surfaces. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/026-goal-constraint-modeling/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one explicit negotiated delivery packet that gates planning and remains visible through existing follow-up surfaces. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/026-goal-constraint-modeling/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── negotiated-goal-surface-contract.md
│   ├── constraint-follow-up-surface-contract.md
│   └── compatibility-negotiation-boundary-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   └── session.rs
├── domain/
│   ├── brief.rs
│   ├── goal_plan.rs
│   ├── negotiation.rs
│   ├── session.rs
│   ├── task.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── goal_planner.rs
│   └── session_runtime.rs
└── lib.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/

assistant/
└── README.md

docs/
├── configuration.md
└── getting-started.md

README.md
CONTRIBUTING.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing session, goal-plan,
trace, CLI rendering, and documentation surfaces. The only new source file
expected is a delivery-focused domain module for negotiation state. No new
top-level runtime, persistence area, or operator surface is justified because
the feature extends the current local session story rather than creating a
second planning subsystem.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
