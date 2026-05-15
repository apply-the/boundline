# Spring Boot Testing Guidelines

## Principi

Spring Boot rende facile avviare il contesto completo. Proprio per questo va fatto solo quando serve. Una suite che usa `@SpringBootTest` per tutto diventa lenta e fragile.

## Livelli

### Unit test

Niente Spring context.

```java
class OrderServiceTest {
    private final OrderRepository repository = new FakeOrderRepository();
    private final OrderService service = new OrderService(repository);
}
```

### Slice test

Usare slice per layer specifici.

- `@WebMvcTest`
- `@DataJpaTest`
- `@JsonTest`
- `@RestClientTest`

### Full integration

Usare `@SpringBootTest` per test che richiedono wiring completo.

## Testcontainers

Per DB e broker, preferire Testcontainers rispetto a H2 quando il comportamento SQL conta.

## MockBean

Usare `@MockBean` con moderazione. Troppi mock nel context rendono il test difficile da capire.

## Transactions

Capire rollback automatico nei test. Non assumere che il comportamento coincida con produzione quando ci sono transaction boundary diversi.

## Controller test

Con `@WebMvcTest`, verificare:

- status
- validation
- serialization
- error mapping
- security se rilevante

## Repository test

Con `@DataJpaTest`, verificare query, mapping e constraint.

## Anti-pattern

- `@SpringBootTest` ovunque
- H2 per query che in produzione girano su PostgreSQL/Oracle/MySQL con differenze reali
- `@MockBean` per mezzo mondo
- test lenti non categorizzati
- context dirty senza motivo
