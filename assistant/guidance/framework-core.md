# Cross-Framework Delivery Guidance

Use frameworks for composition, transport, and lifecycle management; keep business behavior outside framework-specific magic.

- Keep handlers, controllers, pages, and components thin: validate input, call a use case, map the result.
- Separate domain rules from framework DTOs, request models, serializers, and view state.
- Keep state by responsibility: local UI state near the component, server state in a cache-aware data layer, global state minimal.
- Make loading, empty, error, and success states explicit for every async boundary.
- Keep transactions, background work, and external calls narrow and visible.
- Use middleware, filters, or interceptors for cross-cutting concerns, not hidden business logic.
- Prefer framework-independent tests for domain behavior and focused integration tests for framework boundaries.
