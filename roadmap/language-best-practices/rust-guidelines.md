# Rust Guidelines

## Principi

Rust premia design esplicito, ownership chiara e modellazione precisa degli stati. Usare Rust come se fosse un linguaggio object-oriented classico o uno scripting language porta rapidamente a codice fragile.

## Organizzazione dei moduli

Evitare `mod.rs` nei nuovi progetti. Preferire il layout moderno.

### Da evitare

```text
src/
  domain/
    mod.rs
    order.rs
```

### Preferibile

```text
src/
  domain.rs
  domain/
    order.rs
```

In `domain.rs`:

```rust
pub mod order;
```

Questo rende la struttura più chiara e riduce file con nomi ripetuti.

## Non usare `panic!` fuori dal punto di ingresso

`panic!`, `unwrap()` ed `expect()` non devono essere usati nella logica applicativa normale.

### Accettabile

- prototipi
- test
- `main`
- inizializzazione fallita in modo irrecuperabile
- invariant violation chiaramente documentata

### Da evitare

```rust
fn load_user(id: UserId) -> User {
    repository::find(id).unwrap()
}
```

### Preferibile

```rust
fn load_user(id: UserId) -> Result<User, LoadUserError> {
    repository::find(id).map_err(LoadUserError::Repository)
}
```

## Usare errori semantici

Non propagare errori generici fino ai livelli alti senza significato di dominio.

### Preferibile

```rust
#[derive(Debug, thiserror::Error)]
pub enum CreateOrderError {
    #[error("customer not found")]
    CustomerNotFound,

    #[error("invalid order: {0}")]
    InvalidOrder(String),

    #[error("repository error")]
    Repository(#[from] RepositoryError),
}
```

Usare `anyhow` ai boundary applicativi, CLI o prototipi. Usare errori tipizzati nelle librerie e nella business logic.

## Preferire tipi semantici

Evitare primitive obsession.

### Da evitare

```rust
fn find_order(id: String) {}
```

### Preferibile

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(String);

fn find_order(id: OrderId) {}
```

Le tuple struct sono ottime per newtype leggeri.

## Modellare stati con enum

Usare `enum` per stati alternativi invece di boolean e campi opzionali scollegati.

```rust
pub enum PaymentStatus {
    Pending,
    Completed { transaction_id: TransactionId },
    Failed { reason: String },
}
```

## Rendere gli oggetti validi alla costruzione

Non esporre costruttori che permettono stati invalidi.

```rust
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(value: String) -> Result<Self, EmailAddressError> {
        if value.contains('@') {
            Ok(Self(value))
        } else {
            Err(EmailAddressError::InvalidFormat)
        }
    }
}
```

## RAII e ownership

Usare ownership, borrowing e `Drop` per gestire risorse. Evitare cleanup manuale quando il ciclo di vita può essere legato allo scope.

### Preferibile

```rust
struct LockGuard {
    lock: Lock,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        self.lock.release();
    }
}
```

Nella maggior parte dei casi, usare primitive standard o crate maturi invece di implementare `Drop` manualmente.

## Dependency injection

Rust non ha bisogno di framework DI. Usare trait, generic parameter o trait object.

### Static dispatch

```rust
pub struct OrderService<R> {
    repository: R,
}

impl<R> OrderService<R>
where
    R: OrderRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}
```

### Dynamic dispatch

```rust
pub struct OrderService {
    repository: Box<dyn OrderRepository + Send + Sync>,
}
```

Usare static dispatch di default. Usare dynamic dispatch quando serve eterogeneità o boundary più stabile.

## Async

Non bloccare dentro funzioni async.

### Da evitare

```rust
std::thread::sleep(duration);
```

### Preferibile

```rust
tokio::time::sleep(duration).await;
```

Regole:

- non tenere lock sincroni attraverso `.await`
- preferire `tokio::sync` in contesti async
- propagare cancellation dove possibile
- evitare task detached senza gestione del risultato

## Logging e tracing

Preferire `tracing` rispetto a logging testuale non strutturato.

```rust
#[tracing::instrument(skip(repository))]
pub async fn create_order(
    repository: &dyn OrderRepository,
    command: CreateOrderCommand,
) -> Result<OrderId, CreateOrderError> {
    tracing::info!("creating order");
    // ...
}
```

Non loggare dati sensibili.

## Test

Usare test unitari per logica pura e integration test per boundary.

### Preferire

```rust
#[test]
fn rejects_empty_order() {
    let result = Order::new(vec![]);

    assert!(matches!(result, Err(OrderError::Empty)));
}
```

### Evitare

- test che dipendono dal tempo reale
- `unwrap()` nei test lunghi dove il messaggio di errore sarebbe poco chiaro
- mock complessi quando basterebbe una fake implementation

## Cose da evitare

- `panic!` nella business logic
- `unwrap()` ed `expect()` fuori da test, main o invariant documentate
- `String` per ogni identificativo
- `Arc<Mutex<T>>` come soluzione automatica a ogni problema
- lock tenuti attraverso `.await`
- trait troppo grandi
- `pub` ovunque
- moduli con responsabilità miste
- `clone()` usato per aggirare ownership senza capire il costo
