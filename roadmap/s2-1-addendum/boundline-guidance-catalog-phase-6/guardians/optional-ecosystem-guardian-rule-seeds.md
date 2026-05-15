# Optional Ecosystem Guardian Rule Seeds — Phase 6

## Purpose

This document defines guardian rule seeds for optional ecosystem guidance:

- modern frontend frameworks
- Rails and Laravel
- mobile applications
- data and AI systems

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

## Modern Frontend Guardians

### frontend-reactivity-guardian

Rules:
- reactive-side-effect-hidden
- derived-state-stored
- subscription-cleanup-missing
- global-store-local-state

Candidate kind:
- hybrid

### frontend-state-ownership-guardian

Rules:
- server-state-in-global-ui-store
- url-state-hidden-in-component
- persisted-state-without-migration
- api-payload-used-as-domain-model

Candidate kind:
- hybrid

### frontend-accessibility-guardian

Rules:
- missing-accessible-name
- non-semantic-interactive-element
- keyboard-navigation-missing
- focus-management-missing
- color-contrast-risk

Candidate kind:
- deterministic or hybrid

### frontend-server-client-boundary-guardian

Rules:
- secret-in-client-code
- server-only-import-in-client
- unnecessary-client-boundary
- cache-invalidation-missing

Candidate kind:
- deterministic or hybrid

## Rails / Laravel Guardians

### rails-laravel-controller-boundary-guardian

Rules:
- business-logic-in-controller
- persistence-orchestration-in-controller
- authorization-split-across-controller-and-view
- route-action-too-broad

Candidate kind:
- hybrid

### active-record-domain-leakage-guardian

Rules:
- god-model
- unrelated-policy-in-model
- persistence-model-as-public-contract
- query-scope-hides-business-policy

Candidate kind:
- llm or hybrid

### callback-hidden-workflow-guardian

Rules:
- critical-side-effect-in-callback
- callback-publishes-external-event
- callback-order-dependence
- callback-without-observability

Candidate kind:
- hybrid

### queue-job-idempotency-guardian

Rules:
- job-without-idempotency
- retry-unsafe-job
- poison-job-unhandled
- job-without-progress-visibility

Candidate kind:
- hybrid

### authorization-policy-guardian

Rules:
- view-only-authorization
- missing-negative-authorization-test
- client-owned-permission-field
- policy-coverage-missing

Candidate kind:
- hybrid

## Mobile Guardians

### mobile-compatibility-guardian

Rules:
- backend-change-assumes-immediate-client-update
- supported-client-contract-broken
- minimum-version-policy-missing
- kill-switch-missing-risky-feature

Candidate kind:
- hybrid

### offline-state-guardian

Rules:
- offline-input-loss-risk
- stale-cache-without-disclosure
- sync-conflict-unhandled
- retry-write-without-idempotency

Candidate kind:
- hybrid

### mobile-permission-guardian

Rules:
- permission-request-without-context
- permission-denial-unhandled
- excessive-permission-scope
- background-permission-without-justification

Candidate kind:
- hybrid

### mobile-privacy-guardian

Rules:
- pii-in-crash-report
- token-in-insecure-storage
- secret-in-client-bundle
- tracking-without-policy

Candidate kind:
- deterministic or hybrid

### app-lifecycle-guardian

Rules:
- foreground-assumption
- process-death-state-loss
- deep-link-validation-missing
- token-refresh-lifecycle-risk

Candidate kind:
- hybrid

### mobile-release-readiness-guardian

Rules:
- risky-feature-without-staged-rollout
- crash-monitoring-missing
- rollback-not-possible-without-kill-switch
- store-review-risk-undocumented

Candidate kind:
- llm or hybrid

## Data And AI Systems Guardians

### data-provenance-guardian

Rules:
- dataset-provenance-missing
- derived-table-owner-missing
- feature-version-missing
- transformation-lineage-missing

Candidate kind:
- hybrid

### data-quality-guardian

Rules:
- freshness-check-missing
- null-rate-check-missing
- uniqueness-check-missing
- business-invariant-check-missing
- quality-check-not-tied-to-action

Candidate kind:
- deterministic or hybrid

### schema-contract-guardian

Rules:
- breaking-schema-change
- semantic-field-meaning-changed
- downstream-consumer-unknown
- event-schema-compatibility-missing

Candidate kind:
- hybrid

### ai-evaluation-guardian

Rules:
- evaluation-missing
- benchmark-not-representative
- train-test-leakage-risk
- golden-set-owner-missing
- failure-cases-not-tracked

Candidate kind:
- hybrid

### llm-output-validation-guardian

Rules:
- model-output-unvalidated
- tool-args-without-schema
- mutation-without-authorization-recheck
- hallucination-sensitive-claim-without-evidence

Candidate kind:
- deterministic or hybrid

### prompt-versioning-guardian

Rules:
- prompt-change-without-versioning
- prompt-change-without-evaluation
- prompt-hardcoded-in-production-path
- prompt-owner-missing

Candidate kind:
- hybrid

### retrieval-authority-guardian

Rules:
- vector-similarity-treated-as-truth
- source-provenance-missing
- stale-document-policy-missing
- citation-not-linked-to-authority

Candidate kind:
- hybrid

### embedding-privacy-guardian

Rules:
- sensitive-data-embedded-without-policy
- remote-embedding-without-opt-in
- embedding-index-retention-missing
- pii-in-vector-store

Candidate kind:
- hybrid

### tool-boundary-guardian

Rules:
- tool-access-too-broad
- side-effect-tool-without-approval
- tool-input-schema-missing
- tool-audit-missing

Candidate kind:
- hybrid

### drift-monitoring-guardian

Rules:
- model-drift-monitoring-missing
- data-drift-check-missing
- retrieval-quality-drift-untracked
- human-override-rate-untracked

Candidate kind:
- hybrid

## Default Disposition Guidance

Use `blocker` when:
- model output can mutate critical state without validation
- secrets or tokens are exposed in client/mobile code
- mobile/server compatibility breaks supported clients
- sensitive data is embedded or transmitted without policy
- authorization or tenant isolation is at risk

Use `warning` when:
- production AI/data behavior lacks evaluation or monitoring
- mobile rollout lacks kill switch for risky feature
- callbacks or background jobs hide operational workflows
- frontend accessibility is likely broken in critical flows

Use `concern` when:
- framework boundaries are blurred
- state ownership is unclear
- data provenance is incomplete
- prompt or retrieval behavior lacks versioning

Use `observation` when:
- modernization would help but repository constraints may justify current design

## Authority Note

These rule seeds are recommendations by default.

They become stronger only when promoted by:
- workspace policy
- Canon-governed standard
- shared expert pack configuration
- S3/S4 governance posture
