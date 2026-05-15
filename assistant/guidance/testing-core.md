# Cross-Framework Testing Strategy

Treat testing as release confidence engineering, not a coverage vanity exercise.

- Prefer a pyramid: many fast unit tests, targeted integration tests, and a small number of high-value end-to-end flows.
- Name tests by scenario and behavior so failures explain what regressed.
- Keep tests deterministic: no hidden clock, timezone, sleep, random order, or shared mutable state.
- Mock external boundaries, not the logic under test; use fakes when they keep behavior clearer than mocks.
- Use real infrastructure for integration boundaries that matter: database, filesystem, HTTP client, message broker, or CLI surface.
- Treat coverage as a signal; prioritize domain logic, error mapping, and boundary behavior over trivial glue.
- Use contract tests when independent components evolve against a shared API or schema.
