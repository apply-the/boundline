# Node.js Service Frameworks

Conventions for Node.js/TypeScript server frameworks including Express, NestJS, and Fastify.

## Architecture

Separate transport (routes, middleware, serialization) from application logic (use cases, orchestration) from domain (entities, value objects, policies). Framework decorators and middleware should not appear in domain code.

## Request Handling

Validate all incoming request data with schema validators (Zod, Joi, class-validator). Parse once at the boundary, then pass typed objects inward.

```ts
const handler = async (req: ValidatedRequest<CreateOrderBody>) => {
  const result = await orderService.create(req.body);
  return mapToResponse(result);
};
```

## Error Handling

Use a centralized error handler. Map domain errors to HTTP status codes at the transport boundary. Do not throw generic errors from deep in the stack.

## Middleware

Keep middleware focused on cross-cutting concerns: authentication, rate limiting, correlation IDs, request logging. Avoid business logic in middleware.

## Dependency Injection

NestJS: use constructor injection via modules. Express/Fastify: use factory functions or composition root pattern. Avoid global singletons accessed via import.

## Database Access

Use repository pattern or data-mapper. Keep SQL/ORM details out of domain code. Use transactions explicitly where consistency requires them.

## Anti-Patterns

- Business logic in route handlers or middleware
- Unvalidated request bodies used as domain objects
- ORM entities used as API responses
- Missing centralized error handling
- Global state via module-level singletons
- Database queries in controller layer

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, public-contract-stability
- `ts_runtime_validation`: unchecked-external-json, any-at-boundary
- `security_boundary`: input validation at request boundary
