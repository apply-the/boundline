# FastAPI and Django Testing Guidelines

## Principi

FastAPI e Django hanno ottimi strumenti di test, ma non bisogna trasformare ogni test in un test HTTP completo. Separare domain/service test, API test e integration DB.

## FastAPI

### TestClient/AsyncClient

Usare client HTTP per testare routing, validation e response mapping.

```python
def test_create_order_returns_201(client: TestClient) -> None:
    response = client.post("/orders", json=valid_order_payload())

    assert response.status_code == 201
```

### Dependency overrides

Usare dependency override per sostituire boundary.

```python
app.dependency_overrides[get_order_service] = lambda: fake_order_service
```

Non abusare per nascondere tutto.

### Async

Usare client async per endpoint async se la suite lo richiede.

## Django/DRF

### APIClient

```python
response = api_client.post("/orders/", data=payload, format="json")
assert response.status_code == 201
```

### Database

Usare marker/fixture DB solo quando serve.

```python
@pytest.mark.django_db
def test_repository_loads_order() -> None:
    ...
```

Non usare DB per test di funzioni pure.

### Factories

Usare factory_boy o factory dedicate con default validi.

## Error mapping

Testare:

- validation error
- not found
- unauthorized
- forbidden
- conflict
- happy path

## Anti-pattern

- test HTTP per ogni funzione
- database usato in ogni test senza bisogno
- dependency override globale non ripulito
- fixture Django enormi
- test che dipendono da settings locali
