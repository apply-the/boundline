# .NET Testing Guidelines

## Principi

In .NET, scegliere xUnit, NUnit o MSTest in modo coerente. Le regole chiave restano: test veloci per dominio, integration test realistici per ASP.NET/EF, mock solo dove utile.

## xUnit

```csharp
public sealed class OrderServiceTests
{
    [Fact]
    public void Rejects_empty_order()
    {
        // ...
    }
}
```

## Theory

Usare `[Theory]` per casi tabellari.

```csharp
[Theory]
[InlineData("")]
[InlineData(" ")]
public void Rejects_blank_email(string value)
{
    // ...
}
```

## Assertions

Usare FluentAssertions se adottato dal team.

```csharp
result.Should().BeEquivalentTo(expected);
```

## Mock

Moq/NSubstitute/FakeItEasy vanno usati con moderazione.

Preferire fake quando il comportamento è più leggibile.

## ASP.NET Core integration test

Usare `WebApplicationFactory`.

```csharp
public sealed class OrdersApiTests : IClassFixture<WebApplicationFactory<Program>>
{
}
```

## EF Core

Non usare InMemory provider per test che devono verificare comportamento SQL/relazionale.

Preferire:

- SQLite in-memory per casi semplici
- container database reale per integration critici

## Async

Test async devono ritornare `Task`.

```csharp
[Fact]
public async Task Creates_order()
{
    // ...
}
```

Non usare `.Result` o `.Wait()`.

## Time

Iniettare clock/time provider.

## Anti-pattern

- InMemory EF usato come se fosse DB reale
- mock di DbSet complessi
- `.Result` nei test
- test integration senza isolamento DB
- fixture condivise mutabili
