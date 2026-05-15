# Clean Code

Language-agnostic clean code principles that shape planning, implementation, refactoring, testing, and review.

## Core Principles

### Intent-Revealing Names

Names must express the reason a thing exists, not merely its implementation mechanism.

Prefer:

```text
TokenRotationPolicy
InvoiceApprovalWindow
CustomerEligibilityRule
```

Avoid:

```text
Manager
Processor
Handler
Util
Data
Thing
```

A name is weak when a reviewer must inspect implementation to understand business intent.

### Small Cohesive Units

Functions, classes, modules, and services should have one coherent reason to change. This does not mean every file must be tiny. It means responsibilities must not be mixed.

Common violations:
- Persistence and business policy in the same object
- Validation and side effects mixed in the same function
- HTTP transport details leaking into domain logic
- Orchestration logic hidden inside low-level helpers
- UI state, API shape, and domain decisions coupled together

### Explicit Boundaries

Validate at system boundaries: HTTP request input, CLI arguments, message bus payloads, database read models, third-party API responses, AI-generated structured data.

Internal code should not repeatedly defend against invalid states that should have been rejected at the boundary.

### No Magic Values In Domain Logic

Avoid unexplained numbers, strings, flags, and enum-like literals.

Bad:

```text
if retry_count > 3
if status == "A"
if amount > 9999
```

Better:

```text
if retry_count > MAX_AUTH_RETRY_ATTEMPTS
if status == AccountStatus.Active
if amount > HIGH_VALUE_TRANSFER_THRESHOLD
```

### Error Ownership

Errors must be owned at the correct layer:
- Domain errors should be typed and meaningful.
- Infrastructure errors should not leak as business semantics.
- Public APIs should expose stable errors.
- Internal errors should preserve causal chain.
- Do not both log and return the same error unless there is a clear ownership boundary.

### Primitive Obsession

Avoid raw primitives for domain concepts.

Bad:

```text
user_id: string
amount: number
country: string
```

Better:

```text
UserId
Money
CountryCode
```

This is especially important for identifiers, quantities, permissions, time windows, and ownership boundaries.

### Side Effects Must Be Visible

A function that mutates state, sends messages, writes files, calls external systems, or changes persistent data must make that behavior clear through naming and placement.

Hidden side effects are especially dangerous in AI-generated code because generated diffs often look plausible while changing operational behavior.

### Temporal Coupling Must Be Explicit

If operations must happen in a specific order, the design should make the order hard to violate: state machines, typed workflow stages, builders that prevent invalid construction, transaction boundaries, or explicit orchestration objects.

### Comments Explain Why, Not What

Comments should explain constraints, decisions, tradeoffs, invariants, workarounds, and domain rules. They should not narrate obvious implementation.

Bad:

```text
// Increment i by one
```

Good:

```text
// Keep this threshold below the provider timeout to preserve retry budget.
```

## Anti-Patterns

- Names that require reading implementation to understand purpose
- Functions with mixed validation, persistence, and notification logic
- Magic numbers and strings repeated across domain logic
- Errors swallowed without context or leaked across layer boundaries
- Hidden side effects triggered by getters or seemingly pure functions
- Raw `String`, `int`, or `float` for domain concepts like money, IDs, or durations
- Temporal dependencies enforced only by comments or documentation
- Defensive validation deep inside domain core for states rejected at boundaries

## Guardian Hooks

Guardians that apply to this guidance:
- `clean_code`: intent-revealing-names, no-mixed-responsibilities, no-hidden-side-effects, no-primitive-obsession
- `architecture_boundary`: dependency-direction (when responsibility leaks cross boundaries)
- `testability`: untestable-design (when hidden coupling prevents isolation)
