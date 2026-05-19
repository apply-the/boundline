# S7 Delight-Boundline Contract

**Version**: 0.5.0  
**Effective Date**: 2026-05-17  
**Status**: Specification Phase (implementation follows in later features)

**Parties**: Boundline S7 runtime (consumer) ↔ Canon governance runtime (provider)

---

## I. Contract Overview

This contract defines explicit boundaries between Boundline-owned assistant delight surfaces and Canon-governed knowledge inputs. It establishes:

1. What Boundline OWNS (runtime decision logic, UX/CLI rendering, explanation vocabulary, operator command behavior)
2. What Boundline MAY CONSUME FROM CANON (governed artifacts, approval states, readiness signals, etc.)
3. HOW BOTH TEAMS maintain the boundary (explicit amendment procedures, validation rules, degradation handling)

**Core Commitment**: S7 delight surfaces remain trustworthy, auditable, and aligned with Canon governance without Boundline becoming dependent on Canon's availability or internal semantics.

---

## II. Boundline Owned Responsibilities

Boundline EXCLUSIVELY OWNS these capabilities. Canon MUST NOT attempt to govern or directly control them:

### UX & Rendering
- CLI formatting and layout of S7 explanations
- Chat assistant command behavior and conversation flow
- IDE extension display and interaction patterns
- Selection of which surfaces to render (e.g., risk vs. next-action emphasis)

### Explanation Vocabulary
- Definition and evolution of "risk", "assumption", "blocker", "confidence", etc.
- How explanations are worded and structured for operators
- Translation of Boundline runtime concepts into operator-facing language
- Customization per surface (CLI vs. chat vs. IDE)

### Runtime Decision Logic
- What execution traces to highlight in explanations
- Which evidence to prioritize when both Boundline and Canon contribute
- How to combine multiple evidence sources into coherent narratives
- Fallback behavior when Canon governance is unavailable

### Operator Command Handling
- Which commands trigger S7 surfaces
- How operator input is parsed and routed to delight handlers
- Error handling when S7 generation fails
- Rate limiting or caching decisions

### Internal State & Context
- Session state management for S7 interactions
- Caching of explanations for identical queries
- Trace buffering and compression for operator inspection
- Performance optimization within S7 generation

**Amendment**: Changes to owned responsibilities do NOT require Canon approval. Boundline may iterate on its owned surfaces independently (within the constraint that Canon inputs must remain explicitly sourced).

---

## III. Contracted Canon Inputs

Boundline MAY consume the following Canon-owned artifacts for S7 explanations. NO OTHER Canon inputs are authorized:

### 1. Packets (Promoted Governance Packets)
- **What**: Structured governance decision artifacts (change classification, approval status, stage progression)
- **When Available**: When Canon has promoted a packet for the current bounded task
- **Schema**: Governed by Canon's packet-structure spec
- **Metadata Required**: `promoted_at`, `promoted_by` (authority zone), `contract_version` (which S7 contract line this packet supports)
- **Max Age**: 1 month (older packets → stale degradation signal)
- **Use in S7**: Cite packet ID in "assumption" and "next-action" sections when packet governs the current stage

### 2. Approval States
- **What**: Review council approval/rejection verdicts for governance decisions
- **When Available**: After review council decision (if present in current Canon deployment)
- **Schema**: Enumeration (Approved, Rejected, PendingReview, AppealableDecision)
- **Metadata Required**: `decision_timestamp`, `reviewer_authority_zone`, `contract_version`
- **Max Age**: Approval decisions do not age; mark as stale only if packet it governs has been superseded
- **Use in S7**: Cite approval state in "confidence" sections to justify high confidence when governance agrees with runtime assessment

### 3. Readiness Signals
- **What**: Promotion-readiness verdicts (whether work is safe to promote to next stage)
- **When Available**: After readiness assessment (if present in current Canon deployment)
- **Schema**: Enumeration (Ready, NotReady, ReadyWithCaveats, PendingValidation)
- **Metadata Required**: `assessed_at`, `assessing_authority`, `contract_version`
- **Max Age**: 7 days (older readiness signals → stale degradation signal; re-run assessment)
- **Use in S7**: Cite in "blocker" sections when readiness says "NotReady", or in "confidence" when readiness confirms runtime assessment

### 4. Security Findings
- **What**: Security assessment results (if performed and promoted)
- **When Available**: After security scan completion and Canon promotion
- **Schema**: List of findings with severity, category, remediation guidance
- **Metadata Required**: `scan_date`, `scanner_version`, `contract_version`
- **Max Age**: 2 weeks (older scans → stale degradation signal; re-run scan)
- **Use in S7**: Cite in "risk" and "assumption" sections when security findings exist

### 5. Audit/Review Findings
- **What**: Audit trail findings from prior stages (if promoted and applicable)
- **When Available**: When prior work has completed review and findings are marked as relevant to current stage
- **Schema**: List of findings with context, severity, category
- **Metadata Required**: `finding_date`, `finding_authority`, `stage_applicable_to`, `contract_version`
- **Max Age**: 90 days (older findings → evaluate staleness in context)
- **Use in S7**: Cite in "assumption" sections when prior findings identify patterns relevant to current assessment

### 6. Promotion References
- **What**: Metadata identifying which Canon authority promoted an artifact and under which governance mode
- **When Available**: Attached to every other contracted input
- **Schema**: Authority zone identifier, promotion timestamp, stage identifier
- **Metadata Required**: `promoted_at`, `promoted_by`, `governance_mode`
- **Max Age**: N/A (metadata, not data)
- **Use in S7**: Include in source attribution to help operator understand "who governs this evidence"

---

## IV. Contracted Input Classes Summary

| Input Class | When Present | Max Age | S7 Vocabulary Use | Example |
|---|---|---|---|---|
| Packets | Always (if task is governed) | 1 month | assumption, next-action | "Canon promotes this to stage 'Review'" |
| Approval States | If council enabled | No age | confidence | "Review council approved this stage" |
| Readiness Signals | If assessment enabled | 7 days | blocker, confidence | "Readiness says NotReady, must address" |
| Security Findings | If security enabled | 2 weeks | risk, assumption | "Security found X vulnerability" |
| Audit Findings | If prior stage exists | 90 days | assumption | "Prior audit found Y pattern" |
| Promotion References | Always (metadata) | N/A | source attribution | "Promoted by security authority" |

---

## V. What Boundline MUST NOT Consume

The following Canon concepts and outputs are OUT OF SCOPE for S7:

- **Internal Canon state** (unreleased packets, internal stage names, internal authority calculations)
- **Predicted or speculative governance** (what Canon _might_ do based on trends)
- **Ambient Canon semantics** (implicit meanings that exist in Canon's implementation but are not explicitly contracted)
- **External system recommendations** (outputs from other tools integrated to Canon, unless explicitly promoted as governing)
- **Configuration parameters** (internal Canon settings that are not promoted artifacts)
- **Raw evidence** that Canon has filtered or governed (only use promoted summaries)

**Rationale**: Out-of-scope inputs risk creating divergence if Canon changes internal implementation without notice. Explicit contracts prevent this.

---

## VI. Degradation Handling

When contracted Canon inputs are missing, stale, incompatible, or contradictory, Boundline MUST:

### Missing Inputs
- Surface an explicit degradation signal (not silent fallback)
- Example: "Canon readiness signal not yet available; based on Boundline runtime evidence only, risk is Medium"
- Operator knows what is missing and can decide whether to wait or proceed

### Stale Inputs
- Surface the staleness in the degradation signal with recommended action
- Example: "Security scan is 3 weeks old (max age 2 weeks); recommend re-running security scan"
- Operator can choose to refresh the evidence before making decisions

### Incompatible Inputs
- Surface the incompatibility with a note about which contract line mismatch occurred
- Example: "Approval state was promoted for S7 contract 0.4.0, but current S7 contract is 0.5.0; consult release notes"
- Operator and maintainers can detect version misalignment

### Contradictory Inputs
- Surface the contradiction explicitly rather than merging conflicting signals
- Example: "Boundline runtime assessment: Risk=Low | Canon security findings: Risk=High | Conflict: must investigate"
- Operator sees that governance and runtime disagree and can decide which to trust

**Never**: Silently drop a degraded input, fabricate certainty, or merge contradictory signals without surfacing the conflict.

---

## VII. Extension Procedures

### Adding New Canon Input Classes

If S7 future capabilities require consuming a Canon input class NOT in this contract, follow:

1. **Proposal**: Create a GitHub issue in Boundline repo titled "S7 Contract Amendment: Add [InputClass]"
   - Describe the new capability and why it needs a new Canon input
   - List what metadata the input must carry
   - Explain max age and degradation behavior
   - Estimate implementation effort

2. **Cross-Repo Amendment**: Simultaneously propose the amendment to Canon repo (spec/057 or equivalent)
   - Canon documents what it commits to providing
   - Canon defines the metadata schema
   - Canon documents validation rules for promoting this input class to S7-safe status

3. **Bilateral Review**:
   - Boundline team reviews and approves the amendment in Boundline 060 spec
   - Canon team reviews and approves the amendment in Canon 057 spec
   - Both approvals must occur before merge (not sequential reviews)

4. **Release Coordination**:
   - Amendments are released in the same batch (same release date for Boundline and Canon)
   - OR amendments are released with explicit forward/backward compatibility guidance
   - Release notes MUST call out the new contract line version

5. **Deprecation Path** (for removing input classes):
   - Marked as deprecated in contract for 2 releases
   - Replaced with upgraded alternative (if applicable) or explicitly removed
   - Operator guidance provided on what to do if they depend on the deprecated input

### Example Amendment: Add "Promotion Readiness Assessment"

**Boundline 060 amendment**:
```
### 7. Promotion Readiness Assessment (NEW)
- What: Structured assessment of whether work is ready to promote
- When Available: [After readiness workflow]
- Max Age: 7 days
- Use: Cite in "assumption" when readiness assessment is available
```

**Canon 057 amendment**:
```
### 7. Promotion Readiness Assessment (NEW)
- What: Canon provides readiness assessment promoted for S7 consumption
- Metadata: assessed_at, assessing_authority, contract_version
- Validation: [Canon validation rules]
```

**Both must merge together** before the new contract line (e.g., 0.6.0) is released.

---

## VIII. Validation & Boundary Maintenance

### S7 Implementation Must Pass:

1. **Schema Validation**: Every S7 answer conforms to the explanation vocabulary (risk, assumption, blocker, etc.)
2. **Source Attribution**: Every claim cites exactly one source from contracted inputs or Boundline runtime
3. **Contract Boundary**: No answer uses Canon inputs outside the contracted classes
4. **Degradation Handling**: All degradation scenarios are explicitly surfaced
5. **Metadata Correctness**: Every cited Canon input includes required metadata (promoted_at, promoted_by, contract_version)

### Quarterly Boundary Review:

- Boundline maintainers audit S7 implementation against this contract
- Canon maintainers confirm promoted inputs match contract requirements
- Any drift from contract is addressed before next release
- Results logged in `validation-log.md` in this feature directory

### Amendment Tracking:

- Every contract amendment is recorded with timestamp, approvers, and release version
- Changes are visible in CHANGELOG.md in both repos
- Deprecated contract lines have explicit sunset guidance

---

## IX. Special Cases

### When Canon Is Unavailable
- S7 continues to work using only Boundline-owned runtime evidence
- Degradation signals indicate what Canon inputs would improve the answer
- Operator can proceed with confidence levels lowered but still useful

### When New S7 Surfaces Are Added
- They MUST use only contracted Canon inputs plus Boundline runtime evidence
- They MUST include degradation signals when appropriate
- They MUST pass contract validation before merge

### When Boundline And Canon Terminology Diverges
- Define translations in the amendment that adds the new surface
- Use S7 explanation vocabulary for operator-facing text
- Document what the equivalent Canon concept is in internal comments

### Multiple S7 Answers In Sequence
- Each answer is independently validated
- Source attribution preserves differences in available evidence across the sequence
- Degradation signals can improve (evidence becomes available) or worsen (evidence becomes stale)

---

## X. Governance & Amendment Authority

- **Amendment Proposal Authority**: Any Boundline or Canon maintainer
- **Approval Authority**: Boundline lead reviewer + Canon lead reviewer (bilateral)
- **Implementation Authority**: Boundline owns implementation; Canon owns promoted data quality
- **Dispute Resolution**: If amendment is contested, escalate to project leadership and document rationale

---

## XI. Contract History

| Version | Date | Change | Approved By |
|---|---|---|---|
| 0.5.0 | 2026-05-17 | Initial S7 contract definition | Specification Phase |
| (future) | (future) | Amendments per extension procedures | (bilateral) |

---

## XII. Cross-Reference

**Related Specs**:
- Boundline: `/specs/060-assistant-delight-layer/spec.md` (feature specification)
- Canon: `/specs/057-s7-delight-provider/spec.md` (provider side of contract)
- Both repos: Amendment tracking in respective CHANGELOG.md files

**Validation Tooling** (future):
- Schema validation script in Boundline repo (Phase 2 implementation)
- Cross-repo reference check in CI
- Maintainer checklist for quarterly boundary review
