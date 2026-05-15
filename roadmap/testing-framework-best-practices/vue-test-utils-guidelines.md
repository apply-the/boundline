# Vue Test Utils Guidelines

## Principi

Vue Test Utils deve verificare comportamento dei componenti, eventi emessi, props e integrazione con composables/store quando rilevante. Evitare test accoppiati a dettagli interni.

## Mount vs shallowMount

Usare `mount` quando il comportamento dipende dai figli. Usare `shallowMount` quando i figli sono irrilevanti e costosi.

Non usare `shallowMount` come default automatico se nasconde bug di integrazione.

## Props ed emits

Testare contratti esterni.

```ts
const wrapper = mount(UserCard, {
  props: { userId },
});

await wrapper.get("button").trigger("click");

expect(wrapper.emitted("selected")).toEqual([[userId]]);
```

## Testing Library

Per test più user-centric, considerare Vue Testing Library.

Preferire query accessibili.

## Async

Vue aggiorna DOM async.

```ts
await wrapper.get("button").trigger("click");
await nextTick();
```

Per promise:

```ts
await flushPromises();
```

## Store

Con Pinia, creare store isolato per test.

```ts
setActivePinia(createPinia());
```

Non condividere store mutabile tra test.

## Router

Usare router reale per integration component test quando il routing è parte del comportamento. Altrimenti mock semplice.

## Anti-pattern

- assert su dettagli interni non pubblici
- `shallowMount` sempre
- store condiviso tra test
- snapshot enormi
- test che leggono implementation detail invece di DOM/eventi
