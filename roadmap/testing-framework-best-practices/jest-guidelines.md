# Jest Guidelines

## Principi

Jest è adatto a unit test, component test e integration test leggeri in JavaScript/TypeScript. Il rischio principale è abusare di mock globali, snapshot enormi e test accoppiati all’implementazione.

## Struttura dei test

Usare nomi descrittivi.

```ts
describe("calculateInvoiceTotal", () => {
  it("applies the customer discount before VAT", () => {
    // ...
  });
});
```

## TypeScript

Evitare `any` nei test. I test devono proteggere il contratto, non aggirarlo.

## Fake timers

Usare fake timer per codice time-based.

```ts
jest.useFakeTimers();

await act(async () => {
  jest.advanceTimersByTime(1000);
});
```

Ripristinare timer reali quando necessario.

```ts
afterEach(() => {
  jest.useRealTimers();
});
```

## Mock

Mockare boundary esterni, non tutto.

```ts
jest.mock("../payment-client");
```

Regole:

- mock chiari vicino al test
- reset tra test
- evitare mock globali opachi
- preferire fake per repository semplici

## Snapshot

Snapshot piccoli e intenzionali.

### Accettabile

- piccolo JSON stabile
- output serializzato di una funzione pura
- markup limitato di un componente semplice

### Da evitare

- DOM enorme
- payload completo pieno di campi irrilevanti
- snapshot aggiornati senza review

## Async

Aspettare sempre le promise.

```ts
await expect(service.createOrder(command)).resolves.toEqual(expected);
```

Non lasciare promise pendenti.

## Coverage

Usare coverage per trovare buchi, non come unico target.

## Anti-pattern

- snapshot giganti
- `jest.mock` ovunque
- test che passano anche senza assert
- `done` usato quando basta `async/await`
- mock condivisi fra test senza reset
- test di implementazione invece di comportamento
