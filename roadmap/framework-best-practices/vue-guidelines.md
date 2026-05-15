# Vue Guidelines

## Principi

Vue è efficace quando componenti, composables e store hanno responsabilità chiare. Evitare componenti monolitici, watcher usati come colla e store globale per stato locale.

## Composition API

Preferire Composition API nei nuovi progetti.

```vue
<script setup lang="ts">
const props = defineProps<{
  orderId: string
}>()
</script>
```

## Componenti

Separare:

- componenti UI presentazionali
- composables per logica riusabile
- store per stato condiviso
- servizi per API/I/O
- domain logic fuori dai componenti

## Props ed emits tipizzati

```ts
const props = defineProps<{
  userId: UserId
}>()

const emit = defineEmits<{
  selected: [id: UserId]
}>()
```

## Stato

Regole:

- `ref`/`reactive` per stato locale
- `computed` per stato derivato
- Pinia per stato globale reale
- query library per server state quando adatta
- non duplicare stato derivato

## Computed invece di watcher

### Da evitare

```ts
const fullName = ref("")

watch([firstName, lastName], () => {
  fullName.value = `${firstName.value} ${lastName.value}`
})
```

### Preferibile

```ts
const fullName = computed(() => `${firstName.value} ${lastName.value}`)
```

Usare watcher per side effect, non per valori derivati.

## Pinia

Mantenere store piccoli e coesi.

Evitare store unico gigantesco.

```ts
export const useSessionStore = defineStore("session", () => {
  const currentUser = ref<User | null>(null)
  return { currentUser }
})
```

## API access

Non chiamare `fetch` grezzo ovunque nei componenti. Usare servizi o composables.

```ts
const { data, error, isLoading } = useOrder(orderId)
```

## Forms

Usare validazione schema-based per form complessi. Mostrare errori accessibili.

## Performance

- usare `computed` per caching derivato
- evitare watcher profondi non necessari
- usare key stabili
- lazy load route/componenti pesanti
- attenzione a reattività su oggetti enormi

## Testing

Usare Vue Test Utils e testare comportamento. Evitare test accoppiati a dettagli interni.

## Anti-pattern

- watcher per tutto
- store globale per stato locale
- componenti `.vue` enormi
- logica business nel template
- `any` nelle props
- side effect dentro computed
- Pinia store monolitico
