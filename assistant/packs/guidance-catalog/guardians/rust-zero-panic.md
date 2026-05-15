# Rust Zero Panic Guardian

Enforce explicit error propagation in domain and library code. Panic-prone control flow is forbidden outside `main.rs`, `#[cfg(test)]`, and `tests/`.

## Rules

### no-unwrap-in-domain-code
`unwrap()` in domain or library code creates invisible crash points. Use `?` operator, `map_err`, or explicit error handling instead.

Triggers: `.unwrap()` outside test code, `.unwrap()` on user-facing data paths, `.unwrap()` in library crate code.

### no-expect-in-library-code
`expect()` is a panic with a message. In library and domain code, return errors to callers rather than crashing the process.

Triggers: `.expect()` outside `main.rs` or test modules, `.expect()` in code paths reachable from public API.

### no-panic-in-production-path
`panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()` in production paths crash the process. Use typed errors and explicit handling.

Triggers: `panic!` macro in non-test code, `todo!()` merged to main branch, `unreachable!()` in paths that could be reached through malformed input.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to Rust code only. Language-specific guardian. Enforced by clippy lints and code review.
