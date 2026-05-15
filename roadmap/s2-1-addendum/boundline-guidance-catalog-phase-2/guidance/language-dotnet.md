# .NET And C# Guidance

## Purpose

This guidance defines idiomatic .NET and C# practices for AI-assisted planning, architecture, implementation, testing, review, and refactoring.

It is intended for .NET services, APIs, libraries, workers, and enterprise applications.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, expert pack configuration, or Canon-governed standards.

## Version Posture

Active support window:
- .NET 6+

Target excellence:
- .NET 8 LTS where repository constraints allow
- modern C# language features
- `TimeProvider` for testable time
- `ProblemDetails` for API error responses
- built-in observability integration where appropriate

Legacy warnings:
- .NET Framework in active new development
- .NET 5 or older
- missing async/await in I/O paths
- uncontrolled static time access
- exception-driven expected business flow

## Core Design Principles

### Model Expected Failures Explicitly

Do not use exceptions as ordinary business control flow.

For expected outcomes:
- Result pattern
- OneOf
- FluentResults
- domain-specific result types
- validation result objects

Use exceptions for:
- unexpected infrastructure failure
- impossible states
- programmer errors
- unrecoverable platform failures

### API Errors

Use `ProblemDetails` style responses for HTTP APIs where appropriate.

Keep:
- stable error codes
- user-safe messages
- internal diagnostics in logs/traces
- correlation IDs

Avoid:
- returning raw exception messages
- leaking stack traces
- ambiguous 500s for validation failures
- inconsistent error shapes across endpoints

### Time And Testability

Avoid direct `DateTime.Now`, `DateTime.UtcNow`, or static time access in business logic.

Prefer:
- `TimeProvider`
- injected clocks
- deterministic test time

Time-sensitive logic without injectable time is hard to test and easy to break.

### Cancellation

Async methods that perform I/O or long-running work should accept and respect `CancellationToken`.

Avoid:
- ignoring cancellation token
- creating new tokens without linking
- swallowing cancellation exceptions as generic failures

### Async Guidance

Avoid:
- blocking on async with `.Result` or `.Wait()`
- sync-over-async deadlocks
- fire-and-forget tasks without supervision
- unbounded parallelism

Prefer:
- async all the way
- explicit concurrency limits
- background worker supervision
- timeout policies

### Dependency Injection

Use DI to express runtime dependencies.

Avoid:
- service locator patterns
- injecting giant dependency bags
- domain model depending on container concepts
- hidden static dependencies

### Resilience

Use resilience policies intentionally.

Recommended:
- Polly
- timeout
- retry with budget
- circuit breaker
- bulkhead

Never add retries to non-idempotent writes without explicit idempotency strategy.

### Validation

Use explicit validation at boundaries.

Recommended:
- FluentValidation where accepted
- domain validation in domain types
- API validation mapped to consistent error responses

Avoid:
- duplicating validation across layers without ownership
- trusting DTOs inside domain logic without transformation

## Testing Guidance

Recommended:
- xUnit, NUnit, or MSTest according to repo standard
- FluentAssertions where accepted
- test builders
- deterministic time via TimeProvider
- WebApplicationFactory for ASP.NET integration tests where appropriate
- Testcontainers where infrastructure integration matters

Avoid:
- sleeps for timing
- hidden global state
- tests that depend on execution order
- mocks that only mirror implementation
- testing private methods instead of behavior

## Anti-Patterns

- hardcoded `DateTime.Now`
- ignored `CancellationToken`
- `.Result` or `.Wait()` on async tasks
- logic inside catch blocks beyond log/wrap/notify
- exceptions for expected validation outcomes
- raw exception messages in API responses
- service locator
- static mutable dependencies
- adding Polly retries without idempotency
- domain logic depending on ASP.NET abstractions

## Guardian Hooks

Recommended guardians:
- dotnet-timeprovider-guardian
- dotnet-cancellation-guardian
- dotnet-async-boundary-guardian
- dotnet-result-pattern-guardian
- dotnet-problemdetails-guardian
- dotnet-resilience-guardian

## Structured Finding Example

```json
{
  "guardian": "dotnet-timeprovider",
  "rule": "hardcoded-current-time",
  "disposition": "concern",
  "summary": "Token expiry logic reads DateTime.UtcNow directly, making time-sensitive behavior difficult to test.",
  "evidence_refs": ["src/Auth/TokenRotationService.cs"],
  "recommended_action": "Inject TimeProvider and use deterministic time in tests."
}
```

## Lifecycle Usage

Planning:
- identify async, resilience, and API error strategy requirements

Architecture:
- check API error consistency, dependency boundaries, and resilience placement

Implementation:
- guide cancellation, async, time, and validation

Testing:
- enforce deterministic time and integration boundary tests

Review:
- check exception misuse, service locator, async deadlocks, and retry safety

Refactoring:
- modernize .NET code while preserving behavior and compatibility
