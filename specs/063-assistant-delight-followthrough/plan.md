# Implementation Plan: S7.1 Assistant Delight Follow-Through

**Branch**: `063-assistant-delight-followthrough` | **Date**: 2026-05-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/063-assistant-delight-followthrough/spec.md`

## Summary

Extend the existing S7 delight layer by reusing current session and trace
projections to: disclose active reasoning-profile contribution inside
explanation surfaces; close human-facing inspect views for context, council,
and timeline; make Cursor and Gemini host parity or fallback decisions explicit
in assistant assets; and record lightweight usefulness signals in existing
session or trace state. The slice remains Boundline-only and does not introduce
new Canon contracts, a second assistant runtime, or an external analytics
surface.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024, plus repository-managed Markdown and JSON assistant assets  
**Primary Dependencies**: existing workspace crates and runtime dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite`); no new runtime dependencies planned for the first slice  
**Storage**: workspace-local `.boundline/session.json`, persisted traces under `.boundline/traces/`, and repository-managed assistant asset manifests and host docs under `assistant/`  
**Testing**: focused `cargo test` unit, integration, and contract suites; especially inspect or output projections, session state, and assistant command pack contracts; use `cargo test --no-run --all-targets` if shared view structs change  
**Target Platform**: macOS and Linux developer workstations plus Linux CI  
**Project Type**: Rust workspace CLI with repository-managed assistant command assets  
**Execution Model**: sequential session-native CLI projections over existing session and trace state, with no new background workers, hidden loops, or automatic retries  
**Observability Surface**: `.boundline/session.json`, persisted execution traces, `status` and `inspect` JSON plus human-facing CLI output, assistant manifests and fallback docs, and focused contract tests for assistant asset coverage  
**Performance Goals**: profile-aware explanations and inspect closure should render from the authoritative session or trace in one operator invocation; operators should not need raw trace reads to understand context, council, or timeline; usefulness signals must be inspectable without reconstructing unrelated logs  
**Constraints**: no new Canon provider artifact classes or contract lines; no second assistant runtime; no expansion of the default assistant palette beyond existing contextual commands; no hidden fallback behavior; respect repository Rust no-panic, no-magic-literal, and typed-serialization rules  
**Scale/Scope**: one active workspace session at a time, one latest authoritative trace per inspect call, existing Claude, Codex, and Copilot repo-local assets plus explicit Cursor and Gemini parity or fallback decisions, and lightweight session-scoped usefulness signals rather than organization-wide analytics

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- PASS Delivery identity: The feature improves bounded engineering delivery by
  making explanation, inspect, and next-action surfaces more trustworthy on
  real session state instead of chat memory (see Summary and
  [research.md](./research.md)).
- PASS Delivery-first scope: The plan is limited to runtime projections,
  assistant asset behavior, and inspectable usefulness signals; it does not
  prioritize optimization-only or polish-only work ahead of delivery behavior
  (see Summary and Phase Plan).
- PASS Primary workflow: The main operator path remains session-native
  `goal -> plan -> run -> status -> next -> inspect`; assistant
  hosts remain wrappers over CLI and session authority, with Cursor and Gemini
  fallback paths explicit rather than implied (see Technical Context and
  [contracts/assistant-host-parity.md](./contracts/assistant-host-parity.md)).
- PASS Bounded execution: Start condition is an operator invoking a delight or
  inspect command against an active workspace session or latest trace; terminal
  condition is a rendered projection or explicit missing-state or fallback
  disclosure; no new autonomous loop or hidden retries are introduced (see
  Technical Context and
  [contracts/delight-projection-contract.md](./contracts/delight-projection-contract.md)).
- PASS Stateful execution: Projections read `ActiveSessionRecord`,
  `TraceSummaryView`, and assistant manifest state; lightweight usefulness
  signals write back into existing session or trace authority rather than a new
  telemetry store (see [data-model.md](./data-model.md)).
- PASS Mutable planning: The feature does not replace planning; it exposes
  existing plan mutation, recovery, and reasoning outcomes more clearly through
  explanation and timeline projections (see Summary and
  [contracts/delight-projection-contract.md](./contracts/delight-projection-contract.md)).
- PASS Sequential-first design: All behavior remains one command at a time over
  the current authoritative state, with no background workers or parallel
  orchestration added (see Technical Context).
- PASS Tool-agent symmetry: Assistant commands and CLI commands share the same
  state authority and projection contracts, keeping reasoning, action, and
  evaluation surfaces explicit (see
  [contracts/assistant-host-parity.md](./contracts/assistant-host-parity.md)
  and
  [contracts/delight-projection-contract.md](./contracts/delight-projection-contract.md)).
- PASS Observability and explicit intelligence: Reasoning-profile disclosure,
  source attribution, fallback disclosure, timeline composition, and host
  fallback state are all surfaced through inspect or status output, manifests,
  and tests; hidden heuristics are out of scope (see [research.md](./research.md)
  and [data-model.md](./data-model.md)).
- PASS Catalog currency: Current Anthropic, OpenAI, and Gemini public model
  docs were rechecked during specification; no bundled catalog delta was needed
  and that no-change result remains explicit in planning (see [research.md](./research.md)
  and [spec.md](./spec.md)).
- PASS Non-goals and external separation: The plan reuses existing Canon
  evidence boundaries only; it does not depend on new Canon behavior, does not
  add councils or voting logic, long-term memory, UI, or deployment work (see
  Summary and [spec.md](./spec.md)).
- PASS Minimal slice: The smallest independently valuable slice is to close the
  remaining delight and inspect surfaces using existing projections and
  assistant manifests instead of building a new subsystem (see Summary and
  [research.md](./research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/063-assistant-delight-followthrough/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── assistant-host-parity.md
│   ├── delight-projection-contract.md
│   └── feedback-signal-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
assistant/
├── commands/
├── global/
├── claude/
├── codex/
├── copilot/
└── gemini/
src/
├── cli/
├── domain/
├── orchestrator/
└── registry/
tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Reuse the existing CLI, domain, and assistant-asset
surfaces already shipping the delight layer. This feature should extend
`src/cli/*`, `src/domain/*`, repository-managed assistant assets under
`assistant/`, and focused tests under `tests/`, avoiding any new top-level
runtime or product surface.

## Phase Plan

### Phase 0 Research

- Confirm no Canon contract delta is needed for profile-aware disclosure and
  inspect closure.
- Choose a projection strategy that reuses `ProfileActivationRecord`,
  `TraceSummaryView`, and current status or inspect output paths.
- Decide explicit host parity or fallback states for Cursor and Gemini based on
  the current manifest and host docs.
- Choose the smallest usefulness signals that can live in session or trace
  authority without external telemetry.

### Phase 1 Design

- Define projection and state entities in [data-model.md](./data-model.md).
- Define CLI, assistant, and maintainer-facing interface contracts in
  [contracts/](./contracts/).
- Write validation-oriented developer flow in
  [quickstart.md](./quickstart.md).
- Refresh agent context after the design artifacts land.

### Phase 2 Implementation Strategy

- Workstream 1: extend status, inspect, and delight output to disclose
  reasoning-profile contribution and fallback semantics.
- Workstream 2: add inspect context, inspect council, and inspect timeline
  closures from existing trace and session summaries.
- Workstream 3: make Cursor and Gemini parity or fallback explicit across
  manifests, docs, and generated assets.
- Workstream 4: add lightweight usefulness signal capture and projection, then
  cover it with focused unit, integration, and contract tests.

## Complexity Tracking

No constitutional violations or exceptions are currently required.
