# REST Assured Guidelines

## Principi

REST Assured è utile per test API JVM leggibili. Va usato per verificare contratti HTTP, status, payload, headers e auth, non per duplicare test unitari di dominio.

## Struttura

```java
given()
    .contentType(ContentType.JSON)
    .body(validCreateOrderRequest())
.when()
    .post("/orders")
.then()
    .statusCode(201)
    .body("id", notNullValue());
```

## Setup

Centralizzare base URI, auth e config comune senza nascondere troppo.

## Assertions

Verificare:

- status code
- response body
- headers importanti
- error format
- schema se rilevante

## Test data

Creare dati in modo isolato.

- API setup
- repository/test fixture
- container DB
- cleanup

## Schema validation

Usare JSON schema validation per contratti stabili, ma non affidarsi solo allo schema.

## Auth

Testare casi:

- senza token
- token invalido
- permessi insufficienti
- permessi corretti

## Anti-pattern

- test API che dipendono da ordine
- payload copiati enormi e non mantenibili
- assert solo status code
- ambiente condiviso sporco
- sleep per eventual consistency senza polling controllato
