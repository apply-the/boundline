# pytest Guidelines

## Principi

pytest è estremamente produttivo grazie a fixture, parametrizzazione e assert naturali. Il rischio è creare fixture troppo magiche e dipendenze implicite difficili da seguire.

## Test naming

```python
def test_rejects_empty_order() -> None:
    ...
```

## Fixture

Usare fixture per setup riusabile, ma mantenerle esplicite.

```python
@pytest.fixture
def order_repository() -> InMemoryOrderRepository:
    return InMemoryOrderRepository()
```

## Fixture scope

Usare scope più largo solo per risorse costose e sicure.

- function: default, più isolato
- module/session: solo per risorse immutabili o gestite bene

## Parametrize

```python
@pytest.mark.parametrize("value", ["", " ", "\t"])
def test_rejects_blank_email(value: str) -> None:
    ...
```

## Monkeypatch

Utile per environment, clock o boundary legacy. Non deve compensare design non testabile.

Preferire dependency injection quando possibile.

## tmp_path

Usare `tmp_path` per filesystem test.

```python
def test_writes_report(tmp_path: Path) -> None:
    report_path = tmp_path / "report.txt"
```

## caplog

Usare `caplog` per verificare log solo quando il log è comportamento rilevante.

## Async

Usare plugin async appropriati. Non mischiare event loop manuali e fixture senza criterio.

## Anti-pattern

- fixture autouse opache
- fixture con side effect nascosti
- monkeypatch ovunque
- test dipendenti da ordine
- sleep
- assert su implementazione interna
