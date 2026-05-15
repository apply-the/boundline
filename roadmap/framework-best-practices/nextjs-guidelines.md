# Next.js Guidelines

## Principi

Next.js è un framework full-stack. La complessità nasce quando non è chiaro cosa gira server-side, cosa gira client-side e dove vivono dati, cache e side effect.

## Server e client components

Usare Server Components di default quando possibile. Passare a Client Components solo per:

- interazione browser
- state locale
- event handler
- browser API
- librerie client-only

### Da evitare

```tsx
"use client";

export default async function Page() {
  const data = await fetchData();
  return <View data={data} />;
}
```

se non serve interattività.

## Boundary chiari

Separare:

- page/layout
- componenti server
- componenti client
- data access
- server actions/API routes
- domain/application services

## Data fetching

Centralizzare accesso dati in funzioni dedicate.

```ts
async function getOrder(orderId: OrderId): Promise<Order> {
  // fetch/db access here
}
```

Non spargere fetch grezzi ovunque.

## Cache

Essere espliciti su cache e revalidation.

Regole:

- capire default caching di `fetch`
- usare `no-store` per dati sempre freschi
- usare revalidation per contenuti cacheable
- invalidare dopo mutation
- non cacheare dati utente sensibili per errore

## Server Actions

Usarle per mutation semplici e coese. Non trasformarle in mega-service.

Regole:

- validare input
- controllare autorizzazione server-side
- gestire errori applicativi
- invalidare cache
- non fidarsi dei dati client

## API routes

Usarle quando serve API pubblica, webhook, integrazioni esterne o client non Next.

## Environment variables

Distinguere:

- variabili server-only
- variabili esposte al client con prefisso previsto

Non mettere segreti in variabili client.

## Routing

Tenere page e layout sottili. Spostare logica in moduli applicativi.

## Performance

- ridurre Client Components
- evitare bundle client inutili
- lazy load componenti pesanti
- ottimizzare immagini
- attenzione a waterfall server-side
- usare streaming/suspense dove utile

## Security

- autorizzazione sempre server-side
- validare input in server action e API route
- proteggere webhook con signature
- non esporre stack trace
- attenzione a caching di dati personali

## Anti-pattern

- `"use client"` in alto a interi alberi senza motivo
- fetch duplicati e cache inconsapevole
- business logic in page
- server action senza auth
- segreti esposti al client
- componenti client enormi
- dipendenza da localStorage per auth server-side
