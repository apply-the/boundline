# ASP.NET Core Guidelines

## Principi

ASP.NET Core dà ottimi strumenti per DI, middleware, configuration, logging e hosting. Il codice resta sano se controller/minimal API sono boundary sottili e la logica applicativa è separata.

## Controller o Minimal API sottili

Endpoint:

- binding input
- validation
- auth
- chiamata service/use case
- mapping result
- status code

Non business logic profonda.

## Dependency injection

Usare DI nativa e constructor injection.

```csharp
public sealed class OrderService
{
    private readonly IOrderRepository _repository;

    public OrderService(IOrderRepository repository)
    {
        _repository = repository;
    }
}
```

## Options pattern

Usare options tipizzate e validate.

```csharp
builder.Services
    .AddOptions<PaymentOptions>()
    .BindConfiguration("Payment")
    .ValidateDataAnnotations()
    .ValidateOnStart();
```

Non leggere configuration ovunque.

## Middleware

Middleware per cross-cutting concerns:

- correlation ID
- request logging
- auth
- exception handling
- security headers

Non business logic.

## Error handling

Usare exception handling middleware o ProblemDetails.

Mappare errori applicativi a status code coerenti.

## Entity Framework Core

Regole:

- evitare DbContext in controller per logica complessa
- usare query esplicite
- evitare N+1
- usare projection per read model
- transazioni esplicite per casi complessi
- attenzione a tracking non necessario
- non esporre entity direttamente

## Async

Usare async end-to-end.

Regole:

- evitare `.Result` e `.Wait()`
- passare `CancellationToken`
- timeout per chiamate esterne
- evitare fire-and-forget senza background service gestito

## Background services

Usare `IHostedService` o `BackgroundService`.

Regole:

- cancellation token rispettato
- error handling
- logging
- retry/backoff
- scope DI creato correttamente

## Logging

Usare structured logging.

```csharp
_logger.LogInformation("Order created {OrderId}", orderId);
```

Non usare interpolazione nei log.

## Testing

- unit test su service/domain
- integration test con `WebApplicationFactory`
- database test con container o provider realistico
- evitare mock eccessivo di ASP.NET internals

## Anti-pattern

- controller con business logic
- service locator tramite `IServiceProvider`
- configuration letta ovunque
- `.Result` su async
- entity EF esposte direttamente
- middleware con logica di dominio
- background task non gestiti
