# Python Guidelines

## Principi

Python permette di andare veloci, ma proprio per questo serve disciplina. Type hints, validazione ai boundary e dipendenze esplicite evitano codice fragile.

## Type hints

Usare type hints in tutto il codice applicativo.

```python
def find_order(order_id: OrderId) -> Order | None:
    ...
```

I type hints non sostituiscono la validazione runtime su dati esterni.

## Tipi semantici

Evitare primitive generiche per concetti di dominio.

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class OrderId:
    value: str

    def __post_init__(self) -> None:
        if not self.value:
            raise ValueError("OrderId cannot be empty")
```

## Dataclass immutabili

Preferire dataclass frozen per value object.

```python
@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str
```

## Validazione ai boundary

Validare input da HTTP, messaggi, file e variabili ambiente usando strumenti dedicati o costruttori espliciti.

```python
@dataclass(frozen=True)
class CreateOrderCommand:
    customer_id: CustomerId
    lines: list[OrderLine]
```

Non far circolare dizionari non validati nella business logic.

## Dependency injection

Passare dipendenze nel costruttore.

```python
class OrderService:
    def __init__(
        self,
        repository: OrderRepository,
        payment_client: PaymentClient,
    ) -> None:
        self._repository = repository
        self._payment_client = payment_client
```

Evitare creazione nascosta di client dentro i metodi.

## Error handling

Usare eccezioni specifiche.

```python
class OrderNotFoundError(Exception):
    pass
```

Regole:

- non catturare `Exception` genericamente senza rilanciare o gestire
- non usare `except:`
- non nascondere errori con fallback silenziosi
- aggiungere contesto quando serve
- non usare eccezioni per controllo di flusso ordinario ad alta frequenza

## Resource management

Python supporta cleanup deterministico tramite context manager.

```python
with open(path) as file:
    content = file.read()
```

Per risorse custom:

```python
from contextlib import contextmanager
from collections.abc import Iterator

@contextmanager
def managed_connection() -> Iterator[Connection]:
    connection = create_connection()
    try:
        yield connection
    finally:
        connection.close()
```

## Async

Non bloccare l’event loop.

### Da evitare

```python
time.sleep(1)
```

### Preferibile

```python
await asyncio.sleep(1)
```

Regole:

- non usare librerie sincrone dentro path async critici
- gestire cancellation
- non creare task senza osservare errori
- usare timeout espliciti per I/O

## Logging

Usare logging strutturato dove possibile.

```python
logger.info("Order created", extra={"order_id": order_id.value})
```

Regole:

- non usare `print` in codice applicativo
- non loggare segreti
- includere correlation ID
- non loggare lo stesso errore a ogni livello

## Test

Preferire funzioni pure e fake semplici.

```python
def test_rejects_empty_order() -> None:
    result = create_order(lines=[])

    assert isinstance(result, InvalidOrder)
```

Usare fixture leggibili. Evitare test che dipendono da sleep reali.

## Cose da evitare

- dizionari non tipizzati che attraversano tutta l’applicazione
- `except:`
- `except Exception` con fallback silenzioso
- `print` in servizi
- dipendenze create dentro business logic
- variabili globali mutabili
- monkey patching come strategia standard di test
- task async fire-and-forget senza gestione errori
