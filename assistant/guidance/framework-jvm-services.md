# JVM Service Framework Guidance

Apply Spring, Quarkus, and Micronaut as delivery frameworks around explicit use cases and narrow transactions.

- Keep controllers and handlers thin: validate, authorize, call a use case, and map the result.
- Keep domain behavior out of entity callbacks, framework annotations, and hidden auto-wiring side effects.
- Use DTOs at transport boundaries and map them explicitly into domain commands or queries.
- Keep transactions narrow and do not wrap remote calls or event fan-out inside database transactions.
- Centralize error mapping, logging, and security filters while keeping business rules in the application layer.
- Make async work, retries, and idempotency explicit when using events, schedulers, or message listeners.
