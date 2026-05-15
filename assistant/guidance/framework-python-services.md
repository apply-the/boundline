# Python Service Framework Guidance

Apply FastAPI and Django as request and persistence shells around explicit domain behavior.

- Keep views, endpoints, and routers thin: validate input, authorize, call an application service, map the response.
- Use serializers, schemas, or forms at the boundary instead of letting unvalidated request data leak into business logic.
- Keep service and domain logic out of ORM callbacks, signal handlers, and view functions.
- Keep transactions narrow and do not mix database work with long-running external calls in the same request path.
- Treat auth, permissions, rate limits, and audit concerns as explicit boundary decisions.
- Use background jobs or task queues for slow or fan-out side effects instead of stretching request latency.
