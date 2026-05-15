# Mocking Guidelines

## Principi

Mockare troppo rende i test fragili e falsamente verdi. Mockare troppo poco rende i test lenti e difficili da isolare. La scelta va fatta in base al rischio.

## Test double

### Fake

Implementazione semplice ma funzionante.

Esempio: repository in memoria.

Da preferire quando il comportamento è importante e semplice da simulare.

### Stub

Restituisce dati predefiniti.

Utile per input controllati.

### Mock

Verifica interazioni.

Utile quando l’interazione è il comportamento: invio email, publish evento, chiamata API.

### Spy

Registra chiamate ma può eseguire comportamento reale o parziale.

## Preferire fake per logica di dominio

### Preferibile

```ts
const repository = new InMemoryOrderRepository();
const service = new OrderService(repository);
```

piuttosto che mockare ogni metodo del repository se serve solo salvare e rileggere.

## Mockare boundary esterni

Mock appropriati per:

- API di pagamento
- email provider
- push notification
- servizi terzi
- clock
- random generator
- ID generator

## Non mockare ciò che si vuole testare

Se un test di repository mocka il database, non sta testando il repository.

Se un test di controller mocka completamente routing, serialization e validation, forse è solo un unit test del mapper.

## Evitare test accoppiati a dettagli interni

### Fragile

```java
verify(repository).findById(id);
verify(repository).save(order);
verify(eventPublisher).publish(any());
```

Se il comportamento osservabile è “ordine creato”, testare risultato e side effect significativo, non ogni chiamata interna.

## Mock reset

Ogni test deve partire da stato pulito.

Non condividere mock mutabili fra test senza reset affidabile.

## Anti-pattern

- mock di ogni classe
- mock del domain model
- mock di value object
- test che verificano ordine di chiamate non importante
- mock usati per nascondere design non testabile
- partial mock come default
- fixture di mock incomprensibili
