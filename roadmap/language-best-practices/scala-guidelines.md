# Scala Guidelines

## Principi

Scala può produrre codice molto espressivo o molto incomprensibile. La priorità è modellare il dominio con tipi, mantenere gli effetti espliciti e non usare astrazioni avanzate dove il team non le sa mantenere.

## Tipi semantici

Usare opaque type, value class o case class per concetti di dominio.

```scala
opaque type OrderId = String

object OrderId:
  def from(value: String): Either[String, OrderId] =
    if value.nonEmpty then Right(value) else Left("OrderId cannot be empty")
```

Oppure:

```scala
final case class CustomerId(value: String) extends AnyVal
```

## Case class immutabili

Preferire modelli immutabili.

```scala
final case class Order(
  id: OrderId,
  lines: List[OrderLine]
)
```

Evitare `var` salvo casi locali e motivati.

## Modellare stati con sealed trait

```scala
sealed trait PaymentStatus

object PaymentStatus:
  case object Pending extends PaymentStatus
  final case class Completed(transactionId: TransactionId) extends PaymentStatus
  final case class Failed(reason: String) extends PaymentStatus
```

Questo consente pattern matching esaustivo.

## Error handling

Preferire errori come valori per errori attesi.

```scala
def createOrder(command: CreateOrderCommand): Either[CreateOrderError, OrderId]
```

Usare eccezioni per bug, integrazioni Java o errori realmente eccezionali.

Con effect system:

```scala
def createOrder(command: CreateOrderCommand): IO[CreateOrderError, OrderId]
```

a seconda dello stack scelto.

## Effetti espliciti

Non nascondere I/O in funzioni che sembrano pure.

### Da evitare

```scala
def calculateTotal(order: Order): Money = {
  repository.loadDiscount(order.customerId)
  // ...
}
```

### Preferibile

```scala
def calculateTotal(order: Order, discount: Discount): Money
```

Separare calcolo puro da recupero dati.

## Dependency injection

Scegliere un approccio coerente:

- constructor injection semplice per servizi tradizionali
- Reader/ZLayer/tagless-final solo se il team li usa bene
- evitare service locator e global singleton

```scala
final class OrderService(repository: OrderRepository, paymentClient: PaymentClient)
```

## Collections

Usare collection immutabili di default.

```scala
val lines: List[OrderLine] = List.empty
```

Evitare conversioni implicite costose e catene poco leggibili.

## Resource management

Usare costrutti di resource safety dello stack scelto.

Con Cats Effect:

```scala
Resource.make(acquire)(release)
```

Con ZIO:

```scala
ZIO.acquireRelease(acquire)(release)
```

Senza effect system, usare `Using`.

```scala
Using.resource(Source.fromFile(path)) { source =>
  source.getLines().toList
}
```

## Futures

Se si usano `Future`, gestire execution context, errori e composizione.

Regole:

- non usare `Await.result` nella business logic
- non usare global execution context per tutto senza criterio
- non perdere errori in fire-and-forget
- preferire effect system se il progetto lo richiede

## Logging

Usare logging strutturato dove disponibile. Non loggare dati sensibili.

```scala
logger.info(s"Order created: orderId=$orderId customerId=$customerId")
```

Se possibile, preferire campi strutturati rispetto a interpolazione libera.

## Cose da evitare

- implicits/givens troppo magici
- operatori simbolici custom non ovvi
- `var` diffusi
- `null`
- `Option.get`
- `Await.result` nella business logic
- eccezioni per errori di dominio attesi
- astrazioni FP avanzate non comprese dal team
