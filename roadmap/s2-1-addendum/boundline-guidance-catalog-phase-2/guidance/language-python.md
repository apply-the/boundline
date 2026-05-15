# Python Guidance

## Purpose

This guidance defines idiomatic Python practices for AI-assisted planning, implementation, testing, review, and refactoring.

It is intended for Python services, CLIs, data pipelines, automation tools, and APIs.

## Authority Classification

Default strength: recommended  
Specific rules may become mandatory by workspace policy, expert pack configuration, or Canon-governed standards.

## Version Posture

Active support window:
- Python 3.9+

Target excellence:
- Python 3.11+ where repository constraints allow
- Python 3.12+ when performance and ecosystem support are acceptable
- modern type hints
- `pathlib`
- structured concurrency where applicable

Legacy warnings:
- Python 3.8 or older
- old-style typing syntax when modern syntax is available
- `os.path` in new code where `pathlib` is clearer
- untyped public APIs
- silent exception handling

## Core Design Principles

### Explicit Runtime Boundaries

Python is dynamic. Runtime validation matters.

Validate:
- API input
- CLI input
- environment variables
- external JSON
- database records
- AI-produced structured data
- message queue payloads

Recommended:
- Pydantic v2 for structured validation
- dataclasses or attrs for internal immutable structures where validation needs are simple

### Type Hints Are Design Signals

Type hints should communicate intent.

Use:
- precise return types
- `Protocol` for structural abstractions
- `TypedDict` or Pydantic models for structured dictionaries
- `Literal` or enums for constrained states
- `Self` where useful

Avoid:
- `Any` in domain logic
- untyped public functions
- dictionaries as anonymous domain models
- broad `dict[str, Any]` without boundary validation

### Exception Ownership

Never swallow exceptions silently.

Bad:
```python
try:
    do_work()
except Exception:
    pass
```

When re-raising, preserve cause:

```python
raise TokenRotationError(user_id) from exc
```

Use `add_note()` when additional context improves debugging without changing exception type.

### Error Boundaries

Expected business failures should be modeled deliberately.

Options:
- typed exceptions
- result objects
- explicit validation errors
- domain-specific error classes

Do not let low-level infrastructure exceptions become public API semantics.

### Async Guidance

Use async intentionally.

Avoid:
- blocking calls inside async functions
- unbounded `gather`
- ignored cancellations
- event-loop global side effects

Prefer:
- `asyncio.TaskGroup` where suitable
- timeouts
- cancellation-aware code
- dependency injection for I/O clients

### Configuration

Configuration must be validated at startup.

Avoid:
- reading environment variables throughout domain code
- untyped configuration dictionaries
- defaulting missing secrets silently

Prefer:
- validated settings object
- single configuration boundary
- explicit startup failure for invalid required config

### Logging

Use structured logging for service code.

Recommended:
- `structlog`
- standard logging with structured adapters if dependency footprint matters

Logs should include:
- correlation ID
- operation
- stable entity IDs
- error cause

Avoid:
- print debugging in production paths
- logging secrets
- duplicate logs at every layer

### File And Path Handling

Use `pathlib` in new code.

Avoid stringly path manipulation.

### API Frameworks

FastAPI and Django Ninja are good API boundaries when the repo already uses them.

Keep:
- request parsing and validation at the edge
- domain logic framework-independent
- persistence isolated from route handlers

Avoid:
- route handlers containing business policy
- Pydantic models leaking into every domain layer when they are transport schemas
- uncontrolled dependency injection magic

## Testing Guidance

Use `pytest`.

Prefer:
- fixtures for reusable setup
- parameterized tests
- builders for complex domain data
- temporary directories via pytest fixtures
- explicit monkeypatching when needed

Avoid:
- `unittest` style classes in new pytest suites
- hidden fixture coupling
- global mutable test state
- sleeps for timing
- testing implementation details over behavior

## Packaging And Tooling

Recommended:
- `uv` for fast dependency and environment management where accepted
- Poetry where already established
- Ruff for linting/formatting
- mypy or pyright for static typing, depending on team standards

## Anti-Patterns

- `except Exception: pass`
- missing `raise ... from exc`
- `Any` as escape hatch
- unvalidated external JSON
- domain logic inside route handlers
- stringly typed dictionaries everywhere
- global configuration access
- blocking I/O inside async functions
- tests that depend on execution order
- legacy string formatting in new code where f-strings are clearer

## Guardian Hooks

Recommended guardians:
- python-exception-ownership-guardian
- python-boundary-validation-guardian
- python-typing-guardian
- python-async-safety-guardian
- python-config-boundary-guardian
- python-test-fixture-guardian

## Structured Finding Example

```json
{
  "guardian": "python-exception-ownership",
  "rule": "swallowed-exception",
  "disposition": "blocker",
  "summary": "The exception handler suppresses all errors without recording or propagating failure.",
  "evidence_refs": ["src/payments/retry.py"],
  "recommended_action": "Catch a specific exception and re-raise with domain context or return an explicit failure result."
}
```

## Lifecycle Usage

Planning:
- identify validation, typing, and async concerns

Implementation:
- guide boundary validation, exception handling, and configuration design

Testing:
- shape pytest fixtures, builders, and deterministic tests

Review:
- check exception ownership, untyped boundaries, and framework leakage

Refactoring:
- migrate untyped dictionaries into explicit models without changing behavior
