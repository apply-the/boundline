# Testing TypeScript

Testing conventions for TypeScript/JavaScript projects using Jest, Vitest, and related tools.

## Test Organization

- Unit tests: co-located with source (`*.test.ts` or `*.spec.ts`)
- Integration tests: dedicated `tests/` or `__tests__/` directory
- E2E tests: separate from unit/integration, with explicit setup/teardown

## Type-Safe Testing

Use TypeScript in tests. Avoid `as any` casts. Use factory functions for test data that maintain type safety.

```ts
function buildOrder(overrides: Partial<Order> = {}): Order {
  return {
    id: "order-1",
    status: "pending",
    ...overrides,
  };
}
```

## Async Testing

Always await async operations. Use `async/await` over callbacks. Handle rejected promises explicitly.

## Mocking

Mock at boundaries (HTTP clients, databases, external services). Avoid mocking internal modules. Use dependency injection to make mocking unnecessary for business logic.

```ts
// Prefer injecting dependencies
const service = new OrderService(mockRepository, mockEventBus);
```

## DOM Testing

Use Testing Library. Query by role, label, or text; not by CSS class or test ID. Test behavior, not implementation.

## Recommended Tools

| Tool | Purpose |
|------|---------|
| `vitest` | Fast, Vite-native test runner (preferred for new projects) |
| `jest` | Mature test runner with broad ecosystem |
| `msw` | Service worker-based HTTP mocking |
| `@testing-library/*` | DOM testing by accessible queries |
| `supertest` | HTTP assertion for Express/Fastify/Koa |
| `testcontainers` | Docker-based integration test dependencies |
| `faker` | Realistic test data generation |
| `nock` | HTTP interception (Node.js only) |

## Anti-Patterns

- Mocking internal module imports instead of injecting dependencies
- `as any` to suppress type errors in tests
- Testing implementation details (internal state, private methods)
- Snapshot tests for logic validation
- Missing async error handling in tests
- Tests coupled to specific mock call counts

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `ts_runtime_validation`: type safety in test utilities
