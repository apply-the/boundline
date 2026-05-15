# React Native Guidelines

## Principi

React Native combina complessità React, mobile platform e native bridge. Servono confini chiari fra UI, state, native capabilities, networking e domain logic.

## Componenti

Valgono molte regole React:

- componenti piccoli e coesi
- stato locale vicino alla UI
- server state con query library
- global state piccolo
- niente business logic pesante nel JSX
- key stabili

## Platform-specific code

Isolare differenze iOS/Android.

```text
Button.ios.tsx
Button.android.tsx
```

oppure wrapper espliciti.

Evitare `Platform.OS` sparso ovunque.

## Native modules

Incapsulare moduli nativi dietro adapter.

```ts
interface BiometricAuthenticator {
  authenticate(): Promise<AuthResult>;
}
```

La UI non dovrebbe conoscere dettagli nativi.

## Navigation

Tipizzare route e parametri.

Evitare stringhe route sparse.

## Performance

Attenzione a:

- liste grandi
- re-render inutili
- immagini pesanti
- bridge chatter
- animazioni JS thread
- bundle size
- startup time

Usare:

- FlashList/FlatList correttamente
- memoizzazione mirata
- immagini ottimizzate
- native driver/Reanimated dove serve

## Offline e rete mobile

Gestire:

- assenza rete
- retry
- timeout
- optimistic update
- sync conflict
- cache locale
- idempotenza

## Permissions

Centralizzare gestione permessi.

Non chiedere permessi senza contesto UI.

## Security

- non hardcodare secret nell’app
- proteggere token con secure storage
- certificate pinning se richiesto dal rischio
- non loggare dati sensibili
- attenzione a deep link

## Testing

- unit test su domain e hook
- component test su UI
- integration/e2e con Detox o equivalente per flussi critici
- test platform-specific quando necessario

## Anti-pattern

- business logic nei componenti
- `Platform.OS` dappertutto
- route non tipizzate
- token in AsyncStorage senza valutazione
- chiamate native non incapsulate
- FlatList configurate male
- offline state ignorato
