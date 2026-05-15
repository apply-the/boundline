# .NET Service Framework Guidance

Apply ASP.NET Core as a transport and composition shell, not as the owner of domain behavior.

- Keep endpoints and controllers thin: bind input, authorize, call a use case, return a mapped result.
- Validate request models before they enter domain logic and map domain failures centrally to stable error responses.
- Keep EF Core persistence concerns out of domain types and keep transactions narrow.
- Do not hide business rules in filters, middleware, or model binders.
- Keep background services, retries, and outbox or message dispatch behavior explicit.
- Enforce authorization in policies and use cases, not only in the client or route table.
