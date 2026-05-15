# Groovy Guidelines

## Principi

Groovy è espressivo e comodo per build, scripting e DSL, ma la dinamicità può nascondere errori. Nei progetti grandi, usare tipi espliciti, `@CompileStatic` dove possibile e dipendenze chiare.

## Static compilation dove utile

Per codice applicativo o librerie interne, preferire `@CompileStatic`.

```groovy
import groovy.transform.CompileStatic

@CompileStatic
final class OrderService {
    private final OrderRepository repository

    OrderService(OrderRepository repository) {
        this.repository = repository
    }
}
```

Lasciare dinamico solo dove serve davvero, come DSL controllate o script piccoli.

## Dependency injection

Passare dipendenze dal costruttore.

```groovy
@CompileStatic
final class OrderService {
    private final OrderRepository repository
    private final PaymentClient paymentClient

    OrderService(OrderRepository repository, PaymentClient paymentClient) {
        this.repository = repository
        this.paymentClient = paymentClient
    }
}
```

Evitare lookup globali dentro la business logic.

## Tipi semantici

Usare classi o record-like object per concetti di dominio.

```groovy
@CompileStatic
final class OrderId {
    final String value

    OrderId(String value) {
        if (!value) {
            throw new IllegalArgumentException('OrderId cannot be empty')
        }
        this.value = value
    }
}
```

## Evitare mappe dinamiche nel dominio

### Da evitare

```groovy
def createOrder(Map data) {
    def customerId = data.customerId
}
```

### Preferibile

```groovy
@CompileStatic
final class CreateOrderCommand {
    final CustomerId customerId
    final List<OrderLine> lines

    CreateOrderCommand(CustomerId customerId, List<OrderLine> lines) {
        this.customerId = customerId
        this.lines = List.copyOf(lines)
    }
}
```

Mappe e `def` vanno bene ai boundary o in script piccoli, non come modello interno.

## Error handling

Usare eccezioni specifiche o result object per errori attesi.

```groovy
final class OrderNotFoundException extends RuntimeException {
    OrderNotFoundException(OrderId orderId) {
        super("Order not found: ${orderId.value}")
    }
}
```

Regole:

- non catturare `Exception` genericamente senza motivo
- non nascondere errori
- evitare `assert` per validazione runtime applicativa
- aggiungere contesto

## Null handling

Groovy rende facile propagare `null` con safe navigation. Non abusarne.

### Da evitare

```groovy
def id = customer?.account?.id
```

se l’assenza dovrebbe essere un errore di dominio.

Validare prima e modellare il caso.

## Resource management

Usare metodi `withCloseable` o `try/finally`.

```groovy
new FileInputStream(path).withCloseable { stream ->
    // ...
}
```

## Gradle/Groovy DSL

Nei build script:

- tenere logica complessa fuori da `build.gradle`
- spostare convenzioni in plugin o script dedicati
- evitare side effect in configuration phase
- usare lazy configuration API dove disponibile
- non leggere environment variabili ovunque

## Logging

Usare logger del framework o di Gradle nei plugin.

```groovy
logger.lifecycle('Generating artifacts')
```

Non usare `println` in plugin o codice applicativo, salvo script locali.

## Test

Spock è ottimo, ma non deve diventare una scusa per test troppo magici.

```groovy
def 'rejects empty order id'() {
    when:
    new OrderId('')

    then:
    thrown(IllegalArgumentException)
}
```

## Cose da evitare

- `def` ovunque in codice applicativo
- mappe dinamiche come dominio
- metaprogramming globale
- monkey patching via metaclass
- `println` in plugin o servizi
- logica complessa dentro `build.gradle`
- safe navigation usata per nascondere errori reali
- `@CompileStatic` evitato senza motivo
