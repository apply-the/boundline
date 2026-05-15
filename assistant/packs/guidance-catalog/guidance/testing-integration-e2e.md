# Testing Integration And E2E

Conventions for integration tests, contract tests, and end-to-end tests that cross service or system boundaries.

## Integration Tests

Test the integration between your code and real infrastructure: databases, message queues, external APIs. Use test containers or embedded services.

## Contract Tests

Verify API contracts between producer and consumer without requiring both to run simultaneously. Use Pact, Spring Cloud Contract, or schema-based validation.

## E2E Tests

Limit E2E tests to critical user journeys. They are expensive to maintain and slow to run. Use them to verify the system works end-to-end, not for exhaustive behavior coverage.

## Test Data Management

Use factories or builders for test data. Isolate test data between tests. Clean up after tests. Avoid shared databases between test suites running in parallel.

## Environment Management

Tests should be runnable locally without manual setup. Use docker-compose, test containers, or embedded services. Document environment requirements.

## Flaky Test Strategy

Quarantine flaky tests rather than disabling. Investigate root causes: timing issues, shared state, network dependencies. Fix or remove within a bounded time.

## Anti-Patterns

- E2E tests as the primary testing strategy (test pyramid inversion)
- Shared test databases between parallel test suites
- Tests that require manual environment setup
- Integration tests that mock everything (testing nothing)
- Missing cleanup between test runs
- Ignoring flaky tests instead of fixing root causes
- Contract tests that test implementation instead of contracts

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: test-isolation, untestable-design
- `architecture_boundary`: contract stability between services
- `resilience`: handling external dependency failures in tests
