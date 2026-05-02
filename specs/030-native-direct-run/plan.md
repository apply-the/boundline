# Implementation Plan: Native Direct Run

**Branch**: `030-native-direct-run` | **Date**: 2026-05-02 | **Spec**: [/Users/rt/workspace/synod/specs/030-native-direct-run/spec.md](/Users/rt/workspace/synod/specs/030-native-direct-run/spec.md)
**Input**: Feature specification from `/specs/030-native-direct-run/spec.md`

**Note**: This plan narrows Feature 030 to the highest-leverage product gap
still visible in the current CLI: direct `run --goal` still defaults to the
explicit compatibility path. The slice makes direct run native-first, keeps
compatibility deliberate and explicit, protects meaningful active session state,
and closes as `0.30.0` with tests, docs, coverage, clippy, and formatting.

## Summary

Make direct `synod run --workspace <workspace> --goal <goal>` bootstrap and
execute the existing native goal-plan route by default instead of the explicit
compatibility path. The slice stays inside the current CLI, session, trace,
and decision-loop surfaces: it introduces an explicit compatibility opt-in for
operators who still want the execution-profile route, avoids silent session
overwrite, removes execution-profile diagnostics as a prerequisite for native
direct run, and ships as `0.30.0` with release closeout and touched-Rust
coverage above 95%.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.95.0, edition 2024  
**Primary Dependencies**: Existing runtime dependencies `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, plus Rust standard library filesystem and process APIs; no new runtime dependencies planned for this slice  
**Storage**: Workspace-local `.synod/session.json`, persisted execution traces under `<workspace>/.synod/traces/`, optional `.synod/execution.json` for explicit compatibility execution, optional `.synod/workflows.toml`, optional cluster state under `.synod/cluster.toml`, and repository-managed assistant assets under `assistant/`  
**Testing**: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, focused unit/contract/integration tests for direct run routing and session safety, `cargo test --no-run --all-targets`, and `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info` with touched-file coverage review  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Single Rust CLI/library crate with file-backed session and trace state plus repository-managed assistant command packs  
**Execution Model**: Sequential session-native orchestration with bounded decision-loop execution; direct run becomes a native bootstrap entry while explicit compatibility remains opt-in and subordinate  
**Observability Surface**: Persisted session status, execution traces, CLI `run`, `status`, `next`, and `inspect` output, route ownership, `execution_path`, negotiation projection, decision history, compatibility follow-up, and assistant command-pack guidance  
**Performance Goals**: Operators should reach a persisted native run outcome from one direct `run --goal` command in under 2 minutes of operator time; maintainers should validate the release story in under 20 minutes  
**Constraints**: No new background workers, no distributed orchestration, no hidden fallback from native to compatibility, no Canon dependency for the feature to function, no UI work, and no broadened autonomous coding engine  
**Scale/Scope**: One workspace at a time for the direct native bootstrap path, representative success and non-success Rust workspaces, and bounded updates to existing CLI, session runtime, diagnostics, tests, docs, and assistant surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The slice improves bounded engineering-task delivery by letting the primary `run --goal` surface enter the real native delivery path instead of a subordinate compatibility route. See Summary, Technical Context, and [/Users/rt/workspace/synod/specs/030-native-direct-run/spec.md](/Users/rt/workspace/synod/specs/030-native-direct-run/spec.md).
- **PASS** Delivery-first scope: The plan prioritizes execution entry, routing ownership, and session safety ahead of docs or release polish. See Summary and Technical Context.
- **PASS** Primary workflow: Session-native remains the main operator path; explicit compatibility execution stays available only as a deliberate subordinate route. See Summary, Technical Context, research, and quickstart.
- **PASS** Bounded execution: The slice reuses the current sequential session runtime and native decision loop, keeps explicit terminal states, and does not introduce background work or unbounded retry behavior. See Technical Context, research, and quickstart.
- **PASS** Stateful execution: Direct run bootstrap persists session state, negotiation projection, goal-plan state, decisions, and traces so later commands can continue from the same authoritative record. See Summary, Technical Context, research, and data model.
- **PASS** Mutable planning: The slice reuses the existing native planning and replanning model; it only changes how direct run creates an executable plan and avoids pending flow-confirmation dead ends. See Summary, research, and data model.
- **PASS** Sequential-first design: One step remains active at a time through the current native decision loop or explicit compatibility path. See Technical Context and research.
- **PASS** Tool-agent symmetry: Native direct run continues to dispatch explicit analyze/code/test actions through registered adapters and tools. See Technical Context, research, and quickstart.
- **PASS** Observability and explicit intelligence: Route choice, execution path, session safety failures, clarification stops, decisions, and trace continuity remain visible on current CLI and trace surfaces. See Technical Context, research, contracts, and quickstart.
- **PASS** Non-goals and external separation: The slice avoids Canon-owned control flow, provider-gateway expansion, councils, long-term memory, distributed orchestration, UI work, and deployment changes. See Constraints, research, and spec.
- **PASS** Minimal slice: The smallest independently valuable capability is one native-first direct-run entry that preserves explicit compatibility as opt-in and keeps the rest of the current runtime model intact. See Summary and research.

Mark each line as PASS or FAIL in the completed plan and reference the section that satisfies it.

## Project Structure

### Documentation (this feature)

```text
specs/030-native-direct-run/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── native-direct-run-bootstrap-contract.md
│   ├── explicit-compatibility-opt-in-contract.md
│   └── active-session-protection-contract.md
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
│   ├── diagnostics.rs
│   ├── run.rs
│   └── session.rs
├── domain/
│   ├── session.rs
│   └── trace.rs
├── orchestrator/
│   └── session_runtime.rs
└── fixture.rs

tests/
├── contract/
├── integration/
└── unit/

assistant/
├── claude/commands/
├── codex/commands/
└── copilot/prompts/

docs/
├── configuration.md
└── getting-started.md

README.md
CHANGELOG.md
CONTRIBUTING.md
ROADMAP.md
Cargo.toml
Cargo.lock
AGENTS.md
```

**Structure Decision**: Keep the slice inside the existing CLI routing,
session runtime, diagnostics, and assistant-doc surfaces. No new top-level
runtime or persistence model is required because the feature changes how direct
run enters the already existing native execution path rather than introducing a
new orchestration surface.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations are expected for this slice.
