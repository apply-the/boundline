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

### no-dead-code
Remove all unused exports, unreachable branches, and commented-out blocks. Trust version control to remember history.

Triggers: large blocks of commented-out code, variables assigned but never read, private functions that are never called.

### no-magic-values
Avoid unexplained magic strings or numbers in domain logic. Use named constants or typed enums instead.

Triggers: inline numbers (e.g., `if count > 4`), repeated string literals without explanation, hardcoded status codes.

### small-functions
Functions must be kept small (e.g., < 50 lines). If a function becomes long or requires comments to explain its internal steps, extract those steps into well-named helper functions.

Triggers: functions exceeding 50 lines, functions with multiple logical blocks separated by narrative comments.

### comments-explain-why
Comments must explain the *why*, business constraints, and invariants, rather than narrating the obvious *what* that the code already shows.

Triggers: comments that simply restate the code (e.g., `// increment i by 1`), missing explanations for complex workarounds.

## Disposition

Default: `concern` (raise for discussion, do not block).

## Scope

Applies to all languages. Cross-cutting; not language-specific.
