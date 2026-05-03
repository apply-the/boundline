# Implementation Plan: Governed Stage Depth

**Branch**: `020-governed-stage-depth` | **Date**: 2026-05-01 | **Spec**: [/Users/rt/workspace/boundline/specs/020-governed-stage-depth/spec.md](/Users/rt/workspace/boundline/specs/020-governed-stage-depth/spec.md)
**Input**: Feature specification from `/specs/020-governed-stage-depth/spec.md`

## Summary

Deepen Boundline's bounded governance story by making governed `bug-fix:investigate` a credible session-native stage ahead of the existing governed `verify` path, preserving explicit packet reuse and approval-refresh behavior across those two transitions, and projecting the same governance guidance through direct session and named-workflow surfaces. The slice stays intentionally narrow: it exercises existing governance primitives on a second bounded stage, strengthens inspectability and blocked-state guidance, and closes as crate version `0.20.0` with release-aligned docs, changelog, coverage refresh for modified Rust files, clippy cleanup, and formatting.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem, path, process, and collections APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/execution.json`, optional `.canon/` artifacts, and release-aligned repository docs plus assistant assets  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --no-run --all-targets`, targeted `cargo test` suites for governed-stage session and workflow surfaces, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, plus repository-standard `cargo nextest run --workspace --all-features` and `cargo deny check licenses advisories bans sources` during release closeout  
**Target Platform**: macOS/Linux developer workstations and Linux CI  
**Project Type**: Single Rust CLI/library crate with workspace-local persisted execution state  
**Execution Model**: Sequential session-owned execution with one active step at a time, bounded governance overlays at declared stage boundaries, explicit refresh on later commands, and no hidden background progression  
**Observability Surface**: Persisted session record, execution traces, `run`, `status`, `next`, `inspect`, workflow-aware status and resume surfaces, governance packet provenance fields, approval state, blocked reason, and next-command guidance  
**Performance Goals**: Governed-stage status and refresh remain within one normal CLI round-trip; representative governed bug-fix scenarios complete or stop explicitly without manual session edits; maintainers can configure the deeper governed slice from shipped docs in under 15 minutes  
**Constraints**: No Canon-owned orchestration; no generic governance graph, branching, loops, hidden background progression, or new workflow phases; packet reuse remains bounded to packet references, readiness, headlines, and lineage metadata; crate version must bump to `0.20.0`; README, docs, roadmap, changelog, contributing guidance, and assistant docs must be refreshed for the delivered slice  
**Scale/Scope**: One active workspace session, one bounded bug-fix flow at a time, governed depth limited to adding `bug-fix:investigate` ahead of the existing governed `verify` story, and release updates limited to the touched runtime and guidance surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice deepens real bounded delivery work by letting a bug-fix session carry governance into `investigate` before later verify work, rather than widening Boundline into a new platform. See Summary, Technical Context, and [spec.md](/Users/rt/workspace/boundline/specs/020-governed-stage-depth/spec.md).
- **PASS** Delivery-first scope: The work prioritizes execution, governance visibility, packet lineage, and next-step guidance before polish. See Summary, Technical Context, and research decisions.
- **PASS** Primary workflow: The main operator path remains session-native `start -> capture -> plan -> run -> status -> next -> inspect`; the named-workflow layer remains a bounded projection over the same session state, and the compatibility path stays explicit. See Summary, Technical Context, quickstart, and contracts.
- **PASS** Bounded execution: Governance still starts only at declared stage boundaries, stops at the first unmet approval or packet-readiness condition, and never advances hidden work in the background. See Technical Context, data model, research, and quickstart.
- **PASS** Stateful execution: Governed stage state, packet references, reuse lineage, and refreshed approval outcomes remain persisted in `.boundline/session.json` and `.boundline/traces/`. See Summary, data model, and contracts.
- **PASS** Mutable planning: The slice reuses the current goal-plan and stage-boundary governance primitives without introducing a second runtime; later commands may refresh state before the next step resumes. See Summary, research, and data model.
- **PASS** Sequential-first design: One active session step remains in control at a time, with governance acting as an explicit stop or continue boundary. See Technical Context, research, and [spec.md](/Users/rt/workspace/boundline/specs/020-governed-stage-depth/spec.md).
- **PASS** Tool-agent symmetry: Governance mode selection, packet reuse, refresh, and blocked outcomes remain explicit task actions and trace events rather than hidden heuristics. See Summary, research, and contracts.
- **PASS** Observability and explicit intelligence: Stage key, selected mode, packet provenance, approval state, blocked reason, and refresh outcomes remain visible in session, workflow, and trace surfaces. See Technical Context, quickstart, and contracts.
- **PASS** Non-goals and external separation: The slice does not give Canon orchestration ownership, does not widen into a general governance engine, and does not introduce deferred scope such as UI, long-term memory, or distributed execution. See Constraints, research, and [spec.md](/Users/rt/workspace/boundline/specs/020-governed-stage-depth/spec.md).
- **PASS** Minimal slice: The smallest independently valuable capability is one deeper governed bug-fix slice that adds `investigate` governance ahead of the existing verify path while improving packet reuse and refresh guidance. See Summary and research.

## Project Structure

### Documentation (this feature)

```text
specs/020-governed-stage-depth/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── governed-stage-command-surface-contract.md
│   ├── governed-stage-refresh-contract.md
│   └── governance-profile-guidance-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── cli.rs
├── cli/
│   ├── inspect.rs
│   ├── output.rs
│   ├── session.rs
│   └── workflow.rs
├── domain/
│   ├── governance.rs
│   └── session.rs
├── orchestrator/
│   ├── governance.rs
│   └── session_runtime.rs
└── lib.rs

assistant/
└── README.md

docs/
├── configuration.md
└── getting-started.md

tests/
├── contract/
├── integration/
└── support/
```

**Structure Decision**: Keep the slice inside the existing governance domain, session runtime, CLI projection, docs, and test-fixture surfaces. No new top-level runtime or project boundary is justified because the feature deepens an existing stage-boundary governance path rather than creating a second control plane.

## Complexity Tracking

No constitution violations are expected for this slice.
