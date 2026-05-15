# JVM (Java, Kotlin)

Model domain state with explicit types, keep framework annotations out of core logic, and make contract evolution intentional.

## Package Organization

Prefer stable package boundaries over deep inheritance hierarchies. Separate domain, application, infrastructure, and presentation packages. Keep framework-specific annotations at the edges.

## Domain Modeling

Use records (Java 16+) or data classes (Kotlin) for value objects:

```java
public record OrderId(String value) {
    public OrderId {
        Objects.requireNonNull(value);
    }
}
```

Use sealed interfaces/classes for state modeling:

```java
public sealed interface PaymentStatus
    permits Pending, Completed, Failed {}
```

## Error Handling

Prefer unchecked domain exceptions or Result types over checked exceptions for business logic. Keep exception hierarchies shallow. Preserve causal chains.

```java
public class OrderNotFoundError extends RuntimeException {
    public OrderNotFoundError(OrderId id) {
        super("Order not found: " + id.value());
    }
}
```

## Dependency Injection

Use constructor injection. Avoid field injection. Keep the DI framework (Spring, CDI, Dagger) at the composition root, not in domain code.

## Nullability

Avoid `null` in domain models. Use `Optional` only for return types, never for fields or parameters. In Kotlin, leverage the type system's nullability.

## Immutability

Prefer immutable objects for domain state. Use builders or factory methods for complex construction. Avoid mutable setters in value objects.

## Testing

Use JUnit 5 or Kotest. Prefer domain assertions over framework-coupled integration tests when possible. Use Testcontainers for real infrastructure in integration tests.

```java
@Test
void rejectsEmptyOrder() {
    var result = Order.create(List.of());
    assertThat(result).isInstanceOf(OrderError.Empty.class);
}
```

## Recommended Ecosystem Libraries

| Category | Library | Purpose |
|----------|---------|---------|
| Testing | JUnit 5, Mockito, AssertJ | Test framework, mocking, fluent assertions |
| JSON | Jackson or Gson | Serialization/deserialization |
| Logging | SLF4J + Logback | Facade-based structured logging |
| Database | Flyway or Liquibase | Schema migration management |
| Resilience | Resilience4j | Circuit breakers, retries, rate limiting |
| Validation | Jakarta Bean Validation | Annotation-based constraint checking |
| HTTP client | Java HttpClient (stdlib) or OkHttp | Async-capable HTTP |
| Build | Gradle (Kotlin DSL) or Maven | Dependency and build management |
| Kotlin extras | `kotlinx.serialization`, `kotlinx.coroutines` | Kotlin-native patterns |
| DI | Spring Framework or Dagger/Hilt (Android) | Constructor injection |

Use Lombok with caution: it improves ergonomics but hides generated code from tools and reviewers.

## Anti-Patterns

- `Optional` in fields or parameters
- Checked exceptions for domain errors
- Framework annotations scattered through domain code
- Deep inheritance hierarchies for business concepts
- Mutable domain objects with setters
- Service locator pattern
- `null` as a business state signal
- God classes with mixed responsibilities

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-mixed-responsibilities
- `architecture_boundary`: dependency-direction
- `testability`: untestable-design
