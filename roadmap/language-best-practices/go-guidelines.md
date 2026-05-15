# Go Guidelines

## Principi

Go funziona meglio quando il codice è semplice, esplicito e composto da interfacce piccole. Evitare astrazioni premature e framework pesanti.

## Organizzazione

I package devono rappresentare capacità coese, non layer tecnici generici senza significato.

### Preferire

```text
internal/
  order/
  payment/
  customer/
```

### Evitare

```text
internal/
  services/
  managers/
  helpers/
  utils/
```

Un package chiamato `utils` tende a diventare un cestino.

## Error handling

Gli errori non vanno ignorati.

### Da evitare

```go
result, _ := client.Do(req)
```

### Preferibile

```go
result, err := client.Do(req)
if err != nil {
    return fmt.Errorf("send request: %w", err)
}
```

Aggiungere contesto quando si propaga l’errore.

## Errori semantici

Usare errori sentinella o tipi custom quando il chiamante deve distinguere i casi.

```go
var ErrOrderNotFound = errors.New("order not found")
```

```go
if errors.Is(err, ErrOrderNotFound) {
    // handle not found
}
```

Per errori con dati:

```go
type ValidationError struct {
    Field string
    Reason string
}

func (e ValidationError) Error() string {
    return fmt.Sprintf("%s: %s", e.Field, e.Reason)
}
```

## Context

Il `context.Context` deve essere il primo parametro nelle funzioni che fanno I/O, chiamate remote o operazioni cancellabili.

```go
func (s *OrderService) CreateOrder(ctx context.Context, cmd CreateOrderCommand) (OrderID, error) {
    // ...
}
```

Regole:

- non salvare `context.Context` dentro struct
- non passare `nil` context
- rispettare cancellation e deadline
- non usare context come contenitore generico di parametri

## Dependency injection

Iniettare dipendenze nel costruttore.

```go
type OrderService struct {
    repository OrderRepository
    paymentClient PaymentClient
}

func NewOrderService(repository OrderRepository, paymentClient PaymentClient) *OrderService {
    return &OrderService{
        repository: repository,
        paymentClient: paymentClient,
    }
}
```

## Interfacce piccole

Le interfacce appartengono spesso al consumer, non al provider.

```go
type OrderRepository interface {
    Save(ctx context.Context, order Order) error
}
```

Evitare interfacce enormi che rendono i test fragili.

## Tipi semantici

Usare tipi dedicati per identificativi e concetti di dominio.

```go
type OrderID string
type CustomerID string
```

Questo impedisce di confondere parametri con lo stesso tipo sottostante.

## Resource management

Go non ha RAII classico. Usare `defer` appena la risorsa viene acquisita.

```go
file, err := os.Open(path)
if err != nil {
    return fmt.Errorf("open file: %w", err)
}
defer file.Close()
```

Per risorse che possono fallire in `Close`, gestire l’errore dove rilevante.

## Concurrency

Non creare goroutine senza ownership chiara.

### Regole

- ogni goroutine deve avere un modo di terminare
- propagare cancellation via context
- non scrivere su channel chiusi
- chi chiude un channel dovrebbe essere il sender owner
- usare `errgroup` per gruppi di goroutine correlate

```go
group, ctx := errgroup.WithContext(ctx)

group.Go(func() error {
    return worker.Run(ctx)
})

if err := group.Wait(); err != nil {
    return fmt.Errorf("run workers: %w", err)
}
```

## Logging

Usare structured logging.

```go
logger.Info("order created", "order_id", orderID, "customer_id", customerID)
```

Regole:

- non loggare e ritornare lo stesso errore a ogni livello
- includere correlation ID dove disponibile
- non loggare segreti
- evitare `fmt.Println` in servizi applicativi

## Test

Preferire table-driven tests.

```go
func TestValidateOrder(t *testing.T) {
    tests := []struct {
        name string
        order Order
        wantErr bool
    }{
        {name: "empty order", order: Order{}, wantErr: true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := ValidateOrder(tt.order)
            if (err != nil) != tt.wantErr {
                t.Fatalf("unexpected error: %v", err)
            }
        })
    }
}
```

## Cose da evitare

- ignorare errori
- package `utils`
- interfacce troppo grandi
- goroutine senza cancellation
- global mutable state
- `panic` fuori da `main`, init irrecuperabile o test
- configurazione letta ovunque
- context salvato in struct
- log con `fmt.Println`
