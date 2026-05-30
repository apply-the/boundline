# Implementation Plan: Workflow Follow-Through

**Branch**: `019-workflow-follow-through` | **Date**: 2026-05-01 | **Spec**: [specs/019-workflow-follow-through/spec.md](specs/019-workflow-follow-through/spec.md)
**Input**: Feature specification from `/specs/019-workflow-follow-through/spec.md`

## Summary

Complete the first named-workflow slice by making bounded review and govern phases executable from the `boundline workflow` surface, adding an operator-facing workflow discovery story, and shipping clear registry authoring guidance without widening Boundline into a generic workflow engine. The implementation will keep the session-native route authoritative, extend workflow progression and output summaries through review and governance outcomes, add a bounded workflow discovery surface for operators and assistants, preserve the direct session-native and explicit compatibility paths, and close the slice as crate version `0.19.0` with documentation, roadmap, changelog, assistant guidance, and validation updates.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/workflows.toml`, `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, plus repository docs and assistant assets updated as part of rollout  
**Testing**: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted `cargo test` suites for touched workflow surfaces, `cargo nextest run --workspace --all-features`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `cargo deny check licenses advisories bans sources`  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential session-owned workflow progression with one active phase at a time, explicit review and governance follow-through, and no hidden background advancement  
**Observability Surface**: Persisted session record, execution traces, workflow-aware `run`, `status`, `next`, `resume`, `inspect`, bounded workflow discovery output, and assistant-facing docs built on the same session projection story  
**Performance Goals**: Workflow discovery and validation remain under 1 second for typical local workspaces; workflow status or resume remains within one normal CLI round-trip before underlying execution continues; maintainers can author a representative review or govern workflow from shipped guidance in under 15 minutes  
**Constraints**: No generic workflow-engine expansion; no branching, loops, fan-out, fan-in, hidden concurrency, or background progression; no Canon-owned orchestration; no silent override of direct session-native or explicit compatibility routes; crate version must bump to `0.19.0`; README, docs, roadmap, changelog, and assistant docs must be refreshed as part of the slice  
**Scale/Scope**: One active named workflow per workspace session, bounded local engineering tasks, workflow discovery scoped to workspace-local registry definitions, and documentation updates limited to the new workflow follow-through behavior

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice directly improves bounded engineering-task delivery by letting named workflows carry real review and governance work to completion instead of stopping short at declaration-only blockers. See Summary, Technical Context, and [spec.md](specs/019-workflow-follow-through/spec.md).
- **PASS** Delivery-first scope: The work prioritizes orchestration, execution follow-through, operator guidance, and inspectability before optimization or polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The primary operator path remains session-native and workflow phases still compile onto `goal -> plan -> run -> status -> next -> inspect`; the explicit compatibility route remains available only when the operator chooses it. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Start conditions, stop conditions, blocked governance, review outcomes, and existing runtime limits stay explicit; follow-through stops at the first unmet bounded condition. See Technical Context, data model, research, and quickstart.
- **PASS** Stateful execution: Workflow follow-through remains session-owned, with progress, blocked reasons, and next actions persisted in `.boundline/session.json` and trace evidence preserved in `.boundline/traces/`. See Summary, data-model, and contracts.
- **PASS** Mutable planning: Workflow follow-through reuses the existing mutable session-native planning and runtime control plane, so review or governance work can pause, fail, or continue based on explicit evidence rather than a rigid script. See Summary, research, and data model.
- **PASS** Sequential-first design: The design keeps one active workflow phase at a time and rejects hidden concurrency or generic workflow branching. See Technical Context, research, and [spec.md](specs/019-workflow-follow-through/spec.md).
- **PASS** Tool-agent symmetry: Review, governance, and workflow discovery remain visible as explicit runtime or operator actions rather than hidden orchestration magic. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Workflow identity, phase, routing, execution condition, discovery guidance, blocked reasons, and review or govern outcomes all remain visible through session, trace, and CLI surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The plan does not depend on Canon beyond bounded governance or evidence behavior and does not introduce deferred scope such as generic workflow DSL semantics, provider-routing expansion, UI, long-term memory, or distributed execution. See Constraints, research, and [spec.md](specs/019-workflow-follow-through/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one bounded workflow follow-through slice that executes review and govern, adds operator workflow discovery, and documents supported registry authorship. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/019-workflow-follow-through/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── workflow-discovery-contract.md
│   ├── workflow-follow-through-command-contract.md
│   └── workflow-registry-guidance-contract.md
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
│   ├── session.rs
│   └── workflow.rs
├── domain/
│   ├── goal_plan.rs
│   ├── session.rs
│   └── workflow.rs
├── orchestrator/
│   └── session_runtime.rs
├── lib.rs
└── bin/

assistant/
├── README.md
├── claude/
├── codex/
└── copilot/

docs/
├── configuration.md
└── getting-started.md

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Keep the feature entirely inside the existing crate, CLI, domain, orchestrator, docs, assistant-asset, and test surfaces. No new top-level runtime or project boundaries are justified because workflow follow-through extends the session-native control plane rather than creating a second engine.

## Complexity Tracking

No constitution violations are expected for this slice.
