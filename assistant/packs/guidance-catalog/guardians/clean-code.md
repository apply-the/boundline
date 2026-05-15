# Clean Code Guardian

Review intent-revealing names, hidden side effects, mixed responsibilities, and primitive obsession before approving a change.

## Rules

### intent-revealing-names
Names of types, functions, variables, and modules must communicate purpose without requiring comments. A reader should understand what a symbol does from its name alone.

Triggers: generic names (`data`, `info`, `handler`, `manager`, `utils`), abbreviated names without established convention, names that describe implementation rather than intent.

### no-mixed-responsibilities
Each module, type, or function should have one reason to change. Mixing persistence, validation, orchestration, and presentation in a single unit creates fragile code.

Triggers: functions that perform I/O and compute, types that serialize and validate and apply policy, modules that mix transport and domain concerns.

### no-hidden-side-effects
Functions should be honest about their effects. If a function mutates state, performs I/O, or triggers events, its signature and name should make that visible.

Triggers: getters that write, constructors with network calls, pure-looking functions with logging side effects that affect behavior.

### no-primitive-obsession
Domain concepts should be expressed as dedicated types, not raw primitives. An `OrderId` is not a `String`. A `Temperature` is not an `f64`.

Triggers: function parameters that are all `String` or `i32`, domain identifiers passed as raw primitives, quantities without units.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all languages. Cross-cutting; not language-specific.
