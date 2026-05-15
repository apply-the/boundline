# Language Core

Cross-language engineering principles that apply regardless of specific language or runtime. These complement language-specific guidance with universal practices.

## Type Safety

Use the type system to prevent bugs at compile time rather than catching them at runtime. Prefer:
- Named types over raw primitives for domain concepts
- Discriminated unions for alternative states
- Non-nullable types where nullability is not meaningful
- Immutable values where mutation is not required

## Boundary Validation

Validate all data entering the system from external sources. Internal code should trust validated types rather than re-checking everywhere.

External sources: HTTP requests, CLI arguments, file content, environment variables, queue messages, database results, third-party API responses, AI-generated output.

## Error Propagation

Handle errors at the appropriate level. Preserve causal chains. Do not swallow errors silently. Use typed errors where consumers need stable handling behavior.

## Dependency Direction

Domain logic should not depend on infrastructure. Infrastructure adapts to domain interfaces. Framework and transport concerns stay at the edges.

## Immutability By Default

Prefer immutable data structures for domain state. Mutation should be explicit, scoped, and visible. Immutability simplifies reasoning about concurrency, testing, and debugging.

## Deterministic Behavior

Isolate sources of non-determinism (time, randomness, I/O, external services) behind explicit boundaries. This enables deterministic testing and reproducible behavior.

## Anti-Patterns

- Raw primitives for domain concepts across any language
- Errors swallowed without logging or propagation
- Domain logic coupled to framework or infrastructure details
- Mutable shared state without explicit coordination
- Non-deterministic behavior hidden behind pure-looking interfaces
- Missing validation at system boundaries

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-hidden-side-effects, no-mixed-responsibilities
- `architecture_boundary`: dependency-direction
- `testability`: untestable-design (when non-determinism is hidden)
