# Operations And Supply Chain Guardian Rule Seeds — Phase 5

## Purpose

This document defines initial guardian rule seeds for observability, resilience, operations readiness, and supply chain guidance.

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

## Observability Guardians

### observability-guardian

Rules:
- external-call-without-telemetry
- critical-path-without-logs
- background-job-without-lifecycle-logs
- missing-operational-signal

Candidate kind:
- hybrid

### correlation-id-guardian

Rules:
- missing-correlation-id
- correlation-not-propagated-to-downstream-call
- async-work-loses-correlation
- audit-event-without-correlation

Candidate kind:
- deterministic or hybrid

### logging-ownership-guardian

Rules:
- log-and-return
- duplicate-layer-logging
- unstructured-production-log
- secret-in-log
- raw-payload-log

Candidate kind:
- deterministic or hybrid

### metrics-coverage-guardian

Rules:
- critical-dependency-without-metrics
- high-cardinality-label
- retry-without-metric
- timeout-without-metric

Candidate kind:
- hybrid

### trace-boundary-guardian

Rules:
- inbound-request-without-span
- outbound-call-without-span
- message-consumer-without-trace
- model-or-tool-call-without-trace

Candidate kind:
- hybrid

## Resilience Guardians

### timeout-policy-guardian

Rules:
- external-call-without-timeout
- timeout-exceeds-caller-budget
- timeout-not-observable
- blocking-operation-without-deadline

Candidate kind:
- deterministic or hybrid

### retry-safety-guardian

Rules:
- retry-without-idempotency
- infinite-retry
- retry-without-backoff
- retry-validation-error
- retry-authz-error

Candidate kind:
- hybrid

### idempotency-guardian

Rules:
- write-retry-without-idempotency-key
- missing-deduplication
- replay-unsafe-consumer
- duplicate-side-effect-risk

Candidate kind:
- hybrid

### circuit-breaker-guardian

Rules:
- critical-dependency-without-circuit-breaker
- circuit-breaker-without-telemetry
- fallback-hides-critical-failure

Candidate kind:
- hybrid

### bulkhead-guardian

Rules:
- unbounded-fanout
- shared-pool-critical-and-noncritical
- tenant-isolation-missing
- no-concurrency-limit

Candidate kind:
- hybrid

### fallback-safety-guardian

Rules:
- fail-open-security
- fallback-hides-data-loss
- stale-cache-without-disclosure
- silent-degraded-behavior

Candidate kind:
- hybrid

## Operations Readiness Guardians

### runbook-readiness-guardian

Rules:
- critical-feature-without-runbook
- runbook-without-containment-steps
- runbook-without-diagnosis-links
- runbook-without-escalation-path

Candidate kind:
- llm or hybrid

### rollback-readiness-guardian

Rules:
- schema-change-without-rollback-plan
- public-contract-change-without-compatibility-plan
- feature-without-kill-switch
- migration-without-compensation

Candidate kind:
- hybrid

### alertability-guardian

Rules:
- user-impact-without-alert
- alert-without-owner
- alert-without-runbook
- noisy-alert-pattern

Candidate kind:
- hybrid

### ownership-guardian

Rules:
- production-capability-without-owner
- escalation-path-missing
- operational-contact-missing
- dashboard-owner-missing

Candidate kind:
- llm or hybrid

### feature-flag-lifecycle-guardian

Rules:
- flag-without-owner
- flag-without-cleanup-date
- flag-bypasses-security
- permanent-flag-branch

Candidate kind:
- deterministic or hybrid

### worker-operability-guardian

Rules:
- worker-without-progress-tracking
- poison-message-unhandled
- replay-unsafe-worker
- dead-letter-missing
- cancellation-missing

Candidate kind:
- hybrid

### migration-readiness-guardian

Rules:
- migration-without-dry-run
- migration-without-stop-criteria
- backfill-without-monitoring
- migration-without-validation-query
- no-rollback-or-compensation

Candidate kind:
- hybrid

## Supply Chain Guardians

### dependency-introduction-guardian

Rules:
- trivial-helper-dependency
- unmaintained-dependency
- dependency-without-owner
- dependency-risk-not-reviewed

Candidate kind:
- llm or hybrid

### lockfile-consistency-guardian

Rules:
- dependency-manifest-without-lockfile-update
- lockfile-changed-without-manifest-change
- unexplained-lockfile-churn
- broad-version-range

Candidate kind:
- deterministic

### vulnerability-triage-guardian

Rules:
- vulnerability-untriaged
- waiver-without-expiry
- critical-vulnerability-without-mitigation
- scanner-result-ignored

Candidate kind:
- deterministic or hybrid

### license-policy-guardian

Rules:
- unknown-license
- forbidden-license
- dual-license-without-decision
- generated-code-license-unclear

Candidate kind:
- deterministic or hybrid

### install-script-risk-guardian

Rules:
- unreviewed-postinstall-script
- curl-pipe-shell
- build-script-downloads-code
- install-script-can-access-secrets

Candidate kind:
- deterministic or hybrid

### container-base-image-guardian

Rules:
- unpinned-base-image
- latest-tag-base-image
- base-image-with-critical-vulnerability
- build-secrets-leaked

Candidate kind:
- deterministic or hybrid

### generated-code-provenance-guardian

Rules:
- generated-artifact-without-source
- generated-code-manually-edited-without-policy
- generator-version-missing
- provenance-missing

Candidate kind:
- hybrid

### ci-permission-guardian

Rules:
- broad-ci-token-permissions
- secrets-exposed-to-untrusted-pr
- release-job-without-approval
- untrusted-script-in-release-path

Candidate kind:
- deterministic or hybrid

## Default Disposition Guidance

Use `blocker` when:
- security fails open
- non-idempotent write retries can duplicate money/order/security effects
- critical vulnerability is exploitable and untriaged
- secret exposure is likely
- destructive migration has no stop or rollback strategy

Use `warning` when:
- production readiness is incomplete for high-risk work
- operational ownership or alerting is missing
- dependency posture is unknown
- telemetry is missing for critical paths

Use `concern` when:
- observability, resilience, or supply-chain posture is weak but not immediately unsafe
- a pattern may be justified but lacks evidence

Use `observation` when:
- improvement is useful but not required for the current risk profile

## Authority Note

These rule seeds are recommendations by default.

They become stronger only when promoted by:
- workspace policy
- Canon-governed standard
- shared expert pack configuration
- S3/S4 governance posture
