# Testing .NET

Testing conventions for .NET projects using xUnit, NUnit, and related tools.

## Test Organization

- Unit tests: separate test project per production project
- Integration tests: dedicated project with `WebApplicationFactory` or test containers
- Use class fixtures for shared expensive setup

## xUnit

```csharp
[Fact]
public void Rejects_empty_name()
{
    var exception = Assert.Throws<DomainException>(
        () => Name.Create("")
    );
    Assert.Equal("Name cannot be empty", exception.Message);
}
```

## Theory Tests

Use `[Theory]` with `[InlineData]`, `[MemberData]`, or `[ClassData]` for parameterized tests.

## Assertions

Use FluentAssertions for readable assertions: `result.Should().BeEquivalentTo(expected)`.

## Mocking

Use Moq or NSubstitute for boundary mocking. Mock interfaces, not implementations. Use dependency injection.

## Integration Testing

Use `WebApplicationFactory<T>` for ASP.NET integration tests. Use Testcontainers for database tests. Isolate tests from external services.

## Anti-Patterns

- Testing with real HTTP calls or databases without isolation
- Mocking implementation details
- Shared mutable state via `IClassFixture` misuse
- Tests that assert on exact exception messages across cultures
- Missing async test patterns (`Task`-returning test methods)
- Excessive use of `[SetUp]`/constructor for unrelated state

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `clean_code`: test readability and maintenance
