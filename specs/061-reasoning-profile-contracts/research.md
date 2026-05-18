# Phase 0 Research: Governed Reasoning Profile Contracts

**Date**: 2026-05-18  
**Feature**: 061-reasoning-profile-contracts  
**Status**: Resolved all planning unknowns

## Overview

This research phase resolves the architectural and cross-repo decisions needed
to implement S6 as one coherent contract layer instead of a second governance
system. The decisions below keep Boundline runtime-owned, keep Canon posture-
owned, and preserve independent testability in the Boundline repository.

## Research Findings

### R-001: Split Ownership Between Boundline Runtime And Canon Posture

**Unknown**: Where should the S6 contract live so advanced reasoning is
bilateral without making Canon the runtime owner?

**Decision**:
- Boundline owns the executable reasoning-profile contract, activation logic,
  participant selection, disagreement handling, profile-level cost exposure,
  confidence contribution, trace events, and session-visible summaries.
- Canon owns the challenge-posture contract that states when stronger challenge
  is required, what minimum profile family or independence floor applies, and
  which contract line Boundline may consume.
- Boundline consumes Canon posture only through one explicit consumer contract in
  `specs/061-reasoning-profile-contracts/contracts/canon-challenge-posture-consumer-contract.md`.
- Canon publishes the authoritative provider-side contract in the sibling repo at
  `docs/integration/governed-reasoning-posture-contract.md`.

**Rationale**: This split keeps the runtime and the semantic authority in their
existing homes. Boundline stays the execution engine; Canon stays the posture
author.

**Alternatives Considered**:
- Put all reasoning orchestration in Canon: Rejected because it would turn Canon
  into a second control plane.
- Keep the contract entirely inside Boundline: Rejected because Canon would no
  longer own challenge posture or compatibility semantics.

### R-002: Represent Profiles As Typed Runtime Policy, Not As A Second Workflow

**Unknown**: How should reasoning profiles integrate with the current runtime
without creating a parallel orchestration system?

**Decision**:
- Add a typed `ReasoningProfileDefinition` model in Boundline and treat profile
  activation as a bounded subroutine attached to an existing governed stage.
- Keep the outer workflow session-native and sequential-first.
- Reuse existing governance and review state instead of adding a second task or
  second workflow family.
- Support the V1 profile vocabulary directly: `bounded_self_consistency`,
  `independent_pair_review`, `heterogeneous_security_review`,
  `bounded_reflexion`, plus controlled debate rounds as an exceptional
  challenge mechanism.

**Rationale**: S6 is about how challenge is executed inside governance, not
about creating a new operator story.

**Alternatives Considered**:
- New top-level reasoning workflow: Rejected because it duplicates session,
  planning, and terminal-state behavior already owned by Boundline.
- Reuse review profiles with no new reasoning type: Rejected because S6 needs
  self-consistency, reflexion, and debate concepts that are not expressible as
  review-only metadata.

### R-003: Reuse Existing Route Slots And Add Profile Roles Locally

**Unknown**: Does S6 need new global routing slots for every reasoning role?

**Decision**:
- Keep the existing top-level routing slots (`planning`, `implementation`,
  `verification`, `review`, `adjudication`) unchanged.
- Introduce profile-local participant roles such as `independent_path`,
  `blind_reviewer`, `heterogeneous_reviewer`, `critic`, `reviser`, and
  `arbiter` inside the reasoning-profile model.
- Resolve each participant role through existing slot routes and role-specific
  overrides when available.
- Use explicit independence checks over effective routes, provider family,
  context basis, and prompting pattern rather than new global slot names.

**Rationale**: Existing routing already captures the operator-owned route story.
S6 needs role assignment and independence guards, not a new routing taxonomy.

**Alternatives Considered**:
- Add new route slots per reasoning role: Rejected because it would expand the
  operator config surface before the minimal contract is proven.
- Hardcode roles to specific models: Rejected because it would undermine
  provider neutrality and local routing policy.

### R-004: Carry Confidence Through One Additive Handoff Model

**Unknown**: How should profile-level disagreement or convergence affect the
existing governance confidence model?

**Decision**:
- Add a typed `ReasoningConfidenceContribution` model in Boundline.
- Record confidence contribution as additive evidence to the existing governance
  confidence path rather than introducing a second trust model.
- Surface at least these inputs: profile type, independence result,
  convergence/disagreement class, adjudication result, budget exhaustion, and
  interruption state.
- Keep final acceptance authority and governance stop semantics unchanged.

**Rationale**: S6 explicitly says more reasoning does not imply correctness and
must feed S4 confidence, not replace it.

**Alternatives Considered**:
- Standalone reasoning-confidence score: Rejected because it would create a
  competing trust system.
- Ignore confidence handoff: Rejected because S6 would remain disconnected from
  governance decisions.

### R-005: Extend The Existing Trace Model With Additive Reasoning Events

**Unknown**: What is the smallest trace extension that still makes S6
inspectable?

**Decision**:
- Extend `TraceEventType` with additive reasoning-profile events rather than
  creating a separate trace family.
- Planned events: profile activation, participant started/completed,
  convergence or disagreement recorded, debate round completed, reflexion
  revision completed, adjudication recorded, confidence contribution recorded,
  and profile blocked or escalated.
- `status`, `next`, and `inspect` summarize the latest profile lifecycle and
  point operators to the next bounded action.

**Rationale**: Operators already inspect one session and one trace. S6 must
remain visible inside that same story.

**Alternatives Considered**:
- Separate reasoning trace file: Rejected because it hides control flow and
  complicates inspection.
- Summary-only projection with no event detail: Rejected because it would make
  debate, reflexion, and degradation behavior opaque.

### R-006: Keep Boundline Independently Testable With Local Posture Fixtures

**Unknown**: How can Boundline satisfy the constitution's external-separation
rule while still validating a bilateral Canon contract?

**Decision**:
- Boundline unit and integration tests use local posture fixtures and local
  reasoning-profile scenarios.
- Cross-repo contract tests read the sibling Canon contract when available but
  fall back to a feature-local fixture snapshot when the sibling repo is absent.
- Runtime behavior must remain executable with Canon unavailable, treating
  missing Canon posture as an explicit unsupported or degraded input rather than
  as a crash or hidden fallback.

**Rationale**: Boundline cannot require the Canon repo to execute its core
tests, but it still needs a bilateral alignment gate before release.

**Alternatives Considered**:
- Always require the Canon repo during tests: Rejected because it would violate
  independent testability.
- Never read the sibling Canon repo: Rejected because real drift between the two
  contracts would go undetected.

### R-007: Use Explicit Version Windows For The First Bilateral Release

**Unknown**: What release targets should the first task and compatibility tests
pin?

**Decision**:
- Boundline ships this feature as `0.61.0`.
- Canon ships the matching provider-side contract as `0.57.0`.
- The first task in `tasks.md` will bump both repo version anchors and update
  compatibility tests that assert the supported pair.
- The final tasks will close docs, roadmap, changelog, clippy, and coverage
  verification for both repos.

**Rationale**: Making the version window explicit in Phase 0 prevents the task
plan from hiding release drift as cleanup.

**Alternatives Considered**:
- Defer version decisions to polish: Rejected because the user explicitly wants
  version bumps and version tests to anchor the task list.
- Keep Canon version unspecified: Rejected because the bilateral contract has no
  executable compatibility story without it.

## Unknowns Resolved

- Ownership boundary: Boundline runtime, Canon posture
- Runtime carrier: typed profile model inside the existing session runtime
- Routing strategy: reuse existing slots, add profile-local roles
- Confidence strategy: additive handoff into existing governance confidence
- Trace strategy: additive reasoning events in the current trace model
- Test strategy: local fixtures plus optional sibling Canon contract alignment
- Version strategy: Boundline `0.61.0`, Canon `0.57.0`

## Design Readiness

Phase 0 research is complete. Phase 1 design can now proceed to:
1. Define the typed data model for profile activation, participant topology,
   independence assessment, outcome, and confidence contribution.
2. Write the Boundline runtime, trace, and Canon consumer contracts.
3. Document a quickstart that exercises activation, inspection, mismatch, and
   release validation scenarios.