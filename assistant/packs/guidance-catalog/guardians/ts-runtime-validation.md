# TypeScript Runtime Validation Guardian

Check external JSON boundaries for unchecked shapes, schema drift, and missing runtime validation.

## Rules

### unchecked-external-json
JSON from external sources (API responses, file reads, message queues, user input) must be validated against a schema before use. Type assertions (`as T`) do not provide runtime safety.

Triggers: `JSON.parse()` results used without validation, `as` casts on external data, fetch responses accessed without schema parsing.

### any-at-boundary
System boundaries should use validated types, not `any`. The `any` type at a boundary disables the type system for all downstream code.

Triggers: function parameters typed as `any` at API handlers, middleware that passes `any` to business logic, generic catch blocks that lose type information.

### duplicated-schema-and-type
Runtime schemas (Zod, Joi, class-validator) and TypeScript types for the same data should be derived from a single source. Maintaining both separately causes drift.

Triggers: hand-written interfaces alongside equivalent Zod schemas, OpenAPI types and separate TypeScript interfaces, validation code that does not match the type definition.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to TypeScript and JavaScript projects. Language-specific guardian.
