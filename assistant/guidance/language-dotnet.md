# .NET Language Guidance

Use C# and .NET types to make invariants, cancellation, and resource cleanup explicit.

- Prefer constructor injection and `record`, `readonly record struct`, or immutable value objects for domain data.
- Separate domain logic from controllers, EF models, and transport DTOs.
- Use `using` or `await using` for deterministic cleanup of disposable resources.
- Distinguish expected domain failures from exceptional infrastructure failures with explicit result or exception modeling.
- Pass cancellation tokens through I/O and long-running operations instead of ignoring cancellation at the boundary.
- Keep logging structured and keep correlation data flowing across async boundaries.
