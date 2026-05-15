# .NET (C#)

Keep domain rules independent from transport and hosting concerns. Use explicit contracts at public boundaries. Keep asynchronous flows observable.

## Project Organization

Separate domain, application, infrastructure, and API layers into distinct projects or folders. Keep framework dependencies (ASP.NET Core, EF Core) at the edges. Domain project should have no framework references.

## Domain Modeling

Use records for value objects (C# 9+):

```csharp
public record OrderId(string Value);
public record Money(decimal Amount, string Currency);
```

Use discriminated unions via abstract records or OneOf for state modeling. Make invalid states unrepresentable through constructors and factory methods.

## Error Handling

Prefer Result types or domain exceptions over generic `Exception`. Do not use exceptions for control flow. Preserve causal chains with inner exceptions.

```csharp
public class OrderNotFoundError : DomainException
{
    public OrderNotFoundError(OrderId id)
        : base($"Order not found: {id.Value}") { }
}
```

## Dependency Injection

Use constructor injection exclusively. Register services in the composition root. Avoid service locator pattern. Keep `IServiceProvider` out of domain code.

## Async

Use `async`/`await` throughout. Do not block on async code (`.Result`, `.Wait()`). Use `CancellationToken` for cooperative cancellation. Prefer `ValueTask` for hot paths.

## Nullability

Enable nullable reference types project-wide. Treat compiler warnings as errors for null safety. Avoid `null` as a business state signal.

## Testing

Use xUnit or NUnit. Prefer domain-level unit tests. Use `WebApplicationFactory` for integration tests. Use Testcontainers for real infrastructure.

```csharp
[Fact]
public void Rejects_empty_order()
{
    var result = Order.Create(Array.Empty<OrderLine>());
    Assert.IsType<OrderError.Empty>(result);
}
```

## Recommended Ecosystem Libraries

| Category | Package | Purpose |
|----------|---------|---------|
| Testing | xUnit or NUnit | Test framework |
| Mocking | Moq or NSubstitute | Interface-based test doubles |
| Assertions | FluentAssertions | Readable assertion syntax |
| JSON | `System.Text.Json` or Newtonsoft.Json | Serialization (prefer STJ for new code) |
| ORM | EF Core | Database access with migrations |
| Logging | Serilog or NLog | Structured sink-based logging |
| Resilience | Polly | Retry, circuit breaker, timeout policies |
| CQRS | MediatR | In-process request/notification dispatch |
| Validation | FluentValidation | Rule-based input validation |
| HTTP client | `IHttpClientFactory` (built-in) | Managed HttpClient lifecycle |

Prefer packages compatible with .NET's built-in DI container and hosted service model.

## Anti-Patterns

- Service locator pattern or `IServiceProvider` in domain code
- Blocking on async (`.Result`, `.Wait()`)
- Framework attributes scattered through domain models
- Missing `CancellationToken` on async boundaries
- Nullable reference types disabled
- `Exception` for control flow
- Mutable DTOs used as domain objects
- Deep inheritance hierarchies for services

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-mixed-responsibilities
- `architecture_boundary`: dependency-direction
- `testability`: untestable-design
