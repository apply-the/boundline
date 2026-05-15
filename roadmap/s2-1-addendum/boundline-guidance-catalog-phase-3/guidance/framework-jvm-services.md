# JVM Service Framework Guidance

## Purpose

This guidance defines framework-level practices for JVM services, including Spring Boot, Micronaut, Quarkus, and similar service frameworks.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, architecture decision, or Canon-governed standard.

## Framework Posture

Common enterprise choices:
- Spring Boot for ecosystem maturity and broad integration
- Micronaut for compile-time DI and lower reflection overhead
- Quarkus for Kubernetes-native and fast startup scenarios

Framework choice depends on existing repository architecture.

This guidance does not mandate migration.

## Core Design Principles

### Controller Thinness

Controllers should:
- parse and validate transport input
- call application services
- map results to response DTOs
- map errors to stable API errors

Controllers should not:
- contain business policy
- manipulate repositories directly for complex workflows
- expose persistence entities as API contracts
- coordinate distributed workflows without application-layer ownership

### Service Layer Meaning

Application services should represent use cases.

Avoid:
- pass-through services that simply call repositories
- service classes as dumping grounds
- anemic orchestration without domain meaning

A service should clarify:
- transaction boundary
- authorization boundary
- workflow sequencing
- domain policy coordination

### Persistence Boundary

Do not expose persistence entities as public API contracts.

Keep separate:
- entity
- domain model
- command/query
- response DTO
- integration event

In small CRUD systems, collapse may be acceptable only if explicit and governed.

### Transaction Boundaries

Transactions should be explicit and owned.

Check:
- transaction starts at use-case boundary
- no hidden external calls inside long transactions
- idempotency for retryable operations
- no accidental lazy-loading across serialization boundary

### Validation

Use validation at boundaries.

Avoid:
- relying only on database constraints for domain validation
- spreading validation across controller, service, entity without ownership
- returning raw validation exceptions to clients

### Framework Leakage

Domain model should not depend on framework annotations unless the architecture intentionally couples domain and persistence.

Warnings:
- Spring annotations in domain core
- JPA entities as domain objects in complex systems
- framework exceptions as domain errors
- repository interfaces leaking through API layer

### Observability

Service methods and request boundaries should preserve:
- trace ID
- operation name
- stable entity identifiers
- structured error context

Avoid:
- logging same exception repeatedly
- logging sensitive data
- swallowing framework-level exceptions without context

## Testing Guidance

Recommended:
- JUnit 5
- AssertJ
- Spring Boot slice tests where useful
- Testcontainers for real infrastructure dependencies
- contract tests for external APIs/events
- transactional integration tests where persistence behavior matters

Avoid:
- mocking every repository in tests that should verify integration
- loading full application context for simple unit tests
- relying on test ordering
- brittle interaction-only mocks
- unbounded fixture data

## Anti-Patterns

- business logic in controllers
- entity exposed as API response
- framework annotations throughout domain core
- pass-through service layers
- hidden lazy-loading in serialization
- external HTTP call inside transaction
- raw exception message as API response
- overuse of checked exceptions for business decisions
- distributed workflow hidden in service method without compensation

## Guardian Hooks

Recommended guardians:
- jvm-controller-boundary-guardian
- jvm-entity-contract-guardian
- jvm-transaction-boundary-guardian
- jvm-framework-leakage-guardian
- jvm-service-layer-meaning-guardian
- jvm-observability-guardian

## Structured Finding Example

```json
{
  "guardian": "jvm-transaction-boundary",
  "rule": "external-call-inside-transaction",
  "disposition": "warning",
  "summary": "The use case performs an external provider call while a database transaction is open.",
  "evidence_refs": ["src/main/java/com/acme/payments/PaymentService.java"],
  "recommended_action": "Move the external call outside the transaction or introduce an outbox/compensation strategy."
}
```

## Lifecycle Usage

Planning:
- identify transaction, API, and persistence boundaries

Architecture:
- check framework/domain separation and contract ownership

Implementation:
- guide controller thinness, DTO mapping, and service meaning

Testing:
- guide slice vs integration test selection

Review:
- check entity leakage, transaction misuse, and framework leakage
