# Implementation Plan: Human-Facing Brief Ingestion

**Branch**: `010-human-brief-ingestion` | **Date**: 2026-04-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-human-brief-ingestion/spec.md`

## Summary

Allow developers to start Boundline work from plain text and Markdown briefs by extending the existing session-oriented `goal` and `run` entry points with human-facing inputs, workspace-bounded source resolution, explicit clarification when the brief is not credible, and optional governance intent expressed in business terms. The smallest shippable slice keeps the current manifest-driven execution profile as an advanced automation path, normalizes human-authored input into one inspectable brief bundle persisted with session and task state, reuses existing `status`, `next`, `inspect`, and trace surfaces for provenance, and maps high-level governance intent into the existing governance runtime abstraction without asking the user to author JSON manifests or stage wiring.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`) plus Rust standard library filesystem, path, and collections APIs; no new runtime dependencies for the first human-input slice  
**Storage**: Workspace-local `.boundline/session.json`, persisted execution traces under `<workspace>/.boundline/traces/`, existing `<workspace>/.boundline/execution.json` with legacy fallback to `<workspace>/.boundline/fixture.json` for advanced automation, and optional Canon-managed artifacts under `<workspace>/.canon/` when governed execution is selected  
**Testing**: `cargo test --all-targets`, focused contract, integration, and unit coverage for CLI validation, input normalization, session persistence, trace and status projections, governance intent mapping, `cargo fmt --check`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`  
**Target Platform**: macOS and Linux developer workstations, Linux CI, and assistant-driven VS Code sessions operating inside one repository workspace  
**Project Type**: Single Rust CLI crate with file-backed session and trace persistence plus repository-managed assistant assets  
**Execution Model**: Sequential session lifecycle (`start -> capture -> flow -> plan -> step/run`) with one active workspace session, one normalized authored brief bundle per active task, workspace-bounded Markdown resolution, one open clarification at a time, and explicit stop before planning when no credible bounded task can be derived  
**Observability Surface**: Session `status` and `next` output, `inspect` summaries, run trace events, persisted task context state, explicit CLI validation errors naming offending sources, and governance status projections when human input requests governed execution  
**Performance Goals**: Keep human-input normalization interactive for normal CLI use by resolving, deduplicating, and summarizing a small authored bundle of direct text plus up to 10 Markdown sources in a single command round-trip before planning begins  
**Constraints**: No new human-authored JSON or manifest requirement, text and Markdown only in the first slice, workspace-only source resolution, preservation of the manifest-driven automation path, Canon remaining optional, explicit precedence and deduplication, clarification limited to missing business context, and no background workers or new top-level runtimes  
**Scale/Scope**: One active session per workspace, one bounded brief bundle persisted with the active task, up to 10 Markdown inputs or references per invocation, built-in `bug-fix`, `change`, and `delivery` flows only, and at most two clarification turns before explicit stop

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Delivery identity: PASS. The feature directly improves bounded engineering task delivery by removing manifest authoring from the first operator touchpoint while still feeding the existing planner and executor. See Summary and Technical Context.
- Delivery-first scope: PASS. The plan prioritizes input capture, bounded normalization, clarification, governance mapping, and inspection surfaces ahead of polish. See Summary, Technical Context, and Project Structure.
- Bounded execution: PASS. Human input is captured only for one active session, normalized once per invocation, limited to workspace Markdown, and may require at most two clarification turns before explicit stop. See Technical Context and research decisions.
- Stateful execution: PASS. Accepted inputs, provenance, clarification state, and derived task data are persisted through the existing session record and task context so later `plan`, `status`, `next`, `inspect`, and `run` commands can continue without restating the brief. See Technical Context and data model.
- Mutable planning: PASS. The feature preserves current flow selection, planning, and replanning behavior while allowing the plan seed to change when clarification resolves missing context or conflicting sources. See Summary and research decisions.
- Sequential-first design: PASS. There is still one active session, one active task, one active clarification, and one stage executing at a time; the feature adds no hidden concurrency or background ingestion workers. See Technical Context.
- Tool-agent symmetry: PASS. Human input capture, workspace file resolution, clarification, governance mapping, and later execution remain explicit commands and state transitions rather than hidden heuristics. See Summary and contracts.
- Observability and explicit intelligence: PASS. Input provenance, deduplication order, clarification blocks, governance intent, and derived next actions are surfaced through existing status, inspect, and trace contracts instead of raw logs or hidden state. See Technical Context and contracts.
- Non-goals and external separation: PASS. Canon stays optional behind the existing governance runtime abstraction, the plan does not introduce a chat UI, long-term memory, councils, or deployment automation, and manifest-driven automation remains available without defining the default human path. See Summary and Technical Context.
- Minimal slice: PASS. The smallest independently valuable capability is starting `goal` or `run` from plain text and Markdown briefs, preserving source provenance, asking for missing business context explicitly, and optionally carrying human governance intent into the existing governed runtime. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/010-human-brief-ingestion/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── goal-run-cli-contract.md
│   ├── human-governance-intent-contract.md
│   └── session-input-observability-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── adapters/
│   └── governance_runtime.rs
├── cli/
│   ├── output.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── execution.rs
│   ├── governance.rs
│   ├── session.rs
│   ├── task.rs
│   └── task_context.rs
├── fixture.rs
├── orchestrator/
│   ├── governance.rs
│   └── session_runtime.rs
└── cli.rs

tests/
├── contract/
├── integration/
├── support/
└── unit/
```

**Structure Decision**: Keep the feature inside the existing crate and extend the current CLI, session runtime, task context, governance adapter, and inspection surfaces rather than introducing a second ingest service or a second workspace manifest. Any normalization helper added for the slice should live under the existing `cli/`, `orchestrator/`, or `domain/` structure so human-facing input remains one more way to seed the same bounded delivery loop.

## Complexity Tracking

No constitution violations require justification for this slice.
