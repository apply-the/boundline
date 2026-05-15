# Cross-Language Engineering Guidance

Use language features to make ownership, invariants, and failures explicit instead of relying on convention alone.

- Prefer semantic types, value objects, or discriminated states over raw strings, numbers, and boolean bundles.
- Make invalid states difficult to represent through constructors, enums, sealed hierarchies, or validated factories.
- Inject dependencies from the edge; do not hide clients, clocks, or repositories inside business logic.
- Separate pure logic from I/O, framework glue, filesystem, network, and clock access.
- Propagate expected failures with domain-specific errors or typed exceptions, and add context before crossing a boundary.
- Tie resource lifetime to scope or explicit cleanup primitives so error paths remain safe.
- Avoid hidden global state, silent fallbacks, and implicit initialization.
- Design logging and tracing at the boundary, not as a last-minute patch.
