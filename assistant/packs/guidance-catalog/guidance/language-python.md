# Python

Python allows speed but requires discipline. Type hints, boundary validation, and explicit dependencies prevent fragile code.

## Type Hints

Use type hints throughout application code. Type hints do not replace runtime validation on external data.

```python
def find_order(order_id: OrderId) -> Order | None:
    ...
```

## Semantic Types

Avoid generic primitives for domain concepts:

```python
@dataclass(frozen=True)
class OrderId:
    value: str

    def __post_init__(self) -> None:
        if not self.value:
            raise ValueError("OrderId cannot be empty")
```

## Immutable Dataclasses

Prefer frozen dataclasses for value objects:

```python
@dataclass(frozen=True)
class Money:
    amount: Decimal
    currency: str
```

## Boundary Validation

Validate input from HTTP, messages, files, and environment variables using Pydantic v2 or explicit constructors. Do not circulate unvalidated dicts through business logic.

## Dependency Injection

Pass dependencies in the constructor. Avoid hidden client creation inside methods:

```python
class OrderService:
    def __init__(self, repository: OrderRepository, payment_client: PaymentClient) -> None:
        self._repository = repository
        self._payment_client = payment_client
```

## Error Handling

Use specific exceptions. Always chain with `raise NewError(...) from exc`:
- Never use bare `except:` or `except Exception: pass`
- Do not hide errors with silent fallbacks
- Do not use exceptions for ordinary high-frequency control flow

## Resource Management

Use context managers for deterministic cleanup:

```python
with open(path) as file:
    content = file.read()
```

## Async

Do not block the event loop. Use `asyncio.TaskGroup` for safe concurrent task management. Use explicit timeouts for I/O. Do not create tasks without observing errors.

## Logging

Use structured logging (`structlog` preferred). Include correlation IDs. Never use `print` in application code. Do not log secrets.

## Testing

Prefer pure functions and simple fakes:

```python
def test_rejects_empty_order() -> None:
    result = create_order(lines=[])
    assert isinstance(result, InvalidOrder)
```

Use readable fixtures. Avoid tests that depend on real sleep.

## Recommended Ecosystem Libraries

| Category | Package | Purpose |
|----------|---------|---------|
| Validation | `pydantic` v2 | Data validation with type inference |
| HTTP client | `httpx` | Async-capable HTTP with connection pooling |
| Logging | `structlog` | Structured, context-rich logging |
| Database | `sqlalchemy` 2.0 | Typed ORM and Core query builder |
| Testing | `pytest` | De facto test framework |
| Property testing | `hypothesis` | Property-based and generative testing |
| Type checking | `mypy` or `pyright` | Static type verification |
| Linting | `ruff` | Fast all-in-one linter and formatter |
| Data modeling | `attrs` or `dataclasses` | Immutable value objects |
| Async | `asyncio` + `anyio` | Structured concurrency primitives |

Prefer packages with PEP 484 type annotations and active maintenance.

## Anti-Patterns

- Untyped dicts flowing through the entire application
- Bare `except:` or `except Exception` with silent fallback
- `print` in services
- Dependencies created inside business logic
- Global mutable variables
- Monkey patching as standard test strategy
- Fire-and-forget async tasks without error handling
- Old-style `%` or `.format()` instead of f-strings
- Mutable default arguments

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: no-primitive-obsession, no-hidden-side-effects
- `testability`: untestable-design (when dependencies are hidden)
- `architecture_boundary`: dependency-direction
