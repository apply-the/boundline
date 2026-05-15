# Go Testing Guidance

Use Go tests to keep behavior clear, table-driven where useful, and deterministic under concurrency.

- Prefer table-driven tests for behavior matrices that share the same setup and assertion shape.
- Keep package-level helpers small and visible so test logic stays easier to read than the implementation.
- Use integration tests for database, HTTP, queue, and filesystem boundaries instead of mocking what you need to verify.
- Make time, cancellation, and concurrency explicit in tests; avoid sleeps as synchronization.
- Verify wrapped error behavior and boundary contracts when callers depend on them.
