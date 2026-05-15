# Testing Strategy Guidelines

## Principi

Una strategia di test sana non punta ad avere “tanti test”. Punta ad avere test che intercettano regressioni rilevanti, sono veloci da eseguire, leggibili da mantenere e affidabili.

La domanda corretta non è “abbiamo copertura?”, ma “abbiamo fiducia nel rilascio?”.

## Test pyramid

Indicativamente:

- molti unit test veloci
- diversi integration test sui boundary importanti
- pochi end-to-end test sui flussi critici
- contract test nei sistemi distribuiti
- performance/security test dove il rischio lo richiede

## Tipi di test

### Unit test

Verificano logica piccola e isolata.

Devono essere:

- veloci
- deterministici
- leggibili
- senza rete, database o filesystem reale salvo casi banali
- indipendenti dall’ordine di esecuzione

### Integration test

Verificano integrazione con componenti reali:

- database
- message broker
- filesystem
- framework HTTP
- serialization
- transaction boundary
- API client

Non devono mockare ciò che devono verificare.

### Contract test

Verificano che consumer e provider rispettino un contratto condiviso.

Utili quando:

- più team evolvono API indipendentemente
- microservizi hanno deploy separato
- breaking change costano caro
- l’E2E completo è troppo fragile

### End-to-end test

Verificano flussi utente o business critici attraversando più layer.

Devono essere pochi e di alto valore.

Esempi:

- login
- checkout
- pagamento
- creazione ordine
- onboarding
- flusso amministrativo critico

## Coverage

La code coverage è un indicatore, non un obiettivo assoluto.

### Regole

- coverage alta su logica di dominio
- coverage ragionevole su mapping/error handling
- coverage meno importante su glue code banale
- mutation testing utile dove il rischio è alto
- 100% coverage non significa sistema testato bene

## Determinismo

Un test affidabile non deve dipendere da:

- ordine di esecuzione
- timezone locale
- clock reale
- sleep arbitrari
- rete esterna
- dati condivisi non isolati
- stato globale residuo

## Test naming

Il nome del test deve descrivere comportamento e scenario.

### Debole

```text
testCreateOrder
```

### Migliore

```text
rejects_empty_order_lines
creates_order_when_customer_is_active
returns_conflict_when_order_is_already_paid
```

## Arrange, Act, Assert

Strutturare i test in modo leggibile.

```ts
// Arrange
const order = anOrder().withNoLines().build();

// Act
const result = createOrder(order);

// Assert
expect(result).toEqual({ ok: false, error: "empty_order" });
```

Non serve commentare sempre le tre sezioni, ma la struttura deve essere evidente.

## Flaky test

Un test intermittente è debito critico. Non va ignorato.

Cause comuni:

- sleep
- race condition
- selettori UI fragili
- test data condivisi
- servizi esterni reali
- clock reale
- cleanup incompleto
- retry ciechi

## Anti-pattern

- test solo E2E
- test solo unitari con tutto mockato
- snapshot giganti
- test che duplicano l’implementazione
- test che passano solo localmente
- dipendenza da ambiente manuale
- sleep usati come sincronizzazione
- suite che richiede ordine specifico
