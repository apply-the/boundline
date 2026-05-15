# Quarkus and Micronaut Guidelines

## Principi

Quarkus e Micronaut sono framework JVM moderni orientati a startup rapido, cloud-native e dependency injection compile-time/build-time. Le regole sane restano: controller sottili, servizi applicativi chiari, config tipizzata, transazioni consapevoli.

## Dependency injection

Usare constructor injection quando possibile.

```java
@ApplicationScoped
public class OrderService {
    private final OrderRepository repository;

    public OrderService(OrderRepository repository) {
        this.repository = repository;
    }
}
```

## Resource/controller sottili

Resource REST:

- mapping HTTP
- validazione
- auth
- chiamata application service
- mapping response

Non business logic.

## Config tipizzata

Usare meccanismi del framework per config tipizzata e validata.

### Quarkus

```java
@ConfigMapping(prefix = "payment")
interface PaymentConfig {
    Duration timeout();
}
```

### Micronaut

```java
@ConfigurationProperties("payment")
public record PaymentConfig(Duration timeout) {}
```

## Native image

Se si compila native:

- evitare reflection non configurata
- controllare librerie compatibili
- testare native binary
- attenzione a init build-time vs runtime
- evitare classpath scanning runtime

## Reactive vs blocking

Non mischiare blocking I/O su event loop.

Regole:

- endpoint reactive solo se stack end-to-end è reactive
- spostare blocking work su worker thread
- timeout espliciti
- backpressure dove rilevante

## Persistence

Con Hibernate/Panache/JPA:

- evitare entity esposte direttamente
- attenzione a N+1
- transazioni sui casi d’uso
- no remote call dentro transazione
- projection per read model

## Error handling

Usare exception mapper globale.

```java
@Provider
public class ApiExceptionMapper implements ExceptionMapper<OrderNotFoundException> {
    // ...
}
```

## Observability

- OpenTelemetry
- Micrometer/metrics
- health checks
- readiness/liveness distinti
- structured logging con correlation ID

## Testing

- unit test senza framework dove possibile
- framework test per resource e injection
- Testcontainers per database e broker
- native tests se si rilascia native

## Anti-pattern

- resource con business logic
- reflection non controllata in native
- blocking su event loop
- entity esposte in API
- config string-based ovunque
- transazioni troppo larghe
