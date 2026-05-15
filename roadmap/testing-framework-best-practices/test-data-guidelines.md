# Test Data Guidelines

## Principi

I dati di test devono essere leggibili, minimi e intenzionali. Dati enormi, fixture opache e factory troppo magiche rendono i test difficili da capire.

## Dati minimi

Creare solo i dati necessari allo scenario.

### Da evitare

```json
{
  "id": "1",
  "name": "John",
  "surname": "Doe",
  "address": "...",
  "phone": "...",
  "metadata": { "...": "..." },
  "preferences": { "...": "..." }
}
```

se il test verifica solo l’email.

### Preferibile

```ts
const user = aUser().withEmail("active@example.com").build();
```

## Test data builder

Usare builder/factory per scenari frequenti.

```ts
const order = anOrder()
  .withCustomer(activeCustomer())
  .withLine(aProductLine().withQuantity(2))
  .build();
```

## Default validi

Le factory dovrebbero produrre oggetti validi per default. Ogni test modifica solo ciò che è rilevante.

```python
order = an_order().with_no_lines().build()
```

## Nomi semantici

Preferire dati con significato.

### Da evitare

```java
var user1 = user();
var user2 = user();
```

### Preferibile

```java
var activeCustomer = activeCustomer();
var suspendedCustomer = suspendedCustomer();
```

## Isolamento

Ogni test deve possedere i propri dati.

Regole:

- niente dipendenza da dati già presenti in ambiente
- cleanup automatico
- transazioni rollback dove possibile
- namespace/test id univoci per test paralleli
- evitare shared mutable fixture

## Date e tempo

Non usare `now` senza controllo.

Preferire clock fittizio.

```java
Clock fixedClock = Clock.fixed(Instant.parse("2026-01-01T00:00:00Z"), ZoneOffset.UTC);
```

## Random

Random utile per property-based testing, ma deve essere riproducibile.

- seed stampato nei failure
- dati random limitati a test specifici
- non usare random per test normali quando un caso fisso è più leggibile

## Snapshot data

Gli snapshot devono essere piccoli e intenzionali.

Evitare snapshot enormi di payload interi, DOM completo o HTML generato.

## Anti-pattern

- fixture globali enormi
- dati magici non spiegati
- factory che nascondono troppa logica
- test che dipendono da dati di altri test
- `sleep` per aspettare dati
- timestamp reali nei test
- random non riproducibile
