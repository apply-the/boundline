# Domain Modeling

Domain modeling captures business rules and invariants in code structures that prevent illegal states at compile time or construction time.

## Value Objects

Immutable types that represent domain concepts with equality by value. Use for identifiers, quantities, measurements, and compositions.

Enforce invariants at construction: an `EmailAddress` type that cannot hold an invalid email is stronger than a runtime check scattered through the codebase.

## Entities

Types with identity that persists over time. Keep entities focused on invariant enforcement and state transitions. Avoid adding behavior that belongs in services.

## Aggregates

Groups of entities and value objects with a single root that enforces consistency boundaries. Keep aggregates small. Prefer eventual consistency between aggregates.

## Domain Events

Record meaningful state transitions as events. Use events for cross-aggregate communication. Keep events immutable and versioned.

## Invariant Enforcement

Encode business rules as type constraints where possible. Use factory methods that validate preconditions. Make illegal states unrepresentable.

## Anti-Patterns

- Anemic domain models (data bags with no behavior)
- Large aggregates that cause contention
- Mutable value objects
- Business rules scattered across service layers instead of in domain types
- Missing invariant checks at construction boundaries
- Domain events that expose internal implementation details
- Bi-directional associations between aggregates

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-mixed-responsibilities
- `architecture_boundary`: aggregate boundary violations
- `testability`: untestable-design (when invariants are in infrastructure)
