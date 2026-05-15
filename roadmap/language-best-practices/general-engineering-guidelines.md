# General Engineering Guidelines

## Principi

Scrivere codice semplice non significa scrivere codice banale. Significa rendere evidente il flusso delle dipendenze, degli errori, delle risorse e delle decisioni di dominio.

Le linee guida valgono trasversalmente ai linguaggi.

## Modellare il dominio con tipi espliciti

Evitare di passare primitive generiche quando rappresentano concetti di dominio diversi.

### Da evitare

```ts
function transfer(from: string, to: string, amount: number) {}
```

### Preferibile

```ts
type AccountId = string;
type MoneyAmount = number;

function transfer(from: AccountId, to: AccountId, amount: MoneyAmount) {}
```

Nei linguaggi con type system più ricco, usare newtype, tuple struct, record, value object o classi immutabili.

## Rendere impossibili gli stati non validi

Non rappresentare stati mutualmente esclusivi con boolean sparsi.

### Da evitare

```java
class Payment {
    boolean pending;
    boolean completed;
    boolean failed;
}
```

### Preferibile

```java
sealed interface PaymentStatus {
    record Pending() implements PaymentStatus {}
    record Completed(String transactionId) implements PaymentStatus {}
    record Failed(String reason) implements PaymentStatus {}
}
```

## Dipendenze esplicite

Le dipendenze devono essere passate dall’esterno, idealmente da constructor, factory o composition root.

### Da evitare

```java
class OrderService {
    private final PaymentClient paymentClient = new PaymentClient();
}
```

### Preferibile

```java
class OrderService {
    private final PaymentClient paymentClient;

    OrderService(PaymentClient paymentClient) {
        this.paymentClient = paymentClient;
    }
}
```

Questo migliora testabilità, configurabilità e separazione delle responsabilità.

## Separare logica pura e side effect

La logica di dominio dovrebbe essere testabile senza database, rete, filesystem, clock reale o variabili d’ambiente.

### Preferibile

- funzioni pure per calcolo e validazione
- adapter separati per database, HTTP, Pub/Sub, file system
- dependency injection per clock, ID generator, client esterni

## Gestione errori

Gli errori attesi devono essere modellati come valori o eccezioni domain-specific, a seconda del linguaggio.

### Regole

- Non usare errori generici dove serve semantica.
- Non loggare e rilanciare lo stesso errore a ogni livello.
- Non nascondere errori con fallback silenziosi.
- Aggiungere contesto prima di propagare un errore.
- Distinguere errori utente, errori di sistema e bug.

## RAII e cleanup deterministico

Dove possibile, preferire meccanismi che associano il ciclo di vita della risorsa allo scope:

- Rust: `Drop`
- C++: RAII
- C#: `using` / `IDisposable`
- Java: `try-with-resources`
- Python: context manager
- Go: `defer`, anche se non è RAII classico

### Obiettivo

Le risorse devono essere rilasciate anche in caso di errore.

## Logging e osservabilità

Il logging deve aiutare a capire cosa è successo senza leggere il codice.

### Regole

- Usare structured logging.
- Includere correlation ID, request ID o trace ID.
- Non loggare segreti, token, password, dati personali non necessari.
- Loggare ai boundary: ingresso richiesta, chiamata esterna, errore rilevante.
- Evitare log rumorosi in loop o hot path.
- Non usare log come sostituto di metriche e tracing.

## Testabilità

Il codice testabile nasce dal design.

### Preferire

- dipendenze iniettate
- interfacce piccole
- funzioni pure
- clock fittizio nei test
- test table-driven dove adatti
- fixture leggibili
- test che verificano comportamento, non dettagli interni

### Evitare

- singleton globali
- service locator
- static method con side effect
- inizializzazione nascosta
- sleep nei test
- test dipendenti dall’ordine di esecuzione

## Configurazione

La configurazione deve essere letta ai boundary dell’applicazione, validata una volta e poi passata come oggetto tipizzato.

### Da evitare

```go
func Handle() {
    timeout := os.Getenv("TIMEOUT")
}
```

### Preferibile

```go
type Config struct {
    Timeout time.Duration
}

func NewHandler(config Config) Handler {
    return Handler{config: config}
}
```

## Cose da evitare

- Primitive obsession.
- Global mutable state.
- Dipendenze create dentro la business logic.
- Errori ignorati.
- Panics, crash o exit fuori dal punto di ingresso.
- Parsing configurazione sparso nel codice.
- DTO usati come modello di dominio senza validazione.
- Log non strutturati.
- Test fragili basati su timing reale.
