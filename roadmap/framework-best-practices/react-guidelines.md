# React Guidelines

## Principi

React deve essere usato per comporre UI da componenti prevedibili. I problemi tipici arrivano da componenti troppo grandi, stato duplicato, `useEffect` abusato, logica di dominio dentro la view e rendering non controllato.

## Componenti piccoli ma sensati

Dividere per responsabilità, non per numero di righe.

### Preferire

- componenti presentazionali
- container/componenti di orchestrazione quando serve
- custom hook per logica riusabile
- servizi/API client fuori dalla UI
- domain logic fuori dai componenti

## Props esplicite

Evitare componenti che accettano oggetti enormi se usano pochi campi.

### Da evitare

```tsx
<UserCard user={user} />
```

se il componente usa solo nome e avatar.

### Preferibile

```tsx
<UserCard displayName={user.displayName} avatarUrl={user.avatarUrl} />
```

## Stato

Tenere lo stato il più vicino possibile a dove serve.

### Regole

- local state per UI locale
- URL state per filtri/shareability
- server state con TanStack Query, SWR o equivalente
- global state solo per stato realmente globale
- non salvare stato derivato

## useEffect

`useEffect` non deve essere il default per calcolare valori o sincronizzare stato derivato.

### Da evitare

```tsx
const [fullName, setFullName] = useState("");

useEffect(() => {
  setFullName(`${firstName} ${lastName}`);
}, [firstName, lastName]);
```

### Preferibile

```tsx
const fullName = `${firstName} ${lastName}`;
```

Usare `useEffect` per sincronizzarsi con sistemi esterni: DOM imperative API, subscription, timer, analytics, network quando non gestita da query library.

## Custom hook

Estrarre hook quando rappresentano una capacità riusabile, non per nascondere codice a caso.

```tsx
function useCurrentUser() {
  return useQuery({
    queryKey: ["current-user"],
    queryFn: fetchCurrentUser,
  });
}
```

## Error e loading states

Ogni fetch deve avere stato di:

- loading
- error
- empty
- success
- refetching quando rilevante

## Performance

Non usare `memo`, `useMemo` e `useCallback` ovunque. Prima capire il problema.

Usarli quando:

- componenti costosi
- referential equality necessaria
- liste grandi
- callback passate a componenti memoizzati

## Key

Usare key stabili.

### Da evitare

```tsx
items.map((item, index) => <Item key={index} item={item} />)
```

### Preferibile

```tsx
items.map((item) => <Item key={item.id} item={item} />)
```

## Forms

Per form complessi, usare librerie dedicate e schema validation.

Regole:

- validare client e server
- non duplicare stato inutilmente
- gestire submit disabilitato/loading
- mostrare errori accessibili

## Accessibilità

Regole minime:

- usare elementi semantici
- label per input
- button per azioni, link per navigazione
- focus visibile
- aria solo quando serve
- testare keyboard navigation

## Testing

Usare React Testing Library o equivalente per comportamento utente.

Preferire query by role, label, text.

## Anti-pattern

- componenti da centinaia di righe con I/O, mapping e UI insieme
- `useEffect` per tutto
- stato derivato salvato
- store globale usato come discarica
- index come key su liste dinamiche
- business logic dentro JSX
- prop drilling estremo non affrontato con composizione/context mirato
- `any` nelle props
