# Go Testing Guidelines

## Principi

Il pacchetto standard `testing` è sufficiente per moltissimi casi. Go premia test semplici, table-driven e fake esplicite.

## Table-driven tests

```go
func TestValidateEmail(t *testing.T) {
    tests := []struct {
        name string
        value string
        wantErr bool
    }{
        {name: "empty", value: "", wantErr: true},
        {name: "valid", value: "user@example.com", wantErr: false},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := ValidateEmail(tt.value)

            if (err != nil) != tt.wantErr {
                t.Fatalf("unexpected error: %v", err)
            }
        })
    }
}
```

## t.Helper

Usare `t.Helper()` negli helper di test.

```go
func mustCreateOrder(t *testing.T) Order {
    t.Helper()
    // ...
}
```

## Fake invece di mock complessi

Go favorisce interfacce piccole e fake manuali.

```go
type fakeOrderRepository struct {
    saved []Order
}
```

## Test package

Usare `package foo` per test interni e `package foo_test` per test del contratto pubblico.

## Context

Usare context con timeout nei test integration.

```go
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
defer cancel()
```

## Race detector

Eseguire periodicamente:

```bash
go test -race ./...
```

## Integration test

Usare build tags o nomi chiari per separare integration test.

## Anti-pattern

- assertion library pesanti senza motivo
- test dipendenti dall’ordine
- sleep invece di sincronizzazione
- fake troppo complessi
- ignorare race detector
- t.Fatal in goroutine senza comunicazione corretta
