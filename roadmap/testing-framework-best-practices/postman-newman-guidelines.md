# Postman and Newman Guidelines

## Principi

Postman è utile per esplorazione, smoke test API e collection condivise. Newman permette esecuzione in CI. Non deve diventare l’unico sistema di test backend.

## Collection design

Organizzare per dominio/API, non per ordine casuale.

```text
Orders
  Create order
  Get order
  Cancel order
Payments
  Authorize payment
```

## Environment

Separare environment:

- local
- dev
- staging
- production read-only se consentito

Non salvare segreti in collection committate.

## Variables

Usare variabili per base URL, token e dati dinamici.

Pulire variabili temporanee dopo l’uso se possono creare dipendenze nascoste.

## Test scripts

Tenere script semplici.

```js
pm.test("returns 201", function () {
  pm.response.to.have.status(201);
});
```

Per logica complessa, meglio test code-based.

## Data dependency

Evitare collection che funzionano solo se eseguite in ordine totale.

Se serve stato, creare risorsa nel setup dello scenario.

## Newman CI

Eseguire collection smoke/regression in pipeline.

Regole:

- timeout chiari
- report JUnit/HTML
- environment controllato
- secret da CI secret store
- failure leggibili

## Contract

Postman può verificare shape base, ma per contract testing serio considerare strumenti dedicati.

## Anti-pattern

- collection come unica fonte di test
- segreti esportati
- dipendenza da ordine globale
- script Postman troppo complessi
- test contro produzione con dati mutanti
