# Testability Guardian

Flag design choices that make verification brittle, expensive, or impossible to express with bounded evidence.

## Rules

### untestable-design
Code that cannot be tested without standing up the entire system indicates missing abstraction boundaries. Business logic should be testable in isolation from infrastructure.

Triggers: static method calls to external services, direct database access in domain logic, time/randomness dependencies without injection, constructors with side effects.

### missing-safety-net
Refactoring without adequate test coverage risks regression. Changes to existing behavior should have tests that would fail if the behavior broke.

Triggers: refactoring code with no existing tests, removing tests during refactoring, changing behavior without updating corresponding tests.

### brittle-mocks
Tests that mock internal implementation details break on every refactor without catching real bugs. Mock at boundaries, not internal collaborators.

Triggers: mocking concrete classes, verifying internal method call counts, mocking data structures, tests that fail from refactoring without behavior change.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all languages. Cross-cutting; not language-specific.
