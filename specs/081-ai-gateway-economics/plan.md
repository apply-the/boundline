# Implementation Plan: AI Gateway And Inference Economics

**Branch**: `081-ai-gateway-economics` | **Date**: 2026-06-18 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/081-ai-gateway-economics/spec.md`

## Summary

Add provider-agnostic session budget enforcement, authority-zone-based spend exception approval, operator-owned pricing snapshots, and inference cost telemetry to Boundline. The feature introduces three user stories: P1 (session budget enforcement with reservation/reconciliation), P2 (risk-and-health-aware route selection with fallback), and P3 (governed route changes and local/private routes). All new state is additive to existing session, trace, and configuration surfaces. V1 enforces budgets at the session level only, defaults approval scope to `single_call`, and derives approval authority from existing roles without introducing a new RBAC system.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `rusqlite` (bundled SQLite), `dialoguer`, `rust_decimal` (new), existing workspace crates `boundline-core`, `boundline-adapters`, `boundline-cli`

**Storage**: Existing `.boundline/session.json` (additive fields), `.boundline/config.toml` (new `[inference_economics]` section), `.boundline/traces/` (extended event payloads). No new persistence backends.

**Testing**: `cargo test`, `cargo nextest run`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo llvm-cov`

**Target Platform**: macOS, Linux, Windows (CLI)

**Project Type**: CLI + library (workspace crate extension)

**Performance Goals**: Budget arithmetic <1ms per operation; reservation evaluation within task routing cycle (<10ms); no new I/O on the hot path beyond existing session reads.

**Constraints**:
- Exact decimal arithmetic only (no floating-point for monetary values) — use `rust_decimal` or a lightweight `MonetaryAmount(i64)` newtype
- No new RBAC/identity system — derive approval authority from existing session ownership, workspace roles, authority-zone policy, and governance roles
- No silent price fetching or activation — operators own snapshot lifecycle
- Existing model-selection defaults preserved unless budget/policy/health/governance requires otherwise

**Scale/Scope**: Session-level budgets only in V1; `single_call` default approval scope; operator-owned pricing snapshots.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Verdict | Notes |
|-----------|---------|-------|
| I. Delivery Identity | ✅ PASS | Budget enforcement and route economics directly improve delivery cost control |
| II. Delivery-First Scope | ✅ PASS | Answers "does it help deliver working code more economically?" — yes |
| III. No Abstract Agent Systems | ✅ PASS | Routing is delivery infrastructure, not a multi-agent framework |
| IV. Bounded Execution | ✅ PASS | All calls have explicit reservation limits, approval scopes, budget ceilings, and staleness thresholds |
| V. Stateful Execution | ✅ PASS | Budget state, approval records, cost records are all stateful with additive persistence |
| VI. Mutable Planning | ✅ PASS | Budgets are mutable per session; route policies are governed but changeable |
| VII. Execution Over Perfect Planning | ✅ PASS | Starts with simple budget enforcement (P1) before complex routing (P2) |
| VIII. Sequential-First Design | ✅ PASS | One provider call at a time; no concurrency in reservation/admission flow |
| IX. Tool-Agent Symmetry | ✅ PASS | Routes include deterministic non-LLM path (Tier 0); think/act/evaluate visible |
| X. Required Observability | ✅ PASS | FR-038–040, FR-048, SC-007: provider, model, route, cost quality, snapshot staleness, approval state all surfaced |
| XI. No Hidden Intelligence | ✅ PASS | All routing decisions, approvals, cost states are explicit and traceable |
| XII. Strict Non-Goals | ✅ PASS | No councils/voting, no provider abstraction beyond what's needed, no distributed systems |
| XIII. Minimal Capability Slices | ✅ PASS | P1 (budget enforcement + telemetry) is independently shippable |
| XIV. Real Acceptance Criteria | ✅ PASS | All three user stories have concrete engineering scenarios with success and failure paths |
| XV. Failure as a First-Class Path | ✅ PASS | Unknown cost, over-budget, approver unavailable, stale snapshots, egress gaps — all specified |
| XVI. Separation From External Systems | ✅ PASS | No Canon dependency; feature is independently testable |
| XVII. Evolution Without Premature Lock-In | ✅ PASS | Snapshot model supports future auto-fetch; approval scopes extensible; currency model supports additions |
| XVIII. Done Means Executable Delivery | ✅ PASS | 8 measurable success criteria; all FRs testable |

**Gate Result**: ALL PASS. No violations requiring Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/081-ai-gateway-economics/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI contract, internal API signatures)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
  boundline-core/
    src/
      domain/
        inference_economics.rs    # NEW: budget, snapshot, approval, cost record types
        session.rs                # MODIFY: additive budget/approval/cost fields
        trace.rs                  # MODIFY: new TraceEventType variants
        observability.rs          # MODIFY: extended ProviderCallCompleted payload
        configuration.rs          # MODIFY: InferenceEconomicsConfig section
        governance.rs             # MODIFY: ApproverRole derivation helpers
      lib.rs                      # MODIFY: pub mod inference_economics
  boundline-adapters/
    src/
      session_store.rs            # MODIFY: serialize new session fields
      trace_store.rs              # MODIFY: serialize new event payload fields
      config_store.rs             # MODIFY: serialize InferenceEconomicsConfig
      budget_enforcer.rs          # NEW: reservation, reconciliation, approval logic
      pricing_resolver.rs         # NEW: snapshot lookup, staleness check
      route_selector.rs           # NEW: risk-aware route selection, fallback policy
  boundline-cli/
    src/
      cli.rs                      # MODIFY: new subcommands (approve, set-budget, set-pricing, inspect cost)
      cli/
        config.rs                 # MODIFY: set-budget handler
        provider.rs               # MODIFY: set-pricing handler
        status.rs                 # MODIFY: budget projection in status output
        approve.rs                # NEW: interactive spend exception approval
        inspect_cost.rs           # NEW: cost/budget inspection
tests/
  unit/
    inference_economics.rs        # NEW: budget arithmetic, approval state machine, snapshot staleness
    budget_enforcer.rs            # NEW: reservation/reconciliation unit tests
  contract/
    inference_economics_trace.rs  # NEW: extended event payload schema validation
  integration/
    inference_economics_cli.rs    # NEW: end-to-end CLI budget/approval flows
```

**Structure Decision**: New domain module `inference_economics.rs` in `boundline-core` keeps all economics types co-located. Adapter logic for budget enforcement and pricing resolution is in `boundline-adapters` to separate domain models from I/O concerns. CLI extensions are additive to existing subcommand modules.

## Complexity Tracking

No violations. All 18 constitution principles pass without justification needed.
