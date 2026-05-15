# Guardian Rule Seeds

## Purpose

This document seeds initial Guardian capabilities for S2.1.

Guardians produce structured findings. They do not modify code directly.

## Rule Strength

Each rule should declare a default strength:

- info
- recommendation
- concern
- warning
- blocker

The operational consequence is determined later by S3 and S4.

## Initial Guardians

## clean-code-guardian

Rules:
- intent-revealing-names
- no-mixed-responsibilities
- no-hidden-side-effects
- no-primitive-obsession
- explicit-boundary-validation
- log-or-return

Default kind:
- llm or hybrid

## architecture-boundary-guardian

Rules:
- dependency-direction
- domain-infrastructure-separation
- public-contract-stability
- data-ownership-boundary
- integration-contract-required

Default kind:
- hybrid

## testability-guardian

Rules:
- deterministic-time
- no-hardwired-external-service
- behavior-testability
- missing-safety-net
- brittle-mocks

Default kind:
- hybrid

## rust-zero-panic-guardian

Rules:
- no-unwrap-in-domain-code
- no-expect-in-library-code
- no-panic-in-production-path
- no-slice-indexing-without-check

Default kind:
- deterministic or hybrid

## ts-runtime-validation-guardian

Rules:
- validate-external-input
- no-any-at-boundary
- no-duplicated-schema-and-type
- no-unchecked-json-parse

Default kind:
- deterministic or hybrid

## Structured Finding Contract

Required fields:

```json
{
  "guardian": "string",
  "rule": "string",
  "disposition": "info|observation|concern|warning|risk|blocker|error",
  "summary": "string",
  "evidence_refs": ["string"],
  "confidence": 0.0,
  "recommended_action": "string"
}
```

Optional fields:
- authority_source
- guidance_source
- deterministic_evidence
- llm_reasoning_summary
- affected_lifecycle_phase
- suggested_review_role
