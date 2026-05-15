# Testing Core Guidance

## Purpose

This guidance defines testing strategy for AI-assisted delivery.

It should shape implementation, verification, refactor, safety-net work, review, and migration.

## Authority Classification

Default strength: recommended  
May become mandatory when promoted by Canon safety-net or verification packets.

## Testing Principles

### Test Behavior, Not Implementation

Tests should verify externally meaningful behavior.

Avoid tests that assert:
- private helper calls
- internal ordering without behavior relevance
- implementation-specific structure
- mocks that mirror implementation instead of contract

### Test At The Right Boundary

Unit tests:
- pure logic
- domain rules
- value objects
- policies

Integration tests:
- database behavior
- API contracts
- message boundaries
- framework wiring

End-to-end tests:
- critical user journeys
- cross-system flows
- regression-prone paths

### Safety Net Before Refactor

Before refactoring, identify what behavior is already protected.

If no safety net exists:
- write characterization tests
- document what cannot be tested safely
- avoid large structural changes
- prefer small reversible moves

### Deterministic Tests

Tests must not depend on:
- real time
- random order
- external services without isolation
- arbitrary sleeps
- global mutable state
- shared persistent state

Use:
- fake clocks
- deterministic IDs
- isolated test data
- API-level seeding
- contract mocks

### Avoid Brittle Mocks

Mocks should represent contracts, not implementation choreography.

Bad:
- asserting every internal method call
- mocking the unit under test's own logic
- over-specifying incidental interactions

Good:
- contract-level test doubles
- behavior verification at boundary
- fake repositories for domain behavior
- Pact or equivalent for external contracts

### Coverage Is Not Confidence

Coverage can rise while confidence falls.

Testing strategy should answer:
- what behavior is protected?
- what risk is reduced?
- what remains untested?
- what cannot be tested without redesign?
- what evidence supports merge?

### AI-Generated Tests Need Review

AI often generates tests that:
- repeat implementation logic
- assert shallow behavior
- use unrealistic data
- miss edge cases
- overfit the generated code
- create green dashboards without validity

Guardians should challenge whether tests would fail for real defects.

## Guardian Hooks

Recommended guardians:
- testability-guardian
- safety-net-guardian
- coverage-validity-guardian
- brittle-mock-guardian
- characterization-test-guardian

## Structured Finding Examples

```json
{
  "guardian": "testability",
  "rule": "untestable-design",
  "disposition": "concern",
  "summary": "Token expiry logic depends directly on system time, making deterministic tests difficult.",
  "evidence_refs": ["src/auth/token_rotation.rs"],
  "recommended_action": "Inject a clock or time provider before adding additional behavior."
}
```

## Lifecycle Usage

Planning:
- identify required evidence before implementation

Implementation:
- guide test-friendly boundaries

Testing:
- select correct test layer

Review:
- challenge test validity, not only coverage

Refactor:
- require safety net or characterization strategy

Verification:
- compare claims to actual evidence
