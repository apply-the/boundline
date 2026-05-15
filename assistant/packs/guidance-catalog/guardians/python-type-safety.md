# Python Type Safety Guardian

Enforce type annotations at boundaries and prevent untyped code from propagating through the system.

## Rules

### missing-type-hints
Public functions, class methods, and module-level functions must have type annotations on parameters and return values. Internal helpers may omit them only if the intent is obvious from context.

Triggers: public functions without parameter or return annotations, library code with `-> None` omitted on side-effecting functions, APIs that accept or return `Any` without justification.

### untyped-boundary-function
Functions at system boundaries (HTTP handlers, CLI entry points, queue consumers, scheduled jobs) must have fully typed signatures. These are the points where external data enters and type safety begins.

Triggers: Flask/FastAPI/Django view functions without type annotations, CLI handlers accepting untyped arguments, background job processors with `*args, **kwargs`.

### bare-except
Bare `except:` or `except Exception` without re-raising catches and hides all errors including system exits and keyboard interrupts. Use specific exception types.

Triggers: `except:` without a type, `except Exception` with `pass` or `continue`, error handlers that swallow exceptions without logging.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to Python code only. Language-specific guardian. Complements static type checkers (mypy, pyright).
