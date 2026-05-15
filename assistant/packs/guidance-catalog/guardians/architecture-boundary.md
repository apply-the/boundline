# Architecture Boundary Guardian

Check dependency direction, ownership boundaries, and public contract stability when a change crosses module seams.

## Rules

### dependency-direction
Dependencies must flow inward: infrastructure depends on domain, not the reverse. Transport and framework code must not leak into core business logic.

Triggers: domain module importing framework types, core logic referencing database or HTTP crates, shared utilities depending on application-specific modules.

### data-ownership-boundary
Each bounded context or module owns its data types. Sharing internal types across boundaries couples modules and prevents independent evolution.

Triggers: internal types exposed across crate/package boundaries, one module directly accessing another module's persistence layer, shared mutable state between contexts.

### public-contract-stability
Published API surfaces (HTTP endpoints, library interfaces, message schemas, CLI arguments) must not break existing consumers without versioning.

Triggers: removing fields from public types, changing serialization shapes, renaming public functions, changing return types, removing enum variants.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages and architectures. Relevant when changes cross module, crate, package, or service boundaries.
