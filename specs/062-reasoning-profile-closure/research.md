# Phase 0 Research: Reasoning Profile Closure

**Date**: 2026-05-18  
**Feature**: 062-reasoning-profile-closure  
**Status**: Resolved all planning unknowns

## Overview

This research phase resolves how to finish S6.1 honestly without reopening the
Canon posture boundary or introducing a second orchestration system. The key
decisions below keep the closure work bounded, release-ready, and aligned with
the existing `061` runtime contract.

## Research Findings

### R-001: Close Residual Concrete Profiles Through The Existing Runtime

**Unknown**: How should Boundline finish `independent_pair_review`,
`heterogeneous_security_review`, and `bounded_reflexion` without creating a new
reasoning workflow?

**Decision**:
- Reuse the `061` reasoning-profile activation path in `session_runtime.rs`.
- Add the missing positive-path and bounded non-success evidence through the
  existing session-native `run`, `status`, `inspect`, and trace surfaces.
- Keep the same concrete profile identifiers rather than minting replacement
  profile ids.

**Rationale**: S6.1 is closure work, not a new runtime layer. The smallest
honest slice is to prove the already-declared concrete profile ids end-to-end.

**Alternatives Considered**:
- Create a new top-level reasoning workflow: Rejected because it would violate
  the sequential-first and bounded-runtime design already used by Boundline.
- Narrow all three profiles back to fixture-only support: Rejected because it
  would discard already-delivered runtime vocabulary and operator surfaces.

### R-002: Treat Debate As Bounded Substrate Unless A Standalone Profile Is Required

**Unknown**: Should debate ship as a standalone V1 reasoning profile in order to
close S6.1?

**Decision**:
- Keep debate classified as bounded substrate rather than a standalone shipped
  V1 profile.
- Remove or update any release-facing language that implies standalone debate
  shipment.
- Preserve debate-round trace vocabulary and bounded iteration support as shared
  substrate used by concrete profiles when needed.

**Rationale**: The roadmap explicitly allows debate to remain substrate if that
is the honest answer. Promoting it to a standalone profile would broaden scope
without improving the core delivery slice.

**Alternatives Considered**:
- Promote debate to a standalone profile id now: Rejected because it would add a
  new shipped profile without distinct end-to-end delivery value beyond the
  existing bounded reasoning substrate.

### R-003: Treat Adjudication As A Shared Primitive Unless Runtime Evidence Forces Promotion

**Unknown**: Should adjudication become a standalone shipped profile or remain a
shared primitive inside other profile executions?

**Decision**:
- Keep adjudication classified as a shared primitive used by concrete profiles.
- Ensure runtime, trace, inspect, roadmap, and validation artifacts state that
  classification explicitly.
- Preserve adjudication event vocabulary and operator-visible resolution output
  when concrete profiles use it.

**Rationale**: Adjudication already behaves as a disagreement-resolution step
shared across concrete profiles. A standalone shipped profile would expand the
delivery surface without new operator value for this closure pass.

**Alternatives Considered**:
- Promote adjudication to a standalone shipped profile: Rejected because it
  would create a larger feature than the audited carry-forward requires.

### R-004: Require Canon Companion Publication Alignment, Not Canon Runtime Changes

**Unknown**: Does closing S6.1 require a new Canon runtime or contract shape?

**Decision**:
- Canon changes are required for this closure because Boundline `0.62.0`
  changes the published supported pair.
- Those Canon changes stay limited to published compatibility docs, tests,
  changelog, and version-window statements needed to align the new release
  pair.
- No new Canon runtime control flow or posture schema is introduced.

**Rationale**: S6.1 explicitly excludes reopening the Canon posture boundary.
The only valid companion work is release-alignment and published compatibility.

**Alternatives Considered**:
- Leave Canon untouched while changing the supported Boundline release pair:
  Rejected because the bilateral contract story would drift.
- Add new Canon runtime behavior: Rejected because it violates the scope and
  would create a second execution dependency.

### R-005: Resolve Maintainability Findings By Extracting Pure Validation Helpers

**Unknown**: How should the closure slice address the release-blocking
maintainability findings in `SessionStatusView::validate_governance` and
`assess_reasoning_independence`?

**Decision**:
- Refactor `SessionStatusView::validate_governance` into smaller helper checks
  grouped by projection category instead of one large conditional chain.
- Refactor `assess_reasoning_independence` into helper functions that compute
  distinctness, evaluate gap conditions, and assemble the operator-facing
  reason separately.
- Preserve behavior and validation semantics exactly while reducing cognitive
  complexity below the repository threshold.

**Rationale**: The user explicitly called out the maintainability issues, and
the release bar now treats them as blocking for this feature.

**Alternatives Considered**:
- Suppress the code-quality rule: Rejected because the repository constitution
  and user guidance require fixing the owned logic instead of weakening the
  gate.
- Leave the issues to later cleanup: Rejected because the user requested a full
  residual closure, not another carry-forward slice.

### R-006: Boundline Ships As 0.62.0 And Canon Companion Publishes 0.58.0

**Unknown**: What release version strategy keeps the closure work honest and
compatible?

**Decision**:
- Boundline targets `0.62.0` for this closure slice.
- Canon targets `0.58.0` for the required companion publication update.
- Boundline-local fallback artifacts must validate the same `0.62.x`/`0.58.x`
  published pair when the sibling Canon repository is unavailable.

**Rationale**: The user requested version bumps in the closure cycle. Boundline
is unquestionably changed, and the Canon stable contract currently publishes
`supported_boundline_window = 0.61.x`, so a truthful `0.62.0` release requires
the companion publication update.

**Alternatives Considered**:
- Keep Boundline at `0.61.0`: Rejected because this is a new shipped feature
  slice.
- Leave Canon on `0.57.x` while Boundline ships `0.62.0`: Rejected because the
  published provider contract would continue to advertise the wrong supported
  Boundline window.

### R-007: Validation Must End With Repository-Level Confidence, Not Focused Tests Alone

**Unknown**: What validation bar is required before declaring S6.1 finished?

**Decision**:
- Use focused unit, integration, and contract tests during development.
- Finish with repository-level validation: `cargo fmt --check`, full-workspace
  clippy, full-workspace or target-appropriate tests, and refreshed
  `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`.
- Treat the existing SonarCloud quality workflow in `.github/workflows/quality.yml`
  as the maintainability proof point for the touched cognitive-complexity
  findings, using the repository rule threshold currently reported in the
  findings.
- Update docs and changelogs in the same closure pass and treat drift as a
  release blocker.

**Rationale**: S6.1 is only done when the shipped claim, runtime behavior, and
release artifacts all agree.

**Alternatives Considered**:
- Stop after passing focused tests: Rejected because the user explicitly wants
  clippy, coverage, docs, and changelog closure in the same pass.

## Unknowns Resolved

- Concrete profile closure remains inside the existing runtime
- Debate remains bounded substrate
- Adjudication remains a shared primitive
- Canon companion publication and version alignment are required for the new
  release pair
- Maintainability findings are fixed by helper extraction, not suppressions
- Version strategy is Boundline `0.62.0`, Canon `0.58.0`
- Validation must finish with repository-level checks and refreshed `lcov.info`

## Design Readiness

Phase 0 research is complete. Phase 1 design can proceed to:
1. Define the closure-specific classification, evidence, and release-alignment
   entities.
2. Publish explicit closure and release-alignment contracts for the final
   shipped claim set.
3. Document quickstart scenarios that exercise concrete profile closure,
   primitive classification, and release-quality validation.