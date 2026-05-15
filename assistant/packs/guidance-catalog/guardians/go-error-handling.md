# Go Error Handling Guardian

Enforce explicit error checking and prevent panics in library and service code.

## Rules

### unchecked-error-return
Every function that returns an error must have its error value checked by the caller. Assigning to `_` or ignoring the second return value silently discards failure information.

Triggers: `result, _ := SomeFunction()`, calling functions without capturing the error return, error values captured but never checked.

### error-string-comparison
Comparing errors by string content (`err.Error() == "not found"`) is fragile and breaks on message changes. Use sentinel errors, `errors.Is`, or `errors.As` for stable matching.

Triggers: `if err.Error() == ...`, `strings.Contains(err.Error(), ...)`, switch statements on error message text.

### panic-in-library
`panic()` in library or service code crashes the entire process. Use error returns for recoverable situations. Reserve `panic` for truly unrecoverable programmer errors that indicate a bug.

Triggers: `panic()` in exported functions, `panic` in request handlers, `log.Fatal` in library code (which calls `os.Exit`).

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to Go code only. Language-specific guardian. Complements `golangci-lint` error-checking linters.
