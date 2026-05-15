# Flutter Testing Guidelines

## Principi

Flutter ha tre livelli principali: unit test, widget test e integration test. Usarli correttamente evita di testare tutto con UI end-to-end lente.

## Unit test

Per:

- domain logic
- use case
- validator
- formatter
- repository fake

```dart
test('rejects empty order', () {
  final result = Order.create([]);

  expect(result.isFailure, true);
});
```

## Widget test

Per comportamento UI isolato.

```dart
testWidgets('shows validation error', (tester) async {
  await tester.pumpWidget(MyForm());

  await tester.tap(find.text('Submit'));
  await tester.pump();

  expect(find.text('Email is required'), findsOneWidget);
});
```

## pumpAndSettle

Usare con cautela. Può nascondere animazioni infinite o rallentare test.

Preferire pump mirati quando possibile.

## Integration test

Per flussi critici su app reale.

Non usarli per ogni variante di validazione.

## State management

Testare bloc/notifier/provider separatamente dalla UI quando possibile.

## Golden test

Utili per regressioni visuali di componenti stabili.

Regole:

- baseline controllate
- font/rendering coerenti
- evitare golden troppo grandi
- review attenta dei diff

## Mock/fake

Preferire fake repository e fake API client.

## Anti-pattern

- integration test per tutto
- widget test che dipendono da rete reale
- `pumpAndSettle` ovunque
- golden enormi e fragili
- business logic non testata fuori dai widget
