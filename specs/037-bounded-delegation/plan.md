# Implementation Plan: Bounded Delegated Execution

**Branch**: `037-bounded-delegation` | **Date**: 2026-05-03 | **Spec**: [/Users/rt/workspace/synod/specs/037-bounded-delegation/spec.md](/Users/rt/workspace/synod/specs/037-bounded-delegation/spec.md)
**Input**: Feature specification from `/specs/037-bounded-delegation/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extend Synod's session-native delivery path so routed execution is shaped by
declared runtime capability profiles and explicit effort policy rather than by
implicit backend assumptions. Persist bounded handoff and escalation packets in
existing session-owned state, detect when delegated continuity is stuck or
obsolete from explicit evidence, and project the resulting continuity story
through the existing `config show`, `run`, `status`, `next`, and `inspect`
surfaces. Keep compatibility follow-up explicit and subordinate, keep execution
sequential-first, and ship the slice as `0.37.0` with release closure and >95%
coverage for modified Rust files.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, and `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.synod/session.json`, `.synod/config.toml`, optional `.synod/workflows.toml`, persisted traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json`, task-context state embedded in session tasks, and repository-managed docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, targeted unit, integration, and contract tests, `cargo test --no-run --all-targets`, `cargo nextest run --workspace --all-features`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential session-native planning plus bounded observe -> decide -> act -> verify execution where runtime capability and effort policy may redirect continuity into explicit handoff or escalation packets without introducing hidden concurrency  
**Observability Surface**: Persisted configuration, goal-plan/session/task-context state, decision-oriented traces under `.synod/traces/`, CLI summaries on `config show`, `run`, `status`, `next`, and `inspect`, plus release docs and assistant guidance that explain delegated continuity, packet state, and stuck detection  
**Performance Goals**: Operators should recover the decisive capability rule, active delegation packet, and stuck reason from normal CLI output in under 2 minutes; blocked runs should stop with an explicit bounded reason on first blocked boundary rather than after repeated opaque retries; maintainers should complete release validation for the slice in under 20 minutes  
**Constraints**: No new top-level runtime, no tmux or mailbox substrate, no distributed or parallel execution, no background workers, no generic long-term memory subsystem, no provider-abstraction refoundation, no Canon-owned control flow, and explicit compatibility follow-up remains subordinate and trace-authoritative  
**Scale/Scope**: One workspace or registered cluster at a time, bounded by existing session/run limits, with one active delegation continuity story per current bounded goal and explicit packet history for superseded or resolved continuity

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by making route capability, effort policy, and continuity ownership explicit before and during execution. See Summary and Technical Context.
- **PASS** Delivery-first scope: The plan prioritizes execution control, continuity state, stuck recovery, and inspectability ahead of optimization or polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`; explicit compatibility follow-up remains available but subordinate and trace-authoritative. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The design keeps existing step and retry limits, adds explicit handoff and escalation packets, and treats stuck delegation as an explicit bounded stop or recovery path with no hidden background activity. See Technical Context, research, and contracts.
- **PASS** Stateful execution: Runtime capability policy, delegation packets, and stuck evidence remain persisted in existing configuration, session, task-context, and trace state rather than transient runtime flags. See Summary, data model, and contracts.
- **PASS** Mutable planning: Capability-aware routing and delegation packets can influence planning, replanning, and later decision updates while keeping continuity changes traceable to explicit evidence. See Summary, research, and data model.
- **PASS** Sequential-first design: Planning, decision selection, delegation, escalation, supersession, and stuck handling remain one-step-at-a-time state transitions with no concurrency or implicit fan-out. See Technical Context and quickstart.
- **PASS** Tool-agent symmetry: Delegated reasoning remains visible through explicit route policy, handoff or escalation packets, and bounded next-action selection rather than hidden heuristics. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Capability selection, effort policy, packet lineage, stuck markers, and supersession reasons are surfaced through traces and existing CLI summaries. See Technical Context, contracts, and quickstart.
- **PASS** Non-goals and external separation: The slice does not depend on Canon or any external runtime to own Synod control flow and does not reintroduce councils, provider abstraction refoundation, long-term memory, UI work, or deployment pipelines. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is making blocked or redirected delivery continuity explicit through one session-owned delegation model informed by declared capability and effort policy. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/037-bounded-delegation/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── runtime-capability-surface-contract.md
│   ├── delegation-packet-lifecycle-contract.md
│   └── delegated-follow-through-contract.md
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
│   └── session.rs
├── domain/
│   ├── configuration.rs
│   ├── follow_through.rs
│   ├── goal_plan.rs
│   ├── routing_decision.rs
│   ├── session.rs
│   ├── task_context.rs
│   └── trace.rs
├── orchestrator/
│   ├── decision_loop.rs
│   ├── goal_planner.rs
│   ├── session_runtime.rs
│   └── governance.rs
└── lib.rs

tests/
├── contract/
├── integration/
└── unit/

README.md
ROADMAP.md
CHANGELOG.md
Cargo.toml
Cargo.lock
assistant/
docs/
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing configuration,
goal-plan, session, task-context, decision-loop, follow-through, and CLI
read-side surfaces. No new top-level runtime or persistence system is needed
because 037 strengthens how the session-owned path models continuity boundaries
and route declarations rather than introducing a second orchestration plane.

## Complexity Tracking

No constitution violations are expected for this slice.
