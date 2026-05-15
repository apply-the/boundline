# Python Service Framework Guidance

## Purpose

This guidance defines framework-level practices for Python services and APIs, including FastAPI, Django, Django Ninja, Flask, workers, and automation APIs.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, architecture decision, or Canon-governed standard.

## Framework Posture

Recommended modern options:
- FastAPI for typed async-friendly APIs
- Django Ninja for typed APIs in Django systems
- Django for full-stack/product applications
- Flask for small services only when simplicity is intentional

Framework choice is repository-dependent.

This guidance does not mandate migration away from existing frameworks.

## Core Design Principles

### Route Thinness

Route/view functions should:
- validate input
- map request to command/query
- call application service
- map result to response
- map errors to API shape

They should not:
- contain domain policy
- directly manipulate persistence for complex behavior
- mix authorization, validation, persistence, and response mapping in one function

### Schema Ownership

Pydantic models are excellent boundary schemas.

Do not automatically treat them as domain models.

Keep distinction between:
- request schema
- response schema
- domain object
- persistence entity
- integration payload

In small systems, these may intentionally collapse. In governed systems, collapsing them should be explicit.

### Dependency Injection

Framework dependency injection is useful at boundaries.

Avoid:
- hiding domain dependencies behind framework magic
- injecting framework-specific objects deep into domain logic
- using global module state for runtime dependencies

### Async Boundaries

FastAPI and async frameworks require care.

Avoid:
- blocking I/O in async route handlers
- synchronous database clients in async paths
- unbounded async task creation
- fire-and-forget tasks without supervision

Prefer:
- explicit background worker systems
- task groups where applicable
- timeouts
- cancellation handling

### Error Mapping

API errors should be stable.

Do not leak:
- raw exception text
- stack traces
- ORM errors
- provider internals

Expose:
- stable error code
- safe message
- correlation ID
- validation details where appropriate

### Django Boundary Guidance

For Django:
- keep fat models under control
- avoid putting all business behavior in views
- make service/application layer explicit when workflows grow
- avoid signal-based hidden business workflows unless intentional and documented
- be careful with queryset evaluation and N+1 queries

### Validation And Serialization

Validate at external boundaries.

Use:
- Pydantic v2
- Django forms/serializers where appropriate
- explicit mapping between external and internal shapes

Avoid:
- trusting `dict` payloads inside domain logic
- passing raw request objects into services
- untyped JSON from external systems

## Testing Guidance

Recommended:
- pytest
- framework test clients
- fixtures with clear ownership
- test builders
- contract tests for external APIs
- database transaction isolation

Avoid:
- shared mutable fixtures
- hidden autouse fixture behavior
- tests dependent on order
- sleeps
- mocking the function under test instead of external dependencies

## Anti-Patterns

- domain logic in route/view functions
- raw dictionaries as domain commands
- swallowed framework exceptions
- Pydantic request models used as persistence entities without intent
- blocking I/O in async handlers
- fire-and-forget async tasks
- signal-driven hidden workflows
- unbounded queryset N+1 behavior
- framework request object passed to domain service

## Guardian Hooks

Recommended guardians:
- python-route-boundary-guardian
- python-schema-ownership-guardian
- python-framework-leakage-guardian
- python-async-service-guardian
- python-django-signal-guardian
- python-api-error-mapping-guardian

## Structured Finding Example

```json
{
  "guardian": "python-schema-ownership",
  "rule": "request-schema-used-as-domain-object",
  "disposition": "concern",
  "summary": "The FastAPI request schema is passed directly through the domain layer and persisted without explicit mapping.",
  "evidence_refs": ["app/orders/routes.py", "app/orders/service.py"],
  "recommended_action": "Map the request schema into an explicit domain command or value object at the boundary."
}
```

## Lifecycle Usage

Planning:
- identify API schema, domain object, and persistence ownership

Architecture:
- check framework/domain boundary and dependency flow

Implementation:
- guide route thinness, schema mapping, and async safety

Testing:
- guide framework boundary tests and database isolation

Review:
- check route bloat, schema leakage, and hidden signal workflows
