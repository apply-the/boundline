# Framework Guardian Rule Seeds — Phase 3

## Purpose

This document defines initial guardian rule seeds for framework and application-stack guidance.

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

## React Guardians

### react-server-client-boundary-guardian

Rules:
- secret-in-client-component
- server-only-import-in-client
- unnecessary-client-boundary
- hydration-sensitive-side-effect

Candidate kind:
- deterministic or hybrid

### react-server-state-guardian

Rules:
- server-state-in-local-component-state
- manual-cache-invalidation
- duplicated-fetched-state
- mutation-without-invalidation

Candidate kind:
- hybrid

### react-effect-misuse-guardian

Rules:
- effect-for-derived-state
- props-copied-to-state
- effect-chain-workflow
- hidden-business-logic-effect

Candidate kind:
- deterministic or hybrid

### react-accessibility-guardian

Rules:
- missing-accessible-name
- non-semantic-interactive-element
- focus-management-missing
- aria-overuse

Candidate kind:
- deterministic or hybrid

## Node Service Guardians

### node-handler-boundary-guardian

Rules:
- business-logic-in-route-handler
- persistence-in-handler
- route-orchestration-bloat
- auth-policy-in-handler

Candidate kind:
- hybrid

### node-runtime-validation-guardian

Rules:
- unchecked-request-body
- unchecked-external-json
- duplicated-schema-and-type
- any-at-boundary

Candidate kind:
- deterministic or hybrid

### node-error-mapping-guardian

Rules:
- raw-exception-response
- stack-trace-leak
- database-error-leak
- inconsistent-error-shape

Candidate kind:
- deterministic or hybrid

### node-framework-leakage-guardian

Rules:
- request-object-in-domain-service
- framework-response-from-domain
- framework-exception-in-domain-error

Candidate kind:
- hybrid

## Python Service Guardians

### python-route-boundary-guardian

Rules:
- domain-logic-in-route
- persistence-in-view
- authorization-split-across-layers
- route-orchestration-bloat

Candidate kind:
- hybrid

### python-schema-ownership-guardian

Rules:
- request-schema-used-as-domain-object
- pydantic-as-persistence-entity
- raw-dict-domain-command
- missing-schema-mapping

Candidate kind:
- hybrid

### python-framework-leakage-guardian

Rules:
- request-object-in-domain-service
- framework-exception-as-domain-error
- framework-dependency-in-domain-core

Candidate kind:
- hybrid

### python-django-signal-guardian

Rules:
- hidden-business-workflow-signal
- signal-side-effect-without-trace
- signal-order-coupling

Candidate kind:
- llm or hybrid

## JVM Service Guardians

### jvm-controller-boundary-guardian

Rules:
- business-logic-in-controller
- repository-access-in-controller
- distributed-workflow-in-controller
- validation-scattered-without-owner

Candidate kind:
- hybrid

### jvm-entity-contract-guardian

Rules:
- entity-leaked-as-api-contract
- persistence-annotation-in-public-dto
- lazy-loading-through-serialization
- api-shape-coupled-to-schema

Candidate kind:
- deterministic or hybrid

### jvm-transaction-boundary-guardian

Rules:
- external-call-inside-transaction
- hidden-long-transaction
- retry-without-idempotency
- lazy-loading-outside-transaction

Candidate kind:
- hybrid

### jvm-framework-leakage-guardian

Rules:
- framework-annotation-in-domain-core
- framework-exception-as-domain-error
- repository-interface-leaked-upward

Candidate kind:
- hybrid

## .NET Service Guardians

### dotnet-endpoint-boundary-guardian

Rules:
- business-logic-in-endpoint
- persistence-in-controller
- authorization-hidden-in-repository
- endpoint-orchestration-bloat

Candidate kind:
- hybrid

### dotnet-problemdetails-guardian

Rules:
- raw-exception-message-response
- stack-trace-leak
- inconsistent-error-shape
- validation-error-as-500

Candidate kind:
- deterministic or hybrid

### dotnet-background-work-guardian

Rules:
- fire-and-forget-request-task
- unobserved-background-exception
- missing-cancellation-background-work
- missing-worker-observability

Candidate kind:
- hybrid

### dotnet-di-boundary-guardian

Rules:
- service-locator
- giant-constructor-dependency-list
- static-mutable-dependency
- framework-type-in-domain

Candidate kind:
- deterministic or hybrid

### dotnet-authorization-boundary-guardian

Rules:
- client-owned-authorization-field
- authorization-split-unclearly
- missing-tenant-isolation-check
- repository-owned-authorization

Candidate kind:
- hybrid

## Default Disposition Guidance

Use `blocker` when:
- secrets leak to client code
- authorization boundary is violated
- raw exception or stack trace is exposed publicly
- external writes are retried without idempotency in critical flows

Use `warning` when:
- framework leakage creates maintainability or testability risk
- transaction or background work behavior is unsafe
- API error shape is inconsistent

Use `concern` when:
- component or handler boundaries are unclear
- schema ownership is blurred
- tests are likely brittle or shallow

Use `observation` when:
- modernization would help but repository constraints may justify current design

## Authority Note

These rule seeds are recommendations by default.

They become stronger only when promoted by:
- workspace policy
- Canon-governed standard
- shared expert pack configuration
- S3/S4 governance posture
