# Node.js Service Framework Guidance

## Purpose

This guidance defines framework-level practices for Node.js services and APIs, including Express, Fastify, Hono, NestJS, and similar frameworks.

It applies to backend services, API gateways, edge services, and service adapters.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, architecture decision, or Canon-governed standard.

## Framework Posture

Modern recommended options:
- Hono for lightweight type-safe services
- Fastify for high-performance plugin-based services
- NestJS for structured enterprise-style applications
- tRPC for end-to-end TypeScript systems where client/server coupling is intentional

Legacy warning:
- Express remains valid in existing systems, but new work should avoid adding untyped ad hoc handler sprawl when stronger options are available.

This is not a blanket migration rule.

## Core Design Principles

### Handler Thinness

Route handlers should be thin.

They should:
- parse input
- validate boundary data
- call application/domain service
- map output to transport response
- map errors to stable API errors

They should not:
- contain business policy
- directly compose persistence logic
- perform complex orchestration
- embed domain invariants in transport code

### Boundary Validation

Validate all external input.

Inputs:
- path params
- query params
- body
- headers
- cookies
- environment-driven config
- external service responses

Recommended:
- Zod or Valibot
- framework-native validation when strong enough
- single source of truth between runtime schema and TypeScript type

### Error Mapping

Separate internal errors from external API responses.

A service should expose stable:
- status code mapping
- error code
- user-safe message
- correlation ID

Do not return:
- raw exception messages
- stack traces
- database error strings
- provider internals

### Framework Leakage

Domain logic should not depend on framework request/response objects.

Bad:
- domain service takes `Request`
- domain policy reads HTTP headers directly
- repository returns framework-specific response
- domain error imports framework exception class

### Dependency Boundaries

Use dependency injection or explicit construction to keep dependencies visible.

Avoid:
- global singletons hidden across modules
- implicit mutable shared clients
- importing infrastructure directly into domain logic

### Async And Backpressure

Node services must handle async failures explicitly.

Check:
- unhandled promise rejections
- missing timeouts
- unbounded concurrency
- missing abort signals
- missing request cancellation handling
- streaming backpressure

### Observability

Service boundaries should include:
- structured logs
- trace correlation
- request ID
- stable operation names
- metrics for latency and failure
- redaction of sensitive fields

### Security Boundary

Validate:
- authn/authz placement
- tenant isolation
- input size limits
- rate limiting
- CORS policy
- secret handling

Do not:
- implement auth logic ad hoc in route handlers
- trust client-provided ownership fields
- log secrets or tokens

## Testing Guidance

Recommended:
- integration tests for route boundaries
- contract tests for external APIs
- MSW or equivalent for HTTP dependencies
- test builders for request payloads
- fake clocks for time-sensitive behavior

Avoid:
- tests that only mock the service under the handler
- asserting framework internals
- fixed waits
- global shared database state

## Anti-Patterns

- business logic in route handlers
- unchecked `req.body`
- raw `JSON.parse` without validation
- `any` at framework boundaries
- framework request objects passed into domain services
- global mutable clients without lifecycle control
- raw exception messages returned to clients
- missing request timeout
- unbounded `Promise.all`
- duplicated schema and type declarations

## Guardian Hooks

Recommended guardians:
- node-handler-boundary-guardian
- node-runtime-validation-guardian
- node-error-mapping-guardian
- node-framework-leakage-guardian
- node-async-boundary-guardian
- node-service-observability-guardian

## Structured Finding Example

```json
{
  "guardian": "node-framework-leakage",
  "rule": "request-object-in-domain-service",
  "disposition": "concern",
  "summary": "The domain service accepts a framework Request object, coupling business logic to HTTP transport.",
  "evidence_refs": ["src/accounts/account-service.ts"],
  "recommended_action": "Map request data at the route boundary and pass an explicit domain command object."
}
```

## Lifecycle Usage

Planning:
- identify API boundary, validation ownership, and auth placement

Architecture:
- check service boundary and framework/domain separation

Implementation:
- guide handler structure, validation, error mapping, and observability

Testing:
- guide boundary-level tests and contract tests

Review:
- check framework leakage, missing validation, and async failure handling
