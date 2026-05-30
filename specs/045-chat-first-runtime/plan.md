# Implementation Plan: Chat-First Host-Integrated Runtime

**Branch**: `045-chat-first-runtime` | **Date**: 2026-05-09 | **Spec**: [/Users/rt/workspace/apply-the/boundline/specs/045-chat-first-runtime/spec.md](/Users/rt/workspace/apply-the/boundline/specs/045-chat-first-runtime/spec.md)
**Input**: Feature specification from `/specs/045-chat-first-runtime/spec.md`

## Summary

Deliver the first host-integrated runtime slice by adding an opt-in structured
CLI output contract for the existing session-native and inspection surfaces.
Rather than building a new standalone chat app, Boundline will reuse the
existing `SessionStatusView` and `TraceSummaryView` projections, expose them in
machine-readable form on the commands that assistant hosts already invoke, and
keep the current human-readable output as the default. The first slice treats
VS Code chat as the primary host surface while preserving the same runtime
contract for Claude, Codex, and Gemini command-pack execution later.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024  
**Primary Dependencies**: Existing CLI/runtime stack (`clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`) with Rust standard library terminal and filesystem APIs; no new runtime dependencies for the first slice  
**Storage**: Existing workspace-local `.boundline/session.json` and `.boundline/traces/`, plus assistant asset files under `assistant/`; no new persistence surface  
**Testing**: `cargo test`, focused contract tests in `tests/contract/*.rs`, focused integration tests in `tests/integration/*.rs`, unit coverage in `src/cli/*.rs`, plus final `cargo test --no-run --all-targets --all-features`  
**Target Platform**: macOS/Linux developer workstations and Linux CI
**Project Type**: Multi-crate Rust CLI with repository-managed assistant command packs  
**Execution Model**: Sequential CLI command execution with one explicit operator-visible step at a time; structured host output is opt-in per invocation and does not add background work  
**Observability Surface**: Persisted execution traces, `SessionStatusView`, `TraceSummaryView`, CLI stdout in text or JSON form, and assistant command-pack guidance that points hosts to the correct runtime surface  
**Performance Goals**: Structured output rendering must add negligible overhead relative to existing command execution and must not change execution limits or task control behavior  
**Constraints**: No standalone chat app or full-screen TUI; preserve existing text output as the default; preserve existing exit codes; do not require Canon to be present; keep chat-only paste fallback available for hosts that cannot execute shell commands  
**Scale/Scope**: First slice covers the session-native lifecycle commands (`start`, `goal`, `flow`, `plan`, `step`, `run`, `status`, `next`) plus `inspect`, and aligns the assistant command packs that already wrap those surfaces

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS** Delivery identity: The feature improves how operators execute and continue real bounded delivery work from existing host chat surfaces instead of introducing a parallel chat product. See Summary and Technical Context.
- **PASS** Delivery-first scope: The slice focuses on execution continuity, inspection, and host-command reliability rather than branding, standalone UI, or speculative assistant abstractions. See Summary and Constraints.
- **PASS** Primary workflow: The main operator path remains session-native (`goal -> plan -> run -> status -> next -> inspect`), and explicit compatibility continuations stay visible rather than implicit. See Summary and Scale/Scope.
- **PASS** Bounded execution: Start conditions remain the existing CLI commands; terminal conditions remain the current session and trace outcomes; the slice adds no new retries, background work, or hidden control flow. See Execution Model and Constraints.
- **PASS** Stateful execution: The host contract reads the existing workspace-owned session and trace state and returns explicit projections of that state without introducing a stateless shortcut. See Storage and Observability Surface.
- **PASS** Mutable planning: The slice does not alter planning semantics; it exposes the current plan, clarification, failure, and continuation state more reliably to host chats. See Summary and research decisions.
- **PASS** Sequential-first design: All behavior remains one command invocation at a time with no concurrency or fan-out. See Execution Model.
- **PASS** Tool-agent symmetry: Hosts continue to invoke explicit commands (`start`, `goal`, `plan`, `run`, `status`, `next`, `inspect`) and receive explicit action or inspection data, not hidden heuristics. See Scale/Scope and contract.
- **PASS** Observability and explicit intelligence: Session and trace output remain inspectable in both human-readable and structured forms, with visible continuity authority, next command, and failure reasoning. See Observability Surface.
- **PASS** Non-goals and external separation: The plan avoids a new standalone UI, councils, long-term memory, deployment work, or Canon-owned control flow. See Constraints and Scope Boundaries in the spec.
- **PASS** Minimal slice: The smallest independently valuable capability is a host being able to run the existing session-native commands and consume a stable machine-readable response without parsing ad hoc text. See Summary.

## Project Structure

### Documentation (this feature)

```text
specs/045-chat-first-runtime/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
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
│   ├── session.rs
│   └── trace.rs
└── lib.rs

assistant/
├── README.md
├── claude/commands/
├── codex/commands/
├── copilot/prompts/
└── gemini/

tests/
├── contract/
├── integration/
└── support/
```

**Structure Decision**: Keep the slice inside the existing CLI, domain view,
assistant asset, and test surfaces. Reuse `SessionStatusView` and
`TraceSummaryView` instead of introducing a new crate, protocol package, or UI
runtime.

## Complexity Tracking

No constitution violations require special justification for this slice.
