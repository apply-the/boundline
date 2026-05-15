# JVM Testing Guidance

Use JUnit, TestNG, Spock, and framework test slices to verify behavior without loading the whole container by default.

- Keep unit tests focused on domain rules, validation, mapping, and application services.
- Use slice or focused integration tests for controllers, persistence, serialization, and security boundaries.
- Use Testcontainers or equivalent real dependencies when the integration itself matters.
- Keep transaction and database cleanup explicit so tests do not depend on execution order.
- Avoid full application context startup when a narrower test can falsify the same hypothesis.
- Name tests by scenario and expected behavior, not just method names.
