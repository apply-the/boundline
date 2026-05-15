# Testing Python

Testing conventions for Python projects using pytest and related tools.

## Test Organization

- Unit tests: mirror source structure in `tests/unit/`
- Integration tests: `tests/integration/` with explicit markers
- Use `conftest.py` for shared fixtures, scoped appropriately

## Fixtures

Use pytest fixtures for setup and teardown. Prefer factory fixtures over large shared state. Keep fixture scope as narrow as possible.

```python
@pytest.fixture
def order_service(mock_repository: OrderRepository) -> OrderService:
    return OrderService(repository=mock_repository)
```

## Parametrize

Use `@pytest.mark.parametrize` for testing multiple inputs against the same logic. Keep parameter sets readable.

## Async Testing

Use `pytest-asyncio` for async test functions. Mark async tests explicitly. Handle both success and error paths.

## Mocking

Mock at boundaries. Use `unittest.mock.patch` sparingly; prefer dependency injection. Mock external services, not internal logic.

## Type Checking In Tests

Use type annotations in test code. Run mypy/pyright on test files. Use typed test factories.

## Anti-Patterns

- Broad fixtures shared across unrelated tests
- Patching internal functions instead of injecting dependencies
- Tests that require specific execution order
- Missing async/await in test assertions
- Fixtures with `session` scope that accumulate state
- Tests that import and test private functions

## Guardian Hooks

Guardians that apply to this guidance:
- `testability`: untestable-design, test-isolation
- `clean_code`: test readability and maintenance
