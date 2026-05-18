# Phase 1 Design: S7 Data Model

**Date**: 2026-05-17  
**Feature**: 060-assistant-delight-layer  
**Status**: Design inputs generated for runtime implementation

## Overview

The S7 data model defines the formal entities and relationships that support source-attributed explanations, explicit degradation handling, and contract boundary maintenance. These entities are consumed by S7 runtime surfaces (CLI, chat assistants, IDE extensions) to render explanations while preserving source attribution.

For 060, this model is the Boundline runtime projection model consumed by
assistant commands, `inspect` lenses, and contextual diagnostics.

---

## Core Entities

### S7 Explanation

**Purpose**: The primary output entity representing a single source-attributed answer to an operator query.

**Definition**:
```
S7Explanation:
  id: UUID                          # Unique explanation identifier
  query: String                     # Operator's original question
  timestamp: DateTime               # When explanation was generated
  execution_context: ExecutionRef   # Which bounded task/session produced this
  
  answer_sections: [AnswerSection]  # Structured breakdown of the response
  
  source_attribution: SourceBreakdown  # Explicit source accounting
  confidence_level: ConfidenceAssessment  # Bounded confidence with rationale
  degradation_signals: [DegradationSignal]  # Any incompatibilities or absences
  missing_evidence: [EvidenceGap]   # What would improve this answer
  
  validation_result: ContractValidation  # Whether answer stays within contract
```

**Fields**:

- **answer_sections**: Modular breakdown of the explanation, each with its own source attribution. Enables rendering flexible explanation depths.
- **source_attribution**: Explicit mapping of each claim to its source (Boundline runtime, Canon artifact class, missing evidence).
- **confidence_level**: Bounded range (Low/Medium/High) with explicit evidence rationale. Prevents false certainty.
- **degradation_signals**: List of compatibility issues (stale Canon input, contract line mismatch, missing governance). Visible to operator.
- **validation_result**: Contract adherence check; passed/failed/degraded with audit trail.

**Validation Rules**:
- Every claim in answer_sections MUST cite exactly one source from SourceBreakdown
- Confidence_level MUST be justified by presence/absence of evidence cited
- Degradation signals MUST be present when any Canon input is absent/stale/incompatible
- No answer section MAY cite a Canon artifact class outside the contracted list

---

### Answer Section

**Purpose**: A modular component of an explanation with its own claim, evidence, and source attribution.

**Definition**:
```
AnswerSection:
  claim: String                     # The substantive claim being made
  claim_type: AnswerType            # Risk, Assumption, Blocker, Confidence, NextAction, MissingEvidence
  
  contributing_evidence: [Evidence]  # All evidence that informed this claim
  supporting_rationale: String      # Why this claim follows from the evidence
  
  source_breakdown: SourceBreakdown  # Which evidence came from which source
  confidence: ConfidenceAssessment   # Bounded confidence for this specific claim
```

**Fields**:

- **claim_type**: Enumeration (Risk, Assumption, Blocker, NextAction, Confidence, MissingEvidence) that maps to the standardized S7 vocabulary
- **contributing_evidence**: All evidence that was considered, including negative evidence (things that ruled out other claims)
- **source_breakdown**: Separates Boundline-owned evidence from Canon-governed evidence from missing evidence

**Validation Rules**:
- Claim MUST be expressible in one of the standardized vocabulary types
- Supporting_rationale MUST be explicit; no implicit jumps from evidence to claim
- Source_breakdown MUST be present and complete; no unattributed evidence

---

### Source Breakdown

**Purpose**: Explicit accounting of which sources contributed to an answer section.

**Definition**:
```
SourceBreakdown:
  boundline_sources: [BoundlineSource]     # Runtime judgment, workspace evidence, traces
  canon_sources: [CanonSource]              # Governed artifacts consumed
  missing_sources: [MissingEvidence]        # What information would improve this
  
  source_completeness: SourceCompleteness  # Assessment of evidence sufficiency
```

**Fields**:

- **boundline_sources**: Runtime evidence owned by Boundline (execution traces, task state, operator input)
- **canon_sources**: Governed artifacts explicitly cited; each must be from contracted input class
- **missing_sources**: Evidence gaps that would improve confidence if available
- **source_completeness**: Enumeration (Complete, Partial, Degraded) indicating whether all necessary sources were accessible

**Validation Rules**:
- Each Canon source MUST cite its contract line version and promoted timestamp
- Each Canon source MUST be from the contracted input classes (packets, approval states, readiness signals, etc.)
- If any Canon source is stale/incompatible, source_completeness MUST be Degraded
- Missing sources MUST be named explicitly (e.g., "security_findings not yet promoted")

---

### Canon Source Reference

**Purpose**: Explicit citation of a single governed artifact consumed from Canon.

**Definition**:
```
CanonSource:
  artifact_class: ArtifactClass      # packets, approval_states, readiness_signals, etc.
  artifact_id: UUID                   # Unique identifier within Canon workspace
  promoted_timestamp: DateTime        # When Canon promoted this artifact
  authority_zone: AuthorityZone       # Which Canon authority endorsed this
  contract_line: SemanticVersion      # S7 contract version this artifact was promoted for
  
  compatibility_status: CompatibilityStatus  # Valid, Stale, OutOfContract, Missing
  degradation_reason: Option<String>  # Why compatibility status != Valid
```

**Fields**:

- **artifact_class**: Enumeration matching the contract-defined input classes
- **contract_line**: Allows detection of obsolete or incompatible artifact versions
- **compatibility_status**: Enumeration (Valid, Stale, OutOfContract, Missing) enabling degradation visualization
- **degradation_reason**: Explicit statement of why this source is unreliable (e.g., "promoted 3 months ago, current policy is 1 month maximum")

**Validation Rules**:
- artifact_class MUST be in the contracted list
- contract_line MUST match current S7 contract or be within a known deprecation window
- If compatibility_status != Valid, degradation_reason MUST be populated
- Timestamp MUST be recent enough per contract degradation windows (e.g., packets must be less than 1 month old)

---

### Degradation Signal

**Purpose**: Explicit notification of a condition that affects the reliability of the explanation.

**Definition**:
```
DegradationSignal:
  signal_type: DegradationType        # Stale, Missing, Incompatible, Contradictory, OutOfContract
  affected_source: String              # Which source has the issue (artifact_class or "Canon unavailable")
  severity: Severity                   # High (unusable), Medium (partial), Low (informational)
  description: String                  # Human-readable problem statement
  recommended_action: String           # What operator should do (wait, re-run, use fallback, etc.)
  timestamp: DateTime                  # When degradation was detected
```

**Fields**:

- **signal_type**: Enumeration covering all degradation scenarios identified in spec
- **severity**: Allows rendering critical issues prominently while keeping informational degradations visible
- **recommended_action**: Explicit guidance rather than leaving operator to guess

**Validation Rules**:
- If ANY degradation signal exists with severity=High, explanation MUST NOT claim certainty
- Degradation signals MUST be rendered to operator (never silent)
- Recommended_action MUST be actionable (not vague)

---

### Confidence Assessment

**Purpose**: Bounded confidence level with explicit evidence basis.

**Definition**:
```
ConfidenceAssessment:
  level: ConfidenceLevel              # Low, Medium, High
  rationale: String                    # Why this confidence level given the evidence
  evidence_quality: EvidenceQuality    # Complete, Partial, Degraded
  evidence_recency: Duration           # How recent is the newest contributing evidence
  caveats: [String]                   # Explicit conditions that could invalidate this assessment
```

**Fields**:

- **level**: Bounded to three discrete levels to prevent false precision
- **rationale**: Explicit explanation of how evidence maps to confidence
- **evidence_quality**: Indicates whether complete evidence set was available
- **evidence_recency**: Indicates staleness of underlying data
- **caveats**: Conditions that would lower confidence (e.g., "assumes promoted artifacts remain accurate")

**Validation Rules**:
- Confidence=High MUST have evidence_quality=Complete AND no Degradation signals with severity > Low
- Confidence=Medium MUST have evidence_quality=Complete OR current recency
- Confidence=Low is acceptable with any evidence_quality but MUST have explicit rationale
- Caveats MUST be specific (not "things could change")

---

### Evidence Gap

**Purpose**: Explicit naming of missing evidence and its impact.

**Definition**:
```
EvidenceGap:
  evidence_type: String               # What information is missing (e.g., "security_findings")
  source: SourceType                  # Boundline runtime, Canon governed, external
  why_needed: String                  # How this evidence would improve the answer
  impact_if_available: ConfidenceImpact  # Would change confidence from X to Y
  where_to_find: Option<String>       # Guidance on obtaining this evidence
  time_to_availability: Option<Duration>  # Estimated wait time if waiting for data
```

**Fields**:

- **evidence_type**: Specific, nameable gap (not vague "more data")
- **source**: Identifies which system could provide this evidence
- **impact_if_available**: Shows operator why they should care about this gap
- **where_to_find**: Actionable guidance (e.g., "run security scan: boundline security run")
- **time_to_availability**: Realistic expectation (e.g., "~5 minutes for security scan")

**Validation Rules**:
- evidence_type MUST correspond to actual available evidence sources
- why_needed MUST be specific to this explanation (not generic)
- impact_if_available MUST show a meaningful change (not marginal)

---

### Contract Validation Result

**Purpose**: Audit trail of whether an explanation stayed within contract boundaries.

**Definition**:
```
ContractValidation:
  passed: bool                        # Whether explanation adheres to contract
  contract_version: SemanticVersion   # Which S7 contract version was applied
  validation_timestamp: DateTime      # When validation occurred
  
  violations: [ContractViolation]     # If passed=false, what was violated
  warnings: [ContractWarning]         # Issues that did not block but warrant review
  audit_entries: [ValidationAuditEntry]  # Full trace of validation steps
```

**Fields**:

- **violations**: Non-compliance that should block the answer
- **warnings**: Issues flagged for maintainer review (e.g., using un-tested vocabulary combinations)
- **audit_entries**: Queryable trace of how validation proceeded

**Validation Rules**:
- If violations.len() > 0, passed MUST be false
- Contract_version MUST match S7 contract at time of explanation generation
- Audit_entries MUST include at least: input source check, vocabulary check, degradation handling check

---

## Relationships

```
S7Explanation
  ├─ contains multiple AnswerSections
  │   ├─ each has SourceBreakdown
  │   │   ├─ references CanonSources (if any)
  │   │   ├─ references BoundlineSources (if any)
  │   │   └─ lists MissingEvidence (if any)
  │   ├─ has ConfidenceAssessment
  │   └─ may have EvidenceGaps
  │
  ├─ has SourceBreakdown (aggregate)
  │   ├─ all CanonSources across all sections
  │   └─ completeness assessment
  │
  ├─ contains DegradationSignals (if any)
  │   └─ each references affected source
  │
  └─ has ContractValidation
      ├─ audit trail of checks
      └─ violations or warnings (if any)
```

---

## Serialization

### JSON Schema (at runtime)

```json
{
  "s7_explanation": {
    "id": "uuid",
    "query": "what are the risks?",
    "answer_sections": [
      {
        "claim_type": "Risk",
        "claim": "...",
        "source_breakdown": {
          "boundline_sources": [...],
          "canon_sources": [
            {
              "artifact_class": "approval_states",
              "contract_line": "0.5.0",
              "compatibility_status": "Valid"
            }
          ],
          "source_completeness": "Complete"
        },
        "confidence": {
          "level": "High",
          "rationale": "..."
        }
      }
    ],
    "degradation_signals": [],
    "missing_evidence": [],
    "validation": {
      "passed": true,
      "contract_version": "0.5.0"
    }
  }
}
```

### TOML Config (contract parameters)

Stored in `s7-explanation-vocabulary.toml`:
```toml
[vocabulary.Risk]
definition = "What could go wrong with this plan according to all evidence sources"
sources = ["Boundline", "Canon", "MissingEvidence"]
conflicts = "Contradictory signals surface explicitly"

[vocabulary.Assumption]
# ...

[degradation_modes]
Stale = { max_age_months = 1, signal_severity = "Medium" }
Missing = { signal_severity = "High", recommended_action = "Check Canon status" }
# ...
```

---

## Extensibility

The data model supports contract amendment without redesign:

1. **New answer types**: Add to AnswerType enum and vocabulary.toml; existing code automatically recognizes them
2. **New Canon input classes**: Add to ArtifactClass enum and contract-validation logic
3. **New degradation modes**: Add to DegradationType enum and rendering logic
4. **New evidence gaps**: No schema change; evidence_type is free-form text matching actual evidence sources

All extensions MUST go through the amendment procedures documented in `s7-extension-procedures.md` to maintain bilateral agreement.

---

## Design Readiness for Phase 1

✅ Core entities defined with validation rules
✅ Relationships specified with clear ownership
✅ Serialization formats chosen (JSON runtime, TOML config)
✅ Extensibility paths documented
✅ Ready for contract artifact generation (data-model complete)
✅ Ready for quickstart documentation (entity reference complete)

**Next Phase 1 steps**:
1. Generate `contracts/s7-delight-contract.md` with full contract definition
2. Create `s7-explanation-vocabulary.toml` with standardized terms
3. Generate `s7-input-classes.json` with artifact class schema
4. Document degradation modes and handling rules
5. Document extension procedures
6. Update agent context with S7 terminology
