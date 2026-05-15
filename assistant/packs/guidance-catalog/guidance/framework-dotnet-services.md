# .NET Service Frameworks

Conventions for .NET server frameworks including ASP.NET Core and minimal APIs.

## Architecture

Separate controllers/endpoints (transport) from services (application) from domain from infrastructure. Keep framework dependencies (`Microsoft.AspNetCore.*`, `EntityFrameworkCore`) out of domain projects.

## ASP.NET Core

Use constructor injection via built-in DI. Keep controllers thin. Use MediatR or direct service injection for application orchestration.

```csharp
[ApiController]
[Route("api/orders")]
public class OrderController : ControllerBase
{
    private readonly IOrderService _orderService;

    public OrderController(IOrderService orderService)
        => _orderService = orderService;
}
```

## Minimal APIs

Keep endpoint definitions concise. Delegate to services. Use typed request/response records with `FluentValidation` or `DataAnnotations`.

## Error Handling

Use middleware for global exception handling. Map domain exceptions to `ProblemDetails` responses. Do not expose internal errors to clients.

## Database Access

Use EF Core with repository pattern for complex domains. Use explicit transactions. Avoid tracking queries for read-only operations. Keep DbContext scoped to request lifetime.

## Anti-Patterns

- Business logic in controllers or endpoints
- EF Core entities exposed as API contracts
- Missing `IServiceProvider` abstraction (service locator)
- Domain projects referencing ASP.NET or EF Core
- Missing `CancellationToken` on async controller actions
- Global exception handling that swallows errors

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, public-contract-stability
- `clean_code`: no-mixed-responsibilities
- `security_boundary`: input validation at request boundary
