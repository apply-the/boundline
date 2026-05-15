# TypeScript And Node.js Guidance

## Purpose

This guidance defines idiomatic TypeScript and Node.js practices for AI-assisted implementation, review, testing, and refactoring.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy or Canon-governed standards.

## Version Posture

Active support window:
- Node.js 18+
- TypeScript 5+

Target excellence:
- Node.js 20/22 LTS
- TypeScript strict mode
- modern type narrowing and `satisfies`

Legacy warnings:
- Node.js 16 or older
- TypeScript 4.x
- `strict: false`
- unvalidated `any`
- duplicated runtime schema and compile-time type

## Type Safety

Use TypeScript to encode domain constraints.

Prefer:
- discriminated unions
- branded types or opaque wrappers for IDs
- `satisfies` for configuration validation
- exhaustive switch checks
- runtime validation at boundaries

Avoid:
- `any`
- broad `as` casts
- stringly typed state
- manual duplication between schema and interface

## Validation

Validate at runtime boundaries:
- HTTP input
- environment variables
- external API responses
- message payloads
- persisted JSON
- AI-produced structured data

Recommended libraries:
- Zod for common schema validation
- Valibot where modularity and size matter

When using Zod:
- infer TypeScript types from schemas
- avoid duplicating interface definitions

## API And Service Design

For type-safe backend APIs:
- Hono and Fastify are good modern choices
- tRPC is useful for end-to-end TypeScript boundaries
- keep domain logic independent from HTTP framework handlers

For database access:
- Drizzle and Kysely are strong type-safe SQL options
- avoid raw string SQL in business logic without boundary isolation

## Error Handling

Do not throw arbitrary strings.

Prefer:
- typed domain errors
- result-like patterns where expected failures are common
- consistent error mapping at HTTP boundaries
- problem detail style responses where applicable

## Async And Concurrency

Avoid:
- unbounded Promise.all over external calls
- missing cancellation/timeout behavior
- swallowed promise rejections
- side effects in implicit global initialization

Prefer:
- explicit concurrency limits
- abort signals
- timeout wrappers
- structured logging with correlation IDs

## Testing

Recommended:
- Vitest for unit tests
- MSW for HTTP mocking
- Playwright for browser E2E
- user-centric selectors and assertions

Avoid:
- asserting implementation details
- excessive snapshot tests
- `cy.wait(N)` style timing
- tests that mock the code under test instead of external dependencies

## Anti-Patterns

- `any` in domain logic
- manual `as` casting without validation
- `strict: false`
- duplicated Zod schemas and TS interfaces
- throwing strings
- untyped environment variables
- unchecked JSON parsing
- hidden global mutable state

## Guardian Hooks

Recommended guardians:
- ts-strictness-guardian
- runtime-validation-guardian
- duplicated-schema-guardian
- async-boundary-guardian
- node-error-handling-guardian

## Structured Finding Example

```json
{
  "guardian": "runtime-validation",
  "rule": "unchecked-external-json",
  "disposition": "concern",
  "summary": "External provider response is parsed without runtime validation before entering domain logic.",
  "evidence_refs": ["src/providers/payment.ts"],
  "recommended_action": "Introduce a boundary schema and infer the internal type from it."
}
```
