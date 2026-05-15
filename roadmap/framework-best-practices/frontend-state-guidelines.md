# Frontend State Management Guidelines

## Principi

La gestione dello stato frontend deve essere intenzionale. Molti problemi di frontend nascono da stato duplicato, derivato salvato come sorgente di verità, sincronizzazione manuale e componenti che mischiano UI, dominio e I/O.

## Classificare lo stato

Prima di scegliere uno store, capire che tipo di stato si sta gestendo.

### Stato locale

Esempi:

- input corrente di un form
- apertura di una modale
- tab selezionato
- hover/focus
- stato temporaneo di una UI

Deve stare vicino al componente.

### Stato server

Esempi:

- profilo utente caricato da API
- lista ordini
- feature flags remote
- permessi
- dati paginati

Non è “state management generico”: è cache remota. Usare strumenti dedicati come TanStack Query, SWR, Apollo, RTK Query o equivalenti.

### Stato globale client

Esempi:

- sessione autenticata
- tema
- lingua
- tenant selezionato
- feature temporanee condivise da molte viste

Tenere piccolo. Uno store globale enorme diventa presto un database parallelo.

### Stato derivato

Non salvarlo se può essere calcolato.

#### Da evitare

```ts
const [items, setItems] = useState<Item[]>([]);
const [itemCount, setItemCount] = useState(0);
```

#### Preferibile

```ts
const itemCount = items.length;
```

## Single source of truth

Ogni informazione dovrebbe avere una sorgente autorevole.

### Da evitare

- lo stesso utente in store globale, local state e localStorage
- permessi duplicati in più slice
- filtri URL e filtri nello store divergenti
- form state duplicato tra componenti e store

## URL come stato

Se uno stato serve per condividere, ricaricare o tornare indietro, spesso deve stare nell’URL.

Esempi:

- query di ricerca
- filtri
- paginazione
- tab principale
- sort

## Server state

Per dati remoti, preferire librerie che gestiscono:

- caching
- deduplica richieste
- retry
- invalidazione
- refetch
- loading/error states
- optimistic updates

Non reinventare tutto con `useEffect + useState` ovunque.

## Form state

I form complessi meritano gestione dedicata.

Regole:

- validare ai boundary
- tenere separati valori, errori e stato di submit
- evitare validazione solo lato client
- sincronizzare con schema condiviso quando possibile
- non salvare form temporanei nello store globale salvo reale necessità

## Anti-pattern

- store globale usato per tutto
- server state copiato manualmente in store client
- stato derivato salvato
- `useEffect` usato per sincronizzare stati che potrebbero essere calcolati
- localStorage letto/scritto ovunque
- stato URL duplicato in memoria
- mega-store con slice non correlate
