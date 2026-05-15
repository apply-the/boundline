# Vitest Guidelines

## Principi

Vitest è ottimo per progetti Vite, librerie frontend e TypeScript moderno. Le regole sono simili a Jest, con attenzione a ESM, ambiente browser simulato e velocità.

## Ambiente

Scegliere ambiente per test:

- `node` per logica pura
- `jsdom` o `happy-dom` per componenti DOM
- browser mode solo quando serve davvero

Non usare DOM environment per test che non ne hanno bisogno.

## Struttura

```ts
import { describe, expect, it, vi } from "vitest";

describe("parseUserProfile", () => {
  it("rejects missing email", () => {
    // ...
  });
});
```

## Mock

Usare `vi.mock` con moderazione.

```ts
vi.mock("../payment-client", () => ({
  createPaymentClient: () => fakePaymentClient,
}));
```

Regole:

- reset/restore tra test
- evitare mock globali impliciti
- preferire dependency injection quando possibile

## Fake timers

```ts
vi.useFakeTimers();
vi.advanceTimersByTime(1000);
vi.useRealTimers();
```

## Testing Library

Per componenti React/Vue/Svelte, usare testing-library o tool specifici. Testare comportamento, non implementazione.

## ESM

Attenzione a mocking ESM e ordine degli import. Se il mock diventa contorto, è spesso segnale che il modulo crea dipendenze globali invece di riceverle.

## Performance

Vitest è veloce, ma test lenti restano test lenti.

- evitare setup globale pesante
- separare integration test
- non avviare server per ogni test unitario
- usare pool/concurrency con attenzione se c’è stato condiviso

## Anti-pattern

- usare jsdom per tutto
- mock ESM complicati per aggirare design
- snapshot enormi
- test con stato globale non isolato
- integration test infilati nella suite unit senza distinzione
