# Kotlin Guidelines

## Principi

Kotlin dà il meglio con null-safety, immutabilità, sealed classes e dipendenze esplicite. Evitare di portarsi dietro abitudini Java come mutabilità diffusa, nullable ovunque e service statici.

## Null-safety

Non usare `!!` salvo casi estremamente locali e giustificati.

### Da evitare

```kotlin
val customerId = request.customerId!!
```

### Preferibile

```kotlin
val customerId = request.customerId
    ?: return CreateOrderResult.InvalidCustomer
```

Oppure validare ai boundary e trasformare in modello non nullable.

## Immutabilità

Preferire `val` a `var` e collection read-only.

```kotlin
data class Order(
    val id: OrderId,
    val lines: List<OrderLine>,
)
```

Per proteggere invarianti, non esporre liste mutabili interne.

## Tipi semantici

Usare value class per wrapper leggeri.

```kotlin
@JvmInline
value class OrderId(val value: String)
```

Per invarianti, usare factory.

```kotlin
@JvmInline
value class EmailAddress private constructor(val value: String) {
    companion object {
        fun parse(value: String): EmailAddress {
            require(value.contains("@")) { "Invalid email address" }
            return EmailAddress(value)
        }
    }
}
```

## Modellare stati con sealed class

```kotlin
sealed interface PaymentStatus {
    data object Pending : PaymentStatus
    data class Completed(val transactionId: TransactionId) : PaymentStatus
    data class Failed(val reason: String) : PaymentStatus
}
```

Questo evita boolean multipli e stati impossibili.

## Dependency injection

Passare dipendenze nel costruttore.

```kotlin
class OrderService(
    private val repository: OrderRepository,
    private val paymentClient: PaymentClient,
)
```

Evitare singleton globali per logica testabile.

## Error handling

Kotlin non forza checked exception. Per errori di dominio attesi, considerare result type o sealed result.

```kotlin
sealed interface CreateOrderResult {
    data class Success(val orderId: OrderId) : CreateOrderResult
    data object InvalidCustomer : CreateOrderResult
    data class Failed(val reason: String) : CreateOrderResult
}
```

Usare eccezioni per errori eccezionali, non per ogni ramo di business.

## Coroutines

Usare structured concurrency.

```kotlin
suspend fun createOrder(command: CreateOrderCommand): OrderId {
    return coroutineScope {
        // child coroutines are bound to this scope
    }
}
```

Regole:

- non usare `GlobalScope` nella business logic
- propagare cancellation
- non bloccare dispatcher coroutine con I/O sincrono
- usare `withContext(Dispatchers.IO)` per I/O blocking inevitabile
- evitare `runBlocking` fuori da main e test

## Resource management

Usare `use` per risorse `Closeable`.

```kotlin
FileInputStream(path).use { stream ->
    // ...
}
```

## Logging

Usare logger strutturato o pattern coerente.

```kotlin
logger.info("Order created: orderId={}, customerId={}", orderId, customerId)
```

Non usare `println` in codice applicativo.

## Test

Constructor injection rende i test diretti.

```kotlin
val service = OrderService(
    repository = FakeOrderRepository(),
    paymentClient = FakePaymentClient(),
)
```

Per coroutine, usare test dispatcher e virtual time.

## Cose da evitare

- `!!`
- `GlobalScope`
- `runBlocking` nella business logic
- mutabilità diffusa
- nullable usati al posto di validazione
- singleton globali con stato
- `println` in servizi
- sealed class sostituite da stringhe magiche
