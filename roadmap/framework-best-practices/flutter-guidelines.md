# Flutter Guidelines

## Principi

Flutter permette UI molto veloce da sviluppare, ma senza disciplina si creano widget enormi, stato duplicato e logica applicativa dentro la view. Separare UI, state management, domain e data layer.

## Struttura

Una struttura per feature scala meglio di una puramente tecnica.

```text
features/
  orders/
    presentation/
    application/
    domain/
    data/
```

## Widget piccoli

Dividere widget per responsabilità e leggibilità, non per estetica.

Evitare `build` method enormi.

## Stateless quando possibile

Preferire `StatelessWidget` quando lo stato non è interno.

## State management

Scegliere una soluzione coerente:

- Riverpod
- Bloc/Cubit
- Provider
- ValueNotifier per casi piccoli

Non mischiare troppi paradigmi senza motivo.

## Domain separato

Il domain layer non deve dipendere da Flutter.

Separare:

- widget
- controller/bloc/notifier
- use case
- repository interface
- data source/API
- DTO

## Async e lifecycle

Regole:

- non chiamare `setState` dopo dispose
- cancellare subscription/controller
- gestire loading/error/empty
- evitare chiamate async non controllate in `build`
- usare `initState` o provider appropriati

## Navigation

Centralizzare route importanti e argomenti tipizzati dove possibile. Evitare stringhe magiche sparse.

## Forms

- validazione chiara
- errori visibili
- submit state
- evitare business logic nel widget
- dispose dei controller

## Performance

- usare `const` constructors
- evitare rebuild inutili
- estrarre widget stabili
- usare list builder per liste grandi
- ottimizzare immagini
- profiling prima di micro-ottimizzare

## Testing

- unit test su domain/use case
- widget test su UI behavior
- integration test su flussi critici
- fake repository per test

## Anti-pattern

- logica business nel widget
- `build` enorme
- async call dentro `build`
- stato duplicato
- controller non disposed
- stringhe route magic
- domain che importa Flutter
