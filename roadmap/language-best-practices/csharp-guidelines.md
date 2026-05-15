# C# Guidelines

## Principi

C# funziona bene con immutabilità, dependency injection esplicita, async corretto e gestione deterministica delle risorse tramite `IDisposable`.

## Constructor injection

Iniettare dipendenze nel costruttore.

```csharp
public sealed class OrderService
{
    private readonly IOrderRepository _repository;
    private readonly IPaymentClient _paymentClient;

    public OrderService(IOrderRepository repository, IPaymentClient paymentClient)
    {
        _repository = repository;
        _paymentClient = paymentClient;
    }
}
```

Evitare service locator e dipendenze create internamente.

## Immutabilità

Preferire `record`, `readonly record struct` e proprietà init-only.

```csharp
public readonly record struct OrderId(string Value);
```

Per oggetti di dominio più complessi, validare nel costruttore o factory.

```csharp
public sealed record EmailAddress
{
    public string Value { get; }

    private EmailAddress(string value)
    {
        Value = value;
    }

    public static EmailAddress Create(string value)
    {
        if (string.IsNullOrWhiteSpace(value) || !value.Contains('@'))
        {
            throw new ArgumentException("Invalid email address", nameof(value));
        }

        return new EmailAddress(value);
    }
}
```

## Tipi semantici

Evitare `Guid`, `string` e `int` ovunque senza distinzione.

```csharp
public readonly record struct CustomerId(Guid Value);
public readonly record struct OrderId(Guid Value);
```

## Error handling

Usare eccezioni specifiche per errori eccezionali e result type per errori di dominio attesi quando appropriato.

```csharp
public sealed class OrderNotFoundException : Exception
{
    public OrderNotFoundException(OrderId orderId)
        : base($"Order not found: {orderId.Value}")
    {
    }
}
```

Regole:

- non catturare `Exception` genericamente senza motivo
- non ingoiare eccezioni
- non usare eccezioni come normale controllo di flusso in hot path
- aggiungere contesto
- preservare stack trace usando `throw;`, non `throw ex;`

## Resource management

Usare `using` e `await using` per risorse `IDisposable` e `IAsyncDisposable`.

```csharp
using var stream = File.OpenRead(path);
```

```csharp
await using var connection = await dataSource.OpenConnectionAsync(cancellationToken);
```

Questo è l’equivalente pratico del cleanup deterministico in C#.

## Async

Usare async end-to-end.

### Da evitare

```csharp
var result = SomeAsync().Result;
```

### Preferibile

```csharp
var result = await SomeAsync(cancellationToken);
```

Regole:

- non bloccare su task async con `.Result` o `.Wait()`
- passare `CancellationToken`
- rispettare cancellation
- usare timeout espliciti per I/O
- evitare `async void` salvo event handler

## CancellationToken

Accettare `CancellationToken` nei metodi che fanno I/O o lavoro cancellabile.

```csharp
public Task<Order> FindAsync(OrderId orderId, CancellationToken cancellationToken)
{
    // ...
}
```

## Nullable reference types

Abilitare nullable reference types.

```xml
<Nullable>enable</Nullable>
```

Non usare `null` come stato implicito quando si può modellare meglio l’assenza.

## Logging

Usare structured logging.

```csharp
_logger.LogInformation(
    "Order created {OrderId} for customer {CustomerId}",
    orderId,
    customerId);
```

Regole:

- non interpolare stringhe nei log
- non loggare segreti
- includere correlation ID tramite scope/tracing
- non loggare e rilanciare a ogni livello

## Testabilità

Constructor injection e interfacce piccole rendono i test semplici.

```csharp
var repository = new FakeOrderRepository();
var paymentClient = new FakePaymentClient();
var service = new OrderService(repository, paymentClient);
```

Preferire fake esplicite a mock troppo accoppiati ai dettagli interni.

## Cose da evitare

- service locator
- `.Result` o `.Wait()` su task
- `async void`
- `throw ex;`
- nullability disabilitata
- primitive obsession
- dipendenze create dentro i service
- log con string interpolation
- `IDisposable` non rilasciati
