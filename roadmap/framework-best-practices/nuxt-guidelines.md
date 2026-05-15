# Nuxt Guidelines

## Principi

Nuxt aggiunge server rendering, routing, data fetching e convenzioni forti a Vue. La priorità è essere chiari su cosa gira server-side, cosa client-side e come vengono gestiti dati e cache.

## Struttura

Tenere separati:

- pages
- components
- composables
- server routes
- plugins
- middleware
- domain/application services
- API clients

## Data fetching

Usare `useFetch`, `useAsyncData` o composables dedicati in modo coerente.

```ts
const { data, pending, error } = await useAsyncData(
  `order-${orderId}`,
  () => orderApi.getOrder(orderId),
)
```

Regole:

- key stabili
- gestire loading/error/empty
- evitare fetch duplicati
- non fare fetch client-side se può essere server-side
- attenzione ai dati utente e caching

## Server routes

Usare server routes per backend-for-frontend, integrazioni e API interne.

Regole:

- validare input
- controllare auth server-side
- non esporre segreti
- mappare errori applicativi

## Runtime config

Usare runtime config correttamente.

- segreti solo in config privata
- config pubblica solo per valori safe client-side

## Middleware

Usare middleware per cross-cutting concerns come auth/routing guard. Non mettere business logic complessa nei middleware.

## Plugins

I plugin devono inizializzare dipendenze. Evitare side effect pesanti e logica applicativa.

## State

- stato locale nel componente
- Pinia per stato globale client
- server state con fetch/composables
- URL per filtri e paginazione

## SEO e metadata

Gestire metadata, canonical URL e structured data dove rilevante.

## Performance

- sfruttare SSR/SSG dove adatto
- lazy load componenti pesanti
- ottimizzare immagini
- evitare bundle client eccessivo
- attenzione a hydration mismatch

## Anti-pattern

- segreti in runtime config pubblico
- middleware con business logic
- fetch duplicati
- tutto client-side senza motivo
- plugin che fanno troppo
- store globale per cache remota
