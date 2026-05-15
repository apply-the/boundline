# API Framework Guidelines

## Principi

Un framework web non deve dettare il dominio. Handler, controller e route sono boundary: traducono HTTP in comandi applicativi e risultati applicativi in risposte HTTP.

## Separare HTTP dal dominio

### Da evitare

```java
@Service
public class OrderService {
    public ResponseEntity<?> createOrder(HttpServletRequest request) {
        // domain logic here
    }
}
```

### Preferibile

```java
public class OrderController {
    public ResponseEntity<CreateOrderResponse> create(CreateOrderHttpRequest request) {
        CreateOrderCommand command = mapper.toCommand(request);
        CreateOrderResult result = service.create(command);
        return mapper.toResponse(result);
    }
}
```

## Handler sottili

Un handler dovrebbe occuparsi di:

- autenticazione/autorizzazione già risolta o delegata
- parsing input
- validazione boundary
- chiamata application service
- mapping result to response
- status code
- headers

Non dovrebbe contenere regole di business profonde.

## Validazione

Validare sempre input esterni.

Livelli:

1. validazione sintattica: shape, required fields, tipi
2. validazione semantica: business rules
3. autorizzazione: chi può fare cosa
4. invariant domain: oggetti sempre validi

## Error mapping

Definire mapping chiaro fra errori applicativi e HTTP.

| Errore | Status |
| --- | --- |
| input non valido | 400 |
| non autenticato | 401 |
| non autorizzato | 403 |
| risorsa non trovata | 404 |
| conflitto di stato | 409 |
| rate limit | 429 |
| errore infrastrutturale | 503 o 500 |

Non restituire sempre 500.

## API versioning

Non improvvisare versioning dopo che l’API è pubblica.

Strategie:

- `/v1/...`
- header versioning
- media type versioning

Scegliere una strategia e documentarla.

## Idempotenza

Operazioni critiche devono considerare retry e duplicati.

Esempi:

- pagamenti
- creazione ordini
- webhook
- comandi asincroni
- import batch

Usare idempotency key dove serve.

## Timeout, retry e circuit breaker

Ogni chiamata esterna deve avere:

- timeout
- retry solo se safe
- backoff
- cancellation
- osservabilità
- limiti

## Pagination

Non restituire liste illimitate.

Preferire:

- cursor pagination per dataset grandi
- limit massimo
- ordinamento stabile
- metadata chiari

## Security baseline

- validazione input
- auth esplicita
- rate limiting
- CORS controllato
- CSRF se cookie-based
- output encoding
- secret management
- audit log per operazioni sensibili

## Anti-pattern

- controller con business logic
- service che conosce HTTP
- eccezioni grezze esposte al client
- status code incoerenti
- endpoint non paginati
- retry su operazioni non idempotenti
- logging payload sensibili
