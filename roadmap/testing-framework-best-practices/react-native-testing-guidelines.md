# React Native Testing Guidelines

## Principi

React Native richiede test su logica React, componenti, integrazioni native e flussi mobile. Non usare solo E2E: sono costosi e fragili.

## Unit test

Testare:

- hook
- domain logic
- formatter
- validators
- state reducers
- API adapters con mock/fake

## Component test

Usare React Native Testing Library.

```tsx
expect(screen.getByText("Submit")).toBeTruthy();
```

Preferire query user-centric.

## Mock native modules

Mockare moduli nativi solo al boundary.

Evitare mock globali enormi non realistici.

## Navigation

Testare parametri e routing critici con wrapper controllato.

## Async

Usare `findBy...` e wait utilities. Non sleep.

## E2E

Usare Detox/Appium per flussi critici:

- login
- onboarding
- checkout
- permission flow
- offline/online sync

## Platform

Testare differenze iOS/Android se il codice diverge.

## Anti-pattern

- snapshot di alberi enormi
- mock nativi opachi
- E2E per ogni scenario
- test che dipendono da animazioni non controllate
- route string non tipizzate
