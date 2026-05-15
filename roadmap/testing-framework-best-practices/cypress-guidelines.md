# Cypress Guidelines

## Principi

Cypress è ottimo per test end-to-end e component test web, ma può diventare fragile se usato per testare tutto. Va usato per flussi ad alto valore, non per sostituire unit e integration test.

## Cosa testare con Cypress

Buoni candidati:

- login
- checkout
- creazione ordine
- flussi admin critici
- form complessi
- regressioni UI reali
- integrazione frontend-backend importante

Cattivi candidati:

- ogni variante di validazione
- ogni funzione di dominio
- casi combinatori enormi
- logica pura

## Selettori stabili

Usare attributi dedicati.

```html
<button data-testid="submit-order">Submit</button>
```

```ts
cy.get('[data-testid="submit-order"]').click();
```

Ancora meglio, quando possibile, usare selettori accessibili se il setup lo supporta bene.

Evitare classi CSS e struttura DOM fragile.

## Login

Non fare login via UI in ogni test se non è ciò che stai testando.

Preferire:

- session reuse
- API login
- seeded auth state
- `cy.session`

## Test data

Creare dati via API, factory o seed controllati.

Non dipendere da dati manuali in ambiente.

## Network

Usare `cy.intercept` con criterio.

- per controllare error state
- per simulare API esterna
- per rendere deterministico un test UI

Non mockare tutto se il test dovrebbe essere E2E reale.

## Waiting

Non usare wait fissi.

### Da evitare

```ts
cy.wait(5000);
```

### Preferibile

```ts
cy.intercept("POST", "/api/orders").as("createOrder");
cy.get('[data-testid="submit-order"]').click();
cy.wait("@createOrder");
```

## Test isolation

Ogni test deve poter girare da solo.

Regole:

- reset stato
- dati univoci
- no dipendenza da test precedente
- cleanup o ambiente disposable

## Component testing

Cypress Component Testing è utile per componenti interattivi. Non sostituisce test unitari su logica.

## Anti-pattern

- test E2E per ogni caso unitario
- `cy.wait(5000)`
- selettori CSS fragili
- test dipendenti da ordine
- ambiente condiviso sporco
- login UI ripetuto ovunque
- snapshot visuali non controllati
