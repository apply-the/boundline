# Playwright Guidelines

## Principi

Playwright è adatto a E2E moderni, multi-browser, component testing e test di flussi critici. La sua potenza non elimina la necessità di test piccoli, deterministici e ben isolati.

## Locator

Preferire locator accessibili.

```ts
await page.getByRole("button", { name: "Submit order" }).click();
```

Usare `data-testid` quando il testo o ruolo non è stabile.

## Auto-wait

Sfruttare auto-wait di Playwright. Evitare sleep.

### Da evitare

```ts
await page.waitForTimeout(5000);
```

### Preferibile

```ts
await expect(page.getByText("Order created")).toBeVisible();
```

## Fixtures

Usare fixture per setup comune.

```ts
test("creates an order", async ({ page }) => {
  // ...
});
```

Mantenere fixture leggibili. Fixture troppo magiche rendono i test opachi.

## Authentication

Riutilizzare storage state per evitare login ripetitivo.

```ts
use: {
  storageState: "playwright/.auth/user.json",
}
```

Testare il login solo nei test dedicati al login.

## Test data

Creare dati via API o helper dedicati.

Regole:

- dati univoci
- cleanup automatico
- niente dipendenza da ambiente manuale
- parallel-safe

## API testing

Playwright può testare API via `request`. Utile per setup o per API test leggeri.

Non mischiare troppo API assertions ed E2E UI nello stesso test se rende il flusso confuso.

## Traces, screenshots e video

Abilitare su failure per debugging.

Non generare artefatti pesanti sempre se rallentano troppo la pipeline.

## Parallelismo

I test devono essere indipendenti per supportare parallelismo.

Evitare account condivisi mutati in parallelo.

## Page Object

Usare Page Object con moderazione.

Buono per azioni ripetute e semantiche:

```ts
await checkoutPage.submitOrder();
```

Cattivo se nasconde assert importanti o replica l’intera UI.

## Anti-pattern

- `waitForTimeout`
- selettori CSS fragili
- Page Object giganteschi
- test troppo lunghi con molti assert non correlati
- account condivisi in parallelo
- setup UI ripetitivo
- dipendenza da ordine di esecuzione
