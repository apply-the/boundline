# S7 Explanation Vocabulary

**Version**: 0.5.0  
**Effective Date**: 2026-05-17  
**Purpose**: Standardized terminology for S7 cognitive affordance explanations across all surfaces (CLI, chat, IDE)

---

## Core Vocabulary

### Risk

**Definition**: What could go wrong with this plan or decision, according to all available evidence sources.

**When To Use**: When answering "what are the risks?", "what could break?", "what should I worry about?"

**Sources That Contribute**: Boundline runtime checks, Canon security findings, Canon audit findings

**Conflict Handling**: If Boundline and Canon agree on risk level, cite both. If they disagree, surface the conflict explicitly with both assessments.

**Example Answer**:
> Risk: **Medium**
> - Boundline runtime detects timing dependency (LOW-confidence)
> - Canon security findings report unpatched dependency (HIGH-confidence)
> - **Conflict**: Timing risk is uncertain, but dependency risk is confirmed. Recommend addressing dependency first.
> - Missing evidence: Security remediation timeline from Canon

**Confidence Mapping**:
- Low Risk: Few or uncertain blockers identified
- Medium Risk: Identified blockers are manageable or have mitigations
- High Risk: Critical blockers without clear mitigation

---

### Assumption

**Definition**: What must remain true for this plan to succeed, identified from evidence.

**When To Use**: When answering "what must be true for this to work?", "what are we assuming?", "what could make this fail?"

**Sources That Contribute**: Boundline runtime analysis, Canon packets (governing assumptions), Canon audit findings

**Conflict Handling**: If Boundline and Canon assumptions differ, surface the disagreement as a potential blocker.

**Example Answer**:
> Assumptions: **3 critical**
> 1. Operator will have access to security tooling (Boundline runtime detection)
> 2. Current promotion packet is applicable (Canon governance packet, confirmed by approval council)
> 3. Prior audit findings about integration points will not resurface (Canon audit finding, but 90 days old—staleness warning issued)

**Confidence Mapping**:
- Each assumption carries its own confidence based on evidence freshness and source

---

### Blocker

**Definition**: Something that prevents forward progress in the current decision or plan, with identified source authority.

**When To Use**: When answering "what's stopping us?", "why can't we proceed?", "what needs to happen first?"

**Sources That Contribute**: Canon readiness signals, Canon approval states (rejections), Boundline runtime safety checks

**Conflict Handling**: If Boundline says "proceed" but Canon readiness says "blocked", surface as a governance override that operator must acknowledge.

**Example Answer**:
> Blockers: **1 critical**
> - Readiness Assessment: **NOT READY** (stale, 8 days old, max 7 days)
>   - Authority: Canon readiness-assessment authority
>   - Recommended Action: Re-run readiness assessment to update signal
>   - Boundline alternative: Runtime checks show no critical issues, but deferring to governed readiness assessment

**Confidence Mapping**:
- Blockers are either present or absent (binary), but confidence in assessment varies by evidence freshness

---

### Confidence

**Definition**: How certain this assessment is, based on the completeness and freshness of available evidence.

**When To Use**: When answering "how sure are we about this?", "can we trust this assessment?", "what would make us more confident?"

**Sources That Contribute**: All sources proportionally weighted by recency and authority

**Conflict Handling**: If evidence sources conflict, confidence is reduced and conflicts are itemized.

**Example Answer**:
> Confidence: **MEDIUM** (was High, now reduced due to conflict)
> - Evidence Quality: Partial (Canon readiness stale, 8 days old)
> - Evidence Recency: Mixed (Boundline runtime is current, Canon governance is stale)
> - Caveats:
>   - Readiness assessment must be refreshed to validate current state
>   - One prior audit finding was categorized as minor but remains unresolved
> - Impact: Proceed if risk tolerance allows, but prioritize re-assessment

---

### Next Action

**Definition**: What step the operator should take next, with identified source authority and rationale.

**When To Use**: When answering "what should I do?", "what's the next step?", "what happens now?"

**Sources That Contribute**: Canon packet promotion recommendations, Boundline runtime orchestration, Canon approval state implications

**Conflict Handling**: If governance and runtime recommend different next actions, surface both with authority labels.

**Example Answer**:
> Next Action: **Refresh Readiness Assessment**
> - Primary (Canon packet): Packet recommends waiting for readiness assessment update
> - Alternative (Boundline runtime): Runtime checks would allow proceeding with caution
> - Recommendation: Follow governance guidance; readiness assessment is quick (~5 minutes)

---

### Missing Evidence

**Definition**: Information that is not currently available but would improve the confidence or completeness of this assessment.

**When To Use**: When answering "what would make this better?", "what's missing?", "where can I get more information?"

**Sources That Contribute**: Derived from absence of contracted Canon inputs or gaps in Boundline runtime visibility

**Example Answer**:
> Missing Evidence: **2 items**
> 1. **Security Findings** (Canon security scope)
>    - Why needed: Would clarify whether timing risk affects security
>    - Time to availability: ~10 minutes (scheduled security scan)
>    - Recommended action: Run `boundline security run` and re-check assessment
>
> 2. **Prior Remediation Status** (Canon audit scope)
>    - Why needed: Confirms whether prior findings have been addressed
>    - Time to availability: Asynchronous (Canon audit team review, ~24 hours)
>    - Alternative: Proceed assuming prior findings were addressed

---

## Vocabulary Combinations

### Valid Combinations (How Terms Relate)

```
Assessment Request
  ├─ Risk + Confidence                    ✓ (risk level + certainty)
  ├─ Risk + Missing Evidence              ✓ (risk + what would clarify it)
  ├─ Assumption + Blocker                 ✓ (what's needed + what's stopping it)
  ├─ Blocker + Next Action                ✓ (what's stopping it + how to proceed)
  ├─ Confidence + Caveats                 ✓ (certainty + conditions affecting it)
  ├─ Next Action + Missing Evidence       ✓ (what to do + what info would help)
  └─ All terms together                   ✓ (comprehensive assessment)
```

### Conflict Scenarios (Handled Specially)

| Scenario | Example | How To Handle |
|----------|---------|---------------|
| Boundline Risk vs. Canon Risk | Runtime says "Low", Security says "High" | Surface both with sources, note conflict |
| Boundline Next vs. Canon Next | Runtime suggests "continue", Packet says "wait" | Cite canonical authority, note alternative |
| Assumption conflict | Runtime: "external system available", Audit: "system was unavailable last quarter" | Surface as blocker + caveat |
| Blocker resolved | Readiness was "NOT READY", now "READY" | Confidence improves; surface that state changed |
| Evidence staleness | Readiness signal is 8 days old (max 7 days) | Surface staleness in confidence/caveat |

---

## Surface-Specific Rendering

### CLI Rendering

```
$ boundline plan --s7-explain risk

❯ Risk Assessment

  ● MEDIUM RISK

  Factors:
    • Timing dependency detected [Boundline: Low confidence]
    • Unpatched dependency [Canon security: High confidence, 2 days old]

  ⚠ Conflict: See caveats

  Confidence: MEDIUM (was High before conflict detected)
    - Why: Evidence quality is partial; one source conflicts
    - Caveats: Dependency remediation timeline unknown

  Suggested Action: Address dependency first; re-check assessment
```

### Chat Assistant Rendering

```
**Risk: MEDIUM**

Based on current evidence:
- Boundline detected a timing dependency (uncertain impact)
- Canon security findings show an unpatched library (confirmed risk)

These sources conflict about overall risk level. I recommend addressing the security finding first, then re-assessing the timing impact.

What would help: Canon timeline for security remediation.
```

### IDE Inline Rendering

```
Risk: Medium ⚠ (conflict detected)
├─ Timing: Low (Boundline)
├─ Security: High (Canon security, 2d old)
└─ Action: Resolve security, re-assess
```

---

## Reserved Vocabulary (Do Not Extend Without Amendment)

The following terms have specific meanings in S7 and MUST NOT be repurposed or extended without formal contract amendment:

- **Risk, Assumption, Blocker, Confidence, Next Action, Missing Evidence**: Core vocabulary
- **Degradation Signal**: Indicates unreliable or missing evidence (do not conflate with low confidence)
- **Source Attribution**: Indicates whether evidence came from Boundline, Canon, or is missing
- **Artifact Class**: One of: packets, approval states, readiness signals, security findings, audit findings, promotion references

Any new terminology requires amendment to this vocabulary document plus changes to the supporting data model and validation schemas.

---

## Evolution Path

### For Adding New Terms

If future S7 capabilities require new vocabulary:

1. **Proposal**: Document in S7 contract amendment why new term is needed
2. **Boundary Check**: Confirm term is about delight/explanation (owned by Boundline), not governance (owned by Canon)
3. **Example Answers**: Provide example S7 answers using the new term
4. **Conflict Handling**: Define how the new term relates to existing terms and surfaces conflicts
5. **Bilateral Approval**: Amend Boundline 060 spec + Canon 057 spec in same release

### For Deprecating Terms

If future S7 simplification removes terms:

1. **Deprecation Period**: 2 releases where old term is still rendered but marked as deprecated
2. **Migration Guidance**: Document how old term maps to new vocabulary
3. **Example Migration**: Show operators what old answers look like vs. new vocabulary
4. **Removal**: After deprecation period, old term is removed and no longer rendered

---

## Validation Rules

Every S7 explanation MUST:

✅ Use only vocabulary from this document  
✅ Include at least one primary term (Risk, Assumption, Blocker, Confidence, Next Action)  
✅ Include Missing Evidence if confidence is Low or Medium  
✅ Include source attribution (Boundline, Canon, or missing) for every claim  
✅ Surface conflicts explicitly rather than merging contradictory sources  
✅ Include degradation signals when contracted inputs are stale/missing/incompatible  
✅ Map confidence level to available evidence quality and recency  

---

## Metadata

**File Location**: `specs/060-assistant-delight-layer/contracts/assistant-delight-explanation-vocabulary.toml`  
**Maintenance**: Updated per S7 contract amendment procedures  
**Tooling**: Referenced by S7 runtime for validation and rendering  
**Distribution**: Bundled with Boundline release; available for external tool integration
