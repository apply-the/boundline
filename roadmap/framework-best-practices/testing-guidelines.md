# Framework Testing Guidelines

## Principi

I test devono dare fiducia senza rendere il refactoring impossibile. Testare tutto tramite il framework è lento e fragile. Testare solo funzioni isolate non basta.

## Test pyramid

Una suite sana contiene:

- molti unit test veloci su logica pura
- integration test su database, message broker, API client e framework boundary
- pochi end-to-end test sui flussi critici

## Frontend

### Testare comportamento

Usare test che interagiscono come un utente.

Preferire:

- testo visibile
- ruolo accessibile
- label
- stato osservabile

Evitare:

- dettagli interni
- snapshot enormi
- classi CSS come selettori principali
- mock di ogni hook

### Cosa testare

- rendering condizionale importante
- form validation
- error state
- loading state
- interazioni utente
- accessibilità base
- integrazione con router/store/query client

## Backend

### Unit test

Testare:

- domain model
- application service
- mapper
- validation
- error mapping

### Integration test

Testare:

- repository contro database reale o container
- controller con framework test client
- serialization/deserialization
- migrations
- transaction boundaries
- security config

## Test data

Usare builder o factory leggibili.

### Da evitare

```ts
const user = { a: "x", b: 1, c: false, d: null };
```

### Preferibile

```ts
const user = aUser()
  .withEmail("admin@example.com")
  .withRole("admin")
  .build();
```

## Test fragili

Evitare:

- sleep
- ordine casuale
- clock reale
- dipendenza da timezone locale
- rete reale non controllata
- snapshot troppo grandi
- assert su messaggi interni non contrattuali

## Contract test

Per sistemi distribuiti, considerare contract test fra provider e consumer.

## Anti-pattern

- solo E2E test
- solo unit test senza integration
- test che richiedono ambiente manuale
- mock del framework invece della propria logica
- test che duplicano l’implementazione
