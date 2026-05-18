# Rust Zero Panic Guardian

Enforce explicit error propagation in Rust code outside `main.rs`. Panic-prone control flow is forbidden in production code, `#[cfg(test)]` modules, and files under `tests/`.

## Rules

### no-unwrap-outside-main
`unwrap()` in domain or library code creates invisible crash points. Use `?` operator, `map_err`, or explicit error handling instead.

Triggers: `.unwrap()` outside `main.rs`, `.unwrap()` on user-facing data paths, `.unwrap()` in test fixtures or contract setup code.

### no-expect-outside-main
`expect()` is a panic with a message. In library and domain code, return errors to callers rather than crashing the process.

Triggers: `.expect()` outside `main.rs`, `.expect()` in code paths reachable from public API, `.expect()` in test setup or runtime invocation.

### no-panic-outside-main
`panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()` outside `main.rs` crash the process or obscure failures. Use typed errors and explicit handling.

Triggers: `panic!` macro outside `main.rs`, `todo!()` merged to main branch, `unreachable!()` in paths that could be reached through malformed input, panic-prone test setup.

## Disposition

Default: `warning` (likely needs correction before merge).

## Scope

Applies to Rust code only. Language-specific guardian. Enforced by clippy lints and code review.
