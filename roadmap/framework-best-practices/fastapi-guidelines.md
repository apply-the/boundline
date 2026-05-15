# FastAPI Guidelines

## Principi

FastAPI dà ottima ergonomia per API tipizzate, ma bisogna evitare di mettere business logic nelle route e di far diventare Pydantic model, ORM model e dominio la stessa cosa.

## Route sottili

Route:

- input model
- dependency injection boundary
- auth
- chiamata service
- mapping response

```python
@router.post("/orders", response_model=CreateOrderResponse)
async def create_order(
    request: CreateOrderRequest,
    service: OrderService = Depends(get_order_service),
) -> CreateOrderResponse:
    result = await service.create_order(request.to_command())
    return CreateOrderResponse.from_result(result)
```

## Pydantic model non è dominio

Separare:

- request schema
- response schema
- command
- domain model
- ORM model

Pydantic è ottimo al boundary, ma il dominio non deve dipendere sempre dal framework.

## Dependency injection

Usare `Depends` per boundary e wiring, ma evitare dipendenze nascostissime.

Regole:

- provider piccoli
- niente business logic nei dependency provider
- config validata
- session lifecycle chiaro

## Async

Non usare I/O blocking in endpoint async.

### Da evitare

```python
time.sleep(1)
requests.get(url)
```

### Preferibile

```python
await asyncio.sleep(1)
await http_client.get(url)
```

Oppure usare endpoint sync se il lavoro è blocking e controllato.

## Database session

Gestire session lifecycle con dependency.

```python
async def get_session() -> AsyncIterator[AsyncSession]:
    async with async_sessionmaker() as session:
        yield session
```

Regole:

- transazioni esplicite nei casi d’uso
- evitare session globale
- evitare query nelle route complesse
- attenzione a lazy loading fuori sessione

## Error handling

Usare exception handler per mapping coerente.

```python
@app.exception_handler(OrderNotFoundError)
async def order_not_found_handler(request, exc):
    return JSONResponse(status_code=404, content={...})
```

## Validation

Pydantic valida shape e tipi. Le regole di dominio devono stare nel dominio.

## Security

Usare dependency per auth, ma controllare autorizzazione nel caso d’uso.

## Testing

- TestClient/AsyncClient per API
- unit test su service senza FastAPI
- integration test su DB reale/container
- dependency overrides per test

## Anti-pattern

- business logic nelle route
- Pydantic model come dominio universale
- session globale
- blocking I/O in async endpoint
- dependency provider con troppa logica
- eccezioni grezze esposte
