# Implementation Plan: Review Councils And Role-Gated Governance

**Branch**: `074-review-councils-governance` | **Date**: 2026-06-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/074-review-councils-governance/spec.md`

## Summary

Add a guardian activation router (loading versioned TOML rules from `.boundline/guardian-rules.toml` with built-in fallbacks and fail-closed validation) and a single-adjudicator review council (`boundline council adjudicate` CLI) that examines guardian findings and produces a binary clean/blocked decision with full trace visibility.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024

**Primary Dependencies**: Existing workspace crates only; `clap`, `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml`, `boundline-core`, `boundline-adapters`, `boundline-cli`

**Storage**: Read-only inspection of `.boundline/session.json`, `.boundline/guardian-rules.toml` (new ruleset file), guardian finding traces; write-only council decision and structured events via the existing observability event log

**Testing**: `cargo test --test unit`, `cargo test --test contract`, `cargo test --test integration`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`

**Target Platform**: Local CLI runtime; `--json` output for CI/automation

**Project Type**: Rust workspace; one CLI command, one domain module, one new config file

**Performance Goals**: Ruleset loading and validation <100ms; activation plan production <500ms; council adjudication <1s

**Constraints**: Read-only routing; fail-closed on invalid ruleset or missing mandatory guardian evidence; no automatic adjudication in V1; no multi-member voting

**Scale/Scope**: One CLI command (`boundline council adjudicate`), one TOML config file, two new structured event types (`guardian.activation.plan.produced`, `council.decision.produced`), 4+ built-in routing rules

## Constitution Check

*GATE: Must pass before Phase 0 research.*

| Principle | Result | Evidence |
|-----------|--------|----------|
| Delivery identity | PASS | Councils improve delivery quality by catching regressions before execution. |
| No abstract agent systems | PASS | Deterministic rule matching and single-adjudicator decision; no voting simulation. |
| Bounded execution | PASS | Ruleset loaded once; activation plan computed in one pass; adjudication is one synchronous decision. |
| Stateful execution | PASS | Structured events and trace-visible council decisions persist to the event log. |
| Sequential-first | PASS | Single CLI command; no concurrent guardian execution. |
| Required observability | PASS | Structured events for activation plan and council decision; trace-visible rule matching and adjudication rationale. |
| No hidden intelligence | PASS | All routing is explicit TOML rules; adjudication rationale is recorded per finding. |
| Separation from external systems | PASS | Council and router are Boundline-owned; no Canon dependency. |
| Rust language rules | PASS | Typed enums for states/outcomes, named constants for rule keys, `toml` crate for ruleset parsing. |

## Project Structure

### Documentation

```text
specs/074-review-councils-governance/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── council-output-contract.md
└── tasks.md
```

### Source Code

```text
src/
├── domain/
│   └── council.rs              # GuardianActivationPlan, GuardianRule, CouncilDecision types + router logic
├── cli/
│   └── council.rs              # `boundline council adjudicate` CLI command + output rendering
└── orchestrator/
    └── session_runtime_observability.rs  # Event emission hooks (extend)

.boundline/
└── guardian-rules.toml         # Versioned TOML ruleset
```

## Complexity Tracking

> No constitution violations to justify.

## Post-Design Constitution Recheck

*Re-checked after Phase 1 design outputs.*

All 9 constitution gates pass post-design. The design adds no dependency, network call, background process, or hidden inference. Guardian routing is deterministic TOML-based rule matching; council adjudication is single-reviewer with full trace visibility.
