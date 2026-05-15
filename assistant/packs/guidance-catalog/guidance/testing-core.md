# Testing Core

Testing strategy for bounded delivery: behavior verification, boundary selection, safety nets, and evidence validity.

## Core Principles

### Test Behavior, Not Implementation

Tests should verify externally meaningful behavior. Avoid tests that assert private helper calls, internal ordering without behavior relevance, implementation-specific structure, or mocks that mirror implementation instead of contract.

### Test At The Right Boundary

Unit tests: pure logic, domain rules, value objects, policies.

Integration tests: database behavior, API contracts, message boundaries, framework wiring.

End-to-end tests: critical user journeys, cross-system flows, regression-prone paths.

### Safety Net Before Refactor

Before refactoring, identify what behavior is already protected. If no safety net exists:
- Write characterization tests.
- Document what cannot be tested safely.
- Avoid large structural changes.
- Prefer small reversible moves.

### Deterministic Tests

Tests must not depend on real time, random order, external services without isolation, arbitrary sleeps, global mutable state, or shared persistent state.

Use: fake clocks, deterministic IDs, isolated test data, API-level seeding, contract mocks.

### Avoid Brittle Mocks

Mocks should represent contracts, not implementation choreography.

Bad:
- Asserting every internal method call
- Mocking the unit under test's own logic
- Over-specifying incidental interactions

Good:
- Contract-level test doubles
- Behavior verification at boundary
- Fake repositories for domain behavior
- Pact or equivalent for external contracts

### Coverage Is Not Confidence

Coverage can rise while confidence falls. Testing strategy should answer:
- What behavior is protected?
- What risk is reduced?
- What remains untested?
- What cannot be tested without redesign?
- What evidence supports merge?

### AI-Generated Tests Need Review

AI often generates tests that repeat implementation logic, assert shallow behavior, use unrealistic data, miss edge cases, overfit the generated code, or create green dashboards without validity.

Guardians should challenge whether tests would fail for real defects.

## Patterns

### Test Pyramid

Many fast unit tests, targeted integration tests, and a small number of high-value end-to-end flows. Keep cost proportional to confidence gained.

### Test Naming By Scenario

Name tests by scenario and expected behavior so failures explain what regressed:

```text
order_with_expired_coupon_is_rejected
payment_retry_preserves_idempotency_key
```

### Contract Tests

Use contract tests when independent components evolve against a shared API or schema. Consumer-driven contracts prevent breaking changes without coordination.

## Anti-Patterns

- Tests that pass for generated code but would not catch real defects
- Mocks that replicate the exact implementation rather than the boundary contract
- Test suites dependent on execution order or shared mutable fixtures
- Coverage targets met by asserting truthy values without meaningful behavior checks
- Integration tests that depend on external services without isolation or stubbing
- Characterization tests treated as permanent fixtures rather than refactoring scaffolding

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, missing-safety-net, brittle-mocks
- `clean_code`: no-hidden-side-effects (untestable when side effects are invisible)
- `architecture_boundary`: dependency-direction (untestable when domain depends on infrastructure)
