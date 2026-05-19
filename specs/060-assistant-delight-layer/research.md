# Phase 0 Research: S7 Assistant Delight Layer

**Date**: 2026-05-17  
**Feature**: 060-assistant-delight-layer  
**Status**: Resolved all NEEDS CLARIFICATION markers

## Overview

This research phase addresses technical unknowns around the S7 contract architecture, cross-repo coordination, and validation strategies. All findings now serve as implementation inputs for the Boundline-side runtime and assistant-surface feature.

---

## Research Findings

### R-001: Bidirectional Contract Architecture

**Unknown**: How do Boundline and Canon maintain synchronized contract boundaries when each repository has independent versioning and release cycles?

**Finding**: Bidirectional contracts use cross-spec references and synchronized task tracking rather than centralized coordination.

**Decision**: 
- Boundline spec (060) defines what Boundline CONSUMES from Canon (input classes, metadata requirements)
- Canon spec (057) defines what Canon PROVIDES to Boundline (artifact provision, compatibility signaling)
- Both specs are updated atomically when amendments occur; amendment procedures are documented in both repos
- Cross-repo references use explicit spec paths: `../canon/specs/057-s7-delight-provider/spec.md`
- Contract amendment requires PR review in both repos before merge

**Rationale**: Bidirectional specs prevent implicit drift because both teams must affirmatively agree to changes. No hidden ambient semantics can enter the contract without bilateral approval.

**Alternatives Considered**: 
- Single authoritative spec in Canon: Rejected because it would make Boundline dependent on Canon's spec release cycle; Boundline must own its own consumption boundaries.
- Synchronized versioning system: Rejected because it adds infrastructure complexity; explicit cross-spec references are simpler.

---

### R-002: S7 Contract Scope

**Unknown**: What specific Canon artifacts should the S7 contract permit Boundline to consume?

**Finding**: Contract permits bounded consumption of governed artifacts already committed to in prior features (S3-S5).

**Decision**: S7 may consume only:
- **Packets**: Promoted Canon governance packets for the current bounded task
- **Approval States**: Review council approval/rejection states (from S7 canon-contracts, requires explicit listing)
- **Readiness Signals**: Promotion-readiness verdicts (from S7 canon-contracts)
- **Security Findings**: Security assessment results (if promoted and available)
- **Review Findings**: Audit trail findings (if promoted and available)
- **Promotion References**: Metadata identifying which stage promoted an artifact

Every consumed class MUST carry:
- Contract line identifier (versioning)
- Promoted timestamp
- Authority zone (which Canon authority endorsed this)
- Degradation conditions (under what circumstances it becomes unreliable)

**Rationale**: Limiting to already-governed inputs keeps the contract stable and reuses existing Canon semantics. New input classes require formal contract amendment.

**Alternatives Considered**:
- Allow ambient Canon concepts: Rejected because it prevents maintainer review and enables silent drift.
- Consume only high-level readiness: Rejected because it reduces S7 explanation quality and forces Boundline to duplicate what Canon already knows.

---

### R-003: Degradation Signaling Strategy

**Unknown**: How does Boundline know when Canon inputs are stale, incompatible, or outside the contracted scope?

**Finding**: Canon provides explicit compatibility signals with every governed artifact; Boundline renders degradation visibly.

**Decision**: 
- Every Canon artifact consumed by S7 carries a `compatible_with_contract_line` field stating which S7 contract version it was promoted for
- When Boundline S7 receives an artifact, it compares `compatible_with_contract_line` against its own contract version
- Mismatch → degradation signal sent back to operator (not silent fallback)
- Missing artifact → explicit "Canon input not yet available" surface, not blank answer
- Contradiction detected → explicit "conflicting signals from Boundline and Canon" rather than merged signal

S7 NEVER:
- Silently drops a degraded input and pretends to have sufficient evidence
- Fabricates certainty when Canon input is absent
- Merges contradictory signals and hides the conflict

**Rationale**: Delight that hides degraded governance is more dangerous than no delight. Transparency about sources and degradation keeps operators informed.

**Alternatives Considered**:
- Cache old inputs: Rejected because stale cached evidence is worse than transparent absence.
- Silent fallback to Boundline evidence: Rejected because operators cannot distinguish "Canon unavailable" from "Canon agrees with Boundline."

---

### R-004: Extension Procedures

**Unknown**: How do Boundline and Canon add new capabilities to S7 without creating silent divergence?

**Finding**: Formalized amendment procedures with bilateral review gates.

**Decision**: 
- Any new S7 explanation category that would consume a Canon input class NOT already in the contract requires amendment
- Amendment proposal goes into both Boundline 060 spec and Canon 057 spec as new section
- Both specs must be approved simultaneously (same release batch or adjacent approved releases)
- Failed amendments are visible in task logs (not silently rejected)
- Deprecated contract lines receive a 2-release deprecation window with fallback guidance

Amendment procedure document (assistant-delight-extension-procedures.md) will specify:
- Who can propose amendments (any maintainer)
- Review criteria (contract boundary preserved, degradation handling defined)
- Approval gate (must pass in both repos)
- Changelog entries (visible in both CHANGELOG.md files)

**Rationale**: Explicit procedures prevent creeping scope and keep both teams accountable. Visible rejections are better than silent out-of-contract behavior.

**Alternatives Considered**:
- Implicit allowance (S7 consumes anything Canon provides): Rejected because it enables uncontrolled divergence.
- Single-team approval: Rejected because amendments that lack bilateral buy-in lead to misalignment during implementation.

---

### R-005: S7 Explanation Vocabulary

**Unknown**: Should S7 use a standard set of explanation terms across all surfaces (CLI, chat, IDE)?

**Finding**: Yes, standardized vocabulary prevents operator confusion and enables consistent source attribution.

**Decision**: Canonical explanation vocabulary defined in `assistant-delight-explanation-vocabulary.toml` with these key terms:
- **Risk**: What could go wrong with this plan, according to all evidence sources
- **Assumption**: What must remain true for this plan to succeed, from Boundline or Canon
- **Blocker**: What prevents forward progress, with explicit source (Boundline runtime check or Canon authority zone)
- **Confidence**: How certain is this assessment (bounded to low/medium/high with evidence rationale)
- **Next Action**: What step should the operator take, from Boundline orchestrator or Canon authority
- **Missing Evidence**: What information would improve the answer, if available

Each term includes:
- Definition (unambiguous, operator-facing)
- Which sources can contribute (Boundline only, Canon only, or both)
- How conflicts are rendered (if Boundline and Canon give different assessments)

**Rationale**: Consistent vocabulary across surfaces reduces cognitive load and prevents the same concept from being named differently depending on which interface the operator uses.

**Alternatives Considered**:
- Surface-specific vocabulary: Rejected because operators switching between CLI and chat would see confusing terminology drift.
- Minimal vocabulary (just "risk"): Rejected because it loses useful semantic distinction (assumptions vs. blockers).

---

### R-006: Validation and Boundary Maintenance

**Unknown**: How do maintainers verify that S7 stays within the contracted boundary over time?

**Finding**: Three validation layers: schema validation, cross-spec checks, and maintainer review gates.

**Decision**: 
- **Schema validation** (automated): Every S7 answer must conform to the explanation vocabulary schema and cite exactly which inputs contributed to each claim
- **Cross-spec integrity** (CI): Automated check that Boundline 060 spec and Canon 057 spec reference the same contract line versions and input classes
- **Maintainer validation** (manual): Periodic (quarterly or per-amendment) review that S7 implementation has not consumed ungoverned concepts or drifted from the documented contract
- **Traceability audit** (per session): Every S7 explanation includes a trace showing which inputs fed into which conclusions, queryable by operator

Validation failures are recorded in a `validation-log.md` artifact in the feature directory.

**Rationale**: Layered validation catches divergence early (schema), prevents spec misalignment (cross-spec checks), and maintains human accountability (maintainer review).

**Alternatives Considered**:
- Automated-only validation: Rejected because contract boundaries are social/semantic, not just syntactic.
- No validation: Rejected because unmaintained contracts become historical documents rather than living boundaries.

---

### R-007: Cross-Repo Test Strategy

**Unknown**: How are Boundline S7 surfaces tested for contract adherence when Canon inputs are not available in the test environment?

**Finding**: Layered testing with schema mocks and integration fixtures.

**Decision**: 
- **Unit tests** (Boundline repo only): Test S7 explanation generation using fixed mock Canon inputs matching the contract schema
- **Contract tests** (per S7 feature task): Validate that each explanation surface uses only contracted input classes and vocabulary
- **Integration fixtures** (in specs/060-assistant-delight-layer/): Provide reusable test data representing valid, stale, missing, and contradictory Canon inputs so both Boundline and Canon tests can use the same scenarios
- **Cross-repo validation** (CI): Before merge, verify that the proposed S7 surfaces would pass the contract validation checks against the current Canon spec

Test data lives in `specs/060-assistant-delight-layer/test-fixtures/canon-input-scenarios.json` to keep scenarios synchronized.

**Rationale**: Teams can test independently using shared fixtures, yet verify bidirectional compatibility before merge.

**Alternatives Considered**:
- Tight coupling (S7 tests import Canon test code): Rejected because it makes Boundline dependent on Canon's build system.
- No cross-repo testing: Rejected because divergence would be discovered only in production.

---

## Unknowns Resolved

✅ **Contract architecture**: Bidirectional specs with cross-references  
✅ **Input classes**: Bounded to already-governed Canon artifacts  
✅ **Degradation signaling**: Explicit, never silent  
✅ **Extension procedures**: Formalized bilateral review  
✅ **Explanation vocabulary**: Standardized terms in TOML  
✅ **Validation strategy**: Schema + cross-spec + maintainer review  
✅ **Testing approach**: Unit tests + contract tests + fixtures  

---

## Design Readiness

Phase 0 research complete. All technical unknowns are resolved and documented. Phase 1 design can now proceed to:
1. Generate `data-model.md` with formal entity definitions
2. Create `contracts/assistant-delight-contract.md` with complete contract specification
3. Define `assistant-delight-explanation-vocabulary.toml` and `assistant-delight-input-classes.schema.json` schemas
4. Document extension procedures and validation rules
5. Update agent context to reflect new S7 contract terminology
