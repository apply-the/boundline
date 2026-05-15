# Go

Go rewards simplicity, explicit error handling, and narrow interfaces. Keep packages focused, return errors instead of panicking, and make concurrency ownership visible.

## Package Organization

Keep packages small and cohesive. A package should represent one concept. Avoid catch-all `utils` or `common` packages. Prefer consumer-owned interfaces over producer-imposed abstractions.

## Error Handling

Return errors explicitly. Wrap with `fmt.Errorf("context: %w", err)` to preserve chains. Do not use `panic` for business flow. Error strings should be lowercase without trailing punctuation.

```go
func LoadUser(id UserID) (User, error) {
    user, err := repo.Find(id)
    if err != nil {
        return User{}, fmt.Errorf("load user %s: %w", id, err)
    }
    return user, nil
}
```

## Domain Modeling

Use named types for domain concepts:

```go
type OrderID string
type Money struct {
    Amount   int64
    Currency string
}
```

Model states with explicit types rather than boolean flags or string constants.

## Interfaces

Define interfaces at the point of use, not at the point of implementation. Keep interfaces small (1-3 methods).

```go
type OrderRepository interface {
    Find(id OrderID) (Order, error)
    Save(order Order) error
}
```

## Concurrency

Make goroutine ownership and lifecycle explicit. Use `context.Context` for cancellation. Prefer channels for coordination. Use `errgroup` for bounded concurrent work.

Avoid: goroutine leaks, unbounded `go` statements, shared mutable state without synchronization.

## Testing

Use table-driven tests for enumerable cases. Keep test helpers in `_test.go`. Use `testify` or standard assertions with clear failure messages.

```go
func TestRejectsEmptyOrder(t *testing.T) {
    _, err := NewOrder(nil)
    require.ErrorIs(t, err, ErrEmptyOrder)
}
```

## Recommended Ecosystem Libraries

| Category | Package | Purpose |
|----------|---------|---------|
| CLI | `cobra` + `viper` | Subcommand CLI and configuration |
| Logging | `slog` (stdlib) or `zap` | Structured, leveled logging |
| Database | `sqlc` or `ent` | Type-safe SQL / code-generated ORM |
| Testing assertions | `testify` or `go-cmp` | Rich assertions and diffing |
| Integration testing | `testcontainers-go` | Docker-based test dependencies |
| Goroutine leaks | `goleak` | Detect leaked goroutines in tests |
| Linting | `golangci-lint` | Aggregated linter runner |
| Observability | `otel` (OpenTelemetry Go) | Traces, metrics, and propagation |
| HTTP router | `chi` or stdlib `net/http` | Lightweight, composable routing |
| Concurrency | `errgroup` (stdlib) | Structured goroutine coordination |

Prefer stdlib solutions when adequate. Add dependencies only when they reduce complexity.

## Anti-Patterns

- `panic` for recoverable business errors
- Capitalized error strings
- God packages (`utils`, `helpers`, `common`)
- Large interfaces defined by the producer
- Goroutines without lifecycle management
- Ignoring returned errors
- Global mutable state for dependency access
- `init()` with side effects

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-hidden-side-effects
- `architecture_boundary`: dependency-direction
- `testability`: untestable-design
