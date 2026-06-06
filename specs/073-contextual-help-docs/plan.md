# Implementation Plan: Contextual Help And Documentation Architecture (Boundline)

**Branch**: `073-contextual-help-docs` | **Date**: 2026-06-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/073-contextual-help-docs/spec.md`

## Summary

Add `boundline help-next`, a read-only operator guidance command that inspects workspace state across five lifecycle phases (uninitialized, initialized no-session, active, blocked, failed/healthy) and returns the next recommended action with an exact command, reason, and documentation link. Supports `--json` for automation, `--all` for complete diagnostics, and emits a `boundline.help_next.requested` structured event for observability. Documentation links are resolved via a versioned `.boundline/help-links.toml` map file.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates and dependencies only; `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `boundline-core`, `boundline-adapters`, `boundline-cli`

**Storage**: Read-only inspection of `.boundline/session.json`, `.boundline/config.toml`, `.boundline/help-links.toml` (new link map file), `.boundline/provider/` (provider state); write-only append to `.boundline/traces/events.jsonl` for the structured event; no mutation of existing session or config state

**Testing**: `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`, `scripts/common/coverage/intersect_patch_coverage.py`

**Target Platform**: Local CLI runtime; `--json` output consumable by CI and assistants

**Project Type**: Rust workspace; this feature adds one CLI command (`help-next`) that consumes existing readiness/probe/session status surfaces

**Performance Goals**: `help-next` completes within 1 second for any workspace state (purely in-memory diagnostics, no network); event emission is asynchronous append-only

**Constraints**: Read-only over session/config state; no mutation; links resolved from `.boundline/help-links.toml` with generic fallback for missing keys; no hardcoded URLs in Rust source; event payload must exclude secrets, raw prompts, and raw traces

**Scale/Scope**: One CLI command covering 5+ workspace states, one link map file, one new structured event type (`boundline.help_next.requested`), human-readable and JSON output, and `--all` flag for complete diagnostics

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle or standard | Result | Design evidence |
|---|---|---|
| Delivery identity and delivery-first scope | PASS | `help-next` directly improves operator ability to complete delivery work by reducing stuck states. |
| No abstract agent systems | PASS | No agent or reasoning framework; pure state inspection and recommendation. |
| Bounded execution | PASS | One in-memory diagnostic pass per invocation; no background processing. |
| Stateful execution | PASS | Emits a structured event; reads existing session/config state without mutation. |
| Mutable planning and execution over perfect planning | PASS | Recommends the next actionable step rather than demanding a perfect state first. |
| Sequential-first design | PASS | Single synchronous command execution. |
| Tool-agent symmetry and required observability | PASS | Human-readable and `--json` output; structured event emission for trace visibility. |
| No hidden intelligence | PASS | All diagnostics are deterministic inspections of typed state; no inference or heuristics. |
| Strict non-goals and minimal capability slice | PASS | No interactive repair, no Canon-side work, no wiki content generation — just the runtime diagnostic. |
| Separation from external systems | PASS | Link map is Boundline-owned; Canon `help-next` is a separate Canon spec. |
| Catalog currency | PASS | No model-catalog changes needed. |
| Rust language rules | PASS | Typed enums for states/diagnostics, named constants for diagnostic keys, `toml` crate for link map parsing. |

## Project Structure

### Documentation (this feature)

```text
specs/073-contextual-help-docs/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── help-next-output-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── domain/
│   └── help_next.rs              # HelpNextState, HelpNextDiagnostic, HelpNextRecommendation types + diagnostic logic
├── cli/
│   └── help_next.rs              # `boundline help-next` CLI command + output rendering
└── orchestrator/
    └── session_runtime_help_next.rs  # Event emission hook for help_next.requested

.boundline/
└── help-links.toml               # Versioned diagnostic-key → wiki-URL link map

tests/
├── unit/
│   └── help_next_model.rs
├── contract/
│   ├── help_next_output_contract.rs
│   └── help_next_event_contract.rs
└── integration/
    └── help_next_flow.rs
```

**Structure Decision**: Add a single domain module (`help_next.rs`) for types and diagnostic logic, a CLI module for the command surface, and an orchestrator hook for event emission. The link map lives under `.boundline/` as a committed config artifact. No new workspace member crate needed.

## Complexity Tracking

> No constitution violations to justify.

## Post-Design Constitution Recheck

*Re-checked after Phase 1 design outputs (research.md, data-model.md, contracts/, quickstart.md).*

| Principle | Post-design result | Evidence |
|-----------|-------------------|----------|
| Delivery identity | PASS | `help-next` keeps operators moving toward delivery by surfacing the next actionable step. |
| No abstract agent systems | PASS | Pure state inspection; no agent or reasoning abstraction. |
| Bounded execution | PASS | One synchronous in-memory diagnostic pass per invocation; no background work. |
| Stateful execution | PASS | Emits `HelpNextRequested` event to the structured event log. |
| Sequential-first | PASS | Single command, synchronous execution. |
| Tool-agent symmetry + required observability | PASS | Human-readable + `--json` output; structured event emission per spec 072 event vocabulary. |
| No hidden intelligence | PASS | All diagnostics are deterministic inspections of typed Boundline state. |
| Separation from external systems | PASS | Link map is Boundline-owned; Canon `help-next` is independently spec'd. |
| Rust language rules | PASS | Typed enums for `HelpNextState` and `DiagnosticSeverity`; `toml` crate for link map; named diagnostic key constants. |

All 12 constitution gates pass post-design.
