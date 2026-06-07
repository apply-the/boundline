# Implementation Plan: Recursive Stage Refinement Profiles

**Branch**: `076-recursive-stage-refinement` | **Date**: 2026-06-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/076-recursive-stage-refinement/spec.md`

## Summary

Add bounded, inspectable stage-refinement loops to the existing session-native runtime. The first slice supports a single `plan_refinement` profile for the plan stage with a `planner → critic → planner → finalizer` pattern. Each refinement round produces a compact structured round packet (trace-linked, schema-versioned, no inline artifact content). The loop is bounded by hard `max_rounds` and `max_elapsed_time` limits, stops on no-material-delta detection, and surfaces outcomes through existing `boundline status`, `boundline next`, and `boundline inspect` commands. The feature reuses the existing provider registry, session, trace, finding, and stop-semantics surfaces without introducing new council, calibration, or route-cost behavior.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`; existing workspace crates `boundline-core` (domain types, session, trace, findings), `boundline-adapters` (provider registry, protocol), `boundline-cli` (status, next, inspect commands)

**Storage**: Workspace-local `.boundline/refinement-profiles.toml` (new config file); existing `.boundline/traces/` and SQLite trace store for round packets; existing `.boundline/session.json` for stage state

**Testing**: `cargo test` (unit + contract + integration), `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt`, `cargo llvm-cov` with per-crate merge

**Target Platform**: macOS, Linux (CLI tool)

**Project Type**: CLI orchestration tool (single Rust workspace)

**Performance Goals**: Refinement loop completion within `max_elapsed_time` (default 300s); round packet persistence <100ms; inspect output rendering <1s

**Constraints**: No `sqlite-vec` dependency. No new provider registration surface. No new council, calibration, route-cost, session, or trace system. No parallelism or concurrency. Sequential execution only. Round packets must never embed full artifact content inline.

**Scale/Scope**: One stage (plan), one profile (`plan_refinement`), one loop pattern (`planner → critic → planner → finalizer`). Multi-stage and multi-profile expansion deferred to later slices.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Delivery Identity | ✅ PASS | Plan refinement directly improves plan quality, leading to more reliable delivery execution. |
| II. Delivery-First Scope | ✅ PASS | Better plans → better execution. Refinement is a delivery quality step, not speculative exploration. |
| III. No Abstract Agent Systems | ✅ PASS | Roles (planner, critic, finalizer) are concrete delivery steps, not generic agents. Each consumes structured input and produces structured output. |
| IV. Bounded Execution | ✅ PASS | Hard `max_rounds`, `max_elapsed_time`, explicit stop conditions (`no_material_delta`, `round_limit_exhausted`, `time_limit_exhausted`, etc.). No infinite or unbounded loops. |
| V. Stateful Execution | ✅ PASS | Round packets persist in trace store. Refinement reads previous candidate and writes updated candidate, findings, and deltas back to session state. |
| VI. Mutable Planning | ✅ PASS | Refinement is explicit plan mutation (revision deltas) with trace evidence linking each change to critique findings. |
| VII. Execution Over Perfect Planning | ✅ PASS | Bounded iteration (default 3 rounds) over perfect upfront planning. Stops when no material improvement detected. |
| VIII. Sequential-First Design | ✅ PASS | Single sequential pipeline: planner → critic → planner → finalizer. No parallelism, DAG execution, or background workers. |
| IX. Tool-Agent Symmetry | ✅ PASS | "think" (planner generates candidate), "evaluate" (critic produces findings), "act" (planner applies deltas, finalizer produces outcome). All transitions visible in round packets. |
| X. Required Observability | ✅ PASS | Per-round trace events (`refinement.round.completed`), round packets with full fields, inspect/status/next surface integration. |
| XI. No Hidden Intelligence | ✅ PASS | Every decision (confidence, stop reason, delta application) is explicit in round packets. Runtime validates materiality; does not rely solely on provider self-assessment. |
| XII. Strict Non-Goals | ✅ PASS | Council, calibration, route-cost excluded. No voting, distributed agents, or UI work. Out-of-scope items named explicitly in spec. |
| XIII. Minimal Capability Slices | ✅ PASS | Single stage, single profile, single pattern. Delivers immediate value (better plan quality) without bundled future-proof abstractions. |

**Gate Result**: ALL PASS. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/076-recursive-stage-refinement/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
├── tasks.md             # Phase 2 output (/speckit.tasks)
├── spec.md              # Feature specification
├── spec-recursive-stage-refinement-profiles.md  # Roadmap seed (preserved)
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code (repository root)

```text
crates/
├── boundline-core/
│   └── src/
│       ├── domain/
│       │   └── refinement.rs          # NEW: RefinementProfile, RoundPacket, ClosureCheck, StopReason, etc.
│       ├── orchestrator/
│       │   └── refinement.rs          # NEW: refinement loop execution, closure check, trace emission
│       └── domain.rs                  # MODIFIED: add `#[path]` entry for refinement module
├── boundline-adapters/
│   └── src/
│       └── registry.rs                # MODIFIED: role-to-provider resolution for refinement roles
├── boundline-cli/
│   └── src/
│       ├── cli.rs                     # MODIFIED: add --refine/--no-refine/--max-rounds flags to plan command
│       ├── plan_cmd.rs                # MODIFIED: refinement loop integration in plan execution
│       ├── status_cmd.rs              # MODIFIED: surface refinement state
│       ├── next_cmd.rs                # MODIFIED: surface refinement recommendations
│       ├── inspect_cmd.rs             # MODIFIED: surface refinement history
│       └── refinement_cmd.rs          # NEW: CLI refinement rendering helpers
src/
├── domain/
│   ├── refinement.rs                  # NEW (path-mapped): actual domain type implementations
│   ├── trace.rs                       # MODIFIED: add RefinementRoundCompleted to TraceEventType enum
│   └── observability.rs               # MODIFIED: add RefinementRoundCompleted to EventType enum
├── orchestrator/
│   └── refinement.rs                  # NEW (path-mapped): actual orchestrator implementations
├── registry/
│   └── agent_registry.rs              # MODIFIED: refinement role resolution (note: crate facade at crates/boundline-adapters/src/registry.rs may not need changes if no new file is created)
└── cli/
    └── refinement_cmd.rs              # NEW (path-mapped): CLI rendering
tests/
├── unit/
│   └── refinement_model.rs            # NEW: domain type tests
├── contract/
│   ├── refinement_config_contract.rs  # NEW: profile config loading contract tests
│   └── refinement_output_contract.rs  # NEW: round packet schema + inspect output contract tests
└── integration/
    └── refinement_flow.rs             # NEW: end-to-end refinement loop tests
tests/
├── unit/
│   └── refinement_model.rs            # NEW: domain type tests (profile, packet, closure check, stop reason)
├── contract/
│   └── refinement_output_contract.rs  # NEW: round packet schema contract, inspect output contract
└── integration/
    └── refinement_flow.rs             # NEW: end-to-end refinement loop tests
```

**Structure Decision**: Single Rust workspace project. New domain types live in `boundline-core/src/domain/refinement.rs`. CLI integration touches existing plan/status/next/inspect commands in `boundline-cli`. Provider role resolution extends the existing registry in `boundline-adapters`. No new crates needed.

## Complexity Tracking

No constitution violations. This section intentionally empty.
