# Implementation Plan: Browser And Visual Testing Provider

**Branch**: `082-browser-visual-testing-provider` | **Date**: 2026-06-20 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/082-browser-visual-testing-provider/spec.md`

## Summary

Add a browser capability provider to Boundline that enables Playwright-backed screenshot capture, console-error collection, DOM inspection, accessibility auditing, scripted interactions, and visual diff comparison. The provider communicates over the existing external capability provider protocol (S10) via JSON over stdio and produces session-scoped evidence packets with normalized findings. Boundline core does not embed a browser runtime — the provider is an external binary registered and activated through `.boundline/config.toml`.

## Technical Context

**Language/Version**: Rust 1.96.0, edition 2024 + external provider binary (language agnostic; reference implementation in Node.js or Rust over stdio)

**Primary Dependencies**: `serde`, `serde_json`, `thiserror`, `tracing`, `uuid`, `toml` (existing workspace crates `boundline-core`, `boundline-adapters`, `boundline-cli`). No new workspace dependencies.

**Storage**: Session-scoped artifact directory `.boundline/sessions/<id>/browser/<validation_run_id>/` with retention classes (`required_evidence`, `diagnostic`, `verbose`, `ephemeral`). Evidence packet references are workspace-relative. Existing `.boundline/session.json` and `.boundline/traces/` surfaces are extended additively.

**Testing**: `cargo test`, `cargo nextest run`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Provider contract tests validate JSON stdio request/response schemas.

**Target Platform**: macOS, Linux, Windows (CLI). Browser binary is operator-managed — Boundline never bundles or auto-installs it.

**Project Type**: CLI + library extension + external capability provider protocol consumer.

**Performance Goals**: Browser validation step <30s under normal network conditions (SC-001). Provider dispatch and evidence normalization <50ms overhead. Artifact hashing and reference generation <100ms per artifact.

**Constraints**:
- Browser automation MUST NOT be embedded in the Boundline core runtime (Hard Rule, FR-021)
- JSON over stdio transport in V1; future transports gated by protocol evolution
- Network permission policy is a static allowlist; dynamic resolution deferred
- Artifact retention reuses existing session/trace model; no browser-specific cleanup subsystem
- Secrets, credentials, tokens, cookies MUST be redacted before durable artifact storage
- Provider does not auto-retry; retryability hints are advisory only

**Scale/Scope**: Single-URL validation in V1 (P1). DOM inspection, accessibility, interaction scripts, and visual diff are additive P2/P3 capabilities. Auth handling, remote browser execution, and dynamic network policy are deferred.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Verdict | Notes |
|-----------|---------|-------|
| I. Delivery Identity | ✅ PASS | Browser validation produces delivery evidence (screenshots, console errors, accessibility findings) |
| II. Delivery-First Scope | ✅ PASS | Answers "does it help deliver working code?" — catches frontend regressions CLI-only validation misses |
| III. No Abstract Agent Systems | ✅ PASS | Provider is a concrete validation tool, not a multi-agent framework |
| IV. Bounded Execution | ✅ PASS | Configurable timeouts (readiness, execution, queue), max concurrency, bounded queue size — every operation has explicit limits |
| V. Stateful Execution | ✅ PASS | Evidence packets, findings, artifact references all persisted to session-scoped storage |
| VI. Mutable Planning | N/A | Validation provider — not a planning feature |
| VII. Execution Over Perfect Planning | ✅ PASS | P1 ships single-URL screenshot + console capture before complex interaction scripts |
| VIII. Sequential-First Design | ✅ PASS | Sequential step execution; concurrency queue is explicit FIFO with bounded waiting — no implicit parallelism |
| IX. Tool-Agent Symmetry | ✅ PASS | Browser actions (navigate, click, type, screenshot) are explicit tools with observable outcomes in evidence packets |
| X. Required Observability | ✅ PASS | Evidence packets, structured findings, artifact references, retryability hints, trace output — all inspectable (FR-012, FR-025, SC-007) |
| XI. No Hidden Intelligence | ✅ PASS | Retryability hints are advisory and separately traceable; all routing/fallback decisions are explicit |
| XII. Strict Non-Goals | ✅ PASS | No councils, no distributed agents, no UI work, no provider abstraction beyond S10 protocol |
| XIII. Minimal Capability Slices | ✅ PASS | P1 (single-URL screenshot + console) is independently shippable and delivers immediate value |
| XIV. Real Acceptance Criteria | ✅ PASS | 15 scenarios across 3 stories, each with Given/When/Then, success AND failure paths |
| XV. Failure as First-Class Path | ✅ PASS | 12 finding categories covering timeouts, readiness failures, accessibility scan failures, script step failures, queue timeout, queue-full, concurrency timeout — all with explicit handling |
| XVI. Separation From External Systems | ✅ PASS | Provider independently testable; no Canon dependency; evidence CAN link to Canon but feature works without it |
| XVII. Evolution Without Premature Lock-In | ✅ PASS | Finding categories extensible, capability advertisement updatable, retention classes additive, transport gated by protocol version |
| XVIII. Done Means Executable Delivery | ✅ PASS | 8 measurable SCs covering completion time, failure handling, accessibility detection rate, diff accuracy, script execution, network policy enforcement, Canon linkage |

**Gate Result**: ALL PASS (2 N/A). No violations requiring Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/082-browser-visual-testing-provider/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (provider stdio protocol contract)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
  boundline-core/
    src/
      domain/
        browser_provider.rs        # NEW: BrowserValidationStep, EvidencePacket, BrowserFinding
        session.rs                 # MODIFY: additive browser artifact ref fields
        observability.rs           # MODIFY: BrowserValidationCompleted trace event type
        capability_provider.rs     # MODIFY: "browser" capability kind
      lib.rs                       # MODIFY: pub mod browser_provider
  boundline-adapters/
    src/
      browser_provider_runtime.rs  # NEW: JSON stdio dispatch, finding normalization
      browser_artifact_store.rs    # NEW: artifact writing, hashing, retention
  boundline-cli/
    src/
      cli.rs                       # MODIFY: `boundline validate browser` subcommand
      cli/
        validate_browser.rs        # NEW: dispatch, render evidence/findings
        inspect_browser.rs         # NEW: inspect evidence packets, artifacts

tests/
  contract/
    browser_provider_protocol.rs   # NEW: JSON stdio request/response schema
  integration/
    browser_provider_cli.rs        # NEW: end-to-end provider dispatch
  unit/
    browser_provider_types.rs      # NEW: EvidencePacket, BrowserFinding serialization
```

**Structure Decision**: New domain module `browser_provider.rs` in `boundline-core` owns provider message types, finding categories, and evidence packet schema. Adapter logic for stdio dispatch and artifact management lives in `boundline-adapters`. CLI extensions are additive. The reference browser provider binary is a standalone project — the Boundline workspace has no dependency on Playwright or any browser automation library.

## Complexity Tracking

No violations. All 16 applicable constitution principles pass without justification needed.
