# Implementation Plan: Adaptive Governance Calibration

**Branch**: `075-adaptive-governance-calibration` | **Date**: 2026-06-06 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/075-adaptive-governance-calibration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Implement adaptive governance calibration so guardians produce graduated enforcement levels (advisory → catch → rule → hook) instead of binary pass/fail. The first slice makes one guardian's control-level decision fully inspectable via `boundline inspect`, including guardian-provided confidence, trust-metric-adjusted calibrated confidence, override records from `boundline override`, degradation state, and terminal outcome.

The calibration policy is stored in `.boundline/calibration-policy.toml` (separate from `.boundline/guardian-rules.toml`). It formalizes a calibration table shape containing `rule_id`, `authority_source`, `default_level`, `green_level`, `yellow_level`, `red_level`, `confidence_threshold`, and `override_policy`. The green/yellow/red bands map to Boundline's authority or risk zones, with Canon's authority zone and risk level consumed as read-only inputs when available. The control-level selection participates with `guidance strength`, `authority source`, and the aforementioned bands. Trust metrics accumulate continuously after every council adjudication; promotion/demotion evaluation occurs only after the configured evidence window (default 5 adjudicated sessions).

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml` (existing workspace dependencies). No new external crate dependencies planned.

**Storage**: Workspace-local `.boundline/calibration-policy.toml` (TOML, versioned), `.boundline/traces/` (trust metric accumulation via existing trace store), and `.boundline/` override record persistence. Existing `rusqlite` with FTS5 for trace indexing.

**Testing**: `cargo test` (unit, contract, integration), `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt`, `cargo llvm-cov` with per-crate merge. Patch-coverage ≥95% for all modified or created Rust files.

**Target Platform**: macOS (development), Linux (CI), Windows (release distributions). Core logic is platform-agnostic; CLI surface uses `clap` derive.

**Project Type**: CLI delivery orchestrator (existing `boundline` binary + workspace crates `boundline-core`, `boundline-adapters`, `boundline-cli`).

**Performance Goals**: Calibration policy evaluation ≤100ms for up to 10 guardians; `boundline inspect` control-level rendering ≤30 seconds for end-to-end operator comprehension (SC-001). Evidence window evaluation non-blocking.

**Constraints**: Must not introduce new runtime dependencies. Must reuse existing `.boundline/` configuration conventions (TOML, versioned, fail-closed). Must comply with Rust language rules (no panic outside `main.rs`, typed serde models, no magic literals).

**Scale/Scope**: First slice targets a single guardian rule end-to-end. Trust metrics scoped to workspace-local `.boundline/traces/` only (no cross-workspace aggregation). Override policies per-guardian-rule, not per-finding.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Delivery Identity | ✅ PASS | Calibration directly improves delivery reliability by making governance adoptable — teams can start with advisory controls and graduate to enforcement as trust builds. |
| II. Delivery-First Scope | ✅ PASS | Graduated enforcement reduces friction while maintaining safety; the feature answers "does it help deliver working code more reliably?" affirmatively. |
| III. No Abstract Agent Systems | ✅ PASS | Not a multi-agent system. Guardians are concrete delivery-step validators with calibrated enforcement levels. |
| IV. Bounded Execution | ✅ PASS | Every control level has explicit semantics (advisory=visible, catch=attention, rule=block+override, hook=block+privileged). Start/end conditions are defined. Boundline firmly owns the calibration engine boundaries without creating a secondary policy engine. |
| V. Validation, Failure Handling, Observability | ✅ PASS | **HR-001** requires that adaptive governance must be more explainable than static governance, not less. FR-010 defines structured events; 7 edge cases cover failure modes; SC-004/SC-005 require trace visibility; calibration policy fails closed on invalidity. |
| Language Rules: No Panic | ✅ PLAN | Implementation will use `Result<T, E>` and explicit error propagation. No `unwrap`, `expect`, `panic!`, or `todo!` outside `main.rs`. |
| Language Rules: No Magic Literals | ✅ PLAN | Control level names, event type strings, TOML keys, and thresholds will use named constants and typed enums. Calibration policy will have a typed serde model. |
| Language Rules: Typed Serde Models | ✅ PLAN | Calibration policy TOML, override records, trust records, and control level assignments will use typed `struct`/`enum` definitions with `serde` derives. |

## Project Structure

### Documentation (this feature)

```text
specs/075-adaptive-governance-calibration/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── domain/
│   └── calibration.rs           # NEW: CalibrationPolicy, ControlLevel, GuardianTrustRecord, OverrideRecord, degradation/escalation types
├── cli/
│   ├── override.rs              # NEW: `boundline override` command
│   ├── inspect.rs               # MODIFY: add control-level rendering
│   └── council.rs               # MODIFY: integrate calibration policy into adjudication
└── orchestrator/
    └── session_runtime_observability.rs  # MODIFY: emit calibration events

crates/boundline-core/src/
└── domain.rs                    # MODIFY: wire calibration module

tests/
├── contract/
│   ├── calibration_policy_contract.rs   # NEW
│   └── calibration_output_contract.rs   # NEW
├── integration/
│   └── calibration_flow.rs              # NEW
└── unit/
    └── calibration_model.rs             # NEW
```

**Structure Decision**: Single Rust workspace project. Domain types live in `src/domain/calibration.rs` (path-routed into `boundline-core` via `crates/boundline-core/src/domain.rs`). CLI commands follow the existing pattern in `src/cli/`. Orchestrator integration extends the existing council adjudication path and session runtime observability.

## Complexity Tracking

No constitution violations. No complexity justifications needed.
