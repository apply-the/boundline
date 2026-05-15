# Python Service Frameworks

Conventions for Python server frameworks including FastAPI, Django, and Flask.

## Architecture

Separate transport (routes, serializers, middleware) from application logic (services, use cases) from domain (entities, value objects, policies). Framework decorators should not appear in domain code.

## FastAPI

Use Pydantic v2 models for request/response validation. Use `Annotated` dependencies for injection. Keep route handlers thin; delegate to service layer.

```python
@router.post("/orders")
async def create_order(
    command: CreateOrderCommand,
    service: Annotated[OrderService, Depends(get_order_service)],
) -> OrderResponse:
    result = await service.create(command)
    return OrderResponse.from_domain(result)
```

## Django

Use Django REST Framework serializers for boundary validation. Keep business logic in service layer, not in views or models. Use model managers for query encapsulation.

## Dependency Injection

Pass dependencies through function parameters or class constructors. Use framework DI (FastAPI `Depends`, Django signals) only at composition boundaries.

## Database Access

Use repository pattern for complex domain logic. Keep ORM details out of service layer. Use explicit transactions where consistency requires them.

## Anti-Patterns

- Business logic in views or route handlers
- ORM models used directly as API contracts
- Missing Pydantic/serializer validation at request boundary
- Service locator pattern or global service instances
- Mixing sync and async I/O without clear strategy
- Fat models with business logic and persistence mixed

## Guardian Hooks

Guardians that apply to this guidance:
- `architecture_boundary`: dependency-direction, public-contract-stability
- `clean_code`: no-mixed-responsibilities
- `security_boundary`: input validation at request boundary
