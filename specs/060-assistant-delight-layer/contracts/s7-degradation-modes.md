# S7 Degradation Modes and Handling Rules

**Version**: 0.5.0  
**Effective Date**: 2026-05-17  
**Purpose**: Define how S7 surfaces handle degraded, missing, stale, incompatible, or contradictory Canon inputs

---

## Overview

S7 MUST NEVER silently drop evidence, fabricate certainty, or merge contradictory signals without surfacing the conflict. This document specifies how each degradation scenario is handled operator-visibly.

---

## Degradation Mode: MISSING

**Scenario**: Operator requests an explanation that would benefit from a contracted Canon input class, but that input is not available.

**Examples**:
- Operator asks "what are the risks?" but security findings have not been promoted yet
- Operator asks "am I ready to proceed?" but readiness assessment has not been run
- Operator asks "what does governance say?" but no promotion packet exists for this task

### S7 Handling Rules

| Severity | Condition | Signal | Confidence Impact | Recommended Action |
|----------|-----------|--------|-------------------|-------------------|
| **Low** | Nice-to-have input not available (e.g., audit findings from prior stage exist but are not critical) | Display as "Missing Evidence: Audit findings (optional)" | None (already accounting for optional inputs) | None required; operator can proceed |
| **Medium** | Important input not available but Boundline runtime can still make assessment (e.g., readiness signal missing but runtime checks pass) | Display degradation with caveat that governance input would improve confidence | Lower confidence one level (High→Medium or Medium→Low) | Operator can proceed but should re-check when input becomes available |
| **High** | Critical input not available and Boundline cannot confidently assess (e.g., security findings missing when security is mandatory) | Display degradation signal with severity=HIGH; do not render answer without explicit warning | Set confidence to LOW; state reason explicitly | Operator must act to obtain missing evidence or acknowledge risk |

### Message Templates

**Low**: 
```
Missing: Audit findings from prior stage (informational only)
  → Optional detail; your assessment remains confident
```

**Medium**: 
```
⚠ Missing: Security findings (assessment quality reduced)
  → Security scan not yet available
  → Recommended: Re-check after running `boundline security run`
  → Status: Proceeding with Boundline runtime evidence; confidence reduced to MEDIUM
```

**High**: 
```
🛑 BLOCKED: Readiness assessment unavailable (critical)
  → Required for proceeding to next stage
  → Action: Run `boundline readiness assess --current-stage` to generate assessment
  → Current confidence: LOW (cannot reliably assess without governance input)
```

---

## Degradation Mode: STALE

**Scenario**: A contracted Canon input exists but exceeds its maximum age without refresh.

**Examples**:
- Readiness assessment is 8 days old (max age 7 days)
- Security findings are 3 weeks old (max age 2 weeks)
- Audit findings are 120 days old (original max age 90 days)

### S7 Handling Rules

| Age Status | Input Class | Signal Severity | Confidence Impact | Recommended Action |
|---|---|---|---|---|
| **Current** | Any | (no signal) | None | None |
| **Near Expiry** (80-100% of max age) | Any | INFO: "This assessment is approaching expiration" | None yet | Suggest refresh to operator |
| **Stale** (100-150% of max age) | readiness_signals | MEDIUM: "Readiness assessment is stale" | Lower confidence one level | Re-run readiness assessment before proceeding |
| **Stale** (100-150% of max age) | security_findings, audit_findings | MEDIUM to HIGH: "Security/audit information is outdated" | Lower confidence one level | Re-run scan/audit for current assessment |
| **Very Stale** (>150% of max age) | Any | HIGH: "This evidence is too old to rely on" | Set confidence to LOW | Disregard stale evidence and re-collect current data |

### Message Templates

**Near Expiry**:
```
ℹ Readiness assessment will expire in 2 days (expiry window: 7 days from promotion)
  → Consider refreshing soon: `boundline readiness assess --current-stage`
```

**Stale** (Medium):
```
⚠ Security findings are 3 weeks old (max age: 2 weeks)
  → Findings: [list of findings]
  → Confidence impact: MEDIUM → LOW (outdated security info)
  → Recommended: Re-run security scan with `boundline security run --update`
```

**Very Stale** (High):
```
🛑 Audit findings are 120 days old (max age: 90 days)
  → This evidence is too old to include in risk assessment
  → If prior findings are still relevant, they must be re-verified by audit team
  → Current assessment: Using only Boundline runtime evidence (confidence MEDIUM)
```

---

## Degradation Mode: INCOMPATIBLE

**Scenario**: A contracted Canon input exists and is not stale, but was promoted for a different S7 contract version.

**Examples**:
- Approval state was promoted for S7 contract 0.4.0, but current S7 contract is 0.5.0
- Packet carries unknown contract_version field that doesn't map to known versions
- Security findings were promoted before the S7 contract was established

### S7 Handling Rules

| Situation | Signal Severity | Confidence Impact | Recommended Action |
|---|---|---|---|
| **Known but Older Version** (one major version back) | MEDIUM | Lower confidence one level | Check release notes for semantic changes; if compatible, proceed with caveat |
| **Unknown or Too Old Version** (two+ major versions back) | HIGH | Set confidence to LOW | Do not use this evidence; re-generate it under current contract |
| **Future Version** (contract version > current S7 version) | HIGH | Set confidence to LOW | Check if Boundline needs upgrade; if not, do not use evidence |

### Message Templates

**Known Older Version**:
```
⚠ Approval state promoted under S7 contract 0.4.0 (current: 0.5.0)
  → Compatibility: Likely compatible, but verify release notes
  → Risk: Semantic meanings may have evolved between versions
  → Confidence: MEDIUM (from HIGH due to version mismatch)
  → Action: Review /CHANGELOG.md for contract changes; proceed if no breaking changes
```

**Incompatible Version**:
```
🛑 Security findings promoted under S7 contract 0.2.0 (current: 0.5.0)
  → Compatibility: Too old; schema and semantics likely incompatible
  → Action: Re-run security scan under current contract version
  → Current assessment: Disregarding security findings; Boundline runtime only (confidence LOW)
```

---

## Degradation Mode: CONTRADICTORY

**Scenario**: Boundline runtime evidence and Canon governed evidence assess the same dimension but reach different conclusions.

**Examples**:
- Boundline runtime says "no blockers" but Canon readiness assessment says "NOT READY"
- Boundline risk assessment says "Low Risk" but Canon security findings say "High Risk"
- Boundline next-action says "proceed" but approval council rejected the proposal

### S7 Handling Rules

**Rule 1: Surface the Contradiction**
- NEVER silently favor one source over the other
- NEVER merge contradictory assessments into a false middle ground
- Display both assessments with their sources clearly labeled

**Rule 2: Prioritize Governance Authority**
- When Boundline and Canon conflict, Canon governance authority takes precedence for decision-making
- But Boundline alternative must be visible for operator awareness
- Operator must explicitly acknowledge the governance override before proceeding

**Rule 3: Investigate Root Cause**
- Contradictions often signal missing evidence or misunderstanding
- If contradiction exists, elevate "missing evidence" suggestions
- Example: If runtime and Canon security conflict, recommend asking security team for clarification

### Message Templates

**Conflict: Readiness**:
```
🔄 GOVERNANCE OVERRIDE

Boundline Assessment: ✓ Ready (no blockers, all checks pass)
Canon Readiness: ✗ NOT READY (governance authority determination)

Conflict: What to do next?
  → Governance authority is Canon. Follow readiness assessment.
  → Option: Contact readiness assessment authority to understand their concerns
  → Impact: Proceeding despite NOT READY signals operator accepts governance risk

Recommended: Do not proceed until readiness assessment is updated to READY or READY_WITH_CAVEATS
```

**Conflict: Risk Assessment**:
```
🔄 EVIDENCE CONFLICT

Boundline Risk: LOW (runtime checks find no critical issues)
Canon Security: HIGH (unpatched dependency identified)

Conflict: Which assessment is more reliable?
  → Both are accurate for their domain: runtime (process) vs. security (artifact content)
  → Merged Assessment: MEDIUM RISK (security issue takes precedence)
  → Recommended: Dependency patch must be applied before proceeding
  → Re-assess after patch applied
```

**Conflict: Next Action**:
```
🔄 DIVERGENT RECOMMENDATIONS

Boundline Suggests: Proceed to next stage
Canon Authority: Wait for approval council decision

Conflict: Who decides?
  → Canon approval council is governing authority
  → Status: Awaiting approval council decision (submitted for review)
  → Estimated timeline: [from Canon]
  → Alternative: Ask approval council for expedited review if urgent
```

---

## Degradation Mode: OUT OF CONTRACT

**Scenario**: S7 attempts to consume a Canon input that is not in the contracted input classes list.

**Examples**:
- S7 tries to use "prediction confidence" field that is not a contracted input class
- S7 references "internal stage names" which are out of scope
- S7 consumes "raw evidence" that Canon has filtered but not formally promoted

### S7 Handling Rules

**Detection**: Validation layer catches out-of-contract consumption before answer is rendered

**Handling**:
1. **Block the Answer**: Do not render explanation using out-of-contract inputs
2. **Surface the Issue**: Display clear error message to operator
3. **Suggest Amendment**: Point maintainers to contract amendment procedures
4. **Fallback**: Render answer using only in-contract inputs if possible

### Message Templates

```
❌ ANSWER NOT AVAILABLE (Contract Violation)

S7 attempted to use an input class that is not in the current contract:
  → Requested: "internal_stage_name" (not contracted)
  → Current contract: packets, approval_states, readiness_signals, security_findings, audit_findings, promotion_references

Reason: Boundline delight layer cannot consume uncontracted Canon concepts to prevent divergence.

Options:
  1. Propose contract amendment to add this input class
    → See: specs/060-assistant-delight-layer/contracts/s7-extension-procedures.md
  2. Use a contracted input class instead (e.g., stage from packet)
  3. Use Boundline runtime evidence only

Maintainers: This indicates S7 runtime implementation has drifted from contract. Review /CHANGELOG.md for recent S7 updates.
```

---

## Degradation Signal Rendering Priority

When multiple degradation conditions exist, render them in this order:

1. **High-Severity Blockers First** (🛑): Out of contract, missing critical inputs, very stale
2. **Medium Conflicts** (🔄): Contradictory sources, contract version mismatch
3. **Lower Warnings** (⚠): Stale but usable, missing optional inputs, near expiry
4. **Informational** (ℹ): Metadata only, no confidence impact

**Combined Example**:
```
🛑 MAJOR ISSUES (must resolve before proceeding):
  • Readiness assessment missing (required for governance)
  • Security findings conflict with Boundline assessment

🔄 CONFLICTS (review with authority):
  • Approval state from older contract version (0.4.0) — verify compatibility

⚠ MINOR WARNINGS (for awareness):
  • Audit findings approaching expiration (4 days remaining)

ℹ INFORMATION:
  • Evidence from 3 sources: Boundline (current), Canon packet (2 days old), Security (1 week old)
```

---

## Confidence Mapping Under Degradation

```
Degradation State              → Confidence Impact
─────────────────────────────────────────────────
No degradation                 → Use full assessment confidence
Missing optional inputs        → No change
Missing important inputs       → Lower one level (High→Medium, Med→Low)
Missing critical inputs        → Set to LOW
Stale but recent               → No change
Near expiration                → No change (but flag warning)
Stale (1-1.5x max age)        → Lower one level
Very stale (>1.5x max age)    → Set to LOW
Incompatible (older version)   → Lower one level
Incompatible (unknown version) → Set to LOW
Contradictory with governance  → Lower one level + add caveat
Out of contract                → Block answer or set to LOW if fallback available
```

---

## Validation Checklist

Every S7 answer MUST pass:

- [ ] **No Silent Failures**: If any degradation exists, it is surfaced
- [ ] **No False Certainty**: If confidence is High, all critical inputs are present and current
- [ ] **Explicit Sources**: Every claim cites which source(s) contributed
- [ ] **Conflict Visible**: Contradictions between Boundline and Canon are explicit, not merged
- [ ] **Authority Preserved**: Governance authority assessments take precedence (with operator awareness)
- [ ] **Recommendations Clear**: If degradation, what should operator do next?

---

## Amendment Path

To modify degradation handling:

1. Proposal in Boundline 060 + Canon 057 specs
2. Describe new degradation scenario and recommended handling
3. Update data model and validation rules if needed
4. Bilateral review and approval
5. Release with updated contract version number
