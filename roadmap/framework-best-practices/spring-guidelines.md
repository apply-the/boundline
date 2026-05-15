# Spring Guidelines

## Principi

Spring è potente, ma può nascondere dipendenze, transazioni e side effect dietro annotazioni. Il codice deve restare leggibile anche senza conoscere magia implicita.

## Constructor injection

Usare constructor injection. Evitare field injection.

### Da evitare

```java
@Service
class OrderService {
    @Autowired
    private OrderRepository repository;
}
```

### Preferibile

```java
@Service
class OrderService {
    private final OrderRepository repository;

    OrderService(OrderRepository repository) {
        this.repository = repository;
    }
}
```

## Controller sottili

Controller:

- mapping HTTP
- validazione boundary
- auth context
- chiamata application service
- mapping response

Non mettere business logic nei controller.

## Service application vs domain

Evitare service enormi.

Separare:

- application service: orchestration
- domain model/service: regole dominio
- repository: persistenza
- integration client: API esterne
- mapper: DTO/domain

## Transaction boundaries

Essere espliciti su `@Transactional`.

Regole:

- transazioni sui casi d’uso, non ovunque
- evitare chiamate remote dentro transazioni DB
- attenzione a self-invocation: proxy Spring non intercetta chiamate interne
- definire read-only quando appropriato
- conoscere propagation e isolation prima di modificarle

## DTO separati

Non esporre entity JPA direttamente dalle API.

### Da evitare

```java
@GetMapping("/{id}")
OrderEntity getOrder(...)
```

### Preferibile

```java
OrderResponse getOrder(...)
```

## JPA

Regole:

- evitare lazy loading involontario nei controller
- evitare N+1 query
- usare fetch join/entity graph/projection dove serve
- non usare entity come modello universale
- attenzione a equals/hashCode su entity
- non aprire sessione fino alla view come scusa per design pigro

## Validation

Usare Bean Validation per input boundary.

```java
record CreateOrderRequest(
    @NotNull UUID customerId,
    @NotEmpty List<OrderLineRequest> lines
) {}
```

Ma le invariant di dominio devono vivere anche nel dominio.

## Error handling

Usare `@ControllerAdvice` per mapping coerente.

```java
@RestControllerAdvice
class ApiExceptionHandler {
    @ExceptionHandler(OrderNotFoundException.class)
    ResponseEntity<ErrorResponse> handle(OrderNotFoundException ex) {
        return ResponseEntity.status(HttpStatus.NOT_FOUND).body(...);
    }
}
```

## Configuration

Usare `@ConfigurationProperties` tipizzate.

```java
@ConfigurationProperties(prefix = "payment")
record PaymentProperties(Duration timeout, URI endpoint) {}
```

Validare config all’avvio.

## Testing

- unit test senza Spring context per dominio e service
- slice test per controller/repository
- integration test con Testcontainers per DB
- evitare `@SpringBootTest` per tutto

## Observability

- structured logging
- correlation ID/MDC
- Micrometer metrics
- tracing
- health checks sensati

## Anti-pattern

- field injection
- controller con logica
- entity JPA esposte in API
- `@Transactional` messo ovunque
- chiamate remote dentro transazioni
- `@SpringBootTest` per test banali
- component scanning caotico
- magic configuration non tipizzata
