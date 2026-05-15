# JUnit Guidelines

## Principi

JUnit è lo standard de facto per test Java e JVM. Usarlo bene significa scrivere test leggibili, isolati e veloci, senza avviare framework completi quando non serve.

## JUnit 5

Preferire JUnit 5 nei nuovi progetti.

```java
class OrderServiceTest {
    @Test
    void rejectsEmptyOrder() {
        // ...
    }
}
```

## Naming

Usare nomi descrittivi.

```java
@Test
void createsOrderWhenCustomerIsActive() {}
```

Oppure `@DisplayName` per descrizioni più leggibili.

## Arrange Act Assert

```java
@Test
void rejectsEmptyOrder() {
    var command = CreateOrderCommand.empty();

    var result = service.createOrder(command);

    assertThat(result).isEqualTo(CreateOrderResult.invalid("empty_order"));
}
```

## Assertions

Usare assertion library leggibile come AssertJ.

```java
assertThat(order.lines()).hasSize(1);
```

## Parameterized tests

Usare `@ParameterizedTest` per casi simili.

```java
@ParameterizedTest
@ValueSource(strings = {"", " ", "\t"})
void rejectsBlankOrderId(String value) {}
```

## Extension

Usare extension per setup trasversale, ma non nascondere troppo.

## Mock

Usare Mockito con moderazione.

Preferire fake quando il comportamento è più importante dell’interazione.

## Test lifecycle

Evitare stato mutabile condiviso tra test.

`@BeforeEach` va bene per setup semplice. Se diventa enorme, usare builder/factory.

## Anti-pattern

- test che richiedono Spring per logica pura
- mock di value object/domain object
- `@BeforeEach` enorme
- assert multipli non correlati
- test dipendenti dall’ordine
- eccezioni attese testate in modo opaco
