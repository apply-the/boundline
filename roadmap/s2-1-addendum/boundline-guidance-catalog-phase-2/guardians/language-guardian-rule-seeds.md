# Language Guardian Rule Seeds — Phase 2

## Purpose

This document defines initial guardian rule seeds for Go, Python, JVM/Java, and .NET/C# guidance.

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

## Go Guardians

### go-error-ownership-guardian

Rules:
- go-log-or-return
- go-error-wrapping
- go-error-string-style
- go-string-matching-errors

Candidate kind:
- deterministic for style checks
- hybrid for ownership analysis

### go-context-propagation-guardian

Rules:
- context-required-for-io
- no-background-context-deep-call
- context-not-domain-storage
- cancellation-ignored

Candidate kind:
- hybrid

### go-concurrency-guardian

Rules:
- unbounded-goroutine
- missing-errgroup
- unclear-channel-ownership
- shared-state-without-ownership

Candidate kind:
- hybrid

### go-interface-boundary-guardian

Rules:
- producer-owned-interface
- overly-broad-interface
- interface-created-only-for-mock
- pass-through-service-layer

Candidate kind:
- llm or hybrid

### go-panic-policy-guardian

Rules:
- panic-in-business-flow
- panic-in-request-path
- startup-panic-without-context

Candidate kind:
- deterministic or hybrid

## Python Guardians

### python-exception-ownership-guardian

Rules:
- swallowed-exception
- missing-exception-chaining
- broad-except-without-reraise
- infrastructure-exception-leaks-public-boundary

Candidate kind:
- deterministic or hybrid

### python-boundary-validation-guardian

Rules:
- unchecked-external-json
- unvalidated-env-var
- unvalidated-ai-output
- untyped-public-boundary

Candidate kind:
- hybrid

### python-typing-guardian

Rules:
- any-in-domain-logic
- anonymous-dict-domain-model
- missing-return-type-public-function
- broad-dict-any

Candidate kind:
- deterministic or hybrid

### python-async-safety-guardian

Rules:
- blocking-call-in-async
- unbounded-gather
- cancellation-ignored
- timeout-missing

Candidate kind:
- hybrid

### python-config-boundary-guardian

Rules:
- env-read-in-domain-code
- unvalidated-config
- missing-startup-config-failure
- secret-default-silent

Candidate kind:
- deterministic or hybrid

## JVM / Java Guardians

### java-modernization-guardian

Rules:
- legacy-java-version-warning
- sealed-type-opportunity
- record-opportunity
- switch-modernization-opportunity

Candidate kind:
- llm or hybrid

### java-error-modeling-guardian

Rules:
- checked-exception-business-flow
- raw-infrastructure-error-leak
- exception-driven-expected-outcome
- missing-domain-error-shape

Candidate kind:
- hybrid

### java-optional-guardian

Rules:
- optional-field
- optional-parameter
- nested-optional
- optional-serialization-shape

Candidate kind:
- deterministic or hybrid

### java-framework-boundary-guardian

Rules:
- controller-business-logic
- entity-leaked-as-api-contract
- framework-annotation-in-domain-model
- pass-through-service

Candidate kind:
- hybrid

### java-virtual-thread-guardian

Rules:
- synchronized-long-operation-virtual-thread
- unbounded-resource-use
- virtual-thread-assumed-to-fix-blocking-design

Candidate kind:
- hybrid

## .NET / C# Guardians

### dotnet-timeprovider-guardian

Rules:
- hardcoded-current-time
- nondeterministic-time-sensitive-test
- missing-time-abstraction

Candidate kind:
- deterministic or hybrid

### dotnet-cancellation-guardian

Rules:
- missing-cancellation-token
- cancellation-token-ignored
- cancellation-swallowed-as-generic-failure

Candidate kind:
- deterministic or hybrid

### dotnet-async-boundary-guardian

Rules:
- sync-over-async
- fire-and-forget-unsupervised
- unbounded-parallelism
- blocking-async-result

Candidate kind:
- deterministic or hybrid

### dotnet-result-pattern-guardian

Rules:
- exception-for-expected-validation
- missing-domain-result-shape
- raw-exception-as-business-outcome

Candidate kind:
- hybrid

### dotnet-problemdetails-guardian

Rules:
- raw-exception-message-response
- inconsistent-api-error-shape
- stack-trace-leak
- validation-error-as-500

Candidate kind:
- deterministic or hybrid

### dotnet-resilience-guardian

Rules:
- retry-without-idempotency
- timeout-missing
- circuit-breaker-missing-critical-integration
- bulkhead-missing-high-fanout

Candidate kind:
- hybrid

## Default Disposition Guidance

Use `blocker` when:
- behavior can cause production unsafe behavior
- security boundary is violated
- data loss or irreversible corruption is likely
- expected errors are silently swallowed

Use `concern` when:
- maintainability or testability is degraded
- design boundary is blurred
- runtime behavior is plausible but under-evidenced

Use `observation` when:
- modernization would help but is not required
- repository constraints may justify current shape

## Authority Note

These rule seeds are recommendations by default.

They become stronger only when promoted by:
- workspace policy
- Canon-governed standard
- shared expert pack configuration
- S3/S4 governance posture
