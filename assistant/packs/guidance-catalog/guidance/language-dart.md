# Dart

Dart combines null safety, strong typing, and async-first design. Used for Flutter mobile/web and server-side applications (shelf, Dart Frog).

## Null Safety

Sound null safety is non-negotiable. Never disable it. Use nullable types only where absence is semantically meaningful.

```dart
// Good: explicit nullability
String? findUserName(UserId id) => _cache[id]?.name;

// Good: non-null assertion only when provably safe
final user = users.firstWhere((u) => u.id == id);
```

## Domain Modeling

Use sealed classes for sum types and `freezed` for immutable value objects:

```dart
sealed class PaymentStatus {}
final class Pending extends PaymentStatus {}
final class Completed extends PaymentStatus {
  final TransactionId transactionId;
  Completed(this.transactionId);
}
final class Failed extends PaymentStatus {
  final String reason;
  Failed(this.reason);
}
```

Use extension types for lightweight domain newtypes:

```dart
extension type OrderId(String value) {
  OrderId.validated(String raw) : value = raw.isNotEmpty
      ? raw
      : throw ArgumentError('OrderId cannot be empty');
}
```

## Error Handling

Avoid catching generic `Exception` or `Error`. Define domain-specific exceptions or use `Result`-style types (`fpdart`, `dartz`, or custom sealed classes) for expected failures.

```dart
sealed class CreateOrderResult {}
final class OrderCreated extends CreateOrderResult { ... }
final class CustomerNotFound extends CreateOrderResult { ... }
```

## Async

Dart is single-threaded with event-loop concurrency. Use `Future` for one-shot async, `Stream` for event sequences. Never block with `sleep` inside async code.

Always handle errors on Futures. Use `runZonedGuarded` or structured error zones for top-level error capture.

## Dependency Injection

Use constructor injection. For Flutter, prefer `riverpod` or `get_it` for service location at the composition root. Avoid static service access in domain code.

## Immutability

Prefer immutable data. Use `final` fields, `freezed` for data classes, and `const` constructors where applicable. Mutation should be explicit and scoped.

## Recommended Ecosystem Libraries

| Category | Package | Purpose |
|----------|---------|---------|
| Code generation | `freezed` | Immutable data classes, unions, copyWith |
| Serialization | `json_serializable` | JSON (de)serialization code gen |
| State management | `riverpod` or `bloc` | Reactive state with clear lifecycle |
| DI | `get_it` | Service locator at composition root |
| HTTP | `dio` or `http` | HTTP client with interceptors |
| Database | `drift` | Type-safe reactive SQLite |
| Testing | `flutter_test`, `mockito` | Widget and unit testing |
| BLoC testing | `bloc_test` | Specialized BLoC state assertions |
| Linting | `very_good_analysis` or `flutter_lints` | Strict lint rule sets |
| Navigation | `go_router` | Declarative, type-safe routing |
| Functional | `fpdart` | Either, Option, functional utilities |
| Logging | `logger` | Formatted, level-based logging |

## Anti-Patterns

- Disabling null safety or excessive `!` (bang operator)
- Catching generic `Exception` without re-throwing
- Mutable state in global variables
- Business logic inside widgets
- Using `dynamic` at boundaries instead of typed models
- Missing `dispose()` on controllers and streams
- Widget trees with deep nesting instead of extraction
- `late` fields as a substitute for proper initialization

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-hidden-side-effects
- `architecture_boundary`: dependency-direction (business logic in widgets)
- `testability`: untestable-design (when state is coupled to UI)
