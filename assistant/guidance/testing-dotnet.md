# .NET Testing Guidance

Use unit, integration, and host-level tests to prove .NET behavior without over-mocking the framework.

- Unit test domain and application services without bringing up the web host unless the host behavior is the subject.
- Use integration tests for serialization, auth, filters, persistence, and endpoint wiring.
- Prefer realistic fakes or test containers over brittle mocks of EF, HTTP, or DI internals.
- Keep data setup explicit and isolated per test so ordering does not matter.
- Verify cancellation, error mapping, and boundary behavior where they affect operators or clients.
