# Svelte Guidelines

## Principi

Svelte riduce boilerplate, ma la semplicità può portare a componenti troppo carichi. Separare UI, store, API client e logica di dominio resta fondamentale.

## Componenti

Tenere componenti focalizzati.

Separare:

- componenti UI
- store
- moduli di dominio
- API client
- utility pure

## Reactive statements

Usare `$:` per derivazioni semplici, non per side effect complessi e difficili da tracciare.

### Preferibile

```svelte
$: fullName = `${firstName} ${lastName}`;
```

### Da evitare

```svelte
$: {
  saveUser(user);
  sendAnalytics(user);
  updateGlobalState(user);
}
```

## Store

Usare store per stato condiviso reale.

Regole:

- store piccoli e coesi
- evitare store globale unico
- non mettere server cache manuale se esiste soluzione migliore
- non mutare stato in modo opaco

## SvelteKit

Se si usa SvelteKit:

- usare `load` per data loading di pagina
- validare server-side actions
- distinguere codice server-only e client
- non esporre segreti al client
- gestire errori con meccanismi del framework

## Forms

Per form semplici, progressive enhancement. Per form complessi, validazione schema-based.

## Performance

Svelte è efficiente, ma non immune:

- key stabili nelle liste
- evitare store enormi che invalidano troppo
- lazy load moduli pesanti
- attenzione a derived store costosi

## Testing

Usare testing-library per comportamento utente e test unitari per logica pura.

## Anti-pattern

- reactive block con side effect multipli
- store globale monolitico
- business logic nei componenti
- server/client boundary confuso
- mutazioni invisibili di oggetti condivisi
