# Python Language Guidance

Use Python with explicit types, clear boundaries, and predictable resource management instead of dynamic shortcuts.

- Add type hints to production code and validate external input before it reaches domain logic.
- Prefer dataclasses, value objects, and typed DTOs over ad hoc dictionaries for stable data shapes.
- Inject dependencies through constructors or function parameters instead of importing hidden singletons.
- Use context managers for files, locks, transactions, and client lifetimes so cleanup is deterministic.
- Raise specific exceptions with context; avoid bare `except`, silent fallback, or generic `Exception` flows.
- Keep blocking I/O out of async code paths and keep coroutine boundaries explicit.
- Separate framework models, serializers, and ORM entities from domain decisions.
