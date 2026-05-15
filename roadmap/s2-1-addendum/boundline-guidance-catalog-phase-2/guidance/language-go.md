# Go Guidance

## Purpose

This guidance defines idiomatic Go practices for AI-assisted planning, implementation, testing, review, and refactoring.

It is intended for Go services, CLIs, libraries, backend workers, and infrastructure tooling.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, expert pack configuration, or Canon-governed standards.

## Version Posture

Active support window:
- Go 1.18+ for generics support

Target excellence:
- Go 1.22+ where repository constraints allow
- standard library routing where appropriate
- `log/slog` for structured logging
- standard error wrapping patterns

Legacy warnings:
- pre-generics code where generic helpers would reduce duplication
- `pkg/errors` in new code
- ad hoc logging packages where `slog` would be sufficient
- custom router dependencies where standard routing satisfies needs

## Core Design Principles

### Simplicity Over Framework Abstraction

Go code should remain direct.

Avoid building framework-like abstractions unless they remove real duplication or clarify ownership.

AI-generated Go often over-engineers with:
- unnecessary interfaces
- generic repositories without behavior
- service layers that only pass through calls
- factory hierarchies that hide simple construction

Prefer explicit, small, boring code.

### Interfaces At Consumer Boundaries

Define interfaces where they are consumed, not where implementations are produced.

Good:
- a service defines the small dependency behavior it needs

Bad:
- every implementation has a matching interface generated beside it
- broad interfaces leak implementation concerns
- interfaces are created only for mocking without design value

### Error Handling

Use standard library error wrapping.

Preferred:
```go
fmt.Errorf("load account %s: %w", accountID, err)
```

Use:
- `errors.Is`
- `errors.As`
- `errors.Join` when multiple failures must be preserved

Avoid:
- string matching on errors
- losing causal context
- returning raw infrastructure errors across domain boundaries
- capitalized or punctuated error strings

### Log Or Return, Not Both

Do not log an error and then return it unless a clear ownership boundary exists.

Bad:
```go
log.Error("failed", "err", err)
return err
```

Better:
- low-level function returns contextual error
- boundary handler logs once with request context

### Panic Policy

Do not use `panic` for business flow.

Acceptable panic cases:
- impossible programmer error
- startup configuration that makes process continuation invalid
- tests

Guardian checks should flag panic in runtime request paths and domain logic.

### Context Propagation

Functions that perform I/O or blocking work should accept `context.Context`.

Do:
- pass context to database calls
- pass context to HTTP calls
- respect cancellation
- avoid storing context in structs

Do not:
- ignore context cancellation
- create background contexts deep in call chains without explicit reason
- use context as a general key-value bag for domain data

### Concurrency

Prefer structured concurrency.

Use:
- `errgroup`
- bounded worker pools
- context cancellation
- channel ownership conventions

Avoid:
- unbounded goroutines
- goroutine leaks
- writing to channels from unclear owners
- shared mutable state without ownership clarity

### Generics

Use generics when they clarify type-safe behavior.

Good:
- reusable collection utilities
- typed result wrappers
- shared validation helpers

Bad:
- generic abstraction to avoid writing two simple functions
- generic repositories that erase domain meaning
- type parameters where interfaces would be clearer

### Logging

Prefer `log/slog` for structured logging in new code.

Logs should include:
- request ID or correlation ID
- operation name
- stable entity identifiers
- error cause

Avoid:
- string-only logging
- logging secrets
- logging full request bodies by default
- duplicated logs across layers

### HTTP And API Boundaries

Validate request input at the boundary.

Keep:
- transport mapping in handlers
- domain behavior in services or domain types
- persistence in repositories/adapters
- response mapping explicit

Avoid:
- handlers containing business policy
- domain logic importing HTTP framework types
- repository methods returning transport-specific errors

## Testing Guidance

Use table-driven tests when they improve coverage clarity.

Good table tests:
- name each case
- isolate setup
- assert behavior, not implementation
- include edge cases

Avoid table tests when:
- cases become unreadable
- setup differs too much between rows
- failure messages become obscure

Use:
- fakes for domain dependencies
- integration tests for database behavior
- `httptest` for HTTP boundaries
- deterministic clocks for time-sensitive behavior

## Anti-Patterns

- log and return
- panic for expected errors
- broad interfaces generated for every struct
- unbounded goroutines
- ignored context cancellation
- string matching on errors
- global mutable state
- repositories with no domain meaning
- excessive `interface{}` or `any`
- package names like `utils`, `common`, or `helpers` without domain meaning

## Guardian Hooks

Recommended guardians:
- go-error-ownership-guardian
- go-context-propagation-guardian
- go-concurrency-guardian
- go-interface-boundary-guardian
- go-panic-policy-guardian
- go-logging-guardian

## Structured Finding Example

```json
{
  "guardian": "go-error-ownership",
  "rule": "log-or-return",
  "disposition": "concern",
  "summary": "The function logs the database error and returns it, causing duplicate logging at the handler boundary.",
  "evidence_refs": ["internal/accounts/service.go"],
  "recommended_action": "Return a contextual wrapped error here and log once at the request boundary."
}
```

## Lifecycle Usage

Planning:
- detect concurrency, context, and API boundary concerns

Implementation:
- guide error handling, context propagation, and package structure

Testing:
- encourage table-driven tests where useful and boundary-level tests where needed

Review:
- check panic policy, goroutine ownership, logging ownership, and interface misuse

Refactoring:
- remove unnecessary abstractions without changing external behavior
