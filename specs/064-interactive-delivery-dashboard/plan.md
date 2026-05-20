# Implementation Plan: Interactive Delivery Dashboard

**Branch**: `064-interactive-delivery-dashboard` | **Date**: 2026-05-19 | **Spec**: [spec.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/spec.md)
**Input**: Feature specification from `/specs/064-interactive-delivery-dashboard/spec.md`

## Summary

Ship a complete operator-facing terminal dashboard over Boundline's existing runtime truth. The implementation will add a separate dashboard workspace component for the interactive surface, add typed dashboard snapshot and action contracts over existing session, trace, checkpoint, finding, context-pack, diagnostic, and governed-reference state, keep normal Boundline commands and the session-native runtime authoritative, and close the release with version, docs, wiki, changelog, roadmap cleanup, catalog reconciliation, validation, linting, formatting, and modified-file coverage.

The dashboard will not introduce a second workflow engine, state store, configuration path, initialization path, governance runtime, Canon runtime dependency, provider layer, browser surface, or autonomous background execution. All operator actions will apply through the same Boundline-owned state transitions used by the normal command surfaces.

## Technical Context

**Language/Version**: Rust 1.95.0, edition 2024
**Primary Dependencies**: Existing workspace dependencies (`clap`, `dialoguer`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite`) plus a dashboard-local terminal UI stack (`ratatui` with a terminal backend such as `crossterm`) scoped to the dashboard component
**Storage**: Existing workspace-local `.boundline/session.json`, `.boundline/traces/`, optional `.boundline/checkpoints/`, optional `.boundline/config.toml`, optional `.boundline/workflows.toml`, optional `.boundline/execution.json`, and optional `.canon/` references; no new authoritative dashboard state store
**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test`, focused contract and integration tests for dashboard snapshots/actions/render states, local render and refresh performance validation against the plan targets, and modified Rust coverage above the repository target
**Target Platform**: Local developer terminals on macOS and Linux, plus CI validation for non-interactive snapshot/action behavior
**Project Type**: Existing Rust workspace with a root CLI binary, `boundline-core`, `boundline-adapters`, `boundline-cli`, and a new dashboard workspace component
**Execution Model**: Sequential operator loop: resolve the current session from authoritative workspace state, observe snapshot, optionally refresh externally changed state, select one allowed action, apply through existing runtime boundary, verify resulting session or trace state, refresh snapshot
**Observability Surface**: Typed dashboard snapshot, typed action result or refusal, persisted session state, persisted trace summaries, context-pack evidence facts, checkpoint refs, diagnostic readiness facts, existing status/inspect output, and dashboard degraded-state messages
**Performance Goals**: First dashboard render under 1 second on representative local workspaces; refresh after a local action under 1 second excluding the underlying command runtime; operators identify state and blocking reason within 30 seconds
**Constraints**: No second workflow engine; no separate state store; no hidden background workers; no parallel execution; no Canon runtime changes; governed references are read-only; normal command surfaces remain fully usable; terminal branding is a simple `boundline` ASCII wordmark only
**Scale/Scope**: One active workspace session at a time for the interactive view after current-session resolution; latest session plus bounded recent trace/checkpoint/finding/context/diagnostic projections; enough history for operator trust without turning the dashboard into a full log browser

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **PASS - Delivery identity**: The feature improves bounded engineering delivery by making current execution state, next action, failures, findings, checkpoints, and recovery choices legible during real work.
- **PASS - Delivery-first scope**: The work prioritizes execution control, orchestration visibility, state handling, recovery, and validation. Branding and layout remain subordinate to delivery-state comprehension.
- **PASS - Primary workflow**: The main path remains session-native (`start -> capture -> plan -> run -> status -> next -> inspect`). The dashboard observes and invokes that same route. Explicit compatibility traces remain inspectable but labeled as compatibility context.
- **PASS - Bounded execution**: Start condition is a detected workspace or explicit dashboard launch target. Terminal conditions are succeeded, failed, blocked, waiting, exhausted, invalid, degraded, or complete. The dashboard applies at most one operator action at a time and refreshes state after each action.
- **PASS - Stateful execution**: The dashboard reads existing session, trace, checkpoint, finding, and governed-reference state, and writes only through existing Boundline state transitions. It does not own a second task context.
- **PASS - Mutable planning**: Replanning and rejection are explicit dashboard actions that map to existing bounded plan mutation or stopped-state behavior and preserve prior evidence.
- **PASS - Sequential-first design**: Only one active operator action is allowed at a time. There are no background workers, hidden fan-out, or parallel dashboard-controlled execution paths.
- **PASS - Tool-agent symmetry**: Reasoning appears as plan, evidence, findings, and stop rationale. Action appears as confirm, reject, replan, recover, launch, or continue requests. Evaluation appears as refreshed snapshot and trace outcome.
- **PASS - Observability and explicit intelligence**: Decisions, failure signals, stale-action refusals, degraded conditions, governed-reference availability, and next-action rationale are visible in the snapshot, action result, and persisted trace/session evidence.
- **PASS - Catalog currency**: Public provider docs were checked during specification. Planning records that the bundled catalog must be reconciled before release closure because drift was found.
- **PASS - Non-goals and external separation**: The plan does not depend on Canon behavior beyond optional read-only governed references and does not introduce councils, voting, provider abstraction, long-term memory, browser automation, MCP work, or deployment pipelines. The interactive UI surface is explicitly delivery-facing and required by the feature direction, not generic UI polish.
- **PASS - Minimal slice**: The smallest useful capability is a complete dashboard over current Boundline state with action handoff and degraded behavior. A read-only-only dashboard would fail the operator loop; a partial dashboard would not meet the agreed release boundary.

## Project Structure

### Documentation (this feature)

```text
specs/064-interactive-delivery-dashboard/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── dashboard-action-contract.md
│   ├── dashboard-command-contract.md
│   ├── dashboard-render-contract.md
│   └── dashboard-snapshot-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
Cargo.toml
crates/
├── boundline-core/
│   └── src/
│       ├── domain.rs
│       └── lib.rs
├── boundline-adapters/
│   └── src/
│       ├── adapters.rs
│       └── lib.rs
├── boundline-cli/
│   └── src/
│       └── cli.rs
└── boundline-dashboard/
    ├── Cargo.toml
    └── src/
        ├── app.rs
        ├── branding.rs
        ├── input.rs
        ├── lib.rs
        ├── main.rs
        ├── render.rs
        └── state.rs

src/
├── cli.rs
├── cli/
│   └── dashboard.rs
├── domain/
│   └── dashboard.rs
└── adapters/
    └── dashboard_state.rs

tests/
├── contract/
│   ├── dashboard_action_contract.rs
│   ├── dashboard_branding_contract.rs
│   ├── dashboard_command_contract.rs
│   ├── dashboard_inspection_contract.rs
│   ├── dashboard_release_docs_contract.rs
│   ├── dashboard_render_contract.rs
│   └── dashboard_snapshot_contract.rs
├── fixtures/
│   └── 064-interactive-delivery-dashboard/
└── integration/
    ├── dashboard_action_flow.rs
    ├── dashboard_action_refusal_flow.rs
    ├── dashboard_degraded_flow.rs
    ├── dashboard_degraded_state_flow.rs
    ├── dashboard_inspection_flow.rs
    └── dashboard_snapshot_flow.rs
```

**Structure Decision**: Add a dedicated dashboard workspace component for interactive rendering and input handling, while keeping shared dashboard data shapes in root `src/domain/dashboard.rs` and exposing them through the existing `boundline-core` path-based workspace pattern. State assembly stays in root `src/adapters/dashboard_state.rs` and is exposed through `boundline-adapters`. The normal CLI remains authoritative through a thin `src/cli/dashboard.rs` launcher. TUI behavior and dependencies stay in the dashboard component so the slim command path does not become the dashboard implementation owner.

## Complexity Tracking

No constitution violations are introduced. The dashboard is an explicitly delivery-facing operator surface and is isolated so it does not become a second runtime or generic UI platform.

## Phase 0 Research Output

Research is captured in [research.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/research.md). It resolves the major implementation choices:

- isolate the dashboard in a dedicated workspace component
- use typed snapshots rather than parsing human output
- resolve the current session explicitly when persisted state has multiple candidates or stale trace references
- project context-pack reason, source, budget cost, authority, and provenance when available
- provide a dashboard-oriented diagnostics view over existing readiness and fallback facts
- refresh externally changed state explicitly or through a non-autonomous display-only watcher
- apply actions through existing runtime boundaries
- use mature terminal rendering with explicit fallback behavior
- keep governed references read-only and optional
- reconcile assistant model catalog drift during release closure
- use a static terminal wordmark

## Phase 1 Design Output

Design artifacts are captured in:

- [data-model.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/data-model.md)
- [dashboard-snapshot-contract.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/contracts/dashboard-snapshot-contract.md)
- [dashboard-action-contract.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/contracts/dashboard-action-contract.md)
- [dashboard-render-contract.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/contracts/dashboard-render-contract.md)
- [dashboard-command-contract.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/contracts/dashboard-command-contract.md)
- [quickstart.md](/Users/rt/workspace/apply-the/boundline/specs/064-interactive-delivery-dashboard/quickstart.md)

## Post-Design Constitution Check

- **PASS - Delivery identity**: Data model and contracts center on delivery state, next actions, failures, and recovery.
- **PASS - Delivery-first scope**: The render contract keeps layout subordinate to state comprehension and action safety.
- **PASS - Primary workflow**: Command and action contracts preserve session-native authority and label compatibility context explicitly.
- **PASS - Bounded execution**: Action contracts require revision checks, single-action application, and explicit refusal states.
- **PASS - Stateful execution**: Snapshot and action contracts read and write only through Boundline-owned state and trace evidence.
- **PASS - Mutable planning**: Reject and replan actions preserve evidence and route back to explicit plan mutation or stopped state.
- **PASS - Sequential-first design**: No generated design artifact introduces background work, parallel execution, or hidden fan-out.
- **PASS - Tool-agent symmetry**: Snapshot, action, and render contracts expose reasoning, action, and evaluation states separately.
- **PASS - Observability and explicit intelligence**: Contracts require timeline evidence, refusal reasons, degraded states, and valid fallback commands.
- **PASS - Catalog currency**: Research and spec both record current public-provider findings and require catalog reconciliation before release.
- **PASS - Non-goals and external separation**: Governed references remain read-only and optional; no Canon runtime, provider, MCP, browser, or deployment scope is introduced.
- **PASS - Minimal slice**: The design is complete enough to deliver the operator dashboard without pulling in unrelated platform work.
