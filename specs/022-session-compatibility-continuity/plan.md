# Implementation Plan: Session And Compatibility Continuity

**Branch**: `022-session-compatibility-continuity` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/boundline/specs/022-session-compatibility-continuity/spec.md](/Users/rt/workspace/boundline/specs/022-session-compatibility-continuity/spec.md)
**Input**: Feature specification from `/specs/022-session-compatibility-continuity/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Tighten Boundline's operator continuity story after explicit compatibility runs by making `status`, `next`, and `inspect` resolve follow-up state from the existing active session and latest workspace trace without blurring route ownership. The slice stays intentionally bounded: session-native execution remains the primary route, compatibility execution remains explicit, and the runtime reuses persisted traces and session state rather than introducing a new orchestration engine or background reconciler. The release closes as `0.22.0` with updated docs, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and path APIs; no new runtime dependencies planned for this slice  
**Storage**: Existing workspace-local `.boundline/session.json` and `.boundline/traces/` remain authoritative; no new persistence surface is planned unless research proves existing state cannot express continuity safely  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted contract, integration, and unit coverage for compatibility follow-up continuity, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state  
**Execution Model**: Sequential execution with session-native orchestration as the primary route and explicit compatibility execution as a bounded alternate route; follow-up commands derive continuity from persisted session and trace evidence  
**Observability Surface**: `run`, `status`, `next`, `inspect`, persisted session records, persisted traces, routing summaries, terminal conditions, and shared adaptive/review/governance summary lines across route boundaries  
**Performance Goals**: Developers should identify the authoritative follow-up state and correct next command after a compatibility run from CLI output in under 2 minutes; compatibility follow-up checks should reuse existing persisted state in one normal CLI round-trip  
**Constraints**: No hidden promotion of compatibility execution into the primary session route, no background reconciliation worker, no Canon-owned follow-up decisions, no open-ended trace search, no new top-level runtime surface, and crate version must bump to `0.22.0` with docs and changelog updates  
**Scale/Scope**: One workspace at a time, one active session record at a time, one latest workspace trace considered for compatibility continuity, and release updates limited to touched runtime, docs, and test surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: Explain how this feature directly improves bounded engineering task delivery.
- Delivery-first scope: Confirm execution, orchestration, decomposition, or validation work is prioritized ahead of optimization or polish.
- Primary workflow: State whether the main operator path is session-native (`goal -> plan -> run -> status -> next -> inspect`) and identify any explicit compatibility path that remains available.
- Bounded execution: Identify explicit start conditions, terminal conditions, and max step or retry limits.
- Stateful execution: Describe shared task context, read and write points, and justify any stateless segment.
- Mutable planning: Describe initial planning plus replanning, step insertion, or replacement behavior.
- Sequential-first design: Confirm one-step-at-a-time execution or justify why the constitution allows an exception.
- Tool-agent symmetry: Show how reasoning and action remain explicit in the execution model.
- Observability and explicit intelligence: List trace surfaces, visible decisions, failure signals, and any heuristic behavior that must be exposed.
- Non-goals and external separation: Confirm the plan does not depend on Canon behavior beyond bounded governance/evidence boundaries or reintroduce deferred scope such as councils or voting outside an explicitly reprioritized, bounded review slice, provider abstraction complexity beyond the approved slice, long-term memory, UI/UX, or deployment pipelines.
- Minimal slice: Explain the smallest independently valuable capability delivered by this plan.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

- **PASS** Delivery identity: The slice directly improves bounded task delivery by making post-run follow-up state explicit after operators intentionally choose the compatibility route. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/022-session-compatibility-continuity/spec.md).
- **PASS** Delivery-first scope: The work prioritizes orchestration continuity, route ownership, and follow-up inspectability before polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: Session-native remains the main operator path, while compatibility remains an explicit alternate route whose follow-up semantics become clearer in this slice. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: The slice keeps explicit terminal states, bounded use of persisted session and latest workspace trace, and no hidden background progression. See Technical Context, data model, and quickstart.
- **PASS** Stateful execution: The design reuses persisted session state and trace state as the continuity inputs for later commands. See Summary, Technical Context, data model, and contracts.
- **PASS** Mutable planning: The slice does not introduce a new planner; it clarifies which persisted route evidence later commands should trust after explicit compatibility execution. See Summary, research, and contracts.
- **PASS** Sequential-first design: One command resolves one bounded continuity state at a time; no concurrency or background workers are introduced. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/022-session-compatibility-continuity/spec.md).
- **PASS** Tool-agent symmetry: The feature keeps reasoning and action explicit through visible CLI summaries and trace-derived follow-up decisions rather than hidden fallback behavior. See Summary, research, quickstart, and contracts.
- **PASS** Observability and explicit intelligence: Routing, authoritative state, continuity mode, terminal conditions, and shared adaptive/review/governance summaries remain visible to developers. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The slice does not depend on Canon for control flow, does not broaden adaptive mutation power, and does not add provider abstraction, UI, or distributed execution work. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/022-session-compatibility-continuity/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is a coherent post-compatibility follow-up story across `status`, `next`, and `inspect` using existing persisted state. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/022-session-compatibility-continuity/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── compatibility-follow-up-command-contract.md
│   ├── continuity-authority-contract.md
│   └── shared-route-summary-contract.md
└── tasks.md
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Keep the structure minimal, delivery-focused, and sequential-
  first. Do not introduce extra top-level projects or UI/runtime surfaces unless
  the Constitution Check explicitly justifies them.
-->

```text
src/
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   └── session.rs
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
├── getting-started.md
└── adaptive-execution.md
```

**Structure Decision**: Keep the slice inside the existing CLI, session-domain, trace-summary, docs, and test harness surfaces. No new top-level runtime or persistence area is justified because the feature's value comes from clearer interpretation of already persisted state rather than a broader execution mode.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
