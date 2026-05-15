# Testing JVM

Testing conventions for JVM projects using JUnit 5, Kotest, and related tools.

## Test Organization

- Unit tests: mirror source structure, `*Test.java` or `*Test.kt` naming
- Integration tests: separate source set or `@Tag("integration")` marker
- Use `@Nested` classes for grouping related scenarios

## JUnit 5

```java
@Test
void rejectsEmptyName() {
    var exception = assertThrows(
        DomainException.class,
        () -> Name.of("")
    );
    assertEquals("Name cannot be empty", exception.getMessage());
}
```

## Parameterized Tests

Use `@ParameterizedTest` with `@ValueSource`, `@CsvSource`, or `@MethodSource` for multiple inputs.

## Assertions

Use AssertJ for fluent assertions. Prefer specific assertions (`assertThat(x).isEqualTo(y)`) over generic `assertTrue`.

## Mocking

Use Mockito for boundary mocking. Mock interfaces, not concrete classes. Avoid mocking domain logic. Use constructor injection so mocks are straightforward.

## Spring Boot Testing

Use `@SpringBootTest` sparingly (slow). Prefer `@WebMvcTest` for controller tests, `@DataJpaTest` for repository tests. Use sliced contexts.

## Anti-Patterns

- `@SpringBootTest` for unit tests
- Mocking concrete classes instead of interfaces
- Missing `@Nested` grouping for related tests
- Tests that require database or network without containers
- Assert methods with unclear failure messages
- Shared mutable state between test methods

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `clean_code`: test readability and maintenance
