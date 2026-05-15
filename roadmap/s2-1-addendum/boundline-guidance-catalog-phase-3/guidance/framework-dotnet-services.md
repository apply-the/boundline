# .NET Service Framework Guidance

## Purpose

This guidance defines framework-level practices for .NET services and APIs, especially ASP.NET Core applications, workers, and integration services.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, architecture decision, or Canon-governed standard.

## Framework Posture

Recommended baseline:
- ASP.NET Core for HTTP APIs
- built-in dependency injection
- `ProblemDetails` for API errors where suitable
- minimal APIs or controllers based on repository conventions
- hosted services for background work with explicit lifecycle

This guidance does not mandate migration between controller and minimal API styles.

## Core Design Principles

### Endpoint Thinness

Endpoints/controllers should:
- parse and validate request
- call application service
- map result to response
- map errors to stable API shape

They should not:
- contain business policy
- directly manipulate persistence for complex workflows
- hide authorization decisions
- return raw exceptions

### Application Service Ownership

Application services should clarify:
- use case
- authorization boundary
- transaction boundary
- validation ownership
- external integration ownership

Avoid:
- pass-through services
- dependency bags
- logic spread across controllers and repositories
- service locator patterns

### API Error Shape

Use consistent error responses.

Recommended:
- `ProblemDetails`
- stable error codes
- correlation IDs
- safe public messages
- internal details in logs/traces

Avoid:
- raw exception messages
- stack traces
- ambiguous 500s for validation errors
- inconsistent error structures

### Dependency Injection

Use DI to express dependencies.

Avoid:
- service locator
- static mutable dependencies
- injecting framework types into domain logic
- giant constructor dependency lists that signal responsibility bloat

### Background Work

Hosted/background services require:
- cancellation handling
- lifecycle management
- observability
- retry policy
- idempotency
- failure reporting

Avoid:
- fire-and-forget tasks from request handlers
- unobserved background exceptions
- hidden global queues

### Persistence Boundary

Do not expose persistence entities directly as API contracts in governed systems.

Separate:
- request DTO
- command/query object
- domain model
- persistence entity
- response DTO

Collapse only when explicitly accepted by repository policy.

### Authorization Placement

Authorization should be explicit.

Avoid:
- trusting client ownership fields
- authorization checks hidden in repositories
- route handlers that partly authorize and partly delegate
- domain logic depending on ASP.NET authorization objects

### Resilience

Use:
- timeouts
- retries with budgets
- circuit breakers
- idempotency keys
- bulkheads for high-fanout dependencies

Recommended:
- Polly where accepted

Do not add retries to writes without idempotency.

## Testing Guidance

Recommended:
- WebApplicationFactory for integration tests
- Testcontainers for infrastructure
- deterministic time with TimeProvider
- test builders
- FluentAssertions where accepted
- API-level tests for error shape

Avoid:
- tests that require ordering
- direct testing of private methods
- fixed sleeps
- mocking the entire application service beneath an endpoint test
- shared mutable fixture state

## Anti-Patterns

- business logic in controllers/endpoints
- raw exception response
- fire-and-forget request work
- missing cancellation token
- static time in business logic
- service locator
- persistence entity as public response
- retry without idempotency
- authorization based on client-supplied ownership
- giant constructor dependency list

## Guardian Hooks

Recommended guardians:
- dotnet-endpoint-boundary-guardian
- dotnet-problemdetails-guardian
- dotnet-background-work-guardian
- dotnet-di-boundary-guardian
- dotnet-persistence-contract-guardian
- dotnet-authorization-boundary-guardian
- dotnet-resilience-policy-guardian

## Structured Finding Example

```json
{
  "guardian": "dotnet-background-work",
  "rule": "fire-and-forget-request-task",
  "disposition": "warning",
  "summary": "The endpoint starts background work without lifecycle supervision or failure reporting.",
  "evidence_refs": ["src/Orders/OrderController.cs"],
  "recommended_action": "Move background processing to a hosted service or queue-backed worker with cancellation and observability."
}
```

## Lifecycle Usage

Planning:
- identify endpoint, application service, authorization, and background work boundaries

Architecture:
- check API error shape, persistence boundary, and resilience posture

Implementation:
- guide endpoint thinness, DI, cancellation, and error mapping

Testing:
- guide integration tests, deterministic time, and error-shape checks

Review:
- check fire-and-forget work, service locator usage, entity leakage, and retry safety
