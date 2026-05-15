# Security, Domain, Contract, And Migration Guardian Rule Seeds — Phase 4

## Purpose

This document defines guardian rule seeds for security boundaries, domain language, domain modeling, API contracts, and migration safety.

Guardians emit structured findings and do not directly modify code.

## Required Finding Fields

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

## Security Guardians

### security-boundary-guardian

Rules:
- authn-boundary-missing
- authz-boundary-missing
- fail-open-security
- secure-default-missing

Candidate kind:
- hybrid

### authz-ownership-guardian

Rules:
- client-owned-resource-id-trusted
- ownership-check-missing
- authorization-only-in-ui
- repository-hidden-authorization

Candidate kind:
- hybrid

### tenant-isolation-guardian

Rules:
- tenant-filter-missing
- tenant-id-from-request-body
- shared-cache-key-without-tenant
- background-job-loses-tenant-context

Candidate kind:
- deterministic or hybrid

### secret-handling-guardian

Rules:
- secret-in-client-bundle
- secret-in-log
- secret-in-command-argument
- secret-committed-or-hardcoded

Candidate kind:
- deterministic or hybrid

### pii-flow-guardian

Rules:
- pii-logged
- pii-sent-to-external-provider-without-policy
- pii-embedded-without-policy
- retention-policy-missing

Candidate kind:
- hybrid

### token-lifecycle-guardian

Rules:
- missing-token-expiry-validation
- refresh-token-without-rotation
- rotation-without-replay-protection
- revocation-missing

Candidate kind:
- hybrid

### auditability-guardian

Rules:
- privileged-action-without-audit
- audit-event-missing-actor
- audit-event-missing-target
- audit-event-with-sensitive-payload

Candidate kind:
- hybrid

## Domain Language Guardians

### domain-language-drift-guardian

Rules:
- two-terms-same-concept
- one-term-multiple-concepts
- deprecated-term-reintroduced
- api-vocabulary-drift

Candidate kind:
- llm or hybrid

### new-term-without-definition-guardian

Rules:
- new-domain-term-in-code
- new-domain-term-in-api
- new-domain-term-in-test
- term-missing-from-canon-language

Candidate kind:
- hybrid

### external-term-leakage-guardian

Rules:
- provider-term-in-domain-core
- integration-term-replaces-domain-term
- missing-anti-corruption-mapping

Candidate kind:
- llm or hybrid

### implementation-term-guardian

Rules:
- manager-handler-processor-domain-name
- dto-entity-record-as-domain-concept
- generic-service-name-in-domain

Candidate kind:
- llm

## Domain Model Guardians

### domain-invariant-guardian

Rules:
- invariant-enforced-only-in-ui
- invariant-missing-server-side
- invariant-without-test-evidence
- invariant-split-without-coordination

Candidate kind:
- hybrid

### aggregate-boundary-guardian

Rules:
- aggregate-without-invariant
- cross-aggregate-mutation-without-process
- aggregate-root-bypassed
- transaction-boundary-does-not-protect-invariant

Candidate kind:
- llm or hybrid

### bounded-context-guardian

Rules:
- bounded-context-leakage
- duplicated-concept-across-contexts
- missing-translation-boundary
- shared-database-cross-context-write

Candidate kind:
- hybrid

### primitive-domain-modeling-guardian

Rules:
- raw-string-domain-id
- raw-number-domain-quantity
- boolean-state-explosion
- missing-value-object

Candidate kind:
- deterministic or hybrid

### domain-event-guardian

Rules:
- event-name-not-domain-fact
- event-leaks-implementation
- event-missing-owner
- event-without-consumer-compatibility

Candidate kind:
- hybrid

## API Contract Guardians

### api-contract-compatibility-guardian

Rules:
- removed-field
- new-required-field
- changed-field-meaning
- enum-value-removed
- nullability-changed

Candidate kind:
- deterministic or hybrid

### error-contract-guardian

Rules:
- raw-exception-response
- inconsistent-error-shape
- error-code-changed
- retryability-missing

Candidate kind:
- deterministic or hybrid

### event-schema-compatibility-guardian

Rules:
- event-required-field-added
- event-field-meaning-changed
- event-version-missing
- consumer-impact-unknown

Candidate kind:
- hybrid

### consumer-impact-guardian

Rules:
- unknown-consumers
- migration-window-missing
- adoption-telemetry-missing
- deprecation-plan-missing

Candidate kind:
- llm or hybrid

### contract-test-guardian

Rules:
- public-contract-without-test
- provider-only-test-for-consumer-contract
- golden-file-not-updated
- compatibility-test-missing

Candidate kind:
- deterministic or hybrid

## Migration Guardians

### migration-sequencing-guardian

Rules:
- migration-type-unclassified
- reversibility-unknown
- incompatible-ordering
- migration-hidden-in-startup

Candidate kind:
- hybrid

### expand-contract-guardian

Rules:
- destructive-schema-change-first
- required-column-without-default-or-backfill
- old-reader-incompatible-with-new-data
- contract-phase-skipped

Candidate kind:
- deterministic or hybrid

### rollback-safety-guardian

Rules:
- rollback-plan-missing
- irreversible-action-not-declared
- compensation-missing
- previous-version-incompatible

Candidate kind:
- hybrid

### backfill-readiness-guardian

Rules:
- backfill-without-batching
- dry-run-missing
- progress-tracking-missing
- validation-query-missing
- stop-criteria-missing

Candidate kind:
- hybrid

### dual-write-safety-guardian

Rules:
- dual-write-without-reconciliation
- source-of-truth-unclear
- dual-write-without-idempotency
- discrepancy-metric-missing

Candidate kind:
- hybrid

### cutover-readiness-guardian

Rules:
- cutover-owner-missing
- success-criteria-missing
- failure-criteria-missing
- monitoring-window-missing

Candidate kind:
- llm or hybrid

## Default Disposition Guidance

Use `blocker` when:
- authorization is missing for protected resources
- tenant isolation is broken
- secrets are exposed
- PII is sent externally without policy
- invariant is enforced only in UI for critical behavior
- destructive migration occurs before compatibility window
- non-idempotent migration/write can corrupt data
- public contract is broken without versioning or migration plan

Use `warning` when:
- compatibility evidence is missing
- migration rollback is unclear
- domain language drift could affect public API or tests
- auditability is missing for privileged action
- consumer impact is unknown

Use `concern` when:
- model or language clarity is weak
- boundary ownership is unclear
- contract tests are missing for non-critical internal interfaces
- migration observability is incomplete but not immediately unsafe

Use `observation` when:
- modernization or clarity improvement is useful but current behavior may be justified

## Authority Note

These rule seeds are recommendations by default.

They become stronger only when promoted by:

- workspace policy
- Canon-governed standard
- security policy
- architecture decision
- API governance
- migration governance
- S3/S4 governance posture
