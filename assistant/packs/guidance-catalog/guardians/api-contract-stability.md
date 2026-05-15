# API Contract Stability Guardian

Enforce backward-compatible API evolution and prevent silent breaking changes to published contracts.

## Rules

### breaking-field-removal
Removing or renaming fields, endpoints, or enum values from a published API breaks existing consumers. Use deprecation periods or versioning instead of direct removal.

Triggers: removing fields from response types, renaming JSON keys, removing endpoints without redirect, removing enum values from shared schemas.

### missing-versioning
Breaking changes to published APIs must be accompanied by a version increment. Without versioning, consumers cannot pin to a compatible contract.

Triggers: breaking response shape changes without version bump, new required request fields on existing endpoints, changed semantics without version change.

### inconsistent-error-shape
Error responses across an API must use a consistent structure. Consumers cannot build reliable error handling against varying shapes.

Triggers: different error formats between endpoints, missing error codes on some responses, inconsistent HTTP status code usage for similar failures.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to all languages. Cross-cutting; relevant when changes touch public HTTP, gRPC, GraphQL, or message-based API surfaces.
