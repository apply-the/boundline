# Java Guidelines

## Principi

Java dà buoni risultati quando il codice è esplicito, immutabile dove possibile e costruito attorno a dipendenze chiare. Evitare oggetti anemici pieni di setter e servizi che creano internamente le proprie dipendenze.

## Constructor injection

Inizializzare le dipendenze dal costruttore. È la forma più semplice e testabile.

### Da evitare

```java
class OrderService {
    private final PaymentClient paymentClient = new PaymentClient();
}
```

### Preferibile

```java
class OrderService {
    private final PaymentClient paymentClient;
    private final OrderRepository orderRepository;

    OrderService(PaymentClient paymentClient, OrderRepository orderRepository) {
        this.paymentClient = paymentClient;
        this.orderRepository = orderRepository;
    }
}
```

Con Spring, preferire constructor injection senza `@Autowired` sul campo.

## Immutabilità

Preferire campi `final`, record e collezioni immutabili.

```java
public record OrderId(String value) {
    public OrderId {
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("OrderId cannot be blank");
        }
    }
}
```

Non esporre collezioni mutabili interne.

```java
public List<OrderLine> lines() {
    return List.copyOf(lines);
}
```

## Tipi semantici

Evitare `String` e `UUID` ovunque senza semantica.

### Da evitare

```java
Order find(String id) {}
```

### Preferibile

```java
Order find(OrderId id) {}
```

Usare record o value object.

## Gestione errori

Usare eccezioni specifiche e coerenti.

### Preferire

```java
class OrderNotFoundException extends RuntimeException {
    OrderNotFoundException(OrderId orderId) {
        super("Order not found: " + orderId.value());
    }
}
```

Regole:

- non catturare `Exception` genericamente senza motivo
- non ingoiare eccezioni
- non usare eccezioni per controllo di flusso ordinario ad alta frequenza
- aggiungere contesto
- distinguere errori di dominio da errori infrastrutturali

## Optional

Usare `Optional` come return type per assenza esplicita. Non usarlo per campi, parametri o serializzazione DTO salvo motivi forti.

```java
Optional<Order> findById(OrderId id);
```

## RAII equivalente: try-with-resources

Usare `try-with-resources` per risorse `AutoCloseable`.

```java
try (var stream = Files.lines(path)) {
    return stream.count();
}
```

Non affidarsi a cleanup manuale sparso.

## Modellare stati

Usare sealed interface, enum o classi dedicate invece di boolean multipli.

```java
sealed interface PaymentStatus permits Pending, Completed, Failed {}

record Pending() implements PaymentStatus {}
record Completed(TransactionId transactionId) implements PaymentStatus {}
record Failed(String reason) implements PaymentStatus {}
```

## Service design

Un service dovrebbe coordinare casi d’uso, non contenere ogni dettaglio.

### Preferire

- dominio con metodi significativi
- repository per persistenza
- client per sistemi esterni
- mapper ai boundary
- DTO separati dal modello di dominio

## Logging

Usare logging strutturato se supportato dallo stack. Evitare concatenazione stringhe inutile.

```java
logger.info("Order created: orderId={}, customerId={}", orderId, customerId);
```

Regole:

- non loggare segreti
- non loggare e rilanciare a ogni livello
- includere correlation ID tramite MDC o tracing
- non usare `System.out.println` in codice applicativo

## Testabilità

Constructor injection rende i test diretti.

```java
var repository = new FakeOrderRepository();
var paymentClient = new FakePaymentClient();
var service = new OrderService(paymentClient, repository);
```

Preferire fake semplici a mock complessi quando il comportamento è di dominio.

## Cose da evitare

- field injection
- oggetti mutabili senza motivo
- setter pubblici ovunque
- `null` come valore normale
- `catch (Exception e)` generico
- `System.out.println`
- static singleton con stato globale
- dipendenze create dentro i service
- DTO usati come dominio
